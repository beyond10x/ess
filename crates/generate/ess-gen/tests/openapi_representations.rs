//! The `OpenAPI` projection of a view route's CSV representation (beyond10x/ess#753,
//! `story:openapi-publishes-a-view-routes-csv-representation`;
//! `docs/design/http-presentation-binding.md`, "Representations of a view's answer").
//!
//! The selector is one optional `in: query` parameter whose `enum` holds each value; the `200`
//! response holds `text/csv` beside the JSON answer, names each representation under
//! `x-ess-representations`, and describes each declared header with the value that selects it.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{derived, Binding};
use ess_gen::openapi::BoundOpenApi;
use serde_json::{json, Value};

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/reports.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("reports.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("reports.yaml", MODEL);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The `reports` document of `routes`, as JSON.
fn document(ir: &EssIr, routes: &str) -> Value {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\n{routes}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    let binding = Binding::new(http).expect("every route has a path");
    let documents = run(&BoundOpenApi(binding), ir).expect("generates");
    let yaml: ess_yaml::Value =
        ess_yaml::from_str(&documents["openapi/reports.yaml"].contents).expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

const ROWS: &str = "routes:
  - view: demo.reports.Rows
    method: get
    path: /reports/rows
    body: array
";

const CSV: &str = "    representations:
      - query: {format: csv}
        encoding: {csv: {separator: \";\", header: true}}
        headers:
          Content-Disposition: {literal: attachment}
";

#[test]
fn a_csv_representation_is_a_second_media_type_and_its_selector_a_query_parameter() {
    let ir = model();
    let document = document(&ir, &format!("{ROWS}{CSV}"));
    let operation = &document["paths"]["/reports/rows"]["get"];
    assert_eq!(
        operation["parameters"],
        json!([{
            "name": "format",
            "in": "query",
            "description": "Selects another representation of the rows: `csv` answers the rows \
                            as CSV (`text/csv`), fields separated by `;`. Absent, the route \
                            answers `application/json`.",
            "required": false,
            "schema": {"type": "string", "enum": ["csv"]},
        }])
    );
    let read = &operation["responses"]["200"];
    assert_eq!(
        read["content"]["application/json"]["schema"],
        json!({"type": "array", "items": {"$ref": "#/components/schemas/demo.reports.Rows.Row"}}),
        "the JSON answer is today's"
    );
    assert_eq!(
        read["content"]["text/csv"],
        json!({"schema": {"type": "string"}})
    );
    assert_eq!(
        read["x-ess-representations"],
        json!([{
            "query": {"format": "csv"},
            "media_type": "text/csv",
            "separator": ";",
            "header": true,
            "columns": ["row_id", "label", "note"],
        }])
    );
    assert_eq!(
        read["headers"],
        json!({"Content-Disposition": {
            "description": "Written as `attachment` when `format=csv` selects the CSV answer; \
                            not written on the JSON answer.",
            "schema": {"type": "string", "const": "attachment"},
        }})
    );
}

#[test]
fn every_representation_is_named_in_the_order_written() {
    let ir = model();
    let document = document(
        &ir,
        &format!(
            "{ROWS}    representations:
      - query: {{format: tsv}}
        encoding: {{csv: {{separator: \"\\t\", header: false}}}}
      - query: {{format: csv}}
        encoding: {{csv: {{}}}}
"
        ),
    );
    let operation = &document["paths"]["/reports/rows"]["get"];
    assert_eq!(
        operation["parameters"][0]["schema"],
        json!({"type": "string", "enum": ["tsv", "csv"]})
    );
    let named: Vec<(Value, Value, Value)> = operation["responses"]["200"]["x-ess-representations"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|entry| {
            (
                entry["query"].clone(),
                entry["separator"].clone(),
                entry["header"].clone(),
            )
        })
        .collect();
    assert_eq!(
        named,
        [
            (json!({"format": "tsv"}), json!("\t"), json!(false)),
            (json!({"format": "csv"}), json!(","), json!(true)),
        ]
    );
}

#[test]
fn a_view_route_without_representations_keeps_its_answer() {
    let ir = model();
    let document = document(&ir, ROWS);
    let operation = &document["paths"]["/reports/rows"]["get"];
    assert!(operation.get("parameters").is_none(), "{operation}");
    let read = &operation["responses"]["200"];
    assert!(read.get("x-ess-representations").is_none(), "{read}");
    assert!(read["content"].get("text/csv").is_none(), "{read}");
}
