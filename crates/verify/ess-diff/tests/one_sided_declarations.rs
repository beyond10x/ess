//! A declaration present in one revision only is named by `<family>/<name>/added|removed` alone
//! (beyond10x/ess#276).
//!
//! Every `*_changes` comparator already reports a key found in one map only. The residual used to
//! see one revision at a time, so whatever a one-sided declaration carried beyond the keys a
//! two-sided comparison accounts for (an actor's `attributes`, a field's flags) also raised
//! `system/<system>/unclassified-changed` and whole obligations. One test per family, each in both
//! directions, plus the edit the residual must still catch: an actor's `attributes` changed on an
//! actor both revisions declare.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const SYSTEM: &str = r"
format: ess/16
system: gap
version: v1
domains:
  - gap.desk

components:
  - component: desk-service
    owns:
      domains:
        - gap.desk
    accepts:
      commands:
        - gap.desk.OpenItem
    publishes:
      events:
        - gap.desk.ItemOpened
";

const DOMAIN: &str = r"
domain: gap.desk

types:
  - name: gap.desk.ItemId
    kind: newtype
    of: Uuid

entities:
  - name: gap.desk.Item
    identity:
      name: item_id
      type: gap.desk.ItemId
    fields:
      - name: title
        type: String
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed

views:
  - name: gap.desk.ItemById
    source: gap.desk.Item
    consistency: eventual
    fields:
      - name: item_id
        type: gap.desk.ItemId
      - name: title
        type: String

errors:
  - name: gap.desk.ItemConflict
    summary: The item is not in a state this command acts from.
    fields:
      - name: state
        type: gap.desk.Item.State

commands:
  - name: gap.desk.OpenItem
    input:
      - name: item_id
        type: gap.desk.ItemId
    outcomes:
      - name: opened
        emits:
          - gap.desk.ItemOpened
        payload:
          gap.desk.ItemOpened:
            item_id: input.item_id

  - name: gap.desk.CloseItem
    input:
      - name: item_id
        type: gap.desk.ItemId
    outcomes:
      - name: closed
        moves: gap.desk.Item.close
        instance: item_id
        emits:
          - gap.desk.ItemClosed
        payload:
          gap.desk.ItemClosed:
            item_id: input.item_id
      - name: wrong-state
        wrong_state: true
        error: gap.desk.ItemConflict

events:
  - name: gap.desk.ItemOpened
    fields:
      - name: item_id
        type: gap.desk.ItemId

  - name: gap.desk.ItemClosed
    fields:
      - name: item_id
        type: gap.desk.ItemId

actors:
  - name: gap.desk.Clerk
    attributes:
      - {name: clerk_id, type: String}
    may:
      - gap.desk.OpenItem
";

