//! The witness search reaches every leaf a guard reads, however many there are
//! (`story:witness-search-beyond-64-candidates`, follow-up to beyond10x/ess#155).
//!
//! Each integer ladder holds five alternatives, so three leaves describe 216 candidates and four
//! describe 1296. The walk takes the first leaf fastest and stops at 64, so before this story the
//! third leaf never left its first two values and a conjunction over it was refused. Candidates
//! solved for the guard now follow the walk, which keeps every candidate it had in its order.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::mutate;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::witness::{candidates, directed_steps, Distinction, MAX_CANDIDATES};
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

// A first-match chain over finite inputs (`story:a-first-match-chain-is-synthesized-within-a-work-
// bound`, an adopter's report): one four-valued enum, one three-valued enum and six Booleans, and
// input-guarded refusals in declaration order before one creating outcome. Each refusal's witness
// is searched with every refusal declared before it refuted, and refuting a conjunction is a
// choice of which conjunct is false, so the search branches once per earlier refusal.

/// Each refusal's guard: the conjunction of these `input == value` atoms. Drawn at random, two or
/// three atoms over distinct inputs each; before 0.69.0 its first twelve refused the reachable
/// `refuse-10` after about four million guard-directed steps.
const CHAIN: [&[(&str, &str)]; 16] = [
    &[("b6", "true"), ("b5", "false"), ("e2", "X")],
    &[("b2", "false"), ("e1", "D")],
    &[("b2", "true"), ("e2", "Z"), ("e1", "C")],
    &[("b5", "true"), ("b6", "true")],
    &[("b1", "false"), ("b4", "false"), ("b5", "true")],
    &[("b3", "true"), ("e2", "Y")],
    &[("b2", "true"), ("e1", "C")],
    &[("b1", "true"), ("e1", "C")],
    &[("b5", "false"), ("b4", "false"), ("b2", "true")],
    &[("b2", "false"), ("e1", "B"), ("b4", "false")],
    &[("b5", "false"), ("e2", "Y"), ("b3", "false")],
    &[("b3", "false"), ("e2", "X"), ("b6", "true")],
    &[("b5", "true"), ("e2", "X"), ("b1", "true")],
    &[("b6", "true"), ("b4", "false"), ("b2", "false")],
    &[("b2", "true"), ("e1", "B")],
    &[("e1", "B"), ("b3", "false")],
];

const BOOLEANS: [&str; 6] = ["b1", "b2", "b3", "b4", "b5", "b6"];

/// The suffix of a refusal's error type: `refuse-10` refuses with `Refused10`.
fn error_name(outcome: &str) -> String {
    outcome
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |head| {
                head.to_uppercase().chain(chars).collect()
            })
        })
        .collect::<String>()
        .replacen("Refuse", "", 1)
}

/// The model: the refusals named `refuse-<n>` with these guards, in this order, then `created`.
fn chain_model(guards: &[(String, Vec<(&str, &str)>)]) -> EssIr {
    let mut text = String::from(
        "format: ess/19\nsystem: demo\nversion: v1\ndomain: demo.chain\ntypes:\n  - {name: demo.chain.First, kind: enum, variants: [A, B, C, D]}\n  - {name: demo.chain.Second, kind: enum, variants: [X, Y, Z]}\nentities:\n  - name: demo.chain.Request\n    identity: {name: request_id, type: Uuid}\n    fields:\n      - {name: e1, type: demo.chain.First}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nevents:\n  - name: demo.chain.Submitted\n    fields: [{name: request_id, type: Uuid}]\nerrors:\n",
    );
    text.extend(guards.iter().map(|(name, _)| {
        format!(
            "  - {{name: demo.chain.Refused{}, summary: Refused., fields: []}}\n",
            error_name(name)
        )
    }));
    text.push_str(
        "commands:\n  - name: demo.chain.Submit\n    input:\n      - {name: e1, type: demo.chain.First}\n      - {name: e2, type: demo.chain.Second}\n",
    );
    text.extend(
        BOOLEANS
            .iter()
            .map(|boolean| format!("      - {{name: {boolean}, type: Boolean}}\n")),
    );
    text.push_str("    outcomes:\n");
    text.extend(guards.iter().map(|(name, atoms)| {
        let atoms: Vec<String> = atoms
            .iter()
            .map(|(input, value)| format!("\"{input} == {value}\""))
            .collect();
        format!(
            "      - name: {name}\n        when: {{all: [{}]}}\n        error: demo.chain.Refused{}\n",
            atoms.join(", "),
            error_name(name)
        )
    }));
    text.push_str("      - name: created\n        creates: demo.chain.Request\n        instance: request_id\n        sets: {e1: input.e1}\n        emits: [demo.chain.Submitted]\n        payload:\n          demo.chain.Submitted: {request_id: {generated: true}}\n");
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("chain.yaml".to_owned(), text.clone());
    mutate::compile(vec![(Source::new("chain.yaml"), raw)], &texts).unwrap_or_else(|stillborn| {
        panic!(
            "the model compiles: {} {}\n{text}",
            stillborn.code, stillborn.cause
        )
    })
}

