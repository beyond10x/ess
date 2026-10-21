//! A guard says what an absent `Optional` compares as (`ess/29`, beyond10x/ess#620):
//! `coalesce(is_error, true) == false`, a function of one fact on the left of a compact comparison.
//!
//! The spelling was a parse refusal at every format. From `ess/29` it reads as the structured
//! `{compare: {left: {expr: "…"}, op, right}}`, which `ess/27` admitted; below `ess/29` it is
//! refused naming `ess/29` and the structured form. Text on the left that is no function of one
//! fact, and a call on the left outside a command outcome's guards, are refused by name.

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/coalesce-guard.yaml");

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("closing.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

fn at(format: &str, body: &str) -> String {
    body.replacen("format: ess/29", &format!("format: {format}"), 1)
}

#[test]
fn a_call_on_the_left_of_a_guard_validates_from_ess_29() {
    spec(MODEL).unwrap_or_else(|errors| panic!("{errors}"));
    // The input namespace reads the same input.
    spec(&MODEL.replace(
        "coalesce(is_error, true) == false",
        "coalesce(input.is_error, true) == false",
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn below_ess_29_the_spelling_is_refused_naming_ess_29_and_the_structured_form_validates() {
    let errors = refused(&at("ess/28", MODEL));
    assert!(
        errors.contains("unsupported_format_version")
            && errors.contains("ess/29")
            && errors.contains("{compare: {left: {expr:"),
        "{errors}"
    );
    let structured = at("ess/28", MODEL)
        .replace(
            "when: coalesce(stop_reason, \"none\") == \"budget\"",
            "when: {compare: {left: {expr: 'coalesce(stop_reason, \"none\")'}, op: eq, right: budget}}",
        )
        .replace(
            "            - coalesce(is_error, true) == false",
            "            - {compare: {left: {expr: \"coalesce(is_error, true)\"}, op: eq, right: false}}",
        )
        .replace(
            "when: coalesce(attempts, 0) < 3",
            "when: {compare: {left: {expr: \"coalesce(attempts, 0)\"}, op: lt, right: 3}}",
        );
    spec(&structured).unwrap_or_else(|errors| panic!("{errors}\n{structured}"));
}

#[test]
fn text_on_the_left_that_is_no_function_of_one_fact_is_refused() {
    for written in [
        "coalesce(attempts, stop_reason) < 3",
        "unknown_fn(attempts) < 3",
        "coalesce(3, 4) < 3",
    ] {
        let errors = refused(&MODEL.replace("coalesce(attempts, 0) < 3", written));
        assert!(
            errors.contains("type_mismatch") && errors.contains("the left side of a comparison"),
            "{written}: {errors}"
        );
    }
}

#[test]
fn a_mistyped_default_is_refused_naming_coalesce() {
    let errors =
        refused(&MODEL.replace("coalesce(attempts, 0) < 3", "coalesce(attempts, \"x\") < 3"));
    assert!(
        errors.contains("type_mismatch") && errors.contains("`coalesce` takes"),
        "{errors}"
    );
}

#[test]
fn a_call_on_the_left_outside_a_command_guard_is_refused_by_name() {
    let with_entity = MODEL.replace(
        "actors:",
        "entities:
  - name: demo.closing.Record
    identity: {name: id, type: Uuid}
    fields:
      - {name: retries, type: Optional<Integer>}
    invariants:
      - coalesce(retries, 0) >= 0
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:",
    );
    let errors = refused(&with_entity);
    assert!(
        errors.contains("unsupported_construct") && errors.contains("command outcome's guards"),
        "{errors}"
    );
}
