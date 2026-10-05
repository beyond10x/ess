//! Determined values read original subjects and related rows before any writes or events.
use super::{
    input, Completeness, EssIr, Node, Number, ResolvedPayloadField, ResolvedTypeRef, Row,
    Undetermined, Value,
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
    store: &super::State,
    input: &super::Context<'_>,
) -> Option<Row> {
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
        .insert(entity.identity.name.clone(), Value::Known(identity.clone()));
    Some(before)
}

/// Read authority is the original store and original addressed subject, never the evolving writes.
#[derive(Clone, Copy)]
pub(super) struct Reads<'a> {
    pub(super) original: &'a super::State,
    pub(super) before: Option<&'a Row>,
    pub(super) outcome: &'a ResolvedOutcome,
}

impl<'a> Reads<'a> {
    /// `{related: …}` (ess/16): the field of the row the reference names, read from the original
    /// store. From ess/22 (beyond10x/ess#285) a reference may be `Optional<…>` and a chain follows
    /// one further reference from that row: an absent reference reads no row and yields an absent
    /// value, and a present one naming no row is the missing row it always was — never absent.
    pub(super) fn related(
        self,
        ir: &EssIr,
        target: &ResolvedPayloadField,
        input: &super::Context<'_>,
    ) -> Result<Option<Value>, Undetermined> {
        let ResolvedPayloadValue::RelatedField {
            via,
            through,
            entity,
            field,
            type_ref,
        } = &target.value
        else {
            unreachable!("only related sources use this evaluator")
        };
        let absent = || {
            checked_read(
                ir,
                field,
                type_ref,
                &target.target_type,
                target.conversion.as_deref(),
                None,
            )
        };
        let first = match via {
            ResolvedRelatedVia::Input { field, .. } => {
                sent_reference(via.type_ref(), input.get(field))
            }
            ResolvedRelatedVia::Subject { field, .. } => match self.before {
                Some(before) => stored_reference(
                    ir,
                    via.type_ref(),
                    before.fields.get(field),
                    "the original related row address",
                )?,
                None => sent_reference(
                    via.type_ref(),
                    self.creation_input(ir, field)
                        .and_then(|field| input.get(field)),
                ),
            },
        };
        let mut address = match first {
            Reference::Present(address) => address,
            Reference::Absent => return absent(),
            Reference::Missing => {
                return Err(Undetermined::NoValue {
                    what: format!("the related identity named by `{via}` before the outcome"),
                })
            }
        };
        input::validate_typed_value(ir, via.type_ref(), &address).map_err(Undetermined::Request)?;
        // The entity each reference names: the next hop's, and the last one's is `entity`.
        let mut named = through
            .iter()
            .map(|hop| &hop.entity)
            .chain(std::iter::once(entity));
        let mut current = ir.entity(named.next().expect("at least the last entity"));
        let mut row = self.row(ir, current, &address, &via.to_string())?;
        for (hop, next) in through.iter().zip(named) {
            let held = if hop.field == current.identity.name {
                Some(Value::Known(address.clone()))
            } else {
                row.fields.get(&hop.field).cloned()
            };
            let next_address = match stored_reference(
                ir,
                &hop.type_ref,
                held.as_ref(),
                &format!("the related identity `{hop}`"),
            )? {
                Reference::Present(address) => address,
                Reference::Absent => return absent(),
                Reference::Missing => {
                    return Err(Undetermined::NoValue {
                        what: format!("the related identity `{hop}` named through `{via}`"),
                    })
                }
            };
            input::validate_typed_value(ir, &hop.type_ref, &next_address)
                .map_err(Undetermined::Request)?;
            current = ir.entity(next);
            address = next_address;
            row = self.row(ir, current, &address, &format!("{via} through {hop}"))?;
        }
        let value = if *field == current.identity.name {
            Some(Value::Known(address))
        } else {
            row.fields.get(field).cloned()
        };
        checked_read(
            ir,
            field,
            type_ref,
            &target.target_type,
            target.conversion.as_deref(),
            value.as_ref(),
        )
    }

    /// The original row of `entity` that `address` names, after checking the address is that
    /// entity's identity; its absence is a missing row, never an absent value.
    fn row(
        self,
        ir: &EssIr,
        entity: &ess_compiler::ir::ResolvedEntity,
        address: &Node,
        named_by: &str,
    ) -> Result<&'a Row, Undetermined> {
        input::validate_typed_value(ir, &entity.identity.type_ref, address)
            .map_err(Undetermined::Request)?;
        self.original
            .instance_typed(&entity.name, address)
            .ok_or_else(|| Undetermined::NoValue {
                what: format!(
                    "the original related `{}` row named by `{named_by}`",
                    entity.name
                ),
            })
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
    conversion: Option<&str>,
    before: Option<&Row>,
) -> Result<Option<Value>, Undetermined> {
    let before = before.ok_or_else(|| Undetermined::NoValue {
        what: format!("the pre-outcome subject field `{source}`"),
    })?;
    checked_read(
        ir,
        source,
        source_type,
        target_type,
        conversion,
        before.fields.get(source),
    )
}