/// The first `n` refusals of [`CHAIN`], named `refuse-1` onwards.
fn first(n: usize) -> Vec<(String, Vec<(&'static str, &'static str)>)> {
    CHAIN[..n]
        .iter()
        .enumerate()
        .map(|(at, atoms)| (format!("refuse-{}", at + 1), atoms.to_vec()))
        .collect()
}

/// The branches some input selects, first match first, by trying all 768 inputs.
fn reachable(guards: &[(String, Vec<(&str, &str)>)]) -> BTreeSet<String> {
    let mut reached = BTreeSet::new();
    for first in ["A", "B", "C", "D"] {
        for second in ["X", "Y", "Z"] {
            for bits in 0..64u8 {
                let value = |input: &str| -> String {
                    match input {
                        "e1" => first.to_owned(),
                        "e2" => second.to_owned(),
                        boolean => {
                            let at = BOOLEANS
                                .iter()
                                .position(|known| *known == boolean)
                                .expect("a declared Boolean");
                            (bits & (1 << at) != 0).to_string()
                        }
                    }
                };
                let taken = guards
                    .iter()
                    .find(|(_, atoms)| atoms.iter().all(|(input, wanted)| value(input) == *wanted))
                    .map_or("created", |(name, _)| name.as_str());
                reached.insert(taken.to_owned());
            }
        }
    }
    reached
}

/// Every reachable branch has its scenario, and every other one is refused naming itself.
fn assert_covered(guards: &[(String, Vec<(&str, &str)>)], synthesis: &Synthesis) {
    let reached = reachable(guards);
    let names = guards
        .iter()
        .map(|(name, _)| name.as_str())
        .chain(["created"]);
    for name in names {
        let id = format!("demo.chain.Submit/outcome/{name}");
        let present = synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id);
        let refused: Vec<String> = synthesis
            .refusals
            .iter()
            .filter(|refusal| {
                refusal
                    .scenario
                    .as_ref()
                    .is_some_and(|scenario| scenario.to_string() == id)
            })
            .map(|refusal| refusal.cause.to_string())
            .collect();
        if reached.contains(name) {
            assert!(
                present,
                "`{name}` is reachable and has no scenario; refused as {refused:#?}"
            );
        } else {
            assert!(
                !present && !refused.is_empty(),
                "`{name}` is reachable by no input: scenario {present}, refused as {refused:#?}"
            );
        }
    }
}

/// How many guard-directed steps synthesizing the sixteen-refusal chain may take: 53,983 measured
/// at 0.69.0. Before it, a branch cut by a contradiction cost no budget, a contradiction with the
/// target guard was seen only after every earlier refusal had been refuted both ways, and the
/// count grew several times over per refusal added (about four million at twelve).
const CHAIN_WORK: usize = 80_000;

/// Sixteen of the seventeen branches are reachable; the other is refused naming itself.
#[test]
fn a_first_match_chain_of_sixteen_refusals_is_synthesized_within_a_work_bound() {
    let guards = first(16);
    assert_eq!(reachable(&guards).len(), 16);
    let ir = chain_model(&guards);
    let (synthesis, steps) = directed_steps(|| synthesize(&ir));
    assert_covered(&guards, &synthesis);
    assert!(
        steps <= CHAIN_WORK,
        "synthesis took {steps} guard-directed steps, past the bound of {CHAIN_WORK}"
    );
}

