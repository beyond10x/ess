//! `ess generate --kind openapi --http` projects a binding's operation ids and servers
//! (<https://github.com/beyond10x/ess/issues/653>, `docs/design/http-presentation-binding.md`,
//! "Operation ids and servers").

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    process::{Command, Output},
};

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/catalog.yaml");

fn ess(root: &TemporaryDirectory, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The digest a generated schema's provenance carries.
fn digest(root: &TemporaryDirectory) -> String {
    let generated = ess(
        root,
        &[
            "generate", "--path", "model", "--kind", "schema", "--out", "digest",
        ],
    );
    assert!(generated.status.success(), "{}", text(&generated));
    let schema =
        fs::read_to_string(root.join("digest/schema/events/catalog.items.ItemTouched.schema.json"))
            .unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();
    schema["x-ess-provenance"]["source_digest"]
        .as_str()
        .unwrap_or_else(|| panic!("no source_digest in {schema}"))
        .to_owned()
}

#[test]
fn generate_openapi_projects_operation_ids_and_servers() {
    let root = TemporaryDirectory::create("ess-http-presentation-cli").unwrap();
    fs::create_dir_all(root.join("model")).unwrap();
    fs::write(root.join("model/system.yaml"), MODEL).unwrap();
    let digest = digest(&root);
    fs::write(
        root.join("http.yaml"),
        format!(
            "type: ess-http/1
specification:
  system: catalog
  version: v1
  source_digest: sha256:{digest}
servers:
  items-service:
    - {{url: 'https://api.example.test/v2', description: Production}}
routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: put
    path: /items/{{item_id}}
    parameters: {{item_id: input.item_id}}
"
        ),
    )
    .unwrap();

    let validate = ess(
        &root,
        &[
            "specify",
            "http",
            "validate",
            "--path",
            "model",
            "--binding",
            "http.yaml",
        ],
    );
    assert!(validate.status.success(), "{}", text(&validate));

    let generated = ess(
        &root,
        &[
            "generate",
            "--path",
            "model",
            "--kind",
            "openapi",
            "--http",
            "http.yaml",
            "--out",
            "bound",
        ],
    );
    assert!(generated.status.success(), "{}", text(&generated));
    let written = fs::read_to_string(root.join("bound/openapi/items-service.yaml")).unwrap();
    let yaml: ess_yaml::Value = ess_yaml::from_str(&written).unwrap();
    let document = serde_json::to_value(yaml).unwrap();
    assert_eq!(
        document["paths"]["/items/{item_id}"]["put"]["operationId"],
        "listItem"
    );
    assert_eq!(
        document["servers"],
        serde_json::json!([{"url": "https://api.example.test/v2", "description": "Production"}])
    );

    // A refused id exits 1 with its code and path.
    let refused = fs::read_to_string(root.join("http.yaml")).unwrap().replace(
        "operation_id: listItem",
        "operation_id: catalog.items.TouchItem",
    );
    fs::write(root.join("refused.yaml"), refused).unwrap();
    let validate = ess(
        &root,
        &[
            "specify",
            "http",
            "validate",
            "--path",
            "model",
            "--binding",
            "refused.yaml",
        ],
    );
    assert_eq!(validate.status.code(), Some(1), "{}", text(&validate));
    assert!(
        text(&validate).contains("ESS-HTTP-032")
            && text(&validate).contains("routes[0].operation_id"),
        "{}",
        text(&validate)
    );
}
