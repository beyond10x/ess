//! A branch guarded by a row of another entity, named by an identity in the input (beyond10x/ess#211,
//! `ess/18`).
//!
//! `when_related: {via: input.<field>, exists: false}` is taken when no row of the entity whose
//! identity `<field>` carries exists; `when_related: {via: input.<field>, predicate: …}` is taken when
//! that row exists and the predicate over its stored fields (and `input.`) holds. One hop, keyed by
//! the other entity's identity only: a lookup by any other field (rule 2 of
//! `docs/design/cross-record-and-stored-field-guards.md`) stays out of scope.
use ess_domain::{
    command::{OutcomeCondition, RelatedTest},
    spec::RawSpecFile,
    system::Source,
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// The issue's own shape: a creating command refused when the tenant has no configuration, or when
/// the configuration does not register the client.
pub const SIGN_IN: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml");

const ABSENT: &str = "        when_related: {via: input.tenant, exists: false}\n";
const HOLDS: &str =
    "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n";

fn assemble_as(text: &str, file: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new(file), raw)])
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    assemble_as(text, "sign-in.yaml")
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn has(errors: &ValidationErrors, code: ValidationCode, fragment: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(fragment))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn at(text: &str, format: &str) -> String {
    replaced(text, "format: ess/18\n", &format!("format: {format}\n"))
}

/// `text` rewritten as JSON: the same document in the form a JSON-source author writes it.
fn as_json(text: &str) -> String {
    let value: serde_yaml::Value = serde_yaml::from_str(text).expect("fixture is YAML");
    serde_json::to_string_pretty(&value).expect("fixture is JSON-representable")
}

#[test]
fn issue_211_a_create_guarded_by_another_entity_validates_under_ess_18() {
    let spec = accepted(SIGN_IN);
    let command = &spec.commands()[&"demo.signin.InitiateSignIn".parse().unwrap()];
    let branch = |name: &str| {
        command
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == name)
            .unwrap_or_else(|| panic!("no outcome {name}"))
    };
    let OutcomeCondition::Related { via, test, input } = &branch("no-configuration").condition
    else {
        panic!("expected a related guard: {:?}", branch("no-configuration"))
    };
    assert_eq!(via, "tenant");
    assert_eq!(*test, RelatedTest::Absent);
    assert!(input.is_none());
    let OutcomeCondition::Related { via, test, .. } = &branch("no-redirect-entry").condition else {
        panic!("expected a related guard")
    };
    assert_eq!(via, "tenant");
    let RelatedTest::Holds(predicate) = test else {
        panic!("expected a predicate: {test:?}")
    };
    assert_eq!(predicate.to_string(), "redirect_client != input.client");
    assert!(branch("no-redirect-entry").subject.is_none());
    assert_eq!(
        branch("initiated").condition,
        OutcomeCondition::Otherwise,
        "the creation is the default"
    );
}

#[test]
fn issue_211_the_key_is_refused_below_ess_18_in_yaml_and_json() {
    for format in ["ess/17", "ess/16"] {
        let text = at(SIGN_IN, format);
        for (form, file, text) in [
            ("yaml", "sign-in.yaml", text.clone()),
            ("json", "sign-in.json", as_json(&text)),
        ] {
            let errors = assemble_as(&text, file)
                .err()
                .unwrap_or_else(|| panic!("{form} at {format} must refuse"));
            assert!(
                has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/18"),
                "{form} at {format}: {errors}"
            );
            assert!(
                errors
                    .as_slice()
                    .iter()
                    .any(|error| error.to_string().contains("when_related")),
                "{form} at {format} names the key: {errors}"
            );
        }
    }
    assemble_as(&as_json(SIGN_IN), "sign-in.json").unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn issue_211_combining_with_a_subject_guard_on_one_branch_is_conflicting() {
    let both = replaced(
        SIGN_IN,
        ABSENT,
        "        when_related: {via: input.tenant, exists: false}\n        when_subject: {predicate: tenant == input.tenant}\n",
    );
    let errors = refused(&both);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "when_related"
        ),
        "{errors}"
    );
    let state = replaced(
        SIGN_IN,
        ABSENT,
        "        when_related: {via: input.tenant, exists: false}\n        when_subject_state: Initiated\n",
    );
    let errors = refused(&state);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "when_related"
        ),
        "{errors}"
    );
}

