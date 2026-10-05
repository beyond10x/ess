//! Adversary, pass 1, for unit E-U2 (one constant offset, A2): an offset in a stored-row guard.
//!
//! The design admits A2 wherever a predicate is checked (`docs/design/expression-family-source22.md`,
//! A2 and "Target and projection obligations": synthesis "arranges both branches" and returns the
//! named no-witness result only outside the bounded search, which for A2 is "the exact boundary and
//! one unit either side", composing "the base and left candidates"). The unit's own suites exercise
//! offsets in plain input guards and invariants only. Each case here puts one offset in a
//! `when_subject:` guard of a command updating a stored lease, in each arrangement of row and input
//! on the two sides, and asks that synthesis witness both branches and that the native
//! interpreter pass the suite.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const TEMPLATE: &str = r"format: ess/22
system: pool
version: v1
domain: pool.lease
entities:
  - name: pool.lease.Lease
    identity: {name: lease_id, type: Uuid}
    fields:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: expires_at, type: Timestamp}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: pool.lease.Opened
    fields:
      - {name: lease_id, type: Uuid}
  - name: pool.lease.Moved
    fields: []
commands:
  - name: pool.lease.Open
    input:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: expires_at, type: Timestamp}
    outcomes:
      - name: opened
        creates: pool.lease.Lease
        instance: lease_id
        sets: {lower: input.lower, upper: input.upper, expires_at: input.expires_at}
        emits: [pool.lease.Opened]
        payload:
          pool.lease.Opened:
            lease_id: {generated: true}
  - name: pool.lease.Move
    input:
      - {name: lease_id, type: Uuid}
      - {name: to, type: Integer}
      - {name: at, type: Timestamp}
    outcomes:
      - name: held
        when_subject:
          predicate: {GUARD}
        preserves: pool.lease.Lease
        instance: lease_id
      - name: moved
        updates: pool.lease.Lease
        instance: lease_id
        sets: {upper: input.to}
        emits: [pool.lease.Moved]
views:
  - name: pool.lease.Leases
    source: pool.lease.Lease
    consistency: read_your_writes
    fields:
      - {name: lease_id, type: Uuid}
      - {name: state, type: pool.lease.Lease.State}
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: expires_at, type: Timestamp}
";

const HELD: &str = "pool.lease.Move/outcome/held";
const MOVED: &str = "pool.lease.Move/outcome/moved";

fn ir(guard: &str) -> EssIr {
    let text = TEMPLATE.replace("{GUARD}", guard);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("pool.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{guard}: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run(suite: &ConformanceSuite, ir: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Every way the stored-row offset `guard` is not witnessed on both sides and passed by the
/// interpreter.
fn problems(guard: &str) -> Vec<String> {
    let model = ir(guard);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let mut found: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "refused {}: {} {}",
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect();
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for wanted in [HELD, MOVED] {
        if !ids.iter().any(|id| id == wanted) {
            found.push(format!("no scenario {wanted}"));
        }
    }
    for (id, status) in run(&synthesis.suite, model) {
        if status != Status::Passed {
            found.push(format!("{id}: {status:?}"));
        }
    }
    found
}

#[test]
fn adv_e_u2_stored_offset_input_left_row_base() {
    assert_eq!(problems("input.to > upper + 5"), Vec::<String>::new());
}

#[test]
fn adv_e_u2_stored_offset_row_left_row_base() {
    assert_eq!(problems("upper > lower + 10"), Vec::<String>::new());
}

#[test]
fn adv_e_u2_stored_offset_row_left_input_base_equality() {
    assert_eq!(problems("upper == input.to + 5"), Vec::<String>::new());
}

#[test]
fn adv_e_u2_stored_offset_row_left_input_base_ordering() {
    assert_eq!(problems("upper < input.to - 3"), Vec::<String>::new());
}

#[test]
fn adv_e_u2_stored_offset_row_left_input_base_timestamp() {
    assert_eq!(problems("expires_at < input.at + 1h"), Vec::<String>::new());
}

#[test]
fn adv_e_u2_stored_offset_input_left_row_base_timestamp() {
    assert_eq!(
        problems("input.at >= expires_at - 30m"),
        Vec::<String>::new()
    );
}

/// Controls: the same arrangement of row and input without an offset, which synthesis already
/// witnesses — so a refusal above is the offset's, not the stored-row arrangement's.
#[test]
fn adv_e_u2_stored_control_row_left_input_fact() {
    let mut found = problems("upper == input.to");
    found.extend(problems("upper < input.to"));
    found.extend(problems("expires_at < input.at"));
    assert_eq!(found, Vec::<String>::new());
}

/// A struct whose invariant moves one field by a constant (a `TypeInvariant` site), an entity
/// holding it, and a view whose `filter:` is `{FILTER}` (a `View` site).
const SITES: &str = r"format: ess/22
system: pool
version: v1
domain: pool.lease
types:
  - name: pool.lease.Window
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
    invariants:
      - {WINDOW}
entities:
  - name: pool.lease.Lease
    identity: {name: lease_id, type: Uuid}
    fields:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: window, type: pool.lease.Window}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: pool.lease.Opened
    fields:
      - {name: lease_id, type: Uuid}
commands:
  - name: pool.lease.Open
    input:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: window, type: pool.lease.Window}
    outcomes:
      - name: opened
        creates: pool.lease.Lease
        instance: lease_id
        sets: {lower: input.lower, upper: input.upper, window: input.window}
        emits: [pool.lease.Opened]
        payload:
          pool.lease.Opened:
            lease_id: {generated: true}
views:
  - name: pool.lease.Leases
    source: pool.lease.Lease
    consistency: read_your_writes
    fields:
      - {name: lease_id, type: Uuid}
      - {name: state, type: pool.lease.Lease.State}
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: window, type: pool.lease.Window}
  - name: pool.lease.Wide
    source: pool.lease.Lease
    consistency: read_your_writes
    filter: {FILTER}
    fields:
      - {name: lease_id, type: Uuid}
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
";

/// Every refusal and every scenario the interpreter does not pass, for the [`SITES`] model.
fn site_problems(window: &str, filter: &str) -> Vec<String> {
    let text = SITES
        .replace("{WINDOW}", window)
        .replace("{FILTER}", filter);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("pool.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let model = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let mut found: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "refused {}: {} {}",
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect();
    for (id, status) in run(&synthesis.suite, model) {
        if status != Status::Passed {
            found.push(format!("{id}: {status:?}"));
        }
    }
    found
}

#[test]
fn adv_e_u2_struct_invariant_offset_is_witnessed_and_passes() {
    assert_eq!(
        site_problems("high >= low + 3", "upper > lower"),
        Vec::<String>::new()
    );
}

#[test]
fn adv_e_u2_view_filter_offset_is_witnessed_and_passes() {
    assert_eq!(
        site_problems("high >= low", "upper > lower + 5"),
        Vec::<String>::new()
    );
}

/// Control: the same model with neither offset.
#[test]
fn adv_e_u2_sites_control_without_offsets() {
    assert_eq!(
        site_problems("high >= low", "upper > lower"),
        Vec::<String>::new()
    );
}
