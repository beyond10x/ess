//! Software decomposition: which component owns which domain, and what it accepts and publishes.
//!
//! A component is a *logical* boundary, not a deployment decision (design §5). `invoice-service`
//! owning `billing.invoice` says the invoice context is one unit of ownership; whether it ships as
//! its own process or as a module inside one binary is [`crate::topology`]'s business, and changing
//! that answer must not change this file.
//!
//! What a component declares is checkable, and worth checking:
//!
//! | rule | code |
//! |---|---|
//! | it owns a domain nothing declares | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | it accepts a command nothing declares | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | it publishes an event nothing declares | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | it accepts a command another component owns the domain of | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | two components own the same domain | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | it owns nothing, accepts nothing and publishes nothing | [`EmptyDeclaration`](ValidationCode::EmptyDeclaration) |
//! | it is [`reached_by: network`](Reach::Network) and no route follows — it accepts no command and owns no domain that declares a view | [`EmptyDeclaration`](ValidationCode::EmptyDeclaration) |
//!
//! # What a component declares about its configuration
//!
//! [`RawComponentSpec::settings`] is one list, and it is one list on purpose: configuration is
//! unbounded per application, and the guard against this growing a second type language beside
//! `types:` is that a setting declares nothing of its own except a name, two flags and an optional
//! literal. The type is a [`TypeRef`], so every invariant `types:` can state is available here and
//! none of them is restated.
//!
//! | rule | code | where |
//! |---|---|---|
//! | a setting whose `type` nothing declares | [`UndeclaredReference`](ValidationCode::UndeclaredReference) | [`validate_setting_types`] |
//! | a setting typed by an **entity** | [`TypeMismatch`](ValidationCode::TypeMismatch) | [`validate_setting_types`] |
//! | a setting typed by a name whose representation admits absence | [`TypeMismatch`](ValidationCode::TypeMismatch) | [`validate_setting_types`] |
//! | two settings with one name | [`DuplicateDeclaration`](ValidationCode::DuplicateDeclaration) | [`ComponentSpec::validate_settings`] |
//! | `secret: true` beside a literal `value:` | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) | [`ComponentSpec::validate_settings`] |
//! | `secret: true` beside a type that admits absence | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) | [`ComponentSpec::validate_settings`] |
//! | a written `required:` the type contradicts | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) | [`ComponentSpec::validate_settings`] |
//!
//! No new [`ValidationCode`]: each is an existing refusal reaching a new declaration.
//!
//! # `settings:` is `ess/1`, and it is forward-incompatible
//!
//! The two statements are not in tension, and both have to be written down.
//!
//! **No new format version.** The key is optional, the resolved list is empty when it is unstated,
//! and an empty list serialises out — of [`ComponentSpec`], of the IR's resolved component and
//! therefore of the model digest. So every document that does not use it compiles to exactly the
//! bytes it compiled to before the key existed, and no committed artifact in this repository moves.
//! That is the same position `reached_by` and `cli:` took, for the same reason, and
//! `ess-compiler/tests/component_settings.rs` holds it as a test rather than as this paragraph.
//!
//! **And an older binary refuses a document that uses it.** [`RawComponentSpec`] is
//! `deny_unknown_fields`, so a build made before this key existed does not ignore `settings:` — it
//! reports an unknown-field refusal naming `settings` and stops. A document that declares
//! settings therefore requires a build at least this new, while the format version it carries
//! still reads `ess/1`. That is a real incompatibility the version number does not signal: `ess/N`
//! is
//! this repository's statement about *meaning*, and nothing about the meaning of an existing
//! document changed. The upgrade obligation is one-directional and is recorded in
//! `changes/component-settings-0.24.0.yaml`.
//!
//! # Where each rule lives
//!
//! The sixth row is the only one a component can answer alone, so it is [`ComponentSpec::validate`]
//! and it runs during conversion: an empty component never reaches a
//! [`Specification`](crate::spec::Specification). The three reference rules need the domains,
//! commands and events the whole specification declares — [`ComponentSpec::validate_references`].
//! The two conflicts need the *other* components as well, which no component has in hand, so they
//! are [`validate_components`]. So does the last row, for a third reason: a view is declared inside
//! a domain, so "does anything I own project a row" is a question about the domains and not about
//! the component. That function runs all six.
//!
//! # Ownership is what makes a conflict a conflict
//!
//! Design §9 draws the system as one graph, and the edge that matters here is `COMMAND` →
//! *handled by* → `COMPONENT` → *modifies* → `DOMAIN STATE`. A command has one handler, and the
//! handler is the component that owns the domain whose state the command changes. So two
//! components both claiming `billing.invoice` — or a component accepting `billing.invoice.X` while
//! another owns `billing.invoice` — are two well-formed statements that cannot both hold, which is
//! what [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) is for. Wave 2's
//! bindings make it load-bearing rather than tidy: a binding's `invoke.command` has to resolve to
//! one destination, and it cannot if two components answer.
//!
//! # What this deliberately does not refuse
//!
//! Each of these is a shape an author may legitimately want, and §20's rejection list — which names
//! "components accepting undefined commands" and nothing else about components — does not ask for
//! it. Refusing something with no legal way to express it is worse than a rule left unwritten, so
//! each stays legal until evidence says otherwise.
//!
//! **A domain no component owns.** §5 says component responsibility "is logical; it is not yet a
//! deployment decision", and wave 1 shipped a whole specification with no components at all. A
//! model that has domains and has not been decomposed yet is a legitimate state; if an unowned
//! domain were an error, adding the *first* component to a specification would retroactively
//! invalidate every domain that has none.
//!
//! **A command accepted from a domain nobody owns.** Nothing else claims the handler edge, so there
//! is no contradiction — only an ownership statement that has not been written. This is also what
//! makes partial decomposition work: the commands of an undecomposed domain still have a handler.
//!
//! **An event published from a domain the component does not own.** Acceptance is a *destination* —
//! §9 gives a command exactly one handler, so a second claim is ambiguous. Publication is a
//! *source*: §9's `EVENT` → *triggers* → `COMMAND` edge does not ask who published, and §12 pairs a
//! publisher with a consumer by the event's identity rather than by component. §6's outer surface
//! is explicitly where "event topics" and "external APIs" live, so a component that translates and
//! re-publishes is a shape the design contemplates. Two publishers of one event is therefore not a
//! statement that cannot hold, and it is not refused here.

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::error::{
    ConstructKind, ConstructRef, ValidationCode, ValidationError, ValidationErrors,
};

use crate::name::{Naming, QualifiedName};
use crate::refs::Refs;
use crate::types::{TypeBody, TypeRef};

/// What a component is made of, as a document says it.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawComponentSpec {
    /// Its name — a single segment, like `invoice-service`.
    #[serde(alias = "component")]
    pub name: String,
    /// The domains it owns.
    #[serde(default)]
    pub owns: RawComponentOwns,
    /// The commands it accepts.
    #[serde(default)]
    pub accepts: RawComponentSurface,
    /// The events it publishes.
    #[serde(default)]
    pub publishes: RawComponentSurface,
    /// Who reaches its accepted commands and the views its domains declare.
    ///
    /// Unstated is [`Reach::InProcess`] — the shape every specification here had before the word
    /// existed, and the one a reader assumes when nothing says otherwise.
    #[serde(default)]
    pub reached_by: Reach,
    /// Where each accepted command sits in the command tree, when this is a command-line surface.
    ///
    /// Unstated is absent, and absent digests as it did before the key existed. Only meaningful
    /// beside [`Reach::CommandLine`]; anywhere else it is refused.
    #[serde(default)]
    pub cli: Option<RawCommandLineSurface>,
    /// What it is called on the wire and shown as.
    #[serde(default)]
    pub naming: Naming,
    /// What it is, in one line.
    #[serde(default)]
    pub summary: Option<String>,
    /// The configuration inputs it reads.
    ///
    /// Unstated is empty, and empty serialises out, so a specification written before this key
    /// existed digests exactly as it did — the same position `reached_by` and `cli:` took.
    #[serde(default)]
    pub settings: Vec<RawComponentSetting>,
    /// The records outside this model that explain it, such as `jira:DEV-630`.
    ///
    /// Empty by default. See [`crate::refs`] for why this is a reference and not a paragraph.
    #[serde(default, skip_serializing_if = "crate::refs::is_empty")]
    pub refs: Refs,
}

/// One configuration input a component declares, as a document says it.
///
/// The type is a [`TypeRef`], so `types:` is reused entire — invariants included — rather than
/// growing a second type language beside it. Configuration is unbounded per application, and what
/// keeps this list from becoming that language is that it declares nothing of its own beyond a name
/// and three flags.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawComponentSetting {
    /// Its identity, spelt the way a word a person types is.
    ///
    /// The charset is published, because it is checkable there: a schema that accepts what
    /// [`CliName::new`] refuses would hand an author a green editor and a red `ess validate`.
    #[schemars(regex(pattern = "^[a-z][a-z0-9]*(-[a-z0-9]+)*$"))]
    pub name: String,
    /// Its type: anything the specification declares.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    /// Whether a value must be present.
    ///
    /// A restatement of the type, and optional: unstated is unchecked — see
    /// [`ComponentSpec::validate_settings`] for why silence is not a contradiction — and answered
    /// by the type, which is [`requires_a_value`].
    #[serde(default)]
    pub required: Option<bool>,
    /// Whether the model may not hold the value.
    #[serde(default)]
    pub secret: bool,
    /// A public literal the specification fixes.
    #[serde(default)]
    pub value: Option<String>,
    /// What it is, in one line, for generated documentation.
    #[serde(default)]
    pub summary: Option<String>,
}

