//! `ess specify transport` and `ess generate --kind asyncapi --transport` (beyond10x/ess#390,
//! beyond10x/ess#392).

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
streams:
  - {name: USAGE, broker: events, subjects: [usage], storage: file, retention: limits, owner: external}
";

const PARAMETER_MODEL: &str = "format: ess/20
system: routing
version: v1
domain: routing.events
types:
  - name: routing.events.Source
    kind: struct
    fields:
      - {name: service, type: String}
events:
  - name: routing.events.UsageRecorded
    fields:
      - {name: source, type: routing.events.Source}
components:
  - component: producer
    owns: {domains: [routing.events]}
    publishes: {events: [routing.events.UsageRecorded]}
";

const PARAMETER_BODY: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: routing.events.UsageRecorded
    broker: events
    subject: 'usage.{service}'
    parameters: {service: event.source.service}
    envelope: array
    delivery: at_most_once
streams:
  - {name: USAGE, broker: events, subjects: ['usage.>'], storage: file, retention: limits, owner: external}
";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ess-transport-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(path.join("model/system.yaml"), MODEL).unwrap();
        Self(path)
    }

    fn ess(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }

    /// Writes `transport.yaml` with the model real digest, read from a generated schema provenance.
    fn transport(&self, body: &str) {
        let generated = self.ess(&[
            "generate", "--path", "model", "--kind", "schema", "--out", "digest",
        ]);
        assert!(generated.status.success(), "{generated:?}");
        let schema = fs::read_to_string(
            self.0
                .join("digest/schema/events/metering.items.UsageRecorded.schema.json"),
        )
        .unwrap();
        let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();
        let digest = schema["x-ess-provenance"]["source_digest"]
            .as_str()
            .unwrap_or_else(|| panic!("no source_digest in {schema}"))
            .to_owned();
        fs::write(
            self.0.join("transport.yaml"),
            format!(
                "type: ess-transport/1\nspecification:\n  system: metering\n  version: v1\n  source_digest: sha256:{digest}\n{body}"
            ),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parameter_fixture() -> Fixture {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("model/system.yaml"), PARAMETER_MODEL).unwrap();
    let generated = fixture.ess(&[
        "generate", "--path", "model", "--kind", "schema", "--out", "digest",
    ]);
    assert!(generated.status.success(), "{}", text(&generated));
    let schema: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            fixture
                .0
                .join("digest/schema/events/routing.events.UsageRecorded.schema.json"),
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
            "type: ess-transport/2\nspecification:\n  system: routing\n  version: v1\n  source_digest: sha256:{digest}\n{PARAMETER_BODY}"
        ),
    )
    .unwrap();
    fixture
}

#[test]
fn validate_and_compile_a_transport_document() {
    let fixture = Fixture::new();
    fixture.transport(BODY);
    let validate = fixture.ess(&[
        "specify",
        "transport",
        "validate",
        "--path",
        "transport.yaml",
        "--spec",
        "model",
    ]);
    assert!(validate.status.success(), "{}", text(&validate));
    assert!(
        text(&validate).contains("1 channel(s), 1 stream(s), valid"),
        "{}",
        text(&validate)
    );

    let compile = fixture.ess(&[
        "specify",
        "transport",
        "compile",
        "--path",
        "transport.yaml",
        "--spec",
        "model",
        "--out",
        "transport.json",
    ]);
    assert!(compile.status.success(), "{}", text(&compile));
    let ir = fs::read_to_string(fixture.0.join("transport.json")).unwrap();
    assert!(ir.contains("\"format\": \"ess-transport-ir/1\""), "{ir}");
}

#[test]
fn a_refused_document_exits_1_with_its_code() {
    let fixture = Fixture::new();
    fixture.transport(&BODY.replace("subjects: [usage]", "subjects: [other]"));
    let validate = fixture.ess(&[
        "specify",
        "transport",
        "validate",
        "--path",
        "transport.yaml",
        "--spec",
        "model",
    ]);
    assert_eq!(validate.status.code(), Some(1), "{}", text(&validate));
    assert!(
        text(&validate).contains("ESS-TRANSPORT-015"),
        "{}",
        text(&validate)
    );
}

#[test]
fn asyncapi_with_a_transport_carries_servers_and_the_array_payload() {
    let fixture = Fixture::new();
    fixture.transport(BODY);
    let generate = fixture.ess(&[
        "generate",
        "--path",
        "model",
        "--kind",
        "asyncapi",
        "--transport",
        "transport.yaml",
        "--out",
        "out",
    ]);
    assert!(generate.status.success(), "{}", text(&generate));
    let document = fs::read_to_string(fixture.0.join("out/asyncapi/producer.yaml")).unwrap();
    assert!(document.contains("servers:"), "{document}");
    assert!(
        document.contains("x-ess-address-source: transport"),
        "{document}"
    );
    assert!(document.contains("x-ess-envelope: array"), "{document}");
}

#[test]
fn a_transport_beside_another_kind_is_refused() {
    let fixture = Fixture::new();
    fixture.transport(BODY);
    let generate = fixture.ess(&[
        "generate",
        "--path",
        "model",
        "--kind",
        "schema",
        "--transport",
        "transport.yaml",
        "--out",
        "out",
    ]);
    assert!(!generate.status.success(), "{}", text(&generate));
    assert!(
        text(&generate).contains("requires --kind asyncapi"),
        "{}",
        text(&generate)
    );
    assert!(!fixture.0.join("out").exists());
}

#[test]
fn parameterized_transport_compiles_and_reaches_asyncapi() {
    let fixture = parameter_fixture();
    let compile = fixture.ess(&[
        "specify",
        "transport",
        "compile",
        "--path",
        "transport.yaml",
        "--spec",
        "model",
        "--out",
        "transport.json",
    ]);
    assert!(compile.status.success(), "{}", text(&compile));
    let ir = fs::read_to_string(fixture.0.join("transport.json")).unwrap();
    assert!(ir.contains("\"format\": \"ess-transport-ir/2\""), "{ir}");

    let generate = fixture.ess(&[
        "generate",
        "--path",
        "model",
        "--kind",
        "asyncapi",
        "--transport",
        "transport.yaml",
        "--out",
        "out",
    ]);
    assert!(generate.status.success(), "{}", text(&generate));
    let document = fs::read_to_string(fixture.0.join("out/asyncapi/producer.yaml")).unwrap();
    assert!(document.contains("parameters:"), "{document}");
    assert!(
        document.contains("location: $message.payload#/0/source/service"),
        "{document}"
    );
}
