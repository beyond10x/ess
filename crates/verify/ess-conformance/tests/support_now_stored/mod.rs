//! The leases fixture's interpreter, told which of a scenario's commands is the one under test,
//! with its command clock wired healthy or faulty (`docs/design/expression-family-source22.md`, A3;
//! beyond10x/ess#244 part a, unit U5). Shared by the Rust, Go and TypeScript run controls.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use ess_compiler::ir::EssIr;
use ess_conformance::interpret::Interpreted;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;

use super::support_occurrence_clock::{instant, T0, T1};

/// The leases fixture: stored and related instants ordered against `now` in every unit.
pub const LEASES: &str = include_str!("../fixtures/now-stored-rows.yaml");

/// 2026-10-04T11:59:59.900Z: the runner's wall. Rounded up, every `now_offset` resolves against
/// 12:00:00Z, and the decision instant [`T1`] follows it by less than a second.
pub const WALL_MS: u64 = 1_791_115_199_900;

/// How much later each further read within one decision answers: two days, past every witness
/// synthesis writes a day and a second from the reference.
pub const REREAD_SECONDS: i64 = 2 * 86_400;

/// A provider answering [`T0`] while an arranging command decides, and [`T1`] while the command
/// under test decides — each further read within one decision [`REREAD_SECONDS`] later than the
/// one before. A [`Staged`] target tells it which.
#[derive(Debug, Default)]
pub struct Stage {
    state: Mutex<StageState>,
}

#[derive(Debug, Default)]
pub struct StageState {
    testing: bool,
    in_decision: i64,
    reads: usize,
    decisions: usize,
}

impl Stage {
    fn begin(&self, testing: bool) {
        let mut state = self.state.lock().unwrap();
        state.testing = testing;
        state.in_decision = 0;
        state.decisions += 1;
    }

    pub fn reads(&self) -> (usize, usize) {
        let state = self.state.lock().unwrap();
        (state.reads, state.decisions)
    }
}

impl CommandClock for Stage {
    fn read(&self) -> Option<DecisionInstant> {
        let mut state = self.state.lock().unwrap();
        state.reads += 1;
        let base = if state.testing {
            instant(T1)
        } else {
            instant(T0)
        };
        let later = state.in_decision;
        state.in_decision += 1;
        Some(DecisionInstant::from_instant(
            base.instant()
                .plus_seconds(REREAD_SECONDS * later)
                .expect("spelled"),
        ))
    }
}

/// How the interpreter's clock is wired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clocking {
    /// One reading per decision, from the stage.
    Healthy,
    /// The scenario's first reading answered for every later decision: setup time reused.
    SetupReused,
    /// Two readings per decision, the second deciding: a row reread two days later.
    Reread,
    /// No provider at all.
    Absent,
}

/// The first reading of a scenario, answered forever after until the next scenario.
pub struct ReuseFirst {
    pub stage: Arc<Stage>,
    first: Arc<Mutex<Option<DecisionInstant>>>,
}

impl CommandClock for ReuseFirst {
    fn read(&self) -> Option<DecisionInstant> {
        let mut first = self.first.lock().unwrap();
        if first.is_none() {
            *first = self.stage.read();
        }
        *first
    }
}

pub struct Twice(Arc<Stage>);

impl CommandClock for Twice {
    fn read(&self) -> Option<DecisionInstant> {
        let _ = self.0.read();
        self.0.read()
    }
}

/// The commands of the fixture that read a stored or related row against `now`: each decides at
/// the decision instant [`T1`], wherever a scenario sends it. Every other command arranges rows and
/// decides at the setup instant [`T0`].
pub const DECIDING: [&str; 3] = [
    "demo.leases.RenewLease",
    "demo.leases.Join",
    "demo.leases.Lend",
];

/// The interpreter, its command clock staged by the command it decides: [`T1`] for a command of
/// [`DECIDING`], [`T0`] for every arranging one.
pub struct Staged {
    inner: Interpreted,
    pub stage: Arc<Stage>,
    first: Arc<Mutex<Option<DecisionInstant>>>,
}

