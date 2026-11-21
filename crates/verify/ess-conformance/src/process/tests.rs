//! `ess-conformance-target/1` held from both ends: every operation through [`serve`], exact numbers,
//! closed answers, every fault kind, the lifecycle and a full billing run.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;
use std::sync::OnceLock;

use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use ess_primitives::facts::Number;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use ess_primitives::time::{Rfc3339Instant, Timestamp};

use super::*;
use crate::periodic::{Closed, Fact, Observation, Observe, Open, Opened, Record, Scope};
use crate::reading::ReadingObservationRequest;
use crate::scenario::{ActorRef, Elapsed, ScenarioId};
use crate::target::{
    AbsentInputRequest, AwaitAnswer, AwaitRequest, Deadline, DeclaredErrorValue,
    ElapsedObservation, ElapsedObservationRequest, EntitySetupRequest, EventDeliveryRequest,
    EventObservationRequest, ExternalOutcomeControl, InstantMark, InvocationObservationRequest,
    NowHold, ObservedEvent, ObservedInvocation, OrderedScan, OrderedScanRequest, RedeliveryRequest,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
};
use ess_domain::reading::{
    FormatterObservation, OffsetAuthority, ReadingContract, ReadingEncoding, ReadingEvidence,
    ReadingOrigin, ReadingRole, SourceEpoch,
};

const TIMEOUT_MS: u64 = 30_000;
/// 2^53 + 1, the first integer binary64 does not hold.
const BIG: &str = "9007199254740993";
/// A decimal with more digits than binary64 keeps.
const LONG: &str = "0.1000000000000000000001";
/// An integer past 64 bits.
const WIDE: &str = "123456789012345678901234567890";
const SECRET: &str = "s3cr3t-marker";

fn exact(token: &str) -> Node {
    Node::Number(Number::from_json_token(token).expect(token))
}

fn name<T: serde::de::DeserializeOwned>(text: &str) -> T {
    serde_json::from_value(serde_json::Value::String(text.to_owned())).expect(text)
}

fn de<T: serde::de::DeserializeOwned>(json: &str) -> T {
    serde_json::from_str(json).expect(json)
}

fn values() -> BTreeMap<String, Node> {
    BTreeMap::from([
        ("big".to_owned(), exact(BIG)),
        ("long".to_owned(), exact(LONG)),
        ("wide".to_owned(), exact(WIDE)),
        ("text".to_owned(), Node::Text("t".to_owned())),
        (
            "list".to_owned(),
            Node::Seq(vec![exact(LONG), Node::Null, Node::Bool(true)]),
        ),
    ])
}

fn correlation() -> CorrelationId {
    CorrelationId::new("billing-c-000001").unwrap()
}

fn context(scenario: &str) -> ScenarioContext {
    ScenarioContext::new(
        ScenarioId::parse(scenario).unwrap(),
        CorrelationId::new(format!("c-{}", scenario.len())).unwrap(),
    )
}

fn first() -> ScenarioContext {
    context("billing.invoice.CreateInvoice/outcome/accepted")
}

fn second() -> ScenarioContext {
    context("billing.invoice.CreateInvoice/outcome/rejected")
}

fn third() -> ScenarioContext {
    context("billing.invoice.IssueInvoice/outcome/issued")
}

fn deadline() -> Deadline {
    Deadline::at(Timestamp::from_epoch_millis(1_700_000_005_300))
}

fn billing() -> &'static crate::AdmittedSuite {
    static SUITE: OnceLock<crate::AdmittedSuite> = OnceLock::new();
    SUITE.get_or_init(|| {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../suites/generated/billing/suite.json"
        );
        crate::AdmittedSuite::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
    })
}

fn suite() -> HandshakeSuite {
    HandshakeSuite::of(billing())
}

fn periodic_check() -> crate::periodic::Check {
    let text = include_str!("../../../../specify/ess-domain/tests/fixtures/periodic.yaml");
    let spec = ess_domain::Specification::assemble([(
        ess_domain::system::Source::new("periodic.yaml"),
        ess_domain::RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = ess_compiler::source::SourceMap::new();
    sources.insert("periodic.yaml", text);
    let ir = ess_compiler::resolve::compile_locating(&spec, &sources, &["periodic.yaml"]).unwrap();
    crate::synthesize::synthesize(&ir)
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| scenario.steps.iter())
        .find_map(|step| match step {
            crate::scenario::ScenarioStep::CheckPeriodic { check } => Some(check.clone()),
            _ => None,
        })
        .expect("the periodic fixture synthesizes a periodic check")
}

// ---- peers ---------------------------------------------------------------------------------------

/// A scripted misbehaviour for the `nth` request of one operation.
enum Reply {
    Line(Box<dyn Fn(u64) -> Vec<u8>>),
    Fault(PeerFault),
}

fn line(text: &str) -> Reply {
    let text = text.to_owned();
    Reply::Line(Box::new(move |_| text.clone().into_bytes()))
}

/// A peer that answers through [`serve`] over a target in memory, unless told otherwise.
struct Served<'t, T: ConformanceTarget> {
    target: &'t T,
    answer: Option<Result<Vec<u8>, PeerFault>>,
    log: std::rc::Rc<RefCell<Vec<(String, String)>>>,
    counts: BTreeMap<String, usize>,
    overrides: Vec<(String, usize, Reply)>,
    respawn_fault: Option<PeerFault>,
    respawns: std::rc::Rc<RefCell<usize>>,
    kills: std::rc::Rc<RefCell<usize>>,
}

impl<'t, T: ConformanceTarget> Served<'t, T> {
    fn new(target: &'t T) -> Self {
        Self {
            target,
            answer: None,
            log: std::rc::Rc::default(),
            counts: BTreeMap::new(),
            overrides: Vec::new(),
            respawn_fault: None,
            respawns: std::rc::Rc::default(),
            kills: std::rc::Rc::default(),
        }
    }

    fn on(mut self, op: &str, nth: usize, reply: Reply) -> Self {
        self.overrides.push((op.to_owned(), nth, reply));
        self
    }
}

impl<T: ConformanceTarget> Peer for Served<'_, T> {
    fn send(&mut self, line: &str) -> Result<(), PeerFault> {
        let request: serde_json::Value = serde_json::from_str(line).unwrap();
        let op = request["op"].as_str().unwrap().to_owned();
        let id = request["id"].as_u64().unwrap();
        self.log.borrow_mut().push((op.clone(), line.to_owned()));
        let count = self.counts.entry(op.clone()).or_default();
        *count += 1;
        let count = *count;
        if let Some((_, _, reply)) = self
            .overrides
            .iter()
            .find(|(name, nth, _)| *name == op && *nth == count)
        {
            self.answer = Some(match reply {
                Reply::Line(make) => Ok(make(id)),
                Reply::Fault(fault) => Err(*fault),
            });
            return Ok(());
        }
        let mut out = Vec::new();
        serve(self.target, format!("{line}\n").as_bytes(), &mut out).unwrap();
        assert_eq!(out.pop(), Some(b'\n'));
        self.answer = Some(Ok(out));
        Ok(())
    }

    fn receive(&mut self, _limit_ms: u64) -> Result<Vec<u8>, PeerFault> {
        self.answer.take().unwrap_or(Err(PeerFault::Timeout))
    }

    fn kill(&mut self) {
        *self.kills.borrow_mut() += 1;
    }

    fn respawn(&mut self) -> Result<(), PeerFault> {
        *self.respawns.borrow_mut() += 1;
        self.respawn_fault.map_or(Ok(()), Err)
    }
}

