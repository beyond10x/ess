//! A `precedence-swap` of two input-guarded branches no input selects together is `equivalent`
//! (`ESS-MUTATE-005`, reason `unsatisfiable_guard`) also where the overlap is decided by a part of
//! it: an `Optional` input compared with two literals, whose absence selects neither branch; two
//! pinned values beside a conjunct the check cannot decide; enum attributes resolved over the
//! variant table; and an `any:` whose every disjunct contradicts the other guard
//! (<https://github.com/beyond10x/ess/issues/703>). Guards that can both hold keep their scoring.
//!
//! `story:precedence-swap-of-disjoint-guards-is-equivalent`. Every case runs the mutation audit
//! against the interpreter, as `ess verify conform mutate --target interpreted` does.
#![allow(clippy::needless_raw_string_hashes)]

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutantEntry, MutationReport, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn parsed(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

/// The `precedence-swap` audit of `text`, as `--class precedence-swap` runs it.
fn audit(text: &str) -> MutationReport {
    let (files, texts) = parsed(text);
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn verdicts(report: &MutationReport) -> BTreeMap<String, Verdict> {
    report
        .mutants
        .iter()
        .map(|entry| (entry.id.clone(), entry.verdict))
        .collect()
}

fn swap<'a>(report: &'a MutationReport, id: &str) -> &'a MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no {id} in {:#?}", verdicts(report)))
}

/// `id` is `equivalent` because no input selects both branches, and the reason names `overlap`.
fn equivalent_by_guard(report: &MutationReport, id: &str, overlap: &str) {
    let entry = swap(report, id);
    assert_eq!(
        entry.verdict,
        Verdict::Equivalent,
        "{id}\n{}",
        report.render_text()
    );
    assert_eq!(
        entry.unsatisfiable_guard.as_deref(),
        Some(overlap),
        "{id}\n{}",
        report.render_text()
    );
}

/// `id` is scored: neither `equivalent` nor named as an overlap no input satisfies.
fn scored(report: &MutationReport, id: &str) {
    let entry = swap(report, id);
    assert_ne!(
        entry.verdict,
        Verdict::Equivalent,
        "{id}\n{}",
        report.render_text()
    );
    assert_eq!(
        entry.unsatisfiable_guard,
        None,
        "{id}\n{}",
        report.render_text()
    );
}

/// The exit status `ess verify conform mutate` gives `report`: 1 with a survivor, 3 with an
/// unwitnessed or inconclusive mutant, none scored or a site not audited, 0 otherwise.
fn exit_status(report: &MutationReport) -> u8 {
    let ran = report
        .mutants
        .iter()
        .filter(|entry| !matches!(entry.verdict, Verdict::Stillborn | Verdict::Equivalent))
        .count();
    if report.counts.survived > 0 {
        1
    } else if report.counts.unwitnessed > 0
        || report.counts.inconclusive > 0
        || ran == 0
        || report.unaudited_sites() > 0
    {
        3
    } else {
        0
    }
}

/// The issue's reproduction, with one swap of two overlapping refusals a scenario kills.
const REPRODUCTION: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.gate
summary: Variants of two input-only refusals that never both hold.

components:
  - component: demo-server
    owns:
      domains: [demo.gate]
    accepts:
      commands: [demo.gate.Optional, demo.gate.OptionalLane, demo.gate.Mixed, demo.gate.MixedLane, demo.gate.Truth, demo.gate.Measure]
    reached_by: network

types:
  - {name: demo.gate.Lane, kind: enum, variants: [north, south, east]}

errors:
  - name: demo.gate.RefusedA
    summary: Refused for the first reason.
  - name: demo.gate.RefusedB
    summary: Refused for the second reason.

