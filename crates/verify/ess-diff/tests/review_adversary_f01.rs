//! Independent F01 boundary probes through compiled source and public impact APIs.

mod support;

use ess_domain::refs::ExternalRef;
use ess_gen::provenance::ProvenanceMint;

#[test]
fn complete_generated_and_authored_suite_four_bytes_remain_frozen() {
    // Captured by the coordinator from clean pre-F01 09474bdf84e29e5facb1e7b72628685dad9d11d1.
    // SHA256 508b6a3d75d6dabd6fa686b67dcb6c7c881374375aaed9dd8921445a5edc894e, 154846 bytes.
    let frozen = include_str!("fixtures/review-billing-suite-v4.json");
    assert_eq!(frozen.len(), 154_846);
    let ir = support::compiled("examples/billing");
    let mut synthesis = ess_conformance::synthesize(&ir);
    let authoring = ess_conformance::authored::compile(&ir, &[
        ess_conformance::authored::Source::new(
            "examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml",
            include_str!("../../../../examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml"),
        ),
    ]);
    assert!(authoring.is_complete());
    assert_eq!(authoring.scenarios.len(), 1);
    for (id, scenario) in authoring.scenarios {
        synthesis.suite.insert(id, scenario).unwrap();
    }
    assert!(synthesis.refusals.is_empty());
    let candidate = synthesis.suite.to_canonical_json();
    let parsed: serde_json::Value = serde_json::from_str(&candidate).unwrap();
    assert_eq!(parsed["provenance"]["suite_version"], "ess-conformance/4");
    assert_eq!(parsed["scenarios"].as_object().unwrap().len(), 30);
    assert_eq!(candidate, frozen);
    assert_eq!(
        ess_conformance::scenario::ConformanceSuite::from_json(frozen)
            .unwrap()
            .to_canonical_json(),
        frozen
    );
}

#[test]
fn relation_delta_versions_refuse_relabeling_and_public_serialize_bypasses() {
    let before = support::compiled("examples/billing");
    let after = support::compiled_with("examples/billing", |files| {
        files
            .iter_mut()
            .flat_map(|(_, file)| &mut file.entities)
            .find(|entity| !entity.relations.is_empty())
            .unwrap()
            .relations[0]
            .name = "renamed".into();
    });
    let report = ess_diff::impact(&before, &after, None, None).unwrap();
    assert_eq!(
        serde_json::to_value(&report).unwrap()["format"],
        "ess-impact/3"
    );
    let delta = report.delta;
    assert!(delta
        .changes()
        .iter()
        .any(|change| change.kind() == "relations-changed"));
    assert!(delta
        .to_canonical_json_for(ess_diff::DeltaFormat::LEGACY)
        .is_err());
    let mut raw: serde_json::Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    raw["format"] = serde_json::json!("ess-diff/1");
    assert!(ess_diff::EssDelta::try_from(
        serde_json::from_value::<ess_diff::RawEssDelta>(raw).unwrap()
    )
    .is_err());
    let mut relabeled = delta;
    for format in ["ess-diff/1", "ess-diff/99"] {
        relabeled.format = ess_diff::DeltaFormat::parse(format).unwrap();
        assert!(serde_json::to_string(&relabeled).is_err());
    }
}

fn billing_with_outcome_ref(index: usize, classified: bool) -> ess_compiler::EssIr {
    support::compiled_with("examples/billing", |files| {
        let command = files
            .iter_mut()
            .flat_map(|(_, file)| &mut file.commands)
            .find(|command| command.outcomes.len() > 1)
            .unwrap();
        for outcome in &mut command.outcomes {
            outcome.refs.clear();
        }
        command.outcomes[index].refs = vec![ExternalRef::parse("jira:DEV-630").unwrap()];
        if classified {
            command.naming.summary = Some("A classified change beside the moved reference.".into());
        }
    })
}

fn assert_moved_reference_accounted(classified: bool) {
    let before = billing_with_outcome_ref(0, false);
    let after = billing_with_outcome_ref(1, classified);
    assert_ne!(before.source_digest(), after.source_digest());
    let suite = ess_conformance::synthesize(&before).suite;
    let report = ess_diff::impact(&before, &after, Some(&suite), None).unwrap();
    println!("delta: {}", report.delta.to_canonical_json());
    println!("invalidation: {:?}", report.invalidation);
    assert!(
        report.delta.changes().iter().any(|change| change.kind() == "unclassified-changed"),
        "moving an existing external reference to another outcome must retain its owner in residual comparison"
    );
    assert!(matches!(
        report.invalidation,
        Some(ess_diff::Invalidation::Whole { .. })
    ));
}

