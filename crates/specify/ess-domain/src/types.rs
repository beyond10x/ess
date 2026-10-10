//! The type system.
//!
//! Semantic types stay distinct from their representations. `Email` and `InvoiceId` are both strings
//! on the wire and are not interchangeable in the model, because the whole value of a specification
//! is that it can refuse a binding which maps one into the other.
//!
//! ```text
//! primitive   String Boolean Integer Decimal Timestamp Duration Uuid Bytes
//! composite   Struct Enum Optional List Map Union(tagged)
//! named       Email  Money  InvoiceId       — distinct even when representations match
//! ```
//!
//! # Unions are tagged
//!
//! An untagged union cannot round-trip through JSON Schema, `OpenAPI` or Serde without ambiguity: two
//! variants with the same shape are indistinguishable on the way back. The model therefore carries
//! the tag field, and an untagged form is not offered rather than being offered with a warning.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_primitives::error::{
    ConstructRef, ParseError, ValidationCode, ValidationError, ValidationErrors,
};

use crate::entity::{Invariant, RawInvariant};
use crate::name::{Naming, QualifiedName};
use ess_primitives::predicate::PredicateAt;

/// A type with no structure of its own.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Primitive {
    /// Text.
    String,
    /// True or false.
    Boolean,
    /// A whole number.
    Integer,
    /// An exact decimal. Never a float: money does not round the way a float does.
    Decimal,
    /// A finite IEEE-754 binary64, preserving signed zero. Authored in ess/2.
    Binary64,
    /// An instant.
    Timestamp,
    /// A length of time.
    Duration,
    /// A UUID.
    Uuid,
    /// Opaque bytes.
    Bytes,
    /// Any JSON value, compared structurally (ess/15, beyond10x/ess#138). Never a map key, and
    /// never read by a predicate.
    Json,
}

impl Primitive {
    /// Every primitive.
    pub const ALL: &'static [Self] = &[
        Self::String,
        Self::Boolean,
        Self::Integer,
        Self::Decimal,
        Self::Timestamp,
        Self::Duration,
        Self::Uuid,
        Self::Bytes,
        Self::Binary64,
        Self::Json,
    ];

    /// The primitive as written in a specification.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Boolean => "Boolean",
            Self::Integer => "Integer",
            Self::Decimal => "Decimal",
            Self::Timestamp => "Timestamp",
            Self::Duration => "Duration",
            Self::Uuid => "Uuid",
            Self::Bytes => "Bytes",
            Self::Binary64 => "Binary64",
            Self::Json => "Json",
        }
    }

    /// Parses a primitive name.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.as_str() == value)
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reference to a type, as written in a specification.
///
/// `Email`, `List<Money>`, `Optional<CustomerId>`, `Map<String, Money>`.
///
/// # Depth
///
/// Every value of this type that a document can produce is at most [`MAX_TYPE_DEPTH`] deep, because
/// [`TypeRef::parse`] is the only way a document reaches one — [`serde::Deserialize`] goes through
/// it, and the compiler's two conversions
/// (`Resolver::type_ref` and `spec_type_ref`) map one constructor to one constructor and so preserve
/// depth exactly. That is what lets the walkers below — [`Self::named_dependencies`],
/// [`Self::required`], [`Display`](fmt::Display), [`is_assignable`], and the ones in [`crate::view`],
/// [`crate::system`] and [`crate::binding`] — recurse without counting: at 32 levels the deepest of
/// them uses a few kilobytes of stack. It stops being true the moment something builds a `TypeRef`
/// by wrapping rather than by parsing, which is why the bound lives in the parser and this note
/// lives on the type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeRef {
    /// A primitive.
    Primitive(Primitive),
    /// A named type declared elsewhere in the specification.
    Named(QualifiedName),
    /// A value that may be absent.
    Optional(Box<TypeRef>),
    /// An ordered sequence.
    List(Box<TypeRef>),
    /// A mapping. The key must be a primitive, because a structured key has no stable wire form.
    /// A key written as a newtype of a key primitive is resolved to that primitive while the
    /// document is read ([`MapKeyNewtypes`]).
    Map(Primitive, Box<TypeRef>),
}

/// How many generic wrappers a type reference may nest before it is refused.
///
/// A document chooses this number by writing it — `type: Optional<Optional<…>>` is one YAML scalar,
/// and nothing above this function bounds a scalar's length. Measured on this machine, a debug
/// build of [`TypeRef::parse`] overflows the 8 MiB main-thread stack between 3 000 and 4 000
/// wrappers, and the 2 MiB a spawned worker gets between 800 and 1 000; a stack overflow is an abort
/// with no diagnostic, which is the one failure mode this compiler does not otherwise have.
///
/// 32 because the deepest type a real specification writes is
/// `Optional<List<Map<String, Optional<Money>>>>`, which is five. That leaves a factor of
/// twenty-five below the smallest measured floor, and a factor of six above anything anybody has
/// written. The number is this bound's own: `WRAPPER_LIMIT` in [`crate::binding`] is also 32 and no
/// longer bounds anything this one does — it is what a documentation renderer traverses, while this
/// is what keeps a parser off the stack's end — so neither number is a reason for the other.
/// `serde_yaml`'s own structural recursion cap is 128, so this refusal is reached first and carries
/// our message rather than the deserializer's.
pub const MAX_TYPE_DEPTH: usize = 32;

impl TypeRef {
    /// Parses a type reference.
    ///
    /// Refuses nesting beyond [`MAX_TYPE_DEPTH`] with [`ParseError::TooDeep`]. The check is made on
    /// the way down, before the `Box` for that level is allocated, so a refused document never
    /// builds the chain whose `Drop` would recurse just as deeply as the parse did.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        Self::parse_nested(value, 0)
    }

    /// [`Self::parse`], counting how many wrappers deep it already is.
    fn parse_nested(value: &str, depth: usize) -> Result<Self, ParseError> {
        let trimmed = value.trim();
        let reject = |reason: &str| ParseError::identifier("type", value, reason.to_owned());

        if depth > MAX_TYPE_DEPTH {
            return Err(ParseError::too_deep("type", trimmed, MAX_TYPE_DEPTH));
        }

        if let Some(inner) = generic_argument(trimmed, "Optional") {
            return Ok(Self::Optional(Box::new(Self::parse_nested(
                inner,
                depth + 1,
            )?)));
        }
        if let Some(inner) = generic_argument(trimmed, "List") {
            return Ok(Self::List(Box::new(Self::parse_nested(inner, depth + 1)?)));
        }
        if let Some(inner) = generic_argument(trimmed, "Map") {
            let (key, value_type) = inner.split_once(',').ok_or_else(|| {
                reject("a map needs a key and a value, as in `Map<String, Money>`")
            })?;
            let key = Primitive::parse(key.trim())
                .or_else(|| MapKeyNewtypes::in_scope(key.trim()))
                .ok_or_else(|| {
                    ParseError::identifier(
                        "type",
                        value,
                        format!(
                            "a map key must be a primitive, or a newtype of one declared where this \
                             type is read; {:?} is neither, and a structured key has no stable wire \
                             form",
                            key.trim()
                        ),
                    )
                })?;
            if key == Primitive::Binary64 {
                return Err(reject("Binary64 map keys have no admitted wire spelling"));
            }
            if key == Primitive::Json {
                return Err(reject(
                    "a Json value is not a map key: a JSON object key is text, and a value has no \
                     canonical spelling as one",
                ));
            }
            return Ok(Self::Map(
                key,
                Box::new(Self::parse_nested(value_type, depth + 1)?),
            ));
        }
        if trimmed.contains(['<', '>']) {
            return Err(reject(
                "unknown generic; the model has `Optional<T>`, `List<T>` and `Map<K, V>`",
            ));
        }
        if let Some(primitive) = Primitive::parse(trimmed) {
            return Ok(Self::Primitive(primitive));
        }
        Ok(Self::Named(QualifiedName::new(trimmed)?))
    }

    /// Every named type this reference depends on.
    pub fn named_dependencies(&self) -> Vec<&QualifiedName> {
        match self {
            Self::Primitive(_) => Vec::new(),
            Self::Named(name) => vec![name],
            Self::Optional(inner) | Self::List(inner) | Self::Map(_, inner) => {
                inner.named_dependencies()
            }
        }
    }

    /// `true` when a value of this type may be absent.
    pub fn is_optional(&self) -> bool {
        matches!(self, Self::Optional(_))
    }

    /// This reference with any `Optional` wrapper removed.
    pub fn required(&self) -> &Self {
        match self {
            Self::Optional(inner) => inner.required(),
            other => other,
        }
    }
}

/// The newtypes a specification declares that may key a map, each with the primitive it is spelled
/// as on the wire (beyond10x/ess#143).
///
/// `Map<demo.orders.ItemId, Boolean>` with `ItemId` a newtype of `String` has exactly the wire form
/// of `Map<String, Boolean>`, so the reason a structured key is refused does not apply to it. The
/// key is resolved to that primitive while the document is read, so [`TypeRef::Map`] keeps a
/// [`Primitive`] key and nothing downstream of the parser changes: every projection, generator and
/// suite sees `Map<String, Boolean>`. What the resolution gives up is the newtype's identity at
/// the key position — its alphabet, length and name are not carried into the map.
///
/// A reference is a string parsed one document at a time, and the newtype it names may be declared
/// later in the same document or in another file. So the declarations are collected first
/// ([`Self::from_documents`]) and parsing runs inside [`Self::scope`]. Outside any scope, a named
/// key is refused exactly as before.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MapKeyNewtypes {
    /// Every newtype declared, with the spelling of what it wraps; `None` for a name declared
    /// twice with different underlying types.
    declared: BTreeMap<QualifiedName, Option<String>>,
}

std::thread_local! {
    /// The key newtypes in view of [`TypeRef::parse`] on this thread, set only by
    /// [`MapKeyNewtypes::scope`].
    static MAP_KEY_NEWTYPES: std::cell::RefCell<MapKeyNewtypes> =
        std::cell::RefCell::new(MapKeyNewtypes::default());
}

impl MapKeyNewtypes {
    /// Collects the newtypes declared in `texts`.
    ///
    /// Reads only `types:` entries with `kind: newtype`, `name:` and `of:`. A document that does
    /// not parse, or an entry of the wrong shape, contributes nothing here: the real read of that
    /// document reports it. A name declared twice with different underlying types resolves to
    /// nothing, because assembly refuses the duplicate and choosing one would be a guess.
    pub fn from_documents<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut keys = Self::default();
        for text in texts {
            if let Ok(document) = serde_yaml::from_str::<serde_yaml::Value>(text) {
                keys.absorb(&document);
            }
        }
        keys
    }

    /// Newtypes given as `(name, what it wraps as written)` pairs, by a reader that has them in
    /// hand rather than as source text — a compiled model, whose newtype `String`s and chains are
    /// the same ones the specification was read with.
    pub fn from_declarations(
        declarations: impl IntoIterator<Item = (QualifiedName, String)>,
    ) -> Self {
        let mut keys = Self::default();
        for (name, of) in declarations {
            keys.declare(name, of);
        }
        keys
    }

    /// Records one newtype; a second, different declaration of the name makes it resolve to
    /// nothing.
    fn declare(&mut self, name: QualifiedName, of: String) {
        self.declared
            .entry(name)
            .and_modify(|seen| {
                if seen.as_deref() != Some(of.as_str()) {
                    *seen = None;
                }
            })
            .or_insert(Some(of));
    }

    /// The key newtypes in view on this thread: empty outside every [`Self::scope`].
    pub fn current() -> Self {
        MAP_KEY_NEWTYPES.with(|cell| cell.borrow().clone())
    }

    /// These declarations and the ones in `document`, an already-read source file.
    ///
    /// What a reader of one file uses, so that a newtype declared in the same file keys a map
    /// with no loader involved, and one declared in a sibling file does when the loader put it in
    /// view.
    #[must_use]
    pub fn with_value(&self, document: &serde_yaml::Value) -> Self {
        let mut keys = self.clone();
        keys.absorb(document);
        keys
    }

    /// Adds the newtypes one document declares.
    fn absorb(&mut self, document: &serde_yaml::Value) {
        let Some(types) = document
            .get("types")
            .and_then(serde_yaml::Value::as_sequence)
        else {
            return;
        };
        for entry in types {
            if entry.get("kind").and_then(serde_yaml::Value::as_str) != Some("newtype") {
                continue;
            }
            let (Some(name), Some(of)) = (
                entry.get("name").and_then(serde_yaml::Value::as_str),
                entry.get("of").and_then(serde_yaml::Value::as_str),
            ) else {
                continue;
            };
            let Ok(name) = QualifiedName::new(name.trim()) else {
                continue;
            };
            self.declare(name, of.trim().to_owned());
        }
    }

    /// The primitive `name` is spelled as, when it is a newtype whose underlying type, through
    /// newtypes, is a primitive admitted as a map key. A cycle resolves to nothing.
    pub fn get(&self, name: &QualifiedName) -> Option<Primitive> {
        let mut current = name.clone();
        // Each step moves to a declaration, so more steps than declarations is a cycle.
        for _ in 0..=self.declared.len() {
            let of = self.declared.get(&current)?.as_deref()?;
            if let Some(primitive) = Primitive::parse(of) {
                return (!matches!(primitive, Primitive::Binary64 | Primitive::Json))
                    .then_some(primitive);
            }
            current = QualifiedName::new(of).ok()?;
        }
        None
    }

    /// Runs `body` with exactly these key newtypes in view of every [`TypeRef::parse`] it makes on
    /// this thread, and restores whatever was in view before — also when `body` panics.
    pub fn scope<T>(&self, body: impl FnOnce() -> T) -> T {
        struct Restore(Option<MapKeyNewtypes>);
        impl Drop for Restore {
            fn drop(&mut self) {
                if let Some(previous) = self.0.take() {
                    MAP_KEY_NEWTYPES.with(|cell| *cell.borrow_mut() = previous);
                }
            }
        }
        let previous = MAP_KEY_NEWTYPES.with(|cell| cell.replace(self.clone()));
        let _restore = Restore(Some(previous));
        body()
    }

    /// The key newtype `spelling` names in the current scope, if any.
    fn in_scope(spelling: &str) -> Option<Primitive> {
        let name = QualifiedName::new(spelling).ok()?;
        MAP_KEY_NEWTYPES.with(|cell| cell.borrow().get(&name))
    }
}

