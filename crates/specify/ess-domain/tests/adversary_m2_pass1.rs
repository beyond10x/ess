//! Adversary pass 1 for an external refusal answered before the row (`before_row: true`, ess/25,
//! beyond10x/ess#534): where the key validates and where it is refused.

const SIGN_IN: &str = include_str!("../../ess-compiler/tests/fixtures/external-before-row.yaml");

/// `bad-credential` as the fixture declares it.
const BAD: &str = "      - name: bad-credential
        external: secret is not the digest the service derives from user_id
        before_row: true
        error: demo.desk.BadCredential
";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

/// `SIGN_IN` with `extra` errors declared.
fn with_errors(text: &str, extra: &str) -> String {
    replaced(text, "errors:\n", &format!("errors:\n{extra}"))
}

/// Below ess/25 the key is refused once, naming ess/25, and nothing else cascades from it: the
/// held-state branches declared after it are not refused by the #486 rule as well.
#[test]
fn below_ess_25_the_key_is_refused_alone() {
    let errors = spec(&replaced(SIGN_IN, "format: ess/25", "format: ess/24"))
        .map(|_| ())
        .expect_err("ess/24 refuses the key");
    let lines: Vec<&str> = errors
        .lines()
        .filter(|line| line.starts_with('['))
        .collect();
    assert_eq!(
        lines,
        [
            "[unsupported_format_version] command.demo.desk.SignIn.outcomes.bad-credential.\
             before_row: an external refusal answered before the row — `before_row: true` beside \
             `external:` — requires specification format ess/25 (hint: declare `format: \
             ess/25`, or drop `before_row:`)"
        ],
        "{errors}"
    );
}

/// Two externals answered before the row, in declaration order, and an input refusal declared
/// between them: all three are step 2, so the command validates.
#[test]
fn two_before_row_externals_around_an_input_refusal_validate() {
    let text = replaced(
        SIGN_IN,
        BAD,
        &format!(
            "{BAD}      - name: empty-secret
        when: secret == \"\"
        error: demo.desk.EmptySecret
      - name: locked-out
        external: the user is locked out
        before_row: true
        error: demo.desk.LockedOut
"
        ),
    );
    let text = with_errors(
        &text,
        "  - {name: demo.desk.EmptySecret, summary: Empty.}\n  - {name: demo.desk.LockedOut, \
         summary: Locked.}\n",
    );
    if let Err(errors) = spec(&text) {
        panic!("{errors}\n{text}");
    }
}

/// An `external:` branch with an input eligibility guard (`external:` beside `when:`) marked
/// `before_row` is an external refusal too, and validates above the held state.
#[test]
fn a_guarded_before_row_external_validates() {
    let text = replaced(
        SIGN_IN,
        "        before_row: true\n",
        "        when: secret != \"\"\n        before_row: true\n",
    );
    if let Err(errors) = spec(&text) {
        panic!("{errors}\n{text}");
    }
}

/// Declared after `unknown_instance:` (step 3) the key is refused, naming that branch.
#[test]
fn declared_after_unknown_instance_it_is_refused() {
    let first = "      - name: first
        unknown_instance: true
        creates: demo.desk.Seat
        instance: user_id
        sets: {call_ref: \"\"}
        emits: [demo.desk.SignedIn]
        payload: {demo.desk.SignedIn: {user_id: input.user_id}}
";
    // `first` first, then `bad-credential`, then the held-state branches.
    let text = replaced(&replaced(SIGN_IN, first, ""), BAD, &format!("{first}{BAD}"));
    let errors = spec(&text).map(|_| ()).expect_err("the order is refused");
    assert!(
        errors.lines().any(|line| line.contains(
            "[conflicting_declaration] command.demo.desk.SignIn.outcomes.bad-credential.before_row"
        ) && line.contains("`first`")),
        "{errors}"
    );
}

/// Declared after a step-6 external refusal (no key), the key is refused, naming it.
#[test]
fn declared_after_an_unmarked_external_it_is_refused() {
    let other = "      - name: rate-limited
        external: the caller exceeded its rate
        error: demo.desk.BadCredential
";
    let text = replaced(SIGN_IN, BAD, &format!("{BAD}{other}"));
    // `rate-limited` then sits above the held-state branches and is refused by #486; move the
    // marked branch below it to see the before_row refusal.
    let swapped = replaced(&replaced(&text, BAD, ""), other, &format!("{other}{BAD}"));
    let errors = spec(&swapped)
        .map(|_| ())
        .expect_err("the order is refused");
    assert!(
        errors.lines().any(|line| line.contains(
            "[conflicting_declaration] command.demo.desk.SignIn.outcomes.bad-credential.before_row"
        ) && line.contains("`rate-limited`")),
        "{errors}"
    );
}

/// An error payload filled from a stored field nested in a struct of sources reads the row, and is
/// refused as the flat one is.
#[test]
fn a_nested_payload_reading_the_row_is_refused() {
    let text = replaced(
        SIGN_IN,
        "        error: demo.desk.BadCredential\n",
        "        error: demo.desk.BadCredential\n        payload:\n          \
         demo.desk.BadCredential: {seat: {call_ref: {subject: call_ref}}}\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.desk.BadCredential\n",
        "  - name: demo.desk.BadCredential\n    fields: [{name: seat, type: demo.desk.SeatRef}]\n",
    );
    let text = replaced(
        &text,
        "entities:\n",
        "types:\n  - name: demo.desk.SeatRef\n    kind: struct\n    fields: [{name: call_ref, \
         type: String}]\nentities:\n",
    );
    let errors = spec(&text)
        .map(|_| ())
        .expect_err("the payload reads the row");
    assert!(
        errors
            .lines()
            .any(|line| line.contains("before_row") && line.contains("reads")),
        "{errors}"
    );
}
