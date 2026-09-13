//! A component's `settings:` list, and the refusals a component can make on its own.
//!
//! The two refusals that need the rest of the specification — a type nothing declares, and a type
//! that is an entity — are exercised in `ess-compiler/tests/component_settings.rs`, because
//! `compile_locating` is where they currently run. These four are answerable from the component's
//! own declaration, so they run at conversion and a document that trips one never becomes a
//! `Specification`.
//!
//! That placement is wrong, and the two `#[ignore]`d cases at the foot of this file say how: a
//! consumer of `ess-domain` alone does not reach the compiler, and `Specification::validate` is
//! `pub` and documented as checking every reference in the specification. The fix is one line in
//! `spec.rs`, a file another unit of this wave owns; it is written out and waiting, and the two
//! cases come off `#[ignore]` when it lands. They are not relaxed in the meantime.

use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;
use ess_primitives::error::ValidationCode;

/// The header and the one domain every fixture here shares.
const HEADER: &str = "\
format: ess/1
system: connectors
version: v1
domains:
  - connectors.config
";

const DOMAIN: &str = "\
domain: connectors.config
types:
  - name: connectors.config.StateRoot
    kind: newtype
    of: String
  - name: connectors.config.Credential
    kind: newtype
    of: String
  - name: connectors.config.MaybeRoot
    kind: newtype
    of: String
";

/// A components file declaring `settings`, indented into the list entry.
///
/// The block arrives written flush left, one entry per `- name:`, and is indented here: a Rust
/// string literal's `\`-continuation eats the leading whitespace of the line it joins, which is
/// exactly the whitespace YAML reads.
fn components(settings: &str) -> String {
    let mut block = String::new();
    for line in settings.lines().filter(|line| !line.trim().is_empty()) {
        block.push_str("      ");
        block.push_str(line);
        block.push('\n');
    }
    format!(
        "\
components:
  - component: connectors-cli
    owns:
      domains:
        - connectors.config
    settings:
{block}"
    )
}

fn assemble(settings: &str) -> Result<Specification, ess_primitives::error::ValidationErrors> {
    let files = [
        ("system.yaml", HEADER.to_owned()),
        ("domains/config.yaml", DOMAIN.to_owned()),
        ("components.yaml", components(settings)),
    ];
    let parsed: Vec<_> = files
        .into_iter()
        .map(|(label, text)| {
            (
                Source::new(label),
                RawSpecFile::parse(&text)
                    .unwrap_or_else(|error| panic!("{label} is well formed: {error}")),
            )
        })
        .collect();
    Specification::assemble(parsed)
}

fn refusal(settings: &str) -> ess_primitives::error::ValidationErrors {
    assemble(settings).expect_err("the settings list is refused")
}

#[test]
fn a_component_may_declare_settings() {
    let specification = assemble(
        r"
- name: state-root
  type: connectors.config.StateRoot
  required: true
  summary: Where connector state is kept.
- name: slack-bot-token
  type: connectors.config.Credential
  secret: true
- name: retry-window
  type: Optional<connectors.config.MaybeRoot>
  required: false
",
    )
    .expect("a component declaring settings is accepted");
    let component = specification
        .components()
        .values()
        .next()
        .expect("the component survived conversion");
    let serialized = serde_json::to_value(component).expect("a component serializes");
    let names: Vec<&str> = serialized["settings"]
        .as_array()
        .expect("the component carries its settings")
        .iter()
        .map(|setting| setting["name"].as_str().expect("a setting has a name"))
        .collect();
    assert_eq!(
        names,
        ["state-root", "slack-bot-token", "retry-window"],
        "the settings keep the order the document wrote them in"
    );
}

#[test]
fn two_settings_with_one_name_on_a_component_are_refused() {
    let errors = refusal(
        r"
- name: state-root
  type: connectors.config.StateRoot
  required: true
- name: state-root
  type: connectors.config.Credential
  required: true
",
    );
    assert!(
        errors.contains(ValidationCode::DuplicateDeclaration),
        "a repeated setting name is `duplicate_declaration`: {errors}"
    );
    assert!(
        errors.to_string().contains("state-root"),
        "the refusal names the setting: {errors}"
    );
}

