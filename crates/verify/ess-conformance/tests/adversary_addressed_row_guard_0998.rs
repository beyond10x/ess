//! Adversary pass on 0.99.8: variants of a `when_related:` guard that reads the row the command
//! addresses, beside an `unknown_instance:` (or another answer to an unknown identity). Every
//! scenario synthesized for each variant must pass on the interpreter, the reference target.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run_on(suite: &ConformanceSuite, target: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(target))
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

/// Synthesizes `text`, runs the suite on the interpreter, and fails naming every scenario that did
/// not pass, with its steps, and the refusals.
fn every_scenario_passes(text: &str, expected: &[&str]) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    let suite = synthesis.suite;
    let statuses = run_on(&suite, model);
    eprintln!("statuses: {statuses:#?}");
    eprintln!("refusals: {:#?}", synthesis.refusals);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    let steps: Vec<_> = suite
        .scenarios
        .iter()
        .filter(|(id, _)| failing.iter().any(|(name, _)| **name == id.to_string()))
        .map(|(id, scenario)| format!("{id}: {:#?}", scenario.steps))
        .collect();
    assert!(
        failing.is_empty(),
        "failing: {failing:#?}\n{}",
        steps.join("\n")
    );
    for id in expected {
        assert!(
            statuses.contains_key(*id),
            "{id} synthesized: {statuses:#?}"
        );
    }
}

const HEAD: &str = "format: ess/37
system: demo
version: v1
domain: demo.items
types:
  - {name: demo.items.ItemId, kind: newtype, of: Uuid}
  - {name: demo.items.OwnerId, kind: newtype, of: Uuid}
errors:
  - {name: demo.items.ItemNotFound}
  - {name: demo.items.OwnerMismatch}
  - {name: demo.items.LabelMismatch}
  - {name: demo.items.OwnerInactive}
  - {name: demo.items.ItemStateConflict}
events:
  - name: demo.items.ItemCreated
    fields:
      - {name: item_id, type: demo.items.ItemId}
  - name: demo.items.ItemClaimed
    fields:
      - {name: item_id, type: demo.items.ItemId}
views:
  - name: demo.items.Items
    source: demo.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.items.ItemId}
      - {name: owner_id, type: demo.items.OwnerId}
      - {name: label, type: String}
";

const ITEM: &str = "entities:
  - name: demo.items.Item
    identity: {name: item_id, type: demo.items.ItemId}
    fields:
      - {name: owner_id, type: demo.items.OwnerId}
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Claimed]
      terminal: [Claimed]
      transitions:
        - {name: claim, from: [Open], to: Claimed}
";

const CREATE: &str = "commands:
  - name: demo.items.Create
    input:
      - {name: owner_id, type: demo.items.OwnerId}
      - {name: label, type: String}
    outcomes:
      - name: created
        creates: demo.items.Item
        instance: item_id
        sets:
          owner_id: input.owner_id
          label: input.label
        emits: [demo.items.ItemCreated]
        payload:
          demo.items.ItemCreated:
            item_id: {generated: true}
";

const CLAIM_INPUT: &str = "  - name: demo.items.Claim
    input:
      - {name: item_id, type: demo.items.ItemId}
      - {name: owner_id, type: demo.items.OwnerId}
      - {name: label, type: String}
    outcomes:
";

const UNKNOWN: &str = "      - name: no-such-item
        unknown_instance: true
        error: demo.items.ItemNotFound
";
const ROW_MISSING: &str = "      - name: no-such-item-row
        when_related: {via: input.item_id, exists: false}
        error: demo.items.ItemNotFound
";
const OWNER: &str = "      - name: owner-mismatch
        when_related: {via: input.item_id, predicate: owner_id != input.owner_id}
        error: demo.items.OwnerMismatch
";
const LABEL: &str = "      - name: label-mismatch
        when_related: {via: input.item_id, predicate: label != input.label}
        error: demo.items.LabelMismatch
";
const CLAIMED: &str = "      - name: claimed
        moves: demo.items.Item.claim
        instance: item_id
        emits: [demo.items.ItemClaimed]
        payload:
          demo.items.ItemClaimed:
            item_id: input.item_id
";
const WRONG: &str = "      - name: wrong-state
        wrong_state: true
        error: demo.items.ItemStateConflict
";

fn model(entities: &str, outcomes: &[&str]) -> String {
    format!("{HEAD}{entities}{CREATE}{CLAIM_INPUT}{}", outcomes.concat())
}

const NO_SUCH_ITEM: &str = "demo.items.Claim/outcome/no-such-item";

/// The 0.99.8 fixture's order, with the label input added: the baseline the variants move from.
#[test]
fn baseline_order() {
    every_scenario_passes(
        &model(ITEM, &[UNKNOWN, ROW_MISSING, OWNER, CLAIMED, WRONG]),
        &[NO_SUCH_ITEM],
    );
}

