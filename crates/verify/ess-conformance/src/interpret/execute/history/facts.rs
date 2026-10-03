//! Typed facts from exact known leaves and explicit unknown presence.
use super::{Row, Value};
use crate::input::{self, TypedFacts};
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedField, ResolvedTypeRef};
use ess_domain::types::Primitive;
use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue, Scales};
use ess_primitives::predicate::Truth;
use std::collections::BTreeMap;

pub(in super::super) struct Facts<'a> {
    ir: &'a EssIr,
    fields: &'a [ResolvedField],
    values: BTreeMap<String, Value>,
    known: TypedFacts<'a>,
    state: Option<FactValue>,
}

impl<'a> Facts<'a> {
    pub(in super::super) fn unobserved(&self, path: &FactPath) -> bool {
        let Some((root, rest)) = path.segments().split_first() else {
            return false;
        };
        self.values
            .get(root)
            .and_then(|value| at(self.ir, value, rest, 0))
            .is_some_and(|value| value.unobserved())
    }

    pub(in super::super) fn row(
        ir: &'a EssIr,
        fields: &'a [ResolvedField],
        row: &Row,
    ) -> Result<Self, super::Undetermined> {
        let mut facts = Self::new(ir, fields, &row.fields)?;
        let state = FactValue::text(row.state.to_string());
        facts.known.set(
            FactPath::new("state").expect("fixed state fact"),
            state.clone(),
        );
        facts.state = Some(state);
        Ok(facts)
    }

    pub(in super::super) fn new(
        ir: &'a EssIr,
        fields: &'a [ResolvedField],
        values: &BTreeMap<String, Value>,
    ) -> Result<Self, super::Undetermined> {
        let mut known = FactStore::new();
        for field in fields {
            if let Some(value) = values.get(&field.name) {
                project(
                    ir,
                    &field.type_ref,
                    value,
                    &FactPath::new(&field.name).expect("resolved field name"),
                    &mut known,
                    0,
                )?;
            }
        }
        Ok(Self {
            ir,
            fields,
            values: values.clone(),
            known: TypedFacts::new(ir, fields, known),
            state: None,
        })
    }

    /// Evaluate against every member of each retained complete root domain. This is a universal
    /// proof over bounded alternatives; it never installs a member as observed history data.
    pub(in super::super) fn evaluate_with(&self, evaluate: impl Fn(&Self) -> Truth) -> Truth {
        let direct = evaluate(self);
        if direct != Truth::Unknown {
            return direct;
        }
        let Some(alternatives) = self.alternatives() else {
            return Truth::Unknown;
        };
        let mut result = None;
        for alternative in alternatives {
            let truth = evaluate(&alternative);
            match (result, truth) {
                (_, Truth::Unknown) => return Truth::Unknown,
                (None, truth) => result = Some(truth),
                (Some(held), truth) if held == truth => {}
                _ => return Truth::Unknown,
            }
        }
        result.unwrap_or(Truth::Unknown)
    }

