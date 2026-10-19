//! A selector reads a value its own outcome publishes: `payload.<Event>.<field>[.<member>…]` in the
//! `where:` of a value read (`ess/27`, beyond10x/ess#549, `docs/design/published-value-operand.md`).
//!
//! What validation decides:
//! * the operand is admitted in the selector of `{related: {entity, where, field}}` and of
//!   `{related: {entity, where, order_by, first}}`, typed at the event field's declared type;
//! * below `ess/27` it is refused naming `ess/27`, where it names an event the outcome fills;
//! * an entity declaring a field `payload` keeps reading `payload.<member>` as that field;
//! * a guard, `affects[].where`, `instances:` and an error payload refuse it by name;
//! * the read field is one the outcome emits and fills, with a snapshot-value source;
//! * the payload fields of one outcome read one another without a cycle (`self_reference`).

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

/// Slots of a group ranked by score, memberships on either side of a slot ranked by element score,
/// and a command pairing the best element of each side of the best slot. `@…@` mark where a test
/// rewrites the model.
const MODEL: &str = "format: @FORMAT@
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
  - name: demo.nested.EmptyKeyed
    summary: No slot of the group is open.
    fields:
      - {name: key, type: Optional<String>}
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
          where: @GUARD@
          exists: false
@ERROR@      - name: paired
        creates: demo.nested.Pair
        instance: pair_id
        sets:
          group_id: input.group_id
        emits: [demo.nested.PairMade]
        payload:
          demo.nested.PairMade:
            pair_id: input.pair_id
            slot_id:
@SLOT@            left_key:
              related:
                entity: demo.nested.Membership
                where: {all: [slot_id == @LEFT_SCOPE@, side == Left, state == Present]}
                order_by: [element_score desc, membership_id]
                first: element_key
            right_key:
              related:
                entity: demo.nested.Membership
                where: {all: [slot_id == @RIGHT_SCOPE@, side == Right, state == Present]}
                order_by: [element_score desc, membership_id]
                first: element_key
";

/// The best open slot of the group, as `slot_id`'s source.
const SLOT_HEAD: &str = "              related:
                entity: demo.nested.Slot
                where: {all: [group_id == input.group_id, state == Open]}
                order_by: [score desc, slot_id]
                first: slot_id
";

const SLOT_GUARD: &str = "{all: [group_id == input.group_id, state == Open]}";
const ERROR_PLAIN: &str = "        error: demo.nested.Empty\n";
const OPERAND: &str = "payload.demo.nested.PairMade.slot_id";

/// What a test changes of [`MODEL`]; every key left out keeps the nested example.
#[derive(Clone, Copy)]
struct Edit<'a> {
    format: &'a str,
    slot: &'a str,
    left: &'a str,
    right: &'a str,
    guard: &'a str,
    error: &'a str,
}

impl Default for Edit<'_> {
    fn default() -> Self {
        Self {
            format: "ess/27",
            slot: SLOT_HEAD,
            left: OPERAND,
            right: OPERAND,
            guard: SLOT_GUARD,
            error: ERROR_PLAIN,
        }
    }
}

