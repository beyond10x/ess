//! The two settings refusals that need the whole specification, and the format promise.
//!
//! A setting's `type` is resolved against the same registry every other reference is — the system's
//! types plus the enum each entity's lifecycle forms — so "nothing declares it" and "that is an
//! entity, not a type" are questions a component cannot answer alone. They run at the compiler's
//! entrance, beside the revalidation of the sealed specification, and reach `ess validate` through
//! it.
//!
//! The format promise is here too, because it is a statement about compiled bytes: a document that
//! declares no settings compiles to exactly the bytes it compiled to before the key existed.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

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

/// A components file, with the settings block indented into the list entry.
///
/// The block arrives flush left: a Rust string literal's `\`-continuation eats the leading
/// whitespace of the line it joins, which is exactly the whitespace YAML reads.
fn components(settings: &str) -> String {
    if settings.trim().is_empty() {
        return "\
components:
  - component: connectors-cli
    owns:
      domains:
        - connectors.config
"
        .to_owned();
    }
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

/// Compiles the fixture, or returns every diagnostic it produced, rendered.
fn compiled(settings: &str) -> Result<ess_compiler::EssIr, String> {
    let files = [
        ("system.yaml", HEADER.to_owned()),
        ("domains/config.yaml", DOMAIN.to_owned()),
        ("components.yaml", components(settings)),
    ];
    let mut sources = SourceMap::new();
    let parsed: Vec<_> = files
        .iter()
        .map(|(label, text)| {
            sources.insert((*label).to_owned(), text.clone());
            (
                Source::new(*label),
                RawSpecFile::parse(text)
                    .unwrap_or_else(|error| panic!("{label} is well formed: {error}")),
            )
        })
        .collect();
    let specification = match Specification::assemble(parsed) {
        Ok(specification) => specification,
        Err(errors) => return Err(errors.to_string()),
    };
    compile(&specification, &sources).map_err(|diagnostics| {
        diagnostics
            .as_slice()
            .iter()
            .map(|diagnostic| {
                // The details are part of the assertion, not decoration: the `ess-domain` code the
                // story names for each refusal reaches a consumer through `Detail::Note`, and a
                // test that read only the message would pass with any code at all.
                format!(
                    "{} {} {}",
                    diagnostic.code,
                    diagnostic.message,
                    serde_json::to_string(&diagnostic.details).expect("details serialize")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

#[test]
fn a_setting_typed_by_something_nothing_declares_is_refused() {
    let refusal = compiled(
        r"
- name: state-root
  type: connectors.config.Nowhere
  required: true
",
    )
    .expect_err("an undeclared setting type is refused");
    assert!(
        refusal.contains("undeclared_reference"),
        "a setting type nothing declares is `undeclared_reference`: {refusal}"
    );
    assert!(
        refusal.contains("connectors.config.Nowhere"),
        "the refusal names the type: {refusal}"
    );
}

#[test]
fn a_setting_typed_by_an_entity_is_refused() {
    let refusal = compiled(
        r"
- name: workspace
  type: connectors.config.Workspace
  required: true
",
    )
    .expect_err("a setting typed by an entity is refused");
    assert!(
        refusal.contains("type_mismatch"),
        "a setting typed by an entity is `type_mismatch`, not a missing reference: {refusal}"
    );
    assert!(
        refusal.contains("connectors.config.Workspace"),
        "the refusal names the entity: {refusal}"
    );
}

#[test]
fn a_component_declaring_settings_compiles_and_carries_them() {
    let ir = compiled(
        r"
- name: state-root
  type: connectors.config.StateRoot
  required: true
  summary: Where connector state is kept.
",
    )
    .expect("a component declaring a declared type compiles");
    let component = ir
        .components()
        .values()
        .next()
        .expect("the component resolved");
    let serialized = serde_json::to_value(component).expect("a component serializes");
    let settings = serialized["settings"]
        .as_array()
        .expect("the resolved component carries its settings");
    assert_eq!(settings.len(), 1, "the setting reached the IR");
    assert_eq!(settings[0]["name"], "state-root");
    assert_eq!(
        settings[0]["type"],
        serde_json::json!({"kind": "declared", "name": "connectors.config.StateRoot"}),
        "the setting's type is a resolved handle, not a name the IR hopes resolves"
    );
    assert_eq!(settings[0]["required"], true);
}

/// The old-reader property: the key is absent, not empty.
///
/// `settings:` serialises out when the list is empty, so an IR compiled from a document that never
/// mentions the key carries no trace of it — which is the whole reason this lands inside `ess/1`
/// instead of minting `ess/2`. The byte identity that follows from it is pinned by the
/// `examples/oracle-fixture` digests, not here; this asserts the absence those digests rest on.
#[test]
fn a_document_without_settings_compiles_without_a_settings_key() {
    let ir = compiled("").expect("the settings-free fixture compiles");
    let json = ir.to_canonical_json();
    assert!(
        !json.contains("settings"),
        "an unset settings list must not appear in the compiled bytes:\n{json}"
    );
    let component = ir
        .components()
        .values()
        .next()
        .expect("the component resolved");
    let serialized = serde_json::to_string(component).expect("a component serializes");
    assert!(
        !serialized.contains("settings"),
        "nor in the component's own serialization:\n{serialized}"
    );
}

#[test]
fn compiling_a_settings_document_twice_produces_identical_bytes() {
    let settings = r"
- name: state-root
  type: connectors.config.StateRoot
  required: true
- name: retry-window
  type: Optional<connectors.config.StateRoot>
  required: false
";
    let first = compiled(settings).expect("compiles").to_canonical_json();
    let second = compiled(settings).expect("compiles").to_canonical_json();
    assert_eq!(
        first.as_bytes(),
        second.as_bytes(),
        "two compilations of one settings document must be one set of bytes"
    );
}
