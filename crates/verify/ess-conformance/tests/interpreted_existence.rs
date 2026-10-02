//! Existence branches execute real supplied-identity effects in the interpreter.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

fn model(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("existence.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn all_pass(source: &str, minimum: usize) {
    let ir = model(source);
    let synthesis = ess_conformance::synthesize(&ir);
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    assert!(synthesis.suite.scenarios.len() >= minimum);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let failures: Vec<_> = report
        .scenarios
        .iter()
        .filter(|case| case.status != Status::Passed)
        .collect();
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn source_built_upsert_and_duplicate_refusal_scenarios_execute() {
    all_pass(MODEL, 4);
}

#[test]
fn a_deleted_supplied_identity_can_be_created_again() {
    let source = MODEL
        .replace("views:\n", "  - name: demo.items.UnbookSlot\n    input: [{name: slot_id, type: demo.items.ItemId}]\n    outcomes:\n      - {name: missing, unknown_instance: true, error: demo.items.SlotTaken}\n      - name: unbooked\n        deletes: demo.items.Slot\n        instance: slot_id\nviews:\n")
        .replace("demo.items.PutItem, demo.items.BookSlot]", "demo.items.PutItem, demo.items.BookSlot, demo.items.UnbookSlot]");
    all_pass(&source, 6);
}

#[test]
fn optional_creation_identity_is_supplied_or_generated_as_declared() {
    let source = MODEL
        .replace(
            "    input:\n      - {name: slot_id, type: demo.items.ItemId}",
            "    input:\n      - {name: slot_id, type: 'Optional<demo.items.ItemId>'}",
        )
        .replace(
            "{slot_id: input.slot_id, label: input.label}",
            "{slot_id: {input: slot_id, else: {generated: true}}, label: input.label}",
        );
    all_pass(&source, 4);
}

#[test]
fn singleton_supplied_identity_is_created_once() {
    all_pass(
        &MODEL.replace(
            "{name: demo.items.ItemId, kind: newtype, of: String}",
            "{name: demo.items.ItemId, kind: enum, variants: [Only]}",
        ),
        4,
    );
}

#[test]
fn each_guarded_creation_branch_keeps_its_supplied_identity() {
    let source = MODEL.replace("      - name: booked\n", "      - name: booked-vip\n        when: label == \"vip\"\n        creates: demo.items.Slot\n        instance: slot_id\n        emits: [demo.items.SlotBooked]\n        payload: {demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}}\n        sets: {label: input.label}\n      - name: booked\n");
    all_pass(&source, 5);
}

fn run(ir: &EssIr, store: &Store, command: &str, input: &[(&str, &str)]) -> Step {
    let values = input
        .iter()
        .map(|(key, value)| (key.to_string(), Node::Text(value.to_string())))
        .collect::<BTreeMap<_, _>>();
    let mut steps = execute(
        ir,
        store,
        &format!("demo.items.{command}").parse().unwrap(),
        &values,
        &Externals::Withheld,
    )
    .unwrap();
    assert_eq!(steps.len(), 1);
    steps.remove(0)
}

#[test]
fn supplied_identity_is_the_exact_stored_and_emitted_identity() {
    let ir = model(MODEL);
    let first = run(
        &ir,
        &Store::default(),
        "PutItem",
        &[("item_id", "caller-chosen"), ("label", "first")],
    );
    assert_eq!(
        first.outcome.unwrap().to_string(),
        "demo.items.PutItem/created"
    );
    assert_eq!(
        first.events[0].payload["item_id"],
        Node::Text("caller-chosen".into())
    );
    let updated = run(
        &ir,
        &first.next,
        "PutItem",
        &[("item_id", "caller-chosen"), ("label", "second")],
    );
    assert_eq!(
        updated.outcome.unwrap().to_string(),
        "demo.items.PutItem/updated"
    );
    assert_eq!(updated.next.instances().count(), 1);
    assert_eq!(
        updated
            .next
            .instance(&"demo.items.Item".parse().unwrap(), "caller-chosen")
            .unwrap()
            .fields["label"],
        Node::Text("second".into())
    );
}

#[test]
fn input_refusal_precedes_duplicate_lookup_without_related_guards() {
    let source = MODEL.replace("      - name: booked\n", "      - {name: blank, when: 'label == \"\"', error: demo.items.SlotTaken}\n      - name: booked\n");
    let ir = model(&source);
    let first = run(
        &ir,
        &Store::default(),
        "BookSlot",
        &[("slot_id", "held"), ("label", "original")],
    );
    for id in ["held", "fresh"] {
        let refused = run(
            &ir,
            &first.next,
            "BookSlot",
            &[("slot_id", id), ("label", "")],
        );
        assert_eq!(
            refused.outcome.unwrap().to_string(),
            "demo.items.BookSlot/blank"
        );
        assert!(refused.events.is_empty());
        assert_eq!(refused.next, first.next);
    }
    let refused = run(
        &ir,
        &first.next,
        "BookSlot",
        &[("slot_id", "held"), ("label", "replacement")],
    );
    assert_eq!(
        refused.outcome.unwrap().to_string(),
        "demo.items.BookSlot/already-booked"
    );
    assert!(refused.events.is_empty());
    assert_eq!(refused.next, first.next);
}

#[test]
fn duplicate_lookup_precedes_missing_related_row_and_input_refusal() {
    let source = MODEL.replace("format: ess/16", "format: ess/18")
        .replace("  - {name: demo.items.Label", "  - {name: demo.items.SlotId, kind: newtype, of: String}\n  - {name: demo.items.Label")
        .replace("{name: slot_id, type: demo.items.ItemId}", "{name: slot_id, type: demo.items.SlotId}")
        .replace("    input:\n      - {name: slot_id, type: demo.items.SlotId}", "    input:\n      - {name: slot_id, type: demo.items.SlotId}\n      - {name: item_id, type: demo.items.ItemId}")
        .replace("      - name: booked\n", "      - {name: missing-item, when_related: {via: input.item_id, exists: false}, error: demo.items.SlotTaken}\n      - {name: blank, when: 'label == \"\"', error: demo.items.SlotTaken}\n      - name: booked\n");
    let ir = model(&source);
    let item = run(
        &ir,
        &Store::default(),
        "PutItem",
        &[("item_id", "item"), ("label", "item")],
    );
    let booked = run(
        &ir,
        &item.next,
        "BookSlot",
        &[
            ("slot_id", "held"),
            ("item_id", "item"),
            ("label", "original"),
        ],
    );
    let refused = run(
        &ir,
        &booked.next,
        "BookSlot",
        &[("slot_id", "held"), ("item_id", "missing"), ("label", "")],
    );
    assert_eq!(
        refused.outcome.unwrap().to_string(),
        "demo.items.BookSlot/already-booked"
    );
    assert!(refused.events.is_empty());
    assert_eq!(refused.next, booked.next);
}

#[test]
fn a_collision_in_an_unselected_creation_entity_does_not_refuse_the_selected_one() {
    let source = MODEL.replace(
        "  - name: demo.items.BookSlot\n    input:",
        "  - name: demo.items.BookSlot\n    input:\n      - {name: as_item, type: Boolean}",
    ).replace("      - name: booked\n", "      - name: booked-item\n        when: as_item == true\n        creates: demo.items.Item\n        instance: item_id\n        emits: [demo.items.ItemStored]\n        payload: {demo.items.ItemStored: {item_id: input.slot_id, label: input.label}}\n        sets: {label: input.label}\n      - name: booked\n");
    let ir = model(&source);
    for order in [[true, false], [false, true]] {
        let mut store = Store::default();
        for as_item in order {
            let input = BTreeMap::from([
                ("slot_id".into(), Node::Text("shared".into())),
                ("label".into(), Node::Text("original".into())),
                ("as_item".into(), Node::Bool(as_item)),
            ]);
            let steps = execute(
                &ir,
                &store,
                &"demo.items.BookSlot".parse().unwrap(),
                &input,
                &Externals::Withheld,
            )
            .unwrap();
            assert_eq!(steps.len(), 1);
            assert_eq!(
                steps[0].outcome.as_ref().unwrap().to_string(),
                if as_item {
                    "demo.items.BookSlot/booked-item"
                } else {
                    "demo.items.BookSlot/booked"
                }
            );
            store = steps[0].next.clone();
            let duplicate = execute(
                &ir,
                &store,
                &"demo.items.BookSlot".parse().unwrap(),
                &input,
                &Externals::Withheld,
            )
            .unwrap();
            assert_eq!(
                duplicate[0].outcome.as_ref().unwrap().to_string(),
                "demo.items.BookSlot/already-booked"
            );
            assert_eq!(duplicate[0].next, store);
            assert!(duplicate[0].events.is_empty());
        }
        assert_eq!(store.instances().count(), 2);
    }
}
