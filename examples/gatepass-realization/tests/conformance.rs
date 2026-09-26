//! The committed gatepass suite, unchanged, against both synthesised realizations of it.
//!
//! `suites/generated/gatepass/suite.json` runs here twice: in process against the Rust workspace
//! linked with this crate's hand-written obligations, and through the emitted Go runner against the
//! Go module linked with `examples/gatepass-go-realization/`. Both must pass every scenario — among
//! them the two unknown-instance scenarios (`<command>/outcome/wrong-state`,
//! `docs/design/typed-literals-and-unknown-instances.md`), which a seam whose `wrong-state` variant
//! demands a state could only answer with the unmet obligation its served surface reports as `501`.
//!
//! # The bridges are adapters, not implementations
//!
//! [`Synthesized`] implements [`ConformanceTarget`] over the linked Rust system, and
//! `tests/fixtures/go-target/target.go` implements the emitted runner's `Target` over the linked Go
//! system. Their whole job is representation: suite values into the generated types on the way in,
//! typed outcomes, events and rows back into suite values on the way out. Every observation either
//! reports is read off the system — its log and its view ports; nothing is decided there.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::report::ConformanceStatus;
use ess_conformance::runner::Runner;
use ess_conformance::scenario::{CommandRef, ConformanceSuite, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::{
    ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError, ViewRow,
};
use ess_primitives::consistency::ConsistencyToken;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use gatepass_realization::linker::{self, Assembled};
use gatepass_system::SystemEvent;
use gatepass_types::obligation::UnmetObligation;
use gatepass_types::primitives::{Decimal, Duration, Timestamp, Uuid};
use gatepass_types::visit::{
    AdmitVisitor, AdmitVisitorOutcome, Badge, Building, Deposit, EmployeeId, Host, RegisterVisit,
    RegisterVisitOutcome, SignOutVisitor, SignOutVisitorOutcome, VendorRef, VisitId, VisitState,
    VisitorName,
};

// ---- the names the specification declares ----------------------------------------------------

const REGISTER_VISIT: &str = "gatepass.visit.RegisterVisit";
const ADMIT_VISITOR: &str = "gatepass.visit.AdmitVisitor";
const SIGN_OUT_VISITOR: &str = "gatepass.visit.SignOutVisitor";

const INVALID_VISIT_LENGTH: &str = "gatepass.visit.InvalidVisitLength";
const VISIT_STATE_CONFLICT: &str = "gatepass.visit.VisitStateConflict";

const VISIT_REGISTERED: &str = "gatepass.visit.VisitRegistered";
const VISITOR_ADMITTED: &str = "gatepass.visit.VisitorAdmitted";
const VISITOR_DEPARTED: &str = "gatepass.visit.VisitorDeparted";

const EXPECTED_VISITS: &str = "gatepass.visit.ExpectedVisits";
const VISIT_BY_ID: &str = "gatepass.visit.VisitById";

/// The scenarios the unknown-instance rule adds, which the stated purpose of this file turns on.
const UNKNOWN_INSTANCE: [&str; 2] = [
    "gatepass.visit.AdmitVisitor/outcome/wrong-state",
    "gatepass.visit.SignOutVisitor/outcome/wrong-state",
];

/// How many scenarios the committed suite holds; the criterion is all of them.
const SCENARIOS: usize = 14;

// ---- the Rust target -------------------------------------------------------------------------

/// One scenario's linked system, plus the adapter's own token mint.
struct Live {
    assembled: Assembled,
    sequence: u64,
}

/// The generated gatepass workspace, linked and adapted to the conformance target interface.
struct Synthesized {
    live: RefCell<Option<Live>>,
}

/// A borrow of the open scenario, or the refusal that none is open.
fn open(live: &mut Option<Live>) -> Result<&mut Live, TargetError> {
    live.as_mut()
        .ok_or_else(|| TargetError::unavailable("driving the system", "no scenario is open"))
}

/// The next consistency token of this scenario, from a counter rather than a clock.
fn token(live: &mut Live) -> Result<ConsistencyToken, TargetError> {
    live.sequence += 1;
    ConsistencyToken::new(format!("seq:{}", live.sequence))
        .map_err(|error| TargetError::unavailable("minting a consistency token", error.to_string()))
}

impl ConformanceTarget for Synthesized {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "gatepass-synthesized",
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn begin_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        // A freshly linked system per scenario: the suite runs against what the linker produced.
        *self.live.borrow_mut() = Some(Live {
            assembled: linker::honest(),
            sequence: 0,
        });
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut live = self.live.borrow_mut();
        let live = open(&mut live)?;
        let result = match request.command.to_string().as_str() {
            REGISTER_VISIT => register_visit(live, &request)?,
            ADMIT_VISITOR => admit_visitor(live, &request)?,
            SIGN_OUT_VISITOR => sign_out_visitor(live, &request)?,
            other => {
                return Err(TargetError::unavailable(
                    format!("invoking `{other}`"),
                    "this system accepts only the commands `examples/gatepass/` declares",
                ))
            }
        };
        live.assembled
            .system
            .pump()
            .map_err(|refusal| unmet(&refusal))?;
        Ok(result)
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut live = self.live.borrow_mut();
        let live = open(&mut live)?;
        let service = &live.assembled.system.pass_service;
        // Both projections are served straight off the store, so every consistency a request can
        // demand is already met.
        match request.view.to_string().as_str() {
            EXPECTED_VISITS => {
                let rows = service
                    .expected_visits()
                    .map_err(|refusal| unmet(&refusal))?;
                Ok(SemanticViewResult::of(rows.iter().map(|row| {
                    ViewRow::from([
                        ("visit_id".to_owned(), visit_id_node(&row.visit_id)),
                        ("visitor".to_owned(), Node::Text(row.visitor.0.clone())),
                        ("building".to_owned(), building_node(row.building)),
                        ("deposit".to_owned(), deposit_node(&row.deposit)),
                    ])
                })))
            }
            VISIT_BY_ID => {
                let rows = service.visit_by_id().map_err(|refusal| unmet(&refusal))?;
                Ok(SemanticViewResult::of(rows.iter().map(|row| {
                    ViewRow::from([
                        ("visit_id".to_owned(), visit_id_node(&row.visit_id)),
                        ("visitor".to_owned(), Node::Text(row.visitor.0.clone())),
                        ("host".to_owned(), host_node(&row.host)),
                        (
                            "escorts".to_owned(),
                            Node::Seq(
                                row.escorts
                                    .iter()
                                    .map(|name| Node::Text(name.0.clone()))
                                    .collect(),
                            ),
                        ),
                        (
                            "notes".to_owned(),
                            Node::Map(
                                row.notes
                                    .iter()
                                    .map(|(key, value)| (key.clone(), Node::Text(value.clone())))
                                    .collect(),
                            ),
                        ),
                        (
                            "badge".to_owned(),
                            row.badge.as_ref().map_or(Node::Null, badge_node),
                        ),
                    ])
                })))
            }
            other => Err(TargetError::unavailable(
                format!("reading `{other}`"),
                "this system projects only the views `examples/gatepass/` declares",
            )),
        }
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let mut live = self.live.borrow_mut();
        let live = open(&mut live)?;
        Ok(live
            .assembled
            .system
            .published()
            .iter()
            .enumerate()
            .map(|(position, event)| {
                observed(event)
                    .in_activity(request.correlation.clone())
                    .at(position as u64)
            })
            .filter(|event| event.event == request.event)
            .collect())
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        Err(TargetError::unavailable(
            format!("forcing `{}`", request.force),
            "`examples/gatepass/` declares no external outcome",
        ))
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unavailable(
            format!("delivering `{}` again", request.event),
            "`examples/gatepass/` declares no binding to deliver to",
        ))
    }

    fn observe_invocations(
        &self,
        _request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Ok(Vec::new())
    }

    fn end_scenario(&self, _scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.live.borrow_mut() = None;
        Ok(())
    }
}

