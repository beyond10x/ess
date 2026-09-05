//! The actual CLI must refuse compiler-invalid target allocation before writing.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn fixture(target: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/review-boundaries-5/adversary-cli")
        .join(format!(
            "{target}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
    std::fs::create_dir_all(root.join("spec")).unwrap();
    std::fs::write(
        root.join("spec/system.yaml"),
        "format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\n",
    )
    .unwrap();
    let (component, event, reach) = if target == "rust" {
        ("worker", "Out", "    reached_by: network\n")
    } else {
        ("json", "Fired", "")
    };
    std::fs::write(root.join("spec/core.yaml"), format!("domain: demo.core\nevents:\n  - name: demo.core.{event}\ncommands:\n  - name: demo.core.Fire\n    outcomes:\n      - name: done\n        emits: [demo.core.{event}]\n")).unwrap();
    std::fs::write(root.join("spec/wiring.yaml"), format!("components:\n  - component: {component}\n    owns:\n      domains: [demo.core]\n    accepts:\n      commands: [demo.core.Fire]\n    publishes:\n      events: [demo.core.{event}]\n{reach}")).unwrap();
    root
}

fn refuses_before_writes(target: &str) {
    let root = fixture(target);
    let mut missed = Vec::new();
    for existing in [false, true] {
        let out = root.join(if existing { "existing" } else { "absent" });
        if existing {
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("sentinel.txt"), "must stay unchanged\n").unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .args(["synthesize", "--target", target, "--format", "json"])
            .arg("--path")
            .arg(root.join("spec"))
            .arg("--out")
            .arg(&out);
        let output = command.output().expect("actual CLI executes");
        let label = if existing { "existing" } else { "absent" };
        std::fs::write(
            root.join(format!("{label}.command")),
            format!("{command:?}\n"),
        )
        .unwrap();
        std::fs::write(root.join(format!("{label}.stdout")), &output.stdout).unwrap();
        std::fs::write(root.join(format!("{label}.stderr")), &output.stderr).unwrap();
        std::fs::write(
            root.join(format!("{label}.exit")),
            output.status.to_string(),
        )
        .unwrap();
        eprintln!(
            "{command:?}\n{}; stdout {} bytes, stderr {} bytes; complete streams: {}/{label}.stdout and .stderr",
            output.status,
            output.stdout.len(),
            output.stderr.len(),
            root.display()
        );
        if output.status.code() != Some(1) {
            missed.push(format!(
                "{label}: expected target refusal exit 1, got {}",
                output.status
            ));
        }
        let parsed: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("structured CLI output");
        if parsed["format"] != "ess-target-failure/1" || parsed["target"] != target {
            missed.push(format!(
                "{label}: no typed {target} target failure envelope"
            ));
        }
        if existing {
            assert_eq!(
                std::fs::read(out.join("sentinel.txt")).unwrap(),
                b"must stay unchanged\n"
            );
            let entries = std::fs::read_dir(&out).unwrap().count();
            if entries != 1 {
                missed.push(format!("{label}: output root now has {entries} entries"));
            }
        } else if out.exists() {
            missed.push(format!("{label}: rejected output root was created"));
        }
    }
    assert!(
        missed.is_empty(),
        "compiler-invalid target reached successful CLI output:\n{}",
        missed.join("\n")
    );
}

#[test]
fn http_codec_local_collision_is_refused_before_cli_output() {
    refuses_before_writes("rust");
}

#[test]
fn web_dependency_module_collision_is_refused_before_cli_output() {
    refuses_before_writes("web");
}

fn binding_refuses_before_writes(target: &str) {
    let root = fixture(&format!("pass-2-binding-{target}"));
    std::fs::write(root.join("spec/core.yaml"), "domain: demo.core\nevents:\n  - name: demo.core.Fired\n  - name: demo.core.Handled\ncommands:\n  - name: demo.core.Handle\n    outcomes:\n      - name: done\n        emits: [demo.core.Handled]\n").unwrap();
    std::fs::write(root.join("spec/wiring.yaml"), "components:\n  - component: worker\n    owns:\n      domains: [demo.core]\n    accepts:\n      commands: [demo.core.Handle]\n    publishes:\n      events: [demo.core.Fired, demo.core.Handled]\nbindings:\n  - id: event\n    when:\n      event: demo.core.Fired\n    invoke:\n      command: demo.core.Handle\n    mapping: {}\n    delivery: at_least_once\n    on_failure: drop\n").unwrap();
    let mut missed = Vec::new();
    for existing in [false, true] {
        let label = if existing { "existing" } else { "absent" };
        let out = root.join(label);
        if existing {
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("sentinel.txt"), "must stay unchanged\n").unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .args(["synthesize", "--target", target, "--format", "json"])
            .arg("--path")
            .arg(root.join("spec"))
            .arg("--out")
            .arg(&out);
        let output = command.output().unwrap();
        std::fs::write(
            root.join(format!("{label}.command")),
            format!("{command:?}\n"),
        )
        .unwrap();
        std::fs::write(root.join(format!("{label}.stdout")), &output.stdout).unwrap();
        std::fs::write(root.join(format!("{label}.stderr")), &output.stderr).unwrap();
        std::fs::write(
            root.join(format!("{label}.exit")),
            output.status.to_string(),
        )
        .unwrap();
        let mut entries = if out.exists() {
            std::fs::read_dir(&out)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        entries.sort();
        std::fs::write(
            root.join(format!("{label}.inventory")),
            format!("{entries:?}\n"),
        )
        .unwrap();
        eprintln!("{command:?}\n{}; output entries {entries:?}; complete streams: {}/{label}.stdout and .stderr", output.status, root.display());
        if output.status.code() != Some(1) {
            missed.push(format!("{label}: expected exit 1, got {}", output.status));
        }
        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        if parsed["format"] != "ess-target-failure/1" || parsed["target"] != target {
            missed.push(format!("{label}: no typed {target} failure"));
        }
        if existing {
            assert_eq!(
                std::fs::read(out.join("sentinel.txt")).unwrap(),
                b"must stay unchanged\n"
            );
            if entries.len() != 1 {
                missed.push(format!("{label}: destination gained files"));
            }
        } else if out.exists() {
            missed.push(format!("{label}: destination was created"));
        }
    }
    assert!(
        missed.is_empty(),
        "binding collision reached CLI writes:\n{}",
        missed.join("\n")
    );
}

#[test]
fn a_binding_function_capture_is_refused_before_rust_cli_writes() {
    binding_refuses_before_writes("rust");
}

#[test]
fn a_binding_function_capture_is_refused_before_web_cli_writes() {
    binding_refuses_before_writes("web");
}
