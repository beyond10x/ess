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
    rewrite: Option<Rewrite>,
}

/// What a peer does to each answer `serve` wrote before the target reads it.
type Rewrite = Box<dyn Fn(&str) -> String>;

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
            rewrite: None,
        }
    }

    fn on(mut self, op: &str, nth: usize, reply: Reply) -> Self {
        self.overrides.push((op.to_owned(), nth, reply));
        self
    }

    /// Every answer `serve` writes passed through `rewrite` before the target reads it.
    fn rewriting(mut self, rewrite: impl Fn(&str) -> String + 'static) -> Self {
        self.rewrite = Some(Box::new(rewrite));
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
        if let Some(rewrite) = &self.rewrite {
            out = rewrite(std::str::from_utf8(&out).unwrap()).into_bytes();
        }
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

/// `serve` answers every request `exercise` sends as it answers that request with `member`, one
/// `"name":value` pair, added first to its `request` object. Added as text: a round trip through
/// `serde_json::Value` would round the requests' exact numbers.
fn answers_to_every_operation(member: &str) {
    let echo = Echo::default();
    let peer = Served::new(&echo);
    let log = peer.log.clone();
    exercise(&connect(peer));
    let mut seen = BTreeSet::new();
    for (op, line) in log.borrow().iter() {
        let (envelope, request) = line.split_once(r#""request":{"#).unwrap();
        let comma = if request.starts_with('}') { "" } else { "," };
        let wider = format!(r#"{envelope}"request":{{{member}{comma}{request}"#);
        serde_json::from_str::<serde_json::Value>(&wider).expect("the widened line is JSON");
        let answer = |line: &str| {
            let mut out = Vec::new();
            serve(&Echo::default(), format!("{line}\n").as_bytes(), &mut out)
                .map(|()| String::from_utf8(out).unwrap())
                .map_err(|error| error.kind())
        };
        assert_eq!(
            answer(&wider),
            answer(line),
            "{op}: a request member serve does not know changes its answer"
        );
        seen.insert(Op::from_name(op).unwrap());
    }
    assert_eq!(seen, Op::ALL.iter().copied().collect::<BTreeSet<_>>());
}

/// Design "Requests are open": every operation's request object, at its top level.
#[test]
fn serve_drops_an_unknown_top_level_request_member_in_every_operation() {
    answers_to_every_operation(r#""added_later":{"any":[1,"x",2.5]}"#);
}

/// Members whose value is data the target reads, a value map: a key added there is a value.
const DATA: &[&str] = &[
    "input", "caller", "params", "payload", "context", "compared", "identity", "instance",
    "fields", "read",
];
/// Members whose value is a dictionary keyed by name: a key added there is an entry.
const NAMED: &[&str] = &["mapping", "declarations", "variants"];

/// Where each record of members in `value` opens, with its path: every object that is neither a
/// value map nor a dictionary keyed by name, however deep.
fn records(value: &exact::Value, member: Option<&str>, path: &str, out: &mut Vec<(String, usize)>) {
    match &value.kind {
        exact::Kind::Seq(items) => {
            for (index, item) in items.iter().enumerate() {
                records(item, None, &format!("{path}[{index}]"), out);
            }
        }
        exact::Kind::Map(map) => {
            if member.is_some_and(|member| DATA.contains(&member)) {
                return;
            }
            let named = member.is_some_and(|member| NAMED.contains(&member));
            if !named {
                out.push((path.to_owned(), value.at));
            }
            for (key, item) in map {
                let inner = (!named).then_some(key.as_str());
                records(item, inner, &format!("{path}.{key}"), out);
            }
        }
        _ => {}
    }
}

/// What `serve` answers `line` with, and every request the target saw.
fn served_by_echo(line: &str) -> (Result<String, &'static str>, Vec<Seen>) {
    let echo = Echo::default();
    let mut out = Vec::new();
    let answer = serve(&echo, format!("{line}\n").as_bytes(), &mut out)
        .map(|()| String::from_utf8(out).unwrap())
        .map_err(|error| error.kind());
    (answer, echo.log.into_inner())
}

/// Design "Requests are open", at every depth: one member added to any record of a request, a
/// periodic check's host contract and a fixture contract's declared fields included, changes
/// neither the answer nor what the target is handed. A value map and a dictionary keyed by name
/// are not records: a key added there is data.
#[test]
fn serve_drops_an_unknown_request_member_at_every_depth_in_every_operation() {
    let echo = Echo::default();
    let peer = Served::new(&echo);
    let log = peer.log.clone();
    exercise(&connect(peer));
    let mut lines = log.borrow().clone();
    let (_, fixture) = lines
        .iter()
        .find(|(op, _)| op == "fixture_values")
        .unwrap()
        .clone();
    let declared = fixture.replace(
        r#"{"fields":[{"name":"principal-id","type":"Uuid"}],"declarations":{}}"#,
        r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Code":{"kind":"newtype","of":"String"},"acme.Mode":{"kind":"enum","variants":["on","off"]},"acme.Pair":{"kind":"struct","fields":[{"name":"left","type":"acme.Code"},{"name":"mode","type":"Optional<acme.Mode>","wire":"Mode"}]}}}"#,
    );
    assert_ne!(declared, fixture, "the fixture contract is replaced");
    lines.push(("fixture_values".to_owned(), declared));
    let mut points = BTreeSet::new();
    for (op, line) in &lines {
        let control = served_by_echo(line);
        assert!(
            control
                .0
                .as_ref()
                .is_ok_and(|answer| !answer.contains(r#""fault""#)),
            "{op}: {control:?}"
        );
        let tree = exact::parse(line).unwrap();
        let exact::Kind::Map(envelope) = &tree.kind else {
            panic!("{op}: {line}")
        };
        let mut at = Vec::new();
        records(&envelope["request"], None, "request", &mut at);
        for (path, offset) in at {
            assert_eq!(line.as_bytes()[offset], b'{', "{op} {path}");
            let rest = &line[offset + 1..];
            let comma = if rest.starts_with('}') { "" } else { "," };
            let wider = format!(
                r#"{}{{"added_later":{{"any":[1]}}{comma}{rest}"#,
                &line[..offset]
            );
            assert_eq!(
                served_by_echo(&wider),
                control,
                "{op}: a member serve does not know, added at {path}, changes its answer"
            );
            points.insert(format!("{op} {path}"));
        }
    }
    for deep in [
        "open_periodic request.check.periodic.host",
        "open_periodic request.check.periodic.host.read_fields[0]",
        "open_periodic request.check.mapping.",
        "fixture_values request.contract.fields[0]",
        "fixture_values request.contract.declarations.acme.Pair",
        "fixture_values request.contract.declarations.acme.Pair.fields[1]",
        "observe_periodic request.scope",
        "close_periodic request",
    ] {
        assert!(
            points.iter().any(|point| point.starts_with(deep)),
            "no member is added at {deep}: {points:#?}"
        );
    }
}

/// The suite vocabulary `serve` reads openly stays closed where it reads a suite document: a
/// periodic scope, a periodic check and a fixture contract each refuse a member they do not know,
/// at their top level and nested.
#[test]
fn a_suite_scope_check_or_contract_with_a_member_it_does_not_know_is_still_refused() {
    fn refuses<T: serde::de::DeserializeOwned + std::fmt::Debug>(control: &str, wider: &str) {
        serde_json::from_str::<T>(control).expect(control);
        assert!(serde_json::from_str::<T>(wider).is_err(), "{wider}");
    }
    refuses::<Scope>(
        r#"{"correlation":"billing-c-000001","binding":"poll-mailbox","lifetime":"l-1"}"#,
        r#"{"correlation":"billing-c-000001","binding":"poll-mailbox","lifetime":"l-1","added_later":1}"#,
    );
    let contract = r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Pair":{"kind":"struct","fields":[{"name":"left","type":"String"}]}}}"#;
    for wider in [
        r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Pair":{"kind":"struct","fields":[{"name":"left","type":"String"}]}},"added_later":1}"#,
        r#"{"fields":[{"name":"principal-id","type":"acme.Pair","added_later":1}],"declarations":{"acme.Pair":{"kind":"struct","fields":[{"name":"left","type":"String"}]}}}"#,
        r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Pair":{"kind":"struct","added_later":1,"fields":[{"name":"left","type":"String"}]}}}"#,
        r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Pair":{"kind":"struct","fields":[{"name":"left","type":"String","added_later":1}]}}}"#,
    ] {
        refuses::<crate::fixtures::Contract>(contract, wider);
    }
    let check = serde_json::to_value(periodic_check()).unwrap();
    for path in [
        "",
        "/periodic",
        "/periodic/host",
        "/periodic/host/read_fields/0",
    ] {
        let mut wider = check.clone();
        wider.pointer_mut(path).unwrap()["added_later"] = 1.into();
        refuses::<crate::periodic::Check>(&check.to_string(), &wider.to_string());
    }
}

/// A member a request's types know in one object and not in another is dropped only where it is
/// unknown: a declaration's unknown `name` goes, every field's `name` before and after it stays.
#[test]
fn a_request_member_unknown_in_one_object_is_dropped_there_and_kept_where_it_is_known() {
    let control = r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Pair":{"kind":"struct","fields":[{"name":"left","type":"String"}]}}}"#;
    let wider = r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Pair":{"kind":"struct","fields":[{"name":"left","type":"String"}],"name":"later"}}}"#;
    let read =
        |line: &str| exact::request::<crate::fixtures::Contract>(&exact::parse(line).unwrap());
    assert_eq!(read(wider), Ok(read(control).unwrap()));
    let unknown_type = r#"{"fields":[{"name":"principal-id","type":"acme.Pair"}],"declarations":{"acme.Pair":{"kind":"later","fields":[]}}}"#;
    assert!(read(unknown_type).is_err(), "only members are open");
}

// ---- target-supplied text ------------------------------------------------------------------------

/// The identity a handshake naming `name` and `version` admits, or its refusal.
fn handshaken(name: &str, version: &str) -> Result<ImplementationIdentity, HandshakeRefusal> {
    let echo = Echo::default();
    let answer = serde_json::json!({"id": 0, "ok": {"speaks": "ess-conformance-target/1",
        "implementation": {"name": name, "version": version}}})
    .to_string();
    let peer = Served::new(&echo).on("handshake", 1, line(&answer));
    ProcessTarget::connect(peer, suite(), TIMEOUT_MS).map(|target| target.implementation().clone())
}

/// Every character the boundary writes as text, with that text.
const STEERING: &[(char, &str)] = &[
    ('\u{0}', "\\u{0}"),
    ('\u{7}', "\\u{7}"),
    ('\u{8}', "\\u{8}"),
    ('\t', "\\u{9}"),
    ('\n', "\\u{a}"),
    ('\r', "\\u{d}"),
    ('\u{1b}', "\\u{1b}"),
    ('\u{1f}', "\\u{1f}"),
    ('\u{7f}', "\\u{7f}"),
    ('\u{80}', "\\u{80}"),
    ('\u{85}', "\\u{85}"),
    ('\u{9b}', "\\u{9b}"),
    ('\u{9f}', "\\u{9f}"),
    ('\u{2028}', "\\u{2028}"),
    ('\u{2029}', "\\u{2029}"),
    ('\u{202a}', "\\u{202a}"),
    ('\u{202b}', "\\u{202b}"),
    ('\u{202c}', "\\u{202c}"),
    ('\u{202d}', "\\u{202d}"),
    ('\u{202e}', "\\u{202e}"),
    ('\u{2066}', "\\u{2066}"),
    ('\u{2067}', "\\u{2067}"),
    ('\u{2068}', "\\u{2068}"),
    ('\u{2069}', "\\u{2069}"),
];

/// Characters next to the escaped ones that stay as they are.
const KEPT: &str = "a é\u{a0}\u{200b}\u{2027}\u{202f}\u{2065}\u{206a}\u{fe0f}𝄞\\";

fn steering() -> (String, String) {
    let mut raw = String::from(KEPT);
    let mut written = String::from(KEPT);
    for (c, text) in STEERING {
        raw.push(*c);
        raw.push('|');
        written.push_str(text);
        written.push('|');
    }
    (raw, written)
}

#[test]
fn a_handshake_identity_has_every_steering_character_written_as_text() {
    let (raw, written) = steering();
    assert_eq!(
        handshaken(&raw, &format!("1.0{raw}")),
        Ok(ImplementationIdentity::new(
            written.clone(),
            format!("1.0{written}")
        ))
    );
}

#[test]
fn a_handshake_name_or_version_past_256_bytes_is_refused_without_quoting_it() {
    let at_bound = format!("{}é", "n".repeat(254));
    assert_eq!(at_bound.len(), 256);
    assert_eq!(
        handshaken(&at_bound, &at_bound),
        Ok(ImplementationIdentity::new(
            at_bound.clone(),
            at_bound.clone()
        ))
    );
    let past = format!("{}é", "Q".repeat(255));
    for (name, version, member) in [
        (past.as_str(), "1", "name"),
        ("echo", past.as_str(), "version"),
    ] {
        let refusal = handshaken(name, version).unwrap_err();
        assert_eq!(
            refusal,
            HandshakeRefusal::IdentityTooLong { member, bytes: 257 }
        );
        assert_eq!(refusal.kind(), "identity-too-long");
        let text = refusal.to_string();
        assert_eq!(
            text,
            format!(
                "the target's handshake names an implementation {member} of 257 bytes; at most \
                 256 are read"
            )
        );
        assert!(!text.contains("QQ"), "{text}");
    }
    // Bounded before escaping: 256 bytes of escapes are the target's 256 bytes.
    let controls = "\u{1b}".repeat(256);
    assert_eq!(
        handshaken(&controls, "1").unwrap().name,
        "\\u{1b}".repeat(256)
    );
}

#[test]
fn a_restarted_target_naming_the_same_steering_identity_is_the_same_implementation() {
    let (raw, _) = steering();
    let echo = Echo::default();
    let answer = serde_json::json!({"id": 0, "ok": {"speaks": "ess-conformance-target/1",
        "implementation": {"name": raw, "version": "1"}}})
    .to_string();
    let peer = Served::new(&echo)
        .on("handshake", 1, line(&answer))
        .on("handshake", 2, line(&answer))
        .on("execute_command", 1, line("garbage"));
    let respawns = peer.respawns.clone();
    let target = connect(peer);
    target.begin_scenario(&first()).unwrap();
    target.execute_command(command_request()).unwrap_err();
    assert_eq!(target.begin_scenario(&second()), Ok(()));
    assert_eq!(*respawns.borrow(), 1);
}

/// What a query answered with `target_error` comes to.
fn query_refused(target_error: &serde_json::Value) -> TargetError {
    let text = serde_json::json!({"id": 2, "target_error": target_error}).to_string();
    query_answered(&text).unwrap_err()
}

#[test]
fn every_string_of_a_target_error_has_every_steering_character_written_as_text() {
    let (raw, written) = steering();
    assert_eq!(
        query_refused(&serde_json::json!({"kind": "unsupported",
            "observation": format!("o{raw}"), "why": format!("w{raw}")})),
        TargetError::unsupported(format!("o{written}"), format!("w{written}"))
    );
    assert_eq!(
        query_refused(&serde_json::json!({"kind": "unavailable",
            "operation": format!("o{raw}"), "detail": format!("d{raw}")})),
        TargetError::unavailable(format!("o{written}"), format!("d{written}"))
    );
    assert_eq!(
        query_refused(&serde_json::json!({"kind": "not_granted", "actor": format!("a{raw}")})),
        TargetError::NotGranted {
            actor: Some(format!("a{written}"))
        }
    );
}

#[test]
fn a_target_error_string_past_4096_bytes_is_cut_at_a_character_naming_what_was_cut() {
    let at_bound = "w".repeat(4096);
    assert_eq!(
        query_refused(
            &serde_json::json!({"kind": "unsupported", "observation": "rows",
            "why": at_bound})
        ),
        TargetError::unsupported("rows", at_bound.clone())
    );
    // 4095 bytes, then a two-byte character the cut would split.
    let long = format!("{}{}", "w".repeat(4095), "é".repeat(1000));
    assert_eq!(
        query_refused(
            &serde_json::json!({"kind": "unavailable", "operation": "rows",
            "detail": long})
        ),
        TargetError::unavailable("rows", format!("{} … (2000 bytes more)", "w".repeat(4095)))
    );
    // The bound is the target's bytes; what is kept is escaped after the cut.
    let controls = format!("{}tail", "\u{1b}".repeat(4096));
    let cut = match query_refused(&serde_json::json!({"kind": "not_granted", "actor": controls})) {
        TargetError::NotGranted { actor: Some(actor) } => actor,
        other => panic!("expected not_granted, got {other:?}"),
    };
    assert_eq!(cut, format!("{} … (4 bytes more)", "\\u{1b}".repeat(4096)));
    for (kind, member, other) in [
        ("unsupported", "observation", "why"),
        ("unsupported", "why", "observation"),
        ("unavailable", "operation", "detail"),
        ("unavailable", "detail", "operation"),
    ] {
        let mut error = serde_json::json!({"kind": kind});
        error[member] = "x".repeat(5000).into();
        error[other] = "short".into();
        let text = query_refused(&error).to_string();
        assert!(
            text.contains(&format!("{} … (904 bytes more)", "x".repeat(4096))),
            "{member}: {text}"
        );
    }
}

#[test]
fn a_report_through_a_process_target_writes_no_steering_character_the_target_sent() {
    let (raw, _) = steering();
    let echo = Echo::default();
    let handshake = serde_json::json!({"id": 0, "ok": {"speaks": "ess-conformance-target/1",
        "implementation": {"name": format!("n{raw}"), "version": format!("v{raw}")}}})
    .to_string();
    let refusal = serde_json::json!({"kind": "unsupported", "observation": format!("o{raw}"),
        "why": format!("w{raw}")});
    let peer = Served::new(&echo).on("handshake", 1, line(&handshake)).on(
        "query_view",
        1,
        Reply::Line(Box::new(move |id| {
            serde_json::json!({"id": id, "target_error": refusal})
                .to_string()
                .into_bytes()
        })),
    );
    let target = connect(peer);
    let identity = target.identity().unwrap();
    let error = target
        .begin_scenario(&first())
        .and_then(|()| target.query_view(view_request()))
        .unwrap_err();
    let text = format!("{identity} {error}");
    let steering: Vec<char> = text
        .chars()
        .filter(|c| STEERING.iter().any(|(s, _)| s == c))
        .collect();
    assert_eq!(steering, Vec::<char>::new(), "{text}");
}

/// The text report of a billing run through `serve`, every answer passed through `rewrite`.
fn text_report_rewritten(rewrite: impl Fn(&str) -> String + 'static) -> String {
    let admitted = billing();
    let served = crate::reference::Billing::new();
    let target = ProcessTarget::connect(
        Served::new(&served).rewriting(rewrite),
        HandshakeSuite::of(admitted),
        TIMEOUT_MS,
    )
    .unwrap();
    crate::Runner::for_suite(admitted.suite())
        .run_admitted(admitted, &target)
        .into_report()
        .to_string()
}

/// A view row whose `total` the target renamed to `total_as` and whose `issued_at` it set to
/// `issued_at`, both JSON text, as the text report quotes the row: a guard reading `total.amount`
/// cannot be decided on it.
fn quoted_row(total_as: &str, issued_at: &str) -> String {
    let (total_as, issued_at) = (total_as.to_owned(), issued_at.to_owned());
    text_report_rewritten(move |answer| {
        answer
            .replace(r#""total":"#, &format!("{total_as}:"))
            .replace(
                r#""issued_at":null"#,
                &format!(r#""issued_at":{issued_at}"#),
            )
    })
}

/// A member name a target sent holding ESC reaches the text report escaped, never raw.
#[test]
fn a_text_report_writes_an_escape_in_a_row_member_name_as_text() {
    let report = quoted_row(r#""to\u001btal""#, "null");
    assert!(!report.contains('\u{1b}'), "{report}");
    assert!(report.contains(r#"to\u001btal = {"amount""#), "{report}");
}

/// A value a target sent holding U+009B (CSI) or U+202E (right-to-left override) reaches the text
/// report escaped, never raw.
#[test]
fn a_text_report_writes_a_c1_control_or_a_bidirectional_override_in_a_value_as_text() {
    for raw in ['\u{9b}', '\u{202e}'] {
        let escaped = format!("\\u{:04x}", u32::from(raw));
        let report = quoted_row(r#""tootal""#, &format!(r#""text{escaped}text""#));
        assert!(!report.contains(raw), "{report}");
        assert!(
            report.contains(&format!(r#"issued_at = "text{escaped}text""#)),
            "{report}"
        );
    }
}

/// A member name and a value holding no such character print as they always printed.
#[test]
fn a_text_report_prints_an_ordinary_row_member_name_and_value_unchanged() {
    let report = quoted_row(r#""tootal""#, r#""plain""#);
    assert!(
        report.contains(r#"{invoice_id = "00000000-0000-4000-8000-000000000001", issued_at = "plain", reminder_count = 0.0, tootal = {"amount":1.0,"currency":"amount.currency"}} binds nothing for total.amount"#),
        "{report}"
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