// ---- commands, one function each -------------------------------------------------------------

/// `gatepass.visit.RegisterVisit`, through the generated port.
fn register_visit(
    live: &mut Live,
    request: &SemanticCommandRequest,
) -> Result<SemanticCommandResult, TargetError> {
    let input = RegisterVisit {
        visitor: VisitorName(text(request, "visitor")?),
        building: building(request)?,
        host: host(request)?,
        expected_minutes: integer(request, "expected_minutes")?,
        expected_stay: Duration(text(request, "expected_stay")?),
        deposit: deposit(request)?,
        escorts: escorts(request)?,
        notes: notes(request)?,
        on_watchlist: boolean(request, "on_watchlist")?,
    };
    let outcome = live
        .assembled
        .system
        .pass_service
        .register_visit(input)
        .map_err(|refusal| unmet(&refusal))?;
    Ok(match outcome {
        RegisterVisitOutcome::Registered { visit_registered } => {
            let published = observed(&SystemEvent::VisitRegistered(visit_registered))
                .in_activity(request.correlation.clone());
            SemanticCommandResult::took(outcome_ref(REGISTER_VISIT, "registered"))
                .with_consistency(token(live)?)
                .emitting(published)
        }
        RegisterVisitOutcome::Refused { error } => {
            SemanticCommandResult::took(outcome_ref(REGISTER_VISIT, "refused")).with_error(
                DeclaredErrorValue::new(error_ref(INVALID_VISIT_LENGTH))
                    .with("submitted", integer_node(error.submitted)),
            )
        }
    })
}

