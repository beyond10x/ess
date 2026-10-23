//! An outcome may answer one field of the command's response, bare (`body: response.<field>`),
//! and a route may give every error outcome it does not list one answer (`errors:`)
//! (`docs/design/http-presentation-binding.md`, "Response body per outcome" and "Status per
//! outcome"). Both are additive within `ess-http/1` and `ess-http-ir/1`: a document that uses
//! neither compiles to the bytes it compiled to before.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, Body, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method,
    Operation, Refined,
};
use serde_json::{json, Value};

const MODEL: &str = include_str!("fixtures/tasks.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("tasks.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("tasks.yaml", MODEL);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The routes ESS derives for `ir`, as `tests/outcomes.rs` writes them.
fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/tasks/commands/{}", handle.name().local()),
            });
        }
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                table.push(DerivedRoute {
                    component: component.name.to_string(),
                    operation: Operation::View(handle.name().to_string()),
                    method: Method::Get,
                    path: format!("/tasks/views/{}", handle.name().local()),
                });
            }
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method, &left.operation).cmp(&(
            &right.component,
            &right.path,
            right.method,
            &right.operation,
        ))
    });
    table
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

fn compiled(ir: &EssIr, document: &str) -> Result<HttpIr, HttpDiagnostics> {
    let spec = HttpSpec::from_yaml(&(header(ir) + document))
        .unwrap_or_else(|error| panic!("parses: {error}\n{document}"));
    compile(&spec, ir, &derived(ir))
}

fn located(result: Result<HttpIr, HttpDiagnostics>) -> Vec<(&'static str, String)> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .into_iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path))
            .collect(),
    }
}

fn worded(result: Result<HttpIr, HttpDiagnostics>) -> Vec<String> {
    result
        .expect_err("refused")
        .0
        .into_iter()
        .map(|diagnostic| diagnostic.to_string())
        .collect()
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

fn ir_json(http: &HttpIr) -> Value {
    serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON")
}

/// A `SendTask` route on `post /api/tasks` with `keys` beside its path.
fn send_task(keys: &str) -> String {
    format!(
        "routes:\n  - command: catalog.tasks.SendTask\n    method: post\n    path: /api/tasks\n{keys}"
    )
}

/// The only route of `http`, as JSON.
fn only_route(http: &HttpIr) -> Value {
    let json = ir_json(http);
    let routes = json["routes"].as_array().expect("routes");
    assert_eq!(routes.len(), 1, "{json:#}");
    routes[0].clone()
}

/// Compiles, writes, reads back and writes again: the bytes do not move.
fn round_trips(http: &HttpIr) {
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text)
        .unwrap_or_else(|error| panic!("the compiled IR reads back: {error}\n{text}"));
    assert_eq!(read.to_canonical_json(), text, "round trip changed bytes");
}

// --- need (a): `body: response.<field>` --------------------------------------------------------

#[test]
fn an_outcome_answers_one_response_field_bare() {
    let ir = model();
    let http = compiled(
        &ir,
        &send_task("    outcomes:\n      sent: {status: 200, body: response.task_id}\n"),
    )
    .expect("compiles");
    assert_eq!(
        only_route(&http)["outcomes"],
        json!({"sent": {"status": 200, "body": "response.task_id"}})
    );
    round_trips(&http);
    // Any top-level field, under another wire name too: the IR names the field, and a server
    // writes its value; the wire name is the command's to say.
    let http = compiled(
        &ir,
        &send_task(
            "    outcomes:\n      sent:\n        status: 200\n        body: response.version\n        headers: {ETag: {source: response.task_id}}\n",
        ),
    )
    .expect("compiles");
    assert_eq!(
        only_route(&http)["outcomes"]["sent"]["body"],
        json!("response.version")
    );
    let typed = http.routes()[0].outcomes["sent"].body.clone();
    assert_eq!(typed, Body::ResponseField("version".to_owned()));
    assert_eq!(typed.to_string(), "response.version");
    round_trips(&http);
}

