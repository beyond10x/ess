//! Adversary cases for `story:optional-input-narrowed-after-refusal` (beyond10x/ess#169).
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const NOTES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/optional-input-narrowed.yaml");

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

/// The input is a raw `Optional<Uuid>` and the targets are the `AccountId` newtype, crossed by a
/// declared `Optional<Uuid> -> AccountId` conversion: the #169 workaround, for an input whose
/// declared type is not the target's.
fn with_workaround_conversion(format: &str) -> String {
    let text = edited(
        NOTES,
        "      - {name: account_id, type: Optional<demo.notes.AccountId>}\n",
        "      - {name: account_id, type: Optional<Uuid>}\n",
    );
    let text = edited(
        &text,
        "commands:\n",
        "conversions:\n  - {from: Optional<Uuid>, to: demo.notes.AccountId, because: the account parameter is the account's identifier}\ncommands:\n",
    );
    edited(&text, "format: ess/16", &format!("format: {format}"))
}

#[test]
fn the_workaround_conversion_is_admitted_below_ess_16() {
    // Control: the declared crossing admits the copy where nothing narrows.
    validates(&with_workaround_conversion("ess/15"));
}

#[test]
fn narrowing_does_not_revoke_a_declared_crossing_from_the_optional_type() {
    // Under ess/16 the same model, unchanged but for its header, must still validate: narrowing
    // adds a way to admit the copy, it does not take away the conversion the author declared.
    validates(&with_workaround_conversion("ess/16"));
}

#[test]
fn a_branch_guarded_by_defined_under_a_subject_state_reads_the_input_as_present() {
    // The design note: rule 1 covers "the input guard of every subject-guarded condition".
    let text = edited(
        NOTES,
        "    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n",
        "    lifecycle:\n      initial: Open\n      states: [Open, Filed]\n      terminal: [Filed]\n      transitions:\n        - {name: file, from: [Open, Filed], to: Filed}\n",
    );
    let text = edited(
        &text,
        "  - {name: demo.notes.Service, may: [demo.notes.SubmitNote]}\n",
        "  - {name: demo.notes.Service, may: [demo.notes.SubmitNote, demo.notes.FileNote]}\n",
    );
    let text = edited(
        &text,
        "events:\n",
        "  - name: demo.notes.FileNote\n    input:\n      - {name: note_id, type: demo.notes.NoteId}\n      - {name: account_id, type: Optional<demo.notes.AccountId>}\n    outcomes:\n      - name: filed\n        moves: demo.notes.Note.file\n        instance: note_id\n        when_subject_state: Open\n        when: defined(account_id)\n        emits: [demo.notes.NoteFiled]\n        payload:\n          demo.notes.NoteFiled: {note_id: input.note_id, account_id: input.account_id}\n      - name: no-account\n        moves: demo.notes.Note.file\n        instance: note_id\n        emits: [demo.notes.NoteFiledAlone]\n        payload:\n          demo.notes.NoteFiledAlone: {note_id: input.note_id}\nevents:\n  - name: demo.notes.NoteFiledAlone\n    fields:\n      - {name: note_id, type: demo.notes.NoteId}\n  - name: demo.notes.NoteFiled\n    fields:\n      - {name: note_id, type: demo.notes.NoteId}\n      - {name: account_id, type: demo.notes.AccountId}\n",
    );
    validates(&text);
}

#[test]
fn an_enum_and_a_decimal_input_narrow_the_same_way() {
    let text = edited(
        NOTES,
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n  - {name: demo.notes.Kind, kind: enum, variants: [Plain, Urgent]}\n",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\n    outcomes:",
        "      - {name: text, type: String}\n      - {name: kind, type: Optional<demo.notes.Kind>}\n      - {name: weight, type: Optional<Decimal>}\n    outcomes:",
    );
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: kind-missing\n        when: missing(kind)\n        error: demo.notes.AccountMissing\n      - name: weight-missing\n        when: not defined(weight)\n        error: demo.notes.AccountMissing\n      - name: submitted\n",
    );
    let text = edited(
        &text,
        "text: input.text}\nevents:",
        "text: input.text, kind: input.kind, weight: input.weight}\nevents:",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: kind, type: demo.notes.Kind}\n      - {name: weight, type: Decimal}\nviews:",
    );
    validates(&text);
}

