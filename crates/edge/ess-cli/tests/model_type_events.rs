//! `ess generate types` realizes event payloads (beyond10x/ess#393).
//!
//! `--root <event>` and `--all-events` select an event's payload; `--all-events` combines with
//! `--all-types`; `--root` with either `--all-*` selector is refused, as `--root` with
//! `--all-types` always was.

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    process::{Command, Output},
};

const MODEL: &str = "format: ess/20
system: demo
version: v1
domain: demo.metering
types:
  - name: demo.metering.Source
    kind: struct
    fields:
      - {name: service, type: String}
events:
  - name: demo.metering.UsageRecorded
    naming: {wire: usage}
    fields:
      - {name: id, type: String}
      - {name: source, type: demo.metering.Source}
      - {name: created_at, type: Timestamp, wire: createdAt}
";

struct Fixture(TemporaryDirectory);
impl Fixture {
    fn new() -> Self {
        let path = TemporaryDirectory::create("ess-model-type-events").unwrap();
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(path.join("model/system.yaml"), MODEL).unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(["generate", "types", "--path", "model"])
            .args(args)
            .output()
            .unwrap()
    }
    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}
#[test]
fn an_event_root_realizes_the_payload_struct() {
    let fixture = Fixture::new();
    let output = fixture.run(&[
        "--root",
        "demo.metering.UsageRecorded",
        "--target",
        "rust",
        "--package",
        "demo-metering",
        "--out",
        "rust",
    ]);
    assert!(output.status.success(), "{output:?}");
    let types = fixture.read("rust/types.rs");
    assert!(
        types.contains("pub struct DemoMeteringUsageRecorded {"),
        "{types}"
    );
    assert!(
        types.contains("#[serde(rename = \"createdAt\")]"),
        "{types}"
    );
    assert!(types.contains("pub struct DemoMeteringSource {"), "{types}");
}

#[test]
fn all_events_combines_with_all_types() {
    let fixture = Fixture::new();
    let output = fixture.run(&[
        "--all-types",
        "--all-events",
        "--target",
        "go",
        "--package",
        "demometering",
        "--module",
        "example.invalid/demometering",
        "--out",
        "go",
    ]);
    assert!(output.status.success(), "{output:?}");
    let report = fixture.read("go/types-report.json");
    for root in ["demo.metering.Source", "demo.metering.UsageRecorded"] {
        assert!(report.contains(&format!("\"{root}\"")), "{report}");
    }
}

#[test]
fn a_root_beside_an_all_selector_is_refused() {
    let fixture = Fixture::new();
    for selector in ["--all-types", "--all-events"] {
        let output = fixture.run(&[
            "--root",
            "demo.metering.UsageRecorded",
            selector,
            "--target",
            "typescript",
            "--out",
            "ts",
        ]);
        assert!(!output.status.success(), "{selector}: {output:?}");
        assert!(!fixture.0.join("ts").exists(), "{selector} wrote output");
    }
}

/// `--rust-raw-json` (beyond10x/ess#577) names model locations; the Rust target alone takes it.
mod rust_raw_json {
    use super::*;

    const RAW: &str = "format: ess/22
system: demo
version: v1
domain: demo.store
types:
  - name: demo.store.Record
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
";

    fn fixture() -> Fixture {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("model/system.yaml"), RAW).unwrap();
        fixture
    }

    fn types(fixture: &Fixture, target: &str, raw: &str) -> Output {
        let mut args = vec!["--root", "demo.store.Record", "--target", target];
        args.extend(["--package", "demo-store", "--out", "out"]);
        if target == "go" {
            args.extend(["--module", "example.com/demo/store"]);
        }
        args.extend(["--rust-raw-json", raw]);
        fixture.run(&args)
    }

    #[test]
    fn raw_json_on_a_field_keeps_its_text_in_the_rust_library() {
        let fixture = fixture();
        let output = types(&fixture, "rust", "demo.store.Record.body");
        assert!(output.status.success(), "{output:?}");
        let types = fixture.read("out/types.rs");
        assert!(types.contains("pub body: EssRawJson,"), "{types}");
        assert!(!types.contains("::serde_json::Value"), "{types}");
        assert!(
            fixture.read("out/Cargo.toml").contains("raw_value"),
            "{types}"
        );
    }

    #[test]
    fn raw_json_names_an_unknown_or_non_json_location_and_is_refused() {
        for named in ["demo.store.Missing", "demo.store.Record.id"] {
            let fixture = fixture();
            let output = types(&fixture, "rust", named);
            assert!(!output.status.success(), "{named}: {output:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("--rust-raw-json") && stderr.contains(&format!("{named:?}")),
                "{named}: {stderr}"
            );
            assert!(!fixture.0.join("out").exists(), "{named}");
        }
    }

    #[test]
    fn raw_json_is_refused_for_a_target_other_than_rust() {
        for target in ["typescript", "go"] {
            let fixture = fixture();
            let output = types(&fixture, target, "demo.store.Record.body");
            assert!(!output.status.success(), "{target}: {output:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("--rust-raw-json is only supported for the Rust target"),
                "{target}: {stderr}"
            );
        }
    }
}

