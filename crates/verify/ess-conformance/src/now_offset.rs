//! The `now_offset` scenario value: an instant a suite names relative to the moment the runner
//! sends it (beyond10x/ess#171, `docs/design/current-time-guards.md`).
//!
//! A guard `starts_at < now - 60s` (source format `ess/16`) compares an input with the moment the
//! implementation handles the request. No suite can carry that instant: it does not exist until the
//! run. So synthesis decides its witnesses against a fixed [`reference()`] instant, and writes each
//! value it chose for a now-guarded input as `{kind: now_offset, seconds: <n>}` — the reference
//! offset, carried over to the moment of sending. The runner resolves it from its wall clock
//! ([`crate::Clock::wall`]) when the step that first names it runs, rounded up to a whole second,
//! and every later `now_offset` of the same number in the same scenario reads that same instant: the
//! row a view returns afterwards is required to hold exactly what was sent. The target is told
//! nothing new; it receives an RFC 3339 instant like any other.
//!
//! Witnesses sit a second from each boundary and never on it, on the side a latency cannot flip:
//! `now - 61s` requires the refusal, `now - 59s` the accepting branch. A target that handles the
//! request under a second after the runner resolved the value decides both as the suite requires.
//!
//! Such a suite is written in the round-3 pair, ordinary suite/[`ORDINARY`] and coverage
//! suite/[`COVERAGE`], which [`crate::leaf_payloads`] registers. An older reader does not know the
//! value kind and refuses the suite by version; the Go and TypeScript runtimes execute both majors
//! and resolve it (beyond10x/ess#188). A suite without one keeps its format and bytes.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedCondition};
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate};
use ess_primitives::time::{CurrentTime, Rfc3339Instant, Timestamp};

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation};

/// The first ordinary suite major that carries `now_offset` values: the round-3 pair.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

/// The largest offset a suite may carry, either way: twice the largest offset a guard may write,
/// which leaves room for the day-apart witnesses a further instance takes.
pub const MAX_SECONDS: i64 = 2 * CurrentTime::MAX_OFFSET_SECONDS;

const REQUIRES: &str = "now_offset values require suite/26 or /27";

/// The instant synthesis decides a now-guarded input against: `2019-12-30T23:59:59Z`.
///
/// A constant, so synthesis reads no clock. It is a day and a second before the first `Timestamp`
/// witness (`2020-01-01T00:00:00Z`), so an input no guard moves is sent `now + 86401s` — a
/// moment clear of every boundary written in whole minutes or hours — and a boundary a guard
/// writes is tried a second either side of the reference offset it names.
pub fn reference() -> Rfc3339Instant {
    Rfc3339Instant::parse_rfc3339("2019-12-30T23:59:59Z").expect("a constant instant")
}

/// The earliest moment a suite carrying `now_offset` values runs: `2026-09-27T00:00:00Z`, the day
/// the operand was implemented.
///
/// Synthesis decides every value once, at [`reference()`]. A fixed instant a now-guarded field is
/// also ordered against, lying after the reference and not after this moment, is on one side of
/// `now` at the reference and on the other at every run, so a value chosen at that fixed bound is
/// decided the other way by a correct service. Such a field is refused by name.
pub fn earliest_run() -> Rfc3339Instant {
    Rfc3339Instant::parse_rfc3339("2026-09-27T00:00:00Z").expect("a constant instant")
}

// ---- synthesis ------------------------------------------------------------------------------

/// One comparison of an input path with a literal instant: an ordering against the current-time
/// operand, or a comparison with a fixed RFC 3339 instant.
enum Bound {
    /// `now` moved by this many seconds.
    Now(i64),
    /// A fixed instant, and whether the path is ordered against it (`<`, `<=`, `>`, `>=`) rather
    /// than equated with it (`==`, `!=`).
    Fixed {
        instant: Rfc3339Instant,
        ordered: bool,
    },
}

/// What one command's input guards order against instants, by path.
#[derive(Default)]
struct Orderings {
    /// Every path ordered against an instant, with each instant it is ordered against.
    bounds: BTreeMap<FactPath, Vec<Bound>>,
    /// Paths under a quantifier binder ordered against the current time, which no `now_offset`
    /// can carry.
    quantified: Vec<FactPath>,
}

impl Orderings {
    /// The orderings `command`'s input guards write: the plain `when:` and the input half of every
    /// other condition, which `ess-domain` admits the operand in.
    fn of(command: &ResolvedCommand) -> Self {
        let mut found = Self::default();
        for outcome in &command.outcomes {
            if let Some(predicate) = input_predicate(&outcome.condition) {
                found.walk(predicate, &mut Vec::new());
            }
        }
        found
    }

