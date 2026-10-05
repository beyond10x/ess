//! The predicate language.
//!
//! Predicates are the only way AEP asks a question about the world. A transition, a
//! completion condition, a principle's applicability and an obligation are all predicates
//! over [facts](crate::facts).
//!
//! # Three-valued logic
//!
//! Evaluation is Kleene three-valued: [`Truth::True`], [`Truth::False`] and
//! [`Truth::Unknown`]. The third value is the point of the design. `tests.unit.failed == 0`
//! is *false* when a suite failed and *unknown* when nothing ran, and those two situations
//! need different responses from a harness: fix the code, or go run the tests. A predicate
//! only permits a transition when it is `True`, so unknown never advances a workflow.
//!
//! # Syntax
//!
//! Both a compact string form and a structured form parse to the same value:
//!
//! ```yaml
//! # string form
//! - tests.unit.failed == 0
//! - error_rate < service.slo.error_threshold
//! - recovery_verified == true
//! - specification.satisfied              # bare path: the fact is present and truthy
//! - defined(deployment.previous_revision)
//! - not artifact.design.superseded
//!
//! # structured form
//! all:
//!   - any: [service.health == healthy, service.health == degraded]
//!   - not: change.architectural
//!   - task.kind: {any_of: [feature, bugfix]}
//!   - risk: {gte: medium}
//!   - change.architectural: true
//! ```
//!
//! A bare list is an implicit `all`. On the right-hand side of a comparison, a bare word
//! containing a dot is read as a fact path and anything else as a literal; quote a literal
//! that contains dots (`version == "1.2.3"`).
//!
//! A one-segment fact on the right — `depends_on` in a guard that compares two inputs — cannot be
//! told from the text it is spelled like without knowing what the place it is written in declares,
//! so the canonical form writes it as an explicit operand, `task_id: {eq: {fact: depends_on}}`, and
//! this reader takes that mapping as the fact. Which bare words of an authored source are such facts
//! is decided by the source format and its declarations, in `ess-domain`, never here
//! (`docs/design/expression-family-source22.md`, A1). [`Predicate::from_node_spelled`] keeps the one
//! bit that decision needs: which right-hand sides were written as an unquoted, undotted word.
//!
//! One fact moved by one constant — `upper == lower + 5`, `expires_at <= issued_at - 24h` — is
//! [`Operand::Offset`] (A2), canonically `upper: {eq: {offset: {fact: lower, add: 5}}}`. The compact
//! spelling is text to this reader for the same reason a bare word is, and
//! [`Spelled::offsets`] marks where it was written unquoted.
//!
//! The UTF-8 byte length of a text — `label.utf8_bytes <= 255` — is [`Operand::Derived`], canonically
//! `{compare: {left: {utf8_bytes: label}, op: lte, right: 255}}`, or `limit: {gte: {utf8_bytes:
//! label}}` on the right. This reader reads `label.utf8_bytes` as the path it is spelled like: only
//! the source format and the declarations say there is no member of that name, in `ess-domain`.
//!
//! # Quantifiers
//!
//! Everything above asks about one value. A claim about a *collection* — every element, or some
//! element — needs a quantifier, and there is no way to write one by nesting the forms above: a
//! predicate that reads `slots.0.matched` and `slots.1.matched` is a claim about a collection of
//! exactly two, which is not what anybody meant.
//!
//! ```yaml
//! forall:
//!   in: slots            # the collection, a fact path
//!   as: slot             # the name the body binds each element to
//!   that: slot.left.count >= 0
//!
//! exists:
//!   in: pairs
//!   as: pair
//!   that:
//!     all:
//!       - pair.left == input.agent_id
//!       - pair.right != ""
//! ```
//!
//! **There is no string form.** Implication has none either, for the same reason: the compact
//! syntax is one scalar, and a binder introduces a scope that a scalar cannot delimit without
//! becoming a second grammar to keep in step with this one.
//!
//! A collection publishes its size as `<path>.count` and its elements as `<path>.0.…` — see
//! [`FactSource::cardinality`]. An unobserved collection evaluates to [`Truth::Unknown`], never to
//! the vacuous truth an empty one would give: *nobody looked* and *there was nothing to look at*
//! are the two situations three-valued logic exists to keep apart.

use std::cmp::Ordering;
use std::fmt;

use crate::error::ParseError;
use crate::facts::{FactPath, FactSource, FactValue, Scales};
use crate::node::Node;

std::thread_local! {
    /// Whether the `ess/22` operand grammar is read: `{fact: <path>}` on the right of a comparison
    /// and the tagged `{compare: …}` form (`docs/design/expression-family-source22.md`). A parser
    /// that knows its source is older turns it off with [`reading_source22_operands`], and the
    /// mapping is then refused as the non-scalar it always was. With no format known it is read,
    /// and assembly refuses it below `ess/22`.
    static SOURCE22_OPERANDS: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Runs `read` with the `ess/22` operand grammar read or not, restoring what was set before.
pub fn reading_source22_operands<T>(admitted: bool, read: impl FnOnce() -> T) -> T {
    let before = SOURCE22_OPERANDS.with(|cell| cell.replace(admitted));
    let value = read();
    SOURCE22_OPERANDS.with(|cell| cell.set(before));
    value
}

/// Whether the `ess/22` operand grammar is read here; see [`reading_source22_operands`].
fn source22_operands() -> bool {
    SOURCE22_OPERANDS.with(std::cell::Cell::get)
}

/// Whether the source being read admits the `ess/22` grammar: on, unless a parser that knows its
/// source is older turned it off with [`reading_source22_operands`]. Read by the value-source
/// parser too, so an `ess/22` input path (`{input: a.b, else: input.c}`) is admitted where the
/// predicate grammar is (`docs/design/expression-family-source22.md`, A4).
pub fn reads_source22_operands() -> bool {
    source22_operands()
}

/// The result of evaluating a predicate.
///
/// `Unknown` means no observation has been made yet; it is distinct from `False`, which means
/// an observation contradicts the predicate.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Truth {
    /// Observed to hold.
    True,
    /// Observed not to hold.
    False,
    /// Not yet observable: a referenced fact has no value.
    Unknown,
}

impl Truth {
    /// Kleene conjunction: `False` dominates, then `Unknown`.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::True, Self::True) => Self::True,
        }
    }

    /// Kleene disjunction: `True` dominates, then `Unknown`.
    #[must_use]
    pub fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::False, Self::False) => Self::False,
        }
    }

    /// Kleene negation: `Unknown` negates to itself.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }

    /// `true` only for [`Truth::True`]; what a transition requires.
    pub fn is_satisfied(self) -> bool {
        self == Self::True
    }

    /// The truth value as it appears in output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::True => "true",
            Self::False => "false",
            Self::Unknown => "unknown",
        }
    }

    /// Builds a truth value from a boolean observation.
    pub fn from_bool(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

impl fmt::Display for Truth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A comparison operator.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    /// `==`
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
}

impl CompareOp {
    /// The operator as written in a predicate expression.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }

    /// The keyword the canonical writer uses in a structured comparison: `eq`, `ne`, `lt`, `lte`,
    /// `gt` or `gte`. [`Self::from_keyword`] reads each of them back.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Lt => "lt",
            Self::Le => "lte",
            Self::Gt => "gt",
            Self::Ge => "gte",
        }
    }

    /// `true` when this operator needs an ordering, not just equality.
    pub fn needs_ordering(self) -> bool {
        !matches!(self, Self::Eq | Self::Ne)
    }

    /// Parses the map-form spelling of an operator, such as `gte`.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "eq" | "equals" | "==" => Some(Self::Eq),
            "ne" | "not_equals" | "!=" => Some(Self::Ne),
            "lt" | "<" => Some(Self::Lt),
            "le" | "lte" | "<=" => Some(Self::Le),
            "gt" | ">" => Some(Self::Gt),
            "ge" | "gte" | ">=" => Some(Self::Ge),
            _ => None,
        }
    }

    /// Applies this operator to an ordering.
    fn accepts(self, ordering: Ordering) -> bool {
        match self {
            Self::Eq => ordering == Ordering::Equal,
            Self::Ne => ordering != Ordering::Equal,
            Self::Lt => ordering == Ordering::Less,
            Self::Le => ordering != Ordering::Greater,
            Self::Gt => ordering == Ordering::Greater,
            Self::Ge => ordering != Ordering::Less,
        }
    }
}

impl fmt::Display for CompareOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A string operator: a text fact tested against a text literal (beyond10x/ess#95).
///
/// Map form only (`caller: {starts_with: "+44"}`), with one spelling each and no negated one:
/// `not:` negates. Matching is byte-wise and case-sensitive, with no Unicode normalisation, which is
/// what every lane's standard library answers. `docs/design/string-predicate-operators.md` is the
/// design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextOp {
    /// `starts_with`: the text begins with the bytes of the literal.
    StartsWith,
    /// `ends_with`: the text ends with the bytes of the literal.
    EndsWith,
    /// `contains`: the bytes of the literal occur in the text.
    Contains,
}

impl TextOp {
    /// Every string operator, in the order the design names them.
    pub const ALL: [Self; 3] = [Self::StartsWith, Self::EndsWith, Self::Contains];

    /// The map-form key.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::StartsWith => "starts_with",
            Self::EndsWith => "ends_with",
            Self::Contains => "contains",
        }
    }

    /// Parses the map-form key. There is exactly one spelling of each.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|op| op.keyword() == keyword)
    }

    /// Whether `text` begins with, ends with or contains `literal`, byte for byte.
    ///
    /// `str::contains(&str)` matches bytes: a valid UTF-8 needle can only match a valid UTF-8
    /// haystack at a character boundary, so byte and character matching agree.
    pub fn holds(self, text: &str, literal: &str) -> bool {
        match self {
            Self::StartsWith => text.as_bytes().starts_with(literal.as_bytes()),
            Self::EndsWith => text.as_bytes().ends_with(literal.as_bytes()),
            Self::Contains => text.contains(literal),
        }
    }
}

impl fmt::Display for TextOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.keyword())
    }
}

/// Where a typed text operand reads its value (beyond10x/ess#200): a view's parameter or a
/// command's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextNamespace {
    /// `{param: <name>}`: a parameter the view's caller sends.
    Param,
    /// `{input: <name>}`: a field of the command's input.
    Input,
}

impl TextNamespace {
    /// Both namespaces, in the order the design names them.
    pub const ALL: [Self; 2] = [Self::Param, Self::Input];

    /// The mapping key, and the namespace a read under it is rooted at before resolution.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Param => "param",
            Self::Input => "input",
        }
    }

    /// Parses the mapping key.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|namespace| namespace.keyword() == keyword)
    }
}

impl fmt::Display for TextNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.keyword())
    }
}

/// The right side of a string operator (beyond10x/ess#200): a text literal, read byte for byte, or
/// one declared top-level parameter or input, `{param: <name>}` / `{input: <name>}`.
///
/// `path` is the fact the evaluator reads, which depends on where the predicate sits: a view's
/// filter reads `param.<name>`, a stored row's predicate `input.<name>`, and a command's plain
/// `when:` the input root `<name>` itself, as the domain resolver rewrites it. The canonical form
/// writes only the namespace and the name, so it reads the same wherever it is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextOperand {
    /// The literal, verbatim as written: never a fact path, never read as a number.
    Literal(FactValue),
    /// A parameter or an input, by its declared top-level name.
    Fact {
        /// Which namespace the name is declared in.
        namespace: TextNamespace,
        /// The declared name.
        name: String,
        /// The fact read for it where the predicate sits.
        path: FactPath,
    },
}

impl TextOperand {
    /// The operand `{namespace: name}`, read at `namespace.name` until a resolver places it.
    ///
    /// # Errors
    ///
    /// When `name` is not one fact-path segment.
    pub fn fact(namespace: TextNamespace, name: &str) -> Result<Self, ParseError> {
        let single = FactPath::new(name)?;
        if single.segments().len() != 1 {
            return Err(ParseError::predicate(
                &format!("{{{namespace}: {name}}}"),
                TEXT_OPERAND_SHAPE,
            ));
        }
        let path = FactPath::from_segments([namespace.keyword(), name]);
        Ok(Self::Fact {
            namespace,
            name: name.to_owned(),
            path,
        })
    }

    /// The literal, where this is one.
    pub fn as_literal(&self) -> Option<&FactValue> {
        match self {
            Self::Literal(value) => Some(value),
            Self::Fact { .. } => None,
        }
    }

    /// The fact this operand reads, where it reads one.
    pub fn fact_path(&self) -> Option<&FactPath> {
        match self {
            Self::Literal(_) => None,
            Self::Fact { path, .. } => Some(path),
        }
    }

    /// This operand with its fact read at `map(path)`; a literal is unchanged.
    #[must_use]
    pub fn map_path(&self, map: impl FnOnce(&FactPath) -> FactPath) -> Self {
        match self {
            Self::Literal(value) => Self::Literal(value.clone()),
            Self::Fact {
                namespace,
                name,
                path,
            } => Self::Fact {
                namespace: *namespace,
                name: name.clone(),
                path: map(path),
            },
        }
    }

    /// The canonical document form: the literal as written, or `{param: <name>}`.
    pub fn to_node(&self) -> Node {
        match self {
            Self::Literal(value) => value_node(value),
            Self::Fact {
                namespace, name, ..
            } => Node::Map([(namespace.keyword().to_owned(), Node::Text(name.clone()))].into()),
        }
    }

    /// The value this operand compares with: the literal, or what `facts` observe at its path.
    fn resolve(&self, facts: &dyn FactSource) -> Option<FactValue> {
        match self {
            Self::Literal(value) => Some(value.clone()),
            Self::Fact { path, .. } => facts.observe(path),
        }
    }
}

impl From<FactValue> for TextOperand {
    fn from(value: FactValue) -> Self {
        Self::Literal(value)
    }
}

/// For a reader and the semantic diff, never read back: a text literal is quoted by `Debug`, a
/// number or a Boolean is bare, and a parameter or an input is its mapping.
impl fmt::Display for TextOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(FactValue::Text(text)) => write!(f, "{text:?}"),
            Self::Literal(value) => write!(f, "{value}"),
            Self::Fact {
                namespace, name, ..
            } => write!(f, "{{{namespace}: {name}}}"),
        }
    }
}

/// What a string operator's mapping operand must be.
const TEXT_OPERAND_SHAPE: &str = "a comparison operand must be a scalar, or for a string \
     operator `{param: <name>}` naming one view parameter or `{input: <name>}` naming one command \
     input: exactly one key and one top-level name";

/// A case-insensitive text operator (beyond10x/ess#140): a text fact equal, under ASCII case
/// folding, to one text literal or to one of a list of them.
///
/// Map form only (`source: {equals_ignore_case: web}`, `source: {in_ignore_case: [web, phone]}`).
/// Folding is ASCII only — `A`–`Z` fold to `a`–`z` and every other byte compares as itself — so
/// every evaluator lane answers it from its standard library the same way, which Unicode case
/// folding would not. `docs/design/value-expressions.md` § E7 is the design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FoldOp {
    /// `equals_ignore_case`: one text literal.
    EqualsIgnoreCase,
    /// `in_ignore_case`: a list of text literals.
    InIgnoreCase,
}

impl FoldOp {
    /// Both operators, in the order the design names them.
    pub const ALL: [Self; 2] = [Self::EqualsIgnoreCase, Self::InIgnoreCase];

    /// The map-form key.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::EqualsIgnoreCase => "equals_ignore_case",
            Self::InIgnoreCase => "in_ignore_case",
        }
    }

    /// Parses the map-form key. There is exactly one spelling of each.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|op| op.keyword() == keyword)
    }

    /// Whether `text` equals any of `literals` under ASCII case folding.
    ///
    /// `str::eq_ignore_ascii_case` is exactly the design's rule: equal lengths, and each byte pair
    /// equal after mapping `A`–`Z` to `a`–`z`. A non-ASCII byte maps to itself, so `É` and `é`
    /// differ, and so do `@` and `` ` ``, which a folding that set bit 5 would confuse.
    pub fn holds(self, text: &str, literals: &[&str]) -> bool {
        literals
            .iter()
            .any(|literal| text.eq_ignore_ascii_case(literal))
    }
}

impl fmt::Display for FoldOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.keyword())
    }
}

/// One side of a comparison: either a fact to look up, or a literal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(untagged)]
pub enum Operand {
    /// A fact path, resolved against the fact source at evaluation time.
    Fact(FactPath),
    /// A constant written in the document.
    Literal(FactValue),
    /// One fact moved by one constant, `lower + 5` or `issued_at - 24h`
    /// (`docs/design/expression-family-source22.md`, A2). Legal on the right of a comparison only.
    Offset(OffsetOperand),
    /// A value derived from a fact rather than read at its path: the UTF-8 byte length of a text,
    /// `{utf8_bytes: label}` (`docs/design/expression-family-source22.md`, "String `.utf8_bytes`",
    /// final review decision 11). Legal as either operand of a comparison, and nowhere else.
    Derived(Derived),
}

/// A value an evaluator derives from the fact at a path, never reads at a path of its own.
///
/// The resolved predicate carries it as this closed operand rather than as the path
/// `label.utf8_bytes`, so a declared member named `utf8_bytes` keeps meaning that member and no
/// evaluator decides what a path is by its last segment. Its canonical form is the one-key mapping
/// `{utf8_bytes: <parent>}`, which suite `/40` and specification format `ess/22` introduce.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Derived {
    /// The number of bytes the UTF-8 encoding of the text at the parent path takes, an `Integer`.
    ///
    /// A bound full-path observation, `<parent>.utf8_bytes`, wins over the parent — the rule a
    /// text's `.count` follows (`FactSource::observe`) — but, unlike a `.count`, only a whole
    /// number from zero is a byte length: any other bound value is `Unknown`, with no fallback to
    /// the parent. Otherwise the parent's text is measured; an unobserved parent, an absent
    /// `Optional` and a value that is no text are `Unknown`. Empty text is zero, and no Unicode
    /// normalization occurs. A text holding a lone surrogate is no Unicode text and its byte length
    /// is `Unknown` in every lane; a Rust `str` cannot hold one, so here it never arrives.
    Utf8Bytes(FactPath),
}

impl Derived {
    /// The key of the canonical mapping `{utf8_bytes: <parent>}`, and the path segment a source
    /// format reads it from (`label.utf8_bytes`).
    pub const UTF8_BYTES: &'static str = "utf8_bytes";

    /// The fact this value is derived from.
    pub fn parent(&self) -> &FactPath {
        match self {
            Self::Utf8Bytes(parent) => parent,
        }
    }

    /// The same derivation of another fact.
    #[must_use]
    pub fn with_parent(&self, parent: FactPath) -> Self {
        match self {
            Self::Utf8Bytes(_) => Self::Utf8Bytes(parent),
        }
    }

    /// The path a full-path observation of this value is bound at: `<parent>.utf8_bytes`.
    pub fn observed_at(&self) -> FactPath {
        match self {
            Self::Utf8Bytes(parent) => parent.child(Self::UTF8_BYTES),
        }
    }

    /// The value this derivation reads from `facts`, or `None` where it is `Unknown`.
    pub fn value(&self, facts: &dyn FactSource) -> Option<FactValue> {
        self.value_with(&|path| facts.fact(path))
    }

    /// [`Self::value`], reading each fact through `fact`: what a synthesizer writes in for a derived
    /// operand whose parent it already knows. One rule for both, so a witness never measures text
    /// differently from the evaluator that decides it.
    pub fn value_with(&self, fact: &dyn Fn(&FactPath) -> Option<FactValue>) -> Option<FactValue> {
        match self {
            Self::Utf8Bytes(parent) => {
                if let Some(bound) = fact(&self.observed_at()) {
                    let length = bound.as_number()?;
                    return (length.as_i64()? >= 0).then_some(FactValue::Number(length));
                }
                match fact(parent)? {
                    FactValue::Text(text) => Some(FactValue::count(text.len())),
                    FactValue::Bool(_) | FactValue::Number(_) => None,
                }
            }
        }
    }

    /// The canonical mapping, `{utf8_bytes: <parent>}`.
    pub fn to_node(&self) -> Node {
        match self {
            Self::Utf8Bytes(parent) => {
                Node::Map([(Self::UTF8_BYTES.to_owned(), Node::Text(parent.to_string()))].into())
            }
        }
    }
}