#[test]
fn issue_211_an_input_guard_beside_the_related_guard_is_admitted() {
    let spec = accepted(&replaced(
        SIGN_IN,
        HOLDS,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n        when: client != \"\"\n",
    ));
    let command = &spec.commands()[&"demo.signin.InitiateSignIn".parse().unwrap()];
    let OutcomeCondition::Related { input, .. } = &command.outcomes[1].condition else {
        panic!("expected a related guard")
    };
    assert_eq!(input.as_ref().unwrap().to_string(), r#"client != """#);
}

#[test]
fn issue_211_via_is_an_input_field_typed_as_another_entitys_identity() {
    // Not rooted at `input.`: the row is named by the input, one hop.
    let errors = refused(&replaced(
        SIGN_IN,
        ABSENT,
        "        when_related: {via: tenant, exists: false}\n",
    ));
    assert!(
        has(&errors, ValidationCode::TypeMismatch, "input."),
        "{errors}"
    );
    // An input the command does not declare.
    let errors = refused(&replaced(
        SIGN_IN,
        "via: input.tenant",
        "via: input.organisation",
    ));
    assert!(
        has(&errors, ValidationCode::UndeclaredReference, "organisation"),
        "{errors}"
    );
    // A field that is no entity's identity: a lookup by another field is rule 2, out of scope.
    let errors = refused(&replaced(SIGN_IN, "via: input.tenant", "via: input.client"));
    assert!(
        has(&errors, ValidationCode::TypeMismatch, "identity"),
        "{errors}"
    );
}

#[test]
fn issue_211_the_predicate_reads_the_related_rows_fields_and_the_input() {
    let errors = refused(&replaced(
        SIGN_IN,
        HOLDS,
        "        when_related: {via: input.tenant, predicate: redirect_uri != input.client}\n",
    ));
    assert!(
        has(&errors, ValidationCode::UnobservableFact, "redirect_uri"),
        "{errors}"
    );
    let errors = refused(&replaced(
        SIGN_IN,
        HOLDS,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.scope}\n",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("scope")),
        "{errors}"
    );
}

#[test]
fn issue_211_a_predicate_needs_an_exists_false_branch_for_the_missing_row() {
    let errors = refused(&replaced(
        SIGN_IN,
        "      - name: no-configuration\n        when_related: {via: input.tenant, exists: false}\n        error: demo.signin.NoConfiguration\n",
        "",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::NonExhaustiveBranches,
            "exists: false"
        ),
        "{errors}"
    );
}

#[test]
fn issue_211_exists_takes_false_and_one_shape_per_key() {
    let errors = refused(&replaced(
        SIGN_IN,
        ABSENT,
        "        when_related: {via: input.tenant, exists: true}\n",
    ));
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, "exists"),
        "{errors}"
    );
    let both = replaced(
        SIGN_IN,
        ABSENT,
        "        when_related: {via: input.tenant, exists: false, predicate: redirect_client == input.client}\n",
    );
    assert!(
        RawSpecFile::parse(&both).is_err() || assemble(&both).is_err(),
        "both shapes in one key are refused"
    );
}

#[test]
fn issue_211_one_related_row_per_command() {
    let errors = refused(&replaced(
        SIGN_IN,
        HOLDS,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n        error: demo.signin.NoRedirectEntry\n      - name: no-configuration-again\n        when_related: {via: input.tenant, exists: false}\n",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "exists: false"
        ),
        "{errors}"
    );
}

#[test]
fn issue_211_the_guard_round_trips_through_the_document_form() {
    let spec = accepted(SIGN_IN);
    let command = &spec.commands()[&"demo.signin.InitiateSignIn".parse().unwrap()];
    let written = serde_yaml::to_string(command).unwrap();
    assert!(written.contains("when_related:"), "{written}");
    assert!(written.contains("via: input.tenant"), "{written}");
    assert!(written.contains("exists: false"), "{written}");
    let reread: ess_domain::command::RawCommandSpec = serde_yaml::from_str(&written).unwrap();
    let again = serde_yaml::to_string(&reread).unwrap();
    assert_eq!(
        serde_yaml::from_str::<serde_yaml::Value>(&again).unwrap()["outcomes"],
        serde_yaml::from_str::<serde_yaml::Value>(&written).unwrap()["outcomes"]
    );
}

