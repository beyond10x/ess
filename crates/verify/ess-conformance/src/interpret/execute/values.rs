//! Determined values read original subjects and related rows before any writes or events.
use super::{
    input, Completeness, EssIr, Instance, Node, Number, ResolvedPayloadField, ResolvedTypeRef,
    Undetermined,
};
use ess_compiler::ir::{
    ResolvedCommand, ResolvedEffect, ResolvedField, ResolvedInstance, ResolvedOutcome,
    ResolvedPayloadValue, ResolvedRelatedVia,
};
use ess_domain::entity::{Cardinality, RelationKind};
use std::collections::BTreeMap;

/// Capture only the subject already selected for this outcome. Stored-guard refusals have no
/// effect of their own, but their payloads read the same existing row their guard selected.
pub(super) fn selected_subject(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    store: &super::Store,
    input: &super::Invocation<'_>,
) -> Option<Instance> {
    let subject = outcome.subject.as_ref().or_else(|| {
        outcome
            .error
            .as_ref()
            .and_then(|_| command.selection_subject(outcome))
    })?;
    let ResolvedInstance::Supplied { field } = &subject.instance else {
        return None;
    };
    let entity = ir.entity(&subject.entity);
    let identity = input.get(&field.name)?;
    let mut before = store.instance_typed(&entity.name, identity)?.clone();
    before
        .fields
        .insert(entity.identity.name.clone(), identity.clone());
    Some(before)
}

/// Read authority is the original store and original addressed subject, never the evolving writes.
#[derive(Clone, Copy)]
pub(super) struct Reads<'a> {
    pub(super) original: &'a super::Store,
    pub(super) before: Option<&'a Instance>,
    pub(super) outcome: &'a ResolvedOutcome,
}

impl Reads<'_> {
    pub(super) fn related(
        self,
        ir: &EssIr,
        target: &ResolvedPayloadField,
        input: &super::Invocation<'_>,
    ) -> Result<Option<Node>, Undetermined> {
        let ResolvedPayloadValue::RelatedField {
            via,
            entity,
            field,
            type_ref,
        } = &target.value
        else {
            unreachable!("only related sources use this evaluator")
        };
        let address = match via {
            ResolvedRelatedVia::Input { field, .. } => input.get(field),
            ResolvedRelatedVia::Subject { field, .. } => match self.before {
                Some(before) => before.fields.get(field),
                None => self
                    .creation_input(ir, field)
                    .and_then(|field| input.get(field)),
            },
        }
        .ok_or_else(|| Undetermined::NoValue {
            what: format!("the related identity named by `{via}` before the outcome"),
        })?;
        let entity = ir.entity(entity);
        input::validate_typed_value(ir, via.type_ref(), address).map_err(Undetermined::Request)?;
        input::validate_typed_value(ir, &entity.identity.type_ref, address)
            .map_err(Undetermined::Request)?;
        let row = self
            .original
            .instance_typed(&entity.name, address)
            .ok_or_else(|| Undetermined::NoValue {
                what: format!(
                    "the original related `{}` row named by `{via}`",
                    entity.name
                ),
            })?;
        let value = if *field == entity.identity.name {
            Some(address)
        } else {
            row.fields.get(field)
        };
        checked_read(ir, field, type_ref, &target.target_type, value)
    }

    /// E8's creation exception: only the selected branch's unchanged input carrier. Compiler IR
    /// intentionally retains Subject here; no partially constructed row is a source of authority.
    fn creation_input(&self, ir: &EssIr, field: &str) -> Option<&str> {
        let subject = self
            .outcome
            .subject
            .as_ref()
            .filter(|subject| subject.effect == ResolvedEffect::Creates)?;
        let entity = ir.entity(&subject.entity);
        let source = if entity.identity.name == field {
            entity.relations.iter().find(|relation| {
                relation.kind == RelationKind::References
                    && relation.cardinality == Cardinality::One
                    && relation.via == field
                    && relation.target.name() != &entity.name
            })?;
            super::existence::identity_source(self.outcome)?
        } else {
            self.outcome.sets.iter().find(|set| set.target == field)?
        };
        match &source.value {
            ResolvedPayloadValue::InputField { field, .. } => Some(field),
            _ => None,
        }
    }
}

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
    checked_read(
        ir,
        source,
        source_type,
        target_type,
        before.fields.get(source),
    )
}

fn checked_read(
    ir: &EssIr,
    source: &str,
    source_type: &ResolvedTypeRef,
    target_type: &ResolvedTypeRef,
    value: Option<&Node>,
) -> Result<Option<Node>, Undetermined> {
    if let Some(value) = value {
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

pub(super) fn response(
    ir: &EssIr,
    target: &ResolvedPayloadField,
    actual: Option<&super::super::response::Value>,
) -> Result<Option<Node>, Undetermined> {
    let ResolvedPayloadValue::ResponseField { field, type_ref } = &target.value else {
        unreachable!("only response sources use this evaluator")
    };
    let gap = || Undetermined::NotInterpreted {
        construct: "an actual response field or its executable conversion".into(),
    };
    if target.conversion.is_some() {
        return Err(gap());
    }
    let actual = actual.ok_or_else(gap)?;
    // Error text must not disclose a protected field elsewhere in this response.
    checked_read(ir, field, type_ref, &target.target_type, actual.get(field)).map_err(|_| gap())
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

#[cfg(test)]
mod response_tests {
    use super::*;

    #[test]
    fn actual_response_reads_preserve_absence_null_and_exact_values_and_validate_both_types() {
        let source = include_str!("../../../tests/fixtures/response-payload.yaml")
            .replace("type: demo.api.Item", "type: Optional<demo.api.Item>");
        let specification = ess_domain::Specification::assemble([(
            ess_domain::system::Source::new("response.yaml"),
            ess_domain::spec::RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let ir =
            ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
                .unwrap();
        let command = ir.commands().values().next().unwrap();
        let field = &command.outcomes[0].payload[0].fields[0];
        assert_eq!(response(&ir, field, Some(&BTreeMap::new())).unwrap(), None);
        let null = BTreeMap::from([("item".into(), Node::Null)]);
        assert_eq!(response(&ir, field, Some(&null)).unwrap(), Some(Node::Null));
        let mut present: BTreeMap<String, Node> = serde_json::from_str(r#"{"item":{"remaining":9007199254740993,"created":"2026-09-11T10:00:00Z","ended":"2026-09-11T10:01:00Z","state":"Ready","call_type":"incoming","features":{"enabled":true}}}"#).unwrap();
        assert_eq!(
            response(&ir, field, Some(&present)).unwrap(),
            present.get("item").cloned()
        );
        let mut converted = field.clone();
        converted.conversion = Some("opaque transformation".into());
        assert!(response(&ir, &converted, Some(&present)).is_err());
        let mut wrong_target = field.clone();
        wrong_target.target_type = ResolvedTypeRef::Primitive {
            name: ess_domain::types::Primitive::String,
        };
        assert!(response(&ir, &wrong_target, Some(&present)).is_err());
        present.insert("item".into(), Node::Bool(false));
        assert!(response(&ir, field, Some(&present)).is_err());
        assert!(response(&ir, field, None).is_err());
    }
}
