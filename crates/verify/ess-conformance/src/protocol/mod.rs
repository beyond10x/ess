//! Bounded communicating machines. Reference execution and target observations remain separate.
mod engine;
mod monitor;
mod target;
pub use engine::{enabled_actions, initial, simulate, successors};
use ess_primitives::Node;
pub use monitor::{check_trace, explore, ExplorationReport, Report, Verdict};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub use target::{capture_target, run_target, Capabilities, Capture, ProtocolTarget};

/// Strict persisted trace format, separate from existing ESS conformance suites.
pub const TRACE_FORMAT: &str = "ess-prototrace/1";

/// An explicitly selected external stimulus or network scheduling decision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Invoke a declared local input.
    Input {
        /// Target participant.
        participant: String,
        /// Declared input name.
        name: String,
        /// Typed fields of the input.
        #[serde(deserialize_with = "unique_map")]
        payload: BTreeMap<String, Node>,
    },
    /// Consume a queued transmission occurrence.
    Deliver {
        /// Queue occurrence identity.
        transmission: u64,
    },
    /// Lose a queued occurrence, only on a loss-capable channel.
    Drop {
        /// Queue occurrence identity.
        transmission: u64,
    },
    /// Create a duplicate queue occurrence, only on a duplication-capable channel.
    Duplicate {
        /// Existing queue occurrence identity.
        transmission: u64,
    },
    /// Advance logical time without crossing an unfired deadline.
    AdvanceTo {
        /// Absolute logical milliseconds.
        millis: u64,
    },
    /// Fire an owned, current-generation timer exactly at its deadline.
    Fire {
        /// Owning participant.
        participant: String,
        /// Declared timer name.
        timer: String,
        /// Generation returned by the arm observation.
        generation: u64,
    },
}

/// Observable facts, in their causal order within a step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Observation {
    /// A transmission was handed to a channel.
    Sent {
        /// The complete occurrence.
        message: Transmission,
    },
    /// A queued occurrence was delivered.
    Delivered {
        /// Occurrence identity.
        transmission: u64,
    },
    /// A permitted loss occurred.
    Dropped {
        /// Occurrence identity.
        transmission: u64,
    },
    /// A permitted duplicate was created.
    Duplicated {
        /// Original occurrence identity.
        original: u64,
        /// New occurrence identity.
        transmission: u64,
    },
    /// An owned timer was armed.
    TimerArmed {
        /// Owner.
        participant: String,
        /// Timer name.
        timer: String,
        /// Replacement generation.
        generation: u64,
        /// Absolute logical deadline.
        deadline_ms: u64,
    },
    /// A timer was canceled.
    TimerCanceled {
        /// Owner.
        participant: String,
        /// Timer name.
        timer: String,
    },
    /// A current timer fired.
    TimerFired {
        /// Owner.
        participant: String,
        /// Timer name.
        timer: String,
        /// Exact fired generation.
        generation: u64,
    },
    /// Application-visible notification.
    Emitted {
        /// Producer.
        participant: String,
        /// Declared notification.
        name: String,
    },
    /// Prior writes to one channel completed.
    Flushed {
        /// Producer.
        participant: String,
        /// Channel.
        channel: String,
    },
    /// A participant closed.
    Closed {
        /// Closing participant.
        participant: String,
    },
    /// Logical time advanced.
    TimeAdvanced {
        /// New logical instant.
        millis: u64,
    },
}

/// One actual queue occurrence; logical identity survives retransmission and duplication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transmission {
    /// Unique queue occurrence identity.
    pub id: u64,
    /// First occurrence of this transmission (duplicates share it).
    pub original: u64,
    /// Declared channel.
    pub channel: String,
    /// Declared message type.
    pub message: String,
    /// Exchange correlation identity.
    pub exchange: String,
    /// Logical message identity within that exchange.
    pub logical: String,
    /// Typed message fields.
    #[serde(deserialize_with = "unique_map")]
    pub payload: BTreeMap<String, Node>,
}

