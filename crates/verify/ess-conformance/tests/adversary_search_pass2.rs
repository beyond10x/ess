//! Adversary pass 2 on `story:witness-search-beyond-64-candidates`: the per-field solver
//! (`witness::Directed`) added after pass 1. Its limits (64 solved candidates, 64 breakdowns per
//! goal, 1024 combinations per group), groups of fields compared with each other, and guards whose
//! connective rows outnumber the solved-candidate bound.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = r"
format: ess/13
system: demo
version: v1
domain: demo.gate
types:
  - {name: demo.gate.Kind, kind: enum, variants: [Open, Shut, Held]}
  - name: demo.gate.Row
    kind: struct
    fields:
      - {name: p, type: Integer}
      - {name: q, type: Integer}
      - {name: r, type: Integer}
      - {name: s, type: Integer}
      - {name: t, type: Integer}
      - {name: u, type: Integer}
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
      - {name: d, type: Integer}
      - {name: e, type: Integer}
      - {name: f, type: Integer}
      - {name: g, type: Integer}
      - {name: h, type: Integer}
      - {name: k, type: demo.gate.Kind}
      - {name: m, type: Decimal}
      - {name: x, type: Optional<Integer>}
      - {name: w, type: demo.gate.Row}
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

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled(text: &str) -> EssIr {
    let (files, texts) = documents(text);
    mutate::compile(files, &texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

/// The branch each `ExecuteCommand` of `demo.gate.Check` is expected to take, with its input.
fn expected_branches(synthesis: &Synthesis) -> Vec<(String, BTreeMap<String, Node>)> {
    let mut out = Vec::new();
    for scenario in synthesis.suite.scenarios.values() {
        let mut pending: Option<BTreeMap<String, Node>> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "demo.gate.Check" =>
                {
                    pending = Some(
                        input
                            .iter()
                            .filter_map(|(field, value)| {
                                value
                                    .as_literal()
                                    .cloned()
                                    .map(|node| (field.clone(), node))
                            })
                            .collect(),
                    );
                }
                ScenarioStep::ExpectOutcome { outcome } => {
                    if let Some(input) = pending.take() {
                        let branch = outcome
                            .to_string()
                            .rsplit('/')
                            .next()
                            .expect("an outcome ref names its branch")
                            .to_owned();
                        out.push((branch, input));
                    }
                }
                _ => {}
            }
        }
    }
    out
}

fn assert_refused_is_witnessed(guard: &str) {
    let text = MODEL.replace("GUARD", guard);
    let synthesis = synthesize(&compiled(&text));
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(
        expected_branches(&synthesis)
            .iter()
            .any(|(branch, _)| branch == "refused"),
        "the guarded branch of `{}` is witnessed: {refusals:#?}",
        guard.trim()
    );
}

/// The guarded branch of `guard` gets no scenario and is refused with `ESS-SYNTH-003`: a limit of
/// the solver (`story:witness-search-beyond-64-candidates`) that `limit` names.
fn assert_refused_is_refused(guard: &str, limit: &str) {
    let text = MODEL.replace("GUARD", guard);
    let synthesis = synthesize(&compiled(&text));
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(
        !expected_branches(&synthesis)
            .iter()
            .any(|(branch, _)| branch == "refused")
            && refusals.iter().any(|refusal| {
                refusal.contains("ESS-SYNTH-003") && refusal.contains("demo.gate.Check/refused")
            }),
        "story:witness-search-beyond-64-candidates: `{}` is refused with ESS-SYNTH-003 because {limit}; \
         if it is now witnessed, the limit moved and the docs must say so: {refusals:#?}",
        guard.trim()
    );
}

/// Every `GuardConnective` mutant of the guard is killed: some generated scenario expects a
/// branch the original guard disagrees with.
fn assert_connective_mutants_killed(guard: &str) {
    assert_connective_mutants_killed_in(MODEL, guard);
}