    /// Syntactic: `ess-domain` admits the operand only against a `Timestamp` in a command's input
    /// guard, so an admitted IR carries it nowhere else.
    fn walk(&mut self, predicate: &Predicate, binders: &mut Vec<String>) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    self.walk(child, binders);
                }
            }
            Predicate::Not(inner) => self.walk(inner, binders),
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                binders.push(quantified.bind.clone());
                self.walk(&quantified.body, binders);
                binders.pop();
            }
            Predicate::Compare { left, op, right } => {
                let ordered = op.needs_ordering();
                for (fact, literal) in [(left, right), (right, left)] {
                    let (Operand::Fact(path), Operand::Literal(FactValue::Text(text))) =
                        (fact, literal)
                    else {
                        continue;
                    };
                    let bound = if let Some(now) = CurrentTime::parse(text).filter(|_| ordered) {
                        if binders.iter().any(|binder| binder == path.namespace()) {
                            self.quantified.push(path.clone());
                            continue;
                        }
                        Bound::Now(now.offset_seconds())
                    } else if let Some(instant) = Rfc3339Instant::parse_rfc3339(text) {
                        Bound::Fixed { instant, ordered }
                    } else {
                        continue;
                    };
                    if !binders.iter().any(|binder| binder == path.namespace()) {
                        self.bounds.entry(path.clone()).or_default().push(bound);
                    }
                }
            }
            _ => {}
        }
    }

    /// The paths ordered against the current time.
    fn now_paths(&self) -> impl Iterator<Item = &FactPath> {
        self.bounds
            .iter()
            .filter(|(_, bounds)| bounds.iter().any(|bound| matches!(bound, Bound::Now(_))))
            .map(|(path, _)| path)
    }

    /// The first path ordered against the current time that a `now_offset` cannot carry — one
    /// inside a structure, or one under a quantifier binder — as the refusal names it.
    fn uncarried(&self) -> Option<String> {
        self.quantified
            .first()
            .map(|path| (path, "an element a quantifier binds"))
            .or_else(|| {
                self.now_paths()
                    .find(|path| path.segments().len() > 1)
                    .map(|path| (path, "a Timestamp inside a structure"))
            })
            .map(|(path, what)| {
                format!(
                    "{path}: a guard orders {what} against the current time, and a now_offset \
                     can only be sent as a whole input field"
                )
            })
    }

    /// The first path ordered against the current time and also against a fixed instant after
    /// [`reference`] and not after [`earliest_run`], as the refusal names it: synthesis decides
    /// its values at the reference, where that instant lies on the other side of `now` than at
    /// any run.
    fn straddled(&self) -> Option<String> {
        let (from, to) = (reference(), earliest_run());
        self.now_paths().find_map(|path| {
            self.bounds[path].iter().find_map(|bound| match bound {
                Bound::Fixed {
                    instant,
                    ordered: true,
                } if from < *instant && *instant <= to => Some(format!(
                    "{path}: a guard orders it against `now` and another against the fixed \
                     instant {}, which lies after the synthesis reference {} and before a run, \
                     so no value is decided the same at both",
                    instant.to_rfc3339(),
                    from.to_rfc3339()
                )),
                _ => None,
            })
        })
    }

    /// Whether `value` for the top-level `field` was chosen from a boundary of the current time
    /// or from the plain witness, rather than from a fixed instant's boundary.
    ///
    /// A candidate a second either side of a `now` boundary is the current time's. One at, or a
    /// second either side of, a fixed instant the field is also ordered against is that instant's
    /// and stays one: `starts_at > '2030-01-01T00:00:00Z'` is refuted at the bound itself whenever
    /// the scenario runs. Anything else — the plain witness and a further instance's — is sent
    /// relative to the moment of sending, as every other value of a now-guarded field is.
    fn chosen_from_now(&self, field: &str, value: Rfc3339Instant) -> bool {
        let Ok(path) = FactPath::new(field) else {
            return false;
        };
        let Some(bounds) = self.bounds.get(&path) else {
            return false;
        };
        let near = |center: Rfc3339Instant| {
            [-1, 0, 1]
                .into_iter()
                .any(|step| center.plus_seconds(step) == Some(value))
        };
        let mut now = false;
        let mut fixed = false;
        for bound in bounds {
            match bound {
                Bound::Now(seconds) => {
                    now = true;
                    if let Some(boundary) = reference().plus_seconds(*seconds) {
                        if [-1, 1]
                            .into_iter()
                            .any(|step| boundary.plus_seconds(step) == Some(value))
                        {
                            return true;
                        }
                    }
                }
                Bound::Fixed { instant, .. } => fixed |= near(*instant),
            }
        }
        now && !fixed
    }
}

