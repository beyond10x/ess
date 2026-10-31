//! Unsupported codecs must fail before creating or replacing any output artifact.

use std::{fs, path::Path, process::Command};

/// A note entity and its command, and a `Rate` type with a `Binary64` field, declared for another
/// component. `{input}` is the command's further input.
const NOTES: &str = "format: ess/31
system: demo
version: v1
domain: demo.notes
summary: A note is added; a rate type holds a finite floating-point figure.
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - name: demo.notes.Rate
    kind: struct
    fields:
      - {name: per_second, type: Binary64}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
events:
  - name: demo.notes.NoteAdded
    fields: [{name: note_id, type: demo.notes.NoteId}]
commands:
  - name: demo.notes.AddNote
    input:
      - {name: label, type: String}
{input}    outcomes:
      - name: added
        creates: demo.notes.Note
        instance: note_id
        sets: {label: input.label}
        emits: [demo.notes.NoteAdded]
        payload: {demo.notes.NoteAdded: {note_id: {generated: true}}}
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: label, type: String}
";

/// The `Rate` a command input reaches.
const REACHED: &str = "      - {name: rate, type: Optional<demo.notes.Rate>}\n";

#[test]
fn binary64_sparse_model_refuses_every_unsupported_publication_route() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/binary64-publication")
        .join(std::process::id().to_string());
    fs::create_dir_all(root.join("spec")).unwrap();
    fs::write(
        root.join("spec/system.yaml"),
        NOTES.replace("{input}", REACHED),
    )
    .unwrap();
    // Restated for story:a-binary64-value-round-trips-through-a-conformance-suite: the conformance
    // suite routes carry a finite Binary64 a command input reaches from suite/62
    // (`a_reached_binary64_publishes_every_conformance_route`); whole-system code generation and
    // the browser bundle still carry none and still refuse it.
    let operations: &[(&[&str], bool)] = &[
        (&["synthesize", "--target", "rust"], true),
        (&["synthesize", "--target", "go"], true),
        (&["synthesize", "--target", "web"], true),
        (&["synthesize", "--target", "clap"], true),
        (&["verify", "conform", "web"], true),
    ];
    for (index, (arguments, directory)) in operations.iter().enumerate() {
        for existing in [false, true] {
            let out = root.join(format!("out-{index}-{existing}"));
            if existing {
                if *directory {
                    fs::create_dir_all(&out).unwrap();
                    fs::write(out.join("sentinel"), "owned bytes\n").unwrap();
                } else {
                    fs::write(&out, "owned bytes\n").unwrap();
                }
            }
            let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
            command
                .args(*arguments)
                .arg("--path")
                .arg(root.join("spec"))
                .args(["--format", "json"])
                .arg(if arguments.contains(&"run") {
                    "--report-out"
                } else {
                    "--out"
                })
                .arg(&out);
            let result = command.output().unwrap();
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            fs::write(
                root.join(format!("case-{index}-{existing}.log")),
                format!("{command:?}\n{}\n{combined}", result.status),
            )
            .unwrap();
            assert!(!result.status.success(), "{arguments:?}: {combined}");
            assert!(
                combined.contains("Binary64") && combined.contains("demo.notes.Rate"),
                "located finite refusal: {combined}"
            );
            if arguments[0] == "synthesize" {
                let failure: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
                let expected = if arguments[2] == "rust" || arguments[2] == "web" {
                    "ess-target-failure/1"
                } else {
                    "ess-target-failure/2"
                };
                assert_eq!(failure["format"], expected);
            }
            if existing {
                let file = if *directory {
                    out.join("sentinel")
                } else {
                    out.clone()
                };
                assert_eq!(fs::read(file).unwrap(), b"owned bytes\n");
                if *directory {
                    assert_eq!(fs::read_dir(&out).unwrap().count(), 1);
                }
            } else {
                assert!(!out.exists(), "refusal created {}", out.display());
            }
        }
    }
    eprintln!("10 actual CLI refusal executions preserved absent/existing destinations");
}

