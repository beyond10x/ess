//! Adversary pass 1 for `story:a-null-member-matches-an-absent-optional-in-a-nested-assertion`.
//!
//! The unit made an absent struct member and a `null` one one value at any depth in the runners'
//! `matches` (and the Rust runner's `mismatch`). These cases drive the three runners through
//! scenarios the unit's own tests do not: an error's named fields and an immediate event's payload,
//! each carrying a struct whose `Optional` member the interpreter leaves absent, and a view field
//! typed `Map<String, Optional<Integer>>`, whose keys are data and not struct members.

mod support_go;
mod support_ts;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Source as AuthoredSource};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, AdmittedSuite, Runner,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// A command that records a `Quantity` and announces it, and one that refuses with it.
const MODEL: &str = r"
format: ess/19
system: demo
version: v1
domain: demo.usage
types:
  - name: demo.usage.Quantity
    kind: struct
    fields:
      - {name: kind, type: String}
      - {name: source, type: String}
      - {name: value, type: 'Optional<Integer>'}
errors:
  - name: demo.usage.Refused
    summary: The quantity was refused.
    fields:
      - {name: latest, type: demo.usage.Quantity}
entities:
  - name: demo.usage.Record
    identity: {name: record_id, type: Uuid}
    fields:
      - {name: latest, type: demo.usage.Quantity}
      - {name: scores, type: 'Map<String, Optional<Integer>>'}
    lifecycle:
      initial: Recorded
      states: [Recorded]
      terminal: [Recorded]
events:
  - name: demo.usage.Recorded
    fields:
      - {name: record_id, type: Uuid}
      - {name: latest, type: demo.usage.Quantity}
commands:
  - name: demo.usage.Put
    input:
      - {name: latest, type: demo.usage.Quantity}
    outcomes:
      - name: recorded
        creates: demo.usage.Record
        instance: record_id
        emits: [demo.usage.Recorded]
        payload:
          demo.usage.Recorded: {record_id: {generated: true}, latest: input.latest}
        sets:
          latest: input.latest
  - name: demo.usage.Refuse
    input:
      - {name: latest, type: demo.usage.Quantity}
    outcomes:
      - name: refused
        error: demo.usage.Refused
        payload:
          demo.usage.Refused: {latest: input.latest}
views:
  - name: demo.usage.Records
    source: demo.usage.Record
    consistency: read_your_writes
    order_by: [record_id asc]
    fields:
      - {name: record_id, type: Uuid}
      - {name: scores, type: 'Map<String, Optional<Integer>>'}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("usage.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The admitted suite holding exactly the authored `documents`.
fn suite(model: &EssIr, documents: &[(&str, String)]) -> AdmittedSuite {
    let sources: Vec<AuthoredSource> = documents
        .iter()
        .map(|(label, text)| AuthoredSource::new(format!("{label}.yaml"), text.clone()))
        .collect();
    let authored = compile_authored(model, &sources);
    assert!(authored.is_complete(), "{:?}", authored.refusals);
    let mut suite = synthesize(model).suite;
    suite.scenarios = authored.scenarios;
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"))
}

fn written(status: Status) -> String {
    match status {
        Status::Passed => "passed",
        Status::Failed => "failed",
        Status::Error => "error",
        Status::Unsupported => "unsupported",
    }
    .to_owned()
}

/// The Rust, Go and TypeScript verdicts over one recorded run of the interpreter.
struct Verdicts {
    rust: BTreeMap<String, String>,
    go: BTreeMap<String, String>,
    ts: BTreeMap<String, String>,
    log: String,
}

fn three_runners(label: &str, documents: &[(&str, String)]) -> Verdicts {
    let model = ir();
    let admitted = suite(&model, documents);
    let recorder = support_go::Recorder::new(Interpreted::for_model(model));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &recorder)
        .into_report();
    let rust: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .map(|run| (run.scenario.to_string(), written(run.status)))
        .collect();

    let directory = support_go::package(label, admitted.suite(), &[support_go::TRANSCRIPT_TARGET]);
    let go = support_go::replay(&directory, &recorder, &[]);
    assert!(
        go.divergences.is_empty(),
        "the Go runtime sent requests the reference runner did not:\n{}\n{}",
        go.divergences.join("\n"),
        go.go.log
    );

    let package = support_ts::Package::new(label, admitted.suite());
    let ts = package.replay(&admitted, &recorder.transcript(), false);
    assert_ne!(ts.exit, Some(2), "{}", ts.log);
    assert!(
        ts.divergences.is_empty(),
        "the TypeScript runtime sent requests the reference runner did not:\n{}\n{}",
        ts.divergences.join("\n"),
        ts.log
    );
    let ts_outcomes = ts.outcomes.clone();
    Verdicts {
        rust,
        go: go.go.outcomes,
        ts: ts_outcomes,
        log: format!(
            "transcript: {}\nrust report: {report:?}\ngo: {}\nts: {}",
            recorder.transcript(),
            go.go.log,
            ts.log
        ),
    }
}

/// `Refuse` with a `Quantity` whose `value` is left out, asserting the error's field as `asserted`.
fn refused(asserted: &str) -> String {
    format!(
        r"type: ess-scenario/2
domain: demo.usage
scenario: refused-nested-null
summary: The error carries the quantity the command was given.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.usage.Refuse
    input:
      latest: {{kind: cached_input_tokens, source: unknown}}
    error:
      name: demo.usage.Refused
      fields: {{latest: {{kind: cached_input_tokens, source: unknown{asserted}}}}}
"
    )
}

