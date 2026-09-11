//! Whole-target failures must precede every output write, through both CLI spellings.

use std::path::Path;
use std::process::Command;

const OWNER_SYSTEM: &str = "format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\ntypes:\n  - {name: demo.Code, kind: newtype, of: String}\n";
const OWNER_CORE: &str = "domain: demo.core\n";

#[test]
fn accessor_models_version_early_target_failures_before_output_writes() {
    let source = include_str!("fixtures/bounded-accessor.yaml");
    for (target, model, cause) in [
        (
            "rust",
            source.replace("accessor.core", "accessor.lib"),
            "path-collision",
        ),
        (
            "web",
            source.replace("accessor.core", "accessor.lib"),
            "path-collision",
        ),
        (
            "go",
            source.replacen(
                "types:\n",
                "types:\n  - {name: accessor.Unowned, kind: newtype, of: String}\n",
                1,
            ),
            "missing-type-owner",
        ),
        (
            "clap",
            source.replacen(
                "types:\n",
                "types:\n  - {name: accessor.core.Ratio, kind: newtype, of: Binary64}\n",
                1,
            ),
            "missing-representation",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let spec = dir.path().join("system.yaml");
        std::fs::write(&spec, model).unwrap();
        for existing in [false, true] {
            let destination = dir
                .path()
                .join(if existing { "existing" } else { "absent" });
            if existing {
                std::fs::create_dir(&destination).unwrap();
                std::fs::write(destination.join("plan.json"), "untouched\n").unwrap();
            }
            for format in ["json", "yaml"] {
                let output = Command::new(env!("CARGO_BIN_EXE_ess"))
                    .args(["synthesize", "--path"])
                    .arg(&spec)
                    .args(["--target", target, "--format", format, "--out"])
                    .arg(&destination)
                    .output()
                    .unwrap();
                assert_eq!(output.status.code(), Some(1), "{target}: {output:?}");
                assert!(output.stderr.is_empty(), "{target}: {output:?}");
                let failure: serde_json::Value = if format == "json" {
                    serde_json::from_slice(&output.stdout).unwrap()
                } else {
                    serde_yaml::from_slice(&output.stdout).unwrap()
                };
                assert_eq!(
                    failure["format"], "ess-target-failure/3",
                    "{target}: {failure}"
                );
                assert_eq!(failure["target"], target);
                assert!(
                    failure["causes"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|row| row["code"] == cause),
                    "{target}: {failure}"
                );
                if existing {
                    assert_eq!(
                        std::fs::read_to_string(destination.join("plan.json")).unwrap(),
                        "untouched\n"
                    );
                    assert_eq!(std::fs::read_dir(&destination).unwrap().count(), 1);
                } else {
                    assert!(!destination.exists());
                }
            }
        }
    }
}

fn owner_plan() -> serde_json::Value {
    let mut sources = ess_compiler::source::SourceMap::new();
    let files = [("system.yaml", OWNER_SYSTEM), ("core.yaml", OWNER_CORE)]
        .into_iter()
        .map(|(label, text)| {
            sources.insert(label.to_owned(), text.to_owned());
            (
                ess_domain::system::Source::new(label),
                ess_domain::spec::RawSpecFile::parse(text).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let spec = ess_domain::spec::Specification::assemble(files).unwrap();
    let ir = ess_compiler::compile(&spec, &sources).unwrap();
    serde_json::to_value(ess_synth::SynthesisPlan::of(&ir)).unwrap()
}

#[test]
fn go_missing_owner_refuses_before_any_destination_write() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let fixture = root.join(format!(
        "target/review-specification-fuzzing-wave20/cli-owner-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(fixture.join("spec")).unwrap();
    std::fs::write(fixture.join("spec/system.yaml"), OWNER_SYSTEM).unwrap();
    std::fs::write(fixture.join("spec/core.yaml"), OWNER_CORE).unwrap();
    let plan = owner_plan();
    for format in ["text", "json", "yaml"] {
        for destination in ["none", "absent", "existing"] {
            let parent = fixture.join(format!("{format}-{destination}"));
            let out = parent.join("output");
            if destination == "existing" {
                std::fs::create_dir_all(&out).unwrap();
                std::fs::write(out.join("plan.json"), b"untouched plan\n").unwrap();
            }
            let mut streams = Vec::new();
            for prefix in [&["synthesize"][..], &["generate", "synthesize"][..]] {
                let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
                command
                    .args(prefix)
                    .arg("--path")
                    .arg(fixture.join("spec"))
                    .args(["--target", "go", "--format", format]);
                if destination != "none" {
                    command.arg("--out").arg(&out);
                }
                let output = command.output().unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(1),
                    "{command:?}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(output.stderr.is_empty(), "failure belongs on stdout");
                if format == "text" {
                    let text = String::from_utf8(output.stdout.clone()).unwrap();
                    assert!(text.contains("go target cannot emit this workspace"));
                    assert!(text.contains("demo.Code: Go cannot assign a package to type demo.Code: no domain owns it"));
                    assert!(!text.contains("artifact(s)"));
                } else {
                    let value: serde_json::Value = if format == "json" {
                        serde_json::from_slice(&output.stdout).unwrap()
                    } else {
                        serde_yaml::from_slice(&output.stdout).unwrap()
                    };
                    assert_eq!(value["format"], "ess-target-failure/2");
                    assert_eq!(value["target"], "go");
                    assert_eq!(value["plan"], plan);
                    assert_eq!(
                        value["causes"],
                        serde_json::json!([{"code":"missing-type-owner","sources":["demo.Code"],"detail":"Go cannot assign a package to type demo.Code: no domain owns it"}])
                    );
                    assert_eq!(value.as_object().unwrap().len(), 4);
                }
                streams.push((output.stdout, output.stderr));
                if destination == "existing" {
                    assert_eq!(
                        std::fs::read(out.join("plan.json")).unwrap(),
                        b"untouched plan\n"
                    );
                    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 1);
                    assert_eq!(std::fs::read_dir(&parent).unwrap().count(), 1);
                } else {
                    assert!(
                        !parent.exists(),
                        "absent destination and parent stay absent"
                    );
                }
            }
            assert_eq!(streams[0], streams[1]);
        }
    }
}

fn offered_choices(args: &[&str]) -> std::collections::BTreeSet<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let text = String::from_utf8(output.stderr).unwrap();
    let marker = "[possible values: ";
    assert_eq!(
        text.matches(marker).count(),
        1,
        "complete clap value list: {text}"
    );
    let values = text
        .split_once(marker)
        .unwrap()
        .1
        .split_once(']')
        .unwrap()
        .0;
    let rows = values.split(", ").map(str::to_owned).collect::<Vec<_>>();
    let set = rows
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(set.len(), rows.len());
    set
}

fn target_name(target: ess_synth::Target) -> &'static str {
    match target {
        ess_synth::Target::Rust => "rust",
        ess_synth::Target::Go => "go",
        ess_synth::Target::Web => "web",
        ess_synth::Target::Clap => "clap",
    }
}

#[test]
fn offered_cli_choices_match_every_current_library_projection_and_target() {
    let mut projections = ess_gen::generators()
        .iter()
        .map(|g| g.name().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(projections.len(), ess_gen::generators().len());
    assert!(projections.insert("docs-ir".to_owned()));
    assert_eq!(
        offered_choices(&["generate", "--kind", "__unknown_fuzz_choice__"]),
        projections
    );
    let targets = [
        ess_synth::Target::Rust,
        ess_synth::Target::Go,
        ess_synth::Target::Web,
        ess_synth::Target::Clap,
    ]
    .map(|t| {
        assert_eq!(target_name(t), t.name());
        t.name().to_owned()
    })
    .into_iter()
    .collect();
    assert_eq!(
        offered_choices(&["synthesize", "--target", "__unknown_fuzz_choice__"]),
        targets
    );
    assert_eq!(
        offered_choices(&[
            "generate",
            "synthesize",
            "--target",
            "__unknown_fuzz_choice__"
        ]),
        targets
    );
}

#[test]
fn fatal_synthesis_preserves_destinations_and_has_a_typed_envelope() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let fixture = root.join(format!(
        "target/review-boundaries-5/cli-failure-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(fixture.join("spec")).unwrap();
    std::fs::write(
        fixture.join("spec/system.yaml"),
        "format: ess/1\nsystem: demo\nversion: v1\ndomains:\n  - demo.lib\n",
    )
    .unwrap();
    std::fs::write(fixture.join("spec/lib.yaml"), "domain: demo.lib\n").unwrap();
    for target in ["rust", "web"] {
        for format in ["text", "json", "yaml"] {
            for destination in ["absent", "existing", "none"] {
                let out = fixture.join(format!("{target}-{format}-{destination}"));
                if destination == "existing" {
                    std::fs::create_dir_all(&out).unwrap();
                    std::fs::write(out.join("plan.json"), "untouched plan\n").unwrap();
                }
                let mut streams = Vec::new();
                for prefix in [&["generate", "synthesize"][..], &["synthesize"][..]] {
                    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
                    command
                        .args(prefix)
                        .arg("--path")
                        .arg(fixture.join("spec"))
                        .args(["--target", target, "--format", format]);
                    if destination != "none" {
                        command.arg("--out").arg(&out);
                    }
                    let output = command.output().unwrap();
                    assert_eq!(
                        output.status.code(),
                        Some(1),
                        "{command:?}: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    assert!(
                        output.stderr.is_empty(),
                        "structured target failure belongs on stdout"
                    );
                    if format == "text" {
                        let text = String::from_utf8_lossy(&output.stdout);
                        assert!(text.contains("demo.lib") && text.contains(target));
                        assert!(!text.contains("artifact(s)"));
                    } else {
                        let value: serde_json::Value = if format == "json" {
                            serde_json::from_slice(&output.stdout).unwrap()
                        } else {
                            serde_yaml::from_slice(&output.stdout).unwrap()
                        };
                        assert_eq!(value["format"], "ess-target-failure/1");
                        assert_eq!(value["target"], target);
                        assert_eq!(value["plan"]["capabilities"], serde_json::json!([]));
                        assert!(!value["causes"].as_array().unwrap().is_empty());
                    }
                    streams.push((output.stdout, output.stderr));
                }
                assert_eq!(streams[0], streams[1]);
                if destination == "existing" {
                    assert_eq!(
                        std::fs::read(out.join("plan.json")).unwrap(),
                        b"untouched plan\n"
                    );
                    assert_eq!(std::fs::read_dir(out).unwrap().count(), 1);
                } else {
                    assert!(!out.exists());
                }
            }
        }
    }
}
