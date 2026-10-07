//! Adversary pass 1 for story:feature-request-391 (parameterized NATS event addresses).
//!
//! Each case states the contract it drives from: the story's Acceptance, the design page
//! `docs/design/parameterized-event-channel-addresses.md`, or the published reference pages.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_transport::{compile, TransportSpec};

const MODEL: &str = "format: ess/20
system: routing
version: v1
domain: routing.events
types:
  - name: routing.events.Source
    kind: struct
    fields:
      - {name: service, type: String}
      - {name: environment, type: String}
events:
  - name: routing.events.UsageRecorded
    fields:
      - {name: id, type: String}
      - {name: source, type: routing.events.Source}
";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("routing.yaml", MODEL);
    compile_model(&spec, &sources).expect("compiles")
}

fn json_document(ir: &EssIr, parameters: &str) -> String {
    format!(
        concat!(
            "{{\"type\":\"ess-transport/2\",",
            "\"specification\":{{\"system\":\"routing\",\"version\":\"v1\",\"source_digest\":\"sha256:{}\"}},",
            "\"brokers\":[{{\"id\":\"events\",\"protocol\":\"nats\",\"jetstream\":true}}],",
            "\"channels\":[{{\"event\":\"routing.events.UsageRecorded\",\"broker\":\"events\",",
            "\"subject\":\"usage.{{service}}\",\"parameters\":{},",
            "\"envelope\":\"array\",\"delivery\":\"at_most_once\",\"batch\":{{\"max_items\":100,\"max_delay_ms\":5000}}}}],",
            "\"streams\":[{{\"name\":\"USAGE\",\"broker\":\"events\",\"subjects\":[\"usage.>\"],",
            "\"storage\":\"file\",\"retention\":\"limits\",\"owner\":\"external\"}}]}}"
        ),
        ir.source_digest(),
        parameters
    )
}

fn yaml_document(ir: &EssIr, subject: &str, parameters: &str, streams: &str) -> String {
    format!(
        "type: ess-transport/2\nspecification:\n  system: routing\n  version: v1\n  source_digest: sha256:{}\nbrokers:\n  - {{id: events, protocol: nats, jetstream: true}}\nchannels:\n  - event: routing.events.UsageRecorded\n    broker: events\n    subject: '{subject}'\n{parameters}    envelope: array\n    delivery: at_most_once\n    batch: {{max_items: 100, max_delay_ms: 5000}}\nstreams:\n{streams}",
        ir.source_digest()
    )
}

fn stream(name: &str, subject: &str) -> String {
    format!(
        "  - {{name: {name}, broker: events, subjects: ['{subject}'], storage: file, retention: limits, owner: external}}\n"
    )
}

/// The story's closed-admission bullet and the design's "every distinct expression name has
/// exactly one `parameters` entry": a document naming one expression twice, with two different
/// sources, states two contradictory address relations. `/1` already refuses a duplicated
/// top-level key (`compile.rs::transport_v1_rejects_parameters_and_keeps_braces_literal`).
#[test]
fn a_json_parameter_named_twice_with_two_sources_is_refused() {
    let ir = model();
    let text = json_document(
        &ir,
        "{\"service\":\"event.source.service\",\"service\":\"event.id\"}",
    );
    let outcome = TransportSpec::from_json(&text).map(|spec| compile(&spec, &ir));
    match outcome {
        Err(_) | Ok(Err(_)) => {}
        Ok(Ok(compiled)) => panic!(
            "a duplicated `service` mapping compiled; the later source silently won: {}",
            compiled.to_canonical_json()
        ),
    }
}

#[test]
fn a_yaml_parameter_named_twice_with_two_sources_is_refused() {
    let ir = model();
    let text = yaml_document(
        &ir,
        "usage.{service}",
        "    parameters:\n      service: event.source.service\n      service: event.id\n",
        &stream("USAGE", "usage.>"),
    );
    let outcome = TransportSpec::from_yaml(&text).map(|spec| compile(&spec, &ir));
    match outcome {
        Err(_) | Ok(Err(_)) => {}
        Ok(Ok(compiled)) => panic!(
            "a duplicated `service` mapping compiled; the later source silently won: {}",
            compiled.to_canonical_json()
        ),
    }
}

