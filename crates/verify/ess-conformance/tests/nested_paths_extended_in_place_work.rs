//! A nested value is projected into facts with one path the walk extends and shortens in place,
//! not with a copy of the whole path made for every member it descends into.
//!
//! Every fact and presence mark of a nested input is keyed by its whole path, so those copies are
//! owed. Before, each member the walk descended into copied the path above it again, whether or
//! not the value held that member, and the copies of a value grew with its depth at every level:
//! at twelve levels the extensions were about a tenth of a small synthesis. A value nested twelve
//! structs deep is held to a bound on the path segments its projection copies or appends, a work
//! bound rather than a clock: 182 when each member copied the path above it (one segment for the
//! sequence, the twelve presence marks 1 + … + 12 = 78, the twelve extensions 2 + … + 13 = 90 and
//! the leaf's 13), 106 when the walk extends one path (the entry copy, the same marks, twelve
//! appended segments and the leaf).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::input::{flatten, path_segments_copied};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
    QualifiedName,
};
use ess_primitives::node::Node;

/// How many structs deep the value is.
const DEPTH: usize = 12;

/// The path segments the projection of one input may copy or append: 106 when the walk extends
/// one path, 182 when every member copied the path above it.
const BOUND: u64 = 120;

/// The segments every form of the walk copies, because a fact or a mark is keyed by them: the
/// twelve presence marks, 1 + … + 12 = 78, and the leaf's 13. A count below it says the walk went
/// past the counter, not that it was saved.
const FLOOR: u64 = 91;

fn model() -> String {
    let mut text =
        String::from("format: ess/37\nsystem: demo\nversion: v1\ndomain: demo.depth\ntypes:\n");
    for level in 1..=DEPTH {
        let inner = if level == DEPTH {
            "String".to_owned()
        } else {
            format!("demo.depth.Shell{}", level + 1)
        };
        let _ = writeln!(
            text,
            "  - {{name: demo.depth.Shell{level}, kind: struct, fields: [{{name: inner, type: \
             {inner}}}]}}"
        );
    }
    text.push_str(
        "events:\n  - name: demo.depth.Noted\n    fields:\n      - {name: sequence, type: \
         Integer}\ncommands:\n  - name: demo.depth.Note\n    input:\n      - {name: sequence, \
         type: Integer}\n      - {name: shell, type: demo.depth.Shell1}\n    outcomes:\n      - \
         name: noted\n        emits: [demo.depth.Noted]\n        payload:\n          \
         demo.depth.Noted: {sequence: input.sequence}\n",
    );
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("depth.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn a_deeply_nested_value_extends_one_path() {
    let text = model();
    let ir = ir(&text);
    let name: QualifiedName = "demo.depth.Note".parse().expect("a command name");
    let command = ir.commands().get(&name).expect("the command is declared");
    let mut shell = Node::Text("bottom".to_owned());
    for _ in 0..DEPTH {
        shell = Node::Map(BTreeMap::from([("inner".to_owned(), shell)]));
    }
    let candidate = BTreeMap::from([
        (
            "sequence".to_owned(),
            Node::Number(ess_primitives::facts::Number::from(1_i64)),
        ),
        ("shell".to_owned(), shell),
    ]);
    let (facts, copied) = path_segments_copied(|| flatten(&ir, command, &candidate));
    let facts = facts.unwrap_or_else(|errors| panic!("{errors}"));
    let leaf = format!("shell.{}", ["inner"; DEPTH].join("."));
    assert!(
        facts
            .facts()
            .iter()
            .any(|(path, _)| path.to_string() == leaf),
        "the leaf is bound at {leaf}"
    );
    assert!(
        copied >= FLOOR,
        "projecting the input counted {copied} path segments, fewer than the {FLOOR} it owes"
    );
    assert!(
        copied <= BOUND,
        "projecting the input copied or appended {copied} path segments, more than {BOUND}"
    );
}
