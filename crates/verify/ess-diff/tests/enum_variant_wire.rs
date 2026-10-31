//! A variant renamed on the wire is a reported change, and it versions its own delta vocabulary.
//!
//! `story:an-enum-variant-carries-its-own-wire-spelling`. The variant's own name does not move
//! when its wire spelling does, so the variant set and the variant order both say nothing — and
//! before these cases existed the comparison reported an empty delta for a change every deployed
//! consumer breaks on.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{
    change::{SemanticChange, TypeChange},
    diff, EssDelta, RawEssDelta,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(variants: &str) -> EssIr {
    let text = format!(
        "format: ess/5\nsystem: recordings\nversion: v1\ndomains: [recordings.backend]\ndomain: \
         recordings.backend\ntypes:\n  - name: recordings.backend.RecordingAction\n    kind: \
         enum\n    variants:\n{variants}"
    );
    let spec = Specification::assemble([(
        Source::new("recordings.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("recordings.yaml", text);
    compile(&spec, &sources).unwrap()
}

const BARE: &str = "      - Stop\n      - Flag\n";
const NAMED: &str = "      - name: Stop\n        wire: \"\"\n      - Flag\n";

#[test]
fn a_variant_renamed_on_the_wire_is_reported_and_needs_the_new_delta_vocabulary() {
    for (before, after, expected) in [
        (model(BARE), model(NAMED), ("Stop", "")),
        (model(NAMED), model(BARE), ("", "Stop")),
    ] {
        let delta = diff(&before, &after).unwrap();

        assert_eq!(
            delta.changes().len(),
            1,
            "the variant set and its order did not move: {:?}",
            delta.changes()
        );
        assert!(
            matches!(
                &delta.changes()[0],
                SemanticChange::Type {
                    changed: TypeChange::VariantWireNameChanged { variant, before, after },
                    ..
                } if variant == "Stop" && before == expected.0 && after == expected.1
            ),
            "{:?}",
            delta.changes()[0]
        );
        assert_eq!(delta.format.major(), 5);

        let text = delta.to_canonical_json();
        let raw: RawEssDelta = serde_json::from_str(&text).unwrap();
        EssDelta::try_from(raw).unwrap();
        for old in 1..=4 {
            let raw: RawEssDelta =
                serde_json::from_str(&text.replace("ess-diff/5", &format!("ess-diff/{old}")))
                    .unwrap();
            assert!(
                EssDelta::try_from(raw).is_err(),
                "ess-diff/{old} cannot represent a variant wire name"
            );
        }
    }
}

#[test]
fn presentation_moves_are_reported_apart_from_the_wire_move() {
    let delta = diff(
        &model(BARE),
        &model(
            "      - name: Stop\n        display: Stop recording\n        summary: Ends the \
             recording\n      - Flag\n",
        ),
    )
    .unwrap();

    let kinds: Vec<&str> = delta
        .changes()
        .iter()
        .map(|change| match change {
            SemanticChange::Type { changed, .. } => changed.kind(),
            other => panic!("a type change was expected, got {other:?}"),
        })
        .collect();

    assert_eq!(
        kinds,
        vec!["variant-display-name-changed", "variant-summary-changed"],
        "the wire spelling did not move, so nothing claims it did"
    );
}

#[test]
fn a_bare_variant_list_that_did_not_move_reports_nothing() {
    assert_eq!(diff(&model(BARE), &model(BARE)).unwrap().changes().len(), 0);
}

/// A lifecycle state's summary (`ess/33`, `story:a-transition-and-a-state-may-carry-a-summary`) is
/// the summary of its variant of `<Entity>.State`, and a moved one is that variant's
/// `variant-summary-changed`, reported once.
#[test]
fn a_moved_state_summary_is_the_state_variants_summary_change() {
    let fixture = include_str!("../../../generate/ess-gen/tests/fixtures/state-wire.yaml");
    let spelled = "states: [{name: Indexing, wire: indexing}, Ready]";
    assert!(fixture.contains(spelled) && fixture.contains("format: ess/32\n"));
    let compiled = |states: &str| {
        let text = fixture
            .replacen("format: ess/32\n", "format: ess/33\n", 1)
            .replacen(spelled, states, 1);
        let spec = Specification::assemble([(
            Source::new("jobs.yaml"),
            RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}")),
        )])
        .unwrap_or_else(|errors| panic!("{errors}"));
        let mut sources = SourceMap::new();
        sources.insert("jobs.yaml", text.clone());
        compile(&spec, &sources).unwrap_or_else(|errors| panic!("{errors}"))
    };
    for (before, after) in [
        (
            spelled,
            "states: [{name: Indexing, wire: indexing}, {name: Ready, summary: Searchable}]",
        ),
        (
            "states: [{name: Indexing, wire: indexing, summary: Being indexed}, Ready]",
            "states: [{name: Indexing, wire: indexing, summary: Being written}, Ready]",
        ),
    ] {
        let delta = diff(&compiled(before), &compiled(after)).unwrap();
        assert_eq!(delta.changes().len(), 1, "{:?}", delta.changes());
        assert!(
            matches!(
                &delta.changes()[0],
                SemanticChange::Type {
                    subject,
                    changed: TypeChange::VariantSummaryChanged { .. },
                } if subject.name().to_string() == "jobs.queue.Job.State"
            ),
            "{:?}",
            delta.changes()[0]
        );
    }
}

/// A lifecycle state's wire name (`ess/32`, beyond10x/ess#639) changes the wire spelling of
/// the variant of `<Entity>.State`, and is reported as that variant's change.
#[test]
fn a_changed_state_wire_name_is_reported() {
    let spelled = include_str!("../../../generate/ess-gen/tests/fixtures/state-wire.yaml");
    let bare = spelled.replace(
        "states: [{name: Indexing, wire: indexing}, Ready]",
        "states: [Indexing, Ready]",
    );
    assert_ne!(bare, spelled);
    let compiled = |text: &str| {
        let spec = Specification::assemble([(
            Source::new("jobs.yaml"),
            RawSpecFile::parse(text).unwrap(),
        )])
        .unwrap_or_else(|errors| panic!("{errors}"));
        let mut sources = SourceMap::new();
        sources.insert("jobs.yaml", text.to_owned());
        compile(&spec, &sources).unwrap_or_else(|errors| panic!("{errors}"))
    };
    for (before, after, expected) in [
        (bare.as_str(), spelled, ("Indexing", "indexing")),
        (spelled, bare.as_str(), ("indexing", "Indexing")),
    ] {
        let delta = diff(&compiled(before), &compiled(after)).unwrap();
        assert_eq!(delta.changes().len(), 1, "{:?}", delta.changes());
        assert!(
            matches!(
                &delta.changes()[0],
                SemanticChange::Type {
                    subject,
                    changed: TypeChange::VariantWireNameChanged { variant, before, after },
                } if subject.name().to_string() == "jobs.queue.Job.State"
                    && variant == "Indexing"
                    && before == expected.0
                    && after == expected.1
            ),
            "{:?}",
            delta.changes()[0]
        );
    }
}