// ---- a target answering every operation from its request ---------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Seen {
    Fixture(ScenarioContext, crate::fixtures::Contract),
    Open(Open),
    Observe(Observe),
    Close(Scope),
    Reading(ReadingObservationRequest),
    Entity(EntitySetupRequest),
    Begin(ScenarioContext),
    Command(SemanticCommandRequest),
    Absent(AbsentInputRequest),
    View(SemanticViewRequest),
    ViewAs(SemanticViewRequest, ActorRef),
    ViewAnonymous(SemanticViewRequest),
    Events(EventObservationRequest),
    Force(ExternalOutcomeControl),
    ForceTimes(ExternalOutcomeControl, NonZeroU32),
    Redeliver(RedeliveryRequest),
    Deliver(EventDeliveryRequest),
    Invocations(InvocationObservationRequest),
    Mark(InstantMark),
    Hold(NowHold),
    Elapsed(ElapsedObservationRequest),
    Scan(OrderedScanRequest),
    ScanAs(OrderedScanRequest, ActorRef),
    ScanAnonymous(OrderedScanRequest),
    Await(AwaitRequest),
    End(ScenarioContext),
}

#[derive(Default)]
struct Echo {
    log: RefCell<Vec<Seen>>,
}

impl Echo {
    fn saw(&self, seen: Seen) {
        self.log.borrow_mut().push(seen);
    }
}

fn outcome() -> crate::scenario::OutcomeRef {
    de(r#"{"command":"billing.invoice.CreateInvoice","outcome":"accepted"}"#)
}

impl ConformanceTarget for Echo {
    fn fixture_values(
        &self,
        scenario: &ScenarioContext,
        contract: &crate::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        self.saw(Seen::Fixture(scenario.clone(), contract.clone()));
        Ok(values())
    }
    fn open_periodic(&self, request: Open) -> Result<Opened, TargetError> {
        self.saw(Seen::Open(request.clone()));
        Ok(Opened {
            scope: Scope {
                correlation: request.mark.correlation,
                binding: request.check.binding,
                lifetime: "l-1".to_owned(),
            },
            anchor_ms: 5,
            context: values(),
        })
    }
    fn observe_periodic(&self, request: Observe) -> Result<Observation, TargetError> {
        self.saw(Seen::Observe(request.clone()));
        Ok(Observation {
            elapsed_ms: request.elapsed.hold.millis(),
            complete_through_ms: request.elapsed.hold.millis(),
            cursor: request.after + 2,
            records: vec![
                Record {
                    scope: request.scope.clone(),
                    fact: Fact::Received {
                        ordinal: 1,
                        due_ms: 1000,
                        at_ms: 1000,
                        eligible: true,
                    },
                },
                Record {
                    scope: request.scope,
                    fact: Fact::Invoked {
                        ordinal: 1,
                        at_ms: 1001,
                        invocation: "i-1".to_owned(),
                        command: name("billing.invoice.CreateInvoice"),
                        read: values(),
                        input: values(),
                    },
                },
            ],
        })
    }
    fn close_periodic(&self, scope: Scope) -> Result<Closed, TargetError> {
        self.saw(Seen::Close(scope.clone()));
        Ok(Closed { scope, at_ms: 9 })
    }
    fn observe_clock_reading(
        &self,
        request: ReadingObservationRequest,
    ) -> Result<ReadingEvidence, TargetError> {
        self.saw(Seen::Reading(request.clone()));
        Ok(ReadingEvidence {
            correlation: request.correlation.to_string(),
            occurrence: request.reading.occurrence_key(),
            source: Some(SourceEpoch {
                process_instance: "p-1".to_owned(),
                epoch: "e-1".to_owned(),
            }),
            origin: ReadingRole::ProducerProcess,
            formatter: FormatterObservation::FixedOffset { minutes: -60 },
        })
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.saw(Seen::Entity(request));
        Ok(())
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("echo", "1.0.0"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.saw(Seen::Begin(scenario.clone()));
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.saw(Seen::Command(request.clone()));
        Ok(SemanticCommandResult {
            outcome: Some(outcome()),
            error: Some(DeclaredErrorValue {
                error: name("billing.invoice.InvalidAmount"),
                fields: request.input.clone(),
            }),
            consistency: Some(ConsistencyToken::new("7").unwrap()),
            direct_events: vec![ObservedEvent {
                event: name("billing.invoice.InvoiceCreated"),
                payload: request.input.clone(),
                correlation: Some(request.correlation.clone()),
                sequence: Some(u64::MAX),
            }],
            response: Some(request.input),
        })
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.saw(Seen::Absent(request));
        Ok(SemanticCommandResult::undeclared())
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.saw(Seen::View(request.clone()));
        Ok(SemanticViewResult::of([request.params, values()]).with_total(2))
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        self.saw(Seen::ViewAs(request, reader.clone()));
        Ok(SemanticViewResult::of([BTreeMap::from([(
            "reader".to_owned(),
            Node::Text(reader.to_string()),
        )])]))
    }
    fn query_view_anonymous(
        &self,
        request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        self.saw(Seen::ViewAnonymous(request));
        Err(TargetError::not_granted(None::<String>))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.saw(Seen::Events(request.clone()));
        Ok(vec![
            ObservedEvent::new(request.event).with("x", exact(LONG))
        ])
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.saw(Seen::Force(request));
        Ok(())
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: NonZeroU32,
    ) -> Result<(), TargetError> {
        self.saw(Seen::ForceTimes(request, times));
        Ok(())
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.saw(Seen::Redeliver(request));
        Ok(())
    }
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        self.saw(Seen::Deliver(request));
        Ok(())
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.saw(Seen::Invocations(request.clone()));
        Ok(vec![ObservedInvocation {
            binding: request.binding,
            command: request.command,
            input: values(),
        }])
    }
    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.saw(Seen::Mark(request));
        Ok(())
    }
    fn hold_now(&self, request: NowHold) -> Result<(), TargetError> {
        self.saw(Seen::Hold(request));
        Ok(())
    }
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        self.saw(Seen::Elapsed(request.clone()));
        Ok(ElapsedObservation {
            elapsed_ms: request.hold.millis() + 4,
            published: 1,
        })
    }
    fn scan_view(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        self.saw(Seen::Scan(request.clone()));
        Ok(OrderedScan {
            produced: request.stop_after,
            halted: true,
        })
    }
    fn scan_view_as(
        &self,
        request: OrderedScanRequest,
        reader: &ActorRef,
    ) -> Result<OrderedScan, TargetError> {
        self.saw(Seen::ScanAs(request, reader.clone()));
        Ok(OrderedScan {
            produced: 0,
            halted: false,
        })
    }
    fn scan_view_anonymous(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        self.saw(Seen::ScanAnonymous(request));
        Err(TargetError::unavailable("scan", "down"))
    }
    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        self.saw(Seen::Await(request.clone()));
        Ok(if request.within.is_some() {
            AwaitAnswer::Reached {
                result: SemanticCommandResult::took(outcome()),
                input: request.compared,
                elapsed_ms: Some(3),
            }
        } else {
            AwaitAnswer::NotIssued { elapsed_ms: None }
        })
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.saw(Seen::End(scenario.clone()));
        Ok(())
    }
}

