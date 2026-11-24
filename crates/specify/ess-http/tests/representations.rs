//! A view route answers its rows in a second representation, CSV, that one literal value of one
//! query parameter selects (`representations:`, beyond10x/ess#753,
//! `story:a-view-route-answers-a-csv-representation-its-query-value-selects`;
//! `docs/design/http-presentation-binding.md`, "Representations of a view's answer").

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation,
    Refined,
};
use serde_json::{json, Value};

const MODEL: &str = include_str!("fixtures/reports.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("reports.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("reports.yaml", MODEL);
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
                path: format!("/reports/commands/{}", handle.name().local()),
            });
        }
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                table.push(DerivedRoute {
                    component: component.name.to_string(),
                    operation: Operation::View(handle.name().to_string()),
                    method: Method::Get,
                    path: format!("/reports/views/{}", handle.name().local()),
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
    match result {
        Ok(_) => String::new(),
        Err(diagnostics) => diagnostics.to_string(),
    }
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

fn ir_json(http: &HttpIr) -> Value {
    serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON")
}

/// Compiles, writes, reads back and writes again: the bytes do not move.
fn round_trips(http: &HttpIr) {
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text)
        .unwrap_or_else(|error| panic!("the compiled IR reads back: {error}\n{text}"));
    assert_eq!(read.to_canonical_json(), text, "round trip changed bytes");
}

/// One route serving `operation` (`view: demo.reports.Rows`, say) on `get <path>` with `keys`.
fn route(operation: &str, path: &str, keys: &str) -> String {
    format!("routes:\n  - {operation}\n    method: get\n    path: {path}\n{keys}")
}

/// `view: demo.reports.Rows` on `get /reports/rows` with `keys`.
fn rows(keys: &str) -> String {
    route("view: demo.reports.Rows", "/reports/rows", keys)
}

/// The route of the request: rows as JSON, and as `;`-separated CSV with `?format=csv`.
const CSV: &str = "    body: array
    representations:
      - query: {format: csv}
        encoding: {csv: {separator: \";\", header: true}}
        headers:
          Content-Disposition: {literal: attachment}
";

#[test]
fn a_view_route_compiles_its_csv_representation() {
    let ir = model();
    let http = compiled(&ir, &rows(CSV)).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert_eq!(
        ir_json(&http)["routes"][0]["representations"],
        json!([{
            "query": {"format": "csv"},
            "encoding": {"csv": {"separator": ";"}},
            "headers": {"Content-Disposition": {"literal": "attachment"}},
        }]),
        "`header: true` is the default and is omitted"
    );
    round_trips(&http);
}

#[test]
fn representations_compile_in_the_order_written_with_their_defaults_omitted() {
    let ir = model();
    let http = compiled(
        &ir,
        &rows(
            "    representations:
      - query: {format: tsv}
        encoding: {csv: {separator: \"\\t\", header: false}}
      - query: {format: csv}
        encoding: {csv: {}}
      - query: {format: plain}
        encoding: {csv: {separator: \",\", header: true}}
",
        ),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert_eq!(
        ir_json(&http)["routes"][0]["representations"],
        json!([
            {"query": {"format": "tsv"}, "encoding": {"csv": {"separator": "\t", "header": false}}},
            {"query": {"format": "csv"}, "encoding": {"csv": {}}},
            {"query": {"format": "plain"}, "encoding": {"csv": {}}},
        ])
    );
    round_trips(&http);
}

#[test]
fn a_binding_without_representations_keeps_its_bytes() {
    let ir = model();
    let http = compiled(&ir, &rows("    body: array\n"))
        .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let text = http.to_canonical_json();
    assert!(!text.contains("representations"), "{text}");
    assert_eq!(
        ir_json(&http)["routes"][0],
        json!({"view": "demo.reports.Rows", "method": "get", "path": "/reports/rows", "body": "array"})
    );
}

/// `query: <query>` with a CSV encoding, as the one representation of `Rows`.
fn one(query: &str) -> String {
    rows(&format!(
        "    representations:\n      - query: {query}\n        encoding: {{csv: {{}}}}\n"
    ))
}

#[test]
fn a_representation_out_of_rules_is_refused_at_its_key() {
    let ir = model();
    let at = |key: &str| refused("ESS-HTTP-035", &format!("routes[0].representations{key}"));
    let representation =
        "    representations:\n      - query: {format: csv}\n        encoding: {csv: {}}\n";
    for (document, expected) in [
        // On a command, a `single` view and a paged view.
        (
            route(
                "command: demo.reports.TouchRow",
                "/reports/{row_id}/touch",
                &format!("    parameters: {{row_id: input.row_id}}\n{representation}"),
            ),
            at(""),
        ),
        (
            route(
                "view: demo.reports.RowById",
                "/reports/{row_id}",
                &format!(
                    "    parameters: {{row_id: param.row_id}}\n    body: single\n    empty: {{status: 404}}\n{representation}"
                ),
            ),
            at(""),
        ),
        (
            route("view: demo.reports.RowPage", "/reports/page", representation),
            at(""),
        ),
        (rows("    representations: []\n"), at("")),
        // The selector: exactly one parameter, each name once and one name for all, each value
        // once, never a view parameter or a path variable, `unreserved` characters only.
        (one("{format: csv, kind: x}"), at("[0].query")),
        (one("{}"), at("[0].query")),
        (one("{\"for mat\": csv}"), at("[0].query")),
        (one("{format: \"\"}"), at("[0].query")),
        (one("{format: c/sv}"), at("[0].query")),
        (
            rows(
                "    representations:
      - query: {format: csv}
        encoding: {csv: {}}
      - query: {export: csv}
        encoding: {csv: {}}
",
            ),
            at("[1].query"),
        ),
        (
            rows(
                "    representations:
      - query: {format: csv}
        encoding: {csv: {}}
      - query: {format: csv}
        encoding: {csv: {separator: \";\"}}
",
            ),
            at("[1].query"),
        ),
        (
            route(
                "view: demo.reports.RowsOfKind",
                "/reports/kinds",
                "    representations:\n      - query: {kind: csv}\n        encoding: {csv: {}}\n",
            ),
            at("[0].query"),
        ),
        (
            route(
                "view: demo.reports.RowsOfKind",
                "/reports/kinds/{k}",
                "    parameters: {k: param.kind}\n    representations:\n      - query: {k: csv}\n        encoding: {csv: {}}\n",
            ),
            at("[0].query"),
        ),
        // The encoding.
        (
            rows(
                "    representations:\n      - query: {format: csv}\n        encoding: {csv: {separator: \"|\"}}\n",
            ),
            at("[0].encoding.csv.separator"),
        ),
        (
            route(
                "view: demo.reports.RowTags",
                "/reports/tags",
                representation,
            ),
            at("[0].encoding"),
        ),
    ] {
        assert_eq!(located(compiled(&ir, &document)), [expected], "{document}");
    }
    let list = worded(compiled(
        &ir,
        &route(
            "view: demo.reports.RowTags",
            "/reports/tags",
            representation,
        ),
    ));
    assert!(list.contains("`tags`"), "{list}");
}

#[test]
fn a_representation_takes_literal_headers_only() {
    let ir = model();
    let headers = |headers: &str| {
        rows(&format!(
            "    representations:\n      - query: {{format: csv}}\n        encoding: {{csv: {{}}}}\n        headers:\n{headers}"
        ))
    };
    for (written, name) in [
        ("          X-Kind: {source: input.kind}\n", "X-Kind"),
        (
            "          WWW-Authenticate: {challenge: {scheme: Bearer}}\n",
            "WWW-Authenticate",
        ),
        (
            "          Content-Type: {literal: text/plain}\n",
            "Content-Type",
        ),
        ("          X-Note: {literal: \" padded\"}\n", "X-Note"),
        ("          Bad Name: {literal: x}\n", "Bad Name"),
        (
            "          X-Note: {literal: a}\n          x-note: {literal: b}\n",
            "x-note",
        ),
    ] {
        assert_eq!(
            located(compiled(&ir, &headers(written))),
            [refused(
                "ESS-HTTP-026",
                &format!("routes[0].representations[0].headers.{name}")
            )],
            "{written}"
        );
    }
}

#[test]
fn a_refinement_restates_representations() {
    let ir = model();
    let base = rows(CSV);
    let refined = Refined::from_yaml((header(&ir) + &base).as_bytes()).expect("parses");
    let refining = |document: &str| {
        let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + document;
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        located(compile_refining(&spec, Some(&refined), &ir, &derived(&ir)))
    };
    assert_eq!(refining(&base), []);
    // A default written out or left out counts as that default.
    assert_eq!(refining(&base.replace(", header: true", "")), []);
    for changed in [
        rows("    body: array\n"),
        base.replace("\";\"", "\",\""),
        base.replace("attachment", "inline"),
        base.replace("{format: csv}", "{format: text}"),
        rows(&format!(
            "{CSV}      - query: {{format: tsv}}\n        encoding: {{csv: {{separator: \"\\t\"}}}}\n"
        )),
    ] {
        assert_eq!(
            refining(&changed),
            [refused("ESS-HTTP-012", "routes[0].representations")],
            "{changed}"
        );
    }
}

/// The first route's `representations` of compiled IR read as JSON.
fn representation(ir: &mut Value) -> &mut Value {
    &mut ir["routes"][0]["representations"]
}

#[test]
fn compiled_ir_with_representations_out_of_rules_is_refused() {
    let ir = model();
    let http = compiled(&ir, &rows(CSV)).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let good = ir_json(&http);
    let reads = |ir: &Value| HttpIr::from_json(&serde_json::to_string_pretty(ir).unwrap());
    assert!(reads(&good).is_ok());
    let with = |edit: &dyn Fn(&mut Value)| {
        let mut ir = good.clone();
        edit(&mut ir);
        ir
    };
    for (what, broken) in [
        ("an empty list", with(&|ir| *representation(ir) = json!([]))),
        (
            "`,` written out",
            with(&|ir| representation(ir)[0]["encoding"]["csv"]["separator"] = json!(",")),
        ),
        (
            "`header: true` written out",
            with(&|ir| representation(ir)[0]["encoding"]["csv"]["header"] = json!(true)),
        ),
        (
            "a separator outside the set",
            with(&|ir| representation(ir)[0]["encoding"]["csv"]["separator"] = json!("|")),
        ),
        (
            "two parameters",
            with(&|ir| representation(ir)[0]["query"] = json!({"a": "x", "format": "csv"})),
        ),
        (
            "a value not `unreserved`",
            with(&|ir| representation(ir)[0]["query"] = json!({"format": "c/sv"})),
        ),
        (
            "a source header",
            with(&|ir| {
                representation(ir)[0]["headers"] =
                    json!({"Content-Disposition": {"source": "input.kind"}});
            }),
        ),
        (
            "two parameter names",
            with(&|ir| {
                let second = json!({"query": {"export": "csv"}, "encoding": {"csv": {}}});
                representation(ir).as_array_mut().unwrap().push(second);
            }),
        ),
        (
            "one value twice",
            with(&|ir| {
                let second = json!({"query": {"format": "csv"}, "encoding": {"csv": {}}});
                representation(ir).as_array_mut().unwrap().push(second);
            }),
        ),
        (
            "a `single` body",
            with(&|ir| {
                ir["routes"][0]["body"] = json!("single");
                ir["routes"][0]["empty"] = json!({"status": 404});
            }),
        ),
    ] {
        assert!(reads(&broken).is_err(), "{what} read back");
    }
    // Beside a command.
    let command = compiled(
        &ir,
        &route(
            "command: demo.reports.TouchRow",
            "/reports/{row_id}/touch",
            "    parameters: {row_id: input.row_id}\n",
        ),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let mut broken = ir_json(&command);
    broken["routes"][0]["representations"] = good["routes"][0]["representations"].clone();
    assert!(
        reads(&broken).is_err(),
        "a command's representations read back"
    );
}

#[test]
fn an_older_reader_refuses_an_unknown_key_of_a_representation_by_name() {
    let ir = model();
    let text = header(&ir)
        + &rows(
            "    representations:\n      - query: {format: csv}\n        encoding: {csv: {}}\n        accept: text/csv\n",
        );
    let error = HttpSpec::from_yaml(&text)
        .expect_err("unknown key")
        .to_string();
    assert!(error.contains("unknown field `accept`"), "{error}");
}
