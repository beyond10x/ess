//! Clock contracts survive compilation and produce executable native observation helpers.
use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

fn fixture() -> EssIr {
    fixture_text(include_str!("fixtures/clock-reading.yaml"))
}
fn fixture_text(text: &str) -> EssIr {
    let specification = Specification::assemble([(
        Source::new("reading.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("reading.yaml", text);
    compile_locating(&specification, &sources, &["reading.yaml"]).unwrap()
}
#[test]
fn native_readings_retain_contract_and_emit_shared_runtime_helpers() {
    let ir = fixture();
    assert_eq!(
        ir.types()
            .values()
            .filter(|declared| declared.reading.is_some())
            .count(),
        3
    );
    for target in [Target::Rust, Target::Go] {
        let result = synthesize_for(&ir, target).unwrap();
        let code = result
            .artifacts
            .values()
            .map(|a| a.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(code.contains("resolve_reading") || code.contains("ResolveReading"));
        assert!(code.contains("MismatchedOccurrence"));
        if let Some(directory) = std::env::var_os("ESS_CLOCK_NATIVE_OUT") {
            let base = std::path::PathBuf::from(directory).join(target.name());
            for (relative, artifact) in result.artifacts {
                let path = base.join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                let mut contents = artifact.contents;
                if target == Target::Rust && contents.contains("pub struct OffsetText(") {
                    contents.push_str(include_str!("fixtures/clock-native.rs"));
                }
                if target == Target::Go && contents.contains("type OffsetText struct") {
                    let package = contents
                        .lines()
                        .find(|line| line.starts_with("package "))
                        .unwrap();
                    std::fs::write(
                        path.with_file_name("clock_reading_test.go"),
                        format!("{package}\n{}", include_str!("fixtures/clock-native.go")),
                    )
                    .unwrap();
                }
                std::fs::write(path, contents).unwrap();
            }
        }
    }
}
#[test]
fn changing_clock_encoding_changes_contract_digest_without_erasing_legacy_primitives() {
    let ir = fixture();
    let before = ess_gen::Provenance::of(&ir);
    let serialized = serde_json::to_string(&ir).unwrap();
    assert!(serialized.contains("\"reading\":{\"encoding\""));
    assert!(serialized.contains("requires_observation"));
    let changed = fixture_text(
        &include_str!("fixtures/clock-reading.yaml")
            .replace("producer_process", "consumer_process")
            .replace(
                ", {role: consumer_process, offset: requires_observation}",
                "",
            ),
    );
    assert_ne!(
        before.contract_digest,
        ess_gen::Provenance::of(&changed).contract_digest
    );
    let schema = ess_gen::artifact::run(&ess_gen::schema::JsonSchema, &ir).unwrap();
    let schema = schema
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(schema.contains("x-ess-reading"));
    assert!(schema.contains("local_date_time_millis_literal_z"));
}

#[test]
fn existing_target_refusals_on_reading_models_use_failure3() {
    let source = include_str!("fixtures/clock-reading.yaml").replace(
        "      - {name: offset",
        "      - {name: unsupported, type: Binary64}\n      - {name: offset",
    );
    for target in [Target::Rust, Target::Go] {
        let failure = synthesize_for(&fixture_text(&source), target)
            .err()
            .unwrap();
        let json = serde_json::to_value(&failure).unwrap();
        assert_eq!(json["format"], "ess-target-failure/3");
        assert_eq!(json["causes"][0]["code"], "missing-representation");
    }
}
