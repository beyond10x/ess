//! Source-derived view rows; projection lag is owned by the target, not by assertions.
use super::execute::Store;
use crate::target::{SemanticViewRequest, SemanticViewResult, TargetError, ViewRow};
use ess_compiler::{
    ir::{ResolvedBody, ResolvedTypeRef, ResolvedView},
    EssIr,
};
use ess_domain::{types::Primitive, view::Direction};
use ess_primitives::{node::Node, predicate::Truth, time::Rfc3339Instant};
use std::{cmp::Ordering, collections::BTreeMap};
fn unsupported() -> TargetError {
    TargetError::unsupported(
        "interpreted view",
        "the model does not determine a complete typed view observation",
    )
}

/// `now` is the instant of the read, from the target's clock (ess/30, beyond10x/ess#560): a filter
/// ordering a field against `now` reads it, and is Unknown, so unsupported, without one.
pub(super) fn query(
    ir: &EssIr,
    store: &Store,
    request: &SemanticViewRequest,
    now: Option<Rfc3339Instant>,
) -> Result<SemanticViewResult, TargetError> {
    let view = ir
        .views()
        .get(request.view.name())
        .ok_or_else(unsupported)?;
    for param in &view.params {
        if view
            .paging
            .as_ref()
            .is_some_and(|paging| paging.reads(&param.name))
            && !request.params.contains_key(&param.name)
        {
            continue;
        }
        let Some(value) = request.params.get(&param.name) else {
            if param.type_ref.is_optional() {
                continue;
            }
            return Err(unsupported());
        };
        crate::input::validate_typed_value(ir, &param.type_ref, value)
            .map_err(|_| unsupported())?;
    }
    let parameters = crate::input::TypedFacts::new(
        ir,
        &view.params,
        crate::input::bind(
            ir,
            &view.params,
            &request.params,
            crate::input::Completeness::Partial,
        )
        .map_err(|_| unsupported())?,
    );
    let mut rows = project(ir, view, select(ir, view, store, &parameters, now)?)?;
    // `absent:` (ess/25, beyond10x/ess#533): the filter is the identity comparison alone, so no
    // row selected is no stored row carrying the requested identity, and the read answers the
    // declared default row.
    if rows.is_empty() {
        if let Some(absent) = &view.absent {
            rows.push(default_row(ir, view, absent, &request.params)?);
        }
    }
    rows.sort_by(|left, right| rank(ir, view, left, right));
    let total = rows.len();
    if let Some(paging) = &view.paging {
        match (
            request.params.get(&paging.page),
            request.params.get(&paging.size),
        ) {
            (None, None) => {}
            (Some(page), Some(size)) => {
                let integer = |value: &Node| -> Result<usize, TargetError> {
                    let Node::Number(value) = value else {
                        return Err(unsupported());
                    };
                    value
                        .as_i64()
                        .and_then(|value| usize::try_from(value).ok())
                        .ok_or_else(unsupported)
                };
                let page = integer(page)?;
                let size = integer(size)?;
                let offset = page
                    .checked_sub(usize::try_from(paging.first_page).map_err(|_| unsupported())?)
                    .and_then(|page| page.checked_mul(size))
                    .ok_or_else(unsupported)?;
                rows = rows.into_iter().skip(offset).take(size).collect();
            }
            _ => return Err(unsupported()),
        }
    }
    let mut result = SemanticViewResult::of(rows);
    if view.paging.as_ref().is_some_and(|paging| paging.total) {
        result.total = Some(u64::try_from(total).map_err(|_| unsupported())?);
    }
    Ok(result)
}
/// The row a view declaring `absent:` answers for an identity no row carries: the requested value
/// in the identity field, each literal of the default row typed by its field, and nothing in every
/// other field, which `ess-domain` holds to be `Optional`.
fn default_row(
    ir: &EssIr,
    view: &ResolvedView,
    absent: &ess_compiler::ir::ResolvedAbsent,
    params: &BTreeMap<String, Node>,
) -> Result<ViewRow, TargetError> {
    let identity = &ir.entity(&view.source).identity.name;
    let requested = params.get(&absent.param).ok_or_else(unsupported)?;
    let mut row = BTreeMap::new();
    for field in &view.fields {
        let value = if &field.name == identity {
            requested.clone()
        } else if let Some(text) = absent.row.get(&field.name) {
            super::execute::literal(ir, &field.type_ref, text).map_err(|_| unsupported())?
        } else if field.type_ref.is_optional() {
            Node::Null
        } else {
            return Err(unsupported());
        };
        crate::input::validate_typed_value(ir, &field.type_ref, &value)
            .map_err(|_| unsupported())?;
        row.insert(field.name.clone(), value);
    }
    Ok(row)
}

