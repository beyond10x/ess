//! Adversary pass 1 of unit F (ESS 0.70.0): an empty struct (`fields: []`, beyond10x/ess#649)
//! and a lifecycle state's wire spelling (beyond10x/ess#639) through the synthesized Rust surface
//! and the synthesis plan.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{
    synthesize_laid_out, CapabilityKind, OutputLayout, SynthesisDisposition, SynthesisPlan, Target,
};

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}")),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.to_owned());
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The body of the generated function whose signature opens with `signature`.
fn body<'a>(text: &'a str, signature: &str) -> &'a str {
    let start = text
        .find(signature)
        .unwrap_or_else(|| panic!("`{signature}` is generated:\n{text}"));
    let rest = &text[start..];
    let end = rest.find("\n}\n").expect("a generated function closes");
    &rest[..end]
}

const EMPTY: &str = "format: ess/32
system: hollow
version: v1
domain: hollow.notes
types:
  - {name: hollow.notes.NoteId, kind: newtype, of: Uuid}
  - {name: hollow.notes.Ack, kind: struct, undeclared_fields: ignored, fields: []}
  - {name: hollow.notes.Nothing, kind: struct, fields: []}
  - {name: hollow.notes.Holder, kind: struct, fields: [{name: open, type: hollow.notes.Ack}]}
