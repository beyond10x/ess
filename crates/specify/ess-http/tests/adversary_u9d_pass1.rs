//! Adversary pass 1 on `body: response.<field>` and route `errors:`: the reference row names both
//! forms, and `errors` is held against `refusals:` overrides and listed outcomes on one status
//! (`ESS-HTTP-025`) as a listed outcome is.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const MODEL: &str = include_str!("fixtures/tasks.yaml");
const FORMATS: &str = include_str!("../../../../website/docs/reference/formats.md");

fn model_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new("tasks.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("tasks.yaml", text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

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

fn compiled(ir: &EssIr, document: &str) -> Result<HttpIr, HttpDiagnostics> {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n{document}",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
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

fn send_task(keys: &str) -> String {
    format!(
        "routes:\n  - command: catalog.tasks.SendTask\n    method: post\n    path: /api/tasks\n{keys}"
    )
}

/// The two rows of `formats.md` an adopter reads for `ess-http/1` and `ess-http-ir/1`.
fn rows() -> (&'static str, &'static str) {
    let authored = FORMATS
        .lines()
        .find(|line| line.contains("**`type: ess-http/1`**"))
        .expect("the ess-http/1 row");
    let compiled = FORMATS
        .lines()
        .find(|line| line.contains("**`format: ess-http-ir/1`**"))
        .expect("the ess-http-ir/1 row");
    (authored, compiled)
}

#[test]
fn the_reference_rows_name_a_response_field_body_and_route_errors() {
    let (authored, compiled) = rows();
    // `response.<field>` as a body, not only as a header source.
    assert!(
        authored.contains("`response.<field>` (one non-`Optional` field"),
        "{authored}"
    );
    assert!(authored.contains("`errors:`"), "{authored}");
    assert!(
        authored.contains("`outcomes` (with their `headers` and `redirect`), `errors`"),
        "the refinement sentence restates `errors`: {authored}"
    );
    assert!(
        compiled.contains("`errors`") && compiled.contains("absent when not declared"),
        "{compiled}"
    );
}

/// A `not_granted` literal equal to the wire name of an outcome `errors` covers, on that status
/// and member: a reader cannot tell the refusal from the outcome, as with a listed outcome.
#[test]
fn ess_http_025_a_refusal_literal_equal_to_a_covered_wire_name() {
    let ir = model_of(MODEL);
    let refusals = |literal: &str| {
        format!(
            "refusals:\n  not_granted: {{status: 403, body: {{error: {{literal: {literal}}}}}}}\n"
        )
    };
    let errors = "    errors: {status: 403, body: {error: error}}\n";
    let clashing = refusals("unsupported") + &send_task(errors);
    assert_eq!(
        located(compiled(&ir, &clashing)),
        [("ESS-HTTP-025", "refusals.not_granted".to_owned())]
    );
    // The same answer listed by name is refused the same way.
    let listed = refusals("unsupported")
        + &send_task(
            "    outcomes:\n      unsupported: {status: 403, body: {error: error}}\n      too-large: {status: 403, body: {error: error}}\n",
        );
    assert_eq!(
        located(compiled(&ir, &listed)),
        located(compiled(&ir, &clashing))
    );
    // A literal no covered error writes is told apart.
    let apart = refusals("access_denied") + &send_task(errors);
    assert_eq!(located(compiled(&ir, &apart)), []);
}

#[test]
fn ess_http_025_a_bodiless_refusal_on_the_status_errors_answers() {
    let ir = model_of(MODEL);
    let document = "refusals:\n  malformed: {status: 400, body: none}\n".to_owned()
        + &send_task("    errors: {status: 400, body: {error: error}}\n");
    assert_eq!(
        located(compiled(&ir, &document)),
        [("ESS-HTTP-025", "refusals.malformed".to_owned())]
    );
}

/// `errors` answering the envelope beside a listed error object on one status: the two bodies
/// differ in shape, and a reader still cannot tell which outcome ran from a member it reads.
#[test]
fn ess_http_025_errors_envelope_beside_a_listed_error_object() {
    let ir = model_of(MODEL);
    let document = send_task(
        "    outcomes:\n      too-large: {status: 400, body: {error: error}}\n    errors: {status: 400}\n",
    );
    let found = located(compiled(&ir, &document));
    assert!(
        found.contains(&("ESS-HTTP-025", "routes[0].errors".to_owned()))
            || found.contains(&("ESS-HTTP-025", "routes[0].outcomes.too-large".to_owned())),
        "{found:?}"
    );
}

/// The design page says a `response.<field>` body is "never `null`"; a `Json` field is not
/// `Optional`, so the check admits it, and its value may be JSON `null`.
#[test]
fn a_json_response_field_body_is_admitted_though_its_value_may_be_null() {
    let text = MODEL.replace(
        "      - {name: note, type: Optional<String>}\n",
        "      - {name: note, type: Optional<String>}\n      - {name: extra, type: Json}\n",
    );
    assert_ne!(text, MODEL);
    let ir = model_of(&text);
    let result = compiled(
        &ir,
        &send_task("    outcomes:\n      sent: {status: 200, body: response.extra}\n"),
    );
    assert_eq!(located(result), []);
    let design = include_str!("../../../../docs/design/http-presentation-binding.md");
    let row = design
        .lines()
        .find(|line| line.starts_with("| `response.<field>` |"))
        .expect("the design row");
    assert!(
        !row.contains("never `null`") || row.contains("Json"),
        "the design row promises no `null`, and a `Json` field answers one: {row}"
    );
}