/// Every source row the filter admits, each with the measures whose condition (`where:`,
/// beyond10x/ess#363) it satisfies. Each measure reads its own subset of the row's group; an
/// unknown filter or condition makes the whole observation undetermined — never a partial row,
/// never a false.
fn select(
    ir: &EssIr,
    view: &ResolvedView,
    store: &Store,
    parameters: &crate::input::TypedFacts<'_>,
    now: Option<Rfc3339Instant>,
) -> Result<Vec<(ViewRow, std::collections::BTreeSet<String>)>, TargetError> {
    let entity = ir.entity(&view.source);
    let declared = entity.observable_fields();
    let conditions: Vec<(&String, &ess_primitives::predicate::Predicate)> = view
        .aggregation
        .iter()
        .flat_map(|aggregation| &aggregation.functions)
        .filter_map(|(name, aggregate)| {
            aggregate
                .r#where
                .as_ref()
                .map(|condition| (name, condition))
        })
        .collect();
    // The observable fields of each row a reference names (ess/28), typing what a filter reads
    // through it.
    let far_fields: Vec<(String, Vec<ess_compiler::ir::ResolvedField>)> = view
        .references
        .iter()
        .map(|reference| {
            (
                reference.via.clone(),
                ir.entity(&reference.entity).observable_fields(),
            )
        })
        .collect();
    let mut selected = Vec::new();
    for (identity, instance) in store.instances_of(&entity.name) {
        let mut fields = instance.fields.clone();
        fields.insert(entity.identity.name.clone(), identity.clone());
        fields.insert("state".into(), Node::Text(instance.state.to_string()));
        // The rows the source's references name, as they are now (ess/28, beyond10x/ess#564):
        // none where the reference is absent or names no row.
        let far = referenced_rows(ir, view, store, &fields);
        let mut navigated = BTreeMap::new();
        for (via, declared) in &far_fields {
            if let Some(Some(row)) = far.get(via) {
                navigated.insert(
                    via.clone(),
                    crate::input::TypedFacts::new(
                        ir,
                        declared,
                        crate::input::bind(ir, declared, row, crate::input::Completeness::Partial)
                            .map_err(|_| unsupported())?,
                    ),
                );
            }
        }
        let mut admitted_by = std::collections::BTreeSet::new();
        if view.filter.is_some() || !conditions.is_empty() {
            let row = crate::input::TypedFacts::new(
                ir,
                &declared,
                crate::input::bind(ir, &declared, &fields, crate::input::Completeness::Partial)
                    .map_err(|_| unsupported())?,
            );
            let facts = ViewFacts {
                ir,
                view,
                row: &row,
                parameters,
                navigated: &navigated,
                now,
            };
            if let Some(filter) = &view.filter {
                match filter.evaluate(&facts) {
                    Truth::True => {}
                    Truth::False => continue,
                    Truth::Unknown if absent_text_operands(view, filter, &facts) => continue,
                    // A comparison over the row a reference does not name is unknown, and a row
                    // whose filter is unknown is not shown, as every target reads one.
                    Truth::Unknown if absent_navigation(&facts, filter) => continue,
                    Truth::Unknown => return Err(unsupported()),
                }
            }
            for (measure, condition) in &conditions {
                match condition.evaluate(&facts) {
                    Truth::True => {
                        admitted_by.insert((*measure).clone());
                    }
                    Truth::False => {}
                    Truth::Unknown => return Err(unsupported()),
                }
            }
        }
        // Each view field read through a reference: the referenced row's value, or nothing.
        for reference in &view.references {
            let held = far.get(&reference.via).and_then(Option::as_ref);
            for read in &reference.fields {
                let value = held
                    .and_then(|row| row.get(&read.read))
                    .cloned()
                    .unwrap_or(Node::Null);
                fields.insert(read.field.clone(), value);
            }
        }
        selected.push((fields, admitted_by));
    }
    Ok(selected)
}

