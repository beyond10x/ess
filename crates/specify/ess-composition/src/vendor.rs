//! `ess specify vendor` (beyond10x/ess#663): a consumer's copy of an owner's type, generated
//! rather than written by hand, and the composition entries that prove it against the pinned owner
//! (`docs/design/composition-type-conformance.md`, "Generating the copy").
//!
//! The copy is the named closure of the type: the type and every named type it reaches, each
//! renamed into one consumer domain and kept field for field, with every wire name, presence,
//! invariant, alphabet, prefix and variant attribute. Nothing here changes a source, composition or
//! IR format: the copy is an ordinary source document of the consumer's, and the proof is an
//! ordinary `conformances:` entry per copied type.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::{ResolvedBody, ResolvedType, ResolvedTypeRef};
use ess_compiler::EssIr;
use ess_domain::name::{Naming, QualifiedName};
use ess_domain::types::TypeRef;

use crate::ServiceKey;

/// The named closure of one owner type, renamed into one consumer domain.
#[derive(Debug, Clone)]
pub struct Vendored {
    /// The owner's system, version and IR digest, as the copy's header names them.
    system: QualifiedName,
    version: String,
    digest: String,
    /// The owner's type the closure is taken of.
    root: QualifiedName,
    /// The owner component that owns the domain of every copied type.
    component: String,
    /// The consumer domain the copies are declared in.
    into: QualifiedName,
    /// Each copied owner type with its name in the consumer.
    renamed: BTreeMap<QualifiedName, QualifiedName>,
    /// Each copy's declaration, as the source document writes it, by its name in the consumer.
    blocks: BTreeMap<QualifiedName, String>,
}

/// Takes the closure of `type_name` in `owner` and renames it into `into`.
///
/// # Errors
///
/// A sentence naming what is refused: a type `owner` does not declare; a type whose domain no
/// component of `owner` owns, or a closure spread over domains of two components; two copied types
/// that would take one name in `into`; a type that carries a clock-reading attachment.
pub fn vendor(
    owner: &EssIr,
    type_name: &QualifiedName,
    into: &QualifiedName,
) -> Result<Vendored, String> {
    if !owner.types().contains_key(type_name) {
        return Err(format!(
            "`{type_name}` is not a type `{}` {} declares",
            owner.system(),
            owner.version()
        ));
    }
    let closure = closure(owner, type_name);

    let mut component = None;
    for name in &closure {
        let domain = owner
            .domains()
            .values()
            .find(|domain| domain.types.iter().any(|handle| handle.name() == name))
            .map(|domain| domain.name.clone());
        let owning = domain.as_ref().and_then(|domain| {
            owner
                .components()
                .values()
                .find(|component| component.owns.iter().any(|owned| owned.name() == domain))
                .map(|component| component.name.to_string())
        });
        let Some(owning) = owning else {
            return Err(format!(
                "no component of `{}` owns `{}`, the domain of `{name}`, so no composition can \
                 name it: add the domain to a component's `owns`",
                owner.system(),
                domain.map_or_else(|| name.to_string(), |domain| domain.to_string()),
            ));
        };
        match &component {
            None => component = Some(owning),
            Some(first) if *first != owning => {
                return Err(format!(
                    "`{name}` is owned by component `{owning}` and `{type_name}` by `{first}`: a \
                     copy is proved against one component of `{}`",
                    owner.system()
                ));
            }
            Some(_) => {}
        }
    }

    let mut renamed = BTreeMap::new();
    let mut taken = BTreeMap::new();
    for name in &closure {
        let local = into.child(name.local());
        if let Some(other) = taken.insert(local.clone(), name.clone()) {
            return Err(format!(
                "`{other}` and `{name}` would both be copied as `{local}`"
            ));
        }
        renamed.insert(name.clone(), local);
    }

    let mut blocks = BTreeMap::new();
    for (name, local) in &renamed {
        let declared = &owner.types()[name];
        if declared.reading.is_some() {
            return Err(format!(
                "`{name}` carries a clock-reading attachment, which a copy does not carry"
            ));
        }
        blocks.insert(local.clone(), block(owner, declared, local, &renamed));
    }

    Ok(Vendored {
        system: owner.system().clone(),
        version: owner.version().to_string(),
        digest: owner.source_digest(),
        root: type_name.clone(),
        component: component.unwrap_or_default(),
        into: into.clone(),
        renamed,
        blocks,
    })
}