/// `gatepass.visit.AdmitVisitor`, through the generated port.
fn admit_visitor(
    live: &mut Live,
    request: &SemanticCommandRequest,
) -> Result<SemanticCommandResult, TargetError> {
    let input = AdmitVisitor {
        visit_id: visit_id(request)?,
        badge: badge(field(request, "badge")?, "badge")?,
    };
    let outcome = live
        .assembled
        .system
        .pass_service
        .admit_visitor(input)
        .map_err(|refusal| unmet(&refusal))?;
    Ok(match outcome {
        AdmitVisitorOutcome::Admitted { visitor_admitted } => {
            let published = observed(&SystemEvent::VisitorAdmitted(visitor_admitted))
                .in_activity(request.correlation.clone());
            SemanticCommandResult::took(outcome_ref(ADMIT_VISITOR, "admitted"))
                .with_consistency(token(live)?)
                .emitting(published)
        }
        AdmitVisitorOutcome::WrongState { error } => wrong_state(ADMIT_VISITOR, error.state),
        AdmitVisitorOutcome::WrongStateUnknownInstance => unknown_visit(ADMIT_VISITOR),
    })
}

/// `gatepass.visit.SignOutVisitor`, through the generated port.
fn sign_out_visitor(
    live: &mut Live,
    request: &SemanticCommandRequest,
) -> Result<SemanticCommandResult, TargetError> {
    let input = SignOutVisitor {
        visit_id: visit_id(request)?,
    };
    let outcome = live
        .assembled
        .system
        .pass_service
        .sign_out_visitor(input)
        .map_err(|refusal| unmet(&refusal))?;
    Ok(match outcome {
        SignOutVisitorOutcome::SignedOut { visitor_departed } => {
            let published = observed(&SystemEvent::VisitorDeparted(visitor_departed))
                .in_activity(request.correlation.clone());
            SemanticCommandResult::took(outcome_ref(SIGN_OUT_VISITOR, "signed-out"))
                .with_consistency(token(live)?)
                .emitting(published)
        }
        SignOutVisitorOutcome::WrongState { error } => wrong_state(SIGN_OUT_VISITOR, error.state),
        SignOutVisitorOutcome::WrongStateUnknownInstance => unknown_visit(SIGN_OUT_VISITOR),
    })
}

/// The `wrong-state` branch for a visit that exists, with the state it is really in.
fn wrong_state(command: &str, state: VisitState) -> SemanticCommandResult {
    let declared = match state {
        VisitState::Departed => "Departed",
        VisitState::Expected => "Expected",
        VisitState::OnSite => "OnSite",
    };
    SemanticCommandResult::took(outcome_ref(command, "wrong-state")).with_error(
        DeclaredErrorValue::new(error_ref(VISIT_STATE_CONFLICT))
            .with("state", Node::Text(declared.to_owned())),
    )
}

