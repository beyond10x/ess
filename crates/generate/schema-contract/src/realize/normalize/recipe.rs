//! Typed authored recipe; shared verbatim with standalone normalization targets.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Authored behavior envelope, independent of schema and model formats.
pub const FORMAT: &str = "ess-normalization/1";

/// One replay-checked bundle root, identified without filesystem or network authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    /// SHA-256 of canonical qualified-bundle bytes, not only its underlying schema source.
    pub bundle_digest: String,
    /// An independently selectable root admitted by that bundle.
    pub root: String,
}

/// Explicit external dispatch and ordered stages. Construction alone does not validate it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    /// Must equal [`FORMAT`].
    pub format: String,
    /// Exact external discriminator values; no trial decoding or implicit default branch.
    #[serde(deserialize_with = "unique_map")]
    pub branches: BTreeMap<String, Vec<Stage>>,
}

/// An immutable data boundary, with explicit checks before transformation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    /// Input contract; after the first stage it must equal the previous output identity.
    pub input: Root,
    /// Contract validated after evaluation, including schema refinements.
    pub output: Root,
    /// Additional requirements on this stage's input, evaluated in declaration order.
    pub requires: Vec<Condition>,
    /// Complete output value. Record construction omits missing members explicitly.
    pub value: Expr,
}

/// No floating-point coercion is hidden inside integer operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Overflow {
    /// Refuse an arithmetic result outside signed 64-bit representation.
    Reject,
    /// Two's-complement signed 64-bit arithmetic, explicitly requested by the author.
    Wrap,
}

/// Concrete integer operations, not an expression-language string.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegerOp {
    /// Sum the operands.
    Add,
    /// Multiply the operands.
    Multiply,
}

/// Read scopes are lexical and never resolved from a global runtime context.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// The current stage's immutable input.
    Input,
    /// The current map/count item's immutable value.
    Item,
}

/// Pure value operations with separate absence and JSON-null semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expr {
    /// JSON null, which is never the missing-value sentinel.
    Null,
    /// A boolean literal.
    Boolean {
        /// The value.
        value: bool,
    },
    /// A string literal.
    String {
        /// The exact string, without interpolation.
        value: String,
    },
    /// An exact signed integer literal.
    Integer {
        /// The value.
        value: i64,
    },
    /// Read a declared object-member path. An empty path reads the whole scoped value.
    Read {
        /// Input or lexical collection item.
        scope: Scope,
        /// Exact field keys, not a dotted expression.
        path: Vec<String>,
    },
    /// Read a member of an explicitly computed object, such as one with a null fallback.
    Field {
        /// Object expression.
        object: Box<Expr>,
        /// Exact declared key.
        name: String,
    },
    /// Construct a closed record; missing member values are omitted, not changed to null.
    Record {
        /// Wire-keyed expressions.
        #[serde(deserialize_with = "unique_map")]
        fields: BTreeMap<String, Expr>,
    },
    /// Construct a list. A missing element is an error, not a shifted index.
    List {
        /// Ordered elements.
        items: Vec<Expr>,
    },
    /// Replace absence and optionally null. Zero, false and empty strings are not absence.
    Fallback {
        /// Value checked first.
        value: Box<Expr>,
        /// Evaluated only when fallback is required.
        fallback: Box<Expr>,
        /// Whether present null also requests fallback.
        on_null: bool,
    },
    /// Only one value branch is evaluated.
    Choose {
        /// Selection condition.
        condition: Box<Condition>,
        /// Value when true.
        then_value: Box<Expr>,
        /// Value when false.
        else_value: Box<Expr>,
    },
    /// Signed-64-bit arithmetic, refusing nonintegral tokens and out-of-range operands.
    Arithmetic {
        /// The operation.
        operation: IntegerOp,
        /// Left operand.
        left: Box<Expr>,
        /// Right operand.
        right: Box<Expr>,
        /// Explicit result-overflow policy.
        overflow: Overflow,
    },
    /// Transform every list item, retaining order.
    Map {
        /// List expression.
        list: Box<Expr>,
        /// Evaluated with a fresh lexical item binding.
        value: Box<Expr>,
    },
    /// Count distinct computed string keys of selected items.
    DistinctCount {
        /// List expression.
        list: Box<Expr>,
        /// Item selection; rejected items do not evaluate their key.
        condition: Box<Condition>,
        /// String-valued category expression.
        key: Box<Expr>,
    },
}

/// Explicit conditions; absence never compares equal to JSON null.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    /// True for every present value, including null.
    Present {
        /// Value to inspect.
        value: Box<Expr>,
    },
    /// Read a required boolean; no truthiness coercion.
    Boolean {
        /// Boolean expression.
        value: Box<Expr>,
    },
    /// Scalar null/boolean/string/signed-integer equality; no coercion between kinds.
    /// Either operand missing makes the condition false. Records, lists and general
    /// JSON numbers are refused during checking rather than compared by target rules.
    Equal {
        /// Left operand.
        left: Box<Expr>,
        /// Right operand.
        right: Box<Expr>,
    },
    /// Strict signed-integer comparison.
    Greater {
        /// Left operand.
        left: Box<Expr>,
        /// Right operand.
        right: Box<Expr>,
    },
    /// Exact case-sensitive string prefix; no trimming, normalization or URI parsing.
    StartsWith {
        /// Present non-null string to inspect.
        value: Box<Expr>,
        /// Present non-null prefix; an empty prefix always matches.
        prefix: Box<Expr>,
    },
    /// Ordered short-circuit conjunction; an empty list is true.
    All {
        /// Child conditions.
        conditions: Vec<Condition>,
    },
    /// Ordered short-circuit disjunction; an empty list is false.
    Any {
        /// Child conditions.
        conditions: Vec<Condition>,
    },
    /// Logical negation.
    Not {
        /// Child condition.
        condition: Box<Condition>,
    },
}

pub(super) fn unique_map<'de, D, V>(deserializer: D) -> Result<BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct Entries<V>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>> serde::de::Visitor<'de> for Entries<V> {
        type Value = BTreeMap<String, V>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a normalization map with unique keys")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut map: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry()? {
                if values.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("duplicate normalization map key"));
                }
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(Entries(std::marker::PhantomData))
}