/// The row each reference `view` reads through names in `store`, from the source row `fields`: its
/// stored fields, its identity and `state`, or `None` where the reference is absent or names no
/// row (ess/28, beyond10x/ess#564).
fn referenced_rows(
    ir: &EssIr,
    view: &ResolvedView,
    store: &Store,
    fields: &ViewRow,
) -> BTreeMap<String, Option<ViewRow>> {
    view.references
        .iter()
        .map(|reference| {
            let far = ir.entity(&reference.entity);
            let row = fields
                .get(&reference.via)
                .filter(|address| **address != Node::Null)
                .and_then(|address| {
                    let held = store.instance_typed(&far.name, address)?;
                    let mut row = held.fields.clone();
                    row.insert(far.identity.name.clone(), address.clone());
                    row.insert("state".into(), Node::Text(held.state.to_string()));
                    Some(row)
                });
            (reference.via.clone(), row)
        })
        .collect()
}

/// Whether a filter is unknown only because a path it reads through a reference reads no row: the
/// reference is absent or names none (ess/28, beyond10x/ess#564).
fn absent_navigation(facts: &ViewFacts<'_>, filter: &ess_primitives::predicate::Predicate) -> bool {
    use ess_primitives::facts::FactSource;
    let mut unobserved = filter
        .fact_paths()
        .into_iter()
        .filter(|path| facts.observe(path).is_none())
        .peekable();
    unobserved.peek().is_some()
        && unobserved.all(|path| {
            facts
                .view
                .navigated(facts.ir, path)
                .is_some_and(|(reference, _)| !facts.navigated.contains_key(&reference.via))
        })
}

fn kind(ir: &EssIr, ty: &ResolvedTypeRef) -> crate::aggregate::ValueKind {
    use crate::aggregate::ValueKind;
    let mut ty = ty.required();
    while let ResolvedTypeRef::Declared { name } = ty {
        match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => ty = of.required(),
            _ => break,
        }
    }
    match ty {
        ResolvedTypeRef::Primitive {
            name: Primitive::Integer | Primitive::Decimal,
        } => ValueKind::Numeric,
        ResolvedTypeRef::Primitive {
            name: Primitive::String,
        } => ValueKind::Text,
        ResolvedTypeRef::Primitive {
            name: Primitive::Timestamp,
        } => ValueKind::Timestamp,
        _ => ValueKind::Other,
    }
}
fn rank(ir: &EssIr, view: &ResolvedView, left: &ViewRow, right: &ViewRow) -> Ordering {
    for key in &view.order_by {
        let field = view
            .fields
            .iter()
            .find(|field| field.name == key.field)
            .expect("resolved ranking field");
        let compare = compare(
            ir,
            &field.type_ref,
            left.get(&key.field),
            right.get(&key.field),
        );
        if !compare.is_eq() {
            return if key.direction == Direction::Descending {
                compare.reverse()
            } else {
                compare
            };
        }
    }
    Ordering::Equal
}

/// How two values of `ty` rank, ascending: a `Timestamp` by its instant, a struct by its members
/// in declaration order, each by its own type's order, and anything else by its node. A view's
/// order and the head of an ordered selection (ess/27, beyond10x/ess#543) both rank with this, so
/// the two cannot disagree; only the head ranks a struct, its identity's
/// (`docs/design/ordered-related-head.md`, "Struct identities").
pub(crate) fn compare(
    ir: &EssIr,
    ty: &ResolvedTypeRef,
    left: Option<&Node>,
    right: Option<&Node>,
) -> Ordering {
    let mut ty = ty.required();
    while let ResolvedTypeRef::Declared { name } = ty {
        match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => ty = of.required(),
            ResolvedBody::Struct { fields, .. } => {
                if let (Some(Node::Map(left)), Some(Node::Map(right))) = (left, right) {
                    return fields
                        .iter()
                        .map(|member| {
                            compare(
                                ir,
                                &member.type_ref,
                                left.get(&member.name),
                                right.get(&member.name),
                            )
                        })
                        .find(|ordering| ordering.is_ne())
                        .unwrap_or(Ordering::Equal);
                }
                break;
            }
            _ => break,
        }
    }
    if matches!(
        ty,
        ResolvedTypeRef::Primitive {
            name: Primitive::Timestamp
        }
    ) {
        if let (Some(Node::Text(left)), Some(Node::Text(right))) = (left, right) {
            if let (Some(left), Some(right)) = (
                Rfc3339Instant::parse_rfc3339(left),
                Rfc3339Instant::parse_rfc3339(right),
            ) {
                return left.cmp(&right);
            }
        }
    }
    left.cmp(&right)
}