/// Where the callers of a component's surface are.
///
/// # A fact about the system, not a protocol
///
/// The model has no transport construct and this is not one: neither variant names a wire format,
/// a port, a path or a verb. What it states is who issues the commands — a question a specification
/// answers about its own design, and the one thing a generator cannot infer. `invoice-service` and
/// `email-service` in the normative example are reached in process, and that is why the one
/// transport their bindings determine is an in-process log; a component reached across a network
/// has callers that are not deployed with it, and its surface has to exist on a wire.
///
/// # What follows from `network`, and what does not
///
/// Which wire is *derived*, never chosen: this repository projects exactly one contract for a
/// component's command surface — the `OpenAPI` document `ess-gen` writes for every component — and
/// an `OpenAPI` document is an HTTP contract. So a `network` surface is served over HTTP because
/// that is the only contract that already exists for it, and a synthesised server that spoke
/// anything else would contradict the document committed beside it. No variant here says `http`,
/// and adding one would be the transport DSL design §7 sketches and this model has not taken.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Reach {
    /// Every caller is deployed with it, so the surface never leaves the process.
    ///
    /// The default, and the only reading of silence that does not invent a deployment: a
    /// specification that has not said where its callers are has not said its surface crosses
    /// anything.
    #[default]
    InProcess,
    /// At least one caller is not deployed with it, so the surface crosses a process boundary.
    ///
    /// The declaration that makes the published `OpenAPI` document load-bearing rather than
    /// documentary: something outside has to issue these commands, and the only way it can is over
    /// the contract.
    Network,
    /// Every caller is a person at a terminal, typing.
    ///
    /// Deployed with it, like [`Self::InProcess`], and yet the surface does leave the process: it
    /// leaves as a *grammar* rather than a call, and a grammar is a contract a person reads. That
    /// is the fact neither other variant can state, and the reason this one exists.
    ///
    /// Which contract follows is derived here too. [`Self::Network`] derives an `OpenAPI` document
    /// because that is the one contract this repository projects for a command surface on a wire;
    /// `command_line` derives the `clap` tree and completion scripts `ess generate synthesize
    /// --target clap` writes, for the same reason. No variant names a wire, a port, a path or a
    /// verb, and this one does not either.
    CommandLine,
}

impl Reach {
    /// The three words `reached_by:` accepts, in the order [`Reach`] declares them.
    pub const ALL: &'static [Reach] = &[Reach::InProcess, Reach::Network, Reach::CommandLine];

    /// The word a document writes.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InProcess => "in_process",
            Self::Network => "network",
            Self::CommandLine => "command_line",
        }
    }

    /// `true` where the surface never leaves the process — the unstated default.
    ///
    /// Used by the IR to leave the default out of the resolved model's serialisation, so a
    /// specification that says nothing about reach digests exactly as it did before the word
    /// existed.
    pub fn is_in_process(self) -> bool {
        matches!(self, Self::InProcess)
    }
}

/// What a component owns.
#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawComponentOwns {
    /// The domains, by qualified name.
    #[serde(default)]
    pub domains: Vec<QualifiedName>,
}

/// The commands a component accepts, or the events it publishes.
///
/// One shape for both because the document spells them the same way (§5) and a second shape would be
/// a second thing to keep in step.
#[derive(Debug, Clone, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawComponentSurface {
    /// Commands, when this is an `accepts:` block.
    #[serde(default)]
    pub commands: Vec<QualifiedName>,
    /// Events, when this is a `publishes:` block.
    #[serde(default)]
    pub events: Vec<QualifiedName>,
}

/// Where a command-line surface puts each command it accepts, as a document says it.
///
/// # Why this is declared and paths are not
///
/// A command's own path is derived, the way an `OpenAPI` path already is: `naming.wire` gives it
/// one token. Which *activity* it belongs under cannot be derived from anything — `doctor` and
/// `providers` are both reads, `init` and `connect` are both first-run, and no field in the model
/// says so. That judgement is the whole content of this block, and writing it here is what lets a
/// checker answer "does every command have exactly one place" at all.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawCommandLineSurface {
    /// What the binary is called, as typed.
    pub binary: String,
    /// Commands that sit at the top level, under no group.
    #[serde(default)]
    pub commands: Vec<QualifiedName>,
    /// Views read at the top level, under no group.
    #[serde(default)]
    pub views: Vec<QualifiedName>,
    /// The groups, each a first-level word with commands under it.
    #[serde(default)]
    pub groups: Vec<RawCommandGroup>,
}

/// One first-level word in a command tree, as a document says it.
#[derive(Debug, Clone, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawCommandGroup {
    /// The word, as typed.
    pub name: String,
    /// What the group is, in one line. Becomes the group's `--help` text.
    #[serde(default)]
    pub summary: Option<String>,
    /// The commands under it.
    #[serde(default)]
    pub commands: Vec<QualifiedName>,
    /// The views read under it.
    #[serde(default)]
    pub views: Vec<QualifiedName>,
}

/// A word a person types: a binary name, or a group.
///
/// The same charset as [`ComponentName`] and a separate type, because these are different things
/// that happen to be spelt alike — a component becomes a workload name, and this becomes an
/// argument. Sharing the type would make a rename of one look like a rename of the other.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct CliName(String);

impl CliName {
    /// What a typed word looks like.
    pub const PATTERN: &'static str = "^[a-z][a-z0-9]*(-[a-z0-9]+)*$";

    /// Parses one.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ess_primitives::error::ParseError> {
        let value = value.as_ref();
        let valid = !value.is_empty()
            && value.starts_with(|c: char| c.is_ascii_lowercase())
            && !value.ends_with('-')
            && !value.contains("--")
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !valid {
            return Err(ess_primitives::error::ParseError::identifier(
                "command-line name",
                value,
                "a binary or group is lower-case words joined by single hyphens, such as \
                 `serve-hosted`; it is typed at a shell"
                    .to_owned(),
            ));
        }
        Ok(Self(value.to_owned()))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CliName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl schemars::JsonSchema for CliName {
    fn schema_name() -> String {
        "CliName".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(Self::PATTERN.to_owned());
        schema.metadata().description =
            Some("A word typed at a shell, such as `inspect`.".to_owned());
        schema.into()
    }
}

/// Where a command-line surface puts each command it accepts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CommandLineSurface {
    /// What the binary is called.
    pub binary: CliName,
    /// Commands at the top level, under no group.
    pub commands: BTreeSet<QualifiedName>,
    /// Views read at the top level, under no group.
    pub views: BTreeSet<QualifiedName>,
    /// The groups, in the order the document wrote them.
    ///
    /// Ordered as written rather than sorted: a command tree's first level is read top to bottom by
    /// a person, and alphabetising it would put `admin` above the verb they came for.
    pub groups: Vec<CommandGroup>,
}

/// One first-level word in a command tree.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CommandGroup {
    /// The word.
    pub name: CliName,
    /// What the group is, in one line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// The commands under it.
    pub commands: BTreeSet<QualifiedName>,
    /// The views read under it.
    pub views: BTreeSet<QualifiedName>,
}

/// A component: one unit of ownership.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ComponentSpec {
    /// Its name.
    pub name: ComponentName,
    /// The domains it owns.
    pub owns: BTreeSet<QualifiedName>,
    /// The commands it accepts.
    pub accepts: BTreeSet<QualifiedName>,
    /// The events it publishes.
    pub publishes: BTreeSet<QualifiedName>,
    /// Where the callers of its surface are.
    pub reached_by: Reach,
    /// Where each accepted command sits in the command tree.
    ///
    /// `None` for every component that is not a command-line surface, and skipped when it is, so a
    /// specification written before this key existed digests exactly as it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<CommandLineSurface>,
    /// What it is called on the wire, and what a person is shown.
    pub naming: Naming,
    /// The configuration inputs it reads, in the order the document declared them.
    ///
    /// Empty for every component that declares none, and skipped when it is, so a specification
    /// written before this key existed serialises exactly as it did.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<ComponentSetting>,
    /// The records outside this model that explain it, such as `jira:DEV-630`.
    ///
    /// Empty by default. See [`crate::refs`] for why this is a reference and not a paragraph.
    #[serde(default, skip_serializing_if = "crate::refs::is_empty")]
    pub refs: Refs,
}

/// One configuration input a component declares.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ComponentSetting {
    /// Its identity.
    pub name: CliName,
    /// Its type.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    /// Whether a value must be present, when the document said so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Whether the model may not hold the value.
    #[serde(skip_serializing_if = "unstated_secrecy")]
    pub secret: bool,
    /// A public literal the specification fixes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// What it is, in one line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// `true` where a setting's secrecy is the unstated default.
///
/// A free function rather than a method, because `skip_serializing_if` hands the field by reference
/// and there is no by-value form of the attribute — the same reason [`Reach`]'s
/// `unstated_reach` sits beside the IR's component rather than on the enum.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn unstated_secrecy(secret: &bool) -> bool {
    !*secret
}

impl ComponentSetting {
    /// The environment variable a runtime binds this setting to.
    ///
    /// # Two setting names never derive one variable
    ///
    /// The map is per character and length-preserving: `[a-z]` uppercases, `[0-9]` passes through,
    /// `-` becomes `_`. Those three images are pairwise disjoint — `[A-Z]`, `[0-9]`, `{_}` — and
    /// each map is injective on its own domain, so the map is injective on a character and
    /// therefore on a string of equal length; and the length is the input's. [`CliName`] admits
    /// nothing else, and in particular no underscore, which is the one character that could have
    /// arrived by two routes. So a colliding pair does not exist, and this is not a sixth refusal.
    ///
    /// That argument holds exactly as far as [`CliName::PATTERN`] does, which is why
    /// `no_two_setting_names_derive_one_environment_variable` enumerates the charset instead of
    /// trusting the paragraph.
    pub fn environment(&self) -> String {
        environment_variable(&self.name)
    }