/// One active timer incarnation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Timer {
    /// Monotonically increasing incarnation.
    pub generation: u64,
    /// Absolute deadline in logical milliseconds.
    pub deadline_ms: u64,
}
/// Participant-local execution state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Peer {
    /// Current finite state.
    pub state: String,
    /// Typed local fields.
    pub fields: BTreeMap<String, Node>,
    /// Active timers by name.
    pub timers: BTreeMap<String, Timer>,
    /// Last allocated generation by name, retained after cancellation.
    pub generations: BTreeMap<String, u64>,
    /// Last interval, retained after firing for capped backoff.
    pub intervals: BTreeMap<String, u64>,
    /// Application notification counts.
    pub events: BTreeMap<String, usize>,
    /// Channels with writes after their latest flush.
    pub unflushed: BTreeSet<String>,
    /// Channels flushed at least once.
    pub flushed: BTreeSet<String>,
    /// Whether this participant has closed.
    pub closed: bool,
}
/// Entire bounded reference configuration. No wall clock or I/O enters it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Configuration {
    /// Logical instant.
    pub now_ms: u64,
    /// Applied action count.
    pub steps: usize,
    /// Next occurrence identity.
    pub next_transmission: u64,
    /// Participant-local state.
    pub peers: BTreeMap<String, Peer>,
    /// Network queue in transmission order.
    pub queue: Vec<Transmission>,
    /// Violated property identities; sticky across later steps.
    pub violations: BTreeSet<String>,
}
impl Configuration {
    /// There are no outstanding network or timer obligations.
    pub fn settled(&self) -> bool {
        self.queue.is_empty()
            && self
                .peers
                .values()
                .all(|p| p.timers.is_empty() && p.unflushed.is_empty())
    }
}
/// One enabled result, allowing nondeterministic model transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Step {
    /// Configuration after the action.
    pub configuration: Configuration,
    /// Required observations in causal order.
    pub observations: Vec<Observation>,
}
/// One recorded stimulus and its independently observed facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceStep {
    /// Applied action.
    pub action: Action,
    /// Actual observations; never filled from the oracle for target traces.
    pub observations: Vec<Observation>,
}
/// Replayable evidence tied to one admitted protocol model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TraceOrigin {
    /// Observations were computed by the reference model, not an implementation.
    Model,
    /// Observations were captured from a separately identified implementation adapter.
    Target {
        /// Adapter-supplied implementation identity; provenance is not authenticated.
        implementation: String,
    },
}
/// Replayable evidence tied to one admitted protocol model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    /// Exactly `ess-prototrace/1`.
    pub format: String,
    /// Canonical model identity.
    pub model_digest: String,
    /// Whether the facts were simulated or supplied by an implementation adapter.
    pub origin: TraceOrigin,
    /// Ordered observations and their stimuli.
    pub steps: Vec<TraceStep>,
    /// Capture was complete at all required observation boundaries.
    pub complete: bool,
}
/// Refusal of an invalid action or exhaustion of a declared bound.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionError {
    /// Whether this is exhausted evidence capacity rather than contradictory behavior.
    pub bound: bool,
    /// Concrete explanation.
    pub message: String,
}
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}
impl std::error::Error for ExecutionError {}
impl ExecutionError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            bound: false,
            message: message.into(),
        }
    }
    fn bound(message: impl Into<String>) -> Self {
        Self {
            bound: true,
            message: message.into(),
        }
    }
}

fn unique_map<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Node>, D::Error> {
    struct Unique;
    impl<'de> serde::de::Visitor<'de> for Unique {
        type Value = BTreeMap<String, Node>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("an object with unique field names")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut result = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, Node>()? {
                if result.insert(key.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!("duplicate field {key}")));
                }
            }
            Ok(result)
        }
    }
    d.deserialize_map(Unique)
}
#[cfg(test)]
mod tests;
