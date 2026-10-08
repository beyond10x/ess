//! Where each branch of a command answers in the precedence order
//! (`docs/design/selection-plan.md`).
//!
//! The precedence order (`docs/design/cross-record-and-stored-field-guards.md`) is six steps;
//! [`Phase`] is those six and the two that sit outside them. A branch's phase is a function of its
//! own shape and of what the rest of its command declares, so it is decided here,
//! once, over data-free shapes: [`ConditionShape`] from an [`OutcomeCondition`], and in
//! `ess-compiler` from a `ResolvedCondition`, each by a match with no wildcard arm. [`order`] groups
//! a command's branches into phases, in the phase order [`with_phase_order`] may exchange for a
//! test; `ess_compiler::ir::PrecedencePlan` is that grouping over a resolved command.
//!
//! Nothing here is serialised, and nothing here is stored on a command: a plan is derived where it
//! is read.
use std::cell::Cell;
use std::fmt;

use super::{
    CommandSpec, Effect, Outcome, OutcomeCondition, PayloadSource, RelatedTest, RelatedVia,
};
use crate::system::FormatVersion;

/// A phase of the precedence order.
///
/// No `Ord`: the order phases are read in is [`phase_order`], which a test may exchange, and a
/// consumer comparing phases directly would read past it. [`Phase::position`] is the comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// A request with no input document (`input_absent:`), before any input field is read.
    InputAbsent,
    /// Step 1: on a command reading a related row or a row set, `existing_instance:`; then each
    /// `exists: false` over a row the input names, in declaration order.
    RelatedRow,
    /// Step 2: the input-guarded refusals, the first declared whose guard holds.
    InputRefusal,
    /// Step 3: existence — `existing_instance:` on any other command, then `unknown_instance:`.
    Existence,
    /// Step 4: the held state — `when_subject_state:`, `when_state_changes:`, `when_subject:` and
    /// `wrong_state:`.
    HeldState,
    /// Step 5: the present-related refusals the command's composition orders before every
    /// accepting branch, and `exists: false` over a stored reference.
    PresentRelated,
    /// Step 6: the accepting and external branches, in declaration order.
    Accepting,
    /// The default, where no guard and no provider answered.
    Default,
}

impl Phase {
    /// Every phase, in the order the precedence page states.
    pub const PRECEDENCE: [Self; 8] = [
        Self::InputAbsent,
        Self::RelatedRow,
        Self::InputRefusal,
        Self::Existence,
        Self::HeldState,
        Self::PresentRelated,
        Self::Accepting,
        Self::Default,
    ];

    /// The step of the precedence order this phase holds, `1..=6`; `None` for the two outside it.
    pub fn step(self) -> Option<u8> {
        match self {
            Self::InputAbsent | Self::Default => None,
            Self::RelatedRow => Some(1),
            Self::InputRefusal => Some(2),
            Self::Existence => Some(3),
            Self::HeldState => Some(4),
            Self::PresentRelated => Some(5),
            Self::Accepting => Some(6),
        }
    }

    /// The phase's name as a table prints it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InputAbsent => "input_absent",
            Self::RelatedRow => "related_row",
            Self::InputRefusal => "input_refusal",
            Self::Existence => "existence",
            Self::HeldState => "held_state",
            Self::PresentRelated => "present_related",
            Self::Accepting => "accepting",
            Self::Default => "default",
        }
    }

    /// Where this phase is read in the current [`phase_order`]: `0` first.
    pub fn position(self) -> usize {
        phase_order()
            .iter()
            .position(|phase| *phase == self)
            .expect("a phase order holds every phase")
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where a branch sits inside its phase.
///
/// The markers carry no guard and answer a phase's question rather than being read in order:
/// `existing_instance:` leads [`Phase::RelatedRow`] and [`Phase::Existence`], `unknown_instance:`
/// and `wrong_state:` close [`Phase::Existence`] and [`Phase::HeldState`]; on a row-set upsert
/// `unknown_instance:` closes [`Phase::PresentRelated`] instead. A stored `exists: false` leads
/// [`Phase::PresentRelated`]. A `when:` refusal step 2 does not hold comes next, in
/// [`Phase::HeldState`] or [`Phase::PresentRelated`] ([`place`]). Every other branch is read in
/// declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank {
    /// Before the phase's declared branches.
    Lead,
    /// A `when:` refusal step 2 does not hold, read where the interpreter's `select` begins: after
    /// the phase's lead and before its declared branches.
    Unguarded,
    /// Among the phase's branches, in declaration order.
    Declared,
    /// After the phase's declared branches.
    Trail,
}

/// A branch's phase and its rank within it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Place {
    /// The phase it answers in.
    pub phase: Phase,
    /// Where it sits in that phase.
    pub rank: Rank,
}

