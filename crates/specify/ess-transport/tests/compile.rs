//! `ess-transport/1` compiles against the specification it names, or says every reason it does not.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_transport::{compile, Envelope, Owner, TransportSpec};

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
";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("metering.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("metering.yaml", MODEL);
    compile_model(&spec, &sources).expect("compiles")
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-transport/1\nspecification:\n  system: metering\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

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
  - name: USAGE
    broker: events
    subjects: [usage]
    storage: file
    retention: limits
    max_age_seconds: 7776000
    owner: external
";

fn codes(ir: &EssIr, body: &str) -> Vec<&'static str> {
    let spec = TransportSpec::from_yaml(&(header(ir) + body)).expect("parses");
    match compile(&spec, ir) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics.0.iter().map(|d| d.code).collect(),
    }
}

#[test]
fn a_bound_event_resolves_to_its_subject_and_capturing_stream() {
    let ir = model();
    let spec = TransportSpec::from_yaml(&(header(&ir) + BODY)).expect("parses");
    let transport = compile(&spec, &ir).expect("compiles");
    let channel = transport
        .channel("metering.items.UsageRecorded")
        .expect("bound");
    assert_eq!(channel.subject, "usage");
    assert_eq!(channel.envelope, Envelope::Array);
    assert_eq!(channel.stream.as_deref(), Some("USAGE"));
    assert_eq!(transport.streams()["USAGE"].owner, Owner::External);
    let json = transport.to_canonical_json();
    assert!(
        json.contains("\"format\": \"ess-transport-ir/1\""),
        "{json}"
    );
    assert_eq!(
        json,
        compile(&spec, &ir).expect("again").to_canonical_json()
    );
}

#[test]
fn a_wildcard_stream_captures_a_subject_under_it() {
    let ir = model();
    let body = BODY.replace("subjects: [usage]", "subjects: ['>']");
    assert_eq!(codes(&ir, &body), Vec::<&str>::new());
}

#[test]
fn a_mismatched_specification_is_refused() {
    let ir = model();
    let text = header(&ir).replace("version: v1", "version: v2") + BODY;
    let spec = TransportSpec::from_yaml(&text).expect("parses");
    let diagnostics = compile(&spec, &ir).expect_err("refused");
    assert_eq!(diagnostics.0[0].code, "ESS-TRANSPORT-002");
    assert_eq!(diagnostics.0[0].path, "specification.version");
}

#[test]
fn every_refusal_has_its_code() {
    let ir = model();
    for (change, from, to, code) in [
        ("wrong type", "", "", "ESS-TRANSPORT-001"),
        (
            "bad broker id",
            "id: events,",
            "id: Events,",
            "ESS-TRANSPORT-003",
        ),
        (
            "bad stream name",
            "name: USAGE",
            "name: US.AGE",
            "ESS-TRANSPORT-005",
        ),
        (
            "unknown stream broker",
            "    broker: events\n    subjects",
            "    broker: nowhere\n    subjects",
            "ESS-TRANSPORT-006",
        ),
        (
            "broker without jetstream",
            "jetstream: true",
            "jetstream: false",
            "ESS-TRANSPORT-007",
        ),
        (
            "bad stream subject",
            "subjects: [usage]",
            "subjects: ['usage.>.x']",
            "ESS-TRANSPORT-008",
        ),
        (
            "unknown event",
            "event: metering.items.UsageRecorded",
            "event: metering.items.Missing",
            "ESS-TRANSPORT-010",
        ),
        (
            "subject differs from wire",
            "subject: usage",
            "subject: other",
            "ESS-TRANSPORT-011",
        ),
        (
            "batch on single",
            "envelope: array",
            "envelope: single",
            "ESS-TRANSPORT-013",
        ),
        (
            "zero batch",
            "max_items: 100",
            "max_items: 0",
            "ESS-TRANSPORT-013",
        ),
        (
            "uncaptured subject",
            "subjects: [usage]",
            "subjects: [other]",
            "ESS-TRANSPORT-015",
        ),
    ] {
        let found = if change == "wrong type" {
            let text = header(&ir).replace("ess-transport/1", "ess-transport/2") + BODY;
            let spec = TransportSpec::from_yaml(&text).expect("parses");
            compile(&spec, &ir)
                .expect_err("refused")
                .0
                .iter()
                .map(|d| d.code)
                .collect()
        } else {
            assert!(BODY.contains(from), "{change}: `{from}` not in the body");
            codes(&ir, &BODY.replace(from, to))
        };
        assert!(
            found.contains(&code),
            "{change}: expected {code}, found {found:?}"
        );
    }
}

#[test]
fn a_publish_subject_with_a_wildcard_is_refused() {
    let ir = model();
    let body = BODY
        .replace(
            "event: metering.items.UsageRecorded",
            "event: metering.items.Audited",
        )
        .replace("subject: usage", "subject: 'audit.*'");
    assert!(codes(&ir, &body).contains(&"ESS-TRANSPORT-012"));
}

#[test]
fn duplicates_and_overlaps_are_refused() {
    let ir = model();
    let duplicate_broker = BODY.replace(
        "  - {id: events, protocol: nats, jetstream: true}\n",
        "  - {id: events, protocol: nats, jetstream: true}\n  - {id: events, protocol: nats}\n",
    );
    assert!(codes(&ir, &duplicate_broker).contains(&"ESS-TRANSPORT-004"));

    let second_stream = format!(
        "{BODY}  - name: ALL\n    broker: events\n    subjects: ['>']\n    storage: file\n    retention: limits\n    owner: external\n"
    );
    assert!(codes(&ir, &second_stream).contains(&"ESS-TRANSPORT-016"));

    let duplicate_stream = second_stream.replace("name: ALL", "name: USAGE");
    assert!(codes(&ir, &duplicate_stream).contains(&"ESS-TRANSPORT-009"));

    let second_channel = BODY.replace(
        "streams:",
        "  - event: metering.items.UsageRecorded\n    broker: events\n    subject: usage\n    envelope: single\n    delivery: at_least_once\nstreams:",
    );
    assert!(codes(&ir, &second_channel).contains(&"ESS-TRANSPORT-014"));
}

#[test]
fn an_unknown_key_is_refused_while_parsing() {
    let ir = model();
    let text = header(&ir) + &BODY.replace("owner: external", "owner: external\n    replicas: 3");
    assert!(TransportSpec::from_yaml(&text).is_err());
}
