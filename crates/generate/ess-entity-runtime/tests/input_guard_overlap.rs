//! An input-guarded refusal is taken before any accepting branch whose guard it overlaps
//! (beyond10x/ess#178, `docs/design/input-guard-overlap-precedence.md`).
//!
//! Entity Runtime selects the first branch whose guard holds, so the lowering orders every
//! input-guarded refusal ahead of the accepting guarded branches — whatever order the source wrote
//! them in.
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use entity_core::{identity, FieldKind, PreloadDecision, Registry, Runtime};
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::json;

/// The billing example with `too-large: amount.amount > 1000` declared after
/// `settled: amount.amount > 0`, which it overlaps above 1000.
fn billing_with_a_late_refusal() -> ess_compiler::ir::EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing");
    let mut paths = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut parsed = Vec::new();
    let mut labels = Vec::new();
    let mut sources = SourceMap::new();
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        if label == "domains/invoice.yaml" {
            let before = "      - name: rejected\n        error: billing.invoice.InvalidAmount\n        summary: The payment was not positive";
            assert!(text.contains(before), "the mutation site is present");
            text = text.replacen(
                before,
                &format!(
                    "      - name: too-large\n        when: amount.amount > 1000\n        error: billing.invoice.InvalidAmount\n        summary: The payment is above the limit.\n{before}"
                ),
                1,
            );
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).expect("fixture validates");
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

#[test]
fn an_input_guarded_refusal_is_selected_before_the_accepting_branch_it_overlaps() {
    let ir = billing_with_a_late_refusal();
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("invoice-service").unwrap())
        .expect("service extracts");
    let options = LoweringOptions {
        definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
            .iter()
            .map(|entry| (QualifiedName::new(entry).unwrap(), NonZeroU32::MIN))
            .collect(),
        scales: BTreeMap::new(),
    };
    let lowered = lower(&service, &options).expect("billing lowers");
    let mut registry = Registry::new();
    for definition in lowered.definitions().values() {
        registry
            .register(definition.as_definition().clone())
            .expect("definition registers");
    }
    registry.validate_all().expect("closure validates");
    let runtime = Runtime::new(&registry);
    let logical_id = json!("b404a1e8-9360-4af5-a0ac-8a483adfa225");
    let storage_id = identity::address(FieldKind::String, &logical_id).unwrap();
    let decide = |amount: i64| {
        runtime
            .decide_before_load(
                "billing.invoice.Invoice",
                1,
                storage_id.clone(),
                "billing.invoice.PayInvoice",
                json!({"input": {"invoice_id": logical_id, "amount": {"amount": amount, "currency": "EUR"}}, "bound": {}}),
            )
            .expect("input-only selection succeeds")
    };
    match decide(2000) {
        PreloadDecision::Refused(refusal) => assert_eq!(refusal.outcome, "too-large"),
        PreloadDecision::Load(prepared) => {
            panic!(
                "the overlap point reached `settled`: {:?}",
                prepared.subject()
            )
        }
    }
    assert!(matches!(decide(25), PreloadDecision::Load(_)));
    match decide(0) {
        PreloadDecision::Refused(refusal) => assert_eq!(refusal.outcome, "rejected"),
        PreloadDecision::Load(_) => panic!("a non-positive payment is refused"),
    }
}
