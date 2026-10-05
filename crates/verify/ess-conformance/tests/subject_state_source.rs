//! The held lifecycle state as a value source, `{subject: state}` (ess/23, beyond10x/ess#458), in
//! the interpreter, in synthesis and in the Rust and Go runners.
//!
//! The documents: `PublishDoc` and `ArchiveDoc` answer `StateConflict {doc_id, current, requested}`
//! on `wrong_state:`, `current` read from the held row and `requested` the literal state the refused
//! move would have entered. `DocPublished.from` reads the state before the move, and `archive` writes
//! it into `previous`. Every `<entity>/state/<S>/refuses/<command>` scenario requires `current == S`
//! and `requested` the literal; a target answering the requested state in `current`, the current one
//! in `requested`, or the state after the move in `from` or `previous`, fails a scenario.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::target::{
    ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/subject-state-source.yaml");

fn ir_of(label: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new(label.to_owned()), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn ir() -> EssIr {
    ir_of("docs.yaml", MODEL)
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", synthesis.refusals);
    synthesis.suite
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

// ---- the interpreter -----------------------------------------------------------------------------

fn name(text: &str) -> ess_domain::name::QualifiedName {
    text.parse().unwrap()
}

fn one(ir: &EssIr, store: &Store, command: &str, doc: Option<&Node>) -> Step {
    let input: BTreeMap<String, Node> = doc
        .map(|doc| BTreeMap::from([("doc_id".to_owned(), doc.clone())]))
        .unwrap_or_default();
    let mut steps = execute(ir, store, &name(command), &input, &Externals::Withheld)
        .unwrap_or_else(|undetermined| panic!("{command}: {undetermined:?}"));
    assert_eq!(steps.len(), 1, "{steps:#?}");
    steps.remove(0)
}

fn created(ir: &EssIr) -> (Store, Node) {
    let doc = text("00000000-0000-4000-8000-000000000001");
    let step = one(ir, &Store::default(), "demo.docs.CreateDoc", Some(&doc));
    (step.next, doc)
}

fn error_field(step: &Step, field: &str) -> Node {
    step.error
        .as_ref()
        .unwrap_or_else(|| panic!("an error: {step:#?}"))
        .fields
        .get(field)
        .cloned()
        .unwrap_or_else(|| panic!("`{field}` in {step:#?}"))
}

#[test]
fn the_interpreter_answers_the_held_state_and_the_requested_one() {
    let ir = ir();
    let (store, doc) = created(&ir);
    // `archive` does not start from `Draft`.
    let refused = one(&ir, &store, "demo.docs.ArchiveDoc", Some(&doc));
    assert_eq!(error_field(&refused, "current"), text("Draft"));
    assert_eq!(error_field(&refused, "requested"), text("Archived"));
    let published = one(&ir, &store, "demo.docs.PublishDoc", Some(&doc));
    // `publish` does not start from `Published`.
    let again = one(&ir, &published.next, "demo.docs.PublishDoc", Some(&doc));
    assert_eq!(error_field(&again, "current"), text("Published"));
    assert_eq!(error_field(&again, "requested"), text("Published"));
}

#[test]
fn subject_state_source_in_event_payload_and_sets() {
    let ir = ir();
    let (store, doc) = created(&ir);
    let published = one(&ir, &store, "demo.docs.PublishDoc", Some(&doc));
    assert_eq!(published.events[0].payload["from"], text("Draft"));
    let archived = one(&ir, &published.next, "demo.docs.ArchiveDoc", Some(&doc));
    let row = archived
        .next
        .instance_typed(&name("demo.docs.Doc"), &doc)
        .unwrap();
    assert_eq!(row.state.as_str(), "Archived");
    assert_eq!(row.fields["previous"], text("Published"));
}

// ---- synthesis -----------------------------------------------------------------------------------

/// Every refusal scenario of `command`, by the state it arranges.
fn refusals(suite: &ConformanceSuite, command: &str) -> BTreeMap<String, Vec<ScenarioStep>> {
    suite
        .scenarios
        .iter()
        .filter_map(|(id, scenario)| {
            let id = id.to_string();
            let state = id
                .strip_prefix("demo.docs.Doc/state/")?
                .strip_suffix(&format!("/refuses/{command}"))?
                .to_owned();
            Some((state, scenario.steps.clone()))
        })
        .collect()
}

fn required_error(steps: &[ScenarioStep]) -> BTreeMap<String, Node> {
    steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectError { error, fields }
                if error.to_string() == "demo.docs.StateConflict" =>
            {
                Some(fields.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{steps:#?}"))
}

#[test]
fn wrong_state_scenarios_compare_current_and_requested() {
    let suite = suite();
    for (command, requested, states) in [
        (
            "demo.docs.PublishDoc",
            "Published",
            ["Archived", "Published"],
        ),
        ("demo.docs.ArchiveDoc", "Archived", ["Archived", "Draft"]),
    ] {
        let scenarios = refusals(&suite, command);
        assert_eq!(
            scenarios.keys().map(String::as_str).collect::<Vec<_>>(),
            states,
            "{command}"
        );
        for (state, steps) in &scenarios {
            let fields = required_error(steps);
            assert_eq!(
                fields.get("current"),
                Some(&text(state)),
                "{command} in {state}"
            );
            assert_eq!(
                fields.get("requested"),
                Some(&text(requested)),
                "{command} in {state}"
            );
        }
    }
}

#[test]
fn the_interpreter_passes_the_synthesized_suite() {
    let suite = suite();
    let verdicts = support_go::rust_outcomes(&suite, &Interpreted::for_model(ir()));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
}

/// The base digests `adversary_244b_window_free_bytes.rs` recorded for every model the tree held:
/// a model that does not read the held state compiles to the bytes it did.
#[test]
fn old_models_keep_ir_bytes() {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let digest = |bytes: &str| {
        let hex = Sha256::digest(bytes.as_bytes()).iter().take(10).fold(
            String::new(),
            |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            },
        );
        format!("{hex}:{}", bytes.len())
    };
    let recorded = include_str!("fixtures/adversary-244b-base-digests.tsv");
    for fixture in ["error-payload-sources.yaml", "subject-state.yaml"] {
        let label = format!("crates/verify/ess-conformance/tests/fixtures/{fixture}");
        let line = recorded
            .lines()
            .find(|line| line.split('\t').next() == Some(label.as_str()))
            .unwrap_or_else(|| panic!("{label} recorded"));
        let ir_digest = line
            .split('\t')
            .nth(1)
            .and_then(|fields| fields.split(' ').next())
            .and_then(|ir| ir.strip_prefix("ir="))
            .unwrap();
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture),
        )
        .unwrap();
        let ir = ir_of(&label, &text);
        assert_eq!(digest(&ir.to_canonical_json()), ir_digest, "{fixture}");
    }
}

// ---- a hand-written target, healthy and wrong in one way each ------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// `current` carries the state the refused move would have entered.
    CurrentIsRequested,
    /// `requested` carries the state the document holds.
    RequestedIsCurrent,
    /// `DocPublished.from` carries the state after the move.
    FromIsAfter,
    /// `previous` holds the state after the move.
    PreviousIsAfter,
}

const MODES: [Mode; 5] = [
    Mode::Correct,
    Mode::CurrentIsRequested,
    Mode::RequestedIsCurrent,
    Mode::FromIsAfter,
    Mode::PreviousIsAfter,
];

#[derive(Clone)]
struct Row {
    state: &'static str,
    previous: Option<&'static str>,
}

struct Docs {
    mode: Mode,
    rows: RefCell<BTreeMap<String, Row>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Docs {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: RefCell::new(BTreeMap::new()),
            published: RefCell::new(Vec::new()),
        }
    }

    fn took(command: &str, outcome: &str) -> SemanticCommandResult {
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            ess_compiler::refs::CommandRef::new(command.parse().unwrap()),
            outcome.parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        result
    }

    fn event(&self, name: &str, payload: &[(&str, &str)]) -> ObservedEvent {
        let mut event = ObservedEvent::new(name.parse().unwrap());
        for (field, value) in payload {
            event.payload.insert((*field).to_owned(), text(value));
        }
        self.published.borrow_mut().push(event.clone());
        event
    }

    /// `command` moving `doc` from `from` to `to`, answering `StateConflict` elsewhere.
    fn moving(
        &self,
        command: &str,
        doc: &str,
        (from, to): (&'static str, &'static str),
    ) -> SemanticCommandResult {
        let held = self.rows.borrow().get(doc).cloned();
        let Some(held) = held else {
            return Self::took(command, "missing").with_error(
                DeclaredErrorValue::new("demo.docs.NoSuchDoc".parse().unwrap())
                    .with("doc_id", text(doc)),
            );
        };
        if held.state != from {
            let (current, requested) = match self.mode {
                Mode::CurrentIsRequested => (to, to),
                Mode::RequestedIsCurrent => (held.state, held.state),
                _ => (held.state, to),
            };
            return Self::took(command, "conflict").with_error(
                DeclaredErrorValue::new("demo.docs.StateConflict".parse().unwrap())
                    .with("doc_id", text(doc))
                    .with("current", text(current))
                    .with("requested", text(requested)),
            );
        }
        let previous = if self.mode == Mode::PreviousIsAfter {
            to
        } else {
            held.state
        };
        self.rows.borrow_mut().insert(
            doc.to_owned(),
            Row {
                state: to,
                previous: (to == "Archived").then_some(previous).or(held.previous),
            },
        );
        let mut result = Self::took(
            command,
            if to == "Published" {
                "published"
            } else {
                "archived"
            },
        );
        let event = if to == "Published" {
            let before = if self.mode == Mode::FromIsAfter {
                to
            } else {
                from
            };
            self.event(
                "demo.docs.DocPublished",
                &[("doc_id", doc), ("from", before)],
            )
        } else {
            self.event("demo.docs.DocArchived", &[("doc_id", doc)])
        };
        result.direct_events.push(event);
        result
    }
}

