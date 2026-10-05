//! The server-package emitter: the same transport the Rust target derives, in the second language.
//!
//! # Same derivation, same routes, same statuses
//!
//! Nothing here decides anything the Rust emitter did not already have decided for it. The paths
//! come from [`ess_gen::http::routes`] and the statuses from [`ess_gen::http::status`] — the same
//! two functions the `OpenAPI` projection builds its document from — so the question "do the two
//! synthesised applications serve the same surface" is not answered by comparing two emitters. It
//! is answered by there being one mapping.
//!
//! # What differs from the Rust target, and why
//!
//! | | Rust | Go |
//! |---|---|---|
//! | the HTTP layer | hand-written over `std::net::TcpListener`, about two hundred fixed lines | `net/http`, which is in the standard library and therefore free under the same no-dependency rule |
//! | JSON | the emitted reader and writer the browser bridge already needed | `encoding/json`, with `UseNumber` so an `Integer` past 2^53 survives |
//! | the codecs | generated encoders and decoders over the emitted `json::Value` | generated encoders and decoders over `any`, because the generated types carry unexported fields and `encoding/json` cannot see them |
//!
//! The third row is the one that matters: a Go type whose field is unexported is invisible to
//! `encoding/json`, and exporting them would undo the distinctness the newtype encoding exists for.
//! So the crossing is emitted beside the types, exactly as it is for Rust, and for the same reason
//! design §9 gives: a semantic type knows nothing about a transport.
//!
//! # What it does not decide
//!
//! It chooses no realization. `Serve*` takes the assembled system, and a system over unimplemented
//! obligations answers `501` naming what the plan owes.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedComponent, ResolvedError, ResolvedEvent,
    ResolvedField, ResolvedOutcome, ResolvedType, ResolvedTypeRef, ResolvedView,
};
use ess_domain::component::Reach;
use ess_domain::name::QualifiedName;
use ess_domain::types::Primitive;
use ess_gen::http::{self, Method, Served};
use ess_gen::{Artifact, Provenance};

use crate::plan::{Capability, CapabilityKind, SynthesisPlan};

use super::layout::{Layout, Package};
use super::refusal::TargetRefusals;
use super::{name, Emit};

/// The label every startup line carries.
const LOG_FORMAT: &str = "ess/1";

/// The transport the emitted server speaks, as the startup record names it.
const TRANSPORT: &str = "http/1.1";

/// Every component whose surface the specification says is reached from outside, in name order.
pub(crate) fn served<'a>(ir: &'a EssIr, refusals: &TargetRefusals) -> Vec<&'a ResolvedComponent> {
    ir.components()
        .values()
        .filter(|component| component.reached_by == Reach::Network)
        .filter(|component| {
            !refusals.refuses(&Capability {
                kind: CapabilityKind::ComponentPort,
                source: component.name.to_string(),
            })
        })
        .collect()
}

/// The server package, when any component's surface is served — and nothing at all when none is.
pub(super) fn server_package(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    refusals: &TargetRefusals,
    covered: &mut BTreeSet<Capability>,
) -> Vec<Artifact> {
    let components = served(ir, refusals);
    if components.is_empty() {
        return Vec::new();
    }
    let package = layout.server();
    let provenance = &plan.provenance;
    // The system records what its bindings invoked exactly where it delivers any binding, which is
    // the condition `system.rs` emits `Invocations` and `TakeInvocations` under.
    let records_invocations = ir.bindings().values().any(|binding| {
        let source = binding.name.to_string();
        plan.is_generated(CapabilityKind::BindingDelivery, &source)
            && !refusals.refuses_kind(CapabilityKind::BindingDelivery, &source)
    });
    let mut artifacts = vec![
        helpers_file(ir, layout, refusals, package, provenance),
        wire_file(ir, plan, layout, refusals, package, provenance),
    ];

    // Every served command checks the caller's grant before it runs (beyond10x/ess#265).
    for actor in ir.actors().keys() {
        covered.insert(Capability {
            kind: CapabilityKind::ActorGrants,
            source: actor.to_string(),
        });
    }
    for component in &components {
        covered.insert(Capability {
            kind: CapabilityKind::ComponentTransport,
            source: component.name.to_string(),
        });
        artifacts.push(surface_file(
            ir,
            plan,
            layout,
            package,
            component,
            &components,
            records_invocations,
        ));
        artifacts.push(Artifact::new(
            format!("{}/{}.openapi.json", package.dir, component.name),
            ess_gen::openapi::json(ir, component),
        ));
        artifacts.push(Artifact::new(
            format!("{}/{}.docs.md", package.dir, component.name),
            ess_gen::docs::served(ir, component),
        ));
    }
    artifacts
}

/// The package's own file: what an answer is, how a body is read, and the two media types.
///
/// One file for the package rather than one per served component, because Go would refuse the
/// second declaration of `response` — and because these lines are the same whatever the
/// specification, exactly as the Rust target's `http` module is.
fn helpers_file(
    ir: &EssIr,
    layout: &Layout,
    refusals: &TargetRefusals,
    package: &Package,
    provenance: &Provenance,
) -> Artifact {
    let emit = Emit::new(ir, layout, package, None);
    emit.import("bytes");
    emit.import("encoding/json");
    emit.import("fmt");
    emit.import("io");
    emit.import("net/http");
    emit.import("strconv");
    emit.import("sync");
    let mut helpers = if super::json::used(ir) {
        super::json::with_json(SURFACE_HELPERS, super::json::SURFACE_SUBSTITUTIONS)
    } else {
        SURFACE_HELPERS.to_owned()
    };
    if http::checks_grants(ir) {
        helpers.push_str(&grant_helpers(ir));
        if http::checks_read_grants(ir) {
            helpers.push_str(&read_grant_helpers(ir));
        }
    }
    if serves_params(ir, refusals) {
        emit.import("net/url");
        emit.import("strings");
        emit.import("unicode/utf8");
        helpers.push_str(QUERY_HELPERS);
    }
    emit.file(provenance, SERVER_DOC, &helpers)
}

/// The Go identifier of one declared actor's constant: every segment of its qualified name,
/// pascal-joined, so two domains declaring one local name cannot collide.
fn actor_ident(actor: &QualifiedName) -> String {
    let mut out = String::from("Actor");
    for segment in actor.segments() {
        out.push_str(&name::pascal(segment));
    }
    out
}

/// Every declared actor, its grants as data, the authenticated caller the surface is handed, and
/// the grant check every command route runs first (beyond10x/ess#265).
///
/// In the server package rather than the types: the served surface is what enforces a grant, and
/// the caller type is what the shell that authenticates a request hands it.
fn grant_helpers(ir: &EssIr) -> String {
    let mut out = String::from(
        "\n// Actor is an actor the specification declares, by its qualified name.\ntype Actor \
         string\n\n// Every declared actor.\nconst (\n",
    );
    // Aligned as gofmt aligns a block of one-line specs, so the emitted file is gofmt-clean.
    let width = ir
        .actors()
        .keys()
        .map(|actor| actor_ident(actor).len())
        .max()
        .unwrap_or(0);
    for actor in ir.actors().keys() {
        let _ = writeln!(
            out,
            "\t{:width$} Actor = {:?}",
            actor_ident(actor),
            actor.to_string()
        );
    }
    out.push_str(
        ")\n\n// grants is every declared actor's grants: the qualified names of the commands it \
         may invoke.\nvar grants = map[Actor][]string{\n",
    );
    for (actor, declared) in ir.actors() {
        let commands: Vec<String> = declared
            .may
            .iter()
            .map(|command| format!("{:?}", command.name().to_string()))
            .collect();
        let key = format!("{}:", actor_ident(actor));
        let _ = writeln!(
            out,
            "\t{key:<pad$} {{{}}},",
            commands.join(", "),
            pad = width + 1
        );
    }
    let _ = write!(
        out,
        "}}\n\n// Caller is who a request was authenticated as.\n//\n// Built by whatever \
         authenticates the request — a session, a token, a certificate — and\n// handed to the \
         served surface, which checks its grant before the command runs. Never\n// derived from \
         the request itself: a client can write anything into a request.\ntype Caller struct \
         {{\n\tActor Actor\n}}\n\n// May reports whether the caller may invoke command, named by \
         its qualified name.\nfunc (c Caller) May(command string) bool {{\n\tfor _, granted := \
         range grants[c.Actor] {{\n\t\tif granted == command {{\n\t\t\treturn \
         true\n\t\t}}\n\t}}\n\treturn false\n}}\n\n// Admit reports whether caller may invoke command, checked as every command route \
         checks it before\n// the command runs. Where it may not, it also names the actor the \
         standard refusal names: the\n// caller's declared actor, or \"\" where the request \
         was authenticated as no actor or as\n// an actor the specification does not declare. \
         A command no declared actor may invoke is\n// refused to every caller.\nfunc \
         Admit(caller *Caller, command string) (bool, Actor) {{\n\tif caller == nil {{\n\t\treturn \
         false, \"\"\n\t}}\n\tif caller.May(command) {{\n\t\treturn true, \"\"\n\t}}\n\tif _, \
         declared := grants[caller.Actor]; !declared {{\n\t\treturn false, \"\"\n\t}}\n\treturn \
         false, caller.Actor\n}}\n\n// admit is nil where caller may invoke command, and \
         otherwise the standard refusal the\n// contract declares: {status}, {{\"refused\": \
         \"not granted\", \"actor\": <name or null>}}.\nfunc admit(caller *Caller, command \
         string) *response {{\n\tadmitted, named := Admit(caller, command)\n\tif admitted \
         {{\n\t\treturn nil\n\t}}\n\tvar actor any\n\tif named != \"\" {{\n\t\tactor = \
         string(named)\n\t}}\n\tanswer := rendered({status}, map[string]any{{\"refused\": \
         {not_granted:?}, \"actor\": actor}})\n\treturn &answer\n}}\n",
        status = http::FORBIDDEN,
        not_granted = http::NOT_GRANTED,
    );
    out
}

