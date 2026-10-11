//! `ess specify http validate` and `ess specify http compile`
//! (<https://github.com/beyond10x/ess/issues/493>, `docs/design/http-presentation-binding.md`,
//! stage 1).

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    process::{Command, Output},
};

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/catalog.yaml");

const ROUTES: &str = "routes:
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
  - command: catalog.items.ListItem
    method: post
";

struct Fixture(TemporaryDirectory);

impl Fixture {
    fn new() -> Self {
        let path = TemporaryDirectory::create("ess-http-cli").unwrap();
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(path.join("model/system.yaml"), MODEL).unwrap();
        Self(path)
    }

    fn ess(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }

    /// Writes `http.yaml` with the model's real digest.
    fn binding(&self, routes: &str) {
        let digest = self.digest_from_schema();
        fs::write(
            self.0.join("http.yaml"),
            format!(
                "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{digest}\n{routes}"
            ),
        )
        .unwrap();
    }

    /// The digest a generated schema's provenance carries, as the transport tests read it.
    fn digest_from_schema(&self) -> String {
        let generated = self.ess(&[
            "generate", "--path", "model", "--kind", "schema", "--out", "digest",
        ]);
        assert!(generated.status.success(), "{}", text(&generated));
        let schema = fs::read_to_string(
            self.0
                .join("digest/schema/events/catalog.items.ItemTouched.schema.json"),
        )
        .unwrap();
        let schema: serde_json::Value = serde_json::from_str(&schema).unwrap();
        schema["x-ess-provenance"]["source_digest"]
            .as_str()
            .unwrap_or_else(|| panic!("no source_digest in {schema}"))
            .to_owned()
    }
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn validate_and_compile_an_http_binding() {
    let fixture = Fixture::new();
    fixture.binding(ROUTES);
    let validate = fixture.ess(&[
        "specify",
        "http",
        "validate",
        "--path",
        "model",
        "--binding",
        "http.yaml",
    ]);
    assert!(validate.status.success(), "{}", text(&validate));
    assert_eq!(
        String::from_utf8_lossy(&validate.stdout),
        "http.yaml — 2 route(s), 1 open, valid\n"
    );

    let compile = fixture.ess(&[
        "specify",
        "http",
        "compile",
        "--path",
        "model",
        "--binding",
        "http.yaml",
        "--out",
        "http.json",
    ]);
    assert!(compile.status.success(), "{}", text(&compile));
    assert_eq!(
        String::from_utf8_lossy(&compile.stdout),
        "http.yaml — 2 route(s), 1 open, compiled to http.json\n"
    );
    let written = fs::read_to_string(fixture.0.join("http.json")).unwrap();
    assert!(
        written.starts_with("{\n  \"format\": \"ess-http-ir/1\",\n"),
        "{written}"
    );
    let ir: serde_json::Value = serde_json::from_str(&written).unwrap();
    assert_eq!(
        ir["routes"],
        serde_json::json!([
            {"command": "catalog.items.ListItem", "method": "post"},
            {"view": "catalog.items.ItemById", "method": "get", "path": "/items/{item_id}",
             "parameters": {"item_id": "param.item_id"}},
        ])
    );
    // The one operation the binding does not name keeps the route ESS derives for it, and the IR
    // records it as the surface the binding was checked against.
    assert_eq!(
        ir["derived_routes"],
        serde_json::json!([
            {"component": "items-service", "command": "catalog.items.TouchItem", "method": "post",
             "path": "/items/commands/TouchItem"},
        ])
    );

    let json = fixture.ess(&[
        "specify",
        "http",
        "compile",
        "--path",
        "model",
        "--binding",
        "http.yaml",
        "--format",
        "json",
    ]);
    assert!(json.status.success(), "{}", text(&json));
    assert_eq!(String::from_utf8_lossy(&json.stdout), written);
}

#[test]
fn a_refused_binding_exits_1_with_every_code_and_path() {
    let fixture = Fixture::new();
    fixture.binding(
        "routes:
  - {view: catalog.items.ItemById, method: post}
  - {command: catalog.items.PurgeItems, method: post}
",
    );
    let validate = fixture.ess(&[
        "specify",
        "http",
        "validate",
        "--path",
        "model",
        "--binding",
        "http.yaml",
    ]);
    assert_eq!(validate.status.code(), Some(1), "{}", text(&validate));
    let shown = text(&validate);
    assert!(
        shown.contains("error[ESS-HTTP-006]: routes[0].method:"),
        "{shown}"
    );
    assert!(
        shown.contains("error[ESS-HTTP-013]: routes[1].command:"),
        "{shown}"
    );

    let json = fixture.ess(&[
        "specify",
        "http",
        "compile",
        "--path",
        "model",
        "--binding",
        "http.yaml",
        "--out",
        "http.json",
        "--format",
        "json",
    ]);
    assert_eq!(json.status.code(), Some(1), "{}", text(&json));
    let diagnostics: serde_json::Value = serde_json::from_slice(&json.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", text(&json)));
    let found: Vec<(&str, &str)> = diagnostics
        .as_array()
        .expect("a list of diagnostics")
        .iter()
        .map(|d| (d["code"].as_str().unwrap(), d["path"].as_str().unwrap()))
        .collect();
    assert_eq!(
        found,
        [
            ("ESS-HTTP-006", "routes[0].method"),
            ("ESS-HTTP-013", "routes[1].command")
        ]
    );
    assert!(!fixture.0.join("http.json").exists(), "a refusal wrote IR");
}

#[test]
fn a_stage_2_key_is_refused_by_name() {
    let fixture = Fixture::new();
    fixture.binding(
        "routes:
  - command: catalog.items.ListItem
    method: post
    request: {encoding: form}
",
    );
    let validate = fixture.ess(&[
        "specify",
        "http",
        "validate",
        "--path",
        "model",
        "--binding",
        "http.yaml",
    ]);
    assert!(!validate.status.success(), "{}", text(&validate));
    assert!(
        text(&validate).contains("unknown field `request`"),
        "{}",
        text(&validate)
    );
}

#[test]
fn a_route_on_another_operations_derived_path_is_refused_as_a_collision() {
    let fixture = Fixture::new();
    // The binding does not name `ListItem`, so it keeps the route `ess_gen::http::routes` derives
    // for it, `post /items/commands/ListItem`, which the binding gives to `TouchItem` as well. The
    // view's `get /items/{item_id}` is another method and shape, and claims nothing twice.
    fixture.binding(
        "routes:
  - {command: catalog.items.TouchItem, method: post, path: /items/commands/ListItem}
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
",
    );
    let validate = fixture.ess(&[
        "specify",
        "http",
        "validate",
        "--path",
        "model",
        "--binding",
        "http.yaml",
    ]);
    assert_eq!(validate.status.code(), Some(1), "{}", text(&validate));
    let shown = text(&validate);
    assert!(
        shown.contains(
            "error[ESS-HTTP-010]: routes[0]: two operations claim the same method and path:"
        ),
        "{shown}"
    );
    assert!(
        shown.contains("command `catalog.items.TouchItem`")
            && shown.contains("command `catalog.items.ListItem`"),
        "{shown}"
    );
    assert!(!shown.contains("ESS-HTTP-011"), "{shown}");
    assert_eq!(shown.matches("error[").count(), 1, "{shown}");
}

#[test]
fn a_server_binding_refines_a_protocol_binding_given_with_refines() {
    let fixture = Fixture::new();
    // The protocol binding: the view's path is fixed, the command's is left to each server.
    fixture.binding(ROUTES);
    fs::rename(fixture.0.join("http.yaml"), fixture.0.join("protocol.yaml")).unwrap();
    let protocol = fs::read(fixture.0.join("protocol.yaml")).unwrap();
    let pin = ess_http::digest(&protocol);
    let server = |routes: &str, refines: &str| {
        fixture.binding(&format!("{refines}{routes}"));
    };
    let filled = ROUTES.replace(
        "    method: post\n",
        "    method: post\n    path: /api/items\n",
    );
    assert_ne!(filled, ROUTES);
    server(&filled, &format!("refines: {pin}\n"));
    let validate = |extra: &[&str]| {
        let mut args = vec![
            "specify",
            "http",
            "validate",
            "--path",
            "model",
            "--binding",
            "http.yaml",
        ];
        args.extend_from_slice(extra);
        fixture.ess(&args)
    };
    let valid = validate(&["--refines", "protocol.yaml"]);
    assert!(valid.status.success(), "{}", text(&valid));
    assert_eq!(
        String::from_utf8_lossy(&valid.stdout),
        "http.yaml — 2 route(s), 0 open, valid\n"
    );

    let compile = fixture.ess(&[
        "specify",
        "http",
        "compile",
        "--path",
        "model",
        "--binding",
        "http.yaml",
        "--refines",
        "protocol.yaml",
        "--format",
        "json",
    ]);
    assert!(compile.status.success(), "{}", text(&compile));
    let ir: serde_json::Value = serde_json::from_slice(&compile.stdout).unwrap();
    assert_eq!(ir["refines"], serde_json::json!(pin));

    // `refines:` with no refined file supplied.
    let missing = validate(&[]);
    assert_eq!(missing.status.code(), Some(1), "{}", text(&missing));
    assert!(
        text(&missing).contains("error[ESS-HTTP-003]: refines:"),
        "{}",
        text(&missing)
    );

    // The refined file reformatted: its bytes, and so its digest, differ from the pin.
    let reformatted = String::from_utf8(protocol.clone())
        .unwrap()
        .replace("version: v1", "version: 'v1'");
    fs::write(fixture.0.join("protocol.yaml"), reformatted).unwrap();
    let moved = validate(&["--refines", "protocol.yaml"]);
    assert_eq!(moved.status.code(), Some(1), "{}", text(&moved));
    assert!(
        text(&moved).contains("error[ESS-HTTP-003]: refines:") && text(&moved).contains(&pin),
        "{}",
        text(&moved)
    );
    fs::write(fixture.0.join("protocol.yaml"), &protocol).unwrap();

    // A refinement that moves the fixed view path.
    server(
        &filled.replace("path: /items/{item_id}", "path: /things/{item_id}"),
        &format!("refines: {pin}\n"),
    );
    let changed = validate(&["--refines", "protocol.yaml", "--format", "json"]);
    assert_eq!(changed.status.code(), Some(1), "{}", text(&changed));
    let diagnostics: serde_json::Value = serde_json::from_slice(&changed.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", text(&changed)));
    assert_eq!(diagnostics[0]["code"], "ESS-HTTP-012", "{diagnostics}");
    assert_eq!(diagnostics[0]["path"], "routes[0].path", "{diagnostics}");

    // `--refines` given to a document that refines nothing.
    server(&filled, "");
    let unpinned = validate(&["--refines", "protocol.yaml"]);
    assert_eq!(unpinned.status.code(), Some(1), "{}", text(&unpinned));
    assert!(
        text(&unpinned).contains("error[ESS-HTTP-003]: refines:")
            && text(&unpinned).contains("refines nothing"),
        "{}",
        text(&unpinned)
    );
}