    fn alternatives(&self) -> Option<Vec<Self>> {
        const LIMIT: usize = 256;
        let mut alternatives = vec![self.values.clone()];
        for (name, value) in &self.values {
            let domain = match value {
                Value::Unknown {
                    domain: Some(domain),
                    ..
                } => domain,
                Value::Unknown { domain: None, .. } => return None,
                Value::Object(_) if value.unobserved() => return None,
                Value::Absent | Value::Known(_) | Value::Object(_) => continue,
            };
            if domain.is_empty() || alternatives.len().checked_mul(domain.len())? > LIMIT {
                return None;
            }
            let mut expanded = Vec::with_capacity(alternatives.len() * domain.len());
            for values in alternatives {
                for member in domain {
                    let mut candidate = values.clone();
                    candidate.insert(name.clone(), Value::Known(member.clone()));
                    expanded.push(candidate);
                }
            }
            alternatives = expanded;
        }
        alternatives
            .into_iter()
            .map(|values| -> Result<Self, super::Undetermined> {
                let mut facts = Self::new(self.ir, self.fields, &values)?;
                if let Some(state) = &self.state {
                    facts.known.set(
                        FactPath::new("state").expect("fixed state fact"),
                        state.clone(),
                    );
                    facts.state = Some(state.clone());
                }
                Ok(facts)
            })
            .collect::<Result<Vec<_>, _>>()
            .ok()
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the recursive projection keeps all resolved value shapes in one exhaustive match"
)]
fn project(
    ir: &EssIr,
    kind: &ResolvedTypeRef,
    value: &Value,
    path: &FactPath,
    facts: &mut FactStore,
    depth: usize,
) -> Result<(), super::Undetermined> {
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return Err(super::Undetermined::Request(
            "abstract fact type depth".into(),
        ));
    }
    match value {
        Value::Known(value) => {
            input::validate_typed_value(ir, kind, value).map_err(super::Undetermined::Request)?;
            input::project_observed(ir, kind, value, path, facts)
                .map_err(|error| super::Undetermined::Request(error.to_string()))
        }
        Value::Absent => Ok(()),
        Value::Unknown {
            declared,
            origin,
            domain,
        } => {
            // Only facts identical throughout an exhaustively validated domain are guaranteed.
            // This is a universal proof, never selection of the feasibility witness.
            if super::values::allows_absence(ir, declared, 0) {
                return Ok(());
            }
            if let ResolvedTypeRef::Declared { name } = declared {
                match &ir.named_type(name).body {
                    ResolvedBody::Struct { fields, .. } => {
                        for field in fields {
                            let child = Value::Unknown {
                                declared: field.type_ref.clone(),
                                origin: origin.child(&field.name),
                                domain: None,
                            };
                            project(
                                ir,
                                &field.type_ref,
                                &child,
                                &path.child(&field.name),
                                facts,
                                depth + 1,
                            )?;
                        }
                    }
                    ResolvedBody::Newtype { of, .. } => {
                        let base = Value::Unknown {
                            declared: of.clone(),
                            origin: origin.clone(),
                            domain: domain.clone(),
                        };
                        project(ir, of, &base, path, facts, depth + 1)?;
                    }
                    _ => {}
                }
            }
            let values = domain
                .clone()
                .or_else(|| super::values::finite(ir, declared, 0));
            let Some(values) = values else {
                return Ok(());
            };
            let mut common: Option<BTreeMap<FactPath, FactValue>> = None;
            for value in values {
                let mut projected = FactStore::new();
                input::project_observed(ir, declared, &value, path, &mut projected)
                    .map_err(|error| super::Undetermined::Request(error.to_string()))?;
                if let Some(common) = &mut common {
                    common.retain(|path, value| projected.fact(path).as_ref() == Some(value));
                } else {
                    common = Some(
                        projected
                            .iter()
                            .map(|(path, value)| (path.clone(), value.clone()))
                            .collect(),
                    );
                }
            }
            if let Some(common) = common {
                facts.extend_facts(common);
            }
            Ok(())
        }
        Value::Object(values) => match kind {
            ResolvedTypeRef::Optional { of } => project(ir, of, value, path, facts, depth + 1),
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => project(ir, of, value, path, facts, depth + 1),
                ResolvedBody::Struct { fields, .. } => {
                    facts.mark_present(path.clone());
                    for field in fields {
                        if let Some(value) = values.get(&field.name) {
                            project(
                                ir,
                                &field.type_ref,
                                value,
                                &path.child(&field.name),
                                facts,
                                depth + 1,
                            )?;
                        }
                    }
                    Ok(())
                }
                _ => Err(super::Undetermined::Request(
                    "constructed history object requires a struct".into(),
                )),
            },
            _ => Err(super::Undetermined::Request(
                "constructed history object requires a declared struct".into(),
            )),
        },
    }
}

