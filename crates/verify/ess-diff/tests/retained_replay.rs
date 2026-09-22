//! Retained-result authority is a semantic delta and an origin dependency.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, EssDelta, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("replay.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
#[test]
fn changing_retained_origin_requires_diff6_and_keeps_its_dependency() {
    let before = include_str!("../../ess-conformance/tests/fixtures/retained-replay.yaml").replace("      - name: replayed", "      - name: alternate\n        external: alternative original success\n        creates: retained.core.Record\n        instance: record_id\n        emits: [retained.core.Seeded]\n        payload:\n          retained.core.Seeded:\n            record_id: {generated: true}\n      - name: replayed");
    let after = before.replace("replays: seeded", "replays: alternate");
    let old = ir(&before);
    let new = ir(&after);
    let delta = diff(&old, &new).unwrap();
    assert_eq!(delta.format.to_string(), "ess-diff/6");
    let json = delta.to_canonical_json();
    assert!(json.contains("outcome-replay-changed"));
    assert!(
        !json.contains("unclassified-changed"),
        "derived retained metadata must not create a second delta"
    );
    for major in 1..6 {
        let raw: RawEssDelta =
            serde_json::from_str(&json.replace("ess-diff/6", &format!("ess-diff/{major}")))
                .unwrap();
        assert!(EssDelta::try_from(raw).is_err());
    }
    for (model, origin) in [(&old, "seeded"), (&new, "alternate")] {
        let graph = ess_compiler::graph::SemanticDependencyGraph::of(model);
        let relation = graph
            .edges()
            .find(|e| e.relation == ess_compiler::graph::DependencyRelation::Replays)
            .unwrap();
        assert!(serde_json::to_string(relation).unwrap().contains(origin));
        let command = model.commands().values().next().unwrap();
        let replay = command
            .outcomes
            .iter()
            .find_map(|o| o.replays.as_ref())
            .unwrap();
        assert_eq!(command.replay_origin(replay).name.as_str(), origin);
    }
}

#[test]
fn source7_complete_refusal_metadata_is_a_typed_bidirectional_observation_delta() {
    let model = include_str!("../../ess-conformance/tests/fixtures/retained-commit.yaml");
    let start = model.find("  - name: retained.core.Commit\n").unwrap();
    let end = model.find("views:\n").unwrap();
    let model = format!("{}{}", &model[..start], &model[end..])
        .replace(", Committed", "")
        .replace(
            "        - {name: commit, from: [Validated], to: Committed}\n",
            "",
        )
        .replace(
            "input: [{name: note, type: String}]",
            "input: [{name: note, type: String}, {name: transaction_id, type: Uuid}]",
        )
        .replace(
            "transaction_id: {generated: true}",
            "transaction_id: input.transaction_id",
        );
    let old = ir(&model.replace("ess/7", "ess/6"));
    let new = ir(&model);
    let old_bytes = old.to_canonical_json();
    assert!(!old_bytes.contains("complete_refusal"));
    for version in 1..6 {
        assert_eq!(
            ir(&model.replace("ess/7", &format!("ess/{version}"))).to_canonical_json(),
            old_bytes
        );
    }
    for (before, after, expected) in [(&old, &new, true), (&new, &old, false)] {
        let delta = diff(before, after).unwrap();
        assert_eq!(delta.format.to_string(), "ess-diff/6");
        let text = delta.to_canonical_json();
        assert!(!text.contains("unclassified-changed"), "{text}");
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        let changes = json["changes"].as_array().unwrap();
        assert_eq!(changes.len(), 3);
        for change in changes {
            assert_eq!(
                change["change"]["changed"]["kind"], "outcome-observation-changed",
                "{text}"
            );
            assert_eq!(change["change"]["changed"]["after"], expected);
        }
        for version in 1..6 {
            let raw: RawEssDelta =
                serde_json::from_str(&text.replace("ess-diff/6", &format!("ess-diff/{version}")))
                    .unwrap();
            assert!(EssDelta::try_from(raw).is_err());
        }
    }
}
