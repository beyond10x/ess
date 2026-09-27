//! A skipping aggregate is rendered with `skip_absent` in a semantic change (beyond10x/ess#148).
//!
//! `docs/design/aggregate-views.md`, "Absent values (ess/15)": `field-aggregate-changed` renders an
//! aggregate as `sum(duration, skip_absent)`, so starting or stopping to read an optional field is
//! a change a reviewer sees, at the diff format that already names the kind.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const ORDERS: &str =
    include_str!("../../ess-conformance/tests/fixtures/aggregate-optional-fields.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("orders.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn a_changed_skipping_aggregate_renders_skip_absent_at_diff_7() {
    let before = ir(ORDERS);
    let after = ir(&ORDERS.replacen(
        "aggregate: {sum: duration, skip_absent: true}",
        "aggregate: {max: duration, skip_absent: true}",
        1,
    ));
    let delta = diff(&before, &after).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/7", "{json}");
    assert!(
        json.contains(r#""before": "sum(duration, skip_absent)""#),
        "{json}"
    );
    assert!(
        json.contains(r#""after": "max(duration, skip_absent)""#),
        "{json}"
    );
}

#[test]
fn reading_a_field_that_became_optional_is_a_change_of_its_aggregate() {
    let head = ORDERS.split_once("views:\n").unwrap().0;
    let counted = |skip: &str| {
        format!(
            "{head}views:\n  - name: demo.orders.Durations\n    source: demo.orders.Order\n    group_by: [customer]\n    fields:\n      - {{name: customer, type: String}}\n      - {{name: durations, type: Integer, aggregate: {{count_distinct: duration{skip}}}}}\n"
        )
    };
    let required = counted("").replace(
        "      - {name: duration, type: Optional<Integer>}\n      - {name: group",
        "      - {name: duration, type: Integer}\n      - {name: group",
    );
    let json = diff(&ir(&required), &ir(&counted(", skip_absent: true")))
        .unwrap()
        .to_canonical_json();
    assert!(
        json.contains(r#""before": "count_distinct(duration)""#),
        "{json}"
    );
    assert!(
        json.contains(r#""after": "count_distinct(duration, skip_absent)""#),
        "{json}"
    );
}
