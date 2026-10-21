//! An `external:` branch narrowed to the records a subject guard selects (`ess/29`,
//! `story:external-branch-narrowed-by-subject-guard`, option 2 of beyond10x/ess#584).
//!
//! `external:` beside `when_subject: {predicate: …}` was refused at every format, "a subject fact has
//! one selection authority". From `ess/29` the branch validates: it reads the addressed record, so
//! the one precedence rule (`docs/design/generic-language-features.md` § G4) answers it at step 3,
//! among the held-state branches in declaration order, and only on a record its predicate selects.
//! Below `ess/29` the pair keeps its refusal, the hint naming `ess/29`.
use ess_domain::command::precedence::{order, BranchShape, Composition, Phase, Rank};
use ess_domain::command::OutcomeCondition;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/external-narrowed-by-subject.yaml"
);

const NARROWED: &str = "      - name: hashed-mismatch
        external: the outside service finds the hashed verifier differs from the stored challenge
        when_subject:
          predicate: method == hashed
        error: demo.vault.VerifierMismatch
";

fn spec(body: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    Specification::assemble([(Source::new("vault.yaml"), raw)]).map_err(|e| e.to_string())
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

fn narrowed(with: &str) -> String {
    assert!(MODEL.contains(NARROWED));
    MODEL.replacen(NARROWED, with, 1)
}

/// The branches of `demo.vault.Redeem`, by phase, in the order the plan reads them.
fn phases(text: &str) -> Vec<(Phase, Vec<String>)> {
    let spec = spec(text).unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"demo.vault.Redeem".parse().unwrap()];
    let shapes: Vec<BranchShape> = command.outcomes.iter().map(BranchShape::of).collect();
    order(&shapes, &Composition::of(command, spec.system().format))
        .into_iter()
        .filter(|(_, branches)| !branches.is_empty())
        .map(|(phase, branches)| {
            (
                phase,
                branches
                    .into_iter()
                    .map(|index| command.outcomes[index].name.to_string())
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn from_ess_29_an_external_branch_narrowed_by_a_subject_guard_validates() {
    let spec = spec(MODEL).unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"demo.vault.Redeem".parse().unwrap()];
    let branch = &command.outcomes[0];
    assert_eq!(branch.name.as_str(), "hashed-mismatch");
    let OutcomeCondition::ExternalSubject {
        cause,
        predicate,
        input,
    } = &branch.condition
    else {
        panic!("{:?}", branch.condition)
    };
    assert_eq!(
        cause,
        "the outside service finds the hashed verifier differs from the stored challenge"
    );
    assert_eq!(predicate.to_string(), "method == hashed");
    assert!(input.is_none());
    assert_eq!(branch.condition.cause(), Some(cause.as_str()));
    assert_eq!(
        branch.condition.subject_predicate().map(|p| p.to_string()),
        Some("method == hashed".to_owned())
    );
    assert!(branch.condition.reads_subject_fact());
}

#[test]
fn the_narrowed_branch_answers_at_the_held_state_step_in_declaration_order() {
    // Declared first among the held-state branches, it answers first among them, before
    // `wrong_state:`; without the narrowing it would answer after every held-state branch.
    assert_eq!(
        phases(MODEL),
        vec![
            (
                Phase::HeldState,
                vec![
                    "hashed-mismatch".to_owned(),
                    "redirect-mismatch".to_owned(),
                    "plain-mismatch".to_owned(),
                    "already-redeemed".to_owned(),
                ]
            ),
            (Phase::Default, vec!["redeemed".to_owned()]),
        ]
    );
    let spec = spec(MODEL).unwrap();
    let command = &spec.commands()[&"demo.vault.Redeem".parse().unwrap()];
    let composition = Composition::of(command, spec.system().format);
    let place = ess_domain::command::precedence::place(
        &BranchShape::of(&command.outcomes[0]),
        &composition,
    );
    assert_eq!(
        (place.phase, place.rank),
        (Phase::HeldState, Rank::Declared)
    );
    assert_eq!(place.phase.step_in(spec.system().format), Some(3));
}

#[test]
fn declared_after_a_held_state_refusal_it_answers_after_it() {
    // No `conflicting_declaration`: the two are one step, and declaration order is its order.
    let moved = narrowed("").replacen(
        "      - name: plain-mismatch\n",
        &format!("{NARROWED}      - name: plain-mismatch\n"),
        1,
    );
    assert_eq!(
        phases(&moved)[0].1,
        vec![
            "redirect-mismatch".to_owned(),
            "hashed-mismatch".to_owned(),
            "plain-mismatch".to_owned(),
            "already-redeemed".to_owned(),
        ]
    );
}

#[test]
fn an_input_guard_beside_the_narrowing_is_kept() {
    let spec = spec(&narrowed(
        "      - name: hashed-mismatch
        external: the outside service finds the hashed verifier differs from the stored challenge
        when: verifier != \"\"
        when_subject:
          predicate: method == hashed
        error: demo.vault.VerifierMismatch
",
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"demo.vault.Redeem".parse().unwrap()];
    let OutcomeCondition::ExternalSubject { input, .. } = &command.outcomes[0].condition else {
        panic!("{:?}", command.outcomes[0].condition)
    };
    assert_eq!(
        input.as_ref().map(ToString::to_string),
        Some("verifier != \"\"".to_owned())
    );
}

#[test]
fn below_ess_29_the_pair_keeps_its_refusal_naming_ess_29() {
    let errors = refused(&at("ess/28", MODEL));
    assert!(
        errors.contains("conflicting_declaration")
            && errors.contains("one selection authority")
            && errors.contains("ess/29"),
        "{errors}"
    );
}

#[test]
fn a_narrowed_branch_answered_before_the_row_is_refused() {
    let errors = refused(&narrowed(&NARROWED.replace(
        "        error: demo.vault.VerifierMismatch\n",
        "        error: demo.vault.VerifierMismatch\n        before_row: true\n",
    )));
    assert!(
        errors.contains("conflicting_declaration")
            && errors.contains("before_row")
            && errors.contains("when_subject"),
        "{errors}"
    );
}

#[test]
fn the_field_equals_form_beside_external_stays_refused_naming_the_predicate_form() {
    let errors = refused(&narrowed(&NARROWED.replace(
        "          predicate: method == hashed\n",
        "          field: method\n          equals: hashed\n",
    )));
    assert!(
        errors.contains("conflicting_declaration") && errors.contains("when_subject: {predicate:"),
        "{errors}"
    );
}

#[test]
fn a_narrowing_predicate_is_checked_against_the_entity() {
    let errors = refused(&narrowed(
        &NARROWED.replace("method == hashed", "flavour == hashed"),
    ));
    assert!(errors.contains("flavour"), "{errors}");
}

#[test]
fn a_narrowed_refusal_needs_a_sibling_naming_an_existing_subject() {
    let without = MODEL
        .replace(
            "      states: [Issued, Redeemed]
      terminal: [Redeemed]
      transitions:
        - {name: redeem, from: [Issued], to: Redeemed}
",
            "      states: [Issued]
      terminal: [Issued]
      transitions: []
",
        )
        .replace(
            "        moves: demo.vault.Grant.redeem
        instance: grant_id
        emits: [demo.vault.Redeemed]
        payload:
          demo.vault.Redeemed: {grant_id: input.grant_id}
",
            "        emits: [demo.vault.Redeemed]
        payload:
          demo.vault.Redeemed: {grant_id: input.grant_id}
",
        )
        .replace(
            "      - name: redirect-mismatch
        when_subject:
          predicate: redirect != input.redirect
        error: demo.vault.RedirectMismatch
      - name: plain-mismatch
        when_subject:
          predicate:
            all:
              - method == plain
              - challenge != input.verifier
        error: demo.vault.VerifierMismatch
",
            "",
        )
        .replace(
            "      - name: already-redeemed
        wrong_state: true
        error: demo.vault.AlreadyRedeemed
",
            "",
        );
    let errors = refused(&without);
    assert!(
        errors.contains("hashed-mismatch") && errors.contains("names an existing subject"),
        "{errors}"
    );
}
