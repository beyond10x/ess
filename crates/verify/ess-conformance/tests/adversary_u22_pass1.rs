//! Adversary pass 1 on `story:a-first-match-chain-is-synthesized-within-a-work-bound`.
//!
//! Every case holds synthesis to the story's acceptance: each branch of a first-match chain that
//! some input reaches has its scenario. Reachability is decided here by trying every input of a
//! small finite input space through `flatten` and `decide`, not by the search under test.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::mutate;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::witness::directed_steps;
use ess_conformance::{flatten, when, Decision};
use ess_domain::name::QualifiedName;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;

/// One input: its name and the values it may take (`Bool` or an enum's variants).
#[derive(Clone)]
enum Kind {
    Boolean,
    Enum(&'static str, Vec<&'static str>),
}

struct Chain {
    inputs: Vec<(String, Kind)>,
    /// Refusals in declaration order: name and the YAML of its `when:`.
    refusals: Vec<(String, String)>,
}

fn error_name(at: usize) -> String {
    format!("Refused{at}")
}

fn model(chain: &Chain) -> EssIr {
    let mut text =
        String::from("format: ess/19\nsystem: demo\nversion: v1\ndomain: demo.adv\ntypes:\n");
    let mut enums = BTreeSet::new();
    for (_, kind) in &chain.inputs {
        if let Kind::Enum(name, variants) = kind {
            if enums.insert(*name) {
                writeln!(
                    text,
                    "  - {{name: demo.adv.{name}, kind: enum, variants: [{}]}}",
                    variants.join(", ")
                )
                .unwrap();
            }
        }
    }
    text.push_str("entities:\n  - name: demo.adv.Request\n    identity: {name: request_id, type: Uuid}\n    fields:\n      - {name: note, type: Boolean}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nevents:\n  - name: demo.adv.Submitted\n    fields: [{name: request_id, type: Uuid}]\nerrors:\n");
    for at in 0..chain.refusals.len() {
        writeln!(
            text,
            "  - {{name: demo.adv.{}, summary: Refused., fields: []}}",
            error_name(at)
        )
        .unwrap();
    }
    text.push_str("commands:\n  - name: demo.adv.Submit\n    input:\n");
    for (name, kind) in &chain.inputs {
        let type_ref = match kind {
            Kind::Boolean => "Boolean".to_owned(),
            Kind::Enum(type_name, _) => format!("demo.adv.{type_name}"),
        };
        writeln!(text, "      - {{name: {name}, type: {type_ref}}}").unwrap();
    }
    text.push_str("    outcomes:\n");
    for (at, (name, guard)) in chain.refusals.iter().enumerate() {
        writeln!(
            text,
            "      - name: {name}\n        when: {guard}\n        error: demo.adv.{}",
            error_name(at)
        )
        .unwrap();
    }
    text.push_str("      - name: created\n        creates: demo.adv.Request\n        instance: request_id\n        sets: {note: true}\n        emits: [demo.adv.Submitted]\n        payload:\n          demo.adv.Submitted: {request_id: {generated: true}}\n");
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("adv.yaml".to_owned(), text.clone());
    mutate::compile(vec![(Source::new("adv.yaml"), raw)], &texts).unwrap_or_else(|stillborn| {
        panic!(
            "the model compiles: {} {}\n{text}",
            stillborn.code, stillborn.cause
        )
    })
}

fn command(ir: &EssIr) -> &ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new("demo.adv.Submit").expect("a valid name"))
        .expect("the command is declared")
}

/// Every input of the chain's finite space.
fn every_input(chain: &Chain) -> Vec<BTreeMap<String, Node>> {
    let mut all = vec![BTreeMap::new()];
    for (name, kind) in &chain.inputs {
        let values: Vec<Node> = match kind {
            Kind::Boolean => vec![Node::Bool(false), Node::Bool(true)],
            Kind::Enum(_, variants) => variants
                .iter()
                .map(|v| Node::Text((*v).to_owned()))
                .collect(),
        };
        let mut next = Vec::with_capacity(all.len() * values.len());
        for input in &all {
            for value in &values {
                let mut grown: BTreeMap<String, Node> = input.clone();
                grown.insert(name.clone(), value.clone());
                next.push(grown);
            }
        }
        all = next;
    }
    all
}

/// The branches some input selects, first match first, decided by the crate's own evaluator.
fn reachable(ir: &EssIr, chain: &Chain) -> BTreeSet<String> {
    let command = command(ir);
    let mut reached = BTreeSet::new();
    for input in every_input(chain) {
        let facts = flatten(ir, command, &input).expect("an input of the declared types");
        let taken = command
            .outcomes
            .iter()
            .find(|outcome| {
                when(outcome).is_some_and(|guard| facts.decide(guard) == Decision::Satisfied)
            })
            .map_or_else(|| "created".to_owned(), |outcome| outcome.name.to_string());
        reached.insert(taken);
    }
    reached
}