/// What a branch's condition contributes to its phase, with the condition's data left out.
///
/// One case per [`OutcomeCondition`] variant. `ess-compiler` maps `ResolvedCondition` into the
/// same shape, so the two crates classify by one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConditionShape {
    /// `when:` over the input; `trivially_true` where the guard always holds.
    When {
        /// The guard is [`Predicate::Always`](ess_primitives::predicate::Predicate::Always).
        trivially_true: bool,
    },
    /// `when_subject: {field, equals}`.
    SubjectField,
    /// `when_subject: {predicate}`.
    SubjectPredicate,
    /// `when_related:` over one row named by identity.
    Related {
        /// The row is named by a stored field of the addressed subject, not by the input.
        stored: bool,
        /// `exists: false`, rather than a predicate over a present row.
        absent: bool,
    },
    /// `when_related: {entity, where, …}`, a row set.
    RelatedSet,
    /// `when_subject_state:`.
    SubjectState,
    /// `when_state_changes:`.
    StateChange,
    /// `otherwise:`.
    Otherwise,
    /// `external:` beside `when:`.
    ExternalWhen,
    /// `external:`.
    External,
    /// `wrong_state:`.
    WrongState,
    /// `unknown_instance:`.
    UnknownInstance,
    /// `input_absent:`.
    InputAbsent,
    /// `existing_instance:`.
    ExistingInstance,
}

impl From<&OutcomeCondition> for ConditionShape {
    fn from(condition: &OutcomeCondition) -> Self {
        match condition {
            OutcomeCondition::When(predicate) => Self::When {
                trivially_true: predicate.is_trivially_true(),
            },
            OutcomeCondition::SubjectField { .. } => Self::SubjectField,
            OutcomeCondition::SubjectPredicate { .. } => Self::SubjectPredicate,
            OutcomeCondition::Related { via, test, .. } => Self::Related {
                stored: match via {
                    RelatedVia::Subject(_) => true,
                    RelatedVia::Input(_) => false,
                },
                absent: match test {
                    RelatedTest::Absent => true,
                    RelatedTest::Holds(_) => false,
                },
            },
            OutcomeCondition::RelatedSet { .. } => Self::RelatedSet,
            OutcomeCondition::SubjectState { .. } => Self::SubjectState,
            OutcomeCondition::StateChange { .. } => Self::StateChange,
            OutcomeCondition::Otherwise => Self::Otherwise,
            OutcomeCondition::ExternalWhen { .. } => Self::ExternalWhen,
            OutcomeCondition::External { .. } => Self::External,
            OutcomeCondition::WrongState => Self::WrongState,
            OutcomeCondition::UnknownInstance => Self::UnknownInstance,
            OutcomeCondition::InputAbsent => Self::InputAbsent,
            OutcomeCondition::ExistingInstance => Self::ExistingInstance,
        }
    }
}

/// One branch as the classification reads it: its condition's shape, and whether it names an
/// `error:`, a subject and `replays:`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BranchShape {
    /// Its condition.
    pub condition: ConditionShape,
    /// It names an `error:`.
    pub error: bool,
    /// It names a subject of its own.
    pub subject: bool,
    /// It replays a retained result.
    pub replays: bool,
}

impl BranchShape {
    /// The shape of a declared outcome.
    pub fn of(outcome: &Outcome) -> Self {
        Self {
            condition: ConditionShape::from(&outcome.condition),
            error: outcome.error.is_some(),
            subject: outcome.subject.is_some(),
            replays: outcome.replays.is_some(),
        }
    }

    /// A present-related predicate refusal or a row-set refusal: what step 5 may hold.
    fn present_related_refusal(self) -> bool {
        self.error
            && matches!(
                self.condition,
                ConditionShape::Related { absent: false, .. } | ConditionShape::RelatedSet
            )
    }
}

/// What the interpreter reads of a command's rows before `select` begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ReadFirst {
    /// Nothing beyond the input refusals and existence.
    Nothing,
    /// A stored reference: the addressed row's existence and the stored row
    /// (`stored_reference`).
    StoredReference,
    /// A row set: the addressed row's existence and held state (`addressed_row`).
    RowSet,
}

/// What the rest of a command declares that moves a branch between phases: the facts [`place`]
/// reads, decided once per command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Composition {
    /// A branch reads a related row, through the input or a stored field, or a row set: its
    /// `existing_instance:` answers at step 1.
    reads_related: bool,
    /// Its present-related refusals answer at step 5.
    orders_present_related_refusals: bool,
    /// What the interpreter answers for before `select` begins: on a stored reference or a row
    /// set, a `when:` refusal step 2 does not hold answers after it, at step 5.
    read_first: ReadFirst,
    /// A creation takes, from the input, the identity the command's acting branches address (an
    /// upsert, beyond10x/ess#462): on a row-set command an absent addressed row is left to the row
    /// sets, so `unknown_instance:` answers after them.
    upsert: bool,
}

