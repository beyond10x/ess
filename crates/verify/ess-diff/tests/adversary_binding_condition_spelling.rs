//! Adversary pass 1 against #268 slice 2: `binding/predicate-changed` for two spellings of one
//! condition that the source grammar gives one meaning — a one-element list (the `all` shorthand)
//! and an explicit one-element `all` around the same comparison. The design asks for "typed,
//! normalized before/after content"; a re-spelling with no semantic difference should not move a
//! binding's occurrences in the delta.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_diff::SemanticChange;
use ess_domain::{system::Source, RawSpecFile, Specification};

const MODEL: &str = include_str!("../../ess-conformance/tests/fixtures/binding-condition.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";

fn model(text: &str) -> EssIr {
    let specification = Specification::assemble([(
        Source::new("messages.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("messages.yaml", text);
    compile_locating(&specification, &sources, &["messages.yaml"]).unwrap()
}

fn with_condition(condition: &str) -> String {
    assert_eq!(MODEL.matches(WHERE).count(), 1);
    MODEL.replace(WHERE, condition).replace(
        "      order_id: event.order.id\n",
        "      order_id: event.message_id\n",
    )
}

fn binding_changes(before: &str, after: &str) -> Vec<String> {
    let delta = ess_diff::diff(&model(before), &model(after)).unwrap();
    delta
        .changes()
        .iter()
        .filter(|change| matches!(change, SemanticChange::Binding { .. }))
        .map(|change| format!("{} {}", change.id(), change.describe()))
        .collect()
}

#[test]
fn a_one_element_conjunction_is_the_same_condition_as_its_element() {
    let bare = with_condition("      where: event.kind == ship\n");
    let mut moved = Vec::new();
    for spelling in [
        "      where: [event.kind == ship]\n",
        "      where: {all: [event.kind == ship]}\n",
    ] {
        let changes = binding_changes(&bare, &with_condition(spelling));
        if !changes.is_empty() {
            moved.push(format!("{}: {changes:?}", spelling.trim()));
        }
    }
    let reordered = binding_changes(
        &with_condition("      where: [defined(event.order), event.kind == ship]\n"),
        &with_condition("      where: [event.kind == ship, defined(event.order)]\n"),
    );
    println!("reordered conjunction: {reordered:?}");
    assert_eq!(
        moved,
        Vec::<String>::new(),
        "re-spellings reported as changes"
    );
}