/// Extracts `T` from `Name<T>`.
fn generic_argument<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    let rest = value.strip_prefix(name)?;
    let rest = rest.trim_start().strip_prefix('<')?;
    rest.trim_end().strip_suffix('>').map(str::trim)
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(primitive) => write!(f, "{primitive}"),
            Self::Named(name) => write!(f, "{name}"),
            Self::Optional(inner) => write!(f, "Optional<{inner}>"),
            Self::List(inner) => write!(f, "List<{inner}>"),
            Self::Map(key, value) => write!(f, "Map<{key}, {value}>"),
        }
    }
}

impl std::str::FromStr for TypeRef {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl serde::Serialize for TypeRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for TypeRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for TypeRef {
    fn schema_name() -> String {
        "TypeRef".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.metadata().description = Some(
            "A type: a primitive, a named type, or `Optional<T>`, `List<T>` or `Map<K, V>`."
                .to_owned(),
        );
        schema.metadata().examples = ["Email", "Money", "Optional<CustomerId>", "List<Money>"]
            .iter()
            .map(|value| serde_json::Value::String((*value).to_owned()))
            .collect();
        schema.into()
    }
}

/// One field of a struct or an event.
///
/// Read through [`RawField`], which also takes the nested `naming: {wire: …}` spelling
/// (beyond10x/ess#142) and normalizes it to the flat keys this type is written back with.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RawField")]
pub struct Field {
    /// Its name, which [`RawField`] checks is a field name.
    pub name: String,
    /// Its type.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    /// What it is on the wire, and what a person is shown.
    #[serde(flatten, skip_serializing_if = "Naming::is_empty")]
    pub naming: Naming,
}

impl Field {
    /// The pattern published in generated JSON Schema.
    ///
    /// Kept beside the parser that enforces it, and a test asserts the published schema carries
    /// this one: a schema that accepts what the parser refuses is worse than no schema.
    pub const PATTERN: &'static str = "^_*[A-Za-z][A-Za-z0-9_]*$";

    /// A field with no naming overrides.
    pub fn new(name: impl Into<String>, type_ref: TypeRef) -> Self {
        Self {
            name: name.into(),
            type_ref,
            naming: Naming::default(),
        }
    }
}

impl Field {
    /// How an absent value of this `Optional` field travels, when declared (ess/15).
    pub fn presence(&self) -> Option<Presence> {
        self.naming.presence
    }
}

/// How an absent value of an `Optional<T>` field is spelled on the wire (beyond10x/ess#139, ess/15).
///
/// An `Optional` that declares neither admits both spellings, as it always did. A field that
/// declares one is held to it by the published JSON Schema and by a conformance suite's payload
/// shape, so an implementation that swaps the two fails.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    /// The key is always sent; an absent value is an explicit `null`.
    NullWhenAbsent,
    /// The key is left out when the value is absent, and never sent as `null`.
    OmittedWhenAbsent,
}

impl fmt::Display for Presence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NullWhenAbsent => "null_when_absent",
            Self::OmittedWhenAbsent => "omitted_when_absent",
        })
    }
}

// A field as written: `Field`'s keys, plus the nested `naming:` synonym.
//
// `wire:`, `display:`, `summary:` and `code:` are written on the field itself, flat, because that
// is where they have always been read. A command, an event and a type write the same [`Naming`]
// nested under `naming:`, so an author who writes `naming: {wire: orderId}` on a field is writing
// what the model already means (beyond10x/ess#142). Both spellings are read; writing both on one
// field is refused rather than one silently winning, and the field is written back flat, so a
// model's bytes do not depend on which spelling its author chose.
//
// The rustdoc below is the published schema's description of a field, unchanged.
/// One field of a struct or an event.
#[derive(serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Field")]
pub struct RawField {
    /// Its name.
    #[serde(deserialize_with = "deserialize_field_name")]
    #[schemars(regex(pattern = "^_*[A-Za-z][A-Za-z0-9_]*$"))]
    pub name: String,
    /// Its type.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    /// What it is on the wire, and what a person is shown.
    #[serde(default, flatten)]
    pub naming: Naming,
    /// The same naming, written nested as commands and events write theirs. Not beside the flat
    /// keys.
    #[serde(default, rename = "naming")]
    pub nested_naming: Option<Naming>,
    /// How an absent value travels, on an `Optional<T>` field (ess/15). A field key rather than a
    /// naming key, so no other construct's `naming:` can declare one.
    #[serde(default)]
    pub presence: Option<Presence>,
}

/// One field's naming from its two spellings: the flat keys, or the nested `naming:` mapping,
/// with the field's presence policy carried beside the wire name it governs.
///
/// Shared by every field type that reads both, so the refusal is worded once.
pub(crate) fn field_naming(
    flat: Naming,
    nested: Option<Naming>,
    presence: Option<Presence>,
) -> Result<Naming, String> {
    let naming = match nested {
        None => flat,
        Some(nested) if flat.is_empty() => nested,
        Some(_) => {
            return Err(
                "a field's naming is written either flat (`wire:`, `display:`, `summary:`, \
                 `code:`) or nested under `naming:`, not both"
                    .to_owned(),
            )
        }
    };
    Ok(Naming { presence, ..naming })
}

impl TryFrom<RawField> for Field {
    type Error = String;

    fn try_from(raw: RawField) -> Result<Self, Self::Error> {
        Ok(Self {
            name: raw.name,
            type_ref: raw.type_ref,
            naming: field_naming(raw.naming, raw.nested_naming, raw.presence)?,
        })
    }
}

impl schemars::JsonSchema for Field {
    fn schema_name() -> String {
        <RawField as schemars::JsonSchema>::schema_name()
    }

    // The raw form, because the published schema describes what a document may say.
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <RawField as schemars::JsonSchema>::json_schema(generator)
    }
}

/// One variant of an enum.
///
/// Authored either as a bare name — `variants: [Stop, Flag]` — or as a mapping that also says what
/// the variant is called on the wire:
///
/// ```yaml
/// variants:
///   - Flag
///   - name: Stop
///     wire: ""
/// ```
///
/// The second form exists because a variant's wire spelling is not always derivable from its name.
/// A command that is one name over several route shapes has one variant whose path suffix is empty
/// and others that are not, and without a declared spelling the alternatives are splitting the
/// command to satisfy a path or writing the table again in every target.
///
/// A variant that declares nothing serializes back as a bare name, so a document written before
/// this existed keeps its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    /// Its name.
    pub name: String,
    /// What it is called on the wire, and what a person is shown.
    pub naming: Naming,
    /// The typed attributes its enum declares, in declaration order, each with this variant's
    /// value (`ess/23`, beyond10x/ess#450). Empty for an enum that declares none.
    ///
    /// Every variant carries every declared attribute, so the declaration is read off any variant
    /// and no shape that carries variants — the model, the IR — had to change to carry it.
    pub attributes: Vec<VariantAttribute>,
}

/// One typed attribute of an enum variant (`ess/23`, beyond10x/ess#450).
///
/// `attributes: [{name, type}]` on the enum declares it; `attributes: {<name>: <literal>}` on the
/// variant fills it, checked by the rule `sets:` types a literal by. A predicate reads it as
/// `<fact>.<name>`, lowered to membership over the variants that satisfy the comparison, so no
/// runner ever reads an attribute itself.
#[derive(Debug, Clone, Eq, serde::Serialize)]
pub struct VariantAttribute {
    /// The attribute's name, as the enum declares it.
    pub name: String,
    /// Its declared type.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    /// The variant's value, as the canonical text of its literal; `None` where an `Optional`
    /// attribute is left unfilled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// The YAML scalar the value was written as, `None` for text: only the literal rule reads it,
    /// to say `quote it` where text was meant.
    #[serde(skip)]
    pub written: Option<crate::command::ScalarKind>,
}

/// Equal when the name, the type and the value are: how the value was spelled is not part of it.
impl PartialEq for VariantAttribute {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.type_ref == other.type_ref && self.value == other.value
    }
}

/// A variant's attribute value as a document writes it: text, or an unquoted YAML scalar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeLiteral {
    /// The canonical text: as written for text, `true`, `3`, `0.5` for a scalar.
    pub value: String,
    /// The YAML scalar it was written as, `None` for text.
    pub scalar: Option<crate::command::ScalarKind>,
}

impl<'de> serde::Deserialize<'de> for AttributeLiteral {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Literal;
        impl serde::de::Visitor<'_> for Literal {
            type Value = AttributeLiteral;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a literal: text, a boolean or a number")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(AttributeLiteral {
                    value: value.to_owned(),
                    scalar: None,
                })
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(AttributeLiteral {
                    value: value.to_string(),
                    scalar: Some(crate::command::ScalarKind::Boolean),
                })
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(AttributeLiteral {
                    value: value.to_string(),
                    scalar: Some(crate::command::ScalarKind::Integer),
                })
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(AttributeLiteral {
                    value: value.to_string(),
                    scalar: Some(crate::command::ScalarKind::Integer),
                })
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
                crate::command::decimal_text(value)
                    .map(|value| AttributeLiteral {
                        value,
                        scalar: Some(crate::command::ScalarKind::Decimal),
                    })
                    .ok_or_else(|| E::custom("a decimal literal must be a finite number"))
            }
        }
        deserializer.deserialize_any(Literal)
    }
}

impl serde::Serialize for AttributeLiteral {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.value)
    }
}

impl schemars::JsonSchema for AttributeLiteral {
    fn schema_name() -> String {
        "AttributeLiteral".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{InstanceType, SchemaObject, SingleOrVec};
        SchemaObject {
            instance_type: Some(SingleOrVec::Vec(vec![
                InstanceType::String,
                InstanceType::Boolean,
                InstanceType::Number,
            ])),
            ..Default::default()
        }
        .into()
    }
}

impl EnumVariant {
    /// A variant with no naming overrides.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            naming: Naming::default(),
            attributes: Vec::new(),
        }
    }

    /// The attribute of that name, when its enum declares one.
    pub fn attribute(&self, name: &str) -> Option<&VariantAttribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == name)
    }

    /// Its name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Its wire spelling, falling back to its name.
    ///
    /// An authored empty string is a wire spelling like any other — it is how the variant that
    /// carries no path suffix is written — so this returns it rather than treating it as absent.
    pub fn wire(&self) -> &str {
        self.naming.wire.as_deref().unwrap_or(&self.name)
    }

    /// `true` when nothing beyond the name is declared.
    pub fn is_bare(&self) -> bool {
        self.naming.is_empty()
    }

    /// Variants that declare nothing beyond their names, in the order given.
    pub fn bare<I, S>(names: I) -> Vec<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        names.into_iter().map(Self::new).collect()
    }
}