#[test]
fn a_narrowed_leaf_beside_an_input_or_generated_leaf_in_one_struct() {
    let text = edited(
        NOTES,
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n  - name: demo.notes.Owner\n    kind: struct\n    fields:\n      - {name: account_id, type: demo.notes.AccountId}\n      - {name: delegate_id, type: demo.notes.AccountId}\n",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\n    outcomes:",
        "      - {name: text, type: String}\n      - {name: delegate_id, type: Optional<demo.notes.AccountId>}\n    outcomes:",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: owner, type: demo.notes.Owner}\nviews:",
    );
    let text = edited(
        &text,
        "account_id: input.account_id, text: input.text}\nevents:",
        "account_id: input.account_id, text: input.text, owner: {account_id: input.account_id, delegate_id: {input: delegate_id, else: {generated: true}}}}\nevents:",
    );
    validates(&text);
}

#[test]
fn a_mismatch_under_ess_16_names_the_declared_type_where_nothing_narrows() {
    // A read in a branch that is not narrowed keeps the #169 message byte for byte.
    let text = edited(
        NOTES,
        "        when: not defined(account_id)\n        error: demo.notes.AccountMissing\n",
        "        when: text == \"x\"\n        error: demo.notes.AccountMissing\n",
    );
    let errors = assemble(&text).expect_err("nothing narrows");
    let payload = errors
        .as_slice()
        .iter()
        .find(|error| {
            error.code == ValidationCode::TypeMismatch && error.location.contains(".payload.")
        })
        .unwrap_or_else(|| panic!("{}", listed(&errors)));
    assert_eq!(
        payload.message,
        "`demo.notes.SubmitNote.account_id` has type `Optional<demo.notes.AccountId>`, and \
         `demo.notes.NoteSubmitted.account_id` requires `demo.notes.AccountId`; no conversion is \
         declared"
    );
}

#[test]
fn a_refusal_of_absence_scoped_by_a_subject_guard_narrows_nothing() {
    // `no-account` refuses an absent account only for a note whose text is "x"; any other note with no
    // account reaches the default, so the default cannot read the account as present.
    let text = edited(
        NOTES,
        "    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n",
        "    lifecycle:\n      initial: Open\n      states: [Open, Filed]\n      terminal: [Filed]\n      transitions:\n        - {name: file, from: [Open, Filed], to: Filed}\n",
    );
    let text = edited(
        &text,
        "  - {name: demo.notes.Service, may: [demo.notes.SubmitNote]}\n",
        "  - {name: demo.notes.Service, may: [demo.notes.SubmitNote, demo.notes.FileNote]}\n",
    );
    let text = edited(
        &text,
        "events:\n",
        "  - name: demo.notes.FileNote\n    input:\n      - {name: note_id, type: demo.notes.NoteId}\n      - {name: account_id, type: Optional<demo.notes.AccountId>}\n    outcomes:\n      - name: no-account\n        when_subject:\n          predicate: 'text == \"x\"'\n        when: not defined(account_id)\n        error: demo.notes.AccountMissing\n      - name: filed\n        moves: demo.notes.Note.file\n        instance: note_id\n        emits: [demo.notes.NoteFiled]\n        payload:\n          demo.notes.NoteFiled: {note_id: input.note_id, account_id: input.account_id}\nevents:\n  - name: demo.notes.NoteFiled\n    fields:\n      - {name: note_id, type: demo.notes.NoteId}\n      - {name: account_id, type: demo.notes.AccountId}\n",
    );
    let errors = assemble(&text).expect_err("a subject-scoped refusal narrows nothing");
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch
                && error.location.contains("FileNote")),
        "{}",
        listed(&errors)
    );
}
