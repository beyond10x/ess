//! Adversary pass 2 for the generated Go runtime on suite/22–/27 (beyond10x/ess#188).
//!
//! Attacks the pass-1 correction — `eventuallyEvent` selecting the first payload-matching
//! occurrence and holding it to its shape, `expectEvent` reading the first occurrence by name and
//! requiring every named value to be carried and equal, and `containerHolds` — through the
//! recorded-transcript replay: the Rust reference runner and the Go runtime are asked about one
//! recorded behaviour, so a verdict that differs is a difference in how the suite was read.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::refs::{BindingRef, CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

fn ir(name: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new(name), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edited(suite: &ConformanceSuite, edit: impl FnOnce(&mut serde_json::Value)) -> ConformanceSuite {
    let mut document = serde_json::to_value(suite).unwrap();
    edit(&mut document);
    serde_json::from_value(document).unwrap()
}

// ---- the ledger: a binding whose consequence is observed with `eventually_event` ---------------

const LEDGER: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");
const PLACE: &str = "demo.ledger.Place";
const RECORD: &str = "demo.ledger.Record";
const RECORDED: &str = "demo.ledger.Recorded";
const BINDING: &str = "notify-ledger";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LedgerMode {
    Correct,
    /// `observe_events` answers a malformed `Recorded` first and the well-formed one after it.
    MalformedThenGood,
    /// `observe_events` answers the well-formed `Recorded` first and a malformed one after it.
    GoodThenMalformed,
    /// `observe_events` answers nothing for the first nine asks of a scenario, then the log: the
    /// consequence arrives inside the reference runner's fifty-ask budget.
    ArrivesOnTheTenthAsk,
}

#[derive(Default)]
struct LedgerState {
    forced: Option<(String, u32)>,
    log: Vec<ObservedEvent>,
    invocations: Vec<ObservedInvocation>,
    asks: u32,
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
                let event = Self::event(RECORDED, order_id);
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
        Ok(ImplementationIdentity::new("adversary-ledger-2", "1"))
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
        let mut state = self.state.borrow_mut();
        state.asks += 1;
        if self.mode == LedgerMode::ArrivesOnTheTenthAsk && state.asks < 10 {
            return Ok(Vec::new());
        }
        let malformed = ObservedEvent::new(RECORDED.parse().unwrap())
            .with("order_id", Node::Number(7_i64.into()));
        let mut answer = Vec::new();
        for event in state
            .log
            .iter()
            .filter(|event| event.event == request.event)
        {
            if event.event.to_string() != RECORDED {
                answer.push(event.clone());
                continue;
            }
            match self.mode {
                LedgerMode::Correct | LedgerMode::ArrivesOnTheTenthAsk => {
                    answer.push(event.clone());
                }
                LedgerMode::MalformedThenGood => {
                    answer.push(malformed.clone());
                    answer.push(event.clone());
                }
                LedgerMode::GoodThenMalformed => {
                    answer.push(event.clone());
                    answer.push(malformed.clone());
                }
            }
        }
        Ok(answer)
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
    ess_conformance::synthesize::synthesize(&ir("bounded-retry.yaml", LEDGER)).suite
}

/// The id of every scenario whose steps end in `eventually_event` for `Recorded`.
fn eventually_scenarios(suite: &ConformanceSuite) -> Vec<String> {
    let document = serde_json::to_value(suite).unwrap();
    document["scenarios"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, scenario)| {
            scenario["steps"]
                .as_array()
                .unwrap()
                .iter()
                .any(|step| step["step"] == "eventually_event" && step["event"] == RECORDED)
        })
        .map(|(id, _)| id.clone())
        .collect()
}

/// Several occurrences in one answer, in both orders. The reference runner selects the first
/// occurrence by name (the step's payload is empty) and holds it to the shape: malformed-first
/// fails, good-first passes. The Go runtime must agree scenario for scenario.
#[test]
fn go_selects_the_same_eventual_occurrence_as_the_reference_runner_in_either_order() {
    let suite = ledger_suite();
    let eventual = eventually_scenarios(&suite);
    assert!(
        !eventual.is_empty(),
        "the ledger suite observes Recorded eventually"
    );
    let bad_first = support_go::assert_parity(
        "adv2-ledger-malformed-first",
        &suite,
        Ledger::new(LedgerMode::MalformedThenGood),
    );
    for id in &eventual {
        assert_eq!(
            bad_first.get(id).map(String::as_str),
            Some("failed"),
            "{id}: {bad_first:?}"
        );
    }
    let good_first = support_go::assert_parity(
        "adv2-ledger-good-first",
        &suite,
        Ledger::new(LedgerMode::GoodThenMalformed),
    );
    assert!(
        support_go::not_passed(&good_first).is_empty(),
        "{good_first:?}"
    );
}