impl From<&str> for EnumVariant {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for EnumVariant {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

/// A variant reads as its name everywhere a name was read before this type existed.
///
/// The alternative was `variant.name()` at every one of those positions, which is the same string
/// with more places to get it wrong during the change that introduced the type.
impl std::ops::Deref for EnumVariant {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.name
    }
}

impl AsRef<str> for EnumVariant {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl PartialEq<str> for EnumVariant {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

impl PartialEq<&str> for EnumVariant {
    fn eq(&self, other: &&str) -> bool {
        self.name == *other
    }
}

impl PartialEq<String> for EnumVariant {
    fn eq(&self, other: &String) -> bool {
        &self.name == other
    }
}

/// The mapping form, used only to read a variant that declares naming or attributes, and to write
/// one that declares naming.
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct NamedEnumVariant {
    name: String,
    #[serde(default, flatten, skip_serializing_if = "Naming::is_empty")]
    naming: Naming,
    /// The variant's value for each attribute its enum declares (`ess/23`, beyond10x/ess#450).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    attributes: BTreeMap<String, AttributeLiteral>,
}

/// What a variant that declares attributes is written as: its naming, and each attribute typed.
#[derive(serde::Serialize)]
struct AttributedEnumVariant<'a> {
    name: &'a str,
    #[serde(flatten, skip_serializing_if = "Naming::is_empty")]
    naming: &'a Naming,
    attributes: &'a [VariantAttribute],
}

impl serde::Serialize for EnumVariant {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !self.attributes.is_empty() {
            return AttributedEnumVariant {
                name: &self.name,
                naming: &self.naming,
                attributes: &self.attributes,
            }
            .serialize(serializer);
        }
        if self.naming.is_empty() {
            return serializer.serialize_str(&self.name);
        }
        NamedEnumVariant {
            name: self.name.clone(),
            naming: self.naming.clone(),
            attributes: BTreeMap::new(),
        }
        .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for EnumVariant {
    /// Reads either authored form.
    ///
    /// A visitor rather than `#[serde(untagged)]`: an untagged enum discards the inner error and
    /// reports `data did not match any variant`, so a misspelt key inside the mapping form would
    /// stop naming the key. Forwarding the map to the mapping form keeps that refusal, which
    /// `deny_unknown_fields` on it produces.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Authored;

        impl<'de> serde::de::Visitor<'de> for Authored {
            type Value = EnumVariant;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a variant name, or a mapping carrying `name` and its naming")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(EnumVariant::new(value))
            }

            // YAML reads an unquoted `True` as a boolean before a variant list sees it; the
            // refusal names the repair (beyond10x/ess#426). A type's `variants:` is read through
            // `RawEnumVariant`, which refuses it at the type's path instead.
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Err(E::custom(format!(
                    "`{value}` is a YAML boolean, not a variant name; quote it, as the text the \
                     variant is named"
                )))
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                let NamedEnumVariant {
                    name,
                    naming,
                    attributes,
                } = <NamedEnumVariant as serde::Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                if !attributes.is_empty() {
                    return Err(serde::de::Error::custom(
                        "`attributes:` is written on the variants of a declared enum type",
                    ));
                }
                Ok(EnumVariant {
                    name,
                    naming,
                    attributes: Vec::new(),
                })
            }
        }

        deserializer.deserialize_any(Authored)
    }
}

impl schemars::JsonSchema for EnumVariant {
    fn schema_name() -> String {
        "EnumVariant".to_owned()
    }

    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        // No name pattern on either form. A variant name has never been checked against one on
        // the way in, and publishing a pattern the parser does not enforce is the inconsistency
        // `Field::PATTERN` exists to prevent, in the direction where the editor refuses a document
        // this repository accepts.
        let bare = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };

        let mut schema = schemars::schema::SchemaObject::default();
        schema.subschemas().any_of = Some(vec![
            bare.into(),
            generator.subschema_for::<NamedEnumVariant>(),
        ]);
        schema.metadata().description = Some(
            "An enum variant: a bare name, or a mapping that also declares its wire spelling."
                .to_owned(),
        );
        schema.into()
    }
}

/// Checks that `value` can be spelled as a field name.
///
/// A field name is not decoration: it becomes a struct field in generated code, a key on the wire
/// and a property in an `OpenAPI` document, so `""` and `not a field name!` each become three
/// things nobody can spell, in files where the specification that wrote them is no longer in view.
/// [`StateName`](crate::entity::StateName), [`OutcomeName`](crate::command::OutcomeName) and
/// [`QualifiedName`] check theirs for the same reason.
///
/// Leading underscores are admitted before the first letter (beyond10x/ess#141): `_url` and `__v`
/// are wire names real services publish, and every target this repository emits for has a
/// deterministic identifier for them. `_` and `_1` stay refused, because once the underscores are
/// taken off nothing is left that starts like a name.
pub(crate) fn field_name(value: &str) -> Result<String, ParseError> {
    let reject = |reason: String| Err(ParseError::identifier("field name", value, reason));

    if value.is_empty() {
        return reject("must not be empty".to_owned());
    }
    let Some(first) = value.trim_start_matches('_').chars().next() else {
        return reject("must have a letter after its leading underscores, as in `_url`".to_owned());
    };
    if !first.is_ascii_alphabetic() {
        return reject(format!(
            "must start with a letter, optionally after underscores, as in `invoice_id` or \
             `_url`, got {first:?}"
        ));
    }
    for character in value.chars() {
        if !(character.is_ascii_alphanumeric() || character == '_') {
            return reject(format!(
                "contains {character:?}; a field name has to survive into generated code as an \
                 identifier"
            ));
        }
    }
    Ok(value.to_owned())
}

/// `true` when `value` is a field name: [`Field::PATTERN`], decided by the parser that enforces it.
///
/// For the checks outside this crate that hold a persisted name to the field-name rule — a suite's
/// reading member, a periodic contract's input, an entity setup's field. Each used to spell the
/// rule itself, and each went on refusing `_url` after the specification admitted it.
pub fn is_field_name(value: &str) -> bool {
    field_name(value).is_ok()
}

/// Serde entry point for [`field_name`], so a name nothing could generate is refused while the
/// document is read rather than surviving into the model.
pub(crate) fn deserialize_field_name<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let raw = <String as serde::Deserialize>::deserialize(deserializer)?;
    field_name(&raw).map_err(serde::de::Error::custom)
}

/// What a reader does with a field a record does not declare (`ess/24`, beyond10x/ess#500).
///
/// Written `undeclared_fields:` on a `kind: struct` type and on a command, where it governs the
/// command's `response:` only. A record is closed unless it says otherwise: `refused` is the
/// default, and every declaration that does not write the key keeps the bytes it had. `ignored`
/// admits members beyond the declared ones, as a protocol does that lets a producer add extension
/// members its readers must ignore; every declared field stays required and typed either way.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum UndeclaredFields {
    /// A field the record does not declare is refused. The default.
    #[default]
    Refused,
    /// A field the record does not declare is ignored; the declared ones are still checked.
    Ignored,
}

impl UndeclaredFields {
    /// The key it is written under.
    pub const KEY: &'static str = "undeclared_fields";

    /// `true` for the default, closed record: what serialization skips, so a declaration that does
    /// not ignore undeclared fields keeps its bytes.
    #[allow(clippy::trivially_copy_pass_by_ref)] // Serde's skip_serializing_if passes a reference.
    pub fn is_refused(&self) -> bool {
        *self == Self::Refused
    }

    /// `true` when a field the record does not declare is admitted.
    pub fn is_ignored(self) -> bool {
        self == Self::Ignored
    }

    /// What `written` amounts to: [`Self::Refused`] where nothing was written.
    pub fn of(written: Option<Self>) -> Self {
        written.unwrap_or_default()
    }

    /// How it is written.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Refused => "refused",
            Self::Ignored => "ignored",
        }
    }
}

impl fmt::Display for UndeclaredFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What a named type is made of.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeBody {
    /// A wrapper around one representation, distinct from it.
    ///
    /// This is what makes `Email` refusable where `String` is expected.
    Newtype {
        /// What it wraps.
        #[serde(rename = "of")]
        of: TypeRef,
        /// The characters every value is drawn from, when declared (ess/11).
        ///
        /// A set written in a fixed order: every character of a value is one of these, decided per
        /// Unicode scalar value with no normalization or case folding. The order is canonical,
        /// which is why a character written twice is refused, and it is what the witness maps a
        /// field's own text into (`docs/design/string-alphabet-and-length.md`, section 4).
        #[serde(skip_serializing_if = "Option::is_none")]
        alphabet: Option<String>,
        /// The text every value starts with, when declared (ess/15, beyond10x/ess#146).
        ///
        /// A literal prefix, compared character for character with no normalization; not a
        /// pattern. Only a newtype of `String` takes one, its characters are in the effective
        /// alphabet, and nested prefixes extend one another
        /// (`docs/design/wire-presence-json-prefix.md`).
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        /// Conditions every value must satisfy, as predicates over `value`, what it wraps.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        invariants: Vec<Invariant>,
    },
    /// Named fields.
    Struct {
        /// Its fields.
        fields: Vec<Field>,
        /// Conditions every value must satisfy, as predicates over those fields.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        invariants: Vec<Invariant>,
        /// What a reader does with a field this struct does not declare, as written (`ess/24`,
        /// beyond10x/ess#500). `None` where the declaration does not say, which is `refused`.
        #[serde(skip_serializing_if = "Option::is_none")]
        undeclared_fields: Option<UndeclaredFields>,
    },
    /// One of a fixed set of names.
    Enum {
        /// The variants, in declaration order.
        variants: Vec<EnumVariant>,
    },
    /// One of several shapes, distinguished by a tag field.
    ///
    /// Always tagged: an untagged union does not round-trip, because two variants with the same
    /// shape cannot be told apart on the way back.
    Union {
        /// The field carrying the variant's name.
        tag: String,
        /// The variants, by tag value, each with the payload it carries.
        ///
        /// `None` is a unit variant (ess/22, beyond10x/ess#418): it carries nothing, and its value
        /// on the wire is the tag alone (`docs/design/union-unit-variants.md`).
        variants: BTreeMap<String, Option<TypeRef>>,
    },
}

/// A named type in a specification.
///
/// The body is flattened, so a declaration reads as one object: `{name, kind, fields}` rather than
/// `{name, body: {kind, fields}}`. A document becomes one of these through
/// [`RawNamedType`], which is where a misspelled key is refused.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NamedType {
    /// Its stable identity.
    pub name: QualifiedName,
    /// What it is made of.
    #[serde(flatten)]
    pub body: TypeBody,
    /// What it is called on the wire and shown as.
    #[serde(skip_serializing_if = "Naming::is_empty")]
    pub naming: Naming,
    /// Optional clock-reading contract; absence retains the legacy type meaning and bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading: Option<crate::reading::ReadingContract>,
}

impl NamedType {
    /// The pseudo-field a newtype's invariants read.
    ///
    /// A newtype has no fields — it is one representation under another name — so an invariant on
    /// it names what it wraps, the way an entity's invariant names
    /// [`state`](crate::entity::EntitySpec::STATE) for a lifecycle no field carries.
    pub const VALUE: &'static str = "value";

    /// Refuses a body that declares nothing.
    ///
    /// A type with no variants and a struct with no fields both parse, and both name something no
    /// value can be: every generator downstream would emit an enum with no cases or a struct with no
    /// members, and the first place anyone notices is a compiler error in generated code, where the
    /// mistake is furthest from the document that caused it.
    fn check_shape(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at = |part: &str| format!("types.{}.{part}", self.name);

        if let Some(reading) = &self.reading {
            let valid_representation = matches!(&self.body, TypeBody::Newtype { of: TypeRef::Primitive(primitive), .. } if *primitive == reading.representation());
            if !valid_representation {
                errors.push(ValidationError::new(ValidationCode::TypeMismatch, at("reading"), "reading requires a directly String/Integer-backed newtype matching its encoding"));
            }
            if let Err(reason) = reading.validate() {
                errors.push(ValidationError::new(
                    ValidationCode::TypeMismatch,
                    at("reading"),
                    reason,
                ));
            }
        }

        let (part, empty) = match &self.body {
            TypeBody::Enum { variants } => ("variants", variants.is_empty()),
            TypeBody::Union { variants, .. } => ("variants", variants.is_empty()),
            TypeBody::Struct { fields, .. } => ("fields", fields.is_empty()),
            TypeBody::Newtype { .. } => ("of", false),
        };
        if empty {
            errors.push(
                ValidationError::new(
                    ValidationCode::EmptyDeclaration,
                    at(part),
                    format!("`{}` declares no {part}, so no value can be one", self.name),
                )
                .with_hint("give it at least one, or delete the declaration"),
            );
        }

        if let TypeBody::Newtype {
            alphabet: Some(alphabet),
            ..
        } = &self.body
        {
            errors.extend(self.check_alphabet(alphabet));
        }

        if let TypeBody::Newtype {
            prefix: Some(prefix),
            ..
        } = &self.body
        {
            if prefix.is_empty() {
                errors.push(
                    ValidationError::new(
                        ValidationCode::EmptyDeclaration,
                        at("prefix"),
                        format!(
                            "`{}` declares an empty prefix, which every text already starts with",
                            self.name
                        ),
                    )
                    .with_hint("write the text every value starts with, or delete `prefix:`"),
                );
            }
        }

        // A tagged union whose tag collides with a variant's own field is decodable only by luck.
        if let TypeBody::Union { tag, .. } = &self.body {
            if tag.is_empty() {
                errors.push(
                    ValidationError::new(
                        ValidationCode::EmptyDeclaration,
                        at("tag"),
                        format!("`{}` is a union with no tag field", self.name),
                    )
                    .with_hint(
                        "an untagged union cannot be decoded without guessing; name the field that \
                         carries the variant",
                    ),
                );
            }
        }

        errors
    }

