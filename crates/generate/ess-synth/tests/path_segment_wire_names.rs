//! The Rust and Go servers never serve a path built from a refused wire name (beyond10x/ess#493).
//!
//! Both targets route from `ess_gen::http::routes`, which copies a domain's, a command's and a
//! view's `naming.wire` into the path verbatim. A wire name holding `/`, or one that is `.` or
//! `..`, is refused before any IR exists, so neither target can be handed one; the accepted control
//! shows the same fixture does reach both targets and that its names land in the served paths.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_synth::{synthesize_for, Target};

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

fn emitted(ir: &EssIr, target: Target) -> String {
    synthesize_for(ir, target)
        .unwrap_or_else(|failure| panic!("{} synthesizes: {failure:?}", target.name()))
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

const REFUSED: [&str; 4] = ["a/b", ".", "..", "../.well-known/demo-configuration"];

/// Every placement of every refused shape stops at validation, before either server exists.
fn assert_refused_before(target: Target) {
    for wire in REFUSED {
        for (domain, command, view) in [
            (wire, "place", "open"),
            ("orders", wire, "open"),
            ("orders", "place", wire),
        ] {
            let text = source(domain, command, view);
            match assemble(&text) {
                Ok(_) => panic!(
                    "{wire:?} reached the IR, and so the {} server's routes:\n{text}",
                    target.name()
                ),
                Err(errors) => assert!(
                    errors.contains(ValidationCode::PathSegmentWireName),
                    "{wire:?} refused for another reason: {errors}"
                ),
            }
        }
    }
}

/// The accepted control reaches the target and serves its names verbatim.
fn assert_control_served(target: Target) {
    let ir = compiled(&source("orders.v1", ".well-known", "..."));
    let code = emitted(&ir, target);
    for path in ["/orders.v1/commands/.well-known", "/orders.v1/views/..."] {
        assert!(
            code.contains(&format!("\"{path}\"")),
            "{} does not serve `{path}`",
            target.name()
        );
    }
}

#[test]
fn the_rust_server_never_routes_a_refused_wire_name() {
    assert_refused_before(Target::Rust);
    assert_control_served(Target::Rust);
}

#[test]
fn the_go_server_never_routes_a_refused_wire_name() {
    assert_refused_before(Target::Go);
    assert_control_served(Target::Go);
}
