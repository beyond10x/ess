//! Typed identity of each generated finite disclosure obligation.
use crate::scenario::{ActorRef, CommandRef, OutcomeRef, ViewRef};
use ess_domain::{OutcomeName, QualifiedName};
use std::fmt;

/// The follow-up whose observations must not disclose a captured origin value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Aspect {
    /// The first successful origin and its direct and independent publications.
    Origin,
    /// An exact repeated invocation under the same actor.
    Retry,
    /// A further successful origin, fresh against every prior captured value.
    Rotation,
    /// One declared view, before and after rotation where reachable.
    Read(ViewRef),
    /// One reachable declared follow-up outcome.
    Command(OutcomeRef),
    /// An actual denied caller invoking a declared command.
    Denied(CommandRef),
}

/// A source-owned inventory cell; never an authored label or ordinal counter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cell {
    /// The successful marked origin.
    pub origin: OutcomeRef,
    /// One declared marked field. The observer still protects every marked field.
    pub field: String,
    /// The independent obligation represented by this cell.
    pub aspect: Aspect,
    /// Actual follow-up caller; `None` is the model's actorless seam.
    pub actor: Option<ActorRef>,
}

impl Cell {
    /// Read the one canonical generated identity spelling.
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        let parts: Vec<_> = value.split('/').collect();
        let [command, "disclosure", outcome, field, rest @ ..] = parts.as_slice() else {
            return Err("invalid disclosure identity");
        };
        let name = |value| QualifiedName::new(value).map_err(|_| "invalid disclosure name");
        let branch = |command, outcome| -> Result<OutcomeRef, &'static str> {
            Ok(OutcomeRef::new(
                CommandRef::new(name(command)?),
                OutcomeName::new(outcome).map_err(|_| "invalid disclosure outcome")?,
            ))
        };
        if !ess_domain::types::is_field_name(field) {
            return Err("invalid disclosure field");
        }
        let (aspect, actor) = match rest {
            [aspect @ .., "as", "anonymous"] => (aspect, None),
            [aspect @ .., "as", "actor", actor] => (aspect, Some(ActorRef::new(name(actor)?))),
            _ => return Err("disclosure identity omits caller"),
        };
        let aspect = match aspect {
            ["origin"] => Aspect::Origin,
            ["retry"] => Aspect::Retry,
            ["rotation"] => Aspect::Rotation,
            ["read", view] => Aspect::Read(ViewRef::new(name(view)?)),
            ["command", command, outcome] => Aspect::Command(branch(command, outcome)?),
            ["denied", command] => Aspect::Denied(CommandRef::new(name(command)?)),
            _ => return Err("unknown disclosure aspect"),
        };
        Ok(Self {
            origin: branch(command, outcome)?,
            field: (*field).to_owned(),
            aspect,
            actor,
        })
    }

    /// Whether `scenario` carries the obligation this cell names, read as a suite of `major`
    /// carries it: from suite/[`crate::one_time_reader::ORDINARY`] a read cell's actor is the
    /// reader, below it the issuer.
    pub(super) fn validate(
        &self,
        scenario: &crate::ConformanceScenario,
        major: u32,
    ) -> Result<(), &'static str> {
        let policy = scenario
            .one_time_response
            .as_ref()
            .ok_or("disclosure identity has no trace policy")?;
        if !policy.required_origins.contains(&self.origin)
            || !policy
                .origins
                .iter()
                .any(|origin| origin.outcome == self.origin && origin.fields.contains(&self.field))
        {
            return Err("disclosure identity contradicts required marked origin");
        }
        let steps = &scenario.steps;
        let (invoked, selected_at) = selected(steps, &self.origin, None)
            .next()
            .ok_or("disclosure identity has no selected originating invocation")?;
        let tail = &steps[selected_at + 1..];
        let origin_actor = invocation(&steps[invoked]).map(|(_, actor)| actor);
        let fulfilled = match &self.aspect {
            Aspect::Origin => origin_actor == Some(&self.actor),
            Aspect::Retry => origin_actor == Some(&self.actor) && tail.contains(&steps[invoked]),
            Aspect::Rotation => selected(tail, &self.origin, Some(&self.actor)).next().is_some(),
            Aspect::Read(view) if major >= crate::one_time_reader::ORDINARY => {
                read_as_reader(steps, selected_at, view, self.actor.as_ref())
            }
            Aspect::Read(view) => origin_actor == Some(&self.actor) && tail.iter().any(|step| matches!(step, crate::ScenarioStep::QueryView { view: queried, .. } | crate::ScenarioStep::EventuallyView { view: queried, .. } if view == queried)),
            Aspect::Command(expected) => selected(tail, expected, Some(&self.actor)).next().is_some(),
            Aspect::Denied(command) => denied(tail, command, self.actor.as_ref()),
        };
        if fulfilled {
            Ok(())
        } else {
            Err("disclosure identity has no matching follow-up obligation")
        }
    }
}

