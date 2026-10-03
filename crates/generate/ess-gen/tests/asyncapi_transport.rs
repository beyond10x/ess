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