impl Composition {
    /// The composition of a command whose branches are `branches`, reading `input_rows` distinct
    /// related rows through its input, in source format `format`, with no upsert
    /// ([`Self::with_upsert`]).
    ///
    /// Its present-related refusals answer at step 5, after the addressed row's existence and
    /// held state and before every accepting branch: on a row-set command; from `ess/22` beside
    /// `wrong_state:` (beyond10x/ess#282) or over several input rows (#283); and through a stored
    /// reference (#304).
    pub fn new(branches: &[BranchShape], input_rows: usize, format: FormatVersion) -> Self {
        let any = |test: fn(&BranchShape) -> bool| branches.iter().any(test);
        let row_set = any(|branch| matches!(branch.condition, ConditionShape::RelatedSet));
        let stored_reference = any(|branch| {
            matches!(
                branch.condition,
                ConditionShape::Related { stored: true, .. }
            )
        });
        let present_related_refusal = any(|branch| branch.present_related_refusal());
        let ordered = if row_set {
            present_related_refusal
        } else {
            format.major() >= FormatVersion::V22.major()
                && present_related_refusal
                && (input_rows > 1
                    || any(|branch| matches!(branch.condition, ConditionShape::WrongState)))
        };
        Self {
            reads_related: any(|branch| {
                matches!(
                    branch.condition,
                    ConditionShape::Related { .. } | ConditionShape::RelatedSet
                )
            }),
            orders_present_related_refusals: ordered || stored_reference,
            read_first: if row_set {
                ReadFirst::RowSet
            } else if stored_reference {
                ReadFirst::StoredReference
            } else {
                ReadFirst::Nothing
            },
            upsert: false,
        }
    }

    /// This composition, where `upsert` says whether a creation takes, from the input, the identity
    /// the command's acting branches address: the first branch naming no `error:` that acts on a
    /// supplied instance addresses it, and a creation of the same entity takes it where its new
    /// identity is that input field (`input.<field>`, or `{input: <field>, else: …}`). The
    /// interpreter's `creation_takes_absent` (beyond10x/ess#462).
    #[must_use]
    pub fn with_upsert(self, upsert: bool) -> Self {
        Self { upsert, ..self }
    }

    /// The composition of a declared command in source format `format`.
    pub fn of(command: &CommandSpec, format: FormatVersion) -> Self {
        let branches: Vec<BranchShape> = command.outcomes.iter().map(BranchShape::of).collect();
        let mut rows: Vec<&str> = command
            .outcomes
            .iter()
            .filter_map(|outcome| match &outcome.condition {
                OutcomeCondition::Related {
                    via: RelatedVia::Input(field),
                    ..
                } => Some(field.as_str()),
                _ => None,
            })
            .collect();
        rows.sort_unstable();
        rows.dedup();
        Self::new(&branches, rows.len(), format).with_upsert(upsert(command))
    }

    /// Whether the command's present-related refusals answer at step 5 ([`Self::new`]).
    pub fn orders_present_related_refusals(&self) -> bool {
        self.orders_present_related_refusals
    }
}

/// [`Composition::with_upsert`] read off a declared command. A creation's new identity is the
/// source its payload gives the event field `instance:` names, read from the first emitted event
/// whose payload fills it; `ess-compiler` reads the event the compiler resolved, the first emitted
/// one declaring that field at the identity's type.
fn upsert(command: &CommandSpec) -> bool {
    let Some(addressed) = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_none())
        .find_map(|outcome| {
            outcome
                .subject
                .as_ref()
                .filter(|subject| subject.effect != Effect::Creates)
        })
    else {
        return false;
    };
    command.outcomes.iter().any(|outcome| {
        let Some(subject) = outcome.subject.as_ref().filter(|subject| {
            subject.effect == Effect::Creates && subject.entity == addressed.entity
        }) else {
            return false;
        };
        outcome.error.is_none()
            && outcome
                .emits
                .iter()
                .find_map(|event| outcome.payload.get(event)?.get(&subject.instance))
                .is_some_and(|source| match source {
                    PayloadSource::InputField { field }
                    | PayloadSource::InputOrGenerated { field, .. } => *field == addressed.instance,
                    _ => false,
                })
    })
}

