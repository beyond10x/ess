//! The head of an ordered selection as a value source, `{related: {entity, where, order_by,
//! first}}` (`ess/25`, beyond10x/ess#543, `docs/design/ordered-related-head.md`).
//!
//! What validation decides: the shape is recognised by its list-valued `order_by:` and admitted
//! from `ess/25`; the order is total, ending in the entity's identity and nothing after it; every
//! key is a bare, required, orderable field of the entity; `first:` replaces `field:`, never stands
//! beside it, and is never written without an order.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("ranked.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

/// A slot of ranked memberships and a command pairing the best-ranked one; `@SETS@` and `@PAYLOAD@`
/// mark where [`model`] writes the `picked` outcome's entries.
const MODEL: &str = "format: @FORMAT@
system: demo
version: v1
domain: demo.ranked
types:
  - {name: demo.ranked.MembershipId, kind: newtype, of: Uuid}
  - {name: demo.ranked.PairId, kind: newtype, of: Uuid}
  - {name: demo.ranked.Score, kind: newtype, of: Integer}
  - name: demo.ranked.Selector
    kind: struct
    fields:
      - {name: entity, type: String}
      - {name: where, type: String}
      - {name: order_by, type: String}
      - {name: first, type: String}
  - name: demo.ranked.Reference
    kind: struct
    fields:
      - {name: related, type: demo.ranked.Selector}
entities:
  - name: demo.ranked.Membership
    identity: {name: membership_id, type: demo.ranked.MembershipId}
    fields:
      - {name: slot_id, type: String}
      - {name: element_key, type: String}
      - {name: element_score, type: demo.ranked.Score}
      - {name: joined_seq, type: Integer}
      - {name: note, type: Optional<String>}
      - {name: waited, type: Duration}
    lifecycle:
      initial: Present
      states: [Present, Removed]
      terminal: [Removed]
      transitions:
        - {name: remove, from: [Present], to: Removed}
  - name: demo.ranked.Pair
    identity: {name: pair_id, type: demo.ranked.PairId}
    fields:
      - {name: slot_id, type: String}
      - {name: element_key, type: String}
      - {name: element_score, type: Optional<demo.ranked.Score>}
      - {name: reference, type: Optional<demo.ranked.Reference>}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Open], to: Done}
errors:
  - {name: demo.ranked.Empty, summary: No element is waiting in the slot., fields: []}
events:
  - name: demo.ranked.Joined
    fields:
      - {name: membership_id, type: demo.ranked.MembershipId}
  - name: demo.ranked.Paired
    fields:
      - {name: pair_id, type: demo.ranked.PairId}
      - {name: element_key, type: String}
  - name: demo.ranked.Left
    fields:
      - {name: membership_id, type: demo.ranked.MembershipId}
  - name: demo.ranked.Finished
    fields:
      - {name: pair_id, type: demo.ranked.PairId}
commands:
  - name: demo.ranked.Leave
    input:
      - {name: membership_id, type: demo.ranked.MembershipId}
    outcomes:
      - name: left
        moves: demo.ranked.Membership.remove
        instance: membership_id
        emits: [demo.ranked.Left]
        payload:
          demo.ranked.Left: {membership_id: input.membership_id}
  - name: demo.ranked.Finish
    input:
      - {name: pair_id, type: demo.ranked.PairId}
    outcomes:
      - name: finished
        moves: demo.ranked.Pair.finish
        instance: pair_id
        emits: [demo.ranked.Finished]
        payload:
          demo.ranked.Finished: {pair_id: input.pair_id}
  - name: demo.ranked.Join
    input:
      - {name: membership_id, type: demo.ranked.MembershipId}
      - {name: slot_id, type: String}
      - {name: element_key, type: String}
      - {name: element_score, type: demo.ranked.Score}
      - {name: joined_seq, type: Integer}
      - {name: waited, type: Duration}
    outcomes:
      - name: joined
        creates: demo.ranked.Membership
        instance: membership_id
        sets:
          slot_id: input.slot_id
          element_key: input.element_key
          element_score: input.element_score
          joined_seq: input.joined_seq
          waited: input.waited
        emits: [demo.ranked.Joined]
        payload:
          demo.ranked.Joined: {membership_id: input.membership_id}
  - name: demo.ranked.Pick
    input:
      - {name: pair_id, type: demo.ranked.PairId}
      - {name: slot_id, type: String}
    outcomes:
      - name: empty
        when_related:
          entity: demo.ranked.Membership
          where: {all: [slot_id == input.slot_id, state == Present]}
          exists: false
        error: demo.ranked.Empty
      - name: picked
        creates: demo.ranked.Pair
        instance: pair_id
        sets:
          slot_id: input.slot_id
@SETS@        emits: [demo.ranked.Paired]
        payload:
          demo.ranked.Paired:
            pair_id: input.pair_id
@PAYLOAD@";