/// The input half of a condition, as `ess-compiler`'s predicate sites read it.
fn input_predicate(condition: &ResolvedCondition) -> Option<&Predicate> {
    match condition {
        ResolvedCondition::When { predicate }
        | ResolvedCondition::ExternalWhen { predicate, .. } => Some(predicate),
        ResolvedCondition::SubjectState { predicate, .. }
        | ResolvedCondition::StateChange { predicate, .. }
        | ResolvedCondition::SubjectField { predicate, .. } => predicate.as_ref(),
        ResolvedCondition::SubjectPredicate { input, .. }
        | ResolvedCondition::Related { input, .. } => input.as_ref(),
        ResolvedCondition::Otherwise
        | ResolvedCondition::External { .. }
        | ResolvedCondition::WrongState
        | ResolvedCondition::UnknownInstance
        | ResolvedCondition::InputAbsent
        | ResolvedCondition::ExistingInstance => None,
    }
}

/// The value a scenario sends for `field` of `command`: a `now_offset` where the command orders
/// the field against the current time and synthesis chose the instant from a `now` boundary or as
/// the plain witness, a whole number of seconds from [`reference`]; the literal otherwise,
/// including an instant chosen from a fixed instant's boundary.
pub(crate) fn sent(command: &ResolvedCommand, field: &str, value: &Node) -> ScenarioValue {
    let Node::Text(text) = value else {
        return ScenarioValue::literal(value.clone());
    };
    Rfc3339Instant::parse_rfc3339(text)
        .filter(|instant| Orderings::of(command).chosen_from_now(field, *instant))
        .and_then(|instant| instant.whole_seconds_since(reference()))
        .filter(|seconds| seconds.unsigned_abs() <= MAX_SECONDS.unsigned_abs())
        .map_or_else(
            || ScenarioValue::literal(value.clone()),
            |seconds| ScenarioValue::NowOffset { seconds },
        )
}

/// Writes every value a now-guarded input is sent as a `now_offset`, after the scenarios are
/// assembled, and refuses each scenario sending a command whose guard orders against the current
/// time a path a `now_offset` cannot carry — one inside a structure, or an element a quantifier
/// binds: a `now_offset` replaces a whole input field, and nothing carries one inside a literal —
/// or a field also ordered against a fixed instant between [`reference`] and [`earliest_run`].
/// Each refusal carries the reason the synthesis refusal states.
pub(crate) fn install(
    ir: &EssIr,
    suite: &mut ConformanceSuite,
) -> Vec<(ScenarioId, String, &'static str)> {
    let mut refused = Vec::new();
    for (id, scenario) in &mut suite.scenarios {
        for step in &mut scenario.steps {
            let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
                continue;
            };
            let Some(declared) = ir.commands().get(command.name()) else {
                continue;
            };
            let orderings = Orderings::of(declared);
            if let Some(path) = orderings.uncarried() {
                refused.push((
                    id.clone(),
                    path,
                    "a now_offset replaces a whole input field",
                ));
                break;
            }
            if let Some(path) = orderings.straddled() {
                refused.push((
                    id.clone(),
                    path,
                    "a fixed instant after the synthesis reference beside `now` on one field",
                ));
                break;
            }
            for (field, value) in input.iter_mut() {
                if let ScenarioValue::Literal { value: literal } = value {
                    *value = sent(declared, field, literal);
                }
            }
        }
    }
    refused
}

// ---- the suite format -----------------------------------------------------------------------

/// Whether one step carries a `now_offset` value anywhere it carries scenario values.
pub fn in_step(step: &ScenarioStep) -> bool {
    let mut found = false;
    visit(step, &mut |value| {
        found |= matches!(value, ScenarioValue::NowOffset { .. });
    });
    found
}

/// Whether any step of the suite carries a `now_offset` value.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .any(|scenario| scenario.steps.iter().any(in_step))
}

/// Refuse an explicitly pinned older format carrying a `now_offset`, before serialization or
/// target effects.
pub(crate) fn admit_format(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}

// ---- the runner -----------------------------------------------------------------------------