/// The `wrong-state` branch for a visit no record carries: the declared error, and no `state`.
fn unknown_visit(command: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(outcome_ref(command, "wrong-state"))
        .with_error(DeclaredErrorValue::new(error_ref(VISIT_STATE_CONFLICT)))
}

/// An unmet obligation surfacing to the runner — on the served surface, the `501` naming it.
fn unmet(refusal: &UnmetObligation) -> TargetError {
    TargetError::unavailable("driving the linked system", refusal.to_string())
}

// ---- representation: nodes in ----------------------------------------------------------------

/// The refusal for an input this adapter cannot read.
fn unreadable(at: &str, expected: &str) -> TargetError {
    TargetError::unavailable(
        format!("reading the input `{at}`"),
        format!("the suite carries {expected} there"),
    )
}

/// A required input field.
fn field<'a>(request: &'a SemanticCommandRequest, name: &str) -> Result<&'a Node, TargetError> {
    request
        .input
        .get(name)
        .ok_or_else(|| unreadable(name, "a value"))
}

/// Text at `node`.
fn text_of(node: &Node, at: &str) -> Result<String, TargetError> {
    node.as_text()
        .map(ToOwned::to_owned)
        .ok_or_else(|| unreadable(at, "text"))
}

/// A required text input.
fn text(request: &SemanticCommandRequest, name: &str) -> Result<String, TargetError> {
    text_of(field(request, name)?, name)
}

/// A required boolean input.
fn boolean(request: &SemanticCommandRequest, name: &str) -> Result<bool, TargetError> {
    match field(request, name)? {
        Node::Bool(value) => Ok(*value),
        _ => Err(unreadable(name, "a boolean")),
    }
}

/// A required `Integer` input, which the suite carries as an integral number.
fn integer(request: &SemanticCommandRequest, name: &str) -> Result<i64, TargetError> {
    match field(request, name)? {
        Node::Number(number) => {
            let value = number.get();
            if value.fract() == 0.0 && value.abs() < 9_007_199_254_740_992.0 {
                #[allow(clippy::cast_possible_truncation)]
                Ok(value as i64)
            } else {
                Err(unreadable(name, "an integral number"))
            }
        }
        _ => Err(unreadable(name, "a number")),
    }
}

/// The visit a command's input names.
fn visit_id(request: &SemanticCommandRequest) -> Result<VisitId, TargetError> {
    Ok(VisitId(Uuid(text(request, "visit_id")?)))
}

/// The `Building` input, one of its declared variant names.
fn building(request: &SemanticCommandRequest) -> Result<Building, TargetError> {
    match text(request, "building")?.as_str() {
        "North" => Ok(Building::North),
        "South" => Ok(Building::South),
        "Annex" => Ok(Building::Annex),
        _ => Err(unreadable("building", "a declared `Building` variant")),
    }
}

/// The `Host` input: a map tagged by `kind`, holding its shape under `value`.
fn host(request: &SemanticCommandRequest) -> Result<Host, TargetError> {
    let fields = field(request, "host")?
        .as_map()
        .ok_or_else(|| unreadable("host", "a tagged map"))?;
    let value = text_of(
        fields
            .get("value")
            .ok_or_else(|| unreadable("host.value", "text"))?,
        "host.value",
    )?;
    match fields.get("kind").and_then(Node::as_text) {
        Some("employee") => Ok(Host::Employee(EmployeeId(value))),
        Some("contractor") => Ok(Host::Contractor(VendorRef(value))),
        _ => Err(unreadable("host.kind", "`employee` or `contractor`")),
    }
}

