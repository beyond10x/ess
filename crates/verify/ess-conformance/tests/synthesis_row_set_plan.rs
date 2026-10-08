//! A row-set command's witnesses ask the precedence plan which branches answer first
//! (`story:synthesis-reads-selection-plan` unit 5, `docs/design/selection-plan.md`).
//!
//! `row_set::input_for` chooses the input a branch is sent with, refuting every `when:` the plan
//! reads before it; `row_set::consistent` checks that the rows a scenario arranges pass over every
//! row-set branch the plan reads before the expected one. Each command below is added to the
//! attempts fixture and witnessed on rows of the attempts the fixture's `Record` creates:
//!
//! * `Claim` declares a row-set refusal `crowded` (one attempt or more) and an accepting row-set
//!   branch `claimed` (at most one). Under the precedence order the refusal (step 5) answers first,
//!   so `claimed` is witnessed on no attempt and `crowded` on one. With the present-related and
//!   accepting phases exchanged through the plan's one test seam (`with_phase_order`), `claimed`
//!   answers first: it is witnessed on one attempt and `crowded` on two.
//! * `Reserve`, `Admit` and `Start` each pin one order inside a phase, the three order flips in
//!   `row_set.rs` no test caught before (wave 4, § 5): two row-set refusals, two accepting row-set
//!   branches, and an accepting `when:` declared before an accepting row-set branch. In each the
//!   first count or input the search tries is the one the earlier branch claims, so a search
//!   refuting the later branch instead writes a scenario the interpreter answers otherwise.
//!
//! The model is compiled once, outside the exchange; only synthesis and the model interpreter run
//! inside it.
mod support_row_sets;

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_primitives::node::Node;
use support_row_sets::{model, READS};

const SELECTED: &str = "where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}";

/// The attempts fixture with `commands` added after its own.
fn with_commands(commands: &str) -> EssIr {
    const VIEWS: &str = "views:\n";
    assert!(READS.contains(VIEWS), "the fixture declares its views last");
    model(&READS.replacen(VIEWS, &format!("{commands}{VIEWS}"), 1))
}

/// One row-set branch over the attempts of the input's worker and batch, testing `count`.
fn row_set(name: &str, count: &str, rest: &str) -> String {
    format!(
        "      - name: {name}
        when_related:
          entity: demo.jobs.Attempt
          {SELECTED}
          count: {count}
{rest}"
    )
}

fn accepting() -> &'static str {
    "        emits: [demo.jobs.WithinLimit]
        payload:
          demo.jobs.WithinLimit: {worker_id: input.worker_id}
"
}

fn command(name: &str, extra_input: &str, outcomes: &[String]) -> String {
    format!(
        "  - name: demo.jobs.{name}
    input:
      - {{name: worker_id, type: String}}
      - {{name: batch_id, type: String}}
{extra_input}    outcomes:
{}",
        outcomes.concat()
    )
}

/// `crowded` refuses one attempt or more; `claimed` accepts at most one.
fn claim() -> String {
    command(
        "Claim",
        "",
        &[
            row_set("crowded", "{gte: 1}", "        error: demo.jobs.Crowded\n"),
            row_set("claimed", "{lte: 1}", accepting()),
        ],
    )
}

/// `single` refuses exactly one attempt, `busy` at most two; three or more are `reserved`.
fn reserve() -> String {
    command(
        "Reserve",
        "",
        &[
            row_set("single", "{eq: 1}", "        error: demo.jobs.Crowded\n"),
            row_set("busy", "{lte: 2}", "        error: demo.jobs.Ambiguous\n"),
            format!("      - name: reserved\n{}", accepting()),
        ],
    )
}

/// `few` accepts at most one attempt, `many` one or more.
fn admit() -> String {
    command(
        "Admit",
        "",
        &[
            row_set("few", "{lte: 1}", accepting()),
            row_set("many", "{gte: 1}", accepting()),
        ],
    )
}

/// `rushed` accepts a positive delay; `started` accepts no attempt with a delay under ten;
/// `queued` is the default.
fn start() -> String {
    command(
        "Start",
        "      - {name: delay, type: Integer}\n",
        &[
            format!(
                "      - name: rushed\n        when: delay > 0\n{}",
                accepting()
            ),
            row_set(
                "started",
                "{eq: 0}",
                &format!("        when: delay < 10\n{}", accepting()),
            ),
            format!("      - name: queued\n{}", accepting()),
        ],
    )
}

/// The precedence order with the present-related and accepting phases exchanged.
fn exchanged() -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| match phase {
        Phase::PresentRelated => Phase::Accepting,
        Phase::Accepting => Phase::PresentRelated,
        other => other,
    })
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                let refusals: Vec<String> =
                    result.refusals.iter().map(ToString::to_string).collect();
                panic!("no scenario {id}; refusals: {refusals:#?}")
            },
            |(_, scenario)| scenario,
        )
}

