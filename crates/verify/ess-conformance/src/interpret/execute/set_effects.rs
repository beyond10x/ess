//! Filtered set effects select from the pre-outcome store, then apply writes in declaration order.
use super::{
    act, at_rest, input, Acted, Completeness, EssIr, Instance, Node, ResolvedCommand,
    ResolvedEffect, ResolvedInstance, ResolvedOutcome, ResolvedPayloadField, Store, TypedFacts,
    Undetermined, Work,
};
use ess_compiler::ir::{ResolvedEntity, ResolvedField};
use ess_primitives::facts::{FactPath, FactSource, FactValue, Scales};
use ess_primitives::predicate::{Predicate, Truth};
use std::collections::{BTreeMap, BTreeSet};

struct Plan<'a> {
    entity: &'a ResolvedEntity,
    effect: ResolvedEffect,
    sets: &'a [ResolvedPayloadField],
    keys: Vec<String>,
}

/// None means this is not an instances outcome. Some(0) is its successful zero-match result.
pub(super) fn apply(
    ir: &EssIr,
    spec: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    before: &Store,
    supplied: &super::Invocation<'_>,
    work: &mut Work<'_>,
) -> Result<Option<usize>, Undetermined> {
    if outcome.instances.is_none() && outcome.affects.is_empty() {
        return Ok(None);
    }
    let input = input::flatten(ir, spec, supplied)
        .map_err(|error| Undetermined::Request(error.to_string()))?;
    let mut plans = Vec::new();
    if let Some(set) = &outcome.instances {
        let entity = ir.entity(&set.entity);
        plans.push(Plan {
            entity,
            effect: set.effect.clone(),
            sets: &outcome.sets,
            keys: select(ir, before, entity, &set.filter, &input, None, None)?
                .into_iter()
                .filter(|key| match &set.effect {
                    ResolvedEffect::Moves { transition } => before
                        .instance(&entity.name, key)
                        .is_some_and(|row| transition.from.contains(&row.state)),
                    _ => true,
                })
                .collect(),
        });
    }
    if !outcome.affects.is_empty() {
        let (entity, key, row) = subject(ir, outcome, before, supplied)?;
        let fields = fields(entity);
        let facts = row_facts(ir, &fields, entity, key, row)?;
        for affect in &outcome.affects {
            let affected = ir.entity(&affect.entity);
            let keys = select(
                ir,
                before,
                affected,
                &affect.filter,
                &input,
                Some(&facts),
                (affected.name == entity.name).then_some(key),
            )?;
            plans.push(Plan {
                entity: affected,
                effect: ResolvedEffect::Updates,
                sets: &affect.sets,
                keys,
            });
        }
    }
    let mut applied = 0;
    let mut touched = BTreeSet::new();
    for plan in plans {
        for key in plan.keys {
            let row = work
                .next
                .instance(&plan.entity.name, &key)
                .expect("a selected row remains held through moves and updates")
                .clone();
            let Acted::Rests(after) = act(ir, plan.sets, &plan.effect, &row, supplied, work)?
            else {
                return Err(Undetermined::Request(
                    "a selected set row cannot undergo its declared effect".into(),
                ));
            };
            validate_writes(ir, plan.sets, &after)?;
            work.next
                .instances
                .entry(plan.entity.name.clone())
                .or_default()
                .insert(key.clone(), after);
            touched.insert((plan.entity.name.clone(), key));
            applied += 1;
        }
    }
    for (entity, key) in touched {
        at_rest(ir, &work.next, &entity, &key)?;
    }
    Ok(outcome.instances.as_ref().map(|_| applied))
}

fn validate_writes(
    ir: &EssIr,
    sets: &[ResolvedPayloadField],
    after: &Instance,
) -> Result<(), Undetermined> {
    for field in sets {
        match after.fields.get(&field.target) {
            Some(value) => input::validate_typed_value(ir, &field.target_type, value)
                .map_err(Undetermined::Request)?,
            None if field.target_type.is_optional() => {}
            None => {
                return Err(Undetermined::NoValue {
                    what: format!("set field `{}`", field.target),
                })
            }
        }
    }
    Ok(())
}

