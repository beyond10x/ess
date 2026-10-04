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
    let expected = format!(
        "{{\n  \"format\": \"ess-transport-ir/1\",\n  \"specification\": {{\n    \"system\": \"metering\",\n    \"version\": \"v1\",\n    \"source_digest\": \"sha256:{}\"\n  }},\n  \"brokers\": {{\n    \"events\": {{\n      \"protocol\": \"nats\",\n      \"jetstream\": true\n    }}\n  }},\n  \"channels\": {{\n    \"metering.items.UsageRecorded\": {{\n      \"broker\": \"events\",\n      \"subject\": \"usage\",\n      \"envelope\": \"array\",\n      \"delivery\": \"at_most_once\",\n      \"batch\": {{\n        \"max_items\": 100,\n        \"max_delay_ms\": 5000\n      }},\n      \"stream\": \"USAGE\"\n    }}\n  }},\n  \"streams\": {{\n    \"USAGE\": {{\n      \"broker\": \"events\",\n      \"subjects\": [\n        \"usage\"\n      ],\n      \"storage\": \"file\",\n      \"retention\": \"limits\",\n      \"max_age_seconds\": 7776000,\n      \"owner\": \"external\"\n    }}\n  }}\n}}\n",
        ir.source_digest()
    );
    assert_eq!(json, expected, "released transport IR bytes changed");
    assert_eq!(
        json,
        compile(&spec, &ir).expect("again").to_canonical_json()
    );

    let changed = text.replace(
        "environment: event.source.environment",
        "environment: event.id",
    );
    let changed = TransportSpec::from_yaml(&changed).expect("changed source parses");
    let changed = compile(&changed, &ir)
        .expect("changed source compiles")
        .to_canonical_json();
    assert_ne!(json, changed, "a source-path mutation must change IR2 bytes");
    assert!(
        changed.contains("\"path\": [\n            \"id\"\n          ]"),
        "{changed}"
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
            let text = header(&ir).replace("ess-transport/1", "ess-transport/3") + BODY;
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

const PARAMETER_MODEL: &str = "format: ess/20
system: routing
version: v1
domain: routing.events
types:
  - name: routing.events.Alias
    kind: newtype
    of: String
  - name: routing.events.Label
    kind: enum
    variants: [Primary, Secondary]
  - name: routing.events.Region
    kind: struct
    fields:
      - {name: code, type: String}
  - name: routing.events.Choice
    kind: union
    tag: kind
    variants:
      alias: routing.events.Alias
      region: routing.events.Region
  - name: routing.events.Source
    kind: struct
    fields:
      - {name: service, type: String}
      - {name: environment, type: String}
      - {name: region, type: routing.events.Region}
      - {name: optional_region, type: 'Optional<routing.events.Region>'}
      - {name: count, type: Integer}
      - {name: optional_label, type: 'Optional<String>'}
events:
  - name: routing.events.UsageRecorded
    fields:
      - {name: id, type: String}
      - {name: source, type: routing.events.Source}
  - name: routing.events.NamedUsageRecorded
    naming: {wire: usage}
    fields:
      - {name: id, type: String}
      - {name: source, type: routing.events.Source}
  - name: routing.events.TypeShapes
    fields:
      - {name: list, type: 'List<String>'}
      - {name: map, type: 'Map<String, String>'}
      - {name: label, type: routing.events.Label}
      - {name: alias, type: routing.events.Alias}
      - {name: choice, type: routing.events.Choice}
      - {name: nested_list, type: 'List<routing.events.Region>'}
";

fn parameter_model() -> EssIr {
    let raw = RawSpecFile::parse(PARAMETER_MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("routing.yaml", PARAMETER_MODEL);
    compile_model(&spec, &sources).expect("compiles")
}

fn parameter_header(ir: &EssIr, format: &str) -> String {
    format!(
        "type: {format}\nspecification:\n  system: routing\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

fn parameter_transport(
    ir: &EssIr,
    event: &str,
    subject: &str,
    parameters: &str,
    stream_subjects: &str,
) -> String {
    format!(
        "{}brokers:\n  - {{id: events, protocol: nats, jetstream: true}}\nchannels:\n  - event: {event}\n    broker: events\n    subject: '{subject}'\n{parameters}    envelope: array\n    delivery: at_most_once\n    batch: {{max_items: 100, max_delay_ms: 5000}}\nstreams:\n  - name: USAGE\n    broker: events\n    subjects: {stream_subjects}\n    storage: file\n    retention: limits\n    owner: external\n",
        parameter_header(ir, "ess-transport/2")
    )
}

fn parameter_diagnostics(ir: &EssIr, text: &str) -> Vec<(&'static str, String)> {
    let spec = TransportSpec::from_yaml(text).expect("parameterized transport parses");
    compile(&spec, ir)
        .expect_err("parameterized transport is refused")
        .0
        .into_iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.path))
        .collect()
}

fn with_other_stream(mut text: String, subject: &str) -> String {
    text.push_str(&format!(
        "  - name: OTHER\n    broker: events\n    subjects: ['{subject}']\n    storage: file\n    retention: limits\n    owner: external\n"
    ));
    text
}

#[test]
fn transport_v2_compiles_payload_parameters_to_typed_deterministic_ir() {
    let ir = parameter_model();
    let text = parameter_transport(
        &ir,
        "routing.events.UsageRecorded",
        "usage.{service}.{environment}",
        "    parameters:\n      service: event.source.service\n      environment: event.source.environment\n",
        "['usage.>']",
    );
    let spec = TransportSpec::from_yaml(&text).expect("parameterized transport parses");
    let compiled = compile(&spec, &ir).expect("parameterized transport compiles");
    let json = compiled.to_canonical_json();

    assert!(json.contains("\"format\": \"ess-transport-ir/2\""), "{json}");
    assert!(
        json.contains("\"subject\": \"usage.{service}.{environment}\""),
        "{json}"
    );
    assert!(json.contains("\"kind\": \"event_path\""), "{json}");
    assert!(json.contains("\"source\",\n            \"service\""), "{json}");
    assert!(
        json.contains("\"source\",\n            \"environment\""),
        "{json}"
    );
    assert_eq!(
        json,
        compile(&spec, &ir).expect("again").to_canonical_json()
    );
}

#[test]
fn every_admitted_event_path_depth_and_literal_v2_compile() {
    let ir = parameter_model();
    for (subject, mapping) in [
        ("usage.{value}", "event.id"),
        ("usage.{value}", "event.source.service"),
        ("usage.{value}", "event.source.region.code"),
    ] {
        let text = parameter_transport(
            &ir,
            "routing.events.UsageRecorded",
            subject,
            &format!("    parameters:\n      value: {mapping}\n"),
            "['usage.>']",
        );
        let spec = TransportSpec::from_yaml(&text).expect("parameterized transport parses");
        compile(&spec, &ir).unwrap_or_else(|error| panic!("{mapping}: {error}"));
    }

    let literal = parameter_transport(
        &ir,
        "routing.events.NamedUsageRecorded",
        "usage",
        "",
        "['usage']",
    );
    let spec = TransportSpec::from_yaml(&literal).expect("literal v2 parses");
    let compiled = compile(&spec, &ir).expect("literal v2 compiles");
    assert!(compiled.channels()["routing.events.NamedUsageRecorded"]
        .parameters
        .is_empty());
    assert!(compiled
        .to_canonical_json()
        .contains("\"format\": \"ess-transport-ir/2\""));

    let null_parameters = literal.replace("    subject: usage\n", "    subject: usage\n    parameters: null\n");
    assert!(TransportSpec::from_yaml(&null_parameters).is_err());
    let null_json = serde_json::to_string(
        &serde_yaml::from_str::<serde_yaml::Value>(&null_parameters).unwrap(),
    )
    .unwrap();
    assert!(TransportSpec::from_json(&null_json).is_err());
}

#[test]
fn stream_intersection_unifies_repeated_and_source_aliased_parameters() {
    let ir = parameter_model();
    for (case, subject, parameters) in [
        (
            "one expression repeated",
            "usage.{same}.{same}",
            "    parameters:\n      same: event.source.service\n",
        ),
        (
            "two names alias one source",
            "usage.{left}.{right}",
            "    parameters:\n      left: event.source.service\n      right: event.source.service\n",
        ),
    ] {
        let text = with_other_stream(parameter_transport(
            &ir,
            "routing.events.UsageRecorded",
            subject,
            parameters,
            "['usage.*.*']",
        ), "usage.a.b");
        let spec = TransportSpec::from_yaml(&text).expect("parameterized transport parses");
        let compiled = compile(&spec, &ir).unwrap_or_else(|error| panic!("{case}: {error}"));
        assert_eq!(
            compiled.channels()["routing.events.UsageRecorded"]
                .stream
                .as_deref(),
            Some("USAGE"),
            "{case}"
        );
    }

    let independent = with_other_stream(parameter_transport(
        &ir,
        "routing.events.UsageRecorded",
        "usage.{left}.{right}",
        "    parameters:\n      left: event.source.service\n      right: event.source.environment\n",
        "['usage.*.*']",
    ), "usage.a.b");
    assert!(
        parameter_diagnostics(&ir, &independent)
            .iter()
            .any(|(code, path)| *code == "ESS-TRANSPORT-016" && path == "channels[0].subject")
    );
}

#[test]
fn a_cover_is_universal_and_every_other_stream_must_be_disjoint() {
    let ir = parameter_model();
    let parameters = "    parameters:\n      same: event.source.service\n";

    let exact_only = parameter_transport(
        &ir,
        "routing.events.UsageRecorded",
        "usage.{same}.{same}",
        parameters,
        "['usage.a.a']",
    );
    assert!(
        parameter_diagnostics(&ir, &exact_only)
            .iter()
            .any(|(code, _)| *code == "ESS-TRANSPORT-015")
    );

    let overlapping = with_other_stream(parameter_transport(
        &ir,
        "routing.events.UsageRecorded",
        "usage.{same}.{same}",
        parameters,
        "['usage.*.*']",
    ), "usage.a.a");
    assert!(
        parameter_diagnostics(&ir, &overlapping)
            .iter()
            .any(|(code, _)| *code == "ESS-TRANSPORT-016")
    );

    let two_covers = with_other_stream(
        parameter_transport(
            &ir,
            "routing.events.UsageRecorded",
            "usage.{same}.{same}",
            parameters,
            "['usage.*.*']",
        ),
        "usage.>",
    );
    assert!(
        parameter_diagnostics(&ir, &two_covers)
            .iter()
            .any(|(code, _)| *code == "ESS-TRANSPORT-016")
    );
}

#[test]
fn transport_v2_reports_the_closed_template_and_event_path_diagnostics() {
    let ir = parameter_model();
    for (case, subject, parameters, expected_code, expected_path) in [
        (
            "embedded expression",
            "usage.prefix-{service}",
            "    parameters:\n      service: event.source.service\n",
            "ESS-TRANSPORT-017",
            "channels[0].subject",
        ),
        (
            "unmatched expression",
            "usage.{service",
            "    parameters:\n      service: event.source.service\n",
            "ESS-TRANSPORT-017",
            "channels[0].subject",
        ),
        (
            "invalid expression name",
            "usage.{service!}",
            "    parameters:\n      service!: event.source.service\n",
            "ESS-TRANSPORT-017",
            "channels[0].subject",
        ),
        (
            "wildcard is not a static publish token",
            "usage.*",
            "",
            "ESS-TRANSPORT-012",
            "channels[0].subject",
        ),
        (
            "missing mapping",
            "usage.{service}",
            "",
            "ESS-TRANSPORT-018",
            "channels[0].parameters.service",
        ),
        (
            "empty mapping",
            "usage.{service}",
            "    parameters: {}\n",
            "ESS-TRANSPORT-018",
            "channels[0].parameters",
        ),
        (
            "literal subject with empty mapping",
            "usage",
            "    parameters: {}\n",
            "ESS-TRANSPORT-018",
            "channels[0].parameters",
        ),
        (
            "unused mapping",
            "usage.{service}",
            "    parameters:\n      service: event.source.service\n      other: event.source.environment\n",
            "ESS-TRANSPORT-018",
            "channels[0].parameters.other",
        ),
        (
            "producer context is outside the payload-only contract",
            "usage.{service}",
            "    parameters:\n      service: context.service\n",
            "ESS-TRANSPORT-019",
            "channels[0].parameters.service",
        ),
        (
            "unknown event path",
            "usage.{service}",
            "    parameters:\n      service: event.source.missing\n",
            "ESS-TRANSPORT-019",
            "channels[0].parameters.service",
        ),
        (
            "path is too deep",
            "usage.{service}",
            "    parameters:\n      service: event.source.service.more.too_deep\n",
            "ESS-TRANSPORT-019",
            "channels[0].parameters.service",
        ),
        (
            "non-string terminal",
            "usage.{count}",
            "    parameters:\n      count: event.source.count\n",
            "ESS-TRANSPORT-020",
            "channels[0].parameters.count",
        ),
        (
            "optional terminal",
            "usage.{label}",
            "    parameters:\n      label: event.source.optional_label\n",
            "ESS-TRANSPORT-020",
            "channels[0].parameters.label",
        ),
        (
            "non-struct intermediate",
            "usage.{nested}",
            "    parameters:\n      nested: event.source.count.value\n",
            "ESS-TRANSPORT-020",
            "channels[0].parameters.nested",
        ),
        (
            "optional struct intermediate",
            "usage.{nested}",
            "    parameters:\n      nested: event.source.optional_region.code\n",
            "ESS-TRANSPORT-020",
            "channels[0].parameters.nested",
        ),
    ] {
        let text = parameter_transport(
            &ir,
            "routing.events.UsageRecorded",
            subject,
            parameters,
            "['usage.>']",
        );
        let diagnostics = parameter_diagnostics(&ir, &text);
        assert!(
            diagnostics
                .iter()
                .any(|(code, path)| *code == expected_code && path == expected_path),
            "{case}: expected {expected_code} at {expected_path}, found {diagnostics:?}"
        );
    }

    for (case, mapping) in [
        ("list terminal", "event.list"),
        ("map terminal", "event.map"),
        ("enum terminal", "event.label"),
        ("newtype terminal", "event.alias"),
        ("union terminal", "event.choice"),
        ("collection intermediate", "event.nested_list.code"),
    ] {
        let text = parameter_transport(
            &ir,
            "routing.events.TypeShapes",
            "usage.{value}",
            &format!("    parameters:\n      value: {mapping}\n"),
            "['usage.>']",
        );
        let diagnostics = parameter_diagnostics(&ir, &text);
        assert!(
            diagnostics
                .iter()
                .any(|(code, path)| *code == "ESS-TRANSPORT-020"
                    && path == "channels[0].parameters.value"),
            "{case}: expected ESS-TRANSPORT-020, found {diagnostics:?}"
        );
    }
}

#[test]
fn parameterized_subjects_cannot_reinterpret_a_literal_event_wire_name() {
    let ir = parameter_model();
    let text = parameter_transport(
        &ir,
        "routing.events.NamedUsageRecorded",
        "usage.{service}",
        "    parameters:\n      service: event.source.service\n",
        "['usage.>']",
    );
    assert!(
        parameter_diagnostics(&ir, &text)
            .iter()
            .any(|(code, path)| *code == "ESS-TRANSPORT-011" && path == "channels[0].subject")
    );
}

#[test]
fn transport_v1_rejects_parameters_and_keeps_braces_literal() {
    let ir = parameter_model();
    let v2 = parameter_transport(
        &ir,
        "routing.events.UsageRecorded",
        "usage.{service}.{environment}",
        "    parameters:\n      service: event.source.service\n      environment: event.source.environment\n",
        "['usage.>']",
    );
    let v1_with_parameters = v2.replacen("ess-transport/2", "ess-transport/1", 1);
    assert!(TransportSpec::from_yaml(&v1_with_parameters).is_err());
    let v1_json = serde_json::to_string(
        &serde_yaml::from_str::<serde_yaml::Value>(&v1_with_parameters).unwrap(),
    )
    .unwrap();
    assert!(TransportSpec::from_json(&v1_json).is_err());
    assert!(serde_yaml::from_str::<TransportSpec>(&v1_with_parameters).is_err());
    assert!(serde_json::from_str::<TransportSpec>(&v1_json).is_err());
    assert!(TransportSpec::from_yaml(&v1_with_parameters.replace(
        "    parameters:\n      service: event.source.service\n      environment: event.source.environment\n",
        "    parameters: null\n",
    ))
    .is_err());

    let v1_literal = v1_with_parameters.replace(
        "    parameters:\n      service: event.source.service\n      environment: event.source.environment\n",
        "",
    );
    let v1_literal_json = serde_json::to_string(
        &serde_yaml::from_str::<serde_yaml::Value>(&v1_literal).unwrap(),
    )
    .unwrap();
    let duplicate_type = v1_literal_json.replacen(
        "\"type\":\"ess-transport/1\"",
        "\"type\":\"ess-transport/1\",\"type\":\"ess-transport/1\"",
        1,
    );
    assert_ne!(duplicate_type, v1_literal_json);
    assert!(TransportSpec::from_json(&duplicate_type).is_err());
    let spec = TransportSpec::from_yaml(&v1_literal).expect("strict transport/1 parses");
    let compiled = compile(&spec, &ir).expect("braces stay literal under transport/1");
    assert!(
        compiled
            .to_canonical_json()
            .contains("\"format\": \"ess-transport-ir/1\"")
    );
    assert_eq!(
        compiled.channels()["routing.events.UsageRecorded"].subject,
        "usage.{service}.{environment}"
    );
}
