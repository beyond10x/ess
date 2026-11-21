//! Each operation's request and answer as lines spell them: the Rust `serde` members of the trait's
//! types, with every value map written and read exactly.
//!
//! Requests whose types hold no [`Node`] are written by their own `Serialize`; the others through a
//! borrowed copy here whose value maps are [`ExactMap`]s. `wire_requests_spell_the_derived_members`
//! holds the two spellings together. Answers are read by hand, closed, through [`exact`].

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use serde::Serialize;

use super::exact::{self, exact_opt, Bad, ExactMap, ExactNode, Kind, Value};
use super::HandshakeSuite;
use crate::scenario::{
    ActorRef, BindingRef, CommandRef, Elapsed, EntityRef, ErrorRef, EventRef, OutcomeRef, ViewRef,
};
use crate::target::{
    AbsentInputRequest, AwaitAnswer, AwaitRequest, Deadline, DeclaredErrorValue,
    ElapsedObservation, EntitySetupRequest, EventDeliveryRequest, ImplementationIdentity,
    ObservedEvent, ObservedInvocation, OrderedScan, OrderedScanRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};

// ---- requests, written -------------------------------------------------------------------------

#[derive(Serialize)]
pub(crate) struct HandshakeRequest<'a> {
    pub reads: &'a [&'a str],
    pub suite: &'a HandshakeSuite,
}

#[derive(Serialize)]
pub(crate) struct FixtureRequest<'a> {
    pub scenario: &'a ScenarioContext,
    pub contract: &'a crate::fixtures::Contract,
}

#[derive(Serialize)]
pub(crate) struct EntityRequest<'a> {
    entity: &'a EntityRef,
    identity: ExactNode<'a>,
    fields: ExactMap<'a>,
    state: &'a ess_domain::entity::StateName,
    correlation: &'a CorrelationId,
}