/// The adopter's thirteen-branch shape: `refuse-10` (`b2 == false`, `e1 == B`, `b4 == false`) is
/// reached by `e1=B, e2=Z, b1=true` and every other Boolean `false`, which refutes the nine
/// refusals before it. It was refused as no candidate of those tried satisfying it outside them.
#[test]
fn a_reachable_branch_late_in_a_chain_of_twelve_refusals_has_its_scenario() {
    let guards = first(12);
    assert!(reachable(&guards).contains("refuse-10"));
    let ir = chain_model(&guards);
    assert_covered(&guards, &synthesize(&ir));
}

/// A refusal every input of which an earlier refusal claims is refused, under its own name.
#[test]
fn a_refusal_shadowed_by_earlier_ones_in_the_chain_is_refused_naming_itself() {
    let mut guards = first(6);
    // `e2` and `b3` avoid the #702 refusal: beside `refuse-2` the guards cross 96 assignments, past
    // the finite prover's 64, so validation stays silent and synthesis names the shadow.
    guards.insert(
        3,
        (
            "shadowed".to_owned(),
            vec![
                ("e1", "D"),
                ("b2", "false"),
                ("b6", "true"),
                ("e2", "Y"),
                ("b3", "true"),
            ],
        ),
    );
    assert!(!reachable(&guards).contains("shadowed"));
    let ir = chain_model(&guards);
    assert_covered(&guards, &synthesize(&ir));
}

/// A chain over one input `k` of type `shared` that every guard reads: four core refusals over
/// `c1`, `c2`, `s` that nothing refutes together while `s` is false, ten decoys `d<i> and x<i>`,
/// `s and z`, then the target `t`. The target also requires `target_k` (YAML), every other guard
/// `other` (YAML).
///
/// `t`, `s` and `k = witness_k` with every other Boolean (`m` among them) false reach the target.
/// Searching for it walks every way of refuting the decoys under `s` false before it tries `z`,
/// unless the guards `other` joins are searched apart.
fn shared_input_model(shared: &str, target_k: &str, other: &str) -> EssIr {
    let mut booleans: Vec<String> = Vec::new();
    for at in 1..=10 {
        booleans.push(format!("d{at}"));
        booleans.push(format!("x{at}"));
    }
    booleans.extend(["c1", "c2", "s", "z", "t", "m"].map(str::to_owned));
    let atom = |input: &str| format!("\"{input} == true\"");
    let negated = |input: &str| format!("{{not: \"{input} == true\"}}");
    let other = other.to_owned();
    let mut guards: Vec<Vec<String>> = Vec::new();
    for (c1, c2) in [(false, false), (false, true), (true, false), (true, true)] {
        let literal = |input: &str, held: bool| if held { atom(input) } else { negated(input) };
        guards.push(vec![
            literal("c1", c1),
            literal("c2", c2),
            negated("s"),
            other.clone(),
        ]);
    }
    for at in 1..=10 {
        guards.push(vec![
            atom(&format!("d{at}")),
            atom(&format!("x{at}")),
            other.clone(),
        ]);
    }
    guards.push(vec![atom("s"), atom("z"), other]);
    guards.push(vec![atom("t"), target_k.to_owned()]);
    let mut text = String::from(
        "format: ess/19\nsystem: demo\nversion: v1\ndomain: demo.shared\ntypes:\n  - {name: demo.shared.First, kind: enum, variants: [A, B, C, D]}\nentities:\n  - name: demo.shared.Request\n    identity: {name: request_id, type: Uuid}\n    fields:\n      - {name: note, type: Boolean}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nevents:\n  - name: demo.shared.Submitted\n    fields: [{name: request_id, type: Uuid}]\nerrors:\n",
    );
    for at in 0..guards.len() {
        let _ = writeln!(
            text,
            "  - {{name: demo.shared.Refused{at}, summary: Refused., fields: []}}"
        );
    }
    text.push_str("commands:\n  - name: demo.shared.Submit\n    input:\n");
    for boolean in &booleans {
        let _ = writeln!(text, "      - {{name: {boolean}, type: Boolean}}");
    }
    let _ = write!(text, "      - {{name: k, type: {shared}}}\n    outcomes:\n");
    let last = guards.len() - 1;
    for (at, guard) in guards.iter().enumerate() {
        let name = if at == last {
            "target".to_owned()
        } else {
            format!("refuse-{at}")
        };
        let _ = writeln!(
            text,
            "      - name: {name}\n        when: {{all: [{}]}}\n        error: demo.shared.Refused{at}",
            guard.join(", ")
        );
    }
    text.push_str("      - name: created\n        creates: demo.shared.Request\n        instance: request_id\n        sets: {note: true}\n        emits: [demo.shared.Submitted]\n        payload:\n          demo.shared.Submitted: {request_id: {generated: true}}\n");
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("shared.yaml".to_owned(), text.clone());
    mutate::compile(vec![(Source::new("shared.yaml"), raw)], &texts).unwrap_or_else(|stillborn| {
        panic!(
            "the model compiles: {} {}\n{text}",
            stillborn.code, stillborn.cause
        )
    })
}

