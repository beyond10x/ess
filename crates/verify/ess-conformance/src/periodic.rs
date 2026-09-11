//! Bounded periodic observations: elapsed target time plus explicit causal occurrences.
use crate::scenario::{
    BindingAspect, BindingRef, CommandRef, ConformanceScenario, ConformanceSuite, Elapsed,
    InstantName, ScenarioId, ScenarioPurpose, ScenarioStep,
};
use crate::target::{ConformanceTarget, ElapsedObservationRequest, InstantMark, TargetError};
use ess_compiler::ir::{EssIr, ResolvedBinding};
use ess_domain::binding::periodic::PeriodicCause;
use ess_primitives::{ids::CorrelationId, node::Node};
use std::collections::{BTreeMap, BTreeSet};

/// A required host fixture exercises the actual adapter with controlled inputs and time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fixture {
    /// Eligible, successful reads at each idle deadline.
    Ready,
    /// The first received tick is ineligible; subsequent ticks are ready.
    InitiallyInactive,
    /// The first read fails; subsequent reads succeed without retrying that ordinal.
    FirstReadFails,
    /// First operation stays busy through the next two deadlines; excess ticks coalesce.
    SlowFirstRead,
}

/// A standalone suite's required host contract, not credentials or a generated substitute host.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    /// The binding whose cause is exercised.
    pub binding: BindingRef,
    /// Its declared invoked command.
    pub command: CommandRef,
    /// The complete profile and typed authority contract.
    pub periodic: PeriodicCause,
    /// Mapping from command inputs to host field sources; checked before effects.
    pub mapping: BTreeMap<String, HostSource>,
    /// Controlled fixture required from that authority.
    pub fixture: Fixture,
}
/// An identity mapping from one explicit host phase, retaining the declared type in the contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HostSource {
    /// Lifetime-constant input.
    Context {
        /// Declared host-context field name.
        field: String,
    },
    /// Fresh read input.
    Read {
        /// Declared fresh-read field name.
        field: String,
    },
}
/// Runtime scope. The opaque lifetime is returned by the host after authority admission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    /// Runner-minted scenario activity.
    pub correlation: CorrelationId,
    /// The binding whose timer is observed.
    pub binding: BindingRef,
    /// Opaque session lifetime, never a credential.
    pub lifetime: String,
}
/// Activation request. The target must bind and validate the named typed authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Open {
    /// The required source contract and controlled fixture.
    pub check: Check,
    /// Earlier elapsed mark; activation reports its offset on this clock.
    pub mark: InstantMark,
}
/// Activation acknowledgement and immutable typed context.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Opened {
    /// Actual scope after activation.
    pub scope: Scope,
    /// Actual activation offset on the marked target clock.
    pub anchor_ms: u64,
    /// Lifetime-constant context fields; credentials stay private to the host.
    pub context: BTreeMap<String, Node>,
}
/// A complete held window on the same clock as the activation mark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observe {
    /// Exact capture scope.
    pub scope: Scope,
    /// Existing target elapsed hold semantics; no event is fabricated.
    pub elapsed: ElapsedObservationRequest,
    /// Exclusive cursor of the previously returned ledger.
    pub after: u64,
}
/// One append-only fact, not a target verdict.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Fact {
    /// The loop received a due tick and sampled eligibility.
    Received {
        /// Nominal tick ordinal, beginning at one.
        ordinal: u64,
        /// Nominal fixed-grid deadline on the marked clock.
        due_ms: u64,
        /// Actual observation time on the marked clock.
        at_ms: u64,
        /// Host eligibility sampled at receipt.
        eligible: bool,
    },
    /// The required fresh read could not produce command inputs.
    ReadFailed {
        /// Nominal tick ordinal, beginning at one.
        ordinal: u64,
        /// Actual observation time on the marked clock.
        at_ms: u64,
    },
    /// The actual command invocation, traced from that received occurrence.
    Invoked {
        /// Nominal tick ordinal, beginning at one.
        ordinal: u64,
        /// Actual observation time on the marked clock.
        at_ms: u64,
        /// Unique actual command invocation identity.
        invocation: String,
        /// Actual command invoked.
        command: CommandRef,
        /// Fresh values returned for this occurrence.
        read: BTreeMap<String, Node>,
        /// Actual inputs delivered to the command.
        input: BTreeMap<String, Node>,
    },
    /// All read/apply work for the received occurrence completed.
    Completed {
        /// Nominal tick ordinal, beginning at one.
        ordinal: u64,
        /// Actual observation time on the marked clock.
        at_ms: u64,
    },
    /// Nominal due ticks discarded while the receiver was busy.
    Dropped {
        /// First nominal discarded ordinal.
        first: u64,
        /// Last nominal discarded ordinal, inclusive.
        last: u64,
        /// Actual observation time on the marked clock.
        at_ms: u64,
    },
    /// An independent cause invoked the same command; cannot satisfy periodic liveness.
    Independent {
        /// Actual observation time on the marked clock.
        at_ms: u64,
        /// Unique actual command invocation identity.
        invocation: String,
        /// Actual command invoked.
        command: CommandRef,
    },
}
impl Fact {
    fn at(&self) -> u64 {
        match self {
            Self::Received { at_ms, .. }
            | Self::ReadFailed { at_ms, .. }
            | Self::Invoked { at_ms, .. }
            | Self::Completed { at_ms, .. }
            | Self::Dropped { at_ms, .. }
            | Self::Independent { at_ms, .. } => *at_ms,
        }
    }
}
/// Every fact retains its actual cause scope, even when it does not match the requested one.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// Actual observed origin.
    pub scope: Scope,
    /// Actual timer/read/invocation fact.
    pub fact: Fact,
}
/// Time and complete causal capture, returned together after settling the held window.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Reported elapsed time in the target's clock.
    pub elapsed_ms: u64,
    /// No scoped invocation or timer fact through this time was omitted.
    pub complete_through_ms: u64,
    /// Exclusive cursor after this batch.
    pub cursor: u64,
    /// Newly captured facts, bounded to 4096 across the check.
    pub records: Vec<Record>,
}
/// Stop means the actual loop and its in-flight work quiesced, not just cancellation requested.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Closed {
    /// Scope that has actually stopped.
    pub scope: Scope,
    /// Acknowledged quiescence on the marked target clock.
    pub at_ms: u64,
}