    /// The rules an alphabet keeps on its own: it holds a character, and each only once.
    ///
    /// A character written twice is refused rather than read as one, so the written order is the
    /// canonical order: two spellings of one set would diff as a change and synthesize different
    /// witnesses (`docs/design/string-alphabet-and-length.md`, section 1).
    fn check_alphabet(&self, alphabet: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at = format!("types.{}.alphabet", self.name);
        if alphabet.is_empty() {
            errors.push(
                ValidationError::new(
                    ValidationCode::EmptyDeclaration,
                    at.clone(),
                    format!(
                        "`{}` declares an empty alphabet, which admits no character",
                        self.name
                    ),
                )
                .with_hint("list the characters a value may hold, or delete `alphabet:`"),
            );
        }
        let mut first: BTreeMap<char, usize> = BTreeMap::new();
        for (index, character) in alphabet.chars().enumerate() {
            let position = index + 1;
            if let Some(earlier) = first.get(&character) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        at.clone(),
                        format!(
                            "`{}` writes {character:?} twice in its alphabet, at positions \
                             {earlier} and {position}",
                            self.name
                        ),
                    )
                    .with_hint(
                        "an alphabet is a set written in a fixed order; write each character once",
                    ),
                );
            } else {
                first.insert(character, position);
            }
        }
        errors
    }

    /// Check a declared alphabet against the registry: what it sits on is text, and it shares a
    /// character with every alphabet it wraps.
    pub fn validate_alphabet(&self, registry: &TypeRegistry) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let TypeBody::Newtype {
            of,
            alphabet: Some(alphabet),
            ..
        } = &self.body
        else {
            return errors;
        };
        let at = format!("types.{}.alphabet", self.name);
        let layers = registry.newtype_layers(of);
        match &layers.terminal {
            TypeRef::Primitive(Primitive::String) if !layers.optional => {}
            // An unresolved name is its own refusal, made where references are resolved.
            TypeRef::Named(name) if registry.get(name).is_none() => {}
            other => {
                let reached = match other {
                    TypeRef::Named(name) => format!("`{name}`"),
                    _ if layers.optional => format!("`{of}`, which may be absent,"),
                    _ => format!("`{other}`"),
                };
                errors.push(
                    ValidationError::new(
                        ValidationCode::TypeMismatch,
                        at.clone(),
                        format!(
                            "`{}` declares an alphabet over `{of}`, which is {reached} rather \
                             than text",
                            self.name
                        ),
                    )
                    .with_hint("only a newtype of `String`, at any depth, takes `alphabet:`"),
                );
            }
        }
        for inner in &layers.newtypes {
            let TypeBody::Newtype {
                alphabet: Some(held),
                ..
            } = &inner.body
            else {
                continue;
            };
            if !alphabet.is_empty() && !alphabet.chars().any(|character| held.contains(character)) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        at.clone(),
                        format!(
                            "`{}`'s alphabet shares no character with the alphabet of `{}`, which \
                             it wraps, so no text but the empty one is a value of it",
                            self.name, inner.name
                        ),
                    )
                    .with_hint(
                        "a nested alphabet narrows the one it wraps; keep at least one character \
                         both hold",
                    ),
                );
            }
        }
        errors
    }

    /// Check a declared prefix against the registry: what it sits on is text, it extends or is
    /// extended by every prefix it wraps, and every character of the effective prefix is in the
    /// effective alphabet (beyond10x/ess#146).
    pub fn validate_prefix(&self, registry: &TypeRegistry) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let TypeBody::Newtype {
            of,
            prefix,
            alphabet,
            ..
        } = &self.body
        else {
            return errors;
        };
        let at = format!("types.{}.prefix", self.name);
        if let Some(prefix) = prefix {
            let layers = registry.newtype_layers(of);
            match &layers.terminal {
                TypeRef::Primitive(Primitive::String) if !layers.optional => {}
                TypeRef::Named(name) if registry.get(name).is_none() => {}
                other => {
                    let reached = match other {
                        TypeRef::Named(name) => format!("`{name}`"),
                        _ if layers.optional => format!("`{of}`, which may be absent,"),
                        _ => format!("`{other}`"),
                    };
                    errors.push(
                        ValidationError::new(
                            ValidationCode::TypeMismatch,
                            at.clone(),
                            format!(
                                "`{}` declares a prefix over `{of}`, which is {reached} rather \
                                 than text",
                                self.name
                            ),
                        )
                        .with_hint("only a newtype of `String`, at any depth, takes `prefix:`"),
                    );
                }
            }
            for inner in &layers.newtypes {
                let TypeBody::Newtype {
                    prefix: Some(held), ..
                } = &inner.body
                else {
                    continue;
                };
                if !(prefix.starts_with(held.as_str()) || held.starts_with(prefix.as_str())) {
                    errors.push(
                        ValidationError::new(
                            ValidationCode::ConflictingDeclaration,
                            at.clone(),
                            format!(
                                "`{}`'s prefix {prefix:?} and the prefix {held:?} of `{}`, which \
                                 it wraps, do not extend one another, so no text starts with both",
                                self.name, inner.name
                            ),
                        )
                        .with_hint(
                            "a nested prefix narrows the one it wraps; start it with that prefix",
                        ),
                    );
                }
            }
        }
        // Reported where a prefix or an alphabet is declared, so a chain that combines one layer's
        // prefix with another's alphabet is refused at the layer that brought them together.
        if prefix.is_some() || alphabet.is_some() {
            let this = TypeRef::Named(self.name.clone());
            if let (Some(effective), Some(characters)) = (
                registry.effective_prefix(&this),
                registry.effective_alphabet(&this),
            ) {
                if let Some(outside) = effective
                    .chars()
                    .find(|character| !characters.contains(*character))
                {
                    errors.push(
                        ValidationError::new(
                            ValidationCode::ConflictingDeclaration,
                            if prefix.is_some() {
                                at
                            } else {
                                format!("types.{}.alphabet", self.name)
                            },
                            format!(
                                "every value of `{}` starts with the prefix {effective:?}, and \
                                 {outside:?} is not in its alphabet (\"{characters}\"), so no \
                                 text is a value of it",
                                self.name
                            ),
                        )
                        .with_hint("keep the prefix's characters inside the alphabet"),
                    );
                }
            }
        }
        errors
    }

    /// Every named type this one depends on.
    pub fn dependencies(&self) -> Vec<&QualifiedName> {
        match &self.body {
            TypeBody::Newtype { of, .. } => of.named_dependencies(),
            TypeBody::Struct { fields, .. } => fields
                .iter()
                .flat_map(|field| field.type_ref.named_dependencies())
                .collect(),
            TypeBody::Enum { .. } => Vec::new(),
            TypeBody::Union { variants, .. } => variants
                .values()
                .flatten()
                .flat_map(TypeRef::named_dependencies)
                .collect(),
        }
    }

    /// The field with this name, for a struct.
    pub fn field(&self, name: &str) -> Option<&Field> {
        match &self.body {
            TypeBody::Struct { fields, .. } => fields.iter().find(|field| field.name == name),
            _ => None,
        }
    }

    /// Check an enum's typed attributes against the complete registry (`ess/23`,
    /// beyond10x/ess#450): the format admits them, each declared type is one an attribute may
    /// have, no name is declared twice, and each variant's value is one of its attribute's type by
    /// the rule `sets:` types a literal by.
    pub fn validate_attributes(&self, registry: &TypeRegistry) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let TypeBody::Enum { variants } = &self.body else {
            return errors;
        };
        let Some(declared) = variants.first().map(|variant| &variant.attributes) else {
            return errors;
        };
        if declared.is_empty() {
            return errors;
        }
        let at = format!("types.{}.attributes", self.name);
        if registry
            .format()
            .is_some_and(|format| format.major() < crate::system::FormatVersion::V23.major())
        {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    at,
                    format!(
                        "typed attributes on the variants of `{}` require specification format \
                         ess/23",
                        self.name
                    ),
                )
                .with_hint("write `format: ess/23` on the source that declares the system"),
            );
            return errors;
        }
        let mut seen = BTreeSet::new();
        let mut admitted = BTreeSet::new();
        for attribute in declared {
            if !seen.insert(attribute.name.as_str()) {
                errors.push(ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    format!("{at}.{}", attribute.name),
                    format!(
                        "`{}` declares the attribute `{}` twice",
                        self.name, attribute.name
                    ),
                ));
                continue;
            }
            match attribute_type(&attribute.type_ref, registry) {
                Ok(()) => {
                    admitted.insert(attribute.name.as_str());
                }
                Err((code, reason)) => errors.push(
                    ValidationError::new(
                        code,
                        format!("{at}.{}", attribute.name),
                        format!(
                            "the attribute `{}` of `{}` is `{}`, {reason}",
                            attribute.name, self.name, attribute.type_ref
                        ),
                    )
                    .with_hint(
                        "an attribute is a `Boolean`, an `Integer`, a `Decimal`, a `String`, a \
                         newtype of one, an enum, or an `Optional` of one of these; a `List` \
                         attribute is not in this cut",
                    ),
                ),
            }
        }
        let inhabitation = crate::system::Inhabitation::of(registry);
        let conversions = ConversionRegistry::new();
        for variant in variants {
            for attribute in &variant.attributes {
                let Some(value) = &attribute.value else {
                    continue;
                };
                if !admitted.contains(attribute.name.as_str()) {
                    continue;
                }
                let held = Field::new(attribute.name.clone(), attribute.type_ref.clone());
                if let Some((reason, hint)) = crate::command::literal_refusal(
                    (&self.name, &format!("{}.{}", variant.name, attribute.name)),
                    &held,
                    (value, attribute.written),
                    "variant's `attributes:`",
                    &self.name,
                    (registry, &conversions, &inhabitation),
                ) {
                    errors.push(
                        ValidationError::new(
                            ValidationCode::TypeMismatch,
                            format!(
                                "types.{}.variants.{}.attributes.{}",
                                self.name, variant.name, attribute.name
                            ),
                            reason,
                        )
                        .with_hint(hint),
                    );
                }
            }
        }
        errors
    }

    /// Check complete invariant paths after every named and lifecycle type is registered.
    pub fn validate_invariants(&self, registry: &TypeRegistry) -> ValidationErrors {
        let value;
        let (fields, invariants) = match &self.body {
            TypeBody::Struct {
                fields, invariants, ..
            } => (fields.as_slice(), invariants),
            TypeBody::Newtype { of, invariants, .. } => {
                value = [Field::new(Self::VALUE, of.clone())];
                (value.as_slice(), invariants)
            }
            TypeBody::Enum { .. } | TypeBody::Union { .. } => return ValidationErrors::new(),
        };
        let environment = crate::expression::DomainEnvironment::new(registry, fields);
        let mut errors = ValidationErrors::new();
        for (index, invariant) in invariants.iter().enumerate() {
            errors.extend(
                crate::expression::check_predicate(
                    &environment,
                    &invariant.predicate,
                    &format!("types.{}.invariants[{index}]", self.name),
                )
                .validation_errors(),
            );
        }
        errors
    }

    /// Checks that every invariant reads something this type has (§36.6).
    ///
    /// Only this type's own fields are resolvable here: the check runs while the declaration is
    /// being converted, before any [`TypeRegistry`] exists, so a path that leaves the type
    /// (`total.amount`) is checked as far as `total` and no further. The registry-aware
    /// [`Self::validate_invariants`] pass checks complete paths and operands at specification admission.
    fn check_invariants(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let (invariants, readable, hint) = match &self.body {
            TypeBody::Newtype { invariants, .. } => (
                invariants,
                vec![Self::VALUE.to_owned()],
                format!(
                    "a newtype has no fields; its invariants read `{}`, the representation it wraps",
                    Self::VALUE
                ),
            ),
            TypeBody::Struct {
                fields, invariants, ..
            } => {
                let names: Vec<String> =
                    fields.iter().map(|field| field.name.clone()).collect();
                let hint = format!("readable here: {}", names.join(", "));
                (invariants, names, hint)
            }
            TypeBody::Enum { .. } | TypeBody::Union { .. } => return errors,
        };

        for (index, invariant) in invariants.iter().enumerate() {
            let at = format!("types.{}.invariants[{index}]", self.name);
            for path in invariant.predicate.fact_paths() {
                let root = path.namespace();
                if readable.iter().any(|name| name == root) {
                    continue;
                }
                errors.push(
                    ValidationError::new(
                        ValidationCode::UnobservableFact,
                        at.clone(),
                        format!(
                            "`{invariant}` reads `{path}`, and `{root}` is not a field of `{}`",
                            self.name
                        ),
                    )
                    .with_hint(hint.clone()),
                );
            }
        }

        errors
    }
}