fn assert_connective_mutants_killed_in(model: &str, guard: &str) {
    let text = model.replace("GUARD", guard);
    let (files, texts) = documents(&text);
    let original = compiled(&text);
    let witnessed = expected_branches(&synthesize(&original));
    assert!(
        witnessed.iter().any(|(branch, _)| branch == "refused"),
        "the guarded branch of `{}` is witnessed",
        guard.trim()
    );
    let original_command = original
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.gate.Check")
        .expect("declared");
    let original_guard = original_command
        .outcomes
        .iter()
        .find_map(ess_conformance::when)
        .expect("guarded");
    let mut survivors = Vec::new();
    let mut tried = 0;
    for mutant in mutate::mutants(&files, &[MutantClass::GuardConnective]) {
        let mutated = mutate::apply(&files, &mutant.mutation).expect("the site exists");
        let Ok(ir) = mutate::compile(mutated, &texts) else {
            continue;
        };
        tried += 1;
        let killed = expected_branches(&synthesize(&ir))
            .into_iter()
            .any(|(branch, input)| {
                let facts =
                    ess_conformance::flatten(&original, original_command, &input).expect("fits");
                let satisfied =
                    facts.decide(original_guard) == ess_conformance::Decision::Satisfied;
                satisfied != (branch == "refused")
            });
        if !killed {
            survivors.push(mutant.id.clone());
        }
    }
    assert!(tried > 0, "some connective mutant compiles");
    assert!(
        survivors.is_empty(),
        "surviving connective mutants of `{}` ({} tried): {survivors:#?}",
        guard.trim(),
        tried
    );
}

/// Four two-field disjunctions under one conjunction. The solver's goals are the guard, then each
/// connective's rows outermost first, three contexts each: well over 64 distinct solutions, so
/// the rows of the last disjunction fall past the solved-candidate bound.
#[test]
fn adversary_search_pass2_four_disjunctions_connective_mutants_are_killed() {
    assert_connective_mutants_killed(
        "          all:\n            - any: [a > 10, b > 10]\n            - any: [c > 10, d > 10]\n            - any: [e > 10, f > 10]\n            - any: [g > 10, h > 10]",
    );
}

/// Three disjunctions, mixed types: an enum, a Decimal and an Optional among them.
#[test]
fn adversary_search_pass2_mixed_type_disjunctions_connective_mutants_are_killed() {
    assert_connective_mutants_killed(
        "          all:\n            - any: [k == Held, m < -5.5]\n            - any: [x > 10, a > 10]\n            - any: [b > 10, c > 10]",
    );
}

/// A chain of fields compared with each other forms one group. A field compared only with other
/// fields writes no literal, so its alternatives are the base, 0 and -1; four fields in a strict
/// chain need four distinct values and three are all there are. Refused today, by that limit.
#[test]
fn adversary_search_pass2_a_chain_of_four_compared_fields_is_refused() {
    assert_refused_is_refused(
        "          all: [w.p > w.q, w.q > w.r, w.r > w.s]",
        "a field compared only with other fields gets base, 0 and -1",
    );
}

/// A compared pair beside literals on one of its own fields and three more fields.
#[test]
fn adversary_search_pass2_a_compared_pair_with_literals_beside_three_fields_is_witnessed() {
    assert_refused_is_witnessed("          all: [w.p > w.q, w.p > 10, c > 10, d > 10, e > 10]");
}

/// A goal gives up after 64 breakdowns, and the search takes each disjunction's first child first.
/// Eight disjunctions whose first children all contradict `a < 5` leave the one satisfying
/// breakdown (every second child) at position 256, and each connective row still has 2^7 = 128
/// breakdowns. (Seven fit: a row pins one disjunction, leaving exactly 64.) Refused today, by that
/// limit.
#[test]
fn adversary_search_pass2_eight_disjunctions_whose_first_children_contradict_are_refused() {
    assert_refused_is_refused(
        "          all:\n            - a < 5\n            - any: [a > 11, b > 10]\n            - any: [a > 12, c > 10]\n            - any: [a > 13, d > 10]\n            - any: [a > 14, e > 10]\n            - any: [a > 15, f > 10]\n            - any: [a > 16, g > 10]\n            - any: [a > 17, h > 10]\n            - any: [a > 18, m > 10.5]",
        "64 breakdowns per goal",
    );
}

