//! A resolved command's precedence plan (`docs/design/selection-plan.md`): its branches grouped
//! into the phases of the precedence order, each phase ordered.
//!
//! The classification is `ess-domain`'s ([`ess_domain::command::precedence`]); this module maps
//! [`ResolvedCondition`] into its [`ConditionShape`] and orders the command's outcomes by it. The
//! plan is derived where it is read and is never serialised: no type here implements `Serialize`,
//! and no IR type holds one, so [`EssIr::to_canonical_json`](super::EssIr::to_canonical_json) keeps
//! its bytes.
use ess_domain::command::precedence::{self, BranchShape, Composition, ConditionShape, Phase};
use ess_domain::command::OutcomeName;
use ess_domain::system::FormatVersion;

use super::{
    ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedInstance, ResolvedOutcome,
    ResolvedPayloadField, ResolvedPayloadValue, ResolvedRelatedTest, ResolvedRelatedVia,
};

/// One phase of a [`PrecedencePlan`] and the branches it holds, in the order they are read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPhase<'c> {
    /// The phase.
    pub phase: Phase,
    /// Its branches, in order; empty where the command declares none here.
    pub branches: Vec<&'c ResolvedOutcome>,
}

/// A command's branches grouped into every phase of the precedence order, in the current
/// [`phase_order`](precedence::phase_order), each phase ordered.
///
/// Built on demand by [`PrecedencePlan::new`] and never stored or serialised: a command a consumer
/// clones and trims has the plan of what it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecedencePlan<'c> {
    phases: Vec<PlannedPhase<'c>>,
}

impl<'c> PrecedencePlan<'c> {
    /// The plan of `command`, declared in source format `format`, read in the current phase order
    /// ([`with_phase_order`](precedence::with_phase_order) may exchange it for a test).
    pub fn new(command: &'c ResolvedCommand, format: FormatVersion) -> Self {
        let shapes: Vec<BranchShape> = command.outcomes.iter().map(branch).collect();
        let mut rows: Vec<&str> = command
            .outcomes
            .iter()
            .filter_map(|outcome| match &outcome.condition {
                ResolvedCondition::Related {
                    via: ResolvedRelatedVia::Input { field, .. },
                    ..
                } => Some(field.as_str()),
                _ => None,
            })
            .collect();
        rows.sort_unstable();
        rows.dedup();
        let composition =
            Composition::new(&shapes, rows.len(), format).with_upsert(upsert(command));
        let phases = precedence::order(&shapes, &composition)
            .into_iter()
            .map(|(phase, indices)| PlannedPhase {
                phase,
                branches: indices
                    .into_iter()
                    .map(|index| &command.outcomes[index])
                    .collect(),
            })
            .collect();
        Self { phases }
    }

    /// Every phase, in the order it is read, empty ones included.
    pub fn phases(&self) -> &[PlannedPhase<'c>] {
        &self.phases
    }

    /// The branches `phase` holds, in order.
    pub fn branches(&self, phase: Phase) -> &[&'c ResolvedOutcome] {
        self.phases
            .iter()
            .find(|planned| planned.phase == phase)
            .map_or(&[], |planned| planned.branches.as_slice())
    }

    /// The phase the branch named `outcome` answers in; `None` for a name the command does not
    /// declare.
    pub fn phase_of(&self, outcome: &OutcomeName) -> Option<Phase> {
        self.phases.iter().find_map(|planned| {
            planned
                .branches
                .iter()
                .any(|branch| branch.name == *outcome)
                .then_some(planned.phase)
        })
    }

    /// Every branch with its phase, in the order the plan reads them.
    pub fn iter(&self) -> impl Iterator<Item = (Phase, &'c ResolvedOutcome)> + '_ {
        self.phases.iter().flat_map(|planned| {
            planned
                .branches
                .iter()
                .map(move |branch| (planned.phase, *branch))
        })
    }
}

/// Whether a creation takes, from the input, the identity the command's acting branches address
/// ([`Composition::with_upsert`]): the interpreter's `creation_takes_absent` over the row
/// `addressed_row` reads (beyond10x/ess#462).
fn upsert(command: &ResolvedCommand) -> bool {
    let Some((entity, field)) = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_none())
        .find_map(|outcome| {
            let subject = outcome.subject.as_ref()?;
            match (&subject.effect, &subject.instance) {
                (ResolvedEffect::Creates, _) | (_, ResolvedInstance::Observed { .. }) => None,
                (_, ResolvedInstance::Supplied { field }) => {
                    Some((&subject.entity, field.name.as_str()))
                }
            }
        })
    else {
        return false;
    };
    command.outcomes.iter().any(|outcome| {
        outcome.error.is_none()
            && outcome
                .subject
                .as_ref()
                .is_some_and(|subject| subject.entity == *entity)
            && identity_source(outcome).is_some_and(|source| {
                matches!(&source.value,
                    ResolvedPayloadValue::InputField { field: taken, .. }
                    | ResolvedPayloadValue::InputOrGenerated { field: taken, .. }
                    if taken == field)
            })
    })
}

/// The payload field a creation takes its new identity from: the field of the event its identity
/// is observed in.
fn identity_source(outcome: &ResolvedOutcome) -> Option<&ResolvedPayloadField> {
    let subject = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == ResolvedEffect::Creates)?;
    let ResolvedInstance::Observed { event, field } = &subject.instance else {
        return None;
    };
    outcome
        .payload
        .iter()
        .filter(|payload| &payload.event == event)
        .flat_map(|payload| &payload.fields)
        .find(|source| source.target == field.name)
}

/// The shape the classification reads of one resolved outcome.
fn branch(outcome: &ResolvedOutcome) -> BranchShape {
    BranchShape {
        condition: shape(&outcome.condition),
        error: outcome.error.is_some(),
        subject: outcome.subject.is_some(),
        replays: outcome.replays.is_some(),
    }
}

/// The shape of a resolved condition: a match with no wildcard arm, so a new condition fails to
/// compile until it is placed.
fn shape(condition: &ResolvedCondition) -> ConditionShape {
    match condition {
        ResolvedCondition::When { predicate } => ConditionShape::When {
            trivially_true: predicate.is_trivially_true(),
        },
        ResolvedCondition::SubjectField { .. } => ConditionShape::SubjectField,
        ResolvedCondition::SubjectPredicate { .. } => ConditionShape::SubjectPredicate,
        ResolvedCondition::Related { via, test, .. } => ConditionShape::Related {
            stored: match via {
                ResolvedRelatedVia::Subject { .. } => true,
                ResolvedRelatedVia::Input { .. } => false,
            },
            absent: match test {
                ResolvedRelatedTest::Absent => true,
                ResolvedRelatedTest::Holds { .. } => false,
            },
        },
        ResolvedCondition::RelatedSet { .. } => ConditionShape::RelatedSet,
        ResolvedCondition::SubjectState { .. } => ConditionShape::SubjectState,
        ResolvedCondition::StateChange { .. } => ConditionShape::StateChange,
        ResolvedCondition::Otherwise => ConditionShape::Otherwise,
        ResolvedCondition::ExternalWhen { .. } => ConditionShape::ExternalWhen,
        ResolvedCondition::External { .. } => ConditionShape::External,
        ResolvedCondition::WrongState => ConditionShape::WrongState,
        ResolvedCondition::UnknownInstance => ConditionShape::UnknownInstance,
        ResolvedCondition::InputAbsent => ConditionShape::InputAbsent,
        ResolvedCondition::ExistingInstance => ConditionShape::ExistingInstance,
    }
}
