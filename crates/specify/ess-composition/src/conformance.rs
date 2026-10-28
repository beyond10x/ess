//! Type conformance (from `ess-composition/2`): a consumer's local type against an imported one,
//! and the input comparison of an `implements` contract role (`ess-composition/4`).
//!
//! The comparison is structural, because the two ends live in different compiled models and share
//! no handle. Two named types conform when they have the same kind and:
//!
//! * structs have the same field names, each with the same wire name and a conforming type, and
//!   the same `presence` where both ends are `Optional`;
//! * newtypes wrap conforming representations;
//! * enums have the same variant names, each with the same wire spelling;
//! * unions have the same tag and the same variant names, each with a conforming shape.
//!
//! An `ess-composition/5` end may name an event, an error or a command's input or response
//! instead of a type. Its payload is compared as a struct of its fields, against another payload or
//! a struct type, by the struct rule above; a newtype, enum or union meets no payload.
//!
//! Primitives, lists and maps must match exactly. The one tolerance is the one a reader of the
//! imported type can afford: where the imported value is required the local one may be
//! `Optional`, at any position. The reverse, a local value required where the imported one may be
//! absent, is drift. Newtype alphabets, prefixes and invariants, struct invariants and display
//! names are not
//! compared. A pair of named types already on the current path is taken as conforming, so recursive
//! shapes terminate.
//!
//! A reader assertion (`reader: true`, `ess-composition/3`) admits what a consumer that only reads
//! the imported type can also afford, because it rejects no value the imported type allows: an
//! imported newtype read as what it wraps, through any chain; an imported enum read as `String`;
//! enum variants compared by wire name, the local set including every imported one; a value that is
//! always a JSON object (a struct, or a map with `String` keys) read as `Map<String, Json>`; and a
//! subset of a struct's fields. An extra local field is matched by wire name against every imported
//! field, omitted ones included, and compared as a reader when the keys meet; one that meets none
//! must be `Optional` and not `null_when_absent`. Everything else is compared as above. The subset
//! assumes a tolerant reader that ignores keys it does not declare; ESS-generated closed types do
//! not, and that is the author's assertion, not something this comparison can see.

use std::collections::BTreeSet;

use ess_compiler::ir::{ResolvedBody, ResolvedField, ResolvedType, ResolvedTypeRef};
use ess_compiler::EssIr;
use ess_domain::name::QualifiedName;
use ess_domain::types::{EnumVariant, Presence, Primitive};

use crate::{
    Admission, CommandPart, CompositionCode, CompositionDiagnostic, CompositionSpec,
    ConformanceEnd, TypeBinding, TypeConformance,
};

/// Admits both ends of every assertion and compares their shapes.
pub(crate) fn resolve_conformances(
    specification: &CompositionSpec,
    admission: &Admission<'_, '_>,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> BTreeSet<TypeConformance> {
    let mut conformances = BTreeSet::new();
    for asserted in specification.conformances() {
        let local = shape(admission, &asserted.local, diagnostics);
        let imported = shape(admission, &asserted.conforms_to, diagnostics);
        let (Some((local_ir, local)), Some((imported_ir, imported))) = (local, imported) else {
            continue;
        };
        let mut comparison = Comparison {
            local: local_ir,
            imported: imported_ir,
            reader: asserted.reader(),
            visiting: BTreeSet::new(),
            drifts: Vec::new(),
        };
        comparison.shapes(&local, &imported);
        if comparison.drifts.is_empty() {
            conformances.insert(asserted.clone());
        } else {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::TypeConformanceDrift,
                Some(asserted.local.service.clone()),
                format!(
                    "`{}` {} does not conform to `{}` {}: {}",
                    asserted.local.service,
                    asserted.local.end,
                    asserted.conforms_to.service,
                    asserted.conforms_to.end,
                    comparison.drifts.join("; ")
                ),
            ));
        }
    }
    conformances
}

