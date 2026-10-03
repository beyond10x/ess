//! `ess generate client` (beyond10x/ess#395).

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const MODEL: &str = "format: ess/20
system: metering
version: v1
domain: metering.items
events:
  - name: metering.items.UsageRecorded
    naming: {wire: usage}
    fields:
      - {name: id, type: String}
commands:
  - name: metering.items.RecordUsage
    input:
      - {name: id, type: String}
    outcomes:
      - name: recorded
        emits: [metering.items.UsageRecorded]
        payload:
          metering.items.UsageRecorded: {id: input.id}
components:
  - component: producer
    owns: {domains: [metering.items]}
    accepts: {commands: [metering.items.RecordUsage]}
    publishes: {events: [metering.items.UsageRecorded]}
";

const BODY: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: metering.items.UsageRecorded
    broker: events
    subject: usage
    envelope: array
    delivery: at_most_once
    batch: {max_items: 100, max_delay_ms: 5000}
streams:
  - {name: USAGE, broker: events, subjects: [usage], storage: file, retention: limits, owner: external}
";

struct Fixture(PathBuf);

impl Fixture {
    fn new(body: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ess-client-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(path.join("model/system.yaml"), MODEL).unwrap();
        let fixture = Self(path);
        let generated = fixture.ess(&[
            "generate", "--path", "model", "--kind", "schema", "--out", "digest",
        ]);
        assert!(generated.status.success(), "{generated:?}");
        let schema: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(
                fixture
                    .0
                    .join("digest/schema/events/metering.items.UsageRecorded.schema.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let digest = schema["x-ess-provenance"]["source_digest"]
            .as_str()
            .unwrap();
        fs::write(
            fixture.0.join("transport.yaml"),
            format!(
                "type: ess-transport/1\nspecification:\n  system: metering\n  version: v1\n  source_digest: sha256:{digest}\n{body}"
            ),
        )
        .unwrap();
        fixture
    }

    fn ess(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }

    fn client(&self, extra: &[&str]) -> Output {
        let mut args = vec![
            "generate",
            "client",
            "--path",
            "model",
            "--component",
            "producer",
            "--transport",
            "transport.yaml",
        ];
        args.extend_from_slice(extra);
        self.ess(&args)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn short_names_reach_the_generate_client_cli_and_its_publisher() {
    let fixture = Fixture::new(BODY);
    let output = fixture.client(&[
        "--target",
        "rust",
        "--package",
        "metering-client",
        "--names",
        "short",
        "--out",
        "short",
    ]);
    assert!(output.status.success(), "{output:?}");
    let types = fs::read_to_string(fixture.0.join("short/types.rs")).unwrap();
    let publisher = fs::read_to_string(fixture.0.join("short/lib.rs")).unwrap();
    assert!(types.contains("pub struct UsageRecorded"), "{types}");
    assert!(!types.contains("MeteringItemsUsageRecorded"), "{types}");
    assert!(publisher.contains("UsageRecorded"), "{publisher}");
    assert!(
        !publisher.contains("MeteringItemsUsageRecorded"),
        "{publisher}"
    );
}

#[test]
fn a_rust_and_a_go_publisher_are_written_with_their_adapters_and_report() {
    let fixture = Fixture::new(BODY);
    let rust = fixture.client(&[
        "--target",
        "rust",
        "--package",
        "metering-client",
        "--out",
        "rust",
    ]);
    assert!(rust.status.success(), "{rust:?}");
    for file in [
        "Cargo.toml",
        "lib.rs",
        "types.rs",
        "nats/Cargo.toml",
        "nats/lib.rs",
        "client-report.json",
        "types-report.json",
    ] {
        assert!(fixture.0.join("rust").join(file).is_file(), "rust/{file}");
    }
    let lib = fs::read_to_string(fixture.0.join("rust/lib.rs")).unwrap();
    assert!(lib.contains("pub fn publish_usage_recorded("), "{lib}");
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(fixture.0.join("rust/client-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(report["format"], "ess-client-report/1");
    assert_eq!(report["operations"][0]["subject"], "usage");

    let go = fixture.client(&[
        "--target",
        "go",
        "--package",
        "meteringclient",
        "--module",
        "example.invalid/meteringclient",
        "--out",
        "go",
    ]);
    assert!(go.status.success(), "{go:?}");
    for file in [
        "go.mod",
        "types.go",
        "publisher.go",
        "natsjs/go.mod",
        "natsjs/natsjs.go",
        "client-report.json",
    ] {
        assert!(fixture.0.join("go").join(file).is_file(), "go/{file}");
    }
}

#[test]
fn at_least_once_and_an_unknown_component_are_refused() {
    let fixture = Fixture::new(&BODY.replace("at_most_once", "at_least_once"));
    let refused = fixture.client(&[
        "--target",
        "rust",
        "--package",
        "metering-client",
        "--out",
        "rust",
    ]);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("unsupported_delivery"));
    assert!(!fixture.0.join("rust").exists());

    let fixture = Fixture::new(BODY);
    let unknown = fixture.ess(&[
        "generate",
        "client",
        "--path",
        "model",
        "--component",
        "nobody",
        "--transport",
        "transport.yaml",
        "--target",
        "rust",
        "--package",
        "x",
        "--out",
        "rust",
    ]);
    assert_eq!(unknown.status.code(), Some(1), "{unknown:?}");
}
