//! A wire name that a generated path segment reads refuses `/`, `.` and `..` (beyond10x/ess#493).
//!
//! Every generated HTTP path is `/{domain wire}/commands/{command wire}` or
//! `/{domain wire}/views/{view wire}`, copied verbatim from `naming.wire`. A wire name holding `/`,
//! or one that is exactly `.` or `..`, would make that segment address another route, so
//! validation refuses it on the domain, the command and the view, naming the declaration and the
//! wire name. A view is refused whether or not a network component serves it: the specification
//! is valid or not on its own, not per composition.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
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
"
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    Specification::assemble([(Source::new("orders.yaml"), raw)])
}

fn refusals(text: &str) -> Vec<(String, String)> {
    let Err(errors) = assemble(text) else {
        panic!("validated, and should have been refused:\n{text}")
    };
    errors
        .into_iter()
        .filter(|error| error.code == ValidationCode::PathSegmentWireName)
        .map(|error| (error.location, error.message))
        .collect()
}

#[test]
fn the_neighbouring_names_validate() {
    // Dots inside a segment, a leading dot and three dots are not traversal: only `/`, `.` and
    // `..` are. The control the refusals below are measured against.
    for (domain, command, view) in [
        ("orders", "place", "open"),
        ("orders.v1", ".well-known", "..."),
        ("orders", "place.v2", "open-orders"),
    ] {
        assemble(&source(domain, command, view))
            .unwrap_or_else(|errors| panic!("{domain} {command} {view}: {errors}"));
    }
}

#[test]
fn a_view_wire_name_that_climbs_out_of_its_segment_is_refused_by_name() {
    let found = refusals(&source(
        "orders",
        "place",
        "../.well-known/demo-configuration",
    ));
    assert_eq!(found.len(), 1, "{found:?}");
    let (location, message) = &found[0];
    assert_eq!(location, "view.demo.orders.Open.naming.wire");
    assert!(message.contains("demo.orders.Open"), "{message}");
    assert!(
        message.contains("\"../.well-known/demo-configuration\""),
        "{message}"
    );
}

#[test]
fn a_command_wire_name_holding_a_slash_is_refused_by_name() {
    let found = refusals(&source("orders", "place/now", "open"));
    assert_eq!(found.len(), 1, "{found:?}");
    let (location, message) = &found[0];
    assert_eq!(location, "command.demo.orders.Place.naming.wire");
    assert!(message.contains("demo.orders.Place"), "{message}");
    assert!(message.contains("\"place/now\""), "{message}");
}

#[test]
fn a_domain_wire_name_that_is_a_dot_segment_is_refused_by_name() {
    for dots in [".", ".."] {
        let found = refusals(&source(dots, "place", "open"));
        assert_eq!(found.len(), 1, "{dots}: {found:?}");
        let (location, message) = &found[0];
        assert_eq!(location, "domain.demo.orders.naming.wire");
        assert!(message.contains("demo.orders"), "{message}");
        assert!(message.contains(&format!("{dots:?}")), "{message}");
    }
}

#[test]
fn every_refused_declaration_is_reported_in_one_run() {
    let found = refusals(&source("/", "..", "."));
    let locations: Vec<&str> = found.iter().map(|(at, _)| at.as_str()).collect();
    assert_eq!(
        locations,
        [
            "domain.demo.orders.naming.wire",
            "command.demo.orders.Place.naming.wire",
            "view.demo.orders.Open.naming.wire",
        ]
    );
}
