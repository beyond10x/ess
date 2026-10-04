//! Which view queries the specification fully determines — language-neutral.
//!
//! A view's query is generated when every row it answers is something an emitter can compute from
//! the stored rows of its source entity, with the semantics the conformance suite holds a target
//! to: a projection copies the entity's own fields and `state`; a `filter:` shows a row where it
//! holds, and hides it where it is false or unknown (the suite's `shows_row`, which counts an
//! undecided filter as no row); an `order_by:` ranks rows as the runner's `ranked` compares them;
//! and an aggregation partitions the admitted rows by their group keys and computes each function
//! as `ess_conformance::aggregate::evaluate` does (`docs/design/aggregate-views.md`, "Semantics"
//! and "Absent values"). Otherwise the query stays an obligation, and [`view`] names the construct
//! that kept it one. The planner asks this module and nothing else, so the plan and every emitter
//! agree on the line.

use ess_compiler::ir::{
    EssIr, ResolvedAggregate, ResolvedBody, ResolvedEntity, ResolvedField, ResolvedTypeRef,
    ResolvedView,
};
use ess_domain::types::Primitive;
use ess_domain::view::AggregateFunction;
use ess_primitives::facts::FactPath;

use crate::determined::{self, Env, Kind, Resolved};

/// `Ok` when every row of `view` is determined by its source's stored rows; `Err` names the
/// construct that keeps its query an obligation, in a phrase that reads after "kept an obligation
/// by".
pub(crate) fn view(ir: &EssIr, view: &ResolvedView) -> Result<(), String> {
    if view.paging.is_some() {
        return Err("a paged view (`paging:`)".to_owned());
    }
    if !view.params.is_empty() {
        return Err(
            "a view parameter (`params:`), which a generated query does not apply".to_owned(),
        );
    }
    let entity = ir.entity(&view.source);
    if let Some(filter) = &view.filter {
        determined::supported(ir, &Env::Row(entity), filter)
            .map_err(|construct| format!("{construct}, in `filter:`"))?;
    }
    view.aggregation.as_ref().map_or_else(
        || projected(ir, view, entity),
        |aggregation| aggregated(ir, view, entity, aggregation),
    )
}

/// `Ok` where every group key is compared and every aggregate computed as the suite reads them.
fn aggregated(
    ir: &EssIr,
    view: &ResolvedView,
    entity: &ResolvedEntity,
    aggregation: &ess_compiler::ir::ResolvedAggregation,
) -> Result<(), String> {
    for key in &aggregation.group_by {
        let field = view_field(view, key)?;
        let source = source_field(entity, key)?;
        if !determined::assignable(&source, &field.type_ref) {
            return Err(format!(
                "the group key `{key}`, whose type is not the source field's"
            ));
        }
        match resolve(ir, entity, key)?.kind {
            Kind::Number(_) | Kind::Text | Kind::Enum(..) | Kind::State | Kind::Bool => {}
            Kind::Opaque => {
                return Err(format!(
                    "the group key `{key}`, a value the generated query does not compare"
                ))
            }
        }
    }
    for (name, aggregate) in &aggregation.functions {
        self::aggregate(ir, entity, view_field(view, name)?, aggregate)
            .map_err(|construct| format!("{construct}, in the field `{name}`"))?;
    }
    Ok(())
}

/// `Ok` where every field is the entity's own and every order key ranks as the runner compares.
fn projected(ir: &EssIr, view: &ResolvedView, entity: &ResolvedEntity) -> Result<(), String> {
    for field in &view.fields {
        let source = source_field(entity, &field.name)?;
        if !determined::assignable(&source, &field.type_ref) {
            return Err(format!(
                "the field `{}`, which `{}` does not hold at that type",
                field.name, entity.name
            ));
        }
    }
    for ranking in &view.order_by {
        let field = view_field(view, &ranking.field)?;
        if field.type_ref.is_optional() {
            return Err(format!(
                "an order over the optional field `{}`, which has no order against a \
                     present value",
                ranking.field
            ));
        }
        order(ir, entity, &ranking.field)?;
    }
    Ok(())
}

/// How one view parameter's query-string value becomes the JSON value its decoder reads
/// (story:served-view-params).
///
/// A query value is text. The servers turn it into the JSON scalar the parameter's declared type
/// is written as on the wire, then decode it with the same generated decoder a command input uses,
/// so an enum variant, a newtype and an absent `Optional` are refused or accepted exactly as they
/// are in a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueryScalar {
    /// Written as a JSON string: `String`, `Decimal`, `Timestamp`, `Duration`, `Uuid`, `Bytes`
    /// (base64), and an enum's wire spelling.
    Text,
    /// Written as a JSON number: `Integer`. A value that is not a whole number's spelling stays
    /// text, so the decoder names what arrived instead of a number that never did.
    Integer,
    /// Written as a JSON boolean: `true` or `false`, and anything else stays text.
    Boolean,
}

