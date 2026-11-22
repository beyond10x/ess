//! The row a creating branch makes is made once per synthesis and context, not once per scenario
//! that acts on such a row.
//!
//! Every scenario acting on a row starts by creating one, and most ask exactly what the one before
//! them asked: each invariant scenario of an entity runs another branch on a row arranged afresh,
//! and the stored-row searches create the same first row under every goal. On a downstream
//! specification the rows of creating branches took 18 % of the first reading, and the invariant
//! scenarios arranging their rows again another 18 %. Synthesis of a model of one entity two
//! commands create and five commands move, with two invariants, is held to a bound on the rows it
//! makes rather than answers from memory, a work bound rather than a clock, and its suite, refusals
//! and notes must equal the ones made with every row made afresh.

use std::path::{Path, PathBuf};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{created_rows_made, created_rows_unkept, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/created-rows-kept.yaml");
const STORED: &str = include_str!("fixtures/stored-rows-decided-once.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn billing() -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("billing exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("validates");
    compile(&specification, &sources).expect("resolves")
}

fn bytes(synthesis: &Synthesis) -> String {
    serde_json::to_string(&synthesis.suite).expect("the suite serializes")
}

/// Synthesizes `ir` from memory and afresh, requires both to be equal, and answers how many rows
/// were asked for and how many made.
fn kept_and_unkept(name: &str, ir: &EssIr) -> (u64, u64) {
    let (kept, asked, made) = created_rows_made(|| synthesize(ir));
    let unkept = created_rows_unkept(|| synthesize(ir));
    assert_eq!(bytes(&kept), bytes(&unkept), "{name}: the suite bytes");
    assert_eq!(kept, unkept, "{name}: the suite, refusals and notes");
    eprintln!("{name}: {asked} rows of creating branches asked for, {made} made");
    (asked, made)
}

/// Without the memory every row asked for was made: 0 answered from memory (made == asked).
#[test]
fn rows_of_creating_branches_are_made_once_per_context() {
    let ir = ir(MODEL);
    let (asked, made) = kept_and_unkept("shop.order", &ir);
    assert!(asked > 0, "the model creates rows");
    assert!(
        made < asked,
        "{made} of {asked} rows of creating branches were made rather than answered from memory"
    );
}

#[test]
fn the_suites_made_from_memory_equal_the_ones_made_afresh() {
    for (name, ir) in [("stored", ir(STORED)), ("billing", billing())] {
        kept_and_unkept(name, &ir);
    }
}
