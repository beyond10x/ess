//! The precedence classification (`docs/design/selection-plan.md`) is reachable from `ess-domain`:
//! one branch of every `OutcomeCondition` is placed, the compositions the page names move a branch
//! between phases, and the classification follows a phase order exchanged inside the test seam.
use ess_domain::command::precedence::{
    order, phase_order, place, with_phase_order, BranchShape, Composition, ConditionShape, Phase,
    Rank, ONE_RULE,
};
use ess_domain::command::row_set::{RowSelection, RowSetTest};
use ess_domain::command::{Outcome, OutcomeCondition, OutcomeName, RelatedTest, RelatedVia};
use ess_domain::entity::{HeldStates, StateName};
use ess_domain::system::{FormatVersion, Source};
use ess_domain::{spec::RawSpecFile, Specification};
use ess_primitives::predicate::Predicate;

const MULTIPLE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml");
const ROTATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml");
const UPSERT_UNKNOWN: &str =
    include_str!("../../ess-compiler/tests/fixtures/precedence-upsert-unknown-instance.yaml");

fn predicate(text: &str) -> Predicate {
    text.parse()
        .unwrap_or_else(|error| panic!("{text}: {error:?}"))
}

fn outcome(name: &str, condition: OutcomeCondition, refuses: bool) -> Outcome {
    let mut outcome = Outcome::when(
        OutcomeName::new(name).unwrap(),
        Predicate::Always,
        Vec::new(),
    );
    outcome.condition = condition;
    if refuses {
        outcome.error = Some("demo.desk.Refused".parse().unwrap());
    }
    outcome
}

/// One condition of every variant, each as the refusal or the accepting branch it is written as.
fn every_condition() -> Vec<(OutcomeCondition, bool)> {
    let state = || HeldStates::One(StateName::new("Open").unwrap());
    vec![
        (OutcomeCondition::When(predicate("amount > 10")), true),
        (
            OutcomeCondition::SubjectField {
                field: "kind".into(),
                equals: "Express".into(),
                predicate: None,
            },
            true,
        ),
        (
            OutcomeCondition::SubjectPredicate {
                predicate: predicate("weight > 20"),
                input: None,
            },
            true,
        ),
        (
            OutcomeCondition::Related {
                via: RelatedVia::Input("tenant".into()),
                test: RelatedTest::Absent,
                input: None,
                also: Vec::new(),
            },
            true,
        ),
        (
            OutcomeCondition::RelatedSet {
                selection: RowSelection {
                    entity: "demo.desk.Lease".parse().unwrap(),
                    filter: predicate("desk == input.desk"),
                },
                test: RowSetTest::Exists(true),
                input: None,
            },
            true,
        ),
        (
            OutcomeCondition::SubjectState {
                state: state(),
                predicate: None,
            },
            true,
        ),
        (
            OutcomeCondition::StateChange {
                changes: false,
                predicate: None,
            },
            false,
        ),
        (OutcomeCondition::Otherwise, false),
        (
            OutcomeCondition::ExternalWhen {
                cause: "the provider refuses".into(),
                predicate: predicate("amount > 10"),
            },
            true,
        ),
        (
            OutcomeCondition::External {
                cause: "the provider refuses".into(),
            },
            true,
        ),
        (
            OutcomeCondition::ExternalSubject {
                cause: "the provider refuses".into(),
                predicate: predicate("weight > 20"),
                input: None,
            },
            true,
        ),
        (OutcomeCondition::WrongState, true),
        (OutcomeCondition::UnknownInstance, true),
        (OutcomeCondition::InputAbsent, true),
        (OutcomeCondition::ExistingInstance, true),
    ]
}

/// The phase `docs/design/selection-plan.md` gives `condition` on a command declaring it alone at
/// `ess/23`. A match with no wildcard arm: a new variant fails to compile here until it is placed.
fn expected(condition: &OutcomeCondition) -> (Phase, Rank) {
    match condition {
        OutcomeCondition::When(_) => (Phase::InputRefusal, Rank::Declared),
        OutcomeCondition::SubjectField { .. }
        | OutcomeCondition::SubjectPredicate { .. }
        | OutcomeCondition::ExternalSubject { .. }
        | OutcomeCondition::SubjectState { .. }
        | OutcomeCondition::StateChange { .. } => (Phase::HeldState, Rank::Declared),
        OutcomeCondition::Related { .. } => (Phase::RelatedRow, Rank::Declared),
        OutcomeCondition::RelatedSet { .. } => (Phase::PresentRelated, Rank::Declared),
        OutcomeCondition::Otherwise => (Phase::Default, Rank::Declared),
        OutcomeCondition::ExternalWhen { .. } | OutcomeCondition::External { .. } => {
            (Phase::Accepting, Rank::Declared)
        }
        OutcomeCondition::WrongState => (Phase::HeldState, Rank::Trail),
        OutcomeCondition::UnknownInstance => (Phase::Existence, Rank::Trail),
        OutcomeCondition::InputAbsent => (Phase::InputAbsent, Rank::Declared),
        OutcomeCondition::ExistingInstance => (Phase::Existence, Rank::Lead),
    }
}

