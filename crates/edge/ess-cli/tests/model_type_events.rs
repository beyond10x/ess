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

/// `--rust-derive`, `--rust-fold-null` and `--rust-drop-unknown-members` (beyond10x/ess#657,
/// #667, #668) reach the Rust realizer; the library's own scenarios are in schema-contract's
/// `rust_generator_options.rs`.
mod rust_generator_options {
    use super::*;

    const MODEL: &str = "format: ess/22
system: demo
version: v1
domain: demo.lab
types:
  - name: demo.lab.Tag
    kind: newtype
    of: String
  - name: demo.lab.Reading
    kind: struct
    fields:
      - {name: tag, type: demo.lab.Tag}
      - {name: note, type: 'Optional<String>'}
      - {name: ratio, type: Binary64}
";

    fn types(target: &str, options: &[&str]) -> (Fixture, Output) {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("model/system.yaml"), MODEL).unwrap();
        let mut args = vec!["--root", "demo.lab.Reading", "--target", target];
        args.extend(["--package", "demo-lab", "--out", "out"]);
        if target == "go" {
            args.extend(["--module", "example.com/demo/lab"]);
        }
        args.extend(options);
        let output = fixture.run(&args);
        (fixture, output)
    }

    #[test]
    fn no_option_keeps_the_bytes() {
        let (fixture, output) = types("rust", &[]);
        assert!(output.status.success(), "{output:?}");
        let types = fixture.read("out/types.rs");
        assert!(
            types.contains("#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]\n#[serde(transparent)]\npub struct DemoLabTag"),
            "{types}"
        );
        let report = fixture.read("out/types-report.json");
        assert!(
            report.contains("\"configuration\": {\n    \"language\": \"rust\",\n    \"package\": \"demo-lab\"\n  }"),
            "{report}"
        );
    }

    #[test]
    fn a_per_type_derive_is_added_to_that_type_only() {
        let (fixture, output) = types(
            "rust",
            &[
                "--rust-derive",
                "demo.lab.Tag=Hash",
                "--rust-derive",
                "demo.lab.Tag=Eq",
            ],
        );
        assert!(output.status.success(), "{output:?}");
        let types = fixture.read("out/types.rs");
        assert!(
            types.contains("::serde::Deserialize, ::std::cmp::Eq, ::std::hash::Hash)]\n#[serde(transparent)]\npub struct DemoLabTag"),
            "{types}"
        );
        assert_eq!(types.matches("::std::hash::Hash)]").count(), 1, "{types}");
    }

    #[test]
    fn an_unsatisfiable_derive_is_refused_by_name() {
        let (fixture, output) = types("rust", &["--rust-derive", "Eq"]);
        assert!(!output.status.success(), "{output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("rust_derive_unsatisfied")
                && stderr
                    .contains("`DemoLabReading` cannot derive Eq: field `ratio` holds EssBinary64"),
            "{stderr}"
        );
        assert!(!fixture.0.join("out").exists());
        let (_, output) = types("rust", &["--rust-derive", "Copy"]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() && stderr.contains("--rust-derive Copy"),
            "{stderr}"
        );
    }

    #[test]
    fn the_report_records_each_option() {
        let (fixture, output) = types("rust", &["--rust-fold-null", "--rust-drop-unknown-members"]);
        assert!(output.status.success(), "{output:?}");
        let types = fixture.read("out/types.rs");
        assert!(
            types.contains("deserialize_with = \"ess_fold_null\""),
            "{types}"
        );
        let report = fixture.read("out/types-report.json");
        for expected in [
            "\"fold_null\": true",
            "\"drop_unknown_members\": true",
            "\"rule\": \"rust_fold_null\"",
        ] {
            assert!(report.contains(expected), "{expected}: {report}");
        }
    }

    #[test]
    fn the_options_are_refused_for_a_target_other_than_rust() {
        for target in ["typescript", "go"] {
            for (option, flag) in [
                (&["--rust-derive", "Eq"][..], "--rust-derive"),
                (&["--rust-fold-null"][..], "--rust-fold-null"),
                (
                    &["--rust-drop-unknown-members"][..],
                    "--rust-drop-unknown-members",
                ),
            ] {
                let (_, output) = types(target, option);
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    !output.status.success()
                        && stderr
                            .contains(&format!("{flag} is only supported for the Rust target")),
                    "{target} {flag}: {stderr}"
                );
            }
        }
    }

    /// The SHA-256 of each file `ess generate types` wrote for the declaration-order fixture
    /// before `--declared-field-order` and `--rust-box-only-recursion` existed, the generator's
    /// version replaced by `<version>`: the declarations, then the report, for each target.
    const BEFORE: [(&str, &str, &str, &str); 3] = [
        (
            "rust",
            "types.rs",
            "2d05fd72d7aaad1c8712872e69f8bddbed3272a3351a14bf21b4e72a67d2bcde",
            "17f7b053eabf91d3bdca447e4b301c8a7007a95ebd07c18ae6daaa00c0fb0837",
        ),
        (
            "go",
            "types.go",
            "ec665e71dee281af5f0d780fcde55c391c63936711aba596be06025021e6e20a",
            "0a4fbcf1a5ee029c2148b421556a5863397d64a97c4b8d4289071e7466a888fb",
        ),
        (
            "typescript",
            "types.ts",
            "c4198fd65af95b63e59b9833c691783155ccda9ffce9b78e074acfdb2f4ef7a4",
            "b775b4addeec2166e5cb5e6a808ddd110b1e5a0ecdb3bf8ba1e4f1d7bb972634",
        ),
    ];

    fn ordered(target: &str, options: &[&str]) -> (Fixture, Output) {
        let fixture = Fixture::new();
        fs::write(
            fixture.0.join("model/system.yaml"),
            super::declaration_order::MODEL,
        )
        .unwrap();
        let mut args = Vec::new();
        for root in &super::declaration_order::ROOTS {
            args.extend(["--root", root]);
        }
        args.extend(["--target", target, "--out", "out"]);
        match target {
            "rust" => args.extend(["--package", "demo-g"]),
            "go" => args.extend(["--package", "demog", "--module", "example.com/demo/g"]),
            _ => {}
        }
        args.extend(options);
        let output = fixture.run(&args);
        (fixture, output)
    }

    fn digest(text: &str) -> String {
        use sha2::Digest as _;
        use std::fmt::Write as _;
        let text = text.replace(env!("CARGO_PKG_VERSION"), "<version>");
        sha2::Sha256::digest(text.as_bytes())
            .iter()
            .fold(String::new(), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            })
    }

    #[test]
    fn declared_field_order_and_box_only_recursion_are_opt_in() {
        for (target, declarations, before, report) in BEFORE {
            let (fixture, output) = ordered(target, &[]);
            assert!(output.status.success(), "{target}: {output:?}");
            assert_eq!(
                digest(&fixture.read(&format!("out/{declarations}"))),
                before,
                "{target}"
            );
            assert_eq!(
                digest(&fixture.read("out/types-report.json")),
                report,
                "{target}"
            );
        }

        let (fixture, output) = ordered(
            "rust",
            &["--declared-field-order", "--rust-box-only-recursion"],
        );
        assert!(output.status.success(), "{output:?}");
        let types = fixture.read("out/types.rs");
        let outer = &types[types.find("pub struct DemoGOuter {").unwrap()..];
        let zeta = outer.find("pub zeta: DemoGInner,").unwrap();
        let alpha = outer.find("pub alpha: ").unwrap();
        let mid = outer.find("pub mid: EssPresence<DemoGInner>,").unwrap();
        assert!(zeta < alpha && alpha < mid, "{outer}");
        assert!(
            types.contains("pub next: EssPresence<::std::boxed::Box<DemoGNode>>,")
                && types.contains("pub tree: DemoGNode,"),
            "{types}"
        );
        let report = fixture.read("out/types-report.json");
        for expected in [
            "\"declared_field_order\": true",
            "\"box_only_recursion\": true",
        ] {
            assert!(report.contains(expected), "{expected}: {report}");
        }

        for target in ["go", "typescript"] {
            let (fixture, output) = ordered(target, &["--declared-field-order"]);
            assert!(output.status.success(), "{target}: {output:?}");
            let report = fixture.read("out/types-report.json");
            assert!(
                report.contains("\"declared_field_order\": true"),
                "{target}: {report}"
            );
            let (_, output) = ordered(target, &["--rust-box-only-recursion"]);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success()
                    && stderr.contains(
                        "--rust-box-only-recursion is only supported for the Rust target"
                    ),
                "{target}: {stderr}"
            );
        }

        let fixture = Fixture::new();
        let source = r#"{"openapi": "3.0.0", "info": {"title": "", "version": ""}, "paths": {},
            "components": {"schemas": {"Root": {"type": "object",
                "properties": {"b": {"type": "string"}, "a": {"type": "string"}}}}}}"#;
        fs::write(fixture.0.join("source.json"), source).unwrap();
        let schema = |args: &[&str]| {
            Command::new(env!("CARGO_BIN_EXE_ess"))
                .current_dir(&fixture.0)
                .args(["generate", "schema"])
                .args(args)
                .output()
                .unwrap()
        };
        let imported = schema(&[
            "import-bundle",
            "--path",
            "source.json",
            "--component",
            "Root",
            "--dialect",
            "draft-2020-12",
            "--out",
            "root.bundle.json",
        ]);
        assert!(imported.status.success(), "{imported:?}");
        let refused = schema(&[
            "types-bundle",
            "--bundle",
            "root.bundle.json",
            "--root",
            "Root",
            "--target",
            "typescript",
            "--declared-field-order",
            "--out",
            "types",
        ]);
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            !refused.status.success() && stderr.contains("declared_field_order_bundle"),
            "{stderr}"
        );
        assert!(!fixture.0.join("types").exists());
    }
}

#[allow(dead_code)]
#[path = "../../../generate/ess-gen/tests/fixtures/declaration_order.rs"]
mod declaration_order;

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