/// A runner clock with a wall clock of the caller's: `clock` measures budgets and durations, and
/// `wall` is read wherever a `now_offset` is resolved ([`crate::Clock::wall`]).
///
/// This crate reads no clock of the machine's, so a generated suite and a run stay functions of
/// what was built and run. A caller running a suite with `now_offset` values against an
/// implementation that reads the machine's clock supplies that clock here, such as
/// `WithWall::new(AdvancingClock::default(), || /* SystemTime::now() in epoch ms */)`.
pub struct WithWall<C, F> {
    clock: C,
    wall: F,
}

impl<C: crate::Clock, F: FnMut() -> Timestamp> WithWall<C, F> {
    /// `clock`, with `wall` as its wall clock.
    pub fn new(clock: C, wall: F) -> Self {
        Self { clock, wall }
    }
}

impl<C: crate::Clock, F: FnMut() -> Timestamp> crate::Clock for WithWall<C, F> {
    fn now(&mut self) -> Timestamp {
        self.clock.now()
    }

    fn wall(&mut self) -> Timestamp {
        (self.wall)()
    }
}

/// The instant each `now_offset` of one scenario resolved to, by its number of seconds.
#[derive(Debug, Clone, Default)]
pub(crate) struct Resolved {
    fixed: BTreeMap<i64, Rfc3339Instant>,
}

impl Resolved {
    /// Fixes every `now_offset` `step` names that no earlier step of the scenario fixed, against
    /// one reading of the wall clock, rounded up to a whole second: a moment that has not yet
    /// passed when the target reads its own clock, so a latency under a second cannot move a
    /// witness across the boundary it sits a second from.
    pub(crate) fn fix(&mut self, step: &ScenarioStep, wall: impl FnOnce() -> Timestamp) {
        let mut pending = Vec::new();
        visit(step, &mut |value| {
            if let ScenarioValue::NowOffset { seconds } = value {
                if !self.fixed.contains_key(seconds) && !pending.contains(seconds) {
                    pending.push(*seconds);
                }
            }
        });
        if pending.is_empty() {
            return;
        }
        let millis = i64::try_from(wall().epoch_millis()).unwrap_or(i64::MAX);
        let Some(now) = Rfc3339Instant::from_epoch_millis(millis) else {
            return;
        };
        let now = now.ceil_to_second();
        for seconds in pending {
            if let Some(instant) = now.plus_seconds(seconds) {
                self.fixed.insert(seconds, instant);
            }
        }
    }

    /// The instant a `now_offset` of `seconds` resolved to in this scenario, as the RFC 3339 text
    /// a `Timestamp` travels as.
    pub(crate) fn get(&self, seconds: i64) -> Result<Node, String> {
        self.fixed
            .get(&seconds)
            .map(|instant| Node::Text(instant.to_rfc3339()))
            .ok_or_else(|| {
                format!("now_offset {seconds} names no instant the runner's wall clock can spell")
            })
    }
}

/// Every scenario value `step` carries, in the places [`in_step`] reads.
pub(crate) fn visit(step: &ScenarioStep, each: &mut dyn FnMut(&ScenarioValue)) {
    let mut map = |values: &BTreeMap<String, ScenarioValue>| values.values().for_each(&mut *each);
    match step {
        ScenarioStep::ExecuteCommand { input, .. }
        | ScenarioStep::ExpectInvocation { input, .. } => {
            map(input);
        }
        ScenarioStep::ExpectEventValues { payload, .. } => map(payload),
        ScenarioStep::ExpectEveryInvocation {
            selecting, input, ..
        } => {
            map(selecting);
            map(input);
        }
        ScenarioStep::SnapshotCompleteSubject { subject, .. }
        | ScenarioStep::SnapshotSubject { subject, .. }
        | ScenarioStep::ExpectSubjectAbsent { subject, .. } => map(subject),
        ScenarioStep::QueryView { params, .. }
        | ScenarioStep::ExpectHalt { params, .. }
        | ScenarioStep::EventuallyHalt { params, .. } => map(params),
        ScenarioStep::EventuallyView {
            params,
            expectation,
            ..
        } => {
            map(params);
            expectation_values(expectation, &mut map);
        }
        ScenarioStep::ExpectView { expectation, .. } => expectation_values(expectation, &mut map),
        _ => {}
    }
}

fn expectation_values(
    expectation: &ViewExpectation,
    map: &mut dyn FnMut(&BTreeMap<String, ScenarioValue>),
) {
    match expectation {
        ViewExpectation::Contains { fields }
        | ViewExpectation::Excludes { fields }
        | ViewExpectation::At { fields, .. } => map(fields),
        _ => {}
    }
}
