//! Independent second-pass semantic and dependency coverage probes.

use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::Provenance;

fn fixture(row_invariant: &str, view_summary: &str) -> EssIr {
    fixture_with(row_invariant, view_summary, |_| {})
}

fn fixture_with(
    row_invariant: &str,
    view_summary: &str,
    transform: impl FnOnce(&mut RawSpecFile),
) -> EssIr {
    let yaml = format!(
        r"
format: ess/1
system: probe
version: v1
domain: probe.core
types:
  - name: probe.core.Row
    kind: struct
    fields:
      - name: item_id
        type: Uuid
      - name: amount
        type: Integer
    invariants: [{row_invariant}]
entities:
  - name: probe.core.Item
    identity:
      name: item_id
      type: Uuid
    fields:
      - name: amount
        type: Integer
    lifecycle:
      initial: Ready
      states: [Ready]
      terminal: [Ready]
views:
  - name: probe.core.Items
    source: probe.core.Item
    shape: probe.core.Row
    consistency: read_your_writes
    naming:
      summary: {view_summary}
components:
  - component: read-service
    owns:
      domains: [probe.core]
    reached_by: network
"
    );
    let mut raw = RawSpecFile::parse(&yaml).unwrap();
    transform(&mut raw);
    let spec = Specification::assemble(vec![(Source::new("probe.yaml"), raw)]).unwrap();
    ess_compiler::compile(&spec, &SourceMap::new()).unwrap()
}

fn assert_served_contract_invalidates(before: &EssIr, after: &EssIr) {
    let old = ess_gen::generate_all(before).unwrap();
    let new = ess_gen::generate_all(after).unwrap();
    let path = "openapi/read-service.yaml";
    let old_stamp = Provenance::read_digests(&old[path].contents).unwrap();
    let new_stamp = Provenance::read_digests(&new[path].contents).unwrap();
    assert_ne!(before.source_digest(), after.source_digest());
    let component = before.components().values().next().unwrap();
    let old_json = ess_gen::openapi::json(before, component);
    let component = after.components().values().next().unwrap();
    let new_json = ess_gen::openapi::json(after, component);
    let mut old_body: serde_json::Value = serde_json::from_str(&old_json).unwrap();
    let mut new_body: serde_json::Value = serde_json::from_str(&new_json).unwrap();
    old_body["info"]
        .as_object_mut()
        .unwrap()
        .remove("x-ess-provenance");
    new_body["info"]
        .as_object_mut()
        .unwrap()
        .remove("x-ess-provenance");
    assert_ne!(
        old_body, new_body,
        "the emitted contract changes beyond its provenance"
    );
    let tree = ess_diff::impact::GeneratedTree {
        files: old
            .into_iter()
            .map(|(path, artifact)| (path, artifact.contents))
            .collect(),
    };
    let report = ess_diff::impact(before, after, None, Some(&tree)).unwrap();
    let id = ess_diff::impact::ArtifactId::Projection { path: path.into() };
    let owed = report
        .artifacts
        .owed()
        .is_none_or(|owed| owed.contains_key(&id));
    let digest_moved = old_stamp.contract_digest != new_stamp.contract_digest;
    println!("delta: {}", report.delta.to_canonical_json());
    println!(
        "OpenAPI contract digest moved: {digest_moved}; OpenAPI owed: {owed}; old={old_stamp:?}; new={new_stamp:?}"
    );
    assert_eq!(
        [digest_moved, owed],
        [true, true],
        "an observably changed served contract must not retain a current slice claim"
    );
}

#[test]
fn served_view_change_reaches_its_openapi_artifact() {
    assert_served_contract_invalidates(
        &fixture("amount >= 0", "Original rows."),
        &fixture("amount >= 0", "Revised rows."),
    );
}

#[test]
fn reusable_row_invariant_change_reaches_its_openapi_artifact() {
    assert_served_contract_invalidates(
        &fixture("amount >= 0", "Original rows."),
        &fixture("amount > 0", "Original rows."),
    );
}

#[test]
fn reusable_row_type_belongs_to_the_view_slice_it_supplies() {
    let before = fixture("amount >= 0", "Original rows.");
    let after = fixture("amount > 0", "Original rows.");
    let view: ess_compiler::refs::EssSemanticRef = ess_compiler::refs::ViewRef::new(
        ess_domain::name::QualifiedName::new("probe.core.Items").unwrap(),
    )
    .into();
    let old = ess_gen::provenance::ProvenanceMint::new(&before).of_seeds([view.clone()]);
    let new = ess_gen::provenance::ProvenanceMint::new(&after).of_seeds([view]);
    let row_name = ess_domain::name::QualifiedName::new("probe.core.Row").unwrap();
    assert_ne!(
        before.types().get(&row_name).unwrap(),
        after.types().get(&row_name).unwrap()
    );
    println!(
        "view old digest={}, new digest={}",
        old.provenance.contract_digest, new.provenance.contract_digest
    );
    assert_ne!(
        old.provenance.contract_digest, new.provenance.contract_digest,
        "a reusable row type's changed invariant belongs to the view's supplied schema contract"
    );
}

#[test]
fn switching_equal_row_shapes_retains_independent_residual_coverage() {
    let with_shape = |switched: bool, classified: bool| {
        fixture_with("amount >= 0", "Original rows.", |raw| {
            let mut twin = raw.types[0].clone();
            twin.name = ess_domain::name::QualifiedName::new("probe.core.TwinRow").unwrap();
            raw.types.push(twin);
            if switched {
                raw.views[0].shape =
                    Some(ess_domain::name::QualifiedName::new("probe.core.TwinRow").unwrap());
            }
            if classified {
                raw.views[0].naming.summary = Some("Revised rows.".into());
            }
        })
    };
    let before = with_shape(false, false);
    for classified in [false, true] {
        let after = with_shape(true, classified);
        assert_eq!(
            before.views().values().next().unwrap().fields,
            after.views().values().next().unwrap().fields
        );
        let report = ess_diff::impact(&before, &after, None, None).unwrap();
        assert!(report
            .delta
            .changes()
            .iter()
            .any(|change| change.kind() == "unclassified-changed"));
        assert!(matches!(
            report.artifacts,
            ess_diff::impact::ArtifactAnswer::Whole { .. }
        ));
    }
}

#[test]
fn ranking_precedence_survives_the_checked_delta_roundtrip() {
    let ranked = |reverse: bool| {
        fixture_with("amount >= 0", "Original rows.", |raw| {
            raw.views[0].order_by =
                serde_json::from_value(serde_json::json!(["amount asc", "item_id desc"])).unwrap();
            if reverse {
                raw.views[0].order_by.reverse();
            }
        })
    };
    let delta = ess_diff::diff(&ranked(false), &ranked(true)).unwrap();
    let decoded = ess_diff::EssDelta::try_from(
        serde_json::from_str::<ess_diff::RawEssDelta>(&delta.to_canonical_json()).unwrap(),
    )
    .unwrap();
    assert_eq!(delta, decoded);
    let change = delta
        .changes()
        .iter()
        .find(|change| change.kind() == "ranking-changed")
        .unwrap();
    let serialized = serde_json::to_value(change).unwrap();
    assert_eq!(serialized["changed"]["before"][0]["field"], "amount");
    assert_eq!(serialized["changed"]["after"][0]["field"], "item_id");
    assert!(delta
        .to_canonical_json_for(ess_diff::DeltaFormat::LEGACY)
        .is_err());
}