/// The query-string scalar a parameter of `type_ref` is carried as, through `Optional` and
/// newtypes; `None` for a list, a map, a struct, a union and `Json`, which no single query value
/// carries (the same line `ess_ui_check::binding` draws).
pub(crate) fn query_scalar(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<QueryScalar> {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => match name {
            Primitive::Integer => Some(QueryScalar::Integer),
            Primitive::Boolean => Some(QueryScalar::Boolean),
            Primitive::Json | Primitive::Binary64 => None,
            _ => Some(QueryScalar::Text),
        },
        ResolvedTypeRef::Optional { of } => query_scalar(ir, of),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => query_scalar(ir, of),
            ResolvedBody::Enum { .. } => Some(QueryScalar::Text),
            ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => None,
        },
        ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => None,
    }
}

/// Every parameter of a view some `reached_by: network` component serves whose type no query
/// value carries, named at the key its author wrote — refused by the targets that serve views, as
/// `paging:` is, rather than served with a parameter no request can supply.
pub(crate) fn refuse_unqueryable(
    ir: &EssIr,
    plan: &crate::SynthesisPlan,
    target: crate::Target,
) -> Result<(), crate::failure::TargetFailure> {
    use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
    let mut causes = Vec::new();
    for component in ir.components().values() {
        if component.reached_by != ess_domain::component::Reach::Network {
            continue;
        }
        for route in ess_gen::http::routes(ir, component) {
            let ess_gen::http::Served::View(handle) = route.serves else {
                continue;
            };
            let view = ir.view(handle);
            for param in &view.params {
                if query_scalar(ir, &param.type_ref).is_none() {
                    causes.push(TargetFailureCause::new(
                        TargetFailureCode::MissingRepresentation,
                        vec![format!("views.{}.params.{}", view.name, param.name)],
                        "a served view's parameter is read from the query string, and a query \
                         value carries only a scalar"
                            .to_owned(),
                    ));
                }
            }
        }
    }
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}

/// Every pair of one view's parameters whose names a target spells as one identifier — `minHours`
/// and `min_hours` — named at both keys, refused by that target: its query methods take the
/// parameters as arguments, and two arguments of one name are a program that does not build.
/// `ident` is the target's own spelling of a parameter name, before any escape it applies.
pub(crate) fn refuse_colliding_params(
    ir: &EssIr,
    plan: &crate::SynthesisPlan,
    target: crate::Target,
    ident: impl Fn(&str) -> String,
) -> Result<(), crate::failure::TargetFailure> {
    use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
    let mut causes = Vec::new();
    for view in ir.views().values() {
        let mut seen: std::collections::BTreeMap<String, &str> = std::collections::BTreeMap::new();
        for param in &view.params {
            let spelled = ident(&param.name);
            if let Some(earlier) = seen.get(&spelled) {
                causes.push(TargetFailureCause::new(
                    TargetFailureCode::SymbolCollision,
                    vec![
                        format!("views.{}.params.{earlier}", view.name),
                        format!("views.{}.params.{}", view.name, param.name),
                    ],
                    format!(
                        "two parameters of one view are both spelled `{spelled}` in this target, \
                         and a query method cannot take two arguments of one name"
                    ),
                ));
            } else {
                seen.insert(spelled, &param.name);
            }
        }
    }
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}

/// `true` where the plan marks this view's query generated.
pub(crate) fn generated(ir: &EssIr, view: &ResolvedView) -> bool {
    self::view(ir, view).is_ok()
}

/// `true` where any view of the model has a generated query.
pub(crate) fn any_generated(ir: &EssIr) -> bool {
    ir.views().values().any(|view| generated(ir, view))
}

/// The view's own declaration of one field.
fn view_field<'v>(view: &'v ResolvedView, name: &str) -> Result<&'v ResolvedField, String> {
    view.fields
        .iter()
        .find(|field| field.name == name)
        .ok_or_else(|| format!("the field `{name}`, which the view does not declare"))
}

/// The type of the source's observable field `name`: the identity, a stored field, or `state`.
fn source_field(entity: &ResolvedEntity, name: &str) -> Result<ResolvedTypeRef, String> {
    if name == "state" {
        return Ok(ResolvedTypeRef::Declared {
            name: entity.state_type.clone(),
        });
    }
    std::iter::once(&entity.identity)
        .chain(&entity.fields)
        .find(|field| field.name == name)
        .map(|field| field.type_ref.clone())
        .ok_or_else(|| {
            format!(
                "the field `{name}`, which is not a field of `{}`",
                entity.name
            )
        })
}

/// One field of a stored row, resolved as a filter reads it.
pub(crate) fn resolve(ir: &EssIr, entity: &ResolvedEntity, name: &str) -> Result<Resolved, String> {
    let path = FactPath::new(name).map_err(|error| error.to_string())?;
    determined::resolve(ir, &Env::Row(entity), &path)
}

