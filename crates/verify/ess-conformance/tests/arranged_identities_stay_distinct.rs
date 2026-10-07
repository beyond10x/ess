//! A witness drawn for a guard on the identity stays unique per arranged instance
//! (beyond10x/ess#480).
//!
//! The issue's `catalog` reproducer: `CreateItem` refuses an id with `item_id.utf8_bytes < 1`, and
//! `AddNote answers added` arranges three items. On 0.55.0 all three were created as `é`: the text a
//! byte-length guard is first tried at (`wide_first`) is the same literal at every witness
//! distinction, so every arranged instance sent it, and a target with unique identities answered
//! the second and third creation with its existing-identity branch. Written `item_id == ""`, the ids
//! were `item_id-138599`, `item_id-138600`, `item_id-138601`.
//!
//! The class is every guard-derived value that does not move with the distinction. Measured on
//! 0.55.0 with the same reproducer: `{item_id: {equals_ignore_case: reserved}}` sends `xeserved`
//! three times (the case-insensitive refutation tried first); `item_id.count > 10` sends
//! `item_id-13` and `item_id.utf8_bytes > 10` sends U+1F600 then `item_i` three times, because the
//! witness of those distinctions is longer than the bound and its shortened text loses the part
//! that differs.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The issue's `system.yaml`.
const SYSTEM: &str = "format: ess/22
system: catalog
version: v1

domains:
  - catalog.items
";

/// The issue's `domains/items.yaml`, its one guard line replaced by `{GUARD}`.
const DOMAIN: &str = r"domain: catalog.items

summary: Items, each kept under an id of 1 to 170 UTF-8 bytes.

naming:
  wire: items
  display: Items

types:
  - name: catalog.items.ItemId
    kind: newtype
    of: String

  - name: catalog.items.NoteId
    kind: newtype
    of: Uuid

entities:
  - name: catalog.items.Note
    identity:
      name: note_id
      type: catalog.items.NoteId
    fields:
      - name: item_id
        type: catalog.items.ItemId
      - name: text
        type: String
    lifecycle:
      initial: Kept
      states: [Kept]
      terminal: [Kept]
      transitions: []

  - name: catalog.items.Item
    identity:
      name: item_id
      type: catalog.items.ItemId
    fields:
      - name: title
        type: String
    relations:
      - name: notes
        kind: owns
        target: catalog.items.Note
        cardinality: many
        via: item_id
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed

errors:
  - name: catalog.items.ItemNotFound
    summary: No item has this id.
    fields:
      - name: item_id
        type: catalog.items.ItemId

  - name: catalog.items.ItemClosedError
    summary: The item is closed.
    fields:
      - name: item_id
        type: catalog.items.ItemId

  - name: catalog.items.ItemExists
    summary: An item already has this id.
    fields:
      - name: item_id
        type: catalog.items.ItemId

  - name: catalog.items.InvalidItemId
    summary: The item id is empty or longer than 170 UTF-8 bytes.
    fields:
      - name: item_id
        type: catalog.items.ItemId