impl fmt::Display for Derived {
    /// The canonical mapping in one line, never the path `label.utf8_bytes`: that spelling names a
    /// declared member wherever there is one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8Bytes(parent) => write!(f, "{{{}: {parent}}}", Self::UTF8_BYTES),
        }
    }
}

impl serde::Serialize for Derived {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_node().serialize(serializer)
    }
}

/// Which way an offset moves its base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OffsetDirection {
    /// `+`, written `add`.
    Add,
    /// `-`, written `subtract`.
    Subtract,
}

impl OffsetDirection {
    /// The key the canonical mapping writes the magnitude under.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Subtract => "subtract",
        }
    }

    /// The sign the compact spelling writes.
    pub fn symbol(self) -> char {
        match self {
            Self::Add => '+',
            Self::Subtract => '-',
        }
    }
}

/// The constant an offset moves its base by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetMagnitude {
    /// A whole number, from zero to `i64::MAX`, added to or taken from an `Integer`. The sum is the
    /// exact mathematical integer, never wrapped, clamped, rounded or turned `Unknown`.
    Integer(crate::facts::Number),
    /// Elapsed UTC seconds added to or taken from a `Timestamp`, with the unit they were written in.
    ElapsedSeconds {
        /// How many seconds, from zero to [`crate::time::CurrentTime::MAX_OFFSET_SECONDS`].
        seconds: i64,
        /// `s`, `m` or `h`, as written.
        written_unit: crate::time::ElapsedUnit,
    },
}

impl OffsetMagnitude {
    /// Reads a magnitude as a source writes it: a whole number without a sign, fraction or
    /// leading zero that fits `i64` (an Integer magnitude), or the same digits followed by `s`, `m`
    /// or `h` under the current-time bound (an elapsed one). `None` for anything else.
    pub fn parse(text: &str) -> Option<Self> {
        if let Some((seconds, written_unit)) = crate::time::ElapsedUnit::parse_magnitude(text) {
            return Some(Self::ElapsedSeconds {
                seconds,
                written_unit,
            });
        }
        if text.is_empty()
            || !text.bytes().all(|byte| byte.is_ascii_digit())
            || (text.len() > 1 && text.starts_with('0'))
        {
            return None;
        }
        text.parse::<i64>()
            .ok()
            .map(|value| Self::Integer(crate::facts::Number::from(value)))
    }

    /// The Integer magnitude as an `i64`, where it is a whole number from zero to `i64::MAX`.
    pub fn integer(self) -> Option<i64> {
        match self {
            Self::Integer(number) => number.as_i64().filter(|value| *value >= 0),
            Self::ElapsedSeconds { .. } => None,
        }
    }

    /// The canonical node: the number, or the elapsed text in its written unit (`24h`).
    fn node(self) -> Node {
        match self {
            Self::Integer(number) => Node::Number(number),
            Self::ElapsedSeconds { .. } => Node::Text(self.to_string()),
        }
    }

    /// Reads the canonical node back: a number that is a whole number from zero to `i64::MAX`, or
    /// an elapsed text. A whole number written as text is refused: the canonical form writes it
    /// as a number.
    fn from_node(node: &Node) -> Option<Self> {
        match node {
            Node::Number(number) => number
                .as_i64()
                .filter(|value| *value >= 0)
                .map(|value| Self::Integer(crate::facts::Number::from(value))),
            Node::Text(text) => match Self::parse(text)? {
                elapsed @ Self::ElapsedSeconds { .. } => Some(elapsed),
                Self::Integer(_) => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for OffsetMagnitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(number) => write!(f, "{}", number.exact_text()),
            Self::ElapsedSeconds {
                seconds,
                written_unit,
            } => write!(
                f,
                "{}{}",
                seconds / written_unit.seconds(),
                written_unit.letter()
            ),
        }
    }
}

/// One fact moved by one constant: `lower + 5`, `issued_at - 24h`
/// (`docs/design/expression-family-source22.md`, A2).
///
/// Its canonical form is the closed mapping `{offset: {fact: <path>, add|subtract: <magnitude>}}`,
/// which suite `/40` and specification format `ess/22` introduce. No offset nests: the base is one
/// fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffsetOperand {
    /// The fact moved.
    pub base: FactPath,
    /// Which way.
    pub direction: OffsetDirection,
    /// By how much.
    pub magnitude: OffsetMagnitude,
}

impl OffsetOperand {
    /// Every way an unquoted right side splits into `<base> ws? (+|-) ws? <magnitude>`, left to
    /// right: one per `+` or `-` whose text before it, without the spaces beside the sign, is a
    /// fact path, with the text after it. Whether the base resolves and the magnitude reads is
    /// for the caller to decide (final review decision 4, rule 3a): `my-field-5` splits after
    /// `my` and after `my-field`.
    pub fn spellings(text: &str) -> Vec<(FactPath, OffsetDirection, &str)> {
        let mut found = Vec::new();
        for (at, byte) in text.bytes().enumerate() {
            let direction = match byte {
                b'+' => OffsetDirection::Add,
                b'-' => OffsetDirection::Subtract,
                _ => continue,
            };
            let base = text[..at].trim_end_matches(' ');
            if base.is_empty() {
                continue;
            }
            if let Ok(path) = FactPath::new(base) {
                found.push((path, direction, text[at + 1..].trim_start_matches(' ')));
            }
        }
        found
    }

    /// The same offset the other way: `base - k` for `base + k`. Where `left == base + k`,
    /// `base == left - k`: the offset that reads the base back from the left side.
    #[must_use]
    pub fn reversed(&self, base: FactPath) -> Self {
        Self {
            base,
            direction: match self.direction {
                OffsetDirection::Add => OffsetDirection::Subtract,
                OffsetDirection::Subtract => OffsetDirection::Add,
            },
            magnitude: self.magnitude,
        }
    }

    /// `base ± magnitude` for an Integer magnitude, exactly: every sum of two `i64`s fits `i128`.
    /// `None` for an elapsed magnitude, or a magnitude that is not a whole number in `i64`.
    pub fn integer_at(&self, base: i64) -> Option<i128> {
        let magnitude = i128::from(self.magnitude.integer()?);
        Some(match self.direction {
            OffsetDirection::Add => i128::from(base) + magnitude,
            OffsetDirection::Subtract => i128::from(base) - magnitude,
        })
    }

    /// `base ± seconds` for an elapsed magnitude, through the arithmetic the current time shares
    /// ([`crate::time::Rfc3339Instant::plus_elapsed`]): `None` for an Integer magnitude, or where
    /// the instant is past the years a `date-time` spells.
    pub fn instant_at(
        &self,
        base: crate::time::Rfc3339Instant,
    ) -> Option<crate::time::Rfc3339Instant> {
        let OffsetMagnitude::ElapsedSeconds { seconds, .. } = self.magnitude else {
            return None;
        };
        match self.direction {
            OffsetDirection::Add => base.plus_elapsed(seconds),
            OffsetDirection::Subtract => base.plus_elapsed(seconds.checked_neg()?),
        }
    }

    /// The value this offset names where its base holds `base`: the exact sum where it is an
    /// `i64`, the moved instant in UTC, and `None` where the base is of the wrong kind or the
    /// value is past what a stored `Integer` or a `date-time` holds. What a synthesizer writes in
    /// for an offset whose base it already knows; evaluation never goes through it.
    pub fn value_at(&self, base: &FactValue) -> Option<FactValue> {
        match self.magnitude {
            OffsetMagnitude::Integer(_) => {
                let sum = self.integer_at(base.as_number()?.as_i64()?)?;
                i64::try_from(sum)
                    .ok()
                    .map(|value| FactValue::Number(crate::facts::Number::from(value)))
            }
            OffsetMagnitude::ElapsedSeconds { .. } => {
                let instant = crate::time::Rfc3339Instant::parse_rfc3339(base.as_text()?)?;
                self.instant_at(instant)
                    .map(|moved| FactValue::Text(moved.to_rfc3339()))
            }
        }
    }

    /// The canonical mapping, `{offset: {fact: <path>, add|subtract: <magnitude>}}`.
    pub fn to_node(&self) -> Node {
        Node::Map(
            [(
                "offset".to_owned(),
                Node::Map(
                    [
                        ("fact".to_owned(), Node::Text(self.base.to_string())),
                        (self.direction.keyword().to_owned(), self.magnitude.node()),
                    ]
                    .into(),
                ),
            )]
            .into(),
        )
    }

    /// Reads the inside of `{offset: …}`: exactly `fact` and one of `add` and `subtract`.
    fn from_entries(fields: &std::collections::BTreeMap<String, Node>) -> Option<Self> {
        if fields.len() != 2 {
            return None;
        }
        let Some(Node::Text(base)) = fields.get("fact") else {
            return None;
        };
        let base = FactPath::new(base).ok()?;
        let (direction, magnitude) = match (fields.get("add"), fields.get("subtract")) {
            (Some(magnitude), None) => (OffsetDirection::Add, magnitude),
            (None, Some(magnitude)) => (OffsetDirection::Subtract, magnitude),
            _ => return None,
        };
        Some(Self {
            base,
            direction,
            magnitude: OffsetMagnitude::from_node(magnitude)?,
        })
    }
}

impl fmt::Display for OffsetOperand {
    /// The canonical mapping in one line, never the compact `lower + 5`: that spelling reads back
    /// as text wherever no declaration says `lower` is a fact.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{offset: {{fact: {}, {}: {}}}}}",
            self.base,
            self.direction.keyword(),
            self.magnitude
        )
    }
}

impl serde::Serialize for OffsetOperand {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_node().serialize(serializer)
    }
}

impl Operand {
    /// Resolves this operand, returning `None` when a referenced fact is unobserved. An offset has
    /// no value of its own: [`Predicate::evaluate_offset`] compares with it.
    fn resolve(&self, facts: &dyn FactSource) -> Option<FactValue> {
        match self {
            Self::Fact(path) => facts.observe(path),
            Self::Literal(value) => Some(value.clone()),
            Self::Offset(_) => None,
            Self::Derived(derived) => derived.value(facts),
        }
    }

    /// The fact path this operand reads, if any: an offset reads its base, and a derived value the
    /// fact it is derived from.
    pub fn fact_path(&self) -> Option<&FactPath> {
        match self {
            Self::Fact(path) => Some(path),
            Self::Offset(offset) => Some(&offset.base),
            Self::Derived(derived) => Some(derived.parent()),
            Self::Literal(_) => None,
        }
    }

    /// This operand with the path it reads — a fact, or an offset's base — moved by `map`; a
    /// literal is kept. What every rewrite of a predicate's reads uses, so an offset's base is
    /// never left behind.
    #[must_use]
    pub fn map_path(&self, map: impl FnOnce(&FactPath) -> FactPath) -> Self {
        match self {
            Self::Fact(path) => Self::Fact(map(path)),
            Self::Offset(offset) => Self::Offset(OffsetOperand {
                base: map(&offset.base),
                direction: offset.direction,
                magnitude: offset.magnitude,
            }),
            Self::Derived(derived) => Self::Derived(derived.with_parent(map(derived.parent()))),
            Self::Literal(value) => Self::Literal(value.clone()),
        }
    }

    /// Whether `raw`, read by [`Self::parse_in`] under `binders`, is unquoted text with a `+` or a
    /// `-` after a fact path: a text literal today, and the spelling a source format may read as
    /// an offset (`docs/design/expression-family-source22.md`, A2, rule 3a).
    ///
    /// A dotted spelling with an unspaced `-` — `window.lower-5` — reads as a fact path here, because
    /// `-` may stand inside a segment; it is marked too, so a format whose declarations name no such
    /// field may read it as the offset it also spells.
    fn is_offset_spelling(raw: &str, binders: &[String]) -> bool {
        let unquoted = !raw.trim().starts_with(['"', '\'']);
        match Self::parse_in(raw, binders) {
            Self::Literal(FactValue::Text(text)) => {
                unquoted && !OffsetOperand::spellings(&text).is_empty()
            }
            Self::Fact(path) => {
                unquoted
                    && path.segments().len() > 1
                    && !OffsetOperand::spellings(&path.to_string()).is_empty()
            }
            _ => false,
        }
    }

    /// Parses an operand as written on the right-hand side of a comparison.
    ///
    /// A bare word containing a dot is a fact path; everything else is a literal. Quote a
    /// literal that contains dots.
    fn parse(raw: &str) -> Self {
        Self::parse_in(raw, &[])
    }

    /// [`Self::parse`] inside quantifier bodies whose binders are `binders`, innermost last.
    ///
    /// An unquoted word that is exactly the name of a binder in scope reads that binder
    /// (beyond10x/ess#289): `a != b` under `forall b` compares two elements, not `a` with the text
    /// `"b"`. Quoted, it is the text, which validation refuses as it refuses a field's name.
    fn parse_in(raw: &str, binders: &[String]) -> Self {
        let trimmed = raw.trim();
        let quoted = trimmed.starts_with('"') || trimmed.starts_with('\'');
        if !quoted && binders.iter().any(|binder| binder == trimmed) {
            if let Ok(path) = FactPath::new(trimmed) {
                return Self::Fact(path);
            }
        }
        if !quoted && trimmed.contains('.') && trimmed.parse::<f64>().is_err() {
            if let Ok(path) = FactPath::new(trimmed) {
                return Self::Fact(path);
            }
        }
        Self::Literal(FactValue::parse_literal(trimmed))
    }

    /// Whether `raw`, read by [`Self::parse_in`] under `binders`, is an unquoted, undotted word: a
    /// text literal today, and the one spelling a source format may read as a root fact instead
    /// (`docs/design/expression-family-source22.md`, A1). A quoted text, a Boolean, a number, a
    /// dotted path, a binder and text that is no fact-path segment are not words.
    fn is_word(raw: &str, binders: &[String]) -> bool {
        match Self::parse_in(raw, binders) {
            Self::Literal(FactValue::Text(text)) => {
                let trimmed = raw.trim();
                !trimmed.starts_with(['"', '\''])
                    && FactPath::new(&text).is_ok_and(|path| path.segments().len() == 1)
            }
            _ => false,
        }
    }

    /// The fact this operand reads when it is one segment that no binder in `binders` names: the
    /// operand only the explicit `{fact: …}` mapping can spell.
    fn root_fact<'a>(&'a self, binders: &[&str]) -> Option<&'a FactPath> {
        match self {
            Self::Fact(path)
                if path.segments().len() == 1 && !binders.contains(&path.namespace()) =>
            {
                Some(path)
            }
            _ => None,
        }
    }

    /// Reads the explicit fact operand `{fact: <path>}`, the canonical spelling of a fact on the
    /// right of a comparison. Exactly one key; anything else is refused rather than compared as
    /// the mapping it is.
    fn fact_mapping(
        entries: &std::collections::BTreeMap<String, Node>,
        written: impl Fn() -> String,
    ) -> Result<Self, ParseError> {
        // Below `ess/22` a mapping is no operand at all, and is refused in the words it always was.
        if !source22_operands() {
            return Err(ParseError::predicate(
                &written(),
                "a comparison operand must be a scalar",
            ));
        }
        let refuse = || {
            ParseError::predicate(
                &written(),
                "a comparison operand must be a scalar, `{fact: <path>}` naming a fact, \
                 `{offset: {fact: <path>, add|subtract: <magnitude>}}`, or `{utf8_bytes: <path>}` \
                 naming a text",
            )
        };
        match entries.iter().next() {
            Some((key, Node::Text(path))) if entries.len() == 1 && key == "fact" => {
                FactPath::new(path).map(Self::Fact).map_err(|error| {
                    ParseError::predicate(&written(), format!("`{{fact: …}}`: {error}"))
                })
            }
            Some((key, Node::Text(path))) if entries.len() == 1 && key == Derived::UTF8_BYTES => {
                FactPath::new(path)
                    .map(|parent| Self::Derived(Derived::Utf8Bytes(parent)))
                    .map_err(|error| {
                        ParseError::predicate(&written(), format!("`{{utf8_bytes: …}}`: {error}"))
                    })
            }
            Some((key, Node::Map(fields))) if entries.len() == 1 && key == "offset" => {
                OffsetOperand::from_entries(fields)
                    .map(Self::Offset)
                    .ok_or_else(|| {
                        ParseError::predicate(
                            &written(),
                            "`{offset: …}` takes exactly `fact`, a fact path, and one of `add` \
                             and `subtract`: a whole number from 0 to 9223372036854775807, or a \
                             whole number of `s`, `m` or `h` without a leading zero",
                        )
                    })
            }
            _ => Err(refuse()),
        }
    }
}

/// The instant one side of an ordering over a declared `Timestamp` names: an RFC 3339 `date-time`,
/// or — for a literal only — the current-time operand read against the source's clock
/// (beyond10x/ess#171). A fact is never read as `now`: a caller sending the text `now` sent no
/// instant.
fn instant_operand(
    operand: &Operand,
    text: &str,
    facts: &dyn FactSource,
) -> Option<crate::time::Rfc3339Instant> {
    crate::time::Rfc3339Instant::parse_rfc3339(text).or_else(|| match operand {
        Operand::Literal(_) => crate::time::CurrentTime::parse(text)?.at(facts.now()?),
        Operand::Fact(_) | Operand::Offset(_) | Operand::Derived(_) => None,
    })
}

/// A fact source told the current time: every read is `facts`'s, and a `now` operand is read
/// against `now` (beyond10x/ess#171, [`crate::time::CurrentTime`]).
///
/// How an evaluator that decides a command guard at the moment it handles the request supplies the
/// moment. The clock is read by the caller and handed in, so one decision is replayable.
pub struct WithNow<'a> {
    facts: &'a dyn FactSource,
    now: crate::time::Rfc3339Instant,
}

impl<'a> WithNow<'a> {
    /// `facts`, with the current time `now`.
    pub fn new(facts: &'a dyn FactSource, now: crate::time::Rfc3339Instant) -> Self {
        Self { facts, now }
    }
}

impl FactSource for WithNow<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.facts.fact(path)
    }

    fn observe(&self, path: &FactPath) -> Option<FactValue> {
        self.facts.observe(path)
    }

    fn present(&self, path: &FactPath) -> bool {
        self.facts.present(path)
    }

    fn observed_presence(&self, path: &FactPath) -> Option<bool> {
        self.facts.observed_presence(path)
    }

    fn scales(&self) -> &Scales {
        self.facts.scales()
    }

    fn orders_as_instant(&self, path: &FactPath) -> bool {
        self.facts.orders_as_instant(path)
    }

    fn now(&self) -> Option<crate::time::Rfc3339Instant> {
        Some(self.now)
    }

    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        self.facts.orders_text_by_bytes(path)
    }

    fn cardinality(&self, path: &FactPath) -> Option<usize> {
        self.facts.cardinality(path)
    }
}

/// The unquoted operands a compact comparison refuses, because each is YAML's spelling of null.
///
/// Quoted, every one of them is a text like any other: `note == "null"` compares with the four
/// characters, and [`Operand`]'s `Display` quotes them so that the rendering reads back as that.
const NULL_SPELLINGS: &[&str] = &["null", "Null", "NULL", "~"];

/// The tokens a compact comparison refuses in an unquoted operand: the compact form has no
/// conjunction or disjunction, and `sku == A1 && gift` used to compare `sku` with the text
/// `A1 && gift`. Quoted, they are text like any other.
const COMBINATORS: &[&str] = &["&&", "||"];

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fact(path) => write!(f, "{path}"),
            Self::Offset(offset) => write!(f, "{offset}"),
            Self::Derived(derived) => write!(f, "{derived}"),
            Self::Literal(FactValue::Text(text))
                if text.contains('.')
                    || text.is_empty()
                    || NULL_SPELLINGS.contains(&&**text)
                    || text
                        .split_whitespace()
                        .any(|token| COMBINATORS.contains(&token)) =>
            {
                write!(f, "{text:?}")
            }
            Self::Literal(value) => write!(f, "{value}"),
        }
    }
}

/// How deep `all`/`any`/`not` nesting may go before a predicate is refused.
///
/// Both spellings let a document choose the parser's recursion depth. The structured form nests
/// mappings, which `serde_yaml` caps at 128 on its own; the string form is one scalar, so
/// `not not not …` is bounded by nothing but the file size. Measured on this machine, a debug build
/// of [`Predicate::parse_expression`] overflows the 8 MiB main-thread stack between 2 500 and 3 000
/// prefixes and the 2 MiB a spawned worker gets between 600 and 800. A stack overflow aborts the
/// process with no diagnostic, which is the opposite of what this parser does with every other bad
/// input.
///
/// 32 because the deepest predicate anybody writes is an `all` of `any`s of comparisons — three or
/// four — and because `MAX_TYPE_DEPTH` and `ess-domain`'s `WRAPPER_LIMIT` are also 32: one number
/// across the workspace is easier to defend, and to remember, than three. It sits an order of
/// magnitude under the smallest measured floor and well under `serde_yaml`'s 128, so a document
/// nested past it is refused here, with a code and a limit, rather than by the deserializer.
pub const MAX_PREDICATE_DEPTH: usize = 32;

