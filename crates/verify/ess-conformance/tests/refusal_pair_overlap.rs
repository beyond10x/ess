//! Two input refusals whose guards overlap: the first declared answers, and the suite says so
//! apart from the first refusal's own witness (beyond10x/ess#455).
//!
//! `invalid-code: code == ""` is declared before `pause-refused: pause == true`. The first
//! refusal's primary send lies outside the second's guard where some input does, and the overlap
//! `{code: "", pause: true}` is sent once more, separately, requiring `invalid-code`. A target that
//! checks `pause` first then fails only that send; a target that never refuses a blank code fails
//! the primary one.
use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ErrorRef, OutcomeRef};
use ess_conformance::synthesize::Note;
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, DeclaredErrorValue, EventObservationRequest,
    ExternalOutcomeControl, ImplementationIdentity, InvocationObservationRequest, ObservedEvent,
    ObservedInvocation, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner,
    ScenarioStep, ScenarioValue,
};
use ess_domain::command::OutcomeName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/refusal-pair-overlap.yaml");

const INVALID_CODE: &str = "demo.hold.Place/outcome/invalid-code";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("refusal-pair-overlap.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// A literal input value as text, for comparison.
fn literal(input: &BTreeMap<String, ScenarioValue>, field: &str) -> String {
    match input.get(field).and_then(ScenarioValue::as_literal) {
        Some(Node::Text(text)) => format!("{text:?}"),
        Some(Node::Bool(value)) => value.to_string(),
        other => panic!("a literal {field}, got {other:?}"),
    }
}

/// Each `Place` the scenario sends: its `code`, its `pause`, and the outcome the next step requires.
fn sends(scenario: &ConformanceScenario) -> Vec<(String, String, Option<String>)> {
    let mut out = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { input, .. } = step else {
            continue;
        };
        let required = scenario.steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        out.push((literal(input, "code"), literal(input, "pause"), required));
    }
    out
}

fn send(code: &str, pause: bool, required: &str) -> (String, String, Option<String>) {
    (
        format!("{code:?}"),
        pause.to_string(),
        Some(required.to_owned()),
    )
}

/// The suite with `invalid-code`'s scenario cut to the steps of one of its sends: `0` the primary
/// send, `1` the next.
fn only_send(suite: &ConformanceSuite, nth: usize) -> ConformanceSuite {
    let mut cut = suite.clone();
    let (_, scenario) = cut
        .scenarios
        .iter_mut()
        .find(|(key, _)| key.to_string() == INVALID_CODE)
        .expect("the scenario is filed");
    let starts: Vec<usize> = scenario
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .map(|(at, _)| at)
        .collect();
    let end = starts.get(nth + 1).copied().unwrap_or(scenario.steps.len());
    scenario.steps = scenario.steps[starts[nth]..end].to_vec();
    cut
}

fn run(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    /// The model as declared.
    None,
    /// Checks `pause` before `code`: the declared order swapped.
    PauseFirst,
    /// Never refuses a blank code: it reads the code as if one were given.
    NeverBlank,
}

struct Target {
    inner: Interpreted,
    change: Change,
}

impl Target {
    fn new(change: Change) -> Self {
        Self {
            inner: Interpreted::for_model(ir(MODEL)),
            change,
        }
    }
}