commands:
  - name: catalog.items.CreateItem
    naming:
      wire: create-item
      display: Create an item
    input:
      - name: item_id
        type: catalog.items.ItemId
      - name: title
        type: String
    outcomes:
      - name: invalid-item-id
        when:
          any:
            - {GUARD}
        error: catalog.items.InvalidItemId
        payload:
          catalog.items.InvalidItemId:
            item_id: input.item_id
        summary: The id cannot be kept, so nothing was created.
      - name: created
        creates: catalog.items.Item
        instance: item_id
        sets:
          title: input.title
        emits:
          - catalog.items.ItemCreated
        payload:
          catalog.items.ItemCreated:
            item_id: input.item_id
            title: input.title

  - name: catalog.items.CloseItem
    naming:
      wire: close-item
      display: Close an item
    input:
      - name: item_id
        type: catalog.items.ItemId
    outcomes:
      - name: closed
        moves: catalog.items.Item.close
        instance: item_id
        emits:
          - catalog.items.ItemClosed
        payload:
          catalog.items.ItemClosed:
            item_id: input.item_id

  - name: catalog.items.AddNote
    naming:
      wire: add-note
      display: Add a note to an item
    input:
      - name: item_id
        type: catalog.items.ItemId
      - name: text
        type: String
    outcomes:
      - name: no-such-item
        when_related: {via: input.item_id, exists: false}
        error: catalog.items.ItemNotFound
        payload:
          catalog.items.ItemNotFound:
            item_id: input.item_id
      - name: item-closed
        when_related: {via: input.item_id, predicate: {state: [Closed]}}
        error: catalog.items.ItemClosedError
        payload:
          catalog.items.ItemClosedError:
            item_id: input.item_id
      - name: added
        creates: catalog.items.Note
        instance: note_id
        sets:
          item_id: input.item_id
          text: input.text
        emits:
          - catalog.items.NoteAdded
        payload:
          catalog.items.NoteAdded:
            note_id: {generated: true}
            item_id: input.item_id

events:
  - name: catalog.items.NoteAdded
    fields:
      - name: note_id
        type: catalog.items.NoteId
      - name: item_id
        type: catalog.items.ItemId

  - name: catalog.items.ItemClosed
    fields:
      - name: item_id
        type: catalog.items.ItemId

  - name: catalog.items.ItemCreated
    fields:
      - name: item_id
        type: catalog.items.ItemId
      - name: title
        type: String

views:
  - name: catalog.items.Items
    source: catalog.items.Item
    consistency: read_your_writes
    fields:
      - name: item_id
        type: catalog.items.ItemId
      - name: state
        type: catalog.items.Item.State
      - name: title
        type: String
    naming:
      wire: items
      display: Items

  - name: catalog.items.Notes
    source: catalog.items.Note
    consistency: read_your_writes
    fields:
      - name: note_id
        type: catalog.items.NoteId
      - name: state
        type: catalog.items.Note.State
      - name: item_id
        type: catalog.items.ItemId
    naming:
      wire: notes
      display: Notes
";

const ADDED: &str = "catalog.items.AddNote/outcome/added";

/// One guard on `CreateItem`'s id, and whether an id satisfies it.
struct Form {
    guard: &'static str,
    holds: fn(&str) -> bool,
}

fn under_one_byte(id: &str) -> bool {
    id.is_empty()
}

fn empty(id: &str) -> bool {
    id.is_empty()
}

fn reserved(id: &str) -> bool {
    id.eq_ignore_ascii_case("reserved")
}

fn over_ten_characters(id: &str) -> bool {
    id.chars().count() > 10
}

fn over_ten_bytes(id: &str) -> bool {
    id.len() > 10
}

/// The two forms the issue compares.
fn issue_forms() -> Vec<Form> {
    vec![
        Form {
            guard: "item_id.utf8_bytes < 1",
            holds: under_one_byte,
        },
        Form {
            guard: r#"item_id == """#,
            holds: empty,
        },
    ]
}

/// A guard-derived value tried ahead of the distinction's own witness, which that witness
/// still refutes the guard beside.
fn folded_form() -> Form {
    Form {
        guard: "{item_id: {equals_ignore_case: reserved}}",
        holds: reserved,
    }
}

/// Bounds the distinction's own witness breaks, so the id sent is a shortened text.
fn bounded_forms() -> Vec<Form> {
    vec![
        Form {
            guard: "item_id.count > 10",
            holds: over_ten_characters,
        },
        Form {
            guard: "item_id.utf8_bytes > 10",
            holds: over_ten_bytes,
        },
    ]
}