/// The branch `input` selects, first match first.
fn selected(ir: &EssIr, input: &BTreeMap<String, Node>) -> String {
    let command = command(ir);
    let facts = flatten(ir, command, input).expect("an input of the declared types");
    command
        .outcomes
        .iter()
        .find(|outcome| {
            when(outcome).is_some_and(|guard| facts.decide(guard) == Decision::Satisfied)
        })
        .map_or_else(|| "created".to_owned(), |outcome| outcome.name.to_string())
}

/// `t`, `s` and `c1` true, every other Boolean false.
fn deep_backtrack_witness(chain: &Chain) -> BTreeMap<String, Node> {
    chain
        .inputs
        .iter()
        .map(|(name, _)| {
            (
                name.clone(),
                Node::Bool(matches!(name.as_str(), "t" | "s" | "c1")),
            )
        })
        .collect()
}

/// The reachable branches with no scenario, each with what it was refused as.
fn missing(chain: &Chain, ir: &EssIr, synthesis: &Synthesis) -> Vec<(String, Vec<String>)> {
    missing_of(&reachable(ir, chain), synthesis)
}

/// Of `reached`, the branches with no scenario, each with what it was refused as.
fn missing_of(reached: &BTreeSet<String>, synthesis: &Synthesis) -> Vec<(String, Vec<String>)> {
    let mut missing = Vec::new();
    for name in reached {
        let id = format!("demo.adv.Submit/outcome/{name}");
        let present = synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id);
        if !present {
            let refused = synthesis
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
            missing.push((name.clone(), refused));
        }
    }
    missing
}

fn atom(input: &str) -> String {
    format!("\"{input} == true\"")
}

fn negated(input: &str) -> String {
    format!("{{not: \"{input} == true\"}}")
}

fn all(children: &[String]) -> String {
    format!("{{all: [{}]}}", children.join(", "))
}

/// A satisfiable chain whose target needs the search to undo its first choice after trying every
/// combination of ten independent choices below it.
///
/// Every Boolean is read through the one atom `x == true` (negated with `not:`), so the search
/// before this change saw every contradiction as two truths of one atom, charged no budget for it,
/// and its first complete assignment was the witness. Declared, top to bottom:
///
/// - four refusals over `c1`, `c2` and `s` that no input refutes together while `s` is false
///   (`c1 or c2 or s`, `c1 or not c2 or s`, ... as refuted conjunctions);
/// - ten refusals `d<i> and x<i>`, each refuted either way;
/// - `s and z`, refuted first by `s` false, which is the wrong choice;
/// - the target `t`.
///
/// `t, s, d* false, x* false, z false` reaches the target. The search branches on the last
/// refusal first, so it refutes `s and z` by `s`, then walks all 2^10 ways of refuting the ten
/// decoys, each failing in the core, before it tries `z`.
fn deep_backtrack_chain(decoys: usize) -> Chain {
    let mut inputs: Vec<(String, Kind)> = Vec::new();
    for at in 1..=decoys {
        inputs.push((format!("d{at}"), Kind::Boolean));
        inputs.push((format!("x{at}"), Kind::Boolean));
    }
    for name in ["c1", "c2", "s", "z", "t"] {
        inputs.push((name.to_owned(), Kind::Boolean));
    }
    let mut refusals = Vec::new();
    let core = [
        [negated("c1"), negated("c2"), negated("s")],
        [negated("c1"), atom("c2"), negated("s")],
        [atom("c1"), negated("c2"), negated("s")],
        [atom("c1"), atom("c2"), negated("s")],
    ];
    for (at, guard) in core.iter().enumerate() {
        refusals.push((format!("core-{at}"), all(guard)));
    }
    for at in 1..=decoys {
        refusals.push((
            format!("decoy-{at}"),
            all(&[atom(&format!("d{at}")), atom(&format!("x{at}"))]),
        ));
    }
    refusals.push(("wrong-first".to_owned(), all(&[atom("s"), atom("z")])));
    refusals.push(("target".to_owned(), atom("t")));
    Chain { inputs, refusals }
}