impl Check {
    /// Validate standalone contracts before opening an authenticated host instance.
    pub fn validate(&self) -> Result<(), String> {
        let errors = self.periodic.validate("periodic");
        if !errors.is_empty() {
            return Err(errors.to_string());
        }
        if self.periodic.every.seconds().checked_mul(5).is_none() {
            return Err(
                "PeriodicResource: five-period witness exceeds elapsed representation".into(),
            );
        }
        if self.mapping.is_empty() || self.mapping.len() > 64 {
            return Err("PeriodicResource: require 1..64 mapped host inputs".into());
        }
        for (target, source) in &self.mapping {
            if !target
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
                || !target
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                return Err("PeriodicContract: empty input name".into());
            }
            let (name, fields) = match source {
                HostSource::Context { field } => (field, &self.periodic.host.context_fields),
                HostSource::Read { field } => (field, &self.periodic.host.read_fields),
            };
            if !fields.iter().any(|entry| entry.name == *name) {
                return Err(format!("PeriodicContract: undeclared host field {name}"));
            }
        }
        Ok(())
    }
}

#[derive(Default, PartialEq, Eq)]
enum Attempt {
    #[default]
    Pending,
    Invoked,
    Failed,
}
#[derive(Default)]
struct Operation {
    eligible: bool,
    received: u64,
    attempt: Attempt,
    completed: bool,
}
/// Stateful ledger checker; the runner owns this, never the target.
struct Ledger<'a> {
    check: &'a Check,
    opened: &'a Opened,
    operations: BTreeMap<u64, Operation>,
    invocations: BTreeSet<String>,
    dropped: BTreeSet<u64>,
    cursor: u64,
    through: u64,
    last_at: u64,
}
impl<'a> Ledger<'a> {
    fn new(check: &'a Check, opened: &'a Opened) -> Self {
        Self {
            check,
            opened,
            operations: BTreeMap::new(),
            invocations: BTreeSet::new(),
            dropped: BTreeSet::new(),
            cursor: 0,
            through: opened.anchor_ms,
            last_at: opened.anchor_ms,
        }
    }
    fn observe(
        &mut self,
        observation: Observation,
        hold_ms: u64,
        closed: Option<u64>,
    ) -> Result<(), String> {
        if observation.records.len() > 4096
            || observation.cursor > 4096
            || observation.cursor
                != self
                    .cursor
                    .checked_add(observation.records.len() as u64)
                    .ok_or("PeriodicResource: cursor overflow")?
        {
            return Err("PeriodicResource: invalid or oversized capture cursor".into());
        }
        if observation.elapsed_ms < hold_ms
            || observation.complete_through_ms < hold_ms
            || observation.complete_through_ms > observation.elapsed_ms
            || observation.complete_through_ms < self.through
        {
            return Err("PeriodicObservation: held window is not completely observed".into());
        }
        for record in observation.records {
            if record.scope != self.opened.scope {
                return Err("PeriodicOrigin: wrong binding, scenario or lifetime".into());
            }
            let at = record.fact.at();
            if at < self.last_at || at <= self.through || at > observation.complete_through_ms {
                return Err("PeriodicObservation: invalid fact time/order".into());
            }
            if closed.is_some_and(|stop| at >= stop)
                && !matches!(record.fact, Fact::Independent { .. })
            {
                return Err("PeriodicLifetime: occurrence after acknowledged stop".into());
            }
            self.last_at = at;
            match record.fact {
                Fact::Received {
                    ordinal,
                    due_ms,
                    at_ms,
                    eligible,
                } => self.received(ordinal, due_ms, at_ms, eligible)?,
                Fact::ReadFailed { ordinal, .. } => self.read_failed(ordinal)?,
                Fact::Invoked {
                    ordinal,
                    at_ms,
                    invocation,
                    command,
                    read,
                    input,
                } => self.invoked(ordinal, at_ms, invocation, &command, &read, &input)?,
                Fact::Completed { ordinal, at_ms } => self.completed(ordinal, at_ms)?,
                Fact::Dropped { first, last, .. } => self.dropped(first, last, at)?,
                Fact::Independent {
                    invocation,
                    command,
                    ..
                } => {
                    if invocation.is_empty()
                        || command != self.check.command
                        || !self.invocations.insert(invocation)
                    {
                        return Err("PeriodicOrigin: malformed independent invocation".into());
                    }
                }
            }
        }
        self.cursor = observation.cursor;
        self.through = observation.complete_through_ms;
        self.require_liveness(closed.map_or(self.through, |stop| stop.saturating_sub(1)))?;
        Ok(())
    }
    fn received(
        &mut self,
        ordinal: u64,
        due_ms: u64,
        at_ms: u64,
        eligible: bool,
    ) -> Result<(), String> {
        if self
            .check
            .periodic
            .every
            .deadline(self.opened.anchor_ms, ordinal)
            != Some(due_ms)
            || at_ms < due_ms
            || self.operations.contains_key(&ordinal)
            || self.dropped.contains(&ordinal)
        {
            return Err("PeriodicOccurrence: early, duplicate or incorrect nominal tick".into());
        }
        if self.operations.values().any(|op| !op.completed) {
            return Err("PeriodicOverlap: concurrent poll work".into());
        }
        let expected_eligible = !(self.check.fixture == Fixture::InitiallyInactive && ordinal == 1);
        if eligible != expected_eligible {
            return Err("PeriodicHost: eligibility contradicts controlled fixture".into());
        }
        self.operations.insert(
            ordinal,
            Operation {
                eligible,
                received: at_ms,
                ..Operation::default()
            },
        );

        Ok(())
    }
    fn read_failed(&mut self, ordinal: u64) -> Result<(), String> {
        let op = self
            .operations
            .get_mut(&ordinal)
            .ok_or("PeriodicOccurrence: failure without receipt")?;
        if !op.eligible
            || op.attempt != Attempt::Pending
            || op.completed
            || self.check.fixture != Fixture::FirstReadFails
            || ordinal != 1
        {
            return Err("PeriodicHost: unexpected required-read failure".into());
        }
        op.attempt = Attempt::Failed;

        Ok(())
    }
    fn invoked(
        &mut self,
        ordinal: u64,
        at_ms: u64,
        invocation: String,
        command: &CommandRef,
        read: &BTreeMap<String, Node>,
        input: &BTreeMap<String, Node>,
    ) -> Result<(), String> {
        let op = self
            .operations
            .get_mut(&ordinal)
            .ok_or("PeriodicOccurrence: invocation without receipt")?;
        if !op.eligible
            || op.attempt != Attempt::Pending
            || op.completed
            || command != &self.check.command
            || invocation.is_empty()
            || !self.invocations.insert(invocation)
            || at_ms < op.received
        {
            return Err(
                "PeriodicInvocation: duplicated, misattributed or forbidden invocation".into(),
            );
        }
        validate_fields(&self.check.periodic.host.read_fields, read)?;
        let mut expected = BTreeMap::new();
        for (target, source) in &self.check.mapping {
            let value = match source {
                HostSource::Context { field } => self.opened.context.get(field),
                HostSource::Read { field } => read.get(field),
            }
            .ok_or("PeriodicHost: missing required mapped value")?;
            expected.insert(target.clone(), value.clone());
        }
        if input != &expected {
            return Err(
                "PeriodicMapping: command inputs differ from this occurrence's host values".into(),
            );
        }
        op.attempt = Attempt::Invoked;

        Ok(())
    }
    fn completed(&mut self, ordinal: u64, at_ms: u64) -> Result<(), String> {
        if self.check.fixture == Fixture::SlowFirstRead
            && ordinal == 1
            && self
                .check
                .periodic
                .every
                .deadline(self.opened.anchor_ms, 3)
                .is_none_or(|deadline| at_ms <= deadline)
        {
            return Err(
                "PeriodicHost: first read did not remain busy through both missed deadlines".into(),
            );
        }
        let op = self
            .operations
            .get_mut(&ordinal)
            .ok_or("PeriodicOccurrence: completion without receipt")?;
        if op.completed
            || (self.check.fixture == Fixture::FirstReadFails
                && ordinal == 1
                && op.attempt != Attempt::Failed)
            || (op.eligible && op.attempt == Attempt::Pending)
        {
            return Err("PeriodicOccurrence: completion without required attempt".into());
        }
        op.completed = true;

        Ok(())
    }
    fn dropped(&mut self, first: u64, last: u64, at: u64) -> Result<(), String> {
        if first == 0
            || first > last
            || last > 5
            || self.check.fixture != Fixture::SlowFirstRead
            || !self.operations.values().any(|op| !op.completed)
        {
            return Err("PeriodicMissed: loss outside bounded busy interval".into());
        }
        for ordinal in first..=last {
            if self.operations.contains_key(&ordinal) || !self.dropped.insert(ordinal) {
                return Err("PeriodicMissed: duplicate dropped ordinal".into());
            }
            if self
                .check
                .periodic
                .every
                .deadline(self.opened.anchor_ms, ordinal)
                .is_none_or(|due| due > at)
            {
                return Err("PeriodicMissed: dropping a future deadline".into());
            }
        }

        Ok(())
    }
    fn require_liveness(&self, hold_ms: u64) -> Result<(), String> {
        let due = (hold_ms.saturating_sub(self.opened.anchor_ms))
            / self.check.periodic.every.milliseconds();
        if due > 5 {
            return Err("PeriodicResource: live observation exceeds five-period witness".into());
        }
        if self.check.fixture == Fixture::SlowFirstRead && due >= 4 {
            let pending = [2, 3]
                .iter()
                .filter(|n| self.operations.get(n).is_some_and(|op| op.completed))
                .count();
            let dropped = [2, 3].iter().filter(|n| self.dropped.contains(n)).count();
            if pending != 1 || dropped != 1 {
                return Err(
                    "PeriodicMissed: exactly one pending tick and one discarded tick required"
                        .into(),
                );
            }
        }
        for ordinal in 1..=due {
            if self.check.fixture == Fixture::SlowFirstRead && (2..=3).contains(&ordinal) {
                continue;
            }
            let op = self
                .operations
                .get(&ordinal)
                .ok_or("PeriodicLiveness: missing eligible idle occurrence")?;
            let deadline = self
                .check
                .periodic
                .every
                .deadline(self.opened.anchor_ms, ordinal)
                .ok_or("PeriodicResource: deadline overflow")?;
            if op.received != deadline {
                return Err(
                    "PeriodicLateness: idle occurrence dispatched at a later controlled time"
                        .into(),
                );
            }
            if !(op.completed
                || self.check.fixture == Fixture::SlowFirstRead && ordinal == 1 && due < 4)
            {
                return Err("PeriodicLiveness: ready occurrence did not complete".into());
            }
        }

        Ok(())
    }
}