#[test]
fn a_secret_setting_beside_a_literal_value_is_refused() {
    let errors = refusal(
        r"
- name: slack-bot-token
  type: connectors.config.Credential
  required: true
  secret: true
  value: xoxb-not-a-real-token
",
    );
    assert!(
        errors.contains(ValidationCode::ConflictingDeclaration),
        "a secret carrying its own value is `conflicting_declaration`: {errors}"
    );
    assert!(
        errors.to_string().contains("slack-bot-token"),
        "the refusal names the setting: {errors}"
    );
}

#[test]
fn required_false_without_an_optional_type_is_refused() {
    let errors = refusal(
        r"
- name: state-root
  type: connectors.config.StateRoot
  required: false
",
    );
    assert!(
        errors.contains(ValidationCode::ConflictingDeclaration),
        "`required: false` beside a type that admits no absence is `conflicting_declaration`: \
         {errors}"
    );
    assert!(
        errors.to_string().contains("Optional<"),
        "the refusal says what the type would have to be: {errors}"
    );
}

/// The question the story leaves open, answered in the direction it did not name.
///
/// `Optional` is the model's only statement that a value may be absent — `ess-domain`'s
/// `binding` module says so in as many words — so `required: true` over an `Optional<…>` is the
/// same contradiction as `required: false` over a type that is not one, read from the other end.
/// Refusing only one direction would leave `Optional` meaning two things depending on which field
/// a reader looked at first.
#[test]
fn required_true_with_an_optional_type_is_refused() {
    let errors = refusal(
        r"
- name: state-root
  type: Optional<connectors.config.StateRoot>
  required: true
",
    );
    assert!(
        errors.contains(ValidationCode::ConflictingDeclaration),
        "`required: true` beside an `Optional<…>` is `conflicting_declaration`: {errors}"
    );
}

#[test]
fn a_setting_name_outside_the_cli_name_charset_is_refused() {
    let errors = refusal(
        r"
- name: state_root
  type: connectors.config.StateRoot
  required: true
",
    );
    assert!(
        errors.contains(ValidationCode::TypeMismatch),
        "a setting name is spelt like a word a person types: {errors}"
    );
}

/// A `required:` a document is allowed to write never changes an answer, so deleting it buys
/// nothing.
///
/// This is the property that makes the `required:` refusals unevadable, and it is stated over the
/// whole grid rather than over one example. For each of the four `(required, type)` combinations:
/// the combination is accepted exactly when the restatement agrees with the type, and wherever it
/// is accepted, `is_required()` is `!is_optional()` — the same answer the type alone gives, and
/// therefore the same answer the document gives with the `required:` line removed.
///
/// Reading an unstated `required` as `false` broke the bottom-left cell: it was accepted, and it
/// answered `false` over a type that admits no absence, which is the state the top-left refusal
/// exists to prevent (`review-result:adversary-wave25-unit3-pass-1` F1). A future reader that
/// makes `required:` load-bearing again fails here rather than downstream.
#[test]
fn an_accepted_required_restates_the_type_and_never_overrides_it() {
    let plain = "connectors.config.StateRoot";
    let optional = "Optional<connectors.config.StateRoot>";
    for (written, type_ref, accepted) in [
        (Some(true), plain, true),
        (Some(false), plain, false),
        (Some(true), optional, false),
        (Some(false), optional, true),
        (None, plain, true),
        (None, optional, true),
    ] {
        let line = written.map_or_else(String::new, |value| format!("  required: {value}\n"));
        let document = format!("\n- name: state-root\n  type: {type_ref}\n{line}");
        let result = assemble(&document);
        assert_eq!(
            result.is_ok(),
            accepted,
            "`required: {written:?}` over `{type_ref}` is accepted iff the restatement agrees \
             with the type"
        );
        let Ok(specification) = result else {
            continue;
        };
        let component = specification
            .components()
            .values()
            .next()
            .expect("the component survived conversion");
        let setting = &component.settings[0];
        assert_eq!(
            setting.is_required(),
            !setting.type_ref.is_optional(),
            "`required: {written:?}` over `{type_ref}` is accepted, so it restates the type and \
             must answer what the type answers — otherwise the refusal of the disagreeing form is \
             evaded by deleting the line"
        );
    }
}

