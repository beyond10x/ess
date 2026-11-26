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

/// The path or `request.query` carries every input field but the bare body (beyond10x/ess#754,
/// `story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body`).
#[test]
fn a_bare_body_beside_an_input_the_query_carries_compiles() {
    let ir = model();
    let routes = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/priority
    request: {body: input.priorities, query: [input.board_id]}
";
    let http = compiled(&ir, routes).expect("compiles");
    let json = http.to_canonical_json();
    assert!(
        json.contains(
            "      \"request\": {\n        \"body\": \"input.priorities\",\n        \"query\": [\n          \"input.board_id\"\n        ]\n      }\n"
        ),
        "{json}"
    );
    let read = HttpIr::from_json(&json).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), json, "round trip changed bytes");
    // Listing the body itself in the query leaves it in two places.
    let twice = routes.replace(
        "query: [input.board_id]",
        "query: [input.board_id, input.priorities]",
    );
    assert_eq!(
        located(&compiled(&ir, &twice)),
        [("ESS-HTTP-020", "routes[0].request.query".to_owned())]
    );
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

// --- `request.members` (`ae1d-a-json-request-reads-an-input-from-a-nested-member`) ---

/// An envelope: `board_id` and `title` under `data`, `note` a top-level member as before. The map
/// is written out of field order; compiled IR writes it in field order.
const ENVELOPE: &str = "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /hooks
    request: {encoding: json, members: {title: data.title, board_id: data.board_id}}
";

#[test]
fn a_json_request_reads_an_input_from_a_nested_member() {
    let ir = model();
    let http = compiled(&ir, ENVELOPE).expect("compiles");
    let json = http.to_canonical_json();
    assert!(
        json.contains(
            "      \"request\": {\n        \"members\": {\n          \"board_id\": \"data.board_id\",\n          \"title\": \"data.title\"\n        }\n      }\n"
        ),
        "{json}"
    );
    let route = http
        .route(&Operation::Command("catalog.board.RenameBoard".into()))
        .expect("routed");
    let request = route.request.as_ref().expect("a request");
    let read: Vec<(&str, String)> = request
        .members
        .iter()
        .map(|(field, member)| (field, member.to_string()))
        .collect();
    assert_eq!(
        read,
        [
            ("board_id", "data.board_id".to_owned()),
            ("title", "data.title".to_owned())
        ]
    );
    assert_eq!(
        request
            .member("title")
            .map(|member| member.segments().to_vec()),
        Some(vec!["data".to_owned(), "title".to_owned()])
    );
    assert_eq!(request.member("note"), None);
    let back = HttpIr::from_json(&json).expect("the compiled IR reads back");
    assert_eq!(back.to_canonical_json(), json, "round trip changed bytes");
    // A deeper path, and `unknown: ignore` beside it.
    let deeper = ENVELOPE
        .replace("data.title", "event.data.title")
        .replace("encoding: json,", "unknown: ignore,");
    assert_eq!(located(&compiled(&ir, &deeper)), []);
}

fn members_refused(index: usize, field: &str) -> (&'static str, String) {
    (
        "ESS-HTTP-020",
        format!("routes[{index}].request.members.{field}"),
    )
}