impl ConformanceTarget for Docs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("subject-state-source", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        self.published.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let doc = request
            .input
            .get("doc_id")
            .and_then(Node::as_text)
            .unwrap_or_default()
            .to_owned();
        Ok(match request.command.to_string().as_str() {
            "demo.docs.CreateDoc" => {
                let id = doc.clone();
                self.rows.borrow_mut().insert(
                    id.clone(),
                    Row {
                        state: "Draft",
                        previous: None,
                    },
                );
                let mut result = Self::took("demo.docs.CreateDoc", "created");
                result
                    .direct_events
                    .push(self.event("demo.docs.DocCreated", &[("doc_id", &id)]));
                result
            }
            "demo.docs.PublishDoc" => {
                self.moving("demo.docs.PublishDoc", &doc, ("Draft", "Published"))
            }
            "demo.docs.ArchiveDoc" => {
                self.moving("demo.docs.ArchiveDoc", &doc, ("Published", "Archived"))
            }
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self
                .rows
                .borrow()
                .iter()
                .map(|(id, row)| {
                    let mut fields = BTreeMap::from([
                        ("doc_id".to_owned(), text(id)),
                        ("state".to_owned(), text(row.state)),
                    ]);
                    if let Some(previous) = row.previous {
                        fields.insert("previous".to_owned(), text(previous));
                    }
                    fields
                })
                .collect(),
            total: None,
        })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external outcomes", "unused"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
}

