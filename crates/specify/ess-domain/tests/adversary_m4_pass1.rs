//! Adversary pass 1 for one event per changed row (ess/25, beyond10x/ess#531,
//! `docs/design/set-effects-over-filtered-instances.md`, "One event per changed row").
//!
//! Each case reads the page's refusals as the contract: below `ess/25` the entry is refused alone,
//! whatever else its filter or payload says; `{changed: …}` anywhere but a whole payload field of a
//! per-row event is `unsupported_construct`; an empty `when_state:` is `conflicting_declaration`.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects-per-row.yaml");
const ENTRY: &str = "{event: demo.desk.Left, each: changed, when_state: Busy}";

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the model parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn edited(body: &str, from: &str, to: &str) -> String {
    assert!(body.contains(from), "the model holds:\n{from}");
    body.replacen(from, to, 1)
}

/// Exactly one error, with `code`, at a location containing `at`.
fn alone(errors: &ValidationErrors, code: ValidationCode, at: &str) {
    let all = errors.as_slice();
    assert_eq!(all.len(), 1, "one refusal, alone:\n{errors}");
    assert_eq!(all[0].code, code, "{errors}");
    assert!(
        all[0].location.contains(at),
        "refused at `{at}`, got:\n{errors}"
    );
}

fn has(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

fn ess_24(body: &str) -> String {
    edited(body, "format: ess/25", "format: ess/24")
}

/// Below `ess/25` the refusal "comes alone": an undeclared state in `when_state:` and a field the
/// entity lacks in `{changed: …}` add nothing to it.
#[test]
fn below_ess_25_an_entry_with_a_bad_filter_and_a_bad_field_is_refused_alone() {
    let body = edited(
        &edited(
            MODEL,
            ENTRY,
            "{event: demo.desk.Left, each: changed, when_state: Away}",
        ),
        "{changed: call_ref}",
        "{changed: phone}",
    );
    alone(
        &refused(&ess_24(&body)),
        ValidationCode::UnsupportedFormatVersion,
        "outcomes.closed-all.emits[1]",
    );
}

/// Below `ess/25`, on an outcome with one subject, the refusal names `ess/25` at the entry; beside
/// the fixture's own per-row entry it adds one refusal of the same code and nothing else.
#[test]
fn below_ess_25_an_entry_on_a_single_subject_outcome_is_refused_alone() {
    let body = edited(
        MODEL,
        "        instance: user_id
        emits: [demo.desk.CallTaken]",
        "        instance: user_id
        emits: [{event: demo.desk.CallTaken, each: changed}]",
    );
    let errors = refused(&ess_24(&body));
    let all = errors.as_slice();
    assert_eq!(all.len(), 2, "one refusal per entry:\n{errors}");
    assert!(
        all.iter()
            .all(|error| error.code == ValidationCode::UnsupportedFormatVersion),
        "{errors}"
    );
    assert!(
        all.iter()
            .any(|error| error.location.contains("outcomes.taken.emits[0]")
                && error.to_string().contains("ess/25")),
        "{errors}"
    );
}

/// "An empty list ... is `conflicting_declaration`."
#[test]
fn an_empty_state_filter_is_a_conflicting_declaration() {
    let errors = refused(&edited(
        MODEL,
        ENTRY,
        "{event: demo.desk.Left, each: changed, when_state: []}",
    ));
    has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "when_state",
    );
}

/// "... in `sets:` ... it is `unsupported_construct`."
#[test]
fn changed_in_sets_is_refused() {
    let body = edited(
        MODEL,
        "          demo.desk.Left: {user_id: {changed: user_id}, call_ref: {changed: call_ref}, was: {changed: state}}\n",
        "          demo.desk.Left: {user_id: {changed: user_id}, call_ref: {changed: call_ref}, was: {changed: state}}\n        sets:\n          call_ref: {changed: call_ref}\n",
    );
    has(
        &refused(&body),
        ValidationCode::UnsupportedConstruct,
        "{changed: …}",
    );
}