#[test]
fn ess_http_024_a_response_field_body_the_outcome_cannot_answer() {
    let ir = model();
    for (outcome, body, why) in [
        ("sent", "response.note", "Optional"),
        ("sent", "response.missing", "no response field `missing`"),
        ("sent", "response.", "not a body"),
        ("sent", "response.task_id.value", "not a body"),
        ("too-large", "response.task_id", "reports error"),
        ("sent", "input.task_id", "`input.task_id`"),
        ("too-large", "error.limit", "`error.limit`"),
        ("too-large", "error.wire", "`error.wire`"),
    ] {
        let status = if outcome == "sent" { 200 } else { 400 };
        let document = send_task(&format!(
            "    outcomes:\n      {outcome}: {{status: {status}, body: \"{body}\"}}\n"
        ));
        let result = compiled(&ir, &document);
        let words = match &result {
            Ok(_) => panic!("`{body}` on `{outcome}` compiles"),
            Err(diagnostics) => diagnostics.to_string(),
        };
        assert_eq!(
            located(result),
            [refused(
                "ESS-HTTP-024",
                &format!("routes[0].outcomes.{outcome}.body")
            )],
            "`{body}` on `{outcome}`: {words}"
        );
        assert!(words.contains(why), "`{body}`: {words}");
    }
    // On an outcome without `returns: true`.
    let document = "routes:\n  - command: catalog.tasks.NoteTask\n    method: post\n    path: /api/notes\n    outcomes:\n      noted: {status: 200, body: response.task_id}\n";
    let words = worded(compiled(&ir, document));
    assert!(
        words.iter().any(|word| word.contains("ESS-HTTP-024")
            && word.contains("routes[0].outcomes.noted.body")
            && word.contains("returns: true")),
        "{words:?}"
    );
    // A response field body on `204` or beside a redirect is no `none`.
    let words = worded(compiled(
        &ir,
        &send_task("    outcomes:\n      sent: {status: 204, body: response.task_id}\n"),
    ));
    assert!(
        words.iter().any(|word| word.contains("ESS-HTTP-024")
            && word.contains("`204`")
            && word.contains("response.task_id")),
        "{words:?}"
    );
}

// --- need (b): `errors:` -----------------------------------------------------------------------

#[test]
fn errors_answers_every_unlisted_error_outcome() {
    let ir = model();
    let http = compiled(
        &ir,
        &send_task(
            "    outcomes:\n      sent: {status: 200, body: response.task_id}\n    errors: {status: 400, body: {error: error}}\n",
        ),
    )
    .expect("compiles");
    let route = only_route(&http);
    assert_eq!(
        route["outcomes"],
        json!({"sent": {"status": 200, "body": "response.task_id"}})
    );
    assert_eq!(
        route["errors"],
        json!({"status": 400, "body": {"error": "error"}})
    );
    let errors = http.routes()[0].errors.as_ref().expect("typed errors");
    assert_eq!(errors.status, 400);
    round_trips(&http);
    // Without `outcomes:` at all, and with headers.
    let http = compiled(
        &ir,
        &send_task(
            "    errors:\n      status: 400\n      headers: {X-Error: {source: error.wire}}\n",
        ),
    )
    .expect("compiles");
    let route = only_route(&http);
    assert_eq!(route.get("outcomes"), None);
    assert_eq!(
        route["errors"],
        json!({"status": 400, "headers": {"X-Error": {"source": "error.wire"}}})
    );
    round_trips(&http);
}

#[test]
fn errors_replaces_the_upstream_status_of_an_external_refusal() {
    // `unsupported` is an `external:` refusal, derived `502`; listed under `errors`, it answers
    // `400`, as listing it by name does.
    let ir = model();
    let named = compiled(
        &ir,
        &send_task(
            "    outcomes:\n      unsupported: {status: 400, body: {error: error}}\n      too-large: {status: 400, body: {error: error}}\n",
        ),
    )
    .expect("compiles");
    let shared = compiled(
        &ir,
        &send_task("    errors: {status: 400, body: {error: error}}\n"),
    )
    .expect("compiles");
    assert_eq!(
        only_route(&named)["outcomes"]["unsupported"],
        only_route(&shared)["errors"]
    );
    // One listed by name keeps its own answer; `errors` covers the other.
    let http = compiled(
        &ir,
        &send_task(
            "    outcomes:\n      too-large: {status: 413}\n    errors: {status: 400, body: {error: error}}\n",
        ),
    )
    .expect("compiles");
    let route = only_route(&http);
    assert_eq!(route["outcomes"], json!({"too-large": {"status": 413}}));
    assert_eq!(
        route["errors"],
        json!({"status": 400, "body": {"error": "error"}})
    );
    let typed = &http.routes()[0];
    assert_eq!(
        typed
            .answer("unsupported", true)
            .map(|outcome| outcome.status),
        Some(400)
    );
    assert_eq!(
        typed
            .answer("too-large", true)
            .map(|outcome| outcome.status),
        Some(413)
    );
    assert_eq!(typed.answer("sent", false), None);
}

