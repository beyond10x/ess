//! A guard over the rows a `List` of references names, `when_related: {via: input.<list>, exists:
//! false}`, and the first element naming no row, `{missing: input.<list>}` (`ess/28`,
//! beyond10x/ess#558): what validation admits and refuses.

const LEDGER: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/list-of-references.yaml");

/// The fixture's payload source naming the first missing element, as written.
const MISSING: &str = "{service_id: {missing: input.services}}";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("ledger.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

fn with(from: &str, to: &str) -> String {
    assert!(LEDGER.contains(from), "the fixture holds `{from}`");
    LEDGER.replacen(from, to, 1)
}

#[test]
fn the_fixture_validates_at_ess_28() {
    let spec = spec(LEDGER).unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"ledger.desk.OpenIncident".parse().unwrap()];
    let refusal = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "unknown-service")
        .expect("the refusal");
    assert!(matches!(
        &refusal.condition,
        ess_domain::command::OutcomeCondition::Related { via, .. } if via.field() == "services"
    ));
    assert_eq!(
        refusal.error_payload.get("service_id"),
        Some(&ess_domain::command::PayloadSource::MissingElement {
            field: "services".to_owned()
        })
    );
}

#[test]
fn below_ess_28_the_list_guard_is_refused_naming_it() {
    let errors = refused(&with("format: ess/28", "format: ess/27"));
    assert!(
        errors.contains("`input.services` is `List<ledger.desk.ServiceId>`")
            && errors.contains("ess/28"),
        "{errors}"
    );
}

#[test]
fn a_predicate_over_the_rows_a_list_names_is_refused_by_name() {
    let errors = refused(&with(
        "      - name: opened\n",
        "      - name: unlabelled\n        when_related: {via: input.services, predicate: label == x}\n        error: ledger.desk.UnknownService\n      - name: opened\n",
    ));
    assert!(
        errors.contains("unsupported_construct")
            && errors.contains("tested for a missing row only"),
        "{errors}"
    );
}

#[test]
fn the_missing_element_is_read_only_on_the_branch_guarding_the_list() {
    // On the accepting branch, which no list guard selects.
    let moved = with(
        "        payload: {ledger.desk.IncidentOpened: {incident_id: {generated: true}}}",
        "        payload: {ledger.desk.IncidentOpened: {incident_id: {missing: input.services}}}",
    );
    let errors = refused(&moved);
    assert!(
        errors.contains("{missing: input.services}") && errors.contains("not guarded by"),
        "{errors}"
    );
}

#[test]
fn the_missing_element_is_typed_as_the_lists_element() {
    let errors = refused(&with(
        "      - {name: service_id, type: ledger.desk.ServiceId}\nevents:",
        "      - {name: service_id, type: String}\nevents:",
    ));
    assert!(
        errors.contains("type_mismatch") && errors.contains("is `ledger.desk.ServiceId`"),
        "{errors}"
    );
}

#[test]
fn below_ess_28_the_missing_source_is_the_nested_mapping_it_was() {
    // Below ess/28 `{missing: …}` reads as a nested mapping, refused over a non-struct field; the
    // list guard is refused too, so both name what they are under the older format.
    let errors = refused(&with("format: ess/28", "format: ess/27"));
    assert!(
        !errors.contains("{missing: input.services}` reads"),
        "{errors}"
    );
    assert!(LEDGER.contains(MISSING));
}
