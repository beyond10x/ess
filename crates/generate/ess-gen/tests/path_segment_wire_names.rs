//! The `OpenAPI` projection never builds a path from a refused wire name (beyond10x/ess#493).
//!
//! `ess_gen::http::routes` copies a domain's, a command's and a view's `naming.wire` into
//! `/{domain}/commands/{command}` and `/{domain}/views/{view}`, and the `OpenAPI` document keys its
//! `paths` by them. A wire name holding `/`, or one that is `.` or `..`, is refused before any IR
//! exists, so no projection can be handed one; the accepted control shows the same fixture does
//! reach the projection and that its names land in the paths verbatim.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::routes;
use ess_gen::openapi::OpenApi;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn source(domain: &str, command: &str, view: &str) -> String {
    format!(
        "format: ess/1
system: demo
version: v1
domain: demo.orders
naming: {{wire: '{domain}'}}
types:
  - {{name: demo.orders.OrderId, kind: newtype, of: String}}
entities:
  - name: demo.orders.Order
    identity: {{name: id, type: demo.orders.OrderId}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
errors:
  - name: demo.orders.Rejected
commands:
  - name: demo.orders.Place
    naming: {{wire: '{command}'}}
    input: []
    outcomes: [{{name: refused, error: demo.orders.Rejected}}]
views:
  - name: demo.orders.Open
    naming: {{wire: '{view}'}}
    source: demo.orders.Order
    fields:
      - {{name: id, type: demo.orders.OrderId}}
components:
  - component: orders
    owns: {{domains: [demo.orders]}}
    accepts: {{commands: [demo.orders.Place]}}
    reached_by: network
"
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    Specification::assemble([(Source::new("orders.yaml"), raw)])
}

fn compiled(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("orders.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

const REFUSED: [&str; 4] = ["a/b", ".", "..", "../.well-known/demo-configuration"];

#[test]
fn a_refused_wire_name_in_any_segment_stops_before_openapi() {
    for wire in REFUSED {
        for (domain, command, view) in [
            (wire, "place", "open"),
            ("orders", wire, "open"),
            ("orders", "place", wire),
        ] {
            let text = source(domain, command, view);
            match assemble(&text) {
                Ok(_) => panic!("{wire:?} reached the IR, and so the OpenAPI paths:\n{text}"),
                Err(errors) => assert!(
                    errors.contains(ValidationCode::PathSegmentWireName),
                    "{wire:?} refused for another reason: {errors}"
                ),
            }
        }
    }
}

#[test]
fn the_accepted_control_reaches_openapi_with_its_names_verbatim() {
    let ir = compiled(&source("orders.v1", ".well-known", "..."));
    let component = ir.components().values().next().expect("one component");
    let paths: Vec<String> = routes(&ir, component)
        .into_iter()
        .map(|route| route.path)
        .collect();
    assert_eq!(
        paths,
        ["/orders.v1/commands/.well-known", "/orders.v1/views/..."]
    );
    let document = run(&OpenApi, &ir)
        .expect("openapi generates")
        .into_values()
        .map(|artifact| artifact.contents)
        .collect::<String>();
    for path in &paths {
        assert!(document.contains(&format!("{path}:")), "{path}\n{document}");
    }
}
