//! An `Optional<T>` input read as `T` once a sibling outcome refuses its absence (beyond10x/ess#169,
//! source format `ess/16`, `docs/design/optional-input-narrowing.md`).
//!
//! The default branch of a command is taken only when no guarded sibling matched. A sibling that
//! refuses exactly `not defined(x)` (or `missing(x)`) therefore leaves the default with `x` present,
//! and a branch whose own guard requires `defined(x)` is taken only with it present. Both read
//! `input.x` as `T`. Below `ess/16` the type mismatch stands as it always has.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const NOTES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/optional-input-narrowed.yaml");

const REFUSAL: &str = "      - name: account-missing\n        when: not defined(account_id)\n        error: demo.notes.AccountMissing\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("notes.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn edited(base: &str, before: &str, after: &str) -> String {
    assert!(base.contains(before), "fixture holds {before:?}");
    base.replacen(before, after, 1)
}

fn at_format(text: &str, format: &str) -> String {
    edited(text, "format: ess/16", &format!("format: {format}"))
}

fn validates(text: &str) {
    if let Err(errors) = assemble(text) {
        panic!("{}\n---\n{text}", listed(&errors));
    }
}

/// The two `type_mismatch` refusals #169 reports, one for `payload:` and one for `sets:`, and
/// nothing about a format.
fn refused_as_the_crossing(text: &str) {
    let errors = assemble(text).expect_err("the crossing must be refused");
    let mismatches: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::TypeMismatch)
        .collect();
    assert_eq!(mismatches.len(), 2, "{}", listed(&errors));
    assert!(
        mismatches
            .iter()
            .any(|error| error.location.contains(".payload.")),
        "{}",
        listed(&errors)
    );
    assert!(
        mismatches
            .iter()
            .any(|error| error.location.contains(".sets.")),
        "{}",
        listed(&errors)
    );
    for error in &mismatches {
        assert!(
            error
                .message
                .contains("has type `Optional<demo.notes.AccountId>`"),
            "{}",
            listed(&errors)
        );
    }
    assert!(
        !errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedFormatVersion),
        "{}",
        listed(&errors)
    );
}

#[test]
fn issue_169_the_repro_validates_under_ess_16_without_a_conversion() {
    validates(NOTES);
}

#[test]
fn below_ess_16_the_repro_is_still_refused_as_an_undeclared_crossing() {
    refused_as_the_crossing(&at_format(NOTES, "ess/15"));
    refused_as_the_crossing(&at_format(NOTES, "ess/14"));
}

#[test]
fn missing_is_the_same_refusal_as_not_defined() {
    validates(&edited(
        NOTES,
        "when: not defined(account_id)",
        "when: missing(account_id)",
    ));
}

#[test]
fn the_refusal_may_be_declared_after_the_branch_it_narrows() {
    let without = edited(NOTES, REFUSAL, "");
    // `events:` follows the command's last outcome, so the refusal now closes the list.
    validates(&edited(
        &without,
        "events:\n",
        &format!("{REFUSAL}events:\n"),
    ));
}

#[test]
fn a_branch_guarded_by_defined_reads_the_input_as_present() {
    let text = edited(
        NOTES,
        REFUSAL,
        "      - name: account-missing\n        error: demo.notes.AccountMissing\n",
    );
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: submitted\n        when: defined(account_id)\n",
    );
    validates(&text);
}