/// Silence is not a contradiction.
///
/// `required:` is a restatement of what the type already says, and a restatement can only be wrong
/// when it is made. Refusing an unstated `required` would make the field mandatory on every
/// setting whose type is not `Optional<…>`, which is the opposite of optional.
#[test]
fn an_unstated_required_is_not_refused_either_way() {
    for type_ref in [
        "connectors.config.StateRoot",
        "Optional<connectors.config.StateRoot>",
    ] {
        assemble(&format!(
            r"
- name: state-root
  type: {type_ref}
"
        ))
        .unwrap_or_else(|errors| panic!("`{type_ref}` with no `required:` is accepted: {errors}"));
    }
}

/// The derivation the deployment projector pays for, and the reason there is no sixth refusal.
///
/// The brief asks whether two distinct setting names can derive one environment variable. They
/// cannot, and this enumerates the charset rather than asserting the argument: every `CliName` of
/// up to three characters is derived, and the derived set is exactly as large as the name set. The
/// map is per character and length-preserving, so an injective map on strings of length `n` for
/// every `n` in the enumeration is injective on all of them.
#[test]
fn no_two_setting_names_derive_one_environment_variable() {
    use ess_domain::component::{environment_variable, CliName};
    use std::collections::BTreeSet;

    let alphabet: Vec<char> = ('a'..='z').chain('0'..='9').chain(['-']).collect();
    let mut names = Vec::new();
    let mut words: Vec<String> = vec![String::new()];
    for _ in 0..3 {
        let mut next = Vec::new();
        for word in &words {
            for character in &alphabet {
                let candidate = format!("{word}{character}");
                if let Ok(name) = CliName::new(&candidate) {
                    names.push(name);
                }
                next.push(candidate);
            }
        }
        words = next;
    }
    assert!(
        names.len() > 1_000,
        "the enumeration is meant to be exhaustive over three characters, not a handful: {}",
        names.len()
    );
    let derived: BTreeSet<String> = names.iter().map(environment_variable).collect();
    assert_eq!(
        derived.len(),
        names.len(),
        "two setting names derived one environment variable, so the derivation needs a refusal \
         the story does not have"
    );
    assert_eq!(
        environment_variable(&CliName::new("slack-bot-token").unwrap()),
        "SLACK_BOT_TOKEN"
    );
    assert_eq!(
        environment_variable(&CliName::new("s3-bucket2").unwrap()),
        "S3_BUCKET2"
    );
}