#[test]
fn errors_is_held_to_every_check_a_listed_outcome_is() {
    let ir = model();
    // A `2xx` on an error outcome, once for each outcome it covers.
    assert_eq!(
        located(compiled(&ir, &send_task("    errors: {status: 200}\n"))),
        [
            refused("ESS-HTTP-023", "routes[0].errors.status"),
            refused("ESS-HTTP-023", "routes[0].errors.status"),
        ]
    );
    // `response` and a response field: an error answers no response.
    for body in ["response", "response.task_id"] {
        let words = worded(compiled(
            &ir,
            &send_task(&format!("    errors: {{status: 400, body: {body}}}\n")),
        ));
        assert!(
            words
                .iter()
                .all(|word| word.contains("ESS-HTTP-024") && word.contains("routes[0].errors.body")),
            "{body}: {words:?}"
        );
        assert!(
            words.iter().any(|word| word.contains("ESS-HTTP-024")),
            "{body}"
        );
    }
    // An error object under a member a covered error already wires a field to.
    let label = "routes:\n  - command: catalog.tasks.LabelTask\n    method: post\n    path: /api/labels\n    errors: {status: 400, body: {error: error}}\n";
    assert_eq!(
        located(compiled(&ir, label)),
        [refused("ESS-HTTP-024", "routes[0].errors.body")]
    );
    // Two errors `body: none` cannot be told apart on one status.
    let words = worded(compiled(
        &ir,
        &send_task("    errors: {status: 400, body: none}\n"),
    ));
    assert!(
        words.iter().any(|word| word.contains("ESS-HTTP-025")
            && word.contains("routes[0].errors")
            && word.contains("unsupported")
            && word.contains("too-large")),
        "{words:?}"
    );
    // Nor an error under `errors` from a listed one on its status with the same body.
    let words = worded(compiled(
        &ir,
        &send_task(
            "    outcomes:\n      too-large: {status: 400, body: none}\n    errors: {status: 400, body: none}\n",
        ),
    ));
    assert!(
        words
            .iter()
            .any(|word| word.contains("ESS-HTTP-025") && word.contains("unsupported")),
        "{words:?}"
    );
    // A header source the covered outcome cannot supply.
    let words = worded(compiled(
        &ir,
        &send_task(
            "    errors:\n      status: 400\n      headers: {X-Limit: {source: error.limit}}\n",
        ),
    ));
    assert!(
        words.iter().any(|word| word.contains("ESS-HTTP-026")
            && word.contains("routes[0].errors.headers.X-Limit")
            && word.contains("unsupported")),
        "{words:?}"
    );
}

#[test]
fn ess_http_022_errors_that_cover_nothing_or_sit_on_a_view() {
    let ir = model();
    // Every error outcome is listed by name.
    assert_eq!(
        located(compiled(
            &ir,
            &send_task(
                "    outcomes:\n      unsupported: {status: 400}\n      too-large: {status: 400}\n    errors: {status: 400}\n",
            ),
        )),
        [refused("ESS-HTTP-022", "routes[0].errors")]
    );
    // A command that reports no error.
    let note = "routes:\n  - command: catalog.tasks.NoteTask\n    method: post\n    path: /api/notes\n    errors: {status: 400}\n";
    assert_eq!(
        located(compiled(&ir, note)),
        [refused("ESS-HTTP-022", "routes[0].errors")]
    );
    // A view answers no outcome.
    let view = "routes:\n  - view: catalog.tasks.Tasks\n    method: get\n    path: /api/tasks\n    errors: {status: 400}\n";
    assert_eq!(
        located(compiled(&ir, view)),
        [refused("ESS-HTTP-022", "routes[0].errors")]
    );
}

