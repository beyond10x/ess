//! Adversary pass 2 on `story:a-first-match-chain-is-synthesized-within-a-work-bound`.
//!
//! Attacks the mechanism pass 1's fixes added: goals split into parts over disjoint inputs
//! (`Directed::parts`), a refuted connective replaced by a child already decided (`entailed`), and
//! the `cut` a refusal now carries. Reachability is decided here by trying inputs through `flatten`
//! and `decide`, never by the search under test; an input counts as reaching a branch only where
//! every guard declared before it is decided `false` and its own `true`.

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
use ess_primitives::Number;

/// What one input may hold.
#[derive(Clone)]
enum Kind {
    Boolean,
    /// An enum type and its variants.
    Enum(&'static str, &'static [&'static str]),
    /// `Optional<Boolean>`: absent, `false` or `true`.
    MaybeBoolean,
    /// `Integer`, tried at these values.
    Integer(&'static [i64]),
    /// `demo.adv.Pair`, a struct of two `First` members `p` and `q`.
    Pair,
}

struct Chain {
    inputs: Vec<(String, Kind)>,
    /// Refusals in declaration order: name and the YAML of its `when:`.
    refusals: Vec<(String, String)>,
}

const FIRST: &[&str] = &["A", "B", "C", "D"];
const SECOND: &[&str] = &["X", "Y", "Z"];
const NUMBERS: &[i64] = &[-1, 0, 1, 2, 3, 4];

fn model_text(chain: &Chain) -> String {
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
    if chain
        .inputs
        .iter()
        .any(|(_, kind)| matches!(kind, Kind::Pair))
    {
        if enums.insert("First") {
            writeln!(
                text,
                "  - {{name: demo.adv.First, kind: enum, variants: [{}]}}",
                FIRST.join(", ")
            )
            .unwrap();
        }
        text.push_str("  - name: demo.adv.Pair\n    kind: struct\n    fields:\n      - {name: p, type: demo.adv.First}\n      - {name: q, type: demo.adv.First}\n");
    }
    text.push_str("entities:\n  - name: demo.adv.Request\n    identity: {name: request_id, type: Uuid}\n    fields:\n      - {name: note, type: Boolean}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nevents:\n  - name: demo.adv.Submitted\n    fields: [{name: request_id, type: Uuid}]\nerrors:\n");
    for at in 0..chain.refusals.len() {
        writeln!(
            text,
            "  - {{name: demo.adv.Refused{at}, summary: Refused., fields: []}}"
        )
        .unwrap();
    }
    text.push_str("commands:\n  - name: demo.adv.Submit\n    input:\n");
    for (name, kind) in &chain.inputs {
        let type_ref = match kind {
            Kind::Boolean => "Boolean".to_owned(),
            Kind::Enum(type_name, _) => format!("demo.adv.{type_name}"),
            Kind::MaybeBoolean => "Optional<Boolean>".to_owned(),
            Kind::Integer(_) => "Integer".to_owned(),
            Kind::Pair => "demo.adv.Pair".to_owned(),
        };
        writeln!(text, "      - {{name: {name}, type: {type_ref}}}").unwrap();
    }
    text.push_str("    outcomes:\n");
    for (at, (name, guard)) in chain.refusals.iter().enumerate() {
        writeln!(
            text,
            "      - name: {name}\n        when: {guard}\n        error: demo.adv.Refused{at}"
        )
        .unwrap();
    }
    text.push_str("      - name: created\n        creates: demo.adv.Request\n        instance: request_id\n        sets: {note: true}\n        emits: [demo.adv.Submitted]\n        payload:\n          demo.adv.Submitted: {request_id: {generated: true}}\n");
    text
}

fn model(chain: &Chain) -> EssIr {
    let text = model_text(chain);
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

/// Every input of the chain's space, an absent optional as a missing key.
fn every_input(chain: &Chain) -> Vec<BTreeMap<String, Node>> {
    let mut all = vec![BTreeMap::new()];
    for (name, kind) in &chain.inputs {
        let values: Vec<Option<Node>> = match kind {
            Kind::Boolean => vec![Some(Node::Bool(false)), Some(Node::Bool(true))],
            Kind::MaybeBoolean => vec![None, Some(Node::Bool(false)), Some(Node::Bool(true))],
            Kind::Enum(_, variants) => variants
                .iter()
                .map(|v| Some(Node::Text((*v).to_owned())))
                .collect(),
            Kind::Integer(values) => values.iter().map(|v| Some(serde_yaml_number(*v))).collect(),
            Kind::Pair => {
                let mut pairs = Vec::new();
                for p in FIRST {
                    for q in FIRST {
                        pairs.push(Some(Node::Map(BTreeMap::from([
                            ("p".to_owned(), Node::Text((*p).to_owned())),
                            ("q".to_owned(), Node::Text((*q).to_owned())),
                        ]))));
                    }
                }
                pairs
            }
        };
        let mut next = Vec::with_capacity(all.len() * values.len());
        for input in &all {
            for value in &values {
                let mut grown: BTreeMap<String, Node> = input.clone();
                if let Some(value) = value {
                    grown.insert(name.clone(), value.clone());
                }
                next.push(grown);
            }
        }
        all = next;
    }
    all
}

fn serde_yaml_number(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// The branch `input` selects where every guard before it is decided, `None` where an earlier
/// guard is unknown for it.
fn selected(ir: &EssIr, input: &BTreeMap<String, Node>) -> Option<String> {
    let command = command(ir);
    let facts = flatten(ir, command, input).ok()?;
    for outcome in &command.outcomes {
        let Some(guard) = when(outcome) else {
            return Some(outcome.name.to_string());
        };
        match facts.decide(guard) {
            Decision::Satisfied => return Some(outcome.name.to_string()),
            Decision::Refuted(_) => {}
            Decision::Unevaluable(_) => return None,
        }
    }
    Some("created".to_owned())
}

fn reachable(ir: &EssIr, chain: &Chain) -> BTreeMap<String, BTreeMap<String, Node>> {
    let mut reached = BTreeMap::new();
    for input in every_input(chain) {
        if let Some(name) = selected(ir, &input) {
            reached.entry(name).or_insert(input);
        }
    }
    reached
}

/// Of `reached`, the branches with no scenario, each with a witness and what it was refused as.
fn missing_of(
    reached: &BTreeMap<String, BTreeMap<String, Node>>,
    synthesis: &Synthesis,
) -> Vec<(String, String, Vec<String>)> {
    let mut missing = Vec::new();
    for (name, witness) in reached {
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
            missing.push((name.clone(), format!("{witness:?}"), refused));
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

/// Pass 1's deep-backtrack chain, with one Boolean `k` that every guard also requires.
///
/// Declared, top to bottom: four core refusals over `c1`, `c2`, `s` that no input refutes together
/// while `s` is false; `decoys` refusals `d<i> and x<i>`; `s and z`, refuted first by `s` false,
/// the wrong choice; the target `t`. Each also requires `k`. `t, k, s` true and every other
/// Boolean false reaches the target.
///
/// `k` is what an adopter's chain has everywhere: one input that most guards read. It joins every
/// guard into one part, so the split that keeps pass 1's chain under the step bound does not
/// apply, and refuting the target's predecessors walks every way of refuting the decoys under
/// `s` false before it tries `z`.
fn shared_flag_chain(decoys: usize) -> Chain {
    let mut inputs: Vec<(String, Kind)> = Vec::new();
    for at in 1..=decoys {
        inputs.push((format!("d{at}"), Kind::Boolean));
        inputs.push((format!("x{at}"), Kind::Boolean));
    }
    for name in ["c1", "c2", "s", "z", "t", "k"] {
        inputs.push((name.to_owned(), Kind::Boolean));
    }
    let mut refusals = Vec::new();
    let core = [
        [negated("c1"), negated("c2"), negated("s"), atom("k")],
        [negated("c1"), atom("c2"), negated("s"), atom("k")],
        [atom("c1"), negated("c2"), negated("s"), atom("k")],
        [atom("c1"), atom("c2"), negated("s"), atom("k")],
    ];
    for (at, guard) in core.iter().enumerate() {
        refusals.push((format!("core-{at}"), all(guard)));
    }
    for at in 1..=decoys {
        refusals.push((
            format!("decoy-{at}"),
            all(&[atom(&format!("d{at}")), atom(&format!("x{at}")), atom("k")]),
        ));
    }
    refusals.push((
        "wrong-first".to_owned(),
        all(&[atom("s"), atom("z"), atom("k")]),
    ));
    refusals.push(("target".to_owned(), all(&[atom("t"), atom("k")])));
    Chain { inputs, refusals }
}

/// `t`, `k` and `s` true, every other Boolean false.
fn shared_flag_witness(chain: &Chain) -> BTreeMap<String, Node> {
    chain
        .inputs
        .iter()
        .map(|(name, _)| {
            (
                name.clone(),
                Node::Bool(matches!(name.as_str(), "t" | "k" | "s")),
            )
        })
        .collect()
}

fn target_missing(synthesis: &Synthesis) -> Vec<String> {
    let id = "demo.adv.Submit/outcome/target";
    if synthesis
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
    {
        return Vec::new();
    }
    let mut causes: Vec<String> = synthesis
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
    if causes.is_empty() {
        causes.push("no scenario and no refusal".to_owned());
    }
    causes
}

/// Attack on `parts`: a reachable target behind ten decoys, the deep backtrack pass 1 found, with
/// one input every guard reads. Before this unit the search had no step bound and found it; the
/// split into parts is what kept pass 1's chain under the bound, and one shared input defeats it.
#[test]
fn a_reachable_refusal_behind_ten_decoys_sharing_one_input_has_its_scenario() {
    let chain = shared_flag_chain(10);
    let ir = model(&chain);
    assert_eq!(
        selected(&ir, &shared_flag_witness(&chain)).as_deref(),
        Some("target"),
        "the witness reaches the target"
    );
    let (synthesis, steps) = directed_steps(|| synthesize(&ir));
    let missing = target_missing(&synthesis);
    assert!(
        missing.is_empty(),
        "the reachable target has no scenario after {steps} guard-directed steps: {missing:#?}"
    );
}

/// Control: with three decoys the same shape fits under the bound and the target is synthesized,
/// so the step bound, not the shape, refuses it at ten.
#[test]
fn control_the_shared_input_chain_with_three_decoys_has_its_scenario() {
    let chain = shared_flag_chain(3);
    let ir = model(&chain);
    assert_eq!(
        selected(&ir, &shared_flag_witness(&chain)).as_deref(),
        Some("target")
    );
    let (synthesis, steps) = directed_steps(|| synthesize(&ir));
    let missing = target_missing(&synthesis);
    assert!(missing.is_empty(), "after {steps} steps: {missing:#?}");
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

/// A struct of two enums one guard compares, one more enum, three Booleans, two `Optional<Boolean>`
/// and an `Integer`: the input kinds `parts` and `entailed` group differently from the adopter's.
fn mixed_inputs() -> Vec<(String, Kind)> {
    vec![
        ("w".to_owned(), Kind::Pair),
        ("e2".to_owned(), Kind::Enum("Second", SECOND)),
        ("b1".to_owned(), Kind::Boolean),
        ("b2".to_owned(), Kind::Boolean),
        ("b3".to_owned(), Kind::Boolean),
        ("o1".to_owned(), Kind::MaybeBoolean),
        ("o2".to_owned(), Kind::MaybeBoolean),
        ("n".to_owned(), Kind::Integer(NUMBERS)),
    ]
}

/// A random literal over input `input` of [`mixed_inputs`].
fn mixed_literal(random: &mut Seeded, input: usize) -> String {
    let plain = match input {
        0 => match random.below(3) {
            0 => "\"w.p == w.q\"".to_owned(),
            1 => format!("\"w.p != {}\"", FIRST[random.below(4)]),
            _ => format!("\"w.p == {}\"", FIRST[random.below(4)]),
        },
        1 => format!("\"e2 == {}\"", SECOND[random.below(3)]),
        2 => format!("\"w.q == {}\"", FIRST[random.below(4)]),
        3..=5 => format!("\"b{} == {}\"", input - 2, random.below(2) == 1),
        6 | 7 => {
            let name = format!("o{}", input - 5);
            match random.below(3) {
                0 => format!("{{{name}: {{defined: {}}}}}", random.below(2) == 1),
                _ => format!("\"{name} == {}\"", random.below(2) == 1),
            }
        }
        _ => {
            let k = random.below(4);
            match random.below(3) {
                0 => format!("\"n > {k}\""),
                1 => format!("\"n <= {k}\""),
                _ => format!("\"n == {k}\""),
            }
        }
    };
    if random.below(5) == 0 {
        format!("{{not: {plain}}}")
    } else {
        plain
    }
}

fn mixed_guard(random: &mut Seeded) -> String {
    let size = 2 + random.below(3);
    let mut picked: Vec<usize> = Vec::new();
    while picked.len() < size {
        let input = random.below(9);
        if !picked.contains(&input) {
            picked.push(input);
        }
    }
    let mut children: Vec<String> = picked
        .iter()
        .map(|input| mixed_literal(random, *input))
        .collect();
    if random.below(3) == 0 {
        let a = random.below(9);
        let b = (a + 1 + random.below(8)) % 9;
        children.push(format!(
            "{{any: [{}, {}]}}",
            mixed_literal(random, a),
            mixed_literal(random, b)
        ));
    }
    all(&children)
}

fn mixed_chain(seed: u64, length: usize) -> Chain {
    let mut random = Seeded(seed);
    Chain {
        inputs: mixed_inputs(),
        refusals: (0..length)
            .map(|at| (format!("refuse-{at}"), mixed_guard(&mut random)))
            .collect(),
    }
}

/// Attack on `parts` and `entailed` over inputs they treat differently: an `Optional<Boolean>`
/// whose absence leaves a comparison unknown, an `Integer`, and two enums one guard compares with
/// each other, which joins their groups. Every branch some input reaches has its scenario.
#[test]
fn every_reachable_branch_of_seeded_chains_over_optional_integer_and_compared_enums_has_its_scenario(
) {
    let mut failures = Vec::new();
    for seed in 0..16u64 {
        let chain = mixed_chain(seed, 14);
        let ir = model(&chain);
        let reached = reachable(&ir, &chain);
        let (synthesis, steps) = directed_steps(|| synthesize(&ir));
        // A branch refused only as undecidable over an absent optional is excluded: ess 0.68.0
        // refuses the same branches of these seeds the same way, so that class is not this unit's.
        let missing: Vec<_> = missing_of(&reached, &synthesis)
            .into_iter()
            .filter(|(_, _, causes)| {
                causes.is_empty()
                    || !causes
                        .iter()
                        .all(|cause| cause.contains("cannot be decided"))
            })
            .collect();
        if !missing.is_empty() {
            failures.push((seed, steps, missing, model_text(&chain)));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failing seeds; seed, steps, missing, model: {failures:#?}",
        failures.len()
    );
}

/// Determinism: one chain synthesized on two threads gives the same suite bytes and the same
/// refusals, whatever each thread's memo already held.
#[test]
fn a_chain_synthesizes_the_same_suite_on_two_threads() {
    let chain = mixed_chain(3, 14);
    let ir = model(&chain);
    let warm = synthesize(&ir);
    let first = serde_json::to_string(&warm.suite).expect("serializes");
    let first_refusals: Vec<String> = warm.refusals.iter().map(ToString::to_string).collect();
    let (second, second_refusals) = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let cold = synthesize(&ir);
                (
                    serde_json::to_string(&cold.suite).expect("serializes"),
                    cold.refusals
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                )
            })
            .join()
            .expect("the thread finishes")
    });
    assert_eq!(first, second);
    assert_eq!(first_refusals, second_refusals);
}
