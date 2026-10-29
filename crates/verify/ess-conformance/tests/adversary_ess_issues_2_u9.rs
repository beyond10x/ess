//! Adversary pass on the `precedence-swap` equivalence of disjoint input guards
//! (<https://github.com/beyond10x/ess/issues/703>).
//!
//! Every swap the audit scores `equivalent` is checked against the model itself: the baseline and
//! the swapped model are executed by the interpreter on every input of a grid that covers each
//! input's values and, for an `Optional` one, its absence. A swap is observable on an input where
//! the two answer differently: one branch against the other, or an answer against a guard the
//! interpreter cannot decide. An `equivalent` swap must be observable on no input. Each shape also
//! states what the audit is expected to score, and a swap expected to stay scored must be
//! observable on some input of its grid, so the fixture is shown to mean what it says.
#![allow(clippy::needless_raw_string_hashes)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    /// Scored `equivalent`, and observable on no input.
    Equivalent,
    /// Not scored `equivalent`, and observable on some input.
    Scored,
    /// Either verdict, but `equivalent` only where observable on no input.
    Either,
}

struct Shape {
    command: &'static str,
    input: &'static [(&'static str, &'static str)],
    first: &'static str,
    second: &'static str,
    accepting: bool,
    expect: Expect,
}

const SHAPES: &[Shape] = &[
    // Grouping by shared leaf: the contradiction runs x -> y -> z through three groups' leaves.
    Shape {
        command: "Chain",
        input: &[
            ("x", "String"),
            ("y", "Boolean"),
            ("z", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{all: ['x == "u"', 'amount > 0']}"#,
        second: r#"{all: [{any: ['x == "v"', 'y == true']}, {any: ['y == false', 'z == true']}, 'z == false']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "ChainOpen",
        input: &[
            ("x", "String"),
            ("y", "Boolean"),
            ("z", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{all: ['x == "u"', 'amount > 0']}"#,
        second: r#"{all: [{any: ['x == "v"', 'y == true']}, {any: ['y == true', 'z == true']}, 'z == false']}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    // `any:` in `all:` in `any:`.
    Shape {
        command: "NestedShut",
        input: &[
            ("m", "Boolean"),
            ("n", "Boolean"),
            ("r", "Boolean"),
            ("s", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{any: [{all: ['m == true', {any: ['n == true', 'r == true']}]}, 's == true']}"#,
        second: r#"{all: ['s == false', 'm == false', 'amount > 0']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "NestedOpen",
        input: &[
            ("m", "Boolean"),
            ("o", "Optional<Boolean>"),
            ("s", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{any: [{all: ['m == true', {any: ['o == true', 'amount > 0']}]}, 's == true']}"#,
        second: r#"{all: ['s == false', 'm == true', 'o == false']}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "NestedUnknown",
        input: &[
            ("m", "Boolean"),
            ("o", "Optional<Boolean>"),
            ("s", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{any: [{all: ['m == true', {any: ['o == true', 'amount > 0']}]}, 's == true']}"#,
        second: r#"{all: ['s == false', 'm == false', 'o == false']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    // `not`.
    Shape {
        command: "NotOptional",
        input: &[("mark", "Optional<Boolean>")],
        first: r#"{not: 'mark == true'}"#,
        second: r#"mark == true"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "NotDefined",
        input: &[("mark", "Optional<Boolean>")],
        first: r#"{not: {mark: {defined: true}}}"#,
        second: r#"mark == false"#,
        accepting: false,
        expect: Expect::Scored,
    },
    // `defined`.
    Shape {
        command: "DefinedShut",
        input: &[("mark", "Optional<Boolean>")],
        first: r#"{all: [{mark: {defined: true}}, 'mark == true']}"#,
        second: r#"mark == false"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "DefinedOpen",
        input: &[("mark", "Optional<Boolean>")],
        first: r#"{any: [{mark: {defined: false}}, 'mark == true']}"#,
        second: r#"mark == false"#,
        accepting: false,
        expect: Expect::Scored,
    },
    // An `Optional` enum compared with `!=`.
    Shape {
        command: "LaneNe",
        input: &[("lane", "Optional<demo.adv.Lane>")],
        first: r#"'lane != "north"'"#,
        second: r#"'lane == "north"'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "LaneNeAbsent",
        input: &[("lane", "Optional<demo.adv.Lane>")],
        first: r#"'lane != "north"'"#,
        second: r#"{lane: {defined: false}}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "LaneNeBeside",
        input: &[
            ("lane", "Optional<demo.adv.Lane>"),
            ("flag", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{all: ['lane != "north"', 'amount > 0']}"#,
        second: r#"{all: ['lane == "north"', 'flag == true']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "LaneNoneOf",
        input: &[("lane", "Optional<demo.adv.Lane>")],
        first: r#"{lane: {none_of: [north]}}"#,
        second: r#"'lane == "north"'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    // Numeric comparisons beside pins.
    Shape {
        command: "NumberPins",
        input: &[("amount", "Integer")],
        first: r#"{all: ['amount == 5', 'amount > 3']}"#,
        second: r#"{all: ['amount != 5', 'amount > 3']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "NumberOpen",
        input: &[("amount", "Integer")],
        first: r#"'amount == 5'"#,
        second: r#"'amount > 3'"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "NumberOrder",
        input: &[("flag", "Boolean"), ("amount", "Integer")],
        first: r#"{all: ['flag == true', 'amount > 0']}"#,
        second: r#"{all: ['flag == true', 'amount < 0']}"#,
        accepting: false,
        expect: Expect::Either,
    },
    // One leaf reached through two paths: an input and a struct member.
    Shape {
        command: "TwoPaths",
        input: &[("lane", "demo.adv.Lane"), ("route", "demo.adv.Route")],
        first: r#"'route.lane == "north"'"#,
        second: r#"'lane == "south"'"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "OptionalRoute",
        input: &[("route", "Optional<demo.adv.Route>")],
        first: r#"'route.lane == "north"'"#,
        second: r#"'route.lane != "north"'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "OptionalRouteAbsent",
        input: &[
            ("lane", "demo.adv.Lane"),
            ("route", "Optional<demo.adv.Route>"),
        ],
        first: r#"{route: {defined: false}}"#,
        second: r#"'lane == "south"'"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "OptionalRouteMember",
        input: &[
            ("lane", "demo.adv.Lane"),
            ("route", "Optional<demo.adv.Route>"),
        ],
        first: r#"{all: ['route.lane != "north"', 'lane == "north"']}"#,
        second: r#"{all: ['route.lane == "north"', 'lane == "north"']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    // Guards that differ only by the refusal's answer.
    Shape {
        command: "SameGuard",
        input: &[("flag", "Boolean"), ("amount", "Integer")],
        first: r#"{all: ['flag == true', 'amount > 0']}"#,
        second: r#"{all: ['flag == true', 'amount > 0']}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "SameGuardOptional",
        input: &[("mark", "Optional<Boolean>")],
        first: r#"mark == true"#,
        second: r#"mark == true"#,
        accepting: false,
        expect: Expect::Scored,
    },
    // Accepting branches.
    Shape {
        command: "AcceptShut",
        input: &[("mark", "Optional<Boolean>")],
        first: r#"mark == true"#,
        second: r#"mark == false"#,
        accepting: true,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "AcceptOpen",
        input: &[("mark", "Optional<Boolean>")],
        first: r#"{mark: {defined: false}}"#,
        second: r#"mark == true"#,
        accepting: true,
        expect: Expect::Scored,
    },
    Shape {
        command: "AcceptBeside",
        input: &[("flag", "Boolean"), ("amount", "Integer")],
        first: r#"{all: ['flag == true', 'amount > 0']}"#,
        second: r#"{all: ['flag == false', 'amount > 0']}"#,
        accepting: true,
        expect: Expect::Equivalent,
    },
    // Attributes of an `Optional` enum.
    Shape {
        command: "OptionalAttribute",
        input: &[("op", "Optional<demo.adv.Op>")],
        first: r#"'op.attr_a == true'"#,
        second: r#"'op.attr_a == false'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "OptionalAttributeNe",
        input: &[("op", "Optional<demo.adv.Op>")],
        first: r#"'op.attr_a != true'"#,
        second: r#"'op.attr_a == true'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "OptionalAttributeAbsent",
        input: &[("op", "Optional<demo.adv.Op>")],
        first: r#"'op.attr_a == true'"#,
        second: r#"{op: {defined: false}}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    // Pass 2: an enum leaf decided over the variants its guard names and one stand-in for the
    // rest. `demo.adv.Wide` has six variants; the grid sends every one.
    Shape {
        command: "WideNe",
        input: &[("wide", "demo.adv.Wide")],
        first: r#"'wide != "w0"'"#,
        second: r#"'wide == "w0"'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    // The two guards overlap only on variants neither names.
    Shape {
        command: "WideUnnamedOverlap",
        input: &[("wide", "demo.adv.Wide")],
        first: r#"{wide: {none_of: [w0]}}"#,
        second: r#"{wide: {none_of: [w1]}}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "WideNotIn",
        input: &[("wide", "demo.adv.Wide")],
        first: r#"{not: {wide: {in: [w0, w1]}}}"#,
        second: r#"{not: 'wide == "w2"'}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "WideAnyNot",
        input: &[
            ("wide", "demo.adv.Wide"),
            ("flag", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{any: [{wide: {none_of: [w0, w1, w2]}}, 'flag == true']}"#,
        second: r#"{all: ['flag == false', {wide: {in: [w0, w1]}}, 'amount > 0']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "WideAnyNotOpen",
        input: &[
            ("wide", "demo.adv.Wide"),
            ("flag", "Boolean"),
            ("amount", "Integer"),
        ],
        first: r#"{any: [{wide: {none_of: [w0, w1, w2]}}, 'flag == true']}"#,
        second: r#"{all: ['flag == false', {not: 'wide == "w0"'}, 'amount > 0']}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "OptionalWideAbsent",
        input: &[("owide", "Optional<demo.adv.Wide>")],
        first: r#"'owide != "w0"'"#,
        second: r#"{owide: {defined: false}}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "OptionalWideShut",
        input: &[("owide", "Optional<demo.adv.Wide>")],
        first: r#"{owide: {none_of: [w0]}}"#,
        second: r#"'owide == "w0"'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    // Two enum inputs of one type.
    Shape {
        command: "SharedType",
        input: &[("left", "demo.adv.Wide"), ("right", "demo.adv.Wide")],
        first: r#"{all: ['left == "w0"', 'right != "w0"']}"#,
        second: r#"{all: ['right == "w0"', 'left != "w0"']}"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "SharedTypeOpen",
        input: &[("left", "demo.adv.Wide"), ("right", "demo.adv.Wide")],
        first: r#"'left != "w0"'"#,
        second: r#"'right != "w1"'"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "SharedTypeCross",
        input: &[("left", "demo.adv.Wide"), ("right", "demo.adv.Wide")],
        first: r#"{all: ['left == "w0"', 'right == "w1"']}"#,
        second: r#"{all: ['left != "w1"', 'right != "w0"']}"#,
        accepting: false,
        expect: Expect::Scored,
    },
    // An earlier refusal (`EARLIER`) takes part of the overlap: only a variant no guard names is
    // left over.
    Shape {
        command: "WideTaken",
        input: &[("wide", "demo.adv.Wide")],
        first: r#"'wide != "w1"'"#,
        second: r#"'wide != "w2"'"#,
        accepting: false,
        expect: Expect::Scored,
    },
    Shape {
        command: "WideTakenShut",
        input: &[("wide", "demo.adv.Wide")],
        first: r#"'wide != "w1"'"#,
        second: r#"'wide != "w2"'"#,
        accepting: false,
        expect: Expect::Equivalent,
    },
    // Accepting branches that emit.
    Shape {
        command: "WideEmits",
        input: &[("wide", "demo.adv.Wide")],
        first: r#"'wide == "w0"'"#,
        second: r#"'wide != "w0"'"#,
        accepting: true,
        expect: Expect::Equivalent,
    },
    Shape {
        command: "WideEmitsUnnamed",
        input: &[("wide", "demo.adv.Wide")],
        first: r#"{wide: {none_of: [w0]}}"#,
        second: r#"{wide: {none_of: [w1]}}"#,
        accepting: true,
        expect: Expect::Scored,
    },
];

/// A refusal declared before `first` in the commands named.
const EARLIER: &[(&str, &str)] = &[
    ("WideTaken", r#"'wide == "w0"'"#),
    ("WideTakenShut", r#"{wide: {none_of: [w1, w2]}}"#),
];

const HEADER: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.adv
summary: Adjacent input-guarded branches whose order may or may not be observable.

types:
  - {name: demo.adv.Lane, kind: enum, variants: [north, south, east]}
  - name: demo.adv.Route
    kind: struct
    fields:
      - {name: lane, type: demo.adv.Lane}
  - name: demo.adv.Op
    kind: enum
    attributes:
      - {name: attr_a, type: Boolean}
    variants:
      - {name: V1, attributes: {attr_a: true}}
      - {name: V2, attributes: {attr_a: false}}
      - {name: V3, attributes: {attr_a: true}}
  - {name: demo.adv.Wide, kind: enum, variants: [w0, w1, w2, w3, w4, w5]}

errors:
  - name: demo.adv.RefusedA
    summary: Refused for the first reason.
  - name: demo.adv.RefusedB
    summary: Refused for the second reason.

events:
  - {name: demo.adv.Left, fields: []}
  - {name: demo.adv.Right, fields: []}
"#;

/// The fixture over `shapes`, with the two branches of `swapped` in the other order.
fn fixture(shapes: &[Shape], swapped: Option<&str>) -> String {
    let names: Vec<String> = shapes
        .iter()
        .map(|shape| format!("demo.adv.{}", shape.command))
        .collect();
    let mut text = HEADER.replace(
        "\ntypes:",
        &format!(
            "\ncomponents:\n  - component: demo-server\n    owns:\n      domains: [demo.adv]\n    accepts:\n      commands: [{}]\n    reached_by: network\n\ntypes:",
            names.join(", ")
        ),
    );
    text.push_str("\ncommands:\n");
    for shape in shapes {
        writeln!(text, "  - name: demo.adv.{}\n    input:", shape.command)
            .expect("a String takes text");
        for (name, kind) in shape.input {
            writeln!(text, "      - {{name: {name}, type: {kind}}}").expect("a String takes text");
        }
        text.push_str("    outcomes:\n");
        if let Some((_, guard)) = EARLIER.iter().find(|(it, _)| *it == shape.command) {
            writeln!(
                text,
                "      - {{name: earlier, when: {guard}, error: demo.adv.RefusedA}}"
            )
            .expect("a String takes text");
        }
        let (first_answer, second_answer) = if shape.accepting {
            ("emits: [demo.adv.Left]", "emits: [demo.adv.Right]")
        } else {
            ("error: demo.adv.RefusedA", "error: demo.adv.RefusedB")
        };
        let first = format!(
            "      - {{name: first, when: {}, {first_answer}}}\n",
            shape.first
        );
        let second = format!(
            "      - {{name: second, when: {}, {second_answer}}}\n",
            shape.second
        );
        if swapped == Some(shape.command) {
            text.push_str(&second);
            text.push_str(&first);
        } else {
            text.push_str(&first);
            text.push_str(&second);
        }
        text.push_str("      - name: rest\n        accepts: nothing\n");
    }
    text
}

fn parsed(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled(text: &str) -> EssIr {
    let (files, texts) = parsed(text);
    mutate::compile(files, &texts).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn report() -> &'static MutationReport {
    static REPORT: OnceLock<MutationReport> = OnceLock::new();
    REPORT.get_or_init(|| {
        let text = fixture(SHAPES, None);
        let (files, texts) = parsed(&text);
        let ir = compiled(&text);
        mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
            Interpreted::for_model(ir.clone())
        })
        .unwrap_or_else(|refusal| panic!("{refusal}"))
    })
}

/// Every value an input of declared type `kind` takes in the grid; `None` is its absence.
fn values(kind: &str) -> Vec<Option<Node>> {
    if let Some(inner) = kind
        .strip_prefix("Optional<")
        .and_then(|rest| rest.strip_suffix('>'))
    {
        let mut out = values(inner);
        out.push(None);
        return out;
    }
    let text = |items: &[&str]| -> Vec<Option<Node>> {
        items
            .iter()
            .map(|it| Some(Node::Text((*it).to_owned())))
            .collect()
    };
    match kind {
        "Boolean" => vec![Some(Node::Bool(true)), Some(Node::Bool(false))],
        "Integer" => [-1_i64, 0, 1, 4, 5, 6]
            .into_iter()
            .map(|it| Some(Node::Number(Number::from(it))))
            .collect(),
        "String" => text(&["u", "v", "w"]),
        "demo.adv.Lane" => text(&["north", "south", "east"]),
        "demo.adv.Op" => text(&["V1", "V2", "V3"]),
        "demo.adv.Wide" => text(&["w0", "w1", "w2", "w3", "w4", "w5"]),
        "demo.adv.Route" => ["north", "south", "east"]
            .into_iter()
            .map(|lane| {
                Some(Node::Map(BTreeMap::from([(
                    "lane".to_owned(),
                    Node::Text(lane.to_owned()),
                )])))
            })
            .collect(),
        other => panic!("no grid for `{other}`"),
    }
}

/// Every input of the grid over `input`.
fn grid(input: &[(&str, &str)]) -> Vec<BTreeMap<String, Node>> {
    let mut out = vec![BTreeMap::new()];
    for (name, kind) in input {
        let mut next = Vec::new();
        for partial in &out {
            for value in values(kind) {
                let mut one: BTreeMap<String, Node> = partial.clone();
                if let Some(value) = value {
                    one.insert((*name).to_owned(), value);
                }
                next.push(one);
            }
        }
        out = next;
    }
    out
}

/// What the model `ir` answers `command` sent `input`: the branches taken, or `undecided` where a
/// guard read is unknown.
fn answer(ir: &EssIr, command: &str, input: &BTreeMap<String, Node>) -> String {
    let name = format!("demo.adv.{command}")
        .parse()
        .expect("a command name");
    match execute(ir, &Store::default(), &name, input, &Externals::Withheld) {
        Ok(steps) => steps
            .iter()
            .map(|step| {
                step.outcome
                    .as_ref()
                    .map_or_else(|| "undeclared".to_owned(), ToString::to_string)
            })
            .collect::<Vec<_>>()
            .join(","),
        Err(why) if why.is_capability_gap() => "undecided".to_owned(),
        Err(why) => panic!("{command} {input:?}: {why}"),
    }
}

/// The first input on which the baseline and the swapped model of `shape` answer differently.
/// A swapped model that validation refuses because the moved refusal now answers every input of
/// the branch behind it (`unreachable_branch`, #702) is told apart before any input is sent.
fn observed_on(shape: &Shape) -> Option<String> {
    let baseline = compiled(&fixture(SHAPES, None));
    let text = fixture(SHAPES, Some(shape.command));
    let (files, texts) = parsed(&text);
    let swapped = match mutate::compile(files, &texts) {
        Ok(ir) => ir,
        Err(refused) if refused.cause == "ESS-COMMAND-012" => {
            return Some(format!("swapped model refused: {}", refused.message));
        }
        Err(refused) => panic!("{refused:?}\n{text}"),
    };
    grid(shape.input).into_iter().find_map(|input| {
        let (before, after) = (
            answer(&baseline, shape.command, &input),
            answer(&swapped, shape.command, &input),
        );
        (before != after).then(|| format!("{input:?}: baseline {before}, swapped {after}"))
    })
}

/// Every way the shapes named in `commands` disagree with their expectation.
fn disagreements(commands: &[&str]) -> Vec<String> {
    let report = report();
    let mut out = Vec::new();
    for shape in SHAPES.iter().filter(|it| commands.contains(&it.command)) {
        let id = format!("precedence-swap/demo.adv.{}/first/second", shape.command);
        let Some(entry) = report.mutants.iter().find(|it| it.id == id) else {
            out.push(format!("{id}: no such mutant"));
            continue;
        };
        let observed = observed_on(shape);
        let equivalent = entry.verdict == Verdict::Equivalent;
        if equivalent {
            if let Some(input) = &observed {
                out.push(format!(
                    "{id}: scored equivalent ({:?}), yet observable on {input}",
                    entry.unsatisfiable_guard
                ));
            }
        }
        match shape.expect {
            Expect::Equivalent if !equivalent => out.push(format!(
                "{id}: expected equivalent, scored {:?}",
                entry.verdict
            )),
            Expect::Scored if equivalent => out.push(format!(
                "{id}: expected scored, scored equivalent ({:?})",
                entry.unsatisfiable_guard
            )),
            Expect::Scored if observed.is_none() => out.push(format!(
                "{id}: expected observable, the grid found no input that tells the orders apart"
            )),
            _ => {}
        }
    }
    out
}

fn assert_agree(commands: &[&str]) {
    let found = disagreements(commands);
    assert!(
        found.is_empty(),
        "{}\n\n{}",
        found.join("\n"),
        report().render_text()
    );
}

#[test]
fn groups_joined_through_a_third_leaf_decide_soundly() {
    assert_agree(&["Chain", "ChainOpen"]);
}

#[test]
fn any_nested_in_all_nested_in_any_decides_soundly() {
    assert_agree(&["NestedShut", "NestedOpen", "NestedUnknown"]);
}

#[test]
fn not_and_defined_decide_soundly() {
    assert_agree(&["NotOptional", "NotDefined", "DefinedShut", "DefinedOpen"]);
}

#[test]
fn an_optional_enum_compared_with_ne_decides_soundly() {
    assert_agree(&["LaneNe", "LaneNeAbsent", "LaneNeBeside", "LaneNoneOf"]);
}

#[test]
fn numeric_comparisons_beside_pins_decide_soundly() {
    assert_agree(&["NumberPins", "NumberOpen", "NumberOrder"]);
}

#[test]
fn one_leaf_through_an_input_and_a_struct_member_decides_soundly() {
    assert_agree(&[
        "TwoPaths",
        "OptionalRoute",
        "OptionalRouteAbsent",
        "OptionalRouteMember",
    ]);
}

#[test]
fn guards_differing_only_by_the_answer_stay_scored() {
    assert_agree(&["SameGuard", "SameGuardOptional"]);
}

#[test]
fn accepting_pairs_decide_soundly() {
    assert_agree(&["AcceptShut", "AcceptOpen", "AcceptBeside"]);
}

#[test]
fn attributes_of_an_optional_enum_decide_soundly() {
    assert_agree(&[
        "OptionalAttribute",
        "OptionalAttributeNe",
        "OptionalAttributeAbsent",
    ]);
}

#[test]
fn the_enum_stand_in_decides_soundly() {
    assert_agree(&[
        "WideNe",
        "WideUnnamedOverlap",
        "WideNotIn",
        "WideAnyNot",
        "WideAnyNotOpen",
        "OptionalWideAbsent",
        "OptionalWideShut",
    ]);
}

#[test]
fn two_enum_inputs_of_one_type_decide_soundly() {
    assert_agree(&["SharedType", "SharedTypeOpen", "SharedTypeCross"]);
}

#[test]
fn an_earlier_branch_taking_the_named_part_of_the_overlap_decides_soundly() {
    assert_agree(&["WideTaken", "WideTakenShut"]);
}

#[test]
fn accepting_pairs_over_the_enum_stand_in_decide_soundly() {
    assert_agree(&["WideEmits", "WideEmitsUnnamed"]);
}

/// The reading `ess-diff` asks: over declared values only, an overlap reached only by a variant
/// no literal names is satisfiable, and one every variant is named out of is not.
#[test]
fn declared_values_reading_keeps_the_unnamed_variants() {
    use ess_compiler::ir::ResolvedCondition;
    use ess_primitives::predicate::Predicate;
    let ir = compiled(&fixture(SHAPES, None));
    let overlap = |command: &str| {
        let found = ir
            .commands()
            .values()
            .find(|it| it.name.to_string() == format!("demo.adv.{command}"))
            .expect("a command of the fixture");
        let guard = |name: &str| match &found
            .outcomes
            .iter()
            .find(|it| it.name.to_string() == name)
            .expect("a branch")
            .condition
        {
            ResolvedCondition::When { predicate } => predicate.clone(),
            other => panic!("{other:?}"),
        };
        (found, Predicate::All(vec![guard("first"), guard("second")]))
    };
    let decided = |command: &str| {
        let (found, guard) = overlap(command);
        mutate::satisfiable_by_declared_values(&ir, found, &guard)
    };
    assert_eq!(
        [
            decided("WideUnnamedOverlap"),
            decided("SharedTypeOpen"),
            decided("WideNe"),
            decided("SharedType"),
        ],
        [Some(true), Some(true), Some(false), Some(false)]
    );
}

/// The soundness property over every shape at once: no swap scored `equivalent` is observable.
#[test]
fn no_equivalent_swap_is_observable() {
    let report = report();
    let mut found = Vec::new();
    for shape in SHAPES {
        let id = format!("precedence-swap/demo.adv.{}/first/second", shape.command);
        let Some(entry) = report.mutants.iter().find(|it| it.id == id) else {
            continue;
        };
        if entry.verdict == Verdict::Equivalent {
            if let Some(input) = observed_on(shape) {
                found.push(format!("{id}: {input}"));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// One plain enum input of 70 variants pinned to two of them: no input selects both branches,
/// whatever the number of variants, yet one leaf's domain alone exceeds the 64 combinations the
/// check tries, so no part of the overlap is decided.
#[test]
fn an_enum_of_seventy_variants_pinned_to_two_values_is_equivalent() {
    let variants: Vec<String> = (0..70).map(|index| format!("v{index}")).collect();
    let text = format!(
        r#"format: ess/23
system: demo
version: v1
domain: demo.wide
summary: Two refusals pinned to two variants of a wide enum.

components:
  - component: demo-server
    owns:
      domains: [demo.wide]
    accepts:
      commands: [demo.wide.Pick]
    reached_by: network

types:
  - {{name: demo.wide.Wide, kind: enum, variants: [{}]}}

errors:
  - name: demo.wide.RefusedA
    summary: Refused for the first reason.
  - name: demo.wide.RefusedB
    summary: Refused for the second reason.

commands:
  - name: demo.wide.Pick
    input:
      - {{name: wide, type: demo.wide.Wide}}
    outcomes:
      - {{name: first, when: 'wide == "v1"', error: demo.wide.RefusedA}}
      - {{name: second, when: 'wide == "v2"', error: demo.wide.RefusedB}}
      - name: rest
        accepts: nothing
"#,
        variants.join(", ")
    );
    let (files, texts) = parsed(&text);
    let ir = compiled(&text);
    let report = mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let entry = report
        .mutants
        .iter()
        .find(|it| it.id == "precedence-swap/demo.wide.Pick/first/second")
        .expect("the swap is a site");
    assert_eq!(
        (entry.verdict, entry.unsatisfiable_guard.as_deref()),
        (Verdict::Equivalent, Some(r#"(wide == v1 and wide == v2)"#)),
        "{}",
        report.render_text()
    );
}