// ---- every request, once ------------------------------------------------------------------------

fn command_request() -> SemanticCommandRequest {
    SemanticCommandRequest {
        command: name("billing.invoice.CreateInvoice"),
        actor: Some(name("billing.invoice.Clerk")),
        caller: Some(values()),
        input: values(),
        correlation: correlation(),
    }
}

fn view_request() -> SemanticViewRequest {
    SemanticViewRequest {
        view: name("billing.invoice.InvoiceById"),
        params: values(),
        consistency: QueryConsistency::at_least(ConsistencyToken::new("1").unwrap()),
        correlation: correlation(),
        deadline: deadline(),
    }
}

fn scan_request() -> OrderedScanRequest {
    OrderedScanRequest {
        view: name("billing.invoice.OutstandingInvoices"),
        params: BTreeMap::new(),
        stop_after: 1,
        consistency: QueryConsistency::Current,
        correlation: correlation(),
        deadline: deadline(),
    }
}

fn await_request(within: Option<Elapsed>) -> AwaitRequest {
    AwaitRequest {
        command: name("billing.invoice.ExpireInvoice"),
        instance: exact(WIDE),
        within,
        deadline: deadline(),
        compared: values(),
        correlation: correlation(),
    }
}

fn scope() -> Scope {
    Scope {
        correlation: correlation(),
        binding: name("poll-mailbox"),
        lifetime: "l-1".to_owned(),
    }
}

fn elapsed_request(hold: u32) -> ElapsedObservationRequest {
    ElapsedObservationRequest {
        instant: name("sent"),
        hold: Elapsed::seconds(hold),
        watching: Some(name("billing.email.DeliveryEscalated")),
        correlation: correlation(),
    }
}

/// Each operation's answer, comparable.
#[derive(Debug, PartialEq)]
enum Answer {
    Values(Result<BTreeMap<String, Node>, TargetError>),
    Opened(Result<Opened, TargetError>),
    Observation(Result<Observation, TargetError>),
    Closed(Result<Closed, TargetError>),
    Reading(Result<ReadingEvidence, TargetError>),
    Unit(Result<(), TargetError>),
    Identity(Result<ImplementationIdentity, TargetError>),
    Command(Result<SemanticCommandResult, TargetError>),
    View(Result<SemanticViewResult, TargetError>),
    Events(Result<Vec<ObservedEvent>, TargetError>),
    Invocations(Result<Vec<ObservedInvocation>, TargetError>),
    Elapsed(Result<ElapsedObservation, TargetError>),
    Scan(Result<OrderedScan, TargetError>),
    Await(Result<AwaitAnswer, TargetError>),
}