impl Vendored {
    /// The owner component that owns the domain of every copied type.
    pub fn owner_component(&self) -> &str {
        &self.component
    }

    /// Each copied owner type with its name in the consumer, by owner name.
    pub fn renamed(&self) -> &BTreeMap<QualifiedName, QualifiedName> {
        &self.renamed
    }

    /// Each copy's declaration as the source document writes it, by its name in the consumer.
    pub fn blocks(&self) -> &BTreeMap<QualifiedName, String> {
        &self.blocks
    }

    /// The source document declaring every copy in the consumer domain, headed by a comment naming
    /// the owner's system, version and IR digest.
    pub fn source(&self) -> String {
        let mut out = format!(
            "# Generated by `ess specify vendor` from `{}` of {} {}, IR digest {}.\n\
             # Regenerate it with the same command; `--check` says when the owner has changed.\n\
             domain: {}\n\ntypes:\n",
            self.root,
            self.system,
            self.version,
            self.digest,
            quoted(&self.into.to_string()),
        );
        let blocks = self.blocks.values().cloned().collect::<Vec<_>>();
        out.push_str(&blocks.join("\n"));
        out
    }

    /// The copies in `consumer` (compiled without this copy) that another shape already takes:
    /// a declared type whose declaration is not the copy's. The same declaration is no collision.
    pub fn collisions(&self, consumer: &EssIr) -> Vec<QualifiedName> {
        let identity = BTreeMap::new();
        self.blocks
            .iter()
            .filter(|(local, copy)| {
                consumer
                    .types()
                    .get(*local)
                    .is_some_and(|declared| block(consumer, declared, local, &identity) != **copy)
            })
            .map(|(local, _)| local.clone())
            .collect()
    }

    /// The composition fragment: the `services:` entry of each side, the consumer pinned by
    /// `consumer` (compiled with the copy in it), and one `conformances:` entry per copy.
    pub fn conformances(
        &self,
        owner_key: &ServiceKey,
        consumer_key: &ServiceKey,
        consumer: &EssIr,
        consumer_component: &str,
    ) -> String {
        let service =
            |key: &ServiceKey, system: &str, version: &str, digest: &str, component: &str| {
                format!(
                    "  - {{key: {}, system: {}, version: {}, source_digest: {}, component: {}}}\n",
                    quoted(&key.to_string()),
                    quoted(system),
                    quoted(version),
                    quoted(digest),
                    quoted(component),
                )
            };
        let mut out = String::from("services:\n");
        out.push_str(&service(
            owner_key,
            &self.system.to_string(),
            &self.version,
            &self.digest,
            &self.component,
        ));
        out.push_str(&service(
            consumer_key,
            &consumer.system().to_string(),
            &consumer.version().to_string(),
            &consumer.source_digest(),
            consumer_component,
        ));
        out.push_str("conformances:\n");
        let by_local = self
            .renamed
            .iter()
            .map(|(owner, local)| (local, owner))
            .collect::<BTreeMap<_, _>>();
        for (local, owner) in by_local {
            let _ = write!(
                out,
                "  - local: {{service: {}, type: {}}}\n    conforms_to: {{service: {}, type: {}}}\n",
                quoted(&consumer_key.to_string()),
                quoted(&local.to_string()),
                quoted(&owner_key.to_string()),
                quoted(&owner.to_string()),
            );
        }
        out
    }
}

