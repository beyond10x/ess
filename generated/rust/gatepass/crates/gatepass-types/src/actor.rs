// generated from gatepass v1
// model digest f8ccea748a49e127ca2e18f725481394cc0eab1787fafd77d16c52485bf2abba
// contract digest a6fdd92f3a88ac0abbe59789406f3001df466e87f222e4aad1a8348c17f91d7c
// do not edit: regenerate with `ess synthesize`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! A grant is checked against a caller identity, which these types do not read from anywhere:
//! whatever authenticates a request builds a [`Caller`], and a served surface checks it against
//! [`may`] before the command runs. The `PLAN.md` beside this workspace says, per actor,
//! whether a generated surface enforces the grant or the caller does.

/// An actor the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Actor {
    /// `gatepass.visit.Receptionist`.
    Receptionist,
    /// `gatepass.visit.SecurityAuditor`.
    SecurityAuditor,
}

impl Actor {
    /// Every declared actor, ordered by qualified name.
    pub const ALL: &'static [Actor] = &[
        Actor::Receptionist,
        Actor::SecurityAuditor,
    ];

    /// The actor's qualified name, as the specification spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Actor::Receptionist => "gatepass.visit.Receptionist",
            Actor::SecurityAuditor => "gatepass.visit.SecurityAuditor",
        }
    }
}

/// The qualified names of the commands `actor` may invoke, ordered by name; empty for an
/// actor that only observes.
pub fn may(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::Receptionist => &[
            "gatepass.visit.AdmitVisitor",
            "gatepass.visit.RegisterVisit",
            "gatepass.visit.SignOutVisitor",
        ],
        Actor::SecurityAuditor => &[],
    }
}

/// Who a request was authenticated as.
///
/// Built by whatever authenticates the request — a session, a token, a certificate — and
/// handed to the served surface's `dispatch` and `handle`, which check its grant
/// before the command runs. Never derived from the request itself: a client can write
/// anything into a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Caller {
    /// The declared actor.
    pub actor: Actor,
}

impl Caller {
    /// `true` when this caller may invoke `command`, named by its qualified name.
    pub fn may(&self, command: &str) -> bool {
        may(self.actor).contains(&command)
    }
}