/// The phase and rank `branch` answers at on a command composed as `command`.
///
/// A match over [`ConditionShape`] with no wildcard arm: a new condition is placed here before
/// anything compiles.
pub fn place(branch: &BranchShape, command: &Composition) -> Place {
    let at = |phase, rank| Place { phase, rank };
    match branch.condition {
        ConditionShape::InputAbsent => at(Phase::InputAbsent, Rank::Declared),
        ConditionShape::ExistingInstance => {
            if command.reads_related {
                at(Phase::RelatedRow, Rank::Lead)
            } else {
                at(Phase::Existence, Rank::Lead)
            }
        }
        ConditionShape::Related {
            stored: false,
            absent: true,
        } => at(Phase::RelatedRow, Rank::Declared),
        ConditionShape::Related {
            stored: true,
            absent: true,
        } => at(Phase::PresentRelated, Rank::Lead),
        ConditionShape::When { trivially_true } => {
            if !branch.error {
                at(Phase::Accepting, Rank::Declared)
            } else if trivially_true || branch.subject || branch.replays {
                // Not an input-guarded refusal: the interpreter's `refused_by_input` declines it
                // and the head of `select` takes it. That is after existence and before every
                // branch the held state selects; on a command whose stored row or row set is read
                // first (`stored_reference`, `addressed_row`), also after the addressed row's held
                // state and a stored row's `exists: false`, and before the present-related
                // refusals (beyond10x/ess#470 adversary pass 1, C1 and C2).
                if command.read_first == ReadFirst::Nothing {
                    at(Phase::HeldState, Rank::Unguarded)
                } else {
                    at(Phase::PresentRelated, Rank::Unguarded)
                }
            } else {
                at(Phase::InputRefusal, Rank::Declared)
            }
        }
        ConditionShape::UnknownInstance => {
            // On a row-set upsert the interpreter's `addressed_row` and `selected_subject_refusal`
            // leave an absent addressed row to the row sets (`creation_takes_absent`), and `take`
            // answers it for the branch selected after them (beyond10x/ess#470 adversary pass 2,
            // R1).
            if command.read_first == ReadFirst::RowSet && command.upsert {
                at(Phase::PresentRelated, Rank::Trail)
            } else {
                at(Phase::Existence, Rank::Trail)
            }
        }
        ConditionShape::SubjectState
        | ConditionShape::StateChange
        | ConditionShape::SubjectField
        | ConditionShape::SubjectPredicate => at(Phase::HeldState, Rank::Declared),
        ConditionShape::WrongState => at(Phase::HeldState, Rank::Trail),
        ConditionShape::Related { absent: false, .. } | ConditionShape::RelatedSet => {
            if branch.error && command.orders_present_related_refusals() {
                at(Phase::PresentRelated, Rank::Declared)
            } else {
                at(Phase::Accepting, Rank::Declared)
            }
        }
        ConditionShape::External | ConditionShape::ExternalWhen => {
            at(Phase::Accepting, Rank::Declared)
        }
        ConditionShape::Otherwise => at(Phase::Default, Rank::Declared),
    }
}

/// `branches`, by index, grouped into every phase in the current [`phase_order`], each phase in
/// its order: by [`Rank`], then by declaration. A phase no branch answers in is present and empty.
pub fn order(branches: &[BranchShape], command: &Composition) -> Vec<(Phase, Vec<usize>)> {
    let places: Vec<Place> = branches
        .iter()
        .map(|branch| place(branch, command))
        .collect();
    phase_order()
        .into_iter()
        .map(|phase| {
            let mut held: Vec<usize> = (0..branches.len())
                .filter(|index| places[*index].phase == phase)
                .collect();
            held.sort_by_key(|index| (places[*index].rank, *index));
            (phase, held)
        })
        .collect()
}

std::thread_local! {
    /// The phase order [`with_phase_order`] set on this thread, where one is set.
    static PHASE_ORDER: Cell<Option<[Phase; 8]>> = const { Cell::new(None) };
}

/// The order phases are read in on this thread: [`Phase::PRECEDENCE`], unless a test exchanged it
/// with [`with_phase_order`].
pub fn phase_order() -> [Phase; 8] {
    PHASE_ORDER.with(Cell::get).unwrap_or(Phase::PRECEDENCE)
}

/// Runs `run` with the phases read in `order` on this thread, restoring the order before it
/// afterwards, also where `run` unwinds.
///
/// A test seam (`docs/design/selection-plan.md`, "One test seam"): every consumer builds its plan
/// inside its own entry point, so an order the classification and every plan constructor read is
/// the one place a "phases exchanged" test reaches them all.
///
/// # Panics
///
/// Where `order` does not name every phase exactly once.
#[doc(hidden)]
pub fn with_phase_order<T>(order: [Phase; 8], run: impl FnOnce() -> T) -> T {
    struct Restore(Option<[Phase; 8]>);
    impl Drop for Restore {
        fn drop(&mut self) {
            PHASE_ORDER.with(|cell| cell.set(self.0));
        }
    }
    assert!(
        Phase::PRECEDENCE
            .iter()
            .all(|phase| order.iter().filter(|held| *held == phase).count() == 1),
        "a phase order names every phase exactly once: {order:?}"
    );
    let _restore = Restore(PHASE_ORDER.with(|cell| cell.replace(Some(order))));
    run()
}