fn model(edit: Edit<'_>) -> String {
    MODEL
        .replace("@FORMAT@", edit.format)
        .replace("@SLOT@", edit.slot)
        .replace("@LEFT_SCOPE@", edit.left)
        .replace("@RIGHT_SCOPE@", edit.right)
        .replace("@GUARD@", edit.guard)
        .replace("@ERROR@", edit.error)
}

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the model parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("nested.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

/// The refusals of `errors` with `code`.
fn with_code(errors: &ValidationErrors, code: ValidationCode) -> Vec<&ValidationError> {
    errors
        .as_slice()
        .iter()
        .filter(|error| error.code == code)
        .collect()
}

/// The one refusal of `errors` with `code`, which must also be the only refusal at all.
fn alone(errors: &ValidationErrors, code: ValidationCode) -> &ValidationError {
    let [only] = errors.as_slice() else {
        panic!("exactly one refusal, got:\n{errors}");
    };
    assert_eq!(only.code, code, "{errors}");
    only
}

fn rendered(error: &ValidationError) -> String {
    ValidationErrors::from(error.clone()).to_string()
}

#[test]
fn a_nested_head_reading_the_published_slot_validates_at_ess_27() {
    admitted(&model(Edit::default()));

    // The operand stands on either side of the comparison.
    admitted(
        &model(Edit {
            left: "slot_id",
            right: "slot_id",
            ..Edit::default()
        })
        .replace(
            "slot_id == slot_id, side == Left",
            "payload.demo.nested.PairMade.slot_id == slot_id, side == Left",
        )
        .replace(
            "slot_id == slot_id, side == Right",
            "payload.demo.nested.PairMade.slot_id == slot_id, side == Right",
        ),
    );

    // A one-row read (`field:`) reads it too, and so does a read whose published field is an
    // input, a literal-free snapshot of the request.
    admitted(&model(Edit {
        slot: "              related:
                entity: demo.nested.Slot
                where: {all: [group_id == input.group_id, state == Open]}
                field: slot_id
",
        ..Edit::default()
    }));
}

/// The nested example without ordered heads, which `ess/27` introduced: every read is a one-row
/// read (`ess/22`), and the inner ones are scoped by `scope`.
fn flat(format: &str, scope: &str) -> String {
    model(Edit {
        format,
        left: scope,
        right: scope,
        ..Edit::default()
    })
    .replace(
        SLOT_HEAD,
        "              related:
                entity: demo.nested.Slot
                where: {all: [group_id == input.group_id, state == Open]}
                field: slot_id
",
    )
    .replace(
        "                order_by: [element_score desc, membership_id]
                first: element_key
",
        "                field: element_key
",
    )
}

#[test]
fn the_operand_below_ess_27_is_refused_naming_ess_27() {
    admitted(&flat("ess/27", OPERAND));
    admitted(&flat("ess/26", "slot_id"));

    let errors = refused(&flat("ess/26", OPERAND));
    let refusals = with_code(&errors, ValidationCode::UnsupportedFormatVersion);
    assert_eq!(refusals.len(), 2, "one per reading field:\n{errors}");
    assert_eq!(errors.as_slice().len(), 2, "{errors}");
    let text = errors.to_string();
    assert!(text.contains(OPERAND), "{text}");
    assert!(text.contains("ess/27"), "{text}");
    assert!(
        text.contains("payload.demo.nested.PairMade.left_key"),
        "{text}"
    );
    assert!(
        text.contains("payload.demo.nested.PairMade.right_key"),
        "{text}"
    );
    assert!(
        refusals
            .iter()
            .all(|error| error.hint.as_deref() == Some("declare `format: ess/27`")),
        "{text}"
    );

    // Any other `payload.` path keeps the refusal it always had.
    let errors = refused(&flat("ess/26", "payload.demo.nested.Nothing.slot_id"));
    assert!(
        errors
            .as_slice()
            .iter()
            .all(|error| error.code == ValidationCode::UnobservableFact),
        "{errors}"
    );
}

/// An entity with a struct field named `payload`, read by a one-row read at `format`.
fn enveloped(format: &str, member: &str) -> String {
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
      - {{name: body, type: Optional<String>}}
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
                where: {{all: [payload.{member} == input.topic, state == Kept]}}
                field: payload
"
    )
}

#[test]
fn a_selector_over_an_entity_with_a_payload_field_keeps_its_meaning() {
    // `payload.topic` is the note's own field, at either format, and never the published value.
    for format in ["ess/26", "ess/27"] {
        let errors = refused(&enveloped(format, "topic"));
        // `field: payload` reads the struct into an `Optional<String>`: the type is the one
        // refusal, which proves the selector itself typed `payload.topic` as the note's field.
        let only = alone(&errors, ValidationCode::TypeMismatch);
        assert!(rendered(only).contains("demo.env.Note.payload"), "{errors}");
    }
    for format in ["ess/26", "ess/27"] {
        let body = enveloped(format, "topic").replace("field: payload", "field: note_id");
        let body = body.replace(
            "{name: body, type: Optional<String>}",
            "{name: body, type: Optional<demo.env.NoteId>}",
        );
        admitted(&body);
    }
    // A member the struct does not declare is the predicate checker's own refusal, naming the
    // envelope, not a published-value refusal.
    let body = enveloped("ess/27", "Read.topic")
        .replace("field: payload", "field: note_id")
        .replace(
            "{name: body, type: Optional<String>}",
            "{name: body, type: Optional<demo.env.NoteId>}",
        );
    let errors = refused(&body);
    let only = alone(&errors, ValidationCode::UnobservableFact);
    assert!(rendered(only).contains("has no field `Read`"), "{errors}");
}