impl schemars::JsonSchema for NamedType {
    fn schema_name() -> String {
        <RawNamedType as schemars::JsonSchema>::schema_name()
    }

    // Delegated to the raw form, because the published schema has to describe what a document may
    // say, and the raw form is what a document is read into.
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <RawNamedType as schemars::JsonSchema>::json_schema(generator)
    }
}

/// What a named type is made of, as parsed.
///
/// `deny_unknown_fields` sits here rather than on [`RawNamedType`], which cannot carry it: serde
/// ignores the attribute on a struct with a flattened field. The body is what the keys nothing else
/// claimed are offered to, so refusing an unknown one here is what refuses `invarants:` and
/// `namin:` — each of which used to parse clean and silently drop what the author wrote.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RawTypeBody {
    /// A wrapper around one representation, distinct from it.
    Newtype {
        /// What it wraps.
        #[serde(rename = "of")]
        of: TypeRef,
        /// The characters every value is drawn from (ess/11); only a newtype of `String` takes one.
        #[serde(default)]
        alphabet: Option<String>,
        /// The text every value starts with (ess/15); only a newtype of `String` takes one.
        #[serde(default)]
        prefix: Option<String>,
        /// Conditions every value must satisfy, as predicates over `value`, what it wraps.
        #[serde(default)]
        invariants: Vec<RawInvariant>,
    },
    /// Named fields.
    Struct {
        /// Its fields.
        fields: Vec<Field>,
        /// Conditions every value must satisfy, as predicates over those fields.
        #[serde(default)]
        invariants: Vec<RawInvariant>,
    },
    /// One of a fixed set of names.
    Enum {
        /// The variants, in declaration order.
        variants: Vec<RawEnumVariant>,
        /// The typed attributes every variant gives a value (`ess/23`, beyond10x/ess#450), in the
        /// `{name, type}` shape of `fields:`.
        #[serde(default)]
        attributes: Vec<Field>,
    },
    /// One of several shapes, distinguished by a tag field.
    Union {
        /// The field carrying the variant's name.
        tag: String,
        /// The variants, by tag value, each with the payload it carries.
        ///
        /// A variant written with no type (`Open:` or `Open: ~`) is a unit variant (ess/22,
        /// beyond10x/ess#418): it carries nothing, and its value on the wire is the tag alone.
        variants: BTreeMap<String, Option<TypeRef>>,
    },
}

/// An enum variant as a document writes it (beyond10x/ess#426).
///
/// A variant, or a YAML boolean where a name was meant: `variants: [True, False, Unknown]` reads
/// `True` and `False` as booleans before ESS sees them. The boolean is kept, rather than refused
/// by the reader with no key path, so the type's own pass refuses it at its `variants` with the
/// repair — quoting it — and never guesses which spelling, `True`, `true` or `TRUE`, was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawEnumVariant {
    /// A variant.
    Declared(EnumVariant),
    /// A variant that gives values to its enum's attributes (`ess/23`, beyond10x/ess#450), each
    /// typed once the enum's declaration is read.
    Attributed(EnumVariant, BTreeMap<String, AttributeLiteral>),
    /// A YAML boolean written where a variant name was meant.
    Boolean(bool),
}

impl RawEnumVariant {
    /// The variant, where a name was written rather than a boolean.
    pub fn declared(&self) -> Option<&EnumVariant> {
        match self {
            Self::Declared(variant) | Self::Attributed(variant, _) => Some(variant),
            Self::Boolean(_) => None,
        }
    }
}

impl From<EnumVariant> for RawEnumVariant {
    fn from(variant: EnumVariant) -> Self {
        Self::Declared(variant)
    }
}

impl<'de> serde::Deserialize<'de> for RawEnumVariant {
    /// [`EnumVariant`]'s two authored forms, and a boolean kept rather than refused.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Written;

        impl<'de> serde::de::Visitor<'de> for Written {
            type Value = RawEnumVariant;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a variant name, or a mapping carrying `name` and its naming")
            }

            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(RawEnumVariant::Boolean(value))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(RawEnumVariant::Declared(EnumVariant::new(value)))
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                let NamedEnumVariant {
                    name,
                    naming,
                    attributes,
                } = <NamedEnumVariant as serde::Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                let variant = EnumVariant {
                    name,
                    naming,
                    attributes: Vec::new(),
                };
                Ok(if attributes.is_empty() {
                    RawEnumVariant::Declared(variant)
                } else {
                    RawEnumVariant::Attributed(variant, attributes)
                })
            }
        }

        deserializer.deserialize_any(Written)
    }
}

impl schemars::JsonSchema for RawEnumVariant {
    fn schema_name() -> String {
        <EnumVariant as schemars::JsonSchema>::schema_name()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        <EnumVariant as schemars::JsonSchema>::schema_id()
    }

    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <EnumVariant as schemars::JsonSchema>::json_schema(generator)
    }
}

impl RawTypeBody {
    /// The body, with the refusal of every invariant that does not parse added to `errors`,
    /// located under `location` (beyond10x/ess#448).
    fn read(self, location: &str, errors: &mut ValidationErrors) -> TypeBody {
        let at = |index: usize| PredicateAt::from(format!("{location}.invariants[{index}]"));
        match self {
            Self::Newtype {
                of,
                alphabet,
                prefix,
                invariants,
            } => TypeBody::Newtype {
                of,
                alphabet,
                prefix,
                invariants: RawInvariant::read_all(invariants, at, errors),
            },
            // The struct's `undeclared_fields:` is read beside the body, not in it (`ess/24`,
            // `crate::undeclared_fields`), and set once the type is converted.
            Self::Struct { fields, invariants } => TypeBody::Struct {
                fields,
                invariants: RawInvariant::read_all(invariants, at, errors),
                undeclared_fields: None,
            },
            Self::Enum {
                variants,
                attributes,
            } => {
                let mut booleans = Vec::new();
                let variants = variants
                    .into_iter()
                    .filter_map(|variant| match variant {
                        RawEnumVariant::Declared(variant) => {
                            Some(attributed(variant, BTreeMap::new(), &attributes))
                        }
                        RawEnumVariant::Attributed(variant, values) => {
                            Some(attributed(variant, values, &attributes))
                        }
                        RawEnumVariant::Boolean(value) => {
                            booleans.push(value);
                            None
                        }
                    })
                    .collect();
                if !booleans.is_empty() {
                    errors.push(boolean_variants(location, &booleans));
                }
                TypeBody::Enum { variants }
            }
            Self::Union { tag, variants } => TypeBody::Union { tag, variants },
        }
    }
}

/// `types.<name>`, where the refusals a named type's own declaration raises are located.
fn declared_at(name: &QualifiedName) -> String {
    format!("types.{name}")
}

/// Whether an enum attribute may have the type `reference` (`ess/23`, beyond10x/ess#450): a
/// `Boolean`, `Integer`, `Decimal` or `String`, a newtype of one, an enum, or an `Optional` of any
/// of these — the types a literal spells. Otherwise the refusal's code and the clause that says why.
fn attribute_type(
    reference: &TypeRef,
    registry: &TypeRegistry,
) -> Result<(), (ValidationCode, String)> {
    let mut current = reference;
    for _ in 0..=MAX_TYPE_DEPTH {
        match current {
            TypeRef::Optional(inner) => current = inner,
            TypeRef::Primitive(
                Primitive::Boolean | Primitive::Integer | Primitive::Decimal | Primitive::String,
            ) => return Ok(()),
            TypeRef::Primitive(primitive) => {
                return Err((
                    ValidationCode::UnsupportedConstruct,
                    format!("and `{primitive}` has no literal an attribute could be written as"),
                ))
            }
            TypeRef::List(_) | TypeRef::Map(..) => {
                return Err((
                    ValidationCode::UnsupportedConstruct,
                    "and a `List` or `Map` attribute is not in this cut".to_owned(),
                ))
            }
            TypeRef::Named(name) => match registry.get(name).map(|declared| &declared.body) {
                None => {
                    return Err((
                        ValidationCode::UndeclaredReference,
                        format!("and nothing declares `{name}`"),
                    ))
                }
                Some(TypeBody::Newtype { of, .. }) => current = of,
                Some(TypeBody::Enum { .. }) => return Ok(()),
                Some(TypeBody::Struct { .. } | TypeBody::Union { .. }) => {
                    return Err((
                        ValidationCode::UnsupportedConstruct,
                        format!("and `{name}` has structure, which no literal spells"),
                    ))
                }
            },
        }
    }
    Err((
        ValidationCode::SelfReference,
        "whose wrappers do not end".to_owned(),
    ))
}

/// `variant` with a value for each attribute its enum declares (`ess/23`, beyond10x/ess#450), in
/// declaration order; `None` for one it leaves out. What it leaves out or adds is refused before
/// the conversion ([`RawNamedType::withhold_unread`]), and whether each value is one of its
/// attribute's type needs the complete registry, so [`NamedType::validate_attributes`] decides it.
fn attributed(
    mut variant: EnumVariant,
    mut values: BTreeMap<String, AttributeLiteral>,
    declared: &[Field],
) -> EnumVariant {
    for attribute in declared {
        let value = values.remove(&attribute.name);
        variant.attributes.push(VariantAttribute {
            name: attribute.name.clone(),
            type_ref: attribute.type_ref.clone(),
            written: value.as_ref().and_then(|value| value.scalar),
            value: value.map(|value| value.value),
        });
    }
    variant
}

/// The refusal of every required attribute of the enum at `location` that `variant` gives no
/// value (`ess/23`, beyond10x/ess#450).
fn unfilled(
    location: &str,
    variant: &EnumVariant,
    values: &BTreeMap<String, AttributeLiteral>,
    declared: &[Field],
    errors: &mut ValidationErrors,
) {
    let owner = location.strip_prefix("types.").unwrap_or(location);
    for attribute in declared {
        if values.contains_key(&attribute.name) || attribute.type_ref.is_optional() {
            continue;
        }
        errors.push(
            ValidationError::new(
                ValidationCode::MissingDeclaration,
                format!("{location}.variants.{}.attributes", variant.name),
                format!(
                    "variant `{}` of `{owner}` gives no value for the attribute `{}`, which is \
                     `{}`",
                    variant.name, attribute.name, attribute.type_ref
                ),
            )
            .with_hint(format!(
                "write `attributes: {{{}: …}}` on the variant, or declare the attribute \
                 `Optional<{}>` if a variant may leave it out",
                attribute.name, attribute.type_ref
            )),
        );
    }
}