/// `Record` with a `Quantity` whose `value` is left out, asserting the event's payload as `asserted`.
fn recorded(asserted: &str) -> String {
    format!(
        r"type: ess-scenario/2
domain: demo.usage
scenario: recorded-nested-null
summary: The event carries the quantity the command was given.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.usage.Put
    input:
      latest: {{kind: cached_input_tokens, source: unknown}}
    outcome: recorded
    events:
      - event: demo.usage.Recorded
        payload: {{latest: {{kind: cached_input_tokens, source: unknown{asserted}}}}}
"
    )
}

/// Attack 3: the CHANGELOG says the three runners compare an error's named fields alike. The Rust
/// runner's `expect_error` reads `mismatch`, now `same_value`; the TypeScript `expectError` reads
/// `matches`, now `sameValue`; the Go `expectError` reads `payloadCarries`, still `equal`.
#[test]
fn adv_u28_the_three_runners_agree_on_an_error_field_with_a_nested_null_member() {
    let verdicts = three_runners("u28-error", &[("refused", refused(", value: null"))]);
    assert!(!verdicts.rust.is_empty(), "{}", verdicts.log);
    assert_eq!(
        (&verdicts.go, &verdicts.ts),
        (&verdicts.rust, &verdicts.rust),
        "(Go, TypeScript) left, (Rust, Rust) right\n{}",
        verdicts.log
    );
}

/// Attack 2/3: an immediate event's payload. The Rust runner's `expect_payload` and `carries` and
/// the Go `payloadCarries` keep exact comparison (the CHANGELOG does not list immediate events), but
/// the TypeScript `expectEvent`, `expectEventValues` and `claimEvent` read `matches`, which the unit
/// relaxed, so the TypeScript runner now passes what the other two fail.
#[test]
fn adv_u28_the_three_runners_agree_on_an_immediate_event_with_a_nested_null_member() {
    let verdicts = three_runners("u28-event", &[("recorded", recorded(", value: null"))]);
    assert!(!verdicts.rust.is_empty(), "{}", verdicts.log);
    assert_eq!(
        (&verdicts.go, &verdicts.ts),
        (&verdicts.rust, &verdicts.rust),
        "(Go, TypeScript) left, (Rust, Rust) right\n{}",
        verdicts.log
    );
}

/// Pins the CHANGELOG's claim for errors on the Rust runner: `mismatch` is the comparison
/// `expect_error` reads, and no test of the unit's reaches it, so `mismatch` reverted to `==` stays
/// green on the unit's suite.
#[test]
fn adv_u28_an_error_field_with_a_nested_null_member_passes_on_the_rust_runner() {
    let model = ir();
    let admitted = suite(&model, &[("refused", refused(", value: null"))]);
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report();
    let statuses: Vec<Status> = report.scenarios.iter().map(|run| run.status).collect();
    assert_eq!(statuses, vec![Status::Passed], "{report:?}");
}

/// A view row's `scores` set by the arrangement, asserted by `contains`, on the interpreter.
fn scores_status(stored: &str, asserted: &str) -> (Status, String) {
    let model = ir();
    let document = format!(
        r"type: ess-scenario/2
domain: demo.usage
scenario: map-null
summary: A map's keys are data.
arrange:
  - instance: usage
    entity: demo.usage.Record
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {{latest: {{kind: k, source: s}}, scores: {stored}}}
      state: Recorded
assert:
  - view: demo.usage.Records
    contains: {{record_id: {{$instance: usage}}, scores: {asserted}}}
"
    );
    let admitted = suite(&model, &[("map", document)]);
    let recorder = support_go::Recorder::new(Interpreted::for_model(model));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &recorder)
        .into_report();
    let status = report.scenarios.first().expect("one scenario").status;
    (
        status,
        format!("transcript: {}\nreport: {report:?}", recorder.transcript()),
    )
}

/// Attack 4: a `Map` value is not a struct, but a view assertion carries no declared type, so
/// `same_value` reads a map's keys as struct members and a key holding `null` reads as absent.
#[test]
fn a_map_key_holding_null_reads_as_absent_until_assertions_carry_their_type() {
    // story:view-and-error-assertions-carry-their-value-shape makes these two differ.
    let failing: Vec<String> = [("{a: null}", "{}"), ("{}", "{a: null}")]
        .into_iter()
        .filter_map(|(stored, asserted)| {
            let (status, log) = scores_status(stored, asserted);
            (status != Status::Passed)
                .then(|| format!("stored `{stored}`, asserted `{asserted}`: {status:?}\n{log}"))
        })
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}

/// Control for the map case: a map whose key holds a value still differs from the empty map.
#[test]
fn adv_u28_a_map_key_holding_a_value_is_not_the_empty_map() {
    let (status, log) = scores_status("{a: 3}", "{}");
    assert_eq!(status, Status::Failed, "{log}");
}

/// Attack 1: a required nested member asserted `null` is refused where the scenario is authored, so
/// the relaxed comparison never sees it.
#[test]
fn adv_u28_a_required_nested_member_asserted_null_is_refused_at_authoring() {
    let model = ir();
    let text = r"type: ess-scenario/2
domain: demo.usage
scenario: required-null
summary: A required member asserted null.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.usage.Refuse
    input:
      latest: {kind: cached_input_tokens, source: unknown}
    error:
      name: demo.usage.Refused
      fields: {latest: {kind: null, source: unknown}}
";
    let authored = compile_authored(&model, &[AuthoredSource::new("required.yaml", text)]);
    assert!(
        !authored.is_complete(),
        "a required member written null was admitted: {:?}",
        authored.scenarios
    );
}