/// story:a-binary64-value-round-trips-through-a-conformance-suite: with `Rate` reached through a
/// command input, every conformance suite route publishes, each suite at /62, and the interpreted
/// target passes the suite it synthesizes.
#[test]
fn a_reached_binary64_publishes_every_conformance_route() {
    const STORY: &str = "story:a-binary64-value-round-trips-through-a-conformance-suite";
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/binary64-publication-reached")
        .join(std::process::id().to_string());
    fs::create_dir_all(root.join("spec")).unwrap();
    fs::write(
        root.join("spec/system.yaml"),
        NOTES.replace("{input}", REACHED),
    )
    .unwrap();
    let operations: &[(&[&str], &str)] = &[
        (
            &["verify", "conform", "synthesize", "--target", "ir"],
            "--out",
        ),
        (
            &["verify", "conform", "synthesize", "--target", "go"],
            "--out",
        ),
        (
            &["verify", "conform", "synthesize", "--target", "typescript"],
            "--out",
        ),
        (&["verify", "conform", "author"], "--out"),
        (
            &[
                "verify",
                "conform",
                "run",
                "--target",
                "interpreted",
                "--report-format",
                "2",
            ],
            "--report-out",
        ),
    ];
    for (index, (arguments, destination)) in operations.iter().enumerate() {
        let out = root.join(format!("out-{index}"));
        let result = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(*arguments)
            .arg("--path")
            .arg(root.join("spec"))
            .arg(destination)
            .arg(&out)
            .output()
            .unwrap();
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            result.status.success(),
            "{STORY}: {arguments:?}: {combined}"
        );
        assert!(
            out.exists(),
            "{STORY}: {arguments:?} wrote nothing: {combined}"
        );
        assert!(
            !combined.contains("UnsupportedPrimitive"),
            "{STORY}: {arguments:?}: {combined}"
        );
        if arguments.contains(&"ir") {
            let suite = fs::read_to_string(&out).unwrap();
            assert!(
                suite.contains("\"ess-conformance/62\""),
                "{STORY}: the suite is /62: {suite}"
            );
        }
    }
    eprintln!("5 conformance routes published a reached Binary64");
}

/// The adopter's reproduction: nothing reaches `Rate`, so every conformance route admits the model
/// and `synthesize` notes the position instead of refusing it.
#[test]
fn an_unreached_binary64_type_publishes_every_route() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/binary64-publication-unreached")
        .join(std::process::id().to_string());
    fs::create_dir_all(root.join("spec")).unwrap();
    fs::write(root.join("spec/system.yaml"), NOTES.replace("{input}", "")).unwrap();
    let operations: &[(&[&str], &str)] = &[
        (
            &["verify", "conform", "synthesize", "--target", "ir"],
            "--out",
        ),
        (
            &["verify", "conform", "synthesize", "--target", "go"],
            "--out",
        ),
        (
            &["verify", "conform", "synthesize", "--target", "typescript"],
            "--out",
        ),
        (&["verify", "conform", "web"], "--out"),
        (
            &[
                "verify",
                "conform",
                "run",
                "--target",
                "interpreted",
                "--report-format",
                "2",
            ],
            "--report-out",
        ),
    ];
    for (index, (arguments, destination)) in operations.iter().enumerate() {
        let out = root.join(format!("out-{index}"));
        let result = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(*arguments)
            .arg("--path")
            .arg(root.join("spec"))
            .arg(destination)
            .arg(&out)
            .output()
            .unwrap();
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.status.success(), "{arguments:?}: {combined}");
        assert!(out.exists(), "{arguments:?} wrote nothing: {combined}");
        assert!(
            !combined.contains("UnsupportedPrimitive"),
            "{arguments:?}: {combined}"
        );
        if arguments.contains(&"ir") {
            assert!(
                combined.contains("types.demo.notes.Rate.fields.per_second"),
                "{arguments:?} notes the position: {combined}"
            );
        }
    }
    eprintln!("5 conformance routes admitted an unreached Binary64 type");
}