/// What one conformance end compares: a declared type, or a payload's fields.
enum Shape<'ir> {
    Named(&'ir QualifiedName),
    Payload {
        name: &'ir QualifiedName,
        fields: &'ir [ResolvedField],
        /// What the payload is, as a drift names it: `an event payload`.
        what: &'static str,
    },
}

/// Admits one end and finds its shape in its compiled model, recording why when it has none.
fn shape<'ir>(
    admission: &Admission<'_, 'ir>,
    binding: &TypeBinding,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> Option<(&'ir EssIr, Shape<'ir>)> {
    let ir = admission.admit(&binding.service, &binding.end.semantic(), diagnostics)?;
    // Admission resolved the name, so every lookup below finds it.
    let shape = match &binding.end {
        ConformanceEnd::Type(name) => Shape::Named(&ir.types().get(name.name())?.name),
        ConformanceEnd::Event(name) => {
            let event = ir.events().get(name.name())?;
            Shape::Payload {
                name: &event.name,
                fields: &event.fields,
                what: "an event payload",
            }
        }
        ConformanceEnd::Error(name) => {
            let error = ir.errors().get(name.name())?;
            Shape::Payload {
                name: &error.name,
                fields: &error.fields,
                what: "an error payload",
            }
        }
        ConformanceEnd::Command { command, part } => {
            let resolved = ir.commands().get(command.name())?;
            match part {
                CommandPart::Input => Shape::Payload {
                    name: &resolved.name,
                    fields: &resolved.input,
                    what: "a command input",
                },
                CommandPart::Response if resolved.response.is_empty() => {
                    diagnostics.push(CompositionDiagnostic::new(
                        CompositionCode::UnresolvedSemanticReference,
                        Some(binding.service.clone()),
                        format!(
                            "service `{}` command `{command}` declares no `response`, so a \
                             conformance end cannot name its response",
                            binding.service
                        ),
                    ));
                    return None;
                }
                CommandPart::Response => Shape::Payload {
                    name: &resolved.name,
                    fields: &resolved.response,
                    what: "a command response",
                },
            }
        }
    };
    Some((ir, shape))
}

/// Compares a local command's input with a contract command's for an `implements` role entry,
/// returning every drift, each naming its field.
///
/// Every contract input field must be a local input field of the same name, with the same wire
/// name, a conforming type and, where both are `Optional`, the same `presence`: the comparison a
/// type conformance applies to a struct's fields, without reader widening. A local input field the
/// contract does not declare must be `Optional` and not `null_when_absent`, since the contract's
/// caller never sends it.
pub(crate) fn compare_inputs(
    (local, local_command, mine): (&EssIr, &QualifiedName, &[ResolvedField]),
    (contract, contract_command, theirs): (&EssIr, &QualifiedName, &[ResolvedField]),
) -> Vec<String> {
    let mut comparison = Comparison {
        local,
        imported: contract,
        reader: false,
        visiting: BTreeSet::new(),
        drifts: Vec::new(),
    };
    for field in theirs {
        match mine.iter().find(|candidate| candidate.name == field.name) {
            None => comparison.drifts.push(format!(
                "field `{}`: `{contract_command}` sends it and `{local_command}` does not declare it",
                field.name
            )),
            Some(own) => {
                comparison.field_naming(&field.name, own, field, contract_command);
                comparison.references(&field.name, &own.type_ref, &field.type_ref);
            }
        }
    }
    for field in mine {
        if theirs.iter().any(|candidate| candidate.name == field.name) {
            continue;
        }
        if !matches!(field.type_ref, ResolvedTypeRef::Optional { .. }) {
            comparison.drifts.push(format!(
                "field `{}`: `{local_command}` requires it and `{contract_command}` never sends it",
                field.name
            ));
        } else if field.naming.presence == Some(Presence::NullWhenAbsent) {
            comparison.drifts.push(format!(
                "field `{}`: `{local_command}` reads it as {} and `{contract_command}` never \
                 sends it",
                field.name,
                presence(field.naming.presence)
            ));
        }
    }
    comparison.drifts
}