fn checked_read(
    ir: &EssIr,
    source: &str,
    source_type: &ResolvedTypeRef,
    target_type: &ResolvedTypeRef,
    conversion: Option<&str>,
    value: Option<&Value>,
) -> Result<Option<Value>, Undetermined> {
    match value {
        Some(value) => {
            if let Ok(concrete) = value.concrete() {
                checked_concrete_read(ir, source, source_type, target_type, concrete.as_ref())
                    .map(|_| Some(value.clone()))
            } else {
                checked_abstract_read(ir, source_type, target_type, conversion, value).map(Some)
            }
        }
        None => checked_concrete_read(ir, source, source_type, target_type, None)
            .map(|value| value.map(Value::Known)),
    }
}

fn checked_abstract_read(
    ir: &EssIr,
    source_type: &ResolvedTypeRef,
    target_type: &ResolvedTypeRef,
    conversion: Option<&str>,
    value: &Value,
) -> Result<Value, Undetermined> {
    super::history::validate(ir, source_type, value)?;
    if source_type == target_type
        || ess_domain::types::is_assignable(
            &crate::accessor::unresolve(source_type),
            &crate::accessor::unresolve(target_type),
        )
    {
        super::history::validate(ir, target_type, value)?;
        return Ok(value.clone());
    }
    let Value::Unknown {
        declared,
        origin,
        domain,
    } = value
    else {
        return Err(Undetermined::Undecidable {
            outcome: "history value transfer".into(),
            guard: format!("abstract conversion from `{source_type}` to `{target_type}`"),
        });
    };
    if declared != source_type || !mechanical_conversion(ir, source_type, target_type, conversion) {
        return Err(Undetermined::Undecidable {
            outcome: "history value transfer".into(),
            guard: format!(
                "declared mechanical conversion from `{source_type}` to `{target_type}`"
            ),
        });
    }
    let complete = domain
        .clone()
        .or_else(|| super::history::finite(ir, declared, 0))
        .ok_or_else(|| Undetermined::Undecidable {
            outcome: "history value transfer".into(),
            guard: format!("complete source domain for `{source_type}`"),
        })?;
    let admitted = complete
        .iter()
        .filter(|member| input::validate_typed_value(ir, target_type, member).is_ok())
        .count();
    if admitted == 0 {
        return Err(Undetermined::Request(format!(
            "no value of `{source_type}` satisfies conversion target `{target_type}`"
        )));
    }
    if admitted != complete.len() {
        return Err(Undetermined::Undecidable {
            outcome: "history value transfer".into(),
            guard: format!(
                "every value of `{source_type}` satisfies conversion target `{target_type}`"
            ),
        });
    }
    Ok(Value::Unknown {
        declared: target_type.clone(),
        origin: origin.clone(),
        domain: Some(complete),
    })
}

fn mechanical_conversion(
    ir: &EssIr,
    source: &ResolvedTypeRef,
    target: &ResolvedTypeRef,
    conversion: Option<&str>,
) -> bool {
    let Some(named) = conversion else {
        return false;
    };
    let Some(declared) = ir.conversions().iter().find(|candidate| {
        candidate.from == *source && candidate.to == *target && candidate.because == named
    }) else {
        return false;
    };
    let (
        ResolvedTypeRef::Declared { name: source_name },
        ResolvedTypeRef::Declared { name: target_name },
    ) = (&declared.from, &declared.to)
    else {
        return false;
    };
    matches!(
        (&ir.named_type(source_name).body, &ir.named_type(target_name).body),
        (
            ess_compiler::ir::ResolvedBody::Newtype { of: source_inner, .. },
            ess_compiler::ir::ResolvedBody::Newtype { of: target_inner, .. }
        ) if source_inner == target_inner
    )
}

fn checked_concrete_read(
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
    checked_concrete_read(ir, field, type_ref, &target.target_type, actual.get(field))
        .map_err(|_| gap())
}

