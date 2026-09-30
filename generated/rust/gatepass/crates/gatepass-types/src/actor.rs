// generated from gatepass v1
// model digest f8ccea748a49e127ca2e18f725481394cc0eab1787fafd77d16c52485bf2abba
// contract digest a6fdd92f3a88ac0abbe59789406f3001df466e87f222e4aad1a8348c17f91d7c
// do not edit: regenerate with `ess synthesize`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! Generated, not enforced: a grant is checked against a caller identity, which types do not
//! carry, so the caller that knows who is calling enforces it, against [`may`]. The `PLAN.md`
//! beside this workspace records the same refusal for every actor.

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
