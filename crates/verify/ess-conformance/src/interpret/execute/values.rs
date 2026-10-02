//! Determined values read the immutable subject before any assignment or event is evaluated.
use super::{
    input, Completeness, EssIr, Instance, Node, Number, ResolvedPayloadField, ResolvedTypeRef,
    Undetermined,
};
use ess_compiler::ir::ResolvedField;
use std::collections::BTreeMap;

pub(super) fn subject(
    ir: &EssIr,
    source: &str,
    source_type: &ResolvedTypeRef,
    target_type: &ResolvedTypeRef,
    before: Option<&Instance>,
) -> Result<Option<Node>, Undetermined> {
    let before = before.ok_or_else(|| Undetermined::NoValue {
        what: format!("the pre-outcome subject field `{source}`"),
    })?;
    if let Some(value) = before.fields.get(source) {
        input::validate_typed_value(ir, source_type, value).map_err(Undetermined::Request)?;
        input::validate_typed_value(ir, target_type, value).map_err(Undetermined::Request)?;
        Ok(Some(value.clone()))
    } else {
        // Reuse input presence rules, including newtypes over Optional. A missing required
        // field is unknown, never an absent optional or a value supplied by another write.
        let field = ResolvedField {
            name: source.into(),
            type_ref: source_type.clone(),
            naming: ess_domain::name::Naming::default(),
        };
        input::bind(ir, &[field], &BTreeMap::new(), Completeness::Total)
            .map_err(|why| Undetermined::Request(why.to_string()))?;
        Ok(None)
    }
}

pub(super) fn increment(
    ir: &EssIr,
    field: &ResolvedPayloadField,
    by: &str,
    before: Option<&Instance>,
) -> Result<Node, Undetermined> {
    let no_value = || Undetermined::NoValue {
        what: format!("the exact previous `{}` plus `{by}`", field.target),
    };
    let Some(Node::Number(previous)) = before.and_then(|row| row.fields.get(&field.target)) else {
        return Err(no_value());
    };
    let increment = Number::decimal_literal(by).ok_or_else(no_value)?;
    let value = Node::Number(previous.checked_add(increment).ok_or_else(no_value)?);
    input::validate_typed_value(ir, &field.target_type, &value).map_err(Undetermined::Request)?;
    Ok(value)
}