/// The consequence of a binding arrives on the tenth ask. The reference runner's budget is its
/// clock (`RunnerConfig::DEFAULT_EVENTUAL_TIMEOUT_MS` over `AdvancingClock::DEFAULT_STEP_MS`,
/// fifty asks) and it passes; the Go harness gives an `eventually` step eight attempts
/// (`NewHarness`, `attempts: 8`).
#[test]
fn go_waits_for_an_eventual_event_as_long_as_the_reference_runner_does() {
    let suite = ledger_suite();
    let (rust, replayed) = support_go::compare(
        "adv2-ledger-tenth-ask",
        &suite,
        Ledger::new(LedgerMode::ArrivesOnTheTenthAsk),
    );
    assert!(
        support_go::not_passed(&rust).is_empty(),
        "the reference runner passes a consequence observed on the tenth ask: {rust:?}"
    );
    assert_eq!(
        replayed.go.outcomes, rust,
        "per-scenario verdicts, Go (left) and Rust (right)\n{}",
        replayed.go.log
    );
}

/// The implementor's own residual, made concrete. The reference runner's `expect_event` and
/// `expect_no_event` read the last command's `direct_events` only; the Go runtime reads
/// `r.observed`, which `eventuallyEvent` appends to. Constructed by hand: neither the synthesizer
/// (eventually steps are always a scenario's last) nor an authored scenario (no eventually step)
/// writes an event assertion after an `eventually_event`.
#[test]
fn go_reads_only_the_last_commands_events_after_an_eventual_observation() {
    let suite = ledger_suite();
    let id = eventually_scenarios(&suite)
        .into_iter()
        .next()
        .expect("an eventual scenario");
    for (label, step) in [
        (
            "adv2-ledger-no-event-after-eventually",
            serde_json::json!({"step": "expect_no_event", "event": RECORDED}),
        ),
        (
            "adv2-ledger-event-after-eventually",
            serde_json::json!({"step": "expect_event", "event": RECORDED}),
        ),
    ] {
        let hand = edited(&suite, |document| {
            document["scenarios"][&id]["steps"]
                .as_array_mut()
                .unwrap()
                .push(step.clone());
        });
        let (rust, replayed) = support_go::compare(label, &hand, Ledger::new(LedgerMode::Correct));
        assert_eq!(
            replayed.go.outcomes, rust,
            "{label}: per-scenario verdicts, Go (left) and Rust (right)\n{}",
            replayed.go.log
        );
    }
}

// ---- the dialer: dotted leaves, numbers and nested optionals on an event ------------------------

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
      - {name: data, type: Optional<String>}
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
      - {name: lead_data, type: Optional<String>}
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
const LEAD_SET: &str = "demo.dialer.LeadSet";

/// What the dialer publishes on `LeadSet`, beside the correct row it stores.
#[derive(Clone, Debug)]
struct Published {
    /// Replaces the `rank` leaf of the event.
    rank: Option<Node>,
    /// Replaces the `data` leaf of the event; `Some(None)` leaves it out.
    #[allow(clippy::option_option)]
    data: Option<Option<Node>>,
}

