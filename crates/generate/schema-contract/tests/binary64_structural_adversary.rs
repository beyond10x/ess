//! Adversarial source-token paths through compiler-owned, mutually recursive model roots.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use schema_contract::realize::Plan;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE: &str = r"format: ess/2
system: probe
version: v1
domains: [probe.float]
domain: probe.float
types:
  - {name: probe.float.Scalar, kind: newtype, of: Binary64}
  - {name: probe.float.Maybe, kind: newtype, of: 'Optional<probe.float.Scalar>'}
  - name: probe.float.A
    kind: struct
    fields:
      - {name: number, wire: 'a/b~', type: probe.float.Scalar}
      - {name: next, type: 'Optional<probe.float.B>'}
  - name: probe.float.B
    kind: struct
    fields:
      - {name: back, type: 'Optional<probe.float.A>'}
      - {name: numbers, type: 'Map<String, List<Optional<probe.float.Scalar>>>'}
  - name: probe.float.Choice
    kind: union
    tag: value
    variants:
      node: probe.float.A
      numbers: 'Map<String, List<Optional<probe.float.Scalar>>>'
      text: String
  - name: probe.float.Envelope
    kind: struct
    fields:
      - {name: choice, type: probe.float.Choice}
      - {name: nullableBag, type: 'Map<String, Optional<probe.float.Choice>>'}
      - {name: optionalMaybe, type: 'Optional<probe.float.Maybe>'}
      - {name: optionalInts, type: 'Optional<Map<Integer, Binary64>>'}
  - {name: probe.float.Plain, kind: newtype, of: Integer}
";

fn plan() -> Plan {
    Plan::from_model(&model::selection(SOURCE, &["probe.float.Envelope"]))
        .expect("public compiler-owned recursive selection")
}

fn native_root(language: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "binary64-structural-adversary-{language}-{}",
        std::process::id()
    ))
}

fn run(command: &mut Command) {
    println!("native command: {command:?}");
    let result = command.output().expect("execute native case");
    println!(
        "native stdout:\n{}\nnative stderr:\n{}\nnative exit: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
        result.status
    );
    assert!(result.status.success(), "native assertion failed");
}

#[test]
fn rust_recursive_unions_and_nullable_maps_keep_original_tokens() {
    let generated = plan().rust("adversary_types").unwrap();
    let root = native_root("rust");
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::write(root.join("Cargo.toml"), &generated.supporting["Cargo.toml"]).unwrap();
    std::fs::write(root.join("types.rs"), generated.declarations).unwrap();
    std::fs::write(
        root.join("tests/wire.rs"),
        include_str!("fixtures/binary64_structural_adversary_rust.txt"),
    )
    .unwrap();
    run(Command::new(env!("CARGO"))
        .args(["generate-lockfile", "--offline"])
        .current_dir(&root));
    for case in [
        "recursive_ref_map_union_paths_keep_bits_and_float_markers",
        "marker_objects_and_late_invalid_values_cannot_take_a_float_path",
        "optional_nullable_alias_and_independent_text_alternative_remain_distinct",
    ] {
        run(Command::new(env!("CARGO"))
            .args([
                "test",
                "--offline",
                "--locked",
                "--test",
                "wire",
                case,
                "--",
                "--exact",
                "--nocapture",
            ])
            .current_dir(&root));
    }
}

#[cfg(feature = "go-typecheck")]
#[test]
fn go_recursive_maps_and_failed_decode_preserve_complete_receiver_state() {
    let generated = plan()
        .go("adversary_types", "example.invalid/adversary-types")
        .unwrap();
    let root = native_root("go");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("go.mod"), &generated.supporting["go.mod"]).unwrap();
    std::fs::write(root.join("types.go"), generated.declarations).unwrap();
    std::fs::write(
        root.join("wire_test.go"),
        include_str!("fixtures/binary64_structural_adversary_go.txt"),
    )
    .unwrap();
    for case in [
        "^TestRecursiveSourceBits$",
        "^TestEveryLateFailureLeavesTheWholeReceiverUntouched$",
        "^TestOptionalNullableAndRetryResetAreExplicit$",
    ] {
        run(Command::new(
            std::env::var_os("ESS_GO_COMPILER")
                .expect("explicit go-typecheck lane requires ESS_GO_COMPILER"),
        )
        .args([
            "test", "-count=1", "-race", "-p=1", "-v", "-run", case, "./...",
        ])
        .current_dir(&root)
        .env("GOTOOLCHAIN", "local")
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOWORK", "off")
        .env("GOFLAGS", ""));
    }
}

#[test]
fn compiler_owned_helper_names_and_selected_root_reports_remain_conditional() {
    let selected = model::selection(SOURCE, &["probe.float.Envelope"]);
    let expected = selected.binary64_locations().clone();
    assert!(!expected.is_empty());
    let plan = Plan::from_model(&selected).unwrap();
    let rust = json!(plan.rust("adversary_types").unwrap().report);
    let go = json!(
        plan.go("adversary_types", "example.invalid/adversary-types")
            .unwrap()
            .report
    );
    let ts = json!(plan.typescript().report);
    for report in [&rust, &go, &ts] {
        assert_eq!(report["roots"], json!(["probe.float.Envelope"]));
        assert_eq!(report["input"]["kind"], "model");
        assert!(report["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["rule"] == "model_map_keys"));
    }
    for at in &expected {
        assert!(ts["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["rule"] == "model_binary64" && e["pointer"] == *at));
        assert!(rust["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["rule"] == "rust_binary64_source" && e["pointer"] == *at));
    }
    for report in [&rust, &go] {
        assert!(!report["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["rule"] == "model_binary64"));
        assert!(report["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["rule"] == "oneOf"));
    }
    let non_binary64 = Plan::from_model(&model::selection(SOURCE, &["probe.float.Plain"])).unwrap();
    assert!(
        !non_binary64.rust("plain_types").unwrap().supporting["Cargo.toml"].contains("raw_value")
    );
    assert!(!non_binary64
        .go("plain_types", "example.invalid/plain-types")
        .unwrap()
        .declarations
        .contains("type EssBinary64 struct"));

    for (name, rust_collision, go_collision) in [
        ("ess.Binary64", true, true),
        ("ess.Binary64Error", true, false),
        ("new.ess.Binary64", false, true),
    ] {
        let domain = name.rsplit_once('.').unwrap().0;
        let system = domain.split('.').next().unwrap();
        let finite = format!("{domain}.Finite");
        let source = format!("format: ess/2\nsystem: {system}\nversion: v1\ndomains: []\ntypes:\n  - {{name: {name}, kind: newtype, of: String}}\n  - {{name: {finite}, kind: newtype, of: Binary64}}\n");
        // Public compiler names, never mutations of sealed target metadata.
        let collision = Plan::from_model(&model::selection(&source, &[name, &finite])).unwrap();
        let rust = collision.rust("adversary_types");
        let go = collision.go("adversary_types", "example.invalid/adversary-types");
        assert_eq!(rust.is_err(), rust_collision, "Rust helper {name}");
        assert_eq!(go.is_err(), go_collision, "Go helper {name}");
        if let Err(errors) = rust {
            assert!(errors.0.iter().any(|e| e.rule == "rust_helper_collision"));
        }
        if let Err(errors) = go {
            assert!(errors.0.iter().any(|e| e.rule == "go_helper_collision"));
        }
    }
}