/// Calls every operation once, in one scenario.
#[allow(clippy::too_many_lines)]
fn exercise(target: &impl ConformanceTarget) -> Vec<Answer> {
    let check = periodic_check();
    let contract: crate::fixtures::Contract =
        de(r#"{"fields":[{"name":"principal-id","type":"Uuid"}],"declarations":{}}"#);
    let reader: ActorRef = name("billing.invoice.Clerk");
    let reading = ReadingObservationRequest {
        reading: crate::reading::ReadingReference {
            event: name("acme.audit.Stamped"),
            occurrence: 0,
            field: "at".to_owned(),
            declared_type: name("acme.audit.Instant"),
            contract: ReadingContract {
                encoding: ReadingEncoding::UnixSeconds,
                origins: vec![ReadingOrigin {
                    role: ReadingRole::ProducerProcess,
                    offset: OffsetAuthority::EncodingDefinedEpoch,
                }],
            },
        },
        correlation: correlation(),
    };
    let control = ExternalOutcomeControl {
        force: outcome(),
        correlation: correlation(),
    };
    vec![
        Answer::Identity(target.identity()),
        Answer::Values(target.fixture_values(&first(), &contract)),
        Answer::Unit(target.begin_scenario(&first())),
        Answer::Opened(target.open_periodic(Open {
            check,
            mark: InstantMark {
                instant: name("opened"),
                correlation: correlation(),
            },
        })),
        Answer::Observation(target.observe_periodic(Observe {
            scope: scope(),
            elapsed: elapsed_request(10),
            after: 0,
        })),
        Answer::Closed(target.close_periodic(scope())),
        Answer::Reading(target.observe_clock_reading(reading)),
        Answer::Unit(target.establish_entity(EntitySetupRequest {
            entity: name("billing.invoice.Invoice"),
            identity: exact(WIDE),
            fields: values(),
            state: ess_domain::entity::StateName::new("Draft").unwrap(),
            correlation: correlation(),
        })),
        Answer::Command(target.execute_command(command_request())),
        Answer::Command(target.execute_command_without_input(AbsentInputRequest {
            command: name("billing.invoice.CreateInvoice"),
            actor: None,
            caller: None,
            correlation: correlation(),
        })),
        Answer::View(target.query_view(view_request())),
        Answer::View(target.query_view_as(view_request(), &reader)),
        Answer::View(target.query_view_anonymous(view_request())),
        Answer::Events(target.observe_events(EventObservationRequest {
            event: name("billing.invoice.InvoiceCreated"),
            correlation: correlation(),
            deadline: deadline(),
        })),
        Answer::Unit(target.configure_external_outcome(control.clone())),
        Answer::Unit(
            target.configure_external_outcome_repeatedly(control, NonZeroU32::new(3).unwrap()),
        ),
        Answer::Unit(target.redeliver_event(RedeliveryRequest {
            event: name("billing.invoice.InvoiceCreated"),
            correlation: correlation(),
        })),
        Answer::Unit(target.deliver_event(EventDeliveryRequest {
            event: name("acme.inbound.MessageReceived"),
            authority: "mailbox".to_owned(),
            payload: values(),
            context: values(),
            correlation: correlation(),
        })),
        Answer::Invocations(target.observe_invocations(InvocationObservationRequest {
            binding: name("notify-on-invoice-created"),
            command: name("billing.email.SendEmail"),
            correlation: correlation(),
            deadline: deadline(),
        })),
        Answer::Unit(target.mark_instant(InstantMark {
            instant: name("sent"),
            correlation: correlation(),
        })),
        Answer::Unit(target.hold_now(NowHold {
            instant: Rfc3339Instant::parse_rfc3339("2026-11-21T10:00:00Z").unwrap(),
            correlation: correlation(),
        })),
        Answer::Elapsed(target.observe_elapsed(elapsed_request(20))),
        Answer::Scan(target.scan_view(scan_request())),
        Answer::Scan(target.scan_view_as(scan_request(), &reader)),
        Answer::Scan(target.scan_view_anonymous(scan_request())),
        Answer::Await(target.await_outcome(await_request(Some(Elapsed::seconds(60))))),
        Answer::Await(target.await_outcome(await_request(None))),
        Answer::Unit(target.end_scenario(&first())),
    ]
}

fn connect<T: ConformanceTarget>(peer: Served<'_, T>) -> ProcessTarget<Served<'_, T>> {
    ProcessTarget::connect(peer, suite(), TIMEOUT_MS).unwrap()
}

#[test]
fn every_operation_round_trips_through_serve() {
    let direct = Echo::default();
    let served = Echo::default();
    let peer = Served::new(&served);
    let log = peer.log.clone();
    let target = connect(peer);

    let expected = exercise(&direct);
    assert_eq!(exercise(&target), expected);
    assert_eq!(*served.log.borrow(), *direct.log.borrow());

    let sent: BTreeSet<Op> = log
        .borrow()
        .iter()
        .map(|(op, _)| Op::from_name(op).unwrap())
        .collect();
    assert_eq!(sent, Op::ALL.iter().copied().collect::<BTreeSet<_>>());
    assert_eq!(Op::ALL.len(), 27);
}

#[test]
fn the_handshake_names_the_versions_read_and_the_suite() {
    let echo = Echo::default();
    let peer = Served::new(&echo);
    let log = peer.log.clone();
    let target = connect(peer);
    assert_eq!(
        target.implementation(),
        &ImplementationIdentity::new("echo", "1.0.0")
    );
    let handshake: serde_json::Value = serde_json::from_str(&log.borrow()[0].1).unwrap();
    assert_eq!(handshake["id"], 0);
    assert_eq!(handshake["op"], "handshake");
    assert_eq!(handshake["limit_ms"], 30_000);
    assert_eq!(
        handshake["request"]["reads"],
        serde_json::json!(["ess-conformance-target/1"])
    );
    assert_eq!(
        handshake["request"]["suite"]["suite_digest"],
        billing().digest()
    );
    assert_eq!(
        handshake["request"]["suite"]["system"],
        billing().suite().provenance.system
    );
}

/// Each borrowed wire copy writes the members its type's own `Serialize` writes.
#[test]
#[allow(clippy::too_many_lines)]
fn wire_requests_spell_the_derived_members() {
    fn same(wire: &impl serde::Serialize, derived: &impl serde::Serialize) {
        let wire: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(wire).unwrap()).unwrap();
        let derived: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(derived).unwrap()).unwrap();
        assert_eq!(wire, derived);
    }
    // No numbers: the derived `Serialize` writes `Number::from(7)` as `7.0`, the exact writer as
    // `7` (one value, two spellings); this test pins member names, the exactness tests numbers.
    let plain = BTreeMap::from([
        (
            "n".to_owned(),
            Node::Seq(vec![Node::Bool(true), Node::Null]),
        ),
        ("s".to_owned(), Node::Text("x".to_owned())),
    ]);
    let mut command = command_request();
    command.input.clone_from(&plain);
    command.caller = None;
    same(&wire::CommandRequest::of(&command), &command);
    command.caller = Some(plain.clone());
    same(&wire::CommandRequest::of(&command), &command);
    let absent = AbsentInputRequest {
        command: command.command.clone(),
        actor: None,
        caller: Some(plain.clone()),
        correlation: correlation(),
    };
    same(&wire::AbsentInputWire::of(&absent), &absent);
    let mut view = view_request();
    view.params.clone_from(&plain);
    same(&wire::ViewRequest::of(&view, None), &view);
    view.params.clear();
    same(&wire::ViewRequest::of(&view, None), &view);
    let mut scan = scan_request();
    same(&wire::ScanRequest::of(&scan, None), &scan);
    scan.params.clone_from(&plain);
    same(&wire::ScanRequest::of(&scan, None), &scan);
    let delivery = EventDeliveryRequest {
        event: name("acme.inbound.MessageReceived"),
        authority: "mailbox".to_owned(),
        payload: plain.clone(),
        context: plain.clone(),
        correlation: correlation(),
    };
    same(&wire::DeliveryRequest::of(&delivery), &delivery);
    let mut awaited = await_request(Some(Elapsed::seconds(1)));
    awaited.instance = Node::Text("i".to_owned());
    awaited.compared.clone_from(&plain);
    same(&wire::AwaitWire::of(&awaited), &awaited);
    let entity = EntitySetupRequest {
        entity: name("billing.invoice.Invoice"),
        identity: Node::Text("i".to_owned()),
        fields: plain.clone(),
        state: ess_domain::entity::StateName::new("Draft").unwrap(),
        correlation: correlation(),
    };
    same(&wire::EntityRequest::of(&entity), &entity);

    let result = SemanticCommandResult::took(outcome())
        .with_error(DeclaredErrorValue {
            error: name("billing.invoice.InvalidAmount"),
            fields: plain.clone(),
        })
        .with_consistency(ConsistencyToken::new("1").unwrap())
        .emitting(
            ObservedEvent::new(name("billing.invoice.InvoiceCreated"))
                .with("n", plain["n"].clone()),
        );
    same(&wire::ResultWire::of(&result), &result);
    let view_result = SemanticViewResult::of([plain.clone()]);
    same(&wire::ViewResultWire::of(&view_result), &view_result);
    same(
        &wire::ViewResultWire::of(&view_result.clone().with_total(4)),
        &view_result.with_total(4),
    );
    let invocation = ObservedInvocation {
        binding: name("notify-on-invoice-created"),
        command: name("billing.email.SendEmail"),
        input: plain.clone(),
    };
    same(&wire::InvocationWire::of(&invocation), &invocation);
    for answer in [
        AwaitAnswer::Reached {
            result: result.clone(),
            input: plain.clone(),
            elapsed_ms: Some(1),
        },
        AwaitAnswer::NotIssued { elapsed_ms: None },
    ] {
        same(&wire::AwaitAnswerWire::of(&answer), &answer);
    }
    let opened = Opened {
        scope: scope(),
        anchor_ms: 1,
        context: plain.clone(),
    };
    same(&wire::OpenedWire::of(&opened), &opened);
    let observation = Observation {
        elapsed_ms: 1,
        complete_through_ms: 1,
        cursor: 1,
        records: vec![
            Record {
                scope: scope(),
                fact: Fact::Invoked {
                    ordinal: 1,
                    at_ms: 1,
                    invocation: "i".to_owned(),
                    command: name("billing.email.SendEmail"),
                    read: plain.clone(),
                    input: plain,
                },
            },
            Record {
                scope: scope(),
                fact: Fact::Completed {
                    ordinal: 1,
                    at_ms: 2,
                },
            },
        ],
    };
    same(&wire::ObservationWire::of(&observation), &observation);
}