/// A one-level rendering of a node, for an error that must not walk what it is refusing.
///
/// [`Display`] on a [`Node`] recurses, and the node being refused here is by definition the deep
/// one — rendering it to describe it would take exactly the stack the refusal exists to save.
fn shallow(node: &Node) -> String {
    match node {
        Node::Text(text) => text.clone(),
        Node::Seq(items) => format!("a list of {}", items.len()),
        Node::Map(entries) => entries
            .keys()
            .next()
            .map_or_else(|| "an empty mapping".to_owned(), |key| format!("{key}: …")),
        scalar => scalar.type_name().to_owned(),
    }
}

/// How a comparison compares its two operands (`docs/design/expression-family-source22.md`, final
/// review decision 2).
///
/// [`CompareKind::Value`] is every comparison there was before: the evaluator compares by the
/// representation it reads, and orders a declared `Timestamp` by its instant only where the fact
/// source says the path is one. [`CompareKind::Instant`] says so in the predicate itself — both
/// operands are `Timestamp` facts and compare as the instants they name under every operator — so a
/// reader with no declared types (a suite runner over view rows) compares instants, never spellings.
/// The domain's resolver tags it from `ess/22`; its canonical form is
/// `{compare: {left, op, right, as: timestamp}}`, which only suite `/40` and later read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompareKind {
    /// The representation the evaluator reads.
    #[default]
    Value,
    /// The instants two `Timestamp` operands name (`as: timestamp`).
    Instant,
}

impl CompareKind {
    /// The canonical spelling of [`CompareKind::Instant`]'s tag.
    pub const TIMESTAMP: &'static str = "timestamp";
}

/// A condition over facts.
///
/// # Depth
///
/// Every predicate a document can produce is at most [`MAX_PREDICATE_DEPTH`] deep: the two parsers
/// below are the only routes from a document to one, [`serde::Deserialize`] goes through
/// [`Predicate::from_node`], and [`Predicate::all`], [`Predicate::any`] and [`Predicate::not`]
/// simplify rather than deepen — `not not x` is `x`. That is what lets [`Predicate::evaluate`],
/// [`Predicate::outcome`], [`Predicate::fact_paths`], [`Predicate::to_node`],
/// [`Display`](fmt::Display) and the walkers in `ess-domain` recurse without counting.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Predicate {
    /// Always holds. The default where a document omits a condition.
    #[default]
    Always,
    /// Never holds. Useful to disable a transition without deleting it.
    Never,
    /// Every child must hold.
    All(Vec<Predicate>),
    /// At least one child must hold.
    Any(Vec<Predicate>),
    /// The child must not hold.
    Not(Box<Predicate>),
    /// Compares two operands.
    Compare {
        /// Left-hand side.
        left: Operand,
        /// The operator.
        op: CompareOp,
        /// Right-hand side.
        right: Operand,
        /// How the operands compare: by value, or as instants (decision 2).
        kind: CompareKind,
    },
    /// The fact is observed and truthy.
    Truthy(FactPath),
    /// The fact has been observed at all, whatever its value.
    Defined(FactPath),
    /// The fact equals one of the listed values.
    AnyOf {
        /// The fact to read.
        path: FactPath,
        /// Accepted values.
        values: Vec<FactValue>,
    },
    /// The fact equals none of the listed values.
    NoneOf {
        /// The fact to read.
        path: FactPath,
        /// Rejected values.
        values: Vec<FactValue>,
    },
    /// The text fact begins with, ends with or contains a literal (beyond10x/ess#95), or a view
    /// parameter or command input (beyond10x/ess#200).
    ///
    /// Unobserved — the fact, or the parameter or input — is [`Truth::Unknown`]; an observed value
    /// that is not text is [`Truth::False`], and so is any observed value against an operand that
    /// is not text. Validation refuses such an operand, so only an unchecked caller reaches that
    /// row.
    TextMatch {
        /// The fact to read.
        path: FactPath,
        /// Which of the three tests.
        op: TextOp,
        /// What it is tested against: a literal, verbatim as written, or a typed operand.
        value: TextOperand,
    },
    /// The text fact equals a literal, or one of a list of them, under ASCII case folding
    /// (beyond10x/ess#140).
    ///
    /// Unobserved is [`Truth::Unknown`]; an observed value that is not text is [`Truth::False`],
    /// and a literal that is not text matches nothing. Validation refuses such a literal, so only
    /// an unchecked caller reaches that row. [`FoldOp::EqualsIgnoreCase`] carries exactly one
    /// value, which is what keeps the two spellings apart when the predicate is rendered.
    FoldMatch {
        /// The fact to read.
        path: FactPath,
        /// Which of the two spellings.
        op: FoldOp,
        /// The literals, verbatim as written: never a fact path, never read as a number.
        values: Vec<FactValue>,
    },
    /// Every element of a collection satisfies the body.
    ///
    /// Empty holds vacuously; unobserved is [`Truth::Unknown`].
    Forall(Box<Quantified>),
    /// At least one element of a collection satisfies the body.
    ///
    /// Empty does not hold; unobserved is [`Truth::Unknown`].
    Exists(Box<Quantified>),
    /// No two elements of a list share a key (`ess/22`).
    ///
    /// Empty and one element hold; unobserved is [`Truth::Unknown`]. See [`Distinct`].
    Distinct(Box<Distinct>),
    /// An instant falls inside a weekly calendar window at UTC or a fixed offset (`ess/22`,
    /// beyond10x/ess#244, `docs/design/calendar-window-guards.md`).
    ///
    /// An unobserved instant, text that names none, and `now` read without a clock are
    /// [`Truth::Unknown`].
    Window(Box<crate::window::CalendarWindow>),
}

/// A predicate as read from a document, with the one fact about its spelling that the reading
/// itself discards (`docs/design/expression-family-source22.md`, A1).
///
/// `words` holds one flag per comparison, in the order a pre-order walk of `predicate` meets them:
/// `true` where the right-hand side was written as an unquoted, undotted word that no binder in
/// scope names, which the reader keeps as a text literal. A quoted `"b"` and the bare `b` read as the
/// same literal, and only the second may name a root fact under a source format that says so.
/// Nothing here decides that; the domain's resolver does, with the declarations in hand. This is
/// not a predicate and is never persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spelled {
    /// The predicate every reader of this crate reads, with every word a text literal.
    pub predicate: Predicate,
    /// Per comparison, in pre-order: whether its right-hand side was a bare word.
    pub words: Vec<bool>,
    /// Per comparison, in pre-order: whether its right-hand side was unquoted text with a `+` or a
    /// `-` after a fact path, or a dotted path with a `-` in it — the spelling a source format may
    /// read as one constant offset of that fact (`docs/design/expression-family-source22.md`, A2,
    /// rule 3a), and text or the dotted fact otherwise.
    pub offsets: Vec<bool>,
}

/// What the spelled readers record per comparison, in pre-order: [`Spelled::words`] and
/// [`Spelled::offsets`].
#[derive(Default)]
struct Spellings {
    words: Vec<bool>,
    offsets: Vec<bool>,
}

impl Spellings {
    /// One comparison whose right side is not text an author could have meant as a fact.
    fn literal(&mut self) {
        self.words.push(false);
        self.offsets.push(false);
    }

    /// One comparison whose right side was written as `raw` under `binders`.
    fn written(&mut self, raw: &str, binders: &[String]) {
        self.words.push(Operand::is_word(raw, binders));
        self.offsets.push(Operand::is_offset_spelling(raw, binders));
    }
}

/// A quantified claim: a collection, the name its elements are bound to, and the body.
///
/// # Why the binder is part of the value rather than a positional index
///
/// The body could have been written against `element.…` with no binder at all, and one nested
/// inside another would then have no way to reach the outer element. Naming the binder costs one
/// line in the document and makes nesting mean something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantified {
    /// The collection to walk.
    pub over: FactPath,
    /// The name the body reads each element under. One fact-path segment.
    pub bind: String,
    /// The condition each element is held to.
    pub body: Predicate,
}

impl Quantified {
    /// Evaluates the body once per element, folding conjunctively when `universal`.
    fn evaluate(&self, facts: &dyn FactSource, universal: bool) -> Truth {
        let Some(count) = facts.cardinality(&self.over) else {
            return Truth::Unknown;
        };
        let mut result = if universal { Truth::True } else { Truth::False };
        for index in 0..count {
            let element = Element {
                inner: facts,
                bind: &self.bind,
                prefix: self.over.child(&index.to_string()),
            };
            let truth = self.body.evaluate(&element);
            result = if universal {
                result.and(truth)
            } else {
                result.or(truth)
            };
            // `False` dominates a conjunction and `True` a disjunction, so the remaining elements
            // cannot change the answer. Stopping is not an optimisation: walking a collection whose
            // verdict is already settled would report causes from elements nobody is waiting on.
            if result == if universal { Truth::False } else { Truth::True } {
                break;
            }
        }
        result
    }
}

/// One element of a collection, seen as a fact source.
///
/// Reads of `<bind>.rest` become reads of `<over>.<index>.rest`; every other path passes through
/// untouched, which is what lets a body mix element facts with free ones. Nesting one of these
/// inside another is how an inner quantifier still reaches the outer element: the inner rewrites
/// its own binder and hands the result to the outer, which rewrites its own.
struct Element<'a> {
    inner: &'a dyn FactSource,
    bind: &'a str,
    prefix: FactPath,
}

impl Element<'_> {
    fn rebind(&self, path: &FactPath) -> FactPath {
        if path.namespace() != self.bind {
            return path.clone();
        }
        let mut rebound = self.prefix.clone();
        for segment in &path.segments()[1..] {
            rebound = rebound.child(segment);
        }
        rebound
    }
}

impl FactSource for Element<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.inner.fact(&self.rebind(path))
    }

    fn observe(&self, path: &FactPath) -> Option<FactValue> {
        self.inner.observe(&self.rebind(path))
    }

    fn present(&self, path: &FactPath) -> bool {
        self.inner.present(&self.rebind(path))
    }

    fn observed_presence(&self, path: &FactPath) -> Option<bool> {
        self.inner.observed_presence(&self.rebind(path))
    }

    fn scales(&self) -> &Scales {
        self.inner.scales()
    }

    fn orders_as_instant(&self, path: &FactPath) -> bool {
        self.inner.orders_as_instant(&self.rebind(path))
    }

    fn now(&self) -> Option<crate::time::Rfc3339Instant> {
        self.inner.now()
    }

    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        self.inner.orders_text_by_bytes(&self.rebind(path))
    }

    fn cardinality(&self, path: &FactPath) -> Option<usize> {
        self.inner.cardinality(&self.rebind(path))
    }
}

/// The equality a [`Distinct`] key is compared under: the scalar domain its declared type resolves
/// to, through newtypes and `Optional` (`docs/design/expression-family-source22.md`, `distinct`).
///
/// The domains `count_distinct` already compares (`docs/design/aggregate-views.md`): a whole struct,
/// list, map, union or `Json` value has no key equality, so a struct list names one member with
/// `by`. The resolver in `ess-domain` decides the kind from the declarations and the canonical form
/// carries it, so a reader with no declared types — a suite runner over view rows — never infers an
/// instant from a spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DistinctKeyKind {
    /// `Boolean`: `true` and `false`.
    Boolean,
    /// `Integer`: exact whole numbers, never through a binary64.
    Integer,
    /// `Decimal`: exact numeric equality, so `1` and `1.0` are one key.
    Decimal,
    /// `String`: exact text.
    String,
    /// `Uuid`: exact text.
    Uuid,
    /// `Timestamp`: the instant, so two spellings of one instant are one key.
    Timestamp,
    /// An enum, through any newtype: exact variant text.
    Enum,
}

impl DistinctKeyKind {
    /// Every kind, in declaration order.
    pub const ALL: [Self; 7] = [
        Self::Boolean,
        Self::Integer,
        Self::Decimal,
        Self::String,
        Self::Uuid,
        Self::Timestamp,
        Self::Enum,
    ];

    /// The canonical spelling of this kind, the value of `kind:`.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::String => "string",
            Self::Uuid => "uuid",
            Self::Timestamp => "timestamp",
            Self::Enum => "enum",
        }
    }

    /// The kind a canonical spelling names.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.keyword() == keyword)
    }

    /// The key an observed value is under this kind, or `None` where the value lies outside it —
    /// a fraction under `Integer`, a text under a number, a text no `date-time` spells under
    /// `Timestamp` — which the comparison reads as `Unknown` rather than coercing.
    fn key(self, value: &FactValue) -> Option<DistinctKey> {
        match (self, value) {
            (Self::Boolean, FactValue::Bool(flag)) => Some(DistinctKey::Bool(*flag)),
            (Self::Integer, FactValue::Number(number)) if number.is_integral() => {
                Some(DistinctKey::Number(*number))
            }
            (Self::Decimal, FactValue::Number(number)) => Some(DistinctKey::Number(*number)),
            (Self::String | Self::Uuid | Self::Enum, FactValue::Text(text)) => {
                Some(DistinctKey::Text(text.clone()))
            }
            (Self::Timestamp, FactValue::Text(text)) => {
                crate::time::Rfc3339Instant::parse_rfc3339(text).map(DistinctKey::Instant)
            }
            _ => None,
        }
    }
}

impl fmt::Display for DistinctKeyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.keyword())
    }
}

/// One observed key, under the equality of its [`DistinctKeyKind`]. [`crate::facts::Number`]'s
/// order is exact, and an instant's is the UTC line's.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum DistinctKey {
    Bool(bool),
    Number(crate::facts::Number),
    Text(String),
    Instant(crate::time::Rfc3339Instant),
}

/// No two elements of a list share a key (`docs/design/expression-family-source22.md`,
/// `distinct`): `distinct: {in: files, as: file, by: file.path}`.
///
/// The key is the element itself, or the one scalar member `key` names under the binder. A present
/// empty or one-element list holds. For two or more, two known equal keys make it `False` wherever
/// they stand; every key known and pairwise unequal makes it `True`; anything else is `Unknown`. An
/// absent list is `Unknown`, not empty, and an absent key is neither skipped nor one shared null.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distinct {
    /// The list.
    pub over: FactPath,
    /// The name each element is read under. One fact-path segment.
    pub bind: String,
    /// The member of the element that is its key, rooted at `bind`; `None` keys the element.
    pub key: Option<FactPath>,
    /// The equality the keys compare under. `None` only as an authored source writes it, before the
    /// domain's resolver reads it off the declarations; nothing compares an unresolved key, and no
    /// suite reader admits one.
    pub key_kind: Option<DistinctKeyKind>,
}

impl Distinct {
    /// The path each element's key is read at: `key`, or the binder itself.
    pub fn key_path(&self) -> FactPath {
        self.key
            .clone()
            .unwrap_or_else(|| FactPath::from_segments([self.bind.as_str()]))
    }

    /// The canonical document form, `{distinct: {in, as, by, kind}}`. There is no compact form;
    /// `kind` is written once resolved and left out before, so a source reads back as written.
    pub fn to_node(&self) -> Node {
        let mut fields = std::collections::BTreeMap::new();
        fields.insert("in".to_owned(), Node::Text(self.over.to_string()));
        fields.insert("as".to_owned(), Node::Text(self.bind.clone()));
        if let Some(key) = &self.key {
            fields.insert("by".to_owned(), Node::Text(key.to_string()));
        }
        if let Some(kind) = self.key_kind {
            fields.insert("kind".to_owned(), Node::Text(kind.keyword().to_owned()));
        }
        Node::Map([("distinct".to_owned(), Node::Map(fields))].into())
    }

    fn evaluate(&self, facts: &dyn FactSource) -> Truth {
        let Some(kind) = self.key_kind else {
            return Truth::Unknown;
        };
        let Some(count) = facts.cardinality(&self.over) else {
            return Truth::Unknown;
        };
        if count < 2 {
            return Truth::True;
        }
        let key = self.key_path();
        let mut seen = std::collections::BTreeSet::new();
        let mut unknown = false;
        for index in 0..count {
            let element = Element {
                inner: facts,
                bind: &self.bind,
                prefix: self.over.child(&index.to_string()),
            };
            match element.observe(&key).and_then(|value| kind.key(&value)) {
                // Every pair, not neighbours only: a duplicate anywhere settles it.
                Some(observed) => {
                    if !seen.insert(observed) {
                        return Truth::False;
                    }
                }
                None => unknown = true,
            }
        }
        if unknown {
            Truth::Unknown
        } else {
            Truth::True
        }
    }
}

impl Predicate {
    /// Conjunction, simplified: an empty list is [`Predicate::Always`], a single child is
    /// returned unwrapped.
    pub fn all(children: Vec<Self>) -> Self {
        Self::combine(children, true)
    }

    /// Disjunction, simplified: an empty list is [`Predicate::Never`], a single child is
    /// returned unwrapped.
    pub fn any(children: Vec<Self>) -> Self {
        Self::combine(children, false)
    }

    fn combine(mut children: Vec<Self>, conjunction: bool) -> Self {
        match children.len() {
            0 => {
                if conjunction {
                    Self::Always
                } else {
                    Self::Never
                }
            }
            1 => children.remove(0),
            _ => {
                if conjunction {
                    Self::All(children)
                } else {
                    Self::Any(children)
                }
            }
        }
    }

    /// Negation, simplified.
    #[allow(clippy::should_implement_trait)]
    pub fn not(inner: Self) -> Self {
        match inner {
            Self::Always => Self::Never,
            Self::Never => Self::Always,
            Self::Not(nested) => *nested,
            other => Self::Not(Box::new(other)),
        }
    }

    /// `true` when this predicate holds without observing anything.
    pub fn is_trivially_true(&self) -> bool {
        matches!(self, Self::Always)
    }

    /// Evaluates this predicate against `facts`.
    pub fn evaluate(&self, facts: &dyn FactSource) -> Truth {
        match self {
            Self::Always => Truth::True,
            Self::Never => Truth::False,
            Self::All(children) => children
                .iter()
                .fold(Truth::True, |acc, child| acc.and(child.evaluate(facts))),
            Self::Any(children) => children
                .iter()
                .fold(Truth::False, |acc, child| acc.or(child.evaluate(facts))),
            Self::Not(inner) => inner.evaluate(facts).not(),
            Self::Compare {
                left,
                op,
                right,
                kind,
            } => Self::evaluate_compare(left, *op, right, *kind, facts).0,
            Self::Truthy(path) => facts
                .observe(path)
                .map_or(Truth::Unknown, |value| Truth::from_bool(value.is_truthy())),
            Self::Defined(path) => facts
                .observed_presence(path)
                .map_or(Truth::Unknown, Truth::from_bool),
            Self::AnyOf { path, values } => {
                facts.observe(path).map_or(Truth::Unknown, |observed| {
                    Truth::from_bool(values.contains(&observed))
                })
            }
            Self::NoneOf { path, values } => {
                facts.observe(path).map_or(Truth::Unknown, |observed| {
                    Truth::from_bool(!values.contains(&observed))
                })
            }
            // A parameter or an input nobody observed is unknown, as the fact is: never the empty
            // text, which every text would begin with, end with and contain.
            Self::TextMatch { path, op, value } => {
                match (facts.observe(path), value.resolve(facts)) {
                    (None, _) | (_, None) => Truth::Unknown,
                    (Some(FactValue::Text(text)), Some(FactValue::Text(operand))) => {
                        Truth::from_bool(op.holds(&text, &operand))
                    }
                    _ => Truth::False,
                }
            }
            Self::FoldMatch { path, op, values } => {
                facts.observe(path).map_or(Truth::Unknown, |observed| {
                    let FactValue::Text(text) = &observed else {
                        return Truth::False;
                    };
                    let literals: Vec<&str> =
                        values.iter().filter_map(FactValue::as_text).collect();
                    Truth::from_bool(op.holds(text, &literals))
                })
            }
            Self::Forall(quantified) => quantified.evaluate(facts, true),
            Self::Exists(quantified) => quantified.evaluate(facts, false),
            Self::Distinct(distinct) => distinct.evaluate(facts),
            Self::Window(window) => window.evaluate(facts).0,
        }
    }

