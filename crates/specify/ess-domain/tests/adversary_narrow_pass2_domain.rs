//! Adversary pass 2 for `story:optional-input-narrowed-after-refusal` (beyond10x/ess#169).
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

fn validates(text: &str) {
    if let Err(errors) = assemble(text) {
        panic!("{}\n---\n{text}", listed(&errors));
    }
}

/// The repro with the refusal spelled `refusal`.
fn with_refusal(refusal: &str) -> String {
    edited(NOTES, REFUSAL, refusal)
}

#[test]
fn the_not_mapping_form_the_design_note_names_narrows_like_not_defined() {
    // docs/design/optional-input-narrowing.md: "`missing(x)` and `{not: "defined(x)"}` parse to
    // the same predicate".
    validates(&with_refusal(
        "      - name: account-missing\n        when: {not: \"defined(account_id)\"}\n        error: demo.notes.AccountMissing\n",
    ));
}

#[test]
fn the_exists_false_mapping_form_narrows_like_not_defined() {
    validates(&with_refusal(
        "      - name: account-missing\n        when: {account_id: {exists: false}}\n        error: demo.notes.AccountMissing\n",
    ));
}

/// The command also takes and answers an `Optional<String>` named `memo`, and a second sibling
/// refuses the input `memo`'s absence.
fn with_memo(text_source: &str) -> String {
    let text = edited(
        NOTES,
        "      - {name: text, type: String}\n    outcomes:\n",
        "      - {name: text, type: String}\n      - {name: memo, type: Optional<String>}\n    response:\n      - {name: memo, type: Optional<String>}\n    outcomes:\n",
    );
    let text = edited(
        &text,
        REFUSAL,
        &format!(
            "{REFUSAL}      - name: memo-missing\n        when: not defined(memo)\n        error: demo.notes.AccountMissing\n"
        ),
    );
    edited(
        &text,
        "{generated: true}, account_id: input.account_id, text: input.text}",
        &format!("{{generated: true}}, account_id: input.account_id, text: {text_source}}}"),
    )
}

#[test]
fn control_the_input_memo_narrows_in_the_default() {
    validates(&with_memo("input.memo"));
}

#[test]
fn a_response_field_named_like_a_narrowed_input_is_not_narrowed() {
    // Refusing the absence of the *input* `memo` says nothing about the *response* `memo`: a
    // `{response: memo}` read keeps `Optional<String>` and cannot fill a `String`.
    let errors = assemble(&with_memo("{response: memo}"))
        .expect_err("a response read is never narrowed by an input refusal");
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch
                && error.location.contains(".payload.")),
        "{}",
        listed(&errors)
    );
}

#[test]
fn an_optional_list_input_narrows_to_the_list() {
    let text = edited(
        NOTES,
        "      - {name: text, type: String}\n    outcomes:\n",
        "      - {name: text, type: String}\n      - {name: tags, type: Optional<List<String>>}\n    outcomes:\n",
    );
    let text = edited(
        &text,
        REFUSAL,
        &format!(
            "{REFUSAL}      - name: tags-missing\n        when: missing(tags)\n        error: demo.notes.AccountMissing\n"
        ),
    );
    let text = edited(
        &text,
        "{generated: true}, account_id: input.account_id, text: input.text}",
        "{generated: true}, account_id: input.account_id, text: input.text, tags: input.tags}",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: tags, type: List<String>}\nviews:",
    );
    validates(&text);
}

/// The default spelled `when: true` and declared before the refusal.
fn with_always_default_first() -> String {
    let text = edited(NOTES, REFUSAL, "");
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: submitted\n        when: \"true\"\n",
    );
    // The refusal after the default.
    let payload = "          demo.notes.NoteSubmitted: {note_id: {generated: true}, account_id: input.account_id, text: input.text}\n";
    edited(&text, payload, &format!("{payload}{REFUSAL}"))
}

#[test]
fn an_always_default_declared_first_is_not_narrowed_by_a_later_refusal() {
    // Coordinator decision, correction round 2: only the default written without a condition
    // (`OutcomeCondition::Otherwise`) is narrowed by a refusal of absence. `when: true` is a guard
    // that holds, and Entity Runtime tries it in declared order, so declared before the refusal it
    // takes a request without the account. The copy is the #169 `type_mismatch`.
    let text = with_always_default_first();
    assert!(text.find("name: submitted") < text.find("name: account-missing"));
    let errors = assemble(&text).expect_err("a `when: true` branch is not narrowed");
    let mismatches: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::TypeMismatch)
        .collect();
    assert_eq!(mismatches.len(), 2, "{}", listed(&errors));
    for error in mismatches {
        assert!(
            error
                .message
                .contains("has type `Optional<demo.notes.AccountId>`"),
            "{}",
            listed(&errors)
        );
    }
}
