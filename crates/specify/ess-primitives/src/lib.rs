//! Shared deterministic primitives owned by Executable System Specification.
//!
//! These types preserve the serialized behavior ESS used before repository extraction without
//! retaining a dependency on AEP's domain or contract crates. They are value-in/value-out and
//! contain no clock, filesystem, environment, network or random source.

#![forbid(unsafe_code)]

pub mod consistency;
pub mod entity;
pub mod error;
pub mod evidence;
pub mod expr;
pub mod facts;
pub mod ids;
pub mod json;
pub mod node;
pub mod predicate;
pub mod time;
pub mod verification;
pub mod window;

/// YAML, as ESS reads and writes it: the `ess-yaml` crate, re-exported so that a crate whose only
/// ESS dependency is this one (the `infra-*` crates) reads YAML the way the rest of ESS does.
pub use ess_yaml as yaml;

pub use entity::{EntityLocator, EntityType};
pub use error::{ParseError, ValidationCode, ValidationError, ValidationErrors};
pub use evidence::SpecDigest;
pub use facts::{FactPath, FactPattern, FactSource, FactStore, FactValue, Number, Scales};
pub use ids::CorrelationId;
pub use node::Node;
pub use predicate::{
    CompareOp, LeafOutcome, Operand, Predicate, PredicateOutcome, Quantified, Truth,
};
pub use time::{CivilDate, Granularity, Horizon, ObservedAt, Rfc3339Instant, Timestamp};
pub use verification::VerificationStatus;