/// The `Deposit` input: a numeric `amount` and a text `currency`.
fn deposit(request: &SemanticCommandRequest) -> Result<Deposit, TargetError> {
    let fields = field(request, "deposit")?
        .as_map()
        .ok_or_else(|| unreadable("deposit", "a map"))?;
    let amount = match fields.get("amount") {
        Some(Node::Number(number)) => format!("{}", number.get()),
        _ => return Err(unreadable("deposit.amount", "a number")),
    };
    Ok(Deposit {
        amount: Decimal(amount),
        currency: text_of(
            fields
                .get("currency")
                .ok_or_else(|| unreadable("deposit.currency", "text"))?,
            "deposit.currency",
        )?,
    })
}

/// The `escorts` input: a list of names.
fn escorts(request: &SemanticCommandRequest) -> Result<Vec<VisitorName>, TargetError> {
    match field(request, "escorts")? {
        Node::Seq(items) => items
            .iter()
            .map(|item| text_of(item, "escorts[]").map(VisitorName))
            .collect(),
        _ => Err(unreadable("escorts", "a list")),
    }
}

/// The `notes` input: text by text.
fn notes(request: &SemanticCommandRequest) -> Result<BTreeMap<String, String>, TargetError> {
    let fields = field(request, "notes")?
        .as_map()
        .ok_or_else(|| unreadable("notes", "a map"))?;
    fields
        .iter()
        .map(|(key, value)| Ok((key.clone(), text_of(value, "notes[]")?)))
        .collect()
}

/// A `Badge`: `serial`, an optional `printed_at`, and a base64 `signature`.
fn badge(node: &Node, at: &str) -> Result<Badge, TargetError> {
    let fields = node.as_map().ok_or_else(|| unreadable(at, "a map"))?;
    let printed_at = match fields.get("printed_at") {
        None | Some(Node::Null) => None,
        Some(value) => Some(Timestamp(text_of(value, "badge.printed_at")?)),
    };
    let signature = fields
        .get("signature")
        .and_then(Node::as_text)
        .and_then(base64_decode)
        .ok_or_else(|| unreadable("badge.signature", "base64 text"))?;
    Ok(Badge {
        serial: text_of(
            fields
                .get("serial")
                .ok_or_else(|| unreadable("badge.serial", "text"))?,
            "badge.serial",
        )?,
        printed_at,
        signature,
    })
}

// ---- representation: nodes out ---------------------------------------------------------------

/// A `VisitId` as the text node the suite compares.
fn visit_id_node(id: &VisitId) -> Node {
    Node::Text(id.0 .0.clone())
}

/// A `Building` as its declared variant name.
fn building_node(building: Building) -> Node {
    Node::Text(
        match building {
            Building::North => "North",
            Building::South => "South",
            Building::Annex => "Annex",
        }
        .to_owned(),
    )
}

/// A `Host` as the tagged map the suite carries.
fn host_node(host: &Host) -> Node {
    let (kind, value) = match host {
        Host::Employee(id) => ("employee", id.0.clone()),
        Host::Contractor(reference) => ("contractor", reference.0.clone()),
    };
    Node::Map(BTreeMap::from([
        ("kind".to_owned(), Node::Text(kind.to_owned())),
        ("value".to_owned(), Node::Text(value)),
    ]))
}

/// A `Deposit`, its decimal amount as a number.
///
/// A rendering this adapter did not mint may not parse; that surfaces as text, which the shape
/// check reports readably instead of this adapter guessing a number.
fn deposit_node(deposit: &Deposit) -> Node {
    let amount = deposit
        .amount
        .0
        .parse::<f64>()
        .ok()
        .and_then(|value| Number::new(value).ok())
        .map_or_else(|| Node::Text(deposit.amount.0.clone()), Node::Number);
    Node::Map(BTreeMap::from([
        ("amount".to_owned(), amount),
        ("currency".to_owned(), Node::Text(deposit.currency.clone())),
    ]))
}

/// A `Badge`, its signature as base64 text.
fn badge_node(badge: &Badge) -> Node {
    Node::Map(BTreeMap::from([
        ("serial".to_owned(), Node::Text(badge.serial.clone())),
        (
            "printed_at".to_owned(),
            badge
                .printed_at
                .as_ref()
                .map_or(Node::Null, |at| Node::Text(at.0.clone())),
        ),
        (
            "signature".to_owned(),
            Node::Text(base64_encode(&badge.signature)),
        ),
    ]))
}