impl Staged {
    pub fn new(ir: EssIr, _suite: &ConformanceSuite, clocking: Clocking) -> Self {
        let stage = Arc::new(Stage::default());
        let first = Arc::new(Mutex::new(None));
        let inner = Interpreted::for_model(ir);
        let inner = match clocking {
            Clocking::Healthy => inner.with_command_clock(stage.clone()),
            Clocking::SetupReused => inner.with_command_clock(ReuseFirst {
                stage: stage.clone(),
                first: first.clone(),
            }),
            Clocking::Reread => inner.with_command_clock(Twice(stage.clone())),
            Clocking::Absent => inner,
        };
        Self {
            inner,
            stage,
            first,
        }
    }

    fn staged(&self, request: &SemanticCommandRequest) {
        self.stage
            .begin(DECIDING.contains(&request.command.to_string().as_str()));
    }
}

impl ConformanceTarget for Staged {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.first.lock().unwrap() = None;
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
        self.staged(&request);
        self.inner.execute_command(request)
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        self.staged(&request);
        self.inner.execute_command_recorded(request)
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

// ---- implementation faults, run against the synthesized suite -----------------------------------

/// An implementation fault in how a decision reads its rows (the A3 controls besides the clock
/// edge's): each is a decision the specification does not make, taken by an otherwise correct
/// target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// `Join` and `Lend` decide by the first member the scenario stored, not the one named.
    WrongRow,
    /// `RenewLease` decides by the expiry its own effect would write, not the one stored before it.
    PostEffect,
    /// Every instant is compared by its bytes with the decision's instant moved by the guard's
    /// offset, spelled in the target's zone, this many hours from UTC.
    Bytes(i64),
    /// The `now` predicates are read before the input and existence refusals declared ahead of
    /// them: a time refusal answers where an earlier refusal should.
    Precedence,
}

#[derive(Debug, Clone)]
struct LeaseRow {
    id: String,
    expires: String,
    grace: String,
    active: bool,
}

#[derive(Debug, Clone, Default)]
struct Rows {
    leases: Vec<LeaseRow>,
    members: Vec<(String, String)>,
    books: Vec<(String, String)>,
}

fn at(text: &str) -> Option<ess_primitives::time::Rfc3339Instant> {
    ess_primitives::time::Rfc3339Instant::parse_rfc3339(text)
}

fn text_of(node: Option<&ess_primitives::node::Node>) -> String {
    node.and_then(ess_primitives::node::Node::as_text)
        .unwrap_or_default()
        .to_owned()
}

/// `stored` against `t` moved by `seconds`: by instant, or — a [`Fault::Bytes`] target — by the
/// bytes of the two spellings, the bound spelled in the target's zone.
fn order(
    stored: &str,
    t: ess_primitives::time::Rfc3339Instant,
    seconds: i64,
    fault: Option<Fault>,
) -> Option<std::cmp::Ordering> {
    let bound = t.plus_seconds(seconds)?;
    let Some(Fault::Bytes(hours)) = fault else {
        return Some(at(stored)?.cmp(&bound));
    };
    let local = bound.plus_seconds(3600 * hours)?.to_rfc3339();
    let local = match hours {
        0 => local,
        _ => format!(
            "{}{}{:02}:00",
            local.strip_suffix('Z')?,
            if hours < 0 { '-' } else { '+' },
            hours.abs()
        ),
    };
    Some(stored.as_bytes().cmp(local.as_bytes()))
}

/// The member a `Join` or `Lend` decides by: the one named, or — [`Fault::WrongRow`] — the first
/// the scenario stored.
fn member<'r>(rows: &'r Rows, named: &str, fault: Option<Fault>) -> Option<&'r (String, String)> {
    if fault == Some(Fault::WrongRow) {
        rows.members.first()
    } else {
        rows.members.iter().find(|(id, _)| id == named)
    }
}

