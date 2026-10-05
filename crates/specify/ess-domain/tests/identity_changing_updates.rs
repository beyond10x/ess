//! An `updates:` whose `sets:` writes the entity's identity re-keys the record (ess/23,
//! beyond10x/ess#429, `docs/design/identity-changing-updates.md`).
//!
//! The issue's rename, written fresh: `demo.vault.Secret` identified by `name`, renamed by
//! `RenameSecret {name, new_name}`. It validates at `ess/23` with its collision answer declared, and
//! is refused below `ess/23` naming it, without one (`missing_declaration`), on an entity a relation
//! carries, beside `compensates:` and in a create-or-update pair.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/identity-changing-updates.yaml");

const RENAMED: &str = "command.demo.vault.RenameSecret.outcomes.renamed";

const TAKEN: &str = "      - name: taken
        when_related:
          entity: demo.vault.Secret
          where: name == input.new_name
          exists: true
        error: demo.vault.NameTaken
";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("vault.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
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

fn only(errors: &[ValidationError]) -> &ValidationError {
    assert_eq!(errors.len(), 1, "{errors:#?}");
    &errors[0]
}

fn repository(relative: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    std::fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

#[test]
fn ess_23_is_admitted_and_recorded() {
    assert!(
        ess_domain::system::SUPPORTED_FORMATS.contains(&23),
        "{:?}",
        ess_domain::system::SUPPORTED_FORMATS
    );
    assert_eq!(ess_domain::system::FormatVersion::V23.major(), 23);
    assert!(ess_domain::system::is_supported_format(
        ess_domain::system::FormatVersion::V23
    ));
    // The toolchain model, the release table the docs lane reads, and the version history.
    assert!(
        repository("models/toolchain/domains/specify.yaml").contains("\"ess/23\""),
        "SpecificationFormat lists ess/23"
    );
    assert!(
        repository("crates/edge/ess-xtask/src/docs.rs").contains("(\"ess\", 23, "),
        "FORMAT_RELEASES has an ess/23 row"
    );
    assert!(
        repository("website/docs/reference/spec-versions.md")
            .lines()
            .any(|line| line.starts_with("| `ess/23` |")),
        "spec-versions.md has an ess/23 row"
    );
}

#[test]
fn the_rename_validates_at_ess_23() {
    let spec = accepted(MODEL);
    let command = &spec.commands()[&"demo.vault.RenameSecret".parse().unwrap()];
    let renamed = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "renamed")
        .unwrap();
    assert!(renamed.sets.contains_key("name"));
}

#[test]
fn identity_write_below_ess23_is_refused_naming_ess23() {
    for format in ["ess/22", "ess/16"] {
        let text = replaced(MODEL, "format: ess/23\n", &format!("format: {format}\n"));
        // The collision answer is an `ess/22` form; below it the row-set guard is refused on its
        // own, so `ess/16` drops it and keeps only the identity write.
        let text = if format == "ess/16" {
            replaced(&text, TAKEN, "")
        } else {
            text
        };
        let errors = refused(&text);
        let error = only(&errors);
        assert_eq!(
            error.code,
            ValidationCode::UnsupportedFormatVersion,
            "{format}: {error:#?}"
        );
        assert_eq!(error.location, format!("{RENAMED}.sets.name"), "{format}");
        assert!(error.message.contains("ess/23"), "{format}: {error:#?}");
        assert!(
            error
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("ess/23")),
            "{format}: {error:#?}"
        );
    }
}

#[test]
fn identity_write_without_collision_answer_is_refused() {
    let text = replaced(MODEL, TAKEN, "");
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(error.code, ValidationCode::MissingDeclaration, "{error:#?}");
    assert_eq!(error.location, RENAMED);
    assert!(error.message.contains("`name`"), "{error:#?}");
    assert!(
        error
            .hint
            .as_deref()
            .is_some_and(|hint| hint.contains("when_related") && hint.contains("input.new_name")),
        "{error:#?}"
    );
}

#[test]
fn a_collision_answer_over_another_selector_is_no_answer() {
    for (label, from, to) in [
        (
            "another field",
            "where: name == input.new_name",
            "where: value == input.new_name",
        ),
        (
            "another input",
            "where: name == input.new_name",
            "where: name == input.name",
        ),
        (
            "a narrower selector",
            "where: name == input.new_name",
            "where: {all: [name == input.new_name, value == \"x\"]}",
        ),
        ("an absence test", "exists: true", "exists: false"),
    ] {
        let text = replaced(MODEL, from, to);
        let errors = refused(&text);
        assert!(
            errors
                .iter()
                .any(|error| error.code == ValidationCode::MissingDeclaration
                    && error.location == RENAMED),
            "{label}: {errors:#?}"
        );
    }
}

