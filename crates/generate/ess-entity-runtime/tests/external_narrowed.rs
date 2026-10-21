//! An `external:` branch narrowed to the rows a subject guard selects (`ess/29`,
//! `story:external-branch-narrowed-by-subject-guard`) at Entity Runtime lowering: the branch lowers
//! to its stored-field predicate over the loaded row and the host's external evidence, both
//! required, at its declared place among the held-state branches, and the evidence is one supplied
//! slot the binding plan names, as for an unnarrowed external branch.
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, BindingRequirement, BoundTarget, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::json;

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/external-narrowed-by-subject.yaml"
);

const COMPONENT: &str = "components:
  - component: vault-service
    owns: {domains: [demo.vault]}
    accepts: {commands: [demo.vault.Issue, demo.vault.Redeem]}
    publishes: {events: [demo.vault.Issued, demo.vault.Redeemed]}
    reached_by: network
";

fn ir_of(text: &str) -> EssIr {
    let text = format!("{text}{COMPONENT}");
    let spec = Specification::assemble([(
        Source::new("vault.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn a_narrowed_external_branch_lowers_to_its_row_guard_and_the_hosts_evidence() {
    let ir = ir_of(MODEL);
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("vault-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("demo.vault.Grant").unwrap(),
            NonZeroU32::new(1).unwrap(),
        )]
        .into_iter()
        .collect(),
        scales: BTreeMap::new(),
    };
    let lowered = lower(&service, &options).unwrap_or_else(|diagnostics| {
        panic!("the narrowed branch lowers: {:#?}", diagnostics.into_vec())
    });
    let grant = &lowered.definitions()[&QualifiedName::new("demo.vault.Grant").unwrap()];
    let redeem = &grant.operations["demo.vault.Redeem"];
    let names: Vec<&str> = redeem
        .outcomes
        .iter()
        .map(|outcome| outcome.name.as_str())
        .collect();
    assert_eq!(names[0], "hashed-mismatch", "{names:?}");
    assert_eq!(names[1], "redirect-mismatch", "{names:?}");
    let binding = &lowered.bindings().commands()[&QualifiedName::new("demo.vault.Redeem").unwrap()];
    let evidence = binding
        .slots
        .iter()
        .find(|(_, value)| {
            matches!(&value.target, BoundTarget::ExternalEvidence { outcome } if outcome.as_str() == "hashed-mismatch")
        })
        .map(|(slot, _)| *slot)
        .expect("the external verdict is one supplied slot");
    assert!(lowered
        .bindings()
        .requirements()
        .iter()
        .any(|requirement| matches!(
            requirement,
            BindingRequirement::ExternalEvidenceSupplied { outcome, .. }
                if outcome.as_str() == "hashed-mismatch"
        )));
    let when = serde_json::to_value(&redeem.outcomes[0].when).expect("condition serializes");
    let all = when["all"]
        .as_array()
        .expect("an `all` of the guard and the evidence");
    assert_eq!(all.len(), 2, "{when}");
    assert!(
        all[0].to_string().contains("$fields.method"),
        "the row guard reads the stored method: {when}"
    );
    assert_eq!(
        all[1],
        json!({"truthy": format!("$args.bound.b{:08}", evidence.index())})
    );
}
