//! Row sets read from the store as it was before the branch is selected
//! (`docs/design/filtered-related-reads.md`; beyond10x/ess#228, #299).
//!
//! Every row of the selector's entity is a candidate, in no order that decides anything. A row is
//! selected where the selector holds of it, read over the row's fields, identity and held state,
//! the input under `input.`, the addressed subject under `subject.` and the decision's one instant
//! as `now`. A row the selector leaves `Unknown` — a value the store holds unobserved — stays a
//! possible member: it is never dropped, and a test or a read it leaves undecided decides nothing.
use std::collections::BTreeMap;

use ess_compiler::ir::{
    ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedEntity, ResolvedInstance,
    ResolvedRowSelection, ResolvedRowSetTest, RowMember, Selected,
};
use ess_domain::command::OutcomeName;
use ess_primitives::facts::FactSource;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

use super::{caller, input, set_effects, Context, EssIr, Row, State, Undetermined};

/// Whether any branch of the command is guarded by a row set.
pub(super) fn uses(spec: &ResolvedCommand) -> bool {
    spec.outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, ResolvedCondition::RelatedSet { .. }))
}

/// What one decision reads its row sets against: the pre-outcome store, the input, the addressed
/// subject where the command names one, and the decision's instant.
pub(super) struct Reader<'a> {
    ir: &'a EssIr,
    store: &'a State,
    facts: input::InputFacts<'a>,
    now: Option<crate::occurrence_clock::DecisionInstant>,
    subject: Subject<'a>,
}

/// The subject a selector's `subject.` reads.
enum Subject<'a> {
    /// The command addresses none.
    None,
    /// It addresses one the store does not hold: a read of it decides nothing.
    Missing,
    /// The row, as it was before the outcome.
    Held(&'a ResolvedEntity, &'a Node, &'a Row),
}

impl<'a> Reader<'a> {
    pub(super) fn new(
        ir: &'a EssIr,
        spec: &'a ResolvedCommand,
        store: &'a State,
        input: &Context<'_>,
    ) -> Result<Self, Undetermined> {
        let facts = input::flatten(ir, spec, input)
            .map_err(|errors| Undetermined::Request(errors.to_string()))?;
        Ok(Self {
            ir,
            store,
            facts,
            now: input.now,
            subject: subject(ir, spec, store, input),
        })
    }

    /// Every candidate row of `selection`'s entity, with whether it is selected and — where
    /// `satisfies` is given — whether it satisfies that predicate.
    fn members(
        &self,
        selection: &ResolvedRowSelection,
        satisfies: Option<&Predicate>,
    ) -> Result<Vec<(&'a Node, &'a Row, RowMember)>, Undetermined> {
        let entity = self.ir.entity(&selection.entity);
        let fields = set_effects::fields(entity);
        let subject_fields;
        let held = match &self.subject {
            Subject::Held(owner, key, row) => {
                subject_fields = set_effects::fields(owner);
                Some(set_effects::row_facts(
                    self.ir,
                    &subject_fields,
                    owner,
                    key,
                    row,
                )?)
            }
            Subject::None | Subject::Missing => None,
        };
        let subject = held.as_ref().map(|facts| facts as &dyn FactSource);
        if subject.is_none()
            && (reads_subject(&selection.filter) || satisfies.is_some_and(reads_subject))
        {
            return Err(Undetermined::Undecidable {
                outcome: format!("row set over {}", entity.name),
                guard: "a selector reading `subject.` where no addressed subject is held".into(),
            });
        }
        let mut members = Vec::new();
        for (_, key, row) in self
            .store
            .instances()
            .filter(|(name, _, _)| **name == entity.name)
        {
            let candidate = set_effects::row_facts(self.ir, &fields, entity, key, row)?;
            let truth = |predicate: &Predicate| {
                candidate.evaluate_with(|candidate| {
                    let read = set_effects::Facts {
                        row: candidate,
                        input: &self.facts,
                        subject,
                    };
                    predicate.evaluate(&caller::Facts::new(&read, None, &[], self.now))
                })
            };
            let selected = truth(&selection.filter);
            let satisfies = match (satisfies, selected) {
                (Some(_), Truth::False) | (None, _) => Truth::True,
                (Some(predicate), _) => truth(predicate),
            };
            members.push((
                key,
                row,
                RowMember {
                    selected,
                    satisfies,
                },
            ));
        }
        Ok(members)
    }

