//! Adversary pass against story `a-union-variant-scenario-reads-each-row-under-its-own-identity`.
//!
//! Its acceptance: each variant's view expectation names the identity of the event that created
//! that row, the reference target passes, and an observed-identity reference ambiguous between two
//! events never silently binds to the first. The unit's own tests check one model, a single union
//! of two unit variants on a creating branch with nothing beside it. These drive the same claim
//! over neighbouring shapes: three variants, two unions, a payload variant beside the unit ones, an
//! updating branch whose arrangement creates the row, and a creation guarded by a related row.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn ir_of(model: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(model).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// Every scenario the Rust reference does not pass, with its diagnostics.
fn not_passed(model: &str, suite: &ConformanceSuite) -> Vec<(String, Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir_of(model)))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect::<Vec<_>>()
                .join("\n");
            (result.scenario.to_string(), result.status, diagnostics)
        })
        .collect()
}

fn observed_events(node: &serde_json::Value, found: &mut Vec<String>) {
    match node {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(serde_json::Value::as_str) == Some("observed") {
                if let Some(event) = map.get("event").and_then(serde_json::Value::as_str) {
                    found.push(event.to_owned());
                }
            }
            for value in map.values() {
                observed_events(value, found);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                observed_events(item, found);
            }
        }
        _ => {}
    }
}

/// Every observed reference to an event written after that event was published twice in the same
/// scenario: the runners resolve it to the first occurrence, so it is ambiguous (the story's second
/// acceptance bullet, on the synthesis side the unit claims).
fn ambiguous_observed(ir: &EssIr, suite: &ConformanceSuite) -> Vec<(String, usize, String)> {
    let mut emits: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, command) in ir.commands() {
        for outcome in &command.outcomes {
            emits.insert(
                format!("{name}/{}", outcome.name),
                outcome
                    .emits
                    .iter()
                    .map(|event| event.name().to_string())
                    .collect(),
            );
        }
    }
    let mut found = Vec::new();
    let mut most = 0;
    for (id, scenario) in &suite.scenarios {
        let mut published: BTreeMap<String, usize> = BTreeMap::new();
        for (at, step) in scenario.steps.iter().enumerate() {
            let mut read = Vec::new();
            observed_events(&serde_json::to_value(step).unwrap(), &mut read);
            for event in read {
                if published.get(&event).copied().unwrap_or(0) >= 2 {
                    found.push((id.to_string(), at, event));
                }
            }
            if let ScenarioStep::ExpectOutcome { outcome } = step {
                for event in emits.get(&outcome.to_string()).into_iter().flatten() {
                    let count = published.entry(event.clone()).or_default();
                    *count += 1;
                    most = most.max(*count);
                }
            }
        }
    }
    if most < 2 {
        // Say what the suite holds, so a vacuous probe is diagnosed in the same run.
        let shapes: Vec<String> = suite
            .scenarios
            .iter()
            .map(|(id, scenario)| {
                let tags: Vec<String> = scenario
                    .steps
                    .iter()
                    .map(|step| match step {
                        ScenarioStep::ExpectOutcome { outcome } => {
                            format!("expect_outcome {outcome}")
                        }
                        other => serde_json::to_value(other).unwrap()["step"]
                            .as_str()
                            .unwrap_or("?")
                            .to_owned(),
                    })
                    .collect();
                format!("{id}: {tags:?}")
            })
            .collect();
        panic!(
            "no scenario published an event twice: the probe would be vacuous\nemits: {emits:#?}\n{}",
            shapes.join("\n")
        );
    }
    found
}

fn check(model: &str) {
    let ir = ir_of(model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let suite = synthesis.suite;
    let failing = not_passed(model, &suite);
    let ambiguous = ambiguous_observed(&ir, &suite);
    assert!(
        failing.is_empty() && ambiguous.is_empty(),
        "reference not passed: {failing:#?}\nobserved after a second publication: {ambiguous:#?}\nrefusals: {:#?}",
        synthesis.refusals
    );
}

const HEAD: &str = "format: ess/22
system: store
version: v1
domain: store.box
";

const THREE: &str = "types:
  - {name: store.box.BoxId, kind: newtype, of: Uuid}
  - name: store.box.Kind
    kind: union
    tag: kind
    variants:
      Plain:
      Fancy:
      Huge:
entities:
  - name: store.box.Box
    identity: {name: box_id, type: store.box.BoxId}
    fields:
      - {name: kind, type: store.box.Kind}
    lifecycle: {initial: Made, states: [Made], terminal: [Made]}
events:
  - name: store.box.BoxMade
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: kind, type: store.box.Kind}
commands:
  - name: store.box.MakeBox
    input:
      - {name: kind, type: store.box.Kind}
    outcomes:
      - name: made
        creates: store.box.Box
        instance: box_id
        emits: [store.box.BoxMade]
        payload:
          store.box.BoxMade:
            box_id: {generated: true}
            kind: input.kind
        sets: {kind: input.kind}
views:
  - name: store.box.Boxes
    source: store.box.Box
    consistency: read_your_writes
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: kind, type: store.box.Kind}
";

/// Three unit variants: two further runs, each on a row of its own.
#[test]
fn adv_u6_three_variants_each_row_under_its_own_identity() {
    check(&format!("{HEAD}{THREE}"));
}

/// A payload variant beside the unit ones.
#[test]
fn adv_u6_a_payload_variant_beside_unit_variants() {
    check(&format!(
        "{HEAD}{}",
        THREE.replace("      Huge:\n", "      Custom: String\n")
    ));
}