fn subject<'a>(
    ir: &'a EssIr,
    outcome: &ResolvedOutcome,
    store: &'a Store,
    supplied: &'a BTreeMap<String, Node>,
) -> Result<(&'a ResolvedEntity, &'a str, &'a Instance), Undetermined> {
    let declared = outcome.subject.as_ref().ok_or_else(|| {
        Undetermined::Request("secondary effects require an existing subject".into())
    })?;
    let ResolvedInstance::Supplied { field } = &declared.instance else {
        return Err(Undetermined::Request(
            "secondary effects require a supplied subject".into(),
        ));
    };
    let entity = ir.entity(&declared.entity);
    let key = supplied
        .get(&field.name)
        .and_then(Node::as_text)
        .ok_or_else(|| {
            Undetermined::Request("secondary effect subject identity is absent".into())
        })?;
    let row = store
        .instance(&entity.name, key)
        .ok_or_else(|| Undetermined::Request("secondary effect subject is not held".into()))?;
    Ok((entity, key, row))
}

fn fields(entity: &ResolvedEntity) -> Vec<ResolvedField> {
    let mut fields = entity.fields.clone();
    fields.push(entity.identity.clone());
    fields
}

fn row_facts<'a>(
    ir: &'a EssIr,
    fields: &'a [ResolvedField],
    entity: &ResolvedEntity,
    key: &str,
    row: &Instance,
) -> Result<TypedFacts<'a>, Undetermined> {
    let mut values = row.fields.clone();
    values.insert(entity.identity.name.clone(), Node::Text(key.into()));
    let facts = input::bind(ir, fields, &values, Completeness::Partial)
        .map_err(|why| Undetermined::Request(why.to_string()))?;
    let mut facts = TypedFacts::new(ir, fields, facts);
    facts.set(
        FactPath::new("state").expect("fixed path"),
        FactValue::text(row.state.to_string()),
    );
    Ok(facts)
}

fn select(
    ir: &EssIr,
    store: &Store,
    entity: &ResolvedEntity,
    filter: &Predicate,
    input: &input::InputFacts<'_>,
    subject: Option<&TypedFacts<'_>>,
    excluded: Option<&str>,
) -> Result<Vec<String>, Undetermined> {
    let fields = fields(entity);
    let mut selected = Vec::new();
    for (_, key, row) in store
        .instances()
        .filter(|(name, key, _)| *name == &entity.name && Some(*key) != excluded)
    {
        let row = row_facts(ir, &fields, entity, key, row)?;
        match filter.evaluate(&Facts {
            row: &row,
            input,
            subject: subject.map(|facts| facts as &dyn FactSource),
        }) {
            Truth::True => selected.push(key.into()),
            Truth::False => {}
            Truth::Unknown => {
                return Err(Undetermined::Undecidable {
                    outcome: format!("set effect on {}", entity.name),
                    guard: format!("{filter:?}"),
                })
            }
        }
    }
    Ok(selected)
}

struct Facts<'a> {
    row: &'a dyn FactSource,
    input: &'a dyn FactSource,
    subject: Option<&'a dyn FactSource>,
}
impl Facts<'_> {
    fn source(&self, path: &FactPath) -> Option<(&dyn FactSource, FactPath)> {
        let (first, rest) = path.segments().split_first()?;
        match (first.as_str(), rest.is_empty()) {
            ("input", false) => Some((self.input, FactPath::from_segments(rest))),
            ("subject", false) => Some((self.subject?, FactPath::from_segments(rest))),
            _ => Some((self.row, path.clone())),
        }
    }
}
impl FactSource for Facts<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.source(path)
            .and_then(|(source, path)| source.fact(&path))
    }
    fn present(&self, path: &FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.present(&path))
    }
    fn scales(&self) -> &Scales {
        self.input.scales()
    }
    fn orders_as_instant(&self, path: &FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.orders_as_instant(&path))
    }
    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        self.source(path)
            .is_some_and(|(source, path)| source.orders_text_by_bytes(&path))
    }
}