fn alone(branch: BranchShape) -> Composition {
    Composition::new(&[branch], 0, FormatVersion::V23)
}

#[test]
fn one_branch_of_every_condition_is_placed() {
    let conditions = every_condition();
    assert_eq!(conditions.len(), 15, "one of every variant");
    for (condition, refuses) in conditions {
        let branch = BranchShape::of(&outcome("branch", condition.clone(), refuses));
        let placed = place(&branch, &alone(branch));
        assert_eq!(
            (placed.phase, placed.rank),
            expected(&condition),
            "{condition:?}"
        );
    }
}

#[test]
fn a_branch_shape_carries_the_facts_its_phase_depends_on() {
    let mut refusal = outcome("late", OutcomeCondition::When(Predicate::Always), true);
    assert_eq!(
        BranchShape::of(&refusal),
        BranchShape {
            condition: ConditionShape::When {
                trivially_true: true
            },
            error: true,
            subject: false,
            replays: false,
            before_row: false,
        }
    );
    refusal.replays = Some(OutcomeName::new("created").unwrap());
    assert!(BranchShape::of(&refusal).replays);
    assert_eq!(
        ConditionShape::from(&OutcomeCondition::Related {
            via: RelatedVia::Subject("assignee".into()),
            test: RelatedTest::Holds(predicate("active == false")),
            input: None,
            also: Vec::new(),
        }),
        ConditionShape::Related {
            stored: true,
            absent: false
        }
    );
}

fn shape(condition: ConditionShape, error: bool) -> BranchShape {
    BranchShape {
        condition,
        error,
        subject: false,
        replays: false,
        before_row: false,
    }
}

const INPUT_ABSENT: ConditionShape = ConditionShape::Related {
    stored: false,
    absent: true,
};
const INPUT_HOLDS: ConditionShape = ConditionShape::Related {
    stored: false,
    absent: false,
};
const STORED_ABSENT: ConditionShape = ConditionShape::Related {
    stored: true,
    absent: true,
};
const STORED_HOLDS: ConditionShape = ConditionShape::Related {
    stored: true,
    absent: false,
};

fn placed(
    branch: BranchShape,
    command: &[BranchShape],
    rows: usize,
    format: FormatVersion,
) -> Phase {
    place(&branch, &Composition::new(command, rows, format)).phase
}

#[test]
fn existing_instance_answers_first_on_a_command_reading_a_related_row_or_a_row_set() {
    let existing = shape(ConditionShape::ExistingInstance, true);
    let accepting = shape(
        ConditionShape::When {
            trivially_true: true,
        },
        false,
    );
    for beside in [INPUT_ABSENT, STORED_HOLDS, ConditionShape::RelatedSet] {
        let command = [existing, shape(beside, true), accepting];
        let placed = place(
            &existing,
            &Composition::new(&command, 1, FormatVersion::V22),
        );
        assert_eq!(
            (placed.phase, placed.rank),
            (Phase::RelatedRow, Rank::Lead),
            "{beside:?}"
        );
    }
    assert_eq!(
        placed(existing, &[existing, accepting], 0, FormatVersion::V22),
        Phase::Existence
    );
}

#[test]
fn a_missing_row_answers_at_step_one_through_the_input_and_step_five_through_a_stored_field() {
    let accepting = shape(
        ConditionShape::When {
            trivially_true: true,
        },
        false,
    );
    let input = shape(INPUT_ABSENT, true);
    assert_eq!(
        placed(input, &[input, accepting], 1, FormatVersion::V18),
        Phase::RelatedRow
    );
    let stored = shape(STORED_ABSENT, true);
    let at = place(
        &stored,
        &Composition::new(&[stored, accepting], 0, FormatVersion::V22),
    );
    assert_eq!((at.phase, at.rank), (Phase::PresentRelated, Rank::Lead));
}

