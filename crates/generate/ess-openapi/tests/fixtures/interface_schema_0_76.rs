// Exact `InterfaceSchema` of the 0.76.0 release (`crates/generate/ess-openapi/src/lib.rs`), frozen
// so that an old reader's answer to the nodes added later is checked against real code.
use std::collections::{BTreeMap, BTreeSet};

/// Structural JSON Schema subset preserved by this adapter.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InterfaceSchema {
    /// Reference to a named component schema.
    Reference {
        /// Component-schema name.
        name: String,
    },
    /// String value, optionally refined without changing its structural type.
    String {
        /// `OpenAPI` format annotation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        /// Regular-expression refinement.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        /// Closed string enumeration.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        values: Vec<String>,
        /// One required literal value.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        constant: Option<String>,
    },
    /// Integer value.
    Integer,
    /// General JSON number.
    Number,
    /// Boolean value.
    Boolean,
    /// Ordered array.
    Array {
        /// Element shape.
        items: Box<InterfaceSchema>,
    },
    /// Closed tuple: exactly these items, in this order (`prefixItems` with `items: false`, or
    /// `maxItems: 0` for none).
    Tuple {
        /// Item shapes, in order.
        items: Vec<InterfaceSchema>,
    },
    /// Closed object with explicitly typed properties.
    Object {
        /// Properties keyed deterministically by wire name.
        properties: BTreeMap<String, InterfaceSchema>,
        /// Required property names.
        #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
        required: BTreeSet<String>,
    },
}