fn accepts(ty: &ess_domain::TypeRef, value: &Node, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    match ty {
        ess_domain::TypeRef::Primitive(kind) => {
            crate::input::primitive_value(*kind, value).is_some()
        }
        ess_domain::TypeRef::Optional(of) => {
            matches!(value, Node::Null) || accepts(of, value, depth + 1)
        }
        ess_domain::TypeRef::List(_) => matches!(value, Node::Seq(_)),
        ess_domain::TypeRef::Map(_, _) => matches!(value, Node::Map(_)),
        ess_domain::TypeRef::Named(_) => true, // The authoritative target re-admits named representations.
    }
}

// Standalone structural admission is conservative. The required host additionally re-admits named
// type identities, nested values and invariants against its authoritative contract before opening.
fn validate_fields(
    fields: &[ess_domain::Field],
    values: &BTreeMap<String, Node>,
) -> Result<(), String> {
    if fields.len() != values.len() {
        return Err("PeriodicHost: exact declared fields required".into());
    }
    for field in fields {
        let value = values
            .get(&field.name)
            .ok_or("PeriodicHost: missing declared field")?;
        if !accepts(&field.type_ref, value, 0) {
            return Err(format!(
                "PeriodicHost: field {} violates declared representation",
                field.name
            ));
        }
    }
    Ok(())
}

