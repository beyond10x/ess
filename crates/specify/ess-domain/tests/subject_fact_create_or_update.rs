//! A create-or-update command selects among its update branches by the stored row's fields
//! (ess/25, beyond10x/ess#530, `docs/design/cross-record-and-stored-field-guards.md`).
//!
//! The issue's `SignIn`: three `when_subject:` branches read the held state and the stored
//! `call_ref`, and `first` creates the seat for an identity no seat carries. From `ess/25` the
//! creating branch answers at step 3 of the precedence order, before any stored-field branch reads
//! the row, and the partition runs over the existing-row branches only. Below `ess/25` the pairing
//! keeps the refusal it always had, now naming the format that admits it.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

const MODEL: &str =
    include_str!("../../ess-compiler/tests/fixtures/create-or-update-stored-fields.yaml");

const SIGN_IN_OUTCOMES: &str = "command.demo.desk.SignIn.outcomes";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("desk.yaml"), raw)])
}

fn refused(text: &str) -> Vec<ValidationError> {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
        .as_slice()
        .to_vec()
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn at(format: &str) -> String {
    replaced(MODEL, "format: ess/25\n", &format!("format: {format}\n"))
}

fn repository(relative: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    std::fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

#[test]
fn ess_25_is_admitted_and_recorded() {
    assert!(
        ess_domain::system::SUPPORTED_FORMATS.contains(&25),
        "{:?}",
        ess_domain::system::SUPPORTED_FORMATS
    );
    assert_eq!(ess_domain::system::FormatVersion::V25.major(), 25);
    assert!(ess_domain::system::is_supported_format(
        ess_domain::system::FormatVersion::V25
    ));
    assert!(
        repository("models/toolchain/domains/specify.yaml").contains("\"ess/25\""),
        "SpecificationFormat lists ess/25"
    );
    assert!(
        ess_domain::system::FORMAT_HISTORY
            .iter()
            .any(|entry| entry.major == 25),
        "FORMAT_HISTORY has an ess/25 row"
    );
    assert!(
        repository("website/docs/reference/spec-versions.md")
            .lines()
            .any(|line| line.starts_with("| `ess/25` |")),
        "spec-versions.md has an ess/25 row"
    );
    assert!(
        repository("website/docs/reference/formats.md").contains("`format: ess/25`"),
        "formats.md has an ess/25 row"
    );
}

/// The issue's `SignIn` as filed: `reopened` selected by `state == Open or (state == Busy and
/// call_ref == "")` rather than answering every other stored seat.
fn as_filed() -> String {
    let text = replaced(
        MODEL,
        "      # Every other stored seat: Open, or Busy with no call. `call_ref` is a text, which the
      # finite prover does not enumerate, so the branch answering the rest is the default.
      - name: reopened
        moves: demo.desk.Seat.reopen
        instance: user_id
        emits: [demo.desk.SignedIn]
        payload: {demo.desk.SignedIn: {user_id: input.user_id}}
",
        "",
    );
    replaced(
        &text,
        "      - name: opened\n",
        "      - name: reopened
        when_subject: {predicate: {any: [state == Open, {all: [state == Busy, call_ref == \"\"]}]}}
        moves: demo.desk.Seat.reopen
        instance: user_id
        emits: [demo.desk.SignedIn]
        payload: {demo.desk.SignedIn: {user_id: input.user_id}}
      - name: opened\n",
    )
}

/// The same command with every predicate over the held state alone, a closed set the finite
/// prover enumerates, and no default.
fn state_only() -> String {
    let text = replaced(
        MODEL,
        "when_subject: {predicate: {all: [state == Busy, call_ref != \"\"]}}",
        "when_subject: {predicate: state == Busy}",
    );
    replaced(
        &text,
        "      - name: reopened\n        moves:",
        "      - name: reopened\n        when_subject: {predicate: state == Open}\n        moves:",
    )
}

#[test]
fn create_or_update_with_stored_field_branches_validates_at_ess_25() {
    for (label, text) in [("fixture", MODEL.to_owned()), ("state only", state_only())] {
        let spec = assemble(&text).unwrap_or_else(|errors| panic!("{label}: {errors}\n{text}"));
        let command = &spec.commands()[&"demo.desk.SignIn".parse().unwrap()];
        let names: Vec<&str> = command
            .outcomes
            .iter()
            .map(|outcome| outcome.name.as_str())
            .collect();
        assert_eq!(
            names,
            ["reopened-busy", "opened", "reopened", "first"],
            "{label}"
        );
    }
}

#[test]
fn create_or_update_with_stored_field_branches_is_refused_below_ess_25() {
    for format in ["ess/24", "ess/18"] {
        let errors = refused(&at(format));
        assert_eq!(errors.len(), 1, "{format}: {errors:#?}");
        let error = &errors[0];
        assert_eq!(
            error.code,
            ValidationCode::ConflictingDeclaration,
            "{format}: {error:#?}"
        );
        assert_eq!(error.location, SIGN_IN_OUTCOMES, "{format}");
        assert_eq!(
            error.message, "subject fact selection requires one existing subject identity",
            "{format}: the refusal is the one it always was"
        );
        assert!(
            error
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("ess/25")),
            "{format}: the hint names the format that admits the pairing: {error:#?}"
        );
    }
}
#[test]
fn the_issue_as_filed_reaches_the_partition_and_only_the_text_field_is_refused() {
    // At ess/25 the creating branch no longer stops validation: the partition runs, and the one
    // refusal left is the finite prover declining the text `call_ref` without a default — the
    // answer the same three predicates get on a command that creates nothing.
    let errors = refused(&as_filed());
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, ValidationCode::NonExhaustiveBranches);
    assert_eq!(errors[0].location, SIGN_IN_OUTCOMES);
    assert!(
        errors[0]
            .message
            .contains("`call_ref` has no closed set of values"),
        "{errors:#?}"
    );
    let without_creation = replaced(
        &as_filed(),
        "      - name: first
        unknown_instance: true
        creates: demo.desk.Seat
        instance: user_id
        sets: {call_ref: \"\"}
        emits: [demo.desk.SignedIn]
        payload: {demo.desk.SignedIn: {user_id: input.user_id}}
",
        "",
    );
    let same: Vec<_> = refused(&without_creation)
        .into_iter()
        .filter(|error| error.location == SIGN_IN_OUTCOMES)
        .collect();
    assert_eq!(
        same, errors,
        "the creating branch changes nothing the partition says"
    );
}

#[test]
fn a_creating_branch_without_unknown_instance_stays_refused_at_ess_25() {
    // Only the create-or-update pairing is relaxed: a creating branch the existence step does not
    // select (no `unknown_instance:`) beside stored-field branches is still refused.
    let text = replaced(
        MODEL,
        "      - name: first\n        unknown_instance: true\n",
        "      - name: first\n        when: secret == \"new\"\n",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration
                && error.location == SIGN_IN_OUTCOMES
                && error.message
                    == "subject fact selection requires one existing subject identity"
                && error.hint.is_none()),
        "{errors:#?}"
    );
}

#[test]
fn stored_field_branches_still_partition_existing_rows() {
    // `opened` gone: a stored `Closed` seat selects no branch, and the creating branch, which
    // answers only an identity no seat carries, does not cover it.
    let text = replaced(
        &state_only(),
        "      - name: opened
        when_subject: {predicate: state == Closed}
        moves: demo.desk.Seat.open
        instance: user_id
        emits: [demo.desk.SignedIn]
        payload: {demo.desk.SignedIn: {user_id: input.user_id}}
",
        "",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::NonExhaustiveBranches
                && error.location == SIGN_IN_OUTCOMES
                && error.message.contains("Closed")),
        "{errors:#?}"
    );
    // Two branches claiming one stored row are still ambiguous.
    let overlapping = replaced(
        &state_only(),
        "when_subject: {predicate: state == Open}",
        "when_subject: {predicate: {any: [state == Open, state == Busy]}}",
    );
    let errors = refused(&overlapping);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration
                && error.location == SIGN_IN_OUTCOMES
                && error.message.contains("select 2 branches")),
        "{errors:#?}"
    );
}

#[test]
fn the_creating_branch_may_be_declared_first() {
    // Existence answers before any stored row is read whatever the declaration order, so the
    // creating branch declared above the stored-field branches validates alike.
    let first = "      - name: first
        unknown_instance: true
        creates: demo.desk.Seat
        instance: user_id
        sets: {call_ref: \"\"}
        emits: [demo.desk.SignedIn]
        payload: {demo.desk.SignedIn: {user_id: input.user_id}}
";
    let text = replaced(MODEL, first, "");
    let text = replaced(
        &text,
        "    outcomes:\n      - name: reopened-busy\n",
        &format!("    outcomes:\n{first}      - name: reopened-busy\n"),
    );
    assemble(&text).unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let errors = refused(&replaced(&text, "format: ess/25\n", "format: ess/24\n"));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .hint
            .as_deref()
            .is_some_and(|hint| hint.contains("ess/25")),
        "{errors:#?}"
    );
}

// ---- `when_subject:` beside `unknown_instance:` under the one precedence rule (G4) ---------

const READ_ONLY: &str =
    include_str!("../../ess-compiler/tests/fixtures/read-only-unknown-instance.yaml");

/// `READ_ONLY` at `ess/27` with one more command, `demo.desk.Touch`, whose `outcomes` are `body`
/// with every `GUARD(<state>)` written by `guard`.
fn touch(body: &str, guard: fn(&str) -> String) -> String {
    let mut outcomes = String::new();
    for line in body.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("GUARD(") {
            outcomes.push_str("        ");
            outcomes.push_str(&guard(rest.trim_end_matches(')')));
        } else {
            outcomes.push_str(line);
        }
        outcomes.push('\n');
    }
    let text = READ_ONLY
        .replacen("format: ess/24\n", "format: ess/27\n", 1)
        .replacen(
            "    may: [demo.desk.TakeSeat, demo.desk.CloseSeat, demo.desk.Ring]\n",
            "    may: [demo.desk.TakeSeat, demo.desk.CloseSeat, demo.desk.Ring, demo.desk.Touch]\n",
            1,
        )
        .replacen(
            "views:\n",
            &format!(
                "  - name: demo.desk.Touch\n    input: [{{name: user_id, type: String}}]\n    outcomes:\n{outcomes}views:\n"
            ),
            1,
        );
    assert!(text.contains("demo.desk.Touch\n    input"));
    text
}

/// Every shape in which `when_subject_state:` sits beside an `unknown_instance:` branch —
/// refusing or creating, the held-state branches refusing, preserving or moving — also validates
/// with each literal held-state guard written as a `when_subject:` predicate over `state`: under
/// the one rule both answer at step 3, after the existence step that `unknown_instance:` answers
/// at (`docs/design/generic-language-features.md` § G4).
#[test]
fn format_a_when_subject_admits_unknown_instance_wherever_when_subject_state_does() {
    let refusing = "      - {name: no-seat, unknown_instance: true, error: demo.desk.NoSeat}\n";
    let creating = "      - name: no-seat\n        unknown_instance: true\n        creates: demo.desk.Seat\n        instance: user_id\n        emits: [demo.desk.SeatTaken]\n        payload:\n          demo.desk.SeatTaken: {user_id: input.user_id}\n";
    let shut = "      - name: shut\n        GUARD(Closed)\n        error: demo.desk.NoSeat\n";
    let kept = "      - name: kept\n        preserves: demo.desk.Seat\n        instance: user_id\n";
    let closing = "      - name: closing\n        GUARD(Open)\n        moves: demo.desk.Seat.close\n        instance: user_id\n        emits: [demo.desk.SeatClosed]\n        payload:\n          demo.desk.SeatClosed: {user_id: input.user_id}\n";
    let shapes = [
        format!("{refusing}{shut}{kept}"),
        format!("{creating}{shut}{kept}"),
        format!("{refusing}{closing}{shut}"),
        format!("{creating}{closing}{shut}"),
    ];
    for shape in &shapes {
        let literal = touch(shape, |state| format!("when_subject_state: {state}"));
        if let Err(errors) = assemble(&literal) {
            panic!("the when_subject_state shape validates:\n{errors}\n{shape}");
        }
        let stored = touch(shape, |state| {
            format!("when_subject: {{predicate: state == {state}}}")
        });
        if let Err(errors) = assemble(&stored) {
            panic!("the when_subject shape validates too:\n{errors}\n{shape}");
        }
    }
}