/// Two unions, of two and three variants (least common multiple six).
#[test]
fn adv_u6_two_unions_of_different_sizes() {
    let model = THREE
        .replace(
            "      Huge:\n",
            "  - name: store.box.Size\n    kind: union\n    tag: size\n    variants:\n      Small:\n      Medium:\n      Large:\n",
        )
        .replace(
            "      - {name: kind, type: store.box.Kind}\n    lifecycle",
            "      - {name: kind, type: store.box.Kind}\n      - {name: size, type: store.box.Size}\n    lifecycle",
        )
        .replace(
            "      - {name: kind, type: store.box.Kind}\ncommands",
            "      - {name: kind, type: store.box.Kind}\n      - {name: size, type: store.box.Size}\ncommands",
        )
        .replace(
            "      - {name: kind, type: store.box.Kind}\n    outcomes",
            "      - {name: kind, type: store.box.Kind}\n      - {name: size, type: store.box.Size}\n    outcomes",
        )
        .replace(
            "            kind: input.kind\n        sets: {kind: input.kind}\n",
            "            kind: input.kind\n            size: input.size\n        sets: {kind: input.kind, size: input.size}\n",
        );
    let model = format!("{model}      - {{name: size, type: store.box.Size}}\n");
    check(&format!("{HEAD}{model}"));
}

/// An updating branch with a union input: each further run's arrangement creates a row through
/// `MakeBox`, which publishes `BoxMade` again.
#[test]
fn adv_u6_an_updating_branch_on_rows_its_arrangement_creates() {
    let model = "types:
  - {name: store.box.BoxId, kind: newtype, of: Uuid}
  - name: store.box.Kind
    kind: union
    tag: kind
    variants:
      Plain:
      Fancy:
entities:
  - name: store.box.Box
    identity: {name: box_id, type: store.box.BoxId}
    fields:
      - {name: label, type: String}
      - {name: kind, type: Optional<store.box.Kind>}
    lifecycle: {initial: Made, states: [Made], terminal: [Made]}
errors:
  - {name: store.box.NoSuchBox, fields: []}
events:
  - name: store.box.BoxMade
    fields:
      - {name: box_id, type: store.box.BoxId}
  - name: store.box.KindSet
    fields:
      - {name: box_id, type: store.box.BoxId}
commands:
  - name: store.box.MakeBox
    input:
      - {name: label, type: String}
    outcomes:
      - name: made
        creates: store.box.Box
        instance: box_id
        emits: [store.box.BoxMade]
        payload:
          store.box.BoxMade: {box_id: {generated: true}}
        sets: {label: input.label}
  - name: store.box.SetKind
    input:
      - {name: box_id, type: store.box.BoxId}
      - {name: kind, type: store.box.Kind}
    outcomes:
      - name: set
        updates: store.box.Box
        instance: box_id
        emits: [store.box.KindSet]
        payload:
          store.box.KindSet: {box_id: input.box_id}
        sets: {kind: input.kind}
      - {name: no-such-box, unknown_instance: true, error: store.box.NoSuchBox}
views:
  - name: store.box.Boxes
    source: store.box.Box
    consistency: read_your_writes
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: label, type: String}
      - {name: kind, type: Optional<store.box.Kind>}
";
    check(&format!("{HEAD}{model}"));
}

/// A creation guarded by a related row: each further run arranges a shelf of its own, which
/// publishes `ShelfMade` again, and the box names it.
#[test]
fn adv_u6_a_creation_guarded_by_a_related_row() {
    let model = "types:
  - {name: store.box.BoxId, kind: newtype, of: Uuid}
  - {name: store.box.ShelfId, kind: newtype, of: Uuid}
  - name: store.box.Kind
    kind: union
    tag: kind
    variants:
      Plain:
      Fancy:
entities:
  - name: store.box.Shelf
    identity: {name: shelf_id, type: store.box.ShelfId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Up, states: [Up], terminal: [Up]}
  - name: store.box.Box
    identity: {name: box_id, type: store.box.BoxId}
    fields:
      - {name: shelf_id, type: store.box.ShelfId}
      - {name: kind, type: store.box.Kind}
    lifecycle: {initial: Made, states: [Made], terminal: [Made]}
errors:
  - {name: store.box.NoSuchShelf, fields: []}
events:
  - name: store.box.ShelfMade
    fields:
      - {name: shelf_id, type: store.box.ShelfId}
  - name: store.box.BoxMade
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: shelf_id, type: store.box.ShelfId}
commands:
  - name: store.box.MakeShelf
    input:
      - {name: label, type: String}
    outcomes:
      - name: made
        creates: store.box.Shelf
        instance: shelf_id
        emits: [store.box.ShelfMade]
        payload:
          store.box.ShelfMade: {shelf_id: {generated: true}}
        sets: {label: input.label}
  - name: store.box.MakeBox
    input:
      - {name: shelf_id, type: store.box.ShelfId}
      - {name: kind, type: store.box.Kind}
    outcomes:
      - name: no-shelf
        when_related: {via: input.shelf_id, exists: false}
        error: store.box.NoSuchShelf
      - name: made
        creates: store.box.Box
        instance: box_id
        emits: [store.box.BoxMade]
        payload:
          store.box.BoxMade: {box_id: {generated: true}, shelf_id: input.shelf_id}
        sets: {shelf_id: input.shelf_id, kind: input.kind}
views:
  - name: store.box.Boxes
    source: store.box.Box
    consistency: read_your_writes
    fields:
      - {name: box_id, type: store.box.BoxId}
      - {name: shelf_id, type: store.box.ShelfId}
      - {name: kind, type: store.box.Kind}
";
    check(&format!("{HEAD}{model}"));
}
