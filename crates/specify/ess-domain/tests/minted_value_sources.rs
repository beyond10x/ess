//! A minted value stored present, `{generated: true, present: true}`, and a `sets:` value read from
//! the outcome's own payload, `{payload: <Event>.<field>}` (`ess/27`, beyond10x/ess#545,
//! `docs/design/minted-value-sources.md`).
//!
//! What validation decides: `present` only over an `Optional` target, the payload source only in
//! `sets:`, naming an event this outcome's `payload:` fills and a field it gives a source, at a
//! type assignable to the stored field; and below `ess/27` both shapes keep the nested-mapping
//! reading they always had.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("routing.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// The issue's model at `format`, with `SelectAgent`'s `sets:` and `payload:` entries as given.
fn model(format: &str, sets: &str, payload: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.routing
types:
  - {{name: demo.routing.EntryId, kind: newtype, of: Uuid}}
  - {{name: demo.routing.ConnectSeq, kind: newtype, of: Integer}}
  - name: demo.routing.Flags
    kind: struct
    fields:
      - {{name: generated, type: Boolean}}
      - {{name: present, type: Boolean}}
  - name: demo.routing.Note
    kind: struct
    fields:
      - {{name: payload, type: String}}
entities:
  - name: demo.routing.Entry
    identity: {{name: entry_id, type: demo.routing.EntryId}}
    fields:
      - {{name: connect_seq, type: Optional<demo.routing.ConnectSeq>}}
      - {{name: rank, type: demo.routing.ConnectSeq}}
      - {{name: label, type: Optional<String>}}
      - {{name: flags, type: Optional<demo.routing.Flags>}}
      - {{name: note, type: Optional<demo.routing.Note>}}
    lifecycle:
      initial: Waiting
      states: [Waiting, Selected]
      terminal: [Selected]
      transitions:
        - {{name: select, from: [Waiting], to: Selected}}
events:
  - name: demo.routing.EntryQueued
    fields:
      - {{name: entry_id, type: demo.routing.EntryId}}
  - name: demo.routing.EntryAgentSelected
    fields:
      - {{name: entry_id, type: demo.routing.EntryId}}
      - {{name: connect_seq, type: demo.routing.ConnectSeq}}
      - {{name: label, type: String}}
  - name: demo.routing.EntryAudited
    fields:
      - {{name: entry_id, type: demo.routing.EntryId}}
      - {{name: connect_seq, type: demo.routing.ConnectSeq}}
commands:
  - name: demo.routing.QueueEntry
    input: []
    outcomes:
      - name: queued
        creates: demo.routing.Entry
        instance: entry_id
        sets:
          rank: {{generated: true}}
        emits: [demo.routing.EntryQueued]
        payload:
          demo.routing.EntryQueued: {{entry_id: {{generated: true}}}}
  - name: demo.routing.SelectAgent
    input:
      - {{name: entry_id, type: demo.routing.EntryId}}
    outcomes:
      - name: selected
        moves: demo.routing.Entry.select
        instance: entry_id
        emits: [demo.routing.EntryAgentSelected, demo.routing.EntryAudited]
        payload:
          demo.routing.EntryAgentSelected:
            entry_id: input.entry_id
            connect_seq: {{generated: true}}
            label: {{generated: true}}
{payload}        sets:
{sets}"
    )
}

const AUDITED: &str = "          demo.routing.EntryAudited:
            entry_id: input.entry_id
            connect_seq: {generated: true}
";

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

