//! The `AsyncAPI` projection with an `ess-transport/1` document beside the model
//! (beyond10x/ess#390, beyond10x/ess#392).
//!
//! A bound channel's address is its subject and it names its broker and capturing stream; an
//! `array` envelope makes the payload an array of the event payload; the send operation carries
//! delivery and batch. Without a transport document the documents are byte-identical.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::asyncapi::{AsyncApi, TransportedAsyncApi};
use ess_transport::TransportSpec;
use serde_yaml::Value;
use sha2::{Digest as _, Sha256};

const MODEL: &str = "format: ess/20
system: metering
version: v1
domain: metering.items
events:
  - name: metering.items.UsageRecorded
    naming: {wire: usage}
    fields:
      - {name: id, type: String}
  - name: metering.items.Audited
    fields:
      - {name: id, type: String}
commands:
  - name: metering.items.RecordUsage
    input:
      - {name: id, type: String}
    outcomes:
      - name: recorded
        emits: [metering.items.UsageRecorded, metering.items.Audited]
        payload:
          metering.items.UsageRecorded: {id: input.id}
          metering.items.Audited: {id: input.id}
components:
  - component: producer
    owns: {domains: [metering.items]}
    accepts: {commands: [metering.items.RecordUsage]}
    publishes: {events: [metering.items.UsageRecorded, metering.items.Audited]}
";

const TRANSPORT: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: metering.items.UsageRecorded
    broker: events
    subject: usage
    envelope: array
    delivery: at_most_once
    batch: {max_items: 100, max_delay_ms: 5000}
streams:
  - name: USAGE
    broker: events
    subjects: [usage]
    storage: file
    retention: limits
    max_age_seconds: 7776000
    owner: external
";

const PARAMETER_MODEL: &str = "format: ess/20
system: routing
version: v1
domain: routing.events
types:
  - name: routing.events.Source
    kind: struct
    fields:
      - {name: service, wire: serviceName, type: String}
      - {name: environment, type: String}
events:
  - name: routing.events.UsageRecorded
    fields:
      - {name: id, type: String}
      - {name: source, wire: origin, type: routing.events.Source}
components:
  - component: producer
    owns: {domains: [routing.events]}
    publishes: {events: [routing.events.UsageRecorded]}
";

const PARAMETER_TRANSPORT: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: routing.events.UsageRecorded
    broker: events
    subject: 'usage.{service}.{again}.{environment}'
    parameters:
      service: event.source.service
      again: event.source.service
      environment: event.source.environment
    envelope: array
    delivery: at_most_once
    batch: {max_items: 100, max_delay_ms: 5000}
streams:
  - {name: USAGE, broker: events, subjects: ['usage.>'], storage: file, retention: limits, owner: external}
";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("metering.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("metering.yaml", MODEL);
    compile(&spec, &sources).expect("compiles")
}

fn transported(ir: &EssIr, body: &str) -> Value {
    let text = format!(
        "type: ess-transport/1\nspecification:\n  system: metering\n  version: v1\n  source_digest: sha256:{}\n{body}",
        ir.source_digest()
    );
    let spec = TransportSpec::from_yaml(&text).expect("parses");
    let transport = ess_transport::compile(&spec, ir).expect("compiles");
    let artifacts = run(&TransportedAsyncApi(transport), ir).expect("generates");
    serde_yaml::from_str(&artifacts["asyncapi/producer.yaml"].contents).expect("YAML")
}

#[test]
fn transport_1_asyncapi_keeps_the_released_0_52_0_bytes() {
    let ir = model();
    let text = format!(
        "type: ess-transport/1\nspecification:\n  system: metering\n  version: v1\n  source_digest: sha256:{}\n{TRANSPORT}",
        ir.source_digest()
    );
    let spec = TransportSpec::from_yaml(&text).expect("parses");
    let transport = ess_transport::compile(&spec, &ir).expect("compiles");
    let artifacts = run(&TransportedAsyncApi(transport), &ir).expect("generates");
    let digest = Sha256::digest(artifacts["asyncapi/producer.yaml"].contents.as_bytes())
        .iter()
        .fold(String::new(), |mut text, byte| {
            use std::fmt::Write as _;
            let _ = write!(text, "{byte:02x}");
            text
        });
    assert_eq!(
        digest, "e16157db3270fc266628440c35f7c0377d96f265c99890c0efc17775c1fa43f5",
        "released transported AsyncAPI bytes changed"
    );
}

