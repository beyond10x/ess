//! A replay derives one retained identity rather than inventing caller authority.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/retained-replay.yaml");

fn admits(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|e| e.to_string())?;
    Specification::assemble([(Source::new("replay.yaml"), raw)]).map_err(|e| e.to_string())
}

#[test]
fn command_local_replay_retains_the_original_create_identity_without_input_id() {
    let spec = admits(MODEL).expect("a document-only create has an observed original identity");
    let command = spec.commands().values().next().unwrap();
    let replay = command
        .outcomes
        .iter()
        .find(|o| o.name.as_str() == "replayed")
        .unwrap();
    assert_eq!(replay.test_strategy().as_str(), "replay_result");
    let encoded = serde_yaml::to_string(replay).unwrap();
    assert!(encoded.contains("replays: seeded"));
    assert!(!encoded.contains("instance:"));
}

#[test]
fn replay_requires_new_source_and_one_successful_command_local_origin() {
    for text in (1..=6).map(|v| MODEL.replace("ess/7", &format!("ess/{v}"))).chain([
        MODEL.replace("replays: seeded", "replays: missing"),
        MODEL.replace("replays: seeded", "replays: replayed"),
        MODEL.replace("replays: seeded", "replays: retained.core.Seed.seeded"),
        MODEL.replace("replays: seeded", "replays: seeded\n        emits: [retained.core.Seeded]"),
        MODEL.replace("replays: seeded", "replays: seeded\n        preserves: retained.core.Record\n        instance: document"),
        MODEL.replace("replays: seeded", "replays: seeded\n        sets: {value: input.document}"),
    ]) {
        assert!(admits(&text).is_err(), "must refuse:\n{text}");
    }
}

#[test]
fn finite_state_default_is_observation_only_and_old_formats_still_refuse_it() {
    let text = include_str!("../../../verify/ess-conformance/tests/fixtures/retained-commit.yaml");
    let spec = admits(text).unwrap();
    let command = spec
        .commands()
        .get(&"retained.core.Commit".parse().unwrap())
        .unwrap();
    assert!(command.has_state_refusal());
    assert!(command
        .outcomes
        .iter()
        .find(|o| o.name.as_str() == "refused")
        .unwrap()
        .subject
        .is_none());
    for version in 1..7 {
        assert!(admits(&text.replace("ess/7", &format!("ess/{version}"))).is_err());
    }
    for invalid in [
        text.replace("replays: committed", "replays: committed\n        error: retained.core.TransactionStateConflict"),
        text.replace("replays: committed", "replays: committed\n      - name: another-replay\n        external: additional\n        replays: replayed"),
        text.replace("      - name: refused\n        error:", "      - name: refused\n        when: transaction_id != none\n        error:"),
        text.replace("when_subject_state: Validated", "when_subject_state: Missing"),
    ] { assert!(admits(&invalid).is_err(), "must refuse:\n{invalid}"); }
}