/// Finding 1 (attack 2): a reachable branch whose search needs more than `MAX_DIRECTED_STEPS`
/// (1,024) steps is refused. The search before this change had no step bound; its first complete
/// assignment on this chain is the witness, so it was synthesized.
#[test]
fn a_reachable_refusal_past_ten_independent_choices_and_a_refuted_core_has_its_scenario() {
    let chain = deep_backtrack_chain(10);
    let ir = model(&chain);
    let witness = deep_backtrack_witness(&chain);
    assert_eq!(
        selected(&ir, &witness),
        "target",
        "the witness reaches the target"
    );
    let (synthesis, steps) = directed_steps(|| synthesize(&ir));
    let missing = missing_of(&BTreeSet::from(["target".to_owned()]), &synthesis);
    assert!(
        missing.is_empty(),
        "reachable branches without a scenario after {steps} guard-directed steps: {missing:#?}"
    );
}

/// Control for the case above: with six decoys the backtracking fits under the step bound and the
/// same target is synthesized, so the bound, not the shape, is what refuses it at ten.
#[test]
fn control_the_same_chain_with_six_decoys_has_its_scenario() {
    let chain = deep_backtrack_chain(6);
    let ir = model(&chain);
    let witness = deep_backtrack_witness(&chain);
    assert_eq!(
        selected(&ir, &witness),
        "target",
        "the witness reaches the target"
    );
    let (synthesis, steps) = directed_steps(|| synthesize(&ir));
    let missing = missing_of(&BTreeSet::from(["target".to_owned()]), &synthesis);
    assert!(missing.is_empty(), "after {steps} steps: {missing:#?}");
}

/// Finding 2 (attack 2): where the work bound, not the absence of an input, ends the search, the
/// refusal says so. It says "no candidate of the N tried satisfies ...", which reads as a guard no
/// input meets and sends an adopter to rewrite a condition that is fine.
///
/// The ten-decoy chain this case first used is synthesized now, so it is cut short on a chain that
/// still meets the bound: every guard above the target also reads a seventeen-variant enum `k`
/// through `{any: [k == V0, ..., k == V16]}`, one value more than the search splits a shared input
/// into cases for (`adversary_u22_pass3.rs` holds the same chain at sixteen and seventeen). The
/// refusal must be there before what it says is checked.
#[test]
fn a_refusal_the_step_bound_cut_short_names_the_bound() {
    let chain = every_variant_glue_chain(17);
    let ir = model(&chain);
    let synthesis = synthesize(&ir);
    let id = "demo.adv.Submit/outcome/target";
    assert!(
        !synthesis
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "the target is synthesized, so this chain no longer meets the step bound"
    );
    let causes: Vec<String> = synthesis
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
        !causes.is_empty(),
        "the target is neither synthesized nor refused"
    );
    for cause in &causes {
        assert!(
            cause.contains("at its bound of 1024 steps"),
            "a refusal the step bound caused does not name it: {cause}"
        );
    }
}

const VARIANTS: &[&str] = &[
    "V0", "V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9", "V10", "V11", "V12", "V13", "V14",
    "V15", "V16", "V17", "V18", "V19",
];

/// Four cores over `c1`, `c2` and `s`, six decoys `d<i> and x<i>`, `s and z`, then the target
/// `t and not m`; every guard but the target also reads the enum `k` through a disjunction over all
/// `variants` of it, true for every `k`. `t` and `s` true, everything else false, reaches the target.
fn every_variant_glue_chain(variants: usize) -> Chain {
    let big: Vec<&'static str> = VARIANTS[..variants].to_vec();
    let glue = big
        .iter()
        .map(|v| format!("\"k == {v}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let glue = format!("{{any: [{glue}]}}");
    let mut inputs = Vec::new();
    for i in 1..=6 {
        inputs.push((format!("d{i}"), Kind::Boolean));
        inputs.push((format!("x{i}"), Kind::Boolean));
    }
    for name in ["c1", "c2", "s", "z", "t", "m"] {
        inputs.push((name.to_owned(), Kind::Boolean));
    }
    inputs.push(("k".to_owned(), Kind::Enum("Big", big)));
    let mut refusals = Vec::new();
    for (at, (c1, c2)) in [
        (negated("c1"), negated("c2")),
        (negated("c1"), atom("c2")),
        (atom("c1"), negated("c2")),
        (atom("c1"), atom("c2")),
    ]
    .into_iter()
    .enumerate()
    {
        refusals.push((
            format!("core-{at}"),
            all(&[c1, c2, negated("s"), glue.clone()]),
        ));
    }
    for i in 1..=6 {
        refusals.push((
            format!("decoy-{i}"),
            all(&[atom(&format!("d{i}")), atom(&format!("x{i}")), glue.clone()]),
        ));
    }
    refusals.push(("wrong-first".to_owned(), all(&[atom("s"), atom("z"), glue])));
    refusals.push(("target".to_owned(), all(&[atom("t"), negated("m")])));
    Chain { inputs, refusals }
}

/// splitmix64, so the chains below are the same on every host.
struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).expect("fits")
    }
}