fn parameterized_model() -> EssIr {
    let raw = RawSpecFile::parse(PARAMETER_MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("routing.yaml", PARAMETER_MODEL);
    compile(&spec, &sources).expect("compiles")
}

fn parameterized(ir: &EssIr) -> Value {
    parameterized_with(ir, PARAMETER_TRANSPORT)
}

fn parameterized_with(ir: &EssIr, body: &str) -> Value {
    let text = format!(
        "type: ess-transport/2\nspecification:\n  system: routing\n  version: v1\n  source_digest: sha256:{}\n{body}",
        ir.source_digest()
    );
    let spec = TransportSpec::from_yaml(&text).expect("parses");
    let transport = ess_transport::compile(&spec, ir).expect("compiles");
    let artifacts = run(&TransportedAsyncApi(transport), ir).expect("generates");
    serde_yaml::from_str(&artifacts["asyncapi/producer.yaml"].contents).expect("YAML")
}

#[test]
fn a_bound_channel_is_its_subject_on_its_broker_captured_by_its_stream() {
    let ir = model();
    let document = transported(&ir, TRANSPORT);
    let channel = &document["channels"]["metering.items.UsageRecorded"];
    assert_eq!(channel["address"], "usage");
    assert_eq!(channel["x-ess-address-source"], "transport");
    assert_eq!(channel["servers"][0]["$ref"], "#/servers/events");
    let stream = &channel["x-ess-stream"];
    assert_eq!(stream["name"], "USAGE");
    assert_eq!(stream["owner"], "external");
    assert_eq!(stream["max_age_seconds"], 7_776_000);
    let server = &document["servers"]["events"];
    assert_eq!(server["protocol"], "nats");
    assert_eq!(server["host"], "{host}");
    assert!(server["variables"]["host"]["description"].is_string());
    assert!(
        server["description"]
            .as_str()
            .is_some_and(|text| text.contains("JetStream") && text.contains("USAGE")),
        "{server:?}"
    );
}

#[test]
fn an_array_envelope_makes_the_payload_an_array_of_the_event() {
    let ir = model();
    let document = transported(&ir, TRANSPORT);
    let message = &document["components"]["messages"]["metering.items.UsageRecorded"];
    assert_eq!(message["x-ess-envelope"], "array");
    assert_eq!(message["payload"]["type"], "array");
    assert_eq!(
        message["payload"]["items"]["$ref"],
        "#/components/schemas/event.metering.items.UsageRecorded"
    );
    let send = &document["operations"]["send.metering.items.UsageRecorded"];
    assert_eq!(send["x-ess-delivery"], "at_most_once");
    assert_eq!(send["x-ess-batch"]["max_items"], 100);
    assert_eq!(send["x-ess-batch"]["max_delay_ms"], 5000);
}

#[test]
fn an_unbound_event_keeps_the_transport_free_projection() {
    let ir = model();
    let document = transported(&ir, TRANSPORT);
    let channel = &document["channels"]["metering.items.Audited"];
    assert_eq!(channel["address"], "metering.items.Audited");
    assert_eq!(channel["x-ess-address-source"], "qualified-name");
    assert!(channel.get("servers").is_none());
    let message = &document["components"]["messages"]["metering.items.Audited"];
    assert!(message["payload"].get("$ref").is_some());
    assert!(document["operations"]["send.metering.items.Audited"]
        .get("x-ess-delivery")
        .is_none());
}

#[test]
fn a_fixed_host_needs_no_variable() {
    let ir = model();
    let body = TRANSPORT.replace("jetstream: true}", "jetstream: true, host: 'nats:4222'}");
    let server = &transported(&ir, &body)["servers"]["events"];
    assert_eq!(server["host"], "nats:4222");
    assert!(server.get("variables").is_none());
}

#[test]
fn a_single_envelope_keeps_the_payload_reference() {
    let ir = model();
    let body = TRANSPORT
        .replace("envelope: array", "envelope: single")
        .replace("    batch: {max_items: 100, max_delay_ms: 5000}\n", "");
    let document = transported(&ir, &body);
    let message = &document["components"]["messages"]["metering.items.UsageRecorded"];
    assert!(message["payload"].get("$ref").is_some(), "{message:?}");
    assert!(message.get("x-ess-envelope").is_none());
}

#[test]
fn without_a_transport_the_document_says_it_has_none() {
    let ir = model();
    let artifacts = run(&AsyncApi, &ir).expect("generates");
    let text = &artifacts["asyncapi/producer.yaml"].contents;
    assert!(!text.contains("servers:"), "{text}");
    assert!(text.contains("declares no transport"), "{text}");
}

#[test]
fn address_parameters_use_wire_locations_and_closed_semantic_metadata() {
    let ir = parameterized_model();
    let document = parameterized(&ir);
    let channel = &document["channels"]["routing.events.UsageRecorded"];
    assert_eq!(channel["address"], "usage.{service}.{again}.{environment}");
    let service = &channel["parameters"]["service"];
    assert_eq!(
        service["location"],
        "$message.payload#/0/origin/serviceName"
    );
    assert_eq!(service["x-ess-source"]["kind"], "event_path");
    assert_eq!(service["x-ess-source"]["path"][0], "source");
    assert_eq!(service["x-ess-source"]["path"][1], "service");
    assert_eq!(service["x-ess-source"]["scope"], "every_item");
    assert_eq!(service["x-ess-source"]["constraint"], "nats_subject_token");
    assert_eq!(
        channel["parameters"]["environment"]["location"],
        "$message.payload#/0/origin/environment"
    );
    assert_eq!(
        channel["parameters"]["again"]["x-ess-source"]["path"],
        service["x-ess-source"]["path"]
    );
    assert_eq!(
        channel["parameters"]["again"]["location"],
        service["location"]
    );
}

#[test]
fn a_single_envelope_uses_one_item_parameter_scope_and_location() {
    let ir = parameterized_model();
    let body = PARAMETER_TRANSPORT
        .replace("envelope: array", "envelope: single")
        .replace("    batch: {max_items: 100, max_delay_ms: 5000}\n", "");
    let document = parameterized_with(&ir, &body);
    let service = &document["channels"]["routing.events.UsageRecorded"]["parameters"]["service"];
    assert_eq!(service["location"], "$message.payload#/origin/serviceName");
    assert_eq!(service["x-ess-source"]["scope"], "one_item");
}

#[test]
fn parameter_locations_escape_every_wire_json_pointer_segment() {
    let text = PARAMETER_MODEL
        .replace("wire: origin", "wire: 'root~/part'")
        .replace("wire: serviceName", "wire: 'service~/id'");
    let raw = RawSpecFile::parse(&text).expect("well formed");
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("routing.yaml", text);
    let ir = compile(&spec, &sources).expect("compiles");
    let document = parameterized(&ir);
    assert_eq!(
        document["channels"]["routing.events.UsageRecorded"]["parameters"]["service"]["location"],
        "$message.payload#/0/root~0~1part/service~0~1id"
    );
}

/// A directory under `TMPDIR` removed when dropped, so a refused document leaves nothing behind.
struct Scratch(std::path::PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "requires the official AsyncAPI CLI; CI runs this test explicitly"]
fn the_official_asyncapi_cli_accepts_the_parameterized_document() {
    let ir = parameterized_model();
    let text = format!(
        "type: ess-transport/2\nspecification:\n  system: routing\n  version: v1\n  source_digest: sha256:{}\n{PARAMETER_TRANSPORT}",
        ir.source_digest()
    );
    let spec = TransportSpec::from_yaml(&text).expect("parses");
    let transport = ess_transport::compile(&spec, &ir).expect("compiles");
    let artifacts = run(&TransportedAsyncApi(transport), &ir).expect("generates");
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "ess-asyncapi-parameter-validation-{}",
        std::process::id()
    )));
    let root = &scratch.0;
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root).unwrap();
    let document = root.join("producer.yaml");
    std::fs::write(&document, &artifacts["asyncapi/producer.yaml"].contents).unwrap();
    let cli = std::env::var_os("ESS_ASYNCAPI_CLI")
        .expect("ESS_ASYNCAPI_CLI must name the pinned official AsyncAPI CLI executable");
    let output = std::process::Command::new(cli)
        .arg("validate")
        .arg(&document)
        .output()
        .expect("the official AsyncAPI CLI runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
