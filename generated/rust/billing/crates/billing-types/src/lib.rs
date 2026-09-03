// generated from billing v3
// model digest 62706dc8de60f859f9fa11d363bae20825e7c74e71435e2fd28691488d787af1
// contract digest d0791c480f462a0bd205e4eda077f60c22bedf0f83756f7ff35687682ce8e3dd
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