#[test]
fn ess_http_012_a_refinement_restates_errors() {
    let ir = model();
    let base = send_task("    errors: {status: 400, body: {error: error}}\n");
    let refined = Refined::from_yaml((header(&ir) + &base).as_bytes()).expect("parses");
    let refining = |document: &str| {
        let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + document;
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
    };
    assert_eq!(located(refining(&base)), []);
    for changed in [
        send_task("    errors: {status: 409, body: {error: error}}\n"),
        send_task("    errors: {status: 400}\n"),
        send_task(""),
    ] {
        assert_eq!(
            located(refining(&changed)),
            [refused("ESS-HTTP-012", "routes[0].errors")],
            "{changed}"
        );
    }
    // Added where the refined binding had none.
    let without = send_task("");
    let refined = Refined::from_yaml((header(&ir) + &without).as_bytes()).expect("parses");
    let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + &base;
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    assert_eq!(
        located(compile_refining(&spec, Some(&refined), &ir, &derived(&ir))),
        [refused("ESS-HTTP-012", "routes[0].errors")]
    );
}

// --- the format: additive ----------------------------------------------------------------------

/// A document written before either form compiles to exactly these bytes, which carry neither.
#[test]
fn a_document_without_either_form_keeps_its_bytes() {
    let ir = model();
    let http = compiled(
        &ir,
        &send_task(
            "    outcomes:\n      sent: {status: 201, body: response}\n      too-large: {status: 400, body: {error: error}}\n",
        ),
    )
    .expect("compiles");
    let text = http.to_canonical_json();
    // The route, byte for byte as `ess-http-ir/1` wrote it before either form: no `errors` key.
    let route = "  \"routes\": [\n    {\n      \"command\": \"catalog.tasks.SendTask\",\n      \"method\": \"post\",\n      \"path\": \"/api/tasks\",\n      \"outcomes\": {\n        \"sent\": {\n          \"status\": 201,\n          \"body\": \"response\"\n        },\n        \"too-large\": {\n          \"status\": 400,\n          \"body\": {\n            \"error\": \"error\"\n          }\n        }\n      }\n    }\n  ],\n";
    assert!(text.contains(route), "{text}");
    assert!(!text.contains("errors"), "{text}");
    assert_eq!(http.routes()[0].errors, None);
    round_trips(&http);
    // IR written before either form reads back as these bytes: a reader of the new format reads
    // the old one unchanged.
    let read = HttpIr::from_json(&text).expect("old IR reads");
    assert_eq!(read.to_canonical_json(), text);
    assert_eq!(read, http);
}

#[test]
fn compiled_ir_holds_errors_and_response_fields_to_the_format() {
    let ir = model();
    let http = compiled(
        &ir,
        &send_task(
            "    outcomes:\n      sent: {status: 200, body: response.task_id}\n    errors: {status: 400, body: {error: error}}\n",
        ),
    )
    .expect("compiles");
    let text = http.to_canonical_json();
    let errors = "\"errors\": {\n        \"status\": 400,\n        \"body\": {\n          \"error\": \"error\"\n        }\n      }";
    assert!(text.contains(errors), "{text}");
    for (written, named) in [
        (
            "\"errors\": {\"status\": 400, \"body\": \"envelope\"}",
            "envelope",
        ),
        (
            "\"errors\": {\"status\": 400, \"body\": \"response\"}",
            "response",
        ),
        (
            "\"errors\": {\"status\": 400, \"body\": \"response.task_id\"}",
            "response",
        ),
        ("\"errors\": {\"status\": 200}", "200"),
        ("\"errors\": {\"status\": 400, \"code\": 1}", "code"),
    ] {
        let broken = text.replace(errors, written);
        assert_ne!(broken, text);
        let error = HttpIr::from_json(&broken).expect_err(written).to_string();
        assert!(error.contains(named), "{written}: {error}");
    }
    // A field body spelled with no field, or a source that is not the response.
    let field = "\"body\": \"response.task_id\"";
    assert!(text.contains(field), "{text}");
    for written in [
        "\"body\": \"response.\"",
        "\"body\": \"input.task_id\"",
        "\"body\": \"response.a.b\"",
    ] {
        let broken = text.replace(field, written);
        assert!(HttpIr::from_json(&broken).is_err(), "{written} reads");
    }
    // `errors` on a view route.
    let view = compiled(
        &ir,
        "routes:\n  - view: catalog.tasks.Tasks\n    method: get\n    path: /api/tasks\n",
    )
    .expect("compiles")
    .to_canonical_json();
    let path = "\"path\": \"/api/tasks\"";
    assert!(view.contains(path), "{view}");
    let broken = view.replace(
        path,
        "\"path\": \"/api/tasks\", \"errors\": {\"status\": 400}",
    );
    let error = HttpIr::from_json(&broken)
        .expect_err("errors on a view")
        .to_string();
    assert!(error.contains("errors"), "{error}");
}
