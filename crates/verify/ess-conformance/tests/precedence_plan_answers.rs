//! The precedence plan gives the model interpreter's answer where a marker's question is decided by
//! what the interpreter reads first (`docs/design/selection-plan.md`, "The markers answer"; adversary
//! pass 2 of `story:selection-plan-design-and-type`, R1 and R3).
//!
//! Each case compiles a fixture, asks the interpreter for one request, and asks the plan which of
//! the branches whose question holds for that request it reads first. Which branches hold is the
//! page's rule, applied by hand and stated beside each case.
//!
//! The `wrong_state:` cases bound where that marker can sit. On
//! `precedence-row-set-updating-branch.yaml`, a closed attempt alone, sent with a delay, is answered
//! by `annotated`; a closed attempt beside a second one, sent with none, by `already-closed` before
//! the row-set refusal `crowded`; an open attempt beside a second one, sent with a delay, by
//! `crowded` before `annotated`. So `already-closed` reads before `crowded`, and `crowded` before
//! `annotated`: no order reads `annotated` before `already-closed` where both are counted as
//! holding, and the first request is answered alike only because `already-closed`'s question does
//! not hold there.
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, PrecedencePlan};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const UPSERT_UNKNOWN: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/precedence-upsert-unknown-instance.yaml"
);
const UPDATING: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/precedence-row-set-updating-branch.yaml"
);

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn step(ir: &EssIr, store: &Store, command: &str, input: &[(&str, Node)]) -> Step {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect();
    let mut steps = execute(
        ir,
        store,
        &command.parse().unwrap(),
        &input,
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{command} {input:?} is interpreted: {error}"));
    assert_eq!(steps.len(), 1, "{command} has one selected outcome");
    steps.remove(0)
}

/// The branch name the interpreter answered with.
fn answered(step: &Step) -> String {
    let outcome = step
        .outcome
        .as_ref()
        .expect("a declared outcome")
        .to_string();
    outcome
        .rsplit_once('/')
        .map_or(outcome.clone(), |(_, name)| name.to_owned())
}

/// The identity the first text field of `event`'s payload carries.
fn minted(step: &Step, event: &str) -> String {
    step.events
        .iter()
        .find(|published| published.event.to_string() == event)
        .and_then(|published| published.payload.values().find_map(Node::as_text))
        .unwrap_or_else(|| panic!("{event} carries the identity"))
        .to_owned()
}

/// Of `holding`, the branch `command`'s plan reads first.
fn plan_first(ir: &EssIr, command: &str, holding: &[&str]) -> String {
    let command = &ir.commands()[&command.parse().unwrap()];
    let read: Vec<String> = PrecedencePlan::new(command, ir.format())
        .iter()
        .map(|(_, outcome)| outcome.name.to_string())
        .collect();
    for name in holding {
        assert!(read.contains(&(*name).to_owned()), "`{name}` is a branch");
    }
    read.into_iter()
        .find(|name| holding.contains(&name.as_str()))
        .expect("a holding branch is planned")
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

// ---- `unknown_instance:` beside a row-set upsert ----------------------------------------------

/// A library whose curator is `the-curator`.
fn opened(ir: &EssIr) -> Store {
    let opened = step(
        ir,
        &Store::default(),
        "demo.shelf.OpenLibrary",
        &[("library", text("main")), ("curator", text("the-curator"))],
    );
    assert_eq!(answered(&opened), "opened");
    opened.next
}

/// A book no row carries, sent by the curator of an open library and not as new: the row set
/// `refused` holds; `unknown-book` holds, since the branch selected after the row sets, the
/// default `replaced`, addresses the absent row; `replaced` holds. The row sets answer first.
#[test]
fn a_row_set_refusal_answers_before_unknown_instance_on_an_upsert() {
    let ir = ir_of(UPSERT_UNKNOWN);
    let store = opened(&ir);
    let interpreter = answered(&step(
        &ir,
        &store,
        "demo.shelf.Shelve",
        &[
            ("book", text("absent-book")),
            ("curator", text("the-curator")),
            ("mode", text("old")),
        ],
    ));
    assert_eq!(interpreter, "refused");
    assert_eq!(
        plan_first(
            &ir,
            "demo.shelf.Shelve",
            &["refused", "unknown-book", "replaced"]
        ),
        interpreter
    );
}

/// The same book sent by a curator with no library: the row set does not hold, and the absent row
/// the default addresses is answered by `unknown-book`, before the default.
#[test]
fn unknown_instance_answers_the_branch_selected_after_the_row_sets_on_an_upsert() {
    let ir = ir_of(UPSERT_UNKNOWN);
    let store = opened(&ir);
    let interpreter = answered(&step(
        &ir,
        &store,
        "demo.shelf.Shelve",
        &[
            ("book", text("absent-book")),
            ("curator", text("someone-else")),
            ("mode", text("old")),
        ],
    ));
    assert_eq!(interpreter, "unknown-book");
    assert_eq!(
        plan_first(&ir, "demo.shelf.Shelve", &["unknown-book", "replaced"]),
        interpreter
    );
}

// ---- `wrong_state:` beside an accepting branch that updates -----------------------------------

/// An attempt of worker `w` and batch `b`, recorded into `store`, and its identity.
fn recorded(ir: &EssIr, store: &Store) -> (Store, String) {
    let recorded = step(
        ir,
        store,
        "demo.jobs.Record",
        &[
            ("worker_id", text("w")),
            ("batch_id", text("b")),
            ("delay", Node::Number(Number::from(1_i64))),
        ],
    );
    let attempt = minted(&recorded, "demo.jobs.AttemptRecorded");
    (recorded.next, attempt)
}

fn shut(ir: &EssIr, store: &Store, attempt: &str) -> Store {
    let shut = step(
        ir,
        store,
        "demo.jobs.Shut",
        &[("attempt_id", text(attempt))],
    );
    assert_eq!(answered(&shut), "shut");
    shut.next
}

fn close(ir: &EssIr, store: &Store, attempt: &str, delay: i64) -> String {
    answered(&step(
        ir,
        store,
        "demo.jobs.Close",
        &[
            ("attempt_id", text(attempt)),
            ("delay", Node::Number(Number::from(delay))),
        ],
    ))
}

/// A closed attempt alone, sent with a delay: `annotated` updates and moves nothing, so not every
/// accepting branch acting on the row moves and `already-closed` does not answer at once; the
/// branch selected, `annotated`, moves nothing, so it does not answer for it either. `annotated`
/// and the default `closed` hold.
#[test]
fn an_updating_branch_answers_on_a_closed_row_where_wrong_state_does_not_hold() {
    let ir = ir_of(UPDATING);
    let (store, attempt) = recorded(&ir, &Store::default());
    let store = shut(&ir, &store, &attempt);
    let interpreter = close(&ir, &store, &attempt, 5);
    assert_eq!(interpreter, "annotated");
    assert_eq!(
        plan_first(&ir, "demo.jobs.Close", &["annotated", "closed"]),
        interpreter
    );
}

/// A closed attempt beside a second one, sent with no delay: the branch selected with the row-set
/// refusal left out is the default `closed`, which moves from `Open`, so `already-closed` holds;
/// `crowded` holds (two rows); `closed` holds. `wrong_state:` answers before the row sets.
#[test]
fn wrong_state_answers_before_the_row_set_refusal_for_the_branch_selected() {
    let ir = ir_of(UPDATING);
    let (store, attempt) = recorded(&ir, &Store::default());
    let (store, _) = recorded(&ir, &store);
    let store = shut(&ir, &store, &attempt);
    let interpreter = close(&ir, &store, &attempt, 0);
    assert_eq!(interpreter, "already-closed");
    assert_eq!(
        plan_first(
            &ir,
            "demo.jobs.Close",
            &["already-closed", "crowded", "closed"]
        ),
        interpreter
    );
}

/// An open attempt beside a second one, sent with a delay: `crowded`, `annotated` and `closed`
/// hold, and the row-set refusal answers before the accepting branch.
#[test]
fn the_row_set_refusal_answers_before_the_updating_branch() {
    let ir = ir_of(UPDATING);
    let (store, attempt) = recorded(&ir, &Store::default());
    let (store, _) = recorded(&ir, &store);
    let interpreter = close(&ir, &store, &attempt, 5);
    assert_eq!(interpreter, "crowded");
    assert_eq!(
        plan_first(&ir, "demo.jobs.Close", &["crowded", "annotated", "closed"]),
        interpreter
    );
}

// ---- the one precedence rule (G4) ----------------------------------------------------------

const RULE: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/precedence-rule-format-a.yaml");

/// A tier no row carries.
const NO_TIER: &str = "00000000-0000-4000-8000-0000000000aa";
/// A seat no row carries.
const NO_SEAT: &str = "00000000-0000-4000-8000-0000000000bb";

/// `RULE` one format below the rule, without the `unknown_instance:` branch that is itself an
/// `ess/27` spelling beside `when_related:`.
fn below_the_rule() -> String {
    let below = RULE.replacen("format: ess/27", "format: ess/26", 1).replacen(
        "      - name: unknown-seat\n        unknown_instance: true\n        error: demo.rule.UnknownSeat\n",
        "",
        1,
    );
    assert!(below.contains("ess/26") && !below.contains("unknown-seat"));
    below
}

/// An open tier, a held seat and a released seat, and the identities of the two seats.
fn seats(ir: &EssIr) -> (Store, String, String) {
    let defined = step(
        ir,
        &Store::default(),
        "demo.rule.DefineTier",
        &[("open", Node::Bool(true))],
    );
    let tier = minted(&defined, "demo.rule.TierDefined");
    let held = step(
        ir,
        &defined.next,
        "demo.rule.Issue",
        &[("tier_id", text(&tier))],
    );
    let held_seat = minted(&held, "demo.rule.Issued");
    let other = step(
        ir,
        &held.next,
        "demo.rule.Issue",
        &[("tier_id", text(&tier))],
    );
    let released_seat = minted(&other, "demo.rule.Issued");
    let released = step(
        ir,
        &other.next,
        "demo.rule.Release",
        &[("seat_id", text(&released_seat))],
    );
    assert_eq!(answered(&released), "released");
    (released.next, held_seat, released_seat)
}

fn change(ir: &EssIr, store: &Store, seat: &str, tier: &str, reason: &str) -> String {
    answered(&step(
        ir,
        store,
        "demo.rule.ChangeTier",
        &[
            ("seat_id", text(seat)),
            ("tier_id", text(tier)),
            ("reason", text(reason)),
        ],
    ))
}

/// A tier no row carries, sent beside each earlier step's question holding: the blank reason
/// (step 1), the unknown seat (step 2) and the released seat (step 3) each answer before the
/// missing tier (step 4), which answers only for a held seat and a reason. Below the rule the
/// missing tier answered first in every case.
#[test]
fn format_a_answers_a_missing_input_named_row_after_unknown_instance_and_held_state() {
    let ir = ir_of(RULE);
    let (store, held, released) = seats(&ir);
    let cases = [
        (
            held.as_str(),
            "",
            "blank-reason",
            &["blank-reason", "unknown-tier", "changed"][..],
        ),
        (
            NO_SEAT,
            "moving",
            "unknown-seat",
            &["unknown-seat", "unknown-tier", "changed"][..],
        ),
        (
            released.as_str(),
            "moving",
            "not-held",
            &["not-held", "unknown-tier", "changed"][..],
        ),
        (
            held.as_str(),
            "moving",
            "unknown-tier",
            &["unknown-tier", "changed"][..],
        ),
    ];
    for (seat, reason, expected, holding) in cases {
        let interpreter = change(&ir, &store, seat, NO_TIER, reason);
        assert_eq!(interpreter, expected, "{seat} {reason:?}");
        assert_eq!(
            plan_first(&ir, "demo.rule.ChangeTier", holding),
            interpreter
        );
    }

    let below = ir_of(&below_the_rule());
    let (store, held, released) = seats(&below);
    for (seat, reason) in [(held.as_str(), ""), (released.as_str(), "moving")] {
        assert_eq!(
            change(&below, &store, seat, NO_TIER, reason),
            "unknown-tier",
            "{seat} {reason:?}"
        );
    }
}

/// A below-rule model whose plan the one rule would change: `PublishRelease` declares the accepting
/// `rushed` before the related refusal `not-accepted`, which below the rule answers among the
/// accepting branches in declaration order, and under it before them. Synthesis builds the plans its
/// searches read in the document's format, not the newest: the witness of `not-accepted` refutes
/// `rushed`, and the interpreter passes every scenario.
#[test]
fn synthesis_plans_read_the_documents_format_not_the_newest() {
    use ess_conformance::interpret::Interpreted;
    use ess_conformance::report::Status;
    use ess_conformance::synthesize::synthesize;
    use ess_conformance::{
        AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig, ScenarioStep, ScenarioValue,
    };

    let release = include_str!("fixtures/related-guard-release.yaml");
    let model = release
        .replacen(
            "      - {name: candidate, type: demo.release.CandidateId}\n    outcomes:\n",
            "      - {name: candidate, type: demo.release.CandidateId}\n      - {name: rush, type: Boolean}\n    outcomes:\n",
            1,
        )
        .replacen(
            "      - name: not-accepted\n",
            "      - name: rushed\n        when: rush == true\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n      - name: not-accepted\n",
            1,
        );
    assert!(model.contains("name: rushed") && model.contains("name: rush,"));
    let ir = ir_of(&model);
    assert!(ir.format().major() < ess_domain::command::precedence::ONE_RULE.major());
    let synthesis = synthesize(&ir);
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.release.PublishRelease/outcome/not-accepted")
        .map_or_else(
            || panic!("synthesized: {:#?}", synthesis.refusals),
            |(_, scenario)| scenario,
        );
    let rushes: Vec<Option<&ScenarioValue>> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.release.PublishRelease" =>
            {
                Some(input.get("rush"))
            }
            _ => None,
        })
        .collect();
    assert!(
        !rushes.is_empty()
            && rushes.iter().all(|rush| *rush
                == Some(&ScenarioValue::Literal {
                    value: Node::Bool(false)
                })),
        "the witness of `not-accepted` refutes the accepting branch declared before it: {rushes:?}"
    );
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(&synthesis.suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(ir))
    .into_report();
    let failed: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

// ---- the guard-pair corpus (G4) -----------------------------------------------------------

const PAIRS: &str = include_str!("fixtures/guard-pair-corpus.yaml");

/// A tier of `openness` and `load`, a held seat, a released one and a retired one, a VIP held seat
/// and a VIP released one, by name.
fn pair_rows(ir: &EssIr, openness: &str, load: &str) -> (Store, BTreeMap<&'static str, String>) {
    let defined = step(
        ir,
        &Store::default(),
        "demo.pairs.DefineTier",
        &[("openness", text(openness)), ("load", text(load))],
    );
    let tier = minted(&defined, "demo.pairs.TierDefined");
    let mut store = defined.next;
    let mut rows = BTreeMap::from([("tier", tier.clone())]);
    for (name, kind, moves) in [
        ("held", "plain", &[][..]),
        ("released", "plain", &["demo.pairs.Release"][..]),
        (
            "retired",
            "plain",
            &["demo.pairs.Release", "demo.pairs.Retire"][..],
        ),
        ("vip-held", "vip", &[][..]),
        ("vip-released", "vip", &["demo.pairs.Release"][..]),
    ] {
        let issued = step(
            ir,
            &store,
            "demo.pairs.Issue",
            &[("tier_id", text(&tier)), ("kind", text(kind))],
        );
        let seat = minted(&issued, "demo.pairs.Issued");
        store = issued.next;
        for command in moves {
            store = step(ir, &store, command, &[("seat_id", text(&seat))]).next;
        }
        rows.insert(name, seat);
    }
    (store, rows)
}

/// Every command of the guard-pair corpus validates at `ess/27`; synthesis witnesses every branch
/// and refuses none, the interpreted target passes the suite, and the interpreter answers each pair
/// in the rule's order where both guards hold: the earlier step first, within a step the first
/// declared.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one corpus: its suite, then each pair's overlap asked of the interpreter"
)]
fn guard_pair_corpus_answers_by_the_rule() {
    use ess_conformance::interpret::Interpreted;
    use ess_conformance::report::Status;
    use ess_conformance::synthesize::synthesize;
    use ess_conformance::{AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};

    let ir = ir_of(PAIRS);
    let synthesis = synthesize(&ir);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?}: {:?}", refusal.subject, refusal.cause))
        .collect();
    assert!(
        refused.is_empty(),
        "synthesis refuses no branch: {refused:#?}"
    );
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(&synthesis.suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
    .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{:?} {:?}", scenario.scenario, scenario.status))
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");

    let send = |store: &Store, command: &str, input: &[(&str, Node)]| {
        answered(&step(&ir, store, command, input))
    };
    let (open, rows) = pair_rows(&ir, "Open", "Spare");
    let (shut_full, full_rows) = pair_rows(&ir, "Shut", "Full");
    let seat = |name: &str| text(&rows[name]);
    let missing = || text(NO_TIER);
    // step 3 before step 4
    for (command, seat_name, expected) in [
        ("demo.pairs.StateBesideRelated", "released", "released"),
        ("demo.pairs.StateBesideRelated", "held", "unknown-tier"),
        ("demo.pairs.SubjectBesideRelated", "vip-held", "vip"),
        ("demo.pairs.SubjectBesideRelated", "held", "unknown-tier"),
        (
            "demo.pairs.WrongStateBesideRelatedMove",
            "held",
            "unknown-tier",
        ),
    ] {
        assert_eq!(
            send(
                &open,
                command,
                &[("seat_id", seat(seat_name)), ("tier_id", missing())]
            ),
            expected,
            "{command} {seat_name}"
        );
    }
    // `wrong_state:` answers for the branch step 5 selects: on an open tier the move, which does
    // not start from a released seat; on a missing one no branch, so the missing tier answers.
    assert_eq!(
        send(
            &open,
            "demo.pairs.WrongStateBesideRelatedMove",
            &[
                ("seat_id", seat("released")),
                ("tier_id", text(&rows["tier"]))
            ]
        ),
        "not-held"
    );
    assert_eq!(
        send(
            &open,
            "demo.pairs.WrongStateBesideRelatedMove",
            &[("seat_id", seat("released")), ("tier_id", missing())]
        ),
        "unknown-tier"
    );
    // within step 3, the first declared; `wrong_state:` last
    for (command, seat_name, expected) in [
        ("demo.pairs.SubjectBesideState", "vip-released", "released"),
        ("demo.pairs.SubjectBesideState", "vip-held", "vip"),
        ("demo.pairs.StateBesideWrongState", "released", "parked"),
        ("demo.pairs.StateBesideWrongState", "retired", "not-held"),
    ] {
        assert_eq!(
            send(
                &open,
                command,
                &[("seat_id", seat(seat_name)), ("kind", text("plain"))]
            ),
            expected,
            "{command} {seat_name}"
        );
    }
    // within step 4, the first declared
    assert_eq!(
        send(
            &shut_full,
            "demo.pairs.TwoRefusalsOneRow",
            &[
                ("seat_id", text(&full_rows["held"])),
                ("tier_id", text(&full_rows["tier"]))
            ]
        ),
        "closed"
    );
    // step 1 before step 4
    let forced = execute(
        &ir,
        &open,
        &"demo.pairs.BeforeRowBesideRelated".parse().unwrap(),
        &BTreeMap::from([
            ("seat_id".to_owned(), seat("held")),
            ("tier_id".to_owned(), missing()),
            ("secret".to_owned(), text("s")),
        ]),
        &Externals::Forced(ess_domain::command::OutcomeName::new("bad-secret").unwrap()),
    )
    .unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(forced.len(), 1);
    assert_eq!(answered(&forced[0]), "bad-secret");
}