commands:
  - name: demo.gate.Optional
    input:
      - {name: flag, type: Optional<Boolean>}
    outcomes:
      - name: refused-a
        when: flag == false
        error: demo.gate.RefusedA
      - name: refused-b
        when: flag == true
        error: demo.gate.RefusedB
      - name: passed
        accepts: nothing
  - name: demo.gate.OptionalLane
    input:
      - {name: lane, type: Optional<demo.gate.Lane>}
    outcomes:
      - name: refused-north
        when: lane == "north"
        error: demo.gate.RefusedA
      - name: refused-south
        when: lane == "south"
        error: demo.gate.RefusedB
      - name: routed
        accepts: nothing
  - name: demo.gate.Mixed
    input:
      - {name: flag, type: Boolean}
      - {name: amount, type: Integer}
    outcomes:
      - name: refused-a
        when: {all: ['flag == false', 'amount > 0']}
        error: demo.gate.RefusedA
      - name: refused-b
        when: {all: ['flag == true', 'amount > 0']}
        error: demo.gate.RefusedB
      - name: passed
        accepts: nothing
  - name: demo.gate.MixedLane
    input:
      - {name: lane, type: demo.gate.Lane}
      - {name: note, type: String}
    outcomes:
      - name: refused-north
        when: {all: ['lane == "north"', {note: {starts_with: "x"}}]}
        error: demo.gate.RefusedA
      - name: refused-south
        when: {all: ['lane == "south"', {note: {starts_with: "x"}}]}
        error: demo.gate.RefusedB
      - name: routed
        accepts: nothing
  - name: demo.gate.Truth
    input:
      - {name: flag, type: Boolean}
    outcomes:
      - name: refused-a
        when: {not: flag}
        error: demo.gate.RefusedA
      - name: refused-b
        when: flag
        error: demo.gate.RefusedB
      - name: passed
        accepts: nothing
  - name: demo.gate.Measure
    input:
      - {name: amount, type: Integer}
    outcomes:
      - name: too-low
        when: amount < 10
        error: demo.gate.RefusedA
      - name: low
        when: amount < 20
        error: demo.gate.RefusedB
      - name: measured
        accepts: nothing
"#;

#[test]
fn an_optional_input_pinned_to_two_values_is_equivalent() {
    let report = audit(REPRODUCTION);
    equivalent_by_guard(
        &report,
        "precedence-swap/demo.gate.Optional/refused-a/refused-b",
        "(flag == false and flag == true)",
    );
    equivalent_by_guard(
        &report,
        "precedence-swap/demo.gate.OptionalLane/refused-north/refused-south",
        "(lane == north and lane == south)",
    );
}

#[test]
fn a_contradiction_beside_an_undecidable_conjunct_is_equivalent() {
    let report = audit(REPRODUCTION);
    // The reason names the contradictory conjuncts, not the ordering or the text match beside them.
    equivalent_by_guard(
        &report,
        "precedence-swap/demo.gate.Mixed/refused-a/refused-b",
        "(flag == false and flag == true)",
    );
    equivalent_by_guard(
        &report,
        "precedence-swap/demo.gate.MixedLane/refused-north/refused-south",
        "(lane == north and lane == south)",
    );
    // The control decided before keeps its reason byte for byte.
    equivalent_by_guard(
        &report,
        "precedence-swap/demo.gate.Truth/refused-a/refused-b",
        "(not (flag) and flag)",
    );
}

#[test]
fn the_precedence_swap_class_reaches_exit_zero() {
    let report = audit(REPRODUCTION);
    assert_eq!(
        swap(&report, "precedence-swap/demo.gate.Measure/too-low/low").verdict,
        Verdict::Killed,
        "{}",
        report.render_text()
    );
    assert_eq!(report.counts.equivalent, 5, "{}", report.render_text());
    assert_eq!(exit_status(&report), 0, "{}", report.render_text());
}

/// Swaps whose guards some input satisfies together, and one (`Absent`) whose guards no input
/// satisfies together but where an absent `Optional` input makes the first guard true and leaves
/// the second unknown: read first, the unknown guard is a refusal naming the command where the
/// first declared answered, so their order is observable there.
const OVERLAPPING: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.both
summary: Input-only refusals that can both hold.

