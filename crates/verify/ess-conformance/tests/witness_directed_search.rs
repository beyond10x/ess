//! The witness search reaches every leaf a guard reads, however many there are
//! (`story:witness-search-beyond-64-candidates`, follow-up to beyond10x/ess#155).
//!
//! Each integer ladder holds five alternatives, so three leaves describe 216 candidates and four
//! describe 1296. The walk takes the first leaf fastest and stops at 64, so before this story the
//! third leaf never left its first two values and a conjunction over it was refused. Candidates
//! solved for the guard now follow the walk, which keeps every candidate it had in its order.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::mutate;
use ess_conformance::witness::{candidates, Distinction, MAX_CANDIDATES};
use ess_conformance::{flatten, when, Decision};
use ess_domain::name::QualifiedName;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;

const MODEL: &str = r"
format: ess/13
system: demo
version: v1
domain: demo.gate
events:
  - name: demo.gate.Passed
    fields:
      - {name: a, type: Integer}
errors:
  - name: demo.gate.Refused
    summary: Refused.
    fields: []
commands:
  - name: demo.gate.Check
    input:
      - {name: a, type: Integer}
      - {name: b, type: Integer}
      - {name: c, type: Integer}
      - {name: d, type: Decimal}
      - {name: e, type: String}
    outcomes:
      - name: refused
        when:
GUARD
        error: demo.gate.Refused
      - name: passed
        emits: [demo.gate.Passed]
        payload:
          demo.gate.Passed: {a: input.a}
";

fn compiled(guard: &str) -> EssIr {
    let text = MODEL.replace("GUARD", guard);
    let raw = RawSpecFile::parse(&text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.clone());
    mutate::compile(vec![(Source::new("fixture.yaml"), raw)], &texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

fn command(ir: &EssIr) -> &ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new("demo.gate.Check").expect("a valid name"))
        .expect("the command is declared")
}

fn guard(command: &ResolvedCommand) -> &Predicate {
    command
        .outcomes
        .iter()
        .find_map(when)
        .expect("the refused branch is guarded")
}

fn inputs(ir: &EssIr) -> Vec<BTreeMap<String, Node>> {
    let command = command(ir);
    candidates(ir, command, &[guard(command)], Distinction::PLAIN)
        .expect("every input has a witness")
}

fn satisfied(ir: &EssIr, predicate: &Predicate, input: &BTreeMap<String, Node>) -> bool {
    flatten(ir, command(ir), input)
        .expect("a candidate fits its command")
        .decide(predicate)
        == Decision::Satisfied
}

fn children(predicate: &Predicate) -> &[Predicate] {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => children,
        other => panic!("a connective: {other}"),
    }
}

#[test]
fn a_conjunction_over_five_leaves_of_three_types_has_a_satisfying_candidate() {
    let ir = compiled(
        "          all:\n            - a > 10\n            - b > 10\n            - c > 10\n            - d < -5.5\n            - e == \"open\"",
    );
    let inputs = inputs(&ir);
    let predicate = guard(command(&ir));

    assert!(
        inputs.len() <= 2 * MAX_CANDIDATES,
        "the walk and the solved candidates after it are each bounded: {} candidates",
        inputs.len()
    );
    assert!(
        inputs.iter().any(|input| satisfied(&ir, predicate, input)),
        "no candidate of {} satisfies {predicate}",
        inputs.len()
    );
}

#[test]
fn every_truth_assignment_of_a_three_leaf_guard_has_a_candidate() {
    let ir = compiled("          all:\n            - any: [a > 10, b > 10]\n            - c > 10");
    let inputs = inputs(&ir);
    let predicate = guard(command(&ir));
    let [inner, c] = children(predicate) else {
        panic!("two conjuncts")
    };
    let [a, b] = children(inner) else {
        panic!("two disjuncts")
    };

    let reached: BTreeSet<[bool; 3]> = inputs
        .iter()
        .map(|input| [a, b, c].map(|atom| satisfied(&ir, atom, input)))
        .collect();
    let missing: Vec<[bool; 3]> = (0..8u8)
        .map(|bits| [bits & 1 != 0, bits & 2 != 0, bits & 4 != 0])
        .filter(|assignment| !reached.contains(assignment))
        .collect();

    assert!(
        missing.is_empty(),
        "no candidate decides (a > 10, b > 10, c > 10) as {missing:?} among {}",
        inputs.len()
    );
}

/// The walk keeps its 64 candidates in its order; the one satisfying the guard, which the walk
/// never reaches, comes after them rather than in place of one of them.
#[test]
fn a_solved_candidate_follows_the_walk_rather_than_replacing_part_of_it() {
    let ir = compiled("          all:\n            - any: [a > 10, b > 10]\n            - c > 10");
    let inputs = inputs(&ir);
    let predicate = guard(command(&ir));

    let first = inputs
        .iter()
        .position(|input| satisfied(&ir, predicate, input));
    assert!(
        first.is_some_and(|at| at >= MAX_CANDIDATES),
        "the first satisfying candidate is at {first:?} of {}",
        inputs.len()
    );
}
