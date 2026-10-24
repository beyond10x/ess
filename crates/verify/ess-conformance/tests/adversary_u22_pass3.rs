//! Adversary pass 3 on `story:a-first-match-chain-is-synthesized-within-a-work-bound`.
//!
//! Attacks what pass 2's fixes added: an atom the goal already decides joins no part
//! (`Directed::determined`), and guards that still share an input are searched once per value of
//! it (`Directed::cases`). Reachability is decided here by trying inputs through `flatten` and
//! `decide`, never by the search under test; an input counts as reaching a branch only where every
//! guard declared before it is decided `false` and its own `true`.

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
    /// `demo.adv.Window`, a struct of two `Integer` members `a` and `b`, each tried at these values.
    Window(&'static [i64]),
}

struct Chain {
    inputs: Vec<(String, Kind)>,
    /// Refusals in declaration order: name and the YAML of its `when:`.
    refusals: Vec<(String, String)>,
}

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
        .any(|(_, kind)| matches!(kind, Kind::Window(_)))
    {
        text.push_str("  - name: demo.adv.Window\n    kind: struct\n    fields:\n      - {name: a, type: Integer}\n      - {name: b, type: Integer}\n");
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
            Kind::Window(_) => "demo.adv.Window".to_owned(),
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

fn number(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// Every input of the chain's space.
fn every_input(chain: &Chain) -> Vec<BTreeMap<String, Node>> {
    let mut all = vec![BTreeMap::new()];
    for (name, kind) in &chain.inputs {
        let values: Vec<Node> = match kind {
            Kind::Boolean => vec![Node::Bool(false), Node::Bool(true)],
            Kind::Enum(_, variants) => variants
                .iter()
                .map(|v| Node::Text((*v).to_owned()))
                .collect(),
            Kind::Window(values) => {
                let mut pairs = Vec::new();
                for a in *values {
                    for b in *values {
                        pairs.push(Node::Map(BTreeMap::from([
                            ("a".to_owned(), number(*a)),
                            ("b".to_owned(), number(*b)),
                        ])));
                    }
                }
                pairs
            }
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

const SMALL: &[&str] = &[
    "V0", "V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8", "V9", "V10", "V11", "V12", "V13", "V14",
    "V15", "V16", "V17", "V18", "V19",
];
const NUMBERS: &[i64] = &[-1, 0, 1, 2, 3, 4, 5, 6, 7, 8];

/// Four Booleans, a twenty-variant enum `e` and a struct `w` of two `Integer`s `a` and `b`.
fn wide_inputs() -> Vec<(String, Kind)> {
    vec![
        ("b1".to_owned(), Kind::Boolean),
        ("b2".to_owned(), Kind::Boolean),
        ("b3".to_owned(), Kind::Boolean),
        ("b4".to_owned(), Kind::Boolean),
        ("e".to_owned(), Kind::Enum("Small", SMALL)),
        ("w".to_owned(), Kind::Window(NUMBERS)),
    ]
}

/// A literal over input `input` of [`wide_inputs`]; the shared inputs (`e`, `a`, `b`) are read with
/// a literal of their own each time, so the guards sharing them have many cases between them.
fn wide_literal(random: &mut Seeded, input: usize) -> String {
    let plain = match input {
        0..=3 => format!("\"b{} == {}\"", input + 1, random.below(2) == 1),
        4 => match random.below(2) {
            0 => format!("\"e != {}\"", SMALL[random.below(SMALL.len())]),
            _ => format!("\"e == {}\"", SMALL[random.below(SMALL.len())]),
        },
        5 => match random.below(3) {
            0 => format!("\"w.a > {}\"", random.below(8)),
            1 => format!("\"w.a <= {}\"", random.below(8)),
            _ => format!("\"w.a != {}\"", random.below(8)),
        },
        _ => match random.below(3) {
            0 => format!("\"w.b <= {}\"", random.below(8)),
            1 => format!("\"w.b > {}\"", random.below(8)),
            _ => format!("\"w.b != {}\"", random.below(8)),
        },
    };
    if random.below(6) == 0 {
        format!("{{not: {plain}}}")
    } else {
        plain
    }
}

fn wide_guard(random: &mut Seeded) -> String {
    let size = 2 + random.below(3);
    let mut picked: Vec<usize> = Vec::new();
    // Most guards read a shared input.
    if random.below(4) != 0 {
        picked.push(4 + random.below(3));
    }
    while picked.len() < size {
        let input = random.below(7);
        if !picked.contains(&input) {
            picked.push(input);
        }
    }
    let children: Vec<String> = picked
        .iter()
        .map(|input| wide_literal(random, *input))
        .collect();
    format!("{{all: [{}]}}", children.join(", "))
}

fn wide_chain(seed: u64, length: usize) -> Chain {
    let mut random = Seeded(seed);
    Chain {
        inputs: wide_inputs(),
        refusals: (0..length)
            .map(|at| (format!("refuse-{at}"), wide_guard(&mut random)))
            .collect(),
    }
}

/// Attack on `cases` and `determined` where the shared inputs have more values than a case split
/// takes: a twenty-variant enum and two `Integer` members, each read with a literal of its own by
/// most guards. Every branch some input reaches has its scenario.
#[test]
fn every_reachable_branch_of_seeded_chains_over_wide_shared_inputs_has_its_scenario() {
    let mut failures = Vec::new();
    for seed in 0..24u64 {
        let chain = wide_chain(seed, 16);
        let ir = model(&chain);
        let reached = reachable(&ir, &chain);
        let started = std::time::Instant::now();
        let (synthesis, steps) = directed_steps(|| synthesize(&ir));
        let took = started.elapsed();
        eprintln!("seed {seed}: {steps} steps, {took:?}");
        let missing = missing_of(&reached, &synthesis);
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

/// Determinism: the same wide chain synthesized twice on one thread and once on another gives the
/// same suite bytes and the same refusals.
#[test]
fn a_wide_chain_synthesizes_the_same_suite_every_time() {
    let chain = wide_chain(5, 14);
    let ir = model(&chain);
    let render = |synthesis: &Synthesis| {
        (
            serde_json::to_string(&synthesis.suite).expect("serializes"),
            synthesis
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        )
    };
    let first = render(&synthesize(&ir));
    let second = render(&synthesize(&ir));
    let third = std::thread::scope(|scope| {
        scope
            .spawn(|| render(&synthesize(&ir)))
            .join()
            .expect("the thread finishes")
    });
    assert_eq!(first, second);
    assert_eq!(first, third);
}

/// The unit's own `shared_input_model` shape (four cores over `c1`, `c2` and `s`, six decoys, a
/// wrong-first guard, then the target `t == true and m == false`), where every guard but the target
/// also reads an enum `k` through `{any: [k == V0, ...]}` over all of its variants. That glue is
/// true for every `k`, so the target is reached by `t` and `s` true and everything else false.
fn every_variant_glue_chain(variants: usize) -> Chain {
    let (big, _) = SMALL.split_at(variants);
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
    let not = |atom: &str| format!("{{not: \"{atom} == true\"}}");
    let is = |atom: &str| format!("\"{atom} == true\"");
    let mut refusals = Vec::new();
    for (at, (c1, c2)) in [
        (not("c1"), not("c2")),
        (not("c1"), is("c2")),
        (is("c1"), not("c2")),
        (is("c1"), is("c2")),
    ]
    .into_iter()
    .enumerate()
    {
        refusals.push((
            format!("core-{at}"),
            format!("{{all: [{c1}, {c2}, {}, {glue}]}}", not("s")),
        ));
    }
    for i in 1..=6 {
        refusals.push((
            format!("decoy-{i}"),
            format!("{{all: [\"d{i} == true\", \"x{i} == true\", {glue}]}}"),
        ));
    }
    refusals.push((
        "wrong-first".to_owned(),
        format!("{{all: [\"s == true\", \"z == true\", {glue}]}}"),
    ));
    refusals.push((
        "target".to_owned(),
        "{all: [\"t == true\", \"m == false\"]}".to_owned(),
    ));
    Chain { inputs, refusals }
}

/// `cases` takes a shared input only when it has between two and sixteen cases. With sixteen
/// variants the target and `created` both get their scenario; with seventeen the split is refused,
/// every guard joins one group over `k`, and the search gives both up at its 1024-step bound,
/// although the witness below reaches the target and all-false-but-`s` reaches `created`.
/// 0.68.0 refuses the same two branches (not a regression; the unit's mechanism stops at a cliff),
/// and 0.69.0 keeps that as a known limit: at seventeen both are refused, each refusal there and
/// naming the bound, so an adopter is not sent to rewrite a guard some input meets.
#[test]
fn guards_sharing_an_enum_have_their_scenarios_at_sixteen_variants_and_name_the_bound_at_seventeen()
{
    let mut failures = Vec::new();
    let mut unbounded = Vec::new();
    for variants in [16, 17] {
        let chain = every_variant_glue_chain(variants);
        let ir = model(&chain);
        let mut target = BTreeMap::new();
        let mut created = BTreeMap::new();
        for (name, kind) in &chain.inputs {
            let value = match kind {
                Kind::Enum(..) => Node::Text("V0".to_owned()),
                _ => Node::Bool(name == "t" || name == "s"),
            };
            target.insert(name.clone(), value.clone());
            created.insert(
                name.clone(),
                match kind {
                    Kind::Enum(..) => value,
                    _ => Node::Bool(name == "s"),
                },
            );
        }
        assert_eq!(selected(&ir, &target).as_deref(), Some("target"));
        assert_eq!(selected(&ir, &created).as_deref(), Some("created"));
        let reached = BTreeMap::from([
            ("target".to_owned(), target),
            ("created".to_owned(), created),
        ]);
        let (synthesis, steps) = directed_steps(|| synthesize(&ir));
        let missing = missing_of(&reached, &synthesis);
        if variants <= 16 {
            if !missing.is_empty() {
                failures.push((variants, steps, missing));
            }
            continue;
        }
        let refused: BTreeSet<&str> = missing.iter().map(|(name, ..)| name.as_str()).collect();
        if refused != BTreeSet::from(["created", "target"]) {
            failures.push((variants, steps, missing.clone()));
        }
        for (name, _, causes) in &missing {
            if causes.is_empty()
                || !causes
                    .iter()
                    .all(|cause| cause.contains("at its bound of 1024 steps"))
            {
                unbounded.push((variants, name.clone(), causes.clone()));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "variants, steps, missing (at seventeen: exactly `created` and `target`): {failures:#?}"
    );
    assert!(
        unbounded.is_empty(),
        "a branch the step bound refused is not refused naming it: {unbounded:#?}"
    );
}