    /// Whether a value must be present.
    ///
    /// Unstated is answered by the type, through [`requires_a_value`] rather than a second copy of
    /// the rule.
    pub fn is_required(&self) -> bool {
        requires_a_value(self.required, self.type_ref.is_optional())
    }
}

/// Whether a setting written with this `required:` over a type of this optionality must carry a
/// value.
///
/// # Silence is the type's answer, and never `false`
///
/// `required:` is a *restatement* of the type — [`ComponentSpec::validate_settings`] refuses it in
/// both directions the moment it disagrees — so the type is the authority and the key is the
/// reading aid. Answering an unstated `required` with `false` made that refusal evadable and left
/// it preventing nothing: a setting typed by something that admits no absence and written
/// `required: false` is refused, and the identical document with that one line deleted was
/// accepted and derived the very slot the refusal exists to prevent, which
/// `ess-deployment`'s `environment.rs` then never requires an environment binding for. Deleting
/// the line now changes no answer, so there is nothing to evade.
///
/// Free and public for the reason [`environment_variable`] is: the compiler's resolved setting and
/// the deployment projector behind it ask this question through this function, so the rule has one
/// site and a third reader cannot quietly disagree with the first two.
pub fn requires_a_value(required: Option<bool>, type_is_optional: bool) -> bool {
    required.unwrap_or(!type_is_optional)
}

/// The environment variable a setting of this name binds to.
///
/// Free and public so that the IR's own setting — which holds a resolved type and the same name —
/// derives the variable through this function rather than through a second copy of the rule.
pub fn environment_variable(name: &CliName) -> String {
    name.as_str()
        .chars()
        .map(|character| {
            if character == '-' {
                '_'
            } else {
                character.to_ascii_uppercase()
            }
        })
        .collect()
}

/// A component's name.
///
/// Not a [`QualifiedName`]: a component is not inside a domain, and giving it a dotted name would
/// invite the reading that `billing.invoice-service` belongs to `billing.invoice`. It becomes a
/// workload name, a container name and a metrics label, so it is spelt the way those are.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct ComponentName(String);

impl ComponentName {
    /// What a component name looks like.
    pub const PATTERN: &'static str = "^[a-z][a-z0-9]*(-[a-z0-9]+)*$";

    /// Parses one.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ess_primitives::error::ParseError> {
        let value = value.as_ref();
        let valid = !value.is_empty()
            && value.starts_with(|c: char| c.is_ascii_lowercase())
            && !value.ends_with('-')
            && !value.contains("--")
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !valid {
            return Err(ess_primitives::error::ParseError::identifier(
                "component name",
                value,
                "a component name is lower-case words joined by single hyphens, such as \
                 `invoice-service`; it becomes a workload name and a metrics label"
                    .to_owned(),
            ));
        }
        Ok(Self(value.to_owned()))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ComponentName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl schemars::JsonSchema for ComponentName {
    fn schema_name() -> String {
        "ComponentName".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(Self::PATTERN.to_owned());
        schema.metadata().description =
            Some("A component's name, such as `invoice-service`.".to_owned());
        schema.into()
    }
}

impl TryFrom<RawComponentSpec> for ComponentSpec {
    type Error = ValidationErrors;

    fn try_from(raw: RawComponentSpec) -> Result<Self, Self::Error> {
        let mut errors = ValidationErrors::new();
        let name = match ComponentName::new(&raw.name) {
            Ok(name) => name,
            Err(error) => {
                return Err(ValidationErrors::new().with(ValidationError::new(
                    ValidationCode::TypeMismatch,
                    format!("component {}", raw.name),
                    error.to_string(),
                )));
            }
        };

        let cli = match raw.cli {
            None => None,
            Some(cli) => match resolve_command_line(&raw.name, cli) {
                Ok(cli) => Some(cli),
                Err(refusals) => {
                    errors.extend(refusals);
                    None
                }
            },
        };

        let settings = match resolve_settings(&raw.name, raw.settings) {
            Ok(settings) => settings,
            Err(refusals) => {
                errors.extend(refusals);
                Vec::new()
            }
        };

        let component = Self {
            name,
            owns: raw.owns.domains.into_iter().collect(),
            accepts: raw.accepts.commands.into_iter().collect(),
            publishes: raw.publishes.events.into_iter().collect(),
            reached_by: raw.reached_by,
            cli,
            naming: Naming {
                summary: raw.naming.summary.or(raw.summary),
                ..raw.naming
            },
            settings,
            refs: raw.refs,
        };

        errors.extend(component.validate());
        errors.into_result(component)
    }
}

/// Turns a written `cli:` block into the resolved one, or names every word that is not a word.
///
/// Separate from [`ComponentSpec::validate_command_line`] because the two answer different
/// questions: this one is *is this spelt like something a person types*, and that one is *does the
/// tree it describes cover the commands exactly once*. A name that failed here would make the
/// second check report a second, derived problem about a name nobody can type anyway.
fn resolve_command_line(
    component: &str,
    raw: RawCommandLineSurface,
) -> Result<CommandLineSurface, ValidationErrors> {
    let mut errors = ValidationErrors::new();
    let mut named = |value: &str| match CliName::new(value) {
        Ok(name) => Some(name),
        Err(error) => {
            errors.push(ValidationError::new(
                ValidationCode::TypeMismatch,
                format!("component {component}"),
                error.to_string(),
            ));
            None
        }
    };

    let binary = named(&raw.binary);
    let groups: Vec<_> = raw
        .groups
        .into_iter()
        .filter_map(|group| {
            named(&group.name).map(|name| CommandGroup {
                name,
                summary: group.summary,
                commands: group.commands.into_iter().collect(),
                views: group.views.into_iter().collect(),
            })
        })
        .collect();

    match binary {
        Some(binary) if errors.is_empty() => Ok(CommandLineSurface {
            binary,
            commands: raw.commands.into_iter().collect(),
            views: raw.views.into_iter().collect(),
            groups,
        }),
        _ => Err(errors),
    }
}

/// Where a refusal about one setting points.
///
/// One definition, used by every settings rule, so that the document path a reader is sent to is
/// the same string whichever rule refused and cannot drift between them — which is the whole point
/// of a typed [`ConstructRef`] over a `format!` per call site.
fn setting_site(component: &str, setting: &str) -> ConstructRef {
    ConstructRef::new(ConstructKind::Component, component)
        .key("settings")
        .named(setting)
}

/// Turns written settings into resolved ones, or names every name that is not a name.
///
/// Separate from [`ComponentSpec::validate_settings`] for the reason
/// [`resolve_command_line`] is separate from
/// [`ComponentSpec::validate_command_line`]: this one asks *is this spelt like something a person
/// types*, and that one asks *do these declarations contradict each other*. A setting whose name
/// failed here would make the second check report a derived problem about a name nobody can write.
fn resolve_settings(
    component: &str,
    raw: Vec<RawComponentSetting>,
) -> Result<Vec<ComponentSetting>, ValidationErrors> {
    let mut errors = ValidationErrors::new();
    let mut settings = Vec::with_capacity(raw.len());
    for setting in raw {
        match CliName::new(&setting.name) {
            Ok(name) => settings.push(ComponentSetting {
                name,
                type_ref: setting.type_ref,
                required: setting.required,
                secret: setting.secret,
                value: setting.value,
                summary: setting.summary,
            }),
            Err(error) => errors.push(ValidationError::at(
                setting_site(component, &setting.name),
                ValidationCode::TypeMismatch,
                error.to_string(),
            )),
        }
    }
    errors.into_result(settings)
}

impl ComponentSpec {
    /// Everything checkable without the rest of the specification.
    pub fn validate(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if self.owns.is_empty() && self.accepts.is_empty() && self.publishes.is_empty() {
            errors.push(
                ValidationError::new(
                    ValidationCode::EmptyDeclaration,
                    format!("component {}", self.name),
                    format!(
                        "`{}` owns nothing, accepts nothing and publishes nothing",
                        self.name
                    ),
                )
                .with_hint(
                    "a component that does nothing is a name; delete it or give it a domain",
                ),
            );
        }
        errors.extend(self.validate_command_line());
        errors.extend(self.validate_settings());
        errors
    }

