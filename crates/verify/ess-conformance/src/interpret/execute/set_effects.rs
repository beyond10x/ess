//! Filtered set effects select from the pre-outcome store, then apply writes in declaration order.
use super::{
    act, at_rest, input, Acted, EssIr, Node, ResolvedCommand, ResolvedEffect, ResolvedInstance,
    ResolvedOutcome, ResolvedPayloadField, Row, State, Undetermined, Value, Work,
};
use ess_compiler::ir::{ResolvedEntity, ResolvedField};
use ess_primitives::facts::{FactPath, FactSource, FactValue, Scales};
use ess_primitives::predicate::{Predicate, Truth};
use std::collections::{BTreeMap, BTreeSet};

struct Plan<'a> {
    entity: &'a ResolvedEntity,
    effect: ResolvedEffect,
    sets: &'a [ResolvedPayloadField],
    keys: Vec<Node>,
}

#[derive(Clone, Copy)]
struct Eligibility<'a> {
    excluded: Option<&'a Node>,
    effect: Option<&'a ResolvedEffect>,
    /// The subject row a `subject.` operand reads, and its entity.
    subject: Option<(&'a ResolvedEntity, &'a Row)>,
}

/// None means this is not an instances outcome. Some(0) is its successful zero-match result.
pub(super) fn apply(
    ir: &EssIr,
    spec: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    before: &State,
    supplied: &super::Context<'_>,
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
            keys: select(
                ir,
                before,
                entity,
                &set.filter,
                &input,
                None,
                Eligibility {
                    excluded: None,
                    effect: Some(&set.effect),
                    subject: None,
                },
            )?,
        });
    }
    if !outcome.affects.is_empty() {
        let (entity, key, row) = subject(ir, outcome, before, supplied)?;
        let fields = fields(entity);
        let facts = row_facts(ir, &fields, entity, key, row)?;
        for affect in &outcome.affects {
            let affected = ir.entity(&affect.entity);
            // From ess/22 an entry may move its rows (beyond10x/ess#229): a selected row resting
            // outside the move's `from` states is skipped, as under `instances:`.
            let effect = affect
                .moves
                .clone()
                .map_or(ResolvedEffect::Updates, |transition| {
                    ResolvedEffect::Moves { transition }
                });
            let keys = select(
                ir,
                before,
                affected,
                &affect.filter,
                &input,
                Some(&facts),
                Eligibility {
                    excluded: (affected.name == entity.name).then_some(key),
                    effect: affect.moves.is_some().then_some(&effect),
                    subject: Some((entity, row)),
                },
            )?;
            plans.push(Plan {
                entity: affected,
                effect,
                sets: &affect.sets,
                keys,
            });
        }
    }
    let mut applied = 0;
    let mut touched = BTreeSet::new();
    for (occurrence, plan) in plans.into_iter().enumerate() {
        for key in plan.keys {
            let row = work
                .next
                .instance_typed(&plan.entity.name, &key)
                .expect("a selected row remains held through moves and updates")
                .clone();
            work.location = vec![
                "set-effect".into(),
                occurrence.to_string(),
                "row".into(),
                plan.entity.name.to_string(),
                format!("{key:?}"),
                "sets".into(),
            ];
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
    after: &Row,
) -> Result<(), Undetermined> {
    for field in sets {
        match after.fields.get(&field.target) {
            Some(value) => super::history::validate(ir, &field.target_type, value)?,
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
    store: &'a State,
    supplied: &'a BTreeMap<String, Node>,
) -> Result<(&'a ResolvedEntity, &'a Node, &'a Row), Undetermined> {
    let declared = outcome.subject.as_ref().ok_or_else(|| {
        Undetermined::Request("secondary effects require an existing subject".into())
    })?;
    let ResolvedInstance::Supplied { field } = &declared.instance else {
        return Err(Undetermined::Request(
            "secondary effects require a supplied subject".into(),
        ));
    };
    let entity = ir.entity(&declared.entity);
    let key = supplied.get(&field.name).ok_or_else(|| {
        Undetermined::Request("secondary effect subject identity is absent".into())
    })?;
    let row = store
        .instance_typed(&entity.name, key)
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
    key: &Node,
    row: &Row,
) -> Result<super::history::Facts<'a>, Undetermined> {
    let mut row = row.clone();
    row.fields
        .insert(entity.identity.name.clone(), Value::Known(key.clone()));
    super::history::Facts::row(ir, fields, &row)
}

fn select(
    ir: &EssIr,
    store: &State,
    entity: &ResolvedEntity,
    filter: &Predicate,
    input: &input::InputFacts<'_>,
    subject: Option<&super::history::Facts<'_>>,
    eligibility: Eligibility<'_>,
) -> Result<Vec<Node>, Undetermined> {
    let fields = fields(entity);
    let mut selected = Vec::new();
    for (_, key, row) in store
        .instances()
        .filter(|(name, key, _)| *name == &entity.name && Some(*key) != eligibility.excluded)
    {
        if matches!(eligibility.effect, Some(ResolvedEffect::Moves { transition })
            if !transition.from.contains(&row.state))
        {
            continue;
        }
        let held = row;
        let row = row_facts(ir, &fields, entity, key, row)?;
        match row.evaluate_with(|candidate| {
            filter.evaluate(&Facts {
                row: candidate,
                input,
                subject: subject.map(|facts| facts as &dyn FactSource),
            })
        }) {
            Truth::True => selected.push(key.clone()),
            Truth::False => {}
            // Only a filter that holds selects a row. A filter left unknown because a field it
            // reads is absent (an `Optional<…>` row or subject field holding nothing, or an
            // Optional input left out) does not hold, so the row is not selected; one left
            // unknown by a value the store never observed stays undecidable.
            Truth::Unknown if only_absent(filter, entity, held, eligibility.subject) => {}
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

/// Whether every row or subject field `filter` reads that holds no value is an `Optional<…>` one
/// holding nothing: what makes an unknown filter a row left unselected rather than an undecidable
/// effect. A required field never written, or a value the store holds as unobserved (a generated
/// value no history recorded), keeps it undecidable. An `input.` operand is supplied or absent,
/// never unknown.
fn only_absent(
    filter: &Predicate,
    entity: &ResolvedEntity,
    row: &Row,
    subject: Option<(&ResolvedEntity, &Row)>,
) -> bool {
    filter.fact_paths().into_iter().all(|path| {
        let segments = path.segments();
        let (owner, held, field) = match segments.split_first() {
            Some((first, rest)) if first == "input" && !rest.is_empty() => return true,
            Some((first, rest)) if first == "subject" && !rest.is_empty() => match subject {
                Some((owner, held)) => (owner, held, &rest[0]),
                None => return true,
            },
            Some((first, _)) => (entity, row, first),
            None => return true,
        };
        match held.fields.get(field) {
            Some(Value::Absent) => true,
            Some(value) => !value.unobserved(),
            None => {
                owner.identity.name == *field
                    || owner
                        .fields
                        .iter()
                        .any(|declared| declared.name == *field && declared.type_ref.is_optional())
            }
        }
    })
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
    fn observed_presence(&self, path: &FactPath) -> Option<bool> {
        self.source(path).map_or(Some(false), |(source, path)| {
            source.observed_presence(&path)
        })
    }
    fn observe(&self, path: &FactPath) -> Option<FactValue> {
        self.source(path)
            .and_then(|(source, path)| source.observe(&path))
    }
    fn cardinality(&self, path: &FactPath) -> Option<usize> {
        self.source(path)
            .and_then(|(source, path)| source.cardinality(&path))
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
