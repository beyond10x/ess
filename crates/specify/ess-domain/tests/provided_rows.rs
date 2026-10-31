//! An entity may declare that the deployment provides its rows, `provided: true`, from `ess/33`
//! (story:a-fixture-may-be-a-row-the-deployment-already-holds,
//! architecture-decision-record:a-deployment-may-provide-an-entitys-rows).
//!
//! Below `ess/33` the key is refused as `unsupported_format_version` naming `ess/33`, at the
//! entity's `provided` key; `false` is the same document as leaving it out.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn model(format: u32, marker: &str) -> String {
    format!(
        "format: ess/{format}
system: linking
version: v1
domain: linking.links
types:
  - {{name: linking.links.AppId, kind: newtype, of: Uuid}}
entities:
  - name: linking.links.App
{marker}    identity: {{name: app_id, type: linking.links.AppId}}
    fields: [{{name: title, type: String}}]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
"
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("links.yaml"), raw)])
}

fn app(spec: &Specification) -> &ess_domain::entity::EntitySpec {
    spec.entities()
        .values()
        .find(|entity| entity.name.to_string() == "linking.links.App")
        .expect("the entity is declared")
}

#[test]
fn an_entity_may_declare_that_the_deployment_provides_its_rows_from_ess_33() {
    let spec = assemble(&model(33, "    provided: true\n"))
        .unwrap_or_else(|errors| panic!("`provided: true` validates at ess/33: {errors}"));
    assert!(app(&spec).provided);
}

#[test]
fn below_ess_33_the_key_is_refused_naming_ess_33() {
    for format in [27, 32] {
        let errors = assemble(&model(format, "    provided: true\n"))
            .expect_err("the key is refused below ess/33");
        let refusal = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::UnsupportedFormatVersion)
            .unwrap_or_else(|| panic!("ess/{format}: an unsupported_format_version: {errors}"));
        assert!(
            refusal.location.ends_with("provided"),
            "ess/{format}: {refusal:?}"
        );
        assert!(
            refusal.message.contains("ess/33"),
            "ess/{format}: {refusal:?}"
        );
        assert!(
            refusal
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("ess/33")),
            "ess/{format}: {refusal:?}"
        );
    }
}

#[test]
fn false_is_the_document_without_the_key() {
    for format in [32, 33] {
        let without = assemble(&model(format, "")).unwrap_or_else(|errors| panic!("{errors}"));
        let written_false =
            assemble(&model(format, "    provided: false\n")).unwrap_or_else(|errors| {
                panic!("`provided: false` validates at ess/{format}: {errors}")
            });
        assert!(!app(&without).provided);
        assert_eq!(app(&without), app(&written_false), "ess/{format}");
        let serialized = serde_json::to_string(app(&without)).expect("serializes");
        assert!(!serialized.contains("provided"), "{serialized}");
    }
}

#[test]
fn the_key_is_a_boolean() {
    let text = model(33, "    provided: deployment\n");
    let refused = RawSpecFile::parse(&text);
    assert!(refused.is_err(), "a non-boolean `provided:` does not parse");
}
