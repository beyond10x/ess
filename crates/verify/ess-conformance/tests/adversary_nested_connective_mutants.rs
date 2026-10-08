//! Adversary pass on `story:witness-nested-connectives-and-decide-optional-presence-equivalence`
//! (<https://github.com/beyond10x/ess/issues/501>): deeper nesting, `not:` around a nested
//! connective, a sibling that shields the default's rows, presence guards that do overlap, and the
//! determinism and size of a synthesized suite for a wide guard.
#![allow(
    clippy::needless_raw_string_hashes,
    clippy::format_collect,
    clippy::format_push_string
)]

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{flatten, when, Decision};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn parsed(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled(text: &str) -> EssIr {
    let (files, texts) = parsed(text);
    mutate::compile(files, &texts).expect("the fixture compiles")
}

fn audit(text: &str) -> MutationReport {
    let (files, texts) = parsed(text);
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(&files, &texts, MutantClass::ALL, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn verdicts(report: &MutationReport) -> BTreeMap<String, (Verdict, String)> {
    report
        .mutants
        .iter()
        .map(|entry| (entry.id.clone(), (entry.verdict, entry.change.clone())))
        .collect()
}

/// Every `guard-connective` mutant, at least `at_least` of them, is killed.
fn connectives_killed(report: &MutationReport, at_least: usize) {
    let connectives: Vec<_> = report
        .mutants
        .iter()
        .filter(|entry| entry.class == MutantClass::GuardConnective)
        .collect();
    assert!(
        connectives.len() >= at_least,
        "expected at least {at_least} connective mutants: {:#?}",
        verdicts(report)
    );
    let alive: Vec<String> = connectives
        .iter()
        .filter(|entry| entry.verdict != Verdict::Killed)
        .map(|entry| format!("{} {:?}: {}", entry.id, entry.verdict, entry.change))
        .collect();
    assert!(alive.is_empty(), "not killed: {alive:#?}");
}

fn command<'ir>(ir: &'ir EssIr, name: &str) -> &'ir ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new(name).expect("a valid name"))
        .expect("the command is declared")
}

/// The branch the specification selects for `input`, by declared precedence.
fn selected(ir: &EssIr, name: &str, input: &BTreeMap<String, Node>) -> String {
    let command = command(ir, name);
    let facts = flatten(ir, command, input).expect("a synthesised input fits its command");
    for outcome in &command.outcomes {
        if outcome.test_strategy == TestStrategy::ConstructInput {
            let guard = when(outcome).expect("a constructed branch has a guard");
            if facts.decide(guard) == Decision::Satisfied {
                return outcome.name.to_string();
            }
        }
    }
    command
        .outcomes
        .iter()
        .find(|outcome| outcome.test_strategy == TestStrategy::DefaultBranch)
        .map(|outcome| outcome.name.to_string())
        .expect("a default")
}

/// Every single-command invocation of the suite: (scenario, expected branch, input).
fn sent(synthesis: &Synthesis) -> Vec<(String, String, BTreeMap<String, Node>)> {
    let mut out = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut pending: Option<BTreeMap<String, Node>> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { input, .. } => {
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
                        out.push((id.to_string(), branch, input));
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// Every scenario expects the branch the specification selects for what it sends.
fn consistent(text: &str, name: &str) {
    let ir = compiled(text);
    let synthesis = synthesize(&ir);
    let wrong: Vec<String> = sent(&synthesis)
        .into_iter()
        .filter(|(_, branch, input)| &selected(&ir, name, input) != branch)
        .map(|(id, branch, input)| format!("{id} expects {branch} for {input:?}"))
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

fn spec(command: &str, inputs: &[&str], outcomes: &str) -> String {
    let (domain, verb) = command.rsplit_once('.').expect("qualified");
    let fields: String = inputs
        .iter()
        .map(|field| format!("      - {field}\n"))
        .collect();
    format!(
        r#"format: ess/23
system: adv
version: v1
domain: {domain}

components:
  - component: adv-server
    owns:
      domains: [{domain}]
    accepts:
      commands: [{command}]
    reached_by: network

errors:
  - name: {domain}.First
  - name: {domain}.Second

commands:
  - name: {domain}.{verb}
    input:
{fields}    outcomes:
{outcomes}"#
    )
}

const BOOLS: &[&str] = &[
    "{name: a, type: Boolean}",
    "{name: b, type: Boolean}",
    "{name: c, type: Boolean}",
    "{name: d, type: Boolean}",
    "{name: p, type: Boolean}",
];

/// `all` in `any` in `all`, three levels.
fn all_any_all() -> String {
    spec(
        "adv.x.Run",
        BOOLS,
        r#"      - name: first
        when:
          all:
            - any:
                - all: ["a == true", "b == true"]
                - "c == true"
            - "d == true"
        error: adv.x.First
      - name: done
        accepts: nothing
"#,
    )
}

#[test]
fn adversary_all_in_any_in_all_kills_every_connective_mutant() {
    let text = all_any_all();
    consistent(&text, "adv.x.Run");
    connectives_killed(&audit(&text), 3);
}

/// `not:` around `any` holding an `all`, under a top-level `any`.
fn not_around_nested() -> String {
    spec(
        "adv.x.Run",
        BOOLS,
        r#"      - name: first
        when:
          any:
            - not:
                any:
                  - all: ["a == true", "b == true"]
                  - "c == true"
            - "d == true"
        error: adv.x.First
      - name: done
        accepts: nothing
"#,
    )
}

#[test]
fn adversary_not_around_a_nested_connective_kills_every_connective_mutant() {
    let text = not_around_nested();
    consistent(&text, "adv.x.Run");
    connectives_killed(&audit(&text), 3);
}

/// Mixed polarity: `not:` around an `all` holding an `any`, as one conjunct of a top `all`.
fn mixed_polarity() -> String {
    spec(
        "adv.x.Run",
        BOOLS,
        r#"      - name: first
        when:
          all:
            - not:
                all:
                  - any: ["a == true", "b == true"]
                  - "c == true"
            - "d == true"
        error: adv.x.First
      - name: done
        accepts: nothing
"#,
    )
}

#[test]
fn adversary_mixed_polarity_kills_every_connective_mutant() {
    let text = mixed_polarity();
    consistent(&text, "adv.x.Run");
    connectives_killed(&audit(&text), 3);
}

/// A nested `all` whose default-side rows a later guarded sibling catches: every input where
/// `all: [a, b]` alone refuses `first` while `p` holds and `c` fails is `second`'s, so no row
/// reaches the default. The connective mutant `any: [a, b]` still answers `first` where the
/// specification answers `second` (a = true, b = false, c = false, p = true).
fn shielded_default() -> String {
    spec(
        "adv.x.Run",
        BOOLS,
        r#"      - name: first
        when:
          all:
            - "p == true"
            - any:
                - all: ["a == true", "b == true"]
                - "c == true"
        error: adv.x.First
      - name: second
        when:
          any: ["a == false", "b == false"]
        error: adv.x.Second
      - name: done
        accepts: nothing
"#,
    )
}

#[test]
fn adversary_a_nested_all_shielded_by_a_later_sibling_is_still_killed() {
    let text = shielded_default();
    consistent(&text, "adv.x.Run");
    connectives_killed(&audit(&text), 2);
}

/// A guard of four disjuncts of three conjuncts each.
fn wide_of(groups: usize, members: usize) -> String {
    let inputs: Vec<String> = (0..groups * members)
        .map(|index| format!("{{name: f{index}, type: Boolean}}"))
        .collect();
    let inputs: Vec<&str> = inputs.iter().map(String::as_str).collect();
    let disjuncts: String = (0..groups)
        .map(|group| {
            let conjuncts: Vec<String> = (0..members)
                .map(|member| format!("\"f{} == true\"", group * members + member))
                .collect();
            format!("            - all: [{}]\n", conjuncts.join(", "))
        })
        .collect();
    spec(
        "adv.x.Run",
        &inputs,
        &format!(
            r#"      - name: first
        when:
          any:
{disjuncts}        error: adv.x.First
      - name: done
        accepts: nothing
"#
        ),
    )
}

#[test]
fn adversary_a_wide_guard_is_deterministic_bounded_and_fully_killed() {
    let text = wide_of(4, 3);
    let ir = compiled(&text);
    let once = serde_json::to_string(&synthesize(&ir).suite).expect("serializes");
    let twice = serde_json::to_string(&synthesize(&compiled(&text)).suite).expect("serializes");
    assert_eq!(once, twice, "the suite bytes differ between two runs");
    let synthesis = synthesize(&ir);
    let count = synthesis.suite.scenarios.len();
    let invocations = sent(&synthesis).len();
    eprintln!("wide guard: {count} scenarios, {invocations} invocations");
    // Two plain witnesses, one row per disjunct and one per conjunct: 2 + 4 + 12.
    assert!(
        invocations <= 18,
        "{invocations} invocations for a 4x3 guard"
    );
    consistent(&text, "adv.x.Run");
    connectives_killed(&audit(&text), 5);
}

/// The shapes the unit's own fixtures stop short of: `any` of `all`s, each witnessed per child
/// "at any depth" (`website/docs/reference/predicates.md`), whatever the width.
#[test]
fn adversary_every_any_of_alls_shape_up_to_four_by_three_kills_its_connectives() {
    let mut alive = Vec::new();
    for groups in 2..=4 {
        for members in 2..=3 {
            let report = audit(&wide_of(groups, members));
            for entry in &report.mutants {
                if entry.class == MutantClass::GuardConnective && entry.verdict != Verdict::Killed {
                    alive.push(format!(
                        "{groups}x{members} {} {:?}",
                        entry.id, entry.verdict
                    ));
                }
            }
        }
    }
    assert!(alive.is_empty(), "not killed: {alive:#?}");
}

/// The issue's reproduction with a fourth optional place: six pairs over four `Optional` inputs.
#[test]
fn adversary_the_issue_reproduction_with_four_places_kills_its_connectives() {
    let places = ["header", "body", "query", "cookie"];
    let inputs: Vec<String> = places
        .iter()
        .map(|place| format!("{{name: {place}, type: Optional<String>}}"))
        .collect();
    let inputs: Vec<&str> = inputs.iter().map(String::as_str).collect();
    let mut pairs = String::new();
    for (index, one) in places.iter().enumerate() {
        for other in &places[index + 1..] {
            pairs.push_str(&format!(
                "            - all: [{{{one}: {{defined: true}}}}, {{{other}: {{defined: true}}}}]\n"
            ));
        }
    }
    let text = spec(
        "adv.x.Run",
        &inputs,
        &format!(
            r#"      - name: first
        when:
          any:
{pairs}        error: adv.x.First
      - name: done
        accepts: nothing
"#
        ),
    );
    consistent(&text, "adv.x.Run");
    connectives_killed(&audit(&text), 7);
}

fn swap_verdict(text: &str, id: &str) -> (Verdict, Option<String>) {
    let report = audit(text);
    let entry = report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no {id} in {:#?}", verdicts(&report)));
    (entry.verdict, entry.unsatisfiable_guard.clone())
}

fn two_guards(inputs: &[&str], first: &str, second: &str) -> String {
    spec(
        "adv.x.Run",
        inputs,
        &format!(
            r#"      - name: first
        when: {first}
        error: adv.x.First
      - name: second
        when: {second}
        error: adv.x.Second
      - name: done
        accepts: nothing
"#
        ),
    )
}

const SWAP: &str = "precedence-swap/adv.x.Run/first/second";

/// Two guards some input satisfies together are never called `equivalent`.
#[test]
fn adversary_overlapping_presence_and_equality_swaps_are_never_equivalent() {
    let optional = &[
        "{name: x, type: Optional<String>}",
        "{name: n, type: Optional<Integer>}",
        "{name: flag, type: Optional<Boolean>}",
        "{name: draft, type: Boolean}",
    ];
    let cases = [
        // `x == "a"` holds only where `x` is defined.
        (r#"{x: {defined: true}}"#, r#""x == \"a\"""#),
        // An absent `x` is not "a": a target may well take both.
        (r#"{x: {defined: false}}"#, r#""x != \"a\"""#),
        // Presence and inequality on the same field, beside equality on it.
        (
            r#"{all: [{x: {defined: true}}, "x != \"a\""]}"#,
            r#""x == \"b\"""#,
        ),
        // Truth of an optional boolean, beside its presence.
        (r#"{flag: {defined: true}}"#, r#""flag == true""#),
        // A numeric bound beside presence.
        (r#"{n: {defined: true}}"#, r#""n > 5""#),
        // The same input in both branches, overlapping where draft holds.
        (
            r#"{all: [{x: {defined: true}}, "draft == true"]}"#,
            r#"{any: [{x: {defined: false}}, "draft == true"]}"#,
        ),
    ];
    let mut wrong = Vec::new();
    for (first, second) in cases {
        let text = two_guards(optional, first, second);
        let (verdict, overlap) = swap_verdict(&text, SWAP);
        eprintln!("{first} / {second}: {verdict:?}");
        if verdict == Verdict::Equivalent {
            wrong.push(format!("{first} / {second}: equivalent over {overlap:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}