components:
  - component: demo-server
    owns:
      domains: [demo.both]
    accepts:
      commands: [demo.both.Beside, demo.both.Apart, demo.both.Unknown, demo.both.Either, demo.both.Absent]
    reached_by: network

types:
  - {name: demo.both.Lane, kind: enum, variants: [north, south, east]}

errors:
  - name: demo.both.RefusedA
    summary: Refused for the first reason.
  - name: demo.both.RefusedB
    summary: Refused for the second reason.

commands:
  - name: demo.both.Beside
    input:
      - {name: flag, type: Boolean}
      - {name: amount, type: Integer}
    outcomes:
      - name: refused-a
        when: {all: ['flag == true', 'amount > 0']}
        error: demo.both.RefusedA
      - name: refused-b
        when: {all: ['flag == true', 'amount > 5']}
        error: demo.both.RefusedB
      - name: passed
        accepts: nothing
  - name: demo.both.Apart
    input:
      - {name: flag, type: Boolean}
      - {name: lane, type: demo.both.Lane}
      - {name: amount, type: Integer}
    outcomes:
      - name: refused-a
        when: {all: ['flag == false', 'amount > 0']}
        error: demo.both.RefusedA
      - name: refused-b
        when: {all: ['lane == "north"', 'amount > 0']}
        error: demo.both.RefusedB
      - name: passed
        accepts: nothing
  - name: demo.both.Unknown
    input:
      - {name: flag, type: Optional<Boolean>}
      - {name: lane, type: demo.both.Lane}
    outcomes:
      - name: refused-a
        when: flag == true
        error: demo.both.RefusedA
      - name: refused-b
        when: lane == "north"
        error: demo.both.RefusedB
      - name: passed
        accepts: nothing
  - name: demo.both.Either
    input:
      - {name: flag, type: Boolean}
      - {name: amount, type: Integer}
    outcomes:
      - name: refused-a
        when: {any: ['flag == false', 'amount > 0']}
        error: demo.both.RefusedA
      - name: refused-b
        when: flag == true
        error: demo.both.RefusedB
      - name: passed
        accepts: nothing
  - name: demo.both.Absent
    input:
      - {name: mark, type: Optional<Boolean>}
    outcomes:
      - name: refused-a
        when: {mark: {defined: false}}
        error: demo.both.RefusedA
      - name: refused-b
        when: mark == true
        error: demo.both.RefusedB
      - name: passed
        accepts: nothing
"#;

#[test]
fn guards_that_can_both_hold_are_not_equivalent() {
    let report = audit(OVERLAPPING);
    for id in [
        "precedence-swap/demo.both.Beside/refused-a/refused-b",
        "precedence-swap/demo.both.Apart/refused-a/refused-b",
        "precedence-swap/demo.both.Unknown/refused-a/refused-b",
        "precedence-swap/demo.both.Either/refused-a/refused-b",
        "precedence-swap/demo.both.Absent/refused-a/refused-b",
    ] {
        scored(&report, id);
    }
}

/// The downstream report's shapes, with neutral names. Every enum and every guard has more value
/// combinations than the 64 the whole-overlap check tries, as the reported specification's do.
const DOWNSTREAM: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.flow
summary: Input-only refusals over plain Boolean and enum inputs and enum attributes.

components:
  - component: demo-server
    owns:
      domains: [demo.flow]
    accepts:
      commands: [demo.flow.Pinned, demo.flow.Routed, demo.flow.Split, demo.flow.Shared, demo.flow.Either, demo.flow.EitherBeside]
    reached_by: network