#[test]
fn a_present_related_refusal_answers_at_step_five_only_where_the_composition_orders_it() {
    let refusal = shape(INPUT_HOLDS, true);
    let accepting = shape(INPUT_HOLDS, false);
    let wrong_state = shape(ConditionShape::WrongState, true);
    let plain = shape(
        ConditionShape::When {
            trivially_true: true,
        },
        false,
    );
    // ess/22 beside `wrong_state` (#282), and over several input rows (#283).
    let command = [refusal, wrong_state, plain];
    assert_eq!(
        placed(refusal, &command, 1, FormatVersion::V22),
        Phase::PresentRelated
    );
    assert_eq!(
        placed(refusal, &[refusal, plain], 2, FormatVersion::V22),
        Phase::PresentRelated
    );
    // Below ess/22, and on one row without `wrong_state`, declaration order with the accepting
    // branches.
    assert_eq!(
        placed(refusal, &command, 1, FormatVersion::V21),
        Phase::Accepting
    );
    assert_eq!(
        placed(refusal, &[refusal, plain], 2, FormatVersion::V21),
        Phase::Accepting
    );
    assert_eq!(
        placed(refusal, &[refusal, plain], 1, FormatVersion::V22),
        Phase::Accepting
    );
    // A stored reference orders it whatever else is declared (#304).
    let stored = shape(STORED_HOLDS, true);
    assert_eq!(
        placed(stored, &[stored, plain], 0, FormatVersion::V22),
        Phase::PresentRelated
    );
    // An accepting related branch is never a refusal.
    assert_eq!(
        placed(
            accepting,
            &[refusal, accepting, wrong_state],
            1,
            FormatVersion::V22
        ),
        Phase::Accepting
    );
    // A row-set refusal is the composition; an accepting row set reads in declaration order.
    let rows = shape(ConditionShape::RelatedSet, true);
    assert_eq!(
        placed(rows, &[rows, plain], 0, FormatVersion::V22),
        Phase::PresentRelated
    );
    let accepting_rows = shape(ConditionShape::RelatedSet, false);
    assert_eq!(
        placed(
            accepting_rows,
            &[rows, accepting_rows],
            0,
            FormatVersion::V22
        ),
        Phase::Accepting
    );
}

/// A `when:` + `error:` branch `refused_by_input` declines — a guard that always holds, or a subject
/// or `replays:` validation refuses beside an `error:` — answers where the interpreter's `select`
/// begins: first among the held-state branches, or, on a command whose stored row or row set the
/// interpreter answers before `select` (beyond10x/ess#470 adversary pass 1, C1 and C2), after a
/// stored row's `exists: false` and before the present-related refusals.
#[test]
fn a_when_refusal_refused_by_input_declines_answers_where_select_begins() {
    let plain = shape(
        ConditionShape::When {
            trivially_true: true,
        },
        false,
    );
    let late_shapes = [
        shape(
            ConditionShape::When {
                trivially_true: true,
            },
            true,
        ),
        BranchShape {
            subject: true,
            ..shape(
                ConditionShape::When {
                    trivially_true: false,
                },
                true,
            )
        },
        BranchShape {
            replays: true,
            ..shape(
                ConditionShape::When {
                    trivially_true: false,
                },
                true,
            )
        },
    ];
    let wrong_state = shape(ConditionShape::WrongState, true);
    for late in late_shapes {
        for (beside, rows, phase) in [
            (plain, 0, Phase::HeldState),
            (wrong_state, 0, Phase::HeldState),
            (shape(INPUT_HOLDS, true), 2, Phase::HeldState),
            (shape(STORED_HOLDS, true), 0, Phase::PresentRelated),
            (shape(STORED_ABSENT, true), 0, Phase::PresentRelated),
            (
                shape(ConditionShape::RelatedSet, true),
                0,
                Phase::PresentRelated,
            ),
            (
                shape(ConditionShape::RelatedSet, false),
                0,
                Phase::PresentRelated,
            ),
        ] {
            let at = place(
                &late,
                &Composition::new(
                    &[late, beside, wrong_state, plain],
                    rows,
                    FormatVersion::V23,
                ),
            );
            assert_eq!(
                (at.phase, at.rank),
                (phase, Rank::Unguarded),
                "{late:?} beside {beside:?}"
            );
        }
    }
    let input = shape(
        ConditionShape::When {
            trivially_true: false,
        },
        true,
    );
    assert_eq!(
        placed(input, &[input, plain], 0, FormatVersion::V23),
        Phase::InputRefusal
    );
}

fn stored_related(test: RelatedTest) -> OutcomeCondition {
    OutcomeCondition::Related {
        via: RelatedVia::Subject("blocked_by".into()),
        test,
        input: None,
        also: Vec::new(),
    }
}