    /// Evaluates a comparison, also returning a note when the comparison is not well defined.
    fn evaluate_compare(
        left: &Operand,
        op: CompareOp,
        right: &Operand,
        kind: CompareKind,
        facts: &dyn FactSource,
    ) -> (Truth, Option<String>) {
        if let Some(offset) = Self::offset_compare(left, op, right, facts) {
            return offset;
        }
        let (Some(left_value), Some(right_value)) = (left.resolve(facts), right.resolve(facts))
        else {
            return (Truth::Unknown, None);
        };

        // A tagged comparison (decision 2) compares the instants its two `Timestamp` operands
        // name, whatever the fact source knows of their types. A text that names no instant is
        // `Unknown`: an instant nobody can read has no order, and sorting its spelling would
        // invent one.
        if kind == CompareKind::Instant {
            let instant = |value: &FactValue| {
                value
                    .as_text()
                    .and_then(crate::time::Rfc3339Instant::parse_rfc3339)
            };
            return match (instant(&left_value), instant(&right_value)) {
                (Some(left_instant), Some(right_instant)) => (
                    Truth::from_bool(op.accepts(left_instant.cmp(&right_instant))),
                    None,
                ),
                _ => (
                    Truth::Unknown,
                    Some(format!(
                        "cannot compare {left_value} with {right_value} as instants: each side \
                         must be an RFC 3339 date-time"
                    )),
                ),
            };
        }

        // A declared Timestamp compares by the instant it names under every operator, so `==`
        // agrees with `<=` and `>=` on two spellings of one instant.
        let mut declared_instant = false;
        if let (FactValue::Text(left_text), FactValue::Text(right_text)) =
            (&left_value, &right_value)
        {
            declared_instant = [left, right].into_iter().any(|operand| {
                operand
                    .fact_path()
                    .is_some_and(|path| facts.orders_as_instant(path))
            });
            let instant = |operand: &Operand, text: &str| instant_operand(operand, text, facts);
            if let (true, Some(left_instant), Some(right_instant)) = (
                declared_instant,
                instant(left, left_text),
                instant(right, right_text),
            ) {
                return (
                    Truth::from_bool(op.accepts(left_instant.cmp(&right_instant))),
                    None,
                );
            }
        }

        match (&left_value, &right_value) {
            (FactValue::Number(left_number), FactValue::Number(right_number)) => (
                Truth::from_bool(op.accepts(left_number.cmp(right_number))),
                None,
            ),
            (FactValue::Text(left_text), FactValue::Text(right_text)) if op.needs_ordering() => {
                match facts.scales().compare(left_text, right_text) {
                    Some(ordering) => (Truth::from_bool(op.accepts(ordering)), None),
                    // `str`'s `Ord` is the lexicographic order of the UTF-8 bytes, which is the
                    // order the Go (`strings.Compare`) and TypeScript (`byteCompare`) lanes use.
                    // Never for a declared `Timestamp` that did not parse: an instant nobody can
                    // read has no order, and sorting its spelling would invent one.
                    None if !declared_instant
                        && [left, right].into_iter().all(|operand| {
                            operand
                                .fact_path()
                                .is_none_or(|path| facts.orders_text_by_bytes(path))
                        }) =>
                    {
                        (
                            Truth::from_bool(
                                op.accepts(left_text.as_str().cmp(right_text.as_str())),
                            ),
                            None,
                        )
                    }
                    None => (
                        Truth::Unknown,
                        Some(format!(
                            "cannot order {left_text:?} against {right_text:?}: no protocol scale \
                             contains both values"
                        )),
                    ),
                }
            }
            _ if op.needs_ordering() => (
                Truth::Unknown,
                Some(format!(
                    "cannot order a {} against a {}",
                    left_value.type_name(),
                    right_value.type_name()
                )),
            ),
            _ => {
                let equal = left_value == right_value;
                (
                    Truth::from_bool(if op == CompareOp::Eq { equal } else { !equal }),
                    None,
                )
            }
        }
    }

    /// A comparison with an offset on either side, or `None` where neither side is one: one on
    /// the right is [`Self::evaluate_offset`]; one on the left, which no reader builds, is `Unknown`.
    fn offset_compare(
        left: &Operand,
        op: CompareOp,
        right: &Operand,
        facts: &dyn FactSource,
    ) -> Option<(Truth, Option<String>)> {
        match (left, right) {
            (_, Operand::Offset(offset)) => Some(Self::evaluate_offset(left, op, offset, facts)),
            (Operand::Offset(offset), _) => Some((
                Truth::Unknown,
                Some(format!(
                    "`{offset}` stands on the left of `{op}`; an offset is legal on the right of a \
                     comparison only"
                )),
            )),
            _ => None,
        }
    }

    /// `left <op> base ± magnitude` (`docs/design/expression-family-source22.md`, A2).
    ///
    /// An Integer magnitude compares two `Integer` values with the exact mathematical sum: every
    /// `i64 ± i64` fits `i128`, so nothing wraps, saturates, rounds through binary64 or turns
    /// `Unknown` at the ends of the stored range. An elapsed magnitude moves a `Timestamp` by UTC
    /// seconds and compares instants under all six operators. An unobserved or absent operand is
    /// `Unknown`, and so is a value of the wrong kind or an instant no `date-time` spells, with a
    /// note saying which.
    fn evaluate_offset(
        left: &Operand,
        op: CompareOp,
        offset: &OffsetOperand,
        facts: &dyn FactSource,
    ) -> (Truth, Option<String>) {
        let (Some(left_value), Some(base_value)) =
            (left.resolve(facts), facts.observe(&offset.base))
        else {
            return (Truth::Unknown, None);
        };
        let ordering = match offset.magnitude {
            OffsetMagnitude::Integer(_) => {
                let integer =
                    |value: &FactValue| value.as_number().and_then(crate::facts::Number::as_i64);
                match (integer(&left_value), integer(&base_value)) {
                    (Some(left_integer), Some(base_integer)) => offset
                        .integer_at(base_integer)
                        .map(|bound| i128::from(left_integer).cmp(&bound)),
                    _ => None,
                }
                .ok_or_else(|| {
                    format!(
                        "cannot compare {left_value} with `{offset}` of {base_value}: an Integer \
                         offset compares two whole numbers"
                    )
                })
            }
            OffsetMagnitude::ElapsedSeconds { .. } => {
                let instant = |value: &FactValue| {
                    value
                        .as_text()
                        .and_then(crate::time::Rfc3339Instant::parse_rfc3339)
                };
                match (instant(&left_value), instant(&base_value)) {
                    (Some(left_instant), Some(base_instant)) => offset
                        .instant_at(base_instant)
                        .map(|bound| left_instant.cmp(&bound))
                        .ok_or_else(|| {
                            format!(
                                "`{offset}` of {base_value} is no instant an RFC 3339 date-time \
                                 spells"
                            )
                        }),
                    _ => Err(format!(
                        "cannot compare {left_value} with `{offset}` of {base_value} as instants: \
                         each side must be an RFC 3339 date-time"
                    )),
                }
            }
        };
        match ordering {
            Ok(ordering) => (Truth::from_bool(op.accepts(ordering)), None),
            Err(note) => (Truth::Unknown, Some(note)),
        }
    }

    /// Evaluates this predicate and explains why it is not satisfied.
    ///
    /// The returned causes are minimal: for a conjunction only the children that did not hold,
    /// for a disjunction every child (since all of them failed), and for a negation the leaves
    /// of the inner predicate that did hold.
    pub fn outcome(&self, facts: &dyn FactSource) -> PredicateOutcome {
        let mut causes = Vec::new();
        let truth = self.collect_causes(facts, false, &mut causes);
        PredicateOutcome {
            expression: self.to_string(),
            truth,
            causes,
        }
    }

    /// Walks the predicate, recording the leaves responsible for a non-satisfied result.
    fn collect_causes(
        &self,
        facts: &dyn FactSource,
        negated: bool,
        causes: &mut Vec<LeafOutcome>,
    ) -> Truth {
        let satisfied = |truth: Truth| {
            if negated {
                truth == Truth::False
            } else {
                truth == Truth::True
            }
        };

        match self {
            Self::All(children) | Self::Any(children) => {
                let conjunction = matches!(self, Self::All(_));
                let truths: Vec<Truth> =
                    children.iter().map(|child| child.evaluate(facts)).collect();
                let truth = if conjunction {
                    truths.iter().fold(Truth::True, |acc, item| acc.and(*item))
                } else {
                    truths.iter().fold(Truth::False, |acc, item| acc.or(*item))
                };
                if !satisfied(truth) {
                    // For a conjunction, the failing children are the cause; for a
                    // disjunction, every child failed, so all of them are.
                    for (child, child_truth) in children.iter().zip(&truths) {
                        if conjunction != negated && satisfied(*child_truth) {
                            continue;
                        }
                        child.collect_causes(facts, negated, causes);
                    }
                }
                truth
            }
            Self::Not(inner) => inner.collect_causes(facts, !negated, causes).not(),
            Self::Always => Truth::True,
            Self::Never => {
                if !satisfied(Truth::False) {
                    causes.push(LeafOutcome {
                        expression: "never".to_owned(),
                        truth: Truth::False,
                        observed: Vec::new(),
                        missing: Vec::new(),
                        note: Some("this transition is explicitly disabled".to_owned()),
                        negated,
                    });
                }
                Truth::False
            }
            leaf => {
                let (truth, note) = leaf.evaluate_leaf(facts);
                if !satisfied(truth) {
                    let mut observed = Vec::new();
                    let mut missing = Vec::new();
                    for path in leaf.fact_paths() {
                        match facts.observe(path) {
                            Some(value) => observed.push((path.clone(), value)),
                            None => missing.push(path.clone()),
                        }
                    }
                    causes.push(LeafOutcome {
                        expression: leaf.to_string(),
                        truth,
                        observed,
                        missing,
                        note,
                        negated,
                    });
                }
                truth
            }
        }
    }

    /// Evaluates a leaf, returning any note about an ill-defined comparison.
    fn evaluate_leaf(&self, facts: &dyn FactSource) -> (Truth, Option<String>) {
        match self {
            Self::Compare {
                left,
                op,
                right,
                kind,
            } => Self::evaluate_compare(left, *op, right, *kind, facts),
            Self::Window(window) => window.evaluate(facts),
            other => (other.evaluate(facts), None),
        }
    }

    /// Every fact path this predicate reads, in traversal order, without duplicates.
    pub fn fact_paths(&self) -> Vec<&FactPath> {
        let mut paths = Vec::new();
        self.visit_fact_paths(&mut |path| {
            if !paths.contains(&path) {
                paths.push(path);
            }
        });
        paths
    }

    fn visit_fact_paths<'a>(&'a self, visit: &mut impl FnMut(&'a FactPath)) {
        self.visit_free_paths(&mut Vec::new(), visit);
    }

    /// [`Self::visit_fact_paths`], skipping paths rooted at a quantifier's binder.
    ///
    /// A binder is not a fact and no projection publishes one, so a caller validating that "every
    /// path this predicate reads exists in the model" must not be shown `slot.left`. It is shown
    /// the collection the binder ranges over instead, which *is* a model path, and every free path
    /// the body reads beside it.
    fn visit_free_paths<'a>(
        &'a self,
        bound: &mut Vec<&'a str>,
        visit: &mut impl FnMut(&'a FactPath),
    ) {
        match self {
            Self::Always | Self::Never => {}
            Self::All(children) | Self::Any(children) => {
                for child in children {
                    child.visit_free_paths(bound, visit);
                }
            }
            Self::Not(inner) => inner.visit_free_paths(bound, visit),
            Self::Compare { left, right, .. } => {
                for path in [left.fact_path(), right.fact_path()].into_iter().flatten() {
                    if !bound.contains(&path.namespace()) {
                        visit(path);
                    }
                }
            }
            Self::Truthy(path)
            | Self::Defined(path)
            | Self::AnyOf { path, .. }
            | Self::NoneOf { path, .. }
            | Self::FoldMatch { path, .. } => {
                if !bound.contains(&path.namespace()) {
                    visit(path);
                }
            }
            // A parameter or an input is read as the fact is (beyond10x/ess#200).
            Self::TextMatch { path, value, .. } => {
                for path in std::iter::once(path).chain(value.fact_path()) {
                    if !bound.contains(&path.namespace()) {
                        visit(path);
                    }
                }
            }
            Self::Forall(quantified) | Self::Exists(quantified) => {
                if !bound.contains(&quantified.over.namespace()) {
                    visit(&quantified.over);
                }
                bound.push(&quantified.bind);
                quantified.body.visit_free_paths(bound, visit);
                bound.pop();
            }
            // The key is read under the binder, so only the list is a free read.
            Self::Distinct(distinct) => {
                if !bound.contains(&distinct.over.namespace()) {
                    visit(&distinct.over);
                }
            }
            Self::Window(window) => {
                if let Some(path) = window.at.fact_path() {
                    if !bound.contains(&path.namespace()) {
                        visit(path);
                    }
                }
            }
        }
    }

    /// Every collection a quantifier in this predicate walks, outermost first.
    ///
    /// A caller validating an invariant needs these separately from [`Self::fact_paths`]: those are
    /// checked for *existing*, and a collection is additionally checked for *being* one. Walking a
    /// scalar is not a condition nobody can decide, it is a condition that means nothing.
    /// A nested quantifier over an outer element — `forall e in slot.left` inside `forall slot in
    /// slots` — is *not* returned: `slot.left` names no field of anything the caller can resolve.
    /// It is checked where it can be, by the same walk one level down.
    pub fn quantified_collections(&self) -> Vec<&FactPath> {
        let mut found = Vec::new();
        self.visit_quantified(&mut Vec::new(), &mut found);
        found
    }