#[test]
fn a_member_naming_no_input_field_or_one_carried_elsewhere_is_refused() {
    let ir = model();
    for (request, field, says) in [
        (
            "{members: {nobody: data.nobody}}",
            "nobody",
            "no input field",
        ),
        (
            "{members: {note: data.note}, query: [input.note]}",
            "note",
            "query",
        ),
        (
            "{encoding: form, members: {note: data.note}}",
            "note",
            "`form`",
        ),
        (
            "{encoding: multipart, members: {note: data.note}}",
            "note",
            "`multipart`",
        ),
        (
            "{body: input.title, members: {note: data.note}}",
            "note",
            "body",
        ),
    ] {
        let routes = format!(
            "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /boards/{{board_id}}/rename
    parameters: {{board_id: input.board_id}}
    request: {request}
"
        );
        let result = compiled(&ir, &routes);
        let found = located(&result);
        assert!(
            found.contains(&members_refused(0, field)),
            "{request}: {found:?}"
        );
        let shown = worded(&result);
        assert!(
            shown.iter().any(|line| line.contains(says)),
            "{request}: {shown:?}"
        );
    }
    // The path carries `board_id`.
    let routes = "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /boards/{board_id}/rename
    parameters: {board_id: input.board_id}
    request: {members: {board_id: data.board_id}}
";
    assert_eq!(
        located(&compiled(&ir, routes)),
        [members_refused(0, "board_id")]
    );
    // `query` on `delete`, and any body on `get`.
    let routes = "routes:
  - command: catalog.board.RenameBoard
    method: delete
    path: /boards/{board_id}/title
    parameters: {board_id: input.board_id}
    request: {encoding: query, members: {title: data.title}}
  - command: catalog.board.UpdateCard
    method: get
    path: /boards/{board_id}/cards/{card_id}
    parameters: {board_id: input.board_id, card_id: input.card_id}
    request: {members: {title: data.title}}
";
    let found = located(&compiled(&ir, routes));
    assert!(found.contains(&members_refused(0, "title")), "{found:?}");
    assert!(found.contains(&members_refused(1, "title")), "{found:?}");
    // A view takes no request at all.
    let routes = "routes:
  - view: catalog.board.BoardById
    method: get
    path: /boards/{board_id}
    parameters: {board_id: param.board_id}
    request: {members: {board_id: data.board_id}}
";
    assert_eq!(
        located(&compiled(&ir, routes)),
        [members_refused(0, "board_id")]
    );
}

#[test]
fn a_member_path_breaking_the_grammar_or_meeting_another_is_refused() {
    let ir = model();
    for (members, field, says) in [
        ("{title: data..title}", "title", "empty"),
        ("{title: .title}", "title", "empty"),
        ("{title: data.}", "title", "empty"),
        ("{title: ''}", "title", "empty"),
        // Two fields on one member, in field order the second refused.
        ("{note: data.x, title: data.x}", "note", "`title`"),
        // One path a prefix of another.
        ("{title: data, note: data.note}", "note", "`title`"),
        (
            "{title: data.title.text, note: data.title}",
            "note",
            "`title`",
        ),
        // A top-level member that is an unmapped field's wire name, or the object above one.
        ("{note: title}", "note", "`title`"),
        ("{note: title.note}", "note", "`title`"),
    ] {
        let routes = format!(
            "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /hooks
    request: {{members: {members}}}
"
        );
        let result = compiled(&ir, &routes);
        assert_eq!(located(&result), [members_refused(0, field)], "{members}");
        let shown = worded(&result);
        assert!(shown[0].contains(says), "{members}: {shown:?}");
    }
    // Written empty.
    let routes = "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /hooks
    request: {members: {}}
";
    assert_eq!(
        located(&compiled(&ir, routes)),
        [("ESS-HTTP-020", "routes[0].request.members".to_owned())]
    );
}

#[test]
fn compiled_members_hold_to_their_rules_when_read_back() {
    let ir = model();
    let json = compiled(&ir, ENVELOPE)
        .expect("compiles")
        .to_canonical_json();
    let members = "        \"members\": {\n          \"board_id\": \"data.board_id\",\n          \"title\": \"data.title\"\n        }\n";
    assert!(json.contains(members), "{json}");
    for (broken, why) in [
        (
            "        \"members\": {\n          \"board_id\": \"data..board_id\",\n          \"title\": \"data.title\"\n        }\n",
            "an empty segment",
        ),
        (
            "        \"members\": {\n          \"board_id\": \"data\",\n          \"title\": \"data.title\"\n        }\n",
            "a prefix",
        ),
        (
            "        \"members\": {\n          \"board_id\": \"data.title\",\n          \"title\": \"data.title\"\n        }\n",
            "one path twice",
        ),
        ("        \"members\": {}\n", "written empty"),
        (
            "        \"encoding\": \"form\",\n        \"members\": {\n          \"board_id\": \"data.board_id\",\n          \"title\": \"data.title\"\n        }\n",
            "beside `form`",
        ),
    ] {
        let edited = json.replace(members, broken);
        assert_ne!(edited, json, "{why}");
        assert!(HttpIr::from_json(&edited).is_err(), "{why}: {edited}");
    }
}