#[test]
fn an_unguarded_refusal_answers_after_the_rows_the_interpreter_reads_before_select() {
    let declined = || outcome("declined", OutcomeCondition::When(Predicate::Always), true);
    let wrong_state = || outcome("wrong-state", OutcomeCondition::WrongState, true);
    // C1: a stored row's `exists: false` is answered in `stored_reference`, before `select`.
    let stored = vec![
        outcome(
            "blocked",
            stored_related(RelatedTest::Holds(predicate("state != Done"))),
            true,
        ),
        declined(),
        outcome("blocker-missing", stored_related(RelatedTest::Absent), true),
        wrong_state(),
        outcome(
            "completed",
            OutcomeCondition::When(predicate("force == true")),
            false,
        ),
    ];
    assert_eq!(
        named(&stored, FormatVersion::V22),
        expect(&[
            (Phase::HeldState, &["wrong-state"]),
            (
                Phase::PresentRelated,
                &["blocker-missing", "declined", "blocked"]
            ),
            (Phase::Accepting, &["completed"]),
        ])
    );
    // C2: a row-set command's held state is answered in `addressed_row`, before `select`.
    let row_set = vec![
        outcome("unknown-attempt", OutcomeCondition::UnknownInstance, true),
        wrong_state(),
        outcome(
            "crowded",
            OutcomeCondition::RelatedSet {
                selection: RowSelection {
                    entity: "demo.jobs.Attempt".parse().unwrap(),
                    filter: predicate("worker_id == subject.worker_id"),
                },
                test: RowSetTest::Exists(true),
                input: None,
            },
            true,
        ),
        outcome(
            "closed",
            OutcomeCondition::When(predicate("confirm == true")),
            false,
        ),
        declined(),
    ];
    assert_eq!(
        named(&row_set, FormatVersion::V22),
        expect(&[
            (Phase::Existence, &["unknown-attempt"]),
            (Phase::HeldState, &["wrong-state"]),
            (Phase::PresentRelated, &["declined", "crowded"]),
            (Phase::Accepting, &["closed"]),
        ])
    );
    // Anywhere else, `select` begins before the held-state guards and `wrong_state:`.
    let held = vec![
        wrong_state(),
        outcome(
            "held",
            OutcomeCondition::SubjectState {
                state: HeldStates::One(StateName::new("Open").unwrap()),
                predicate: None,
            },
            true,
        ),
        declined(),
    ];
    assert_eq!(
        named(&held, FormatVersion::V22),
        expect(&[(Phase::HeldState, &["declined", "held", "wrong-state"])])
    );
}

/// The phases of `order`, with the branch names each holds, skipping empty phases.
fn named(command: &[Outcome], format: FormatVersion) -> Vec<(Phase, Vec<String>)> {
    let shapes: Vec<BranchShape> = command.iter().map(BranchShape::of).collect();
    let rows = command
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            OutcomeCondition::Related {
                via: RelatedVia::Input(field),
                ..
            } => Some(field.as_str()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    order(&shapes, &Composition::new(&shapes, rows, format))
        .into_iter()
        .filter(|(_, branches)| !branches.is_empty())
        .map(|(phase, branches)| {
            (
                phase,
                branches
                    .into_iter()
                    .map(|index| command[index].name.to_string())
                    .collect(),
            )
        })
        .collect()
}

/// The non-empty phases of `command` in the model `text`, by `Composition::of`, which must read the
/// command as its branch shapes, its input rows and `upsert` (a creation taking the identity its
/// acting branches address) say.
fn phases(text: &str, command: &str, upsert: bool) -> Vec<(Phase, Vec<String>)> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&command.parse().unwrap()];
    let format = spec.system().format;
    let shapes: Vec<BranchShape> = command.outcomes.iter().map(BranchShape::of).collect();
    let composition = Composition::of(command, format);
    let expected = Composition::new(&shapes, named_rows(command), format).with_upsert(upsert);
    assert_eq!(
        composition, expected,
        "Composition::of reads the command as its branch shapes, rows and upsert do"
    );
    let by_command = order(&shapes, &composition);
    assert_eq!(
        by_command,
        order(&shapes, &expected),
        "Composition::of reads the command as its branch shapes do"
    );
    by_command
        .into_iter()
        .filter(|(_, branches)| !branches.is_empty())
        .map(|(phase, branches)| {
            (
                phase,
                branches
                    .into_iter()
                    .map(|index| command.outcomes[index].name.to_string())
                    .collect(),
            )
        })
        .collect()
}