/// [`MODEL`] at `format`, with `sets` under the `picked` outcome's `sets:` and `payload` under
/// its `Paired` event.
fn model(format: &str, sets: &str, payload: &str) -> String {
    MODEL
        .replace("@FORMAT@", format)
        .replace("@SETS@", sets)
        .replace("@PAYLOAD@", payload)
}

/// `{related: {entity: Membership, where: <the slot's present rows>, order_by: <keys>, first:
/// <field>}}` as a `sets:` entry for `target`.
fn head_in_sets(target: &str, order_by: &str, first: &str) -> String {
    format!(
        "          {target}:
            related:
              entity: demo.ranked.Membership
              where: {{all: [slot_id == input.slot_id, state == Present]}}
              order_by: {order_by}
              first: {first}
"
    )
}

/// The same source under the `Paired` event's `element_key`.
fn head_in_payload(order_by: &str, first: &str) -> String {
    format!(
        "            element_key:
              related:
                entity: demo.ranked.Membership
                where: {{all: [slot_id == input.slot_id, state == Present]}}
                order_by: {order_by}
                first: {first}
"
    )
}

const GENERATED_KEY: &str = "            element_key: {generated: true}\n";

/// `element_key` in `sets:` read from the head by `order_by`, the event's field minted.
fn sets_only(format: &str, order_by: &str) -> String {
    model(
        format,
        &head_in_sets("element_key", order_by, "element_key"),
        GENERATED_KEY,
    )
}

