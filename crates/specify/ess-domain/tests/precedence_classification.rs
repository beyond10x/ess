//! The precedence classification (`docs/design/selection-plan.md`) is reachable from `ess-domain`:
//! one branch of every `OutcomeCondition` is placed, the compositions the page names move a branch
//! between phases, and the classification follows a phase order exchanged inside the test seam.
use ess_domain::command::precedence::{
    order, phase_order, place, with_phase_order, BranchShape, Composition, ConditionShape, Phase,
    Rank,
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
    assert_eq!(conditions.len(), 14, "one of every variant");
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
        }
    );
    refusal.replays = Some(OutcomeName::new("created").unwrap());
    assert!(BranchShape::of(&refusal).replays);
    assert_eq!(
        ConditionShape::from(&OutcomeCondition::Related {
            via: RelatedVia::Subject("assignee".into()),
            test: RelatedTest::Holds(predicate("active == false")),
            input: None,
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