    fn visit_quantified<'a>(&'a self, bound: &mut Vec<&'a str>, found: &mut Vec<&'a FactPath>) {
        match self {
            Self::All(children) | Self::Any(children) => {
                for child in children {
                    child.visit_quantified(bound, found);
                }
            }
            Self::Not(inner) => inner.visit_quantified(bound, found),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                if !bound.contains(&quantified.over.namespace()) {
                    found.push(&quantified.over);
                }
                bound.push(&quantified.bind);
                quantified.body.visit_quantified(bound, found);
                bound.pop();
            }
            Self::Distinct(distinct) => {
                if !bound.contains(&distinct.over.namespace()) {
                    found.push(&distinct.over);
                }
            }
            Self::Always
            | Self::Never
            | Self::Compare { .. }
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::FoldMatch { .. }
            | Self::Window(_) => {}
        }
    }

    /// Whether any leaf of this predicate, at any depth, is a string operator.
    ///
    /// The one question every format gate asks of the construct: `ess/8` for an authored
    /// specification, the suite pair for a suite, and `infra-spec/1`'s refusal.
    pub fn uses_text_match(&self) -> bool {
        match self {
            Self::TextMatch { .. } => true,
            Self::All(children) | Self::Any(children) => children.iter().any(Self::uses_text_match),
            Self::Not(inner) => inner.uses_text_match(),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                quantified.body.uses_text_match()
            }
            Self::Always
            | Self::Never
            | Self::Compare { .. }
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::FoldMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// Whether any string operator, at any depth, compares with a parameter or an input rather than
    /// a literal (beyond10x/ess#200): the question every lane that executes only literal operands
    /// asks before refusing by name.
    pub fn uses_text_operand(&self) -> bool {
        match self {
            Self::TextMatch { value, .. } => value.fact_path().is_some(),
            Self::All(children) | Self::Any(children) => {
                children.iter().any(Self::uses_text_operand)
            }
            Self::Not(inner) => inner.uses_text_operand(),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                quantified.body.uses_text_operand()
            }
            Self::Always
            | Self::Never
            | Self::Compare { .. }
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::FoldMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// Whether any string operator, at any depth, compares with the view parameter `name`,
    /// `{param: <name>}` (beyond10x/ess#200).
    pub fn reads_text_parameter(&self, name: &str) -> bool {
        match self {
            Self::TextMatch {
                value:
                    TextOperand::Fact {
                        namespace: TextNamespace::Param,
                        name: read,
                        ..
                    },
                ..
            } => read == name,
            Self::All(children) | Self::Any(children) => children
                .iter()
                .any(|child| child.reads_text_parameter(name)),
            Self::Not(inner) => inner.reads_text_parameter(name),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                quantified.body.reads_text_parameter(name)
            }
            Self::Always
            | Self::Never
            | Self::Compare { .. }
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::FoldMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// Whether any comparison, at any depth, reads a one-segment fact on its right that no binder in
    /// scope names: the operand the canonical form writes as `{fact: …}`
    /// (`docs/design/expression-family-source22.md`, A1).
    ///
    /// The question the `ess/22` source gate and the suite pair `/40` and `/41` ask. A binder and a
    /// dotted path are not such operands: their compact spelling was always read as a fact.
    pub fn reads_root_fact_operand(&self) -> bool {
        self.reads_root_fact_in(&mut Vec::new())
    }

    fn reads_root_fact_in<'a>(&'a self, binders: &mut Vec<&'a str>) -> bool {
        match self {
            Self::Compare {
                right,
                kind: CompareKind::Value,
                ..
            } => right.root_fact(binders).is_some(),
            Self::All(children) | Self::Any(children) => children
                .iter()
                .any(|child| child.reads_root_fact_in(binders)),
            Self::Not(inner) => inner.reads_root_fact_in(binders),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                binders.push(&quantified.bind);
                let found = quantified.body.reads_root_fact_in(binders);
                binders.pop();
                found
            }
            // The tagged form always writes its fact operand explicitly.
            Self::Compare {
                kind: CompareKind::Instant,
                ..
            }
            | Self::Always
            | Self::Never
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::FoldMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// Whether any comparison, at any depth, is tagged to compare instants (decision 2): the
    /// question the `ess/22` source gate and the suite pair `/40` and `/41` ask beside
    /// [`Self::reads_root_fact_operand`].
    pub fn compares_instants(&self) -> bool {
        match self {
            Self::Compare { kind, .. } => *kind == CompareKind::Instant,
            Self::All(children) | Self::Any(children) => {
                children.iter().any(Self::compares_instants)
            }
            Self::Not(inner) => inner.compares_instants(),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                quantified.body.compares_instants()
            }
            Self::Always
            | Self::Never
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::FoldMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// Whether any comparison, at any depth, compares with one constant offset of a fact
    /// (`docs/design/expression-family-source22.md`, A2): the question the `ess/22` source gate and
    /// the suite pair `/40` and `/41` ask beside [`Self::reads_root_fact_operand`].
    pub fn reads_offset(&self) -> bool {
        match self {
            Self::Compare { left, right, .. } => {
                matches!(left, Operand::Offset(_)) || matches!(right, Operand::Offset(_))
            }
            Self::All(children) | Self::Any(children) => children.iter().any(Self::reads_offset),
            Self::Not(inner) => inner.reads_offset(),
            Self::Forall(quantified) | Self::Exists(quantified) => quantified.body.reads_offset(),
            Self::Always
            | Self::Never
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::FoldMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// Whether any leaf, at any depth, is a [`Predicate::Distinct`]
    /// (`docs/design/expression-family-source22.md`, `distinct`): the question the `ess/22` source
    /// gate and the suite pair `/40` and `/41` ask beside [`Self::reads_offset`].
    pub fn reads_distinct(&self) -> bool {
        match self {
            Self::Distinct(_) => true,
            Self::All(children) | Self::Any(children) => children.iter().any(Self::reads_distinct),
            Self::Not(inner) => inner.reads_distinct(),
            Self::Forall(quantified) | Self::Exists(quantified) => quantified.body.reads_distinct(),
            Self::Always
            | Self::Never
            | Self::Compare { .. }
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::FoldMatch { .. }
            | Self::Window(_) => false,
        }
    }

    /// Whether any leaf, at any depth, is a calendar window (`ess/22`,
    /// `docs/design/calendar-window-guards.md`): the question the source gate asks.
    pub fn reads_window(&self) -> bool {
        !self.windows().is_empty()
    }

    /// Every calendar window of this predicate, at any depth, in pre-order.
    pub fn windows(&self) -> Vec<&crate::window::CalendarWindow> {
        fn walk<'a>(predicate: &'a Predicate, found: &mut Vec<&'a crate::window::CalendarWindow>) {
            match predicate {
                Predicate::Window(window) => found.push(window),
                Predicate::All(children) | Predicate::Any(children) => {
                    for child in children {
                        walk(child, found);
                    }
                }
                Predicate::Not(inner) => walk(inner, found),
                Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                    walk(&quantified.body, found);
                }
                Predicate::Always
                | Predicate::Never
                | Predicate::Compare { .. }
                | Predicate::Truthy(_)
                | Predicate::Defined(_)
                | Predicate::AnyOf { .. }
                | Predicate::NoneOf { .. }
                | Predicate::TextMatch { .. }
                | Predicate::FoldMatch { .. }
                | Predicate::Distinct(_) => {}
            }
        }
        let mut found = Vec::new();
        walk(self, &mut found);
        found
    }

    /// Every [`Distinct`] in this predicate, at any depth, outermost first, each with the binders
    /// in scope where it stands, outermost first: what a checker or a producer asks of each one.
    pub fn distincts(&self) -> Vec<(&Distinct, Vec<&Quantified>)> {
        fn walk<'a>(
            predicate: &'a Predicate,
            scope: &mut Vec<&'a Quantified>,
            found: &mut Vec<(&'a Distinct, Vec<&'a Quantified>)>,
        ) {
            match predicate {
                Predicate::Distinct(distinct) => found.push((distinct, scope.clone())),
                Predicate::All(children) | Predicate::Any(children) => {
                    for child in children {
                        walk(child, scope, found);
                    }
                }
                Predicate::Not(inner) => walk(inner, scope, found),
                Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                    scope.push(quantified);
                    walk(&quantified.body, scope, found);
                    scope.pop();
                }
                _ => {}
            }
        }
        let mut found = Vec::new();
        walk(self, &mut Vec::new(), &mut found);
        found
    }

    /// Whether any comparison, at any depth, reads a derived operand — the UTF-8 byte length of a
    /// text, `{utf8_bytes: <path>}` (`docs/design/expression-family-source22.md`, decision 11): the
    /// question the `ess/22` source gate and the suite pair `/40` and `/41` ask beside
    /// [`Self::reads_offset`]. A path that merely ends in `utf8_bytes` is a field, and is not one.
    pub fn reads_utf8_bytes(&self) -> bool {
        match self {
            Self::Compare { left, right, .. } => {
                matches!(left, Operand::Derived(_)) || matches!(right, Operand::Derived(_))
            }
            Self::All(children) | Self::Any(children) => {
                children.iter().any(Self::reads_utf8_bytes)
            }
            Self::Not(inner) => inner.reads_utf8_bytes(),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                quantified.body.reads_utf8_bytes()
            }
            Self::Always
            | Self::Never
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::FoldMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// A comparison by value: what every comparison was before decision 2's tag.
    pub fn compare(left: Operand, op: CompareOp, right: Operand) -> Self {
        Self::Compare {
            left,
            op,
            right,
            kind: CompareKind::Value,
        }
    }

    /// Whether any leaf of this predicate, at any depth, is a case-insensitive text operator.
    ///
    /// The question every format gate asks of the construct (beyond10x/ess#140): `ess/15` for an
    /// authored specification, the suite pair `/20` and `/21` for a suite, and `infra-spec/1`'s
    /// refusal. Apart from [`Self::uses_text_match`], because the two arrived in different formats.
    pub fn uses_case_fold(&self) -> bool {
        match self {
            Self::FoldMatch { .. } => true,
            Self::All(children) | Self::Any(children) => children.iter().any(Self::uses_case_fold),
            Self::Not(inner) => inner.uses_case_fold(),
            Self::Forall(quantified) | Self::Exists(quantified) => quantified.body.uses_case_fold(),
            Self::Always
            | Self::Never
            | Self::Compare { .. }
            | Self::Truthy(_)
            | Self::Defined(_)
            | Self::AnyOf { .. }
            | Self::NoneOf { .. }
            | Self::TextMatch { .. }
            | Self::Distinct(_)
            | Self::Window(_) => false,
        }
    }

    /// Parses a predicate from a document fragment.
    ///
    /// Refuses nesting beyond [`MAX_PREDICATE_DEPTH`] with [`ParseError::TooDeep`]. The budget is
    /// shared with [`Self::parse_expression`], because the two forms interleave: `{not: {not: "not
    /// not a.b"}}` is four levels of one predicate written two ways, and two counters would let a
    /// document alternate between them to buy twice the depth.
    pub fn from_node(node: &Node) -> Result<Self, ParseError> {
        Self::from_node_nested(node, 0, &[], &mut Spellings::default())
    }

    /// [`Self::from_node`], also saying which comparisons' right-hand sides were bare words.
    ///
    /// The predicate is exactly the one [`Self::from_node`] reads; see [`Spelled`].
    pub fn from_node_spelled(node: &Node) -> Result<Spelled, ParseError> {
        let mut spellings = Spellings::default();
        let predicate = Self::from_node_nested(node, 0, &[], &mut spellings)?;
        Ok(Spelled {
            predicate,
            words: spellings.words,
            offsets: spellings.offsets,
        })
    }

    /// [`Self::parse_expression`], also saying which comparisons' right-hand sides were bare
    /// words. The predicate is exactly the one [`Self::parse_expression`] reads; see [`Spelled`].
    pub fn parse_expression_spelled(expression: &str) -> Result<Spelled, ParseError> {
        let mut spellings = Spellings::default();
        let predicate = Self::parse_expression_nested(expression, 0, &[], &mut spellings)?;
        Ok(Spelled {
            predicate,
            words: spellings.words,
            offsets: spellings.offsets,
        })
    }

    /// [`Self::from_node`], counting how deep it already is.
    fn from_node_nested(
        node: &Node,
        depth: usize,
        binders: &[String],
        words: &mut Spellings,
    ) -> Result<Self, ParseError> {
        if depth > MAX_PREDICATE_DEPTH {
            return Err(ParseError::too_deep(
                "predicate",
                &shallow(node),
                MAX_PREDICATE_DEPTH,
            ));
        }
        match node {
            Node::Bool(true) => Ok(Self::Always),
            Node::Bool(false) => Ok(Self::Never),
            Node::Text(expression) => {
                Self::parse_expression_nested(expression, depth, binders, words)
            }
            Node::Seq(items) => {
                let children = items
                    .iter()
                    .map(|item| Self::from_node_nested(item, depth + 1, binders, words))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self::all(children))
            }
            Node::Map(entries) => {
                let mut children = Vec::new();
                for (key, value) in entries {
                    children.push(Self::from_entry(key, value, depth, binders, words)?);
                }
                Ok(Self::all(children))
            }
            Node::Null => Err(ParseError::shape(
                "predicate",
                "an expression, list or mapping",
                "null",
            )),
            Node::Number(number) => Err(ParseError::shape(
                "predicate",
                "an expression, list or mapping",
                format!("the number {number}"),
            )),
        }
    }

    /// Parses one `key: value` entry of a predicate mapping.
    fn from_entry(
        key: &str,
        value: &Node,
        depth: usize,
        binders: &[String],
        words: &mut Spellings,
    ) -> Result<Self, ParseError> {
        let mut nested = |node: &Node| Self::from_node_nested(node, depth + 1, binders, words);
        match key {
            "all" | "and" | "all_of" => {
                let children = value
                    .as_seq_or_single()
                    .into_iter()
                    .map(&mut nested)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self::all(children))
            }
            "any" | "or" => {
                let children = value
                    .as_seq_or_single()
                    .into_iter()
                    .map(&mut nested)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self::any(children))
            }
            "not" => Ok(Self::not(nested(value)?)),
            "forall" => Ok(Self::Forall(Box::new(Self::quantifier(
                value, depth, binders, words,
            )?))),
            "exists" => Ok(Self::Exists(Box::new(Self::quantifier(
                value, depth, binders, words,
            )?))),
            // `as` is no operator, so a mapping holding it under `distinct` was never a constraint
            // on a fact of that name: `distinct: {in: [a, b]}` keeps its meaning.
            "distinct" if matches!(value, Node::Map(fields) if fields.contains_key("as")) => {
                if !source22_operands() {
                    return Err(ParseError::predicate(
                        &format!("distinct: {}", shallow(value)),
                        "`distinct: {in, as, by}` requires specification format ess/22",
                    ));
                }
                Self::distinct(value).map(|distinct| Self::Distinct(Box::new(distinct)))
            }
            "compare"
                if source22_operands()
                    && matches!(value, Node::Map(fields) if fields.contains_key("left")) =>
            {
                words.literal();
                Self::tagged_compare(value, binders)
            }
            // A calendar window (`docs/design/calendar-window-guards.md`). A mapping under `window`
            // without `at` is a constraint on a fact named `window`, as it always was; one with it
            // was never a constraint, `at` being no operator, and is refused below `ess/22` naming
            // the format that reads it.
            "window" if matches!(value, Node::Map(fields) if fields.contains_key("at")) => {
                if !source22_operands() {
                    return Err(ParseError::predicate(
                        &format!("window: {}", shallow(value)),
                        "a calendar window, `window: {at, days, from, to, offset}`, requires \
                         specification format ess/22",
                    ));
                }
                Ok(Self::Window(Box::new(
                    crate::window::CalendarWindow::parse_mapping(value)?,
                )))
            }
            "none" | "none_of_these" => {
                let children = value
                    .as_seq_or_single()
                    .into_iter()
                    .map(&mut nested)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Self::not(Self::any(children)))
            }
            path => {
                let path = FactPath::new(path).map_err(|error| {
                    ParseError::predicate(
                        path,
                        format!(
                            "{error}; expected a fact path or one of `all`, `any`, `not`, `none`, \
                             `forall`, `exists`"
                        ),
                    )
                })?;
                Self::from_constraint(path, value, binders, words)
            }
        }
    }

    /// Parses the body of a `forall:` or `exists:` entry.
    fn quantifier(
        value: &Node,
        depth: usize,
        binders: &[String],
        words: &mut Spellings,
    ) -> Result<Quantified, ParseError> {
        let Node::Map(entries) = value else {
            return Err(ParseError::shape(
                "quantifier",
                "a mapping with `in`, `as` and `that`",
                value.type_name(),
            ));
        };

        // The body is read with this binder in scope wherever `as` is written, so a bare word on
        // the right of a comparison naming it — or an outer binder — reads the binder rather than
        // the text (beyond10x/ess#289). A malformed `as` is refused below, in document order.
        let mut scope = binders.to_vec();
        if let Some(name) = entries
            .iter()
            .find(|(key, _)| key.as_str() == "as")
            .and_then(|(_, node)| Self::quantifier_binder(node).ok())
        {
            scope.push(name);
        }
        let mut over = None;
        let mut bind = None;
        let mut body = None;
        for (key, node) in entries {
            match key.as_str() {
                "in" => over = Some(Self::quantifier_collection(node)?),
                "as" => bind = Some(Self::quantifier_binder(node)?),
                "that" => body = Some(Self::from_node_nested(node, depth + 1, &scope, words)?),
                other => {
                    return Err(ParseError::predicate(
                        other,
                        "a quantifier takes `in`, `as` and `that`, and nothing else",
                    ));
                }
            }
        }

        let missing = |what: &str| {
            ParseError::shape(
                "quantifier",
                "`in`, `as` and `that`",
                format!("no `{what}`"),
            )
        };
        Ok(Quantified {
            over: over.ok_or_else(|| missing("in"))?,
            bind: bind.ok_or_else(|| missing("as"))?,
            body: body.ok_or_else(|| missing("that"))?,
        })
    }

    /// Parses the closed canonical `{compare: …}` form of a comparison: tagged to compare instants
    /// (decision 2), `{compare: {left: <path>, op: <keyword>, right: <operand>, as: timestamp}}`, or
    /// untagged with a derived operand (decision 11),
    /// `{compare: {left: {utf8_bytes: <path>}, op: <keyword>, right: <operand>}}`.
    ///
    /// Exactly those keys. `left` is a fact path or `{utf8_bytes: <path>}`; `right` is a fact path,
    /// the explicit `{fact: <path>}`, an operand mapping or a scalar; `as` is `timestamp`, the one
    /// kind a tag names, and never beside a derived operand, which compares as a number. The untagged
    /// form carries a derived operand: every other comparison has a form of its own. A mapping under
    /// `compare` without `left` is a constraint on a fact named `compare`, as it always was.
    fn tagged_compare(value: &Node, binders: &[String]) -> Result<Self, ParseError> {
        let written = || format!("compare: {}", shallow(value));
        let Node::Map(fields) = value else {
            unreachable!("dispatched on a mapping");
        };
        let refuse = |reason: &str| ParseError::predicate(&written(), reason.to_owned());
        let tagged = fields.contains_key("as");
        if fields.len() != if tagged { 4 } else { 3 }
            || ["left", "op", "right"]
                .iter()
                .any(|key| !fields.contains_key(*key))
        {
            return Err(refuse(
                "a `{compare: …}` comparison takes exactly `left`, `op` and `right`, and `as` \
                 where it is tagged",
            ));
        }
        let left = match &fields["left"] {
            Node::Text(path) => Operand::Fact(FactPath::new(path)?),
            Node::Map(entries)
                if entries.len() == 1 && entries.contains_key(Derived::UTF8_BYTES) =>
            {
                Operand::fact_mapping(entries, written)?
            }
            _ => return Err(refuse("`left` names a fact path or `{utf8_bytes: <path>}`")),
        };
        let op = match &fields["op"] {
            Node::Text(keyword) => CompareOp::from_keyword(keyword)
                .ok_or_else(|| refuse("`op` is one of eq, ne, lt, lte, gt, gte"))?,
            _ => return Err(refuse("`op` is one of eq, ne, lt, lte, gt, gte")),
        };
        let right = match &fields["right"] {
            Node::Text(text) => Operand::parse_in(text, binders),
            Node::Bool(value) => Operand::Literal(FactValue::Bool(*value)),
            Node::Number(number) => Operand::Literal(FactValue::Number(*number)),
            Node::Map(entries) => Operand::fact_mapping(entries, written)?,
            _ => return Err(refuse("`right` is a scalar or `{fact: <path>}`")),
        };
        let derived = [&left, &right]
            .into_iter()
            .any(|operand| matches!(operand, Operand::Derived(_)));
        if !tagged {
            if !derived {
                return Err(refuse(
                    "an untagged `{compare: …}` carries a derived operand, `{utf8_bytes: <path>}`; \
                     compare two facts as `<path>: {<op>: …}`",
                ));
            }
            return Ok(Self::Compare {
                left,
                op,
                right,
                kind: CompareKind::Value,
            });
        }
        if fields["as"] != Node::Text(CompareKind::TIMESTAMP.to_owned()) {
            return Err(refuse(
                "`as` names the one kind a comparison is tagged with, `timestamp`",
            ));
        }
        if derived || !matches!(left, Operand::Fact(_)) {
            return Err(refuse(
                "a derived operand compares as a number, never `as: timestamp`",
            ));
        }
        Ok(Self::Compare {
            left,
            op,
            right,
            kind: CompareKind::Instant,
        })
    }

    /// Parses the body of a `distinct:` entry: `{in: <list>, as: <binder>, by: <binder>.<member>,
    /// kind: <key kind>}`, `by` and `kind` optional. A source leaves `kind` to the domain's resolver;
    /// the canonical form writes it, and a suite reader requires it.
    fn distinct(value: &Node) -> Result<Distinct, ParseError> {
        let Node::Map(entries) = value else {
            unreachable!("dispatched on a mapping");
        };
        let written = || format!("distinct: {}", shallow(value));
        let refuse = |reason: String| ParseError::predicate(&written(), reason);
        let mut over = None;
        let mut bind = None;
        let mut key = None;
        let mut key_kind = None;
        for (field, node) in entries {
            match field.as_str() {
                "in" => over = Some(Self::quantifier_collection(node)?),
                "as" => bind = Some(Self::quantifier_binder(node)?),
                "by" => {
                    let text = node.as_text().ok_or_else(|| {
                        refuse("`by` names one member of the element, a fact path".to_owned())
                    })?;
                    key = Some(FactPath::new(text)?);
                }
                "kind" => {
                    key_kind = Some(
                        node.as_text()
                            .and_then(DistinctKeyKind::from_keyword)
                            .ok_or_else(|| {
                                refuse(format!(
                                    "`kind` is one of {}",
                                    DistinctKeyKind::ALL
                                        .iter()
                                        .map(|kind| kind.keyword())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ))
                            })?,
                    );
                }
                other => {
                    return Err(refuse(format!(
                        "`{other}`: `distinct` takes `in`, `as`, `by` and `kind`, and nothing else"
                    )));
                }
            }
        }
        let (Some(over), Some(bind)) = (over, bind) else {
            return Err(ParseError::shape(
                "distinct",
                "`in` and `as`, with an optional `by`",
                "no `in`",
            ));
        };
        if let Some(key) = &key {
            if key.namespace() != bind || key.segments().len() < 2 {
                return Err(refuse(format!(
                    "`by: {key}` names one member under the binder, such as `{bind}.path`; without \
                     `by` the element itself is the key"
                )));
            }
        }
        Ok(Distinct {
            over,
            bind,
            key,
            key_kind,
        })
    }

    /// Parses the `in:` of a quantifier: the collection it walks.
    fn quantifier_collection(node: &Node) -> Result<FactPath, ParseError> {
        let text = node
            .as_text()
            .ok_or_else(|| ParseError::shape("quantifier `in`", "a fact path", node.type_name()))?;
        FactPath::new(text)
    }

    /// Parses the `as:` of a quantifier: the name its body binds each element to.
    ///
    /// One segment, because a binder is a name and not a path — `as: slot.left` would read as a
    /// claim about where the element comes from, which is what `in:` already says.
    fn quantifier_binder(node: &Node) -> Result<String, ParseError> {
        let text = node
            .as_text()
            .ok_or_else(|| ParseError::shape("quantifier `as`", "a name", node.type_name()))?;
        let path = FactPath::new(text)?;
        if path.segments().len() != 1 {
            return Err(ParseError::identifier(
                "quantifier binder",
                text,
                "must be one segment; a binder names an element, it does not path into one"
                    .to_owned(),
            ));
        }
        Ok(text.to_owned())
    }

    /// Parses the constraint attached to a fact path in mapping form.
    fn from_constraint(
        path: FactPath,
        value: &Node,
        binders: &[String],
        words: &mut Spellings,
    ) -> Result<Self, ParseError> {
        // The equality shorthand is always a literal, so no right-hand side of it is a word.
        let shorthand = |right: FactValue, words: &mut Spellings| {
            words.literal();
            Ok(Self::Compare {
                kind: CompareKind::Value,
                left: Operand::Fact(path.clone()),
                op: CompareOp::Eq,
                right: Operand::Literal(right),
            })
        };
        match value {
            Node::Bool(expected) => shorthand(FactValue::Bool(*expected), words),
            Node::Number(number) => shorthand(FactValue::Number(*number), words),
            Node::Text(text) => shorthand(FactValue::parse_literal(text), words),
            Node::Seq(items) => Ok(Self::AnyOf {
                path,
                values: literal_values(items)?,
            }),
            Node::Map(entries) => {
                let mut children = Vec::new();
                for (operator, operand) in entries {
                    children.push(Self::from_operator(
                        path.clone(),
                        operator,
                        operand,
                        binders,
                        words,
                    )?);
                }
                Ok(Self::all(children))
            }
            // `note: null` is `note == null` in mapping form, and is refused as one.
            Node::Null => Err(ParseError::NullComparison {
                expression: format!("{path}: null"),
                path: path.to_string(),
                operator: CompareOp::Eq.as_str().to_owned(),
                spelling: "null".to_owned(),
            }),
        }
    }

    /// Parses one operator constraint, such as `any_of` or `gte`.
    fn from_operator(
        path: FactPath,
        operator: &str,
        operand: &Node,
        binders: &[String],
        words: &mut Spellings,
    ) -> Result<Self, ParseError> {
        if let Some(op) = CompareOp::from_keyword(operator) {
            let right = match operand {
                Node::Text(text) => Operand::parse_in(text, binders),
                Node::Bool(value) => Operand::Literal(FactValue::Bool(*value)),
                Node::Number(number) => Operand::Literal(FactValue::Number(*number)),
                Node::Map(entries) => {
                    Operand::fact_mapping(entries, || format!("{path}: {{{operator}: {operand}}}"))?
                }
                Node::Null => {
                    return Err(ParseError::NullComparison {
                        expression: format!("{path}: {{{operator}: null}}"),
                        path: path.to_string(),
                        operator: op.as_str().to_owned(),
                        spelling: "null".to_owned(),
                    })
                }
                other @ Node::Seq(_) => {
                    return Err(ParseError::predicate(
                        &format!("{path}: {{{operator}: {other}}}"),
                        "a comparison operand must be a scalar",
                    ))
                }
            };
            match operand {
                Node::Text(text) => words.written(text, binders),
                _ => words.literal(),
            }
            return Ok(Self::Compare {
                kind: CompareKind::Value,
                left: Operand::Fact(path),
                op,
                right,
            });
        }

        match operator {
            "any_of" | "in" | "one_of" => Ok(Self::AnyOf {
                path,
                values: literal_values(&operand.as_seq_or_single_owned())?,
            }),
            "none_of" | "not_in" => Ok(Self::NoneOf {
                path,
                values: literal_values(&operand.as_seq_or_single_owned())?,
            }),
            "exists" | "defined" => {
                let expected = match operand {
                    Node::Bool(value) => *value,
                    other => {
                        return Err(ParseError::predicate(
                            &format!("{path}: {{{operator}: {other}}}"),
                            "`exists` takes a boolean",
                        ))
                    }
                };
                let defined = Self::Defined(path);
                Ok(if expected {
                    defined
                } else {
                    Self::not(defined)
                })
            }
            "truthy" => Ok(Self::Truthy(path)),
            "starts_with" | "ends_with" | "contains" => Self::text_match(path, operator, operand),
            "equals_ignore_case" | "in_ignore_case" => Self::fold_match(path, operator, operand),
            unknown => Err(ParseError::predicate(
                &format!("{path}: {{{unknown}: …}}"),
                format!(
                    "unknown operator {unknown:?}; expected one of eq, ne, lt, lte, gt, gte, \
                     any_of, none_of, exists, truthy, starts_with, ends_with, contains, \
                     equals_ignore_case, in_ignore_case"
                ),
            )),
        }
    }

    /// Parses a string operator's operand: a text is the literal byte for byte, never read through
    /// [`Operand::parse`], so `"+44"` stays text and `"a.b"` is no fact path. A number or a Boolean
    /// is kept as that value, for validation to refuse with a code and a site; anything else is not
    /// a scalar.
    ///
    /// From `ess/22` the operand may instead be the mapping `{param: <name>}` or `{input: <name>}`
    /// (beyond10x/ess#200): one key, one top-level name. Below it a mapping is refused in the words
    /// it always was.
    fn text_match(path: FactPath, operator: &str, operand: &Node) -> Result<Self, ParseError> {
        let op = TextOp::from_keyword(operator).expect("dispatched on a string operator keyword");
        let written = || format!("{path}: {{{operator}: {operand}}}");
        let value = match operand {
            Node::Text(text) => FactValue::Text(text.clone()).into(),
            Node::Number(number) => FactValue::Number(*number).into(),
            Node::Bool(value) => FactValue::Bool(*value).into(),
            Node::Map(entries) if source22_operands() => match entries.iter().next() {
                Some((key, Node::Text(name))) if entries.len() == 1 => {
                    let namespace = TextNamespace::from_keyword(key)
                        .ok_or_else(|| ParseError::predicate(&written(), TEXT_OPERAND_SHAPE))?;
                    TextOperand::fact(namespace, name)
                        .map_err(|_| ParseError::predicate(&written(), TEXT_OPERAND_SHAPE))?
                }
                _ => return Err(ParseError::predicate(&written(), TEXT_OPERAND_SHAPE)),
            },
            _ => {
                return Err(ParseError::predicate(
                    &written(),
                    "a comparison operand must be a scalar",
                ))
            }
        };
        Ok(Self::TextMatch { path, op, value })
    }

    /// Parses a case-insensitive operator's operand (beyond10x/ess#140): one scalar under
    /// `equals_ignore_case`, a list of scalars under `in_ignore_case`, each text read byte for byte
    /// as a string operator's is. A number or a Boolean is kept for validation to refuse with a
    /// code and a site. The shapes are not interchangeable, so a list under the one and a scalar
    /// under the other are refused here rather than read as the other operator.
    fn fold_match(path: FactPath, operator: &str, operand: &Node) -> Result<Self, ParseError> {
        let op = FoldOp::from_keyword(operator).expect("dispatched on a fold operator keyword");
        let scalar = |node: &Node| match node {
            Node::Text(text) => Ok(FactValue::Text(text.clone())),
            Node::Number(number) => Ok(FactValue::Number(*number)),
            Node::Bool(value) => Ok(FactValue::Bool(*value)),
            other => Err(ParseError::predicate(
                &format!("{path}: {{{operator}: {other}}}"),
                "a comparison operand must be a scalar",
            )),
        };
        let values =
            match (op, operand) {
                (FoldOp::InIgnoreCase, Node::Seq(items)) => {
                    items.iter().map(scalar).collect::<Result<_, _>>()?
                }
                (FoldOp::InIgnoreCase, other) => {
                    return Err(ParseError::predicate(
                        &format!("{path}: {{{operator}: {other}}}"),
                        "`in_ignore_case` takes a list of text literals",
                    ))
                }
                (FoldOp::EqualsIgnoreCase, Node::Seq(_)) => return Err(ParseError::predicate(
                    &format!("{path}: {{{operator}: {operand}}}"),
                    "`equals_ignore_case` takes one text literal; use `in_ignore_case` for a list",
                )),
                (FoldOp::EqualsIgnoreCase, other) => vec![scalar(other)?],
            };
        Ok(Self::FoldMatch { path, op, values })
    }

    /// Parses the compact string form of a predicate.
    ///
    /// Refuses a `not` prefix stacked deeper than [`MAX_PREDICATE_DEPTH`] with
    /// [`ParseError::TooDeep`]. This is the string half of the same budget
    /// [`Self::from_node`] spends.
    pub fn parse_expression(expression: &str) -> Result<Self, ParseError> {
        Self::parse_expression_nested(expression, 0, &[], &mut Spellings::default())
    }

    /// [`Self::parse_expression`], counting how deep it already is.
    fn parse_expression_nested(
        expression: &str,
        depth: usize,
        binders: &[String],
        words: &mut Spellings,
    ) -> Result<Self, ParseError> {
        let trimmed = expression.trim();
        if depth > MAX_PREDICATE_DEPTH {
            return Err(ParseError::too_deep(
                "predicate",
                trimmed,
                MAX_PREDICATE_DEPTH,
            ));
        }
        if trimmed.is_empty() {
            return Err(ParseError::predicate(expression, "expression is empty"));
        }
        match trimmed {
            "always" | "true" => return Ok(Self::Always),
            "never" | "false" => return Ok(Self::Never),
            _ => {}
        }
        if let Some(rest) = trimmed.strip_prefix("not ") {
            return Ok(Self::not(Self::parse_expression_nested(
                rest,
                depth + 1,
                binders,
                words,
            )?));
        }
        for (function, negate) in [("defined", false), ("exists", false), ("missing", true)] {
            if let Some(inner) = call_argument(trimmed, function) {
                let path = FactPath::new(inner).map_err(|error| {
                    ParseError::predicate(expression, format!("{function}() argument: {error}"))
                })?;
                let predicate = Self::Defined(path);
                return Ok(if negate {
                    Self::not(predicate)
                } else {
                    predicate
                });
            }
        }

        if let Some((left, op, right)) = split_comparison(trimmed) {
            // Before either side is read as a path or a literal: `null == note` and `~ == note`
            // are the same mistake as `note == null`, and each would otherwise be refused, or
            // admitted, for a reason that has nothing to do with it.
            for (compared, other) in [(left, right), (right, left)] {
                if NULL_SPELLINGS.contains(&other.trim()) {
                    return Err(ParseError::NullComparison {
                        expression: expression.to_owned(),
                        path: compared.trim().to_owned(),
                        operator: op.as_str().to_owned(),
                        spelling: other.trim().to_owned(),
                    });
                }
            }
            let left_path = FactPath::new(left.trim()).map_err(|error| {
                ParseError::predicate(
                    expression,
                    format!("left-hand side must be a fact path: {error}"),
                )
            })?;
            if right.trim().is_empty() {
                return Err(ParseError::predicate(
                    expression,
                    format!("nothing to compare against after `{op}`"),
                ));
            }
            validate_quoted_operand(right, expression)?;
            if !right.trim_start().starts_with(['"', '\''])
                && right
                    .split_whitespace()
                    .any(|token| COMBINATORS.contains(&token))
            {
                let operand = right.trim();
                return Err(ParseError::predicate(
                    expression,
                    format!(
                        "`&&` and `||` are not part of the compact form, so `{operand}` is not one \
                         operand; use structured all/any/not to combine predicates, or quote \
                         \"{operand}\" to compare with that text"
                    ),
                ));
            }
            words.written(right, binders);
            return Ok(Self::Compare {
                kind: CompareKind::Value,
                left: Operand::Fact(left_path),
                op,
                right: Operand::parse_in(right, binders),
            });
        }

        let path = FactPath::new(trimmed).map_err(|error| {
            ParseError::predicate(
                expression,
                format!(
                    "{error}; a predicate is either a comparison (`a.b == 0`) or a bare fact path"
                ),
            )
        })?;
        Ok(Self::Truthy(path))
    }

    /// Whether this predicate needs the lossless structured text-comparison writer.
    ///
    /// This detects only literals whose historical compact spelling fails to preserve the typed
    /// comparison, including beneath Boolean composition and quantifiers. Ordinary structured
    /// predicates do not require this writer. Reader version selection uses the narrower
    /// [`Self::requires_lossless_text_reader`] capability.
    pub fn requires_structured_text_comparison(&self) -> bool {
        match self {
            Self::Compare {
                left: Operand::Fact(_),
                right: Operand::Literal(FactValue::Text(_)),
                ..
            } => !compact_comparison_roundtrips(self, &self.to_string()),
            Self::All(children) | Self::Any(children) => children
                .iter()
                .any(Self::requires_structured_text_comparison),
            Self::Not(inner) => inner.requires_structured_text_comparison(),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                quantified.body.requires_structured_text_comparison()
            }
            _ => false,
        }
    }

    /// Whether canonical output requires a reader that normalizes structured text operands.
    ///
    /// A fallback scalar identical to the literal remains compatible with the historical Go
    /// reader. Only a fallback needing protective quotes requires the corrected reader.
    pub fn requires_lossless_text_reader(&self) -> bool {
        match self {
            Self::Compare {
                left: Operand::Fact(_),
                right: Operand::Literal(FactValue::Text(text)),
                ..
            } => {
                self.requires_structured_text_comparison()
                    && Operand::parse(text) != Operand::Literal(FactValue::Text(text.clone()))
            }
            Self::All(children) | Self::Any(children) => {
                children.iter().any(Self::requires_lossless_text_reader)
            }
            Self::Not(inner) => inner.requires_lossless_text_reader(),
            Self::Forall(quantified) | Self::Exists(quantified) => {
                quantified.body.requires_lossless_text_reader()
            }
            _ => false,
        }
    }

    /// Renders this predicate back into document form.
    ///
    /// A one-segment fact on the right of a comparison that no binder in scope names is written as
    /// the explicit operand `{fact: …}` (`docs/design/expression-family-source22.md`, A1): its
    /// compact spelling reads back as text. A binder and a dotted path keep their compact bytes.
    pub fn to_node(&self) -> Node {
        self.node_in(&mut Vec::new())
    }

    #[allow(clippy::too_many_lines)]
    fn node_in<'a>(&'a self, binders: &mut Vec<&'a str>) -> Node {
        let seq = |children: &'a [Self], binders: &mut Vec<&'a str>| {
            let mut nodes = Vec::with_capacity(children.len());
            for child in children {
                nodes.push(child.node_in(binders));
            }
            Node::Seq(nodes)
        };
        match self {
            Self::Always => Node::Bool(true),
            Self::Never => Node::Bool(false),
            Self::All(children) => Node::Map(
                [("all".to_owned(), seq(children, binders))]
                    .into_iter()
                    .collect(),
            ),
            Self::Any(children) => Node::Map(
                [("any".to_owned(), seq(children, binders))]
                    .into_iter()
                    .collect(),
            ),
            Self::Not(inner) => Node::Map(
                [("not".to_owned(), inner.node_in(binders))]
                    .into_iter()
                    .collect(),
            ),
            Self::AnyOf { path, values } => values_constraint_node(path, "any_of", values),
            Self::NoneOf { path, values } => values_constraint_node(path, "none_of", values),
            Self::Forall(quantified) => quantifier_node("forall", quantified, binders),
            Self::Exists(quantified) => quantifier_node("exists", quantified, binders),
            // Explicit: there is no compact form. `kind` is written once resolved and left out
            // before, so a source document reads back as written.
            Self::Distinct(distinct) => distinct.to_node(),
            // Explicit: a window has no compact form.
            Self::Window(window) => Node::Map([("window".to_owned(), window.to_node())].into()),
            // Explicit, never the compact fallback below: there is no compact form, so a string
            // operator rendered as text would be a document no reader parses back. A text operand
            // is written as the text, with no quotes added, because the reader takes it verbatim; a
            // parameter or an input as its mapping, `{param: <name>}` (beyond10x/ess#200).
            Self::TextMatch { path, op, value } => Node::Map(
                [(
                    path.to_string(),
                    Node::Map([(op.keyword().to_owned(), value.to_node())].into()),
                )]
                .into(),
            ),
            // Explicit for the same reason: there is no compact form. `equals_ignore_case` carries
            // exactly one value by construction; should a caller build it with another count, the
            // list is written, which the reader refuses rather than reading as something else.
            Self::FoldMatch { path, op, values } => {
                let operand = match (op, values.as_slice()) {
                    (FoldOp::EqualsIgnoreCase, [only]) => value_node(only),
                    _ => Node::Seq(values.iter().map(value_node).collect()),
                };
                Node::Map(
                    [(
                        path.to_string(),
                        Node::Map([(op.keyword().to_owned(), operand)].into()),
                    )]
                    .into(),
                )
            }
            Self::Compare {
                left,
                op,
                right,
                kind: CompareKind::Instant,
            } => tagged_comparison_node(left, *op, right),
            Self::Compare {
                left,
                op,
                right,
                kind: CompareKind::Value,
            } if [left, right]
                .into_iter()
                .any(|operand| matches!(operand, Operand::Derived(_))) =>
            {
                derived_comparison_node(left, *op, right)
            }
            // Always the closed mapping: `lower + 5` reads back as text wherever no declaration
            // says `lower` is a fact (A2).
            Self::Compare {
                left: Operand::Fact(path),
                op,
                right: Operand::Offset(offset),
                kind: CompareKind::Value,
            } => Node::Map(
                [(
                    path.to_string(),
                    Node::Map([(op.keyword().to_owned(), offset.to_node())].into()),
                )]
                .into(),
            ),
            Self::Compare {
                left: Operand::Fact(path),
                op,
                right: Operand::Literal(FactValue::Text(text)),
                kind: CompareKind::Value,
            } => comparison_text_node(self, path, *op, text),
            Self::Compare {
                left: Operand::Fact(path),
                op,
                right,
                kind: CompareKind::Value,
            } if right.root_fact(binders).is_some() => {
                fact_comparison_node(path, *op, right.root_fact(binders).expect("the guard"))
            }
            leaf => Node::Text(
                InScope {
                    predicate: leaf,
                    binders,
                }
                .to_string(),
            ),
        }
    }
}