/// The adopter's input shape: one four-valued enum, one three-valued enum, six Booleans.
fn adopter_inputs() -> Vec<(String, Kind)> {
    let mut inputs = vec![
        (
            "e1".to_owned(),
            Kind::Enum("First", vec!["A", "B", "C", "D"]),
        ),
        ("e2".to_owned(), Kind::Enum("Second", vec!["X", "Y", "Z"])),
    ];
    for at in 1..=6 {
        inputs.push((format!("b{at}"), Kind::Boolean));
    }
    inputs
}

/// A random literal over the adopter's inputs, in one of the forms an adopter writes.
fn literal(random: &mut Seeded, input: usize) -> String {
    match input {
        0 => {
            let v = ["A", "B", "C", "D"][random.below(4)];
            if random.below(4) == 0 {
                format!("{{not: \"e1 == {v}\"}}")
            } else {
                format!("\"e1 == {v}\"")
            }
        }
        1 => {
            let v = ["X", "Y", "Z"][random.below(3)];
            format!("\"e2 == {v}\"")
        }
        b => {
            let name = format!("b{}", b - 1);
            match random.below(3) {
                0 => format!("\"{name} == true\""),
                1 => format!("\"{name} == false\""),
                _ => format!("{{not: \"{name} == true\"}}"),
            }
        }
    }
}

/// A random guard: a conjunction of two to four literals over distinct inputs, sometimes with a
/// nested disjunction.
fn guard(random: &mut Seeded) -> String {
    let size = 2 + random.below(3);
    let mut picked: Vec<usize> = Vec::new();
    while picked.len() < size {
        let input = random.below(8);
        if !picked.contains(&input) {
            picked.push(input);
        }
    }
    let mut children: Vec<String> = picked.iter().map(|input| literal(random, *input)).collect();
    if random.below(3) == 0 {
        let a = random.below(8);
        let b = (a + 1 + random.below(7)) % 8;
        children.push(format!(
            "{{any: [{}, {}]}}",
            literal(random, a),
            literal(random, b)
        ));
    }
    all(&children)
}

fn seeded_chain(seed: u64, length: usize) -> Chain {
    let mut random = Seeded(seed);
    Chain {
        inputs: adopter_inputs(),
        refusals: (0..length)
            .map(|at| (format!("refuse-{at}"), guard(&mut random)))
            .collect(),
    }
}

/// Attacks 1 and 2 over the adopter's own input shape: sixteen-refusal chains drawn from fixed
/// seeds, with `not:` and nested `any:` beside `==`, each reachable branch held to its scenario.
#[test]
fn every_reachable_branch_of_seeded_sixteen_refusal_chains_over_the_adopter_shape_has_its_scenario()
{
    let mut failures = Vec::new();
    for seed in 0..24u64 {
        let chain = seeded_chain(seed, 16);
        let ir = model(&chain);
        let (synthesis, steps) = directed_steps(|| synthesize(&ir));
        let missing = missing(&chain, &ir, &synthesis);
        if !missing.is_empty() {
            failures.push((seed, steps, missing));
        }
    }
    assert!(failures.is_empty(), "seed, steps, missing: {failures:#?}");
}

/// The first seed above that fails, pinned: `refuse-14` of seed 22 is reached by some input and
/// refused as ESS-SYNTH-003.
#[test]
fn seed_22_reachable_refuse_14_over_the_adopter_shape_has_its_scenario() {
    let chain = seeded_chain(22, 16);
    let ir = model(&chain);
    let reached = reachable(&ir, &chain);
    let witness = every_input(&chain)
        .into_iter()
        .find(|input| selected(&ir, input) == "refuse-14");
    assert!(witness.is_some(), "refuse-14 is reachable");
    let (synthesis, steps) = directed_steps(|| synthesize(&ir));
    let missing = missing_of(&reached, &synthesis);
    let guards: Vec<String> = chain
        .refusals
        .iter()
        .map(|(name, guard)| format!("{name}: {guard}"))
        .collect();
    assert!(
        missing.is_empty(),
        "after {steps} steps, witness {witness:?}\nchain {guards:#?}\nmissing {missing:#?}"
    );
}

