//! Adversary pass 1, ESS 0.70.0 unit A (beyond10x/ess#554): synthesis and the interpreter over an
//! identity-addressed `when_related:` beside a row set, admitted from `ess/32`.

mod support_row_sets;

use std::collections::BTreeMap;

use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use support_row_sets::{model, rows};

const REPRO: &str =
    include_str!("../../../specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml");

const SOURCE_TAKEN: &str = "      - name: source-taken
        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          exists: true
        error: wiring.board.SourceTaken
";
const SOURCE_NOT_OUTPUT: &str = "      - name: source-not-output
        when_related: {via: input.from_port, predicate: direction != output}
        error: wiring.board.SourceNotOutput
";
const SINK_NOT_INPUT: &str = "      - name: sink-not-input
        when_related: {via: input.to_port, predicate: direction != input}
        error: wiring.board.SinkNotInput
";
const NO_SINK: &str = "      - name: no-sink\n";
const CONNECTED: &str = "      - name: connected\n        creates: wiring.board.Wire\n";

fn edited(text: &str, before: &str, after: &str) -> String {
    let changed = text.replacen(before, after, 1);
    assert_ne!(changed, text, "`{before}` is in the model");
    changed
}

/// `branch` cut out of the reproduction and declared again right above `anchor`.
fn moved(branch: &str, anchor: &str) -> String {
    let cut = edited(REPRO, branch, "");
    edited(&cut, anchor, &format!("{branch}{anchor}"))
}