/// The target of [`shared_input_model`] is reached by its witness and has its scenario.
fn assert_shared_target_synthesized(ir: &EssIr, witness_k: &Node) {
    let command = ir
        .commands()
        .get(&QualifiedName::new("demo.shared.Submit").expect("a valid name"))
        .expect("the command is declared");
    let mut witness: BTreeMap<String, Node> = BTreeMap::new();
    for input in &command.input {
        let name = input.name.clone();
        let value = if name == "k" {
            witness_k.clone()
        } else {
            Node::Bool(matches!(name.as_str(), "t" | "s"))
        };
        witness.insert(name, value);
    }
    let facts = flatten(ir, command, &witness).expect("the witness fits the command");
    let taken = command
        .outcomes
        .iter()
        .find(|outcome| {
            when(outcome).is_none_or(|guard| facts.decide(guard) == Decision::Satisfied)
        })
        .map(|outcome| outcome.name.to_string());
    assert_eq!(
        taken.as_deref(),
        Some("target"),
        "the witness reaches the target"
    );
    let (synthesis, steps) = directed_steps(|| synthesize(ir));
    let id = "demo.shared.Submit/outcome/target";
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|s| s.to_string() == id)
        })
        .map(|refusal| refusal.cause.to_string())
        .collect();
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "the reachable target has no scenario after {steps} steps: {refused:#?}"
    );
    assert!(refused.is_empty(), "{refused:#?}");
}

/// The shared input is a Boolean the target requires: the goal wants `k == true` itself.
#[test]
fn a_reachable_refusal_behind_guards_sharing_a_boolean_the_target_decides_has_its_scenario() {
    let ir = shared_input_model("Boolean", "\"k == true\"", "\"k == true\"");
    assert_shared_target_synthesized(&ir, &Node::Bool(true));
}

/// The shared input is an enum: the target wants `k == A`, which leaves `k != B` no other truth.
#[test]
fn a_reachable_refusal_behind_guards_sharing_an_enum_the_target_decides_has_its_scenario() {
    let ir = shared_input_model("demo.shared.First", "\"k == A\"", "\"k != B\"");
    assert_shared_target_synthesized(&ir, &Node::Text("A".to_owned()));
}

/// The shared input is an `Integer`: the target wants `k > 5`, which leaves `k > 0` no other
/// truth.
#[test]
fn a_reachable_refusal_behind_guards_sharing_an_integer_the_target_decides_has_its_scenario() {
    let ir = shared_input_model("Integer", "\"k > 5\"", "\"k > 0\"");
    assert_shared_target_synthesized(&ir, &Node::Number(ess_primitives::Number::from(6_i64)));
}

/// The shared input is one no goal decides: every guard but the target also reads `m`, in a
/// disjunction no value of `m` refutes (`m == true or not m == true`), so the search never wants a
/// truth of `m` and `m` keeps every guard in one part.
#[test]
fn a_reachable_refusal_behind_guards_sharing_a_boolean_no_goal_decides_has_its_scenario() {
    let ir = shared_input_model(
        "Boolean",
        "\"k == true\"",
        "{any: [\"m == true\", {not: \"m == true\"}]}",
    );
    assert_shared_target_synthesized(&ir, &Node::Bool(true));
}