#[test]
fn the_operand_in_a_row_set_guard_is_refused() {
    let errors = refused(&model(Edit {
        guard:
            "{all: [group_id == input.group_id, slot_id == payload.demo.nested.PairMade.slot_id]}",
        ..Edit::default()
    }));
    let only = alone(&errors, ValidationCode::UnsupportedConstruct);
    let text = rendered(only);
    assert!(text.contains("outcomes.empty.when_related"), "{text}");
    assert!(text.contains(OPERAND), "{text}");
    assert!(text.contains("guard"), "{text}");
    assert!(text.contains("published"), "{text}");

    // `forall:` is a guard predicate too.
    let errors = refused(&model(Edit::default()).replace(
        "          exists: false\n",
        "          forall: slot_id == payload.demo.nested.PairMade.slot_id\n",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedConstruct
                && rendered(error).contains(OPERAND)),
        "{errors}"
    );
    assert!(
        with_code(&errors, ValidationCode::UnobservableFact).is_empty(),
        "{errors}"
    );
}

#[test]
fn the_operand_in_an_input_guard_is_refused() {
    let guarded = |body: String| {
        body.replace(
            "          exists: false\n        error: demo.nested.Empty\n",
            "          exists: false\n        when: payload.demo.nested.PairMade.left_key == group_id\n        error: demo.nested.Empty\n",
        )
    };
    let naming = |errors: &ValidationErrors| -> ValidationError {
        let named: Vec<&ValidationError> = errors
            .as_slice()
            .iter()
            .filter(|error| rendered(error).contains("payload.demo.nested.PairMade.left_key"))
            .collect();
        let [only] = named.as_slice() else {
            panic!("one refusal names the operand:\n{errors}");
        };
        (*only).clone()
    };
    let errors = refused(&guarded(model(Edit::default())));
    let only = naming(&errors);
    assert_eq!(only.code, ValidationCode::UnsupportedConstruct, "{errors}");
    let text = rendered(&only);
    assert!(text.contains("outcomes.empty.when"), "{text}");
    assert!(text.contains("guard"), "{text}");

    // Below `ess/27` there is no operand: the root stays unknown, as it always was.
    let errors = refused(&guarded(flat("ess/26", "slot_id")));
    assert_eq!(
        naming(&errors).code,
        ValidationCode::UnobservableFact,
        "{errors}"
    );
}

#[test]
fn the_operand_in_an_affects_where_is_refused() {
    let affects = "        affects:
          - entity: demo.nested.Membership
            where: slot_id == payload.demo.nested.SlotClosed.slot_id
            moves: demo.nested.Membership.remove
";
    let body = model(Edit::default()).replace(
        "          demo.nested.SlotClosed: {slot_id: input.slot_id}\n",
        &format!("          demo.nested.SlotClosed: {{slot_id: input.slot_id}}\n{affects}"),
    );
    let errors = refused(&body);
    let only = alone(&errors, ValidationCode::UnsupportedConstruct);
    let text = rendered(only);
    assert!(text.contains("affects[0]"), "{text}");
    assert!(
        text.contains("payload.demo.nested.SlotClosed.slot_id"),
        "{text}"
    );
    assert!(text.contains("effect"), "{text}");

    // The same entry selecting by the input is admitted, so the operand is the one refusal.
    admitted(&body.replace("payload.demo.nested.SlotClosed.slot_id", "subject.slot_id"));
}

#[test]
fn an_operand_reading_a_mint_is_refused() {
    let errors = refused(&model(Edit {
        slot: "              {generated: true}\n",
        ..Edit::default()
    }));
    let refusals = with_code(&errors, ValidationCode::UnsupportedConstruct);
    assert_eq!(refusals.len(), 2, "one per reading field:\n{errors}");
    assert_eq!(errors.as_slice().len(), 2, "{errors}");
    for refusal in refusals {
        let text = rendered(refusal);
        assert!(text.contains(OPERAND), "{text}");
        assert!(text.contains("mint"), "{text}");
    }

    // A fallback to a mint is a mint where the caller sends nothing.
    let errors = refused(
        &model(Edit {
            slot: "              {input: slot_id, else: {generated: true}}\n",
            ..Edit::default()
        })
        .replace(
            "      - {name: group_id, type: String}\n    outcomes:\n      - name: empty",
            "      - {name: group_id, type: String}\n      - {name: slot_id, type: Optional<demo.nested.SlotId>}\n    outcomes:\n      - name: empty",
        ),
    );
    assert_eq!(
        with_code(&errors, ValidationCode::UnsupportedConstruct).len(),
        2,
        "{errors}"
    );

    // The created identity is published from the input, which is what the reader reads.
    let errors = refused(&model(Edit {
        left: "payload.demo.nested.PairMade.pair_id",
        ..Edit::default()
    }));
    let refusals = with_code(&errors, ValidationCode::UnsupportedConstruct);
    let [only] = refusals.as_slice() else {
        panic!("one refusal for the created identity:\n{errors}");
    };
    assert!(
        only.hint
            .as_deref()
            .is_some_and(|hint| hint.contains("input.pair_id")),
        "{errors}"
    );
}

