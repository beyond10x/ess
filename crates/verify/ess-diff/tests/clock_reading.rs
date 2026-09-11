//! Reading contracts have typed semantic deltas even when their scalar representation is unchanged.
use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_diff::{
    change::{SemanticChange, TypeChange},
    diff, EssDelta, RawEssDelta,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
const SOURCE: &str = include_str!("../../../generate/ess-synth/tests/fixtures/clock-reading.yaml");
fn compile(text: &str) -> EssIr {
    let specification = Specification::assemble([(
        Source::new("reading.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("reading.yaml", text);
    compile_locating(&specification, &sources, &["reading.yaml"]).unwrap()
}
fn changed() -> EssDelta {
    let after = SOURCE.replacen(
        "origins: [{role: producer_process, offset: encoded_offset}]",
        "origins: [{role: consumer_process, offset: encoded_offset}]",
        1,
    );
    diff(&compile(SOURCE), &compile(&after)).unwrap()
}
#[test]
fn origin_only_changes_are_typed_without_claiming_scalar_representation_changed() {
    let delta = changed();
    assert!(delta.changes().iter().any(|change| matches!(
        change,
        SemanticChange::Type {
            changed: TypeChange::ReadingContractChanged {
                before: Some(_),
                after: Some(_)
            },
            ..
        }
    )));
    assert!(!delta.changes().iter().any(|change| matches!(
        change,
        SemanticChange::Type {
            changed: TypeChange::RepresentationChanged { .. },
            ..
        }
    )));
}
#[test]
fn attachment_addition_and_removal_are_visible() {
    let plain=SOURCE.replacen("    reading:\n      encoding: offset_date_time_text\n      origins: [{role: producer_process, offset: encoded_offset}]\n","",1);
    for (before, after, adding) in [
        (plain.as_str(), SOURCE, true),
        (SOURCE, plain.as_str(), false),
    ] {
        let delta = diff(&compile(before), &compile(after)).unwrap();
        assert!(delta.changes().iter().any(|change| match change {
            SemanticChange::Type {
                changed: TypeChange::ReadingContractChanged { before, after },
                ..
            } => before.is_none() == adding && after.is_some() == adding,
            _ => false,
        }));
    }
}
#[test]
fn reading_diff_roundtrips_only_under_the_coordinated_new_vocabulary() {
    let delta = changed();
    assert_eq!(delta.format.major(), 3);
    let text = delta.to_canonical_json();
    let raw: RawEssDelta = serde_json::from_str(&text).unwrap();
    EssDelta::try_from(raw).unwrap();
    for old in 1..=2 {
        let raw: RawEssDelta =
            serde_json::from_str(&text.replace("ess-diff/3", &format!("ess-diff/{old}"))).unwrap();
        assert!(EssDelta::try_from(raw).is_err());
        assert!(delta
            .to_canonical_json_for(format!("ess-diff/{old}").parse().unwrap())
            .is_err());
    }
    if let Some(path) = std::env::var_os("ESS_CLOCK_DIFF_OUT") {
        std::fs::write(path, text).unwrap();
    }
}