/// Every declared actor's read grants — the views its `may:` names — and the check every
/// read-granted view's route runs first (beyond10x/ess#286). Emitted only for a model naming a
/// view in a grant, so a model naming none keeps its bytes; a view no actor names is open.
fn read_grant_helpers(ir: &EssIr) -> String {
    let width = ir
        .actors()
        .keys()
        .map(|actor| actor_ident(actor).len())
        .max()
        .unwrap_or(0);
    let mut out = String::from(
        "\n// readGrants is every declared actor's read grants: the qualified names of the views \
         its grant\n// names. A view no actor's grant names is open to every caller.\nvar \
         readGrants = map[Actor][]string{\n",
    );
    for (actor, declared) in ir.actors() {
        let views: Vec<String> = declared
            .may_read
            .iter()
            .map(|view| format!("{:?}", view.name().to_string()))
            .collect();
        let key = format!("{}:", actor_ident(actor));
        let _ = writeln!(
            out,
            "\t{key:<pad$} {{{}}},",
            views.join(", "),
            pad = width + 1
        );
    }
    let _ = write!(
        out,
        "}}\n\n// MayRead reports whether the caller may read view, a view some actor's grant \
         names, named by\n// its qualified name.\nfunc (c Caller) MayRead(view string) bool \
         {{\n\tfor _, granted := range readGrants[c.Actor] {{\n\t\tif granted == view \
         {{\n\t\t\treturn true\n\t\t}}\n\t}}\n\treturn false\n}}\n\n// AdmitRead reports whether \
         caller may read view, a view some actor's grant names, checked\n// as its route checks \
         it before the view is read. Where it may not, it also names the actor the\n// standard \
         refusal names: the caller's declared actor, or \"\" where the request was\n// \
         authenticated as no actor or as an actor the specification does not declare.\nfunc \
         AdmitRead(caller *Caller, view string) (bool, Actor) {{\n\tif caller == nil \
         {{\n\t\treturn false, \"\"\n\t}}\n\tif caller.MayRead(view) {{\n\t\treturn true, \
         \"\"\n\t}}\n\tif _, declared := grants[caller.Actor]; !declared {{\n\t\treturn false, \
         \"\"\n\t}}\n\treturn false, caller.Actor\n}}\n\n// admitRead is nil where caller may read \
         view, and otherwise the standard refusal the\n// contract declares: {status}, \
         {{\"refused\": \"not granted\", \"actor\": <name or null>}}.\nfunc admitRead(caller \
         *Caller, view string) *response {{\n\tadmitted, named := AdmitRead(caller, view)\n\tif \
         admitted {{\n\t\treturn nil\n\t}}\n\tvar actor any\n\tif named != \"\" {{\n\t\tactor = \
         string(named)\n\t}}\n\tanswer := rendered({status}, map[string]any{{\"refused\": \
         {not_granted:?}, \"actor\": actor}})\n\treturn &answer\n}}\n",
        status = http::FORBIDDEN,
        not_granted = http::NOT_GRANTED,
    );
    out
}

// ---- the codecs -------------------------------------------------------------------------------

/// Every generated declaration, as JSON, in both directions.
fn wire_file(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    refusals: &TargetRefusals,
    package: &Package,
    provenance: &Provenance,
) -> Artifact {
    let emit = Emit::new(ir, layout, package, None);
    emit.import("encoding/base64");
    emit.import("encoding/json");
    emit.import("fmt");
    emit.import("strconv");
    emit.import("strings");

    let presents = |kind: CapabilityKind, declared: &QualifiedName| {
        plan.is_generated(kind, &declared.to_string())
            && !refusals.refuses(&Capability {
                kind,
                source: declared.to_string(),
            })
    };

    // A model that uses `Json` reads objects with their member order (see `super::json`).
    let mut body = if super::json::used(ir) {
        emit.import("bytes");
        let mut body = super::json::with_json(WIRE_HELPERS, super::json::WIRE_SUBSTITUTIONS);
        body.push_str(super::json::WIRE);
        body
    } else {
        String::from(WIRE_HELPERS)
    };
    for declared in ir.types().values() {
        if presents(CapabilityKind::DomainType, &declared.name) {
            type_encoder(&mut body, &emit, declared);
            type_decoder(&mut body, &emit, declared);
        }
    }
    for event in ir.events().values() {
        if presents(CapabilityKind::EventType, &event.name) {
            event_encoder(&mut body, &emit, event);
        }
    }
    for error in ir.errors().values() {
        if presents(CapabilityKind::ErrorType, &error.name) {
            error_encoder(&mut body, &emit, error);
        }
    }
    for view in ir.views().values() {
        if presents(CapabilityKind::ViewType, &view.name) {
            view_encoder(&mut body, &emit, view);
            if !view.params.is_empty() {
                params_decoder(&mut body, &emit, view);
            }
        }
    }
    for command in ir.commands().values() {
        if presents(CapabilityKind::CommandContract, &command.name) {
            command_decoder(&mut body, &emit, command);
            response_encoder(&mut body, &emit, command);
        }
    }
    emit.file_at(format!("{}/wire.go", package.dir), provenance, "", &body)
}

/// The function-name fragment of a declaration: its whole qualified name, pascal-cased.
///
/// The whole name and never the local one, for the reason the Rust wire gives: two declarations in
/// two contexts can share a last segment, and a codec that named only that would silently be one
/// function. Pascal-casing still drops separators (`renewal.input.AB` and `renewal.input.A_B` are
/// both `RenewalInputAB`), so the fragment is the layout's allocated codec stem, which suffixes
/// the later of two such declarations rather than declaring one function twice
/// (`crate::codec_names`). The lower-case helpers below never start with `encode`/`decode` plus
/// an upper-case letter, so the two families cannot collide.
fn ident<'a>(layout: &'a Layout, declared: &QualifiedName) -> &'a str {
    layout.codec(declared)
}

/// One declared type, written.
fn type_encoder(out: &mut String, emit: &Emit<'_>, declared: &ResolvedType) {
    let layout = emit.layout;
    let go = emit.reference(&declared.name);
    let function = format!("encode{}", ident(layout, &declared.name));
    let _ = write!(
        out,
        "\n// {function} writes `{}` as JSON.\nfunc {function}(value {go}) any {{\n",
        declared.name
    );
    match &declared.body {
        ResolvedBody::Newtype { of, .. } => {
            let mut slot = 0;
            let expression = encode_into(out, layout, "\t", "value.Value()", of, &mut slot);
            let _ = writeln!(out, "\treturn {expression}");
        }
        ResolvedBody::Struct { fields, .. } => {
            out.push_str("\tout := map[string]any{}\n");
            let mut slot = 0;
            for field in fields {
                encode_member(
                    out,
                    layout,
                    "\t",
                    ess_gen::schema::wire_field_name(field),
                    &format!("value.{}", super::items::member_ident(fields, &field.name)),
                    &field.type_ref,
                    &mut slot,
                );
            }
            out.push_str("\treturn out\n");
        }
        ResolvedBody::Enum { variants } => {
            out.push_str("\tswitch value.(type) {\n");
            for variant in variants {
                let _ = writeln!(
                    out,
                    "\tcase {}:\n\t\treturn {:?}",
                    emit.reference_variant(&declared.name, variant),
                    variant.wire()
                );
            }
            out.push_str(UNREACHABLE_VARIANT);
        }
        ResolvedBody::Union { tag, variants } => {
            let content = ess_gen::schema::union_content_key(tag);
            out.push_str("\tswitch shape := value.(type) {\n");
            for (label, payload) in variants {
                let _ = writeln!(
                    out,
                    "\tcase {}:",
                    emit.reference_variant(&declared.name, label)
                );
                let _ = writeln!(out, "\t\tout := map[string]any{{}}");
                let _ = writeln!(out, "\t\tout[{tag:?}] = {label:?}");
                let mut slot = 0;
                encode_member(
                    out,
                    layout,
                    "\t\t",
                    content,
                    "shape.Value",
                    payload,
                    &mut slot,
                );
                out.push_str("\t\treturn out\n");
            }
            out.push_str(UNREACHABLE_SHAPE);
        }
    }
    out.push_str("}\n");
}

/// One declared type, read.
fn type_decoder(out: &mut String, emit: &Emit<'_>, declared: &ResolvedType) {
    let go = emit.reference(&declared.name);
    let function = format!("decode{}", ident(emit.layout, &declared.name));
    let _ = write!(
        out,
        "\n// {function} reads `{}` from JSON, or refuses at the path it was reached \
         at.\nfunc {function}(value any, at string) ({go}, error) {{\n\tvar out {go}\n",
        declared.name
    );
    match &declared.body {
        ResolvedBody::Newtype { of, .. } => {
            let mut slot = 0;
            let held = decode_into(out, emit, "\t", "value", "at", of, &mut slot);
            let _ = writeln!(
                out,
                "\treturn {}({held}), nil",
                emit.reference_ctor(&declared.name)
            );
        }
        ResolvedBody::Struct { fields, .. } => {
            let _ = writeln!(
                out,
                "\tif _, err := objectAt(value, at, \"an object\"); err != nil {{\n\t\treturn out, \
                 err\n\t}}"
            );
            let mut slot = 0;
            for field in fields {
                decode_member(
                    out,
                    emit,
                    "\t",
                    &format!("out.{}", super::items::member_ident(fields, &field.name)),
                    field,
                    &mut slot,
                );
            }
            out.push_str("\treturn out, nil\n");
        }
        ResolvedBody::Enum { variants } => {
            let expected = variant_list(
                &variants
                    .iter()
                    .map(|variant| variant.wire().to_owned())
                    .collect::<Vec<_>>(),
            );
            let _ = writeln!(
                out,
                "\ttext, err := textAt(value, at, {expected:?})\n\tif err != nil {{\n\t\treturn \
                 out, err\n\t}}\n\tswitch text {{"
            );
            for variant in variants {
                let _ = writeln!(
                    out,
                    "\tcase {:?}:\n\t\treturn {}{{}}, nil",
                    variant.wire(),
                    emit.reference_variant(&declared.name, variant)
                );
            }
            let _ = writeln!(
                out,
                "\t}}\n\treturn out, DecodeError{{At: at, Expected: {expected:?}, Found: \
                 fmt.Sprintf(\"`%s`\", text)}}"
            );
        }
        ResolvedBody::Union { tag, variants } => {
            let content = ess_gen::schema::union_content_key(tag);
            let labels: Vec<String> = variants.keys().cloned().collect();
            let expected = variant_list(&labels);
            let _ = writeln!(
                out,
                "\ttagged, tagAt, err := required(value, at, {tag:?})\n\tif err != nil {{\n\t\t\
                 return out, err\n\t}}\n\tlabel, err := textAt(tagged, tagAt, \
                 {expected:?})\n\tif err != nil {{\n\t\treturn out, err\n\t}}\n\tswitch label {{"
            );
            for (label, payload) in variants {
                let _ = writeln!(out, "\tcase {label:?}:");
                let carried = ResolvedField {
                    name: content.to_owned(),
                    type_ref: payload.clone(),
                    naming: ess_domain::name::Naming::default(),
                };
                let mut slot = 0;
                decode_member(out, emit, "\t\t", "", &carried, &mut slot);
                let _ = writeln!(
                    out,
                    "\t\treturn {}{{Value: shape}}, nil",
                    emit.reference_variant(&declared.name, label)
                );
            }
            let _ = writeln!(
                out,
                "\t}}\n\treturn out, DecodeError{{At: tagAt, Expected: {expected:?}, Found: \
                 fmt.Sprintf(\"`%s`\", label)}}"
            );
        }
    }
    out.push_str("}\n");
}

