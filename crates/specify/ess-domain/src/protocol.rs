//! Experimental finite communicating-machine authoring, independent of ordinary ESS documents.
//!
//! Values are admitted against declared scalar fields by `ess-compiler::protocol`. This module
//! reads syntax only; callers must not execute a raw document without compiler admission.
use std::collections::BTreeMap;

use crate::TypeRef;
use ess_primitives::node::Node;

/// Experimental source contract. Unknown versions are refused by compiler admission.
pub const FORMAT: &str = "ess-protospec/1";

/// An authored protocol sidecar. All referenced declarations are local to this document.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawProtocol {
    /// Versioned experimental vocabulary.
    pub format: String,
    /// Protocol identity.
    pub name: String,
    /// Independently evolving peer machines.
    pub participants: Vec<Participant>,
    /// Typed messages available to channels.
    pub messages: Vec<Message>,
    /// Directed bounded channels.
    pub channels: Vec<Channel>,
    /// Required safety properties.
    #[serde(default)]
    pub properties: Vec<Property>,
    /// Finite execution and exploration limits.
    pub bounds: Bounds,
}

impl RawProtocol {
    /// Read YAML or JSON through an intermediate YAML value, refusing duplicate mapping keys.
    pub fn parse(text: &str) -> Result<Self, serde_yaml::Error> {
        let value: serde_yaml::Value = serde_yaml::from_str(text)?;
        serde_yaml::from_value(value)
    }
}

/// One local state machine and its declarations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    /// Local participant identity.
    pub name: String,
    /// Initial state.
    pub initial: String,
    /// Closed state vocabulary.
    pub states: Vec<String>,
    /// Typed local values and initial assignments.
    #[serde(default)]
    pub fields: Vec<LocalField>,
    /// Local application input shapes.
    #[serde(default)]
    pub inputs: Vec<Message>,
    /// Timer names scoped to this participant.
    #[serde(default)]
    pub timers: Vec<String>,
    /// Permitted transitions; multiple matches represent nondeterminism.
    pub transitions: Vec<Transition>,
}

/// A required scalar field.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    /// Field identity.
    pub name: String,
    /// ESS type; this experimental profile admits String, Boolean and Integer only.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
}

/// A local scalar field with a mandatory initial value.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalField {
    /// Field identity.
    pub name: String,
    /// Admitted ESS scalar type.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    /// Value before the first input.
    pub initial: Node,
}

/// A message or local input shape; no undeclared payload fields are admitted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    /// Message identity.
    pub name: String,
    /// Required payload fields.
    #[serde(default)]
    pub fields: Vec<Field>,
}

/// One atomic local transition; external effects are ordered observations.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    /// Stable transition identity within its participant.
    pub name: String,
    /// Required current state.
    pub from: String,
    /// State after processing the input.
    pub to: String,
    /// Input that enables this transition.
    pub trigger: Trigger,
    /// Conjunction of typed equalities.
    #[serde(default)]
    pub guards: Vec<Equality>,
    /// Effects in observation order.
    #[serde(default)]
    pub effects: Vec<Effect>,
}

/// An input available to one local machine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Trigger {
    /// A local application action.
    Input {
        /// Declared local input.
        name: String,
    },
    /// Delivery of one message copy.
    Receive {
        /// Incoming channel.
        channel: String,
        /// Declared message.
        message: String,
    },
    /// A current timer generation expires.
    Timer {
        /// Timer owned by this participant.
        name: String,
    },
}

/// A value read from the local machine or the input currently being processed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValueSource {
    /// A typed scalar literal.
    Literal {
        /// Scalar value.
        value: Node,
    },
    /// A participant-local field; remote reads have no syntax.
    Local {
        /// Declared local field.
        field: String,
    },
    /// A field of the triggering input/message.
    Input {
        /// Declared triggering payload field.
        field: String,
    },
}

/// Equality over two operands of the same admitted scalar type.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Equality {
    /// First operand.
    pub left: ValueSource,
    /// Second operand.
    pub right: ValueSource,
}

