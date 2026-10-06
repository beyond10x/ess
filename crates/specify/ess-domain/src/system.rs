//! The whole specification: one system, its domains, and every type in it.
//!
//! A [`SystemSpec`] is what the rest of the toolchain compiles, generates from and checks against.
//! It carries three things a domain cannot know on its own:
//!
//! | | why it lives here |
//! |---|---|
//! | [`FormatVersion`] | the language the document is written in, refused rather than guessed |
//! | [`TypeRegistry`] | every type from every domain, so a reference resolves without knowing who declared it |
//! | the domain list | so [`SystemSpec::owner_of`] can answer who owns a name — the question every later validation asks |
//!
//! # A specification is not a file
//!
//! Design §24 asks for large specifications to be decomposable:
//!
//! ```text
//! ess/
//! ├── system.yaml          <- the header: system, version, format
//! └── domains/
//!     ├── invoice.yaml
//!     └── email.yaml
//! ```
//!
//! So the unit this module models is the [`SpecPart`] — one source's contribution — and a
//! [`Manifest`] of parts that [`SystemSpec::merge`] compiles into one graph. Every declaration
//! remembers which [`Source`] it came from, so a name declared twice is refused with both files
//! named rather than with one silently winning.
//!
//! No file is opened here. A [`Source`] is a label, not a path to read: reading a directory belongs
//! to the compiler crate, and keeping it out means this crate stays clock-free, IO-free and
//! testable from string literals.
//!
//! # The version that matters
//!
//! Design §23 versions several things. Two of them are here: the **format** (`ess/1`) and the
//! **system** (`billing/v3`). A format this build does not implement is refused with an instruction
//! to upgrade the tooling, because a later format may mean something different by the same words —
//! the same rule, and the same reasoning, as `ess_primitives::protocol::is_supported_major`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use ess_primitives::error::{ParseError, ValidationCode, ValidationError, ValidationErrors};

#[cfg(test)]
use crate::domain::RawDomainSpec;
use crate::domain::{DomainSpec, MemberKind};
use crate::name::{Naming, QualifiedName, Version};
use crate::types::{NamedType, TypeBody, TypeRef, TypeRegistry};

/// Specification format major versions this build implements.
pub const SUPPORTED_FORMATS: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
];

/// What one specification format admits that the format before it did not.
///
/// One row of [`FORMAT_HISTORY`]. `ess specify formats` prints it, and `cargo xtask
/// format-history` renders the `ess/` table of `website/docs/reference/spec-versions.md` from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatHistoryEntry {
    /// The format's major version, `N` in `ess/N`.
    pub major: u32,
    /// The release that first shipped it, or `None` while no release has.
    pub release: Option<&'static str>,
    /// The constructs it admits, one sentence each. Never empty.
    pub added: &'static [&'static str],
    /// Rules that read a document differently from this format on, one sentence each.
    pub stricter: &'static [&'static str],
}