/// The set of legal spellings, as one phrase a refusal can carry.
fn variant_list<T: AsRef<str>>(variants: &[T]) -> String {
    format!(
        "one of {}",
        variants
            .iter()
            .map(|variant| format!("`{}`", variant.as_ref()))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// One event's encoder, for the `published` list a command's answer carries.
fn event_encoder(out: &mut String, emit: &Emit<'_>, event: &ResolvedEvent) {
    record_encoder(
        out,
        emit.layout,
        &format!("encodeEvent{}", ident(emit.layout, &event.name)),
        &emit.reference(&event.name),
        &format!("the event `{}`", event.name),
        &event.fields,
    );
}

/// One declared error's encoder.
fn error_encoder(out: &mut String, emit: &Emit<'_>, error: &ResolvedError) {
    record_encoder(
        out,
        emit.layout,
        &format!("encodeError{}", ident(emit.layout, &error.name)),
        &emit.reference(&error.name),
        &format!("the declared error `{}`", error.name),
        &error.fields,
    );
}

/// One view row's encoder.
fn view_encoder(out: &mut String, emit: &Emit<'_>, view: &ResolvedView) {
    record_encoder(
        out,
        emit.layout,
        &format!("encodeView{}", ident(emit.layout, &view.name)),
        &emit.reference(&view.name),
        &format!("one row of the view `{}`", view.name),
        &view.fields,
    );
}

/// An encoder over a fixed list of fields.
fn record_encoder(
    out: &mut String,
    layout: &Layout,
    function: &str,
    go: &str,
    describes: &str,
    fields: &[ResolvedField],
) {
    let _ = write!(
        out,
        "\n// {function} writes {describes} as JSON.\nfunc {function}(value {go}) any \
         {{\n\tout := map[string]any{{}}\n"
    );
    let mut slot = 0;
    for field in fields {
        encode_member(
            out,
            layout,
            "\t",
            ess_gen::schema::wire_field_name(field),
            &format!("value.{}", super::items::member_ident(fields, &field.name)),
            &field.type_ref,
            &mut slot,
        );
    }
    out.push_str("\treturn out\n}\n");
    if fields.is_empty() {
        // A declaration with no fields is an empty struct here, so the parameter is never read.
        // Named `_` rather than silenced, because Go has no attribute for it and an unread
        // parameter is not an error — leaving it named would just be misleading.
        *out = out.replace(
            &format!("func {function}(value {go}) any {{"),
            &format!("func {function}(_ {go}) any {{"),
        );
    }
}

/// One command input's decoder.
fn command_decoder(out: &mut String, emit: &Emit<'_>, command: &ResolvedCommand) {
    let go = emit.reference(&command.name);
    let function = format!("decodeCommand{}", ident(emit.layout, &command.name));
    let _ = write!(
        out,
        "\n// {function} reads the input of `{}` from JSON.\nfunc {function}(value any, at string) \
         ({go}, error) {{\n\tvar out {go}\n\tif _, err := objectAt(value, at, \"an object\"); err \
         != nil {{\n\t\treturn out, err\n\t}}\n",
        command.name
    );
    let mut slot = 0;
    for field in &command.input {
        decode_member(
            out,
            emit,
            "\t",
            &format!(
                "out.{}",
                super::items::member_ident(&command.input, &field.name)
            ),
            field,
            &mut slot,
        );
    }
    out.push_str("\treturn out, nil\n}\n");
}

// ---- the two walkers ---------------------------------------------------------------------------

/// One member of an object being written — or nothing at all where an optional value is absent.
fn encode_member(
    out: &mut String,
    layout: &Layout,
    indent: &str,
    wire: &str,
    source: &str,
    type_ref: &ResolvedTypeRef,
    slot: &mut usize,
) {
    if let ResolvedTypeRef::Optional { of } = type_ref {
        let held = format!("held{}", next(slot));
        let _ = writeln!(out, "{indent}if {source} != nil {{");
        let _ = writeln!(out, "{indent}\t{held} := *{source}");
        let inner = format!("{indent}\t");
        let expression = encode_into(out, layout, &inner, &held, of, slot);
        let _ = writeln!(out, "{indent}\tout[{wire:?}] = {expression}");
        let _ = writeln!(out, "{indent}}}");
        return;
    }
    let expression = encode_into(out, layout, indent, source, type_ref, slot);
    let _ = writeln!(out, "{indent}out[{wire:?}] = {expression}");
}

/// Emits whatever statements one value needs and returns the expression that is its JSON.
fn encode_into(
    out: &mut String,
    layout: &Layout,
    indent: &str,
    source: &str,
    type_ref: &ResolvedTypeRef,
    slot: &mut usize,
) -> String {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => encode_primitive(*name, source),
        ResolvedTypeRef::Declared { name } => {
            format!("encode{}({source})", ident(layout, name.name()))
        }
        // An absent optional inside a list or a map is `null`, which is the one place this
        // rendering differs from an absent *member*: a member is omitted, because that is what the
        // published contract's `required` list says, and a hole in an array has no such spelling.
        ResolvedTypeRef::Optional { of } => {
            let target = format!("held{}", next(slot));
            let _ = writeln!(out, "{indent}var {target} any");
            let _ = writeln!(out, "{indent}if {source} != nil {{");
            let inner = format!("{indent}\t");
            let held = format!("{target}Some");
            let _ = writeln!(out, "{indent}\t{held} := *{source}");
            let expression = encode_into(out, layout, &inner, &held, of, slot);
            let _ = writeln!(out, "{indent}\t{target} = {expression}");
            let _ = writeln!(out, "{indent}}}");
            target
        }
        ResolvedTypeRef::List { of } => {
            let target = format!("items{}", next(slot));
            let _ = writeln!(
                out,
                "{indent}{target} := make([]any, 0, len({source}))\n{indent}for _, element := \
                 range {source} {{"
            );
            let inner = format!("{indent}\t");
            let expression = encode_into(out, layout, &inner, "element", of, slot);
            let _ = writeln!(out, "{indent}\t{target} = append({target}, {expression})");
            let _ = writeln!(out, "{indent}}}");
            target
        }
        ResolvedTypeRef::Map { key, value } => {
            let target = format!("entries{}", next(slot));
            let _ = writeln!(
                out,
                "{indent}{target} := map[string]any{{}}\n{indent}for key, element := range \
                 {source} {{"
            );
            let inner = format!("{indent}\t");
            let expression = encode_into(out, layout, &inner, "element", value, slot);
            let _ = writeln!(
                out,
                "{indent}\t{target}[{}] = {expression}",
                encode_key(*key, "key")
            );
            let _ = writeln!(out, "{indent}}}");
            target
        }
    }
}

/// One primitive, in the rendering the published contracts fix.
fn encode_primitive(primitive: Primitive, source: &str) -> String {
    match primitive {
        Primitive::Binary64 => unreachable!("Binary64 is refused before target rendering"),
        Primitive::Json => {
            // Embedded as it is spelled: members in order, numbers as written (beyond10x/ess#224).
            format!("json.RawMessage({source}.Value())")
        }
        Primitive::String | Primitive::Boolean | Primitive::Integer => source.to_owned(),
        Primitive::Bytes => format!("base64.StdEncoding.EncodeToString({source})"),
        Primitive::Decimal | Primitive::Timestamp | Primitive::Duration | Primitive::Uuid => {
            format!("{source}.Value()")
        }
    }
}

/// One map key, as the text a JSON object key has to be.
fn encode_key(primitive: Primitive, source: &str) -> String {
    match primitive {
        Primitive::Binary64 => unreachable!("Binary64 is refused before target rendering"),
        Primitive::Json => unreachable!("ess-domain refuses a Json map key"),
        Primitive::String => source.to_owned(),
        Primitive::Boolean => format!("strconv.FormatBool({source})"),
        Primitive::Integer => format!("strconv.FormatInt({source}, 10)"),
        Primitive::Bytes => format!("base64.StdEncoding.EncodeToString({source})"),
        Primitive::Decimal | Primitive::Timestamp | Primitive::Duration | Primitive::Uuid => {
            format!("{source}.Value()")
        }
    }
}

/// One member of an object being read, assigned to `target` — or left alone when it is optional
/// and absent. An empty `target` binds `shape` instead, which is what a union variant needs.
fn decode_member(
    out: &mut String,
    emit: &Emit<'_>,
    indent: &str,
    target: &str,
    field: &ResolvedField,
    slot: &mut usize,
) {
    let wire = ess_gen::schema::wire_field_name(field);
    let position = next(slot);
    let member = format!("member{position}");
    let at = format!("at{position}");
    // An optional union payload must remain in the case scope even when absent.
    let target = if target.is_empty() && matches!(&field.type_ref, ResolvedTypeRef::Optional { .. })
    {
        let _ = writeln!(out, "{indent}var shape {}", emit.go_type(&field.type_ref));
        "shape"
    } else {
        target
    };
    let assign = |out: &mut String, indent: &str, expression: &str| {
        if target.is_empty() {
            let _ = writeln!(out, "{indent}shape := {expression}");
        } else {
            let _ = writeln!(out, "{indent}{target} = {expression}");
        }
    };

    if let ResolvedTypeRef::Optional { of } = &field.type_ref {
        let found = format!("found{position}");
        let _ = writeln!(
            out,
            "{indent}{member}, {at}, {found}, err := optional(value, at, {wire:?})\n{indent}if err \
             != nil {{\n{indent}\treturn out, err\n{indent}}}\n{indent}if {found} {{"
        );
        let inner = format!("{indent}\t");
        let held = decode_into(out, emit, &inner, &member, &at, of, slot);
        let holder = format!("some{position}");
        let _ = writeln!(out, "{indent}\t{holder} := {held}");
        assign(out, &inner, &format!("&{holder}"));
        let _ = writeln!(out, "{indent}}}");
        return;
    }

    let _ = writeln!(
        out,
        "{indent}{member}, {at}, err := required(value, at, {wire:?})\n{indent}if err != nil \
         {{\n{indent}\treturn out, err\n{indent}}}"
    );
    let held = decode_into(out, emit, indent, &member, &at, &field.type_ref, slot);
    assign(out, indent, &held);
}

/// Emits whatever statements one value needs and returns the variable holding the decoded value.
fn decode_into(
    out: &mut String,
    emit: &Emit<'_>,
    indent: &str,
    source: &str,
    at: &str,
    type_ref: &ResolvedTypeRef,
    slot: &mut usize,
) -> String {
    let position = next(slot);
    let held = format!("held{position}");
    match type_ref {
        ResolvedTypeRef::Primitive { name } => {
            let (helper, expected) = decode_primitive(*name);
            let _ = writeln!(
                out,
                "{indent}{held}, err := {helper}({source}, {at}, {expected:?})\n{indent}if err != \
                 nil {{\n{indent}\treturn out, err\n{indent}}}"
            );
            match name {
                Primitive::Decimal
                | Primitive::Timestamp
                | Primitive::Duration
                | Primitive::Uuid
                | Primitive::Json => {
                    format!("{}({held})", emit.primitive_ctor(*name))
                }
                _ => held,
            }
        }
        ResolvedTypeRef::Declared { name } => {
            let _ = writeln!(
                out,
                "{indent}{held}, err := decode{}({source}, {at})\n{indent}if err != nil \
                 {{\n{indent}\treturn out, err\n{indent}}}",
                ident(emit.layout, name.name())
            );
            held
        }
        // Inside a list or a map, `null` is the absent value: a member that is absent is handled by
        // `decode_member`, and this arm is the one an array element reaches.
        ResolvedTypeRef::Optional { of } => {
            let go = emit.go_type(of);
            let _ = writeln!(out, "{indent}var {held} *{go}");
            let _ = writeln!(out, "{indent}if {source} != nil {{");
            let inner = format!("{indent}\t");
            let inner_held = decode_into(out, emit, &inner, source, at, of, slot);
            let holder = format!("{held}Some");
            let _ = writeln!(
                out,
                "{indent}\t{holder} := {inner_held}\n{indent}\t{held} = &{holder}"
            );
            let _ = writeln!(out, "{indent}}}");
            held
        }
        ResolvedTypeRef::List { of } => {
            let go = emit.go_type(of);
            let items = format!("items{position}");
            let _ = writeln!(
                out,
                "{indent}{items}, err := itemsAt({source}, {at}, \"an array\")\n{indent}if err != \
                 nil {{\n{indent}\treturn out, err\n{indent}}}\n{indent}{held} := \
                 make([]{go}, 0, len({items}))\n{indent}for index, element := range {items} \
                 {{\n{indent}\telementAt := indexed({at}, index)"
            );
            let inner = format!("{indent}\t");
            let inner_held = decode_into(out, emit, &inner, "element", "elementAt", of, slot);
            let _ = writeln!(out, "{indent}\t{held} = append({held}, {inner_held})");
            let _ = writeln!(out, "{indent}}}");
            held
        }
        ResolvedTypeRef::Map { key, value } => {
            let key_go = emit.primitive_type(*key);
            let value_go = emit.go_type(value);
            let entries = format!("entries{position}");
            let _ = writeln!(
                out,
                "{indent}{entries}, err := objectAt({source}, {at}, \"an object\")\n{indent}if err \
                 != nil {{\n{indent}\treturn out, err\n{indent}}}\n{indent}{held} := \
                 make(map[{key_go}]{value_go}, len({entries}))\n{indent}for key, element := range \
                 {entries} {{\n{indent}\tentryAt := nested({at}, key)"
            );
            let inner = format!("{indent}\t");
            let decoded_key = decode_key(out, emit, &inner, *key, "key", "entryAt", slot);
            let inner_held = decode_into(out, emit, &inner, "element", "entryAt", value, slot);
            let _ = writeln!(out, "{indent}\t{held}[{decoded_key}] = {inner_held}");
            let _ = writeln!(out, "{indent}}}");
            held
        }
    }
}

/// The helper that reads one primitive, and what a refusal says belongs there — in the Rust
/// target's words (`rust::wire::decode_primitive`), so the two servers refuse one value alike.
fn decode_primitive(primitive: Primitive) -> (&'static str, &'static str) {
    match primitive {
        Primitive::Binary64 => unreachable!("Binary64 is refused before target rendering"),
        Primitive::Json => ("jsonAt", "any JSON value"),
        Primitive::String => ("textAt", "a string"),
        Primitive::Boolean => ("boolAt", "a boolean"),
        Primitive::Integer => ("integerAt", "an integer"),
        Primitive::Bytes => ("bytesAt", "base64-encoded bytes"),
        Primitive::Decimal => ("decimalAt", "a decimal string"),
        Primitive::Timestamp => ("textAt", "an RFC 3339 instant"),
        Primitive::Duration => ("textAt", "an ISO 8601 duration"),
        Primitive::Uuid => ("uuidAt", "a UUID"),
    }
}

/// One map key, read back out of the text a JSON object key is.
fn decode_key(
    out: &mut String,
    emit: &Emit<'_>,
    indent: &str,
    primitive: Primitive,
    source: &str,
    at: &str,
    slot: &mut usize,
) -> String {
    let held = format!("key{}", next(slot));
    match primitive {
        Primitive::Binary64 => unreachable!("Binary64 is refused before target rendering"),
        Primitive::Json => unreachable!("ess-domain refuses a Json map key"),
        Primitive::String => return source.to_owned(),
        Primitive::Boolean => {
            let _ = writeln!(
                out,
                "{indent}{held}, err := keyBool({source}, {at})\n{indent}if err != nil \
                 {{\n{indent}\treturn out, err\n{indent}}}"
            );
        }
        Primitive::Integer => {
            let _ = writeln!(
                out,
                "{indent}{held}, err := keyInteger({source}, {at})\n{indent}if err != nil \
                 {{\n{indent}\treturn out, err\n{indent}}}"
            );
        }
        Primitive::Bytes => {
            let _ = writeln!(
                out,
                "{indent}{held}, err := keyBytes({source}, {at})\n{indent}if err != nil \
                 {{\n{indent}\treturn out, err\n{indent}}}"
            );
        }
        Primitive::Decimal | Primitive::Timestamp | Primitive::Duration | Primitive::Uuid => {
            let _ = writeln!(
                out,
                "{indent}{held} := {}({source})",
                emit.primitive_ctor(primitive)
            );
        }
    }
    held
}

/// The next slot number, so two generated variables never share a name.
fn next(slot: &mut usize) -> usize {
    let position = *slot;
    *slot += 1;
    position
}

// ---- the surface ------------------------------------------------------------------------------

/// One served component: its routes, its startup record, its handlers and its listener.
fn surface_file(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    package: &Package,
    component: &ResolvedComponent,
    served: &[&ResolvedComponent],
    records_invocations: bool,
) -> Artifact {
    let provenance = &plan.provenance;
    let emit = Emit::new(ir, layout, package, None);
    emit.import_blank("embed");
    emit.import("encoding/json");
    emit.import("fmt");
    emit.import("net");
    emit.import("net/http");
    let system = emit.qualify(layout.system(), "System");
    let routes = http::routes(ir, component);
    let rows = table(&routes, ir);
    let exported = name::exported(&component.name.to_string());

    let mut body = String::new();
    documents(&mut body, component, &exported);
    route_table(&mut body, &rows, &exported);
    startup(&mut body, ir, plan, component, &rows, served, &exported);

    let grants = http::checks_grants(ir);
    let _ = write!(
        body,
        "\n// Serve{exported} serves `{}` at address, and does not return while it can \
         answer.\n//\n// address may name port 0, which binds an ephemeral port; the startup \
         record says which one\n// was taken, because a caller that cannot learn the port cannot \
         make a request.\n//\n// It chooses no realization. Every command reaches the port, and a \
         port over unimplemented\n// obligations answers the typed refusal this surface reports as \
         501.{}\nfunc Serve{exported}(system *{system}, address string{}) error {{\n\tlistener, \
         err := net.Listen(\"tcp\", address)\n\tif err != nil {{\n\t\treturn err\n\t}}\n\tbound, \
         ok := listener.Addr().(*net.TCPAddr)\n\tif !ok {{\n\t\treturn fmt.Errorf(\"the listener \
         bound something that is not a TCP address\")\n\t}}\n\tannounce{exported}(bound)\n\treturn \
         http.Serve(listener, http.HandlerFunc(func(writer http.ResponseWriter, request \
         *http.Request) {{\n\t\tanswer := dispatch{exported}(system, {}request)\n\t\t\
         answer.write(writer)\n\t}}))\n}}\n",
        component.name,
        if grants {
            "\n//\n// authenticate is the realization's: it says who each request was sent by, or \
             nil, and every\n// command checks that caller's grant before it runs. Nothing here \
             reads an actor from the\n// request itself."
        } else {
            ""
        },
        if grants {
            ", authenticate func(*http.Request) *Caller"
        } else {
            ""
        },
        if grants {
            "authenticate(request), "
        } else {
            ""
        },
    );

    serve_static(&mut body, &system, &exported, grants);

    dispatch(&mut body, ir, layout, &routes, &rows, &system, &exported);

    for route in &routes {
        match route.serves {
            Served::Command(handle) => {
                command_handler(
                    &mut body,
                    &emit,
                    layout,
                    component,
                    ir.command(handle),
                    &system,
                    records_invocations,
                );
            }
            Served::View(handle) => {
                view_handler(
                    &mut body,
                    &emit,
                    layout,
                    component,
                    ir.view(handle),
                    &system,
                );
            }
        }
    }

    emit.file_at(
        format!("{}/{}.go", package.dir, module_stem(component)),
        provenance,
        &format!(
            "// The `{}` component of `{}` {}, on the wire.\n//\n// The specification says this \
             component's callers are not deployed with it, so its surface\n// exists on a wire. \
             Which wire is derived rather than chosen: the one contract this model\n// projects \
             for a command surface is the OpenAPI document, and an OpenAPI document is an\n// HTTP \
             contract. The document is beside this file, served verbatim at \
             `/openapi.json`.\n",
            component.name,
            ir.system(),
            ir.version()
        ),
        &body,
    )
}

fn serve_static(body: &mut String, system: &str, exported: &str, grants: bool) {
    let authenticate = if grants {
        ", authenticate func(*http.Request) *Caller"
    } else {
        ""
    };
    let caller = if grants {
        "authenticate(request), "
    } else {
        ""
    };
    let _ = writeln!(
        body,
        r#"
// Serve{exported}WithStatic adds files only for paths outside this surface's route table.
func Serve{exported}WithStatic(system *{system}, address string{authenticate}, staticRoot string) error {{
	root, err := memoryStaticRoot(staticRoot)
	if err != nil {{
		return err
	}}
	listener, err := net.Listen("tcp", address)
	if err != nil {{
		return err
	}}
	bound := listener.Addr().(*net.TCPAddr)
	announce{exported}(bound)
	return http.Serve(listener, http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {{
		known := false
		for _, route := range Routes{exported} {{
			if route[1] == request.URL.Path {{
				known = true
				break
			}}
		}}
		if !known && root != "" {{
			memoryStatic(writer, request, root)
			return
		}}
		answer := dispatch{exported}(system, {caller}request)
		answer.write(writer)
	}}))
}}"#
    );
}

