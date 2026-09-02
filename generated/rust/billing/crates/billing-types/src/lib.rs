// generated from billing v3
// model digest aacdc2fe065d462cc4f9ba51e6740f88809b6b17ce006ef846b488f957005da3
// contract digest 6ba34a27496cc918b55c749b45599c03b3016fed36487b1763268b95e0c6ffc6
// do not edit: regenerate with `ess synthesize`

//! Semantic types synthesised from the `billing` specification, v3.
//!
//! Invoicing and the notification that follows it: the smallest system that still exercises every construct the model has — two bounded contexts, a command that can be refused, a command with an outcome its input cannot decide, both consistency levels, a state machine, actors, and a type of every kind.
//!
//! Generated, not written: the specification is the source of truth, and the door to changing
//! anything here is `ess synthesize`. What is deliberately absent — behaviour, queries,
//! escalations — is listed with reasons in the `PLAN.md` beside this workspace, and every entry
//! there is owed through a typed seam in an `obligations` module here.

// `deny`, not the source workspace's lint set: this crate must hold on its own, and an undocumented
// public item here is an emitter defect worth failing the gate over.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod email;
pub mod invoice;
pub mod obligation;
pub mod primitives;
