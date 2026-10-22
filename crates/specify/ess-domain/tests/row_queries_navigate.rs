//! What validation decides about reading through a declared reference (`ess/28`,
//! beyond10x/ess#557, beyond10x/ess#564, `docs/design/generic-language-features.md` § G2,
//! navigation): `{related: {via, field: state}}` as a value, a view field
//! `field: {related: {via, field}}`, and a filter path `<via>.<field>`, one hop.

const DESK: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/navigate-a-reference.yaml");

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

/// `DESK` with `from` replaced by `to`, after checking `from` is there.
fn with(from: &str, to: &str) -> String {
    assert!(DESK.contains(from), "the fixture holds `{from}`");
    DESK.replacen(from, to, 1)
}

/// `DESK` at another source format.
fn at(format: &str) -> String {
    with("format: ess/28", &format!("format: {format}"))
}

const VIEWS: &str = "views:\n";

/// `DESK` with one more view declared first.
fn with_view(view: &str) -> String {
    with(VIEWS, &format!("{VIEWS}{view}"))
}

#[test]
fn the_fixture_validates_at_ess_28() {
    spec(DESK).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn every_navigation_is_refused_below_ess_28_naming_it() {
    let errors = refused(&at("ess/27"));
    assert!(errors.contains("unsupported_format_version"), "{errors}");
    // The payload's state read, the view field and the filter path, each refused by name.
    for named in [
        "field: state}}` reads the lifecycle state",
        "`desk_state` reads the row a reference names",
        "`desk_id.state` reads the row a reference names",
    ] {
        assert!(errors.contains(named), "`{named}` is refused: {errors}");
    }
    assert!(errors.matches("ess/28").count() >= 3, "{errors}");
}

#[test]
fn a_related_state_is_typed_as_its_entitys_state() {
    let errors = refused(&with(
        "      - {name: current, type: relstate.desk.Desk.State}",
        "      - {name: current, type: String}",
    ));
    assert!(
        errors.contains("type_mismatch") && errors.contains("relstate.desk.Desk.State"),
        "{errors}"
    );
}

#[test]
fn a_view_field_read_through_a_reference_is_optional() {
    let errors = refused(&with(
        "        type: Optional<relstate.desk.Desk.State>",
        "        type: relstate.desk.Desk.State",
    ));
    assert!(
        errors.contains("type_mismatch")
            && errors.contains("absent where the reference is or names no row")
            && errors.contains("declare `desk_state` as `Optional<relstate.desk.Desk.State>`"),
        "{errors}"
    );
}

#[test]
fn a_view_field_reads_a_field_of_the_referenced_row() {
    let errors = refused(&with(
        "field: {related: {via: desk_id, field: label}}",
        "field: {related: {via: desk_id, field: colour}}",
    ));
    assert!(
        errors.contains("undeclared_reference")
            && errors.contains("`colour` is not a field of `relstate.desk.Desk`")
            && errors.contains("`desk_id`, `label`, `state`"),
        "{errors}"
    );
}

#[test]
fn a_view_field_reads_through_a_reference_only() {
    let errors = refused(&with(
        "field: {related: {via: desk_id, field: label}}",
        "field: {related: {via: ticket_id, field: label}}",
    ));
    assert!(
        errors.contains("type_mismatch") && errors.contains("names no one entity's row"),
        "{errors}"
    );
    let errors = refused(&with(
        "field: {related: {via: desk_id, field: label}}",
        "field: {related: {via: desk, field: label}}",
    ));
    assert!(
        errors.contains("undeclared_reference")
            && errors.contains("`desk`, which is not a field of `relstate.desk.Ticket`"),
        "{errors}"
    );
}

#[test]
fn a_view_field_read_through_a_reference_has_a_name_of_its_own() {
    let errors = refused(&with(
        "      - name: desk_label\n        type: Optional<String>",
        "      - name: state\n        type: Optional<String>",
    ));
    assert!(
        errors.contains("conflicting_declaration")
            && errors.contains("`state` is a field of `relstate.desk.Ticket`"),
        "{errors}"
    );
}

#[test]
fn a_filter_path_through_a_reference_reads_a_field_of_the_referenced_row() {
    let errors = refused(&with(
        "    filter: desk_id.state == Open",
        "    filter: desk_id.colour == Open",
    ));
    assert!(errors.contains("desk_id.colour"), "{errors}");
    let errors = refused(&with(
        "    filter: desk_id.state == Open",
        "    filter: desk_id.state == Shut",
    ));
    assert!(
        errors.contains("Shut"),
        "an unknown state is refused: {errors}"
    );
}

#[test]
fn navigation_beside_an_aggregate_paging_absent_or_an_order_is_refused_by_name() {
    let aggregate = with_view(
        "  - name: relstate.desk.TicketsByDesk
    source: relstate.desk.Ticket
    consistency: read_your_writes
    fields:
      - {name: tickets, type: Integer, aggregate: {count: {}}}
    filter: desk_id.state == Open
",
    );
    let errors = refused(&aggregate);
    assert!(
        errors.contains("unsupported_construct") && errors.contains("an aggregate view"),
        "{errors}"
    );
    let ordered = with(
        "        field: {related: {via: desk_id, field: label}}\n",
        "        field: {related: {via: desk_id, field: label}}\n    order_by: [desk_label]\n",
    );
    let errors = refused(&ordered);
    assert!(
        errors.contains("unsupported_construct") && errors.contains("an order key"),
        "{errors}"
    );
}

/// A view over two entities, the shape beyond10x/ess#564 wrote.
const TWO_SOURCES: &str = "  - name: relstate.desk.TicketDesks
    source: [relstate.desk.Ticket, relstate.desk.Desk]
    consistency: read_your_writes
    fields:
      - {name: ticket_id, type: relstate.desk.TicketId}
      - {name: label, type: String}
";

#[test]
fn a_view_source_list_is_refused_by_name_at_every_format() {
    for format in ["ess/16", "ess/27", "ess/28", "ess/29", "ess/30"] {
        let document =
            with_view(TWO_SOURCES).replacen("format: ess/28", &format!("format: {format}"), 1);
        let errors = refused(&document);
        assert!(
            errors.contains(
                "[unsupported_construct] view.relstate.desk.TicketDesks.source: \
                 `relstate.desk.TicketDesks` names 2 sources"
            ) && errors.contains("a view has one source"),
            "{format}: {errors}"
        );
        assert!(
            errors.contains("{related: {via, field}}"),
            "{format}: the hint names navigation: {errors}"
        );
        assert!(
            !errors.contains("invalid type: sequence"),
            "{format}: no parse error: {errors}"
        );
    }
}

#[test]
fn a_view_source_string_is_unchanged() {
    spec(DESK).unwrap_or_else(|errors| panic!("{errors}"));
    let raw = ess_domain::spec::RawSpecFile::parse(DESK).expect("the fixture parses");
    let written = serde_json::to_value(&raw.views[0]).expect("the view serializes");
    assert_eq!(
        written["source"],
        serde_json::json!("relstate.desk.Ticket"),
        "a one-entity source is written back as the string it was"
    );
}