fn named_rows(command: &ess_domain::command::CommandSpec) -> usize {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            OutcomeCondition::Related {
                via: RelatedVia::Input(field),
                ..
            } => Some(field.as_str()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

fn expect(phases: &[(Phase, &[&str])]) -> Vec<(Phase, Vec<String>)> {
    phases
        .iter()
        .map(|(phase, names)| (*phase, names.iter().map(ToString::to_string).collect()))
        .collect()
}

#[test]
fn several_input_rows_answer_missing_rows_first_and_their_refusals_at_step_five() {
    assert_eq!(
        phases(MULTIPLE, "demo.run.StartRun", false),
        expect(&[
            (Phase::RelatedRow, &["no-such-switch", "no-such-capability"]),
            (
                Phase::PresentRelated,
                &["switch-paused", "capability-revoked"]
            ),
            (Phase::Default, &["started"]),
        ])
    );
}

#[test]
fn the_phases_come_in_the_precedence_order_and_markers_keep_their_rank() {
    let command = vec![
        outcome(
            "accepted",
            OutcomeCondition::When(predicate("amount > 10")),
            false,
        ),
        outcome("wrong-state", OutcomeCondition::WrongState, true),
        outcome(
            "held",
            OutcomeCondition::SubjectState {
                state: HeldStates::One(StateName::new("Open").unwrap()),
                predicate: None,
            },
            true,
        ),
        outcome(
            "too-small",
            OutcomeCondition::When(predicate("amount < 1")),
            true,
        ),
        outcome("unknown", OutcomeCondition::UnknownInstance, true),
        outcome("rest", OutcomeCondition::Otherwise, false),
        outcome("late", OutcomeCondition::When(Predicate::Always), true),
        outcome("absent", OutcomeCondition::InputAbsent, true),
    ];
    assert_eq!(
        named(&command, FormatVersion::V23),
        expect(&[
            (Phase::InputAbsent, &["absent"]),
            (Phase::InputRefusal, &["too-small"]),
            (Phase::Existence, &["unknown"]),
            (Phase::HeldState, &["late", "held", "wrong-state"]),
            (Phase::Accepting, &["accepted"]),
            (Phase::Default, &["rest"]),
        ])
    );
    let every: Vec<Phase> = order(&[], &Composition::new(&[], 0, FormatVersion::V23))
        .into_iter()
        .map(|(phase, _)| phase)
        .collect();
    assert_eq!(
        every,
        Phase::PRECEDENCE,
        "every phase, in the precedence order"
    );
    assert_eq!(
        Phase::PRECEDENCE
            .iter()
            .filter_map(|phase| phase.step())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6],
        "six phases are the six steps"
    );
}

#[test]
fn an_input_refusal_beside_a_held_state_branch_answers_before_it() {
    assert_eq!(
        phases(ROTATE, "demo.secrets.RotateSecret", false),
        expect(&[
            (Phase::InputRefusal, &["too-short"]),
            (Phase::HeldState, &["rotated"]),
            (Phase::Default, &["not-configured"]),
        ])
    );
}

/// An external refusal marked `before_row: true` answers at step 2, among the input refusals, before
/// existence and the held state (ess/25, beyond10x/ess#534); without the key, or on an accepting
/// external branch, it stays at step 6.
#[test]
fn a_before_row_external_refusal_answers_at_step_two() {
    let sign_in = include_str!("../../ess-compiler/tests/fixtures/external-before-row.yaml");
    assert_eq!(
        phases(sign_in, "demo.desk.SignIn", true),
        expect(&[
            (Phase::InputRefusal, &["bad-credential"]),
            (Phase::Existence, &["first"]),
            (Phase::HeldState, &["reopened-busy", "reopened", "opened"]),
        ])
    );
    for condition in [ConditionShape::External, ConditionShape::ExternalWhen] {
        let marked = BranchShape {
            before_row: true,
            ..shape(condition, true)
        };
        assert_eq!(
            place(&marked, &alone(marked)),
            ess_domain::command::precedence::Place {
                phase: Phase::InputRefusal,
                rank: Rank::Declared,
            },
            "{condition:?}"
        );
        let unmarked = shape(condition, true);
        assert_eq!(place(&unmarked, &alone(unmarked)).phase, Phase::Accepting);
        let accepting = BranchShape {
            before_row: true,
            ..shape(condition, false)
        };
        assert_eq!(place(&accepting, &alone(accepting)).phase, Phase::Accepting);
    }
}

/// The precedence order with `one` and `other` exchanged.
fn exchanged(one: Phase, other: Phase) -> [Phase; 8] {
    let mut order = Phase::PRECEDENCE;
    let at = |phase| order.iter().position(|held| *held == phase).unwrap();
    let (one, other) = (at(one), at(other));
    order.swap(one, other);
    order
}

#[test]
fn the_classification_follows_an_exchanged_phase_order() {
    let command = vec![
        outcome(
            "held",
            OutcomeCondition::SubjectState {
                state: HeldStates::One(StateName::new("Open").unwrap()),
                predicate: None,
            },
            true,
        ),
        outcome(
            "accepted",
            OutcomeCondition::When(predicate("amount > 10")),
            false,
        ),
    ];
    let swapped = exchanged(Phase::HeldState, Phase::Accepting);
    assert_eq!(
        named(&command, FormatVersion::V23),
        expect(&[
            (Phase::HeldState, &["held"]),
            (Phase::Accepting, &["accepted"])
        ])
    );
    let inside = with_phase_order(swapped, || {
        assert_eq!(phase_order(), swapped);
        assert!(Phase::Accepting.position() < Phase::HeldState.position());
        named(&command, FormatVersion::V23)
    });
    assert_eq!(
        inside,
        expect(&[
            (Phase::Accepting, &["accepted"]),
            (Phase::HeldState, &["held"])
        ]),
        "the classification follows the exchanged order"
    );
    assert_eq!(
        phase_order(),
        Phase::PRECEDENCE,
        "restored after the closure"
    );
    assert!(Phase::HeldState.position() < Phase::Accepting.position());
}

#[test]
fn the_phase_order_override_nests_and_is_restored_on_unwind() {
    let outer = exchanged(Phase::InputRefusal, Phase::RelatedRow);
    let inner = exchanged(Phase::HeldState, Phase::Accepting);
    with_phase_order(outer, || {
        with_phase_order(inner, || assert_eq!(phase_order(), inner));
        assert_eq!(
            phase_order(),
            outer,
            "the inner order restores the outer one"
        );
        let unwound = std::panic::catch_unwind(|| {
            with_phase_order(inner, || panic!("a consumer under test panics"));
        });
        assert!(unwound.is_err());
        assert_eq!(phase_order(), outer, "restored on unwind");
    });
    assert_eq!(phase_order(), Phase::PRECEDENCE);
    let other_thread = std::thread::spawn(move || {
        with_phase_order(outer, || std::thread::spawn(phase_order).join().unwrap())
    })
    .join()
    .unwrap();
    assert_eq!(
        other_thread,
        Phase::PRECEDENCE,
        "scoped to the thread that set it"
    );
}

#[test]
#[should_panic(expected = "every phase exactly once")]
fn a_phase_order_naming_a_phase_twice_is_refused() {
    let mut order = Phase::PRECEDENCE;
    order[0] = order[1];
    with_phase_order(order, || ());
}

/// On a row-set command whose creation takes, from the input, the identity its other acting
/// branches address (an upsert, beyond10x/ess#462), the interpreter's `addressed_row` and
/// `selected_subject_refusal` leave an absent row to the row sets (`creation_takes_absent`):
/// `unknown_instance:` answers only once they have, for the branch selected after them, so it
/// closes `PresentRelated` (beyond10x/ess#470 adversary pass 2, R1).
#[test]
fn unknown_instance_answers_after_the_row_sets_where_a_creation_takes_the_absent_identity() {
    let unknown = shape(ConditionShape::UnknownInstance, true);
    let rows = shape(ConditionShape::RelatedSet, true);
    let at = |command: Composition| {
        let placed = place(&unknown, &command);
        (placed.phase, placed.rank)
    };
    let row_set = Composition::new(&[unknown, rows], 0, FormatVersion::V23);
    assert_eq!(at(row_set), (Phase::Existence, Rank::Trail));
    assert_eq!(
        at(row_set.with_upsert(true)),
        (Phase::PresentRelated, Rank::Trail)
    );
    // Only a row set hands the absent row on; anywhere else existence answers at step 3.
    let plain = Composition::new(&[unknown], 0, FormatVersion::V23).with_upsert(true);
    assert_eq!(at(plain), (Phase::Existence, Rank::Trail));
    let stored = Composition::new(&[unknown, shape(STORED_HOLDS, true)], 0, FormatVersion::V23)
        .with_upsert(true);
    assert_eq!(at(stored), (Phase::Existence, Rank::Trail));

    assert_eq!(
        phases(UPSERT_UNKNOWN, "demo.shelf.Shelve", true),
        expect(&[
            (Phase::PresentRelated, &["refused", "unknown-book"]),
            (Phase::Accepting, &["added"]),
            (Phase::Default, &["replaced"]),
        ])
    );
    // No creation takes the identity: the absent row is the command's not-found answer, at once.
    assert_eq!(
        phases(UPSERT_UNKNOWN, "demo.shelf.Restock", false),
        expect(&[
            (Phase::Existence, &["unknown-book"]),
            (Phase::PresentRelated, &["refused"]),
            (Phase::Default, &["restocked"]),
        ])
    );
    // The creation of another identity is not an upsert of this one.
    let other = UPSERT_UNKNOWN.replacen(
        "          demo.shelf.BookShelved: {book: input.book}\n      - name: replaced",
        "          demo.shelf.BookShelved: {book: input.mode}\n      - name: replaced",
        1,
    );
    assert_ne!(
        other, UPSERT_UNKNOWN,
        "the creation's payload is in the model"
    );
    assert_eq!(
        phases(&other, "demo.shelf.Shelve", false),
        expect(&[
            (Phase::Existence, &["unknown-book"]),
            (Phase::PresentRelated, &["refused"]),
            (Phase::Accepting, &["added"]),
            (Phase::Default, &["replaced"]),
        ])
    );
}

// ---- the one precedence rule (G4, `docs/design/generic-language-features.md` § G4) -------------

const RULE: &str = include_str!("../../ess-compiler/tests/fixtures/precedence-rule-format-a.yaml");

/// Under the one rule a missing row the input names answers at step 4, after the addressed
/// record's existence (step 2) and held state (step 3), leading the related-row phase; below it,
/// first of all (step 1).
#[test]
fn format_a_places_input_named_absent_rows_after_the_held_state() {
    assert_eq!(ONE_RULE, FormatVersion::V27);
    let absent = shape(INPUT_ABSENT, true);
    let refusal = shape(INPUT_HOLDS, true);
    let accepting = shape(
        ConditionShape::When {
            trivially_true: true,
        },
        false,
    );
    let command = [absent, refusal, accepting];
    let at = place(&absent, &Composition::new(&command, 1, ONE_RULE));
    assert_eq!((at.phase, at.rank), (Phase::PresentRelated, Rank::Lead));
    assert!(at.phase.position() > Phase::HeldState.position());
    assert!(at.phase.position() > Phase::Existence.position());
    assert_eq!(at.phase.step_in(ONE_RULE), Some(4));
    // Every present-related refusal answers at step 4 under the rule, whatever the composition.
    assert_eq!(
        placed(refusal, &command, 1, ONE_RULE),
        Phase::PresentRelated
    );
    let below = place(&absent, &Composition::new(&command, 1, FormatVersion::V26));
    assert_eq!(
        (below.phase, below.rank),
        (Phase::RelatedRow, Rank::Declared)
    );
    assert_eq!(
        placed(refusal, &command, 1, FormatVersion::V26),
        Phase::Accepting
    );

    assert_eq!(
        phases(RULE, "demo.rule.ChangeTier", false),
        expect(&[
            (Phase::InputRefusal, &["blank-reason"]),
            (Phase::Existence, &["unknown-seat"]),
            (Phase::HeldState, &["not-held"]),
            (Phase::PresentRelated, &["unknown-tier", "tier-closed"]),
            (Phase::Default, &["changed"]),
        ])
    );
}

/// Under the one rule `existing_instance:` answers at step 2, existence, on every command, also
/// one reading a related row or a row set; below it, such a command reads it at step 1.
#[test]
fn format_a_places_existing_instance_at_existence_on_a_related_row_command() {
    let existing = shape(ConditionShape::ExistingInstance, true);
    let accepting = shape(
        ConditionShape::When {
            trivially_true: true,
        },
        false,
    );
    for beside in [INPUT_ABSENT, STORED_HOLDS, ConditionShape::RelatedSet] {
        let command = [existing, shape(beside, true), accepting];
        let at = place(&existing, &Composition::new(&command, 1, ONE_RULE));
        assert_eq!(
            (at.phase, at.rank),
            (Phase::Existence, Rank::Lead),
            "{beside:?}"
        );
        assert_eq!(at.phase.step_in(ONE_RULE), Some(2));
        let below = place(
            &existing,
            &Composition::new(&command, 1, FormatVersion::V26),
        );
        assert_eq!(
            (below.phase, below.rank),
            (Phase::RelatedRow, Rank::Lead),
            "{beside:?}"
        );
    }
}

/// Request-only refusals stay first under the rule (record § "Open questions, answered", answer
/// 1): an input-guarded refusal and an `external:` refusal marked `before_row: true` answer at step
/// 1, before existence, the held state and every related row.
#[test]
fn format_a_keeps_input_refusals_and_before_row_externals_before_existence() {
    let refusal = shape(
        ConditionShape::When {
            trivially_true: false,
        },
        true,
    );
    let before_row = BranchShape {
        before_row: true,
        ..shape(ConditionShape::External, true)
    };
    let absent = shape(INPUT_ABSENT, true);
    let unknown = shape(ConditionShape::UnknownInstance, true);
    let wrong = shape(ConditionShape::WrongState, true);
    let command = [refusal, before_row, unknown, wrong, absent];
    let composition = Composition::new(&command, 1, ONE_RULE);
    for branch in [refusal, before_row] {
        let at = place(&branch, &composition);
        assert_eq!((at.phase, at.rank), (Phase::InputRefusal, Rank::Declared));
        assert_eq!(at.phase.step_in(ONE_RULE), Some(1));
        for later in [unknown, wrong, absent] {
            assert!(
                at.phase.position() < place(&later, &composition).phase.position(),
                "{branch:?} before {later:?}"
            );
        }
    }
    assert_eq!(Phase::InputAbsent.step_in(ONE_RULE), Some(0));
    assert_eq!(Phase::HeldState.step_in(ONE_RULE), Some(3));
    assert_eq!(Phase::Accepting.step_in(ONE_RULE), Some(5));
    assert_eq!(Phase::Default.step_in(ONE_RULE), Some(6));
    assert_eq!(Phase::RelatedRow.step_in(ONE_RULE), None);
    for phase in Phase::PRECEDENCE {
        assert_eq!(phase.step_in(FormatVersion::V26), phase.step(), "{phase:?}");
    }
}

/// [`place`] as it stood before the one rule, copied from `precedence.rs` at the story's base and
/// read through the same public facts: the reference every format below the rule is held to.
fn place_before_the_rule(
    branch: BranchShape,
    branches: &[BranchShape],
    input_rows: usize,
    format: FormatVersion,
    upsert: bool,
) -> ess_domain::command::precedence::Place {
    let any = |test: &dyn Fn(&BranchShape) -> bool| branches.iter().any(test);
    let present_related_refusal = |branch: &BranchShape| {
        branch.error
            && matches!(
                branch.condition,
                ConditionShape::Related { absent: false, .. } | ConditionShape::RelatedSet
            )
    };
    let row_set = any(&|branch| matches!(branch.condition, ConditionShape::RelatedSet));
    let stored_reference = any(&|branch| {
        matches!(
            branch.condition,
            ConditionShape::Related { stored: true, .. }
        )
    });
    let refusal = any(&present_related_refusal);
    let ordered = if row_set {
        refusal
    } else {
        format.major() >= FormatVersion::V22.major()
            && refusal
            && (input_rows > 1
                || any(&|branch| matches!(branch.condition, ConditionShape::WrongState))
                || (format.major() >= FormatVersion::V27.major()
                    && any(&|branch| matches!(branch.condition, ConditionShape::UnknownInstance))))
    };
    let reads_related = any(&|branch| {
        matches!(
            branch.condition,
            ConditionShape::Related { .. } | ConditionShape::RelatedSet
        )
    });
    let orders = ordered || stored_reference;
    let read_first_nothing = !row_set && !stored_reference;
    let at = |phase, rank| ess_domain::command::precedence::Place { phase, rank };
    match branch.condition {
        ConditionShape::InputAbsent => at(Phase::InputAbsent, Rank::Declared),
        ConditionShape::ExistingInstance => {
            if reads_related {
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
                if read_first_nothing {
                    at(Phase::HeldState, Rank::Unguarded)
                } else {
                    at(Phase::PresentRelated, Rank::Unguarded)
                }
            } else {
                at(Phase::InputRefusal, Rank::Declared)
            }
        }
        ConditionShape::UnknownInstance => {
            if row_set && upsert {
                at(Phase::PresentRelated, Rank::Trail)
            } else {
                at(Phase::Existence, Rank::Trail)
            }
        }
        ConditionShape::SubjectState
        | ConditionShape::StateChange
        | ConditionShape::SubjectField
        | ConditionShape::SubjectPredicate
        | ConditionShape::ExternalSubject => at(Phase::HeldState, Rank::Declared),
        ConditionShape::WrongState => at(Phase::HeldState, Rank::Trail),
        ConditionShape::Related { absent: false, .. } | ConditionShape::RelatedSet => {
            if branch.error && orders {
                at(Phase::PresentRelated, Rank::Declared)
            } else {
                at(Phase::Accepting, Rank::Declared)
            }
        }
        ConditionShape::External | ConditionShape::ExternalWhen
            if branch.before_row && branch.error =>
        {
            at(Phase::InputRefusal, Rank::Declared)
        }
        ConditionShape::External | ConditionShape::ExternalWhen => {
            at(Phase::Accepting, Rank::Declared)
        }
        ConditionShape::Otherwise => at(Phase::Default, Rank::Declared),
    }
}

/// Every [`ConditionShape`], with every combination of the facts a branch carries, on every
/// command its companions compose, in every format below the rule: placed exactly where it was.
#[test]
fn below_format_a_every_placement_is_unchanged() {
    let conditions = [
        ConditionShape::When {
            trivially_true: true,
        },
        ConditionShape::When {
            trivially_true: false,
        },
        ConditionShape::SubjectField,
        ConditionShape::SubjectPredicate,
        INPUT_ABSENT,
        INPUT_HOLDS,
        STORED_ABSENT,
        STORED_HOLDS,
        ConditionShape::RelatedSet,
        ConditionShape::SubjectState,
        ConditionShape::StateChange,
        ConditionShape::Otherwise,
        ConditionShape::ExternalWhen,
        ConditionShape::External,
        ConditionShape::WrongState,
        ConditionShape::UnknownInstance,
        ConditionShape::InputAbsent,
        ConditionShape::ExistingInstance,
    ];
    let mut branches = Vec::new();
    for condition in conditions {
        for facts in 0_u8..16 {
            branches.push(BranchShape {
                condition,
                error: facts & 1 != 0,
                subject: facts & 2 != 0,
                replays: facts & 4 != 0,
                before_row: facts & 8 != 0,
            });
        }
    }
    let companions = [
        shape(INPUT_ABSENT, true),
        shape(INPUT_HOLDS, true),
        shape(STORED_HOLDS, true),
        shape(ConditionShape::RelatedSet, true),
        shape(ConditionShape::WrongState, true),
        shape(ConditionShape::UnknownInstance, true),
    ];
    let below: Vec<FormatVersion> = ess_domain::system::SUPPORTED_FORMATS
        .iter()
        .filter(|major| **major < ONE_RULE.major())
        .map(|major| FormatVersion::new(*major).unwrap())
        .collect();
    assert_eq!(below.len(), 26, "every format below the rule");
    let mut checked = 0_usize;
    for mask in 0_u32..(1 << companions.len()) {
        let mut command: Vec<BranchShape> = companions
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .map(|(_, branch)| *branch)
            .collect();
        command.push(BranchShape {
            condition: ConditionShape::Otherwise,
            error: false,
            subject: false,
            replays: false,
            before_row: false,
        });
        for branch in &branches {
            let mut declared = command.clone();
            declared.push(*branch);
            for &format in &below {
                for rows in [0, 1, 2] {
                    for upsert in [false, true] {
                        let composition =
                            Composition::new(&declared, rows, format).with_upsert(upsert);
                        assert_eq!(
                            place(branch, &composition),
                            place_before_the_rule(*branch, &declared, rows, format, upsert),
                            "{branch:?} on {declared:?}, {rows} rows, ess/{}, upsert {upsert}",
                            format.major()
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 64 * 18 * 16 * 26 * 3 * 2);
}