/// How the values an aggregate or an order reads compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Values {
    /// `Integer` or `Decimal`: by exact numeric value.
    Numeric,
    /// `String`: by UTF-8 bytes.
    Text,
    /// `Timestamp`: by the instant it names.
    Instant,
    /// `Boolean`, `Uuid`, an enum or `state`: equality only.
    Other,
}

/// How the values of the source field `name` compare, or `None` where nothing here compares them.
pub(crate) fn values(ir: &EssIr, entity: &ResolvedEntity, name: &str) -> Option<Values> {
    let resolved = resolve(ir, entity, name).ok()?;
    match resolved.kind {
        Kind::Number(_) => Some(Values::Numeric),
        Kind::Text => match leaf(ir, &resolved.root_type) {
            Some(Primitive::String) => Some(Values::Text),
            _ => Some(Values::Other),
        },
        Kind::Enum(..) | Kind::State | Kind::Bool => Some(Values::Other),
        Kind::Opaque => match leaf(ir, &resolved.root_type) {
            Some(Primitive::Timestamp) => Some(Values::Instant),
            _ => None,
        },
    }
}

/// The primitive at the end of a type, under every `Optional` and newtype.
pub(crate) fn leaf(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<Primitive> {
    match determined::representation(ir, type_ref) {
        Some(determined::Leaf::Primitive(primitive)) => Some(primitive),
        _ => None,
    }
}

/// `Ok` where the order key `name` ranks as the runner compares the rendered rows: a number by its
/// value, text, an identity, an enum or `state` by the text it is rendered as, a `Boolean` false
/// first.
fn order(ir: &EssIr, entity: &ResolvedEntity, name: &str) -> Result<(), String> {
    if let Kind::Enum(ResolvedTypeRef::Declared { name: declared }, _) =
        resolve(ir, entity, name)?.kind
    {
        if let ResolvedBody::Enum { variants } = &ir.named_type(&declared).body {
            if variants
                .iter()
                .any(|variant| variant.wire() != variant.name)
            {
                return Err(format!(
                    "an order over the enum field `{name}`, whose wire spellings are not its \
                     variant names"
                ));
            }
        }
    }
    match values(ir, entity, name) {
        Some(Values::Numeric | Values::Text | Values::Other) => Ok(()),
        Some(Values::Instant) => Err(format!(
            "an order over the `Timestamp` field `{name}`, which the suite ranks by its instant \
             and a generated query does not"
        )),
        None => Err(format!(
            "an order over the field `{name}`, a value the generated query does not rank"
        )),
    }
}

/// `Ok` where one aggregate field is computed as `aggregate::evaluate` computes it, into the type
/// the view declares.
fn aggregate(
    ir: &EssIr,
    entity: &ResolvedEntity,
    field: &ResolvedField,
    aggregate: &ResolvedAggregate,
) -> Result<(), String> {
    let integer = ResolvedTypeRef::Primitive {
        name: Primitive::Integer,
    };
    let Some(input) = &aggregate.input else {
        return expect(&field.type_ref, &integer);
    };
    let kind = values(ir, entity, &input.name).ok_or_else(|| {
        format!(
            "`{aggregate}` over `{}`, a value the generated query does not compare",
            input.name
        )
    })?;
    let optional = |of: ResolvedTypeRef| ResolvedTypeRef::Optional { of: Box::new(of) };
    match aggregate.function {
        AggregateFunction::Count | AggregateFunction::CountDistinct => {
            expect(&field.type_ref, &integer)
        }
        AggregateFunction::Sum => {
            let Some(primitive @ (Primitive::Integer | Primitive::Decimal)) =
                leaf(ir, &input.type_ref).filter(|_| kind == Values::Numeric)
            else {
                return Err(format!("`{aggregate}` over a value that is not a number"));
            };
            let sum = ResolvedTypeRef::Primitive { name: primitive };
            if aggregate.skip_absent {
                expect(&field.type_ref, &optional(sum))
            } else {
                expect(&field.type_ref, &sum)
            }
        }
        AggregateFunction::Avg => {
            if kind != Values::Numeric {
                return Err(format!("`{aggregate}` over a value that is not a number"));
            }
            expect(
                &field.type_ref,
                &optional(ResolvedTypeRef::Primitive {
                    name: Primitive::Decimal,
                }),
            )
        }
        AggregateFunction::Min | AggregateFunction::Max => {
            if kind == Values::Other {
                return Err(format!("`{aggregate}` over a value with no order"));
            }
            expect(
                &field.type_ref,
                &optional(input.type_ref.required().clone()),
            )
        }
    }
}

/// `Ok` where the view declares the type the computation produces.
fn expect(declared: &ResolvedTypeRef, produced: &ResolvedTypeRef) -> Result<(), String> {
    if declared == produced {
        Ok(())
    } else {
        Err(format!(
            "a declared type `{declared}` where the computation produces `{produced}`"
        ))
    }
}