/// An `Integer` as a number, or as text where a double would not hold it exactly.
fn integer_node(value: i64) -> Node {
    i32::try_from(value)
        .ok()
        .map(f64::from)
        .and_then(|number| Number::new(number).ok())
        .map_or_else(|| Node::Text(value.to_string()), Node::Number)
}

/// One logged occurrence, rendered for the runner.
fn observed(event: &SystemEvent) -> ObservedEvent {
    match event {
        SystemEvent::VisitRegistered(event) => ObservedEvent::new(event_ref(VISIT_REGISTERED))
            .with("visit_id", visit_id_node(&event.visit_id))
            .with("visitor", Node::Text(event.visitor.0.clone()))
            .with("building", building_node(event.building)),
        SystemEvent::VisitorAdmitted(event) => ObservedEvent::new(event_ref(VISITOR_ADMITTED))
            .with("visit_id", visit_id_node(&event.visit_id))
            .with("badge", badge_node(&event.badge)),
        SystemEvent::VisitorDeparted(event) => ObservedEvent::new(event_ref(VISITOR_DEPARTED))
            .with("visit_id", visit_id_node(&event.visit_id)),
    }
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard padded base64, the `Bytes` rendering the suite carries.
fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let group = chunk.iter().enumerate().fold(0_u32, |acc, (index, byte)| {
            acc | u32::from(*byte) << (16 - 8 * index)
        });
        for index in 0..4 {
            if index <= chunk.len() {
                out.push(char::from(
                    BASE64[(group >> (18 - 6 * index)) as usize & 63],
                ));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Standard padded base64 back into bytes, or `None` where it is not that.
fn base64_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::new();
    for chunk in text.as_bytes().chunks(4) {
        let padding = chunk.iter().rev().take_while(|byte| **byte == b'=').count();
        let mut group = 0_u32;
        for (index, byte) in chunk.iter().enumerate() {
            let sextet = if index >= 4 - padding {
                0
            } else {
                u32::try_from(BASE64.iter().position(|candidate| candidate == byte)?).ok()?
            };
            group |= sextet << (18 - 6 * index);
        }
        for index in 0..3 - padding {
            out.push(u8::try_from((group >> (16 - 8 * index)) & 0xff).ok()?);
        }
    }
    Some(out)
}

// ---- names, parsed once ----------------------------------------------------------------------

/// A command this module names as a literal.
fn command_ref(value: &str) -> CommandRef {
    value
        .parse()
        .unwrap_or_else(|error| panic!("`{value}` is a well-formed command: {error}"))
}

/// An event this module names as a literal.
fn event_ref(value: &str) -> EventRef {
    value
        .parse()
        .unwrap_or_else(|error| panic!("`{value}` is a well-formed event: {error}"))
}

/// A declared error this module names as a literal.
fn error_ref(value: &str) -> ErrorRef {
    value
        .parse()
        .unwrap_or_else(|error| panic!("`{value}` is a well-formed error: {error}"))
}

/// One branch of one command.
fn outcome_ref(command: &str, branch: &str) -> OutcomeRef {
    OutcomeRef::new(
        command_ref(command),
        branch
            .parse()
            .unwrap_or_else(|error| panic!("`{branch}` is a well-formed outcome name: {error}")),
    )
}

// ---- the suite, unchanged --------------------------------------------------------------------

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// The committed suite, exactly as synthesis wrote it.
fn committed_suite() -> ConformanceSuite {
    let suite = ConformanceSuite::from_json(include_str!(
        "../../../suites/generated/gatepass/suite.json"
    ))
    .expect("the committed gatepass suite parses");
    assert_eq!(
        suite.len(),
        SCENARIOS,
        "the criterion is the whole committed suite"
    );
    for id in UNKNOWN_INSTANCE {
        assert!(
            suite
                .scenarios
                .keys()
                .any(|scenario| scenario.to_string() == id),
            "the committed suite witnesses the unknown-instance rule as `{id}`"
        );
    }
    suite
}

/// Pins that the suite and a generated tree derive from the same resolved model.
fn assert_same_model(suite: &ConformanceSuite, plan: &str) {
    let plan: serde_json::Value = serde_json::from_str(plan).expect("the committed plan parses");
    assert_eq!(
        suite.provenance.spec_digest.as_str(),
        plan["provenance"]["source_digest"]
            .as_str()
            .expect("the plan names its source digest"),
        "the committed suite and the committed tree derive from different models; regenerate \
         whichever is stale"
    );
}

#[test]
fn the_committed_suite_passes_the_linked_rust_realization_including_unknown_instances() {
    let suite = committed_suite();
    assert_same_model(
        &suite,
        include_str!("../../../generated/rust/gatepass/plan.json"),
    );

    let report = Runner::for_suite(&suite)
        .run(
            &suite,
            &Synthesized {
                live: RefCell::new(None),
            },
        )
        .unwrap();

    let failures: Vec<String> = report
        .failures()
        .map(|result| {
            format!(
                "{} — {}: {}",
                result.scenario,
                result.status,
                result
                    .diagnostics()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })
        .collect();
    assert!(
        failures.is_empty(),
        "the Rust realization fails a scenario its own specification obliges:\n{}",
        failures.join("\n")
    );
    assert_eq!(report.scenarios.len(), SCENARIOS);
    assert_eq!(report.status, ConformanceStatus::Passed);
}

// ---- the Go target ---------------------------------------------------------------------------

/// Where Go is, or `None` when this machine has none — said out loud, never passed silently.
fn go() -> Option<PathBuf> {
    let output = Command::new("go").arg("version").output().ok()?;
    output.status.success().then(|| PathBuf::from("go"))
}

/// The emitted runner, the Go adapter beside it, and a module file naming both realizations.
///
/// Built under the temporary directory, never the source tree: the module file names absolute
/// paths that exist only on this machine.
fn go_module() -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("gatepass-go-conformance-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    for artifact in ess_conformance::go::emit(&committed_suite()).expect("the suite emits") {
        let path = directory.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("an artifact has a directory"))
            .expect("a scratch directory");
        std::fs::write(path, artifact.contents).expect("the artifact writes");
    }
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/go-target");
    for file in ["target.go", "target_test.go"] {
        std::fs::copy(fixture.join(file), directory.join(file)).expect("the fixture copies");
    }
    let root = root();
    std::fs::write(
        directory.join("go.mod"),
        format!(
            "module essgatepass\n\ngo 1.24\n\nrequire (\n\texample.invalid/gatepass v0.0.0\n\t\
             example.invalid/gatepass-realization v0.0.0\n)\n\nreplace example.invalid/gatepass \
             => {}\n\nreplace example.invalid/gatepass-realization => {}\n",
            root.join("generated/go/gatepass").display(),
            root.join("examples/gatepass-go-realization").display(),
        ),
    )
    .expect("the module file writes");
    directory
}

/// The scenario ids `go test -v` reported at `verdict`.
fn go_scenarios(printed: &str, verdict: &str) -> Vec<String> {
    let marker = format!("--- {verdict}: TestConformance/");
    let mut found: Vec<String> = printed
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&marker).map(ToOwned::to_owned))
        .map(|line| {
            line.split_once(' ')
                .map_or(line.clone(), |(id, _)| id.to_owned())
        })
        .collect();
    found.sort();
    found
}