    /// Everything a settings list can be wrong about without the rest of the specification.
    ///
    /// | refused | code |
    /// |---|---|
    /// | two settings with one name | [`DuplicateDeclaration`](ValidationCode::DuplicateDeclaration) |
    /// | `secret: true` beside a literal `value:` | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
    /// | `secret: true` beside a type that admits absence | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
    /// | a written `required:` that the type contradicts | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
    ///
    /// The other three settings rules — a type nothing declares, a type that is an entity, and a
    /// type whose *name* admits absence — need the whole specification, and are
    /// [`validate_setting_types`].
    ///
    /// # Why an optional secret is refused rather than derived
    ///
    /// A setting's optionality survives into the projection as
    /// `ConfigKind::Required`/`ConfigKind::Optional`, and that is the whole of what the projection
    /// can say about it. A secret becomes a `SecretSlot`, which carries a name, an environment
    /// variable and a key and **nothing that says the value may be absent**, and
    /// `ess-deployment`'s `environment.rs` requires every secret slot in a locked runtime to be
    /// bound. So `secret: true` over an `Optional<…>` type is a statement the model accepts at the
    /// declaration and drops at the slot: the setting is declared optional and then refused for
    /// being unbound, in a document whose author never wrote the requirement.
    ///
    /// It is refused here rather than carried because carrying it is a new statement in a
    /// persisted format — a kind on `SecretSlot`, a matching field in the composition lock's
    /// secret set, and a gate that reads it — and `ess-runtime/1` cannot gain one without a format
    /// consequence. Refusing says the same thing at the declaration, where the author can act on
    /// it, and leaves the projection total: every slot the derivation builds carries every
    /// statement its setting made.
    ///
    /// # Why `required:` is checked in both directions, and only when it is written
    ///
    /// [`TypeRef::Optional`] is the model's only statement that a value may be absent;
    /// [`crate::binding`] says so in as many words, and a [`Field`](crate::types::Field) carries no
    /// `required:` at all for that reason. The key here is a *restatement* of the type, written so
    /// that a reader of a settings list sees requiredness without parsing a type expression — so
    /// the rule is that the restatement agrees. `required: false` over a type that admits no
    /// absence is refused, and so is `required: true` over one that does: they are one
    /// contradiction read from two ends, and refusing only the first would leave `Optional` meaning
    /// one thing to the type checker and another to whoever read the flag.
    ///
    /// Silence is not refused. A restatement can only be wrong when it is made, and refusing an
    /// unstated `required` would make the key mandatory on every setting whose type is not
    /// `Optional<…>` — the opposite of the optional key this is.
    ///
    /// Silence is not *free*, either. An unstated `required` is answered by the type, in
    /// [`requires_a_value`], because answering it `false` would have made this refusal evadable by
    /// deleting the line it refuses: the same document without `required: false` would be accepted
    /// and derive the same optional slot. A rule a document escapes by saying less is not a rule.
    pub fn validate_settings(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at =
            |setting: &ComponentSetting| setting_site(self.name.as_str(), setting.name.as_str());
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for setting in &self.settings {
            if !seen.insert(setting.name.as_str()) {
                errors.push(
                    ValidationError::at(
                        at(setting),
                        ValidationCode::DuplicateDeclaration,
                        format!(
                            "`{}` declares the setting `{}` more than once",
                            self.name, setting.name
                        ),
                    )
                    .with_hint(
                        "a setting has one name and one type; keep the declaration that is right \
                         and drop the other",
                    ),
                );
            }
            if setting.secret && setting.value.is_some() {
                errors.push(
                    ValidationError::at(
                        at(setting),
                        ValidationCode::ConflictingDeclaration,
                        format!(
                            "the setting `{}` is `secret: true` and carries a literal `value:`",
                            setting.name
                        ),
                    )
                    .with_hint(
                        "a secret is a value this model may not hold; drop the `value:`, or drop \
                         `secret: true` and keep the literal",
                    ),
                );
            }
            if setting.secret && setting.type_ref.is_optional() {
                errors.push(
                    ValidationError::at(
                        at(setting),
                        ValidationCode::ConflictingDeclaration,
                        format!(
                            "the setting `{}` is `secret: true` and typed `{}`, which says the \
                             value may be absent",
                            setting.name, setting.type_ref
                        ),
                    )
                    .with_hint(format!(
                        "a secret derives a secret slot, and every secret slot must be bound \
                         before an environment is deployed; the model has no way to say that a \
                         secret is optional. Type it `{}`, or drop `secret: true` and declare it \
                         as configuration",
                        setting.type_ref.required()
                    )),
                );
            }
            match (setting.required, setting.type_ref.is_optional()) {
                (Some(false), false) => errors.push(
                    ValidationError::at(
                        at(setting),
                        ValidationCode::ConflictingDeclaration,
                        format!(
                            "the setting `{}` is `required: false` and typed `{}`, which is not \
                             `Optional<…>`",
                            setting.name, setting.type_ref
                        ),
                    )
                    .with_hint(format!(
                        "`Optional<…>` is the model's only way of saying that a value may be \
                         absent: type it `Optional<{}>`, or say `required: true`",
                        setting.type_ref
                    )),
                ),
                (Some(true), true) => errors.push(
                    ValidationError::at(
                        at(setting),
                        ValidationCode::ConflictingDeclaration,
                        format!(
                            "the setting `{}` is `required: true` and typed `{}`, which says the \
                             value may be absent",
                            setting.name, setting.type_ref
                        ),
                    )
                    .with_hint(format!(
                        "drop the wrapper and type it `{}`, or say `required: false`",
                        setting.type_ref.required()
                    )),
                ),
                _ => {}
            }
        }
        errors
    }

    /// Checks that a command tree names every accepted command exactly once, and nothing else.
    ///
    /// Five refusals, and they are the reason the tree is declared here rather than written beside
    /// the parser. A document beside a parser is checked by nobody; these are checked by
    /// `ess validate` before anything is generated from it.
    ///
    /// | refused | why |
    /// |---|---|
    /// | `cli:` without [`Reach::CommandLine`] | it describes a surface the component does not have |
    /// | [`Reach::CommandLine`] without `cli:` | there is nothing to generate a tree from |
    /// | a placed command the component does not accept | the same unknown-target refusal `relations:` gives |
    /// | an accepted command placed nowhere | the tree would be missing a command, and nothing else would say so |
    /// | an accepted command placed twice | a command has one path, and two answers is an undecided question written down twice |
    pub fn validate_command_line(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at = || format!("component {}", self.name);
        let cli = match (&self.cli, self.reached_by) {
            (None, Reach::CommandLine) => {
                errors.push(
                    ValidationError::new(
                        ValidationCode::MissingDeclaration,
                        at(),
                        format!(
                            "`{}` is reached from a command line and says nothing about its \
                             command tree",
                            self.name
                        ),
                    )
                    .with_hint(
                        "give it a `cli:` block naming the binary and where each accepted \
                         command sits, or say `reached_by: in_process`",
                    ),
                );
                return errors;
            }
            (Some(_), reach) if reach != Reach::CommandLine => {
                errors.push(
                    ValidationError::new(
                        ValidationCode::ConflictingDeclaration,
                        at(),
                        format!(
                            "`{}` declares a command tree and is `reached_by: {}`",
                            self.name,
                            reach.as_str()
                        ),
                    )
                    .with_hint(
                        "a command tree is what `reached_by: command_line` produces; say that, \
                         or delete the `cli:` block",
                    ),
                );
                return errors;
            }
            (None, _) => return errors,
            (Some(cli), _) => cli,
        };
        errors.extend(self.validate_command_placement(cli));
        errors
    }

    /// The half of [`Self::validate_command_line`] about the tree rather than about declaring one.
    ///
    /// Split out because the two answer different questions and the first has to settle before the
    /// second can be asked: a component that should not have a tree at all, or has none where one
    /// is required, has no placement to check.
    fn validate_command_placement(&self, cli: &CommandLineSurface) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let at = || format!("component {}", self.name);
        // Where each command was placed, and how often. Ordered, so the refusals below come out in
        // the same order on every run.
        let mut placements: BTreeMap<&QualifiedName, Vec<String>> = BTreeMap::new();
        for command in &cli.commands {
            placements
                .entry(command)
                .or_default()
                .push("the top level".to_owned());
        }
        for group in &cli.groups {
            for command in &group.commands {
                placements
                    .entry(command)
                    .or_default()
                    .push(format!("`{}`", group.name));
            }
        }

        for (command, places) in &placements {
            if !self.accepts.contains(*command) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        at(),
                        format!(
                            "the command tree places `{command}`, which `{}` does not accept",
                            self.name
                        ),
                    )
                    .with_hint(
                        "a tree can only place a command the component accepts; add it to \
                         `accepts.commands`, or remove it from the tree",
                    ),
                );
            }
            if places.len() > 1 {
                errors.push(
                    ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        at(),
                        format!(
                            "the command tree places `{command}` under {}",
                            places.join(" and ")
                        ),
                    )
                    .with_hint(
                        "a command has one path; keep the place it belongs and drop the other",
                    ),
                );
            }
        }

        for command in &self.accepts {
            if !placements.contains_key(command) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::MissingDeclaration,
                        at(),
                        format!(
                            "`{}` accepts `{command}` and the command tree gives it no place",
                            self.name
                        ),
                    )
                    .with_hint(
                        "put it under a group or at the top level; a command with no place is one \
                         the generated tree would not carry",
                    ),
                );
            }
        }

        errors
    }

    /// Checks that a surface declared reachable from outside has something to serve.
    ///
    /// `projecting` is the set of domains that declare at least one view, which is what makes this
    /// a whole-specification rule rather than one a component could answer alone: a view lives
    /// inside a domain document, and a component holds only the domain's name.
    ///
    /// [`Reach::Network`] is a promise that something outside issues commands here or reads a row
    /// from here. A component that accepts no command and owns no domain projecting a view has no
    /// route to serve, so the promise is a contract with no operations in it — an endpoint list
    /// nobody can call, published as if it could be. It is refused rather than emitted empty,
    /// because an empty surface is the shape a rename leaves behind and it reads exactly like a
    /// component whose commands have not been written yet.
    pub fn validate_reach(&self, projecting: &BTreeSet<QualifiedName>) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if self.reached_by != Reach::Network {
            return errors;
        }
        if !self.accepts.is_empty() || self.owns.iter().any(|domain| projecting.contains(domain)) {
            return errors;
        }
        errors.push(
            ValidationError::new(
                ValidationCode::EmptyDeclaration,
                format!("component {}", self.name),
                format!(
                    "`{}` is reached over a network and serves nothing: it accepts no command,                      and no domain it owns declares a view",
                    self.name
                ),
            )
            .with_hint(
                "a surface reachable from outside is one somebody calls; give it a command to                  accept or a view to project, or say `reached_by: in_process`",
            ),
        );
        errors
    }

    /// Checks every reference this component makes against what the specification declares.
    ///
    /// Three references, one rule each: a domain it owns, a command it accepts and an event it
    /// publishes each have to name something that exists. A component is the layer a reader — or a
    /// coding agent — goes to for "what am I building, and what talks to it", so a reference with
    /// nothing behind it reads as a work item for something nobody declared. It is usually a
    /// rename that happened on one side only, which is why the hint lists what was available.
    ///
    /// Whether another component has already claimed the same ground is [`validate_components`].
    pub fn validate_references(
        &self,
        domains: &BTreeSet<QualifiedName>,
        commands: &BTreeSet<QualifiedName>,
        events: &BTreeSet<QualifiedName>,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        for (field, verb, kind, plural, declared, referenced) in [
            (
                "owns.domains",
                "owns",
                "a domain",
                "domains",
                domains,
                &self.owns,
            ),
            (
                "accepts.commands",
                "accepts",
                "a command",
                "commands",
                commands,
                &self.accepts,
            ),
            (
                "publishes.events",
                "publishes",
                "an event",
                "events",
                events,
                &self.publishes,
            ),
        ] {
            for name in referenced {
                if declared.contains(name) {
                    continue;
                }
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        format!("component {}.{field}", self.name),
                        format!(
                            "`{}` {verb} `{name}`, which nothing declares as {kind}",
                            self.name
                        ),
                    )
                    .with_hint(available(plural, declared)),
                );
            }
        }
        errors
    }
}