/// Explicit local effects. Flush is a semantic boundary observation, not proof of remote receipt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    /// Assign a typed local field.
    Set {
        /// Destination field.
        field: String,
        /// Assigned value.
        value: ValueSource,
    },
    /// Enqueue a new transmission occurrence; delivery is a separate action.
    Send {
        /// Outgoing channel.
        channel: String,
        /// Message shape.
        message: String,
        /// Exchange identity, a String.
        exchange: ValueSource,
        /// Logical message identity, a String.
        logical: ValueSource,
        /// Exactly the fields declared by the message.
        #[serde(deserialize_with = "deserialize_unique_map")]
        payload: BTreeMap<String, ValueSource>,
    },
    /// Arm a fresh generation of a local timer.
    Arm {
        /// Timer name.
        timer: String,
        /// Checked duration arithmetic.
        after: Duration,
    },
    /// Cancel the current generation.
    Cancel {
        /// Timer name.
        timer: String,
    },
    /// Notify the local application.
    Emit {
        /// Observable event name.
        name: String,
    },
    /// Observe completion of flushing locally queued sends on the named channel.
    Flush {
        /// Outgoing channel.
        channel: String,
    },
    /// Close this participant's scope.
    Close,
}

/// Supported checked timer arithmetic in milliseconds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Duration {
    /// Constant positive duration.
    Fixed {
        /// Duration in milliseconds.
        millis: u64,
    },
    /// Multiply the last armed interval, capped at a positive ceiling.
    Backoff {
        /// Local timer whose previous interval is retained even after expiry.
        timer: String,
        /// Positive multiplier.
        factor: u32,
        /// Maximum resulting interval.
        ceiling_ms: u64,
    },
}

/// Delivery order on one directed channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ordering {
    /// Only the first queued occurrence may be delivered.
    Fifo,
    /// Any queued occurrence may be delivered.
    Unordered,
}

/// A bounded directed channel. Loss and duplication require explicit permission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    /// Channel identity.
    pub name: String,
    /// Sending participant.
    pub from: String,
    /// Receiving participant.
    pub to: String,
    /// Delivery scheduling policy.
    pub ordering: Ordering,
    /// Maximum in-flight occurrences/copies.
    pub capacity: usize,
    /// Whether the scheduler may drop messages.
    pub loss: bool,
    /// Whether the scheduler may duplicate messages.
    pub duplication: bool,
}

/// Finite resource limits. Exhaustion never constitutes a passing result.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    /// Maximum steps of one execution.
    pub max_steps: usize,
    /// Maximum configurations considered by exploration.
    pub max_states: usize,
    /// Maximum logical time in milliseconds.
    pub max_time_ms: u64,
}

/// Initial safety-property vocabulary; unbounded liveness is deliberately unsupported.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Property {
    /// A participant must never enter a forbidden state.
    NeverState {
        /// Obligation identity.
        name: String,
        /// Observed participant.
        participant: String,
        /// Forbidden state.
        state: String,
    },
    /// Bound the count of one application event over the complete session.
    EventCount {
        /// Obligation identity.
        name: String,
        /// Observed participant.
        participant: String,
        /// Application event.
        event: String,
        /// Inclusive maximum count.
        max: usize,
    },
    /// Closing requires a flush after the latest send on the named channel.
    FlushBeforeClose {
        /// Obligation identity.
        name: String,
        /// Observed participant.
        participant: String,
        /// Outgoing channel.
        channel: String,
    },
    /// One participant's state requires a specified state at another participant.
    StateImplication {
        /// Obligation identity.
        name: String,
        /// Antecedent participant.
        participant: String,
        /// Antecedent state.
        state: String,
        /// Consequent participant.
        other: String,
        /// Consequent state.
        other_state: String,
    },
}

/// Whether a runtime scalar has exactly the declared experimental field type.
pub fn admits(type_ref: &TypeRef, value: &Node) -> bool {
    match (type_ref, value) {
        (TypeRef::Primitive(crate::Primitive::String), Node::Text(_))
        | (TypeRef::Primitive(crate::Primitive::Boolean), Node::Bool(_)) => true,
        (TypeRef::Primitive(crate::Primitive::Integer), Node::Number(value)) => {
            value.as_i64().is_some()
        }
        _ => false,
    }
}

/// Read a string-keyed map without silently replacing a repeated key.
///
/// Use as `deserialize_with` on runtime payload maps as well as authored payload maps.
pub fn deserialize_unique_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: serde::Deserialize<'de>,
{
    struct Visitor<V>(std::marker::PhantomData<V>);
    impl<'de, V: serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<V> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a mapping without duplicate keys")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, V>()? {
                if values.contains_key(&key) {
                    return Err(serde::de::Error::custom(format!("duplicate key `{key}`")));
                }
                values.insert(key, value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(Visitor(std::marker::PhantomData))
}