#[test]
fn the_committed_suite_passes_the_linked_go_realization_including_unknown_instances() {
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go realization is unchecked here");
        return;
    };
    let suite = committed_suite();
    assert_same_model(
        &suite,
        include_str!("../../../generated/go/gatepass/plan.json"),
    );

    let directory = go_module();
    let output = Command::new(go)
        .args(["test", "-count=1", "-v", "./..."])
        .current_dir(&directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .expect("go test runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let failed = go_scenarios(&printed, "FAIL");
    assert!(
        output.status.success() && failed.is_empty(),
        "the Go realization fails {failed:?}:\n{printed}"
    );
    assert_eq!(
        go_scenarios(&printed, "PASS").len(),
        SCENARIOS,
        "every scenario must run, and a suite that skipped them all would also pass:\n{printed}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

// ---- the served surfaces ---------------------------------------------------------------------

/// A running server, killed when dropped so a failing assertion leaves no process behind.
struct Served(std::process::Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts `program` on an ephemeral port and returns it with the port its startup record names.
fn serve(program: &mut Command) -> (Served, u16) {
    use std::io::BufRead as _;
    let mut child = program
        .env("PORT", "0")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("the startup record is piped");
    let served = Served(child);
    let mut lines = std::io::BufReader::new(stdout).lines();
    let first = lines
        .next()
        .expect("the server writes its startup record")
        .expect("the startup record is text");
    let record: serde_json::Value = serde_json::from_str(&first).expect("the record is JSON");
    let port = record["runtime"]["port"]
        .as_u64()
        .and_then(|port| u16::try_from(port).ok())
        .expect("the startup record names the bound port");
    // Drain the rest of the record in the background so the server never blocks on its pipe.
    std::thread::spawn(move || lines.for_each(drop));
    (served, port)
}

/// One `POST` with a JSON body; the status and the parsed body of the answer.
fn post(port: u16, path: &str, body: &str) -> (u16, serde_json::Value) {
    use std::io::{Read as _, Write as _};
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("the server accepts");
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("the request writes");
    let mut answer = String::new();
    stream
        .read_to_string(&mut answer)
        .expect("the answer reads");
    let (head, payload) = answer
        .split_once("\r\n\r\n")
        .expect("the answer has a head and a body");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .expect("the answer has a status");
    let payload = serde_json::from_str(payload)
        .unwrap_or_else(|_| serde_json::Value::String(payload.to_owned()));
    (status, payload)
}

/// What both surfaces must answer for a visit nobody registered: the declared `wrong-state`
/// branch and its error, and no payload, because such a visit is in no state.
fn assert_unknown_visit_is_wrong_state(language: &str, port: u16) {
    let unknown = r#"{"visit_id":"00000000-0000-4000-8000-0000000000ff"}"#;
    let expected = serde_json::json!({
        "outcome": "wrong-state",
        "error": VISIT_STATE_CONFLICT,
    });
    for path in [
        "/visits/commands/sign-out-visitor",
        "/visits/commands/admit-visitor",
    ] {
        let body = if path.ends_with("admit-visitor") {
            r#"{"visit_id":"00000000-0000-4000-8000-0000000000ff","badge":{"serial":"s","printed_at":null,"signature":"AA=="}}"#
        } else {
            unknown
        };
        let (status, answer) = post(port, path, body);
        assert_eq!(
            (status, &answer),
            (409, &expected),
            "the {language} surface answers an unknown visit at `{path}` with the declared \
             `wrong-state` branch, not with an unmet obligation"
        );
    }
}

#[test]
fn an_unknown_visit_is_answered_wrong_state_on_the_rust_served_surface() {
    let (_served, port) = serve(&mut Command::new(env!("CARGO_BIN_EXE_gatepass-server")));
    assert_unknown_visit_is_wrong_state("Rust", port);
}

#[test]
fn an_unknown_visit_is_answered_wrong_state_on_the_go_served_surface() {
    let Some(go) = go() else {
        eprintln!("no Go toolchain on this machine; the Go surface is unchecked here");
        return;
    };
    let binary = std::env::temp_dir().join(format!("gatepass-go-server-{}", std::process::id()));
    let built = Command::new(&go)
        .args(["build", "-o"])
        .arg(&binary)
        .arg("./cmd/gatepass-server")
        .current_dir(root().join("examples/gatepass-go-realization"))
        .env("GOPROXY", "off")
        .output()
        .expect("go build runs");
    assert!(
        built.status.success(),
        "the Go realization builds: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    {
        let (_served, port) = serve(&mut Command::new(&binary));
        assert_unknown_visit_is_wrong_state("Go", port);
    }
    let _ = std::fs::remove_file(&binary);
}
