//! A view field that comes to read the row a reference names, or reads another one (`ess/28`,
//! beyond10x/ess#564, `docs/design/generic-language-features.md` § G2), is a typed change,
//! `view/<v>/field-source-changed {field, before, after}`, breaking for readers, and `ess-diff/21`
//! vocabulary; a filter navigating a reference is reported as the filter change it is.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, Compatibility, EssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const DESK: &str = include_str!("../../ess-conformance/tests/fixtures/navigate-a-reference.yaml");

const SOURCE: &str = "view/relstate.desk.Tickets/field-source-changed/desk_label";
const FILTER: &str = "view/relstate.desk.TicketsAtOpenDesks/filter-changed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn with(from: &str, to: &str) -> EssIr {
    assert!(DESK.contains(from), "the fixture holds `{from}`");
    ir(&DESK.replacen(from, to, 1))
}

fn ids(delta: &EssDelta) -> Vec<String> {
    delta
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

use Compatibility::{Breaking as B, Compatible as C};

#[test]
fn a_field_reading_another_field_of_the_referenced_row_breaks_readers() {
    let before = ir(DESK);
    let after = with(
        "      - name: desk_label\n        type: Optional<String>\n        field: {related: {via: desk_id, field: label}}",
        "      - name: desk_label\n        type: Optional<relstate.desk.DeskId>\n        field: {related: {via: desk_id, field: desk_id}}",
    );
    let delta = classified(&before, &after).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(
        ids(&delta),
        [
            SOURCE,
            "view/relstate.desk.Tickets/field-type-changed/desk_label"
        ],
        "{json}"
    );
    let change = &delta.changes()[0];
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    assert_eq!(
        [
            compatibility.callers(),
            compatibility.readers(),
            compatibility.history()
        ],
        [C, B, C],
        "{json}"
    );
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let changed = &document["changes"][0]["change"]["changed"];
    assert_eq!(changed["field"], "desk_label", "{json}");
    assert_eq!(changed["before"], "desk_id.label", "{json}");
    assert_eq!(changed["after"], "desk_id.desk_id", "{json}");
    assert_eq!(document["format"], "ess-diff/21", "{json}");
    let text = ess_diff::render::text(&delta);
    assert!(
        text.contains("field `desk_label` reads `desk_id.desk_id` through a reference"),
        "{text}"
    );
}

#[test]
fn a_filter_navigating_a_reference_is_a_filter_change() {
    let delta = classified(
        &ir(DESK),
        &with(
            "    filter: desk_id.state == Open",
            "    filter: desk_id.state == Closed",
        ),
    )
    .unwrap();
    assert_eq!(ids(&delta), [FILTER], "{}", delta.to_canonical_json());
}