/// The decision the fixture's command makes over `rows` at instant `t`, with `fault` read in.
/// `None` where the model this mirrors cannot read a value it needs.
fn decide(
    rows: &Rows,
    request: &SemanticCommandRequest,
    t: ess_primitives::time::Rfc3339Instant,
    fault: Option<Fault>,
) -> Option<&'static str> {
    let input = |name: &str| text_of(request.input.get(name));
    match request.command.to_string().as_str() {
        "demo.leases.RenewLease" => renewal(rows, &input, t, fault),
        "demo.leases.Join" => {
            let Some((_, banned)) = member(rows, &input("member_id"), fault) else {
                return Some("no-member-to-join");
            };
            Some(if order(banned, t, 30, fault)?.is_gt() {
                "banned-from-joining"
            } else {
                "joined"
            })
        }
        "demo.leases.Lend" => lending(rows, request, &input, t, fault),
        _ => None,
    }
}

fn renewal(
    rows: &Rows,
    input: &dyn Fn(&str) -> String,
    t: ess_primitives::time::Rfc3339Instant,
    fault: Option<Fault>,
) -> Option<&'static str> {
    let row = rows.leases.iter().find(|row| row.id == input("lease_id"));
    let blank = input("note").is_empty();
    if blank && !(fault == Some(Fault::Precedence) && row.is_some()) {
        return Some("blank-note");
    }
    let row = row?;
    let expires = if fault == Some(Fault::PostEffect) {
        input("new_expires_at")
    } else {
        row.expires.clone()
    };
    let branch = if order(&expires, t, -3600, fault)?.is_ge() {
        "renewed"
    } else if order(&row.grace, t, 300, fault)?.is_gt() {
        "graced"
    } else if blank {
        "blank-note"
    } else {
        "lapsed"
    };
    Some(match branch {
        "renewed" | "graced" if !row.active => "not-active",
        other => other,
    })
}

fn lending(
    rows: &Rows,
    request: &SemanticCommandRequest,
    input: &dyn Fn(&str) -> String,
    t: ess_primitives::time::Rfc3339Instant,
    fault: Option<Fault>,
) -> Option<&'static str> {
    let Some((_, banned)) = member(rows, &input("member_id"), fault) else {
        return Some("no-member");
    };
    let book = rows.books.iter().find(|(id, _)| *id == input("book_id"));
    let copies = request
        .input
        .get("copies")
        .and_then(|node| match node {
            ess_primitives::node::Node::Number(number) => number.as_i64(),
            _ => None,
        })
        .unwrap_or(1);
    let refusal = if book.is_none() {
        Some("no-book")
    } else if copies < 1 {
        Some("too-few")
    } else {
        None
    };
    if let (Some(refusal), false) = (refusal, fault == Some(Fault::Precedence)) {
        return Some(refusal);
    }
    if order(banned, t, 30, fault)?.is_gt() {
        return Some("banned");
    }
    let Some((_, embargo)) = book else {
        return refusal;
    };
    if order(embargo, t, -120, fault)?.is_ge() {
        return Some("embargoed");
    }
    Some(refusal.unwrap_or("lent"))
}

/// The declared error of a refusal branch the faults can answer.
fn refusal_error(branch: &str) -> Option<&'static str> {
    Some(match branch {
        "blank-note" => "demo.leases.BlankNote",
        "lapsed" => "demo.leases.Lapsed",
        "not-active" => "demo.leases.NotActive",
        "unknown-lease" => "demo.leases.UnknownLease",
        "no-member-to-join" | "no-member" => "demo.leases.NoMember",
        "banned-from-joining" | "banned" => "demo.leases.Banned",
        "no-book" => "demo.leases.NoBook",
        "too-few" => "demo.leases.TooFew",
        "embargoed" => "demo.leases.Embargoed",
        _ => return None,
    })
}

/// A command clock answering the instant it was last steered to, `T0` until it is.
#[derive(Debug, Default)]
struct Steer(Mutex<Option<DecisionInstant>>);

