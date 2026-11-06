//! An `ess-http/1` command route may take one input field as its whole request body, bare:
//! `request: {encoding: json, body: input.<field>}`, every other input field carried by the path
//! (`story:a-command-route-takes-one-input-as-its-bare-request-body`, beyond10x/ess#748 part 1;
//! `docs/design/http-presentation-binding.md`, "Request encoding").

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const MODEL: &str = include_str!("fixtures/board.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("board.yaml", MODEL);
    compile_model(&spec, &sources).expect("compiles")
}

/// The routes ESS derives for the board, written out as `ess_gen::http::routes` derives them.
fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/board/commands/{}", handle.name().local()),
            });
        }
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                table.push(DerivedRoute {
                    component: component.name.to_string(),
                    operation: Operation::View(handle.name().to_string()),
                    method: Method::Get,
                    path: format!("/board/views/{}", handle.name().local()),
                });
            }
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method).cmp(&(
            &right.component,
            &right.path,
            right.method,
        ))
    });
    table
}

fn compiled(ir: &EssIr, routes: &str) -> Result<HttpIr, HttpDiagnostics> {
    let header = format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&(header + routes)).expect("parses");
    compile(&spec, ir, &derived(ir))
}

/// Each refusal's code and path, or none when the binding compiles.
fn located(result: &Result<HttpIr, HttpDiagnostics>) -> Vec<(&'static str, String)> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.clone()))
            .collect(),
    }
}

fn worded(result: &Result<HttpIr, HttpDiagnostics>) -> Vec<String> {
    result
        .as_ref()
        .expect_err("refused")
        .0
        .iter()
        .map(ToString::to_string)
        .collect()
}

fn body_refused(index: usize) -> (&'static str, String) {
    ("ESS-HTTP-020", format!("routes[{index}].request.body"))
}

/// The adopter's reproduction: the list of card priorities is the whole body, the board in the
/// path.
const PRIORITIES: &str = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {encoding: json, body: input.priorities}
";

#[test]
fn a_command_route_takes_one_list_input_as_its_bare_body() {
    let ir = model();
    let http = compiled(&ir, PRIORITIES).expect("compiles");
    let route = http
        .route(&Operation::Command(
            "catalog.board.SetCardPriorities".into(),
        ))
        .expect("routed");
    assert!(route.request.is_some(), "the route records its body");
    let json = http.to_canonical_json();
    // `encoding: json` is the default and omitted; `body` is written as the binding spells it.
    assert!(
        json.contains("      \"request\": {\n        \"body\": \"input.priorities\"\n      }\n"),
        "{json}"
    );
    let read = HttpIr::from_json(&json).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), json, "round trip changed bytes");
    // Without `encoding`, the body is `json` all the same, and compiles to the same bytes.
    let unwritten = PRIORITIES.replace("{encoding: json, body:", "{body:");
    assert_ne!(unwritten, PRIORITIES);
    assert_eq!(
        compiled(&ir, &unwritten)
            .expect("compiles")
            .to_canonical_json(),
        json
    );
    // A scalar input is a bare body too: any one input field that is not `Optional`.
    let title = "routes:
  - command: catalog.board.UpdateCard
    method: patch
    path: /boards/{board_id}/cards/{card_id}/title
    parameters: {board_id: input.board_id, card_id: input.card_id}
    request: {body: input.title}
";
    assert_eq!(located(&compiled(&ir, title)), []);
}

#[test]
fn a_body_beside_an_input_the_path_does_not_carry_is_refused() {
    let ir = model();
    let routes = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/priority
    request: {body: input.priorities}
";
    let result = compiled(&ir, routes);
    assert_eq!(located(&result), [body_refused(0)]);
    let shown = worded(&result);
    assert!(
        shown[0].contains("`board_id`") && shown[0].contains("path"),
        "{shown:?}"
    );
    // The body naming a field the path already carries leaves the other field in neither.
    let routes = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {body: input.board_id}
";
    let result = compiled(&ir, routes);
    assert_eq!(located(&result), [body_refused(0)]);
    let shown = worded(&result);
    assert!(shown[0].contains("`board_id`"), "{shown:?}");
}

#[test]
fn a_body_on_a_view_route_or_beside_form_or_query_is_refused() {
    let ir = model();
    let routes = "routes:
  - view: catalog.board.BoardById
    method: get
    path: /boards/{board_id}
    parameters: {board_id: param.board_id}
    request: {body: param.board_id}
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {encoding: form, body: input.priorities}
  - command: catalog.board.RenameBoard
    method: delete
    path: /boards/{board_id}/title
    parameters: {board_id: input.board_id}
    request: {encoding: query, body: input.title}
  - command: catalog.board.UpdateCard
    method: get
    path: /boards/{board_id}/cards/{card_id}
    parameters: {board_id: input.board_id, card_id: input.card_id}
    request: {body: input.title}
";
    let result = compiled(&ir, routes);
    assert_eq!(
        located(&result),
        [
            body_refused(0),
            body_refused(1),
            body_refused(2),
            body_refused(3)
        ]
    );
    let shown = worded(&result);
    assert!(shown[0].contains("view"), "{shown:?}");
    assert!(shown[1].contains("`form`"), "{shown:?}");
    assert!(shown[2].contains("`query`"), "{shown:?}");
    assert!(shown[3].contains("`get`"), "{shown:?}");
}

#[test]
fn a_body_naming_no_input_or_an_optional_one_is_refused() {
    let ir = model();
    for (body, says) in [
        ("input.nothing", "`nothing`"),
        ("param.board_id", "`input.<field>`"),
        ("priorities", "`input.<field>`"),
        ("input.note", "Optional"),
    ] {
        let routes = format!(
            "routes:
  - command: catalog.board.RenameBoard
    method: put
    path: /boards/{{board_id}}/title
    parameters: {{board_id: input.board_id}}
    request: {{body: {body}}}
"
        );
        let result = compiled(&ir, &routes);
        // `title` stays outside the path and the body in each, which is refused too, at the same
        // key.
        assert!(
            located(&result)
                .iter()
                .all(|found| *found == body_refused(0))
                && result.is_err(),
            "{body}: {:?}",
            located(&result)
        );
        let shown = worded(&result);
        assert!(shown[0].contains(says), "{body}: {shown:?}");
    }
}

#[test]
fn a_binding_without_body_compiles_to_the_same_bytes() {
    let ir = model();
    let routes = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
  - command: catalog.board.UpdateCard
    method: post
    path: /cards
    request: {encoding: form, unknown: ignore}
";
    let json = compiled(&ir, routes).expect("compiles").to_canonical_json();
    // The request of a route without `body` is what 0.88.0 wrote: no `body` key, and none at all
    // where the request is all defaults.
    assert!(!json.contains("\"body\""), "{json}");
    assert!(
        json.contains(
            "      \"path\": \"/cards\",\n      \"request\": {\n        \"encoding\": \"form\",\n        \"unknown\": \"ignore\"\n      }\n"
        ),
        "{json}"
    );
    assert!(
        json.contains(
            "      \"parameters\": {\n        \"board_id\": \"input.board_id\"\n      }\n    }"
        ),
        "{json}"
    );
    // A compiled request names a command's input as its body, never a view parameter.
    let foreign = json.replace(
        "        \"encoding\": \"form\",\n",
        "        \"body\": \"param.board_id\",\n        \"encoding\": \"form\",\n",
    );
    assert_ne!(foreign, json);
    assert!(HttpIr::from_json(&foreign).is_err(), "{foreign}");
}