#[test]
fn an_operand_reading_an_event_the_outcome_does_not_fill_is_refused() {
    let errors = refused(&model(Edit {
        left: "payload.demo.nested.SlotOpened.slot_id",
        ..Edit::default()
    }));
    let only = alone(&errors, ValidationCode::UndeclaredReference);
    let text = rendered(only);
    assert!(
        text.contains("payload.demo.nested.PairMade.left_key"),
        "{text}"
    );
    assert!(text.contains("demo.nested.SlotOpened"), "{text}");
    assert!(
        only.hint
            .as_deref()
            .is_some_and(|hint| hint.contains("demo.nested.PairMade")),
        "{text}"
    );

    // A field the event does not carry.
    let errors = refused(&model(Edit {
        left: "payload.demo.nested.PairMade.nothing",
        ..Edit::default()
    }));
    let only = alone(&errors, ValidationCode::UndeclaredReference);
    assert!(rendered(only).contains("nothing"), "{errors}");

    // A field the event carries and this outcome gives no source.
    let body = model(Edit::default()).replace(
        "            pair_id: input.pair_id\n            slot_id:\n",
        "            pair_id: input.pair_id\n            unused_slot:\n",
    );
    let body = body.replace(
        "      - {name: right_key, type: String}\n",
        "      - {name: right_key, type: String}\n      - {name: unused_slot, type: demo.nested.SlotId}\n",
    );
    let errors = refused(&body);
    assert!(
        !with_code(&errors, ValidationCode::UndeclaredReference).is_empty()
            && errors.to_string().contains("no source"),
        "{errors}"
    );
}

/// The `left_key` reading `right_key` and back, or `left_key` reading itself.
fn cyclic(left: &str, right: &str) -> String {
    model(Edit::default())
        .replace(
            "slot_id == payload.demo.nested.PairMade.slot_id, side == Left",
            &format!("element_key == {left}, side == Left"),
        )
        .replace(
            "slot_id == payload.demo.nested.PairMade.slot_id, side == Right",
            &format!("element_key == {right}, side == Right"),
        )
}

#[test]
fn a_cycle_between_two_payload_fields_is_refused_as_self_reference() {
    let errors = refused(&cyclic(
        "payload.demo.nested.PairMade.right_key",
        "payload.demo.nested.PairMade.left_key",
    ));
    let only = alone(&errors, ValidationCode::SelfReference);
    let text = rendered(only);
    assert!(
        text.contains("payload.demo.nested.PairMade.left_key"),
        "{text}"
    );
    assert!(
        text.contains(
            "`demo.nested.PairMade.left_key` reads `demo.nested.PairMade.right_key`, which reads \
             `demo.nested.PairMade.left_key`"
        ),
        "{text}"
    );
}

#[test]
fn a_field_whose_selector_reads_itself_is_refused() {
    let errors = refused(&cyclic(
        "payload.demo.nested.PairMade.left_key",
        "input.group_id",
    ));
    let only = alone(&errors, ValidationCode::SelfReference);
    let text = rendered(only);
    assert!(
        text.contains("payload.demo.nested.PairMade.left_key"),
        "{text}"
    );
    assert!(
        text.contains("`demo.nested.PairMade.left_key` reads `demo.nested.PairMade.left_key`"),
        "{text}"
    );
}

#[test]
fn the_operand_in_an_error_payload_is_refused() {
    let errors = refused(&model(Edit {
        error: "        error: demo.nested.EmptyKeyed
        payload:
          demo.nested.EmptyKeyed:
            key:
              related:
                entity: demo.nested.Membership
                where: slot_id == payload.demo.nested.PairMade.slot_id
                field: element_key
",
        ..Edit::default()
    }));
    let only = alone(&errors, ValidationCode::UnsupportedConstruct);
    let text = rendered(only);
    assert!(text.contains("outcomes.empty.payload"), "{text}");
    assert!(text.contains(OPERAND), "{text}");
}