/// `root` and every named type it reaches in `ir`.
fn closure(ir: &EssIr, root: &QualifiedName) -> BTreeSet<QualifiedName> {
    let mut seen = BTreeSet::new();
    let mut pending = vec![root.clone()];
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(declared) = ir.types().get(&name) else {
            continue;
        };
        let mut reached = Vec::new();
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => reached.extend(of.named_leaves()),
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    reached.extend(field.type_ref.named_leaves());
                }
            }
            ResolvedBody::Union { variants, .. } => {
                for payload in variants.values().flatten() {
                    reached.extend(payload.named_leaves());
                }
            }
            ResolvedBody::Enum { variants } => {
                for attribute in variants.iter().flat_map(|variant| &variant.attributes) {
                    pending.extend(named_in(&attribute.type_ref));
                }
            }
        }
        pending.extend(reached.into_iter().map(|handle| handle.name().clone()));
    }
    seen
}

fn named_in(type_ref: &TypeRef) -> Vec<QualifiedName> {
    match type_ref {
        TypeRef::Primitive(_) => Vec::new(),
        TypeRef::Named(name) => vec![name.clone()],
        TypeRef::Optional(inner) | TypeRef::List(inner) | TypeRef::Map(_, inner) => named_in(inner),
    }
}

/// A YAML double-quoted scalar: JSON's string spelling, which YAML reads as written.
fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| format!("{text:?}"))
}

fn written(type_ref: &ResolvedTypeRef, renamed: &BTreeMap<QualifiedName, QualifiedName>) -> String {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => name.to_string(),
        ResolvedTypeRef::Declared { name } => {
            renamed.get(name.name()).unwrap_or(name.name()).to_string()
        }
        ResolvedTypeRef::Optional { of } => format!("Optional<{}>", written(of, renamed)),
        ResolvedTypeRef::List { of } => format!("List<{}>", written(of, renamed)),
        ResolvedTypeRef::Map { key, value } => format!("Map<{key}, {}>", written(value, renamed)),
    }
}

fn written_ref(type_ref: &TypeRef, renamed: &BTreeMap<QualifiedName, QualifiedName>) -> String {
    match type_ref {
        TypeRef::Primitive(primitive) => primitive.to_string(),
        TypeRef::Named(name) => renamed.get(name).unwrap_or(name).to_string(),
        TypeRef::Optional(inner) => format!("Optional<{}>", written_ref(inner, renamed)),
        TypeRef::List(inner) => format!("List<{}>", written_ref(inner, renamed)),
        TypeRef::Map(key, value) => format!("Map<{key}, {}>", written_ref(value, renamed)),
    }
}

/// The flat naming keys of a field or a variant, each `, key: "value"`.
fn naming_keys(naming: &Naming) -> String {
    let mut out = String::new();
    for (key, value) in [
        ("wire", &naming.wire),
        ("display", &naming.display),
        ("summary", &naming.summary),
        ("code", &naming.code),
    ] {
        if let Some(value) = value {
            let _ = write!(out, ", {key}: {}", quoted(value));
        }
    }
    out
}

/// An invariant as written, every copied owner name in it renamed.
fn statement(text: &str, renamed: &BTreeMap<QualifiedName, QualifiedName>) -> String {
    let mut names = renamed
        .iter()
        .map(|(owner, local)| (owner.to_string(), local.to_string()))
        .collect::<Vec<_>>();
    // The longest first, so a name that is a prefix of another is not renamed inside it.
    names.sort_by(|left, right| right.0.len().cmp(&left.0.len()).then(left.0.cmp(&right.0)));
    names.iter().fold(text.to_owned(), |text, (owner, local)| {
        text.replace(owner, local)
    })
}

/// One struct field's line: its name, type, naming and presence.
fn struct_field(
    field: &ess_compiler::ir::ResolvedField,
    renamed: &BTreeMap<QualifiedName, QualifiedName>,
) -> String {
    let mut out = format!(
        "      - {{name: {}, type: {}{}",
        quoted(&field.name),
        quoted(&written(&field.type_ref, renamed)),
        naming_keys(&field.naming),
    );
    if let Some(presence) = field.naming.presence {
        let presence = serde_json::to_value(presence)
            .ok()
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_default();
        let _ = write!(out, ", presence: {presence}");
    }
    out.push_str("}\n");
    out
}

