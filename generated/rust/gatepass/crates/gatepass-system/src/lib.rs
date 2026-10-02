// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

//! The `gatepass` system, v1: its components assembled, its bindings wired, and its one transport.
//!
//! The transport is derived from the specification, not chosen: `at_least_once` is the only
//! delivery guarantee the model declares, so published events land on an append-only log and a
//! pump delivers each to every binding that reacts to it. The log is the system's observable
//! record, and so is the record of what each binding invoked. What no specification determines
//! — how an escalation event is filled, behaviour behind the ports — stays an obligation; see
//! the `PLAN.md` beside this workspace.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// An event on the system's log: everything any component publishes, and everything a binding
/// escalates into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemEvent {
    /// `gatepass.visit.VisitRegistered`.
    VisitRegistered(gatepass_types::visit::VisitRegistered),
    /// `gatepass.visit.VisitorAdmitted`.
    VisitorAdmitted(gatepass_types::visit::VisitorAdmitted),
    /// `gatepass.visit.VisitorDeparted`.
    VisitorDeparted(gatepass_types::visit::VisitorDeparted),
}

impl SystemEvent {
    /// The qualified name the specification declares this event under.
    pub fn name(&self) -> &'static str {
        match self {
            Self::VisitRegistered(_) => "gatepass.visit.VisitRegistered",
            Self::VisitorAdmitted(_) => "gatepass.visit.VisitorAdmitted",
            Self::VisitorDeparted(_) => "gatepass.visit.VisitorDeparted",
        }
    }
}

impl From<pass_service::PublishedEvent> for SystemEvent {
    fn from(event: pass_service::PublishedEvent) -> Self {
        match event {
            pass_service::PublishedEvent::VisitRegistered(event) => Self::VisitRegistered(event),
            pass_service::PublishedEvent::VisitorAdmitted(event) => Self::VisitorAdmitted(event),
            pass_service::PublishedEvent::VisitorDeparted(event) => Self::VisitorDeparted(event),
        }
    }
}

/// The `gatepass` system: every component behind its port, and the transport between them.
///
/// The component fields are public because commands enter the system through a component's own
/// port; the log and its delivery cursor are not, because publishing happens by pumping, not by
/// writing history directly.
pub struct System<PassServiceBehaviors> {
    /// The `pass-service` component.
    pub pass_service: pass_service::PassService<PassServiceBehaviors>,
    published: Vec<SystemEvent>,
    cursor: usize,
}

impl<PassServiceBehaviors> System<PassServiceBehaviors> {
    /// Assembles the system from its components.
    pub fn new(pass_service: pass_service::PassService<PassServiceBehaviors>) -> Self {
        Self {
            pass_service,
            published: Vec::new(),
            cursor: 0,
        }
    }

    /// Everything published so far, in publication order — the system's observable record.
    pub fn published(&self) -> &[SystemEvent] {
        &self.published
    }

    /// Takes every event the pump has already delivered off the log, in publication order.
    ///
    /// A long-running shell calls this after each `pump`, or the log holds every event the
    /// process ever published. A `pump` returns with every logged event delivered: each
    /// reacting binding has had its attempt, and a binding whose attempt stopped holds the event in
    /// its own held-back list, not on the log. Events published since the last `pump` stay on the
    /// log, so the next `pump` still delivers them; taking never skips a binding.
    pub fn take_published(&mut self) -> Vec<SystemEvent> {
        let delivered: Vec<SystemEvent> = self.published.drain(..self.cursor).collect();
        self.cursor = 0;
        delivered
    }
}

impl<PassServiceBehaviors> System<PassServiceBehaviors>
where
    PassServiceBehaviors: gatepass_types::visit::obligations::AdmitVisitorBehavior + gatepass_types::visit::obligations::RegisterVisitBehavior + gatepass_types::visit::obligations::SignOutVisitorBehavior + gatepass_types::visit::obligations::ExpectedVisitsQuery + gatepass_types::visit::obligations::VisitByIdQuery,
{
    /// Delivers until quiescent: collects every component's outbox onto the log. No binding
    /// reacts to anything this specification publishes, so collecting is the whole delivery.
    pub fn pump(&mut self) -> Result<(), gatepass_types::obligation::UnmetObligation> {
        loop {
            self.collect();
            if self.cursor == self.published.len() {
                return Ok(());
            }
            self.cursor += 1;
        }
    }

    /// Moves every component's outbox onto the log, in component order.
    fn collect(&mut self) {
        for event in self.pass_service.drain_outbox() {
            self.published.push(SystemEvent::from(event));
        }
    }
}