#[test]
fn moved_outcome_reference_retains_its_owner() {
    assert_moved_reference_accounted(false);
}

#[test]
fn moved_outcome_reference_is_independent_of_a_classified_edit() {
    assert_moved_reference_accounted(true);
}

#[test]
fn explicit_domain_naming_defaults_remain_semantically_equivalent() {
    let before = support::compiled("examples/billing");
    let after = support::compiled_with("examples/billing", |files| {
        let domain = files
            .iter_mut()
            .find(|(_, file)| {
                file.domain
                    .as_ref()
                    .is_some_and(|name| name.to_string() == "billing.email")
            })
            .unwrap();
        assert_eq!(domain.1.naming.wire, None);
        assert_eq!(domain.1.naming.display, None);
        domain.1.naming.wire = Some("email".into());
        domain.1.naming.display = Some("email".into());
    });
    assert_ne!(before.source_digest(), after.source_digest());
    let delta = ess_diff::diff(&before, &after).unwrap();
    assert!(
        delta.is_empty(),
        "effective domain naming defaults changed no contract: {}",
        delta.to_canonical_json()
    );
}

#[test]
fn incomplete_schema_stamp_is_owed_by_the_real_impact_reader() {
    let ir = support::compiled("examples/billing");
    let artifacts = ess_gen::generate_all(&ir).unwrap();
    let mut tree = ess_diff::impact::GeneratedTree {
        files: artifacts
            .into_iter()
            .map(|(path, artifact)| (path, artifact.contents))
            .collect(),
    };
    let path = "schema/types/billing.invoice.InvoiceId.schema.json";
    let id = ess_diff::impact::ArtifactId::Projection {
        path: path.to_owned(),
    };
    let control = ess_diff::impact(&ir, &ir, None, Some(&tree)).unwrap();
    assert!(!control.artifacts.owed().unwrap().contains_key(&id));
    let mut schema: serde_json::Value = serde_json::from_str(&tree.files[path]).unwrap();
    schema["x-ess-provenance"]
        .as_object_mut()
        .unwrap()
        .remove("system");
    tree.files.insert(
        path.to_owned(),
        serde_json::to_string_pretty(&schema).unwrap(),
    );
    let report = ess_diff::impact(&ir, &ir, None, Some(&tree)).unwrap();
    let obligation = report.artifacts.owed().unwrap().get(&id);
    println!("incomplete {path} obligation: {obligation:?}");
    assert!(
        matches!(
            obligation,
            Some(ess_diff::impact::ArtifactObligation::ProvenanceUnreadable)
        ),
        "incomplete provenance must not retain a still-current artifact claim"
    );
}

#[test]
fn ownership_cardinality_invalidates_both_emitted_schema_ends() {
    let before = support::compiled("examples/billing");
    let after = support::compiled_with("examples/billing", |files| {
        let owner = files
            .iter_mut()
            .flat_map(|(_, file)| &mut file.entities)
            .find(|entity| !entity.relations.is_empty())
            .unwrap();
        owner.relations[0].cardinality = ess_domain::entity::Cardinality::One;
    });
    let old = ess_gen::generate_all(&before).unwrap();
    let new = ess_gen::generate_all(&after).unwrap();
    let tree = ess_diff::impact::GeneratedTree {
        files: old
            .iter()
            .map(|(path, artifact)| (path.clone(), artifact.contents.clone()))
            .collect(),
    };
    let report = ess_diff::impact(&before, &after, None, Some(&tree)).unwrap();
    let owed = report
        .artifacts
        .owed()
        .expect("typed relation changes narrow");
    for path in [
        "schema/entities/billing.invoice.Account.schema.json",
        "schema/entities/billing.invoice.Invoice.schema.json",
    ] {
        assert_ne!(
            old[path].contents, new[path].contents,
            "both schemas carry relation semantics: {path}"
        );
        assert_ne!(
            ProvenanceMint::new(&before).digest_of(&old[path].slice),
            ProvenanceMint::new(&after).digest_of(&new[path].slice)
        );
        assert!(
            owed.contains_key(&ess_diff::impact::ArtifactId::Projection {
                path: path.to_owned()
            }),
            "both ownership endpoints are owed: {path}"
        );
    }
}
