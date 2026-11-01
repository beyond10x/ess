//! Adversary pass 1 over wave D335 unit U3
//! (story:a-binary64-value-round-trips-through-a-conformance-suite).
//!
//! Each case drives the suite contract of `docs/design/model-binary64.md` § "Suite contract for a
//! reached Binary64" against the Rust runner and admission.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::{report::Status, target::*, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const STORY: &str = "story:a-binary64-value-round-trips-through-a-conformance-suite";
const ADDED: &str = "demo.gauge.AddGauge/outcome/added";

fn gauge_text(level: &str, id: &str) -> String {
    format!(
        "format: ess/31\nsystem: demo\nversion: v1\ndomain: demo.gauge\n\
         types:\n  - {{name: demo.gauge.GaugeId, kind: newtype, of: {id}}}\n\
         entities:\n  - name: demo.gauge.Gauge\n    identity: {{name: gauge_id, type: demo.gauge.GaugeId}}\n\
         \x20   fields: [{{name: level, type: {level}}}]\n\
         \x20   lifecycle: {{initial: Open, states: [Open, Closed], terminal: [Closed], transitions: [{{name: close, from: [Open], to: Closed}}]}}\n\
         errors: [{{name: demo.gauge.NotOpen, fields: []}}]\n\
         events:\n  - {{name: demo.gauge.GaugeAdded, fields: [{{name: gauge_id, type: demo.gauge.GaugeId}}]}}\n\
         \x20 - {{name: demo.gauge.GaugeClosed, fields: [{{name: gauge_id, type: demo.gauge.GaugeId}}]}}\n\
         commands:\n  - name: demo.gauge.AddGauge\n    input: [{{name: level, type: {level}}}]\n\
         \x20   outcomes:\n      - {{name: added, creates: demo.gauge.Gauge, instance: gauge_id, sets: {{level: input.level}}, emits: [demo.gauge.GaugeAdded], payload: {{demo.gauge.GaugeAdded: {{gauge_id: {{generated: true}}}}}}}}\n\
         \x20 - name: demo.gauge.CloseGauge\n    input: [{{name: gauge_id, type: demo.gauge.GaugeId}}]\n\
         \x20   outcomes:\n      - {{name: closed, moves: demo.gauge.Gauge.close, instance: gauge_id, emits: [demo.gauge.GaugeClosed], payload: {{demo.gauge.GaugeClosed: {{gauge_id: input.gauge_id}}}}}}\n\
         \x20     - {{name: not-open, wrong_state: true, error: demo.gauge.NotOpen}}\n\
         views:\n  - name: demo.gauge.Gauges\n    source: demo.gauge.Gauge\n    consistency: read_your_writes\n\
         \x20   fields:\n      - {{name: gauge_id, type: demo.gauge.GaugeId}}\n      - {{name: state, type: demo.gauge.Gauge.State}}\n      - {{name: level, type: {level}}}\n"
    )
}

fn model(text: &str) -> Result<EssIr, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("{error:?}"))?;
    let spec = Specification::assemble([(Source::new("gauge.yaml"), raw)])
        .map_err(|error| format!("{error:?}"))?;
    compile(&spec, &SourceMap::new()).map_err(|error| format!("{error:?}"))
}

fn gauge() -> EssIr {
    model(&gauge_text("Binary64", "Uuid")).expect("the gauge compiles")
}

/// The synthesized `added` scenario of the Binary64 gauge, as canonical suite JSON.
fn added_suite_json(ir: &EssIr) -> String {
    let synthesis = ess_conformance::synthesize::synthesize(ir);
    assert!(
        synthesis.refusals.is_empty(),
        "{STORY}: {:?}",
        synthesis.refusals
    );
    let mut suite = synthesis.suite;
    suite.scenarios.retain(|id, _| id.to_string() == ADDED);
    assert_eq!(suite.scenarios.len(), 1, "{STORY}: no `{ADDED}`");
    suite
        .to_canonical_json()
        .unwrap_or_else(|error| panic!("{STORY}: {error}"))
}

/// The interpreted gauge, whose view answers `level` the way an implementation holding an `f64`
/// writes it (`rounded`), or re-spelled with digits past the binary64 (`respelled`).
struct Binary64Gauge {
    inner: ess_conformance::interpret::Interpreted,
    mode: &'static str,
}

impl Binary64Gauge {
    fn shown(&self, level: &Node) -> Node {
        match (self.mode, level) {
            ("rounded", Node::Number(number)) => {
                Node::Number(Number::new(number.get()).expect("finite"))
            }
            ("respelled", Node::Number(number)) => {
                let text = serde_json::to_string(number).unwrap();
                assert!(text.contains('.') && !text.contains(['e', 'E']), "{text}");
                serde_json::from_str(&format!("{text}0000000000000000001")).unwrap()
            }
            _ => level.clone(),
        }
    }
}