    /// Whether `test` holds of the rows `selection` selects.
    pub(super) fn decide(
        &self,
        selection: &ResolvedRowSelection,
        test: &ResolvedRowSetTest,
    ) -> Result<Truth, Undetermined> {
        let members = self.members(selection, test.predicate())?;
        let members: Vec<RowMember> = members.into_iter().map(|(.., member)| member).collect();
        Ok(test.decide(&members))
    }

    /// The one row `selection` selects, or why there is none to read.
    pub(super) fn one(
        &self,
        selection: &ResolvedRowSelection,
    ) -> Result<(&'a Node, &'a Row), Undetermined> {
        let members = self.members(selection, None)?;
        let read: Vec<RowMember> = members.iter().map(|(.., member)| *member).collect();
        let entity = &self.ir.entity(&selection.entity).name;
        match Selected::of(&read) {
            Selected::One(index) => {
                let (key, row, _) = members[index];
                Ok((key, row))
            }
            Selected::None => Err(Undetermined::NoValue {
                what: format!("a field of the one `{entity}` the selector selects: none is"),
            }),
            Selected::Several => Err(Undetermined::NoValue {
                what: format!(
                    "a field of the one `{entity}` the selector selects: several are, and none is \
                     chosen"
                ),
            }),
            Selected::Unknown => Err(Undetermined::Undecidable {
                outcome: format!("filtered read of {entity}"),
                guard: format!("{:?}", selection.filter),
            }),
        }
    }
}

/// Whether `predicate` reads the addressed subject.
fn reads_subject(predicate: &Predicate) -> bool {
    predicate.fact_paths().iter().any(|path| {
        path.segments().len() > 1
            && path.namespace() == ess_domain::command::set_effects::SUBJECT_NAMESPACE
    })
}

/// The subject the command's branches address through the input, as the store held it.
fn subject<'a>(
    ir: &'a EssIr,
    spec: &ResolvedCommand,
    store: &'a State,
    input: &Context<'_>,
) -> Subject<'a> {
    let Some((entity, field)) = spec.outcomes.iter().find_map(|outcome| {
        let subject = outcome.subject.as_ref()?;
        match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, _) | (_, ResolvedInstance::Observed { .. }) => None,
            (_, ResolvedInstance::Supplied { field }) => Some((&subject.entity, &field.name)),
        }
    }) else {
        return Subject::None;
    };
    let entity: &ResolvedEntity = ir.entity(entity);
    let Some(key) = input.get(field) else {
        return Subject::Missing;
    };
    let Some((key, row)) = store
        .instances()
        .find(|(name, held, _)| **name == entity.name && *held == key)
        .map(|(_, key, row)| (key, row))
    else {
        return Subject::Missing;
    };
    Subject::Held(entity, key, row)
}

/// What each row-set branch's test decides, by branch: computed once, read only where the
/// precedence order reaches the branch, so a test that cannot be decided decides nothing earlier.
pub(super) fn decisions(
    reader: Option<&Reader<'_>>,
    spec: &ResolvedCommand,
) -> BTreeMap<OutcomeName, Result<Truth, String>> {
    let mut out = BTreeMap::new();
    let Some(reader) = reader else {
        return out;
    };
    for outcome in &spec.outcomes {
        if let ResolvedCondition::RelatedSet {
            selection, test, ..
        } = &outcome.condition
        {
            out.insert(
                outcome.name.clone(),
                reader
                    .decide(selection, test)
                    .map_err(|why| why.to_string()),
            );
        }
    }
    out
}