/// The refusals on the addressed row declared above `unknown_instance:`, the accepting branch first.
#[test]
fn refusals_declared_above_the_unknown_instance() {
    every_scenario_passes(
        &model(ITEM, &[CLAIMED, OWNER, ROW_MISSING, WRONG, UNKNOWN]),
        &[NO_SUCH_ITEM],
    );
}

/// Two `when_related:` predicate siblings over the addressed row.
#[test]
fn two_predicate_siblings_on_the_addressed_row() {
    every_scenario_passes(
        &model(ITEM, &[UNKNOWN, ROW_MISSING, OWNER, LABEL, CLAIMED, WRONG]),
        &[NO_SUCH_ITEM],
    );
}

/// Only `exists: false` over the addressed row, no predicate.
#[test]
fn only_exists_false_on_the_addressed_row() {
    every_scenario_passes(
        &model(ITEM, &[UNKNOWN, ROW_MISSING, CLAIMED, WRONG]),
        &[NO_SUCH_ITEM],
    );
}

/// No `unknown_instance:` at all: the unknown identity is answered by whatever the rule names.
#[test]
fn no_unknown_instance_beside_a_guard_on_the_addressed_row() {
    every_scenario_passes(&model(ITEM, &[ROW_MISSING, OWNER, CLAIMED, WRONG]), &[]);
}

/// An external not-found refusal naming the item, in place of `unknown_instance:`.
#[test]
fn an_external_not_found_beside_a_guard_on_the_addressed_row() {
    let text = model(
        ITEM,
        &[
            ROW_MISSING,
            OWNER,
            "      - name: not-found
        external: no item carries the identity
        error: demo.items.ItemMissing
",
            CLAIMED,
            WRONG,
        ],
    )
    .replace(
        "errors:\n",
        "errors:\n  - name: demo.items.ItemMissing\n    fields: [{name: item_id, type: demo.items.ItemId}]\n",
    );
    // Synthesis refuses every Claim branch here (ArrangeRelatedRow): a gap, not a wrong send.
    every_scenario_passes(&text, &[]);
}

/// A lifecycle whose created row is not yet in the state the accepting branch moves from.
#[test]
fn a_created_row_must_be_moved_before_the_guard_branches() {
    let entity = ITEM
        .replace("initial: Open", "initial: Draft")
        .replace("states: [Open, Claimed]", "states: [Draft, Open, Claimed]")
        .replace(
            "        - {name: claim, from: [Open], to: Claimed}\n",
            "        - {name: claim, from: [Open], to: Claimed}\n        - {name: publish, from: [Draft], to: Open}\n",
        );
    let publish = "  - name: demo.items.Publish
    input:
      - {name: item_id, type: demo.items.ItemId}
    outcomes:
      - name: published
        moves: demo.items.Item.publish
        instance: item_id
        emits: [demo.items.ItemPublished]
        payload:
          demo.items.ItemPublished:
            item_id: input.item_id
";
    let text = model(&entity, &[UNKNOWN, ROW_MISSING, OWNER, CLAIMED, WRONG])
        .replace(
            "events:\n",
            "events:\n  - name: demo.items.ItemPublished\n    fields:\n      - {name: item_id, type: demo.items.ItemId}\n",
        )
        .replace(CLAIM_INPUT, &format!("{publish}{CLAIM_INPUT}"));
    every_scenario_passes(&text, &[NO_SUCH_ITEM]);
}

/// A create-or-update: `unknown_instance:` creates the addressed row, and a guard reads it.
#[test]
fn a_creating_unknown_instance_beside_a_guard_on_the_addressed_row() {
    let upsert = "      - name: made
        unknown_instance: true
        creates: demo.items.Item
        instance: item_id
        sets:
          owner_id: input.owner_id
          label: input.label
        emits: [demo.items.ItemCreated]
        payload:
          demo.items.ItemCreated:
            item_id: input.item_id
";
    every_scenario_passes(
        &model(ITEM, &[upsert, ROW_MISSING, OWNER, CLAIMED, WRONG]),
        &["demo.items.Claim/outcome/made"],
    );
}

