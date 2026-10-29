//! JSON Schema for every message that crosses this system's boundary.
//!
//! A command's input, an event's payload, an error's payload, and every named type the model
//! declares: one draft 2020-12 document each. Roadmap W3.1 asks this projection to prove that the
//! type system is projectable without loss, so the interesting part is not the emitting — it is the
//! handful of places where JSON Schema is weaker than the model and something has to be *decided*
//! rather than translated.
//!
//! Those decisions are **not here**. They live in `types`, the one type mapping all three contract
//! projections call, because this file and `asyncapi.rs` each used to carry a copy and the two copies
//! disagreed about the same event. What is left here is the document layout: which files exist, what
//! each one describes, and where a reference resolves.
//!
//! # One self-contained document per message
//!
//! Every file declares the dialect, describes one thing, and carries under `$defs` exactly the named
//! types it reaches transitively. Nothing refers to another file.
//!
//! The alternative — one shared `types.json`, `$ref`d from every message — would give one canonical
//! definition per named type instead of `billing.invoice.Money` appearing in five files, and that is
//! what a code generator walking the whole tree wants: five copies invite five `Money` classes. It
//! lost anyway, because cross-file `$ref` resolution is where JSON Schema tooling actually fails. It
//! needs a base URI and a retriever, and an artifact tree that validates only when loaded through a
//! correctly configured registry is a tree that does not validate in the field. A file here can be
//! copied on its own into a service repository and handed to any validator, which is how these
//! travel.
//!
//! # Draft 2020-12, where `aep-schema` publishes draft-07
//!
//! `OpenAPI` 3.1's schema dialect *is* JSON Schema 2020-12, and the `OpenAPI` projection embeds these
//! schemas: one dialect means it embeds them rather than translating them, and a translation step
//! would have been a second place for `Map<String, Money>` to come out slightly different. 2020-12
//! also keeps keywords that sit beside a `$ref`, which draft-07 discards — this projection puts a
//! `title` and a field's summary beside almost every reference, and a type document, whose root *is*
//! a `$ref`, would lose its `$defs` and stop resolving at all. `aep-schema`'s draft is not a
//! convention worth matching: it is whatever `schemars` emits, for documents an editor validates,
//! which is a different consumer from a service validating a request body.
//!
//! # Provenance goes in a keyword, not a comment
//!
//! `x-ess-provenance`, at the root of every document, holding design §10's four facts.
//! `Attribution` carries why that is a keyword rather than a `$comment`. The
//! cost is that a linter demanding a closed keyword set will flag it.

// The shared mapping lives at `src/types.rs`, which is the layout this extraction was asked for, and
// it is declared from here rather than from `lib.rs` because `lib.rs` belongs to another change. The
// one-line fix, when that file is free, is to move this declaration into it and drop the attribute.
#[path = "types.rs"]
pub(crate) mod types;

#[path = "model_types.rs"]
mod model_types;
pub use model_types::{ModelTypeError, ModelTypes};

use ess_compiler::ir::{ResolvedEntity, ResolvedType};
use ess_compiler::refs::{
    CommandRef, DeclaredTypeRef, EntityRef, ErrorRef, EssSemanticRef, EventRef,
};
use ess_compiler::EssIr;

use crate::artifact::{Artifact, Generator};
use crate::provenance::ProvenanceMint;
use crate::schema::types::{Attribution, Message, Node};

/// The dialect every emitted document declares.
pub const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Where a tagged union's variant value sits beside its tag, on the wire.
///
/// Public because a wire layout is one decision for every projection of one model, and a second
/// answer to it is a second wire format. This module argues the layout at length — adjacent
/// tagging, `value` unless the tag is itself called `value` — and `ess-synth`'s browser target
/// reads the answer from here rather than re-deriving it, so a JSON document the page posts and a
/// JSON Schema a consumer validates against cannot disagree about where a variant's payload sits.
pub fn union_content_key(tag: &str) -> &'static str {
    types::content_key(tag)
}

/// What a field is called on the wire: its declared `wire:` name, or its own.
///
/// Public for [`union_content_key`]'s reason: the property key a document carries is one decision,
/// and a projection that recomputed it would be a projection that spells a renamed field twice.
pub fn wire_field_name(field: &ess_compiler::ir::ResolvedField) -> &str {
    types::wire_name(field)
}

/// The `pattern` a `String` newtype declaring `alphabet` and no prefix publishes beside its
/// `x-ess-alphabet`, or `None` where no class can state it (beyond10x/ess#592).
///
/// Public for [`union_content_key`]'s reason: `schema-contract` recognises this pattern as the
/// alphabet it already reports, rather than reporting the one constraint twice.
pub fn alphabet_pattern(alphabet: &str) -> Option<String> {
    types::text_pattern(None, Some(alphabet))
}

