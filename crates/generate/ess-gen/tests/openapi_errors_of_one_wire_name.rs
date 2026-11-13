//! The `OpenAPI` projection of two outcomes answering one status with `{error: error}` over two
//! errors of one wire name, one of which always writes a field the other never writes
//! (`ESS-HTTP-025`, `docs/design/http-presentation-binding.md`, "Status per outcome"): the
//! response holds both bodies, each as its own schema, and no discriminator, because one value of
//! `error` names both.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{derived, Binding};
use ess_gen::openapi::BoundOpenApi;
use serde_json::{json, Value};

const NOTES: &str = include_str!("../../../specify/ess-http/tests/fixtures/notes.yaml");

/// `notes.yaml` whose `blank` reports `Declined`, of the wire name of `Refused`, with an extra
/// field `details` that `Refused` never writes.
fn model() -> EssIr {
    let text = NOTES
        .replace(
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
            "  - name: demo.notes.Refused
    naming: {wire: refused}
    fields:
      - {name: message, type: String}
  - name: demo.notes.Declined
    naming: {wire: refused}
    fields:
      - {name: message, type: String}
      - {name: details, type: String}
",
        )
        .replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    let raw = RawSpecFile::parse(&text).expect("well formed");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("notes.yaml", &text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The document of `notes-service` with `CreateNote` bound to `post /api/notes` and `keys` beside
/// its path.
fn document(ir: &EssIr, keys: &str) -> Value {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\nroutes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n{keys}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let compiled = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}\n{text}"));
    let binding = Binding::new(compiled).expect("every route has a path");
    let artifacts = run(&BoundOpenApi(binding), ir).expect("generates");
    let yaml: ess_yaml::Value =
        ess_yaml::from_str(&artifacts["openapi/notes-service.yaml"].contents).expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

fn schema<'a>(document: &'a Value, reference: &Value) -> &'a Value {
    let key = reference["$ref"]
        .as_str()
        .and_then(|reference| reference.strip_prefix("#/components/schemas/"))
        .unwrap_or_else(|| panic!("a reference: {reference}"));
    &document["components"]["schemas"][key]
}

#[test]
fn both_bodies_are_on_the_response_each_as_its_own_schema() {
    let ir = model();
    let document = document(
        &ir,
        "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n",
    );
    let response = &document["paths"]["/api/notes"]["post"]["responses"]["409"];
    assert_eq!(response["x-ess-outcomes"], json!(["negative", "blank"]));
    let content = &response["content"]["application/json"]["schema"];
    let members = content["oneOf"]
        .as_array()
        .unwrap_or_else(|| panic!("a `oneOf` of both bodies: {response}"));
    assert_eq!(members.len(), 2, "{response}");
    assert!(
        content.get("discriminator").is_none(),
        "one value of `error` names both bodies, so no discriminator maps it: {content}"
    );
    let [negative, blank] = [&members[0], &members[1]].map(|member| schema(&document, member));
    for (body, fields) in [
        (negative, vec!["error", "message"]),
        (blank, vec!["error", "message", "details"]),
    ] {
        assert_eq!(body["required"], json!(fields), "{body}");
        assert_eq!(body["additionalProperties"], json!(false), "{body}");
        assert_eq!(body["properties"]["error"]["const"], "refused", "{body}");
    }
}

/// Distinct wire names keep the discriminator, which maps each name to its body.
#[test]
fn distinct_wire_names_keep_their_discriminator() {
    let ir = model();
    let document = document(
        &ir,
        "    outcomes:\n      too-large: {status: 409, body: {error: error}}\n      negative: {status: 409, body: {error: error}}\n",
    );
    let content = &document["paths"]["/api/notes"]["post"]["responses"]["409"]["content"]
        ["application/json"]["schema"];
    assert_eq!(
        content["discriminator"]["propertyName"], "error",
        "{content}"
    );
    assert_eq!(
        content["discriminator"]["mapping"],
        json!({
            "refused": "#/components/schemas/demo.notes.CreateNote.negative.Response",
            "too_large": "#/components/schemas/demo.notes.CreateNote.too-large.Response",
        }),
        "{content}"
    );
}
