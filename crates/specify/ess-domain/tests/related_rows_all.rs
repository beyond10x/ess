//! A branch requiring two related rows at once, `when_related: {all: [{via, predicate}, …]}`
//! (`ess/28`, downstream report 2026-10-19, finding 11): what validation admits and refuses.

const WIRING: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/two-rows-one-branch.yaml");

/// The fixture's first `all:` branch, as written.
const ALL: &str = "        when_related:
          all:
            - {via: input.from_port, predicate: direction == inbound}
            - {via: input.to_port, predicate: direction == inbound}
";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("wiring.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

/// The fixture with its first `all:` branch written as `all`.
fn with_all(all: &str) -> String {
    assert!(WIRING.contains(ALL), "the fixture holds its `all:` branch");
    WIRING.replacen(ALL, all, 1)
}

#[test]
fn the_fixture_validates_at_ess_28_and_keeps_both_rows() {
    let spec = spec(WIRING).unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"wiring.board.Connect".parse().unwrap()];
    let branch = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "both-inbound")
        .expect("the branch");
    let ess_domain::command::OutcomeCondition::Related { via, also, .. } = &branch.condition else {
        panic!("a related guard: {:?}", branch.condition)
    };
    assert_eq!(via.field(), "from_port");
    assert_eq!(
        also.iter().map(|row| row.via.field()).collect::<Vec<_>>(),
        ["to_port"]
    );
}

#[test]
fn below_ess_28_the_branch_is_refused_naming_it() {
    let errors = refused(&WIRING.replacen("format: ess/28", "format: ess/27", 1));
    assert!(
        errors.contains("unsupported_format_version")
            && errors.contains("several related rows at once (`all:`)")
            && errors.contains("ess/28"),
        "{errors}"
    );
}

#[test]
fn a_row_listed_needs_its_own_missing_row_answer() {
    let errors = refused(&WIRING.replacen(
        "      - name: no-sink
        when_related: {via: input.to_port, exists: false}
        error: wiring.board.SinkNotFound
",
        "",
        1,
    ));
    assert!(
        errors.contains("non_exhaustive_branches") && errors.contains("input.to_port"),
        "{errors}"
    );
}

#[test]
fn every_element_is_a_row_an_input_names_with_a_predicate() {
    for (all, expected) in [
        (
            "        when_related:
          all:
            - {via: input.from_port, predicate: direction == inbound}
",
            "fewer than two rows",
        ),
        (
            "        when_related:
          all:
            - {via: input.from_port, predicate: direction == inbound}
            - {via: input.from_port, predicate: direction == outbound}
",
            "lists `input.from_port` twice",
        ),
        (
            "        when_related:
          all:
            - {via: input.from_port, predicate: direction == inbound}
            - {via: input.to_port, exists: false}
",
            "something other than a row an input identity names",
        ),
        (
            "        when_related:
          via: input.from_port
          all:
            - {via: input.from_port, predicate: direction == inbound}
            - {via: input.to_port, predicate: direction == inbound}
",
            "beside another key",
        ),
        (
            "        when_related:
          all:
            - {via: input.from_port, predicate: direction == inbound}
            - {via: input.to_port, predicate: colour == inbound}
",
            "colour",
        ),
    ] {
        let errors = refused(&with_all(all));
        assert!(errors.contains(expected), "`{expected}` in {errors}");
    }
}
