//! A binding states an attempt bound and which failures are final (`on_failure: {retry:
//! {attempts, final}}`, ess/16; beyond10x/ess#165, `docs/design/binding-delivery-guarantees.md`).
//!
//! `attempts` counts invocations including the first, and is at least two: one attempt is `drop`.
//! `final` names refusals of the invoked command — an outcome that carries an `error:`, or the error
//! itself — that end the retry at once. Below `ess/16` the block is refused with
//! `unsupported_format_version`; `on_failure: retry` written bare keeps meaning what it meant.

use ess_domain::binding::{BindingName, Failure};
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/bounded-retry.yaml");
const BLOCK: &str = "retry: {attempts: 3, final: [demo.ledger.Unknown]}";

fn with_policy(policy: &str) -> String {
    let model = MODEL.replace(BLOCK, policy);
    assert_ne!(
        model, MODEL,
        "the fixture carries the block this test rewrites"
    );
    model
}

fn parse(body: &str) -> Result<ess_domain::spec::RawSpecFile, String> {
    ess_domain::spec::RawSpecFile::parse(body).map_err(|error| error.to_string())
}

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = parse(body).unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("ledger.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn assert_code(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

fn binding(spec: &Specification) -> serde_json::Value {
    let name = BindingName::new("notify-ledger").expect("a binding name");
    let binding = &spec.bindings()[&name];
    serde_json::to_value(binding).expect("a binding serializes")
}

#[test]
fn a_bounded_retry_is_read_under_ess_16() {
    let spec = admitted(MODEL);
    let name = BindingName::new("notify-ledger").expect("a binding name");
    assert_eq!(spec.bindings()[&name].failure, Failure::Retry);
    let binding = binding(&spec);
    assert_eq!(binding["failure"], "retry");
    assert_eq!(
        binding["retry"],
        serde_json::json!({"attempts": 3, "final": ["demo.ledger.Unknown"]}),
        "{binding}"
    );
}

#[test]
fn a_bare_retry_keeps_its_meaning_and_its_bytes() {
    let spec = admitted(&with_policy("retry"));
    let binding = binding(&spec);
    assert_eq!(binding["failure"], "retry");
    assert!(
        binding.get("retry").is_none(),
        "an unbounded retry serializes as it did: {binding}"
    );
}

#[test]
fn final_is_optional_and_may_name_an_outcome() {
    let spec = admitted(&with_policy("retry: {attempts: 4}"));
    assert_eq!(binding(&spec)["retry"], serde_json::json!({"attempts": 4}));

    let spec = admitted(&with_policy("retry: {attempts: 2, final: [rejected]}"));
    assert_eq!(
        binding(&spec)["retry"],
        serde_json::json!({"attempts": 2, "final": ["rejected"]})
    );
}

#[test]
fn below_ess_16_the_bound_is_refused_by_format() {
    let model = MODEL.replace("format: ess/16", "format: ess/15");
    let errors = refused(&model);
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "binding.notify-ledger.on_failure.retry",
    );
    assert_code(&errors, ValidationCode::UnsupportedFormatVersion, "ess/16");

    // The bare word is what ess/15 already had.
    let bare = model.replace(BLOCK, "retry");
    admitted(&bare);
}

#[test]
fn a_bound_of_fewer_than_two_attempts_is_refused() {
    for attempts in [0, 1] {
        let errors = refused(&with_policy(&format!("retry: {{attempts: {attempts}}}")));
        assert_code(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "binding.notify-ledger.on_failure.retry.attempts",
        );
        assert!(
            errors.to_string().contains("drop"),
            "the hint names the word that means one attempt: {errors}"
        );
    }
}

#[test]
fn a_final_name_that_is_no_refusal_of_the_invoked_command_is_refused() {
    // An error the command never reports, an outcome it does not have, and its success branch:
    // none of them is a refusal the retry could end on.
    for name in ["demo.ledger.Missing", "vanished", "recorded"] {
        let errors = refused(&with_policy(&format!(
            "retry: {{attempts: 3, final: [{name}]}}"
        )));
        assert_code(
            &errors,
            ValidationCode::UndeclaredReference,
            "binding.notify-ledger.on_failure.retry.final",
        );
        assert!(errors.to_string().contains(name), "{errors}");
    }
}

#[test]
fn a_final_name_written_twice_is_refused() {
    let errors = refused(&with_policy(
        "retry: {attempts: 3, final: [rejected, rejected]}",
    ));
    assert_code(
        &errors,
        ValidationCode::DuplicateDeclaration,
        "binding.notify-ledger.on_failure.retry.final",
    );
}

#[test]
fn the_block_is_closed_and_only_retry_and_escalate_take_one() {
    let unknown = parse(&with_policy("retry: {attempts: 3, backoff: 2s}"))
        .expect_err("an unknown key under retry is refused");
    assert!(unknown.contains("backoff"), "{unknown}");

    let missing = parse(&with_policy("retry: {final: [rejected]}"))
        .expect_err("a bound without attempts is refused");
    assert!(missing.contains("attempts"), "{missing}");

    let dropped = parse(&with_policy("drop: {attempts: 3}")).expect_err("drop takes no block");
    assert!(dropped.contains("drop"), "{dropped}");
}

/// The published schema states both limits the reader enforces on the block, so an editor
/// validating against it refuses what [`a_bound_of_fewer_than_two_attempts_is_refused`] and
/// [`a_final_name_written_twice_is_refused`] refuse.
#[test]
fn the_schema_states_the_floor_and_the_uniqueness_the_reader_enforces() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../schemas/generated/ess.schema.json"
    ))
    .expect("the schema is JSON");
    let bound = &schema["definitions"]["RetryBound"]["properties"];
    let floor = f64::from(ess_domain::binding::retry::RetryBound::MIN_ATTEMPTS);
    assert_eq!(
        bound["attempts"]["minimum"].as_f64(),
        Some(floor),
        "{bound:#}"
    );
    assert_eq!(
        bound["final"]["uniqueItems"],
        serde_json::Value::Bool(true),
        "{bound:#}"
    );
}