impl FactSource for Facts<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.known.fact(path)
    }
    fn present(&self, path: &FactPath) -> bool {
        self.observed_presence(path) == Some(true)
    }
    fn observed_presence(&self, path: &FactPath) -> Option<bool> {
        if self.known.present(path) {
            return Some(true);
        }
        let Some((root, rest)) = path.segments().split_first() else {
            return Some(false);
        };
        self.values
            .get(root)
            .and_then(|value| at(self.ir, value, rest, 0))
            .map_or(Some(false), |value| value.presence(self.ir))
    }
    fn scales(&self) -> &Scales {
        self.known.scales()
    }
    fn orders_as_instant(&self, path: &FactPath) -> bool {
        self.known.orders_as_instant(path)
    }
    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        self.known.orders_text_by_bytes(path)
    }
    fn cardinality(&self, path: &FactPath) -> Option<usize> {
        self.known.cardinality(path)
    }
}

fn at(ir: &EssIr, value: &Value, path: &[String], depth: usize) -> Option<Value> {
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return None;
    }
    if matches!(value, Value::Unknown { declared, .. } if super::values::only_absent(ir, declared, depth))
    {
        return Some(Value::Absent);
    }
    let Some((first, rest)) = path.split_first() else {
        return Some(value.clone());
    };
    match value {
        Value::Absent | Value::Known(_) => None,
        Value::Object(fields) => at(ir, fields.get(first)?, rest, depth + 1),
        Value::Unknown {
            declared,
            origin,
            domain: _,
        } => {
            let declared = member(ir, declared, first, depth + 1)?;
            at(
                ir,
                &Value::Unknown {
                    declared,
                    origin: origin.child(first),
                    domain: None,
                },
                rest,
                depth + 1,
            )
        }
    }
}

fn member(ir: &EssIr, kind: &ResolvedTypeRef, name: &str, depth: usize) -> Option<ResolvedTypeRef> {
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return None;
    }
    match kind {
        ResolvedTypeRef::Optional { of } => Some(ResolvedTypeRef::Optional {
            of: Box::new(member(ir, of, name, depth + 1)?),
        }),
        ResolvedTypeRef::Declared { name: declared } => match &ir.named_type(declared).body {
            ResolvedBody::Newtype { of, .. } => member(ir, of, name, depth + 1),
            ResolvedBody::Struct { fields, .. } => fields
                .iter()
                .find(|field| field.name == name)
                .map(|field| field.type_ref.clone()),
            ResolvedBody::Union { tag, .. } if tag == name => Some(ResolvedTypeRef::Primitive {
                name: Primitive::String,
            }),
            ResolvedBody::Union { .. } => Some(ResolvedTypeRef::Optional {
                of: Box::new(ResolvedTypeRef::Primitive {
                    name: Primitive::Json,
                }),
            }),
            ResolvedBody::Enum { .. } => None,
        },
        ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } if name == "count" => {
            Some(ResolvedTypeRef::Primitive {
                name: Primitive::Integer,
            })
        }
        ResolvedTypeRef::List { of } | ResolvedTypeRef::Map { value: of, .. }
            if name.parse::<usize>().is_ok() =>
        {
            Some(ResolvedTypeRef::Optional { of: of.clone() })
        }
        ResolvedTypeRef::Primitive {
            name:
                Primitive::String
                | Primitive::Uuid
                | Primitive::Timestamp
                | Primitive::Duration
                | Primitive::Bytes,
        } if name == "count" => Some(ResolvedTypeRef::Primitive {
            name: Primitive::Integer,
        }),
        ResolvedTypeRef::Primitive {
            name: Primitive::Json,
        } => Some(ResolvedTypeRef::Optional {
            of: Box::new(ResolvedTypeRef::Primitive {
                name: Primitive::Json,
            }),
        }),
        _ => None,
    }
}