impl ComponentSpec {
    /// Checks that the command tree reaches every view the component's own domains project.
    ///
    /// The half of the tree that cannot be answered without the rest of the specification: a view
    /// is declared inside a domain document, and a component holds only the domain's name.
    ///
    /// # The defect this exists for
    ///
    /// A surface that is served and has no verb is invisible, and nothing reports it. `connectors`
    /// found this the long way round: its personal-local Kubernetes backend answers
    /// `b10x.connector-datasource.v0alpha1` for `kubernetes.workloads`, the daemon serves it, and
    /// the command tree has no verb that reads a datasource at all — so an operator on that machine
    /// cannot reach a projection the process beside them is already publishing. Nobody wrote that
    /// down until somebody noticed. Here it is a refusal at `ess validate`, before a tree is
    /// generated from a declaration that forgot it.
    pub fn validate_command_line_views(
        &self,
        views: &BTreeMap<QualifiedName, BTreeSet<QualifiedName>>,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let Some(cli) = &self.cli else {
            return errors;
        };

        let mut placements: BTreeMap<&QualifiedName, Vec<String>> = BTreeMap::new();
        for view in &cli.views {
            placements
                .entry(view)
                .or_default()
                .push("the top level".to_owned());
        }
        for group in &cli.groups {
            for view in &group.views {
                placements
                    .entry(view)
                    .or_default()
                    .push(format!("`{}`", group.name));
            }
        }

        let owned: BTreeSet<&QualifiedName> = self
            .owns
            .iter()
            .filter_map(|domain| views.get(domain))
            .flatten()
            .collect();

        for (view, places) in &placements {
            if !owned.contains(*view) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::UndeclaredReference,
                        format!("component {}", self.name),
                        format!(
                            "the command tree reads `{view}`, which no domain `{}` owns projects",
                            self.name
                        ),
                    )
                    .with_hint(
                        "a tree can only read a view from a domain the component owns; own that \
                         domain, or drop the view from the tree",
                    ),
                );
            }
            if places.len() > 1 {
                errors.push(
                    ValidationError::new(
                        ValidationCode::DuplicateDeclaration,
                        format!("component {}", self.name),
                        format!(
                            "the command tree reads `{view}` under {}",
                            places.join(" and ")
                        ),
                    )
                    .with_hint("a view has one path; keep the place it belongs and drop the other"),
                );
            }
        }

        for view in owned {
            if !placements.contains_key(view) {
                errors.push(
                    ValidationError::new(
                        ValidationCode::MissingDeclaration,
                        format!("component {}", self.name),
                        format!(
                            "`{}` projects `{view}` and the command tree gives it no place",
                            self.name
                        ),
                    )
                    .with_hint(
                        "give it a verb, or stop projecting it; a view served with no way to \
                         read it is one nobody can reach",
                    ),
                );
            }
        }

        errors
    }
}

/// Everything checkable only against the rest of the specification.
///
/// Every rule in the module table except the empty-component one, which conversion already spent —
/// a [`ComponentSpec`] cannot exist without having passed [`ComponentSpec::validate`], so an empty
/// component never arrives here to be reported twice.
///
/// `domains`, `commands` and `events` are what the whole specification declares, so a component may
/// name anything any domain declares: §5's `email-service` accepts `email.SendEmail` and §7's
/// bindings cross contexts, so resolving a component's references against its own domains only
/// would refuse the design's own example.
///
/// `views` is the catalogue by domain rather than the yes-or-no `projecting` set it used to be. Two
/// questions are asked of it now, not one: whether a network surface has anything to serve, and
/// whether a command tree gives every view it projects a verb — and the second needs the names.
///
/// Errors accumulate. One pass reports every broken reference and every conflict, so a document is
/// fixed once rather than four times.
pub fn validate_components(
    components: &BTreeMap<ComponentName, ComponentSpec>,
    domains: &BTreeSet<QualifiedName>,
    commands: &BTreeSet<QualifiedName>,
    events: &BTreeSet<QualifiedName>,
    views: &BTreeMap<QualifiedName, BTreeSet<QualifiedName>>,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let projecting: BTreeSet<QualifiedName> = views
        .iter()
        .filter(|(_, projected)| !projected.is_empty())
        .map(|(domain, _)| domain.clone())
        .collect();
    for component in components.values() {
        errors.extend(component.validate_references(domains, commands, events));
        errors.extend(component.validate_reach(&projecting));
        errors.extend(component.validate_command_line_views(views));
    }

    let ownership = ownership(components, domains);

    for (domain, owners) in &ownership {
        if owners.len() < 2 {
            continue;
        }
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                format!("domain {domain}"),
                format!(
                    "`{domain}` is owned by more than one component: {}",
                    quoted(owners)
                ),
            )
            .with_hint(
                "a domain has one owning component — §9 gives its state one component that \
                 modifies it; split the domain, or give it to one of them and connect the other \
                 with a binding",
            ),
        );
    }

    for (name, component) in components {
        for command in &component.accepts {
            // A command nothing declares is already reported against this component, and the
            // conflict below is derived from it: it disappears the moment the typo is fixed, so
            // reporting it too would send a reader chasing an ownership problem that is not there.
            if !commands.contains(command) {
                continue;
            }
            let Some((domain, owners)) = owner_of(&ownership, command) else {
                continue;
            };
            if owners.contains(name) {
                continue;
            }
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    format!("component {name}.accepts.commands"),
                    format!(
                        "`{name}` accepts `{command}`, and {} owns `{domain}`",
                        quoted(owners)
                    ),
                )
                .with_hint(format!(
                    "§9 gives a command one handler, and it is the component owning its domain: \
                     either move `{domain}` to `{name}`, or let the owner accept `{command}` and \
                     have `{name}` reach it through a binding"
                )),
            );
        }
    }

    errors
}

/// The three settings rules that need the rest of the specification.
///
/// | refused | code |
/// |---|---|
/// | a setting whose `type` nothing declares | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
/// | a setting typed by an **entity** | [`TypeMismatch`](ValidationCode::TypeMismatch) |
/// | a setting typed by a name whose representation admits absence | [`TypeMismatch`](ValidationCode::TypeMismatch) |
///
/// The second is the load-bearing one, and it is why this is its own rule rather than a missing
/// reference: an entity *is* declared, so resolving the name would succeed and configuration would
/// quietly become a second entity model. A setting has no identity and no lifecycle; an entity has
/// both, and a component that says its configuration is one has said something no projection can
/// carry out — there is nothing to bind into a container.
///
/// # Why the third, and what it makes true
///
/// [`TypeRef::is_optional`] is `matches!(self, Self::Optional(_))`. It answers about the wrapper a
/// document wrote, and it cannot see a name: `newtype of: Optional<String>` is a declared type
/// that admits absence and does not look like one, and this repository already declares that shape
/// (`ess-compiler/tests/fixtures/adversary_expression.yaml`). A setting typed by such a name is on
/// the wrong side of every settings rule at once — [`ComponentSetting::is_required`] answers
/// `true` over a representation that permits absence, the projection derives a required slot for a
/// value the model says may be missing, and `required: false`, which is the author's only way to
/// say what they meant, is refused by a grid that was reading the wrapper.
///
/// Refusing the name here is what makes the wrapper question total: **for every setting in an
/// accepted specification, [`TypeRef::is_optional`] is exactly "this type admits absence"**, at
/// all three sites that ask it — this pass, [`ComponentSetting::is_required`], and the compiled
/// IR's own resolved setting. The alternative, resolving optionality through the registry at each
/// of those sites, cannot be done at two of them: they hold a type *reference* and no registry,
/// and giving them one changes a public signature in three crates so that a question the model
/// already has a spelling for can be asked a second way.
///
/// # Where this runs
///
/// At the compiler's entrance, beside the revalidation of the sealed specification
/// (`ess_compiler::resolve::compile_locating`), rather than inside
/// [`Specification::validate`](crate::spec::Specification::validate). Both hold the two catalogues
/// this needs and either would serve; the entrance is where it is because
/// [`validate_components`] is reached from `Specification::validate` with the domain, command and
/// event names only, and widening that call is an edit to a file this change does not own. Nothing
/// reaches resolution without passing here, so `ess validate` refuses both — but a caller using
/// `ess-domain` alone and stopping at `Specification::assemble` does not see them, and that seam is
/// worth closing when `validate_components` can be given the registry.
pub fn validate_setting_types(specification: &crate::spec::Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let entities: BTreeSet<&QualifiedName> = specification.entities().keys().collect();
    let bodies: BTreeMap<&QualifiedName, &TypeBody> = specification
        .system()
        .types
        .iter()
        .map(|named| (&named.name, &named.body))
        .collect();
    let mut declared: BTreeSet<QualifiedName> = specification
        .system()
        .types
        .iter()
        .map(|named| named.name.clone())
        .collect();
    // The enum an entity's lifecycle forms is a type a setting may name, exactly as a view's
    // projection may: leaving them out would refuse a reference the rest of the compiler accepts,
    // and an author would have no way to tell which of the two passes was wrong.
    for entity in specification.entities().values() {
        declared.insert(entity.state_type().name);
    }

    for component in specification.components().values() {
        for setting in &component.settings {
            let at = || setting_site(component.name.as_str(), setting.name.as_str());
            for name in setting.type_ref.named_dependencies() {
                if entities.contains(name) {
                    errors.push(
                        ValidationError::at(
                            at(),
                            ValidationCode::TypeMismatch,
                            format!(
                                "the setting `{}` is typed `{name}`, which is an entity",
                                setting.name
                            ),
                        )
                        .with_hint(
                            "a setting is a value; an entity is a thing with an identity and a \
                             lifecycle, and configuration has neither. Type the setting with what \
                             the entity's field holds, or declare a type for it",
                        ),
                    );
                } else if !declared.contains(name) {
                    errors.push(
                        ValidationError::at(
                            at(),
                            ValidationCode::UndeclaredReference,
                            format!(
                                "the setting `{}` is typed `{name}`, which nothing declares",
                                setting.name
                            ),
                        )
                        .with_hint(declared_types(&declared)),
                    );
                }
            }
            if let TypeRef::Named(name) = &setting.type_ref {
                if let Some(representation) = absence_admitting_representation(name, &bodies) {
                    errors.push(
                        ValidationError::at(
                            at(),
                            ValidationCode::TypeMismatch,
                            format!(
                                "the setting `{}` is typed `{name}`, whose representation \
                                 `{representation}` says the value may be absent",
                                setting.name
                            ),
                        )
                        .with_hint(format!(
                            "a setting says that with `Optional<…>` and with nothing else, \
                             because the wrapper is what `required:` restates and what the \
                             derived slot's kind is read from: type it `{representation}`, or \
                             use a type that admits no absence"
                        )),
                    );
                }
            }
        }
    }
    errors
}