fn check(label: &str, mode: Mode, verdicts: &BTreeMap<String, String>) {
    let failed = support_go::not_passed(verdicts);
    match mode {
        Mode::Correct => assert_eq!(failed, Vec::<&str>::new(), "{label}: {verdicts:#?}"),
        // Wherever the held state and the requested one differ, the swap is seen.
        Mode::CurrentIsRequested | Mode::RequestedIsCurrent => {
            for id in [
                "demo.docs.Doc/state/Draft/refuses/demo.docs.ArchiveDoc",
                "demo.docs.Doc/state/Archived/refuses/demo.docs.PublishDoc",
            ] {
                assert_eq!(verdicts[id], "failed", "{label}: {id}: {verdicts:#?}");
            }
        }
        Mode::FromIsAfter => assert_eq!(
            verdicts["demo.docs.PublishDoc/outcome/published"], "failed",
            "{label}: {verdicts:#?}"
        ),
        Mode::PreviousIsAfter => assert_eq!(
            verdicts["demo.docs.ArchiveDoc/outcome/archived"], "failed",
            "{label}: {verdicts:#?}"
        ),
    }
}

#[test]
fn the_rust_runner_fails_every_faulty_target() {
    let suite = suite();
    for mode in MODES {
        let verdicts = support_go::rust_outcomes(&suite, &Docs::new(mode));
        check(&format!("rust-{mode:?}"), mode, &verdicts);
    }
}

#[test]
fn the_go_runner_gives_the_reference_verdict_for_every_mode() {
    let suite = suite();
    for mode in MODES {
        let label = format!("go-458-{mode:?}").to_lowercase();
        let verdicts = support_go::assert_parity(&label, &suite, Docs::new(mode));
        check(&label, mode, &verdicts);
    }
}

/// Correction round 1: every scenario that arranges the row in a known state and asserts an error
/// reading `{subject: state}` compares it, the compensating refusal's own and each source of its
/// move included (`from_source`).
#[test]
fn a_compensating_refusal_reading_the_held_state_is_compared_from_every_source() {
    let model = include_str!("../../../specify/ess-compiler/tests/fixtures/refusal-with-effect.yaml")
        .replace("format: ess/22\n", "format: ess/23\n")
        .replace(
            "  - name: shop.order.Refused\n    summary: The upstream refused the join.\n    fields: []\n",
            "  - name: shop.order.Refused\n    summary: The upstream refused the join.\n    fields:\n      - {name: was, type: shop.order.Order.State}\n",
        )
        .replace(
            "        compensates: true\n",
            "        compensates: true\n        payload:\n          shop.order.Refused: {was: {subject: state}}\n",
        );
    assert!(model.contains("{was: {subject: state}}"));
    let ir = ir_of("order.yaml", &model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let mut compared = BTreeMap::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExpectError { error, fields } = step {
                if error.to_string() == "shop.order.Refused" {
                    compared.insert(id.to_string(), fields.get("was").cloned());
                }
            }
        }
    }
    assert_eq!(
        compared.get("shop.order.Order/transition/reset/by/shop.order.JoinOrder/failed"),
        Some(&Some(text("Joined"))),
        "{compared:#?}"
    );
    assert_eq!(
        compared.get("shop.order.JoinOrder/outcome/failed"),
        Some(&Some(text("Idle"))),
        "{compared:#?}"
    );
    let verdicts = support_go::rust_outcomes(&synthesis.suite, &Interpreted::for_model(ir));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
}
