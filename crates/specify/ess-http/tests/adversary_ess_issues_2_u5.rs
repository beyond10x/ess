//! Adversarial cases against outcomes declared `alike`, a text body, a route's own `refusals:`, an
//! empty answer's literal body and an `array` view (`docs/design/http-presentation-binding.md`,
//! "Status per outcome", "Response body per outcome", "Precedence").
//!
//! A group of outcomes declared `alike` is admitted only when the wire cannot tell its members
//! apart: `envelope` bodies, error objects under distinct error wire names and redirects whose
//! parameters differ are refused (`ESS-HTTP-025` at `routes[i].alike`). The cases here hold the
//! same rule to the other places an answer names its outcome or its error.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};
use serde_json::{json, Value};

const NOTES: &str = include_str!("fixtures/notes.yaml");

const ORDERS: &str = include_str!("fixtures/orders.yaml");

fn model_of(file: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new(file), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert(file, text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The routes ESS derives for `ir`, as `tests/alike_and_text.rs` writes them.
fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/derived/commands/{}", handle.name().local()),
            });
        }
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                table.push(DerivedRoute {
                    component: component.name.to_string(),
                    operation: Operation::View(handle.name().to_string()),
                    method: Method::Get,
                    path: format!("/derived/views/{}", handle.name().local()),
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

fn compiled(ir: &EssIr, system: &str, document: &str) -> Result<HttpIr, HttpDiagnostics> {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: {system}\n  version: v1\n  source_digest: sha256:{}\n{document}",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    compile(&spec, ir, &derived(ir))
}

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

fn worded(result: &Result<HttpIr, HttpDiagnostics>) -> String {
    match result {
        Ok(http) => format!("compiled:\n{}", http.to_canonical_json()),
        Err(diagnostics) => diagnostics.to_string(),
    }
}

/// `CreateNote` of `fixtures/notes.yaml` on `post /api/notes`, `keys` beside its path.
fn create(keys: &str) -> String {
    format!(
        "routes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n{keys}"
    )
}

fn round_trips(http: &HttpIr) {
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text)
        .unwrap_or_else(|error| panic!("the compiled IR reads back: {error}\n{text}"));
    assert_eq!(read.to_canonical_json(), text, "round trip changed bytes");
}

fn ir_json(http: &HttpIr) -> Value {
    serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON")
}

// --- `alike`: the wire must not tell the members apart --------------------------------------------

/// `too-large` reports `TooLarge` (wire `too_large`), `negative` and `blank` report `Refused` (wire
/// `refused`). A header whose source is `error.wire` writes `too_large` for one and `refused` for
/// the others, so the wire tells them apart exactly as an error object under two wire names does,
/// and as a redirect parameter from `error.wire` does, both of which `alike` refuses.
#[test]
fn alike_members_an_error_wire_header_tells_apart_are_refused() {
    let ir = model_of("notes.yaml", NOTES);
    // The control: without the header, the group is one answer and compiles.
    let control = compiled(
        &ir,
        "demo",
        &create(
            "    errors: {status: 409, body: none}\n    alike: [[too-large, negative, blank]]\n",
        ),
    );
    assert_eq!(located(&control), [], "{}", worded(&control));
    for keys in [
        // Under `errors`, covering all three.
        "    errors:\n      status: 409\n      body: none\n      headers: {X-Error: {source: error.wire}}\n    alike: [[too-large, negative, blank]]\n",
        // Listed, each with the same declared header.
        "    outcomes:\n      too-large:\n        status: 409\n        body: none\n        headers: {X-Error: {source: error.wire}}\n      negative:\n        status: 409\n        body: none\n        headers: {X-Error: {source: error.wire}}\n    alike: [[too-large, negative]]\n",
    ] {
        let result = compiled(&ir, "demo", &create(keys));
        assert!(
            located(&result).contains(&("ESS-HTTP-025", "routes[0].alike".to_owned())),
            "an `error.wire` header tells `too-large` from `negative`, so the group is not one \
             answer the wire cannot tell apart:\n{keys}\n{}",
            worded(&result)
        );
    }
}

/// Two errors under one wire name whose own fields differ, in optionality, in name or in wire name:
/// `{error: error}` writes the wire name beside the error's fields, so the two bodies have
/// different members and the wire tells the outcomes apart.
#[test]
fn alike_error_objects_whose_errors_carry_different_fields_are_refused() {
    for (first, second) in [
        // Optionality.
        (
            "      - {name: detail, type: String}",
            "      - {name: detail, type: Optional<String>}",
        ),
        // Name.
        (
            "      - {name: detail, type: String}",
            "      - {name: reason, type: String}",
        ),
        // Wire name.
        (
            "      - {name: detail, type: String}",
            "      - {name: detail, type: String, naming: {wire: why}}",
        ),
    ] {
        let model = NOTES
            .replace(
                "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
                &format!(
                    "  - name: demo.notes.Refused\n    naming: {{wire: refused}}\n    fields:\n{first}\n  - name: demo.notes.Declined\n    naming: {{wire: refused}}\n    fields:\n{second}\n"
                ),
            )
            .replace(
                "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
                "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
            );
        assert_ne!(model, NOTES);
        let ir = model_of("notes.yaml", &model);
        let result = compiled(
            &ir,
            "demo",
            &create(
                "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n    alike: [[negative, blank]]\n",
            ),
        );
        assert!(
            located(&result).contains(&("ESS-HTTP-025", "routes[0].alike".to_owned())),
            "`{first}` against `{second}`: the error objects differ in their members:\n{}",
            worded(&result)
        );
    }
}

// --- `alike`: names -------------------------------------------------------------------------------

#[test]
fn alike_names_an_outcome_of_another_command_or_an_unknown_one_twice() {
    let ir = model_of("notes.yaml", NOTES);
    let none = "    outcomes:\n      fetched: {status: 204, body: none}\n      inline: {status: 204, body: none}\n";
    // `touched` is an outcome of `TouchNote`, not of `CreateNote`.
    let result = compiled(
        &ir,
        "demo",
        &create(&format!("{none}    alike: [[fetched, inline, touched]]\n")),
    );
    assert_eq!(
        located(&result),
        [("ESS-HTTP-022", "routes[0].alike".to_owned())],
        "{}",
        worded(&result)
    );
    // An unknown name in two groups is refused for being unknown, once per group, and not again
    // for being in two groups. The two known names answer on two statuses, so no pair clashes.
    let result = compiled(
        &ir,
        "demo",
        &create(
            "    outcomes:\n      fetched: {status: 204, body: none}\n      inline: {status: 201, body: response}\n    alike: [[nowhere, fetched], [nowhere, inline]]\n",
        ),
    );
    let located = located(&result);
    assert!(
        !located.is_empty()
            && located
                .iter()
                .all(|(code, path)| *code == "ESS-HTTP-022" && path == "routes[0].alike"),
        "{}",
        worded(&result)
    );
}

// --- an empty answer's literal body ---------------------------------------------------------------

#[test]
fn an_empty_literal_with_quotes_backslashes_and_unicode_keeps_its_characters() {
    let ir = model_of("orders.yaml", ORDERS);
    let text = "n\u{e9}\"\\\u{2028}\u{1f600} x";
    let document = format!(
        "routes:\n  - view: catalog.orders.OrderById\n    method: get\n    path: /orders/{{order_id}}\n    parameters: {{order_id: param.order_id}}\n    body: single\n    empty: {{status: 404, body: {{error: {{literal: {}}}}}}}\n",
        serde_json::to_string(text).expect("JSON string")
    );
    let http = compiled(&ir, "catalog", &document)
        .unwrap_or_else(|errors| panic!("compiles: {errors}\n{document}"));
    assert_eq!(
        ir_json(&http)["routes"][0]["empty"],
        json!({"status": 404, "body": {"error": {"literal": text}}})
    );
    round_trips(&http);
}

// --- a route's own refusals -----------------------------------------------------------------------

/// Route refusals apply on view routes too (the story's fit review: "on command and view routes").
#[test]
fn a_view_route_states_its_own_refusals() {
    let ir = model_of("orders.yaml", ORDERS);
    let document = "refusals:\n  malformed: {status: 400, body: {error: {literal: invalid_request}}}\nroutes:\n  - view: catalog.orders.OrdersOfSize\n    method: get\n    path: /orders/size/{quantity}\n    parameters: {quantity: param.quantity}\n    body: array\n    refusals:\n      malformed: {status: 422, body: none}\n";
    let http =
        compiled(&ir, "catalog", document).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let json = ir_json(&http);
    assert_eq!(json["routes"][0]["body"], "array", "{json:#}");
    assert_eq!(
        json["routes"][0]["refusals"],
        json!({"malformed": {"status": 422, "body": "none"}}),
        "{json:#}"
    );
    assert_eq!(json["refusals"]["malformed"]["status"], 400, "{json:#}");
    round_trips(&http);
    // Written empty, a route's `refusals:` overrides nothing and compiled IR omits it.
    let empty = document.replace(
        "    refusals:\n      malformed: {status: 422, body: none}\n",
        "    refusals: {}\n",
    );
    let http =
        compiled(&ir, "catalog", &empty).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert!(
        ir_json(&http)["routes"][0].get("refusals").is_none(),
        "{}",
        http.to_canonical_json()
    );
}

// --- pass 2: `alike` against the errors' fields and `error.wire` headers --------------------------

/// `fixtures/notes.yaml` with `Refused` carrying `first` and a second error `Declined`, also
/// travelling as `refused`, carrying `second`; `blank` reports `Declined`.
fn two_errors(first: &str, second: &str) -> EssIr {
    let model = NOTES
        .replace(
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
            &format!(
                "  - name: demo.notes.Refused\n    naming: {{wire: refused}}\n    fields:\n{first}  - name: demo.notes.Declined\n    naming: {{wire: refused}}\n    fields:\n{second}"
            ),
        )
        .replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    assert_ne!(model, NOTES);
    model_of("notes.yaml", &model)
}

const ERROR_OBJECTS_ALIKE: &str = "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n    alike: [[negative, blank]]\n";

/// Two errors whose fields are the same, by wire name, type and order, write one body: the group
/// compiles. The fields' own names may differ where their wire names agree.
#[test]
fn alike_error_objects_over_errors_with_one_body_compile() {
    for (first, second) in [
        (
            "      - {name: detail, type: String}\n      - {name: limit, type: Integer}\n",
            "      - {name: detail, type: String}\n      - {name: limit, type: Integer}\n",
        ),
        (
            "      - {name: detail, type: String}\n",
            "      - {name: why, type: String, naming: {wire: detail}}\n",
        ),
    ] {
        let ir = two_errors(first, second);
        let result = compiled(&ir, "demo", &create(ERROR_OBJECTS_ALIKE));
        assert_eq!(
            located(&result),
            [],
            "{first} / {second}: {}",
            worded(&result)
        );
    }
}

/// The same fields in another order, and a required field against an `Optional` one under one wire
/// name, are refused: the first writes the members in another order, the second may leave one out.
#[test]
fn alike_error_objects_over_reordered_or_optional_fields_are_refused() {
    for (first, second) in [
        (
            "      - {name: detail, type: String}\n      - {name: limit, type: Integer}\n",
            "      - {name: limit, type: Integer}\n      - {name: detail, type: String}\n",
        ),
        (
            "      - {name: detail, type: String}\n",
            "      - {name: why, type: Optional<String>, naming: {wire: detail}}\n",
        ),
    ] {
        let ir = two_errors(first, second);
        let result = compiled(&ir, "demo", &create(ERROR_OBJECTS_ALIKE));
        assert!(
            located(&result).contains(&("ESS-HTTP-025", "routes[0].alike".to_owned())),
            "{first} / {second}: {}",
            worded(&result)
        );
    }
}

/// A header reading `error.wire` tells `too-large` from `negative` however it is accompanied: beside
/// a literal header, and as a challenge param beside a literal param. Over one wire name it does not.
#[test]
fn alike_error_wire_beside_literals_is_refused_and_over_one_wire_name_compiles() {
    let ir = model_of("notes.yaml", NOTES);
    for keys in [
        "    errors:\n      status: 409\n      body: none\n      headers:\n        X-Kind: {literal: refused}\n        X-Error: {source: error.wire}\n    alike: [[too-large, negative, blank]]\n",
        "    errors:\n      status: 401\n      body: none\n      headers:\n        X-Kind: {literal: refused}\n        WWW-Authenticate:\n          challenge:\n            scheme: Bearer\n            params:\n              realm: {literal: notes}\n              error: {source: error.wire}\n    alike: [[too-large, negative, blank]]\n",
    ] {
        let result = compiled(&ir, "demo", &create(keys));
        assert!(
            located(&result).contains(&("ESS-HTTP-025", "routes[0].alike".to_owned())),
            "{keys}: {}",
            worded(&result)
        );
    }
    // `negative` and `blank` both report `Refused`: `error.wire` writes `refused` for both.
    let one_wire = "    outcomes:\n      negative:\n        status: 409\n        body: none\n        headers:\n          X-Kind: {literal: refused}\n          X-Error: {source: error.wire}\n      blank:\n        status: 409\n        body: none\n        headers:\n          X-Kind: {literal: refused}\n          X-Error: {source: error.wire}\n    alike: [[negative, blank]]\n";
    let result = compiled(&ir, "demo", &create(one_wire));
    assert_eq!(located(&result), [], "{}", worded(&result));
    round_trips(&result.expect("compiles"));
}