#[test]
fn issue_211_a_command_does_not_select_on_the_related_row_and_its_own_subject_at_once() {
    let errors = refused(&replaced(
        SIGN_IN,
        "      - name: initiated\n",
        "      - name: unknown\n        unknown_instance: true\n        error: demo.signin.NoConfiguration\n      - name: initiated\n",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "`unknown_instance` branch"
        ),
        "{errors}"
    );
}

/// [`SIGN_IN`] with an enum field on the configuration and the creation guarded as well: every
/// branch is guarded, and the partition over the rows that exist has to prove itself.
fn planned(created_when: &str) -> String {
    let text = replaced(
        SIGN_IN,
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n  - name: demo.signin.Plan\n    kind: enum\n    variants: [Basic, Premium]\n",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    lifecycle",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n    lifecycle",
    );
    let text = replaced(
        &text,
        HOLDS,
        "        when_related: {via: input.tenant, predicate: plan == Basic}\n",
    );
    replaced(
        &text,
        "      - name: initiated\n",
        &format!("      - name: initiated\n        when_related: {{via: input.tenant, predicate: {created_when}}}\n"),
    )
}

#[test]
fn issue_211_a_complete_partition_of_the_related_row_needs_no_default() {
    let spec = accepted(&planned("plan == Premium"));
    let command = &spec.commands()[&"demo.signin.InitiateSignIn".parse().unwrap()];
    let initiated = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "initiated")
        .unwrap();
    assert!(
        matches!(initiated.condition, OutcomeCondition::Related { .. }),
        "a creating branch carries the guard: {:?}",
        initiated.condition
    );
    assert!(command.default_outcome().is_none());
}

#[test]
fn issue_211_two_branches_one_related_row_selects_are_conflicting() {
    let errors = refused(&planned("plan != Premium"));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "plan = Basic"
        ),
        "{errors}"
    );
    assert!(
        has(
            &errors,
            ValidationCode::NonExhaustiveBranches,
            "plan = Premium"
        ),
        "{errors}"
    );
}

// ---- the related row's held state (beyond10x/ess#229, `ess/20`) ------------------------------

/// A move guarded by the held state of another entity's row: a release is published only for a
/// candidate in state `Accepted`.
pub const RELEASE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-release.yaml");

const NOT_ACCEPTED: &str =
    "        when_related: {via: input.candidate, predicate: state != Accepted}\n";

fn release_at(format: &str) -> String {
    replaced(RELEASE, "format: ess/20\n", &format!("format: {format}\n"))
}

#[test]
fn issue_229_a_related_guard_reads_the_related_rows_held_state_under_ess_20() {
    let spec = assemble_as(RELEASE, "release.yaml").unwrap_or_else(|errors| panic!("{errors}"));
    let command = &spec.commands()[&"demo.release.PublishRelease".parse().unwrap()];
    let refusal = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "not-accepted")
        .unwrap();
    let OutcomeCondition::Related { via, test, .. } = &refusal.condition else {
        panic!("expected a related guard: {:?}", refusal.condition)
    };
    assert_eq!(via, "candidate");
    let RelatedTest::Holds(predicate) = test else {
        panic!("expected a predicate: {test:?}")
    };
    assert_eq!(predicate.to_string(), "state != Accepted");
    assert!(refusal.subject.is_none());
    assemble_as(&as_json(RELEASE), "release.json").unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn issue_229_state_in_a_related_guard_is_refused_below_ess_20_naming_it() {
    for format in ["ess/19", "ess/18"] {
        let text = release_at(format);
        for (form, file, text) in [
            ("yaml", "release.yaml", text.clone()),
            ("json", "release.json", as_json(&text)),
        ] {
            let errors = assemble_as(&text, file)
                .err()
                .unwrap_or_else(|| panic!("{form} at {format} must refuse"));
            assert!(
                has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/20"),
                "{form} at {format}: {errors}"
            );
            assert!(
                has(
                    &errors,
                    ValidationCode::UnsupportedFormatVersion,
                    "when_related"
                ),
                "{form} at {format} names the key: {errors}"
            );
            assert!(
                !has(&errors, ValidationCode::UnobservableFact, "state"),
                "{form} at {format}: the header is wrong, not the document: {errors}"
            );
        }
    }
}