/// Five disjunctions of mixed types under one conjunction.
#[test]
fn adversary_search_pass2_five_disjunctions_connective_mutants_are_killed() {
    assert_connective_mutants_killed(
        "          all:\n            - any: [a > 10, b > 10]\n            - any: [c > 10, d > 10]\n            - any: [e > 10, f > 10]\n            - any: [g > 10, h > 10]\n            - any: [m > 10.5, k == Held]",
    );
}

/// The same four disjunctions in a command that also copies an unread optional input into a
/// field with a presence policy: every solved candidate is appended as a pair (absent, then
/// filled), and the appended candidates are bounded by count of inputs, not of solutions.
#[test]
fn adversary_search_pass2_four_disjunctions_beside_a_presence_policy_mutants_are_killed() {
    let with_presence = MODEL
        .replace("format: ess/13", "format: ess/15")
        .replace(
            "      - {name: a, type: Integer}\nerrors:",
            "      - {name: a, type: Integer}\n      - {name: note, type: Optional<String>, presence: null_when_absent}\nerrors:",
        )
        .replace(
            "      - {name: x, type: Optional<Integer>}\n",
            "      - {name: x, type: Optional<Integer>}\n      - {name: note, type: Optional<String>}\n",
        )
        .replace(
            "demo.gate.Passed: {a: input.a}",
            "demo.gate.Passed: {a: input.a, note: input.note}",
        );
    assert_ne!(with_presence, MODEL);
    assert_connective_mutants_killed_in(
        &with_presence,
        "          all:\n            - any: [a > 10, b > 10]\n            - any: [c > 10, d > 10]\n            - any: [e > 10, f > 10]\n            - any: [g > 10, h > 10]",
    );
}

fn with_tags() -> String {
    MODEL.replace(
        "      - {name: x, type: Optional<Integer>}\n",
        "      - {name: x, type: Optional<Integer>}\n      - {name: tags, type: List<String>}\n",
    )
}

/// A quantifier is one atom to the solver, read through the guards as written, while its list's
/// ladders come from the rebound bodies (`tags.0`, `tags.count`).
#[test]
fn adversary_search_pass2_an_exists_beside_three_fields_is_witnessed() {
    let model = with_tags();
    let text = model.replace(
        "GUARD",
        "          all:\n            - exists: {in: tags, as: t, that: t == vip}\n            - a > 10\n            - b > 10\n            - c > 10",
    );
    let synthesis = synthesize(&compiled(&text));
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert!(
        expected_branches(&synthesis)
            .iter()
            .any(|(branch, _)| branch == "refused"),
        "the guarded branch is witnessed: {refusals:#?}"
    );
}

/// A `forall` refuted by one element and satisfied by `[]`, inside a disjunction.
#[test]
fn adversary_search_pass2_a_forall_in_a_disjunction_mutants_are_killed() {
    assert_connective_mutants_killed_in(
        &with_tags(),
        "          all:\n            - any:\n                - forall: {in: tags, as: t, that: t != spam}\n                - a > 10\n            - b > 10\n            - c > 10",
    );
}

/// A negated disjunction beside two more fields: the solver must carry the negation down to the
/// atoms (`a <= 10` and `b <= 10` both refuted), past the walk's reach.
#[test]
fn adversary_search_pass2_a_negated_disjunction_beside_two_fields_is_witnessed() {
    assert_refused_is_witnessed(
        "          all:\n            - not: {any: [a <= 10, b <= 10]}\n            - c > 10\n            - d > 10\n            - e > 10",
    );
}

/// The same negation, its connective mutants killed.
#[test]
fn adversary_search_pass2_a_negated_disjunction_mutants_are_killed() {
    assert_connective_mutants_killed(
        "          all:\n            - not: {any: [a <= 10, b <= 10]}\n            - any: [c > 10, d > 10]\n            - e > 10",
    );
}