types:
  - {name: demo.flow.E, kind: enum, variants: [A, B, D]}
  - {name: demo.flow.C, kind: enum, variants: [H, J, K]}
  - {name: demo.flow.F, kind: enum, variants: [X, Y, Z, W, U, R, S, T]}
  - name: demo.flow.Op
    kind: enum
    attributes:
      - {name: attr1, type: Boolean}
      - {name: attr2, type: Boolean}
      - {name: attr_a, type: Boolean}
      - {name: attr_b, type: Boolean}
    variants:
      - {name: V1, attributes: {attr1: true, attr2: false, attr_a: true, attr_b: false}}
      - {name: V2, attributes: {attr1: true, attr2: true, attr_a: true, attr_b: false}}
      - {name: V3, attributes: {attr1: false, attr2: false, attr_a: false, attr_b: true}}
      - {name: V4, attributes: {attr1: false, attr2: true, attr_a: false, attr_b: true}}
      - {name: V5, attributes: {attr1: true, attr2: false, attr_a: false, attr_b: false}}
      - {name: V6, attributes: {attr1: true, attr2: true, attr_a: false, attr_b: false}}
      - {name: V7, attributes: {attr1: false, attr2: false, attr_a: true, attr_b: false}}
      - {name: V8, attributes: {attr1: false, attr2: true, attr_a: false, attr_b: true}}
      - {name: V9, attributes: {attr1: true, attr2: false, attr_a: false, attr_b: false}}
  - name: demo.flow.SharedOp
    kind: enum
    attributes:
      - {name: attr_a, type: Boolean}
      - {name: attr_b, type: Boolean}
    variants:
      - {name: V1, attributes: {attr_a: true, attr_b: false}}
      - {name: V2, attributes: {attr_a: true, attr_b: true}}
      - {name: V3, attributes: {attr_a: false, attr_b: true}}
      - {name: V4, attributes: {attr_a: false, attr_b: false}}
      - {name: V5, attributes: {attr_a: false, attr_b: false}}
      - {name: V6, attributes: {attr_a: false, attr_b: false}}
      - {name: V7, attributes: {attr_a: false, attr_b: false}}
      - {name: V8, attributes: {attr_a: false, attr_b: false}}
      - {name: V9, attributes: {attr_a: false, attr_b: false}}

errors:
  - name: demo.flow.RefusedA
    summary: Refused for the first reason.
  - name: demo.flow.RefusedB
    summary: Refused for the second reason.

commands:
  - name: demo.flow.Pinned
    input:
      - {name: enum_e, type: demo.flow.E}
      - {name: enum_c, type: demo.flow.C}
      - {name: bool_p, type: Boolean}
      - {name: bool_q, type: Boolean}
      - {name: bool_v, type: Boolean}
    outcomes:
      - name: first
        when: {all: ['enum_e == "A"', 'enum_c == "H"', 'bool_p == false', 'bool_q == true']}
        error: demo.flow.RefusedA
      - name: second
        when: {all: ['enum_c == "H"', 'bool_p == true', 'bool_v == false']}
        error: demo.flow.RefusedB
      - name: rest
        accepts: nothing
  - name: demo.flow.Routed
    input:
      - {name: enum_f, type: demo.flow.F}
      - {name: op, type: demo.flow.Op}
    outcomes:
      - name: first
        when: {all: ['enum_f == "X"', 'op.attr1 == true', 'op.attr2 == false']}
        error: demo.flow.RefusedA
      - name: second
        when: {all: ['enum_f == "Y"', 'op.attr1 == true', 'op.attr2 == false']}
        error: demo.flow.RefusedB
      - name: rest
        accepts: nothing
  - name: demo.flow.Split
    input:
      - {name: op, type: demo.flow.Op}
      - {name: bool_i, type: Boolean}
      - {name: bool_h, type: Boolean}
      - {name: bool_k, type: Boolean}
    outcomes:
      - name: first
        when: {all: ['op.attr_a == true', 'bool_i == true', 'bool_h == false']}
        error: demo.flow.RefusedA
      - name: second
        when: {all: ['op.attr_b == true', 'bool_i == true', 'bool_k == false']}
        error: demo.flow.RefusedB
      - name: rest
        accepts: nothing
  - name: demo.flow.Shared
    input:
      - {name: op, type: demo.flow.SharedOp}
      - {name: bool_i, type: Boolean}
      - {name: bool_h, type: Boolean}
      - {name: bool_k, type: Boolean}
    outcomes:
      - name: first
        when: {all: ['op.attr_a == true', 'bool_i == true', 'bool_h == false']}
        error: demo.flow.RefusedA
      - name: second
        when: {all: ['op.attr_b == true', 'bool_i == true', 'bool_k == false']}
        error: demo.flow.RefusedB
      - name: rest
        accepts: nothing
  - name: demo.flow.Either
    input:
      - {name: op, type: demo.flow.Op}
      - {name: bool_s, type: Boolean}
      - {name: bool_k, type: Boolean}
      - {name: bool_i, type: Boolean}
      - {name: bool_b, type: Boolean}
    outcomes:
      - name: first
        when: {all: ['op.attr2 == true', 'bool_s == true', 'bool_k == false']}
        error: demo.flow.RefusedA
      - name: second
        when: {all: [{any: ['op.attr2 == false', 'bool_s == false']}, 'bool_i == true', 'bool_b == false']}
        error: demo.flow.RefusedB
      - name: rest
        accepts: nothing
  - name: demo.flow.EitherBeside
    input:
      - {name: op, type: demo.flow.Op}
      - {name: bool_s, type: Boolean}
      - {name: bool_i, type: Boolean}
      - {name: amount, type: Integer}
    outcomes:
      - name: first
        when: {all: ['op.attr2 == true', 'bool_s == true', 'amount > 0']}
        error: demo.flow.RefusedA
      - name: second
        when: {all: [{any: [{all: ['op.attr2 == false', 'amount > 5']}, 'bool_s == false']}, 'bool_i == true']}
        error: demo.flow.RefusedB
      - name: rest
        accepts: nothing