#[test]
fn issue_229_the_related_state_is_typed_by_the_related_lifecycle() {
    let errors = assemble_as(
        &replaced(
            RELEASE,
            NOT_ACCEPTED,
            "        when_related: {via: input.candidate, predicate: state != Published}\n",
        ),
        "release.yaml",
    )
    .err()
    .unwrap_or_else(|| panic!("a state of another lifecycle is refused"));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("Published")),
        "{errors}"
    );
}

#[test]
fn issue_229_the_related_state_enters_the_partition() {
    // Both branches over the state, and they overlap on `Proposed`.
    let errors = assemble_as(
        &replaced(
            RELEASE,
            "      - name: published\n",
            "      - name: published\n        when_related: {via: input.candidate, predicate: state != Rejected}\n",
        ),
        "release.yaml",
    )
    .err()
    .unwrap_or_else(|| panic!("an unknown state is refused"));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("Rejected")),
        "{errors}"
    );
    let errors = assemble_as(
        &replaced(
            RELEASE,
            "      - name: published\n",
            "      - name: published\n        when_related: {via: input.candidate, predicate: state == Proposed}\n",
        ),
        "release.yaml",
    )
    .err()
    .unwrap_or_else(|| panic!("two branches over one state are refused"));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "state = Proposed"
        ),
        "{errors}"
    );
    assert!(
        has(
            &errors,
            ValidationCode::NonExhaustiveBranches,
            "state = Accepted"
        ),
        "{errors}"
    );
}

// ---- a present related refusal beside wrong_state (beyond10x/ess#282, `ess/22`) -------------

fn issue_282_overlap(wrong_state_first: bool) -> String {
    let text = release_at("ess/22");
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n",
    );
    let wrong =
        "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n";
    if wrong_state_first {
        replaced(
            &text,
            "      - name: not-accepted\n",
            &format!("{wrong}      - name: not-accepted\n"),
        )
    } else {
        replaced(
            &text,
            "      - name: published\n",
            &format!("{wrong}      - name: published\n"),
        )
    }
}

fn issue_282_objective_switch(wrong_state_first: bool) -> String {
    issue_282_overlap(wrong_state_first)
        .replace("Release", "Objective")
        .replace("release", "objective")
        .replace("Candidate", "Switch")
        .replace("candidate", "switch")
        .replace("Accepted", "Running")
        .replace("accepted", "running")
        .replace("Proposed", "Paused")
        .replace("proposed", "paused")
        .replace("Published", "Executing")
        .replace("published", "executing")
        .replace("PublishObjective", "StartObjective")
        .replace("publish", "start")
}

fn issue_282_deployment_approval(wrong_state_first: bool) -> String {
    issue_282_overlap(wrong_state_first)
        .replace("Release", "Deployment")
        .replace("release", "deployment")
        .replace("Candidate", "Release")
        .replace("candidate", "release")
        .replace("Accepted", "Approved")
        .replace("accepted", "approved")
        .replace("Published", "Deployed")
        .replace("published", "deployed")
        .replace("PublishDeployment", "Deploy")
        .replace("publish", "deploy")
}

#[test]
fn issue_282_related_refusal_and_wrong_state_validate_under_ess_22() {
    for (shape, build) in [
        (
            "objective/switch",
            issue_282_objective_switch as fn(bool) -> String,
        ),
        ("deployment/approval", issue_282_deployment_approval),
    ] {
        for wrong_state_first in [true, false] {
            let source = build(wrong_state_first);
            assemble_as(&source, "issue-282.yaml").unwrap_or_else(|errors| {
                panic!(
                    "{shape}, wrong_state_first={wrong_state_first}: {errors}\n{source}"
                )
            });
        }
    }
}

#[test]
fn issue_282_overlap_below_ess_22_keeps_its_refusal() {
    for format in ["ess/21", "ess/20"] {
        let source = replaced(
            &issue_282_overlap(true),
            "format: ess/22\n",
            &format!("format: {format}\n"),
        );
        let errors = assemble_as(&source, "overlap.yaml")
            .err()
            .unwrap_or_else(|| panic!("{format} must retain the overlap refusal"));
        assert!(
            has(
                &errors,
                ValidationCode::ConflictingDeclaration,
                "`wrong_state` branch"
            ),
            "{format}: {errors}"
        );
    }
}