/// `path: {<keyword>: [values…]}`: a value-list constraint.
fn values_constraint_node(path: &FactPath, keyword: &str, values: &[FactValue]) -> Node {
    Node::Map(
        [(
            path.to_string(),
            Node::Map(
                [(
                    keyword.to_owned(),
                    Node::Seq(values.iter().map(value_node).collect()),
                )]
                .into_iter()
                .collect(),
            ),
        )]
        .into_iter()
        .collect(),
    )
}

/// `{compare: {left, op, right, as: timestamp}}`: the closed canonical form of a tagged comparison
/// (decision 2). A fact on the right is always the explicit `{fact: …}`, so the form never depends
/// on what a binder or a root is called.
fn tagged_comparison_node(left: &Operand, op: CompareOp, right: &Operand) -> Node {
    compare_node(left, op, right, Some(CompareKind::TIMESTAMP))
}

/// A comparison holding a derived operand (decision 11): on the right of a fact it is the tagged
/// operand where an operand goes, `path: {<op>: {utf8_bytes: …}}`; anywhere else the comparison
/// takes the untagged `{compare: …}` form, because a mapping cannot stand where the compact grammar
/// wants a path.
fn derived_comparison_node(left: &Operand, op: CompareOp, right: &Operand) -> Node {
    match (left, right) {
        (Operand::Fact(path), Operand::Derived(derived)) => Node::Map(
            [(
                path.to_string(),
                Node::Map([(op.keyword().to_owned(), derived.to_node())].into()),
            )]
            .into(),
        ),
        _ => compare_node(left, op, right, None),
    }
}

/// `{compare: {left, op, right}}`, and `as: <tag>` where the comparison is tagged: the closed
/// canonical `{compare: …}` form (decisions 2 and 11). A fact on the left is its path and on the
/// right always the explicit `{fact: …}`; an offset and a derived operand are their mappings.
fn compare_node(left: &Operand, op: CompareOp, right: &Operand, tag: Option<&str>) -> Node {
    let operand = |operand: &Operand| match operand {
        Operand::Fact(path) => {
            Node::Map([("fact".to_owned(), Node::Text(path.to_string()))].into())
        }
        Operand::Offset(offset) => offset.to_node(),
        Operand::Derived(derived) => derived.to_node(),
        Operand::Literal(value) => value_node(value),
    };
    let left = match left {
        Operand::Fact(path) => Node::Text(path.to_string()),
        other => operand(other),
    };
    let mut fields: std::collections::BTreeMap<String, Node> = [
        ("left".to_owned(), left),
        ("op".to_owned(), Node::Text(op.keyword().to_owned())),
        ("right".to_owned(), operand(right)),
    ]
    .into();
    if let Some(tag) = tag {
        fields.insert("as".to_owned(), Node::Text(tag.to_owned()));
    }
    Node::Map([("compare".to_owned(), Node::Map(fields))].into())
}

/// `path: {<op>: {fact: <fact>}}`: the canonical comparison with a one-segment fact on its right
/// (`docs/design/expression-family-source22.md`, A1).
fn fact_comparison_node(path: &FactPath, op: CompareOp, fact: &FactPath) -> Node {
    let operand = Node::Map([("fact".to_owned(), Node::Text(fact.to_string()))].into());
    Node::Map(
        [(
            path.to_string(),
            Node::Map([(op.keyword().to_owned(), operand)].into()),
        )]
        .into(),
    )
}

/// Preserve legacy compact bytes when they preserve the typed literal. Otherwise use the existing
/// structured operator form, whose scalar is not a compact expression. Wrapping that scalar in
/// quotes protects values such as `true` and dotted text without decoding any escape bytes.
fn comparison_text_node(predicate: &Predicate, path: &FactPath, op: CompareOp, text: &str) -> Node {
    let compact = predicate.to_string();
    if compact_comparison_roundtrips(predicate, &compact) {
        return Node::Text(compact);
    }
    let scalar = if Operand::parse(text) == Operand::Literal(FactValue::Text(text.to_owned())) {
        text.to_owned()
    } else {
        format!("\"{text}\"")
    };
    Node::Map(
        [(
            path.to_string(),
            Node::Map([(op.as_str().to_owned(), Node::Text(scalar))].into()),
        )]
        .into(),
    )
}

fn compact_comparison_roundtrips(predicate: &Predicate, compact: &str) -> bool {
    Predicate::parse_expression(compact).is_ok_and(|reread| reread == *predicate)
}

/// Renders a quantifier back into the mapping it was written as.
///
/// Explicit rather than falling through to the string arm below, because there is no string form
/// to fall through to: a quantifier that rendered as text would be a document this parser refuses
/// to read back.
fn quantifier_node<'a>(
    keyword: &str,
    quantified: &'a Quantified,
    binders: &mut Vec<&'a str>,
) -> Node {
    binders.push(&quantified.bind);
    let body = quantified.body.node_in(binders);
    binders.pop();
    Node::Map(
        [(
            keyword.to_owned(),
            Node::Map(
                [
                    ("in".to_owned(), Node::Text(quantified.over.to_string())),
                    ("as".to_owned(), Node::Text(quantified.bind.clone())),
                    ("that".to_owned(), body),
                ]
                .into_iter()
                .collect(),
            ),
        )]
        .into_iter()
        .collect(),
    )
}