fn ir(guard: &str) -> EssIr {
    let domain = DOMAIN.replace("{GUARD}", guard);
    assert!(!domain.contains("{GUARD}"), "the guard is substituted");
    let spec = Specification::assemble([
        (
            Source::new("system.yaml"),
            RawSpecFile::parse(SYSTEM).unwrap_or_else(|error| panic!("{error}")),
        ),
        (
            Source::new("domains/items.yaml"),
            RawSpecFile::parse(&domain).unwrap_or_else(|error| panic!("{error}")),
        ),
    ])
    .unwrap_or_else(|errors| panic!("the reproducer validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn literal_text(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<String> {
    match input.get(field).and_then(|value| value.as_literal()) {
        Some(Node::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

/// The literal id of every `CreateItem` step of `scenario` that requires `created`, in order.
fn created_ids(scenario: &ConformanceScenario) -> Vec<String> {
    let mut found = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "catalog.items.CreateItem" {
            continue;
        }
        let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
            continue;
        };
        if outcome.outcome.to_string() != "created" {
            continue;
        }
        let id = literal_text(input, "item_id")
            .unwrap_or_else(|| panic!("`CreateItem` is sent a literal id: {input:?}"));
        found.push(id);
    }
    found
}

fn scenario<'s>(synthesis: &'s Synthesis, id: &str) -> Option<&'s ConformanceScenario> {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

fn refused(synthesis: &Synthesis, id: &str) -> bool {
    synthesis.refusals.iter().any(|refusal| {
        refusal
            .scenario
            .as_ref()
            .is_some_and(|scenario| scenario.to_string() == id)
    })
}

/// Every scenario of the suite creates each id it sends `CreateItem` at most once, and each id
/// refutes the guard; a scenario that cannot keep its items apart is refused rather than filed.
/// Returns how many scenarios created more than one item.
fn no_identity_created_twice(form: &Form) -> usize {
    let synthesis = synthesize(&ir(form.guard));
    let mut several = 0;
    for (id, scenario) in &synthesis.suite.scenarios {
        let ids = created_ids(scenario);
        let distinct: BTreeSet<&String> = ids.iter().collect();
        assert_eq!(
            distinct.len(),
            ids.len(),
            "{}: `{id}` creates one item id more than once: {ids:?}",
            form.guard
        );
        for sent in &ids {
            assert!(
                !(form.holds)(sent),
                "{}: `{id}` requires `created` for `{sent}`, which the guard claims",
                form.guard
            );
        }
        if ids.len() > 1 {
            several += 1;
        }
    }
    assert!(
        scenario(&synthesis, ADDED).is_some() || refused(&synthesis, ADDED),
        "{}: `{ADDED}` is filed or refused",
        form.guard
    );
    several
}

/// The issue's acceptance: three arranged items, three ids, each refuting the guard.
#[test]
fn add_note_arranges_three_items_under_three_identities() {
    for form in issue_forms() {
        let synthesis = synthesize(&ir(form.guard));
        let added = scenario(&synthesis, ADDED)
            .unwrap_or_else(|| panic!("{}: `{ADDED}` is synthesized", form.guard));
        let ids = created_ids(added);
        assert_eq!(
            ids.len(),
            3,
            "{}: three items are arranged: {ids:?}",
            form.guard
        );
        let distinct: BTreeSet<&String> = ids.iter().collect();
        assert_eq!(
            distinct.len(),
            3,
            "{}: each arranged item has its own id: {ids:?}",
            form.guard
        );
        for id in &ids {
            assert!(
                !(form.holds)(id),
                "{}: `{id}` refutes the guard, so `created` answers it",
                form.guard
            );
        }
    }
}

/// The issue's forms and a case-insensitive guard, across the whole suite: the witness each
/// distinction draws keeps them apart.
#[test]
fn no_scenario_creates_one_identity_twice() {
    for form in issue_forms().into_iter().chain([folded_form()]) {
        let several = no_identity_created_twice(&form);
        assert!(
            several > 0,
            "{}: some scenario arranges more than one item, or this test checks nothing",
            form.guard
        );
    }
}

/// Bounds the distinction's own witness breaks: the ids are kept apart, or the scenario is refused
/// rather than filed creating one id twice.
#[test]
fn no_scenario_creates_one_identity_twice_under_a_bound_the_witness_breaks() {
    for form in bounded_forms() {
        no_identity_created_twice(&form);
    }
}