"#;

#[test]
fn a_pair_disjoint_on_one_boolean_beside_other_pins_is_equivalent() {
    let report = audit(DOWNSTREAM);
    equivalent_by_guard(
        &report,
        "precedence-swap/demo.flow.Pinned/first/second",
        "(bool_p == false and bool_p == true)",
    );
}

#[test]
fn a_pair_disjoint_on_an_enum_beside_variant_attributes_is_equivalent() {
    let report = audit(DOWNSTREAM);
    equivalent_by_guard(
        &report,
        "precedence-swap/demo.flow.Routed/first/second",
        "(enum_f == X and enum_f == Y)",
    );
}

#[test]
fn a_pair_disjoint_through_variant_attributes_is_equivalent() {
    let report = audit(DOWNSTREAM);
    let entry = swap(&report, "precedence-swap/demo.flow.Split/first/second");
    assert_eq!(
        entry.verdict,
        Verdict::Equivalent,
        "{}",
        report.render_text()
    );
    // Decided over the variant table: the two attribute reads hold for no variant together.
    let overlap = entry
        .unsatisfiable_guard
        .as_deref()
        .expect("an equivalent swap names the overlap no input satisfies");
    assert!(
        overlap.contains("op") && !overlap.contains("bool_"),
        "the reason names the attribute reads alone: {overlap}"
    );
}

#[test]
fn a_pair_sharing_a_variant_with_both_attributes_is_not_equivalent() {
    let report = audit(DOWNSTREAM);
    scored(&report, "precedence-swap/demo.flow.Shared/first/second");
}

#[test]
fn a_pair_disjoint_through_a_disjunction_is_equivalent() {
    let report = audit(DOWNSTREAM);
    for id in [
        "precedence-swap/demo.flow.Either/first/second",
        // A disjunct holding a conjunct the check cannot decide is decided by its pinned part.
        "precedence-swap/demo.flow.EitherBeside/first/second",
    ] {
        let entry = swap(&report, id);
        assert_eq!(
            entry.verdict,
            Verdict::Equivalent,
            "{id}\n{}",
            report.render_text()
        );
        let overlap = entry
            .unsatisfiable_guard
            .as_deref()
            .expect("an equivalent swap names the overlap no input satisfies");
        assert!(
            overlap.contains("bool_s == false") && !overlap.contains("bool_i"),
            "{id}: the reason names the disjunction and what contradicts it: {overlap}"
        );
    }
}