/// A command's input and response and a view's row are roots (beyond10x/ess#656):
/// `--root <command>.input`, `--root <command>.response`, `--root <view>.row`, and every one of
/// them under `--all-commands` and `--all-views`.
mod command_and_view_roots {
    use super::*;

    const NOTES: &str =
        include_str!("../../../generate/ess-gen/tests/fixtures/command-view-roots.yaml");

    fn notes() -> Fixture {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("model/system.yaml"), NOTES).unwrap();
        fixture
    }

    fn generate(fixture: &Fixture, selection: &[&str], target: &str) -> Output {
        let mut args = selection.to_vec();
        args.extend(["--target", target, "--out", target]);
        match target {
            "rust" => args.extend(["--package", "demo-notes"]),
            "go" => args.extend([
                "--package",
                "demonotes",
                "--module",
                "example.invalid/notes",
            ]),
            _ => {}
        }
        fixture.run(&args)
    }

    #[test]
    fn a_command_input_is_a_root() {
        for (target, file, declaration) in [
            ("rust", "rust/types.rs", "pub struct DemoNotesCreateInput {"),
            ("go", "go/types.go", "type DemoNotesCreateInput struct {"),
            (
                "typescript",
                "typescript/types.ts",
                "export type DemoNotesCreateInput = ",
            ),
        ] {
            let fixture = notes();
            let output = generate(&fixture, &["--root", "demo.notes.Create.input"], target);
            assert!(output.status.success(), "{target}: {output:?}");
            let types = fixture.read(file);
            assert!(types.contains(declaration), "{target}: {types}");
            assert!(types.contains("isPinned"), "{target}: {types}");
        }
    }

    #[test]
    fn a_command_response_is_a_root() {
        let fixture = notes();
        let output = generate(&fixture, &["--root", "demo.notes.Create.response"], "rust");
        assert!(output.status.success(), "{output:?}");
        let types = fixture.read("rust/types.rs");
        assert!(
            types.contains("pub struct DemoNotesCreateResponse {"),
            "{types}"
        );
        assert!(
            types.contains("#[serde(rename = \"titleLength\")]"),
            "{types}"
        );
    }

    #[test]
    fn a_view_row_is_a_root() {
        let fixture = notes();
        let output = generate(
            &fixture,
            &[
                "--root",
                "demo.notes.Notes.row",
                "--root",
                "demo.notes.Summaries.row",
            ],
            "rust",
        );
        assert!(output.status.success(), "{output:?}");
        let types = fixture.read("rust/types.rs");
        assert!(types.contains("pub struct DemoNotesNotesRow {"), "{types}");
        assert!(
            types.contains("pub struct DemoNotesSummariesRow("),
            "{types}"
        );
        assert!(types.contains("pub struct DemoNotesSummary {"), "{types}");
    }

    #[test]
    fn all_switches_include_them() {
        let fixture = notes();
        let output = generate(&fixture, &["--all-commands", "--all-views"], "typescript");
        assert!(output.status.success(), "{output:?}");
        let report: serde_json::Value =
            serde_json::from_str(&fixture.read("typescript/types-report.json")).unwrap();
        let roots: Vec<&str> = report["roots"]
            .as_array()
            .unwrap()
            .iter()
            .map(|root| root.as_str().unwrap())
            .collect();
        assert_eq!(
            roots,
            [
                "demo.notes.Create.input",
                "demo.notes.Create.response",
                "demo.notes.Notes.row",
                "demo.notes.Pin.input",
                "demo.notes.Summaries.row",
            ],
            "a command without a response has no response root"
        );

        let fixture = notes();
        let output = generate(
            &fixture,
            &[
                "--all-types",
                "--all-events",
                "--all-commands",
                "--all-views",
            ],
            "rust",
        );
        assert!(output.status.success(), "{output:?}");
        let report = fixture.read("rust/types-report.json");
        for root in [
            "demo.notes.NoteId",
            "demo.notes.Created",
            "demo.notes.Pin.input",
            "demo.notes.Notes.row",
        ] {
            assert!(report.contains(&format!("\"{root}\"")), "{report}");
        }

        for selector in ["--all-commands", "--all-views"] {
            let fixture = notes();
            let output = generate(
                &fixture,
                &["--root", "demo.notes.Create.input", selector],
                "typescript",
            );
            assert!(!output.status.success(), "{selector}: {output:?}");
            assert!(
                !fixture.0.join("typescript").exists(),
                "{selector} wrote output"
            );
        }
    }
}
