//! A command route takes a `multipart/form-data` request (RFC 7578) with file parts:
//! `request: {encoding: multipart, parts: {<field>: {filename, media_type, media_types}}}`
//! (beyond10x/ess#754, `story:a-command-route-takes-a-multipart-request-with-file-parts`;
//! `docs/design/http-presentation-binding.md`, "Request encoding").

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, DerivedRoute, Encoding, HttpDiagnostics, HttpIr, HttpSpec, Method,
    Operation, Refined,
};
use serde_json::{json, Value};

const MODEL: &str = include_str!("fixtures/imports.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("imports.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("imports.yaml", MODEL);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The routes ESS derives for `ir`: each command on `post /imports/commands/<name>`.
fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table: Vec<DerivedRoute> = ir
        .components()
        .values()
        .flat_map(|component| {
            component.accepts.iter().map(|handle| DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/imports/commands/{}", handle.name().local()),
            })
        })
        .collect();
    table
        .sort_by(|left, right| (&left.path, &left.operation).cmp(&(&right.path, &right.operation)));
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

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

fn ir_json(http: &HttpIr) -> Value {
    serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON")
}

/// `ImportRows` on `post /reports/rows/import` with `request`.
fn import(request: &str) -> String {
    format!(
        "routes:\n  - command: demo.imports.ImportRows\n    method: post\n    path: \
         /reports/rows/import\n    request:\n{request}"
    )
}

/// The story's route, less its `query:` line.
const IMPORT: &str = "      encoding: multipart
      parts:
        file:
          filename: input.file_name
          media_types: [text/csv, application/vnd.ms-excel]
";

#[test]
fn a_multipart_route_compiles_its_encoding_and_parts() {
    let ir = model();
    let http = compiled(&ir, &import(IMPORT)).unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(
        ir_json(&http)["routes"][0]["request"],
        json!({
            "encoding": "multipart",
            "parts": {
                "file": {
                    "filename": "input.file_name",
                    "media_types": ["text/csv", "application/vnd.ms-excel"],
                },
            },
        })
    );
    let request = http.routes()[0].request.as_ref().expect("a request");
    assert_eq!(request.encoding, Encoding::Multipart);
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text).unwrap_or_else(|error| panic!("reads back: {error}"));
    assert_eq!(read.to_canonical_json(), text);

    // Parts in name order, a media type read into an `Optional` input, a filename into a newtype
    // of `String`, and a part without attributes left to the encoding: none needs a `parts` entry.
    let photo = "routes:
  - command: demo.imports.AttachPhoto
    method: post
    path: /reports/{report_id}/photos
    parameters: {report_id: input.report_id}
    request:
      encoding: multipart
      unknown: ignore
      parts:
        photo: {media_type: input.photo_type, filename: input.photo_name, media_types: [image/png, image/jpeg]}
";
    let http = compiled(&ir, photo).unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(
        ir_json(&http)["routes"][0]["request"],
        json!({
            "encoding": "multipart",
            "parts": {"photo": {
                "filename": "input.photo_name",
                "media_type": "input.photo_type",
                "media_types": ["image/png", "image/jpeg"],
            }},
            "unknown": "ignore",
        })
    );
    let plain = "routes:
  - command: demo.imports.AttachPhoto
    method: post
    path: /reports/{report_id}/photos
    parameters: {report_id: input.report_id}
    request: {encoding: multipart}
";
    let http = compiled(&ir, plain).unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(
        ir_json(&http)["routes"][0]["request"],
        json!({"encoding": "multipart"})
    );
}

#[test]
fn multipart_on_get_and_a_part_without_a_flat_spelling_are_refused() {
    let ir = model();
    let get = "routes:
  - command: demo.imports.ImportRows
    method: get
    path: /import
    request: {encoding: multipart}
";
    assert_eq!(
        located(&compiled(&ir, get)),
        [refused("ESS-HTTP-020", "routes[0].request.encoding")]
    );
    let shaped = "routes:
  - command: demo.imports.ShapeRows
    method: post
    path: /shape
    request: {encoding: multipart}
";
    assert_eq!(
        located(&compiled(&ir, shaped)),
        [refused("ESS-HTTP-021", "routes[0].request.encoding")]
    );
}

