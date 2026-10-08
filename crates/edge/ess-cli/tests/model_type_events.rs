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