fn ir(system: &str, domain: &str) -> EssIr {
    let spec = Specification::assemble([
        (
            Source::new("system.yaml"),
            RawSpecFile::parse(system).unwrap_or_else(|error| panic!("{error}")),
        ),
        (
            Source::new("domains/desk.yaml"),
            RawSpecFile::parse(domain).unwrap_or_else(|error| panic!("{error}")),
        ),
    ])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn ids(before: &EssIr, after: &EssIr) -> Vec<String> {
    diff(before, after)
        .unwrap()
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

/// Adds `block` right after the `anchor` line of the domain document.
fn inserted(document: &str, anchor: &str, block: &str) -> String {
    assert!(document.contains(anchor), "anchor `{anchor}` is missing");
    document.replacen(anchor, &format!("{anchor}{block}"), 1)
}

/// The base and a revision that declares one more construct, compared both ways.
fn assert_one_sided(system: &str, domain: &str, family: &str, name: &str) {
    assert_all_one_sided(system, domain, &[(family, name)]);
}

/// As [`assert_one_sided`], for a declaration that brings derived ones with it (an entity's
/// `<Entity>.State` type), each named in the order the delta lists them.
fn assert_all_one_sided(system: &str, domain: &str, declared: &[(&str, &str)]) {
    let base = ir(SYSTEM, DOMAIN);
    let grown = ir(system, domain);
    let expected = |kind: &str| {
        declared
            .iter()
            .map(|(family, name)| format!("{family}/{name}/{kind}"))
            .collect::<Vec<_>>()
    };
    let added = ids(&base, &grown);
    assert_eq!(added, expected("added"), "{added:?}");
    let removed = ids(&grown, &base);
    assert_eq!(removed, expected("removed"), "{removed:?}");
}

#[test]
fn a_type_present_on_one_side_is_only_added_or_removed() {
    let domain = inserted(
        DOMAIN,
        "types:\n",
        "  - name: gap.desk.Money
    kind: struct
    fields:
      - {name: amount, type: Decimal}
      - {name: currency, type: Optional<String>}
    invariants:
      - amount >= 0
",
    );
    assert_one_sided(SYSTEM, &domain, "type", "gap.desk.Money");
}

#[test]
fn an_entity_present_on_one_side_is_only_added_or_removed() {
    let domain = inserted(
        DOMAIN,
        "entities:\n",
        "  - name: gap.desk.Note
    identity:
      name: note_id
      type: gap.desk.ItemId
    fields:
      - {name: body, type: Optional<String>}
    lifecycle:
      initial: Draft
      states: [Draft]
      terminal: [Draft]
",
    );
    assert_all_one_sided(
        SYSTEM,
        &domain,
        &[("type", "gap.desk.Note.State"), ("entity", "gap.desk.Note")],
    );
}

#[test]
fn a_command_present_on_one_side_is_only_added_or_removed() {
    let domain = inserted(
        DOMAIN,
        "commands:\n",
        "  - name: gap.desk.ReopenItem
    input:
      - name: item_id
        type: gap.desk.ItemId
      - {name: reason, type: Optional<String>}
    outcomes:
      - name: reopened
        emits:
          - gap.desk.ItemOpened
        payload:
          gap.desk.ItemOpened:
            item_id: input.item_id
      - name: locked
        external: the item is locked by another desk
        error: gap.desk.ItemConflict
",
    );
    assert_one_sided(SYSTEM, &domain, "command", "gap.desk.ReopenItem");
}

#[test]
fn an_event_present_on_one_side_is_only_added_or_removed() {
    let domain = inserted(
        DOMAIN,
        "events:\n",
        "  - name: gap.desk.ItemNoted
    naming:
      wire: item.noted.v1
    fields:
      - name: item_id
        type: gap.desk.ItemId
      - {name: note, type: Optional<String>}
",
    );
    assert_one_sided(SYSTEM, &domain, "event", "gap.desk.ItemNoted");
}

#[test]
fn an_error_present_on_one_side_is_only_added_or_removed() {
    let domain = inserted(
        DOMAIN,
        "errors:\n",
        "  - name: gap.desk.ItemLocked
    summary: The item is locked.
    fields:
      - {name: until, type: Optional<String>}
",
    );
    assert_one_sided(SYSTEM, &domain, "error", "gap.desk.ItemLocked");
}

#[test]
fn a_view_present_on_one_side_is_only_added_or_removed() {
    let domain = inserted(
        DOMAIN,
        "views:\n",
        "  - name: gap.desk.OpenItems
    source: gap.desk.Item
    consistency: eventual
    filter: state == Open
    fields:
      - name: item_id
        type: gap.desk.ItemId
      - name: title
        type: String
",
    );
    assert_one_sided(SYSTEM, &domain, "view", "gap.desk.OpenItems");
}

#[test]
fn an_actor_with_attributes_present_on_one_side_is_only_added_or_removed() {
    let domain = inserted(
        DOMAIN,
        "actors:\n",
        "  - name: gap.desk.Supervisor
    attributes:
      - {name: supervisor_id, type: String}
      - {name: team, type: String}
    may:
      - gap.desk.OpenItem
",
    );
    assert_one_sided(SYSTEM, &domain, "actor", "gap.desk.Supervisor");
}

#[test]
fn a_binding_present_on_one_side_is_only_added_or_removed() {
    let system = format!(
        "{SYSTEM}
bindings:
  - id: reopen-on-open
    summary: Open the item again the moment it opens.
    when:
      event: gap.desk.ItemOpened
    invoke:
      command: gap.desk.OpenItem
    mapping:
      item_id: event.item_id
    delivery: at_least_once
    on_failure: retry
"
    );
    assert_one_sided(&system, DOMAIN, "binding", "reopen-on-open");
}

/// `refs` is residual on a binding, so before #276 an added binding carrying it also reported
/// `unclassified-changed`; this is the binding case that fails on the base.
#[test]
fn a_binding_with_refs_present_on_one_side_is_only_added_or_removed() {
    let system = format!(
        "{SYSTEM}
bindings:
  - id: reopen-on-open
    summary: Open the item again the moment it opens.
    refs: [issue:example/gap#6]
    when:
      event: gap.desk.ItemOpened
    invoke:
      command: gap.desk.OpenItem
    mapping:
      item_id: event.item_id
    delivery: at_least_once
    on_failure: retry
"
    );
    assert_one_sided(&system, DOMAIN, "binding", "reopen-on-open");
}

#[test]
fn a_component_present_on_one_side_is_only_added_or_removed() {
    let system = inserted(
        SYSTEM,
        "components:\n",
        "  - component: desk-reader
    summary: Reads items.
    publishes:
      events:
        - gap.desk.ItemOpened
",
    );
    assert_one_sided(&system, DOMAIN, "component", "desk-reader");
}

#[test]
fn an_attributes_edit_on_an_actor_both_sides_declare_stays_unclassified() {
    let domain = DOMAIN.replace(
        "      - {name: clerk_id, type: String}\n",
        "      - {name: clerk_id, type: String}\n      - {name: desk, type: String}\n",
    );
    assert_ne!(domain, DOMAIN);
    let ids = ids(&ir(SYSTEM, DOMAIN), &ir(SYSTEM, &domain));
    assert_eq!(ids, ["system/gap/unclassified-changed"], "{ids:?}");
}