#[test]
fn numbers_past_binary64_travel_exactly_both_ways() {
    // The hazard: `Number`'s own `Serialize` writes the nearest binary64 of this decimal.
    assert_ne!(serde_json::to_string(&exact(LONG)).unwrap(), LONG);

    let echo = Echo::default();
    let peer = Served::new(&echo);
    let log = peer.log.clone();
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap();
    let result = target.execute_command(command_request()).unwrap();

    let sent = log.borrow()[2].1.clone();
    for token in [BIG, LONG, WIDE] {
        assert!(
            sent.contains(&format!(":{token}")),
            "{token} not sent exactly: {sent}"
        );
    }
    let Some(Seen::Command(received)) = echo.log.borrow().last().cloned() else {
        panic!("the command reached the served target");
    };
    assert_eq!(received.input, values());

    let response = result.response.unwrap();
    assert_eq!(response, values());
    assert_ne!(response["long"], Node::Number(Number::new(0.1).unwrap()));
    assert_eq!(
        response["big"],
        Node::Number(Number::from(9_007_199_254_740_993_i64))
    );
}

// ---- closed answers ------------------------------------------------------------------------------

/// The error a query answered with `text` gives.
fn query_answered(text: &str) -> Result<SemanticViewResult, TargetError> {
    let echo = Echo::default();
    let target = connect(Served::new(&echo).on("query_view", 1, line(text)));
    target.begin_scenario(&first()).unwrap();
    target.query_view(view_request())
}

fn not_an_answer(result: Result<SemanticViewResult, TargetError>) -> String {
    match result {
        Err(TargetError::Unavailable { operation, detail }) => {
            assert_eq!(operation, "query_view");
            assert!(
                detail.starts_with("answer to 2 (query_view) does not match its operation at byte"),
                "{detail}"
            );
            detail
        }
        other => panic!("expected a not-an-answer fault, got {other:?}"),
    }
}

