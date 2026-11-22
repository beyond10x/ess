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
use ess_conformance::synthesize::{companion_work, synthesize, Synthesis};
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