struct Dialer {
    published: Published,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

impl Dialer {
    fn new(published: Published) -> Self {
        Self {
            published,
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
        Ok(ImplementationIdentity::new("adversary-dialer-2", "1"))
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
                if let Some(rank) = &self.published.rank {
                    published.insert("rank".to_owned(), rank.clone());
                }
                match &self.published.data {
                    None => {}
                    Some(None) => {
                        published.remove("data");
                    }
                    Some(Some(data)) => {
                        published.insert("data".to_owned(), data.clone());
                    }
                }
                SemanticCommandResult::took(outcome(&command, "lead-set")).emitting(
                    ObservedEvent::new(LEAD_SET.parse().unwrap())
                        .with("agent_id", Node::Text(id.clone()))
                        .with("lead", Node::Map(published)),
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

fn dialer_suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir("dialer.yaml", DIALER));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    synthesis.suite
}

/// The `LeadSet` assertion of `lead-set`, edited by `edit`.
fn with_lead_set_step(
    suite: &ConformanceSuite,
    edit: impl FnOnce(&mut serde_json::Value),
) -> ConformanceSuite {
    edited(suite, |document| {
        let step = document["scenarios"][SET_LEAD]["steps"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|step| {
                (step["step"] == "expect_event" || step["step"] == "expect_event_values")
                    && step["event"] == LEAD_SET
            })
            .expect("lead-set asserts LeadSet");
        as_expect_event(step);
        edit(step);
    })
}

/// The `LeadSet` assertion in the `expect_event` form the cases here edit. Where it also compares a
/// captured identity (beyond10x/ess#273) the suite writes it as `expect_event_values`, whose literal
/// values are that payload; the identity is dropped, so the step is the one these cases were
/// written against.
fn as_expect_event(step: &mut serde_json::Value) {
    if step["step"] != "expect_event_values" {
        return;
    }
    let literals: serde_json::Map<String, serde_json::Value> = step["payload"]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, value)| value["kind"] == "literal")
        .map(|(key, value)| (key.clone(), value["value"].clone()))
        .collect();
    step["step"] = serde_json::json!("expect_event");
    step["payload"] = serde_json::Value::Object(literals);
}

fn number(value: f64) -> Node {
    Node::Number(Number::new(value).unwrap())
}

fn json_number(text: &str) -> serde_json::Value {
    serde_json::from_str(text).unwrap()
}

/// Exact equality of a named value, as the reference runner's `expect_payload` applies it
/// (`carried_at(..) == Some(expected)`, `Number` compared by exact value): `1` is `1.0`, `-0` is
/// `0`, `2^53 + 1` is not `2^53`, and a nested optional required to hold a text fails on `null`.
/// Each case must give the reference verdict for `lead-set`.
#[test]
fn go_compares_named_event_values_exactly_as_the_reference_runner_does() {
    let suite = dialer_suite();
    let unchanged = Published {
        rank: None,
        data: None,
    };
    let cases: Vec<(&str, &str, serde_json::Value, Published, &str)> = vec![
        (
            "one-as-one-point-zero",
            "lead.rank",
            json_number("1"),
            Published {
                rank: Some(number(1.0)),
                ..unchanged.clone()
            },
            "passed",
        ),
        (
            "zero-as-negative-zero",
            "lead.rank",
            json_number("0"),
            Published {
                rank: Some(number(-0.0)),
                ..unchanged.clone()
            },
            "passed",
        ),
        (
            "big-integer-exact",
            "lead.rank",
            json_number("9007199254740993"),
            Published {
                rank: Some(Node::Number(9_007_199_254_740_993_i64.into())),
                ..unchanged.clone()
            },
            "passed",
        ),
        (
            "big-integer-rounded",
            "lead.rank",
            json_number("9007199254740993"),
            Published {
                rank: Some(number(9_007_199_254_740_992.0)),
                ..unchanged.clone()
            },
            "failed",
        ),
        (
            "optional-text-published-null",
            "lead.data",
            serde_json::json!("x"),
            Published {
                data: Some(Some(Node::Null)),
                ..unchanged.clone()
            },
            "failed",
        ),
        (
            "optional-text-left-out",
            "lead.data",
            serde_json::json!("x"),
            Published {
                data: Some(None),
                ..unchanged.clone()
            },
            "failed",
        ),
    ];
    for (label, key, value, published, expected) in cases {
        let hand = with_lead_set_step(&suite, |step| {
            step["payload"][key] = value.clone();
        });
        let (rust, replayed) = support_go::compare(
            &format!("adv2-dialer-{label}"),
            &hand,
            Dialer::new(published),
        );
        assert_eq!(
            rust.get(SET_LEAD).map(String::as_str),
            Some(expected),
            "{label}: the reference verdict: {rust:?}"
        );
        assert_eq!(
            replayed.go.outcomes, rust,
            "{label}: per-scenario verdicts, Go (left) and Rust (right)\n{}",
            replayed.go.log
        );
    }
}

/// Pass 1's F2, on the sibling step. `expect_event_values` reaches the same `expect_payload` in the
/// reference runner (`expect_event_values` → `expect_event`), so a dotted leaf required to be
/// `null` must be carried there as `null`. The Go runtime's `expectEventValues`
/// (`go/fixtures.go`) still reads the values through `matches`, which lets a left-out leaf stand
/// for `null`. Constructed: the `expect_event` step is rewritten into `expect_event_values` with a
/// literal `null` for `lead.data`.
#[test]
fn go_requires_a_null_leaf_on_expect_event_values_to_be_carried_as_the_reference_runner_does() {
    let hand = with_lead_set_step(&dialer_suite(), |step| {
        step["step"] = serde_json::json!("expect_event_values");
        step["payload"] = serde_json::json!({
            "lead.data": {"kind": "literal", "value": null}
        });
    });
    let (rust, replayed) = support_go::compare(
        "adv2-dialer-values-null-leaf",
        &hand,
        Dialer::new(Published {
            rank: None,
            data: Some(None),
        }),
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

// ---- containers: lists of structs, maps with non-text keys, a union with payloads ---------------

const BAG: &str = "format: ess/15
system: demo
version: v1
domain: demo.bag
types:
  - name: demo.bag.Row
    kind: struct
    fields:
      - {name: label, type: String}
  - name: demo.bag.Choice
    kind: union
    tag: kind
    variants:
      text: String
      row: demo.bag.Row
events:
  - name: demo.bag.Bagged
    fields:
      - {name: rows, type: 'List<demo.bag.Row>'}
      - {name: nested, type: 'List<List<String>>'}
      - {name: counts, type: 'Map<Integer, String>'}
      - {name: choice, type: demo.bag.Choice}
      - {name: maybe, type: 'Optional<List<String>>'}
actors:
  - {name: demo.bag.Clerk, may: [demo.bag.Bag]}
commands:
  - name: demo.bag.Bag
    input:
      - {name: rows, type: 'List<demo.bag.Row>'}
      - {name: nested, type: 'List<List<String>>'}
      - {name: counts, type: 'Map<Integer, String>'}
      - {name: choice, type: demo.bag.Choice}
      - {name: maybe, type: 'Optional<List<String>>'}
    outcomes:
      - name: bagged
        emits: [demo.bag.Bagged]
        payload:
          demo.bag.Bagged:
            rows: {generated: true}
            nested: {generated: true}
            counts: {generated: true}
            choice: {generated: true}
            maybe: {generated: true}
";

struct Bag(Vec<(&'static str, Option<Node>)>);

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

impl ConformanceTarget for Bag {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-bag", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let row = Node::Map(BTreeMap::from([("label".to_owned(), text("a"))]));
        let mut payload = BTreeMap::from([
            ("rows".to_owned(), Node::Seq(vec![row.clone(), row.clone()])),
            (
                "nested".to_owned(),
                Node::Seq(vec![Node::Seq(vec![text("x")]), Node::Seq(Vec::new())]),
            ),
            (
                "counts".to_owned(),
                Node::Map(BTreeMap::from([("1".to_owned(), text("one"))])),
            ),
            (
                "choice".to_owned(),
                Node::Map(BTreeMap::from([
                    ("kind".to_owned(), text("row")),
                    ("row".to_owned(), row),
                ])),
            ),
            ("maybe".to_owned(), Node::Seq(Vec::new())),
        ]);
        for (field, value) in &self.0 {
            match value {
                Some(value) => payload.insert((*field).to_owned(), value.clone()),
                None => payload.remove(*field),
            };
        }
        let mut event = ObservedEvent::new("demo.bag.Bagged".parse().unwrap());
        event.payload = payload;
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            request.command.clone(),
            OutcomeName::new("bagged").unwrap(),
        ))
        .emitting(event))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
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

/// Container leaves in every form the declared types allow and in forms they refuse: empty lists
/// and maps, a list of structs, a list of lists, a map declared with `Integer` keys, a union variant
/// with a struct payload, an optional list published `null` or left out, and each container
/// published as the wrong one. Both runners must give one verdict per case.
#[test]
fn go_holds_container_leaves_as_the_reference_runner_does_in_every_form() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir("bag.yaml", BAG));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, Vec<(&'static str, Option<Node>)>)> = vec![
        ("correct", vec![]),
        (
            "empty",
            vec![
                ("rows", Some(Node::Seq(Vec::new()))),
                ("nested", Some(Node::Seq(Vec::new()))),
                ("counts", Some(Node::Map(BTreeMap::new()))),
            ],
        ),
        ("maybe-null", vec![("maybe", Some(Node::Null))]),
        ("maybe-left-out", vec![("maybe", None)]),
        ("rows-null", vec![("rows", Some(Node::Null))]),
        (
            "rows-as-map",
            vec![("rows", Some(Node::Map(BTreeMap::new())))],
        ),
        (
            "counts-as-list",
            vec![("counts", Some(Node::Seq(Vec::new())))],
        ),
        ("choice-as-text", vec![("choice", Some(text("row")))]),
        (
            "choice-as-list",
            vec![("choice", Some(Node::Seq(Vec::new())))],
        ),
        (
            "choice-empty-map",
            vec![("choice", Some(Node::Map(BTreeMap::new())))],
        ),
        ("choice-left-out", vec![("choice", None)]),
        ("nested-as-text", vec![("nested", Some(text("x")))]),
    ];
    for (label, changes) in cases {
        support_go::assert_parity(&format!("adv2-bag-{label}"), &suite, Bag(changes));
    }
}
