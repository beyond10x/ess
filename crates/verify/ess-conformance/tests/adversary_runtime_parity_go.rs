//! Adversarial parity cases for the generated Go runtime on suite/22–/27 (beyond10x/ess#188).
//!
//! Each case drives a target the reference runner fails through the recorded-transcript replay and
//! requires the Go runtime to fail the same scenarios. They are the wrong targets the unit's own
//! mutants do not include.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn ir(name: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new(name), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

// ---- 1. `eventually_event` holds the arriving occurrence to its declared shape -----------------

const LEDGER: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");
const PLACE: &str = "demo.ledger.Place";
const RECORD: &str = "demo.ledger.Record";
const BINDING: &str = "notify-ledger";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LedgerMode {
    /// Up to three attempts; `rejected` ends the retry at once; every event well-formed.
    Correct,
    /// Correct, except that the `Recorded` the binding produces is observed carrying `order_id`
    /// as a number — a `demo.ledger.OrderId` is a `String`.
    EventualOrderIdIsANumber,
    /// Correct, except that the observed `Recorded` carries no `order_id` at all.
    EventualOrderIdLeftOut,
}

#[derive(Default)]
struct LedgerState {
    forced: Option<(String, u32)>,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
}

struct Ledger {
    mode: LedgerMode,
    state: RefCell<LedgerState>,
}

impl Ledger {
    fn new(mode: LedgerMode) -> Self {
        Self {
            mode,
            state: RefCell::new(LedgerState::default()),
        }
    }
    fn outcome(command: &str, outcome: &str) -> OutcomeRef {
        OutcomeRef::new(
            CommandRef::new(command.parse().unwrap()),
            outcome.parse().unwrap(),
        )
    }
    fn event(name: &str, order_id: &Node) -> ObservedEvent {
        ObservedEvent::new(name.parse().unwrap()).with("order_id", order_id.clone())
    }
    fn record(state: &mut LedgerState, order_id: &Node) -> (String, SemanticCommandResult) {
        let forced = match state.forced.as_mut() {
            Some((outcome, remaining)) if *remaining > 0 => {
                *remaining -= 1;
                Some(outcome.clone())
            }
            _ => None,
        };
        let outcome = forced.unwrap_or_else(|| "recorded".to_owned());
        let mut result = SemanticCommandResult::took(Self::outcome(RECORD, &outcome));
        match outcome.as_str() {
            "recorded" => {
                let event = Self::event("demo.ledger.Recorded", order_id);
                state.log.push(event.clone());
                result.direct_events.push(event);
            }
            "unavailable" => {
                result.error = Some(DeclaredErrorValue::new(
                    "demo.ledger.Unavailable".parse().unwrap(),
                ));
            }
            _ => {
                result.error = Some(DeclaredErrorValue::new(
                    "demo.ledger.Unknown".parse().unwrap(),
                ));
            }
        }
        (outcome, result)
    }
    fn notify(state: &mut LedgerState, placed: &ObservedEvent) {
        let order_id = placed.payload["order_id"].clone();
        for _ in 0..3 {
            state.invocations.push(
                ObservedInvocation::new(
                    BindingRef::new(ess_domain::binding::BindingName::new(BINDING).unwrap()),
                    CommandRef::new(RECORD.parse().unwrap()),
                )
                .with("order_id", order_id.clone()),
            );
            let (outcome, _) = Self::record(state, &order_id);
            if outcome == "recorded" || outcome == "rejected" {
                return;
            }
        }
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-ledger", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.state.borrow_mut() = LedgerState::default();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut state = self.state.borrow_mut();
        let order_id = request.input.get("order_id").cloned().unwrap_or(Node::Null);
        match request.command.to_string().as_str() {
            PLACE => {
                let placed = Self::event("demo.ledger.OrderPlaced", &order_id);
                state.log.push(placed.clone());
                let mut result = SemanticCommandResult::took(Self::outcome(PLACE, "placed"));
                result.direct_events.push(placed.clone());
                Self::notify(&mut state, &placed);
                Ok(result)
            }
            RECORD => Ok(Self::record(&mut state, &order_id).1),
            other => panic!("unexpected command {other}"),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .state
            .borrow()
            .log
            .iter()
            .filter(|event| event.event == request.event)
            .map(|event| {
                if event.event.to_string() != "demo.ledger.Recorded" {
                    return event.clone();
                }
                match self.mode {
                    LedgerMode::Correct => event.clone(),
                    LedgerMode::EventualOrderIdIsANumber => ObservedEvent::new(event.event.clone())
                        .with("order_id", Node::Number(7_i64.into())),
                    LedgerMode::EventualOrderIdLeftOut => ObservedEvent::new(event.event.clone()),
                }
            })
            .collect())
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = Some((request.force.outcome.to_string(), 1));
        Ok(())
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: NonZeroU32,
    ) -> Result<(), TargetError> {
        self.state.borrow_mut().forced = Some((request.force.outcome.to_string(), times.get()));
        Ok(())
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        let mut state = self.state.borrow_mut();
        let placed = state
            .log
            .iter()
            .rev()
            .find(|event| event.event == request.event)
            .cloned()
            .expect("the event was published");
        Self::notify(&mut state, &placed);
        Ok(())
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Ok(self
            .state
            .borrow()
            .invocations
            .iter()
            .filter(|invocation| {
                invocation.binding == request.binding && invocation.command == request.command
            })
            .cloned()
            .collect())
    }
}