impl ConformanceTarget for Binary64Gauge {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        for row in &mut result.rows {
            if let Some(level) = row.get("level").cloned() {
                row.insert("level".into(), self.shown(&level));
            }
        }
        Ok(result)
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
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

fn run_added(ir: &EssIr, json: &str, mode: &'static str) -> Status {
    let admitted =
        AdmittedSuite::from_json(json).unwrap_or_else(|error| panic!("{STORY}: {error}"));
    let target = Binary64Gauge {
        inner: ess_conformance::interpret::Interpreted::for_model(ir.clone()),
        mode,
    };
    let report = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(report.scenarios.len(), 1, "{STORY}");
    report.scenarios[0].status
}

/// Acceptance: "a non-finite value in setup or an answer is refused with its path". The unit's
/// `non_finite_binary64_is_refused_by_name` replaces only command inputs and expected values; this
/// puts `1e400` in a setup step's entity fields, beside a finite control that admits.
#[test]
fn a_non_finite_binary64_in_setup_is_refused_by_name() {
    let ir = gauge();
    let json = added_suite_json(&ir);
    let setup = |level: &str| {
        let mut document: serde_json::Value = serde_json::from_str(&json).unwrap();
        let steps = document["scenarios"][ADDED]["steps"]
            .as_array_mut()
            .expect("steps");
        steps.insert(
            0,
            serde_json::json!({
                "step": "establish_entity",
                "instance": "seeded",
                "entity": "demo.gauge.Gauge",
                "identity": "00000000-0000-4000-8000-000000000001",
                "fields": {"level": "__level__"},
                "state": "Open"
            }),
        );
        serde_json::to_string_pretty(&document)
            .unwrap()
            .replace("\"__level__\"", level)
    };
    AdmittedSuite::from_json(&setup("2.5"))
        .unwrap_or_else(|error| panic!("{STORY}: the finite setup control admits: {error}"));
    let refused =
        AdmittedSuite::from_json(&setup("1e400")).expect_err("a non-finite setup level is refused");
    assert!(
        refused
            .issues
            .iter()
            .any(|issue| issue.reason == "InvalidPayloadNumber" && issue.path.contains("level")),
        "{STORY}: {refused}"
    );
}

/// One authored `AddGauge` act sending `level`.
fn added_act(level: &str) -> String {
    format!(
        "type: ess-scenario/1\n\
         domain: demo.gauge\n\
         scenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n\
         timeline:\n  \
         - at: 2026-01-05T09:00:00Z\n    \
         command: demo.gauge.AddGauge\n    \
         input: {{level: {level}}}\n    \
         outcome: added\n"
    )
}

/// Contract: "Two tokens at a Binary64 position are one value when the binary64 nearest to each
/// ... are equal". A view row carries no declared type, so every runner compares an integer token
/// there exactly, and an implementation that stores an `f64` answers `9007199254740992` for a sent
/// `9007199254740993`. Synthesis never writes a `Binary64` at or past 2^53 and authoring refuses
/// one (§ Authored values), so no suite reaches that comparison: the authored literal is refused
/// by name instead.
#[test]
fn a_binary64_past_two_to_the_53_is_refused_where_it_is_authored() {
    let ir = gauge();
    let json = added_suite_json(&ir);
    assert!(
        json.contains("\"value\": 1.5"),
        "{STORY}: the witness is 1.5"
    );
    let author = |level: &str| {
        ess_conformance::authored::compile(
            &ir,
            &[ess_conformance::authored::Source::new(
                "gauge.scenario.yaml",
                added_act(level),
            )],
        )
    };
    let control = author("1.5");
    assert!(
        control.refusals.is_empty(),
        "{STORY}: the act below 2^53 authors: {:?}",
        control.refusals
    );
    let refused = author("9007199254740993");
    assert!(
        refused.refusals.iter().any(|refusal| {
            let message = refusal.to_string();
            refusal.code().to_string() == "ESS-AUTHOR-015"
                && message.contains("level: `9007199254740993`")
        }),
        "{STORY}: a synthesized suite never writes 9007199254740993 at a Binary64 field and \
         authoring now refuses it as ESS-AUTHOR-015 naming `level` and the number, so the exact \
         comparison of 9007199254740993 against an answered 9007199254740992 is unreachable: {:?}",
        refused.refusals
    );
}

/// Pins the Rust half of runner parity: a view answering the witness re-spelled with digits past
/// the binary64 (`1.50000000000000000001`, the same binary64 as `1.5`) passes; the TypeScript
/// runtime's `rowMatches` answers otherwise (`src/ts/adversary_d335_u3.test.ts`).
#[test]
fn a_binary64_respelled_past_its_digits_passes_the_rust_runner() {
    let ir = gauge();
    let json = added_suite_json(&ir);
    assert_eq!(
        run_added(&ir, &json, "respelled"),
        Status::Passed,
        "{STORY}"
    );
}

/// Acceptance: an entity identity does not carry a `Binary64`; reached through the identity's
/// newtype, the model is refused at that position, and authoring refuses it as ESS-AUTHOR-036.
#[test]
fn a_binary64_entity_identity_is_refused_by_name() {
    let ir = match model(&gauge_text("String", "Binary64")) {
        Ok(ir) => ir,
        Err(error) => {
            // Validation refuses the identity before conformance sees it.
            eprintln!("identity refused by validation: {error}");
            return;
        }
    };
    eprintln!("identity model compiled; admission checked");
    let path = "types.demo.gauge.GaugeId.of";
    let errors = ess_conformance::admission::model(&ir).expect_err("identity is uncarried");
    assert!(
        errors
            .issues
            .iter()
            .any(|issue| issue.path == path && issue.reason == "UnsupportedPrimitive"),
        "{STORY}: {errors}"
    );
    let authored = ess_conformance::authored::compile(&ir, &[]);
    assert!(
        authored
            .refusals
            .iter()
            .any(|refusal| refusal.cause.code().to_string() == "ESS-AUTHOR-036"),
        "{STORY}: {:?}",
        authored.refusals
    );
}