/// The rows `selected` project to: each a source row the filter admits, with the measures whose
/// condition it satisfies.
fn project(
    ir: &EssIr,
    view: &ResolvedView,
    selected: Vec<(ViewRow, std::collections::BTreeSet<String>)>,
) -> Result<Vec<ViewRow>, TargetError> {
    let mut rows = Vec::new();
    if let Some(aggregation) = &view.aggregation {
        let mut groups: BTreeMap<Vec<Node>, Vec<(ViewRow, std::collections::BTreeSet<String>)>> =
            BTreeMap::new();
        if aggregation.group_by.is_empty() {
            groups.insert(Vec::new(), Vec::new());
        }
        for (fields, admitted_by) in selected {
            let mut key = Vec::new();
            for name in &aggregation.group_by {
                let field = view
                    .fields
                    .iter()
                    .find(|field| &field.name == name)
                    .ok_or_else(unsupported)?;
                let value = match fields.get(name) {
                    Some(value) => value.clone(),
                    None if field.type_ref.is_optional() => Node::Null,
                    None => return Err(unsupported()),
                };
                crate::input::validate_typed_value(ir, &field.type_ref, &value)
                    .map_err(|_| unsupported())?;
                key.push(value);
            }
            groups.entry(key).or_default().push((fields, admitted_by));
        }
        for (key, members) in groups {
            let mut row: BTreeMap<_, _> = aggregation.group_by.iter().cloned().zip(key).collect();
            for (name, aggregate) in &aggregation.functions {
                let mut values = Vec::new();
                for (fields, admitted_by) in &members {
                    if aggregate.r#where.is_some() && !admitted_by.contains(name) {
                        continue;
                    }
                    let value = match &aggregate.input {
                        None => Node::Null,
                        Some(field) => match fields.get(&field.name) {
                            Some(value) => value.clone(),
                            None if field.type_ref.is_optional() => Node::Null,
                            None => return Err(unsupported()),
                        },
                    };
                    values.push(value);
                }
                let kind = aggregate
                    .input
                    .as_ref()
                    .map_or(crate::aggregate::ValueKind::Other, |field| {
                        kind(ir, &field.type_ref)
                    });
                let value = if aggregate.skip_absent {
                    crate::aggregate::evaluate_skipping_absent(aggregate.function, &values, kind)
                } else {
                    crate::aggregate::evaluate(aggregate.function, &values, kind)
                }
                .ok_or_else(unsupported)?;
                row.insert(name.clone(), value);
            }
            rows.push(row);
        }
    } else {
        for (fields, _) in selected {
            let mut row = BTreeMap::new();
            for field in &view.fields {
                // ess/27 (beyond10x/ess#560): computed from the row by the shared evaluator.
                if let Some(expr) = view.computed.get(&field.name) {
                    let value = computed(expr, &fields).ok_or_else(unsupported)?;
                    if value != Node::Null {
                        crate::input::validate_typed_value(ir, &field.type_ref, &value)
                            .map_err(|_| unsupported())?;
                    } else if !field.type_ref.is_optional() {
                        return Err(unsupported());
                    }
                    row.insert(field.name.clone(), value);
                    continue;
                }
                let Some(value) = fields.get(&field.name) else {
                    if field.type_ref.is_optional() {
                        row.insert(field.name.clone(), Node::Null);
                        continue;
                    }
                    return Err(unsupported());
                };
                crate::input::validate_typed_value(ir, &field.type_ref, value)
                    .map_err(|_| unsupported())?;
                row.insert(field.name.clone(), value.clone());
            }
            rows.push(row);
        }
    }

    Ok(rows)
}