/// The `Optional<…>` a declared name reaches through its representation, if it reaches one.
///
/// Only [`TypeBody::Newtype`] can: a struct field, a union variant or a map value that is
/// `Optional<…>` says that a *part* of a value may be absent, which is a different statement and
/// is not one a setting's requiredness reads.
///
/// The walk is transitive, because a newtype over a newtype over an `Optional` admits absence just
/// as directly. It is bounded by a visited set rather than by an argument, and the bound is
/// defensive: `A = newtype of B` beside `B = newtype of A` is refused at assembly by the
/// `self_reference` check — measured, not assumed, while writing
/// `every_accepted_setting_answers_the_wrapper_question_the_way_the_registry_does` — so no
/// specification that reaches here carries one. This function is `pub(self)` to a `pub` rule that
/// takes any [`Specification`](crate::spec::Specification), and an unbounded `loop` over a
/// caller-supplied graph is not a thing to leave standing on that argument. A cycle reaches no
/// `Optional`, so `None` is the right answer for one, and the refusal a cycle deserves belongs to
/// the check that already owns it.
fn absence_admitting_representation<'a>(
    name: &'a QualifiedName,
    bodies: &BTreeMap<&'a QualifiedName, &'a TypeBody>,
) -> Option<&'a TypeRef> {
    let mut seen: BTreeSet<&QualifiedName> = BTreeSet::new();
    let mut current = name;
    loop {
        if !seen.insert(current) {
            return None;
        }
        let Some(TypeBody::Newtype { of, .. }) = bodies.get(current) else {
            return None;
        };
        if of.is_optional() {
            return Some(of);
        }
        let TypeRef::Named(next) = of else {
            return None;
        };
        current = next;
    }
}

/// What types were available, which is where a misspelt one shows.
fn declared_types(names: &BTreeSet<QualifiedName>) -> String {
    if names.is_empty() {
        return "no types are declared anywhere in the specification".to_owned();
    }
    format!(
        "declared types: {}",
        names
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Which components claim which domain.
type Ownership<'a> = BTreeMap<&'a QualifiedName, BTreeSet<&'a ComponentName>>;

/// Indexes the ownership claims the components make.
///
/// Only claims on *declared* domains: a component owning a domain nothing declares is already
/// reported as a broken reference, and carrying it further would produce a second error derived from
/// the first — a conflict over a domain that does not exist.
fn ownership<'a>(
    components: &'a BTreeMap<ComponentName, ComponentSpec>,
    domains: &BTreeSet<QualifiedName>,
) -> Ownership<'a> {
    let mut owners: Ownership<'a> = BTreeMap::new();
    for component in components.values() {
        for domain in component.owns.iter().filter(|d| domains.contains(d)) {
            owners.entry(domain).or_default().insert(&component.name);
        }
    }
    owners
}

/// The owned domain `name` sits in, most specific first.
///
/// Most specific rather than first found: `billing` and `billing.invoice` can both be owned, and
/// `billing.invoice.CreateInvoice` is the invoice context's command, not the outer one's.
fn owner_of<'a>(
    ownership: &'a Ownership<'a>,
    name: &QualifiedName,
) -> Option<(&'a QualifiedName, &'a BTreeSet<&'a ComponentName>)> {
    ownership
        .iter()
        .filter(|(domain, _)| name.is_within(domain))
        .max_by_key(|(domain, _)| domain.segments().len())
        .map(|(domain, owners)| (*domain, owners))
}

