//! `repeated: last_wins` on an `each:` entry (ess/35,
//! story:an-each-write-may-take-repeated-identities-in-list-order): declaring it admits lists the
//! earlier revision refused as undeclared, so `ess verify diff` reports the set effect `expanded`
//! (widening); dropping it reports `narrowed`. Any other move of the entry stays `changed`.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
const PLAIN: &str = "each: {in: input.applied, as: doc}";
const REPEATED: &str = "each: {in: input.applied, as: doc, repeated: last_wins}";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("feed.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The fixture at ess/35, keeping its `distinct:` refusal so both revisions validate, its entry
/// written `each`.
fn model(each: &str) -> String {
    assert!(EACH.contains(PLAIN), "the fixture holds {PLAIN}");
    EACH.replacen("format: ess/23", "format: ess/35", 1)
        .replacen(PLAIN, each, 1)
}

/// The relation of the one set-effect change between `before` and `after`.
fn set_effect(before: &str, after: &str) -> SemanticRelation {
    let delta = diff(&ir(before), &ir(after)).unwrap();
    let json = delta.to_canonical_json();
    let changes: Vec<_> = delta
        .changes()
        .iter()
        .filter(|change| change.kind() == "outcome-set-effect-changed")
        .collect();
    let [change] = changes.as_slice() else {
        panic!("one set-effect change: {json}")
    };
    assert_eq!(delta.changes().len(), 1, "{json}");
    change.relation()
}

#[test]
fn adding_the_key_widens_and_removing_it_narrows() {
    assert_eq!(
        set_effect(&model(PLAIN), &model(REPEATED)),
        SemanticRelation::Expanded,
        "declaring `repeated: last_wins` admits a list the earlier revision refused"
    );
    assert_eq!(
        set_effect(&model(REPEATED), &model(PLAIN)),
        SemanticRelation::Narrowed,
        "dropping it refuses a list the earlier revision admitted"
    );
}

#[test]
fn another_move_beside_the_key_stays_changed() {
    let after = model(REPEATED).replacen("revision: doc.revision}", "revision: 1}", 1);
    assert_eq!(set_effect(&model(PLAIN), &after), SemanticRelation::Changed);
}

/// The document carries the relation its own content derives, so it reads back; the line names
/// the key.
#[test]
fn a_widening_delta_reads_back() {
    let delta = diff(&ir(&model(PLAIN)), &ir(&model(REPEATED))).unwrap();
    let json = delta.to_canonical_json();
    assert!(json.contains(r#""relation": "expanded""#), "{json}");
    assert!(json.contains("repeated: last_wins"), "{json}");
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    EssDelta::try_from(raw).unwrap_or_else(|errors| panic!("{errors}\n{json}"));
}