/// The vault with a `Grant` that carries a secret's name, through `relation`.
fn with_grants(relation: &str) -> String {
    let text = replaced(
        MODEL,
        "    lifecycle:\n      initial: Stored\n",
        &format!("    relations:\n{relation}    lifecycle:\n      initial: Stored\n"),
    );
    let text = replaced(
        &text,
        "errors:\n",
        "  - name: demo.vault.Grant
    identity: {name: grant_id, type: demo.vault.SecretName}
    fields:
      - {name: secret, type: demo.vault.SecretName}
    lifecycle:
      initial: Held
      states: [Held]
      terminal: [Held]
      transitions: []
errors:\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "events:
  - name: demo.vault.GrantIssued
    fields:
      - {name: grant_id, type: demo.vault.SecretName}\n",
    );
    let text = replaced(
        &text,
        "may: [demo.vault.StoreSecret, demo.vault.RenameSecret]",
        "may: [demo.vault.StoreSecret, demo.vault.RenameSecret, demo.vault.IssueGrant]",
    );
    replaced(
        &text,
        "views:\n",
        "  - name: demo.vault.IssueGrant
    input:
      - {name: grant_id, type: demo.vault.SecretName}
      - {name: secret, type: demo.vault.SecretName}
    outcomes:
      - name: issued
        creates: demo.vault.Grant
        instance: grant_id
        sets: {secret: input.secret}
        emits: [demo.vault.GrantIssued]
        payload:
          demo.vault.GrantIssued: {grant_id: input.grant_id}
views:\n",
    )
}

#[test]
fn identity_write_refused_on_relation_carried_entity() {
    let relation = "      - {name: grants, kind: owns, target: demo.vault.Grant, cardinality: many, via: secret}\n";
    // The relation, written without the rename, is admitted: the refusal is the re-key's.
    accepted(&replaced(
        &with_grants(relation),
        "        sets: {name: input.new_name}\n",
        "        sets: {value: \"renamed\"}\n",
    ));
    let errors = refused(&with_grants(relation));
    let error = only(&errors);
    assert_eq!(
        error.code,
        ValidationCode::UnsupportedConstruct,
        "{error:#?}"
    );
    assert_eq!(error.location, format!("{RENAMED}.sets.name"));
    assert!(
        error.message.contains("grants") && error.message.contains("demo.vault.Grant"),
        "{error:#?}"
    );
}

#[test]
fn identity_write_refused_on_an_entity_a_reference_carries() {
    // `Grant` references the secret it opens: the carrier is on `Grant`, and it names a secret.
    let text = replaced(
        &with_grants(""),
        "    relations:\n    lifecycle:\n      initial: Stored\n",
        "    lifecycle:\n      initial: Stored\n",
    );
    let text = replaced(
        &text,
        "      - {name: secret, type: demo.vault.SecretName}\n    lifecycle:\n      initial: Held\n",
        "      - {name: secret, type: demo.vault.SecretName}\n    relations:\n      - {name: opens, kind: references, target: demo.vault.Secret, cardinality: one, via: secret}\n    lifecycle:\n      initial: Held\n",
    );
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(
        error.code,
        ValidationCode::UnsupportedConstruct,
        "{error:#?}"
    );
    assert_eq!(error.location, format!("{RENAMED}.sets.name"));
    assert!(error.message.contains("opens"), "{error:#?}");
}

#[test]
fn identity_write_refused_beside_compensates() {
    let text = replaced(
        MODEL,
        "      - {name: no-such-secret, unknown_instance: true, error: demo.vault.NoSuchSecret}\n",
        "      - {name: no-such-secret, unknown_instance: true, error: demo.vault.NoSuchSecret}
      - name: moved-aside
        external: the upstream refuses and parks the secret under the new name
        error: demo.vault.NameTaken
        compensates: true
        updates: demo.vault.Secret
        instance: name
        sets: {name: input.new_name}\n",
    );
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(
        error.code,
        ValidationCode::UnsupportedConstruct,
        "{error:#?}"
    );
    assert_eq!(
        error.location,
        "command.demo.vault.RenameSecret.outcomes.moved-aside.sets.name"
    );
    assert!(error.message.contains("compensates"), "{error:#?}");
}

#[test]
fn identity_write_refused_in_create_or_update_pair() {
    let text = replaced(
        MODEL,
        "      - {name: no-such-secret, unknown_instance: true, error: demo.vault.NoSuchSecret}\n",
        "      - name: created
        unknown_instance: true
        creates: demo.vault.Secret
        instance: name
        sets: {value: \"\"}
        emits: [demo.vault.SecretStored]
        payload:
          demo.vault.SecretStored: {name: input.name}\n",
    );
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(
        error.code,
        ValidationCode::UnsupportedConstruct,
        "{error:#?}"
    );
    assert_eq!(error.location, format!("{RENAMED}.sets.name"));
    assert!(error.message.contains("unknown_instance"), "{error:#?}");
}

#[test]
fn a_set_effect_keeps_refusing_an_identity_write() {
    // `affects:` beside the rename: the entry's identity write is today's refusal, unchanged.
    let text = replaced(
        MODEL,
        "        sets: {name: input.new_name}\n",
        "        sets: {name: input.new_name}
        affects:
          - entity: demo.vault.Secret
            where: value == \"stale\"
            sets: {name: input.new_name}\n",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration
                && error.location.ends_with("affects[0].sets.name")),
        "{errors:#?}"
    );
}