/// Each scenario of `synthesis` run against `target`, by id.
fn run<T: ConformanceTarget>(synthesis: &Synthesis, target: &T) -> BTreeMap<String, Status> {
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("{error}"));
    ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn failing(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// Every declaration order and test shape the rule admits synthesizes with no refusal, witnesses
/// every branch of `Connect`, and passes against its own interpreter.
#[test]
fn adversary_a_variants_synthesize_every_branch_and_pass() {
    let sink_taken = "      - name: sink-taken
        when_related:
          entity: wiring.board.Wire
          where: to_port == input.to_port
          exists: true
        error: wiring.board.SourceTaken
";
    let variants: Vec<(&str, String)> = vec![
        (
            "row set declared first",
            moved(SOURCE_TAKEN, "      - name: no-source\n"),
        ),
        (
            "row set between the named rows",
            moved(SOURCE_TAKEN, NO_SINK),
        ),
        (
            "row set above the predicates",
            moved(SOURCE_TAKEN, SOURCE_NOT_OUTPUT),
        ),
        (
            "count test",
            edited(
                REPRO,
                "          exists: true\n",
                "          count: {gt: 0}\n",
            ),
        ),
        (
            "the row set over the sink instead",
            edited(
                REPRO,
                "          where: from_port == input.from_port\n",
                "          where: to_port == input.to_port\n",
            ),
        ),
        (
            "a second row set over the sink",
            edited(REPRO, CONNECTED, &format!("{sink_taken}{CONNECTED}")),
        ),
    ];
    let mut report = Vec::new();
    for (label, text) in &variants {
        let synthesis = synthesize(&model(text));
        let refused: Vec<String> = synthesis
            .refusals
            .iter()
            .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
            .collect();
        let ids: Vec<String> = synthesis
            .suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect();
        let missing: Vec<&str> = [
            "no-source",
            "no-sink",
            "source-not-output",
            "sink-not-input",
            "source-taken",
            "connected",
        ]
        .into_iter()
        .chain(text.contains("sink-taken").then_some("sink-taken"))
        .filter(|branch| !ids.contains(&format!("wiring.board.Connect/outcome/{branch}")))
        .collect();
        let target = Interpreted::for_model(model(text));
        let failed = failing(&run(&synthesis, &target));
        if !refused.is_empty() || !missing.is_empty() || !failed.is_empty() {
            report.push(format!(
                "{label}: refused {refused:?}, unwitnessed {missing:?}, failing {failed:?}"
            ));
        }
    }
    assert!(report.is_empty(), "{}", report.join("\n"));
}

/// Within step 4 declaration order decides between a refusal over a present named row and a
/// row-set refusal: the reproduction answers an occupied source with a wrong-facing sink as
/// `sink-not-input`. A target answering it `source-taken` (the row set read first, as declaring it
/// first would) passes the suite unless some scenario holds both.
#[test]
fn adversary_a_suite_tells_a_row_set_read_before_a_named_row_refusal() {
    let synthesis = synthesize(&model(REPRO));
    let reordered = moved(SOURCE_TAKEN, SINK_NOT_INPUT);
    let statuses = run(&synthesis, &Interpreted::for_model(model(&reordered)));
    assert!(
        !failing(&statuses).is_empty(),
        "every scenario passes against the row set read before `sink-not-input`: {statuses:#?}"
    );
}

/// The same question about two refusals over present named rows, at `ess/31` where no row set
/// sits beside them: the class the row-set case belongs to, run on the code below the unit's gate.
#[test]
fn adversary_a_suite_tells_two_named_row_refusals_apart_at_ess_31() {
    let base = edited(
        &edited(REPRO, SOURCE_TAKEN, ""),
        "format: ess/32\n",
        "format: ess/31\n",
    );
    let synthesis = synthesize(&model(&base));
    let reordered = edited(
        &edited(&base, SINK_NOT_INPUT, ""),
        SOURCE_NOT_OUTPUT,
        &format!("{SINK_NOT_INPUT}{SOURCE_NOT_OUTPUT}"),
    );
    let statuses = run(&synthesis, &Interpreted::for_model(model(&reordered)));
    assert!(
        !failing(&statuses).is_empty(),
        "every scenario passes against `sink-not-input` read before `source-not-output`: \
         {statuses:#?}"
    );
}

/// The interpreter of the reproduction, except that a refusal of `Connect` is answered
/// `source-taken` wherever a wire already leaves the source: the row set read before the named
/// rows' answers, a missing sink included.
struct RowSetFirst {
    inner: Interpreted,
}

impl ConformanceTarget for RowSetFirst {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let connect = request.command.to_string() == "wiring.board.Connect";
        let from = request.input.get("from_port").cloned();
        let mut answer = self.inner.execute_command(request)?;
        let refused = answer.error.is_some();
        let taken = answer
            .outcome
            .as_ref()
            .is_some_and(|taken| taken.outcome.as_str() == "source-taken");
        if connect && refused && !taken {
            let occupied = rows(&self.inner, "wiring.board.Wires")
                .iter()
                .any(|row| row.get("from_port") == from.as_ref());
            if occupied {
                if let Some(taken) = answer.outcome.as_mut() {
                    taken.outcome = "source-taken".parse().unwrap();
                }
                answer.error = Some(DeclaredErrorValue::new(
                    "wiring.board.SourceTaken".parse().unwrap(),
                ));
            }
        }
        Ok(answer)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// A target reading the row set before a missing named row breaks the rule's first clause; some
/// scenario of the suite holds a missing named row beside an occupied source, or none does.
#[test]
fn adversary_a_suite_tells_a_row_set_read_before_a_missing_named_row() {
    let synthesis = synthesize(&model(REPRO));
    let target = RowSetFirst {
        inner: Interpreted::for_model(model(REPRO)),
    };
    let statuses = run(&synthesis, &target);
    assert!(
        !failing(&statuses).is_empty(),
        "every scenario passes against the row set read before a missing named row: \
         {statuses:#?}"
    );
}

/// A model without the construct keeps its suite, refusals and notes when its format moves from
/// `ess/31` to `ess/32` (the `ess/32` synthesis branches are gated on the format).
#[test]
fn adversary_a_models_without_the_construct_keep_their_suite_at_ess_32() {
    let models: [(&str, &str); 9] = [
        (
            "existence-list-membership",
            include_str!("fixtures/existence-list-membership.yaml"),
        ),
        (
            "updating-commands-keep-scenarios",
            include_str!("fixtures/updating-commands-keep-scenarios.yaml"),
        ),
        (
            "beside-row-once",
            include_str!("fixtures/beside-row-once.yaml"),
        ),
        ("fleet-sites", include_str!("fixtures/fleet-sites.yaml")),
        (
            "affects-where-state",
            include_str!("fixtures/affects-where-state.yaml"),
        ),
        (
            "coalesce-selector",
            include_str!("fixtures/coalesce-selector.yaml"),
        ),
        (
            "enforced-reference",
            include_str!("fixtures/enforced-reference.yaml"),
        ),
        (
            "now-view-filter",
            include_str!("fixtures/now-view-filter.yaml"),
        ),
        (
            "list-input-example",
            include_str!("../../../specify/ess-domain/tests/fixtures/list-input-example.yaml"),
        ),
    ];
    let at = |text: &str, format: &str| {
        let first = text.lines().next().unwrap_or_default();
        assert!(first.starts_with("format: ess/3"), "{first}");
        text.replacen(first, &format!("format: {format}"), 1)
    };
    let mut moved_suites = Vec::new();
    for (label, text) in models {
        let read = |format: &str| {
            let synthesis = synthesize(&model(&at(text, format)));
            let suite = synthesis.suite.to_canonical_json().unwrap();
            let rest = format!(
                "{:?}\n{:?}\n{:?}",
                synthesis.refusals, synthesis.notes, synthesis.outside
            );
            (
                suite.replace("ess/32", "ess/31"),
                rest.replace("ess/32", "ess/31"),
            )
        };
        if read("ess/31") != read("ess/32") {
            moved_suites.push(label);
        }
    }
    assert!(moved_suites.is_empty(), "moved at ess/32: {moved_suites:?}");
}