/// The refusal of every value `variant` gives an attribute the enum at `location` does not
/// declare, each withheld (`ess/23`, beyond10x/ess#450).
fn undeclared(
    location: &str,
    variant: &EnumVariant,
    values: &mut BTreeMap<String, AttributeLiteral>,
    declared: &[Field],
    errors: &mut ValidationErrors,
) {
    let owner = location.strip_prefix("types.").unwrap_or(location);
    let added: Vec<String> = values
        .keys()
        .filter(|name| !declared.iter().any(|attribute| &&attribute.name == name))
        .cloned()
        .collect();
    for name in added {
        values.remove(&name);
        errors.push(
            ValidationError::new(
                ValidationCode::UndeclaredReference,
                format!("{location}.variants.{}.attributes.{name}", variant.name),
                format!(
                    "variant `{}` gives a value for `{name}`, which is not an attribute `{owner}` \
                     declares",
                    variant.name
                ),
            )
            .with_hint(if declared.is_empty() {
                format!("declare it on the enum: `attributes: [{{name: {name}, type: …}}]`")
            } else {
                format!(
                    "declared attributes: {}",
                    declared
                        .iter()
                        .map(|attribute| format!("`{}`", attribute.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }),
        );
    }
}

/// The refusal of the YAML booleans written as variants of the enum at `location`, naming the
/// repair (beyond10x/ess#426).
fn boolean_variants(location: &str, booleans: &[bool]) -> ValidationError {
    let written = booleans
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let quoted = booleans
        .iter()
        .map(|value| if *value { "`'True'`" } else { "`'False'`" })
        .collect::<Vec<_>>()
        .join(", ");
    ValidationError::new(
        ValidationCode::TypeMismatch,
        format!("{location}.variants"),
        format!(
            "{written} {} a YAML boolean, not a variant name: YAML reads an unquoted `True`, \
             `true` or `TRUE` (and `False`, `false`, `FALSE`) as a boolean before the variant \
             list sees it, and which of them was written is not kept",
            if booleans.len() == 1 {
                "is"
            } else {
                "are each"
            }
        ),
    )
    .with_hint(format!(
        "quote it: write each variant as the text it is named, such as {quoted}, in the case the \
         variant is meant to have"
    ))
}

impl RawNamedType {
    /// The refusal of what this type writes that no reading admits — an invariant that does not
    /// parse (beyond10x/ess#448), a YAML boolean written as a variant (beyond10x/ess#426) — each
    /// withheld from the declaration rather than the declaration from the specification, so
    /// nothing declared with the type is refused a second time for it. An invariant's place is held
    /// ([`RawInvariant::withhold_unparsed`]); a boolean is left out of the variants.
    pub(crate) fn withhold_unread(&mut self) -> ValidationErrors {
        let location = declared_at(&self.name);
        match &mut self.body {
            RawTypeBody::Newtype { invariants, .. } | RawTypeBody::Struct { invariants, .. } => {
                RawInvariant::withhold_unparsed(invariants, |index| {
                    PredicateAt::from(format!("{location}.invariants[{index}]"))
                })
            }
            RawTypeBody::Enum {
                variants,
                attributes,
            } => {
                // What each variant's attributes leave out or add (ess/23, beyond10x/ess#450),
                // refused here so the enum stays declared and nothing that names it is refused a
                // second time; an added value is withheld.
                let mut errors = ValidationErrors::new();
                for variant in variants.iter_mut() {
                    match variant {
                        RawEnumVariant::Declared(declared) => {
                            unfilled(
                                &location,
                                declared,
                                &BTreeMap::new(),
                                attributes,
                                &mut errors,
                            );
                        }
                        RawEnumVariant::Attributed(declared, values) => {
                            unfilled(&location, declared, values, attributes, &mut errors);
                            undeclared(&location, declared, values, attributes, &mut errors);
                        }
                        RawEnumVariant::Boolean(_) => {}
                    }
                }
                let booleans: Vec<bool> = variants
                    .iter()
                    .filter_map(|variant| match variant {
                        RawEnumVariant::Boolean(value) => Some(*value),
                        RawEnumVariant::Declared(_) | RawEnumVariant::Attributed(..) => None,
                    })
                    .collect();
                if !booleans.is_empty() {
                    variants.retain(|variant| !matches!(variant, RawEnumVariant::Boolean(_)));
                    errors.push(boolean_variants(&location, &booleans));
                }
                errors
            }
            RawTypeBody::Union { .. } => ValidationErrors::new(),
        }
    }
}

/// A named type, as parsed.
///
/// Everything a declaration can get wrong on its own is settled while it is read — an unspellable
/// name, an unparsable invariant, a key the model does not know — so the only question left when
/// it is converted is whether the invariants read fields the type has.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RawNamedType {
    /// Its stable identity.
    pub name: QualifiedName,
    /// What it is made of.
    #[serde(flatten)]
    pub body: RawTypeBody,
    /// What it is called on the wire and shown as.
    #[serde(default)]
    pub naming: Naming,
    /// Optional closed clock-reading attachment.
    #[serde(default, deserialize_with = "deserialize_reading")]
    pub reading: Option<crate::reading::ReadingContract>,
}

fn deserialize_reading<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::reading::ReadingContract>, D::Error> {
    <crate::reading::ReadingContract as serde::Deserialize>::deserialize(deserializer).map(Some)
}

impl schemars::JsonSchema for RawNamedType {
    fn schema_name() -> String {
        "NamedType".to_owned()
    }

    // Written by hand for the reason [`Version`](crate::name::Version)'s is: the derived schema
    // describes something no document says. `deny_unknown_fields` on the body renders as
    // `additionalProperties: false` in each branch of its `oneOf`, and a branch cannot see the keys
    // the flattened outer struct supplies — so the derived schema calls every real declaration
    // invalid, `examples/billing/` included. Putting `name` and `naming` where the branch can see
    // them is what lets the published schema refuse `invarants:` exactly where the parser does,
    // rather than refusing everything or nothing.
    fn json_schema(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = match <RawTypeBody as schemars::JsonSchema>::json_schema(generator) {
            schemars::schema::Schema::Object(object) => object,
            boolean @ schemars::schema::Schema::Bool(_) => return boolean,
        };
        let name = described(
            generator.subschema_for::<QualifiedName>(),
            "Its stable identity.",
        );
        let naming = described(
            generator.subschema_for::<Naming>(),
            "What it is called on the wire and shown as.",
        );
        let reading = generator.subschema_for::<crate::reading::ReadingContract>();
        let undeclared_fields = described(
            generator.subschema_for::<UndeclaredFields>(),
            "What a reader does with a field this struct does not declare: `refused`, the \
             default, or `ignored`, which admits fields beyond the declared ones while every \
             declared field stays required and typed (`ess/24`, beyond10x/ess#500).",
        );

        let object = schema.object();
        object.properties.insert("name".to_owned(), name.clone());
        object
            .properties
            .insert("naming".to_owned(), naming.clone());
        object.required.insert("name".to_owned());
        object
            .properties
            .insert("reading".to_owned(), reading.clone());
        for branch in schema.subschemas().one_of.iter_mut().flatten() {
            let schemars::schema::Schema::Object(branch) = branch else {
                continue;
            };
            let object = branch.object();
            object.properties.insert("name".to_owned(), name.clone());
            object
                .properties
                .insert("naming".to_owned(), naming.clone());
            object.required.insert("name".to_owned());
            object
                .properties
                .insert("reading".to_owned(), reading.clone());
            // A struct, the one branch with `fields`, may say what a reader does with a field it
            // does not declare (`ess/24`, beyond10x/ess#500). Read beside the body rather than in
            // it (`crate::undeclared_fields`), so it is added here, to that branch only.
            if object.properties.contains_key("fields") {
                object
                    .properties
                    .insert(UndeclaredFields::KEY.to_owned(), undeclared_fields.clone());
            }
        }

        schema.into()
    }
}

/// A `$ref` with a description beside it — the shape a derived schema gives a documented field.
fn described(schema: schemars::schema::Schema, description: &str) -> schemars::schema::Schema {
    let mut wrapper = schemars::schema::SchemaObject::default();
    wrapper.subschemas().all_of = Some(vec![schema]);
    wrapper.metadata().description = Some(description.to_owned());
    wrapper.into()
}

impl TryFrom<RawNamedType> for NamedType {
    type Error = ValidationErrors;

    fn try_from(raw: RawNamedType) -> Result<Self, Self::Error> {
        let mut errors = ValidationErrors::new();
        let body = raw.body.read(&declared_at(&raw.name), &mut errors);
        let declared = Self {
            name: raw.name,
            body,
            naming: raw.naming,
            reading: raw.reading,
        };
        errors.extend(declared.check_shape());
        errors.extend(declared.check_invariants());
        errors.into_result(declared)
    }
}

/// A conversion someone decided to allow.
///
/// Design §20 requires that a mapping's two types "be compatible, or an explicit conversion must
/// exist". This is that conversion, and it is a declaration rather than an inference on purpose:
/// `Email` and `VerifiedEmail` are both a `String` underneath, and the entire value of naming them
/// apart is that the model refuses to treat one as the other. Someone has to write down that this
/// particular crossing is intended, so that the next reader can find who decided it.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Conversion {
    /// The type a value has.
    pub from: TypeRef,
    /// The type it may be used as.
    pub to: TypeRef,
    /// Why this crossing is allowed.
    ///
    /// Required. A conversion with no reason is the thing this declaration exists to prevent: a
    /// silent widening that someone added to make a build pass.
    pub because: String,
}

/// Every conversion a specification declares.
///
/// Directional: declaring `Email → EmailAddress` does not permit the reverse, because the reverse is
/// usually the unsafe direction and nobody would notice it being granted.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct ConversionRegistry {
    declared: BTreeSet<Conversion>,
}

impl ConversionRegistry {
    /// An empty registry: nothing crosses.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one, reporting a second declaration of the same crossing.
    pub fn insert(&mut self, conversion: Conversion) -> Result<(), ValidationError> {
        if let Some(existing) = self
            .declared
            .iter()
            .find(|candidate| candidate.from == conversion.from && candidate.to == conversion.to)
        {
            return Err(ValidationError::new(
                ValidationCode::DuplicateDeclaration,
                format!("conversions.{} -> {}", conversion.from, conversion.to),
                format!(
                    "this crossing is already declared, because `{}`",
                    existing.because
                ),
            )
            .with_hint(
                "one crossing, one reason; two reasons means the decision was not settled",
            ));
        }
        self.declared.insert(conversion);
        Ok(())
    }

    /// `true` when a value of `source` may be used where `target` is expected.
    ///
    /// Structural compatibility first, then a declared crossing. The order matters only for
    /// diagnostics: a declared conversion between identical types is redundant, not wrong.
    pub fn permits(&self, source: &TypeRef, target: &TypeRef) -> bool {
        is_assignable(source, target)
            || self
                .declared
                .iter()
                .any(|conversion| &conversion.from == source && &conversion.to == target)
    }

    /// Every conversion, in a stable order.
    pub fn iter(&self) -> impl Iterator<Item = &Conversion> {
        self.declared.iter()
    }

    /// How many.
    pub fn len(&self) -> usize {
        self.declared.len()
    }

    /// `true` when nothing is declared.
    pub fn is_empty(&self) -> bool {
        self.declared.is_empty()
    }
}

/// Every named type in a specification, indexed by identity.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct TypeRegistry {
    types: BTreeMap<QualifiedName, NamedType>,
    /// The format of the specification this registry serves, when it serves one.
    ///
    /// Skipped, so the registry stays `serde(transparent)` and serializes as its types alone. Set
    /// once, where every predicate environment's registry is built, so a construct gated on the
    /// format is gated in every position the checker serves
    /// (`docs/design/string-alphabet-and-length.md`, section 3).
    #[serde(skip)]
    format: Option<crate::system::FormatVersion>,
}

