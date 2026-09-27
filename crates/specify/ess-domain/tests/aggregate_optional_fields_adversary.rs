//! Adversary cases for beyond10x/ess#148 (`skip_absent`, optional group keys, `ess/15`).
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;

const ORDERS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-optional-fields.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("orders.yaml"), raw)])
}

fn view(body: &str) -> String {
    let head = ORDERS.split_once("views:\n").unwrap().0;
    format!("{head}views:\n  - name: demo.orders.V\n    source: demo.orders.Order\n{body}")
}

/// A group key whose view field is declared required over a source field that may be absent is
/// still a key that may be absent: the rows it partitions can lack it. Below `ess/15` it must not
/// slip past the format gate because the gate reads the declared type instead of the source's.
#[test]
fn adversary_a_required_declared_key_over_an_optional_source_field_is_refused_below_ess_15() {
    let text = view(
        "    group_by: [group]\n    fields:\n      - {name: group, type: demo.orders.Group}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    )
    .replace("format: ess/15", "format: ess/14");
    assert!(
        assemble(&text).is_err(),
        "a key over `group: Optional<demo.orders.Group>` was admitted at ess/14 because its view \
         field is declared `demo.orders.Group`"
    );
}

/// The same view at `ess/15`: admitting it declares a required key while synthesis and SQL give it
/// a `null` group, so the declared type contradicts the rows.
#[test]
fn adversary_a_required_declared_key_over_an_optional_source_field_is_refused_at_ess_15() {
    let text = view(
        "    group_by: [group]\n    fields:\n      - {name: group, type: demo.orders.Group}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    );
    assert!(
        assemble(&text).is_err(),
        "a key declared `demo.orders.Group` over `group: Optional<demo.orders.Group>` was admitted"
    );
}

#[test]
fn adversary_skip_absent_written_before_the_function_is_read() {
    let text = view(
        "    group_by: [customer]\n    fields:\n      - {name: customer, type: String}\n      - {name: total, type: Optional<Integer>, aggregate: {skip_absent: true, sum: duration}}\n",
    );
    assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
}
