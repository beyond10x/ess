//! Typed accessor generation. The optional evidence directory retains executable native witnesses.
use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

fn fixture() -> EssIr {
    fixture_text(include_str!("fixtures/bounded-accessor.yaml"))
}
fn fixture_text(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("fixture syntax");
    let specification =
        Specification::assemble([(Source::new("accessor.yaml"), raw)]).expect("fixture admission");
    let mut sources = SourceMap::new();
    sources.insert("accessor.yaml", text);
    compile_locating(&specification, &sources, &["accessor.yaml"]).expect("fixture compilation")
}

#[test]
fn both_native_targets_preserve_shared_typed_accessor_operations() {
    for target in [Target::Rust, Target::Go] {
        let output = synthesize_for(&fixture(), target).expect("native accessor support");
        let code = output
            .artifacts
            .values()
            .map(|a| a.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(code.contains("total typed accessor"));
        assert!(code.contains("project_0") || code.contains("project0"));
        if let Some(directory) = std::env::var_os("ESS_ACCESSOR_NATIVE_OUT") {
            let base = std::path::PathBuf::from(directory).join(if target == Target::Rust {
                "rust"
            } else {
                "go"
            });
            for (relative, artifact) in output.artifacts {
                let path = base.join(relative);
                std::fs::create_dir_all(path.parent().expect("generated parent"))
                    .expect("evidence directory");
                std::fs::write(path, artifact.contents).expect("native witness");
            }
        }
    }
}

#[test]
fn mechanical_accessor_conversion_is_generated() {
    let text = include_str!("fixtures/bounded-accessor.yaml").replace("String", "projection.core.SourceTag")
        .replace("events:\n", "  - name: projection.core.SourceTag\n    kind: newtype\n    of: String\n  - name: projection.core.TargetTag\n    kind: newtype\n    of: String\nevents:\n")
        .replace("name: text\n        type: projection.core.SourceTag", "name: text\n        type: projection.core.TargetTag")
        + "conversions:\n  - from: projection.core.SourceTag\n    to: projection.core.TargetTag\n    because: both names identify the same source label\n";
    for target in [Target::Rust, Target::Go] {
        let output = synthesize_for(&fixture_text(&text), target).unwrap();
        let code = output
            .artifacts
            .values()
            .map(|a| a.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            code.contains("fn project(event:") || code.contains("func Project(event "),
            "mechanical accessor transformation was left as an obligation"
        );
        if let Some(directory) = std::env::var_os("ESS_ACCESSOR_NATIVE_OUT") {
            let base = std::path::PathBuf::from(directory)
                .join("mechanical")
                .join(target.name());
            for (relative, artifact) in output.artifacts {
                let path = base.join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, artifact.contents).unwrap();
            }
        }
    }
}

#[test]
fn go_accessor_root_uses_the_allocated_event_member_identity() {
    let text = include_str!("fixtures/bounded-accessor.yaml").replace(
        "      - name: data\n",
        "      - name: Data\n        type: projection.core.Body\n      - name: data\n",
    );
    let output = synthesize_for(&fixture_text(&text), Target::Go).unwrap();
    let code = &output.artifacts["system/system.go"].contents;
    assert!(
        code.contains("event.Data_"),
        "the second declared field must not read the first field"
    );
    if let Some(directory) = std::env::var_os("ESS_ACCESSOR_NATIVE_OUT") {
        let base = std::path::PathBuf::from(directory).join("collision");
        for (relative, artifact) in output.artifacts {
            let path = base.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
    }
}

#[test]
fn old_cause_failures_on_accessor_models_use_failure3_while_legacy_bytes_keep_their_major() {
    let text = include_str!("fixtures/bounded-accessor.yaml").replacen(
        "name: result\n        type: String",
        "name: result\n        type: Binary64",
        1,
    );
    for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
        let failure = synthesize_for(&fixture_text(&text), target).err().unwrap();
        let json = serde_json::to_value(&failure).unwrap();
        assert_eq!(json["format"], "ess-target-failure/3");
        assert_eq!(json["causes"][0]["code"], "missing-representation");
        let legacy = text.split("bindings:").next().unwrap();
        let failure = synthesize_for(&fixture_text(legacy), target).err().unwrap();
        let json = serde_json::to_value(&failure).unwrap();
        assert_eq!(
            json["format"],
            if matches!(target, Target::Rust | Target::Web) {
                "ess-target-failure/1"
            } else {
                "ess-target-failure/2"
            }
        );
    }
}