#[test]
fn ess_http_036_parts_out_of_their_rules_are_refused() {
    let ir = model();
    let cases: &[(&str, &str, &str)] = &[
        (
            "      encoding: form\n      parts: {file: {filename: input.file_name}}\n",
            "routes[0].request.parts",
            "`multipart`",
        ),
        (
            "      encoding: multipart\n      parts: {format: {media_types: [text/csv]}}\n",
            "routes[0].request.parts.format",
            "`Bytes`",
        ),
        (
            "      encoding: multipart\n      parts: {nothing: {media_types: [text/csv]}}\n",
            "routes[0].request.parts.nothing",
            "no input field",
        ),
        (
            "      encoding: multipart\n      parts: {file: {filename: file_name}}\n",
            "routes[0].request.parts.file.filename",
            "`input.<field>`",
        ),
        (
            "      encoding: multipart\n      parts: {file: {filename: input.dry_run}}\n",
            "routes[0].request.parts.file.filename",
            "`String`",
        ),
        (
            "      encoding: multipart\n      parts: {file: {filename: input.file}}\n",
            "routes[0].request.parts.file.filename",
            "`String`",
        ),
        (
            "      encoding: multipart\n      parts: {file: {filename: input.file_name, media_type: input.file_name}}\n",
            "routes[0].request.parts.file.media_type",
            "twice",
        ),
        (
            "      encoding: multipart\n      parts: {file: {media_types: []}}\n",
            "routes[0].request.parts.file.media_types",
            "empty",
        ),
        (
            "      encoding: multipart\n      parts: {file: {media_types: [text/csv, csv]}}\n",
            "routes[0].request.parts.file.media_types",
            "`csv`",
        ),
        (
            "      encoding: multipart\n      parts: {file: {media_types: [text/csv, text/csv]}}\n",
            "routes[0].request.parts.file.media_types",
            "twice",
        ),
        (
            "      encoding: multipart\n      parts: {file: {}}\n",
            "routes[0].request.parts.file",
            "states nothing",
        ),
    ];
    for (request, at, words) in cases {
        let result = compiled(&ir, &import(request));
        assert_eq!(located(&result), [refused("ESS-HTTP-036", at)], "{request}");
        let shown = result.expect_err("refused").to_string();
        assert!(shown.contains(words), "{request}: {shown}");
    }
    // A filename read from an input the path carries.
    let carried = "routes:
  - command: demo.imports.AttachPhoto
    method: post
    path: /reports/{report_id}/photos
    parameters: {report_id: input.report_id}
    request:
      encoding: multipart
      parts: {photo: {filename: input.report_id}}
";
    assert_eq!(
        located(&compiled(&ir, carried)),
        [refused(
            "ESS-HTTP-036",
            "routes[0].request.parts.photo.filename"
        )]
    );
}

#[test]
fn compiled_ir_with_parts_out_of_their_rules_is_refused_naming_the_key() {
    let ir = model();
    let json = compiled(&ir, &import(IMPORT))
        .expect("compiles")
        .to_canonical_json();
    let encoding = "\"encoding\": \"multipart\"";
    assert!(json.contains(encoding), "{json}");
    for (from, to, named) in [
        (encoding, "\"encoding\": \"form\"", "parts"),
        (
            "\"filename\": \"input.file_name\"",
            "\"filename\": \"input.file\"",
            "parts.file",
        ),
        (
            "\"filename\": \"input.file_name\"",
            "\"filename\": \"param.file_name\"",
            "filename",
        ),
        ("\"text/csv\",", "\"csv\",", "media_types"),
    ] {
        let broken = json.replacen(from, to, 1);
        assert_ne!(broken, json, "{from}");
        let error = HttpIr::from_json(&broken).expect_err(to).to_string();
        assert!(error.contains(named), "{to}: {error}");
    }
    let bare = compiled(&ir, &import("      encoding: multipart\n"))
        .expect("compiles")
        .to_canonical_json();
    let emptied = bare.replacen(
        encoding,
        "\"encoding\": \"multipart\",\n        \"parts\": {}",
        1,
    );
    let error = HttpIr::from_json(&emptied)
        .expect_err("empty parts")
        .to_string();
    assert!(error.contains("parts"), "{error}");
}

#[test]
fn a_refinement_keeps_the_encoding_and_parts_of_a_route_it_restates() {
    let ir = model();
    let refined_text = header(&ir) + &import(IMPORT);
    let refined = Refined::from_yaml(refined_text.as_bytes()).expect("parses");
    let changed = import(
        "      encoding: multipart
      parts:
        file:
          filename: input.file_name
          media_types: [text/csv]
",
    );
    let line = format!("refines: {}\n", refined.digest());
    let spec = HttpSpec::from_yaml(&(header(&ir) + &line + &changed)).expect("parses");
    let result = compile_refining(&spec, Some(&refined), &ir, &derived(&ir));
    assert_eq!(
        located(&result),
        [refused("ESS-HTTP-012", "routes[0].request.parts")]
    );
    let form = import("      encoding: form\n");
    let spec = HttpSpec::from_yaml(&(header(&ir) + &line + &form)).expect("parses");
    let codes = located(&compile_refining(&spec, Some(&refined), &ir, &derived(&ir)));
    assert!(
        codes.contains(&refused("ESS-HTTP-012", "routes[0].request.encoding")),
        "{codes:?}"
    );
}

#[test]
fn a_binding_without_multipart_compiles_its_bytes() {
    let ir = model();
    let form = "routes:
  - command: demo.imports.AttachPhoto
    method: post
    path: /reports/{report_id}/photos
    parameters: {report_id: input.report_id}
    request: {encoding: form}
";
    let json = compiled(&ir, form).expect("compiles").to_canonical_json();
    assert!(!json.contains("parts"), "{json}");
    assert!(
        json.contains("\"request\": {\n        \"encoding\": \"form\"\n      }"),
        "{json}"
    );
}