/// Every format in [`SUPPORTED_FORMATS`], in the same order, with what it added.
///
/// The only record of what each `ess/N` added that ships inside `ess`. A row missing, extra, out of
/// order or with nothing added is a build error (the assertion below).
pub const FORMAT_HISTORY: &[FormatHistoryEntry] = &[
    FormatHistoryEntry {
        major: 1,
        release: Some("0.1.0"),
        added: &["The first format: types, entities, commands, events, errors, views, actors, components, bindings and topology."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 2,
        release: Some("0.20.0"),
        added: &["Finite `Binary64` fields, distinct from integer and decimal values."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 3,
        release: Some("0.23.0"),
        added: &["`when_subject_state`; binding accessors into an event envelope."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 4,
        release: Some("0.23.0"),
        added: &["Error wire names; command response fields mapped into event payloads."],
        stricter: &["Emitted payload ownership is explicit and complete; below `ess/4` an emitted payload keeps its sparse semantics."],
    },
    FormatHistoryEntry {
        major: 5,
        release: Some("0.27.0"),
        added: &["An enum variant's own `wire`, `display`, `summary` and `code`."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 6,
        release: Some("0.28.0"),
        added: &["An input guard beside an external cause; `when_subject` over an enum field; `preserves`."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 7,
        release: Some("0.29.0"),
        added: &["`replays`; an effect-free named error as the default of subject-state branches."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 8,
        release: Some("0.34.0"),
        added: &["The string operators `starts_with`, `ends_with` and `contains`."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 9,
        release: Some("0.34.0"),
        added: &["`when_subject: {predicate: …}` over the subject's stored fields."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 10,
        release: Some("0.34.0"),
        added: &["Aggregate views: `aggregate:` and `group_by:`."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 11,
        release: Some("0.34.0"),
        added: &["`alphabet:`, input `example:`, and `.count` on a `String`."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 12,
        release: Some("0.34.0"),
        added: &["`outcome_groups:`, one refusal declared once for many commands."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 13,
        release: Some("0.35.0"),
        added: &["`fixture_inputs:` on a command."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 14,
        release: Some("0.36.0"),
        added: &["Value expressions in `payload:` and `sets:`."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 15,
        release: Some("0.37.0"),
        added: &["Outcome shapes, `input.` in subject guards, case-insensitive comparison, `prefix:`, `Json`, `presence:`, and aggregates over `Optional` fields."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 16,
        release: Some("0.38.0"),
        added: &["`input_absent:`, `existing_instance:`, actor `attributes:`, view `paging:`, bounded retry, `instances:` and `affects:`."],
        stricter: &["A guard that cannot hold because every way it could hold needs an input that is not `Optional` to be absent (`not defined(f)`, `missing(f)`) is refused as a type mismatch; below `ess/16` it validates as before."],
    },
    FormatHistoryEntry {
        major: 17,
        release: Some("0.39.0"),
        added: &["`returns: true` on an outcome."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 18,
        release: Some("0.41.0"),
        added: &["Several states in `when_subject_state:`, `state` in `when_subject`, `when_related:`, and a binding's delivery context."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 19,
        release: Some("0.46.0"),
        added: &["`payload:` sources for the fields of the error an outcome reports."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 20,
        release: Some("0.49.0"),
        added: &["`state`, the related row's held lifecycle state, in a `when_related:` predicate."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 21,
        release: Some("0.53.0"),
        added: &["`one_time_response:` names required String response fields whose values may be disclosed only by their originating response."],
        stricter: &[],
    },
    FormatHistoryEntry {
        major: 22,
        release: Some("0.53.0"),
        added: &[
            "A `when_related:` guard's `via: input.<field>` may name an `Optional<…>` input, checked only when present; a command may guard on several related rows named by its input, each with its own `exists: false`.",
            "A `{related: …}` value may read through an `Optional<…>` reference, absent where it is, or across two references: `via: [<field>, <field of the row it names>]`.",
            "An `affects:` entry may move the records it selects: `moves: <Entity>.<transition>`, skipping a selected record outside the move's `from` states (beyond10x/ess#229); below `ess/22` it is refused naming `ess/22`.",
            "An event binding may carry `when.where`, a finite condition over the event payload; it invokes only when the condition holds, and an Optional member the condition proves present may fill a required input.",
            "An actor's `may:` may name a view, which only the actors naming it may read.",
            "On the right of a comparison an unquoted word naming a field of the place is that field (`{fact: x}` written back); a plain `when:` reads `input.<field>`; two `Timestamp` fields compare as instants (`as: timestamp`); identity inputs compare only by `==` and `!=`; the right side may be one fact moved by one constant, `upper <= lower + 5` or `expires_at <= issued_at - 24h` (`{offset: {fact, add|subtract}}` written back), compared exactly for an `Integer` and by elapsed seconds for a `Timestamp`; `<text>.utf8_bytes` on either side of a comparison is the UTF-8 byte length of a `String` (`{utf8_bytes: <text>}` written back), refused in every other position and below `ess/22`, while a struct member named `utf8_bytes` keeps its meaning.",
            r#"A union variant may be declared with no payload (`Open:`): a unit variant, written on the wire as its tag alone (`{"kind": "Open"}`) and generated as a unit enum variant; below `ess/22` it is refused naming `ess/22`."#,
            "`on_failure:` may select its policy per refusal of the invoked command: `drop`, `retry` and `escalate` keyed, each with `outcomes:` or `except:`, exactly one `except:` as the explicit fallback.",
            "An aggregate may declare `where:`, the rows of its group one measure reads (beyond10x/ess#363); below `ess/22` it is refused at the `where:` naming `ess/22`.",
            "A `sets:` or `payload:` value may read `input.<path>`, a member of a struct input, absent where an `Optional` on the path is; `{input: <path>, else: …}` falls back where it is, and `else:` may read another input required along its whole route.",
            "`distinct: {in, as, by}` holds when no two elements of a list share a key, the element or one scalar member, compared exactly or as instants (`kind:` written back); below `ess/22` it is refused naming `ess/22`.",
            "A `when_related:` guard may test the rows a selector selects — `{entity, where}` with `exists`, `count` or `forall` — and a value may read one field of the one row a selector selects — `{related: {entity, where, field}}` (beyond10x/ess#228, #299); below `ess/22` the guard is refused naming `ess/22`, and the value keeps its nested-mapping meaning.",
            r#"A command guard — its `when:`, `when_subject:` or `when_related:` predicate — may hold an instant to a calendar window at UTC or a fixed offset, `window: {at: now | <Timestamp field>, days: [mon, …], from: "HH:MM", to: "HH:MM", offset: Z | ±HH:MM}`; a named time zone is refused, so a window does not follow daylight saving."#,
            "An `external:` refusal marked `compensates: true` changes the record its `instance:` names — `moves:` or `updates:` with `sets:` — and then answers its error (beyond10x/ess#197); every unmarked refusal still changes nothing, and below `ess/22` the marker is refused naming `ess/22`.",
        ],
        stricter: &["An outcome declaring `returns: true` is answered `200` with the command's response under `response`, in the `OpenAPI` projection and the synthesized Rust and Go servers; below `ess/22` it keeps `202` and no `response` member."],
    },
    FormatHistoryEntry {
        major: 23,
        release: None,
        added: &[
            "An `updates:` whose `sets:` writes the entity's identity re-keys the record: the row read under `instance:` comes to rest under the written identity, every field `sets:` does not name carried over, and the old identity names nothing (beyond10x/ess#429).",
            "The command must declare its collision answer, a refusal guarded by `when_related: {entity: <the entity>, where: <identity> == input.<field>, exists: true}` over the input the identity is written from, or validate refuses the outcome as `missing_declaration`; the identity write is refused by name beside `compensates:`, in a create-or-update pair, on an entity a declared relation carries and on a struct identity, and below `ess/23` it is refused naming `ess/23`.",
            "A value may read the held lifecycle state, `{subject: state}`, wherever `{subject: …}` is admitted — an error payload, an event payload and `sets:` — the state before the move (beyond10x/ess#458); below `ess/23` it is refused naming `ess/23`.",
            "`deletes:` takes `instances:`, removing every stored row a filter selects and counting them with `{count: changed}`; an `affects:` entry may declare `deletes: <Entity>` over an entity of the outcome's own domain, and `affects:` is admitted beside a `deletes:` subject (beyond10x/ess#452).",
            "An enum may declare typed variant attributes, `attributes: [{name, type}]`, each variant filling them with typed literals; guards, invariants and view filters read `<fact>.<attribute>`, lowered to variant membership (beyond10x/ess#450).",
            "A refusal selected by `when_subject: {predicate: …}` asserts the whole record unchanged, as a state-guarded refusal does, and a predicate on `state` is witnessed in each state it claims that no earlier branch answers (beyond10x/ess#461).",
            "Synthesis scopes a row-set selector by an equality between a `String` or `Uuid` member of a struct identity and the input or the subject, `at.region == input.place.region`, reads each row's literal identity member by member, arranges its decoys under other identities and never creates an identity a row already carries; the whole-struct `at == input.place` stays a type mismatch, and below `ess/23` such a selector validates and is refused by synthesis as unscoped, as before (beyond10x/ess#463).",
            "A row-set selector that compares with the input is decided on a command that also names an existing record through the input, an update, a delete or an upsert; below `ess/23` such a branch stays refused (beyond10x/ess#462).",
            "An `affects:` entry may write one record per element of an input list with `each: {in: input.<list>, as: <name>}` and `instance: <name>.<member>`: it updates the record an element names when it is held and creates it in the initial state when not, admitted only where a declared `distinct:` keeps that member distinct (beyond10x/ess#459).",
        ],
        stricter: &[],
    },
];

// `FORMAT_HISTORY` and `SUPPORTED_FORMATS` name the same majors in the same order, and every row
// says what it added; otherwise the build fails here.
const _: () = {
    assert!(
        FORMAT_HISTORY.len() == SUPPORTED_FORMATS.len(),
        "FORMAT_HISTORY needs one row per SUPPORTED_FORMATS major"
    );
    let mut index = 0;
    while index < FORMAT_HISTORY.len() {
        assert!(
            FORMAT_HISTORY[index].major == SUPPORTED_FORMATS[index],
            "FORMAT_HISTORY rows follow SUPPORTED_FORMATS in order"
        );
        assert!(
            !FORMAT_HISTORY[index].added.is_empty(),
            "every FORMAT_HISTORY row says what it added"
        );
        index += 1;
    }
};

/// `true` when this build implements `format`.
pub fn is_supported_format(format: FormatVersion) -> bool {
    SUPPORTED_FORMATS.contains(&format.major())
}

/// The version of the specification language a document is written in — `ess/1`.
///
/// Only the major part exists, and an unknown one is refused rather than interpreted. A document
/// written against `ess/2` may use the same words for different things, and a reader that guesses
/// produces a specification nobody wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(into = "String")]
pub struct FormatVersion(u32);

impl FormatVersion {
    /// The first, and so far only, format.
    pub const V1: Self = Self(1);
    /// Finite Binary64 model primitives, retaining the first format's other semantics.
    pub const V2: Self = Self(2);
    /// Bounded, typed event accessors in binding mappings.
    pub const V3: Self = Self(3);
    /// Declared error wire names, typed command responses and complete emitted payloads.
    pub const V4: Self = Self(4);
    /// Declared enum variant wire names.
    pub const V5: Self = Self(5);
    /// Input eligibility on externally decided outcomes.
    pub const V6: Self = Self(6);
    /// Retained command results and effect-free finite state refusals.
    pub const V7: Self = Self(7);
    /// String predicate operators: `starts_with`, `ends_with` and `contains`.
    pub const V8: Self = Self(8);
    /// Predicates over the existing subject's declared stored fields (`when_subject: {predicate}`).
    pub const V9: Self = Self(9);
    /// Aggregate views: `group_by:` and a field-level `aggregate:`.
    pub const V10: Self = Self(10);
    /// Declared `String` alphabets, input examples and `.count` on text.
    pub const V11: Self = Self(11);
    /// Outcome groups: one outcome declared once for many commands.
    pub const V12: Self = Self(12);
    /// Typed conformance inputs resolved from independently provisioned fixtures.
    pub const V13: Self = Self(13);
    /// Value expressions: `{subject}`, `{increment}`, `{input, else}`, nested struct sources,
    /// `input.` operands in subject predicates and case-insensitive text operators.
    pub const V14: Self = Self(14);
    /// Outcome shapes, subject guards over input, case-insensitive text, new value types and
    /// aggregates over optional fields (retrofit wave 2, beyond10x/ess#138-#157).
    pub const V15: Self = Self(15);
    /// Constructs of the retrofit findings round 3 (beyond10x/ess#162-#179): value sources,
    /// guard operands, outcome selection and paging collected for one release.
    pub const V16: Self = Self(16);
    /// Outcomes whose observable result is the command's typed direct return.
    pub const V17: Self = Self(17);
    /// Constructs of the 0.41 defect round (beyond10x/ess#195, #201, #204, #211), collected for
    /// one release.
    pub const V18: Self = Self(18);
    /// Constructs of the 0.46 generated-behaviour round: `payload:` sources for the fields of the
    /// error an outcome reports (`story:error-payload-sources`).
    pub const V19: Self = Self(19);
    /// Constructs of the downstream-gaps round: `state`, the related row's held lifecycle state, in
    /// a `when_related` predicate (beyond10x/ess#229).
    pub const V20: Self = Self(20);
    /// Outcome-scoped one-time response disclosure authority.
    pub const V21: Self = Self(21);
    /// Present-related predicate refusals compose after a held-row `wrong_state` refusal; a union
    /// variant may carry no payload (beyond10x/ess#418).
    pub const V22: Self = Self(22);
    /// An `updates:` whose `sets:` writes the identity re-keys the record (beyond10x/ess#429); the
    /// held lifecycle state as a value source, `{subject: state}` (beyond10x/ess#458).
    pub const V23: Self = Self(23);

    /// How a format version is written.
    pub const PREFIX: &'static str = "ess/";

    /// The pattern published in generated JSON Schema.
    pub const PATTERN: &'static str = "^ess/[1-9][0-9]*$";

    /// Builds a format version. Zero is rejected: it invites "unversioned".
    pub fn new(major: u32) -> Result<Self, ParseError> {
        if major == 0 {
            return Err(ParseError::reference(
                "specification format",
                "ess/0",
                "format versions start at 1",
            ));
        }
        Ok(Self(major))
    }

    /// The numeric part.
    pub const fn major(self) -> u32 {
        self.0
    }

    /// `true` when this build implements it.
    pub fn is_supported(self) -> bool {
        is_supported_format(self)
    }

    /// Parses `ess/1`.
    ///
    /// The shape is checked here; whether the version is one this build implements is a validation
    /// question, so that a document from the future is refused with a code and a location rather
    /// than with a parse error nobody can match on.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        let digits = value.strip_prefix(Self::PREFIX).ok_or_else(|| {
            ParseError::reference(
                "specification format",
                value,
                "a format is written `ess/1`; this is not an ESS document",
            )
        })?;
        // `u32::from_str` would accept `ess/01` and `ess/+1`, which the published pattern
        // `^ess/[1-9][0-9]*$` rejects — a parser looser than its own schema means a document the
        // editor calls invalid and the tool accepts.
        let major = crate::name::parse_major(digits).ok_or_else(|| {
            ParseError::reference(
                "specification format",
                value,
                "expected a whole number after `ess/`, without a leading zero",
            )
        })?;
        Self::new(major)
    }
}

impl fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", Self::PREFIX, self.0)
    }
}

impl FromStr for FormatVersion {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl From<FormatVersion> for String {
    fn from(value: FormatVersion) -> Self {
        value.to_string()
    }
}

impl<'de> serde::Deserialize<'de> for FormatVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for FormatVersion {
    fn schema_name() -> String {
        "FormatVersion".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(Self::PATTERN.to_owned());
        schema.metadata().description =
            Some("The specification language version, such as `ess/1`.".to_owned());
        schema.into()
    }
}

/// Where one part of a specification came from.
///
/// A label, not a handle: this crate never opens it. `system.yaml`, `domains/invoice.yaml`, or
/// `<document>` for a specification that arrived as one string. Its only job is to make a
/// cross-file conflict nameable — "declared in both of these" is a fixable diagnostic, "declared
/// twice" is not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct Source(String);

impl Source {
    /// The label for a specification that arrived as a single document.
    pub const DOCUMENT: &'static str = "<document>";

    /// Labels a part.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The label for a single-document specification.
    pub fn document() -> Self {
        Self::new(Self::DOCUMENT)
    }

    /// The label itself.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Source {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Source {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// The system-level declarations, which exactly one source carries.
///
/// Kept apart from [`SpecPart`] because a `domains/invoice.yaml` has none of them: it contributes
/// domains to a system it does not name.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SpecHeader {
    /// The system's namespace — `billing`.
    pub name: QualifiedName,
    /// The system's own version — `billing/v3`.
    pub version: Version,
    /// The specification language the document is written in.
    pub format: FormatVersion,
    /// What the system is called on the wire, and what a person is shown.
    pub naming: Naming,
    /// What the system is, in one paragraph.
    pub summary: Option<String>,
    /// The ambient command invocations every scenario and every explored sequence runs inside,
    /// in order (ess/15).
    pub preconditions: Vec<Precondition>,
}

/// One ambient command invocation a system runs inside — an open session whose user row exists
/// (ess/15, beyond10x/ess#152, `docs/design/outcome-shapes.md`).
///
/// Synthesis runs the list, in order, before every scenario's arrangement, and a generated explorer
/// runs it before every sequence, so the model it compares against starts from the state the
/// preconditions leave. Each must take the command's declared success branch; one that is refused
/// fails the scenario as setup. An input value a deterministic generator cannot choose comes from
/// the command's `fixture_inputs:` (ess/13), written `{fixture: name}` or left out.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Precondition {
    /// The command invoked.
    pub command: QualifiedName,
    /// The actor it is invoked as; omitted, the actor the specification grants it to.
    #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
    pub actor: Option<QualifiedName>,
    /// Its input by declared field: a literal, or `{fixture: name}`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub input: BTreeMap<String, ess_primitives::node::Node>,
}

/// One source's contribution to a specification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SpecPart {
    /// Where it came from.
    pub source: Source,
    /// The system-level declarations, when this is the source that carries them.
    pub header: Option<SpecHeader>,
    /// The domains it declares. Several sources may contribute to one domain.
    pub domains: Vec<DomainSpec>,
    /// Types that belong to the system rather than to any one domain.
    pub types: Vec<NamedType>,
}

impl SpecPart {
    /// An empty contribution from `source`.
    pub fn new(source: impl Into<Source>) -> Self {
        Self {
            source: source.into(),
            header: None,
            domains: Vec::new(),
            types: Vec::new(),
        }
    }

    /// Makes this the source that declares the system.
    #[must_use]
    pub fn with_header(mut self, header: SpecHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// Adds a domain.
    #[must_use]
    pub fn with_domain(mut self, domain: DomainSpec) -> Self {
        self.domains.push(domain);
        self
    }

    /// Adds a system-level type.
    #[must_use]
    pub fn with_type(mut self, declared: NamedType) -> Self {
        self.types.push(declared);
        self
    }
}

/// Every source that makes up one specification.
///
/// The manifest is what a directory of `.yaml` files becomes once something has read them. It has
/// no opinion about where they were: the compiler crate produces one from a directory, a test
/// produces one from string literals, and both compile the same way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manifest {
    parts: Vec<SpecPart>,
}

impl Manifest {
    /// An empty manifest.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a part.
    pub fn push(&mut self, part: SpecPart) {
        self.parts.push(part);
    }

    /// Adds a part, for builder-style assembly.
    #[must_use]
    pub fn with(mut self, part: SpecPart) -> Self {
        self.push(part);
        self
    }

    /// Every source in the manifest, in the order it was added.
    pub fn sources(&self) -> impl Iterator<Item = &Source> + '_ {
        self.parts.iter().map(|part| &part.source)
    }

    /// The parts.
    pub fn parts(&self) -> &[SpecPart] {
        &self.parts
    }

    /// How many sources contribute.
    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// `true` when nothing contributes.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Compiles every part into one specification.
    pub fn compile(self) -> Result<SystemSpec, ValidationErrors> {
        SystemSpec::merge(self.parts)
    }
}

impl FromIterator<SpecPart> for Manifest {
    fn from_iter<I: IntoIterator<Item = SpecPart>>(parts: I) -> Self {
        Self {
            parts: parts.into_iter().collect(),
        }
    }
}

/// A whole specification: one system, its domains, and every type in it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SystemSpec {
    /// The system's namespace — `billing`. Every domain sits inside it.
    pub name: QualifiedName,
    /// The system's own version — `billing/v3` (design §23).
    pub version: Version,
    /// The specification language it is written in. Always one this build implements: an unknown
    /// format is refused during validation, so a validated specification cannot carry one.
    pub format: FormatVersion,
    /// Its bounded contexts, in declaration order.
    pub domains: Vec<DomainSpec>,
    /// Every type in the system, from every domain, indexed by name.
    ///
    /// Domains keep their own declarations; this registry is the index that lets a reference in one
    /// domain resolve against a type declared in another without knowing where it lives.
    pub types: TypeRegistry,
    /// What the system is called on the wire, and what a person is shown.
    pub naming: Naming,
    /// What the system is, in one paragraph.
    pub summary: Option<String>,
    /// The ambient command invocations every scenario runs inside, in order (ess/15).
    pub preconditions: Vec<Precondition>,
}

impl SystemSpec {
    /// The system at its own version — `billing/v3`.
    pub fn reference(&self) -> String {
        format!("{}/{}", self.name, self.version)
    }

    /// The domain with this name.
    pub fn domain(&self, name: &QualifiedName) -> Option<&DomainSpec> {
        self.domains.iter().find(|domain| &domain.name == name)
    }

    /// The domain that declares `name`.
    ///
    /// The question every later validation asks: a binding checks that the event it listens for is
    /// owned by somebody, a generator asks which module a command belongs in, a diagnostic asks who
    /// to blame. Ownership is by **declaration**, not by namespace containment, so a name that sits
    /// under `billing.invoice` and is declared by nobody has no owner — which is the answer that
    /// makes a dangling reference visible instead of plausible.
    ///
    /// The domain's own name is not one of its members; look that up with [`SystemSpec::domain`].
    pub fn owner_of(&self, name: &QualifiedName) -> Option<&DomainSpec> {
        self.domains.iter().find(|domain| domain.declares(name))
    }

    /// What `name` is declared as, and by which domain.
    pub fn declaration_of(&self, name: &QualifiedName) -> Option<(&DomainSpec, MemberKind)> {
        self.domains
            .iter()
            .find_map(|domain| domain.kind_of(name).map(|kind| (domain, kind)))
    }

    /// Compiles several sources into one specification.
    ///
    /// This is the only way a [`SystemSpec`] is built — a single-document specification is a
    /// manifest of one — so there is exactly one place where duplicate declarations, an unsupported
    /// format and unresolved types are refused.
    ///
    /// Rejections:
    ///
    /// * no source declares the system, or two do;
    /// * a format this build does not implement;
    /// * a name declared by two sources, or by two domains, with both named;
    /// * a domain outside the system's namespace, or equal to it;
    /// * a type reference that resolves to nothing, or a cycle of them.
    ///
    /// Sources may contribute to the same domain — that is what makes `domains/invoice.yaml` and
    /// `domains/invoice-tax.yaml` legal — but not to the same declaration.
    pub fn merge(parts: impl IntoIterator<Item = SpecPart>) -> Result<Self, ValidationErrors> {
        let (system, errors) = Self::merge_reporting(parts);
        match system {
            Some(system) => errors.into_result(system),
            None => Err(errors),
        }
    }

    /// As [`SystemSpec::merge`], but hands back the graph it built as well as what is wrong with it.
    ///
    /// A failed merge still produces a usable system for every failure except a missing header, and
    /// the caller that assembles members needs one: without it, an unsupported format would hide
    /// every broken reference in the specification and the author would fix one problem per run.
    /// `None` therefore means only "no source declares the system" — with no namespace, no domain
    /// list and no type registry, there is nothing left to check anything against.
    pub(crate) fn merge_reporting(
        parts: impl IntoIterator<Item = SpecPart>,
    ) -> (Option<Self>, ValidationErrors) {
        let (system, mut errors) = Self::merge_reporting_with_reference_types(parts, &[]);
        if let Some(system) = &system {
            errors.extend(crate::primitive_admission::system(system));
        }
        (system, errors)
    }

    /// As [`Self::merge_reporting`], with additional types that may satisfy references while a
    /// complete [`crate::spec::Specification`] is assembled.
    ///
    /// Lifecycle state enums are derived from entities, rather than declared in a domain's type
    /// list. A declared struct may still refer to one (for example, a reusable view shape carrying
    /// an entity's `state`). Those derived enums participate in reference and inhabitation checks,
    /// but do not become declared system types; the complete specification owns them.
    pub(crate) fn merge_reporting_with_reference_types(
        parts: impl IntoIterator<Item = SpecPart>,
        reference_types: &[NamedType],
    ) -> (Option<Self>, ValidationErrors) {
        let parts: Vec<SpecPart> = parts.into_iter().collect();
        let mut assembly = Assembly::default();
        let header = assembly.resolve_header(&parts);

        for part in parts {
            assembly.absorb(part);
        }

        let Some(header) = header else {
            return (None, assembly.errors);
        };

        assembly.check_format(&header);
        assembly.check_namespace(&header.name);
        let errors = &mut assembly.errors;
        errors.extend(DomainSpec::validate_all(&assembly.domains));

        let types = build_registry(&assembly.types, &assembly.domains, reference_types, errors);

        let system = Self {
            name: header.name,
            version: header.version,
            format: header.format,
            domains: assembly.domains,
            types,
            naming: header.naming,
            summary: header.summary,
            preconditions: header.preconditions,
        };
        (Some(system), assembly.errors)
    }
}

/// Indexes every type in the system and checks that each one's references resolve.
fn build_registry(
    system_types: &[NamedType],
    domains: &[DomainSpec],
    reference_types: &[NamedType],
    errors: &mut ValidationErrors,
) -> TypeRegistry {
    let mut registry = TypeRegistry::new();
    for declared in system_types
        .iter()
        .chain(domains.iter().flat_map(|domain| domain.types.iter()))
    {
        if let Err(error) = registry.insert(declared.clone()) {
            errors.push(error);
        }
    }

    let mut reference_registry = registry.clone();
    for derived in reference_types {
        // A declaration that collides with a derived type is diagnosed when the complete
        // specification inserts its lifecycle types. Keeping the declared type here avoids
        // reporting the same duplicate twice during assembly.
        let _ = reference_registry.insert(derived.clone());
    }

    for declared in registry.iter() {
        for dependency in declared.dependencies() {
            errors.extend(reference_registry.resolve(
                &TypeRef::Named(dependency.clone()),
                &format!("types.{}", declared.name),
            ));
        }
    }

    check_inhabitation(&reference_registry, errors);

    registry
}

/// Whether a type reference is inhabited, given what is known to be inhabited so far.
///
/// `Optional`, `List` and `Map` are inhabited unconditionally: an absent value and an empty
/// collection are base cases, so a type that reaches itself only through one of them still has
/// values and a generator over it still terminates.
fn reference_inhabited(reference: &TypeRef, inhabited: &BTreeSet<QualifiedName>) -> bool {
    match reference {
        TypeRef::Primitive(_) | TypeRef::Optional(_) | TypeRef::List(_) | TypeRef::Map(_, _) => {
            true
        }
        TypeRef::Named(name) => inhabited.contains(name),
    }
}

/// Whether a declaration is inhabited, given what is known to be inhabited so far.
///
/// The union case is the whole point of this being about inhabitation rather than about cycles: a
/// union needs **one** variant that can be built, not all of them. `Expr = union {leaf: Integer,
/// pair: Pair}` with `Pair = struct {left: Expr, right: Expr}` describes a perfectly ordinary
/// expression tree — every value of it bottoms out in a `leaf` — and a rule that refused every cycle
/// of names would refuse it.
fn body_inhabited(declared: &NamedType, inhabited: &BTreeSet<QualifiedName>) -> bool {
    match &declared.body {
        TypeBody::Newtype { of, .. } => reference_inhabited(of, inhabited),
        // Every field has to be buildable; a struct cannot omit one.
        TypeBody::Struct { fields, .. } => fields
            .iter()
            .all(|field| reference_inhabited(&field.type_ref, inhabited)),
        // A variant is a name, not a reference to another type; an enum with no variants is refused
        // by `NamedType`'s own conversion, so anything reaching here has at least one.
        TypeBody::Enum { .. } => true,
        // One buildable variant is enough, and a unit variant (ess/22) is always buildable.
        TypeBody::Union { variants, .. } => variants.values().any(|variant| {
            variant
                .as_ref()
                .is_none_or(|variant| reference_inhabited(variant, inhabited))
        }),
    }
}

/// What the inhabitation fixpoint concluded about one registry, computed once.
///
/// Two answers, and the reason they travel together is that they are **not** the same set and one
/// of them is a trap. `inhabited` is which names a value can be built for; `refused` is which names
/// [`check_inhabitation`] actually says something about, which is the first set's complement
/// *minus* the declarations another pass already explains.
/// `story:structured-ring-is-refused-by-two-passes` asked the first question where it needed the
/// second, and `review-result:adversary-wave23-unit1-pass-1` measured what that costs: a literal
/// written into `Pick = struct {only: Missing}` was deferred to a refusal that never came and was
/// checked by no pass at all.
///
/// So `inhabited` stays private and [`refuses_declaration`](Self::refuses_declaration) is the whole
/// of what leaves this module. Asking the wrong one of these two is not a mistake a caller can
/// make.
pub(crate) struct Inhabitation {
    /// Every name a value can be built for.
    inhabited: BTreeSet<QualifiedName>,
    /// Every name [`check_inhabitation`] emits `self_reference` for.
    refused: BTreeSet<QualifiedName>,
}

impl Inhabitation {
    /// Run the fixpoint over `registry` and record both answers.
    ///
    /// A least-fixpoint rather than a cycle search: start with nothing known to be inhabited, then
    /// keep marking declarations whose requirements are met until nothing new can be marked.
    /// Whatever is left unmarked cannot be built — which is design §20's forbidden dependency
    /// cycle, stated as the property that actually matters instead of as the shape that usually
    /// causes it.
    ///
    /// Stating it the other way round was a real defect, not a stylistic difference: treating every
    /// union variant as required refused the expression tree above.
    pub(crate) fn of(registry: &TypeRegistry) -> Self {
        let mut inhabited: BTreeSet<QualifiedName> = BTreeSet::new();
        loop {
            let mut grew = false;
            for declared in registry.iter() {
                if !inhabited.contains(&declared.name) && body_inhabited(declared, &inhabited) {
                    inhabited.insert(declared.name.clone());
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        let refused = registry
            .iter()
            .filter(|declared| !inhabited.contains(&declared.name))
            // An undeclared dependency is already reported as an unresolved reference, and a type
            // that is uninhabited only because one of its fields names nothing would be a second
            // message about the same mistake. The one skip, in the one place, so that every reader
            // of `refused` inherits it.
            .filter(|declared| !names_something_undeclared(declared, registry))
            .map(|declared| declared.name.clone())
            .collect();
        Self { inhabited, refused }
    }

    /// Whether [`check_inhabitation`] refuses the declaration `name`.
    ///
    /// Not "can a value of it exist": the two differ exactly on the declarations whose blocker is
    /// an undeclared name, and those get no `self_reference` here.
    ///
    /// **Also not "is a type reference holding `name` somebody else's to refuse".** Named for the
    /// declaration on purpose, because that is the whole of what this answers: a caller reasoning
    /// about a *type reference* — a literal's target, say — has to establish separately that the
    /// reference and the declaration stand or fall together, which they do only while nothing with
    /// a base case sits between them. `Optional<Pick>` is inhabited whatever `Pick` is, and a
    /// caller that read a refusal of `Pick` as a refusal of `Optional<Pick>` fell silent about a
    /// literal nobody else checked — `review-result:adversary-wave23-unit1-pass-2`. The walk in
    /// [`crate::binding`] pairs this with its own `base_cases == 0`; the name is the reminder that
    /// the pairing is the caller's to make.
    pub(crate) fn refuses_declaration(&self, name: &QualifiedName) -> bool {
        self.refused.contains(name)
    }
}

impl TypeRegistry {
    /// Every declaration `check_inhabitation` refuses: the ones no finite value can inhabit.
    ///
    /// The one termination rule, for a reader outside this crate that holds a closed registry of
    /// its own — a conformance fixture contract, say (beyond10x/ess#416). `Optional`, `List` and
    /// `Map` are base cases and a union needs one buildable variant, so a type that reaches itself
    /// only through one of those is absent from this set. Like `Inhabitation::refuses_declaration`
    /// it answers about declarations, and it is silent about one whose only blocker is a name the
    /// registry does not hold.
    #[must_use]
    pub fn without_finite_value(&self) -> BTreeSet<QualifiedName> {
        Inhabitation::of(self).refused
    }
}

/// Reports every type no value can inhabit.
///
/// The set is [`Inhabitation::refused`], and every other pass that stays silent because this one
/// speaks reads the same set rather than re-deriving it — having first established that what it is
/// staying silent about is one of *these* declarations.
fn check_inhabitation(registry: &TypeRegistry, errors: &mut ValidationErrors) {
    let found = Inhabitation::of(registry);

    for declared in registry.iter() {
        if !found.refuses_declaration(&declared.name) {
            continue;
        }
        // Naming what it is waiting for, not just that it is stuck: a fixpoint knows which
        // requirements were never met, and "requires X, which also cannot be built" is the sentence
        // an author can act on. Bare "this cannot exist" leaves them to find the other end.
        let blockers = unmet_requirements(declared, &found.inhabited);
        errors.push(
            ValidationError::new(
                ValidationCode::SelfReference,
                format!("types.{}", declared.name),
                format!(
                    "no value of `{}` can exist: it requires {}, which cannot be built either",
                    declared.name,
                    blockers
                        .iter()
                        .map(|name| format!("`{name}`"))
                        .collect::<Vec<_>>()
                        .join(" and ")
                ),
            )
            .with_hint(
                "break the requirement, put one step behind `Optional` or `List`, which have a \
                 base case, or give a union a variant that does not recurse",
            ),
        );
    }
}

/// The named types a declaration needs and which are themselves uninhabited.
///
/// For a struct, the fields that are stuck; for a union, every variant, because a union is only
/// stuck when all of them are. Sorted and deduplicated so the message is stable.
fn unmet_requirements(
    declared: &NamedType,
    inhabited: &BTreeSet<QualifiedName>,
) -> Vec<QualifiedName> {
    let mut blocked: BTreeSet<QualifiedName> = BTreeSet::new();
    let mut consider = |reference: &TypeRef| {
        if let TypeRef::Named(name) = reference {
            if !inhabited.contains(name) {
                blocked.insert(name.clone());
            }
        }
    };

    match &declared.body {
        TypeBody::Newtype { of, .. } => consider(of),
        TypeBody::Struct { fields, .. } => {
            for field in fields {
                consider(&field.type_ref);
            }
        }
        TypeBody::Enum { .. } => {}
        TypeBody::Union { variants, .. } => {
            for variant in variants.values().flatten() {
                consider(variant);
            }
        }
    }

    blocked.into_iter().collect()
}

/// Whether a declaration references a name the registry does not hold.
///
/// The inner walk recurses once per generic wrapper and does not count: a [`TypeRef`] cannot nest
/// past [`MAX_TYPE_DEPTH`](crate::types::MAX_TYPE_DEPTH), which its parser enforces, and this walk
/// stops at a `Named` leaf rather than following it into the registry.
fn names_something_undeclared(declared: &NamedType, registry: &TypeRegistry) -> bool {
    fn undeclared(reference: &TypeRef, registry: &TypeRegistry) -> bool {
        match reference {
            TypeRef::Named(name) => registry.get(name).is_none(),
            TypeRef::Optional(inner) | TypeRef::List(inner) => undeclared(inner, registry),
            TypeRef::Map(_, value) => undeclared(value, registry),
            TypeRef::Primitive(_) => false,
        }
    }

    match &declared.body {
        TypeBody::Newtype { of, .. } => undeclared(of, registry),
        TypeBody::Struct { fields, .. } => fields
            .iter()
            .any(|field| undeclared(&field.type_ref, registry)),
        TypeBody::Enum { .. } => false,
        TypeBody::Union { variants, .. } => variants
            .values()
            .flatten()
            .any(|variant| undeclared(variant, registry)),
    }
}

/// Who declared a name, and from where.
#[derive(Debug, Clone)]
pub(crate) struct Claim {
    source: Source,
    owner: Option<QualifiedName>,
    kind: MemberKind,
}

impl Claim {
    /// A declaration of a name as `kind`, in `source`, owned by `owner` or by the system itself.
    pub(crate) fn new(source: &Source, owner: Option<&QualifiedName>, kind: MemberKind) -> Self {
        Self {
            source: source.clone(),
            owner: owner.cloned(),
            kind,
        }
    }

    /// The kind this claim declares its name as.
    pub(crate) fn kind(&self) -> MemberKind {
        self.kind
    }

    /// The source that carries the declaration.
    pub(crate) fn source(&self) -> &Source {
        &self.source
    }

    /// The domain that claims the name, or `None` when it is the system's.
    pub(crate) fn owner(&self) -> Option<&QualifiedName> {
        self.owner.as_ref()
    }

    /// How the claimant reads in a message.
    fn owner_label(&self) -> String {
        self.owner.as_ref().map_or_else(
            || "the system itself".to_owned(),
            |owner| format!("`{owner}`"),
        )
    }

    /// The refusal of `second`, a later claim to the `name` this one already holds.
    ///
    /// One sentence for one fault, whichever reporter finds it: [`Assembly::claim`] for parts
    /// merged directly, and `spec.rs`'s `Collected::write` for a specification's documents, which
    /// hands the assembly one copy of each name and refuses the rest itself, converted or not.
    pub(crate) fn refuse(&self, name: &QualifiedName, second: &Self) -> ValidationError {
        let (first, claim) = (self, second);
        let (source, kind) = (&claim.source, claim.kind);
        let location = claim.owner.as_ref().map_or_else(
            || format!("system.{}", kind.field()),
            |owner| format!("domain {}.{}", owner, kind.field()),
        );
        let message = if first.source == claim.source {
            format!(
                "`{name}` is declared twice in `{source}`, as a {} and as a {kind}",
                first.kind
            )
        } else {
            format!(
                "`{name}` is declared in `{}` and in `{source}`",
                first.source
            )
        };
        ValidationError::new(ValidationCode::DuplicateDeclaration, location, message).with_hint(
            format!(
                "claimed by {} and by {}; a name has exactly one owner",
                first.owner_label(),
                claim.owner_label()
            ),
        )
    }
}

/// The state of a merge in progress.
#[derive(Debug, Default)]
struct Assembly {
    domains: Vec<DomainSpec>,
    index: BTreeMap<QualifiedName, usize>,
    claims: BTreeMap<QualifiedName, Claim>,
    types: Vec<NamedType>,
    errors: ValidationErrors,
}

impl Assembly {
    /// Finds the one source that declares the system, refusing zero and refusing two.
    fn resolve_header(&mut self, parts: &[SpecPart]) -> Option<SpecHeader> {
        let mut headers = parts
            .iter()
            .filter_map(|part| part.header.as_ref().map(|header| (&part.source, header)));

        let (first_source, first) = headers.next().or_else(|| {
            self.errors.push(
                // Nothing here references anything: a required declaration is simply absent,
                // which is a different fault from a declaration that declares nothing.
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    "system",
                    "no source declares the system".to_owned(),
                )
                .with_hint(
                    "exactly one source carries `system:`, `version:` and `format:`; the others \
                     contribute domains to it",
                ),
            );
            None
        })?;

        for (source, other) in headers {
            self.errors.push(
                ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    "system",
                    format!(
                        "the system is declared in `{first_source}` as `{}` and in `{source}` as \
                         `{}`",
                        first.name, other.name
                    ),
                )
                .with_hint("exactly one source declares the system"),
            );
        }

        Some(first.clone())
    }

    /// Refuses a format this build does not implement.
    fn check_format(&mut self, header: &SpecHeader) {
        if !header.format.is_supported() {
            self.errors.push(
                ValidationError::new(
                    // A specification format is not a protocol version, and a harness that treats
                    // them as one will go looking for the wrong document to upgrade.
                    ValidationCode::UnsupportedFormatVersion,
                    "system.format",
                    format!(
                        "this build implements specification format(s) {SUPPORTED_FORMATS:?}, not \
                         `{}`",
                        header.format
                    ),
                )
                .with_hint(
                    "upgrade the tooling that reads it rather than reinterpreting the document: a \
                     later format may mean something different by the same words; `ess specify \
                     formats` lists the formats this build implements and what each added",
                ),
            );
        }
    }

    /// Refuses a domain or a system-level type that does not sit inside the system.
    fn check_namespace(&mut self, system: &QualifiedName) {
        for declared in &self.types {
            if !declared.name.is_within(system) {
                // Declared, and declared in the wrong place: a conflict between two well-formed
                // declarations, not a reference with nothing behind it.
                self.errors.push(ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    "system.types",
                    format!(
                        "`{}` is not inside the system `{system}`, so this specification cannot \
                         declare it",
                        declared.name
                    ),
                ));
            }
        }

        for domain in &self.domains {
            if &domain.name == system {
                self.errors.push(
                    ValidationError::new(
                        ValidationCode::SelfReference,
                        format!("domain {}", domain.name),
                        format!(
                            "`{}` is the system itself, not a domain inside it",
                            domain.name
                        ),
                    )
                    .with_hint(format!(
                        "give it a namespace of its own, such as `{system}.core`"
                    )),
                );
            } else if !domain.name.is_within(system) {
                self.errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        format!("domain {}", domain.name),
                        format!("`{}` is not inside the system `{system}`", domain.name),
                    )
                    .with_hint(format!(
                        "a domain of `{system}` is named `{system}.something`"
                    )),
                );
            }
        }
    }

    /// Folds one source's contribution into the graph.
    fn absorb(&mut self, part: SpecPart) {
        let source = part.source;
        for declared in part.types {
            if self.claim(&declared.name, MemberKind::Type, None, &source) {
                self.types.push(declared);
            }
        }
        for domain in part.domains {
            self.absorb_domain(domain, &source);
        }
    }

    /// Folds one domain's declarations into the domain of that name, creating it if this is the
    /// first source to mention it.
    fn absorb_domain(&mut self, domain: DomainSpec, source: &Source) {
        let owner = domain.name.clone();
        let position = if let Some(position) = self.index.get(&owner).copied() {
            let existing = &self.domains[position];
            if !domain.naming.is_empty()
                && !existing.naming.is_empty()
                && existing.naming != domain.naming
            {
                self.errors.push(
                    // Several sources contributing to one domain is the supported case, so nothing
                    // is declared twice here; what cannot hold is two names for the same domain.
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        format!("domain {owner}.naming"),
                        format!("`{owner}` is named differently in `{source}` and elsewhere"),
                    )
                    .with_hint("one source names a domain; the others contribute to it"),
                );
            }
            position
        } else {
            let mut fresh = DomainSpec::new(owner.clone());
            fresh.naming = domain.naming.clone();
            self.domains.push(fresh);
            self.index.insert(owner.clone(), self.domains.len() - 1);
            self.domains.len() - 1
        };

        if self.domains[position].naming.is_empty() {
            self.domains[position].naming = domain.naming;
        }

        for declared in domain.types {
            if self.claim(&declared.name, MemberKind::Type, Some(&owner), source) {
                self.domains[position].types.push(declared);
            }
        }
        self.absorb_names(
            position,
            domain.entities,
            MemberKind::Entity,
            &owner,
            source,
        );
        self.absorb_names(
            position,
            domain.commands,
            MemberKind::Command,
            &owner,
            source,
        );
        self.absorb_names(position, domain.events, MemberKind::Event, &owner, source);
        // Errors before views: the order `spec.rs`'s `Collected` reads them in, so a name written as
        // both is refused against the same first copy whichever copy converts.
        self.absorb_names(position, domain.errors, MemberKind::Error, &owner, source);
        self.absorb_names(position, domain.views, MemberKind::View, &owner, source);
        self.absorb_names(position, domain.actors, MemberKind::Actor, &owner, source);
    }

    /// Adds every name of one kind that is not already claimed.
    fn absorb_names(
        &mut self,
        position: usize,
        names: Vec<QualifiedName>,
        kind: MemberKind,
        owner: &QualifiedName,
        source: &Source,
    ) {
        for name in names {
            if !self.claim(&name, kind, Some(owner), source) {
                continue;
            }
            let domain = &mut self.domains[position];
            match kind {
                MemberKind::Type => unreachable!("types are absorbed as declarations"),
                MemberKind::Entity => domain.entities.push(name),
                MemberKind::Command => domain.commands.push(name),
                MemberKind::Event => domain.events.push(name),
                MemberKind::View => domain.views.push(name),
                MemberKind::Error => domain.errors.push(name),
                MemberKind::Actor => domain.actors.push(name),
            }
        }
    }

    /// Records who declares `name`, refusing a second claim and naming both sources.
    ///
    /// A refused claim is dropped rather than added, so the merged graph never contains a name
    /// twice and the same conflict is not then reported a second time by
    /// [`DomainSpec::validate_all`].
    fn claim(
        &mut self,
        name: &QualifiedName,
        kind: MemberKind,
        owner: Option<&QualifiedName>,
        source: &Source,
    ) -> bool {
        let claim = Claim::new(source, owner, kind);
        let Some(first) = self.claims.get(name) else {
            self.claims.insert(name.clone(), claim);
            return true;
        };
        self.errors.push(first.refuse(name, &claim));
        false
    }
}

/// A system header with its domains inline, for writing a manifest in a test as one string.
///
/// **This is not a document shape the tooling reads.** The specification document is
/// [`RawSpecFile`](crate::spec::RawSpecFile) — the type the JSON Schema is generated from, the CLI
/// parses and `examples/billing/` is written in. The two disagree about what `domains:` means: here
/// it is a list of domain objects carrying member *names*, there it is the header's roster of
/// domain *names*, so `examples/billing/system.yaml` does not parse as one of these. Two public
/// types that look like the same document and are not is a trap, and this one was reachable and
/// read by nothing, so it is compiled out of the shipped crate.
///
/// Unknown fields are refused — the component, interaction and topology layers (design §5, §7, §8)
/// are not modelled yet, and a document carrying them should be told so rather than have them
/// silently dropped.
#[cfg(test)]
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSystemSpec {
    /// The specification language, defaulting to `ess/1`.
    #[serde(default)]
    pub format: Option<FormatVersion>,
    /// The system's namespace. Present in exactly one source.
    #[serde(default, alias = "name")]
    pub system: Option<QualifiedName>,
    /// The system's own version, defaulting to `v1`.
    #[serde(default)]
    pub version: Option<Version>,
    /// The domains this source declares.
    #[serde(default)]
    pub domains: Vec<RawDomainSpec>,
    /// Types belonging to the system rather than to any one domain.
    #[serde(default)]
    pub types: Vec<crate::types::RawNamedType>,
    /// What the system is called on the wire, and what a person is shown.
    #[serde(default)]
    pub naming: Naming,
    /// What the system is, in one paragraph.
    #[serde(default)]
    pub summary: Option<String>,
}

#[cfg(test)]
impl RawSystemSpec {
    /// Turns one parsed document into the contribution it makes.
    ///
    /// The domains are validated here, where the document's own structure is still the subject.
    /// Everything that needs to see the other sources happens in [`SystemSpec::merge`].
    pub(crate) fn into_part(self, source: impl Into<Source>) -> Result<SpecPart, ValidationErrors> {
        let (part, errors) = self.split(source);
        errors.into_result(part)
    }

    /// What this document contributes, and everything wrong with it.
    ///
    /// Separate from [`RawSystemSpec::into_part`] so that a document with one unreadable domain
    /// still reaches [`SystemSpec::merge`]. Otherwise a misplaced command would hide an unsupported
    /// format, and the author would fix one problem only to be told about the next.
    fn split(self, source: impl Into<Source>) -> (SpecPart, ValidationErrors) {
        let source = source.into();
        let mut errors = ValidationErrors::new();

        if self.system.is_none()
            && (self.format.is_some()
                || self.version.is_some()
                || self.summary.is_some()
                || !self.naming.is_empty())
        {
            errors.push(
                // A required declaration is absent; nothing here is a reference.
                ValidationError::new(
                    ValidationCode::MissingDeclaration,
                    format!("{source}.system"),
                    "this source sets system-level fields but does not declare `system:`"
                        .to_owned(),
                )
                .with_hint(
                    "the source that carries the format and version is the one that names the \
                     system",
                ),
            );
        }

        let header = self.system.map(|name| SpecHeader {
            name,
            version: self.version.unwrap_or(Version::V1),
            format: self.format.unwrap_or(FormatVersion::V1),
            naming: self.naming,
            summary: self.summary,
            preconditions: Vec::new(),
        });

        let mut domains = Vec::with_capacity(self.domains.len());
        for raw in self.domains {
            match DomainSpec::try_from(raw) {
                Ok(domain) => domains.push(domain),
                Err(found) => errors.extend(found),
            }
        }

        let mut types = Vec::with_capacity(self.types.len());
        for raw in self.types {
            match NamedType::try_from(raw) {
                Ok(declared) => types.push(declared),
                Err(found) => errors.extend(found),
            }
        }

        (
            SpecPart {
                source,
                header,
                domains,
                types,
            },
            errors,
        )
    }
}

#[cfg(test)]
impl TryFrom<RawSystemSpec> for SystemSpec {
    type Error = ValidationErrors;

    fn try_from(raw: RawSystemSpec) -> Result<Self, Self::Error> {
        let (part, mut errors) = raw.split(Source::document());
        match Self::merge([part]) {
            Ok(system) => errors.into_result(system),
            Err(found) => {
                errors.extend(found);
                Err(errors)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Primitive, TypeBody};

    fn name(value: &str) -> QualifiedName {
        QualifiedName::new(value).expect("a valid name")
    }

    fn system(yaml: &str) -> Result<SystemSpec, ValidationErrors> {
        let raw: RawSystemSpec = serde_yaml::from_str(yaml).expect("document parses");
        SystemSpec::try_from(raw)
    }

    /// Whatever a document was refused for, or nothing when it was accepted.
    fn system_errors(yaml: &str) -> ValidationErrors {
        system(yaml).err().unwrap_or_default()
    }

    fn part(source: &str, yaml: &str) -> SpecPart {
        let raw: RawSystemSpec = serde_yaml::from_str(yaml).expect("document parses");
        raw.into_part(source).expect("the document is well formed")
    }

    fn newtype(qualified: &str, of: Primitive) -> NamedType {
        NamedType {
            reading: None,
            name: name(qualified),
            body: TypeBody::Newtype {
                alphabet: None,
                prefix: None,
                of: TypeRef::Primitive(of),
                invariants: Vec::new(),
            },
            naming: Naming::default(),
        }
    }

    const BILLING: &str = r"
format: ess/1
system: billing
version: v1
summary: Invoicing, and the mail it sends.
domains:
  - domain: billing.invoice
    entities: [Invoice]
    commands: [CreateInvoice]
    events: [InvoiceCreated]
    views: [InvoiceById]
    errors: [InvalidAmount]
  - domain: billing.email
    commands: [SendEmail]
    events: [EmailSent]
";

    #[test]
    fn a_format_version_round_trips_and_nothing_else_is_read_as_one() {
        assert_eq!(FormatVersion::parse("ess/1").expect("parses").major(), 1);
        assert_eq!(FormatVersion::V1.to_string(), "ess/1");
        assert_eq!(
            FormatVersion::parse("ess/7").expect("parses").to_string(),
            "ess/7"
        );

        let wrong_language = FormatVersion::parse("openapi/3").expect_err("not an ESS document");
        assert!(
            wrong_language.to_string().contains("written `ess/1`"),
            "{wrong_language}"
        );
        assert!(
            FormatVersion::parse("ess/0").is_err(),
            "zero invites `unversioned`"
        );
        assert!(
            FormatVersion::parse("1").is_err(),
            "the prefix is part of the spelling"
        );
        assert!(FormatVersion::V1.is_supported());
        assert!(FormatVersion::V2.is_supported());
        assert!(FormatVersion::V3.is_supported());
        assert!(FormatVersion::V4.is_supported());
        assert!(FormatVersion::V5.is_supported());
        assert!(FormatVersion::V6.is_supported());
        assert!(FormatVersion::V7.is_supported());
        assert!(FormatVersion::V8.is_supported());
        assert!(FormatVersion::V9.is_supported());
        assert!(FormatVersion::V10.is_supported());
        assert!(FormatVersion::V11.is_supported());
        assert!(FormatVersion::V12.is_supported());
        assert!(FormatVersion::V13.is_supported());
        assert!(
            FormatVersion::parse("ess/22")
                .expect("ess/22 parses")
                .is_supported(),
            "the coordinated syntax bundle admits ess/22"
        );
        assert!(!FormatVersion::parse("ess/99")
            .expect("parses")
            .is_supported());
    }

    #[test]
    fn a_document_in_a_later_format_is_refused_rather_than_guessed_at() {
        let errors = system(
            r"
format: ess/99
system: billing
",
        )
        .expect_err("a format this build does not implement");

        assert!(
            errors.contains(ValidationCode::UnsupportedFormatVersion),
            "{errors}"
        );
        let error = &errors.as_slice()[0];
        assert_eq!(error.location, "system.format");
        assert!(error.message.contains("ess/99"), "{error}");
        assert!(
            error
                .hint
                .as_deref()
                .expect("a hint")
                .contains("upgrade the tooling"),
            "the remedy is to upgrade, never to reinterpret: {error}"
        );
    }

    #[test]
    fn a_system_is_assembled_from_one_document() {
        let spec = system(BILLING).expect("validates");

        assert_eq!(spec.reference(), "billing/v1");
        assert_eq!(spec.format, FormatVersion::V1);
        assert_eq!(spec.domains.len(), 2);
        assert_eq!(
            spec.summary.as_deref(),
            Some("Invoicing, and the mail it sends.")
        );
        assert_eq!(
            spec.domain(&name("billing.invoice"))
                .expect("declared")
                .commands,
            vec![name("billing.invoice.CreateInvoice")]
        );
        assert!(spec.domain(&name("billing.shipping")).is_none());
    }

    #[test]
    fn owner_of_answers_for_a_name_in_each_domain_and_for_one_that_is_owned_by_nobody() {
        let spec = system(BILLING).expect("validates");

        assert_eq!(
            spec.owner_of(&name("billing.invoice.CreateInvoice"))
                .map(|domain| domain.name.to_string()),
            Some("billing.invoice".to_owned())
        );
        assert_eq!(
            spec.owner_of(&name("billing.email.EmailSent"))
                .map(|domain| domain.name.to_string()),
            Some("billing.email".to_owned())
        );
        assert!(
            spec.owner_of(&name("billing.invoice.CancelInvoice"))
                .is_none(),
            "a name inside a domain's namespace that nobody declared has no owner; that is what \
             makes a dangling reference visible"
        );
        assert!(
            spec.owner_of(&name("billing.invoice")).is_none(),
            "a domain is not one of its own members"
        );
        assert_eq!(
            spec.declaration_of(&name("billing.invoice.InvoiceById"))
                .map(|(_, kind)| kind),
            Some(MemberKind::View)
        );
    }

    #[test]
    fn a_name_declared_in_two_domains_is_refused() {
        // Built rather than parsed: read from a document, `billing.email` claiming a
        // `billing.invoice` name is refused one step earlier, by the namespace rule. That is the
        // point — the two rules together leave no way for one name to have two owners.
        let invoice = DomainSpec {
            events: vec![name("billing.invoice.InvoiceCreated")],
            ..DomainSpec::new(name("billing.invoice"))
        };
        let email = DomainSpec {
            events: vec![name("billing.invoice.InvoiceCreated")],
            ..DomainSpec::new(name("billing.email"))
        };

        let errors = Manifest::new()
            .with(part("system.yaml", "system: billing"))
            .with(SpecPart::new("domains/invoice.yaml").with_domain(invoice))
            .with(SpecPart::new("domains/email.yaml").with_domain(email))
            .compile()
            .expect_err("two owners for one event");

        assert!(
            errors.contains(ValidationCode::DuplicateDeclaration),
            "{errors}"
        );
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::DuplicateDeclaration)
            .expect("the contested claim");
        assert!(
            error.message.contains("billing.invoice.InvoiceCreated"),
            "the refusal names what is contested: {error}"
        );
        let hint = error.hint.as_deref().expect("a hint");
        assert!(
            hint.contains("`billing.invoice`") && hint.contains("`billing.email`"),
            "and both claimants: {error}"
        );
    }

    #[test]
    fn a_member_outside_its_domains_namespace_is_refused() {
        let errors = system(
            r"
system: billing
domains:
  - domain: billing.invoice
    commands: [billing.email.SendEmail]
",
        )
        .expect_err("not this domain's command");

        assert!(
            errors.contains(ValidationCode::ConflictingDeclaration),
            "{errors}"
        );
        assert!(
            errors
                .to_string()
                .contains("is not inside `billing.invoice`"),
            "{errors}"
        );
    }

    #[test]
    fn a_domain_outside_the_system_is_refused() {
        let errors = system(
            r"
system: billing
domains:
  - domain: shipping.parcel
    commands: [DispatchParcel]
",
        )
        .expect_err("not this system's domain");

        assert!(
            errors.contains(ValidationCode::ConflictingDeclaration),
            "{errors}"
        );
        assert!(
            errors
                .to_string()
                .contains("is not inside the system `billing`"),
            "{errors}"
        );
    }

    #[test]
    fn a_domain_that_is_the_system_itself_is_refused() {
        let errors = system(
            r"
system: billing
domains:
  - domain: billing
    commands: [CreateInvoice]
",
        )
        .expect_err("the system is not one of its own domains");

        assert!(errors.contains(ValidationCode::SelfReference), "{errors}");
        assert!(
            errors.to_string().contains("is the system itself"),
            "{errors}"
        );
    }

    #[test]
    fn every_domains_types_land_in_one_registry() {
        let spec = system(
            r"
system: billing
types:
  - name: billing.Money
    kind: newtype
    of: Decimal
domains:
  - domain: billing.invoice
    types:
      - name: InvoiceId
        kind: newtype
        of: Uuid
  - domain: billing.email
    types:
      - name: Address
        kind: newtype
        of: String
",
        )
        .expect("validates");

        assert_eq!(
            spec.types.len(),
            3,
            "system-level and domain-local, in one index"
        );
        assert!(spec.types.get(&name("billing.invoice.InvoiceId")).is_some());
        assert!(spec.types.get(&name("billing.email.Address")).is_some());
        assert!(spec.types.get(&name("billing.Money")).is_some());
        assert_eq!(
            spec.owner_of(&name("billing.Money")),
            None,
            "a system-level type belongs to no domain"
        );
    }

    #[test]
    fn a_type_reference_that_resolves_to_nothing_is_refused() {
        let errors = system(
            r"
system: billing
domains:
  - domain: billing.invoice
    types:
      - name: Line
        kind: struct
        fields:
          - name: amount
            type: billing.Money
",
        )
        .expect_err("`billing.Money` was never declared");

        assert!(
            errors.contains(ValidationCode::UndeclaredReference),
            "{errors}"
        );
        let error = &errors.as_slice()[0];
        assert_eq!(error.location, "types.billing.invoice.Line");
        assert!(error.message.contains("billing.Money"), "{error}");
    }

    #[test]
    fn two_partial_specifications_merge_into_one_graph() {
        let manifest = Manifest::new()
            .with(part(
                "system.yaml",
                r"
format: ess/1
system: billing
version: v2
",
            ))
            .with(part(
                "domains/invoice.yaml",
                r"
domains:
  - domain: billing.invoice
    commands: [CreateInvoice]
    events: [InvoiceCreated]
",
            ))
            .with(part(
                "domains/email.yaml",
                r"
domains:
  - domain: billing.email
    commands: [SendEmail]
",
            ));

        assert_eq!(manifest.len(), 3);
        assert_eq!(
            manifest
                .sources()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["system.yaml", "domains/invoice.yaml", "domains/email.yaml"]
        );

        let spec = manifest.compile().expect("three files, one graph");
        assert_eq!(spec.reference(), "billing/v2");
        assert_eq!(spec.domains.len(), 2);
        assert_eq!(
            spec.owner_of(&name("billing.email.SendEmail"))
                .map(|domain| domain.name.to_string()),
            Some("billing.email".to_owned())
        );
    }

    #[test]
    fn two_sources_may_contribute_to_the_same_domain() {
        let spec = Manifest::new()
            .with(part("system.yaml", "system: billing"))
            .with(part(
                "domains/invoice.yaml",
                r"
domains:
  - domain: billing.invoice
    commands: [CreateInvoice]
",
            ))
            .with(part(
                "domains/invoice-tax.yaml",
                r"
domains:
  - domain: billing.invoice
    commands: [ApplyTax]
",
            ))
            .compile()
            .expect("one domain, split across two files, is the point of \u{a7}24");

        let invoice = spec.domain(&name("billing.invoice")).expect("declared");
        assert_eq!(
            invoice.commands,
            vec![
                name("billing.invoice.CreateInvoice"),
                name("billing.invoice.ApplyTax")
            ]
        );
        assert_eq!(spec.domains.len(), 1, "one domain, not two");
    }

    #[test]
    fn the_same_command_declared_by_two_sources_is_refused_naming_both() {
        let errors = Manifest::new()
            .with(part("system.yaml", "system: billing"))
            .with(part(
                "domains/invoice.yaml",
                r"
domains:
  - domain: billing.invoice
    commands: [CreateInvoice]
",
            ))
            .with(part(
                "domains/invoice-tax.yaml",
                r"
domains:
  - domain: billing.invoice
    commands: [CreateInvoice]
",
            ))
            .compile()
            .expect_err("one command, two files");

        assert_eq!(errors.len(), 1, "one conflict, reported once: {errors}");
        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::DuplicateDeclaration);
        assert_eq!(error.location, "domain billing.invoice.commands");
        assert!(
            error.message.contains("`domains/invoice.yaml`")
                && error.message.contains("`domains/invoice-tax.yaml`"),
            "a cross-file conflict is only fixable if the message names both files: {error}"
        );
    }

    #[test]
    fn a_system_level_type_declared_by_two_sources_is_refused_naming_both() {
        let errors = Manifest::new()
            .with(part("system.yaml", "system: billing"))
            .with(
                SpecPart::new("types/money.yaml")
                    .with_type(newtype("billing.Money", Primitive::Decimal)),
            )
            .with(
                SpecPart::new("types/money-again.yaml")
                    .with_type(newtype("billing.Money", Primitive::Decimal)),
            )
            .compile()
            .expect_err("one type, two files");

        assert!(
            errors.contains(ValidationCode::DuplicateDeclaration),
            "{errors}"
        );
        let rendered = errors.to_string();
        assert!(rendered.contains("`types/money.yaml`"), "{rendered}");
        assert!(rendered.contains("`types/money-again.yaml`"), "{rendered}");
        assert!(
            rendered.contains("the system itself"),
            "and says who claims it: {rendered}"
        );
    }

    #[test]
    fn a_specification_with_no_header_is_refused() {
        let errors = Manifest::new()
            .with(part(
                "domains/invoice.yaml",
                r"
domains:
  - domain: billing.invoice
    commands: [CreateInvoice]
",
            ))
            .compile()
            .expect_err("nothing says what system this is");

        assert_eq!(errors.len(), 1);
        let error = &errors.as_slice()[0];
        assert_eq!(
            error.code,
            ValidationCode::MissingDeclaration,
            "nothing is referenced here; a required declaration is missing: {error}"
        );
        assert_eq!(error.location, "system");
        assert!(
            error.message.contains("no source declares the system"),
            "{error}"
        );
    }

    #[test]
    fn two_headers_are_refused_naming_both_sources() {
        let errors = Manifest::new()
            .with(part("system.yaml", "system: billing"))
            .with(part("other.yaml", "system: shipping"))
            .compile()
            .expect_err("two systems in one specification");

        assert!(
            errors.contains(ValidationCode::DuplicateDeclaration),
            "{errors}"
        );
        let rendered = errors.to_string();
        assert!(
            rendered.contains("`system.yaml`") && rendered.contains("`other.yaml`"),
            "{rendered}"
        );
        assert!(rendered.contains("shipping"), "{rendered}");
    }

    #[test]
    fn a_fragment_that_sets_system_level_fields_without_naming_the_system_is_refused() {
        let raw: RawSystemSpec =
            serde_yaml::from_str("format: ess/1\nversion: v2\n").expect("document parses");
        let errors = raw
            .into_part("domains/invoice.yaml")
            .expect_err("a format without a system");

        assert!(
            errors.contains(ValidationCode::MissingDeclaration),
            "{errors}"
        );
        assert_eq!(errors.as_slice()[0].location, "domains/invoice.yaml.system");
    }

    #[test]
    fn a_specification_reports_every_problem_in_one_run() {
        let errors = system(
            r"
format: ess/99
system: billing
domains:
  - domain: shipping.parcel
    commands: [DispatchParcel]
  - domain: billing.invoice
    commands: [billing.email.SendEmail]
",
        )
        .expect_err("three unrelated problems");

        assert!(errors.len() >= 3, "one run, every problem: {errors}");
        assert!(
            errors.contains(ValidationCode::UnsupportedFormatVersion),
            "{errors}"
        );
        assert!(
            errors.contains(ValidationCode::ConflictingDeclaration),
            "{errors}"
        );
    }

    #[test]
    fn a_wire_name_change_does_not_move_the_system() {
        let spec = system(
            r"
system: billing
naming:
  wire: billing-api
  display: Billing
domains:
  - domain: billing.invoice
    naming:
      wire: invoices
",
        )
        .expect("validates");

        assert_eq!(spec.naming.wire_or(&spec.name), "billing-api");
        assert_eq!(spec.naming.display_or(&spec.name), "Billing");
        assert_eq!(
            spec.name.to_string(),
            "billing",
            "the identity is untouched"
        );
        let invoice = spec.domain(&name("billing.invoice")).expect("declared");
        assert_eq!(invoice.naming.wire_or(&invoice.name), "invoices");
    }

    #[test]
    fn two_types_that_cannot_be_built_without_each_other_are_refused() {
        let errors = system(
            r"
system: billing
types:
  - name: billing.Loop
    kind: newtype
    of: billing.Loop2
  - name: billing.Loop2
    kind: newtype
    of: billing.Loop
",
        )
        .expect_err("neither has a value, and a generator over them does not terminate");

        assert!(errors.contains(ValidationCode::SelfReference), "{errors}");
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::SelfReference)
            .expect("the cycle");
        assert!(
            error.message.contains("`billing.Loop`") && error.message.contains("`billing.Loop2`"),
            "the refusal names the way round: {error}"
        );
    }

    #[test]
    fn an_expression_tree_is_not_a_forbidden_cycle() {
        // The defect this rule used to have. `Expr` reaches itself through `Pair`, so a check that
        // refused every cycle of required names refused it — but every value of `Expr` bottoms out
        // in a `leaf`, so the type is perfectly ordinary and a generator over it terminates. A union
        // needs *one* buildable variant, not all of them.
        let errors = system_errors(
            r"
system: billing
version: v1
types:
  - name: billing.Expr
    kind: union
    tag: kind
    variants:
      leaf: Integer
      pair: billing.Pair
  - name: billing.Pair
    kind: struct
    fields:
      - name: left
        type: billing.Expr
      - name: right
        type: billing.Expr
",
        );
        assert!(
            !errors.contains(ValidationCode::SelfReference),
            "an expression tree is a legitimate specification: {errors}"
        );
    }

    #[test]
    fn a_union_whose_every_variant_recurses_is_refused() {
        // The other side of the same rule, so it cannot be loosened into accepting everything.
        let errors = system_errors(
            r"
system: billing
version: v1
types:
  - name: billing.Endless
    kind: union
    tag: kind
    variants:
      left: billing.Endless
      right: billing.Pair
  - name: billing.Pair
    kind: struct
    fields:
      - name: only
        type: billing.Endless
",
        );
        assert!(errors.contains(ValidationCode::SelfReference), "{errors}");
    }

    #[test]
    fn a_type_that_wraps_itself_is_refused_once_and_not_followed() {
        let errors = system(
            r"
system: billing
types:
  - name: billing.Knot
    kind: struct
    fields:
      - name: inner
        type: billing.Knot
",
        )
        .expect_err("a struct that contains itself has no smallest value");

        assert_eq!(errors.len(), 1, "one cycle, reported once: {errors}");
        assert_eq!(errors.as_slice()[0].code, ValidationCode::SelfReference);
        assert_eq!(errors.as_slice()[0].location, "types.billing.Knot");
    }

    #[test]
    fn a_type_that_reaches_itself_only_through_a_base_case_is_allowed() {
        let spec = system(
            r"
system: billing
types:
  - name: billing.Node
    kind: struct
    fields:
      - name: parent
        type: Optional<billing.Node>
      - name: children
        type: List<billing.Node>
",
        )
        .expect("an absent value and an empty list are values, so the type has some");

        assert_eq!(spec.types.len(), 1);
    }

    #[test]
    fn an_actor_declared_by_a_domain_is_claimed_like_every_other_member() {
        let spec = system(
            r"
system: billing
domains:
  - domain: billing.invoice
    actors: [Customer]
",
        )
        .expect("validates");

        assert_eq!(
            spec.owner_of(&name("billing.invoice.Customer"))
                .map(|domain| domain.name.to_string()),
            Some("billing.invoice".to_owned()),
            "a merge that dropped actors would leave every one of them owned by nobody"
        );
        assert_eq!(
            spec.declaration_of(&name("billing.invoice.Customer"))
                .map(|(_, kind)| kind),
            Some(MemberKind::Actor)
        );
    }

    #[test]
    fn a_type_declared_by_a_domain_and_by_the_system_has_one_owner() {
        let errors = system(
            r"
system: billing
types:
  - name: billing.invoice.Money
    kind: newtype
    of: Decimal
domains:
  - domain: billing.invoice
    types:
      - name: Money
        kind: newtype
        of: Decimal
",
        )
        .expect_err("one type, two claimants");

        assert!(
            errors.contains(ValidationCode::DuplicateDeclaration),
            "{errors}"
        );
        assert!(
            errors.to_string().contains("the system itself"),
            "the message says who else claims it: {errors}"
        );
    }
}

#[cfg(test)]
mod format_version_tests {
    use super::FormatVersion;

    #[test]
    fn the_format_pattern_accepts_exactly_what_the_parser_accepts() {
        // The pattern is what an editor enforces; the parser is what the build enforces. A document
        // one accepts and the other refuses is a contradiction this repository published itself.
        let matches = |value: &str| {
            value
                .strip_prefix(FormatVersion::PREFIX)
                .is_some_and(|digits| {
                    let mut characters = digits.chars();
                    characters
                        .next()
                        .is_some_and(|first| first.is_ascii_digit() && first != '0')
                        && characters.all(|character| character.is_ascii_digit())
                })
        };

        for accepted in ["ess/1", "ess/2", "ess/10", "ess/11", "ess/12", "ess/13"] {
            assert!(
                FormatVersion::parse(accepted).is_ok(),
                "{accepted} should parse"
            );
            assert!(
                matches(accepted),
                "the schema rejects `{accepted}`, which parses"
            );
        }
        for refused in [
            "ess/0", "ess/01", "ess/+1", "ess/", "1", "", "esss/1", "ess/1.0",
        ] {
            assert!(
                FormatVersion::parse(refused).is_err(),
                "{refused} should not parse"
            );
            assert!(
                !matches(refused),
                "the schema accepts `{refused}`, which the parser refuses"
            );
        }
    }
}