pub(super) fn increment(
    ir: &EssIr,
    field: &ResolvedPayloadField,
    by: &str,
    before: Option<&Row>,
    target_location: &[String],
    history: bool,
) -> Result<Value, Undetermined> {
    let location = target_location.join(".");
    let no_value = || Undetermined::NoValue {
        what: format!("the exact previous `{location}` plus `{by}`"),
    };
    let held = value_at(before, target_location)?;
    let increment = Number::decimal_literal(by).ok_or_else(no_value)?;
    if let Some(
        unknown @ Value::Unknown {
            declared,
            origin,
            domain,
        },
    ) = held.as_ref()
    {
        if increment == Number::from(0_i64) {
            return Ok(unknown.clone());
        }
        let complete = domain
            .clone()
            .or_else(|| super::history::finite(ir, declared, 0))
            .ok_or_else(|| Undetermined::Undecidable {
                outcome: "history increment".into(),
                guard: format!(
                    "complete domain and exact overflow proof for `{}` plus `{by}`",
                    field.target
                ),
            })?;
        let mut transformed = Vec::with_capacity(complete.len());
        let mut rejected = false;
        for member in complete {
            let Node::Number(previous) = member else {
                return Err(Undetermined::Request(format!(
                    "`{declared}` contains a nonnumeric value for increment"
                )));
            };
            let Some(sum) = previous.checked_add(increment) else {
                rejected = true;
                continue;
            };
            let value = Node::Number(sum);
            if input::validate_typed_value(ir, &field.target_type, &value).is_ok() {
                transformed.push(value);
            } else {
                rejected = true;
            }
        }
        if transformed.is_empty() {
            return Err(Undetermined::Request(format!(
                "no value of `{declared}` admits the exact increment `{by}`"
            )));
        }
        if rejected {
            return Err(Undetermined::Undecidable {
                outcome: "history increment".into(),
                guard: format!(
                    "exact overflow and target constraints for `{}` plus `{by}`",
                    field.target
                ),
            });
        }
        return Ok(Value::Unknown {
            declared: field.target_type.clone(),
            origin: origin.child(&format!("increment:{by}")),
            domain: Some(transformed),
        });
    }
    let Some(Value::Known(Node::Number(previous))) = held else {
        return Err(no_value());
    };
    let value = Node::Number(previous.checked_add(increment).ok_or_else(|| {
        if history {
            Undetermined::Request(format!(
                "the known exact increment of `{}` is unrepresentable",
                field.target
            ))
        } else {
            no_value()
        }
    })?);
    input::validate_typed_value(ir, &field.target_type, &value).map_err(Undetermined::Request)?;
    Ok(Value::Known(value))
}

/// The value at one typed target location in the immutable pre-outcome row.
fn value_at(before: Option<&Row>, location: &[String]) -> Result<Option<Value>, Undetermined> {
    let Some((root, rest)) = location.split_first() else {
        return Ok(None);
    };
    let Some(value) = before.and_then(|row| row.fields.get(root)) else {
        return Ok(None);
    };
    nested_value(value, rest, location)
}

fn nested_value(
    value: &Value,
    remaining: &[String],
    location: &[String],
) -> Result<Option<Value>, Undetermined> {
    let Some((member, rest)) = remaining.split_first() else {
        return Ok(Some(value.clone()));
    };
    match value {
        Value::Object(fields) => match fields.get(member) {
            Some(value) => nested_value(value, rest, location),
            None => Ok(None),
        },
        Value::Absent | Value::Known(Node::Null) => Ok(None),
        Value::Known(node) => Ok(node_at(node, remaining).cloned().map(Value::Known)),
        Value::Unknown { .. } => Err(Undetermined::Undecidable {
            outcome: "history increment".into(),
            guard: format!(
                "the parent value on `{}` before the outcome",
                location.join(".")
            ),
        }),
    }
}

fn node_at<'a>(node: &'a Node, location: &[String]) -> Option<&'a Node> {
    let Some((member, rest)) = location.split_first() else {
        return Some(node);
    };
    let Node::Map(fields) = node else {
        return None;
    };
    node_at(fields.get(member)?, rest)
}

/// What a reference a `{related: …}` value follows holds (ess/22, beyond10x/ess#285).
enum Reference {
    /// An identity.
    Present(Node),
    /// Nothing, and the reference is `Optional<…>`: the value read through it is absent.
    Absent,
    /// Nothing, and the reference is required: no value can be determined.
    Missing,
}

impl From<Option<Node>> for Reference {
    fn from(value: Option<Node>) -> Self {
        value.map_or(Self::Missing, Self::Present)
    }
}

/// What a reference the request sent holds.
fn sent_reference(type_ref: &ResolvedTypeRef, value: Option<&Node>) -> Reference {
    if super::related::absent_optional(type_ref, value) {
        return Reference::Absent;
    }
    value.cloned().into()
}

/// What a stored reference holds. Whether an abstract `Optional<…>` value is present is not chosen
/// here: it is undecidable.
fn stored_reference(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    value: Option<&Value>,
    what: &str,
) -> Result<Reference, Undetermined> {
    if type_ref.is_optional() {
        match value.map(|value| value.presence(ir)) {
            None | Some(Some(false)) => return Ok(Reference::Absent),
            Some(Some(true)) => {}
            Some(None) => {
                return Err(Undetermined::Undecidable {
                    outcome: "history value read".into(),
                    guard: format!("whether {what} is present"),
                })
            }
        }
    }
    Ok(value.map(|value| value.require(what)).transpose()?.into())
}

/// The field a filtered read takes from the one row its selector selected (ess/22,
/// beyond10x/ess#299): the identity, or the stored field as the row held it before the outcome.
pub(super) fn selected(
    ir: &EssIr,
    target: &ResolvedPayloadField,
    entity: &ess_compiler::ir::ResolvedEntity,
    key: &Node,
    row: &Row,
) -> Result<Option<Value>, Undetermined> {
    let ResolvedPayloadValue::RelatedSelection {
        field, type_ref, ..
    } = &target.value
    else {
        unreachable!("only filtered reads use this evaluator")
    };
    let value = if *field == entity.identity.name {
        Some(Value::Known(key.clone()))
    } else {
        row.fields.get(field).cloned()
    };
    checked_read(
        ir,
        field,
        type_ref,
        &target.target_type,
        target.conversion.as_deref(),
        value.as_ref(),
    )
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