/// Whether `view` is read after the origin selected at `selected_at` as `reader`: the actor of the
/// last [`ReadAs`](crate::ScenarioStep::ReadAs) before the read, or no actor where none stands
/// before it (suite/56, beyond10x/ess#614). The origin's own checks may read the view on the
/// harness's authority before that.
fn read_as_reader(
    steps: &[crate::ScenarioStep],
    selected_at: usize,
    view: &ViewRef,
    reader: Option<&ActorRef>,
) -> bool {
    let mut in_force: Option<&ActorRef> = None;
    steps.iter().enumerate().any(|(index, step)| match step {
        crate::ScenarioStep::ReadAs { actor } => {
            in_force = actor.as_ref();
            false
        }
        crate::ScenarioStep::QueryView { view: queried, .. }
        | crate::ScenarioStep::EventuallyView { view: queried, .. } => {
            index > selected_at && queried == view && in_force == reader
        }
        _ => false,
    })
}

fn invocation(step: &crate::ScenarioStep) -> Option<(&CommandRef, &Option<ActorRef>)> {
    match step {
        crate::ScenarioStep::ExecuteCommand { command, actor, .. }
        | crate::ScenarioStep::ExecuteCommandWithoutInput { command, actor, .. } => {
            Some((command, actor))
        }
        _ => None,
    }
}

fn selected<'a>(
    steps: &'a [crate::ScenarioStep],
    expected: &'a OutcomeRef,
    actor: Option<&'a Option<ActorRef>>,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let mut last = None;
    steps.iter().enumerate().filter_map(move |(index, step)| {
        if invocation(step).is_some() {
            last = Some(index);
        }
        let invoked = last?;
        let (command, caller) = invocation(&steps[invoked])?;
        if command == &expected.command
            && actor.is_none_or(|actor| actor == caller)
            && matches!(step, crate::ScenarioStep::ExpectOutcome { outcome } if outcome == expected)
        {
            last = None; // Two assertions about one invocation are still one invocation.
            Some((invoked, index))
        } else {
            None
        }
    })
}

fn denied(steps: &[crate::ScenarioStep], command: &CommandRef, actor: Option<&ActorRef>) -> bool {
    let Some(actor) = actor else {
        return false;
    };
    let mut last = None;
    steps.iter().any(|step| {
        if let Some(invoked) = invocation(step) { last = Some(invoked); }
        matches!(step, crate::ScenarioStep::ExpectNotGranted { actor: Some(denied), .. } if denied == actor)
            && last.is_some_and(|(invoked, caller)| invoked == command && caller.as_ref() == Some(actor))
    })
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/disclosure/{}/{}/",
            self.origin.command, self.origin.outcome, self.field
        )?;
        match &self.aspect {
            Aspect::Origin => f.write_str("origin")?,
            Aspect::Retry => f.write_str("retry")?,
            Aspect::Rotation => f.write_str("rotation")?,
            Aspect::Read(view) => write!(f, "read/{view}")?,
            Aspect::Command(outcome) => {
                write!(f, "command/{}/{}", outcome.command, outcome.outcome)?;
            }
            Aspect::Denied(command) => write!(f, "denied/{command}")?,
        }
        match &self.actor {
            Some(actor) => write!(f, "/as/actor/{actor}"),
            None => f.write_str("/as/anonymous"),
        }
    }
}
