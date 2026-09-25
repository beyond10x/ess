//! Adversarial case for unit A1 (ess#94): the byte order given to every conformance fact store
//! reaches a declared `Timestamp` wherever that store does not know the declared types.
//!
//! `input::new_facts` turns byte ordering on for entity setup, type invariants, view rows and
//! selections, and none of those stores answers `orders_as_instant`. So a `Timestamp` invariant is
//! decided by the spelling of the instant rather than by the instant — which a command guard over
//! the same type, read through `InputFacts`, is not. Before the unit the same ordering was
//! `Unknown`, and setup refused it.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::entity::StateName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = r#"
format: ess/1
system: book
version: v1
domain: book.slot
entities:
  - name: book.slot.Booking
    identity: {name: booking_id, type: Uuid}
    fields:
      - {name: due, type: Timestamp}
    invariants:
      - due >= "2020-01-01T00:00:00Z"
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - name: book.slot.NotOpen
    summary: The booking is not open.
commands:
  - name: book.slot.CloseBooking
    input:
      - {name: booking_id, type: Uuid}
    outcomes:
      - name: closed
        moves: book.slot.Booking.close
        instance: booking_id
        emits: [book.slot.BookingClosed]
      - name: wrong-state
        wrong_state: true
        error: book.slot.NotOpen
events:
  - name: book.slot.BookingClosed
    fields:
      - {name: booking_id, type: Uuid}
"#;

fn compiled() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the fixture is well formed");
    let specification = Specification::assemble([(Source::new("book.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn setup(ir: &EssIr, due: &str) -> Result<(), String> {
    let entity = serde_json::from_str(r#""book.slot.Booking""#).expect("an entity ref");
    ess_conformance::input::validate_entity_setup(
        ir,
        &entity,
        &Node::Text("7b0a3c1e-5d2f-4a6b-9c8d-0e1f2a3b4c5d".to_owned()),
        &BTreeMap::from([("due".to_owned(), Node::Text(due.to_owned()))]),
        &StateName::new("Open").expect("a state"),
    )
}

/// `2020-01-01T00:30:00+01:00` is `2019-12-31T23:30:00Z`: half an hour *before* the bound. Its
/// bytes sort after `2020-01-01T00:00:00Z`, so a byte order admits it.
#[test]
fn a_timestamp_invariant_is_decided_by_the_instant_in_entity_setup_not_by_its_spelling() {
    let ir = compiled();
    assert!(
        setup(&ir, "2020-01-01T00:00:00Z").is_ok(),
        "the bound itself satisfies `due >= bound`"
    );
    assert!(
        setup(&ir, "2019-12-31T23:59:59Z").is_err(),
        "a UTC instant before the bound violates the invariant"
    );
    let offset = setup(&ir, "2020-01-01T00:30:00+01:00");
    assert!(
        offset.is_err(),
        "`2020-01-01T00:30:00+01:00` is 2019-12-31T23:30:00Z, before the bound, and entity setup \
         admitted it as satisfying `due >= \"2020-01-01T00:00:00Z\"`: {offset:?}"
    );
}