/// A measured contract violation differs from a target that cannot perform the observation.
#[derive(Debug)]
pub enum Error {
    /// Unsupported or unavailable target authority/observation.
    Target(TargetError),
    /// Observed facts contradict the required periodic contract.
    Violation(String),
}
impl From<TargetError> for Error {
    fn from(error: TargetError) -> Self {
        Self::Target(error)
    }
}

/// Exercise the real target's authority, elapsed windows and causal ledger; no runner sleep.
pub fn execute<T: ConformanceTarget>(
    check: &Check,
    correlation: CorrelationId,
    target: &T,
) -> Result<(), Error> {
    let malformed = Error::Violation;
    check.validate().map_err(malformed)?;
    let instant = InstantName::new("periodic-anchor").expect("fixed instant name");
    let mark = InstantMark {
        instant: instant.clone(),
        correlation: correlation.clone(),
    };
    target.mark_instant(mark.clone())?;
    let opened = target.open_periodic(Open {
        check: check.clone(),
        mark,
    })?;
    let work = (|| {
        if opened.scope.correlation != correlation
            || opened.scope.binding != check.binding
            || opened.scope.lifetime.is_empty()
            || opened.scope.lifetime.len() > 128
        {
            return Err(malformed("PeriodicOrigin: invalid opened scope".into()));
        }
        validate_fields(&check.periodic.host.context_fields, &opened.context).map_err(malformed)?;
        let mut ledger = Ledger::new(check, &opened);
        let p = check.periodic.every.seconds();
        // Actual target activation must be representable without inventing rounded hold evidence.
        if opened.anchor_ms % 1000 != 0 {
            return Err(TargetError::unsupported(
                "periodic controlled time",
                "activation must align with the target's whole-second controlled mark",
            )
            .into());
        }
        let anchor = u32::try_from(opened.anchor_ms / 1000)
            .map_err(|_| malformed("PeriodicResource: anchor overflow".into()))?;
        for offset in [
            p - 1,
            p,
            p.checked_mul(2)
                .ok_or_else(|| malformed("PeriodicResource: window overflow".into()))?,
            p.checked_mul(4)
                .ok_or_else(|| malformed("PeriodicResource: window overflow".into()))?,
        ] {
            let seconds = anchor
                .checked_add(offset)
                .ok_or_else(|| malformed("PeriodicResource: held window overflow".into()))?;
            let observation = target.observe_periodic(Observe {
                scope: opened.scope.clone(),
                elapsed: ElapsedObservationRequest {
                    instant: instant.clone(),
                    hold: Elapsed::seconds(seconds),
                    watching: None,
                    correlation: correlation.clone(),
                },
                after: ledger.cursor,
            })?;
            ledger
                .observe(observation, u64::from(seconds) * 1000, None)
                .map_err(malformed)?;
        }
        Ok(ledger)
    })();
    // Closing is attempted even when an observation failed, so a rejected check leaks no poll.
    let closed = target.close_periodic(opened.scope.clone());
    let mut ledger = work?;
    let closed = closed?;
    if closed.scope != opened.scope || closed.at_ms < ledger.through {
        return Err(malformed(
            "PeriodicLifetime: invalid stop acknowledgement".into(),
        ));
    }
    let hold = closed
        .at_ms
        .checked_add(check.periodic.every.milliseconds())
        .ok_or_else(|| malformed("PeriodicResource: closure window overflow".into()))?;
    let seconds = u32::try_from(hold.div_ceil(1000))
        .map_err(|_| malformed("PeriodicResource: closure elapsed overflow".into()))?;
    let observed = target.observe_periodic(Observe {
        scope: opened.scope.clone(),
        elapsed: ElapsedObservationRequest {
            instant,
            hold: Elapsed::seconds(seconds),
            watching: None,
            correlation,
        },
        after: ledger.cursor,
    })?;
    ledger
        .observe(observed, u64::from(seconds) * 1000, Some(closed.at_ms))
        .map_err(malformed)
}