/// What was available, which is where a misspelling shows.
fn available(plural: &str, names: &BTreeSet<QualifiedName>) -> String {
    if names.is_empty() {
        return format!("no {plural} are declared anywhere in the specification");
    }
    format!(
        "declared {plural}: {}",
        names
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Component names as they appear in a message, in name order.
fn quoted(names: &BTreeSet<&ComponentName>) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(yaml: &str) -> ComponentSpec {
        let raw: RawComponentSpec =
            serde_yaml::from_str(yaml).expect("the document is well formed");
        ComponentSpec::try_from(raw).expect("a component is valid on its own")
    }

    fn catalogue(
        specs: impl IntoIterator<Item = ComponentSpec>,
    ) -> BTreeMap<ComponentName, ComponentSpec> {
        specs
            .into_iter()
            .map(|spec| (spec.name.clone(), spec))
            .collect()
    }

    fn names<'a>(values: impl IntoIterator<Item = &'a str>) -> BTreeSet<QualifiedName> {
        values
            .into_iter()
            .map(|value| QualifiedName::new(value).expect("a valid name"))
            .collect()
    }

    /// §5's two contexts.
    fn domains() -> BTreeSet<QualifiedName> {
        names(["billing.invoice", "billing.email"])
    }

    /// The commands §5's components accept.
    fn commands() -> BTreeSet<QualifiedName> {
        names([
            "billing.invoice.CreateInvoice",
            "billing.invoice.MarkInvoicePaid",
            "billing.email.SendEmail",
        ])
    }

    /// The events §5's components publish.
    fn events() -> BTreeSet<QualifiedName> {
        names([
            "billing.invoice.InvoiceCreated",
            "billing.invoice.InvoicePaid",
            "billing.email.EmailSent",
            "billing.email.EmailFailed",
        ])
    }

    fn invoice_service() -> ComponentSpec {
        component(
            "\
component: invoice-service
owns:
  domains:
    - billing.invoice
accepts:
  commands:
    - billing.invoice.CreateInvoice
    - billing.invoice.MarkInvoicePaid
publishes:
  events:
    - billing.invoice.InvoiceCreated
    - billing.invoice.InvoicePaid
",
        )
    }

    fn email_service() -> ComponentSpec {
        component(
            "\
component: email-service
owns:
  domains:
    - billing.email
accepts:
  commands:
    - billing.email.SendEmail
publishes:
  events:
    - billing.email.EmailSent
    - billing.email.EmailFailed
",
        )
    }

    fn check(components: &BTreeMap<ComponentName, ComponentSpec>) -> ValidationErrors {
        validate_components(
            components,
            &domains(),
            &commands(),
            &events(),
            &view_catalogue(),
        )
    }

    /// The domains that declare a view. `billing.email` declares none, which is what makes the
    /// reach rule's refusal reachable at all.
    fn projecting() -> BTreeSet<QualifiedName> {
        names(["billing.invoice"])
    }

    /// The same fact as [`projecting`], by name rather than by yes-or-no.
    ///
    /// `billing.email` is present and empty on purpose: a domain that projects nothing is a
    /// different statement from a domain nobody declared, and the derived `projecting` set has to
    /// come out the same either way.
    fn view_catalogue() -> BTreeMap<QualifiedName, BTreeSet<QualifiedName>> {
        [
            (
                QualifiedName::new("billing.invoice").expect("a valid name"),
                names(["billing.invoice.OutstandingInvoices"]),
            ),
            (
                QualifiedName::new("billing.email").expect("a valid name"),
                BTreeSet::new(),
            ),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn a_component_says_nothing_about_reach_unless_it_says_something_about_reach() {
        assert_eq!(
            invoice_service().reached_by,
            Reach::InProcess,
            "silence is not a deployment: a component that has not said where its callers are has \
             not said its surface crosses anything"
        );
    }

    #[test]
    fn a_component_reached_over_a_network_that_serves_nothing_is_refused() {
        // The fixture has to reach the state the rule is about: `billing.email` declares no view
        // (see `projecting`), and this component accepts no command, so both halves of the
        // disjunction are false and the refusal is the rule's rather than an accident.
        let component = component(
            "\
component: relay-service
owns:
  domains:
    - billing.email
publishes:
  events:
    - billing.email.EmailSent
reached_by: network
",
        );
        let email = QualifiedName::new("billing.email").expect("a valid name");
        assert!(
            component.accepts.is_empty() && !projecting().contains(&email),
            "the fixture must reach the state where the rule decides anything"
        );
        let errors = check(&catalogue([component]));
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::EmptyDeclaration)
            .expect("a network surface with no route is refused");
        assert!(
            error.message.contains("serves nothing"),
            "the refusal names what is missing: {}",
            error.message
        );
    }

    #[test]
    fn a_component_reached_over_a_network_that_only_projects_a_view_is_accepted() {
        // The read-only half, and the reason the rule is a disjunction rather than "accepts
        // nothing": a component that answers queries and takes no command is a legitimate surface.
        let component = component(
            "\
component: ledger-view
owns:
  domains:
    - billing.invoice
reached_by: network
",
        );
        assert!(
            component.accepts.is_empty(),
            "the fixture only holds if the component accepts nothing"
        );
        let errors = component.validate_reach(&projecting());
        assert!(
            errors.is_empty(),
            "a domain that projects a row is a surface: {errors}"
        );
    }

    #[test]
    fn a_reach_the_model_does_not_declare_is_refused_by_the_word() {
        let refused: Result<RawComponentSpec, _> = serde_yaml::from_str(
            "\
component: invoice-service
owns:
  domains:
    - billing.invoice
reached_by: grpc
",
        );
        let error = refused.expect_err("`grpc` is not one of the two words the model has");
        assert!(
            error.to_string().contains("in_process") && error.to_string().contains("network"),
            "the refusal lists the words that exist: {error}"
        );
    }

    #[test]
    fn the_components_from_the_design_document_validate() {
        let components = catalogue([invoice_service(), email_service()]);
        let errors = check(&components);
        assert!(errors.is_empty(), "{errors}");
    }

    #[test]
    fn a_component_owning_a_domain_nothing_declares_is_refused() {
        let components = catalogue([component(
            "\
component: invoice-service
owns:
  domains:
    - billing.ivoice
",
        )]);
        let errors = check(&components);
        assert_eq!(errors.len(), 1, "{errors}");

        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::UndeclaredReference);
        assert_eq!(error.location, "component invoice-service.owns.domains");
        assert!(
            error.message.contains("owns `billing.ivoice`"),
            "the message names the reference that resolves to nothing: {error}"
        );
        assert!(
            error
                .hint
                .as_deref()
                .unwrap_or_default()
                .contains("billing.invoice"),
            "the hint lists what was available, which is where the typo shows: {error}"
        );
    }

    #[test]
    fn a_component_accepting_a_command_nothing_declares_is_refused() {
        let components = catalogue([component(
            "\
component: invoice-service
owns:
  domains:
    - billing.invoice
accepts:
  commands:
    - billing.invoice.MarkInvoicePayed
",
        )]);
        let errors = check(&components);
        assert_eq!(errors.len(), 1, "{errors}");

        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::UndeclaredReference);
        assert_eq!(error.location, "component invoice-service.accepts.commands");
        assert!(
            error
                .message
                .contains("accepts `billing.invoice.MarkInvoicePayed`"),
            "{error}"
        );
    }

    #[test]
    fn a_component_publishing_an_event_nothing_declares_is_refused() {
        let components = catalogue([component(
            "\
component: invoice-service
owns:
  domains:
    - billing.invoice
publishes:
  events:
    - billing.invoice.InvoiceIssued
",
        )]);
        let errors = check(&components);
        assert_eq!(errors.len(), 1, "{errors}");

        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::UndeclaredReference);
        assert_eq!(error.location, "component invoice-service.publishes.events");
        assert!(
            error
                .message
                .contains("publishes `billing.invoice.InvoiceIssued`"),
            "{error}"
        );
    }

    #[test]
    fn every_reference_that_names_nothing_is_reported_not_just_the_first() {
        let components = catalogue([
            component(
                "\
component: invoice-service
owns:
  domains:
    - billing.invoice
    - billing.legder
accepts:
  commands:
    - billing.invoice.ArchiveInvoice
publishes:
  events:
    - billing.invoice.InvoiceArchived
",
            ),
            component(
                "\
component: email-service
owns:
  domains:
    - billing.emails
",
            ),
        ]);
        let errors = check(&components);
        assert_eq!(errors.len(), 4, "one pass reports all four: {errors}");
        assert!(
            errors
                .as_slice()
                .iter()
                .all(|error| error.code == ValidationCode::UndeclaredReference),
            "{errors}"
        );
        let rendered = errors.to_string();
        for missing in [
            "billing.legder",
            "billing.invoice.ArchiveInvoice",
            "billing.invoice.InvoiceArchived",
            "billing.emails",
        ] {
            assert!(
                rendered.contains(missing),
                "{missing} is missing: {rendered}"
            );
        }
    }

    #[test]
    fn a_component_accepting_a_command_another_component_owns_the_domain_of_is_refused() {
        let components = catalogue([
            invoice_service(),
            component(
                "\
component: email-service
owns:
  domains:
    - billing.email
accepts:
  commands:
    - billing.email.SendEmail
    - billing.invoice.MarkInvoicePaid
",
            ),
        ]);
        let errors = check(&components);
        assert_eq!(errors.len(), 1, "{errors}");

        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::ConflictingDeclaration);
        assert_eq!(error.location, "component email-service.accepts.commands");
        assert!(
            error
                .message
                .contains("accepts `billing.invoice.MarkInvoicePaid`")
                && error
                    .message
                    .contains("`invoice-service` owns `billing.invoice`"),
            "the message names both halves of the conflict, because either one could be the fix: \
             {error}"
        );
    }

    #[test]
    fn a_component_accepting_a_command_from_a_domain_no_component_owns_is_allowed() {
        // Half of the previous test's conflict removed: nothing else claims `billing.invoice`, so
        // nothing contradicts `email-service` handling its commands. §20 asks for undeclared
        // commands to be refused, not for a domain to be owned before its commands have a handler.
        let components = catalogue([component(
            "\
component: email-service
owns:
  domains:
    - billing.email
accepts:
  commands:
    - billing.email.SendEmail
    - billing.invoice.MarkInvoicePaid
",
        )]);
        let errors = check(&components);
        assert!(errors.is_empty(), "{errors}");
    }

    #[test]
    fn two_components_owning_the_same_domain_is_refused() {
        let components = catalogue([
            invoice_service(),
            component(
                "\
component: billing-service
owns:
  domains:
    - billing.invoice
accepts:
  commands:
    - billing.invoice.CreateInvoice
",
            ),
        ]);
        let errors = check(&components);
        assert_eq!(
            errors.len(),
            1,
            "the double claim is the fault; each component accepting its own domain's commands is \
             not a second one: {errors}"
        );

        let error = &errors.as_slice()[0];
        assert_eq!(error.code, ValidationCode::ConflictingDeclaration);
        assert_eq!(error.location, "domain billing.invoice");
        assert!(
            error.message.contains("`billing-service`")
                && error.message.contains("`invoice-service`"),
            "both claimants are named, in name order: {error}"
        );
    }

    #[test]
    fn a_declared_domain_no_component_owns_is_not_an_error() {
        // §5: responsibility is logical and "not yet a deployment decision". A model that has
        // domains and has not been decomposed yet is a state a specification is allowed to be in —
        // otherwise the first component ever written invalidates every domain without one.
        let components = catalogue([invoice_service()]);
        let errors = check(&components);
        assert!(
            errors.is_empty(),
            "`billing.email` is declared and unowned: {errors}"
        );
    }

    #[test]
    fn a_component_publishing_an_event_from_a_domain_it_does_not_own_is_allowed() {
        // Acceptance is a destination and publication is a source: §9 gives a command one handler,
        // so a second claim on it is ambiguous, while nothing downstream of an event asks who
        // published it. §6's outer surface is where a translating adapter lives.
        let components = catalogue([
            invoice_service(),
            component(
                "\
component: email-service
owns:
  domains:
    - billing.email
publishes:
  events:
    - billing.email.EmailSent
    - billing.invoice.InvoicePaid
",
            ),
        ]);
        let errors = check(&components);
        assert!(
            errors.is_empty(),
            "no rule refuses a second publisher: {errors}"
        );
    }

    #[test]
    fn a_misspelt_command_is_one_fault_and_reports_one_error() {
        // `billing.invoice.MarkInvoicePayed` is both undeclared and inside a domain another
        // component owns. The second reading is derived from the first and disappears with the
        // typo, so only the reference is reported.
        let components = catalogue([
            invoice_service(),
            component(
                "\
component: email-service
owns:
  domains:
    - billing.email
accepts:
  commands:
    - billing.invoice.MarkInvoicePayed
",
            ),
        ]);
        let errors = check(&components);
        assert_eq!(errors.len(), 1, "{errors}");
        assert_eq!(
            errors.as_slice()[0].code,
            ValidationCode::UndeclaredReference
        );
        assert!(
            !errors.contains(ValidationCode::ConflictingDeclaration),
            "a conflict over a command nobody declared sends the reader after the wrong repair: \
             {errors}"
        );
    }

    #[test]
    fn two_components_claiming_a_domain_nothing_declares_report_the_reference_and_not_a_conflict() {
        let components = catalogue([
            component(
                "\
component: invoice-service
owns:
  domains:
    - billing.ivoice
",
            ),
            component(
                "\
component: billing-service
owns:
  domains:
    - billing.ivoice
",
            ),
        ]);
        let errors = check(&components);
        assert_eq!(errors.len(), 2, "one per component, and no third: {errors}");
        assert!(
            !errors.contains(ValidationCode::ConflictingDeclaration),
            "a conflict over a domain that does not exist is not the problem to fix: {errors}"
        );
    }

    #[test]
    fn a_command_is_handled_by_the_owner_of_its_innermost_domain() {
        let domains = names(["billing.invoice", "billing.invoice.draft", "billing.email"]);
        let commands = names(["billing.invoice.draft.SubmitDraft"]);
        let components = catalogue([
            component(
                "\
component: invoice-service
owns:
  domains:
    - billing.invoice
",
            ),
            component(
                "\
component: draft-service
owns:
  domains:
    - billing.invoice.draft
accepts:
  commands:
    - billing.invoice.draft.SubmitDraft
",
            ),
        ]);
        let errors = validate_components(
            &components,
            &domains,
            &commands,
            &events(),
            &view_catalogue(),
        );
        assert!(
            errors.is_empty(),
            "`billing.invoice.draft.SubmitDraft` sits in both namespaces, and the inner one owns \
             it: {errors}"
        );
    }

    #[test]
    fn a_component_that_owns_nothing_accepts_nothing_and_publishes_nothing_is_refused() {
        let raw: RawComponentSpec =
            serde_yaml::from_str("component: invoice-service\n").expect("well formed");
        let errors = ComponentSpec::try_from(raw).expect_err("a component that does nothing");
        assert_eq!(errors.len(), 1, "{errors}");
        assert_eq!(errors.as_slice()[0].code, ValidationCode::EmptyDeclaration);
        assert_eq!(errors.as_slice()[0].location, "component invoice-service");
    }

    #[test]
    fn a_component_name_spelt_like_a_type_is_refused() {
        let raw: RawComponentSpec = serde_yaml::from_str(
            "\
component: InvoiceService
owns:
  domains:
    - billing.invoice
",
        )
        .expect("well formed");
        let errors = ComponentSpec::try_from(raw).expect_err("not a component name");
        assert_eq!(errors.len(), 1, "{errors}");
        assert_eq!(errors.as_slice()[0].code, ValidationCode::TypeMismatch);
        assert!(
            errors.to_string().contains("workload name"),
            "the message says what the name becomes, which is why the charset is narrow: {errors}"
        );
    }

    #[test]
    fn a_components_one_line_summary_is_read_from_either_spelling() {
        let spec = component(
            "\
component: invoice-service
summary: Issues invoices and tracks payment.
owns:
  domains:
    - billing.invoice
",
        );
        assert_eq!(
            spec.naming.summary.as_deref(),
            Some("Issues invoices and tracks payment."),
            "a top-level `summary:` is the same statement as `naming.summary`"
        );
    }

    #[test]
    fn a_key_the_model_does_not_know_is_refused() {
        let error = serde_yaml::from_str::<RawComponentSpec>(
            "\
component: invoice-service
own:
  domains:
    - billing.invoice
",
        )
        .expect_err("`own` is nothing");
        assert!(
            error.to_string().contains("own"),
            "a misspelt key would otherwise be an ownership claim that silently does not exist: \
             {error}"
        );
    }

    // ---- the command-line surface --------------------------------------------------------

    /// A written `cli:` block, refused or not, without `try_from`'s `expect`.
    fn command_line(yaml: &str) -> Result<ComponentSpec, ValidationErrors> {
        let raw: RawComponentSpec =
            serde_yaml::from_str(yaml).expect("the document is well formed");
        ComponentSpec::try_from(raw)
    }

    /// The shape every refusal below is one edit away from.
    const CLI: &str = "\
component: connectors-cli
reached_by: command_line
accepts:
  commands:
    - billing.invoice.CreateInvoice
    - billing.email.SendEmail