/// Converts a fact value into a document node.
fn value_node(value: &FactValue) -> Node {
    match value {
        FactValue::Bool(inner) => Node::Bool(*inner),
        FactValue::Number(inner) => Node::Number(*inner),
        FactValue::Text(inner) => Node::Text(inner.clone()),
    }
}

/// Parses a list of literal values.
fn literal_values(items: &[Node]) -> Result<Vec<FactValue>, ParseError> {
    items
        .iter()
        .map(|item| match item {
            Node::Bool(value) => Ok(FactValue::Bool(*value)),
            Node::Number(value) => Ok(FactValue::Number(*value)),
            Node::Text(value) => Ok(FactValue::parse_literal(value)),
            other => Err(ParseError::shape(
                "predicate value list",
                "a scalar",
                other.type_name(),
            )),
        })
        .collect()
}

impl Node {
    /// This node as a list of owned nodes, treating a scalar as a one-element list.
    fn as_seq_or_single_owned(&self) -> Vec<Node> {
        self.as_seq_or_single().into_iter().cloned().collect()
    }
}

/// Extracts `argument` from `function(argument)`.
fn call_argument<'a>(expression: &'a str, function: &str) -> Option<&'a str> {
    let rest = expression.strip_prefix(function)?;
    let rest = rest.trim_start().strip_prefix('(')?;
    rest.trim_end().strip_suffix(')').map(str::trim)
}

/// A compact quoted operand ends at its first unescaped matching quote. Its contents remain
/// verbatim, as in `FactValue::parse_literal`; escapes only determine the closing boundary.
/// Structured scalar operands are data and deliberately do not pass through this check.
fn validate_quoted_operand(raw: &str, expression: &str) -> Result<(), ParseError> {
    let trimmed = raw.trim();
    let Some(quote @ (b'"' | b'\'')) = trimmed.as_bytes().first().copied() else {
        return Ok(());
    };
    let mut escaped = false;
    for (index, byte) in trimmed.bytes().enumerate().skip(1) {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return if trimmed[index + 1..].trim().is_empty() {
                Ok(())
            } else {
                Err(ParseError::predicate(
                    expression,
                    "tokens after a quoted operand are unsupported; use structured any/all/not",
                ))
            };
        }
    }
    Err(ParseError::predicate(
        expression,
        "quoted operand is not closed; close the quote and use structured any/all/not to combine predicates",
    ))
}

/// Splits an expression at its first top-level comparison operator.
fn split_comparison(expression: &str) -> Option<(&str, CompareOp, &str)> {
    let bytes = expression.as_bytes();
    let mut quote: Option<u8> = None;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            Some(open) => {
                if byte == open {
                    quote = None;
                }
            }
            None => {
                if byte == b'"' || byte == b'\'' {
                    quote = Some(byte);
                } else {
                    let two = expression.get(index..index + 2);
                    if let Some(op) = two.and_then(|slice| match slice {
                        "==" => Some(CompareOp::Eq),
                        "!=" => Some(CompareOp::Ne),
                        "<=" => Some(CompareOp::Le),
                        ">=" => Some(CompareOp::Ge),
                        _ => None,
                    }) {
                        return Some((&expression[..index], op, &expression[index + 2..]));
                    }
                    if byte == b'<' {
                        return Some((
                            &expression[..index],
                            CompareOp::Lt,
                            &expression[index + 1..],
                        ));
                    }
                    if byte == b'>' {
                        return Some((
                            &expression[..index],
                            CompareOp::Gt,
                            &expression[index + 1..],
                        ));
                    }
                }
            }
        }
        index += 1;
    }
    None
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        InScope {
            predicate: self,
            binders: &[],
        }
        .fmt(f)
    }
}

/// A predicate rendered for a reader inside the quantifier bodies whose binders are `binders`.
///
/// A one-segment fact on the right of a comparison that no binder names reads `a == {fact: b}`, so
/// the rendering of a comparison of two facts never equals the rendering of a comparison with the
/// text `b` (`docs/design/expression-family-source22.md`, A1, decision 7). A binder reads as
/// itself, as it always did.
struct InScope<'a, 'b> {
    predicate: &'a Predicate,
    binders: &'b [&'a str],
}

impl fmt::Display for InScope<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let binders = self.binders;
        let nested = |predicate| InScope { predicate, binders };
        match self.predicate {
            Predicate::Always => f.write_str("always"),
            Predicate::Never => f.write_str("never"),
            Predicate::All(children) => write_joined(f, children, binders, " and "),
            Predicate::Any(children) => write_joined(f, children, binders, " or "),
            Predicate::Not(inner) => write!(f, "not ({})", nested(inner)),
            Predicate::Compare {
                left,
                op,
                right,
                kind,
            } => {
                match right.root_fact(binders) {
                    Some(fact) => write!(f, "{left} {op} {{fact: {fact}}}")?,
                    None => write!(f, "{left} {op} {right}")?,
                }
                match kind {
                    CompareKind::Instant => write!(f, " as {}", CompareKind::TIMESTAMP),
                    CompareKind::Value => Ok(()),
                }
            }
            Predicate::Truthy(path) => write!(f, "{path}"),
            Predicate::Defined(path) => write!(f, "defined({path})"),
            Predicate::AnyOf { path, values } => {
                write!(f, "{path} in [{}]", join_values(values))
            }
            Predicate::NoneOf { path, values } => {
                write!(f, "{path} not in [{}]", join_values(values))
            }
            // For a reader and the semantic diff, never read back: a text literal is quoted by
            // `Debug`, a number or a Boolean is bare, a parameter or an input is its mapping.
            Predicate::TextMatch { path, op, value } => write!(f, "{path} {op} {value}"),
            // For a reader and the semantic diff, never read back: each text quoted by `Debug`.
            Predicate::FoldMatch { path, op, values } => {
                let quoted = |value: &FactValue| match value {
                    FactValue::Text(text) => format!("{text:?}"),
                    other => other.to_string(),
                };
                match (op, values.as_slice()) {
                    (FoldOp::EqualsIgnoreCase, [only]) => write!(f, "{path} {op} {}", quoted(only)),
                    _ => write!(
                        f,
                        "{path} {op} [{}]",
                        values.iter().map(quoted).collect::<Vec<_>>().join(", ")
                    ),
                }
            }
            Predicate::Forall(quantified) => write_quantified(f, "forall", quantified, binders),
            Predicate::Exists(quantified) => write_quantified(f, "exists", quantified, binders),
            // For a reader and the semantic diff, never read back.
            Predicate::Distinct(distinct) => {
                write!(f, "distinct {} in {}", distinct.bind, distinct.over)?;
                if let Some(key) = &distinct.key {
                    write!(f, " by {key}")?;
                }
                match distinct.key_kind {
                    Some(kind) => write!(f, " as {kind}"),
                    None => Ok(()),
                }
            }
            Predicate::Window(window) => write!(f, "{window}"),
        }
    }
}

/// Renders a quantifier for a human: `forall slot in slots: (…)`.
fn write_quantified(
    f: &mut fmt::Formatter<'_>,
    keyword: &str,
    quantified: &Quantified,
    binders: &[&str],
) -> fmt::Result {
    let mut scope = binders.to_vec();
    scope.push(&quantified.bind);
    write!(
        f,
        "{keyword} {} in {}: ({})",
        quantified.bind,
        quantified.over,
        InScope {
            predicate: &quantified.body,
            binders: &scope,
        }
    )
}

/// Renders children joined by `separator`, parenthesised.
fn write_joined(
    f: &mut fmt::Formatter<'_>,
    children: &[Predicate],
    binders: &[&str],
    separator: &str,
) -> fmt::Result {
    f.write_str("(")?;
    for (index, child) in children.iter().enumerate() {
        if index > 0 {
            f.write_str(separator)?;
        }
        write!(
            f,
            "{}",
            InScope {
                predicate: child,
                binders,
            }
        )?;
    }
    f.write_str(")")
}

/// Renders a comma-separated value list.
fn join_values(values: &[FactValue]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

impl std::str::FromStr for Predicate {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_expression(value)
    }
}

impl serde::Serialize for Predicate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_node().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Predicate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let node = Node::deserialize(deserializer)?;
        Self::from_node(&node).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for Predicate {
    fn schema_name() -> String {
        "Predicate".to_owned()
    }

    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject::default();
        schema.subschemas().any_of = Some(vec![
            <String>::json_schema(generator),
            <bool>::json_schema(generator),
            <Vec<Node>>::json_schema(generator),
            <std::collections::BTreeMap<String, Node>>::json_schema(generator),
        ]);
        schema.metadata().description = Some(
            "A condition over facts: the compact expression form (`tests.unit.failed == 0`), a \
             list (implicit `all`), or a mapping using `all`, `any`, `not`, `none`, `forall`, \
             `exists` or a fact path with an operator constraint. A text fact is tested against a \
             text literal, map form only, with `starts_with`, `ends_with` or `contains`, and \
             without ASCII case with `equals_ignore_case` (one literal) or `in_ignore_case` (a \
             list). From `ess/22` a comparison operand may be the explicit fact `{fact: <path>}`, \
             two `Timestamp` facts compare as instants in the closed form `{compare: {left, op, \
             right, as: timestamp}}`, and a comparison operand may be one fact moved by one \
             constant, `{offset: {fact: <path>, add|subtract: <magnitude>}}` — a whole number for \
             an `Integer`, a whole number of `s`, `m` or `h` for a `Timestamp`. Either operand may \
             be the UTF-8 byte length of a `String`, `{utf8_bytes: <path>}`, with the comparison \
             written `{compare: {left, op, right}}` where it stands on the left. From `ess/22` \
             `distinct: {in: <list>, as: <name>, by: <name>.<member>}` holds when no two elements \
             of a list share a key: the element, or the one scalar member `by` names. A \
             string operator may compare with a view parameter, `{param: <name>}`, or a command \
             input, `{input: <name>}`. A command guard may hold an instant to a calendar \
             window at UTC or a fixed offset, `{window: {at: now | <path>, days: [mon, …], \
             from: \"HH:MM\", to: \"HH:MM\", offset: Z | ±HH:MM}}`; a named time zone is \
             refused."
                .to_owned(),
        );
        schema.into()
    }
}

/// The result of evaluating a predicate, with the reason when it is not satisfied.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct PredicateOutcome {
    /// The predicate as written, in canonical compact form.
    pub expression: String,
    /// The result.
    pub truth: Truth,
    /// The leaves responsible for a non-satisfied result, empty when satisfied.
    pub causes: Vec<LeafOutcome>,
}

impl PredicateOutcome {
    /// `true` when the predicate holds.
    pub fn is_satisfied(&self) -> bool {
        self.truth.is_satisfied()
    }