entities:
  - name: hollow.notes.Note
    identity: {name: note_id, type: hollow.notes.NoteId}
    fields:
      - {name: body, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
events:
  - name: hollow.notes.NoteCreated
    fields: [{name: note_id, type: hollow.notes.NoteId}]
actors:
  - {name: hollow.notes.Writer, may: [hollow.notes.CreateNote]}
commands:
  - name: hollow.notes.CreateNote
    input:
      - {name: body, type: String}
      - {name: extra, type: hollow.notes.Ack}
      - {name: closed, type: hollow.notes.Nothing}
      - {name: holder, type: hollow.notes.Holder}
    outcomes:
      - name: created
        creates: hollow.notes.Note
        instance: note_id
        emits: [hollow.notes.NoteCreated]
        payload: {hollow.notes.NoteCreated: {note_id: {generated: true}}}
        sets: {body: input.body}
components:
  - component: notes-service
    owns: {domains: [hollow.notes]}
    accepts: {commands: [hollow.notes.CreateNote]}
    publishes: {events: [hollow.notes.NoteCreated]}
    reached_by: network
";

/// A served Rust decoder of a struct with no fields must still refuse a value that is no object:
/// `42`, `\"x\"` and `[]` are no value of `{type: object}`. A struct with a field refuses them
/// through `member_at`; the empty struct's decoder reads no member, so it reads nothing at all.
/// The generated Go decoder checks `objectAt` for the same struct.
#[test]
fn an_empty_struct_decoder_refuses_a_value_that_is_no_object() {
    let synthesis = synthesize_laid_out(&ir(EMPTY), Target::Rust, OutputLayout::Workspace)
        .unwrap_or_else(|error| panic!("synthesizes: {error:?}"));
    let wire = &synthesis.artifacts["crates/hollow-server/src/wire.rs"].contents;
    let holder = body(wire, "pub fn decode_hollow_notes_holder(");
    assert!(
        holder.contains("json::member_at(value"),
        "a struct with a field reads it from an object:\n{holder}"
    );
    for signature in [
        "pub fn decode_hollow_notes_ack(",
        "pub fn decode_hollow_notes_nothing(",
    ] {
        let decoder = body(wire, signature);
        assert!(
            decoder.contains("members_at(value") || decoder.contains("an object"),
            "an empty struct's decoder checks the value is an object:\n{decoder}"
        );
    }
}

/// The ordered fixture: `Open` before `Closed` on the wire (`a_open` < `z_closed`), `Closed`
/// before `Open` by declared name.
fn ordered(states: &str, kind_field: &str, order_key: &str) -> String {
    format!(
        "format: ess/32
system: ranked
version: v1
domain: ranked.desk
types:
  - {{name: ranked.desk.DeskId, kind: newtype, of: Uuid}}
  - name: ranked.desk.Kind
    kind: enum
    variants: [{{name: Open, wire: a_open}}, {{name: Closed, wire: z_closed}}]
entities:
  - name: ranked.desk.Desk
    identity: {{name: desk_id, type: ranked.desk.DeskId}}
    fields:
      - {{name: label, type: String}}
      - {{name: kind, type: ranked.desk.Kind}}
    lifecycle:
      initial: Open
      states: {states}
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
events:
  - name: ranked.desk.DeskOpened
    fields: [{{name: desk_id, type: ranked.desk.DeskId}}]
  - name: ranked.desk.DeskClosed
    fields: [{{name: desk_id, type: ranked.desk.DeskId}}]
actors:
  - {{name: ranked.desk.Clerk, may: [ranked.desk.OpenDesk, ranked.desk.CloseDesk]}}
commands:
  - name: ranked.desk.OpenDesk
    input:
      - {{name: label, type: String}}
      - {{name: kind, type: ranked.desk.Kind}}
    outcomes:
      - name: opened
        creates: ranked.desk.Desk
        instance: desk_id
        emits: [ranked.desk.DeskOpened]
        payload: {{ranked.desk.DeskOpened: {{desk_id: {{generated: true}}}}}}
        sets: {{label: input.label, kind: input.kind}}
  - name: ranked.desk.CloseDesk
    input:
      - {{name: desk_id, type: ranked.desk.DeskId}}
    outcomes:
      - name: closed
        moves: ranked.desk.Desk.close
        instance: desk_id
        emits: [ranked.desk.DeskClosed]
        payload: {{ranked.desk.DeskClosed: {{desk_id: input.desk_id}}}}
views:
  - name: ranked.desk.Desks
    source: ranked.desk.Desk
    consistency: read_your_writes
    order_by: [{order_key}, desk_id]
    fields:
      - {{name: desk_id, type: ranked.desk.DeskId}}
      - {kind_field}
components:
  - component: desk-service
    owns: {{domains: [ranked.desk]}}
    accepts: {{commands: [ranked.desk.OpenDesk, ranked.desk.CloseDesk]}}
    publishes: {{events: [ranked.desk.DeskOpened, ranked.desk.DeskClosed]}}
    reached_by: network
"
    )
}

fn view_query_obligation(text: &str) -> Option<String> {
    let ir = ir(text);
    match SynthesisPlan::of(&ir).disposition_of(CapabilityKind::ViewQuery, "ranked.desk.Desks") {
        Some(SynthesisDisposition::Obligation(obligation)) => Some(obligation.reason.describes()),
        _ => None,
    }
}

/// Regression guard: an order over an enum field whose wire spellings are not its variant names
/// keeps the generated query owed, because the runner ranks rows by the text they are rendered as.
#[test]
fn an_order_over_a_respelled_enum_keeps_the_view_query_owed() {
    let text = ordered(
        "[Open, Closed]",
        "{name: kind, type: ranked.desk.Kind}",
        "kind",
    );
    let why =
        view_query_obligation(&text).expect("the ordered query over a respelled enum is owed");
    assert!(why.contains("wire spellings"), "{why}");
}

/// A respelled state is the respelled variant of `<Entity>.State`, ranked by the runner as the
/// text it is rendered as (`a_open` before `z_closed`); the generated query ranks it by the
/// declared name (`Closed` before `Open`). It must stay owed, as the respelled enum's query does.
#[test]
fn an_order_over_a_respelled_state_keeps_the_view_query_owed() {
    let text = ordered(
        "[{name: Open, wire: a_open}, {name: Closed, wire: z_closed}]",
        "{name: state, type: ranked.desk.Desk.State}",
        "state",
    );
    let why = view_query_obligation(&text);
    assert!(
        why.as_deref()
            .is_some_and(|why| why.contains("wire spellings")),
        "the ordered query over a respelled state is generated, ranking by declared name: {why:?}"
    );
}
