//! Adversary pass 1 for beyond10x/ess#276: does the residual still catch what it must, now that it
//! drops declarations one revision carries alone?
mod support;

use std::collections::BTreeSet;

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
      states: [Open]
      terminal: [Open]

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

events:
  - name: gap.desk.ItemOpened
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

fn ir_of(documents: &[(&str, &str)]) -> EssIr {
    let spec = Specification::assemble(documents.iter().map(|(path, text)| {
        (
            Source::new(*path),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{path}: {error}")),
        )
    }))
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn ir(system: &str, domain: &str) -> EssIr {
    ir_of(&[("system.yaml", system), ("domains/desk.yaml", domain)])
}

fn ids(before: &EssIr, after: &EssIr) -> Vec<String> {
    diff(before, after)
        .unwrap()
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

fn sorted(ids: Vec<String>) -> Vec<String> {
    let mut ids = ids;
    ids.sort();
    ids
}

fn replaced(document: &str, from: &str, to: &str) -> String {
    assert!(document.contains(from), "`{from}` is missing");
    document.replacen(from, to, 1)
}

const UNCLASSIFIED: &str = "system/gap/unclassified-changed";

// ---- a two-sided declaration whose only change is unclassified -----------------------------------

#[test]
fn a_refs_edit_on_a_command_both_sides_declare_stays_unclassified() {
    let after = replaced(
        DOMAIN,
        "  - name: gap.desk.OpenItem\n",
        "  - name: gap.desk.OpenItem\n    refs: [issue:example/gap#1]\n",
    );
    let ids = ids(&ir(SYSTEM, DOMAIN), &ir(SYSTEM, &after));
    assert_eq!(ids, [UNCLASSIFIED], "{ids:?}");
}

#[test]
fn a_refs_edit_on_an_outcome_both_sides_declare_stays_unclassified() {
    let after = replaced(
        DOMAIN,
        "      - name: opened\n",
        "      - name: opened\n        refs: [issue:example/gap#2]\n",
    );
    let ids = ids(&ir(SYSTEM, DOMAIN), &ir(SYSTEM, &after));
    assert_eq!(ids, [UNCLASSIFIED], "{ids:?}");
}

#[test]
fn a_refs_edit_on_a_component_both_sides_declare_stays_unclassified() {
    let after = replaced(
        SYSTEM,
        "  - component: desk-service\n",
        "  - component: desk-service\n    refs: [issue:example/gap#3]\n",
    );
    let ids = ids(&ir(SYSTEM, DOMAIN), &ir(&after, DOMAIN));
    assert_eq!(ids, [UNCLASSIFIED], "{ids:?}");
}

/// The command family filter, exercised by content the residual keeps: on the base commit this
/// reported `unclassified-changed` beside `command/.../added`, so unlike a command without `refs`
/// it fails if `commands` is ever dropped from the one-sided filter.
#[test]
fn a_command_with_refs_present_on_one_side_is_only_added_or_removed() {
    let after = replaced(
        DOMAIN,
        "commands:\n",
        "commands:
  - name: gap.desk.TouchItem
    refs: [issue:example/gap#4]
    input:
      - name: item_id
        type: gap.desk.ItemId
    outcomes:
      - name: touched
        refs: [issue:example/gap#5]
        emits:
          - gap.desk.ItemOpened
        payload:
          gap.desk.ItemOpened:
            item_id: input.item_id
",
    );
    let (base, grown) = (ir(SYSTEM, DOMAIN), ir(SYSTEM, &after));
    assert_eq!(ids(&base, &grown), ["command/gap.desk.TouchItem/added"]);
    assert_eq!(ids(&grown, &base), ["command/gap.desk.TouchItem/removed"]);
}

/// As above, for the component family: a component's `refs` is residual content.
#[test]
fn a_component_with_refs_present_on_one_side_is_only_added_or_removed() {
    let after = replaced(
        SYSTEM,
        "components:\n",
        "components:
  - component: desk-reader
    refs: [issue:example/gap#6]
    publishes:
      events:
        - gap.desk.ItemOpened
",
    );
    let (base, grown) = (ir(SYSTEM, DOMAIN), ir(&after, DOMAIN));
    assert_eq!(ids(&base, &grown), ["component/desk-reader/added"]);
    assert_eq!(ids(&grown, &base), ["component/desk-reader/removed"]);
}

// ---- a rename that carries changed content ------------------------------------------------------

#[test]
fn a_renamed_actor_whose_attributes_also_changed_is_its_removal_and_its_addition() {
    let after = replaced(
        DOMAIN,
        "  - name: gap.desk.Clerk\n    attributes:\n      - {name: clerk_id, type: String}\n",
        "  - name: gap.desk.Teller\n    attributes:\n      - {name: teller_id, type: String}\n      - {name: till, type: String}\n",
    );
    let ids = sorted(ids(&ir(SYSTEM, DOMAIN), &ir(SYSTEM, &after)));
    assert_eq!(
        ids,
        [
            "actor/gap.desk.Clerk/removed",
            "actor/gap.desk.Teller/added"
        ],
        "{ids:?}"
    );
}

/// The renamed actor's attributes moved, and so did a two-sided actor's: the second is still
/// residual even though a one-sided pair stands beside it in the same family.
#[test]
fn a_rename_beside_a_two_sided_attributes_edit_still_reports_the_edit() {
    let before = replaced(
        DOMAIN,
        "actors:\n",
        "actors:\n  - name: gap.desk.Auditor\n    attributes:\n      - {name: auditor_id, type: String}\n    may:\n      - gap.desk.OpenItem\n",
    );
    let after = replaced(
        &replaced(
            DOMAIN,
            "actors:\n",
            "actors:\n  - name: gap.desk.Inspector\n    attributes:\n      - {name: inspector_id, type: String}\n    may:\n      - gap.desk.OpenItem\n",
        ),
        "      - {name: clerk_id, type: String}\n",
        "      - {name: clerk_id, type: String}\n      - {name: desk, type: String}\n",
    );
    let ids = sorted(ids(&ir(SYSTEM, &before), &ir(SYSTEM, &after)));
    assert_eq!(
        ids,
        [
            "actor/gap.desk.Auditor/removed",
            "actor/gap.desk.Inspector/added",
            UNCLASSIFIED,
        ],
        "{ids:?}"
    );
}

// ---- domain-level content ------------------------------------------------------------------------

#[test]
fn a_domain_naming_edit_stays_unclassified() {
    let after = replaced(
        DOMAIN,
        "domain: gap.desk\n",
        "domain: gap.desk\n\nnaming:\n  display: Front desk\n",
    );
    let ids = ids(&ir(SYSTEM, DOMAIN), &ir(SYSTEM, &after));
    assert_eq!(ids, [UNCLASSIFIED], "{ids:?}");
}

/// A domain has no family of its own, so the residual is the only thing that says one arrived.
#[test]
fn a_domain_added_with_a_declaration_still_reports_unclassified_beside_the_declaration() {
    let system = replaced(SYSTEM, "  - gap.desk\n", "  - gap.desk\n  - gap.notes\n");
    let notes = "
domain: gap.notes

types:
  - name: gap.notes.NoteId
    kind: newtype
    of: Uuid
";
    let base = ir(SYSTEM, DOMAIN);
    let grown = ir_of(&[
        ("system.yaml", &system),
        ("domains/desk.yaml", DOMAIN),
        ("domains/notes.yaml", notes),
    ]);
    let ids = sorted(ids(&base, &grown));
    assert_eq!(
        ids,
        [UNCLASSIFIED, "type/gap.notes.NoteId/added"],
        "{ids:?}"
    );
}

// ---- the revision pair and the IR's own shape ----------------------------------------------------

/// `residual_differs` equals the old `residual(before) != residual(after)` whenever no family has a
/// key on one side only, so the committed pair's delta is byte-identical across the change exactly
/// when this holds.
#[test]
fn the_revision_pair_has_no_one_sided_declaration_so_its_delta_cannot_move() {
    let before = support::compiled("examples/revision-pair/before");
    let after = support::compiled("examples/revision-pair/after");
    let was = serde_json::to_value(&before).unwrap();
    let is = serde_json::to_value(&after).unwrap();
    for family in [
        "types",
        "entities",
        "commands",
        "events",
        "errors",
        "views",
        "actors",
        "bindings",
        "components",
    ] {
        let keys = |value: &serde_json::Value| -> BTreeSet<String> {
            value[family]
                .as_object()
                .map(|map| map.keys().cloned().collect())
                .unwrap_or_default()
        };
        assert_eq!(keys(&was), keys(&is), "{family}");
    }
}

/// Every top-level key the IR serializes, so a new declaration family cannot arrive without
/// somebody deciding whether the one-sided filter covers it. Unlisted, a new family is compared
/// whole, which over-reports and never under-reports.
#[test]
fn the_ir_top_level_keys_are_the_ones_the_residual_was_written_for() {
    let value = serde_json::to_value(support::compiled("examples/billing")).unwrap();
    let keys: BTreeSet<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let known: BTreeSet<&str> = [
        "system",
        "version",
        "naming",
        "summary",
        "domains",
        "types",
        "conversions",
        "entities",
        "commands",
        "events",
        "errors",
        "views",
        "actors",
        "bindings",
        "components",
        "workloads",
        "preconditions",
    ]
    .into_iter()
    .collect();
    let unknown: Vec<_> = keys.difference(&known).collect();
    assert!(unknown.is_empty(), "{unknown:?}");
}