/// A guard on the addressed row beside a guard on a row of another entity the input names.
#[test]
fn a_guard_on_the_addressed_row_beside_a_row_of_another_entity() {
    let entities = format!(
        "{ITEM}  - name: demo.items.Owner
    identity: {{name: owner_id, type: demo.items.OwnerId}}
    fields:
      - {{name: active, type: Boolean}}
    lifecycle: {{initial: Live, states: [Live], terminal: [Live]}}
"
    );
    let owner_missing = "      - name: no-such-owner
        when_related: {via: input.owner_id, exists: false}
        error: demo.items.ItemNotFound
";
    let owner_inactive = "      - name: owner-inactive
        when_related: {via: input.owner_id, predicate: active == false}
        error: demo.items.OwnerInactive
";
    let add_owner = "  - name: demo.items.AddOwner
    input:
      - {name: owner_id, type: demo.items.OwnerId}
      - {name: active, type: Boolean}
    outcomes:
      - name: added
        creates: demo.items.Owner
        instance: owner_id
        sets: {active: input.active}
        emits: [demo.items.OwnerAdded]
        payload:
          demo.items.OwnerAdded:
            owner_id: input.owner_id
";
    let text = model(
        &entities,
        &[
            UNKNOWN,
            ROW_MISSING,
            owner_missing,
            OWNER,
            owner_inactive,
            CLAIMED,
            WRONG,
        ],
    )
    .replace(
        "events:\n",
        "events:\n  - name: demo.items.OwnerAdded\n    fields:\n      - {name: owner_id, type: demo.items.OwnerId}\n",
    )
    .replace(CLAIM_INPUT, &format!("{add_owner}{CLAIM_INPUT}"));
    // `no-such-item` is refused here (two rows read): only what is synthesized is asserted.
    every_scenario_passes(&text, &[]);
}

/// A guard on a row of the same entity named by another input, beside the addressed row: not the
/// addressed row, so a present row is arranged beside the unknown identity.
#[test]
fn a_guard_on_another_row_of_the_addressed_entity() {
    let text = model(
        ITEM,
        &[
            UNKNOWN,
            "      - name: no-such-parent
        when_related: {via: input.parent_id, exists: false}
        error: demo.items.ItemNotFound
      - name: parent-owner-mismatch
        when_related: {via: input.parent_id, predicate: owner_id != input.owner_id}
        error: demo.items.OwnerMismatch
",
            CLAIMED,
            WRONG,
        ],
    )
    .replace(
        "      - {name: owner_id, type: demo.items.OwnerId}\n      - {name: label, type: String}\n    outcomes:\n      - name: no-such-item",
        "      - {name: owner_id, type: demo.items.OwnerId}\n      - {name: label, type: String}\n      - {name: parent_id, type: demo.items.ItemId}\n    outcomes:\n      - name: no-such-item",
    );
    every_scenario_passes(&text, &[NO_SUCH_ITEM]);
}

/// The guard on the addressed row beside a guard on another row of the same entity (two rows).
#[test]
fn a_guard_on_the_addressed_row_beside_another_row_of_its_entity() {
    let text = model(
        ITEM,
        &[
            UNKNOWN,
            ROW_MISSING,
            "      - name: no-such-parent
        when_related: {via: input.parent_id, exists: false}
        error: demo.items.ItemNotFound
",
            OWNER,
            "      - name: parent-owner-mismatch
        when_related: {via: input.parent_id, predicate: owner_id != input.owner_id}
        error: demo.items.OwnerMismatch
",
            CLAIMED,
            WRONG,
        ],
    )
    .replace(
        "      - {name: owner_id, type: demo.items.OwnerId}\n      - {name: label, type: String}\n    outcomes:\n      - name: no-such-item",
        "      - {name: owner_id, type: demo.items.OwnerId}\n      - {name: label, type: String}\n      - {name: parent_id, type: demo.items.ItemId}\n    outcomes:\n      - name: no-such-item",
    );
    // `no-such-item`, `owner-mismatch` and `claimed` are refused here: a gap, not a wrong send.
    every_scenario_passes(&text, &[]);
}

/// The 0.99.8 shape under every older format that admits it: the fix is not gated on a format, so
/// each admitted format must pass on the interpreter as `ess/37` does.
#[test]
fn every_format_that_admits_the_shape_passes() {
    let base = model(ITEM, &[UNKNOWN, ROW_MISSING, OWNER, CLAIMED, WRONG]);
    let mut admitted = Vec::new();
    let mut failing = Vec::new();
    for version in 1..=37 {
        let text = base.replace("format: ess/37", &format!("format: ess/{version}"));
        let Ok(raw) = RawSpecFile::parse(&text) else {
            continue;
        };
        let Ok(spec) = Specification::assemble([(Source::new("model.yaml"), raw)]) else {
            continue;
        };
        let Ok(model) = compile(&spec, &SourceMap::new()) else {
            continue;
        };
        admitted.push(version);
        let suite = synthesize(&model).suite;
        let statuses = run_on(&suite, model);
        for (id, status) in statuses {
            if status != Status::Passed {
                failing.push(format!("ess/{version} {id} {status:?}"));
            }
        }
    }
    eprintln!("admitted: {admitted:?}");
    assert!(admitted.contains(&37), "{admitted:?}");
    assert!(
        failing.is_empty(),
        "admitted {admitted:?}; failing: {failing:#?}"
    );
}
