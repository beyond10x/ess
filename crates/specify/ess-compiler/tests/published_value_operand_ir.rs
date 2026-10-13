//! The compiled IR lists the values of its own outcome's payload a selector reads
//! (`docs/design/published-value-operand.md`, "Format and compatibility", beyond10x/ess#549):
//! `ResolvedRowSelection.reads`, each `{event, field, type_ref}`, in dependency order, omitted when
//! empty. The predicate keeps the path as written. A model without the operand keeps its bytes,
//! and a selector over an entity with a field named `payload` compiles to the same IR at `ess/24`
//! and at `ess/25`.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const HEAD: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/ordered-related-head.yaml");
const READS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/filtered-related-reads.yaml");

/// The best open slot of a group, then the best element on each side of it; the right side also
/// requires an element key other than the left one, so its selector reads two published values.
const NESTED: &str = "format: ess/25
system: demo
version: v1
domain: demo.nested
types:
  - {name: demo.nested.SlotId, kind: newtype, of: Uuid}
  - {name: demo.nested.MembershipId, kind: newtype, of: Uuid}
  - {name: demo.nested.PairId, kind: newtype, of: Uuid}
  - {name: demo.nested.Side, kind: enum, variants: [Left, Right]}
entities:
  - name: demo.nested.Slot
    identity: {name: slot_id, type: demo.nested.SlotId}
    fields:
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.nested.Membership
    identity: {name: membership_id, type: demo.nested.MembershipId}
    fields:
      - {name: slot_id, type: demo.nested.SlotId}
      - {name: side, type: demo.nested.Side}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    lifecycle:
      initial: Present
      states: [Present, Removed]
      terminal: [Removed]
      transitions:
        - {name: remove, from: [Present], to: Removed}
  - name: demo.nested.Pair
    identity: {name: pair_id, type: demo.nested.PairId}
    fields:
      - {name: group_id, type: String}
    lifecycle:
      initial: Made
      states: [Made, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Made], to: Done}
errors:
  - {name: demo.nested.Empty, summary: No slot of the group is open., fields: []}
events:
  - name: demo.nested.SlotOpened
    fields:
      - {name: slot_id, type: demo.nested.SlotId}
  - name: demo.nested.SlotClosed
    fields:
      - {name: slot_id, type: demo.nested.SlotId}
  - name: demo.nested.Joined
    fields:
      - {name: membership_id, type: demo.nested.MembershipId}
  - name: demo.nested.Left
    fields:
      - {name: membership_id, type: demo.nested.MembershipId}
  - name: demo.nested.Finished
    fields:
      - {name: pair_id, type: demo.nested.PairId}
  - name: demo.nested.PairMade
    fields:
      - {name: pair_id, type: demo.nested.PairId}
      - {name: slot_id, type: demo.nested.SlotId}
      - {name: left_key, type: String}
      - {name: right_key, type: String}
commands:
  - name: demo.nested.OpenSlot
    input:
      - {name: slot_id, type: demo.nested.SlotId}
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    outcomes:
      - name: opened
        creates: demo.nested.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
        emits: [demo.nested.SlotOpened]
        payload:
          demo.nested.SlotOpened: {slot_id: input.slot_id}
  - name: demo.nested.CloseSlot
    input:
      - {name: slot_id, type: demo.nested.SlotId}
    outcomes:
      - name: closed
        moves: demo.nested.Slot.close
        instance: slot_id
        emits: [demo.nested.SlotClosed]
        payload:
          demo.nested.SlotClosed: {slot_id: input.slot_id}
  - name: demo.nested.Join
    input:
      - {name: membership_id, type: demo.nested.MembershipId}
      - {name: slot_id, type: demo.nested.SlotId}
      - {name: side, type: demo.nested.Side}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    outcomes:
      - name: joined
        creates: demo.nested.Membership
        instance: membership_id
        sets:
          slot_id: input.slot_id
          side: input.side
          element_key: input.element_key
          element_score: input.element_score
        emits: [demo.nested.Joined]
        payload:
          demo.nested.Joined: {membership_id: input.membership_id}
  - name: demo.nested.Leave
    input:
      - {name: membership_id, type: demo.nested.MembershipId}
    outcomes:
      - name: left
        moves: demo.nested.Membership.remove
        instance: membership_id
        emits: [demo.nested.Left]
        payload:
          demo.nested.Left: {membership_id: input.membership_id}
  - name: demo.nested.Finish
    input:
      - {name: pair_id, type: demo.nested.PairId}
    outcomes:
      - name: finished
        moves: demo.nested.Pair.finish
        instance: pair_id
        emits: [demo.nested.Finished]
        payload:
          demo.nested.Finished: {pair_id: input.pair_id}
  - name: demo.nested.MatchInGroup
    input:
      - {name: pair_id, type: demo.nested.PairId}
      - {name: group_id, type: String}
    outcomes:
      - name: empty
        when_related:
          entity: demo.nested.Slot
          where: {all: [group_id == input.group_id, state == Open]}
          exists: false
        error: demo.nested.Empty
      - name: paired
        creates: demo.nested.Pair
        instance: pair_id
        sets:
          group_id: input.group_id
        emits: [demo.nested.PairMade]
        payload:
          demo.nested.PairMade:
            pair_id: input.pair_id
            slot_id:
              related:
                entity: demo.nested.Slot
                where: {all: [group_id == input.group_id, state == Open]}
                order_by: [score desc, slot_id]
                first: slot_id
            left_key:
              related:
                entity: demo.nested.Membership
                where: {all: [slot_id == payload.demo.nested.PairMade.slot_id, side == Left, state == Present]}
                order_by: [element_score desc, membership_id]
                first: element_key
            right_key:
              related:
                entity: demo.nested.Membership
                where: {all: [element_key != payload.demo.nested.PairMade.left_key, slot_id == payload.demo.nested.PairMade.slot_id, side == Right, state == Present]}
                order_by: [element_score desc, membership_id]
                first: element_key
