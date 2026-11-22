//! A row the guard accepts beside the named one (a companion, beyond10x/ess#270) is searched for
//! once per distinct search, not once per pass.
//!
//! The companion search runs three passes, each steered toward values and taking rows that hold
//! another value in the copied fields. Where one field is copied and it has no ordered neighbour
//! (a flag, an enum variant), the three passes steer toward the same values and take the same
//! rows: each ran the same stored-row search again, and a branch copying a value every accepted
//! row shares (no companion exists) paid for it three times, on both sides of the named row, in
//! every arrangement of that branch.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{
    companion_work, first_rows_made, row_set_attempts, synthesize, Synthesis,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// An order copies the `vip` flag of the ticket it is guarded on, and only a ticket whose flag is
/// set is accepted: every companion holds the same flag, so none is found.
const MODEL: &str = "format: ess/18
system: shop
version: v1
domain: shop.order
summary: An order copies the flag of the ticket it is guarded on.
types:
  - {name: shop.order.TicketId, kind: newtype, of: Uuid}
  - {name: shop.order.OrderId, kind: newtype, of: Uuid}
entities:
  - name: shop.order.Ticket
    identity: {name: ticket_id, type: shop.order.TicketId}
    fields:
      - {name: vip, type: Boolean}
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.order.Order
    identity: {name: order_id, type: shop.order.OrderId}
    fields:
      - {name: ticket_id, type: shop.order.TicketId}
      - {name: vip, type: Boolean}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed]}
errors:
  - {name: shop.order.NoTicket, summary: No ticket carries the identity., fields: []}
  - {name: shop.order.NotVip, summary: The ticket is not a vip ticket., fields: []}
events:
  - name: shop.order.TicketOpened
    fields:
      - {name: ticket_id, type: shop.order.TicketId}
  - name: shop.order.Placed
    fields:
      - {name: order_id, type: shop.order.OrderId}
actors:
  - name: shop.order.Clerk
    may: [shop.order.OpenTicket, shop.order.Place]
commands:
  - name: shop.order.OpenTicket
    input:
      - {name: vip, type: Boolean}
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: shop.order.Ticket
        instance: ticket_id
        emits: [shop.order.TicketOpened]
        payload:
          shop.order.TicketOpened: {ticket_id: {generated: true}}
        sets:
          vip: input.vip
          label: input.label
  - name: shop.order.Place
    input:
      - {name: ticket_id, type: shop.order.TicketId}
    outcomes:
      - name: no-ticket
        when_related: {via: input.ticket_id, exists: false}
        error: shop.order.NoTicket
      - name: not-vip
        when_related: {via: input.ticket_id, predicate: vip == false}
        error: shop.order.NotVip
      - name: placed
        creates: shop.order.Order
        instance: order_id
        emits: [shop.order.Placed]
        payload:
          shop.order.Placed: {order_id: {generated: true}}
        sets:
          ticket_id: input.ticket_id
          vip: {related: {via: input.ticket_id, field: vip}}
views:
  - name: shop.order.Orders
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: shop.order.OrderId}
      - {name: ticket_id, type: shop.order.TicketId}
      - {name: vip, type: Boolean}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("shop.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn shown(synthesis: &Synthesis) -> String {
    format!("{:#?}\n{:#?}", synthesis.suite, synthesis.refusals)
}

/// Each companion search runs one stored-row search: the three passes are alike here. Before the
/// fix each ran three (the `fresh` run below still does: `searched == 3 * asked` there).
#[test]
fn a_companion_pass_alike_to_one_that_found_nothing_is_not_searched_again() {
    let ir = ir(MODEL);
    let (kept, asked, searched) = companion_work(false, || synthesize(&ir));
    let (fresh, fresh_asked, fresh_searched) = companion_work(true, || synthesize(&ir));
    assert!(
        asked > 0,
        "no companion was searched for: the model no longer has the shape"
    );
    assert_eq!(asked, fresh_asked, "companions asked for");
    assert_eq!(
        fresh_searched,
        3 * fresh_asked,
        "every pass searched again: the passes are not alike in this model"
    );
    assert_eq!(
        searched, asked,
        "{searched} stored-row searches for {asked} companions, more than one each"
    );
    assert_eq!(shown(&kept), shown(&fresh), "the suite changed");
}

/// A wire joins two ports; `Connect` reads both through `when_related:` beside a row set over the
/// wires leaving its source port (`ess/32`).
const WIRING: &str =
    include_str!("../../../specify/ess-domain/tests/fixtures/identity-beside-row-set.yaml");

const SOURCE_TAKEN: &str = "      - name: source-taken
        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          exists: true
        error: wiring.board.SourceTaken
";

/// The row-set refusal moved ahead of every branch over a named port.
fn taken_first() -> String {
    let without = WIRING.replacen(SOURCE_TAKEN, "", 1);
    assert_ne!(without, WIRING, "the refusal is in the model");
    let moved = without.replacen(
        "      - name: no-source\n",
        &format!("{SOURCE_TAKEN}      - name: no-source\n"),
        1,
    );
    assert_ne!(moved, without, "the first branch is in the model");
    moved
}

/// The linked arrangement of a row-set branch, tried among the attempts beside named rows and not
/// standing selected there, is not tried again after the plain one: before, every such branch
/// arranged it twice (the `fresh` runs below still do, so they ask for more arrangements).
#[test]
fn a_linked_row_set_attempt_that_ran_beside_named_rows_is_not_tried_again() {
    let (mut asked, mut fresh_asked) = (0, 0);
    for text in [WIRING.to_owned(), taken_first()] {
        let ir = ir(&text);
        let (kept, kept_asked) = row_set_attempts(false, || synthesize(&ir));
        let (fresh, again) = row_set_attempts(true, || synthesize(&ir));
        assert_eq!(shown(&kept), shown(&fresh), "the suite changed");
        asked += kept_asked;
        fresh_asked += again;
    }
    assert!(
        asked < fresh_asked,
        "{asked} row-set arrangements asked for, {fresh_asked} when the linked one is tried again"
    );
}

/// The plain row of the ticket a decoy or companion of `Place` is steered by, asked for at the
/// same distinction by every arrangement of the command, is arranged once per context: before,
/// every ask arranged it (the `fresh` run below still does: `made == asked` there).
#[test]
fn a_further_row_asked_again_in_the_same_context_is_arranged_once() {
    let ir = ir(MODEL);
    let (kept, asked, made) = first_rows_made(false, || synthesize(&ir));
    let (fresh, fresh_asked, fresh_made) = first_rows_made(true, || synthesize(&ir));
    assert_eq!(asked, fresh_asked, "further rows asked for");
    assert_eq!(fresh_made, fresh_asked, "every row arranged afresh");
    assert!(
        made < asked,
        "{made} further rows arranged for {asked} asked: none answered from memory"
    );
    assert_eq!(shown(&kept), shown(&fresh), "the suite changed");
}