/// Design, "Exact stream coverage": `usage.{x}.{x}` intersects `usage.*.a`, and a terminal `>`
/// treats the remaining positions as wildcards. Every vector is one covering stream plus one other.
#[test]
fn correlated_overlap_vectors_from_the_design_refuse_with_016() {
    let ir = model();
    let same = "    parameters:\n      x: event.source.service\n";
    let aliased =
        "    parameters:\n      left: event.source.service\n      right: event.source.service\n";
    for (case, subject, parameters, other, overlapping) in [
        ("repeated vs *.a", "usage.{x}.{x}", same, "usage.*.a", true),
        (
            "aliased vs *.a",
            "usage.{left}.{right}",
            aliased,
            "usage.*.a",
            true,
        ),
        ("repeated vs a.>", "usage.{x}.{x}", same, "usage.a.>", true),
        (
            "aliased vs a.b",
            "usage.{left}.{right}",
            aliased,
            "usage.a.b",
            false,
        ),
        (
            "repeated vs a.*.>",
            "usage.{x}.{x}",
            same,
            "usage.a.*.>",
            false,
        ),
        ("repeated vs >", "usage.{x}.{x}", same, ">", true),
    ] {
        let text = yaml_document(
            &ir,
            subject,
            parameters,
            &(stream("USAGE", "usage.*.*") + &stream("OTHER", other)),
        );
        let spec = TransportSpec::from_yaml(&text).expect("parses");
        let result = compile(&spec, &ir);
        if overlapping {
            let diagnostics = result.expect_err(case);
            assert!(
                diagnostics.0.iter().any(|d| d.code == "ESS-TRANSPORT-016"),
                "{case}: {diagnostics}"
            );
        } else {
            let compiled = result.unwrap_or_else(|error| panic!("{case}: {error}"));
            assert_eq!(
                compiled.channels()["routing.events.UsageRecorded"]
                    .stream
                    .as_deref(),
                Some("USAGE"),
                "{case}"
            );
        }
    }
}

/// Story: "IR2 determinism. Parameters normalize into ordered tagged `event_path` entries."
#[test]
fn authored_mapping_order_does_not_change_ir2_bytes() {
    let ir = model();
    let forward = "    parameters:\n      service: event.source.service\n      environment: event.source.environment\n";
    let backward = "    parameters:\n      environment: event.source.environment\n      service: event.source.service\n";
    let bytes = |parameters: &str| {
        let text = yaml_document(
            &ir,
            "usage.{service}.{environment}",
            parameters,
            &stream("USAGE", "usage.>"),
        );
        compile(&TransportSpec::from_yaml(&text).expect("parses"), &ir)
            .expect("compiles")
            .to_canonical_json()
    };
    assert_eq!(bytes(forward), bytes(backward));
}

fn repository_file(relative: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Story Acceptance, "Accounting and gates ... format/reference checks", and the story's scope
/// (`website/docs/reference/formats.md`, `spec-versions.md`, `crates/edge/ess-xtask/src/docs.rs`
/// format registry). The build now writes three format versions and four refusal codes that
/// no reference page or registry row names.
#[test]
fn the_reference_pages_and_format_registry_name_every_new_transport_format() {
    let formats = repository_file("website/docs/reference/formats.md");
    let versions = repository_file("website/docs/reference/spec-versions.md");
    let registry = repository_file("crates/edge/ess-xtask/src/docs.rs");
    let mut missing = Vec::new();
    for literal in [
        "ess-transport/2",
        "ess-transport-ir/2",
        "ess-client-report/2",
    ] {
        if !formats.contains(literal) {
            missing.push(format!("formats.md lacks `{literal}`"));
        }
        if !versions.contains(literal) {
            missing.push(format!("spec-versions.md lacks `{literal}`"));
        }
    }
    for (family, version) in [
        ("ess-transport", 2),
        ("ess-transport-ir", 2),
        ("ess-client-report", 2),
    ] {
        if !registry.contains(&format!("(\"{family}\", {version},")) {
            missing.push(format!("FORMAT_RELEASES lacks `{family}/{version}`"));
        }
    }
    for code in [
        "ESS-TRANSPORT-017",
        "ESS-TRANSPORT-018",
        "ESS-TRANSPORT-019",
        "ESS-TRANSPORT-020",
    ] {
        let range = "`ESS-TRANSPORT-001`–`020`";
        if !formats.contains(code) && !formats.contains(range) {
            missing.push(format!("formats.md names no `{code}`"));
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
}