#[test]
fn present_generated_into_an_optional_field_validates_at_ess_27() {
    let body = model(
        "ess/27",
        "          connect_seq: {generated: true, present: true}\n",
        AUDITED,
    );
    spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn present_on_a_required_field_is_refused() {
    let body = model(
        "ess/27",
        "          rank: {generated: true, present: true}\n",
        AUDITED,
    );
    let errors = refused(&body);
    assert!(errors.contains("sets.rank"), "{errors}");
    assert!(errors.contains("present"), "{errors}");
    assert!(errors.contains("Optional"), "{errors}");
}

#[test]
fn present_false_is_refused() {
    let body = model(
        "ess/27",
        "          connect_seq: {generated: true, present: false}\n",
        AUDITED,
    );
    let errors = refused(&body);
    assert!(errors.contains("sets.connect_seq"), "{errors}");
    assert!(errors.contains("present"), "{errors}");
}

#[test]
fn a_payload_source_reading_this_outcomes_mint_validates_at_ess_27() {
    for sets in [
        "          connect_seq: {payload: demo.routing.EntryAgentSelected.connect_seq}\n",
        "          rank: {payload: demo.routing.EntryAudited.connect_seq}\n",
        "          label: {payload: demo.routing.EntryAgentSelected.label}\n",
    ] {
        spec(&model("ess/27", sets, AUDITED)).unwrap_or_else(|errors| panic!("{sets}{errors}"));
    }
}

#[test]
fn a_payload_source_naming_another_outcomes_event_is_refused() {
    // `EntryQueued` is emitted by `QueueEntry`, not by this outcome.
    let body = model(
        "ess/27",
        "          connect_seq: {payload: demo.routing.EntryQueued.entry_id}\n",
        AUDITED,
    );
    let errors = refused(&body);
    assert!(errors.contains("sets.connect_seq"), "{errors}");
    assert!(errors.contains("demo.routing.EntryQueued"), "{errors}");
    assert!(errors.contains("payload"), "{errors}");

    // Emitted, but with no `payload:` entry giving the field a source.
    let body = model(
        "ess/27",
        "          connect_seq: {payload: demo.routing.EntryAudited.connect_seq}\n",
        "",
    );
    let errors = refused(&body);
    assert!(errors.contains("sets.connect_seq"), "{errors}");
    assert!(errors.contains("demo.routing.EntryAudited"), "{errors}");

    // A field the event does not declare.
    let body = model(
        "ess/27",
        "          connect_seq: {payload: demo.routing.EntryAgentSelected.seq}\n",
        AUDITED,
    );
    let errors = refused(&body);
    assert!(errors.contains("sets.connect_seq"), "{errors}");
    assert!(errors.contains("seq"), "{errors}");
}

#[test]
fn a_payload_source_with_a_mismatched_type_is_refused() {
    let body = model(
        "ess/27",
        "          connect_seq: {payload: demo.routing.EntryAgentSelected.label}\n",
        AUDITED,
    );
    let errors = refused(&body);
    assert!(errors.contains("type_mismatch"), "{errors}");
    assert!(errors.contains("sets.connect_seq"), "{errors}");
    assert!(errors.contains("EntryAgentSelected.label"), "{errors}");
    assert!(!errors.contains("nested mapping"), "{errors}");
    assert!(!errors.contains("unsupported_format_version"), "{errors}");
}

#[test]
fn a_payload_source_inside_a_payload_is_refused() {
    let body = model(
        "ess/27",
        "          connect_seq: {generated: true, present: true}\n",
        "          demo.routing.EntryAudited:
            entry_id: input.entry_id
            connect_seq: {payload: demo.routing.EntryAgentSelected.connect_seq}
",
    );
    let errors = refused(&body);
    assert!(
        errors.contains("payload.demo.routing.EntryAudited"),
        "{errors}"
    );
    assert!(errors.contains("sets:"), "{errors}");
}

/// Below `ess/27` neither shape is a source: each reads as the nested mapping it always was, so a
/// struct with fields of those names keeps its meaning, and an `Optional` scalar is refused as a
/// nested mapping was, with a hint naming `ess/27`.
#[test]
fn below_ess_27_both_shapes_keep_their_nested_reading() {
    let body = model(
        "ess/26",
        "          flags: {generated: true, present: true}\n          note: {payload: input.entry_id}\n",
        AUDITED,
    );
    // `note.payload` is a String filled from a Uuid input: refused as the nested leaf it is.
    let errors = refused(&body);
    assert!(errors.contains("sets.note.payload"), "{errors}");
    assert!(!errors.contains("sets.flags"), "{errors}");

    let body = model(
        "ess/26",
        "          flags: {generated: true, present: true}\n",
        AUDITED,
    );
    spec(&body).unwrap_or_else(|errors| panic!("{errors}"));

    for sets in [
        "          connect_seq: {generated: true, present: true}\n",
        "          connect_seq: {payload: demo.routing.EntryAgentSelected.connect_seq}\n",
    ] {
        let errors = refused(&model("ess/26", sets, AUDITED));
        assert!(errors.contains("sets.connect_seq"), "{errors}");
        assert!(errors.contains("ess/27"), "{errors}");
    }
}