#[test]
fn an_unknown_answer_member_is_refused() {
    not_an_answer(query_answered(r#"{"id":2,"ok":{"rows":[],"surprise":1}}"#));
    not_an_answer(query_answered(r#"{"id":2,"ok":{"rows":[]},"extra":true}"#));
}

#[test]
fn a_repeated_member_name_is_refused() {
    let detail = not_an_answer(query_answered(r#"{"id":2,"ok":{"rows":[],"rows":[]}}"#));
    assert!(detail.ends_with("byte 31"), "{detail}");
    not_an_answer(query_answered(r#"{"id":2,"ok":{"rows":[{"a":1,"a":2}]}}"#));
}

#[test]
fn a_missing_answer_member_is_refused() {
    not_an_answer(query_answered(r#"{"id":2,"ok":{"total":1}}"#));
    not_an_answer(query_answered(r#"{"id":2}"#));
    not_an_answer(query_answered(r#"{"ok":{"rows":[]}}"#));
}

#[test]
fn a_member_of_the_wrong_type_is_refused() {
    not_an_answer(query_answered(r#"{"id":2,"ok":{"rows":"none"}}"#));
    not_an_answer(query_answered(r#"{"id":2,"ok":{"rows":[],"total":-1}}"#));
    not_an_answer(query_answered(r#"{"id":"2","ok":{"rows":[]}}"#));
    not_an_answer(query_answered(
        r#"{"id":2,"ok":{"rows":[]},"target_error":null}"#,
    ));
}

#[test]
fn a_value_map_holds_any_names_and_optional_members_may_be_absent() {
    let result =
        query_answered(r#"{"id":2,"ok":{"rows":[{"anything":1.5,"else":null}],"total":null}}"#)
            .unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.total, None);
}

#[test]
fn a_target_error_answer_is_the_targets_and_keeps_the_process() {
    let echo = Echo::default();
    let peer = Served::new(&echo).on(
        "query_view",
        1,
        line(r#"{"id":2,"target_error":{"kind":"unsupported","observation":"rows","why":"no"}}"#),
    );
    let kills = peer.kills.clone();
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap();
    assert_eq!(
        target.query_view(view_request()),
        Err(TargetError::unsupported("rows", "no"))
    );
    assert!(target.query_view(view_request()).is_ok());
    assert_eq!(*kills.borrow(), 0);
}

// ---- faults --------------------------------------------------------------------------------------

/// What a command answered with `reply` comes to: its error, and the detail it carries.
fn command_fault(reply: Reply) -> TargetError {
    let echo = Echo::default();
    let target = connect(Served::new(&echo).on("execute_command", 1, reply));
    target.begin_scenario(&first()).unwrap();
    target.execute_command(command_request()).unwrap_err()
}

fn detail(error: &TargetError) -> &str {
    match error {
        TargetError::Unavailable { operation, detail } => {
            assert_eq!(operation, "execute_command");
            detail
        }
        other => panic!("expected an unavailable fault, got {other:?}"),
    }
}

#[test]
fn an_exit_is_the_exited_fault() {
    let error = command_fault(Reply::Fault(PeerFault::Exited(Exit::Code(1))));
    assert_eq!(
        detail(&error),
        "target process exited (status 1) before answering 2 (execute_command)"
    );
}

#[test]
fn a_closed_stdout_is_the_closed_stdout_fault() {
    let error = command_fault(Reply::Fault(PeerFault::ClosedStdout));
    assert_eq!(
        detail(&error),
        "target process closed its standard output before answering 2 (execute_command)"
    );
}

#[test]
fn no_answer_in_time_is_the_timeout_fault() {
    let error = command_fault(Reply::Fault(PeerFault::Timeout));
    assert_eq!(
        detail(&error),
        "no answer to 2 (execute_command) within 30000 ms"
    );
}

#[test]
fn a_long_line_is_the_line_too_long_fault() {
    let expected = "answer to 2 (execute_command) exceeds 16777216 bytes";
    assert_eq!(
        detail(&command_fault(Reply::Fault(PeerFault::LineTooLong))),
        expected
    );
    let error = command_fault(Reply::Line(Box::new(|_| vec![b' '; MAX_LINE + 1])));
    assert_eq!(detail(&error), expected);
}

#[test]
fn a_line_that_is_not_utf8_is_the_not_utf8_fault() {
    let error = command_fault(Reply::Line(Box::new(|_| {
        b"{\"id\":2,\"ok\":\"\xff\"}".to_vec()
    })));
    assert_eq!(
        detail(&error),
        "answer to 2 (execute_command) is not UTF-8 at byte 14"
    );
}

#[test]
fn a_blank_or_broken_line_is_the_not_json_fault() {
    assert_eq!(
        detail(&command_fault(line(""))),
        "answer to 2 (execute_command) is not JSON at byte 0"
    );
    assert_eq!(
        detail(&command_fault(line("   "))),
        "answer to 2 (execute_command) is not JSON at byte 0"
    );
    assert_eq!(
        detail(&command_fault(line(r#"{"id":2,"ok":}"#))),
        "answer to 2 (execute_command) is not JSON at byte 13"
    );
}

#[test]
fn a_shape_the_operation_does_not_have_is_the_not_an_answer_fault() {
    assert_eq!(
        detail(&command_fault(line(r#"{"id":2,"ok":{"outcome":5}}"#))),
        "answer to 2 (execute_command) does not match its operation at byte 24"
    );
}

#[test]
fn an_answer_to_another_request_is_the_wrong_id_fault() {
    assert_eq!(
        detail(&command_fault(line(r#"{"id":9,"ok":null}"#))),
        "answer names id 9 while 2 (execute_command) is outstanding"
    );
}

#[test]
fn a_number_outside_the_domain_is_the_number_out_of_range_fault() {
    assert_eq!(
        detail(&command_fault(line(
            r#"{"id":2,"ok":{"response":{"n":1e400}}}"#
        ))),
        "answer to 2 (execute_command) holds a number outside the domain at byte 30"
    );
}

#[test]
fn a_target_that_cannot_be_started_again_is_the_not_started_fault() {
    let echo = Echo::default();
    let mut peer = Served::new(&echo).on("execute_command", 1, line("garbage"));
    peer.respawn_fault = Some(PeerFault::NotStarted(std::io::ErrorKind::NotFound));
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap();
    target.execute_command(command_request()).unwrap_err();
    assert_eq!(
        target.begin_scenario(&second()),
        Err(TargetError::unavailable(
            "handshake",
            "the target's handshake failed (not-started): target process could not be started \
             again (NotFound) before 0 (handshake)"
        ))
    );
}

#[test]
fn every_fault_kind_has_its_page_name() {
    let names: Vec<&str> = [
        FaultKind::Exited(Exit::Unknown),
        FaultKind::ClosedStdout,
        FaultKind::Timeout,
        FaultKind::LineTooLong,
        FaultKind::NotUtf8 { at: 0 },
        FaultKind::NotJson { at: 0 },
        FaultKind::NotAnAnswer { at: 0 },
        FaultKind::WrongId { named: 0 },
        FaultKind::NumberOutOfRange { at: 0 },
        FaultKind::NotStarted(std::io::ErrorKind::NotFound),
    ]
    .iter()
    .map(FaultKind::name)
    .collect();
    assert_eq!(
        names,
        [
            "exited",
            "closed-stdout",
            "timeout",
            "line-too-long",
            "not-utf8",
            "not-json",
            "not-an-answer",
            "wrong-id",
            "number-out-of-range",
            "not-started"
        ]
    );
}

#[test]
fn a_held_window_waits_its_hold_beyond_the_timeout() {
    let echo = Echo::default();
    let peer = Served::new(&echo).on("observe_elapsed", 1, Reply::Fault(PeerFault::Timeout));
    let log = peer.log.clone();
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap();
    let error = target.observe_elapsed(elapsed_request(20)).unwrap_err();
    assert_eq!(
        error,
        TargetError::unavailable(
            "observe_elapsed",
            "no answer to 2 (observe_elapsed) within 50000 ms"
        )
    );
    let sent: serde_json::Value = serde_json::from_str(&log.borrow()[2].1).unwrap();
    assert_eq!(sent["limit_ms"], 50_000);
}

#[test]
fn a_fault_never_quotes_the_answer() {
    let lines: Vec<Vec<u8>> = vec![
        {
            let mut bad = format!("{{\"id\":2,\"ok\":\"{SECRET}").into_bytes();
            bad.push(0xff);
            bad.extend_from_slice(b"\"}");
            bad
        },
        SECRET.as_bytes().to_vec(),
        format!(r#"{{"id":2,"ok":{{"outcome":"{SECRET}"}}}}"#).into_bytes(),
        format!(r#"{{"id":2,"ok":{{"{SECRET}":1}}}}"#).into_bytes(),
        format!(r#"{{"id":2,"ok":{{"outcome":{{"command":"{SECRET} !","outcome":"x"}}}}}}"#)
            .into_bytes(),
        format!(r#"{{"id":2,"ok":{{"consistency":"","response":{{"k":"{SECRET}"}}}}}}"#)
            .into_bytes(),
        format!(r#"{{"id":9,"ok":"{SECRET}"}}"#).into_bytes(),
        format!(r#"{{"id":2,"ok":{{"response":{{"{SECRET}":1e400}}}}}}"#).into_bytes(),
        format!(
            r#"{{"id":2,"ok":{{"response":{{"n":1{}}}}}}}"#,
            "0".repeat(600)
        )
        .into_bytes(),
        format!(r#"{{"id":2,"ok":{{"rows":1,"{SECRET}":2,"{SECRET}":3}}}}"#).into_bytes(),
        {
            let mut long = format!(r#"{{"id":2,"ok":"{SECRET}"#).into_bytes();
            long.resize(MAX_LINE + 1, b'x');
            long
        },
    ];
    let mut kinds = BTreeSet::new();
    for bytes in lines {
        let error = command_fault(Reply::Line(Box::new(move |_| bytes.clone())));
        let shown = format!("{error} {error:?}");
        assert!(!shown.contains(SECRET), "{shown}");
        assert!(!shown.contains("10000000000"), "{shown}");
        kinds.insert(detail(&error).split(" at byte").next().unwrap().to_owned());
    }
    for fault in [
        PeerFault::Exited(Exit::Signal(9)),
        PeerFault::ClosedStdout,
        PeerFault::Timeout,
        PeerFault::LineTooLong,
    ] {
        let error = command_fault(Reply::Fault(fault));
        assert!(!format!("{error} {error:?}").contains(SECRET));
    }
    assert!(kinds.len() >= 6, "{kinds:?}");

    // A handshake that breaks quotes nothing of it either.
    for answer in [
        format!(
            r#"{{"id":0,"ok":{{"speaks":"{SECRET}","implementation":{{"name":"a","version":"1"}}}}}}"#
        ),
        format!(
            r#"{{"id":0,"target_error":{{"kind":"unavailable","operation":"{SECRET}","detail":"{SECRET}"}}}}"#
        ),
        format!(r#"{{"id":0,"ok":{{"speaks":1,"{SECRET}":2}}}}"#),
    ] {
        let echo = Echo::default();
        let refusal = ProcessTarget::connect(
            Served::new(&echo).on("handshake", 1, line(&answer)),
            suite(),
            TIMEOUT_MS,
        )
        .err()
        .expect("the handshake is refused");
        let shown = format!("{refusal} {refusal:?}");
        assert!(!shown.contains(SECRET), "{shown}");
    }

    // Nor does `serve` quote a request it cannot read.
    let error = serve(
        &Echo::default(),
        format!("{SECRET}\n").as_bytes(),
        Vec::new(),
    )
    .unwrap_err();
    assert!(!format!("{error} {error:?}").contains(SECRET));
}

// ---- the handshake -------------------------------------------------------------------------------

#[test]
fn a_version_this_build_does_not_read_is_refused_naming_the_versions_read() {
    let echo = Echo::default();
    let peer = Served::new(&echo).on(
        "handshake",
        1,
        line(r#"{"id":0,"ok":{"speaks":"ess-conformance-target/2","implementation":{"name":"a","version":"1"}}}"#),
    );
    let kills = peer.kills.clone();
    let refusal = ProcessTarget::connect(peer, suite(), TIMEOUT_MS)
        .err()
        .unwrap();
    assert_eq!(refusal, HandshakeRefusal::Version { speaks: Some(2) });
    assert_eq!(refusal.kind(), "version");
    assert_eq!(
        refusal.to_string(),
        "target speaks ess-conformance-target/2; this ess reads ess-conformance-target/1"
    );
    assert_eq!(*kills.borrow(), 1);
}

#[test]
fn a_handshake_answered_with_a_target_error_or_a_fault_is_refused() {
    let echo = Echo::default();
    let refusal = ProcessTarget::connect(
        Served::new(&echo).on(
            "handshake",
            1,
            line(r#"{"id":0,"target_error":{"kind":"unsupported","observation":"x","why":"y"}}"#),
        ),
        suite(),
        TIMEOUT_MS,
    )
    .err()
    .unwrap();
    assert_eq!(
        refusal,
        HandshakeRefusal::Refused {
            kind: "unsupported"
        }
    );
    let refusal = ProcessTarget::connect(
        Served::new(&echo).on(
            "handshake",
            1,
            Reply::Fault(PeerFault::Exited(Exit::Code(3))),
        ),
        suite(),
        TIMEOUT_MS,
    )
    .err()
    .unwrap();
    assert_eq!(refusal.kind(), "exited");
    assert_eq!(
        refusal.to_string(),
        "the target's handshake failed (exited): target process exited (status 3) before \
         answering 0 (handshake)"
    );
}

// ---- the lifecycle -------------------------------------------------------------------------------

#[test]
fn after_a_fault_the_scenario_answers_alone_and_the_next_one_restarts() {
    let echo = Echo::default();
    let peer = Served::new(&echo).on("execute_command", 1, line("garbage"));
    let (log, kills, respawns) = (peer.log.clone(), peer.kills.clone(), peer.respawns.clone());
    let target = connect(peer);

    target.begin_scenario(&first()).unwrap();
    let fault = target.execute_command(command_request()).unwrap_err();
    assert_eq!(*kills.borrow(), 1);
    let sent = log.borrow().len();
    assert_eq!(target.query_view(view_request()), Err(fault.clone()));
    assert_eq!(target.execute_command(command_request()), Err(fault));
    assert_eq!(target.end_scenario(&first()), Ok(()));
    assert_eq!(
        log.borrow().len(),
        sent,
        "nothing reaches a faulted scenario's peer"
    );
    assert_eq!(*respawns.borrow(), 0);

    target.begin_scenario(&second()).unwrap();
    assert_eq!(*respawns.borrow(), 1);
    let ops: Vec<String> = log.borrow()[sent..]
        .iter()
        .map(|(op, _)| op.clone())
        .collect();
    assert_eq!(ops, ["handshake", "begin_scenario"]);
    let handshake: serde_json::Value = serde_json::from_str(&log.borrow()[sent].1).unwrap();
    assert_eq!(handshake["id"], 0, "ids count per process");
    let begin: serde_json::Value = serde_json::from_str(&log.borrow()[sent + 1].1).unwrap();
    assert_eq!(begin["id"], 1);
    assert!(target.execute_command(command_request()).is_ok());
    assert_eq!(target.end_scenario(&second()), Ok(()));
    assert!(matches!(echo.log.borrow().last(), Some(Seen::End(_))));
}

#[test]
fn a_restart_naming_another_implementation_fails_every_remaining_scenario() {
    let echo = Echo::default();
    let peer = Served::new(&echo).on("execute_command", 1, line("garbage")).on(
        "handshake",
        2,
        line(r#"{"id":0,"ok":{"speaks":"ess-conformance-target/1","implementation":{"name":"echo","version":"2.0.0"}}}"#),
    );
    let (log, respawns) = (peer.log.clone(), peer.respawns.clone());
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap();
    target.execute_command(command_request()).unwrap_err();
    target.end_scenario(&first()).unwrap();

    let failed = target.begin_scenario(&second()).unwrap_err();
    assert_eq!(
        failed,
        TargetError::unavailable(
            "handshake",
            "the restarted target names another implementation than the run's; one report names \
             one implementation"
        )
    );
    let sent = log.borrow().len();
    assert_eq!(target.begin_scenario(&third()), Err(failed.clone()));
    assert_eq!(target.query_view(view_request()).err(), Some(failed));
    assert_eq!(log.borrow().len(), sent);
    assert_eq!(*respawns.borrow(), 1, "a failed restart is not retried");
    assert_eq!(
        target.identity(),
        Ok(ImplementationIdentity::new("echo", "1.0.0"))
    );
}

#[test]
fn a_failed_restart_fails_every_remaining_scenario_without_respawning_again() {
    let echo = Echo::default();
    let mut peer = Served::new(&echo).on("begin_scenario", 1, Reply::Fault(PeerFault::Timeout));
    peer.respawn_fault = Some(PeerFault::NotStarted(std::io::ErrorKind::PermissionDenied));
    let respawns = peer.respawns.clone();
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap_err();
    let failed = target.begin_scenario(&second()).unwrap_err();
    assert_eq!(target.begin_scenario(&third()), Err(failed));
    assert_eq!(*respawns.borrow(), 1);
}

// ---- fallbacks -----------------------------------------------------------------------------------

/// A target that checks no read grant: every `*_as` and `*_anonymous` read is unsupported.
struct Gateless(Echo);

impl ConformanceTarget for Gateless {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn query_view_as(
        &self,
        _: SemanticViewRequest,
        _: &ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("read as", "no grants"))
    }
    fn query_view_anonymous(
        &self,
        _: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("read anonymously", "no grants"))
    }
    fn scan_view(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        self.0.scan_view(request)
    }
    fn scan_view_as(
        &self,
        _: OrderedScanRequest,
        _: &ActorRef,
    ) -> Result<OrderedScan, TargetError> {
        Err(TargetError::unsupported("scan as", "no grants"))
    }
    fn scan_view_anonymous(&self, _: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        Err(TargetError::unsupported("scan anonymously", "no grants"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
}

#[test]
fn an_unsupported_read_as_an_actor_falls_back_to_the_plain_read() {
    let gateless = Gateless(Echo::default());
    let peer = Served::new(&gateless);
    let log = peer.log.clone();
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap();
    let reader: ActorRef = name("billing.invoice.Clerk");
    let plain = target.query_view(view_request()).unwrap();
    assert_eq!(
        target.query_view_as(view_request(), &reader),
        Ok(plain.clone())
    );
    assert_eq!(target.query_view_anonymous(view_request()), Ok(plain));
    let scanned = target.scan_view(scan_request()).unwrap();
    assert_eq!(target.scan_view_as(scan_request(), &reader), Ok(scanned));
    assert_eq!(target.scan_view_anonymous(scan_request()), Ok(scanned));
    let ops: Vec<String> = log.borrow()[2..].iter().map(|(op, _)| op.clone()).collect();
    assert_eq!(
        ops,
        [
            "query_view",
            "query_view_as",
            "query_view",
            "query_view_anonymous",
            "query_view",
            "scan_view",
            "scan_view_as",
            "scan_view",
            "scan_view_anonymous",
            "scan_view"
        ]
    );
}

// ---- serve ---------------------------------------------------------------------------------------

#[test]
fn serve_answers_an_unknown_op_unsupported_and_returns_at_end_of_input() {
    let input = concat!(
        r#"{"id":5,"op":"teleport","limit_ms":1,"request":{}}"#,
        "\n",
        r#"{"id":6,"op":"handshake","limit_ms":1,"request":{"reads":["ess-conformance-target/1"],"later":true}}"#,
        "\n"
    );
    let mut out = Vec::new();
    serve(&Echo::default(), input.as_bytes(), &mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        lines[0],
        serde_json::json!({"id": 5, "target_error": {"kind": "unsupported",
            "observation": "teleport", "why": "this target does not implement the operation"}})
    );
    assert_eq!(
        lines[1],
        serde_json::json!({"id": 6, "ok": {"speaks": "ess-conformance-target/1",
            "implementation": {"name": "echo", "version": "1.0.0"}}})
    );
    assert_eq!(lines.len(), 2);
}

#[test]
fn serve_refuses_a_request_it_cannot_decode_by_kind_and_line() {
    let input = "{\"id\":1,\"op\":\"teleport\",\"limit_ms\":1,\"request\":{}}\n{\"op\":\"begin_scenario\"}\n";
    let error = serve(&Echo::default(), input.as_bytes(), Vec::new()).unwrap_err();
    assert_eq!(error.kind(), "not-a-request");
    assert_eq!(
        error.to_string(),
        "request line 2 is not a request of ess-conformance-target/1 at byte 0"
    );
}

// ---- a full run ----------------------------------------------------------------------------------

#[test]
fn a_billing_run_through_serve_reports_what_the_built_in_run_reports() {
    let admitted = billing();
    let direct = crate::Runner::for_suite(admitted.suite())
        .run_admitted(admitted, &crate::reference::Billing::new())
        .into_report();
    let served = crate::reference::Billing::new();
    let target = ProcessTarget::connect(
        Served::new(&served),
        HandshakeSuite::of(admitted),
        TIMEOUT_MS,
    )
    .unwrap();
    let through = crate::Runner::for_suite(admitted.suite())
        .run_admitted(admitted, &target)
        .into_report();

    let statuses = |report: &crate::ConformanceReport| {
        report
            .scenarios
            .iter()
            .map(|result| (result.scenario.clone(), result.status))
            .collect::<Vec<_>>()
    };
    assert_ne!(direct.scenarios.len(), 0);
    assert_eq!(statuses(&through), statuses(&direct));
    assert_eq!(through.implementation, direct.implementation);
    assert_eq!(through.scenarios, direct.scenarios);
    assert_eq!(through.status, direct.status);
}