/// Periodic vocabulary present in a suite; central fresh-format selection consumes this signal.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::CheckPeriodic { .. }))
    })
}

/// The four bounded fixture cases are each a separate existing binding aspect identity.
pub(crate) fn synthesize(
    ir: &EssIr,
    binding: &ResolvedBinding,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<crate::synthesize::Refusal>,
) {
    let Some(periodic) = binding.cause.periodic() else {
        return;
    };
    let subject = BindingRef::new(binding.name.clone());
    let command = CommandRef::from(&binding.command);
    let mut mapping = BTreeMap::new();
    for entry in &binding.mapping {
        let source = match &entry.value {
            ess_compiler::ir::ResolvedMappingValue::HostContext { field, .. }
                if entry.conversion.is_none() =>
            {
                HostSource::Context {
                    field: field.clone(),
                }
            }
            ess_compiler::ir::ResolvedMappingValue::HostRead { field, .. }
                if entry.conversion.is_none() =>
            {
                HostSource::Read {
                    field: field.clone(),
                }
            }
            _ => {
                refusals.push(crate::synthesize::Refusal { subject: subject.clone().into(), scenario: None, cause: crate::synthesize::RefusalCause::NoWitness(crate::witness::WitnessGap { path: format!("binding.{}.mapping.{}", binding.name, entry.target), type_ref: entry.target_type.to_string(), reason: "PeriodicHostMapping: executable observations require identity host-context/read mappings" }) });
                return;
            }
        };
        mapping.insert(entry.target.clone(), source);
    }
    let mut source = BTreeSet::from([
        subject.clone().into(),
        command.clone().into(),
        crate::scenario::ComponentRef::new(periodic.contract.host.owner.clone()).into(),
    ]);
    let mut types = BTreeSet::new();
    for field in periodic
        .context
        .iter()
        .chain(&periodic.read)
        .chain(&ir.command(&binding.command).input)
    {
        crate::synthesize::reachable_types(ir, &field.type_ref, &mut types);
    }
    source.extend(types.into_iter().map(crate::scenario::EssSemanticRef::from));
    for (aspect, fixture) in [
        (BindingAspect::Flow, Fixture::Ready),
        (BindingAspect::Mapping, Fixture::InitiallyInactive),
        (BindingAspect::Delivery, Fixture::SlowFirstRead),
        (BindingAspect::OnFailure, Fixture::FirstReadFails),
    ] {
        let check = Check {
            binding: subject.clone(),
            command: command.clone(),
            periodic: periodic.contract.clone(),
            mapping: mapping.clone(),
            fixture,
        };
        if check.validate().is_err() {
            refuse_window(&subject, &check, refusals);
            return;
        }
        let id = ScenarioId::Binding {
            binding: subject.clone(),
            aspect,
        };
        crate::synthesize::insert(
            suite,
            id,
            ConformanceScenario::new(
                ScenarioPurpose::new(format!(
                    "Periodic {} through required host: {fixture:?}",
                    periodic.contract.every
                ))
                .expect("bounded purpose"),
                [ScenarioStep::CheckPeriodic { check }],
                source.clone(),
            ),
            refusals,
        );
    }
}

fn refuse_window(
    subject: &BindingRef,
    check: &Check,
    refusals: &mut Vec<crate::synthesize::Refusal>,
) {
    refusals.push(crate::synthesize::Refusal {
        subject: subject.clone().into(),
        scenario: None,
        cause: crate::synthesize::RefusalCause::NoWitness(crate::witness::WitnessGap {
            path: format!("binding.{subject}.periodic"),
            type_ref: check.periodic.every.to_string(),
            reason: "PeriodicResource: required host inputs or five-period window exceed the bounded witness contract",
        }),
    });
}

/// Central writer/reader admission calls this before serialization or target effects.
pub fn admit_suite(suite: &ConformanceSuite) -> Result<(), crate::admission::AdmissionError> {
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            if let ScenarioStep::CheckPeriodic { check } = step {
                if suite.provenance.suite_version.major() < 6 {
                    return Err(crate::admission::AdmissionError::new(
                        "UnsupportedVocabulary",
                        "$suite",
                        "periodic checks require suite/6 or /7",
                    ));
                }
                check.validate().map_err(|error| {
                    crate::admission::AdmissionError::new("PeriodicContract", "$suite", error)
                })?;
            }
        }
    }
    Ok(())
}
