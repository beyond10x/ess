//! Typed authored recipe; shared verbatim with standalone normalization targets.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Authored behavior envelope, independent of schema and model formats.
pub const FORMAT: &str = "ess-normalization/1";

/// Ordered construction and explicitly declared binary64 input conversion.
pub const FORMAT_V2: &str = "ess-normalization/2";

/// Checked model-owned roots in addition to qualified bundle roots.
pub const FORMAT_V3: &str = "ess-normalization/3";

/// Exact branch names mapped to explicit paths through their external input roots.
pub type Binary64Inputs = BTreeMap<String, Vec<Vec<NumberPath>>>;

/// Complete compiler-minted identity, rechecked against a supplied model selection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelIdentity {
    /// Declared system identity.
    pub system: String,
    /// Declared specification version, not the generator version.
    pub specification_version: String,
    /// Source digest minted by the model projector.
    pub source_digest: String,
    /// Contract digest minted from the selected closure.
    pub contract_digest: String,
    /// Digest of the complete retained model projection.
    pub projection_digest: String,
    /// Explicit root admission, distinct from reachable definitions.
    pub roots: BTreeSet<String>,
}

/// One checked source root, identified without filesystem or network authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Root {
    /// Versions 1 and 2 retain this exact persisted representation.
    Bundle {
        /// SHA-256 of canonical qualified-bundle bytes.
        bundle_digest: String,
        /// Independently selectable bundle root.
        root: String,
    },
    /// Version 3 requires a sealed model projection with this complete identity.
    Model {
        /// Compiler-minted source and selection identity.
        model: ModelIdentity,
        /// Qualified root explicitly admitted by the selection.
        root: String,
    },
}

/// Explicit external dispatch and ordered stages. Construction alone does not validate it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    /// Declared version; model roots require version 3, extended operations version 2 or 3.
    pub format: String,
    /// Exact external discriminator values; no trial decoding or implicit default branch.
    #[serde(deserialize_with = "unique_map")]
    pub branches: BTreeMap<String, Vec<Stage>>,
    /// Version 2: branch-specific numeric paths decoded as binary64 at the input edge.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "number_inputs"
    )]
    pub binary64_inputs: Option<Binary64Inputs>,
}

impl Recipe {
    /// The selected branch's explicit numeric input paths, never inferred from values.
    pub fn binary64_paths(&self, branch: &str) -> &[Vec<NumberPath>] {
        self.binary64_inputs
            .as_ref()
            .and_then(|paths| paths.get(branch))
            .map_or(&[], Vec::as_slice)
    }
}

/// Typed traversal of the external input, with no wildcard object-member guessing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NumberPath {
    /// Traverse one declared object field.
    Field {
        /// Exact wire key.
        name: String,
    },
    /// Traverse each element of a declared array.
    Items,
}

/// One rounded binary64 operation, in source order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Binary64Step {
    /// Multiply by a finite JSON numeric token, rounded to binary64.
    Multiply {
        /// Unrounded authored JSON numeric token.
        value: String,
    },
    /// Clamp to an upper bound; negative zero wins over positive zero.
    Minimum {
        /// Unrounded authored JSON numeric token.
        value: String,
    },
    /// Clamp to a lower bound; positive zero wins over negative zero.
    Maximum {
        /// Unrounded authored JSON numeric token.
        value: String,
    },
}

/// A floating-to-integer range policy, distinct from integer arithmetic overflow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Binary64Range {
    /// Refuse values outside [-2^63, 2^63); do not emulate a host-dependent cast.
    Reject,
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
    /// Version 2: concatenate present strings without interpretation or normalization.
    Concat {
        /// String expressions in their required evaluation order.
        parts: Vec<Expr>,
    },
    /// Version 2: join an ordered string list, preserving duplicate and empty entries.
    Join {
        /// List of strings.
        list: Box<Expr>,
        /// Exact separator between entries, never before or after the list.
        separator: Box<Expr>,
    },
    /// Version 2: ordinary base-10 signed-integer text without locale or padding.
    IntegerString {
        /// An exact signed-64-bit integer.
        value: Box<Expr>,
    },
    /// Version 2: concatenate lists without flattening their individual elements.
    ConcatLists {
        /// List-valued expressions evaluated in order.
        lists: Vec<Expr>,
    },
    /// Version 2: original zero-based index of the innermost collection item.
    ItemIndex,
    /// Version 2: ordered filtering and mapping with original indices retained.
    SelectMap {
        /// Input list.
        list: Box<Expr>,
        /// Selects an item before its value expression is evaluated.
        condition: Box<Condition>,
        /// Required output value for each selected item.
        value: Box<Expr>,
    },
    /// Version 2: evaluate the first matching item, or an enclosing-scope fallback.
    Find {
        /// Input list.
        list: Box<Expr>,
        /// Predicate evaluated in item/index scope.
        condition: Box<Condition>,
        /// Required value of the first matching item.
        value: Box<Expr>,
        /// Required fallback in the enclosing scope when no item matches.
        otherwise: Box<Expr>,
    },
    /// Version 2: decode, apply rounded binary64 steps, then truncate toward zero.
    Binary64ToInteger {
        /// Required numeric expression, explicitly converted to binary64.
        value: Box<Expr>,
        /// Operations applied in exact declaration order.
        steps: Vec<Binary64Step>,
        /// Required range policy for the final integer conversion.
        out_of_range: Binary64Range,
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

fn number_inputs<'de, D>(deserializer: D) -> Result<Option<Binary64Inputs>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    unique_map(deserializer).map(Some)
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