impl CommandClock for Steer {
    fn read(&self) -> Option<DecisionInstant> {
        Some(self.0.lock().unwrap().unwrap_or_else(|| instant(T0)))
    }
}

/// The interpreter taking the decisions `fault` makes: each deciding command's branch is the one
/// the faulty reading of the tracked rows takes at the decision instant [`T1`], reached by handing
/// the interpreter an instant at which it takes that branch, by rewriting the input that answers
/// an earlier refusal, or — between two refusals, which have no effects — by answering the faulty
/// refusal in place of the healthy one. A healthy wiring (`None`) decides at `T1`, as [`Staged`].
pub struct Steered {
    inner: Interpreted,
    clock: Arc<Steer>,
    rows: Mutex<Rows>,
    fault: Option<Fault>,
}

impl Steered {
    pub fn new(ir: EssIr, fault: Option<Fault>) -> Self {
        let clock = Arc::new(Steer::default());
        Self {
            inner: Interpreted::for_model(ir).with_command_clock(clock.clone()),
            clock,
            rows: Mutex::new(Rows::default()),
            fault,
        }
    }

    /// The instant at which the healthy model takes `branch` over `rows`, trying the decision
    /// instant first.
    fn steer(
        rows: &Rows,
        request: &SemanticCommandRequest,
        branch: &str,
    ) -> Option<ess_primitives::time::Rfc3339Instant> {
        let t1 = instant(T1).instant();
        let mut candidates = vec![t1, instant(T0).instant()];
        candidates.extend(at("9000-01-01T00:00:00Z"));
        let row_instants: Vec<String> = rows
            .leases
            .iter()
            .flat_map(|row| [row.expires.clone(), row.grace.clone()])
            .chain(rows.members.iter().map(|(_, banned)| banned.clone()))
            .chain(rows.books.iter().map(|(_, embargo)| embargo.clone()))
            .collect();
        for stored in row_instants.iter().filter_map(|text| at(text)) {
            for seconds in [-3601, 3601, -301, 301, -31, 31, -121, 121] {
                candidates.extend(stored.plus_seconds(seconds));
            }
        }
        candidates
            .into_iter()
            .find(|t| decide(rows, request, *t, None) == Some(branch))
    }

    fn track(&self, request: &SemanticCommandRequest, result: &SemanticCommandResult) {
        let outcome = result
            .outcome
            .as_ref()
            .map(|taken| taken.outcome.to_string())
            .unwrap_or_default();
        let field = |name: &str| text_of(request.input.get(name));
        let published = |name: &str| {
            text_of(
                result
                    .direct_events
                    .first()
                    .and_then(|event| event.payload.get(name)),
            )
        };
        let mut rows = self.rows.lock().unwrap();
        match (request.command.to_string().as_str(), outcome.as_str()) {
            ("demo.leases.OpenLease", "opened") => rows.leases.push(LeaseRow {
                id: published("lease_id"),
                expires: field("expires_at"),
                grace: field("grace_until"),
                active: true,
            }),
            ("demo.leases.RegisterMember", "registered") => rows
                .members
                .push((published("member_id"), field("banned_until"))),
            ("demo.leases.AddBook", "added") => rows
                .books
                .push((published("book_id"), field("embargo_until"))),
            ("demo.leases.RenewLease", "renewed" | "graced") => {
                let renewed = outcome == "renewed";
                if let Some(row) = rows
                    .leases
                    .iter_mut()
                    .find(|row| row.id == field("lease_id"))
                {
                    row.active = false;
                    if renewed {
                        row.expires = field("new_expires_at");
                    }
                }
            }
            _ => {}
        }
    }