fn ledger_suite() -> ConformanceSuite {
    let suite = ess_conformance::synthesize::synthesize(&ir("bounded-retry.yaml", LEDGER)).suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
    suite
}

/// A binding's flow and delivery scenarios end in `eventually_event` with the downstream event's
/// declared shape. The reference runner holds the occurrence that arrives to that shape
/// (`runner.rs` `eventually_event` → `expect_payload`); a target publishing it malformed fails.
/// The Go runtime must fail the same scenarios.
#[test]
fn go_holds_an_eventually_observed_event_to_its_shape_as_the_reference_runner_does() {
    let suite = ledger_suite();
    let correct = support_go::assert_parity(
        "adv-ledger-correct",
        &suite,
        Ledger::new(LedgerMode::Correct),
    );
    assert!(support_go::not_passed(&correct).is_empty(), "{correct:?}");
    for mode in [
        LedgerMode::EventualOrderIdIsANumber,
        LedgerMode::EventualOrderIdLeftOut,
    ] {
        let label = format!("adv-ledger-{mode:?}").to_lowercase();
        let (rust, replayed) = support_go::compare(&label, &suite, Ledger::new(mode));
        let rust_failed = support_go::not_passed(&rust);
        assert!(
            !rust_failed.is_empty(),
            "{mode:?}: the reference runner fails this target: {rust:?}"
        );
        assert_eq!(
            replayed.go.outcomes, rust,
            "{mode:?}: per-scenario verdicts, Go (left) and Rust (right); the reference runner \
             fails {rust_failed:?}\n{}",
            replayed.go.log
        );
    }
}

// ---- 2. #188 exactly: the nested `sets:` struct with one generated leaf, published wrong -------

const DIALER: &str = "format: ess/14
system: demo
version: v1
domain: demo.dialer
types:
  - {name: demo.dialer.AgentId, kind: newtype, of: String}
  - name: demo.dialer.Lead
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: uid, type: String}
      - {name: number, type: String}
      - {name: rank, type: Integer}
      - {name: data, type: DATA_TYPE}
entities:
  - name: demo.dialer.Membership
    identity: {name: agent_id, type: demo.dialer.AgentId}
    fields:
      - {name: lead, type: Optional<demo.dialer.Lead>}
    lifecycle: {initial: Idle, states: [Idle], terminal: [Idle]}
events:
  - name: demo.dialer.Joined
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
  - name: demo.dialer.LeadSet
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: demo.dialer.Lead}
actors:
  - {name: demo.dialer.Agent, may: [demo.dialer.Join, demo.dialer.SetLead]}