cli:
  binary: connectors
  commands:
    - billing.invoice.CreateInvoice
  groups:
    - name: inspect
      summary: What is configured, and what cannot work
      commands:
        - billing.email.SendEmail
";

    #[test]
    fn a_command_tree_places_every_accepted_command_exactly_once() {
        let component = command_line(CLI).expect("the tree covers what the component accepts");
        let cli = component.cli.expect("the block is carried through");
        assert_eq!(cli.binary.as_str(), "connectors");
        assert_eq!(cli.groups.len(), 1);
        assert_eq!(cli.groups[0].name.as_str(), "inspect");
        assert_eq!(
            cli.groups[0].summary.as_deref(),
            Some("What is configured, and what cannot work")
        );
    }

    #[test]
    fn a_command_tree_on_a_component_reached_another_way_is_refused() {
        let errors = command_line(&CLI.replace("reached_by: command_line", "reached_by: network"))
            .expect_err("a tree describes a surface a network component does not have");
        assert_eq!(
            errors.as_slice()[0].code,
            ValidationCode::ConflictingDeclaration
        );
    }

    #[test]
    fn a_command_line_surface_without_a_tree_is_refused() {
        let yaml = CLI.split("cli:").next().expect("the block is last");
        let errors =
            command_line(yaml).expect_err("a command-line surface with no tree projects nothing");
        assert_eq!(
            errors.as_slice()[0].code,
            ValidationCode::MissingDeclaration
        );
    }

    #[test]
    fn a_tree_placing_a_command_the_component_does_not_accept_is_refused() {
        let errors = command_line(&CLI.replace(
            "    - billing.invoice.CreateInvoice\n  groups:",
            "    - billing.invoice.MarkInvoicePaid\n  groups:",
        ))
        .expect_err("a tree can only place a command the component accepts");
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::UndeclaredReference),
            "{errors:?}"
        );
    }

    #[test]
    fn a_command_placed_twice_is_refused() {
        let errors = command_line(&CLI.replace(
            "      commands:\n        - billing.email.SendEmail",
            "      commands:\n        - billing.email.SendEmail\n        - \
             billing.invoice.CreateInvoice",
        ))
        .expect_err("a command has one path");
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::DuplicateDeclaration),
            "{errors:?}"
        );
    }

    #[test]
    fn an_accepted_command_the_tree_places_nowhere_is_refused() {
        let errors = command_line(&CLI.replace(
            "  commands:\n    - billing.invoice.CreateInvoice\n  groups:",
            "  groups:",
        ))
        .expect_err("a command with no place is one the generated tree would not carry");
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::MissingDeclaration)
            .unwrap_or_else(|| panic!("{errors:?}"));
        assert!(
            error.message.contains("billing.invoice.CreateInvoice"),
            "the refusal names the command with no place: {}",
            error.message
        );
    }

    #[test]
    fn a_group_that_is_not_a_typed_word_is_refused() {
        let errors = command_line(&CLI.replace("name: inspect", "name: Inspect_It"))
            .expect_err("a group is typed at a shell");
        assert_eq!(errors.as_slice()[0].code, ValidationCode::TypeMismatch);
    }

    /// The old-reader guarantee, stated as a test rather than as a claim in a changelog.
    ///
    /// `ess/AGENTS.md` requires this before any persisted field: a specification that says nothing
    /// about the addition has to serialise exactly as it did before it existed. `cli` is skipped
    /// when unset, so the bytes below carry no such key — and if that ever stops being true, every
    /// committed digest in every consuming repository moves for a statement no author made.
    ///
    /// The digest itself is the IR's, not this type's, and `ess-compiler` skips the field there for
    /// the same reason. This is the half that can be checked without a whole specification.
    #[test]
    fn a_component_saying_nothing_about_a_command_line_serialises_as_it_did_before() {
        let serialised = serde_json::to_string(&invoice_service()).expect("a component serialises");
        assert!(
            !serialised.contains("\"cli\""),
            "an unstated command tree must leave no key behind: {serialised}"
        );
    }

    /// The `connectors` defect, as a test: a projection served with no verb.
    #[test]
    fn a_view_the_command_tree_gives_no_place_is_refused() {
        let component = component(
            "\
component: reader
reached_by: command_line
owns:
  domains:
    - billing.invoice
accepts:
  commands:
    - billing.invoice.CreateInvoice
cli:
  binary: reader
  commands:
    - billing.invoice.CreateInvoice
",
        );
        let errors = component.validate_command_line_views(&view_catalogue());
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::MissingDeclaration)
            .unwrap_or_else(|| panic!("{errors:?}"));
        assert!(
            error
                .message
                .contains("billing.invoice.OutstandingInvoices"),
            "the refusal names the view nobody can read: {}",
            error.message
        );
    }

    #[test]
    fn a_view_the_command_tree_places_is_accepted() {
        let component = component(
            "\
component: reader
reached_by: command_line
owns:
  domains:
    - billing.invoice
accepts:
  commands:
    - billing.invoice.CreateInvoice
cli:
  binary: reader
  commands:
    - billing.invoice.CreateInvoice
  views:
    - billing.invoice.OutstandingInvoices
",
        );
        assert!(
            component
                .validate_command_line_views(&view_catalogue())
                .is_empty(),
            "a view with a verb is reachable"
        );
    }

    #[test]
    fn a_tree_reading_a_view_from_a_domain_it_does_not_own_is_refused() {
        let component = component(
            "\
component: reader
reached_by: command_line
accepts:
  commands:
    - billing.invoice.CreateInvoice
cli:
  binary: reader
  commands:
    - billing.invoice.CreateInvoice
  views:
    - billing.invoice.OutstandingInvoices
",
        );
        let errors = component.validate_command_line_views(&view_catalogue());
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::UndeclaredReference),
            "{errors:?}"
        );
    }
}