#[test]
fn a_guard_that_requires_defined_among_other_conditions_narrows_too() {
    let text = edited(
        NOTES,
        REFUSAL,
        "      - name: account-missing\n        error: demo.notes.AccountMissing\n",
    );
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: submitted\n        when: {all: [\"defined(account_id)\", text != x]}\n",
    );
    validates(&text);
}

#[test]
fn a_sibling_that_does_not_refuse_narrows_nothing() {
    // The absent branch succeeds, so nothing says the input is present anywhere.
    let text = edited(
        NOTES,
        "        error: demo.notes.AccountMissing\n",
        "        emits: [demo.notes.NoteSubmitted]\n        payload:\n          demo.notes.NoteSubmitted: {note_id: {generated: true}, account_id: {generated: true}, text: input.text}\n",
    );
    let errors = assemble(&text).expect_err("no refusal, no narrowing");
    assert_eq!(
        errors
            .as_slice()
            .iter()
            .filter(|error| error.code == ValidationCode::TypeMismatch)
            .count(),
        2,
        "{}",
        listed(&errors)
    );
}

#[test]
fn a_refusal_of_more_than_the_absence_narrows_nothing() {
    // `all: [not defined(account_id), text == x]` leaves an absent account with any other text to the
    // default branch, so the default cannot read it as present.
    refused_as_the_crossing(&edited(
        NOTES,
        "when: not defined(account_id)",
        "when: {all: [\"not defined(account_id)\", text == x]}",
    ));
}

#[test]
fn a_disjunctive_guard_narrows_nothing() {
    let text = edited(
        NOTES,
        REFUSAL,
        "      - name: account-missing\n        error: demo.notes.AccountMissing\n",
    );
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: submitted\n        when: {any: [\"defined(account_id)\", text == x]}\n",
    );
    refused_as_the_crossing(&text);
}

#[test]
fn a_guarded_sibling_of_the_refusal_is_not_narrowed_by_it() {
    // `submitted` is guarded by something other than presence, so an absent account with text
    // `x` satisfies both guards; only the default is known to be left with the account present.
    let text = edited(
        NOTES,
        "      - name: submitted\n",
        "      - name: submitted\n        when: text == \"x\"\n",
    );
    let text = edited(
        &text,
        "events:\n",
        "      - name: other\n        error: demo.notes.AccountMissing\nevents:\n",
    );
    refused_as_the_crossing(&text);
}

#[test]
fn a_different_input_is_not_narrowed_by_the_refusal() {
    let text = edited(
        NOTES,
        "      - {name: text, type: String}\n    outcomes:",
        "      - {name: text, type: String}\n      - {name: other_id, type: Optional<demo.notes.AccountId>}\n    outcomes:",
    );
    let text = edited(
        &text,
        "when: not defined(account_id)",
        "when: not defined(other_id)",
    );
    refused_as_the_crossing(&text);
}

#[test]
fn a_leaf_of_a_nested_mapping_reads_the_narrowed_input() {
    let text = edited(
        NOTES,
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n  - name: demo.notes.Owner\n    kind: struct\n    fields:\n      - {name: account_id, type: demo.notes.AccountId}\n",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: owner, type: demo.notes.Owner}\nviews:",
    );
    let text = edited(
        &text,
        "account_id: input.account_id, text: input.text}\nevents:",
        "account_id: input.account_id, text: input.text, owner: {account_id: input.account_id}}\nevents:",
    );
    validates(&text);

    let errors = assemble(&at_format(&text, "ess/15")).expect_err("below ess/16 the leaf crosses");
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch
                && error.location.contains("owner")),
        "{}",
        listed(&errors)
    );
}

/// The input as a raw `Optional<Uuid>`: narrowed to `Uuid`, which is still not an `AccountId`.
fn raw_uuid_account() -> String {
    edited(
        NOTES,
        "      - {name: account_id, type: Optional<demo.notes.AccountId>}\n",
        "      - {name: account_id, type: Optional<Uuid>}\n",
    )
}

#[test]
fn a_narrowed_read_that_still_crosses_is_refused_under_the_declared_type() {
    // Every site — `payload:`, `sets:` and a nested leaf — names what the author wrote,
    // `Optional<Uuid>`, and hints a crossing from it rather than a domain-wide `Uuid` one.
    let text = edited(
        &raw_uuid_account(),
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n  - name: demo.notes.Owner\n    kind: struct\n    fields:\n      - {name: account_id, type: demo.notes.AccountId}\n",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: owner, type: demo.notes.Owner}\nviews:",
    );
    let text = edited(
        &text,
        "account_id: input.account_id, text: input.text}\nevents:",
        "account_id: input.account_id, text: input.text, owner: {account_id: input.account_id}}\nevents:",
    );
    let errors = assemble(&text).expect_err("Uuid is not an AccountId");
    let mismatches: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::TypeMismatch)
        .collect();
    assert_eq!(mismatches.len(), 3, "{}", listed(&errors));
    for error in mismatches {
        assert!(
            error.message.contains("has type `Optional<Uuid>`"),
            "{}",
            listed(&errors)
        );
        assert!(
            error
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("from: Optional<Uuid>")),
            "{}",
            listed(&errors)
        );
    }
}

#[test]
fn a_declared_crossing_from_the_optional_type_admits_every_site_under_ess_16() {
    let text = edited(
        &raw_uuid_account(),
        "commands:\n",
        "conversions:\n  - {from: Optional<Uuid>, to: demo.notes.AccountId, because: the parameter is the account's identifier}\ncommands:\n",
    );
    validates(&text);
}