commands:
  - name: demo.dialer.Join
    outcomes:
      - name: joined
        creates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.Joined]
        payload:
          demo.dialer.Joined: {agent_id: {generated: true}}
  - name: demo.dialer.SetLead
    input:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead_id, type: String}
      - {name: lead_uid, type: String}
      - {name: lead_number, type: String}
      - {name: lead_data, type: DATA_TYPE}
    outcomes:
      - name: lead-set
        updates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.LeadSet]
        payload:
          demo.dialer.LeadSet:
            agent_id: input.agent_id
            lead:
              id: input.lead_id
              uid: input.lead_uid
              number: input.lead_number
              rank: {generated: true}
              data: input.lead_data
        sets:
          lead:
            id: input.lead_id
            uid: input.lead_uid
            number: input.lead_number
            rank: {generated: true}
            data: input.lead_data
views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: Optional<demo.dialer.Lead>}
";

const SET_LEAD: &str = "demo.dialer.SetLead/outcome/lead-set";

fn dialer(data_type: &str) -> String {
    DIALER.replace("DATA_TYPE", data_type)
}

fn dialer_suite(data_type: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir("dialer.yaml", &dialer(data_type)));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
    synthesis.suite
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DialerMode {
    Correct,
    /// The generated `rank` leaf is published as text.
    RankAsText,
    /// The generated `rank` leaf is left out of the event.
    RankLeftOut,
    /// The event's `lead` is published as `null` — `demo.dialer.Lead` is not `Optional` there.
    LeadNullOnTheEvent,
    /// The event's `lead` is a string, not a struct.
    LeadAsText,
    /// The event's `lead` carries no `data` key at all.
    DataLeftOutOfTheEvent,
}

