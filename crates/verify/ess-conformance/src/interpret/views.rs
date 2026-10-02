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

pub(super) fn query(
    ir: &EssIr,
    store: &Store,
    request: &SemanticViewRequest,
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
    let entity = ir.entity(&view.source);
    let declared = entity.observable_fields();
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
    let mut selected = Vec::new();
    for (name, identity, instance) in store.instances() {
        if name != &entity.name {
            continue;
        }
        let mut fields = instance.fields.clone();
        fields.insert(entity.identity.name.clone(), Node::Text(identity.into()));
        fields.insert("state".into(), Node::Text(instance.state.to_string()));
        if let Some(filter) = &view.filter {
            let row = crate::input::TypedFacts::new(
                ir,
                &declared,
                crate::input::bind(ir, &declared, &fields, crate::input::Completeness::Partial)
                    .map_err(|_| unsupported())?,
            );
            match filter.evaluate(&ViewFacts {
                row: &row,
                parameters: &parameters,
            }) {
                Truth::True => {}
                Truth::False => continue,
                Truth::Unknown => return Err(unsupported()),
            }
        }
        selected.push(fields);
    }
    let mut rows = project(ir, view, selected)?;
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
fn compare(
    ir: &EssIr,
    ty: &ResolvedTypeRef,
    left: Option<&Node>,
    right: Option<&Node>,
) -> Ordering {
    let mut ty = ty.required();
    while let ResolvedTypeRef::Declared { name } = ty {
        match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => ty = of.required(),
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

fn project(
    ir: &EssIr,
    view: &ResolvedView,
    selected: Vec<ViewRow>,
) -> Result<Vec<ViewRow>, TargetError> {
    let mut rows = Vec::new();
    if let Some(aggregation) = &view.aggregation {
        let mut groups: BTreeMap<Vec<Node>, Vec<ViewRow>> = BTreeMap::new();
        if aggregation.group_by.is_empty() {
            groups.insert(Vec::new(), Vec::new());
        }
        for fields in selected {
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
            groups.entry(key).or_default().push(fields);
        }
        for (key, members) in groups {
            let mut row: BTreeMap<_, _> = aggregation.group_by.iter().cloned().zip(key).collect();
            for (name, aggregate) in &aggregation.functions {
                let mut values = Vec::new();
                for fields in &members {
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
        for fields in selected {
            let mut row = BTreeMap::new();
            for field in &view.fields {
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

/// Keep parameter and entity namespaces distinct without discarding their declared scalar kinds.
struct ViewFacts<'a> {
    row: &'a crate::input::TypedFacts<'a>,
    parameters: &'a crate::input::TypedFacts<'a>,
}
impl ViewFacts<'_> {
    fn source<'a>(
        &'a self,
        path: &ess_primitives::facts::FactPath,
    ) -> (
        &'a crate::input::TypedFacts<'a>,
        ess_primitives::facts::FactPath,
    ) {
        if let Some(parameter) = path
            .to_string()
            .strip_prefix("param.")
            .and_then(|name| ess_primitives::facts::FactPath::new(name).ok())
        {
            (self.parameters, parameter)
        } else {
            (self.row, path.clone())
        }
    }
}
impl ess_primitives::facts::FactSource for ViewFacts<'_> {
    fn fact(
        &self,
        path: &ess_primitives::facts::FactPath,
    ) -> Option<ess_primitives::facts::FactValue> {
        let (source, path) = self.source(path);
        source.fact(&path)
    }
    fn present(&self, path: &ess_primitives::facts::FactPath) -> bool {
        let (source, path) = self.source(path);
        source.present(&path)
    }
    fn scales(&self) -> &ess_primitives::facts::Scales {
        self.row.scales()
    }
    fn orders_as_instant(&self, path: &ess_primitives::facts::FactPath) -> bool {
        let (source, path) = self.source(path);
        source.orders_as_instant(&path)
    }
    fn orders_text_by_bytes(&self, path: &ess_primitives::facts::FactPath) -> bool {
        let (source, path) = self.source(path);
        source.orders_text_by_bytes(&path)
    }
}