    /// Fact paths that nothing has observed yet.
    pub fn missing_facts(&self) -> Vec<&FactPath> {
        let mut paths = Vec::new();
        for cause in &self.causes {
            for path in &cause.missing {
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
        paths
    }
}

/// One leaf condition that prevented a predicate from being satisfied.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
pub struct LeafOutcome {
    /// The leaf, in compact form.
    pub expression: String,
    /// Its truth value.
    pub truth: Truth,
    /// Facts it read that had values.
    pub observed: Vec<(FactPath, FactValue)>,
    /// Facts it read that nothing has observed.
    pub missing: Vec<FactPath>,
    /// Why the comparison could not be made, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// `true` when this leaf appears under a negation, so holding is the problem.
    pub negated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{FactStore, Scales};

    fn store(facts: &[(&str, FactValue)]) -> FactStore {
        let mut store = FactStore::new();
        for (path, value) in facts {
            store.set_path(path, value.clone());
        }
        store
    }

    fn parse(input: &str) -> Predicate {
        Predicate::parse_expression(input).expect("parses")
    }

    fn quantifier(yaml: &str) -> Predicate {
        let node: Node = serde_yaml::from_str(yaml).expect("yaml");
        Predicate::from_node(&node).expect("parses")
    }

    struct UnobservedPresence;
    impl FactSource for UnobservedPresence {
        fn fact(&self, path: &FactPath) -> Option<FactValue> {
            matches!(
                path.to_string().as_str(),
                "groups.count" | "groups.0.members.count"
            )
            .then(|| FactValue::count(1))
        }
        fn observe(&self, path: &FactPath) -> Option<FactValue> {
            if path.to_string() == "groups.0.members.0.virtual" {
                Some(FactValue::count(7))
            } else {
                self.fact(path)
            }
        }
        fn observed_presence(&self, path: &FactPath) -> Option<bool> {
            match path.segments().last().map(String::as_str) {
                Some("optional") => None,
                Some("required") => Some(true),
                _ => Some(false),
            }
        }
    }

    #[test]
    fn unobserved_presence_keeps_kleene_logic_and_current_time_wrapping() {
        let now =
            crate::time::Rfc3339Instant::parse_rfc3339("2026-10-03T12:00:00Z").expect("valid time");
        let wrapped = WithNow::new(&UnobservedPresence, now);
        for facts in [&UnobservedPresence as &dyn FactSource, &wrapped] {
            let unknown = parse("defined(optional)");
            assert_eq!(unknown.evaluate(facts), Truth::Unknown);
            assert_eq!(
                Predicate::not(unknown.clone()).evaluate(facts),
                Truth::Unknown
            );
            assert_eq!(parse("defined(required)").evaluate(facts), Truth::True);
            assert_eq!(parse("defined(absent)").evaluate(facts), Truth::False);
            for children in [
                vec![unknown.clone(), Predicate::Never],
                vec![Predicate::Never, unknown.clone()],
            ] {
                assert_eq!(Predicate::All(children).evaluate(facts), Truth::False);
            }
            for children in [
                vec![unknown.clone(), Predicate::Always],
                vec![Predicate::Always, unknown.clone()],
            ] {
                assert_eq!(Predicate::Any(children).evaluate(facts), Truth::True);
            }
        }
    }

    #[test]
    fn nested_binders_forward_unobserved_presence_and_observation() {
        for (body, expected) in [
            ("defined(member.optional)", Truth::Unknown),
            ("defined(member.required)", Truth::True),
            ("defined(member.absent)", Truth::False),
            ("member.virtual == 7", Truth::True),
        ] {
            let predicate = quantifier(&format!(
                "forall: {{in: groups, as: group, that: {{forall: {{in: group.members, as: member, that: '{body}'}}}}}}"
            ));
            assert_eq!(predicate.evaluate(&UnobservedPresence), expected, "{body}");
        }
    }

    /// Two slots, the first matched and the second not.
    fn slots() -> FactStore {
        store(&[
            ("slots.count", FactValue::count(2)),
            ("slots.0.matched", FactValue::Bool(true)),
            ("slots.0.score", FactValue::count(10)),
            ("slots.1.matched", FactValue::Bool(false)),
            ("slots.1.score", FactValue::count(5)),
        ])
    }

    /// beyond10x/ess#176: a present aggregate binds no leaf at its own path, only a presence mark,
    /// and `defined()` reads the mark; every value read still ignores it.
    #[test]
    fn defined_reads_a_present_aggregate_and_no_value_read_does() {
        let mut facts = store(&[("state", FactValue::text("Running"))]);
        facts.mark_present(FactPath::new("metrics").expect("path"));
        assert_eq!(parse("defined(metrics)").evaluate(&facts), Truth::True);
        assert_eq!(parse("defined(other)").evaluate(&facts), Truth::False);
        assert_eq!(parse("metrics").evaluate(&facts), Truth::Unknown);
        assert_eq!(facts.fact(&FactPath::new("metrics").expect("path")), None);
        assert_eq!(
            quantifier("any: [state == Paused, {not: 'defined(metrics)'}]").evaluate(&facts),
            Truth::False,
            "a running queue holding metrics breaks the #176 invariant"
        );
    }

    #[test]
    fn a_quantified_element_reads_presence_through_its_binder() {
        let mut facts = store(&[
            ("queues.count", FactValue::count(2)),
            ("queues.1.state", FactValue::text("Running")),
        ]);
        facts.mark_present(FactPath::new("queues.0.metrics").expect("path"));
        let some = quantifier("exists: {in: queues, as: q, that: 'defined(q.metrics)'}");
        let every = quantifier("forall: {in: queues, as: q, that: 'defined(q.metrics)'}");
        assert_eq!(some.evaluate(&facts), Truth::True);
        assert_eq!(every.evaluate(&facts), Truth::False);
    }

    #[test]
    fn extending_a_store_carries_its_presence_marks() {
        let mut left = FactStore::new();
        let mut right = FactStore::new();
        right.mark_present(FactPath::new("metrics").expect("path"));
        left.extend(right);
        assert!(left.present(&FactPath::new("metrics").expect("path")));
    }

    #[test]
    fn a_universal_holds_only_when_every_element_does() {
        let every = quantifier("forall: {in: slots, as: slot, that: slot.score >= 5}");
        assert_eq!(every.evaluate(&slots()), Truth::True);

        let all_matched = quantifier("forall: {in: slots, as: slot, that: slot.matched}");
        assert_eq!(all_matched.evaluate(&slots()), Truth::False);
    }

    #[test]
    fn an_existential_holds_as_soon_as_one_element_does() {
        let any_matched = quantifier("exists: {in: slots, as: slot, that: slot.matched}");
        assert_eq!(any_matched.evaluate(&slots()), Truth::True);

        let any_negative = quantifier("exists: {in: slots, as: slot, that: slot.score < 0}");
        assert_eq!(any_negative.evaluate(&slots()), Truth::False);
    }

    #[test]
    fn an_unobserved_collection_is_unknown_and_an_empty_one_is_not() {
        let every = quantifier("forall: {in: slots, as: slot, that: slot.matched}");
        let some = quantifier("exists: {in: slots, as: slot, that: slot.matched}");

        // Nothing looked.
        assert_eq!(every.evaluate(&store(&[])), Truth::Unknown);
        assert_eq!(some.evaluate(&store(&[])), Truth::Unknown);

        // Somebody looked and there was nothing there. Vacuously true, and not existentially true.
        let empty = store(&[("slots.count", FactValue::count(0))]);
        assert_eq!(every.evaluate(&empty), Truth::True);
        assert_eq!(some.evaluate(&empty), Truth::False);
    }

    #[test]
    fn a_count_that_is_not_a_whole_number_of_elements_is_unobserved() {
        let every = quantifier("forall: {in: slots, as: slot, that: slot.matched}");
        for bad in [
            FactValue::Number(crate::facts::Number::new(-1.0).expect("finite")),
            FactValue::Text("two".to_owned()),
        ] {
            let facts = store(&[("slots.count", bad)]);
            assert_eq!(every.evaluate(&facts), Truth::Unknown);
        }
    }

    #[test]
    fn a_nested_quantifier_still_reaches_the_outer_element() {
        let facts = store(&[
            ("groups.count", FactValue::count(2)),
            ("groups.0.limit", FactValue::count(10)),
            ("groups.0.members.count", FactValue::count(1)),
            ("groups.0.members.0.size", FactValue::count(4)),
            ("groups.1.limit", FactValue::count(3)),
            ("groups.1.members.count", FactValue::count(1)),
            ("groups.1.members.0.size", FactValue::count(4)),
        ]);
        let within = quantifier(
            "forall:\n  in: groups\n  as: group\n  that:\n    forall:\n      in: group.members\n      as: member\n      that: member.size <= group.limit\n",
        );
        // The second group's only member is over its limit, and `group.limit` is read from the
        // outer binder while `member.size` is read from the inner one.
        assert_eq!(within.evaluate(&facts), Truth::False);
    }

    #[test]
    fn a_binder_is_not_a_fact_path_but_its_collection_is() {
        let every =
            quantifier("forall: {in: slots, as: slot, that: {all: [slot.matched, tenant.active]}}");
        let read: Vec<String> = every.fact_paths().iter().map(ToString::to_string).collect();
        assert_eq!(read, vec!["slots".to_owned(), "tenant.active".to_owned()]);

        let collections: Vec<String> = every
            .quantified_collections()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(collections, vec!["slots".to_owned()]);
    }

    #[test]
    fn a_quantifier_round_trips_through_its_document_form() {
        let original = quantifier("exists: {in: pairs, as: pair, that: pair.left == a}");
        let round_tripped = Predicate::from_node(&original.to_node()).expect("re-reads");
        assert_eq!(round_tripped, original);
    }

    #[test]
    fn a_malformed_quantifier_is_refused_by_what_is_wrong_with_it() {
        let refusal = |yaml: &str| {
            let node: Node = serde_yaml::from_str(yaml).expect("yaml");
            Predicate::from_node(&node)
                .expect_err("refused")
                .to_string()
        };

        assert!(
            refusal("forall: {as: slot, that: slot.matched}").contains("no `in`"),
            "missing collection"
        );
        assert!(
            refusal("forall: {in: slots, as: slot}").contains("no `that`"),
            "missing body"
        );
        assert!(
            refusal("forall: {in: slots, as: slot.left, that: slot.matched}")
                .contains("one segment"),
            "a binder is a name, not a path"
        );
        assert!(
            refusal("forall: {in: slots, as: slot, that: slot.matched, unless: x}")
                .contains("`in`, `as` and `that`"),
            "an unknown key names the vocabulary"
        );
        assert!(
            refusal("forall: [slots, slot]").contains("a mapping"),
            "a sequence is not a quantifier"
        );
    }

    #[test]
    fn parses_the_compact_forms() {
        assert_eq!(
            parse("tests.unit.failed == 0"),
            Predicate::Compare {
                kind: CompareKind::Value,
                left: Operand::Fact("tests.unit.failed".parse().expect("path")),
                op: CompareOp::Eq,
                right: Operand::Literal(FactValue::count(0)),
            }
        );
        assert_eq!(
            parse("specification.satisfied"),
            Predicate::Truthy("specification.satisfied".parse().expect("path"))
        );
        assert_eq!(
            parse("defined(deployment.previous_revision)"),
            Predicate::Defined("deployment.previous_revision".parse().expect("path"))
        );
    }

    #[test]
    fn reads_a_dotted_right_hand_side_as_a_fact_and_a_bare_word_as_text() {
        let compared = parse("error_rate < service.slo.error_threshold");
        let Predicate::Compare { right, .. } = &compared else {
            panic!("expected a comparison, got {compared:?}");
        };
        assert!(matches!(right, Operand::Fact(_)), "{right:?}");

        let equals = parse("test.result == failed");
        let Predicate::Compare { right, .. } = &equals else {
            panic!("expected a comparison, got {equals:?}");
        };
        assert_eq!(right, &Operand::Literal(FactValue::text("failed")));

        let quoted = parse("release.version == \"1.2.3\"");
        let Predicate::Compare { right, .. } = &quoted else {
            panic!("expected a comparison, got {quoted:?}");
        };
        assert_eq!(right, &Operand::Literal(FactValue::text("1.2.3")));
    }

    #[test]
    fn unobserved_facts_are_unknown_not_false() {
        let facts = store(&[]);
        assert_eq!(
            parse("tests.unit.failed == 0").evaluate(&facts),
            Truth::Unknown
        );

        let observed = store(&[("tests.unit.failed", FactValue::count(2))]);
        assert_eq!(
            parse("tests.unit.failed == 0").evaluate(&observed),
            Truth::False
        );
    }

    #[test]
    fn kleene_conjunction_keeps_false_ahead_of_unknown() {
        let facts = store(&[("a.failed", FactValue::count(1))]);
        let predicate = Predicate::all(vec![parse("a.failed == 0"), parse("b.failed == 0")]);
        assert_eq!(
            predicate.evaluate(&facts),
            Truth::False,
            "one observed failure decides the conjunction, whatever the unobserved half holds"
        );

        let partial = store(&[("a.failed", FactValue::count(0))]);
        assert_eq!(
            predicate.evaluate(&partial),
            Truth::Unknown,
            "the unobserved half must stay Unknown, not collapse to False — invariant 5"
        );
    }

    #[test]
    fn explains_only_the_failing_children_of_a_conjunction() {
        let facts = store(&[
            ("tests.unit.failed", FactValue::count(0)),
            ("tests.contract.failed", FactValue::count(3)),
        ]);
        let predicate = Predicate::all(vec![
            parse("tests.unit.failed == 0"),
            parse("tests.contract.failed == 0"),
            parse("static_analysis.errors == 0"),
        ]);

        let outcome = predicate.outcome(&facts);
        assert_eq!(outcome.truth, Truth::False);
        let expressions: Vec<&str> = outcome
            .causes
            .iter()
            .map(|c| c.expression.as_str())
            .collect();
        assert_eq!(
            expressions,
            vec!["tests.contract.failed == 0", "static_analysis.errors == 0"]
        );
        assert_eq!(outcome.causes[0].truth, Truth::False);
        assert_eq!(outcome.causes[1].truth, Truth::Unknown);
        assert_eq!(
            outcome
                .missing_facts()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["static_analysis.errors"]
        );
    }

    #[test]
    fn ordering_text_needs_a_declared_scale() {
        let mut facts = store(&[("risk", FactValue::text("high"))]);
        let predicate = parse("risk >= medium");
        assert_eq!(predicate.evaluate(&facts), Truth::Unknown);

        let mut scales = Scales::default();
        scales.insert(
            "risk",
            ["low", "medium", "high"].map(ToOwned::to_owned).to_vec(),
        );
        facts.set_scales(scales);
        assert_eq!(predicate.evaluate(&facts), Truth::True);
    }

    /// A source that declares `window.*` Timestamps.
    struct Instants(FactStore);

    impl FactSource for Instants {
        fn fact(&self, path: &FactPath) -> Option<FactValue> {
            self.0.fact(path)
        }

        fn orders_as_instant(&self, path: &FactPath) -> bool {
            path.namespace() == "window"
        }
    }

    #[test]
    fn a_declared_timestamp_orders_by_its_instant() {
        let facts = Instants(store(&[
            (
                "window.starts_at",
                FactValue::text("2020-01-01T12:00:00+01:00"),
            ),
            ("window.ends_at", FactValue::text("2020-01-01T11:30:00Z")),
            ("note", FactValue::text("2020-01-01T11:30:00Z")),
        ]));
        for (expression, truth) in [
            ("window.ends_at > window.starts_at", Truth::True),
            ("window.starts_at >= window.ends_at", Truth::False),
            (
                "window.ends_at < \"2020-01-01T12:00:00+00:30\"",
                Truth::False,
            ),
            ("window.ends_at > tomorrow", Truth::Unknown),
            // An undeclared text is still ordered by scales only, however it is spelled.
            ("note > \"2020-01-01T00:00:00Z\"", Truth::Unknown),
        ] {
            assert_eq!(parse(expression).evaluate(&facts), truth, "{expression}");
        }
        assert_eq!(
            parse("window.ends_at > window.starts_at").evaluate(&store(&[
                (
                    "window.starts_at",
                    FactValue::text("2020-01-01T12:00:00+01:00")
                ),
                ("window.ends_at", FactValue::text("2020-01-01T11:30:00Z")),
            ])),
            Truth::Unknown,
            "a source that declares no Timestamp keeps today's answer"
        );
    }

    #[test]
    fn round_trips_through_document_form() {
        let source = Predicate::all(vec![
            parse("tests.unit.failed == 0"),
            Predicate::any(vec![parse("service.health == healthy"), parse("recovered")]),
            Predicate::not(parse("change.architectural")),
            Predicate::AnyOf {
                path: "task.kind".parse().expect("path"),
                values: vec![FactValue::text("feature"), FactValue::text("bugfix")],
            },
        ]);
        let round_tripped = Predicate::from_node(&source.to_node()).expect("re-parses");
        assert_eq!(round_tripped, source);
    }

    #[test]
    fn parses_the_structured_mapping_forms() {
        let node = Node::Map(
            [(
                "task.kind".to_owned(),
                Node::Map(
                    [(
                        "any_of".to_owned(),
                        Node::Seq(vec![Node::from("feature"), Node::from("bugfix")]),
                    )]
                    .into_iter()
                    .collect(),
                ),
            )]
            .into_iter()
            .collect(),
        );
        let predicate = Predicate::from_node(&node).expect("parses");
        let facts = store(&[("task.kind", FactValue::text("bugfix"))]);
        assert_eq!(predicate.evaluate(&facts), Truth::True);

        let other = store(&[("task.kind", FactValue::text("release"))]);
        assert_eq!(predicate.evaluate(&other), Truth::False);
    }

    #[test]
    fn negating_unknown_leaves_it_unknown() {
        // Invariant 5, asserted as the whole table rather than as a sample: `not Unknown == True`
        // would mean `not deployment.failed` permits a transition *because* nothing has run, which
        // is the collapse of unobserved into false that the third value exists to prevent.
        for (input, expected) in [
            (Truth::True, Truth::False),
            (Truth::False, Truth::True),
            (Truth::Unknown, Truth::Unknown),
        ] {
            assert_eq!(
                input.not(),
                expected,
                "not {input:?} must be {expected:?}, was {:?}",
                input.not()
            );
        }
    }

    #[test]
    fn conjunction_follows_the_kleene_table_in_all_nine_rows() {
        for (left, right, expected) in [
            (Truth::True, Truth::True, Truth::True),
            (Truth::True, Truth::False, Truth::False),
            (Truth::True, Truth::Unknown, Truth::Unknown),
            (Truth::False, Truth::True, Truth::False),
            (Truth::False, Truth::False, Truth::False),
            // `False` dominates `Unknown`: a failed suite is a failure whether or not the rest ran.
            (Truth::False, Truth::Unknown, Truth::False),
            (Truth::Unknown, Truth::True, Truth::Unknown),
            (Truth::Unknown, Truth::False, Truth::False),
            (Truth::Unknown, Truth::Unknown, Truth::Unknown),
        ] {
            assert_eq!(
                left.and(right),
                expected,
                "{left:?} and {right:?} must be {expected:?}, was {:?}",
                left.and(right)
            );
        }
    }

    #[test]
    fn disjunction_follows_the_kleene_table_in_all_nine_rows() {
        for (left, right, expected) in [
            (Truth::True, Truth::True, Truth::True),
            (Truth::True, Truth::False, Truth::True),
            // `True` dominates `Unknown`: one branch that holds is enough, unobserved or not.
            (Truth::True, Truth::Unknown, Truth::True),
            (Truth::False, Truth::True, Truth::True),
            (Truth::False, Truth::False, Truth::False),
            (Truth::False, Truth::Unknown, Truth::Unknown),
            (Truth::Unknown, Truth::True, Truth::True),
            (Truth::Unknown, Truth::False, Truth::Unknown),
            (Truth::Unknown, Truth::Unknown, Truth::Unknown),
        ] {
            assert_eq!(
                left.or(right),
                expected,
                "{left:?} or {right:?} must be {expected:?}, was {:?}",
                left.or(right)
            );
        }
    }

    #[test]
    fn only_true_satisfies_a_transition() {
        assert!(Truth::True.is_satisfied());
        assert!(!Truth::False.is_satisfied());
        assert!(
            !Truth::Unknown.is_satisfied(),
            "unknown never advances a workflow"
        );
    }

    #[test]
    fn a_predicate_nested_past_the_limit_is_refused_rather_than_overflowing_the_stack() {
        // The string form: one YAML scalar, so nothing above this parser bounds it. 10 000 `not`
        // prefixes overflowed an 8 MiB stack before this bound existed.
        let stacked = format!("{}a.b", "not ".repeat(10_000));
        let error = Predicate::parse_expression(&stacked).expect_err("nesting past the limit");
        assert!(
            matches!(
                error,
                ParseError::TooDeep {
                    kind: "predicate",
                    limit: MAX_PREDICATE_DEPTH,
                    ..
                }
            ),
            "the refusal names the construct and the limit: {error:?}"
        );

        // The structured form: nested mappings.
        let mut node = Node::Bool(true);
        for _ in 0..(MAX_PREDICATE_DEPTH + 2) {
            node = Node::Map([("not".to_owned(), node)].into_iter().collect());
        }
        let error = Predicate::from_node(&node).expect_err("nesting past the limit");
        assert!(
            matches!(
                error,
                ParseError::TooDeep {
                    kind: "predicate",
                    limit: MAX_PREDICATE_DEPTH,
                    ..
                }
            ),
            "the refusal names the construct and the limit: {error:?}"
        );
    }

    #[test]
    fn the_two_predicate_forms_share_one_depth_budget() {
        // `{not: {not: "not not a.b"}}` is one predicate written two ways. Two counters would let a
        // document alternate between the forms and buy twice the depth, which is the whole bound.
        let half = MAX_PREDICATE_DEPTH / 2;
        let mut node = Node::Text(format!("{}a.b", "not ".repeat(half + 2)));
        for _ in 0..(half + 2) {
            node = Node::Map([("not".to_owned(), node)].into_iter().collect());
        }
        let error = Predicate::from_node(&node).expect_err("the two forms together exceed 32");
        assert!(
            matches!(
                error,
                ParseError::TooDeep {
                    kind: "predicate",
                    ..
                }
            ),
            "{error:?}"
        );
    }

    #[test]
    fn a_predicate_as_deep_as_anybody_writes_is_still_accepted() {
        // The failure mode of a depth bound is refusing a good document. This is deeper than
        // anything in `protocols/` or `examples/`, and it parses.
        let written = "
            all:
              - any:
                  - all:
                      - not: change.architectural
                      - service.health == healthy
                  - risk: {gte: medium}
              - task.kind: {any_of: [feature, bugfix]}
        ";
        let node: Node = serde_yaml::from_str(written).expect("well formed");
        Predicate::from_node(&node).expect("a realistic predicate");

        // And the whole budget, exactly, one level short of the refusal.
        let mut at_limit = Node::Bool(true);
        for _ in 0..MAX_PREDICATE_DEPTH {
            at_limit = Node::Map([("not".to_owned(), at_limit)].into_iter().collect());
        }
        assert_eq!(
            Predicate::from_node(&at_limit).expect("at the limit"),
            Predicate::Always,
            "{MAX_PREDICATE_DEPTH} levels is the limit, not one past it"
        );
    }

    #[test]
    fn rejects_unknown_operators_and_bad_paths() {
        let node = Node::Map(
            [(
                "risk".to_owned(),
                Node::Map(
                    [("approximately".to_owned(), Node::from("medium"))]
                        .into_iter()
                        .collect(),
                ),
            )]
            .into_iter()
            .collect(),
        );
        let error = Predicate::from_node(&node).expect_err("unknown operator");
        assert!(error.to_string().contains("unknown operator"), "{error}");
        assert!(Predicate::parse_expression("== 0").is_err());
        assert!(Predicate::parse_expression("").is_err());
    }

    /// `x == null` used to be a comparison with the four-character text `null` (ess#93).
    #[test]
    fn a_comparison_with_an_unquoted_null_is_refused_and_names_the_presence_test() {
        for (expression, operator, spelling) in [
            ("note == null", "==", "null"),
            ("note != null", "!=", "null"),
            ("note == NULL", "==", "NULL"),
            ("note != Null", "!=", "Null"),
            ("note == ~", "==", "~"),
            ("note < null", "<", "null"),
            ("null == note", "==", "null"),
        ] {
            let error = Predicate::parse_expression(expression).expect_err(expression);
            let ParseError::NullComparison {
                path,
                operator: refused,
                ..
            } = &error
            else {
                panic!("`{expression}` is refused as a null comparison, not as {error:?}")
            };
            assert_eq!(path, "note", "{expression}");
            assert_eq!(refused, operator, "{expression}");
            let rendered = error.to_string();
            assert!(
                rendered.contains("`defined(note)`") && rendered.contains("`not defined(note)`"),
                "the refusal names the presence test an author meant: {rendered}"
            );
            assert!(
                rendered.contains(&format!("`note {operator} {spelling}`"))
                    && rendered.contains(&format!("quote \"{spelling}\""))
                    && rendered.starts_with(ParseError::NULL_COMPARISON_CODE),
                "the refusal echoes what was written and carries its code: {rendered}"
            );
        }

        for (yaml, operator) in [
            ("note: {eq: null}", "=="),
            ("note: {ne: null}", "!="),
            ("note: null", "=="),
        ] {
            let node: Node = serde_yaml::from_str(yaml).expect("yaml");
            let error = Predicate::from_node(&node).expect_err(yaml);
            assert!(
                matches!(
                    &error,
                    ParseError::NullComparison { path, operator: refused, .. }
                        if path == "note" && refused == operator
                ),
                "`{yaml}` is refused as a null comparison, not as {error:?}"
            );
        }
    }

    #[test]
    fn a_quoted_null_is_still_the_text_and_survives_its_own_rendering() {
        for expression in [r#"note == "null""#, "note != 'null'", r#"note == "~""#] {
            let predicate = parse(expression);
            let Predicate::Compare {
                right: Operand::Literal(FactValue::Text(text)),
                ..
            } = &predicate
            else {
                panic!("`{expression}` compares with text: {predicate:?}")
            };
            assert!(["null", "~"].contains(&text.as_str()), "{expression}");
            assert_eq!(
                Predicate::parse_expression(&predicate.to_string()).as_ref(),
                Ok(&predicate),
                "`{predicate}` reads back as the text it was written as"
            );
            assert!(
                !predicate.requires_structured_text_comparison(),
                "the compact form stays compact: {predicate}"
            );
        }
        assert_eq!(
            parse(r#"note == "null""#).evaluate(&store(&[("note", FactValue::text("null"))])),
            Truth::True
        );
    }

    /// `&&` and `||` are not compact syntax; unquoted, they were swallowed into one text literal.
    #[test]
    fn an_unquoted_conjunction_or_disjunction_token_is_refused_toward_the_structured_form() {
        for expression in [
            "sku == A1 && gift",
            "sku == A1 || sku == B2",
            "sku != A1 &&",
            "count > 1 || count < 0",
        ] {
            let error = Predicate::parse_expression(expression).expect_err(expression);
            let rendered = error.to_string();
            assert!(
                matches!(error, ParseError::Predicate { .. })
                    && rendered.contains("structured all/any/not"),
                "`{expression}`: {rendered}"
            );
        }
        for (expression, text) in [
            (r#"sku == "A1 && gift""#, "A1 && gift"),
            ("sku == 'A1 || B2'", "A1 || B2"),
            ("sku == A1&&gift", "A1&&gift"),
        ] {
            let predicate = parse(expression);
            assert_eq!(
                predicate,
                Predicate::Compare {
                    kind: CompareKind::Value,
                    left: Operand::Fact("sku".parse().expect("path")),
                    op: CompareOp::Eq,
                    right: Operand::Literal(FactValue::text(text)),
                },
                "{expression}"
            );
            assert_eq!(
                Predicate::parse_expression(&predicate.to_string()).as_ref(),
                Ok(&predicate),
                "`{predicate}` reads back as the text it was written as"
            );
            assert!(
                !predicate.requires_structured_text_comparison(),
                "{predicate}"
            );
        }
    }

    /// A source that orders text by its bytes, as every ESS lane does (ess#94).
    #[test]
    fn a_source_that_orders_text_by_bytes_decides_where_no_scale_does() {
        let mut facts = store(&[("caller", FactValue::text("B"))]);
        assert_eq!(
            parse("caller < a").evaluate(&facts),
            Truth::Unknown,
            "a source that did not opt in keeps the scale-only reading AEP relies on"
        );

        facts.order_text_by_bytes();
        for (expression, truth) in [
            // `B` is 0x42 and `a` is 0x61: byte order, not the locale order that puts `a` first.
            ("caller < a", Truth::True),
            ("caller > a", Truth::False),
            ("caller <= B", Truth::True),
            ("caller >= Ba", Truth::False),
            (r#"caller > """#, Truth::True),
        ] {
            assert_eq!(parse(expression).evaluate(&facts), truth, "{expression}");
        }

        let mut scales = Scales::default();
        scales.insert("rank", vec!["a".to_owned(), "B".to_owned()]);
        facts.set_scales(scales);
        assert_eq!(
            parse("caller > a").evaluate(&facts),
            Truth::True,
            "a declared scale containing both values still decides first"
        );
    }

    #[test]
    fn a_declared_timestamp_that_does_not_parse_is_not_ordered_by_its_bytes() {
        struct Declared(FactStore);
        impl FactSource for Declared {
            fn fact(&self, path: &FactPath) -> Option<FactValue> {
                self.0.fact(path)
            }
            fn orders_as_instant(&self, _path: &FactPath) -> bool {
                true
            }
            fn orders_text_by_bytes(&self, _path: &FactPath) -> bool {
                true
            }
        }
        let facts = Declared(store(&[("at", FactValue::text("yesterday"))]));
        assert_eq!(
            parse(r#"at < "2020-01-01T00:00:00Z""#).evaluate(&facts),
            Truth::Unknown,
            "an instant nobody can read is not a text to sort"
        );
    }
}