struct Dialer {
    mode: DialerMode,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Dialer {
    fn new(mode: DialerMode) -> Self {
        Self {
            mode,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Dialer {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-dialer", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let mut rows = self.rows.borrow_mut();
        let input = |name: &str| request.input.get(name).cloned().unwrap_or(Node::Null);
        let result = match command.to_string().as_str() {
            "demo.dialer.Join" => {
                let id = format!("agent-{}", self.minted.get());
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("agent_id".to_owned(), Node::Text(id.clone())),
                        ("lead".to_owned(), Node::Null),
                    ]),
                );
                SemanticCommandResult::took(outcome(&command, "joined")).emitting(
                    ObservedEvent::new("demo.dialer.Joined".parse().unwrap())
                        .with("agent_id", Node::Text(id)),
                )
            }
            "demo.dialer.SetLead" => {
                let Some(Node::Text(id)) = request.input.get("agent_id") else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let Some(row) = rows.get_mut(id) else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let lead = BTreeMap::from([
                    ("id".to_owned(), input("lead_id")),
                    ("uid".to_owned(), input("lead_uid")),
                    ("number".to_owned(), input("lead_number")),
                    ("rank".to_owned(), Node::Number(0_i64.into())),
                    ("data".to_owned(), input("lead_data")),
                ]);
                row.insert("lead".to_owned(), Node::Map(lead.clone()));
                let mut published = lead;
                let published = match self.mode {
                    DialerMode::Correct => Node::Map(published),
                    DialerMode::RankAsText => {
                        published.insert("rank".to_owned(), Node::Text("0".to_owned()));
                        Node::Map(published)
                    }
                    DialerMode::RankLeftOut => {
                        published.remove("rank");
                        Node::Map(published)
                    }
                    DialerMode::LeadNullOnTheEvent => Node::Null,
                    DialerMode::LeadAsText => Node::Text("lead".to_owned()),
                    DialerMode::DataLeftOutOfTheEvent => {
                        published.remove("data");
                        Node::Map(published)
                    }
                };
                SemanticCommandResult::took(outcome(&command, "lead-set")).emitting(
                    ObservedEvent::new("demo.dialer.LeadSet".parse().unwrap())
                        .with("agent_id", Node::Text(id.clone()))
                        .with("lead", published),
                )
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// #188's shape exactly, with the generated leaf (and the struct around it) published wrong in each
/// way the declared shape refuses. Every one must fail `lead-set` on both runners.
#[test]
fn issue_188_go_fails_every_wrong_generated_leaf_the_reference_runner_fails() {
    let suite = dialer_suite("String");
    let correct = support_go::assert_parity(
        "adv-dialer-correct",
        &suite,
        Dialer::new(DialerMode::Correct),
    );
    assert!(support_go::not_passed(&correct).is_empty(), "{correct:?}");
    for mode in [
        DialerMode::RankAsText,
        DialerMode::RankLeftOut,
        DialerMode::LeadNullOnTheEvent,
        DialerMode::LeadAsText,
    ] {
        let verdicts = support_go::assert_parity(
            &format!("adv-dialer-{mode:?}").to_lowercase(),
            &suite,
            Dialer::new(mode),
        );
        assert_eq!(
            verdicts.get(SET_LEAD).map(String::as_str),
            Some("failed"),
            "{mode:?}: {verdicts:?}"
        );
    }
}

// ---- 3. An expected `null` at a dotted leaf of an event payload --------------------------------

/// The reference runner's `expect_payload` requires a value named for a leaf path to be *carried*
/// there (`carried_at(..) == Some(expected)`), `null` included. The Go runtime's `expectEvent`
/// reads the same value through `matches`, which lets an unreachable leaf path stand for `null`
/// (the rule the reference runner applies to view rows and to selecting an eventual event, not to a
/// payload's values). Constructed: the suite is the synthesized one with `lead.data` required to be
/// `null` on the event, which is what an `Optional` input supplied as `null` produces.
#[test]
fn go_requires_a_null_leaf_value_on_an_event_to_be_carried_as_the_reference_runner_does() {
    let synthesized = dialer_suite("Optional<String>");
    let mut document = serde_json::to_value(&synthesized).unwrap();
    let steps = document["scenarios"][SET_LEAD]["steps"]
        .as_array_mut()
        .unwrap();
    let step = steps
        .iter_mut()
        .find(|step| step["step"] == "expect_event" && step["event"] == "demo.dialer.LeadSet")
        .expect("lead-set asserts LeadSet");
    step["payload"]["lead.data"] = serde_json::Value::Null;
    let suite: ConformanceSuite = serde_json::from_value(document).unwrap();

    let (rust, replayed) = support_go::compare(
        "adv-dialer-null-leaf",
        &suite,
        Dialer::new(DialerMode::DataLeftOutOfTheEvent),
    );
    assert_eq!(
        rust.get(SET_LEAD).map(String::as_str),
        Some("failed"),
        "the reference runner refuses a leaf left out where null is required: {rust:?}"
    );
    assert_eq!(
        replayed.go.outcomes, rust,
        "per-scenario verdicts, Go (left) and Rust (right)\n{}",
        replayed.go.log
    );
}

// ---- 4. The emitted package is gofmt-clean and vets ---------------------------------------------

fn run_in(directory: &std::path::Path, program: &str, args: &[&str]) -> (bool, String) {
    let output = std::process::Command::new(program)
        .args(args)
        .env("GOWORK", "off")
        .current_dir(directory)
        .output()
        .unwrap_or_else(|error| panic!("{program}: {error}"));
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

#[test]
fn the_emitted_suite_26_package_is_gofmt_clean_and_passes_go_vet() {
    for (label, suite) in [
        ("adv-fmt-dialer", dialer_suite("String")),
        ("adv-fmt-ledger", ledger_suite()),
    ] {
        let directory = support_go::package(label, &suite, &[]);
        let (_, unformatted) = run_in(&directory, "gofmt", &["-l", "essconform"]);
        let (vetted, vet) = run_in(&directory, "go", &["vet", "./essconform"]);
        std::fs::remove_dir_all(&directory).unwrap();
        assert_eq!(unformatted.trim(), "", "{label}: gofmt -l lists files");
        assert!(vetted, "{label}: go vet:\n{vet}");
    }
}
