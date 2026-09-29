// generated from billing v3
// model digest 1e7906786567af32118eb2d0a8c3fcafa16c32c9649a80b60487fd2eeebc4c9c
// contract digest a21fd36f0055057629f4c235962163cdd34a3d178aa068925bcb53be623af301
// do not edit: regenerate with `ess synthesize`

//! Every actor the specification declares, and the commands each may invoke — as data.
//!
//! Generated, not enforced: a grant is checked against a caller identity, which types do not
//! carry, so the caller that knows who is calling enforces it, against [`may`]. The `PLAN.md`
//! beside this workspace records the same refusal for every actor.

/// An actor the specification declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Actor {
    /// `billing.invoice.Auditor`.
    Auditor,
    /// `billing.invoice.Customer`.
    Customer,
}

impl Actor {
    /// Every declared actor, ordered by qualified name.
    pub const ALL: &'static [Actor] = &[
        Actor::Auditor,
        Actor::Customer,
    ];

    /// The actor's qualified name, as the specification spells it.
    pub const fn name(self) -> &'static str {
        match self {
            Actor::Auditor => "billing.invoice.Auditor",
            Actor::Customer => "billing.invoice.Customer",
        }
    }
}

/// The qualified names of the commands `actor` may invoke, ordered by name; empty for an
/// actor that only observes.
pub fn may(actor: Actor) -> &'static [&'static str] {
    match actor {
        Actor::Auditor => &[],
        Actor::Customer => &[
            "billing.invoice.CreateInvoice",
        ],
    }
}
