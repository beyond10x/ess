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
pub mod facts;
pub mod ids;
pub mod node;
pub mod predicate;
pub mod time;
pub mod verification;

pub use entity::{EntityLocator, EntityType};
pub use error::{ParseError, ValidationCode, ValidationError, ValidationErrors};
pub use evidence::{EssConformanceResult, Evidence, Producer, Provenance, SpecDigest};
pub use facts::{FactPath, FactPattern, FactSource, FactStore, FactValue, Number, Scales};
pub use ids::CorrelationId;
pub use node::Node;
pub use predicate::{CompareOp, LeafOutcome, Operand, Predicate, PredicateOutcome, Truth};
pub use time::{CivilDate, Granularity, Horizon, ObservedAt, Timestamp};
pub use verification::{VerificationStatus, Verifier};