impl TypeRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a type, refusing a second declaration of the same name.
    pub fn insert(&mut self, declared: NamedType) -> Result<(), ValidationError> {
        if self.types.contains_key(&declared.name) {
            return Err(ValidationError::new(
                ValidationCode::DuplicateDeclaration,
                format!("types.{}", declared.name),
                format!("`{}` is declared more than once", declared.name),
            ));
        }
        self.types.insert(declared.name.clone(), declared);
        Ok(())
    }

    /// The type with this name.
    pub fn get(&self, name: &QualifiedName) -> Option<&NamedType> {
        self.types.get(name)
    }
    /// The type declared as `name`, to rewrite in place: how an `ess/22` source's invariants are
    /// resolved once the format is known (`docs/design/expression-family-source22.md`, A1).
    pub(crate) fn get_mut(&mut self, name: &QualifiedName) -> Option<&mut NamedType> {
        self.types.get_mut(name)
    }

    /// The fields of the struct `reference` resolves to through `Optional` and newtypes, or `None`
    /// where it resolves to anything else. A nested payload mapping fills exactly these (ess/14).
    pub fn struct_fields<'a>(&'a self, reference: &'a TypeRef) -> Option<&'a [Field]> {
        let mut current = reference;
        // A newtype chain longer than the registry has declarations is a cycle.
        for _ in 0..=self.len() {
            match current {
                TypeRef::Optional(inner) => current = inner,
                TypeRef::Named(name) => match &self.get(name)?.body {
                    TypeBody::Newtype { of, .. } => current = of,
                    TypeBody::Struct { fields, .. } => return Some(fields),
                    TypeBody::Enum { .. } | TypeBody::Union { .. } => return None,
                },
                TypeRef::Primitive(_) | TypeRef::List(_) | TypeRef::Map(..) => return None,
            }
        }
        None
    }

    /// Every type, in name order.
    pub fn iter(&self) -> impl Iterator<Item = &NamedType> {
        self.types.values()
    }

    /// How many types are declared.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// `true` when nothing is declared.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// The format of the specification this registry serves, when one was recorded.
    pub fn format(&self) -> Option<crate::system::FormatVersion> {
        self.format
    }

    /// This registry, recorded as serving a specification in `format`.
    #[must_use]
    pub fn with_format(mut self, format: crate::system::FormatVersion) -> Self {
        self.format = Some(format);
        self
    }

    /// The newtypes `reference` is declared through, outermost first, and what they end at.
    ///
    /// Walks `Optional` and every named newtype, as the predicate checker does; stops at the first
    /// type that is neither. Bounded by [`MAX_TYPE_DEPTH`], so a newtype that wraps itself ends the
    /// walk rather than the stack.
    pub fn newtype_layers<'a>(&'a self, reference: &'a TypeRef) -> NewtypeLayers<'a> {
        let mut layers = NewtypeLayers {
            newtypes: Vec::new(),
            terminal: reference.clone(),
            optional: false,
        };
        let mut current = reference;
        for _ in 0..=MAX_TYPE_DEPTH {
            match current {
                TypeRef::Optional(of) => {
                    layers.optional = true;
                    current = of;
                }
                TypeRef::Named(name) => match self.get(name) {
                    Some(declared) => match &declared.body {
                        TypeBody::Newtype { of, .. } => {
                            layers.newtypes.push(declared);
                            current = of;
                        }
                        _ => break,
                    },
                    None => break,
                },
                _ => break,
            }
        }
        layers.terminal = current.clone();
        layers
    }

    /// The characters every value of `reference` is drawn from: the outermost declared alphabet's
    /// characters that every inner one also holds, in the outer one's order. `None` when no layer
    /// declares one; a layer that declares none imposes nothing.
    pub fn effective_alphabet(&self, reference: &TypeRef) -> Option<String> {
        effective_alphabet(
            self.newtype_layers(reference)
                .newtypes
                .iter()
                .filter_map(|declared| match &declared.body {
                    TypeBody::Newtype { alphabet, .. } => alphabet.as_deref(),
                    _ => None,
                }),
        )
    }

    /// The text every value of `reference` starts with: the longest prefix any layer of its
    /// newtype chain declares. `None` when no layer declares one. Layers whose prefixes do not
    /// extend one another are refused by [`NamedType::validate_prefix`]; the longest is still what
    /// is returned for them.
    pub fn effective_prefix(&self, reference: &TypeRef) -> Option<String> {
        self.newtype_layers(reference)
            .newtypes
            .iter()
            .filter_map(|declared| match &declared.body {
                TypeBody::Newtype { prefix, .. } => prefix.as_deref(),
                _ => None,
            })
            .max_by_key(|prefix| prefix.chars().count())
            .map(str::to_owned)
    }

    /// Recheck all named-type predicates, and every declared alphabet and prefix, against the
    /// complete registry.
    pub(crate) fn validate_invariants(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        for declared in self.iter() {
            errors.extend(declared.validate_alphabet(self));
            errors.extend(declared.validate_prefix(self));
            errors.extend(declared.validate_attributes(self));
            errors.extend(declared.validate_invariants(self));
            // A struct's `undeclared_fields:`, below `ess/24` (beyond10x/ess#500).
            if let TypeBody::Struct {
                undeclared_fields: Some(_),
                ..
            } = &declared.body
            {
                if let Some(refused) = crate::undeclared_fields::below_format(
                    self.format(),
                    ess_primitives::error::ConstructKind::Type,
                    &declared.name.to_string(),
                ) {
                    errors.push(refused);
                }
            }
        }
        errors
    }

    /// [`Self::resolve`], about a construct rather than about a string.
    ///
    /// The refusal a caller gets from here carries the caller's typed site, so its family and its
    /// cited line do not depend on how the caller spelled the path. `resolve` stays for the callers
    /// whose own family has not been migrated — the inventory in
    /// `docs/design/review-typed-diagnostics.md` lists them.
    pub fn resolve_at(&self, reference: &TypeRef, at: &ConstructRef) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let rendered = at.render();
        crate::primitive_admission::reference(reference, None, &rendered, &mut errors);
        for name in reference.named_dependencies() {
            if !self.types.contains_key(name) {
                errors.push(
                    ValidationError::at(
                        at.clone(),
                        ValidationCode::UndeclaredReference,
                        format!("`{name}` is not a declared type"),
                    )
                    .with_hint(format!(
                        "declared types: {}",
                        self.types
                            .keys()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                );
            }
        }
        errors
    }

    /// Checks that every named type a reference mentions exists.
    pub fn resolve(&self, reference: &TypeRef, location: &str) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        crate::primitive_admission::reference(reference, None, location, &mut errors);
        for name in reference.named_dependencies() {
            if !self.types.contains_key(name) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        location.to_owned(),
                        format!("`{name}` is not a declared type"),
                    )
                    .with_hint(format!(
                        "declared types: {}",
                        self.types
                            .keys()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                );
            }
        }
        errors
    }
}

/// What [`TypeRegistry::newtype_layers`] walked through.
#[derive(Debug, Clone)]
pub struct NewtypeLayers<'a> {
    /// The named newtypes, outermost first.
    pub newtypes: Vec<&'a NamedType>,
    /// The first type that is neither a newtype nor an `Optional`.
    pub terminal: TypeRef,
    /// Whether an `Optional` was passed on the way.
    pub optional: bool,
}

/// The intersection of nested alphabets, written outermost first: the first one's characters that
/// every later one also holds, in the first one's order.
///
/// One function, so the conformance witness and the domain's own checks agree on what a chain of
/// newtypes admits.
pub fn effective_alphabet<'a>(alphabets: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut alphabets = alphabets.into_iter();
    let outer = alphabets.next()?;
    let inner: Vec<&str> = alphabets.collect();
    Some(
        outer
            .chars()
            .filter(|character| inner.iter().all(|held| held.contains(*character)))
            .collect(),
    )
}

impl Primitive {
    /// The fact value `value` is as this primitive, or `None` when it is not a value of it.
    ///
    /// The one table for "is this node a value of this primitive": `ess-conformance` asks it of
    /// every candidate, setup value and payload, and validation asks it of an authored `example:`
    /// (`docs/design/string-alphabet-and-length.md`, section 2). A second copy would be a second
    /// opinion about whether `1.5` is an `Integer`.
    pub fn admits(
        self,
        value: &ess_primitives::node::Node,
    ) -> Option<ess_primitives::facts::FactValue> {
        use ess_primitives::facts::{is_canonical_uuid, is_padded_base64, FactValue};
        use ess_primitives::node::Node;
        match (self, value) {
            (Self::Boolean, Node::Bool(flag)) => Some(FactValue::Bool(*flag)),
            (Self::Decimal, Node::Number(number)) => Some(FactValue::Number(*number)),
            // An integer that is not integral is refused rather than rounded: a candidate binding
            // `1.5` to an `Integer` would decide `quantity == 1` differently from the system it is
            // testing.
            (Self::Integer, Node::Number(number)) if number.is_integral() => {
                Some(FactValue::Number(*number))
            }
            // A grammar, not merely a shape (review finding F08): the two constrained primitives
            // ask the one grammar `ess-primitives` holds, which the Go runtime and the browser
            // adapter ask in their own words against the same corpus.
            (Self::Uuid, Node::Text(text)) if is_canonical_uuid(text) => {
                Some(FactValue::text(text))
            }
            (Self::Bytes, Node::Text(text)) if is_padded_base64(text) => {
                Some(FactValue::text(text))
            }
            (Self::String | Self::Timestamp | Self::Duration, Node::Text(text)) => {
                Some(FactValue::text(text))
            }
            _ => None,
        }
    }
}

