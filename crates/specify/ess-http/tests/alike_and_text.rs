//! Two outcomes a route answers alike (`alike:`), and an outcome that answers one response field
//! as text (`encoding: text`) (`docs/design/http-presentation-binding.md`, "Status per outcome"
//! and "Response body per outcome"). Both are additive within `ess-http/1` and `ess-http-ir/1`: a
//! document that uses neither compiles to the bytes it compiled to before.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, Body, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method,
    Operation, Refined, ResponseEncoding,
};
use serde_json::{json, Value};

const MODEL: &str = include_str!("fixtures/notes.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("notes.yaml", MODEL);
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
                path: format!("/notes/commands/{}", handle.name().local()),
            });
        }
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                table.push(DerivedRoute {
                    component: component.name.to_string(),
                    operation: Operation::View(handle.name().to_string()),
                    method: Method::Get,
                    path: format!("/notes/views/{}", handle.name().local()),
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
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\n",
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

fn worded(result: Result<HttpIr, HttpDiagnostics>) -> String {
    result.expect_err("refused").to_string()
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

/// A `CreateNote` route on `post /api/notes` with `keys` beside its path.
fn create(keys: &str) -> String {
    format!(
        "routes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n{keys}"
    )
}

/// The only route of `http`, as JSON.
fn only_route(http: &HttpIr) -> Value {
    let json: Value = serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON");
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

/// `fetched` and `inline` both answering `201` with the response.
const BOTH_RESPONSE: &str =
    "    outcomes:\n      fetched: {status: 201, body: response}\n      inline: {status: 201, body: response}\n";

// --- `alike` ------------------------------------------------------------------------------------

#[test]
fn outcomes_declared_alike_share_a_status_and_body() {
    let ir = model();
    // Without `alike`, the two are one answer a reader cannot tell apart.
    let words = worded(compiled(&ir, &create(BOTH_RESPONSE)));
    assert!(
        words.contains("ESS-HTTP-025") && words.contains("routes[0].outcomes.fetched"),
        "{words}"
    );
    let document = create(&format!("{BOTH_RESPONSE}    alike: [[fetched, inline]]\n"));
    let http = compiled(&ir, &document).expect("compiles");
    assert_eq!(only_route(&http)["alike"], json!([["fetched", "inline"]]));
    assert_eq!(
        http.routes()[0].alike,
        vec![vec!["fetched".to_owned(), "inline".to_owned()]]
    );
    // Each alike outcome still states its own status and body.
    assert_eq!(http.routes()[0].outcomes["inline"].body, Body::Response);
    round_trips(&http);
    // Written in another order, the group compiles to the same bytes.
    let other = compiled(
        &ir,
        &create(&format!("{BOTH_RESPONSE}    alike: [[inline, fetched]]\n")),
    )
    .expect("compiles");
    assert_eq!(other.to_canonical_json(), http.to_canonical_json());
    // Two `none` answers, and error objects under one wire name: listed, covered by `errors`, or
    // both.
    for keys in [
        "    outcomes:\n      fetched: {status: 204, body: none}\n      inline: {status: 204, body: none}\n    alike: [[fetched, inline]]\n",
        "    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n    alike: [[negative, blank]]\n",
        "    outcomes:\n      negative: {status: 409, body: none}\n    errors: {status: 409, body: none}\n    alike: [[negative, blank, too-large]]\n",
        "    errors: {status: 409, body: none}\n    alike: [[too-large, negative, blank]]\n",
    ] {
        let http = compiled(&ir, &create(keys)).unwrap_or_else(|refused| panic!("{keys}: {refused}"));
        round_trips(&http);
    }
    // Members in name order, groups ordered by their first member.
    let http = compiled(
        &ir,
        &create(&format!(
            "{BOTH_RESPONSE}      negative: {{status: 409, body: none}}\n      blank: {{status: 409, body: none}}\n    alike: [[inline, fetched], [negative, blank]]\n"
        )),
    )
    .expect("compiles");
    assert_eq!(
        only_route(&http)["alike"],
        json!([["blank", "negative"], ["fetched", "inline"]])
    );
    round_trips(&http);
}

#[test]
fn alike_outcomes_whose_answers_differ_are_refused() {
    let ir = model();
    for (keys, key) in [
        (
            "    outcomes:\n      fetched: {status: 201, body: response}\n      inline: {status: 201, body: none}\n",
            "body",
        ),
        (
            "    outcomes:\n      fetched: {status: 201, body: response}\n      inline: {status: 200, body: response}\n",
            "status",
        ),
        (
            "    outcomes:\n      fetched:\n        status: 201\n        body: response\n        headers: {X-Kind: {literal: one}}\n      inline:\n        status: 201\n        body: response\n        headers: {X-Kind: {literal: two}}\n",
            "headers",
        ),
        (
            "    outcomes:\n      fetched: {status: 201, body: response.note_id, encoding: text}\n      inline: {status: 201, body: response.note_id}\n",
            "encoding",
        ),
    ] {
        let document = create(&format!("{keys}    alike: [[fetched, inline]]\n"));
        let result = compiled(&ir, &document);
        let words = match &result {
            Ok(_) => panic!("{document} compiles"),
            Err(diagnostics) => diagnostics.to_string(),
        };
        assert_eq!(
            located(result),
            [refused("ESS-HTTP-025", "routes[0].alike")],
            "{keys}: {words}"
        );
        assert!(words.contains(&format!("`{key}`")), "{key}: {words}");
    }
}

#[test]
fn alike_outcomes_the_wire_tells_apart_are_refused() {
    let ir = model();
    for (keys, why) in [
        (
            "    outcomes:\n      fetched: {status: 201}\n      inline: {status: 201}\n    alike: [[fetched, inline]]\n",
            "envelope",
        ),
        (
            "    outcomes:\n      too-large: {status: 409, body: {error: error}}\n      negative: {status: 409, body: {error: error}}\n    alike: [[too-large, negative]]\n",
            "wire name",
        ),
    ] {
        let result = compiled(&ir, &create(keys));
        let words = match &result {
            Ok(_) => panic!("{keys} compiles"),
            Err(diagnostics) => diagnostics.to_string(),
        };
        assert_eq!(
            located(result),
            [refused("ESS-HTTP-025", "routes[0].alike")],
            "{keys}: {words}"
        );
        assert!(words.contains(why), "{why}: {words}");
    }
}

#[test]
fn alike_names_out_of_their_rules_are_refused() {
    let ir = model();
    let none = "    outcomes:\n      fetched: {status: 204, body: none}\n      inline: {status: 204, body: none}\n      skipped: {status: 204, body: none}\n";
    for (alike, why) in [
        ("[[fetched, nowhere]]", "`nowhere`"),
        (
            "[[fetched, inline, skipped], [skipped, inline]]",
            "two groups",
        ),
        ("[[fetched, inline, skipped], [fetched]]", "one"),
        ("[[fetched, fetched, inline, skipped]]", "twice"),
    ] {
        let document = create(&format!("{none}    alike: {alike}\n"));
        let result = compiled(&ir, &document);
        let words = match &result {
            Ok(_) => panic!("{alike} compiles"),
            Err(diagnostics) => diagnostics.to_string(),
        };
        let located = located(result);
        assert!(
            located.contains(&refused("ESS-HTTP-022", "routes[0].alike")),
            "{alike}: {words}"
        );
        assert!(
            located
                .iter()
                .all(|(code, path)| *code != "ESS-HTTP-022" || path == "routes[0].alike"),
            "{alike}: {words}"
        );
        assert!(words.contains(why), "{alike}: {why}: {words}");
    }
    // A name with no declared answer: `inline` is not listed and reports no error, so `errors`
    // does not cover it.
    let document = create(
        "    outcomes:\n      fetched: {status: 201, body: response}\n    alike: [[fetched, inline]]\n",
    );
    let words = worded(compiled(&ir, &document));
    assert_eq!(
        located(compiled(&ir, &document)),
        [refused("ESS-HTTP-022", "routes[0].alike")],
        "{words}"
    );
    assert!(words.contains("no declared answer"), "{words}");
    // On a view.
    let view = "routes:\n  - view: demo.notes.Notes\n    method: get\n    path: /api/notes\n    alike: [[fetched, inline]]\n";
    assert_eq!(
        located(compiled(&ir, view)),
        [refused("ESS-HTTP-022", "routes[0].alike")]
    );
}

#[test]
fn compiled_ir_with_alike_out_of_order_is_refused() {
    let ir = model();
    let http = compiled(
        &ir,
        &create(&format!(
            "{BOTH_RESPONSE}      negative: {{status: 409, body: none}}\n      blank: {{status: 409, body: none}}\n    alike: [[fetched, inline], [negative, blank]]\n"
        )),
    )
    .expect("compiles");
    let text = http.to_canonical_json();
    let alike = "\"alike\": [\n        [\n          \"blank\",\n          \"negative\"\n        ],\n        [\n          \"fetched\",\n          \"inline\"\n        ]\n      ]";
    assert!(text.contains(alike), "{text}");
    for (written, named) in [
        (
            "\"alike\": [[\"negative\", \"blank\"], [\"fetched\", \"inline\"]]",
            "order",
        ),
        (
            "\"alike\": [[\"fetched\", \"inline\"], [\"blank\", \"negative\"]]",
            "order",
        ),
        ("\"alike\": []", "alike"),
        ("\"alike\": [[\"blank\"]]", "two"),
        (
            "\"alike\": [[\"blank\", \"negative\"], [\"blank\", \"fetched\"]]",
            "once",
        ),
        ("\"alike\": [[\"blank\", \"fetched\"]]", "differ"),
        ("\"alike\": [[\"blank\", \"skipped\"]]", "skipped"),
    ] {
        let broken = text.replace(alike, written);
        assert_ne!(broken, text);
        let error = HttpIr::from_json(&broken).expect_err(written).to_string();
        assert!(
            error.contains("alike") && error.contains(named),
            "{written}: {error}"
        );
    }
    // An envelope answer in a group: the envelope names the outcome.
    let envelope = compiled(
        &ir,
        &create("    outcomes:\n      fetched: {status: 201}\n      inline: {status: 202}\n"),
    )
    .expect("compiles")
    .to_canonical_json();
    let inline = "\"status\": 202";
    assert!(envelope.contains(inline), "{envelope}");
    let broken = envelope.replace(inline, "\"status\": 201").replace(
        "\"path\": \"/api/notes\",",
        "\"path\": \"/api/notes\", \"alike\": [[\"fetched\", \"inline\"]],",
    );
    let error = HttpIr::from_json(&broken)
        .expect_err("envelope alike")
        .to_string();
    assert!(error.contains("envelope"), "{error}");
    // `alike` on a view route.
    let view = compiled(
        &ir,
        "routes:\n  - view: demo.notes.Notes\n    method: get\n    path: /api/notes\n",
    )
    .expect("compiles")
    .to_canonical_json();
    let path = "\"path\": \"/api/notes\"";
    assert!(view.contains(path), "{view}");
    let broken = view.replace(
        path,
        "\"path\": \"/api/notes\", \"alike\": [[\"fetched\", \"inline\"]]",
    );
    let error = HttpIr::from_json(&broken)
        .expect_err("alike on a view")
        .to_string();
    assert!(error.contains("alike"), "{error}");
}

#[test]
fn a_refinement_restates_alike() {
    let ir = model();
    let two = "      negative: {status: 409, body: none}\n      blank: {status: 409, body: none}\n";
    let base = create(&format!(
        "{BOTH_RESPONSE}{two}    alike: [[fetched, inline], [negative, blank]]\n"
    ));
    let refined = Refined::from_yaml((header(&ir) + &base).as_bytes()).expect("parses");
    let refining = |document: &str| {
        let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + document;
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
    };
    assert_eq!(located(refining(&base)), []);
    // The same groups written in another order are no change.
    let reordered = create(&format!(
        "{BOTH_RESPONSE}{two}    alike: [[blank, negative], [inline, fetched]]\n"
    ));
    assert_eq!(located(refining(&reordered)), []);
    for changed in [
        create(&format!(
            "{BOTH_RESPONSE}{two}    alike: [[fetched, inline]]\n"
        )),
        create(&format!(
            "{BOTH_RESPONSE}{two}    alike: [[fetched, inline, negative, blank]]\n"
        )),
        create(&format!("{BOTH_RESPONSE}{two}")),
    ] {
        let located = located(refining(&changed));
        assert!(
            located.contains(&refused("ESS-HTTP-012", "routes[0].alike")),
            "{changed}: {located:?}"
        );
    }
}

// --- `encoding: text` ---------------------------------------------------------------------------

#[test]
fn a_command_outcome_answers_a_text_body() {
    let ir = model();
    let http = compiled(
        &ir,
        &create(
            "    outcomes:\n      fetched: {status: 201, body: response.note_id, encoding: text}\n",
        ),
    )
    .expect("compiles");
    assert_eq!(
        only_route(&http)["outcomes"],
        json!({"fetched": {"status": 201, "body": "response.note_id", "encoding": "text"}})
    );
    let outcome = &http.routes()[0].outcomes["fetched"];
    assert_eq!(outcome.encoding, ResponseEncoding::Text);
    assert_eq!(outcome.body, Body::ResponseField("note_id".to_owned()));
    round_trips(&http);
    // `json` written out is the default, which compiled IR omits.
    let json_written = compiled(
        &ir,
        &create(
            "    outcomes:\n      fetched: {status: 201, body: response.note_id, encoding: json}\n",
        ),
    )
    .expect("compiles");
    let default = compiled(
        &ir,
        &create("    outcomes:\n      fetched: {status: 201, body: response.note_id}\n"),
    )
    .expect("compiles");
    assert_eq!(
        json_written.to_canonical_json(),
        default.to_canonical_json()
    );
    assert!(!default.to_canonical_json().contains("encoding"));
    assert_eq!(
        default.routes()[0].outcomes["fetched"].encoding,
        ResponseEncoding::Json
    );
}

#[test]
fn text_out_of_its_rules_is_refused() {
    let ir = model();
    for (outcome, answer, why) in [
        (
            "fetched",
            "{status: 201, body: response, encoding: text}",
            "`response`",
        ),
        (
            "skipped",
            "{status: 204, body: none, encoding: text}",
            "`none`",
        ),
        ("fetched", "{status: 201, encoding: text}", "`envelope`"),
        (
            "too-large",
            "{status: 409, body: {error: error}, encoding: text}",
            "`{error: error}`",
        ),
        (
            "fetched",
            "{status: 201, body: response.size, encoding: text}",
            "`Integer`",
        ),
        (
            "fetched",
            "{status: 201, body: response.note_id, encoding: xml}",
            "`xml`",
        ),
    ] {
        let document = create(&format!("    outcomes:\n      {outcome}: {answer}\n"));
        let result = compiled(&ir, &document);
        let words = match &result {
            Ok(_) => panic!("{answer} compiles"),
            Err(diagnostics) => diagnostics.to_string(),
        };
        assert_eq!(
            located(result),
            [refused(
                "ESS-HTTP-024",
                &format!("routes[0].outcomes.{outcome}.encoding")
            )],
            "{answer}: {words}"
        );
        assert!(words.contains(why), "{answer}: {why}: {words}");
    }
    // Under `errors`, once for each outcome it covers.
    let document = create("    errors: {status: 409, body: none, encoding: text}\n    alike: [[too-large, negative, blank]]\n");
    let located = located(compiled(&ir, &document));
    assert_eq!(
        located,
        vec![refused("ESS-HTTP-024", "routes[0].errors.encoding"); 3],
        "{located:?}"
    );
}

#[test]
fn encoding_is_held_by_the_ir_reader_and_refinement() {
    let ir = model();
    let text_body = create(
        "    outcomes:\n      fetched: {status: 201, body: response.note_id, encoding: text}\n",
    );
    let http = compiled(&ir, &text_body).expect("compiles");
    let text = http.to_canonical_json();
    let encoding = "\"encoding\": \"text\"";
    assert!(text.contains(encoding), "{text}");
    let error = HttpIr::from_json(&text.replace(encoding, "\"encoding\": \"json\""))
        .expect_err("json written out")
        .to_string();
    assert!(error.contains("encoding"), "{error}");
    let error = HttpIr::from_json(&text.replace(encoding, "\"encoding\": \"xml\""))
        .expect_err("xml")
        .to_string();
    assert!(error.contains("xml"), "{error}");
    for body in [
        "\"body\": \"response\"",
        "\"body\": \"none\"",
        "\"body\": {\"error\": \"error\"}",
    ] {
        let broken = text.replace("\"body\": \"response.note_id\"", body);
        assert_ne!(broken, text);
        let error = HttpIr::from_json(&broken).expect_err(body).to_string();
        assert!(error.contains("encoding"), "{body}: {error}");
    }
    // Refinement compares the encoding, a default written out or left out counting as `json`.
    let refined = Refined::from_yaml((header(&ir) + &text_body).as_bytes()).expect("parses");
    let refining = |document: &str| {
        let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + document;
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
    };
    assert_eq!(located(refining(&text_body)), []);
    for changed in [
        create("    outcomes:\n      fetched: {status: 201, body: response.note_id}\n"),
        create(
            "    outcomes:\n      fetched: {status: 201, body: response.note_id, encoding: json}\n",
        ),
    ] {
        assert_eq!(
            located(refining(&changed)),
            [refused("ESS-HTTP-012", "routes[0].outcomes.fetched")],
            "{changed}"
        );
    }
    let json_body = create("    outcomes:\n      fetched: {status: 201, body: response.note_id}\n");
    let refined = Refined::from_yaml((header(&ir) + &json_body).as_bytes()).expect("parses");
    let text = header(&ir)
        + &format!("refines: {}\n", refined.digest())
        + &create(
            "    outcomes:\n      fetched: {status: 201, body: response.note_id, encoding: json}\n",
        );
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    assert_eq!(
        located(compile_refining(&spec, Some(&refined), &ir, &derived(&ir))),
        []
    );
}