/// One enum variant's line: its name, naming and attribute values, a text value quoted and a
/// boolean or a number as it was written.
fn enum_variant(variant: &ess_domain::types::EnumVariant) -> String {
    let mut out = format!(
        "      - {{name: {}{}",
        quoted(&variant.name),
        naming_keys(&variant.naming)
    );
    let values = variant
        .attributes
        .iter()
        .filter_map(|attribute| {
            let value = attribute.value.as_ref()?;
            let value = if attribute.written.is_some() {
                value.clone()
            } else {
                quoted(value)
            };
            Some(format!("{}: {value}", quoted(&attribute.name)))
        })
        .collect::<Vec<_>>();
    if !values.is_empty() {
        let _ = write!(out, ", attributes: {{{}}}", values.join(", "));
    }
    out.push_str("}\n");
    out
}

/// One declaration of `declared`, a type of `ir`, named `local`, every reference read through
/// `renamed`.
fn block(
    ir: &EssIr,
    declared: &ResolvedType,
    local: &QualifiedName,
    renamed: &BTreeMap<QualifiedName, QualifiedName>,
) -> String {
    let mut out = format!("  - name: {}\n", quoted(&local.to_string()));
    let invariants = |out: &mut String, invariants: &[ess_domain::entity::Invariant]| {
        if !invariants.is_empty() {
            out.push_str("    invariants:\n");
            for invariant in invariants {
                let _ = writeln!(
                    out,
                    "      - {}",
                    quoted(&statement(&invariant.statement, renamed))
                );
            }
        }
    };
    match &declared.body {
        ResolvedBody::Newtype {
            of,
            alphabet,
            prefix,
            invariants: conditions,
        } => {
            out.push_str("    kind: newtype\n");
            let _ = writeln!(out, "    of: {}", quoted(&written(of, renamed)));
            if let Some(alphabet) = alphabet {
                let _ = writeln!(out, "    alphabet: {}", quoted(alphabet));
            }
            if let Some(prefix) = prefix {
                let _ = writeln!(out, "    prefix: {}", quoted(prefix));
            }
            invariants(&mut out, conditions);
        }
        ResolvedBody::Struct {
            fields,
            invariants: conditions,
        } => {
            out.push_str("    kind: struct\n    fields:\n");
            for field in fields {
                out.push_str(&struct_field(field, renamed));
            }
            invariants(&mut out, conditions);
            if ir.undeclared_fields(&declared.name).is_ignored() {
                out.push_str("    undeclared_fields: ignored\n");
            }
            if let Some(field) = ir.shorthand(&declared.name) {
                let _ = writeln!(out, "    shorthand: {}", quoted(field));
            }
        }
        ResolvedBody::Enum { variants } => {
            out.push_str("    kind: enum\n");
            if declared.ordered {
                out.push_str("    ordered: true\n");
            }
            if let Some(first) = variants
                .first()
                .filter(|first| !first.attributes.is_empty())
            {
                out.push_str("    attributes:\n");
                for attribute in &first.attributes {
                    let _ = writeln!(
                        out,
                        "      - {{name: {}, type: {}}}",
                        quoted(&attribute.name),
                        quoted(&written_ref(&attribute.type_ref, renamed)),
                    );
                }
            }
            out.push_str("    variants:\n");
            for variant in variants {
                out.push_str(&enum_variant(variant));
            }
        }
        ResolvedBody::Union { tag, variants } => {
            out.push_str("    kind: union\n");
            let _ = writeln!(out, "    tag: {}", quoted(tag));
            out.push_str("    variants:\n");
            for (name, payload) in variants {
                let payload = payload.as_ref().map_or_else(
                    || "null".to_owned(),
                    |payload| quoted(&written(payload, renamed)),
                );
                let _ = writeln!(out, "      {}: {payload}", quoted(name));
            }
        }
    }
    let naming = naming_keys(&declared.naming);
    if !naming.is_empty() {
        let _ = writeln!(out, "    naming: {{{}}}", naming.trim_start_matches(", "));
    }
    out
}
