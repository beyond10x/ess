//! Adversary pass 1 against beyond10x/ess#304 slice 2 (`when_related: {via: <stored field>}`,
//! `ess/22`): what a document below ess/22 is told.
//!
//! Through ess/21 every `via` that is not `input.<field>` was refused `type_mismatch`, "reads the row
//! an input field names, one hop". The unit keeps that refusal except "where the command could read
//! the field from ess/22", where it names ess/22 instead. A stored field that is no entity's
//! identity is one the command can never read, at ess/22 or any other format: telling its author to
//! move to ess/22 sends them to a format that refuses it too.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const STORED_REFERENCE: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/related-guard-stored-reference.yaml"
);

const WRONG_STATE: &str =
    "      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("stored-reference.yaml"), raw)])
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The fixture with a stored `title: String` beside `blocked_by`, its guards reading `via`, at
/// `format`, without the `wrong_state` branch (refused beside a related guard below ess/22).
fn reading(via: &str, format: &str) -> String {
    let text = replaced(
        STORED_REFERENCE,
        "      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}\n    lifecycle:",
        "      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}\n      - {name: title, type: Optional<String>}\n    lifecycle:",
    );
    let text = replaced(&text, "via: blocked_by", &format!("via: {via}"));
    let text = replaced(&text, WRONG_STATE, "");
    text.replace("format: ess/22\n", &format!("format: {format}\n"))
}

#[test]
fn adv304s_below_ess_22_a_non_identity_stored_field_keeps_its_old_refusal() {
    // ess/22 itself refuses the read: `title` is no entity's identity.
    let at_22 = refused(&reading("title", "ess/22"));
    assert!(
        at_22
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch),
        "ess/22 refuses a stored `title` as no identity: {at_22}"
    );
    for format in ["ess/21", "ess/18"] {
        let errors = refused(&reading("title", format));
        assert!(
            !errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::UnsupportedFormatVersion),
            "{format}: a field no format lets `when_related` read is not refused as needing ess/22: \
             {errors}"
        );
        assert!(
            errors.as_slice().iter().any(|error| {
                error.code == ValidationCode::TypeMismatch
                    && error.to_string().contains("an input field names, one hop")
            }),
            "{format}: the refusal every earlier format gave is kept: {errors}"
        );
    }
}
