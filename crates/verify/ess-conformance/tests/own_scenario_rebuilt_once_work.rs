//! The setup run of a creating branch arranged as its own scenario is built once per thing it
//! reads, however many scenarios ask for it.
//!
//! A creating branch reading several related rows, whose strict search finds no run with the field
//! values another scenario chose, is arranged as its own scenario is. Every scenario of every
//! command that needs such a row asks for that arrangement again, with the same command, outcome,
//! chosen values and bound rows. Built anew for each, a model of a few hundred scenarios took
//! about 30 times as long to synthesize as the release before the arrangement took chosen values.
//! Synthesis of a creator whose row several commands read is held to building each arrangement
//! once, a work bound rather than a clock: the arrangements built do not grow with the number of
//! commands asking for them.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{own_overlay_work, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use std::fmt::Write as _;

const MODEL: &str = include_str!("fixtures/a-creator-chosen-values-beside-several-rows.yaml");

/// The first line of the command reading the grant; the command runs up to the views.
const READER: &str = "  - name: shop.desk.UseGrant\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The model with `readers` commands reading the grant the way `UseGrant` does: `UseGrant` and
/// `readers - 1` copies of it, each one more scenario that asks for a grant issued with the mail
/// name and a mode other than denied.
fn desk(readers: usize) -> String {
    let start = MODEL.find(READER).expect("the reader is declared");
    let end = MODEL
        .find("views:\n")
        .expect("the views follow the commands");
    let reader = &MODEL[start..end];
    let mut copies = String::new();
    let mut names = String::from("shop.desk.UseGrant");
    for i in 2..=readers {
        copies.push_str(&reader.replace("shop.desk.UseGrant", &format!("shop.desk.UseGrant{i}")));
        let _ = write!(names, ", shop.desk.UseGrant{i}");
    }
    let mut text = MODEL.replacen("shop.desk.UseGrant]", &format!("{names}]"), 1);
    let at = text
        .find("views:\n")
        .expect("the views follow the commands");
    text.insert_str(at, &copies);
    text
}

/// `(built, stopped, asked)`: the own-scenario arrangements carrying chosen values built, the
/// chosen-values runs the nesting bound stopped, and the arrangements carrying chosen values
/// asked for.
fn work(readers: usize) -> (Synthesis, (u64, u64, u64)) {
    let model = ir(&desk(readers));
    own_overlay_work(|| synthesize(&model))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn an_arrangement_many_readers_ask_for_is_built_once() {
    let (one, (built_one, _, asked_one)) = work(1);
    let (four, (built_four, stopped, asked_four)) = work(4);
    assert!(refusals(&one).is_empty(), "refusals: {:#?}", refusals(&one));
    assert!(
        refusals(&four).is_empty(),
        "refusals: {:#?}",
        refusals(&four)
    );
    assert_eq!(stopped, 0, "chosen-values runs stopped by the bound");
    assert!(
        asked_four > asked_one,
        "four readers ask for the arrangement more often than one: {asked_one} then {asked_four}"
    );
    assert!(built_one >= 1, "the arrangement is built: {built_one}");
    assert_eq!(
        built_four, built_one,
        "the arrangements built with one reader and with four (asked {asked_one} and \
         {asked_four} times)"
    );
}