/// Seed 22 reduced greedily: eight refusals stay before the target, and removing any one of them
/// makes the target synthesized or unreachable. Four of them (`refuse-7`, `-8`, `-10`, `-12`) are
/// already refuted by the target's own `e2 == X`, yet each is still branched on, child by child.
/// `e1=B`, every Boolean false, `e2=X` reaches the target.
#[test]
fn a_reachable_refusal_after_eight_conjunctions_four_already_refuted_by_its_own_guard_has_its_scenario(
) {
    let refusals = [
        (
            "refuse-3",
            "{all: [\"b6 == false\", \"e1 == A\", {any: [\"b2 == false\", {not: \"e1 == A\"}]}]}",
        ),
        (
            "refuse-7",
            "{all: [\"b2 == true\", \"b5 == true\", \"e2 == Z\"]}",
        ),
        (
            "refuse-8",
            "{all: [\"e1 == B\", \"e2 == Y\", \"b6 == false\", \"b2 == false\"]}",
        ),
        (
            "refuse-9",
            "{all: [\"e1 == D\", {not: \"b1 == true\"}, {not: \"b6 == true\"}, \"b4 == true\"]}",
        ),
        (
            "refuse-10",
            "{all: [\"b2 == true\", {not: \"b5 == true\"}, {not: \"b4 == true\"}, \"e2 == Y\"]}",
        ),
        ("refuse-11", "{all: [{not: \"b5 == true\"}, \"e1 == C\"]}"),
        (
            "refuse-12",
            "{all: [\"b1 == true\", \"b5 == true\", \"e2 == Y\", {not: \"b3 == true\"}]}",
        ),
        (
            "refuse-13",
            "{all: [{not: \"e1 == A\"}, \"b5 == true\", {not: \"b4 == true\"}]}",
        ),
        (
            "refuse-14",
            "{all: [{not: \"b6 == true\"}, \"e2 == X\", {not: \"b2 == true\"}, \"b3 == false\"]}",
        ),
    ];
    let chain = Chain {
        inputs: adopter_inputs(),
        refusals: refusals
            .iter()
            .map(|(name, guard)| ((*name).to_owned(), (*guard).to_owned()))
            .collect(),
    };
    let ir = model(&chain);
    let reached = reachable(&ir, &chain);
    assert!(reached.contains("refuse-14"));
    let (synthesis, steps) = directed_steps(|| synthesize(&ir));
    let missing = missing_of(&reached, &synthesis);
    assert!(missing.is_empty(), "after {steps} steps: {missing:#?}");
}

/// A random conjunction in the adopter's own syntax: `input == value` atoms only, two to four of
/// them over distinct inputs, Booleans written `== true` or `== false`.
fn plain_guard(random: &mut Seeded) -> String {
    let size = 2 + random.below(3);
    let mut picked: Vec<usize> = Vec::new();
    while picked.len() < size {
        let input = random.below(8);
        if !picked.contains(&input) {
            picked.push(input);
        }
    }
    let children: Vec<String> = picked
        .iter()
        .map(|input| match input {
            0 => format!("\"e1 == {}\"", ["A", "B", "C", "D"][random.below(4)]),
            1 => format!("\"e2 == {}\"", ["X", "Y", "Z"][random.below(3)]),
            b => format!("\"b{} == {}\"", b - 1, random.below(2) == 1),
        })
        .collect();
    all(&children)
}

/// Acceptance, in the adopter's syntax: sixteen-refusal chains of plain conjunctions over the
/// adopter's input shape, from fixed seeds; every reachable branch has its scenario.
#[test]
fn every_reachable_branch_of_seeded_plain_sixteen_refusal_chains_has_its_scenario() {
    let mut failures = Vec::new();
    for seed in 100..148u64 {
        let mut random = Seeded(seed);
        let chain = Chain {
            inputs: adopter_inputs(),
            refusals: (0..16)
                .map(|at| (format!("refuse-{at}"), plain_guard(&mut random)))
                .collect(),
        };
        let ir = model(&chain);
        let (synthesis, steps) = directed_steps(|| synthesize(&ir));
        let missing = missing(&chain, &ir, &synthesis);
        if !missing.is_empty() {
            let guards: Vec<String> = chain
                .refusals
                .iter()
                .map(|(n, g)| format!("{n}: {g}"))
                .collect();
            failures.push((seed, steps, missing, guards));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failing seeds: {failures:#?}",
        failures.len()
    );
}

/// Attack 3: two runs of one chain give the same suite bytes.
#[test]
fn a_seeded_chain_synthesizes_the_same_suite_twice() {
    let chain = seeded_chain(7, 16);
    let ir = model(&chain);
    let first = serde_json::to_string(&synthesize(&ir).suite).expect("serializes");
    let second = serde_json::to_string(&synthesize(&ir).suite).expect("serializes");
    assert_eq!(first, second);
}