impl ConformanceTarget for Target {
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
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let blank = request.input.get("code") == Some(&Node::Text(String::new()));
        let pause = request.input.get("pause") == Some(&Node::Bool(true));
        if self.change == Change::NeverBlank && blank {
            request
                .input
                .insert("code".to_owned(), Node::Text("code".to_owned()));
        }
        let command = request.command.clone();
        let mut result = self.inner.execute_command(request)?;
        if self.change == Change::PauseFirst && pause {
            // The model refuses every `pause: true` input and changes nothing, so only the answer
            // differs.
            result.outcome = Some(OutcomeRef::new(
                command,
                OutcomeName::new("pause-refused").expect("a name"),
            ));
            result.error = Some(DeclaredErrorValue::new(
                "demo.hold.PauseRefused"
                    .parse::<ErrorRef>()
                    .expect("an error"),
            ));
            result.direct_events.clear();
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

#[test]
fn earlier_refusal_witness_avoids_later_sibling() {
    let result = synthesis(MODEL);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let sent = sends(scenario(&result.suite, INVALID_CODE));
    assert_eq!(
        sent.first(),
        Some(&send("", false, "invalid-code")),
        "the primary send is a blank code outside `pause-refused`'s guard: {sent:#?}"
    );
}

#[test]
fn refusal_pair_overlap_sent_separately() {
    let result = synthesis(MODEL);
    let sent = sends(scenario(&result.suite, INVALID_CODE));
    assert_eq!(
        sent,
        vec![
            send("", false, "invalid-code"),
            send("", true, "invalid-code")
        ],
        "the overlap is sent once more, after the primary send, requiring the first declared"
    );
}

#[test]
fn swapped_order_target_fails_only_overlap() {
    let suite = synthesis(MODEL).suite;
    let honest = run(&suite, &Target::new(Change::None));
    assert_eq!(not_passed(&honest), Vec::<&str>::new(), "{honest:#?}");

    let swapped = Target::new(Change::PauseFirst);
    assert_eq!(
        not_passed(&run(&suite, &swapped)),
        vec![INVALID_CODE],
        "checking `pause` first fails the first refusal's scenario and nothing else"
    );
    assert_eq!(
        not_passed(&run(&only_send(&suite, 0), &swapped)),
        Vec::<&str>::new(),
        "its primary send passes"
    );
    assert_eq!(
        not_passed(&run(&only_send(&suite, 1), &swapped)),
        vec![INVALID_CODE],
        "its overlap send fails"
    );

    let never_blank = Target::new(Change::NeverBlank);
    assert!(
        not_passed(&run(&only_send(&suite, 0), &never_blank)).contains(&INVALID_CODE),
        "a target never refusing a blank code fails the primary send"
    );
}

#[test]
fn inseparable_refusals_keep_overlap_witness() {
    // Every blank code is also not "x": no input separates the two guards.
    let inseparable = MODEL.replace("when: pause == true", "when: code != \"x\"");
    assert_ne!(inseparable, MODEL);
    let result = synthesis(&inseparable);
    let sent: Vec<_> = sends(scenario(&result.suite, INVALID_CODE))
        .into_iter()
        .map(|(code, _, required)| (code, required))
        .collect();
    assert_eq!(
        sent,
        vec![("\"\"".to_owned(), Some("invalid-code".to_owned()))],
        "the one witness is the overlap, sent once"
    );
}

/// The SHA-256 of the disjoint model's scenarios, as `to_canonical_json` writes them, synthesized
/// by the integration base `6192d9be25` (before #454 and #455). The provenance is left out, so a
/// version bump does not move it.
const DISJOINT_SCENARIOS_AT_BASE: &str =
    "39ae4e9a774e9623cc61b6e56d91f6f685e45d6f3f0e095bc1d4d5e0f05f37bb";

#[test]
fn no_overlap_models_bytes_unchanged() {
    use sha2::{Digest, Sha256};
    // Disjoint guards: the first refusal's witness already lies outside the second's.
    let disjoint = MODEL.replace(
        "when: pause == true",
        "when: {all: [pause == true, code != \"\"]}",
    );
    assert_ne!(disjoint, MODEL);
    let result = synthesis(&disjoint);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let canonical: serde_json::Value = serde_json::from_str(
        &result
            .suite
            .to_canonical_json()
            .expect("the suite is canonical"),
    )
    .expect("canonical JSON");
    let scenarios = serde_json::to_string(&canonical["scenarios"]).expect("serializes");
    let mut digest = String::new();
    for byte in Sha256::digest(scenarios.as_bytes()) {
        use std::fmt::Write as _;
        write!(digest, "{byte:02x}").expect("writes to a string");
    }
    assert_eq!(
        digest, DISJOINT_SCENARIOS_AT_BASE,
        "a model without overlapping input refusals synthesizes the bytes it did before:\n{scenarios}"
    );
}

/// The `### Two input refusals whose guards overlap` subsection of `## Which branch answers`.
fn subsection(page: &str) -> Option<&str> {
    let section = page.find("\n## Which branch answers\n")? + 1;
    let rest = &page[section..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |at| at + 4);
    let section = &rest[..end];
    let start = section.find("\n### Two input refusals whose guards overlap\n")? + 1;
    let tail = &section[start..];
    let stop = tail[4..].find("\n### ").map_or(tail.len(), |at| at + 5);
    Some(&tail[..stop])
}

#[test]
fn refusal_pair_paragraph_states_declaration_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/guides/specify/guards-and-predicates.md");
    let page = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let text = subsection(&page).unwrap_or_else(|| {
        panic!(
            "`## Which branch answers` has the heading `### Two input refusals whose guards overlap`"
        )
    });
    for phrase in [
        "the first declared answers",
        "make the guards disjoint",
        "`all:`",
        "sent once more",
    ] {
        assert!(
            text.contains(phrase),
            "the subsection says {phrase:?}:\n{text}"
        );
    }
}

// ---- stored-field pairs: two `when_subject` refusals one row satisfies together ------------------

/// A `demo.order` document of the commands `names`, whose declarations are `commands`.
fn order_document(names: &[&str], commands: &str) -> String {
    let qualified = names
        .iter()
        .map(|name| format!("demo.order.{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r"format: ess/20
system: demo
version: v1
domain: demo.order
summary: Orders refused for two conditions their stored fields or their input can hold together.
types:
  - {{name: demo.order.OrderId, kind: newtype, of: Uuid}}
entities:
  - name: demo.order.Order
    identity: {{name: order_id, type: demo.order.OrderId}}
    fields:
      - {{name: quantity, type: Integer}}
      - {{name: discount, type: Integer}}
      - {{name: locked, type: Boolean}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - name: demo.order.Agent
    may: [{qualified}]
errors:
  - {{name: demo.order.BadQuantity, fields: []}}
  - {{name: demo.order.BadDiscount, fields: []}}
  - {{name: demo.order.Locked, fields: []}}
  - {{name: demo.order.Empty, fields: []}}
events:
  - name: demo.order.Placed
    fields: [{{name: order_id, type: demo.order.OrderId}}]
  - name: demo.order.Changed
    fields: [{{name: order_id, type: demo.order.OrderId}}]
commands:
{commands}components:
  - component: orders
    owns: {{domains: [demo.order]}}
    accepts: {{commands: [{qualified}]}}
    reached_by: network
"
    )
}

/// The command every model creates its rows through; `locked` is what it stores in `locked`, and
/// `refusing` has it refuse a quantity at or below zero and a discount below zero.
fn placing(locked: &str, refusing: bool) -> String {
    let refusals = if refusing {
        "      - {name: bad-quantity, when: quantity <= 0, error: demo.order.BadQuantity}\n      - {name: bad-discount, when: discount < 0, error: demo.order.BadDiscount}\n"
    } else {
        ""
    };
    format!(
        r"  - name: demo.order.Place
    input:
      - {{name: quantity, type: Integer}}
      - {{name: discount, type: Integer}}
      - {{name: locked, type: Boolean}}
    outcomes:
{refusals}      - name: placed
        creates: demo.order.Order
        instance: order_id
        sets: {{quantity: input.quantity, discount: input.discount, locked: {locked}}}
        emits: [demo.order.Placed]
        payload: {{demo.order.Placed: {{order_id: {{generated: true}}}}}}
"
    )
}

/// Model A, the shape of beyond10x/ess#472: two input refusals on a creating command.
fn model_a() -> String {
    order_document(&["Place"], &placing("input.locked", true))
}

/// Model B: the same two input refusals on an updating command, beside a `when_subject` refusal.
fn model_b() -> String {
    order_document(
        &["Place", "Amend"],
        &format!(
            "{}{}",
            placing("input.locked", false),
            r"  - name: demo.order.Amend
    input:
      - {name: order_id, type: demo.order.OrderId}
      - {name: quantity, type: Integer}
      - {name: discount, type: Integer}
    outcomes:
      - {name: bad-quantity, when: quantity <= 0, error: demo.order.BadQuantity}
      - {name: bad-discount, when: discount < 0, error: demo.order.BadDiscount}
      - name: locked
        when_subject: {predicate: locked == true}
        error: demo.order.Locked
      - name: amended
        updates: demo.order.Order
        instance: order_id
        sets: {quantity: input.quantity, discount: input.discount}
        emits: [demo.order.Changed]
        payload: {demo.order.Changed: {order_id: input.order_id}}
"
        ),
    )
}

/// Model C: two `when_subject` refusals of one phase, and a `Ship` that locks the row it accepts.
/// `placed_locked` is what `Place` stores in `locked`; `locked_first` declares `locked` before
/// `empty`.
fn model_c(placed_locked: &str, locked_first: bool) -> String {
    let empty = "      - name: empty\n        when_subject: {predicate: quantity <= 0}\n        error: demo.order.Empty\n";
    let locked = "      - name: locked\n        when_subject: {predicate: locked == true}\n        error: demo.order.Locked\n";
    let (one, other) = if locked_first {
        (locked, empty)
    } else {
        (empty, locked)
    };
    let shipped = "      - name: shipped\n        updates: demo.order.Order\n        instance: order_id\n        sets: {locked: true}\n        emits: [demo.order.Changed]\n        payload: {demo.order.Changed: {order_id: input.order_id}}\n";
    order_document(
        &["Place", "Ship"],
        &format!(
            "{}  - name: demo.order.Ship\n    input:\n      - {{name: order_id, type: demo.order.OrderId}}\n    outcomes:\n{one}{other}{shipped}",
            placing(placed_locked, false)
        ),
    )
}

const SHIP_EMPTY: &str = "demo.order.Ship/outcome/empty";

type Send = (Vec<String>, Option<String>, Option<String>);

/// Each send of the command `short` the scenario makes: the literals it sends for `fields` (`-`
/// for a field sent as a reference) and the outcome and error the next steps require.
fn sent_to(scenario: &ConformanceScenario, short: &str, fields: &[&str]) -> Vec<Send> {
    let mut out = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.name().to_string() != format!("demo.order.{short}") {
            continue;
        }
        let values = fields
            .iter()
            .map(|field| {
                let json = literal_json(input, field);
                if json.is_empty() {
                    "-".to_owned()
                } else {
                    json
                }
            })
            .collect();
        let outcome = scenario.steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        let error = scenario.steps.get(at + 2).and_then(|next| match next {
            ScenarioStep::ExpectError { error, .. } => Some(error.to_string()),
            _ => None,
        });
        out.push((values, outcome, error));
    }
    out
}

/// The literal `field` is sent as, in JSON; empty where it is sent as a reference or not at all.
fn literal_json(input: &BTreeMap<String, ScenarioValue>, field: &str) -> String {
    input
        .get(field)
        .and_then(ScenarioValue::as_literal)
        .map_or_else(String::new, |node| {
            match serde_json::to_value(node).expect("a literal is JSON") {
                // Whatever spelling a number is written with (`0`, `0.0`), the same value.
                serde_json::Value::Number(number) => number
                    .as_f64()
                    .map_or_else(|| number.to_string(), |value| value.to_string()),
                other => other.to_string(),
            }
        })
}

fn audit(text: &str) -> mutate::MutationReport {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    let files = vec![(Source::new("fixture.yaml"), raw)];
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(
        &files,
        &texts,
        &[mutate::MutantClass::PrecedenceSwap],
        || Interpreted::for_model(ir.clone()),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// The exit status `ess verify conform mutate` gives `report`: 1 with a survivor, 3 with an
/// unwitnessed or inconclusive mutant or none run, 0 otherwise.
fn exit_status(report: &mutate::MutationReport) -> u8 {
    let ran = report
        .mutants
        .iter()
        .filter(|entry| {
            !matches!(
                entry.verdict,
                mutate::Verdict::Stillborn | mutate::Verdict::Equivalent
            )
        })
        .count();
    if report.counts.survived > 0 {
        1
    } else if report.counts.unwitnessed > 0
        || report.counts.inconclusive > 0
        || ran == 0
        || report.unaudited_sites() > 0
    {
        3
    } else {
        0
    }
}

fn verdict_of(report: &mutate::MutationReport, id: &str) -> mutate::Verdict {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no site {id}:\n{}", report.render_text()))
        .verdict
}

#[test]
fn input_refusal_pairs_stay_witnessed_and_killed_on_both_paths() {
    for (model, command) in [(model_a(), "Place"), (model_b(), "Amend")] {
        let result = synthesis(&model);
        assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
        let id = format!("demo.order.{command}/outcome/bad-quantity");
        let sent = sent_to(
            scenario(&result.suite, &id),
            command,
            &["quantity", "discount"],
        );
        assert!(
            sent.contains(&(
                vec!["0".to_owned(), "-1".to_owned()],
                Some("bad-quantity".to_owned()),
                Some("demo.order.BadQuantity".to_owned())
            )),
            "{command}: the overlap {{quantity: 0, discount: -1}} requires bad-quantity: {sent:#?}"
        );
        let report = audit(&model);
        let site = format!("precedence-swap/demo.order.{command}/bad-quantity/bad-discount");
        assert_eq!(
            verdict_of(&report, &site),
            mutate::Verdict::Killed,
            "{}",
            report.render_text()
        );
        assert_eq!(exit_status(&report), 0, "{}", report.render_text());
    }
}

/// Whether `scenario` arranges the row `{quantity: 0, locked: true}` and sends `Ship` for it,
/// requiring `empty` and `Empty`.
fn sends_overlap_row(scenario: &ConformanceScenario) -> bool {
    let arranged = sent_to(scenario, "Place", &["quantity", "locked"])
        .iter()
        .any(|(values, _, _)| values == &["0".to_owned(), "true".to_owned()]);
    let refused = sent_to(scenario, "Ship", &["order_id"])
        .iter()
        .filter(|(_, outcome, error)| {
            outcome.as_deref() == Some("empty") && error.as_deref() == Some("demo.order.Empty")
        })
        .count();
    arranged && refused >= 2
}

/// Model C's suite as 0.92.0 wrote it: `empty`'s scenario cut at the first send that arranges the
/// overlap row.
fn without_overlap_send(suite: &ConformanceSuite) -> ConformanceSuite {
    let mut cut = suite.clone();
    let (_, scenario) = cut
        .scenarios
        .iter_mut()
        .find(|(key, _)| key.to_string() == SHIP_EMPTY)
        .expect("the scenario is filed");
    let at = scenario
        .steps
        .iter()
        .position(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                command.name().to_string() == "demo.order.Place"
                    && literal_json(input, "quantity") == "0"
                    && literal_json(input, "locked") == "true"
            }
            _ => false,
        })
        .expect("the overlap row is arranged");
    scenario.steps.truncate(at);
    cut
}

#[test]
fn stored_pair_overlap_is_sent_in_the_first_declared_scenario() {
    let result = synthesis(&model_c("input.locked", false));
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    let sent = scenario(&result.suite, SHIP_EMPTY);
    assert!(
        sends_overlap_row(sent),
        "`empty`'s scenario arranges {{quantity: 0, locked: true}} and requires `empty` and \
         `Empty` of it: {sent:#?}"
    );
}

#[test]
fn locked_first_target_fails_only_the_overlap_send() {
    let suite = synthesis(&model_c("input.locked", false)).suite;
    let honest = Interpreted::for_model(ir(&model_c("input.locked", false)));
    assert_eq!(not_passed(&run(&suite, &honest)), Vec::<&str>::new());
    let swapped = Interpreted::for_model(ir(&model_c("input.locked", true)));
    assert_eq!(
        not_passed(&run(&suite, &swapped)),
        vec![SHIP_EMPTY],
        "a target answering `locked` first fails `empty`'s scenario and nothing else"
    );
    assert_eq!(
        not_passed(&run(&without_overlap_send(&suite), &swapped)),
        Vec::<&str>::new(),
        "and passes every scenario of the suite 0.92.0 wrote"
    );
}

#[test]
fn stored_pair_swap_site_is_killed() {
    let report = audit(&model_c("input.locked", false));
    let site = "precedence-swap/demo.order.Ship/empty/locked";
    assert_eq!(
        verdict_of(&report, site),
        mutate::Verdict::Killed,
        "{}",
        report.render_text()
    );
    assert_eq!(exit_status(&report), 0, "{}", report.render_text());
}

#[test]
fn stored_pair_no_row_satisfies_is_noted_and_swap_is_not_a_survivor() {
    let model = model_c("false", false);
    let result = synthesis(&model);
    let noted: Vec<(String, String)> = result
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnwitnessedOverlap {
                scenario,
                first,
                other,
                ..
            } if scenario.to_string() == SHIP_EMPTY => Some((first.to_string(), other.to_string())),
            _ => None,
        })
        .collect();
    assert_eq!(
        noted,
        vec![("empty".to_owned(), "locked".to_owned())],
        "{:#?}",
        result.notes
    );
    let report = audit(&model);
    assert_eq!(report.counts.survived, 0, "{}", report.render_text());
    assert_eq!(
        verdict_of(&report, "precedence-swap/demo.order.Ship/empty/locked"),
        mutate::Verdict::Unwitnessed,
        "the pair has no send, so the swap asks the target what the baseline did: {}",
        report.render_text()
    );
}

#[test]
fn stored_pair_no_row_can_satisfy_is_not_sent_and_not_noted() {
    // `quantity == 0` and `quantity == 1` name two values of one field: the pair overlaps nowhere.
    let model = model_c("input.locked", false)
        .replace("quantity <= 0", "quantity == 0")
        .replace("locked == true", "quantity == 1");
    let result = synthesis(&model);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    let overlaps: Vec<_> = result
        .notes
        .iter()
        .filter(|note| matches!(note, Note::UnwitnessedOverlap { .. }))
        .collect();
    assert_eq!(overlaps, Vec::<&Note>::new(), "an empty overlap is no gap");
    let sent = scenario(&result.suite, SHIP_EMPTY);
    let refused = sent_to(sent, "Ship", &["order_id"])
        .iter()
        .filter(|(_, outcome, _)| outcome.as_deref() == Some("empty"))
        .count();
    assert_eq!(
        refused, 1,
        "`empty` is sent once, for its own row: {sent:#?}"
    );
}