/// JSON Schema per command input, event payload and error payload.
pub struct JsonSchema;

impl Generator for JsonSchema {
    fn name(&self) -> &'static str {
        "schema"
    }

    fn describes(&self) -> &'static str {
        "JSON Schema per command input, event payload and entity"
    }

    fn directory(&self) -> &'static str {
        "schema"
    }

    fn generate(&self, ir: &EssIr, mint: &ProvenanceMint) -> Vec<Artifact> {
        documents(ir, mint, None)
    }
}

/// [`JsonSchema`], with every document's root identified under a base the adopter chose
/// (`ess generate --kind schema --schema-id-base`, beyond10x/ess#590).
///
/// The `$id` is the base followed by the document's path under `schema/` — `types/<name>.schema.json`,
/// `commands/<name>.schema.json` and so on — so one base gives every document its own identity, and
/// a tree published at the base answers each identity with its document. ESS chooses no namespace:
/// uniqueness, versioning and publication stay the adopter's
/// (`docs/design/review-schema-resource-identity.md`). Without a base no document carries an `$id`,
/// which is why [`JsonSchema`] is unchanged.
#[derive(Debug, Clone)]
pub struct IdentifiedJsonSchema {
    base: String,
}

/// A schema identity base that is not an absolute URI prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIdBaseError {
    /// The base as given.
    pub base: String,
    /// What is wrong with it.
    pub reason: &'static str,
}

impl std::fmt::Display for SchemaIdBaseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "schema id base `{}` {}; give an absolute URI ending in `/` or `:`, such as \
             `https://schemas.example.org/billing/` or `urn:example:billing:`",
            self.base, self.reason
        )
    }
}

impl std::error::Error for SchemaIdBaseError {}

impl IdentifiedJsonSchema {
    /// The projection identified under `base`.
    ///
    /// # Errors
    ///
    /// When `base` does not start with a URI scheme (RFC 3986: a letter, then letters, digits,
    /// `+`, `-` or `.`, then `:`), does not end in `/` or `:`, or holds a `?`, a `#`, whitespace or
    /// a control character. Each of those would make some document's `$id` relative, carry a
    /// query or fragment, or run the base into the path.
    pub fn new(base: &str) -> Result<Self, SchemaIdBaseError> {
        let refuse = |reason| {
            Err(SchemaIdBaseError {
                base: base.to_owned(),
                reason,
            })
        };
        let Some((scheme, rest)) = base.split_once(':') else {
            return refuse("has no URI scheme");
        };
        let mut scheme_bytes = scheme.bytes();
        let scheme_ok = scheme_bytes
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic())
            && scheme_bytes.all(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(&byte));
        if !scheme_ok {
            return refuse("has no URI scheme");
        }
        if base.contains(['?', '#']) {
            return refuse("carries a query or a fragment");
        }
        if base
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        {
            return refuse("holds whitespace or a control character");
        }
        if !base.ends_with(['/', ':']) {
            return refuse("does not end in `/` or `:`");
        }
        // `urn:example` splits as scheme `urn` and `example`, and so ends in neither separator;
        // `urn:` alone names no namespace.
        if rest.is_empty() {
            return refuse("names a scheme and nothing under it");
        }
        Ok(Self {
            base: base.to_owned(),
        })
    }
}

impl Generator for IdentifiedJsonSchema {
    fn name(&self) -> &'static str {
        JsonSchema.name()
    }

    fn describes(&self) -> &'static str {
        JsonSchema.describes()
    }

    fn directory(&self) -> &'static str {
        JsonSchema.directory()
    }

    fn generate(&self, ir: &EssIr, mint: &ProvenanceMint) -> Vec<Artifact> {
        documents(ir, mint, Some(&self.base))
    }
}