/// The published charset and the parser's are the same charset.
///
/// `schemars` takes a string literal, so [`CliName::PATTERN`] is written twice — once as the
/// constant the parser documents and once in the attribute the schema publishes. This is the check
/// that keeps the second copy from drifting: a schema that accepts what the parser refuses hands an
/// author a green editor and a red `ess validate`, which is worse than publishing no pattern.
#[test]
fn the_published_setting_name_pattern_is_the_one_the_parser_enforces() {
    use ess_domain::component::{CliName, RawComponentSetting};

    let schema =
        serde_json::to_value(schemars::schema_for!(RawComponentSetting)).expect("serialises");
    assert_eq!(
        schema["properties"]["name"]["pattern"],
        serde_json::json!(CliName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
}

// ---------------------------------------------------------------------------
// The seam: the two settings refusals a component cannot make alone run from
// `ess_compiler::resolve::compile_locating`, not from `Specification::validate`.
//
// `Specification::validate` is `pub` and documented at `spec.rs` as "Checks every reference in the
// specification", and a setting's `type:` is a reference. The repository's own consumer catalogue
// names two shipped profiles that stop here and never enter the compiler: `authored-assembly`
// ("Assemble original source parts and run model validation") and `authored-validation`
// ("Validate a retained specification including manually assembled model values"), both in
// `crates/edge/ess-xtask/src/consumer_coverage/profiles.json`.
//
// Both cases below assert that documented promise, not the implementation's current shape. They
// are `#[ignore]`d because closing them is one `errors.extend(...)` in
// `crates/specify/ess-domain/src/spec.rs`, which wave 25 assigned to another unit — the patch is
// written and held by the coordinator. They are ignored rather than deleted or relaxed, and they
// must go green unchanged the moment it is applied.
// ---------------------------------------------------------------------------

/// A domain that declares an entity as well as types, so a setting can be typed by one.
const DOMAIN_WITH_AN_ENTITY: &str = "\
domain: connectors.config
types:
  - name: connectors.config.StateRoot
    kind: newtype
    of: String
  - name: connectors.config.RootId
    kind: newtype
    of: String
entities:
  - name: connectors.config.Workspace
    identity:
      name: workspace_id
      type: connectors.config.RootId
    fields:
      - name: root
        type: connectors.config.StateRoot
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
      transitions: []
";

fn assemble_against_an_entity(
    settings: &str,
) -> Result<Specification, ess_primitives::error::ValidationErrors> {
    let parsed: Vec<_> = [
        ("system.yaml", HEADER.to_owned()),
        ("domains/config.yaml", DOMAIN_WITH_AN_ENTITY.to_owned()),
        ("components.yaml", components(settings)),
    ]
    .into_iter()
    .map(|(label, text)| {
        (
            Source::new(label),
            RawSpecFile::parse(&text)
                .unwrap_or_else(|error| panic!("{label} is well formed: {error}")),
        )
    })
    .collect();
    Specification::assemble(parsed)
}

/// `undeclared_reference` is an `ess-domain` code; the check that raises it is not in `ess-domain`'s
/// validation pass.
#[test]
#[ignore = "review-result:adversary-wave25-unit3-pass-1 F3 — `validate_setting_types` runs from \
            `compile_locating` rather than from `Specification::validate`; the one-line fix is in \
            `spec.rs`, owned by another unit of this wave, and is held as \
            `coordinator/spec-rs-placement.patch`"]
fn assembly_refuses_a_setting_typed_by_something_nothing_declares() {
    let errors = assemble_against_an_entity(
        r"
- name: state-root
  type: connectors.config.Nowhere
  required: true
",
    )
    .err()
    .unwrap_or_else(|| {
        panic!(
            "`Specification::validate` is documented as checking every reference in the \
             specification, and a setting's `type:` is a reference; a consumer entering at \
             `Specification::assemble` (consumer profile `authored-assembly`) is handed a sealed \
             specification naming a type nothing declares"
        )
    });
    assert!(
        errors.contains(ValidationCode::UndeclaredReference),
        "a setting type nothing declares is `undeclared_reference`: {errors}"
    );
}

/// The load-bearing refusal — the one that stops configuration becoming a second entity model — is
/// the one a consumer stopping at assembly is most exposed to, because the name *does* resolve.
#[test]
#[ignore = "review-result:adversary-wave25-unit3-pass-1 F3 — `validate_setting_types` runs from \
            `compile_locating` rather than from `Specification::validate`; the one-line fix is in \
            `spec.rs`, owned by another unit of this wave, and is held as \
            `coordinator/spec-rs-placement.patch`"]
fn assembly_refuses_a_setting_typed_by_an_entity() {
    let errors = assemble_against_an_entity(
        r"
- name: workspace
  type: connectors.config.Workspace
  required: true
",
    )
    .err()
    .unwrap_or_else(|| {
        panic!(
            "a setting typed by an entity is accepted by `Specification::assemble`; the refusal \
             that stops configuration becoming a second entity model runs only from \
             `ess_compiler::resolve::compile_locating`"
        )
    });
    assert!(
        errors.contains(ValidationCode::TypeMismatch),
        "a setting typed by an entity is `type_mismatch`: {errors}"
    );
}