/// One structural walk over a local and an imported model.
struct Comparison<'ir> {
    local: &'ir EssIr,
    imported: &'ir EssIr,
    /// A reader assertion (`ess-composition/3`): the widenings [`Comparison::widened`] names apply.
    reader: bool,
    /// Named-type pairs on the current path, so a recursive shape terminates while a pair reached
    /// through two fields is compared, and reported, at each.
    visiting: BTreeSet<(QualifiedName, QualifiedName)>,
    /// Every difference found, each naming the field path where it sits.
    drifts: Vec<String>,
}

impl Comparison<'_> {
    /// Two conformance ends: types by [`Comparison::named`], and a payload as a struct of its
    /// fields, against another payload or a struct type. No other type meets a payload.
    fn shapes(&mut self, local: &Shape<'_>, imported: &Shape<'_>) {
        if let (Shape::Named(local), Shape::Named(imported)) = (local, imported) {
            return self.named("", local, imported);
        }
        let mine = fields(self.local, local);
        let theirs = fields(self.imported, imported);
        match (mine.fields, theirs.fields) {
            (Some(own), Some(sent)) => self.structs("", (mine.name, own), (theirs.name, sent)),
            _ => self.drifts.push(format!(
                "the type itself: `{}` is {} where `{}` is {}",
                mine.name, mine.kind, theirs.name, theirs.kind
            )),
        }
    }

    fn named(&mut self, path: &str, local: &QualifiedName, imported: &QualifiedName) {
        let pair = (local.clone(), imported.clone());
        if !self.visiting.insert(pair.clone()) {
            return;
        }
        if let (Some(local_type), Some(imported_type)) = (
            self.local.types().get(local),
            self.imported.types().get(imported),
        ) {
            // Both ends were resolved by admission and every nested handle is total.
            self.bodies(path, local_type, imported_type);
        }
        self.visiting.remove(&pair);
    }

    /// A shared field's wire name, and its presence where both ends are `Optional`.
    fn field_naming(
        &mut self,
        at: &str,
        own: &ResolvedField,
        field: &ResolvedField,
        imported: &QualifiedName,
    ) {
        let own_wire = own.naming.wire.as_deref().unwrap_or(&own.name);
        let their_wire = field.naming.wire.as_deref().unwrap_or(&field.name);
        if own_wire != their_wire {
            self.drifts.push(format!(
                "field `{at}`: wire name `{own_wire}` where `{imported}` has `{their_wire}`"
            ));
        }
        if matches!(own.type_ref, ResolvedTypeRef::Optional { .. })
            && matches!(field.type_ref, ResolvedTypeRef::Optional { .. })
            && own.naming.presence != field.naming.presence
        {
            self.drifts.push(format!(
                "field `{at}`: presence {} where `{imported}` has {}",
                presence(own.naming.presence),
                presence(field.naming.presence)
            ));
        }
    }

    /// Two structs' fields, matched by name. A reader may omit an imported field, and may add one
    /// the imported struct lacks when it is `Optional` and not `null_when_absent`.
    fn structs(
        &mut self,
        path: &str,
        (local, mine): (&QualifiedName, &[ResolvedField]),
        (imported, theirs): (&QualifiedName, &[ResolvedField]),
    ) {
        for field in theirs {
            let at = join(path, &field.name);
            match mine.iter().find(|candidate| candidate.name == field.name) {
                // A reader may leave out a field it does not read.
                None if self.reader => {}
                None => self.drifts.push(format!(
                    "field `{at}`: `{imported}` declares it and `{local}` does not"
                )),
                Some(own) => {
                    self.field_naming(&at, own, field, imported);
                    self.references(&at, &own.type_ref, &field.type_ref);
                }
            }
        }
        for field in mine {
            if theirs.iter().any(|candidate| candidate.name == field.name) {
                continue;
            }
            let at = join(path, &field.name);
            // A reader decodes by wire name: a local field that travels as a key the imported
            // struct sends reads that field, whatever either end calls it, omitted or not.
            if self.reader {
                if let Some(sent) = theirs
                    .iter()
                    .find(|candidate| wire_name(candidate) == wire_name(field))
                {
                    self.field_naming(&at, field, sent, imported);
                    self.references(&at, &field.type_ref, &sent.type_ref);
                    continue;
                }
            }
            let optional = matches!(field.type_ref, ResolvedTypeRef::Optional { .. });
            if !(self.reader && optional) {
                self.drifts.push(format!(
                    "field `{at}`: `{local}` declares it and `{imported}` does not"
                ));
            } else if field.naming.presence == Some(Presence::NullWhenAbsent) {
                // The imported type never sends the key; a reader that expects it always sent
                // rejects every value.
                self.drifts.push(format!(
                    "field `{at}`: `{local}` reads it as {} and `{imported}` does not declare it",
                    presence(field.naming.presence)
                ));
            }
        }
    }

    /// A reader's enum against an imported one, by wire name: a reader that knows every wire name
    /// the imported enum sends rejects none of its values, whatever it calls them.
    fn wire_names(
        &mut self,
        path: &str,
        (local, mine): (&QualifiedName, &[EnumVariant]),
        (imported, theirs): (&QualifiedName, &[EnumVariant]),
    ) {
        let known: BTreeSet<_> = mine.iter().map(EnumVariant::wire).collect();
        let unknown: BTreeSet<_> = theirs
            .iter()
            .map(EnumVariant::wire)
            .filter(|wire| !known.contains(wire))
            .map(str::to_owned)
            .collect();
        if !unknown.is_empty() {
            self.drifts.push(format!(
                "{}: `{local}` lacks wire names {} that `{imported}` sends",
                at(path),
                names(&unknown)
            ));
        }
    }

    fn bodies(&mut self, path: &str, local: &ResolvedType, imported: &ResolvedType) {
        match (&local.body, &imported.body) {
            (
                ResolvedBody::Struct { fields: mine, .. },
                ResolvedBody::Struct { fields: theirs, .. },
            ) => self.structs(
                path,
                (&local.name, mine.as_slice()),
                (&imported.name, theirs),
            ),
            (ResolvedBody::Newtype { of: mine, .. }, ResolvedBody::Newtype { of: theirs, .. }) => {
                self.references(path, mine, theirs);
            }
            (ResolvedBody::Enum { variants: mine }, ResolvedBody::Enum { variants: theirs })
                if self.reader =>
            {
                self.wire_names(
                    path,
                    (&local.name, mine.as_slice()),
                    (&imported.name, theirs),
                );
            }
            (ResolvedBody::Enum { variants: mine }, ResolvedBody::Enum { variants: theirs }) => {
                let mine: BTreeSet<_> = mine.iter().map(variant_spelling).collect();
                let theirs: BTreeSet<_> = theirs.iter().map(variant_spelling).collect();
                if mine != theirs {
                    self.drifts.push(format!(
                        "{}: variants {} where `{}` has {}",
                        at(path),
                        names(&mine),
                        imported.name,
                        names(&theirs)
                    ));
                }
            }
            (
                ResolvedBody::Union {
                    tag: my_tag,
                    variants: mine,
                },
                ResolvedBody::Union {
                    tag: their_tag,
                    variants: theirs,
                },
            ) => {
                if my_tag != their_tag {
                    self.drifts.push(format!(
                        "{}: tag `{my_tag}` where `{}` has `{their_tag}`",
                        at(path),
                        imported.name
                    ));
                }
                let my_names: BTreeSet<_> = mine.keys().cloned().collect();
                let their_names: BTreeSet<_> = theirs.keys().cloned().collect();
                if my_names == their_names {
                    for (name, shape) in mine {
                        match (shape, &theirs[name]) {
                            (Some(shape), Some(their_shape)) => {
                                self.references(&join(path, name), shape, their_shape);
                            }
                            (None, None) => {}
                            // A unit variant (ess/22) is the tag alone, so it agrees only with
                            // another unit variant.
                            (mine, theirs) => self.drifts.push(format!(
                                "{}: {} where `{}` {}",
                                at(&join(path, name)),
                                if mine.is_some() {
                                    "a payload"
                                } else {
                                    "no payload"
                                },
                                imported.name,
                                if theirs.is_some() {
                                    "carries one"
                                } else {
                                    "carries none"
                                }
                            )),
                        }
                    }
                } else {
                    self.drifts.push(format!(
                        "{}: variants {} where `{}` has {}",
                        at(path),
                        names(&my_names),
                        imported.name,
                        names(&their_names)
                    ));
                }
            }
            _ => self.drifts.push(format!(
                "{}: `{}` is {} where `{}` is {}",
                at(path),
                local.name,
                kind(&local.body),
                imported.name,
                kind(&imported.body)
            )),
        }
    }

    fn references(&mut self, path: &str, local: &ResolvedTypeRef, imported: &ResolvedTypeRef) {
        match (local, imported) {
            (ResolvedTypeRef::Optional { of: mine }, ResolvedTypeRef::Optional { of: theirs })
            | (ResolvedTypeRef::List { of: mine }, ResolvedTypeRef::List { of: theirs }) => {
                self.references(path, mine, theirs);
            }
            // The tolerance: a reader may treat a required value as optional.
            (ResolvedTypeRef::Optional { of: mine }, theirs) => self.references(path, mine, theirs),
            (
                ResolvedTypeRef::Primitive { name: mine },
                ResolvedTypeRef::Primitive { name: theirs },
            ) if mine == theirs => {}
            // A value that is always a JSON object is read by `Map<String, Json>`.
            (local, imported)
                if self.reader && reads_any_object(local) && self.object_shaped(imported) => {}
            (
                ResolvedTypeRef::Map {
                    key: my_key,
                    value: mine,
                },
                ResolvedTypeRef::Map {
                    key: their_key,
                    value: theirs,
                },
            ) if my_key == their_key => self.references(path, mine, theirs),
            (
                ResolvedTypeRef::Declared { name: mine },
                ResolvedTypeRef::Declared { name: theirs },
            ) => {
                self.named(path, mine.name(), theirs.name());
            }
            _ if self.reader && self.widened(path, local, imported) => {}
            _ => self.drifts.push(format!(
                "{}: `{}` where the imported type has `{}`",
                at(path),
                render(local),
                render(imported)
            )),
        }
    }

    /// A reader's widening of `imported` into a `local` reference that is not a named type, or
    /// `false` when none applies:
    ///
    /// * an imported newtype, through any chain, is read as what it wraps;
    /// * an imported enum is read as `String`.
    ///
    /// `Json` is not widened: it may be an array or a scalar, which `Map<String, Json>` rejects.
    /// A widening that applies but recurses into a drift records it and still returns `true`.
    fn widened(&mut self, path: &str, local: &ResolvedTypeRef, imported: &ResolvedTypeRef) -> bool {
        match imported {
            ResolvedTypeRef::Declared { name } => {
                let Some(declared) = self.imported.types().get(name.name()) else {
                    return false;
                };
                match &declared.body {
                    ResolvedBody::Newtype { of, .. } => {
                        self.references(path, local, of);
                        true
                    }
                    ResolvedBody::Enum { .. } => matches!(
                        local,
                        ResolvedTypeRef::Primitive {
                            name: Primitive::String
                        }
                    ),
                    ResolvedBody::Struct { .. } | ResolvedBody::Union { .. } => false,
                }
            }
            _ => false,
        }
    }

    /// Whether every value of the imported reference is a JSON object: a map with `String` keys,
    /// or a struct, directly or through a chain of newtypes. `Json`, `Optional` (which may be
    /// `null`), lists, enums and unions are not.
    fn object_shaped(&self, imported: &ResolvedTypeRef) -> bool {
        match imported {
            ResolvedTypeRef::Map { key, .. } => *key == Primitive::String,
            ResolvedTypeRef::Declared { name } => {
                match self
                    .imported
                    .types()
                    .get(name.name())
                    .map(|found| &found.body)
                {
                    Some(ResolvedBody::Struct { .. }) => true,
                    Some(ResolvedBody::Newtype { of, .. }) => self.object_shaped(of),
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

/// One end as a struct comparison sees it: its name, its kind as a drift names it, and its fields
/// when it has them (a payload, or a struct type).
struct Compared<'s> {
    name: &'s QualifiedName,
    kind: &'static str,
    fields: Option<&'s [ResolvedField]>,
}

fn fields<'s>(ir: &'s EssIr, shape: &Shape<'s>) -> Compared<'s> {
    match *shape {
        Shape::Payload { name, fields, what } => Compared {
            name,
            kind: what,
            fields: Some(fields),
        },
        Shape::Named(name) => {
            let body = ir.types().get(name).map(|found| &found.body);
            Compared {
                name,
                kind: body.map_or("undeclared", kind),
                fields: match body {
                    Some(ResolvedBody::Struct { fields, .. }) => Some(fields),
                    _ => None,
                },
            }
        }
    }
}

/// `Map<String, Json>`: the reader that accepts every JSON object.
fn reads_any_object(local: &ResolvedTypeRef) -> bool {
    matches!(
        local,
        ResolvedTypeRef::Map { key: Primitive::String, value }
            if matches!(**value, ResolvedTypeRef::Primitive { name: Primitive::Json })
    )
}

/// A field's key on the wire.
fn wire_name(field: &ResolvedField) -> &str {
    field.naming.wire.as_deref().unwrap_or(&field.name)
}

fn join(path: &str, name: &str) -> String {
    if path.is_empty() {
        name.to_owned()
    } else {
        format!("{path}.{name}")
    }
}

/// Where a drift sits: its field path, or the asserted type itself.
fn at(path: &str) -> String {
    if path.is_empty() {
        "the type itself".to_owned()
    } else {
        format!("field `{path}`")
    }
}

fn names(values: &BTreeSet<String>) -> String {
    let listed: Vec<_> = values.iter().map(|value| format!("`{value}`")).collect();
    format!("[{}]", listed.join(", "))
}

fn kind(body: &ResolvedBody) -> &'static str {
    match body {
        ResolvedBody::Newtype { .. } => "a newtype",
        ResolvedBody::Struct { .. } => "a struct",
        ResolvedBody::Enum { .. } => "an enum",
        ResolvedBody::Union { .. } => "a union",
    }
}

fn render(type_ref: &ResolvedTypeRef) -> String {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => name.to_string(),
        ResolvedTypeRef::Declared { name } => name.name().to_string(),
        ResolvedTypeRef::Optional { of } => format!("Optional<{}>", render(of)),
        ResolvedTypeRef::List { of } => format!("List<{}>", render(of)),
        ResolvedTypeRef::Map { key, value } => format!("Map<{key}, {}>", render(value)),
    }
}

/// A variant as a reader sees it: its name, and its wire spelling when that differs.
fn variant_spelling(variant: &EnumVariant) -> String {
    if variant.wire() == variant.name {
        variant.name.clone()
    } else {
        format!("{} (wire `{}`)", variant.name, variant.wire())
    }
}

fn presence(value: Option<Presence>) -> String {
    value.map_or_else(|| "unstated".to_owned(), |value| format!("`{value}`"))
}