/// The route match: one arm per path, and one arm for everything else.
fn dispatch(
    body: &mut String,
    ir: &EssIr,
    layout: &Layout,
    routes: &[http::Route<'_>],
    rows: &[Row],
    system: &str,
    exported: &str,
) {
    let grants = http::checks_grants(ir);
    let (grant_doc, caller) = if grants {
        (
            "\n//\n// caller is who the realization authenticated the request as, or nil. Every \
             command checks\n// its grant before it runs, and answers the standard refusal when \
             the caller is nil or is\n// an actor the specification does not grant the command.",
            "caller *Caller, ",
        )
    } else {
        ("", "")
    };
    let _ = write!(
        body,
        "\n// dispatch{exported} answers one request.\n//\n// A path this table does not hold is a \
         404 naming where the whole table is published; a path\n// it holds under a different \
         method is a 405 naming the one it answers. Neither is a status\n// the contract declares, \
         and neither should be: both are facts about a transport rather than\n// about any \
         command.{grant_doc}\nfunc dispatch{exported}(system *{system}, {caller}request \
         *http.Request) response {{\n\tif refused := tooManyHeaders(request); refused != nil {{\n\t\treturn *refused\n\t}}\n\tbody, refused := readBody(request)\n\tif refused != nil {{\n\t\treturn \
         *refused\n\t}}\n\t// Held from the port call through Pump and TakePublished: the system \
         is shared by\n\t// every connection.\n\tserving.Lock()\n\tdefer \
         serving.Unlock()\n\tswitch request.URL.Path {{\n"
    );
    for (method, path, _, _) in rows {
        let _ = writeln!(
            body,
            "\tcase {path:?}:\n\t\tif request.Method != {method:?} {{\n\t\t\treturn \
             methodNotAllowed({method:?})\n\t\t}}"
        );
        if path == http::OPENAPI {
            let _ = writeln!(
                body,
                "\t\treturn response{{status: 200, contentType: mediaJSON, body: \
                 openapi{exported}}}"
            );
        } else if path == http::DOCS {
            let _ = writeln!(
                body,
                "\t\treturn response{{status: 200, contentType: mediaMarkdown, body: \
                 docs{exported}}}"
            );
        } else {
            let route = routes
                .iter()
                .find(|route| &route.path == path)
                .expect("every non-document row of the table is a route");
            match route.serves {
                Served::Command(handle) => {
                    if grants {
                        let _ = writeln!(
                            body,
                            "\t\tif refused := admit(caller, {:?}); refused != nil {{\n\t\t\treturn \
                             *refused\n\t\t}}",
                            ir.command(handle).name.to_string()
                        );
                    }
                    let _ = writeln!(
                        body,
                        "\t\treturn serve{}(system, body)",
                        ident(layout, &ir.command(handle).name)
                    );
                }
                Served::View(handle) => {
                    let view = ir.view(handle);
                    // A read-granted view checks the reader first (beyond10x/ess#286).
                    if http::read_checked(ir, &view.name) {
                        let _ = writeln!(
                            body,
                            "\t\tif refused := admitRead(caller, {:?}); refused != nil {{\n\t\t\treturn \
                             *refused\n\t\t}}",
                            view.name.to_string()
                        );
                    }
                    let _ = writeln!(
                        body,
                        "\t\treturn serve{}(system{})",
                        ident(layout, &view.name),
                        if view.params.is_empty() {
                            ""
                        } else {
                            ", request.URL.RawQuery"
                        }
                    );
                }
            }
        }
    }
    let _ = write!(
        body,
        "\t}}\n\treturn refusal(404, fmt.Sprintf(\"`%s` is not a path this surface declares; `GET \
         /openapi.json` publishes every one that is\", request.URL.Path))\n}}\n"
    );
}

/// The two documents this surface publishes about itself, embedded from the files beside it.
fn documents(body: &mut String, component: &ResolvedComponent, exported: &str) {
    let _ = write!(
        body,
        "\n// The contract this surface answers and the prose the same model produced, byte for \
         byte as\n// `generated/` commits them. Embedded rather than rebuilt at run time: a server \
         that\n// regenerated its own contract could publish one the repository never \
         reviewed.\n//\n//go:embed {0}.openapi.json\nvar openapi{exported} string\n\n//go:embed \
         {0}.docs.md\nvar docs{exported} string\n",
        component.name
    );
}

/// Every route, as a table the startup record and the reader both read.
fn route_table(body: &mut String, rows: &[Row], exported: &str) {
    let _ = write!(
        body,
        "\n// Routes{exported} is every route this surface answers, in path order.\n//\n// The \
         same set the OpenAPI document declares, plus the two documents about the surface \
         itself,\n// which no specification construct names and nothing can therefore derive. A \
         path absent from\n// this table is answered with 404, including one the document declares \
         and this table forgot.\nvar Routes{exported} = [][2]string{{\n"
    );
    for (method, path, _, _) in rows {
        let _ = writeln!(body, "\t{{{method:?}, {path:?}}},");
    }
    body.push_str("}\n");
}

/// The three startup lines, and the function that closes each with this process's own facts.
fn startup(
    body: &mut String,
    ir: &EssIr,
    plan: &SynthesisPlan,
    component: &ResolvedComponent,
    rows: &[Row],
    served: &[&ResolvedComponent],
    exported: &str,
) {
    let lines = startup_lines(ir, plan, component, rows, served);
    let _ = write!(
        body,
        "\n// Startup{exported} is what this process says about itself as it starts.\n//\n// Three \
         lines of JSON on standard output, in this order, every member of them derived from the\n\
         // specification — except `runtime`, which is appended below and holds what is true of \
         *this\n// process*: the language it was synthesised into, and the address it bound. \
         Everything outside\n// `runtime` is the same in every language this plan is emitted into, \
         and `cargo xtask synth\n// --check` starts both and compares \
         them.\nvar Startup{exported} = []string{{\n"
    );
    for line in &lines {
        let _ = writeln!(body, "\t{line:?},");
    }
    body.push_str("}\n");

    let _ = write!(
        body,
        "\n// announce{exported} writes the startup record, with this process's own facts closing \
         each line.\nfunc announce{exported}(address *net.TCPAddr) {{\n\tfor _, facts := range \
         Startup{exported} {{\n\t\truntime, err := json.Marshal(map[string]any{{\"address\": \
         address.String(), \"language\": \"go\", \"port\": address.Port}})\n\t\tif err != nil \
         {{\n\t\t\tcontinue\n\t\t}}\n\t\t\
         fmt.Printf(\"%s,\\\"runtime\\\":%s}}\\n\", facts, runtime)\n\t}}\n}}\n"
    );
}

/// One row of the surface table: `(method, path, what it serves, the construct's name)`.
type Row = (&'static str, String, &'static str, String);

/// The file one component's surface is emitted into, named after the component.
fn module_stem(component: &ResolvedComponent) -> String {
    name::package_ident(&component.name.to_string())
}

/// The whole surface as rows of `(method, path, what it serves, the construct's name)`.
fn table<'a>(routes: &'a [http::Route<'a>], ir: &'a EssIr) -> Vec<Row> {
    let mut rows: Vec<Row> = vec![
        (
            Method::Get.as_str(),
            http::DOCS.to_owned(),
            "documentation",
            "docs".to_owned(),
        ),
        (
            Method::Get.as_str(),
            http::OPENAPI.to_owned(),
            "contract",
            "openapi".to_owned(),
        ),
    ];
    for route in routes {
        rows.push(match route.serves {
            Served::Command(handle) => (
                route.method.as_str(),
                route.path.clone(),
                "command",
                ir.command(handle).name.to_string(),
            ),
            Served::View(handle) => (
                route.method.as_str(),
                route.path.clone(),
                "view",
                ir.view(handle).name.to_string(),
            ),
        });
    }
    rows.sort_by(|left, right| left.1.cmp(&right.1).then(left.0.cmp(right.0)));
    rows
}

/// The three startup lines, as JSON text without the closing brace of each.
///
/// Built by the same code path the Rust emitter uses — [`crate::rust::http::startup_facts`] — so
/// the two languages cannot disagree about a member. What each appends is its own `runtime`.
fn startup_lines(
    ir: &EssIr,
    plan: &SynthesisPlan,
    component: &ResolvedComponent,
    rows: &[Row],
    served: &[&ResolvedComponent],
) -> Vec<String> {
    crate::rust::http::startup_facts(
        ir,
        plan,
        component,
        rows,
        served.len(),
        LOG_FORMAT,
        TRANSPORT,
    )
}

/// The `published` member of one branch's answer: every event the branch carries, in publication
/// order, as `{"event": <qualified name>, "payload": {…}}` — the list the Rust surface and the
/// contract spell the same way.
fn published_list(out: &mut String, emit: &Emit<'_>, outcome: &ResolvedOutcome) {
    let carried = super::items::outcome_event_fields(emit, outcome);
    out.push_str("\t\tbody[\"published\"] = []any{");
    for field in &carried {
        let _ = write!(
            out,
            "\n\t\t\tmap[string]any{{\"event\": {:?}, \"payload\": encodeEvent{}(taken.{})}},",
            field.event.name().to_string(),
            ident(emit.layout, field.event.name()),
            field.field
        );
    }
    out.push_str(if carried.is_empty() {
        "}\n"
    } else {
        "\n\t\t}\n"
    });
}

/// The `response` member of a branch that answers its caller with the command's response
/// (`returns: true`, from `ess/22`, beyond10x/ess#423): the variant's own `Response`, through the
/// wire file's encoder. Nothing for any other branch.
fn direct_response(
    out: &mut String,
    emit: &Emit<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) {
    if http::answers_with_response(emit.ir, outcome) {
        let _ = writeln!(
            out,
            "\t\tbody[\"response\"] = {}(taken.Response)",
            response_encoder_name(emit.layout, &command.name)
        );
    }
}

/// The end of one declared branch's answer: its `response` member where it answers with one, and
/// the status the contract declares for it ([`http::outcome_status`]).
fn answered(
    out: &mut String,
    emit: &Emit<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) {
    direct_response(out, emit, command, outcome);
    let _ = writeln!(
        out,
        "\t\treturn rendered({}, body)",
        http::outcome_status(emit.ir, outcome)
    );
}

/// The name of the wire file's encoder for a command's declared response.
fn response_encoder_name(layout: &Layout, command: &QualifiedName) -> String {
    format!("encodeResponse{}", ident(layout, command))
}

/// A command's declared response, for the `response` member of a branch that answers with it.
/// Only for a command with such a branch, so every other wire file keeps its bytes.
fn response_encoder(out: &mut String, emit: &Emit<'_>, command: &ResolvedCommand) {
    let answers = command
        .outcomes
        .iter()
        .any(|outcome| http::answers_with_response(emit.ir, outcome));
    if !answers {
        return;
    }
    record_encoder(
        out,
        emit.layout,
        &response_encoder_name(emit.layout, &command.name),
        &emit.qualify(
            emit.layout.package_of(&command.name),
            emit.layout.response(&command.name),
        ),
        &format!("the response of `{}`", command.name),
        &command.response,
    );
}

/// One accepted command: body in, declared outcome out, at the status the contract publishes.
fn command_handler(
    out: &mut String,
    emit: &Emit<'_>,
    layout: &Layout,
    component: &ResolvedComponent,
    command: &ResolvedCommand,
    system: &str,
    records_invocations: bool,
) {
    let function = ident(emit.layout, &command.name);
    let field = name::exported(&component.name.to_string());
    let method = layout.declared(&command.name);
    let read = if ess_gen::http::body_required(command) {
        "\tvalue, refused := readJSON(body)\n\tif refused != nil {\n\t\treturn *refused\n\t}\n"
            .to_owned()
    } else {
        emit.import("bytes");
        "\t// The contract declares no required body for this command, so a request without one \
         is\n\t// its input with nothing in it.\n\tvar value any = map[string]any{}\n\tif \
         len(bytes.TrimSpace(body)) != 0 {\n\t\tread, refused := readJSON(body)\n\t\tif refused \
         != nil {\n\t\t\treturn *refused\n\t\t}\n\t\tvalue = read\n\t}\n"
            .to_owned()
    };
    let _ = write!(
        out,
        "\n// serve{function} answers `POST` `{}`: reads the declared input, runs the port, \
         answers the\n// declared outcome.\nfunc serve{function}(system *{system}, body []byte) \
         response {{\n{read}\tinput, err := decodeCommand{function}(value, \"body\")\n\tif err != nil \
         {{\n\t\t// 400 and not 422: this is a body the schema decides, which is the \
         difference\n\t\t// between fixing a value and fixing a serialiser.\n\t\treturn \
         refusal(400, err.Error())\n\t}}\n\toutcome, unmet := \
         system.{field}.{method}(input)\n\tif unmet != nil {{\n\t\treturn \
         unfinished(unmet.Error(), false)\n\t}}\n\t// Deliver what this command published to every binding that reacts to it, then \
         take it\n\t// off the log: a long-running server keeps nothing from one request to the \
         next. Pump answers\n\t// only for what this command published, and returns with all of \
         it delivered.\n\tfailure := system.Pump()\n\tsystem.TakePublished()\n{invocations}\tif \
         failure != nil {{\n\t\treturn unfinished(\"delivering what the command published: \
         \"+failure.Error(), true)\n\t}}\n\treturn answer{function}(outcome)\n}}\n",
        command.name,
        invocations = if records_invocations {
            "\tsystem.TakeInvocations()\n"
        } else {
            ""
        },
    );

    let outcome_type = emit.reference_outcome(&command.name);
    let _ = write!(
        out,
        "\n// answer{function} renders one declared outcome of `{}` as the contract publishes it: \
         the\n// branch that was taken, every event it published in publication order, the \
         declared\n// error where there is one, and that error's own payload.\nfunc \
         answer{function}(outcome {outcome_type}) response {{\n\tbody := \
         map[string]any{{}}\n\tswitch taken := outcome.(type) {{\n",
        command.name
    );
    for outcome in &command.outcomes {
        let _ = writeln!(
            out,
            "\tcase {}:",
            emit.reference_outcome_variant(&command.name, outcome.name.as_str())
        );
        let _ = writeln!(out, "\t\tbody[\"outcome\"] = {:?}", outcome.name.as_str());
        published_list(out, emit, outcome);
        if let Some(handle) = &outcome.error {
            let declared = emit.ir.error(handle);
            let _ = writeln!(
                out,
                "\t\tbody[\"error\"] = {}",
                serde_json::to_string(&declared.wire_code()).expect("a string always serializes")
            );
            if declared.fields.is_empty() {
                out.push_str("\t\t_ = taken\n");
            } else {
                let _ = writeln!(
                    out,
                    "\t\tbody[\"payload\"] = encodeError{}(taken.Error)",
                    ident(emit.layout, &declared.name)
                );
            }
        } else {
            out.push_str("\t\t_ = taken\n");
        }
        answered(out, emit, command, outcome);
    }
    if let Some(declared) = ess_gen::unknown_instance::unknown_instance_answer(emit.ir, command) {
        // The declared branch, status and error, nothing published, and no payload: an instance
        // that does not exist has nothing for the error's fields to describe
        // (`docs/design/unknown-instance-seams.md`).
        let error = emit.ir.error(
            declared
                .error
                .as_ref()
                .expect("an unknown-instance answer reports its declared error"),
        );
        let _ = writeln!(
            out,
            "\tcase {}:\n\t\tbody[\"outcome\"] = {:?}\n\t\tbody[\"published\"] = \
             []any{{}}\n\t\tbody[\"error\"] = {}\n\t\treturn rendered({}, body)",
            emit.reference_unknown_instance_variant(&command.name),
            declared.name.as_str(),
            serde_json::to_string(&error.wire_code()).expect("a string always serializes"),
            http::status(declared)
        );
    }
    out.push_str(
        "\t}\n\t// Go cannot check that a switch over a sealed interface is total, which is this \
         target's\n\t// standing weakening (see TARGET.md). An outcome no branch above named is a \
         value no\n\t// generated code can construct, and it is reported rather than \
         dropped.\n\treturn refusal(500, \"the port answered an outcome this surface has no \
         branch for\")\n}\n",
    );
}

/// One declared view: the rows the projection holds, under the key the contract declares.
fn view_handler(
    out: &mut String,
    emit: &Emit<'_>,
    layout: &Layout,
    component: &ResolvedComponent,
    view: &ResolvedView,
    system: &str,
) {
    let _ = emit;
    let function = ident(emit.layout, &view.name);
    let field = name::exported(&component.name.to_string());
    let method = layout.declared(&view.name);
    let (doc, parameter, decoded, arguments) = if view.params.is_empty() {
        (
            "every row the owed projection\n// holds.".to_owned(),
            "",
            String::new(),
            String::new(),
        )
    } else {
        (
            "every row the owed projection\n// holds for the parameters the query string \
             carries, read by their wire names. A key the\n// view does not declare is ignored."
                .to_owned(),
            ", query string",
            format!(
                "\tvalue, err := queryObject(query, {}, \"query\")\n\tif err != nil {{\n\t\treturn \
                 refusal(400, err.Error())\n\t}}\n\tparams, err := decodeParams{function}(value, \
                 \"query\")\n\tif err != nil {{\n\t\treturn refusal(400, err.Error())\n\t}}\n",
                query_table(emit.ir, view)
            ),
            view.params
                .iter()
                .map(|param| {
                    format!(
                        "params.{}",
                        super::items::member_ident(&view.params, &param.name)
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
        )
    };
    let _ = write!(
        out,
        "\n// serve{function} answers `GET` `{}` at `{}` consistency: {doc}\nfunc \
         serve{function}(system *{system}{parameter}) response {{\n{decoded}\trows, unmet \
         := system.{field}.{method}({arguments})\n\tif unmet != nil {{\n\t\treturn \
         unfinished(unmet.Error(), false)\n\t}}\n\tencoded := make([]any, 0, len(rows))\n\tfor _, row := range rows \
         {{\n\t\tencoded = append(encoded, encodeView{function}(row))\n\t}}\n\treturn \
         rendered(200, map[string]any{{\"rows\": encoded}})\n}}\n",
        view.name,
        view.consistency.as_str()
    );
}

/// A view's declared parameters as the `[]queryParam` literal `queryObject` reads.
fn query_table(ir: &EssIr, view: &ResolvedView) -> String {
    let rows: Vec<String> = view
        .params
        .iter()
        .map(|param| {
            let scalar = match crate::view_query::query_scalar(ir, &param.type_ref) {
                Some(crate::view_query::QueryScalar::Integer) => "queryInteger",
                Some(crate::view_query::QueryScalar::Boolean) => "queryBoolean",
                // `refuse_unqueryable` has refused every other parameter of a served view.
                Some(crate::view_query::QueryScalar::Text) | None => "queryText",
            };
            format!(
                "{{wire: {:?}, scalar: {scalar}}}",
                ess_gen::schema::wire_field_name(param)
            )
        })
        .collect();
    format!("[]queryParam{{{}}}", rows.join(", "))
}

/// One view's declared parameters, decoded: the struct the route reads them into and its decoder,
/// which reads an object keyed by their wire names exactly as a command input's decoder does.
fn params_decoder(out: &mut String, emit: &Emit<'_>, view: &ResolvedView) {
    let function = ident(emit.layout, &view.name);
    let _ = writeln!(
        out,
        "\n// params{function} is the declared parameters of `{}`, decoded.\ntype \
         params{function} struct {{",
        view.name
    );
    let fields: Vec<(String, String)> = view
        .params
        .iter()
        .map(|param| {
            (
                super::items::member_ident(&view.params, &param.name),
                emit.go_type(&param.type_ref),
            )
        })
        .collect();
    // Aligned as gofmt aligns a struct's fields, so the emitted file is gofmt-clean.
    let width = fields
        .iter()
        .map(|(field, _)| field.len())
        .max()
        .unwrap_or(0);
    for (field, of) in &fields {
        let _ = writeln!(out, "\t{field:width$} {of}");
    }
    let _ = write!(
        out,
        "}}\n\n// decodeParams{function} reads the parameters of `{}` from an object keyed by their \
         wire names.\nfunc decodeParams{function}(value any, at string) (params{function}, error) \
         {{\n\tvar out params{function}\n\tif _, err := objectAt(value, at, \"an object\"); err != \
         nil {{\n\t\treturn out, err\n\t}}\n",
        view.name
    );
    let mut slot = 0;
    for param in &view.params {
        decode_member(
            out,
            emit,
            "\t",
            &format!(
                "out.{}",
                super::items::member_ident(&view.params, &param.name)
            ),
            param,
            &mut slot,
        );
    }
    out.push_str("\treturn out, nil\n}\n");
}

/// `true` where some served view declares parameters, and so where the package carries
/// [`QUERY_HELPERS`]: a model without one keeps its bytes.
fn serves_params(ir: &EssIr, refusals: &TargetRefusals) -> bool {
    served(ir, refusals).into_iter().any(|component| {
        http::routes(ir, component).iter().any(
            |route| matches!(route.serves, Served::View(view) if !ir.view(view).params.is_empty()),
        )
    })
}

/// What the package adds where a served view declares parameters: the query string read as the
/// object those parameters' decoder reads (story:served-view-params). The same reading the Rust
/// surface's `http::query_object` makes, refusal for refusal.
const QUERY_HELPERS: &str = r#"
// queryScalar is how one declared parameter's query value is written as JSON before its decoder
// reads it. A query value is text; the parameter's declared type says which JSON scalar the wire
// writes it as, and the generated decoder then reads it exactly as it reads a command's input.
type queryScalar int

const (
	// queryText is a JSON string: text, a decimal, an instant, a duration, a UUID, base64 bytes,
	// an enum's wire spelling.
	queryText queryScalar = iota
	// queryInteger is a JSON number where the value spells a whole number, and a string otherwise,
	// so the decoder names what arrived rather than a number that never did.
	queryInteger
	// queryBoolean is a JSON boolean where the value is true or false, and a string otherwise.
	queryBoolean
)

// queryParam is one declared parameter: the key the query carries it under, and how its value is
// written.
type queryParam struct {
	wire   string
	scalar queryScalar
}

// queryObject is the parameters a query string carries, as the object their decoder reads: one
// member per declared parameter the query names, keyed by its wire name.
//
// Keys and values are form-decoded (%XX and +). A key nothing declares is ignored, and a declared
// key the query omits is absent from the object, for the decoder to refuse or to read as absent.
// A declared key that arrives more than once, or whose value is not percent-encoded UTF-8, is
// refused under at.
func queryObject(query string, declared []queryParam, at string) (map[string]any, error) {
	object := map[string]any{}
	for _, param := range declared {
		var found []string
		for _, pair := range strings.Split(query, "&") {
			if pair == "" {
				continue
			}
			key, value, _ := strings.Cut(pair, "=")
			if decoded, ok := formDecoded(key); ok && decoded == param.wire {
				found = append(found, value)
			}
		}
		place := nested(at, param.wire)
		if len(found) == 0 {
			continue
		}
		if len(found) > 1 {
			return nil, fmt.Errorf("%s: expected one value, found %d", place, len(found))
		}
		text, ok := formDecoded(found[0])
		if !ok {
			return nil, fmt.Errorf("%s: expected percent-encoded UTF-8 text, found `%s`", place, found[0])
		}
		switch {
		case param.scalar == queryInteger && whole(text):
			object[param.wire] = json.Number(text)
		case param.scalar == queryBoolean && text == "true":
			object[param.wire] = true
		case param.scalar == queryBoolean && text == "false":
			object[param.wire] = false
		default:
			object[param.wire] = text
		}
	}
	return object, nil
}

// formDecoded is one form-encoded key or value, decoded: + is a space and %XX a byte. Not ok where
// an escape is not two hexadecimal digits or the bytes are not UTF-8.
func formDecoded(text string) (string, bool) {
	decoded, err := url.QueryUnescape(text)
	if err != nil || !utf8.ValidString(decoded) {
		return "", false
	}
	return decoded, true
}

// whole reports whether text spells a whole number: an optional -, then one or more digits.
func whole(text string) bool {
	digits := strings.TrimPrefix(text, "-")
	if digits == "" {
		return false
	}
	for _, digit := range digits {
		if digit < '0' || digit > '9' {
			return false
		}
	}
	return true
}
"#;

/// What the emitted enum and union encoders answer for a value no branch names.
///
/// A `default` arm rather than a statement after the switch, for two reasons that happen to agree:
/// a switch whose every arm returns and which has a default is a *terminating* statement, so Go
/// does not ask for a second return it would then call unreachable; and the binding a union's
/// switch makes is in scope inside the switch and nowhere else.
const UNREACHABLE_VARIANT: &str = "\tdefault:\n\t\t// Go cannot check that a switch over a \
                                   sealed interface is total (see TARGET.md).\n\t\t// A value no \
                                   branch above names is one no generated code can \
                                   construct.\n\t\treturn nil\n\t}\n";

/// The same, for a union, whose switch binds the shape it matched.
const UNREACHABLE_SHAPE: &str = "\tdefault:\n\t\t_ = shape\n\t\t// Go cannot check that a \
                                 switch over a sealed interface is total (see \
                                 TARGET.md).\n\t\t// A shape no branch above names is one no \
                                 generated code can construct.\n\t\treturn nil\n\t}\n";

/// The package's documentation, on the one file that carries the package clause a reader meets
/// first.
const SERVER_DOC: &str = "// Package server is the HTTP surface of every component the \
                          specification says is reached\n// over a network.\n//\n// The codecs \
                          beside this file are generated rather than derived: a generated type \
                          carries\n// an unexported field, which `encoding/json` cannot see, and \
                          exporting it would undo the\n// distinctness the newtype encoding \
                          exists for. What they render is what the published\n// contracts \
                          already fix — bytes as base64, a decimal, timestamp, duration and UUID \
                          as\n// strings, an absent optional member omitted rather than sent as \
                          null.\n";

/// The fixed helpers the generated codecs call.
const WIRE_HELPERS: &str = r#"
// DecodeError is a refusal at one path, with what the declaration says belongs there and what
// arrived instead.
//
// The path is what makes it usable: a caller that sent a nested command input gets the field, not
// "invalid request".
type DecodeError struct {
	// At is where in the document, as a dotted path from its root.
	At string
	// Expected is what the declaration says belongs there.
	Expected string
	// Found is what was there instead.
	Found string
}

// Error renders the refusal.
func (e DecodeError) Error() string {
	return fmt.Sprintf("%s: expected %s, found %s", e.At, e.Expected, e.Found)
}

// describes names what a decoded JSON value is, for a refusal.
func describes(value any) string {
	switch shaped := value.(type) {
	case nil:
		return "null"
	case bool:
		return "a boolean"
	case json.Number:
		return "a number"
	case string:
		return "a string"
	case []any:
		return "an array"
	case map[string]any:
		return "an object"
	default:
		_ = shaped
		return "a value of an unknown shape"
	}
}

// nested is one step further into a document, for a message a reader can follow back.
func nested(at string, step string) string {
	if at == "" {
		return step
	}
	return at + "." + step
}

// indexed is one step into an array.
func indexed(at string, index int) string {
	return fmt.Sprintf("%s[%d]", at, index)
}

// objectAt is the object at this path.
func objectAt(value any, at string, expected string) (map[string]any, error) {
	object, ok := value.(map[string]any)
	if !ok {
		return nil, DecodeError{At: at, Expected: expected, Found: describes(value)}
	}
	return object, nil
}

// itemsAt is the array at this path.
func itemsAt(value any, at string, expected string) ([]any, error) {
	items, ok := value.([]any)
	if !ok {
		return nil, DecodeError{At: at, Expected: expected, Found: describes(value)}
	}
	return items, nil
}

// required is the member a declaration says must be there, and the path it sits at.
func required(value any, at string, name string) (any, string, error) {
	memberAt := nested(at, name)
	object, err := objectAt(value, at, "an object")
	if err != nil {
		return nil, memberAt, err
	}
	member, ok := object[name]
	if !ok {
		// The same sentence the Rust target's reader writes, word for word. Two applications
		// synthesised from one specification and refusing one request differently would be two
		// diagnostics a caller has to learn, and `cargo xtask synth --check` compares the bodies.
		return nil, memberAt, DecodeError{At: memberAt, Expected: "a value", Found: "nothing"}
	}
	return member, memberAt, nil
}

// optional is the member a declaration says may be there. An absent member and a null one are the
// same answer, because the published contract omits an absent optional rather than sending null.
func optional(value any, at string, name string) (any, string, bool, error) {
	memberAt := nested(at, name)
	object, err := objectAt(value, at, "an object")
	if err != nil {
		return nil, memberAt, false, err
	}
	member, ok := object[name]
	if !ok || member == nil {
		return nil, memberAt, false, nil
	}
	return member, memberAt, true, nil
}

// textAt is the string at this path.
func textAt(value any, at string, expected string) (string, error) {
	text, ok := value.(string)
	if !ok {
		return "", DecodeError{At: at, Expected: expected, Found: describes(value)}
	}
	return text, nil
}

// boolAt is the boolean at this path.
func boolAt(value any, at string, expected string) (bool, error) {
	held, ok := value.(bool)
	if !ok {
		return false, DecodeError{At: at, Expected: expected, Found: describes(value)}
	}
	return held, nil
}

// integerAt is the whole number at this path.
//
// Read through json.Number, which is why the decoder is configured with UseNumber: the default
// float64 loses whole numbers past 2^53, and an Integer in this model is 64 bits.
func integerAt(value any, at string, expected string) (int64, error) {
	number, ok := value.(json.Number)
	if !ok {
		return 0, DecodeError{At: at, Expected: expected, Found: describes(value)}
	}
	held, err := number.Int64()
	if err != nil {
		return 0, DecodeError{At: at, Expected: expected, Found: "the number " + number.String()}
	}
	return held, nil
}

// decimalAt is the decimal string at this path, in the published pattern: an optional -, digits
// without a leading zero, then an optional . and digits. Refused otherwise, as the contract refuses
// it, rather than handed on as a decimal nobody can read.
func decimalAt(value any, at string, expected string) (string, error) {
	text, err := textAt(value, at, expected)
	if err != nil {
		return "", err
	}
	whole, fraction, fractional := strings.Cut(strings.TrimPrefix(text, "-"), ".")
	if !digitsOnly(whole) || (len(whole) > 1 && whole[0] == '0') || (fractional && !digitsOnly(fraction)) {
		return "", DecodeError{At: at, Expected: expected, Found: fmt.Sprintf("`%s`", text)}
	}
	return text, nil
}

// uuidAt is the UUID at this path, in the published pattern: the canonical hyphenated form, in
// either case.
func uuidAt(value any, at string, expected string) (string, error) {
	text, err := textAt(value, at, expected)
	if err != nil {
		return "", err
	}
	valid := len(text) == 36
	for index := 0; valid && index < len(text); index++ {
		switch char := text[index]; {
		case index == 8 || index == 13 || index == 18 || index == 23:
			valid = char == '-'
		default:
			valid = (char >= '0' && char <= '9') || (char >= 'a' && char <= 'f') || (char >= 'A' && char <= 'F')
		}
	}
	if !valid {
		return "", DecodeError{At: at, Expected: expected, Found: fmt.Sprintf("`%s`", text)}
	}
	return text, nil
}

// digitsOnly reports whether text is one or more ASCII digits.
func digitsOnly(text string) bool {
	if text == "" {
		return false
	}
	for index := 0; index < len(text); index++ {
		if text[index] < '0' || text[index] > '9' {
			return false
		}
	}
	return true
}

// bytesAt is the base64-encoded bytes at this path.
func bytesAt(value any, at string, expected string) ([]byte, error) {
	text, err := textAt(value, at, expected)
	if err != nil {
		return nil, err
	}
	return base64Text(text, at, expected)
}

// base64Text is base64 text as bytes, in the published pattern only: whole groups of four characters
// of the alphabet, the last of them padded with one or two = at most — nothing unpadded, nothing
// after padding, no whitespace (which the standard decoder would skip). The same rule and the same
// words as the Rust target's reader.
func base64Text(text string, at string, expected string) ([]byte, error) {
	refused := DecodeError{At: at, Expected: expected, Found: "a string that is not base64"}
	if len(text)%4 != 0 {
		return nil, refused
	}
	body := strings.TrimRight(text, "=")
	if len(text)-len(body) > 2 {
		return nil, refused
	}
	for index := 0; index < len(body); index++ {
		char := body[index]
		if !((char >= 'A' && char <= 'Z') || (char >= 'a' && char <= 'z') || (char >= '0' && char <= '9') || char == '+' || char == '/') {
			return nil, refused
		}
	}
	held, err := base64.StdEncoding.DecodeString(text)
	if err != nil {
		return nil, refused
	}
	return held, nil
}

// keyBool reads a boolean written as an object key: `true` or `false`, and nothing else.
func keyBool(key string, at string) (bool, error) {
	switch key {
	case "true":
		return true, nil
	case "false":
		return false, nil
	}
	return false, DecodeError{At: at, Expected: "a key spelling `true` or `false`", Found: fmt.Sprintf("the key `%s`", key)}
}

// keyInteger reads an integer written as an object key, in the published pattern: no sign but -, no
// leading zero, and within 64 bits.
func keyInteger(key string, at string) (int64, error) {
	digits := strings.TrimPrefix(key, "-")
	if digitsOnly(digits) && (digits == "0" || digits[0] != '0') {
		if held, err := strconv.ParseInt(key, 10, 64); err == nil {
			return held, nil
		}
	}
	return 0, DecodeError{At: at, Expected: "a key spelling an integer", Found: fmt.Sprintf("the key `%s`", key)}
}

// keyBytes reads base64-encoded bytes written as an object key.
func keyBytes(key string, at string) ([]byte, error) {
	return base64Text(key, at, "a base64 key")
}
"#;

/// The fixed helpers one served component's file needs, emitted once per file.
///
/// Once per *file* rather than once per package, because a package holds one file per served
/// component and Go would refuse the second declaration. They are therefore named after nothing —
/// see [`surface_file`], which emits exactly one served component per file, and the emitter refuses
/// more than one served component per package below.
const SURFACE_HELPERS: &str = r#"
// The media type every answer derived from the model carries.
const mediaJSON = "application/json"

// The media type the prose answer carries.
//
// The bytes served are the committed Markdown, unrendered: rendering it to HTML here would be a
// second rendering of the documentation, and the two would differ the first time either moved.
const mediaMarkdown = "text/markdown; charset=utf-8"

// The largest body this surface reads, in bytes.
//
// A caller can claim any length, and a server that allocated whatever it was told to is a server
// anyone can stop by saying a large number. A megabyte is far past any command input this model
// can describe.
const maxBody = 1048576

// serving is held from reading a request's input to rendering its answer, so one request at a time
// runs a port, pumps and takes from the system's log.
//
// net/http answers every connection on its own goroutine, and the system is one value: its log, its
// delivery cursor and every component's outbox are shared, and so is whatever the realization
// behind the ports keeps. One lock for every surface in this package, because two components served
// from one process share one system.
var serving sync.Mutex

// response is one answer: a status, a media type and a body.
type response struct {
	status      int
	contentType string
	body        string
}

// write sends the answer and lets the connection close behind it.
//
// Content-Length is set rather than left to the server: without it a body past the write buffer is
// sent with chunked transfer encoding, and the two applications synthesised from one specification
// would then differ on the wire for a reason no reader of the specification could predict. A caller
// that reads to the end of the connection gets the same bytes from both.
func (r response) write(writer http.ResponseWriter) {
	writer.Header().Set("Content-Type", r.contentType)
	writer.Header().Set("Content-Length", strconv.Itoa(len(r.body)))
	writer.Header().Set("Connection", "close")
	writer.WriteHeader(r.status)
	_, _ = writer.Write([]byte(r.body))
}

// refusal is an answer this surface makes rather than the specification.
//
// A malformed request, a path nothing declares, a method a path does not answer, an obligation
// nothing has satisfied. None of these is a declared outcome and none is published in the
// contract, because each is a fact about a transport rather than about a command. The body is
// JSON with one member: a caller that has just failed to satisfy a contract should not have to
// parse a second format to read why.
func refusal(status int, detail string) response {
	return rendered(status, map[string]any{"refused": detail})
}

// unfinished is the 501 the contract declares: the realization is unfinished.
//
// Its body is refusal's with one more member, committed: true when the command's effect and
// events were committed and delivering what it published failed, and false when an unmet
// obligation stopped it before anything was written.
func unfinished(detail string, committed bool) response {
	return rendered(501, struct {
		Refused   string `json:"refused"`
		Committed bool   `json:"committed"`
	}{detail, committed})
}

// methodNotAllowed is the answer for a path this surface holds under a different method.
func methodNotAllowed(allowed string) response {
	return refusal(405, fmt.Sprintf("this path answers `%s`, and the contract declares no other method for it", allowed))
}

// rendered is one answer whose body is a value this package built.
func rendered(status int, body any) response {
	encoded, err := json.Marshal(body)
	if err != nil {
		return response{status: 500, contentType: mediaJSON, body: `{"refused":"the answer could not be encoded"}`}
	}
	return response{status: status, contentType: mediaJSON, body: string(encoded)}
}

// maxHeaders is the most headers this surface keeps from one request, as the Rust target's
// http::MAX_HEADERS: a hundred is far past what a client and a proxy add together.
const maxHeaders = 100

// tooManyHeaders is the 431 the Rust target answers for a request past maxHeaders, word for word,
// or nil. net/http moves Host out of the header map; it counts as one header, as it does there.
func tooManyHeaders(request *http.Request) *response {
	count := 0
	if request.Host != "" {
		count = 1
	}
	for _, values := range request.Header {
		count += len(values)
	}
	if count <= maxHeaders {
		return nil
	}
	answer := refusal(431, fmt.Sprintf("the request carries more than %d headers, which is all this surface keeps", maxHeaders))
	return &answer
}

// readBody reads at most maxBody bytes of a request, or the refusal that says why it could not.
func readBody(request *http.Request) ([]byte, *response) {
	if request.Body == nil {
		return nil, nil
	}
	defer func() { _ = request.Body.Close() }()
	body, err := io.ReadAll(io.LimitReader(request.Body, maxBody+1))
	if err != nil {
		answer := refusal(400, fmt.Sprintf("the body could not be read: %s", err))
		return nil, &answer
	}
	if len(body) > maxBody {
		answer := refusal(413, fmt.Sprintf("the body is longer than %d bytes, which is all this surface reads", maxBody))
		return nil, &answer
	}
	return body, nil
}

// readJSON parses a request body, or the refusal that says why it is not JSON.
//
// UseNumber, so an Integer past 2^53 survives the crossing: the default reads every number as a
// float64, and a visit id or a count would come back changed.
func readJSON(body []byte) (any, *response) {
	decoder := json.NewDecoder(bytes.NewReader(body))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		answer := refusal(400, fmt.Sprintf("the body is not JSON: %s", err))
		return nil, &answer
	}
	return value, nil
}
"#;