";

/// The canonical IR with every whitespace character removed.
fn ir(text: &str) -> String {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("nested.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
        .to_canonical_json()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

const SLOT_READ: &str = r#"{"event":"demo.nested.PairMade","field":"slot_id","type_ref":{"kind":"declared","name":"demo.nested.SlotId"}}"#;
const LEFT_READ: &str = r#"{"event":"demo.nested.PairMade","field":"left_key","type_ref":{"kind":"primitive","name":"string"}}"#;

#[test]
fn the_selection_lists_its_published_reads_in_dependency_order() {
    let compact = ir(NESTED);
    // The left side reads the slot, and keeps the path as written in its predicate.
    let left = format!(
        r#""where":{{"all":["slot_id==payload.demo.nested.PairMade.slot_id","side==Left","state==Present"]}},"reads":[{SLOT_READ}]}}"#
    );
    assert!(compact.contains(&left), "{left}\n{compact}");
    // The right side writes the left key first; the left key reads the slot, so the slot comes
    // first.
    let right = format!(
        r#""where":{{"all":["element_key!=payload.demo.nested.PairMade.left_key","slot_id==payload.demo.nested.PairMade.slot_id","side==Right","state==Present"]}},"reads":[{SLOT_READ},{LEFT_READ}]}}"#
    );
    assert!(compact.contains(&right), "{right}\n{compact}");
    // The slot's own selector and the guard read nothing, and say nothing.
    assert!(
        compact
            .contains(r#""where":{"all":["group_id==input.group_id","state==Open"]}},"order_by""#),
        "{compact}"
    );
    assert!(
        compact.contains(r#""where":{"all":["group_id==input.group_id","state==Open"]}},"test""#)
            || compact
                .matches(r#""where":{"all":["group_id==input.group_id","state==Open"]}}"#)
                .count()
                == 2,
        "{compact}"
    );
    assert_eq!(compact.matches(r#""reads":"#).count(), 2, "{compact}");
}

#[test]
fn a_model_without_the_operand_keeps_its_ir_bytes() {
    for fixture in [HEAD, READS] {
        let compact = ir(fixture);
        assert!(!compact.contains(r#""reads":"#), "{compact}");
    }
    // The nested example scoped by the input instead carries no list either.
    let scoped = NESTED
        .replace("payload.demo.nested.PairMade.slot_id", "input.slot_id")
        .replace(
            "element_key != payload.demo.nested.PairMade.left_key, ",
            "",
        )
        .replace(
            "      - {name: group_id, type: String}\n    outcomes:\n      - name: empty",
            "      - {name: group_id, type: String}\n      - {name: slot_id, type: demo.nested.SlotId}\n    outcomes:\n      - name: empty",
        );
    assert!(!scoped.contains("payload.demo"), "{scoped}");
    let compact = ir(&scoped);
    assert!(!compact.contains(r#""reads":"#), "{compact}");
}

/// An entity with a struct field named `payload`, read by a one-row read at `format`.
fn enveloped(format: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.env
types:
  - {{name: demo.env.NoteId, kind: newtype, of: Uuid}}
  - name: demo.env.Envelope
    kind: struct
    fields:
      - {{name: topic, type: String}}
      - {{name: body, type: String}}
entities:
  - name: demo.env.Note
    identity: {{name: note_id, type: demo.env.NoteId}}
    fields:
      - {{name: payload, type: demo.env.Envelope}}
    lifecycle:
      initial: Kept
      states: [Kept, Gone]
      terminal: [Gone]
      transitions:
        - {{name: drop, from: [Kept], to: Gone}}
events:
  - name: demo.env.Noted
    fields:
      - {{name: note_id, type: demo.env.NoteId}}
  - name: demo.env.Dropped
    fields:
      - {{name: note_id, type: demo.env.NoteId}}
  - name: demo.env.Read
    fields:
      - {{name: topic, type: String}}
      - {{name: body, type: Optional<demo.env.NoteId>}}
commands:
  - name: demo.env.Write
    input:
      - {{name: note_id, type: demo.env.NoteId}}
      - {{name: payload, type: demo.env.Envelope}}
    outcomes:
      - name: noted
        creates: demo.env.Note
        instance: note_id
        sets: {{payload: input.payload}}
        emits: [demo.env.Noted]
        payload:
          demo.env.Noted: {{note_id: input.note_id}}
  - name: demo.env.Drop
    input:
      - {{name: note_id, type: demo.env.NoteId}}
    outcomes:
      - name: dropped
        moves: demo.env.Note.drop
        instance: note_id
        emits: [demo.env.Dropped]
        payload:
          demo.env.Dropped: {{note_id: input.note_id}}
  - name: demo.env.ReadTopic
    input:
      - {{name: topic, type: String}}
    outcomes:
      - name: read
        emits: [demo.env.Read]
        payload:
          demo.env.Read:
            topic: input.topic
            body:
              related:
                entity: demo.env.Note
                where: {{all: [payload.topic == input.topic, state == Kept]}}
                field: note_id
"
    )
}

#[test]
fn a_selector_over_an_entity_with_a_payload_field_compiles_alike_at_ess_24_and_ess_25() {
    let plain =
        |format: &str| ir(&enveloped(format)).replace(&format!(r#""format":"{format}""#), "");
    let at_24 = plain("ess/24");
    let at_25 = plain("ess/25");
    assert!(at_25.contains("payload.topic==input.topic"), "{at_25}");
    assert!(!at_25.contains(r#""reads":"#), "{at_25}");
    assert_eq!(at_24, at_25);
}