/// Every document, identified under `base` when one is given.
fn documents(ir: &EssIr, mint: &ProvenanceMint, base: Option<&str>) -> Vec<Artifact> {
    let identify = |path: &str| base.map(|base| format!("{base}{path}"));
    let mut out = Vec::new();

    // Every named type, not only the ones a message reaches: the union and the enums in the
    // billing example are declared and referenced by an entity, and a projection that dropped
    // them would look complete while the construct this crate most needs to prove — a tagged
    // union that round-trips — appeared in no file at all.
    //
    // Each document's slice is seeded at the one construct it is about. The closure brings in
    // everything the construct rests on — the types a payload reaches, however deep — so the
    // digest moves exactly when something this document could render moves.
    for declared in ir.types().values() {
        out.push(type_document(ir, declared, mint, identify));
    }
    // An entity is not a message and is published anyway: it is the construct a relation is a
    // fact about, and until this document existed the only tool-readable trace of an ownership
    // was a typed id field every reader had to interpret for itself.
    for entity in ir.entities().values() {
        out.push(entity_document(ir, entity, mint, identify));
    }
    for command in ir.commands().values() {
        if !command.response.is_empty() {
            out.push(message_document(
                ir,
                &Message::of_response(command),
                CommandRef::new(command.name.clone()).into(),
                mint,
                identify,
            ));
        }
        out.push(message_document(
            ir,
            &Message::of_command(command),
            CommandRef::new(command.name.clone()).into(),
            mint,
            identify,
        ));
    }
    for event in ir.events().values() {
        out.push(message_document(
            ir,
            &Message::of_event(event),
            EventRef::new(event.name.clone()).into(),
            mint,
            identify,
        ));
    }
    for error in ir.errors().values() {
        out.push(message_document(
            ir,
            &Message::of_error(error),
            ErrorRef::new(error.name.clone()).into(),
            mint,
            identify,
        ));
    }

    out
}

/// The document for one named type.
///
/// The root is a `$ref` into the document's own `$defs` rather than the type's schema inline, so that
/// the shape is the same whether or not the type reaches itself. A struct with a field of its own
/// type is representable in the model, and inlining would have needed a special case for exactly the
/// construct most likely to be got wrong.
fn type_document(
    ir: &EssIr,
    declared: &ResolvedType,
    mint: &ProvenanceMint,
    identify: impl Fn(&str) -> Option<String>,
) -> Artifact {
    // The type itself is inserted by hand rather than used as the root of the walk: a handle has no
    // public constructor, so a projection holding a `ResolvedType` cannot ask for its own handle —
    // and everything the type reaches is reachable from its body.
    let mut defs = types::definitions(ir, types::body_leaves(&declared.body));
    defs.insert(declared.name.to_string(), types::body(ir, declared));

    let path = format!("types/{}.schema.json", declared.name);
    let sliced = mint.of_seeds([DeclaredTypeRef::new(declared.name.clone()).into()]);
    let root = Node {
        dialect: Some(DIALECT),
        id: identify(&path),
        reference: Some(types::pointer(&declared.name)),
        provenance: Some(Attribution::new(&sliced.provenance)),
        defs,
        ..Node::default()
    };

    Artifact::sliced(path, root.to_canonical_json(), sliced.slice)
}

/// The document for one entity: what an instance holds, and what its fields mean.
///
/// The relations carry no `$ref` to their target. A document here is self-contained and keeps under
/// `$defs` exactly the named types it reaches; a pointer at another entity's document would be a
/// cross-file reference, which this projection refuses everywhere else and for the reason the module
/// header gives — a tree that validates only through a correctly configured registry is a tree that
/// does not validate in the field.
fn entity_document(
    ir: &EssIr,
    entity: &ResolvedEntity,
    mint: &ProvenanceMint,
    identify: impl Fn(&str) -> Option<String>,
) -> Artifact {
    let fields = types::entity_fields(entity);
    let carried = Message::of_entity(ir, entity, &fields, types::carried(ir, entity, false));

    let path = format!("entities/{}.schema.json", entity.name);
    let sliced = mint.of_seeds([EntityRef::new(entity.name.clone()).into()]);
    let root = Node {
        dialect: Some(DIALECT),
        id: identify(&path),
        provenance: Some(Attribution::new(&sliced.provenance)),
        defs: types::definitions(ir, types::field_leaves(&fields)),
        ..types::message(&carried)
    };

    Artifact::sliced(path, root.to_canonical_json(), sliced.slice)
}

/// The document for one message.
fn message_document(
    ir: &EssIr,
    carried: &Message<'_>,
    seed: EssSemanticRef,
    mint: &ProvenanceMint,
    identify: impl Fn(&str) -> Option<String>,
) -> Artifact {
    let path = format!("{}/{}.schema.json", carried.directory(), carried.name);
    let sliced = mint.of_seeds([seed]);
    let root = Node {
        one_time_response: if matches!(carried.kind, types::COMMAND_INPUT | types::COMMAND_RESPONSE)
        {
            ir.commands()
                .get(carried.name)
                .map(crate::one_time_response::command)
                .unwrap_or_default()
        } else {
            Vec::new()
        },
        dialect: Some(DIALECT),
        id: identify(&path),
        provenance: Some(Attribution::new(&sliced.provenance)),
        defs: types::definitions(ir, types::field_leaves(carried.fields)),
        ..types::message(carried)
    };

    Artifact::sliced(path, root.to_canonical_json(), sliced.slice)
}