    fn command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let deciding = DECIDING.contains(&request.command.to_string().as_str());
        let rows = self.rows.lock().unwrap().clone();
        let t1 = instant(T1).instant();
        let healthy = deciding
            .then(|| decide(&rows, &request, t1, None))
            .flatten();
        let faulty = deciding
            .then(|| decide(&rows, &request, t1, self.fault))
            .flatten();
        let mut answer_as = None;
        let mut steer_to = Some(t1);
        if let (Some(healthy), Some(faulty)) = (healthy, faulty) {
            if healthy != faulty {
                if refusal_error(healthy).is_some() && refusal_error(faulty).is_some() {
                    answer_as = Some(faulty);
                } else {
                    if self.fault == Some(Fault::Precedence) {
                        // The earlier refusal the faulty target read past: the input it answers on.
                        request.input.insert(
                            "copies".to_owned(),
                            ess_primitives::node::Node::Number(
                                ess_primitives::facts::Number::from(1_i64),
                            ),
                        );
                        if text_of(request.input.get("note")).is_empty()
                            && request.command.to_string() == "demo.leases.RenewLease"
                        {
                            request.input.insert(
                                "note".to_owned(),
                                ess_primitives::node::Node::Text("read past".to_owned()),
                            );
                        }
                        if request.command.to_string() != "demo.leases.Lend" {
                            request.input.remove("copies");
                        }
                    }
                    if self.fault == Some(Fault::WrongRow) {
                        if let Some((first, _)) = rows.members.first() {
                            request.input.insert(
                                "member_id".to_owned(),
                                ess_primitives::node::Node::Text(first.clone()),
                            );
                        }
                    }
                    steer_to = Self::steer(&rows, &request, faulty).or(Some(t1));
                }
            }
        }
        *self.clock.0.lock().unwrap() = steer_to
            .filter(|_| deciding)
            .map(DecisionInstant::from_instant);
        let mut answer = self.inner.execute_command(request.clone())?;
        if let Some(branch) = answer_as {
            answer.outcome = Some(ess_conformance::scenario::OutcomeRef::new(
                request.command.clone(),
                ess_domain::command::OutcomeName::new(branch).expect("a declared branch"),
            ));
            answer.error = refusal_error(branch)
                .map(|error| DeclaredErrorValue::new(error.parse().expect("a declared error")));
        }
        self.track(&request, &answer);
        Ok(answer)
    }
}

impl ConformanceTarget for Steered {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.rows.lock().unwrap() = Rows::default();
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
        self.command(request)
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

/// Each implementation fault, and synthesized scenarios it fails (read off the run, then held).
pub const FAULT_KILLS: &[(Fault, &[&str])] = &[
    (
        Fault::WrongRow,
        &[
            "demo.leases.Join/outcome/joined",
            "demo.leases.Join/outcome/banned-from-joining",
            "demo.leases.Lend/outcome/banned",
            "demo.leases.Lend/outcome/lent",
        ],
    ),
    (
        Fault::PostEffect,
        &[
            "demo.leases.RenewLease/outcome/graced",
            "demo.leases.RenewLease/outcome/lapsed",
        ],
    ),
    (
        Fault::Bytes(-5),
        &[
            "demo.leases.RenewLease/outcome/lapsed",
            "demo.leases.RenewLease/outcome/graced",
            "demo.leases.Join/outcome/joined",
            "demo.leases.Lend/outcome/embargoed",
            "demo.leases.Lend/outcome/lent",
        ],
    ),
    (
        Fault::Precedence,
        &[
            "demo.leases.RenewLease/outcome/blank-note",
            "demo.leases.Lend/outcome/no-book",
            "demo.leases.Lend/outcome/too-few",
        ],
    ),
];

/// Byte comparisons no synthesized scenario tells apart: the target's clock spelled in UTC, or
/// east of it. A `now_offset` resolves to a whole second spelled in UTC, and a sound witness sits a
/// second inside its bound or a day and a second past the reference: its bytes order as its
/// instant does against a bound spelled in UTC with a fraction, or moved later by the zone. Only a
/// witness on the bound's own second, which a latency flips, or a `now_offset` sent in another
/// zone, which the suite format cannot carry, would.
pub const UNCAUGHT_BYTES: [Fault; 2] = [Fault::Bytes(0), Fault::Bytes(1)];