/// `true` when a value of `source` can be used where `target` is expected.
///
/// Deliberately strict: identical, or an optional target accepting a required source. Nothing else,
/// because the value of naming `Email` separately from `String` is entirely in the conversions this
/// refuses.
///
/// A free function rather than a method on [`TypeRegistry`], because it never consults one: two
/// named types are assignable exactly when they are the same name, so there is nothing to resolve.
/// Taking a registry it ignores would suggest the answer could depend on what is declared.
pub fn is_assignable(source: &TypeRef, target: &TypeRef) -> bool {
    if source == target {
        return true;
    }
    if let TypeRef::Optional(inner) = target {
        return is_assignable(source, inner);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(value: &str) -> QualifiedName {
        QualifiedName::new(value).expect("a valid name")
    }

    fn declared(yaml: &str) -> Result<NamedType, ValidationErrors> {
        let raw: RawNamedType = serde_yaml::from_str(yaml).expect("the document is well formed");
        NamedType::try_from(raw)
    }

    fn registry() -> TypeRegistry {
        let mut registry = TypeRegistry::new();
        registry
            .insert(NamedType {
                reading: None,
                name: name("billing.Email"),
                body: TypeBody::Newtype {
                    alphabet: None,
                    prefix: None,
                    of: TypeRef::Primitive(Primitive::String),
                    invariants: Vec::new(),
                },
                naming: Naming::default(),
            })
            .expect("new");
        registry
            .insert(NamedType {
                reading: None,
                name: name("billing.Money"),
                body: TypeBody::Struct {
                    fields: vec![
                        Field::new("amount", TypeRef::Primitive(Primitive::Decimal)),
                        Field::new("currency", TypeRef::Primitive(Primitive::String)),
                    ],
                    invariants: vec![Invariant::parse("amount >= 0").expect("a predicate")],
                    undeclared_fields: None,
                },
                naming: Naming::default(),
            })
            .expect("new");
        registry
    }

    #[test]
    fn type_references_parse_and_round_trip() {
        for spelling in [
            "Email",
            "String",
            "Optional<billing.CustomerId>",
            "List<billing.Money>",
            "Map<String, billing.Money>",
        ] {
            let parsed = TypeRef::parse(spelling).expect("parses");
            assert_eq!(parsed.to_string(), spelling, "round trip");
        }
    }

    #[test]
    fn the_deepest_type_a_real_specification_writes_is_still_accepted() {
        // The failure mode of a depth bound is refusing a good document, so the bound is asserted
        // from the accepting side first: this is the deepest thing anybody writes, and it is five.
        let spelling = "Optional<List<Map<String, Optional<billing.Money>>>>";
        let parsed = TypeRef::parse(spelling).expect("a real specification's deepest type");
        assert_eq!(parsed.to_string(), spelling, "round trip");

        // And the whole budget, exactly, one wrapper short of the refusal.
        let at_limit = format!(
            "{}String{}",
            "Optional<".repeat(MAX_TYPE_DEPTH),
            ">".repeat(MAX_TYPE_DEPTH)
        );
        assert!(
            TypeRef::parse(&at_limit).is_ok(),
            "{MAX_TYPE_DEPTH} wrappers is the limit, not one past it"
        );
    }

    #[test]
    fn a_type_nested_past_the_limit_is_refused_rather_than_overflowing_the_stack() {
        // 10 000 wrappers overflowed an 8 MiB stack before this bound existed; the point of the
        // test is that the answer is a refusal a caller can read, not an abort.
        let deep = format!("{}String{}", "Optional<".repeat(10_000), ">".repeat(10_000));
        let error = TypeRef::parse(&deep).expect_err("nesting past the limit");
        assert!(
            matches!(
                error,
                ParseError::TooDeep {
                    kind: "type",
                    limit: MAX_TYPE_DEPTH,
                    ..
                }
            ),
            "the refusal names the construct and the limit: {error:?}"
        );
    }

    #[test]
    fn a_refused_type_is_not_built_so_dropping_it_cannot_overflow_either() {
        // `TypeRef` is `Box`-recursive, so a chain deep enough to overflow the parser would also
        // overflow its own `Drop`, and refusing late would not save the stack. The bound is checked
        // on the way down: 10 000 wrappers refuse without ever allocating the tenth box.
        let deep = format!("{}String{}", "Optional<".repeat(10_000), ">".repeat(10_000));
        let refused = TypeRef::parse(&deep);
        assert!(refused.is_err());
        drop(refused);

        // The deepest value the parser will build, dropped explicitly.
        let accepted = TypeRef::parse(&format!(
            "{}String{}",
            "Optional<".repeat(MAX_TYPE_DEPTH),
            ">".repeat(MAX_TYPE_DEPTH)
        ))
        .expect("at the limit");
        drop(accepted);
    }

    #[test]
    fn a_structured_map_key_is_refused() {
        let error = TypeRef::parse("Map<billing.Money, String>").expect_err("structured key");
        assert!(
            error.to_string().contains("no stable wire form"),
            "the refusal must say why: {error}"
        );
    }

    #[test]
    fn an_unknown_generic_is_refused_rather_than_read_as_a_name() {
        let error = TypeRef::parse("Set<Email>").expect_err("unknown generic");
        assert!(error.to_string().contains("Optional<T>"), "{error}");
    }

    #[test]
    fn a_named_type_is_not_its_representation() {
        let email = TypeRef::Named(name("billing.Email"));
        let text = TypeRef::Primitive(Primitive::String);

        assert!(!is_assignable(&email, &text));
        assert!(
            !is_assignable(&text, &email),
            "the entire value of naming `Email` is in this refusal"
        );
        assert!(is_assignable(&email, &email));
    }

    #[test]
    fn a_required_value_satisfies_an_optional_target_and_not_the_reverse() {
        let email = TypeRef::Named(name("billing.Email"));
        let maybe_email = TypeRef::Optional(Box::new(email.clone()));

        assert!(is_assignable(&email, &maybe_email));
        assert!(
            !is_assignable(&maybe_email, &email),
            "a value that may be absent cannot fill a slot that must be present"
        );
        assert!(maybe_email.is_optional());
        assert_eq!(maybe_email.required(), &email);
    }

    #[test]
    fn an_unresolved_type_is_reported_with_what_is_available() {
        let registry = registry();
        let errors = registry.resolve(
            &TypeRef::parse("List<billing.Invoice>").expect("parses"),
            "command.CreateInvoice.input",
        );
        assert_eq!(errors.len(), 1);
        let rendered = errors.to_string();
        assert!(rendered.contains("billing.Invoice"), "{rendered}");
        assert!(
            rendered.contains("billing.Email"),
            "and what was available: {rendered}"
        );
    }

    #[test]
    fn a_type_declared_twice_is_refused() {
        let mut registry = registry();
        let error = registry
            .insert(NamedType {
                reading: None,
                name: name("billing.Email"),
                body: TypeBody::Enum {
                    variants: EnumVariant::bare(["Work", "Personal"]),
                },
                naming: Naming::default(),
            })
            .expect_err("already declared");
        assert!(error.to_string().contains("more than once"), "{error}");
    }

    #[test]
    fn dependencies_are_reported_through_composites() {
        let declared = NamedType {
            reading: None,
            name: name("billing.Basket"),
            body: TypeBody::Struct {
                fields: vec![
                    Field::new(
                        "lines",
                        TypeRef::parse("List<billing.Money>").expect("parses"),
                    ),
                    Field::new(
                        "owner",
                        TypeRef::parse("Optional<billing.Email>").expect("parses"),
                    ),
                ],
                invariants: Vec::new(),
                undeclared_fields: None,
            },
            naming: Naming::default(),
        };
        let dependencies: Vec<String> = declared
            .dependencies()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(dependencies, vec!["billing.Money", "billing.Email"]);
    }

    #[test]
    fn a_type_declaration_reads_as_one_object() {
        let money = declared(
            "name: billing.invoice.Money\nkind: struct\nfields:\n  - name: amount\n    type: Decimal\ninvariants: [amount >= 0]\n",
        )
        .expect("the flattened form parses");
        assert_eq!(money.name.to_string(), "billing.invoice.Money");
        let TypeBody::Struct {
            fields, invariants, ..
        } = &money.body
        else {
            panic!("expected a struct");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(
            invariants[0].statement, "amount >= 0",
            "the author's own spelling is kept"
        );

        let email = declared("name: billing.invoice.Email\nkind: newtype\nof: String\n")
            .expect("the newtype form parses");
        assert!(matches!(email.body, TypeBody::Newtype { .. }));
    }

    #[test]
    fn a_type_invariant_that_is_not_a_predicate_is_refused() {
        // Refused by the type's own pass at the invariant's index, not by the reader, so it does
        // not end the document (beyond10x/ess#448).
        let errors = declared(
            "name: billing.invoice.Money\nkind: struct\nfields:\n  - name: amount\n    type: Decimal\ninvariants: [\"))) this is not a predicate\"]\n",
        )
        .expect_err("a value object's invariants are predicates, exactly like an entity's");
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::UnparsablePredicate)
            .unwrap_or_else(|| panic!("an unparsable invariant is refused as one: {errors}"));
        assert_eq!(error.location, "types.billing.invoice.Money.invariants[0]");
        assert!(
            error.message.contains("a predicate is either a comparison"),
            "the refusal says what an invariant is: {error}"
        );
    }

    #[test]
    fn a_type_invariant_that_reads_a_field_the_type_does_not_have_is_refused() {
        let errors = declared(
            "name: billing.invoice.Money\nkind: struct\nfields:\n  - name: amount\n    type: Decimal\ninvariants: [nonexistent_field >= 0]\n",
        )
        .expect_err("`nonexistent_field` is not a field of `Money`");
        assert_eq!(errors.len(), 1, "{errors}");
        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::UnobservableFact);
        assert_eq!(error.location, "types.billing.invoice.Money.invariants[0]");
        assert!(
            error
                .message
                .contains("`nonexistent_field` is not a field of"),
            "{error}"
        );
        assert!(
            error
                .hint
                .as_deref()
                .unwrap_or_default()
                .contains("readable here: amount"),
            "the hint lists what an invariant may name: {error}"
        );
    }

    #[test]
    fn a_newtype_invariant_reads_the_value_it_wraps() {
        declared("name: billing.Positive\nkind: newtype\nof: Decimal\ninvariants: [value > 0]\n")
            .expect("`value` is how an invariant names what a newtype wraps");

        let errors = declared(
            "name: billing.Positive\nkind: newtype\nof: Decimal\ninvariants: [amount > 0]\n",
        )
        .expect_err("a newtype has no field `amount`");
        assert!(
            errors.contains(ValidationCode::UnobservableFact),
            "{errors}"
        );
        assert!(
            errors.to_string().contains("the representation it wraps"),
            "the hint says what a newtype's invariant may read: {errors}"
        );
    }

    #[test]
    fn a_key_the_model_does_not_know_is_refused_in_a_type_declaration() {
        for (misspelling, dropped) in [
            ("invarants: [value >= 0]", "invarants"),
            ("namin:\n  wire: money", "namin"),
        ] {
            let error = serde_yaml::from_str::<RawNamedType>(&format!(
                "name: billing.Money\nkind: newtype\nof: Decimal\n{misspelling}\n"
            ))
            .expect_err("a misspelt key used to be dropped in silence");
            assert!(
                error.to_string().contains(dropped),
                "the refusal names the key: {error}"
            );
        }

        let money =
            declared("name: billing.Money\nkind: newtype\nof: Decimal\ninvariants: [value >= 0]\n")
                .expect("the spelling the model does know");
        assert!(
            matches!(&money.body, TypeBody::Newtype { invariants, .. } if invariants.len() == 1)
        );
    }

    #[test]
    fn the_published_schema_accepts_what_the_parser_accepts() {
        // The schema is loaded by an author's editor. Refusing a declaration this repository ships
        // as valid is the one thing it must never do, and `deny_unknown_fields` on the body is what
        // makes a derived one do exactly that.
        let schema = serde_json::to_value(schemars::schema_for!(RawNamedType)).expect("serialises");
        let branches = schema["oneOf"].as_array().expect("one branch per kind");
        assert_eq!(branches.len(), 4);
        for branch in branches {
            assert_eq!(
                branch["additionalProperties"],
                serde_json::json!(false),
                "a misspelt key has to be a red squiggle: {branch}"
            );
            for key in ["name", "naming"] {
                assert!(
                    branch["properties"][key].is_object(),
                    "the flattened outer keys have to be visible where that is enforced, or the \
                     schema refuses every real declaration: {branch}"
                );
            }
        }
    }

    /// Reads a type declaration the way a document does.
    fn declaration(yaml: &str) -> Result<NamedType, ValidationErrors> {
        NamedType::try_from(serde_yaml::from_str::<RawNamedType>(yaml).expect("well formed"))
    }

    #[test]
    fn a_type_no_value_can_be_is_refused() {
        for (label, yaml) in [
            (
                "an enum",
                "name: billing.Channel\nkind: enum\nvariants: []\n",
            ),
            (
                "a union",
                "name: billing.Payee\nkind: union\ntag: kind\nvariants: {}\n",
            ),
            (
                "a struct",
                "name: billing.Money\nkind: struct\nfields: []\n",
            ),
        ] {
            let errors = declaration(yaml).expect_err(label);
            assert!(
                errors.contains(ValidationCode::EmptyDeclaration),
                "{label}: {errors}"
            );
        }
    }

    #[test]
    fn a_union_that_names_no_tag_field_is_refused() {
        // An untagged union is decodable only by guessing which branch a payload is, and the guess
        // is wrong at run time rather than at build time.
        let errors = declaration(
            "name: billing.Payee\nkind: union\ntag: \"\"\nvariants: {person: String}\n",
        )
        .expect_err("a union with no tag");
        assert!(
            errors.contains(ValidationCode::EmptyDeclaration),
            "{errors}"
        );
    }

    #[test]
    fn a_type_that_declares_something_is_accepted() {
        // The other side of the rule: the check must not refuse the shapes the example uses.
        for yaml in [
            "name: billing.Channel\nkind: enum\nvariants: [Email, Post]\n",
            "name: billing.Payee\nkind: union\ntag: kind\nvariants: {person: String}\n",
            "name: billing.Money\nkind: struct\nfields: [{name: amount, type: Decimal}]\n",
            "name: billing.Email\nkind: newtype\nof: String\n",
        ] {
            declaration(yaml).unwrap_or_else(|errors| panic!("{yaml} is valid: {errors}"));
        }
    }

    #[test]
    fn a_declaration_reaches_the_model_only_through_the_conversion() {
        // The two-stage rule, checked rather than trusted: `NamedType` has no `Deserialize`, so a
        // document cannot become one without `TryFrom` running every rule on it.
        let raw = serde_yaml::from_str::<RawNamedType>(
            "name: billing.Money\nkind: newtype\nof: Decimal\ninvariants: [amount > 0]\n",
        )
        .expect("well formed");
        let errors = NamedType::try_from(raw).expect_err("a newtype has no field `amount`");
        assert!(
            errors.contains(ValidationCode::UnobservableFact),
            "{errors}"
        );
    }

    #[test]
    fn a_field_name_must_be_spellable_as_an_identifier() {
        for spelling in ["", "not a field name!", "1st", "total-amount"] {
            let error =
                serde_yaml::from_str::<Field>(&format!("name: {spelling:?}\ntype: Decimal\n"))
                    .expect_err(spelling);
            assert!(
                error.to_string().contains("field name"),
                "{spelling:?}: {error}"
            );
        }
        serde_yaml::from_str::<Field>("name: invoice_id\ntype: Decimal\n")
            .expect("`invoice_id` is a field name");

        let schema = serde_json::to_value(schemars::schema_for!(Field)).expect("serialises");
        assert_eq!(
            schema["properties"]["name"]["pattern"],
            serde_json::json!(Field::PATTERN),
            "a schema that accepts what the parser refuses is worse than no schema"
        );
    }

    #[test]
    fn a_union_carries_its_tag() {
        let declared = NamedType {
            reading: None,
            name: name("billing.PaymentMethod"),
            body: TypeBody::Union {
                tag: "method".to_owned(),
                variants: [
                    (
                        "card".to_owned(),
                        Some(TypeRef::Named(name("billing.Card"))),
                    ),
                    (
                        "transfer".to_owned(),
                        Some(TypeRef::Named(name("billing.Transfer"))),
                    ),
                ]
                .into(),
            },
            naming: Naming::default(),
        };
        let TypeBody::Union { tag, variants } = &declared.body else {
            panic!("expected a union");
        };
        assert_eq!(tag, "method");
        assert_eq!(variants.len(), 2);
        assert_eq!(declared.dependencies().len(), 2);
    }
}