#[test]
fn an_ordered_head_source_validates_at_ess_25() {
    let order = "[element_score desc, membership_id]";
    let body = model(
        "ess/25",
        &head_in_sets("element_key", order, "element_key"),
        &head_in_payload(order, "element_key"),
    );
    spec(&body).unwrap_or_else(|errors| panic!("{errors}"));

    // The identity alone is a total order, in either direction, and every spelling of a
    // direction a view admits is admitted here.
    for order in [
        "[membership_id]",
        "[membership_id desc]",
        "[element_score descending, joined_seq ascending, membership_id asc]",
        "[state, slot_id, membership_id]",
    ] {
        spec(&sets_only("ess/25", order)).unwrap_or_else(|errors| panic!("{order}: {errors}"));
    }

    // `first:` reads the identity, or a field into an `Optional<…>` target.
    let body = model(
        "ess/25",
        &(head_in_sets("element_key", order, "element_key")
            + &head_in_sets("element_score", order, "element_score")),
        GENERATED_KEY,
    );
    spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn an_order_without_a_unique_tie_break_is_refused() {
    let errors = refused(&sets_only("ess/25", "[element_score desc, joined_seq]"));
    assert!(errors.contains("missing_declaration"), "{errors}");
    assert!(errors.contains("sets.element_key"), "{errors}");
    assert!(
        errors.contains("append `membership_id` as the last key"),
        "{errors}"
    );

    let errors = refused(&sets_only("ess/25", "[element_score desc]"));
    assert!(
        errors.contains("append `membership_id` as the last key"),
        "{errors}"
    );
}

#[test]
fn first_beside_field_is_refused() {
    let body = model(
        "ess/25",
        "          element_key:
            related:
              entity: demo.ranked.Membership
              where: {all: [slot_id == input.slot_id, state == Present]}
              order_by: [element_score desc, membership_id]
              first: element_key
              field: element_key
",
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("conflicting_declaration"), "{errors}");
    assert!(errors.contains("sets.element_key"), "{errors}");
    assert!(errors.contains("`first:`"), "{errors}");
    assert!(errors.contains("`field:`"), "{errors}");

    // An order read with `field:` says which contract it wants by the key it lacks.
    let body = model(
        "ess/25",
        "          element_key:
            related:
              entity: demo.ranked.Membership
              where: {all: [slot_id == input.slot_id, state == Present]}
              order_by: [element_score desc, membership_id]
              field: element_key
",
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("conflicting_declaration"), "{errors}");
    assert!(errors.contains("`first:`"), "{errors}");
}

#[test]
fn the_head_shape_below_ess_25_is_refused_naming_ess_25() {
    let errors = refused(&sets_only("ess/24", "[element_score desc, membership_id]"));
    assert!(errors.contains("unsupported_format_version"), "{errors}");
    assert!(errors.contains("ess/25"), "{errors}");
    assert!(errors.contains("sets.element_key"), "{errors}");

    // Below `ess/22` the selector itself is no source; the shape stays refused.
    refused(&sets_only("ess/21", "[element_score desc, membership_id]"));
}

/// A text `order_by:` is not the shape: the mapping stays the nested one it always was, at every
/// format, including `ess/25`.
#[test]
fn a_text_order_by_keeps_its_nested_mapping_meaning() {
    for format in ["ess/24", "ess/25"] {
        let body = model(
            format,
            "          reference:
            related:
              entity: demo.ranked.Membership
              where: slot_id
              order_by: element_score desc
              first: element_key
",
            GENERATED_KEY,
        );
        spec(&body).unwrap_or_else(|errors| panic!("{format}: {errors}"));
    }
}

#[test]
fn an_optional_order_key_is_refused() {
    let errors = refused(&sets_only("ess/25", "[note, membership_id]"));
    assert!(errors.contains("type_mismatch"), "{errors}");
    assert!(errors.contains("order_by[0]"), "{errors}");
    assert!(errors.contains("absence has no rank"), "{errors}");
}

#[test]
fn an_unorderable_key_is_refused() {
    let errors = refused(&sets_only("ess/25", "[waited, membership_id]"));
    assert!(errors.contains("type_mismatch"), "{errors}");
    assert!(errors.contains("order_by[0]"), "{errors}");
    assert!(errors.contains("Duration"), "{errors}");
}

#[test]
fn a_key_after_the_identity_is_refused() {
    let errors = refused(&sets_only("ess/25", "[membership_id, element_score]"));
    assert!(errors.contains("unsupported_construct"), "{errors}");
    assert!(errors.contains("order_by[1]"), "{errors}");
    assert!(errors.contains("never decide"), "{errors}");
}

#[test]
fn first_without_order_by_is_refused() {
    let body = model(
        "ess/25",
        "          element_key:
            related:
              entity: demo.ranked.Membership
              where: {all: [slot_id == input.slot_id, state == Present]}
              first: element_key
",
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("missing_declaration"), "{errors}");
    assert!(errors.contains("sets.element_key"), "{errors}");
    assert!(errors.contains("unordered selection"), "{errors}");
}

#[test]
fn an_order_key_the_entity_does_not_hold_is_refused() {
    let errors = refused(&sets_only("ess/25", "[priority desc, membership_id]"));
    assert!(errors.contains("undeclared_reference"), "{errors}");
    assert!(errors.contains("order_by[0]"), "{errors}");
    assert!(errors.contains("priority"), "{errors}");
    assert!(errors.contains("membership_id"), "{errors}");

    let errors = refused(&sets_only("ess/25", "[slot_id.length, membership_id]"));
    assert!(errors.contains("undeclared_reference"), "{errors}");
    assert!(errors.contains("order_by[0]"), "{errors}");
}

#[test]
fn an_empty_order_is_refused() {
    let errors = refused(&sets_only("ess/25", "[]"));
    assert!(errors.contains("empty_declaration"), "{errors}");
    assert!(errors.contains("sets.element_key"), "{errors}");
}

#[test]
fn a_key_that_does_not_parse_is_refused() {
    let errors = refused(&sets_only(
        "ess/25",
        "[element_score upward, membership_id]",
    ));
    assert!(errors.contains("type_mismatch"), "{errors}");
    assert!(errors.contains("order_by[0]"), "{errors}");

    let errors = refused(&sets_only(
        "ess/25",
        "[element_score desc first, membership_id]",
    ));
    assert!(errors.contains("type_mismatch"), "{errors}");
}

#[test]
fn a_key_written_twice_is_refused() {
    let errors = refused(&sets_only(
        "ess/25",
        "[element_score, element_score desc, membership_id]",
    ));
    assert!(errors.contains("duplicate_declaration"), "{errors}");
    assert!(errors.contains("order_by[1]"), "{errors}");
}

#[test]
fn first_naming_no_field_or_the_wrong_type_is_refused() {
    let body = model(
        "ess/25",
        &head_in_sets("element_key", "[membership_id]", "label"),
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("undeclared_reference"), "{errors}");
    assert!(errors.contains("label"), "{errors}");

    let body = model(
        "ess/25",
        &head_in_sets("element_key", "[membership_id]", "joined_seq"),
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("type_mismatch"), "{errors}");
    assert!(errors.contains("joined_seq"), "{errors}");
}

#[test]
fn the_selector_rules_of_a_filtered_read_apply() {
    // A selector over no declared entity.
    let body = model(
        "ess/25",
        "          element_key:
            related:
              entity: demo.ranked.Nothing
              where: {all: [slot_id == input.slot_id]}
              order_by: [membership_id]
              first: element_key
",
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("undeclared_reference"), "{errors}");
    assert!(errors.contains("demo.ranked.Nothing"), "{errors}");

    // A selector reading a field the row does not hold.
    let body = model(
        "ess/25",
        "          element_key:
            related:
              entity: demo.ranked.Membership
              where: {all: [lane == input.slot_id]}
              order_by: [membership_id]
              first: element_key
",
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("lane"), "{errors}");
}

#[test]
fn order_by_beside_via_is_refused() {
    let body = model(
        "ess/25",
        "          element_key:
            related:
              via: input.slot_id
              field: element_key
              order_by: [membership_id]
",
        GENERATED_KEY,
    );
    let errors = refused(&body);
    assert!(errors.contains("`via:`"), "{errors}");
    assert!(errors.contains("order_by"), "{errors}");
}