/// A computed field's value over one source row (`expr:`, ess/27, beyond10x/ess#560): `null` where
/// it is absent, `None` where it is unknown. A field the row does not hold is absent.
pub(crate) fn computed(
    expr: &ess_primitives::expr::Expr,
    fields: &BTreeMap<String, Node>,
) -> Option<Node> {
    use ess_primitives::expr::Reading;
    let reads = |path: &ess_primitives::facts::FactPath| {
        fields
            .get(&path.to_string())
            .map_or(Reading::Absent, Reading::of_node)
    };
    expr.evaluate(&reads).to_node()
}

/// Whether a filter is unknown only because the read left out an optional parameter a string
/// operator compares with (beyond10x/ess#200): an absence the caller chose, not an observation the
/// model failed to determine, so the row is not shown — as the suite and a generated query read an
/// unknown filter — rather than the read refused.
fn absent_text_operands(
    view: &ResolvedView,
    filter: &ess_primitives::predicate::Predicate,
    facts: &ViewFacts<'_>,
) -> bool {
    use ess_primitives::facts::FactSource;
    let mut unobserved = filter
        .fact_paths()
        .into_iter()
        .filter(|path| facts.observe(path).is_none())
        .peekable();
    unobserved.peek().is_some()
        && unobserved.all(|path| {
            let [namespace, name] = path.segments() else {
                return false;
            };
            namespace == ess_domain::view::ViewSpec::PARAM
                && filter.reads_text_parameter(name)
                && view
                    .params
                    .iter()
                    .any(|param| &param.name == name && param.type_ref.is_optional())
        })
}

/// Keep parameter and entity namespaces distinct without discarding their declared scalar kinds.
struct ViewFacts<'a> {
    ir: &'a EssIr,
    view: &'a ResolvedView,
    row: &'a crate::input::TypedFacts<'a>,
    parameters: &'a crate::input::TypedFacts<'a>,
    /// The rows the view's references name, by reference (ess/28, beyond10x/ess#564); a reference
    /// that names none is not here.
    navigated: &'a BTreeMap<String, crate::input::TypedFacts<'a>>,
    /// The instant of the read, which a filter's `now` names (ess/30, beyond10x/ess#560).
    now: Option<Rfc3339Instant>,
}
impl ViewFacts<'_> {
    /// Where `path` is read: the parameters, the row a reference names (`None` where it names
    /// none), or the source row.
    fn source<'a>(
        &'a self,
        path: &ess_primitives::facts::FactPath,
    ) -> Option<(
        &'a crate::input::TypedFacts<'a>,
        ess_primitives::facts::FactPath,
    )> {
        if let Some(parameter) = path
            .to_string()
            .strip_prefix("param.")
            .and_then(|name| ess_primitives::facts::FactPath::new(name).ok())
        {
            Some((self.parameters, parameter))
        } else if let Some((reference, _)) = self.view.navigated(self.ir, path) {
            let far = self.navigated.get(&reference.via)?;
            let rest = ess_primitives::facts::FactPath::from_segments(&path.segments()[1..]);
            Some((far, rest))
        } else {
            Some((self.row, path.clone()))
        }
    }
}
impl ess_primitives::facts::FactSource for ViewFacts<'_> {
    fn fact(
        &self,
        path: &ess_primitives::facts::FactPath,
    ) -> Option<ess_primitives::facts::FactValue> {
        let (source, path) = self.source(path)?;
        source.fact(&path)
    }
    fn present(&self, path: &ess_primitives::facts::FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.present(&path))
    }
    fn scales(&self) -> &ess_primitives::facts::Scales {
        self.row.scales()
    }
    fn orders_as_instant(&self, path: &ess_primitives::facts::FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.orders_as_instant(&path))
    }
    fn orders_text_by_bytes(&self, path: &ess_primitives::facts::FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.orders_text_by_bytes(&path))
    }
    fn now(&self) -> Option<Rfc3339Instant> {
        self.now
    }
}
