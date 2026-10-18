//! `unknown_instance:` beside a `when_related:` guard (source format `ess/27`, downstream report,
//! 2026-10-18).
//!
//! A command addressing a row by its input and reading another entity's row by its input answers an
//! unknown addressed identity at step 3 of the precedence order: after an `exists: false` over the
//! related row (step 1) and the input-guarded refusals (step 2), before the related row's predicate
//! refusals (step 5). From `ess/27` the pairing validates; below it the pairing stays
//! `conflicting_declaration`, its hint naming `ess/27`.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/related-guard-unknown-instance.yaml"
);

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("model.yaml"), raw)])
}

fn at(format: &str) -> String {
    let text = MODEL.replace("format: ess/27\n", &format!("format: {format}\n"));
    assert_ne!(text, MODEL, "the fixture declares ess/27");
    text
}

#[test]
fn unknown_instance_beside_a_related_guard_validates_from_ess_27() {
    if let Err(errors) = assemble(MODEL) {
        panic!("{errors}");
    }
}

#[test]
fn below_ess_27_the_pairing_is_refused_naming_ess_27() {
    let errors = assemble(&at("ess/26")).expect_err("refused below ess/27");
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.code == ValidationCode::ConflictingDeclaration)
        .unwrap_or_else(|| panic!("{errors}"));
    let written = refusal.to_string();
    assert!(
        written.contains("`unknown_instance` branch") && written.contains("format: ess/27"),
        "{written}"
    );
}

/// The relaxation is the `unknown_instance:` branch's alone: `when_subject:` beside the same guard
/// stays refused at `ess/27`.
#[test]
fn a_subject_guard_beside_a_related_guard_stays_refused_at_ess_27() {
    let text = MODEL.replace(
        "      - name: unknown-seat\n        unknown_instance: true\n        error: demo.seats.UnknownSeat\n",
        "      - name: unknown-seat\n        when_subject: {predicate: tier_id == input.tier_id}\n        error: demo.seats.UnknownSeat\n",
    );
    assert_ne!(text, MODEL);
    let errors = assemble(&text).expect_err("refused");
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::ConflictingDeclaration
                && error.to_string().contains("`when_subject` guard")
        }),
        "{errors}"
    );
}