/// The last send of `command` in `scenario`, and its input.
fn sent<'a>(
    scenario: &'a ConformanceScenario,
    command: &str,
) -> (usize, &'a BTreeMap<String, ScenarioValue>) {
    scenario
        .steps
        .iter()
        .enumerate()
        .rev()
        .find_map(|(at, step)| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.name().to_string() == command => Some((at, input)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no send of {command}: {scenario:#?}"))
}

/// How many attempts the scenario records, before its send of `command`, for the worker and batch
/// that send names: the rows the row set selects.
fn selected(scenario: &ConformanceScenario, command: &str) -> usize {
    let (at, input) = sent(scenario, command);
    scenario.steps[..at]
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command: recorded, input: row, .. }
                if recorded.name().to_string() == "demo.jobs.Record"
                    && row.get("worker_id") == input.get("worker_id")
                    && row.get("batch_id") == input.get("batch_id"))
        })
        .count()
}

/// Every scenario of `suite` run on the model interpreter of `ir`, by id, where it did not pass.
fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    statuses
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

fn outcome_id(command: &str, outcome: &str) -> String {
    format!("demo.jobs.{command}/outcome/{outcome}")
}

#[test]
fn exchanging_present_related_and_accepting_moves_the_row_set_witnesses() {
    let ir = with_commands(&claim());
    let declared = synthesize(&ir);
    let swapped = with_phase_order(exchanged(), || synthesize(&ir));
    let claimed = outcome_id("Claim", "claimed");
    let crowded = outcome_id("Claim", "crowded");
    let counts = |result: &Synthesis| {
        (
            selected(scenario(result, &claimed), "demo.jobs.Claim"),
            selected(scenario(result, &crowded), "demo.jobs.Claim"),
        )
    };
    assert_eq!(
        counts(&declared),
        (0, 1),
        "under the precedence order `crowded` answers first: `claimed` is witnessed on no \
         attempt, `crowded` on one"
    );
    assert_eq!(
        counts(&swapped),
        (1, 2),
        "with the phases exchanged `claimed` answers first: it is witnessed on one attempt, and \
         `crowded` on two, which `claimed` passes over"
    );
    assert_eq!(
        not_passed(ir.clone(), &declared.suite),
        Vec::<String>::new(),
        "the interpreter passes the suite in the precedence order"
    );
    let failed = with_phase_order(exchanged(), || not_passed(ir.clone(), &swapped.suite));
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the interpreter, reading the same exchanged order, passes the exchanged suite"
    );
}

/// Of two row-set refusals the first declared answers first: `busy` is witnessed where `single`
/// passes over the rows, on two attempts, not on the one the search tries first.
#[test]
fn a_row_set_refusal_is_witnessed_past_the_refusal_declared_before_it() {
    let ir = with_commands(&reserve());
    let result = synthesize(&ir);
    assert_eq!(
        selected(
            scenario(&result, &outcome_id("Reserve", "busy")),
            "demo.jobs.Reserve"
        ),
        2,
        "one attempt is `single`'s, declared first"
    );
    assert_eq!(
        selected(
            scenario(&result, &outcome_id("Reserve", "single")),
            "demo.jobs.Reserve"
        ),
        1
    );
    assert_eq!(not_passed(ir, &result.suite), Vec::<String>::new());
}

/// Of two accepting row-set branches the first declared answers first: `many` is witnessed on two
/// attempts, not on the one `few` takes.
#[test]
fn an_accepting_row_set_branch_is_witnessed_past_the_one_declared_before_it() {
    let ir = with_commands(&admit());
    let result = synthesize(&ir);
    assert_eq!(
        selected(
            scenario(&result, &outcome_id("Admit", "many")),
            "demo.jobs.Admit"
        ),
        2,
        "one attempt is `few`'s, declared first"
    );
    assert_eq!(not_passed(ir, &result.suite), Vec::<String>::new());
}

/// An accepting `when:` declared before an accepting row-set branch answers first: `started` is
/// sent with a delay `rushed` does not claim.
#[test]
fn an_accepting_row_set_branch_is_sent_an_input_the_earlier_when_does_not_claim() {
    let ir = with_commands(&start());
    let result = synthesize(&ir);
    let started = scenario(&result, &outcome_id("Start", "started"));
    let (_, input) = sent(started, "demo.jobs.Start");
    let delay = match input.get("delay") {
        Some(ScenarioValue::Literal {
            value: Node::Number(delay),
        }) => delay.to_string(),
        other => panic!("`started` is sent a literal delay: {other:?}"),
    };
    let delay: i64 = delay
        .parse()
        .unwrap_or_else(|error| panic!("an integer delay {delay}: {error}"));
    assert!(
        delay <= 0,
        "`rushed`, declared first, claims a positive delay; `started` was sent {delay}"
    );
    assert_eq!(not_passed(ir, &result.suite), Vec::<String>::new());
}