impl<'a> EntityRequest<'a> {
    pub fn of(request: &'a EntitySetupRequest) -> Self {
        Self {
            entity: &request.entity,
            identity: ExactNode(&request.identity),
            fields: ExactMap(&request.fields),
            state: &request.state,
            correlation: &request.correlation,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct CommandRequest<'a> {
    command: &'a CommandRef,
    actor: &'a Option<ActorRef>,
    caller: Option<ExactMap<'a>>,
    input: ExactMap<'a>,
    correlation: &'a CorrelationId,
}

impl<'a> CommandRequest<'a> {
    pub fn of(request: &'a SemanticCommandRequest) -> Self {
        Self {
            command: &request.command,
            actor: &request.actor,
            caller: exact_opt(request.caller.as_ref()),
            input: ExactMap(&request.input),
            correlation: &request.correlation,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct AbsentInputWire<'a> {
    command: &'a CommandRef,
    actor: &'a Option<ActorRef>,
    caller: Option<ExactMap<'a>>,
    correlation: &'a CorrelationId,
}

impl<'a> AbsentInputWire<'a> {
    pub fn of(request: &'a AbsentInputRequest) -> Self {
        Self {
            command: &request.command,
            actor: &request.actor,
            caller: exact_opt(request.caller.as_ref()),
            correlation: &request.correlation,
        }
    }
}

/// A value map omitted when empty, as `params` is.
fn nonempty(map: &BTreeMap<String, Node>) -> Option<ExactMap<'_>> {
    (!map.is_empty()).then_some(ExactMap(map))
}

#[derive(Serialize)]
pub(crate) struct ViewRequest<'a> {
    view: &'a ViewRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<ExactMap<'a>>,
    consistency: &'a QueryConsistency,
    correlation: &'a CorrelationId,
    deadline: &'a Deadline,
    #[serde(skip_serializing_if = "Option::is_none")]
    reader: Option<&'a ActorRef>,
}

impl<'a> ViewRequest<'a> {
    pub fn of(request: &'a SemanticViewRequest, reader: Option<&'a ActorRef>) -> Self {
        Self {
            view: &request.view,
            params: nonempty(&request.params),
            consistency: &request.consistency,
            correlation: &request.correlation,
            deadline: &request.deadline,
            reader,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ScanRequest<'a> {
    view: &'a ViewRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<ExactMap<'a>>,
    stop_after: usize,
    consistency: &'a QueryConsistency,
    correlation: &'a CorrelationId,
    deadline: &'a Deadline,
    #[serde(skip_serializing_if = "Option::is_none")]
    reader: Option<&'a ActorRef>,
}

impl<'a> ScanRequest<'a> {
    pub fn of(request: &'a OrderedScanRequest, reader: Option<&'a ActorRef>) -> Self {
        Self {
            view: &request.view,
            params: nonempty(&request.params),
            stop_after: request.stop_after,
            consistency: &request.consistency,
            correlation: &request.correlation,
            deadline: &request.deadline,
            reader,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct RepeatedOutcomeRequest<'a> {
    pub force: &'a OutcomeRef,
    pub correlation: &'a CorrelationId,
    pub times: NonZeroU32,
}

#[derive(Serialize)]
pub(crate) struct DeliveryRequest<'a> {
    event: &'a EventRef,
    authority: &'a str,
    payload: ExactMap<'a>,
    context: ExactMap<'a>,
    correlation: &'a CorrelationId,
}

impl<'a> DeliveryRequest<'a> {
    pub fn of(request: &'a EventDeliveryRequest) -> Self {
        Self {
            event: &request.event,
            authority: &request.authority,
            payload: ExactMap(&request.payload),
            context: ExactMap(&request.context),
            correlation: &request.correlation,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct AwaitWire<'a> {
    command: &'a CommandRef,
    instance: ExactNode<'a>,
    within: &'a Option<Elapsed>,
    deadline: &'a Deadline,
    compared: ExactMap<'a>,
    correlation: &'a CorrelationId,
}

impl<'a> AwaitWire<'a> {
    pub fn of(request: &'a AwaitRequest) -> Self {
        Self {
            command: &request.command,
            instance: ExactNode(&request.instance),
            within: &request.within,
            deadline: &request.deadline,
            compared: ExactMap(&request.compared),
            correlation: &request.correlation,
        }
    }
}

// ---- answers, read -----------------------------------------------------------------------------

/// The answer of an operation whose Rust result is `()`: `null`.
pub(crate) fn unit(value: &Value) -> Result<(), Bad> {
    match value.kind {
        Kind::Null => Ok(()),
        _ => Err(Bad::Shape(value.at)),
    }
}

/// A string member read by its type's own parsing `Deserialize`.
fn named<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, Bad> {
    exact::text(value)?;
    exact::serde(value)
}

pub(crate) fn handshake_answer(value: &Value) -> Result<(String, ImplementationIdentity), Bad> {
    let members = exact::object(value, &["speaks", "implementation"], &[])?;
    let speaks = exact::text(members.req("speaks"))?;
    let implementation = exact::object(members.req("implementation"), &["name", "version"], &[])?;
    Ok((
        speaks,
        ImplementationIdentity::new(
            exact::text(implementation.req("name"))?,
            exact::text(implementation.req("version"))?,
        ),
    ))
}

pub(crate) fn target_error(value: &Value) -> Result<TargetError, Bad> {
    let kind = match &value.kind {
        Kind::Map(map) => map.get("kind").map(exact::text).transpose()?,
        _ => None,
    };
    match kind.as_deref() {
        Some("unsupported") => {
            let members = exact::object(value, &["kind", "observation", "why"], &[])?;
            Ok(TargetError::unsupported(
                exact::text(members.req("observation"))?,
                exact::text(members.req("why"))?,
            ))
        }
        Some("unavailable") => {
            let members = exact::object(value, &["kind", "operation", "detail"], &[])?;
            Ok(TargetError::unavailable(
                exact::text(members.req("operation"))?,
                exact::text(members.req("detail"))?,
            ))
        }
        Some("not_granted") => {
            let members = exact::object(value, &["kind"], &["actor"])?;
            Ok(TargetError::NotGranted {
                actor: members.opt("actor").map(exact::text).transpose()?,
            })
        }
        _ => Err(Bad::Shape(value.at)),
    }
}

fn outcome_ref(value: &Value) -> Result<OutcomeRef, Bad> {
    let members = exact::object(value, &["command", "outcome"], &[])?;
    exact::text(members.req("command"))?;
    exact::text(members.req("outcome"))?;
    exact::serde(value)
}

fn declared_error(value: &Value) -> Result<DeclaredErrorValue, Bad> {
    let members = exact::object(value, &["error", "fields"], &[])?;
    Ok(DeclaredErrorValue {
        error: named::<ErrorRef>(members.req("error"))?,
        fields: exact::node_map(members.req("fields"))?,
    })
}

pub(crate) fn command_result(value: &Value) -> Result<SemanticCommandResult, Bad> {
    let members = exact::object(
        value,
        &[],
        &[
            "outcome",
            "error",
            "consistency",
            "direct_events",
            "response",
        ],
    )?;
    Ok(SemanticCommandResult {
        outcome: members.opt("outcome").map(outcome_ref).transpose()?,
        error: members.opt("error").map(declared_error).transpose()?,
        consistency: members
            .opt("consistency")
            .map(|token| {
                ConsistencyToken::new(exact::text(token)?).map_err(|_| Bad::Shape(token.at))
            })
            .transpose()?,
        direct_events: members
            .opt("direct_events")
            .map(events)
            .transpose()?
            .unwrap_or_default(),
        response: members.opt("response").map(exact::node_map).transpose()?,
    })
}

fn event(value: &Value) -> Result<ObservedEvent, Bad> {
    let members = exact::object(value, &["event", "payload"], &["correlation", "sequence"])?;
    Ok(ObservedEvent {
        event: named::<EventRef>(members.req("event"))?,
        payload: exact::node_map(members.req("payload"))?,
        correlation: members
            .opt("correlation")
            .map(named::<CorrelationId>)
            .transpose()?,
        sequence: members.opt("sequence").map(exact::unsigned).transpose()?,
    })
}

pub(crate) fn events(value: &Value) -> Result<Vec<ObservedEvent>, Bad> {
    exact::seq(value)?.iter().map(event).collect()
}

pub(crate) fn view_result(value: &Value) -> Result<SemanticViewResult, Bad> {
    let members = exact::object(value, &["rows"], &["total"])?;
    Ok(SemanticViewResult {
        rows: exact::seq(members.req("rows"))?
            .iter()
            .map(exact::node_map)
            .collect::<Result<_, _>>()?,
        total: members.opt("total").map(exact::unsigned).transpose()?,
    })
}

pub(crate) fn invocations(value: &Value) -> Result<Vec<ObservedInvocation>, Bad> {
    exact::seq(value)?
        .iter()
        .map(|item| {
            let members = exact::object(item, &["binding", "command", "input"], &[])?;
            Ok(ObservedInvocation {
                binding: named::<BindingRef>(members.req("binding"))?,
                command: named::<CommandRef>(members.req("command"))?,
                input: exact::node_map(members.req("input"))?,
            })
        })
        .collect()
}

fn size(value: &Value) -> Result<usize, Bad> {
    usize::try_from(exact::unsigned(value)?).map_err(|_| Bad::Shape(value.at))
}

pub(crate) fn elapsed(value: &Value) -> Result<ElapsedObservation, Bad> {
    let members = exact::object(value, &["elapsed_ms", "published"], &[])?;
    Ok(ElapsedObservation {
        elapsed_ms: exact::unsigned(members.req("elapsed_ms"))?,
        published: size(members.req("published"))?,
    })
}

pub(crate) fn scan(value: &Value) -> Result<OrderedScan, Bad> {
    let members = exact::object(value, &["produced", "halted"], &[])?;
    Ok(OrderedScan {
        produced: size(members.req("produced"))?,
        halted: exact::boolean(members.req("halted"))?,
    })
}

pub(crate) fn await_answer(value: &Value) -> Result<AwaitAnswer, Bad> {
    let answer = match &value.kind {
        Kind::Map(map) => map.get("answer").map(exact::text).transpose()?,
        _ => None,
    };
    match answer.as_deref() {
        Some("reached") => {
            let members = exact::object(value, &["answer", "result", "input"], &["elapsed_ms"])?;
            Ok(AwaitAnswer::Reached {
                result: command_result(members.req("result"))?,
                input: exact::node_map(members.req("input"))?,
                elapsed_ms: members.opt("elapsed_ms").map(exact::unsigned).transpose()?,
            })
        }
        Some("not_issued") => {
            let members = exact::object(value, &["answer"], &["elapsed_ms"])?;
            Ok(AwaitAnswer::NotIssued {
                elapsed_ms: members.opt("elapsed_ms").map(exact::unsigned).transpose()?,
            })
        }
        _ => Err(Bad::Shape(value.at)),
    }
}

pub(crate) fn opened(value: &Value) -> Result<crate::periodic::Opened, Bad> {
    let members = exact::object(value, &["scope", "anchor_ms", "context"], &[])?;
    Ok(crate::periodic::Opened {
        scope: exact::serde(members.req("scope"))?,
        anchor_ms: exact::unsigned(members.req("anchor_ms"))?,
        context: exact::node_map(members.req("context"))?,
    })
}

fn fact(value: &Value) -> Result<crate::periodic::Fact, Bad> {
    let kind = match &value.kind {
        Kind::Map(map) => map.get("kind").map(exact::text).transpose()?,
        _ => None,
    };
    if kind.as_deref() != Some("invoked") {
        // Every other fact holds no value map; its own `Deserialize` is closed.
        return exact::serde(value);
    }
    let members = exact::object(
        value,
        &[
            "kind",
            "ordinal",
            "at_ms",
            "invocation",
            "command",
            "read",
            "input",
        ],
        &[],
    )?;
    Ok(crate::periodic::Fact::Invoked {
        ordinal: exact::unsigned(members.req("ordinal"))?,
        at_ms: exact::unsigned(members.req("at_ms"))?,
        invocation: exact::text(members.req("invocation"))?,
        command: named::<CommandRef>(members.req("command"))?,
        read: exact::node_map(members.req("read"))?,
        input: exact::node_map(members.req("input"))?,
    })
}

pub(crate) fn observation(value: &Value) -> Result<crate::periodic::Observation, Bad> {
    let members = exact::object(
        value,
        &["elapsed_ms", "complete_through_ms", "cursor", "records"],
        &[],
    )?;
    Ok(crate::periodic::Observation {
        elapsed_ms: exact::unsigned(members.req("elapsed_ms"))?,
        complete_through_ms: exact::unsigned(members.req("complete_through_ms"))?,
        cursor: exact::unsigned(members.req("cursor"))?,
        records: exact::seq(members.req("records"))?
            .iter()
            .map(|record| -> Result<_, Bad> {
                let members = exact::object(record, &["scope", "fact"], &[])?;
                Ok(crate::periodic::Record {
                    scope: exact::serde(members.req("scope"))?,
                    fact: fact(members.req("fact"))?,
                })
            })
            .collect::<Result<_, _>>()?,
    })
}

pub(crate) fn closed(value: &Value) -> Result<crate::periodic::Closed, Bad> {
    exact::serde(value)
}

// ---- answers, written (by `serve`) -------------------------------------------------------------

#[derive(Serialize)]
pub(crate) struct HandshakeAnswer<'a> {
    pub speaks: &'a str,
    pub implementation: &'a ImplementationIdentity,
}

#[derive(Serialize)]
struct ErrorWire<'a> {
    error: &'a ErrorRef,
    fields: ExactMap<'a>,
}

#[derive(Serialize)]
pub(crate) struct EventWire<'a> {
    event: &'a EventRef,
    payload: ExactMap<'a>,
    correlation: &'a Option<CorrelationId>,
    sequence: Option<u64>,
}

impl<'a> EventWire<'a> {
    pub fn of(event: &'a ObservedEvent) -> Self {
        Self {
            event: &event.event,
            payload: ExactMap(&event.payload),
            correlation: &event.correlation,
            sequence: event.sequence,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ResultWire<'a> {
    outcome: &'a Option<OutcomeRef>,
    error: Option<ErrorWire<'a>>,
    consistency: &'a Option<ConsistencyToken>,
    direct_events: Vec<EventWire<'a>>,
    response: Option<ExactMap<'a>>,
}

impl<'a> ResultWire<'a> {
    pub fn of(result: &'a SemanticCommandResult) -> Self {
        Self {
            outcome: &result.outcome,
            error: result.error.as_ref().map(|error| ErrorWire {
                error: &error.error,
                fields: ExactMap(&error.fields),
            }),
            consistency: &result.consistency,
            direct_events: result.direct_events.iter().map(EventWire::of).collect(),
            response: exact_opt(result.response.as_ref()),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ViewResultWire<'a> {
    rows: Vec<ExactMap<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<u64>,
}

impl<'a> ViewResultWire<'a> {
    pub fn of(result: &'a SemanticViewResult) -> Self {
        Self {
            rows: result.rows.iter().map(ExactMap).collect(),
            total: result.total,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct InvocationWire<'a> {
    binding: &'a BindingRef,
    command: &'a CommandRef,
    input: ExactMap<'a>,
}

impl<'a> InvocationWire<'a> {
    pub fn of(invocation: &'a ObservedInvocation) -> Self {
        Self {
            binding: &invocation.binding,
            command: &invocation.command,
            input: ExactMap(&invocation.input),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "answer", rename_all = "snake_case")]
pub(crate) enum AwaitAnswerWire<'a> {
    Reached {
        result: ResultWire<'a>,
        input: ExactMap<'a>,
        elapsed_ms: Option<u64>,
    },
    NotIssued {
        elapsed_ms: Option<u64>,
    },
}

impl<'a> AwaitAnswerWire<'a> {
    pub fn of(answer: &'a AwaitAnswer) -> Self {
        match answer {
            AwaitAnswer::Reached {
                result,
                input,
                elapsed_ms,
            } => Self::Reached {
                result: ResultWire::of(result),
                input: ExactMap(input),
                elapsed_ms: *elapsed_ms,
            },
            AwaitAnswer::NotIssued { elapsed_ms } => Self::NotIssued {
                elapsed_ms: *elapsed_ms,
            },
        }
    }
}

#[derive(Serialize)]
pub(crate) struct OpenedWire<'a> {
    scope: &'a crate::periodic::Scope,
    anchor_ms: u64,
    context: ExactMap<'a>,
}

impl<'a> OpenedWire<'a> {
    pub fn of(opened: &'a crate::periodic::Opened) -> Self {
        Self {
            scope: &opened.scope,
            anchor_ms: opened.anchor_ms,
            context: ExactMap(&opened.context),
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum FactWire<'a> {
    Invoked {
        kind: &'static str,
        ordinal: u64,
        at_ms: u64,
        invocation: &'a str,
        command: &'a CommandRef,
        read: ExactMap<'a>,
        input: ExactMap<'a>,
    },
    Other(&'a crate::periodic::Fact),
}

#[derive(Serialize)]
struct RecordWire<'a> {
    scope: &'a crate::periodic::Scope,
    fact: FactWire<'a>,
}

#[derive(Serialize)]
pub(crate) struct ObservationWire<'a> {
    elapsed_ms: u64,
    complete_through_ms: u64,
    cursor: u64,
    records: Vec<RecordWire<'a>>,
}

impl<'a> ObservationWire<'a> {
    pub fn of(observation: &'a crate::periodic::Observation) -> Self {
        Self {
            elapsed_ms: observation.elapsed_ms,
            complete_through_ms: observation.complete_through_ms,
            cursor: observation.cursor,
            records: observation
                .records
                .iter()
                .map(|record| RecordWire {
                    scope: &record.scope,
                    fact: match &record.fact {
                        crate::periodic::Fact::Invoked {
                            ordinal,
                            at_ms,
                            invocation,
                            command,
                            read,
                            input,
                        } => FactWire::Invoked {
                            kind: "invoked",
                            ordinal: *ordinal,
                            at_ms: *at_ms,
                            invocation,
                            command,
                            read: ExactMap(read),
                            input: ExactMap(input),
                        },
                        other => FactWire::Other(other),
                    },
                })
                .collect(),
        }
    }
}

// ---- requests, read (by `serve`) ---------------------------------------------------------------

/// The member `name` of an open request object.
pub(crate) fn member<'v>(
    map: &'v BTreeMap<String, Value>,
    name: &str,
    whole: &Value,
) -> Result<&'v Value, Bad> {
    map.get(name).ok_or(Bad::Shape(whole.at))
}

/// An optional value map of a request: `None` when absent or `null`.
fn opt_map(
    map: &BTreeMap<String, Value>,
    name: &str,
) -> Result<Option<BTreeMap<String, Node>>, Bad> {
    match map.get(name) {
        None => Ok(None),
        Some(value) if matches!(value.kind, Kind::Null) => Ok(None),
        Some(value) => exact::node_map(value).map(Some),
    }
}

pub(crate) fn entity_request(value: &Value) -> Result<EntitySetupRequest, Bad> {
    let map = exact::open_object(value)?;
    let mut request: EntitySetupRequest = exact::serde(value)?;
    request.identity = exact::node(member(map, "identity", value)?);
    request.fields = exact::node_map(member(map, "fields", value)?)?;
    Ok(request)
}

pub(crate) fn command_request(value: &Value) -> Result<SemanticCommandRequest, Bad> {
    let map = exact::open_object(value)?;
    let mut request: SemanticCommandRequest = exact::serde(value)?;
    request.input = exact::node_map(member(map, "input", value)?)?;
    request.caller = opt_map(map, "caller")?;
    Ok(request)
}

pub(crate) fn absent_input_request(value: &Value) -> Result<AbsentInputRequest, Bad> {
    let map = exact::open_object(value)?;
    let mut request: AbsentInputRequest = exact::serde(value)?;
    request.caller = opt_map(map, "caller")?;
    Ok(request)
}

pub(crate) fn view_request(value: &Value) -> Result<SemanticViewRequest, Bad> {
    let map = exact::open_object(value)?;
    let mut request: SemanticViewRequest = exact::serde(value)?;
    request.params = opt_map(map, "params")?.unwrap_or_default();
    Ok(request)
}

pub(crate) fn scan_request(value: &Value) -> Result<OrderedScanRequest, Bad> {
    let map = exact::open_object(value)?;
    let mut request: OrderedScanRequest = exact::serde(value)?;
    request.params = opt_map(map, "params")?.unwrap_or_default();
    Ok(request)
}

pub(crate) fn reader(value: &Value) -> Result<ActorRef, Bad> {
    let map = exact::open_object(value)?;
    named(member(map, "reader", value)?)
}

pub(crate) fn delivery_request(value: &Value) -> Result<EventDeliveryRequest, Bad> {
    let map = exact::open_object(value)?;
    let mut request: EventDeliveryRequest = exact::serde(value)?;
    request.payload = exact::node_map(member(map, "payload", value)?)?;
    request.context = exact::node_map(member(map, "context", value)?)?;
    Ok(request)
}

pub(crate) fn await_request(value: &Value) -> Result<AwaitRequest, Bad> {
    let map = exact::open_object(value)?;
    let mut request: AwaitRequest = exact::serde(value)?;
    request.instance = exact::node(member(map, "instance", value)?);
    request.compared = exact::node_map(member(map, "compared", value)?)?;
    Ok(request)
}

pub(crate) fn times(value: &Value) -> Result<NonZeroU32, Bad> {
    let map = exact::open_object(value)?;
    exact::serde(member(map, "times", value)?)
}

pub(crate) fn fixture_request(
    value: &Value,
) -> Result<(ScenarioContext, crate::fixtures::Contract), Bad> {
    let map = exact::open_object(value)?;
    Ok((
        exact::serde(member(map, "scenario", value)?)?,
        exact::serde(member(map, "contract", value)?)?,
    ))
}
