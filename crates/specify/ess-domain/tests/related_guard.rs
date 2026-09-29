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
