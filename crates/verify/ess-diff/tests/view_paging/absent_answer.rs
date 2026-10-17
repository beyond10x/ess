//! A view's answer for an identity no row carries (`absent:`, ess/25, beyond10x/ess#533) gained,
//! lost or changed is `view/<name>/absent-changed`, `ess-diff/19` vocabulary, never
//! `system/<name>/unclassified-changed`. A reader meets a different answer for an unknown identity,
//! so it is breaking for readers and compatible for callers and history.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, Compatibility, EssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const FLAGS: &str = include_str!("../../../ess-conformance/tests/fixtures/view-absent-row.yaml");
const ROW: &str = "    absent:\n      row: {paused: false, level: 0, state: Set}\n";
const CHANGE: &str = "view/demo.status.FlagById/absent-changed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("flags.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn ids(delta: &EssDelta) -> Vec<String> {
    delta
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

fn answers(delta: &EssDelta) -> (SemanticRelation, [Compatibility; 3]) {
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == CHANGE)
        .unwrap_or_else(|| panic!("`{CHANGE}` in {}", delta.to_canonical_json()));
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    (
        change.relation(),
        [
            compatibility.callers(),
            compatibility.readers(),
            compatibility.history(),
        ],
    )
}

use Compatibility::{Breaking as B, Compatible as C};

#[test]
fn gaining_losing_or_changing_the_default_row_is_absent_changed() {
    let with = ir(FLAGS);
    let without = ir(&FLAGS.replace(ROW, ""));
    let paused = ir(&FLAGS.replace(
        ROW,
        "    absent:\n      row: {paused: true, level: 0, state: Set}\n",
    ));
    for (before, after, said) in [
        (
            &without,
            &with,
            "a default row (`level` `0`, `paused` `false`, `state` `Set`), answered no row",
        ),
        (&with, &without, "no row, answered a default row"),
        (&with, &paused, "`paused` `true`"),
    ] {
        let delta = classified(before, after).unwrap();
        let json = delta.to_canonical_json();
        assert_eq!(ids(&delta), [CHANGE], "{json}");
        assert_eq!(
            answers(&delta),
            (SemanticRelation::Changed, [C, B, C]),
            "{json}"
        );
        assert!(json.contains("\"ess-diff/19\""), "{json}");
        let text = ess_diff::render::text(&delta);
        assert!(text.contains(said), "{text}");
    }
    // A model declaring `absent:` reports nothing against itself.
    let same = classified(&with, &with).unwrap();
    assert_eq!(same.changes().len(), 0, "{:?}", same.changes());
}
