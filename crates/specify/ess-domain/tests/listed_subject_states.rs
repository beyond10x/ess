//! `when_subject_state:` names one state or a list of them, on any subject branch
//! (beyond10x/ess#456). The list form (`ess/18`) is the idiom for a record kept unchanged in any of
//! several states; `{in: [...]}` is not a second spelling of it.
use std::path::Path;

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const HEADING: &str = "### Keep a record unchanged in any of several states";

/// A minimal order whose `ShipOrder` outcomes are `outcomes`, a block of the guide indented as a
/// list item of `outcomes:`.
fn model(outcomes: &str) -> String {
    let mut indented = String::new();
    for line in outcomes.lines() {
        indented.push_str("      ");
        indented.push_str(line);
        indented.push('\n');
    }
    format!(
        "format: ess/18
system: demo
version: v1
domain: demo.ship
types:
  - {{name: demo.ship.OrderId, kind: newtype, of: Uuid}}
entities:
  - name: demo.ship.Order
    identity: {{name: order_id, type: demo.ship.OrderId}}
    fields: []
    lifecycle:
      initial: Placed
      states: [Placed, Shipped, Delivered, Cancelled]
      terminal: [Delivered, Cancelled]
      transitions:
        - {{name: ship, from: [Placed], to: Shipped}}
        - {{name: deliver, from: [Shipped], to: Delivered}}
        - {{name: cancel, from: [Placed, Shipped], to: Cancelled}}
errors:
  - {{name: demo.ship.Gone, fields: []}}
events:
  - name: demo.ship.OrderPlaced
    fields: [{{name: order_id, type: demo.ship.OrderId}}]
  - name: demo.ship.OrderShipped
    fields: [{{name: order_id, type: demo.ship.OrderId}}]
  - name: demo.ship.OrderDelivered
    fields: [{{name: order_id, type: demo.ship.OrderId}}]
  - name: demo.ship.OrderCancelled
    fields: [{{name: order_id, type: demo.ship.OrderId}}]
commands:
  - name: demo.ship.PlaceOrder
    input: []
    outcomes:
      - name: placed
        creates: demo.ship.Order
        instance: order_id
        emits: [demo.ship.OrderPlaced]
        payload: {{demo.ship.OrderPlaced: {{order_id: {{generated: true}}}}}}
  - name: demo.ship.DeliverOrder
    input: [{{name: order_id, type: demo.ship.OrderId}}]
    outcomes:
      - name: delivered
        moves: demo.ship.Order.deliver
        instance: order_id
        emits: [demo.ship.OrderDelivered]
        payload: {{demo.ship.OrderDelivered: {{order_id: input.order_id}}}}
  - name: demo.ship.CancelOrder
    input: [{{name: order_id, type: demo.ship.OrderId}}]
    outcomes:
      - name: cancelled
        moves: demo.ship.Order.cancel
        instance: order_id
        emits: [demo.ship.OrderCancelled]
        payload: {{demo.ship.OrderCancelled: {{order_id: input.order_id}}}}
  - name: demo.ship.ShipOrder
    input: [{{name: order_id, type: demo.ship.OrderId}}]
    outcomes:
{indented}"
    )
}

fn assembled(text: &str) -> Result<(), String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("model.yaml"), raw)])
        .map(|_| ())
        .map_err(|errors| errors.to_string())
}

/// Every fenced YAML block's body in `text`, in order.
fn yaml_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut open: Option<String> = None;
    for line in text.lines() {
        match &mut open {
            None if line.trim_start().starts_with("```yaml") => open = Some(String::new()),
            Some(body) if line.trim_start() == "```" => {
                blocks.push(std::mem::take(body));
                open = None;
            }
            Some(body) => {
                body.push_str(line);
                body.push('\n');
            }
            None => {}
        }
    }
    blocks
}

#[test]
fn listed_preserves_guide_example_is_present_and_valid() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/guides/specify/guards-and-predicates.md");
    let page = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let section_start = page
        .find("\n## Select an outcome from the held subject state\n")
        .expect("the guide has the held-state section");
    let section = &page[section_start + 1..];
    let section = &section[..section[3..]
        .find("\n## ")
        .map_or(section.len(), |at| at + 4)];
    let ship_order = section
        .find("error: demo.ship.Gone")
        .expect("the section shows the `ShipOrder` example");
    let heading = section
        .find(&format!("\n{HEADING}\n"))
        .unwrap_or_else(|| panic!("the held-state section has the heading `{HEADING}`"));
    assert!(
        heading > ship_order,
        "`{HEADING}` follows the `ShipOrder` example"
    );
    let subsection = &section[heading + 1..];
    let subsection = &subsection[..subsection[4..]
        .find("\n#")
        .map_or(subsection.len(), |at| at + 5)];
    let block = yaml_blocks(subsection)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("`{HEADING}` shows a YAML block:\n{subsection}"));
    for shape in ["preserves:", "when_subject_state: ["] {
        assert!(
            block.contains(shape),
            "the example carries `{shape}`:\n{block}"
        );
    }
    let spliced = model(&block);
    assembled(&spliced).unwrap_or_else(|error| {
        panic!("the example validates in a minimal order model: {error}\n{spliced}")
    });
}

#[test]
fn in_form_is_refused_with_the_list_hint() {
    let text = model(
        "- name: shipped\n  moves: demo.ship.Order.ship\n  instance: order_id\n  emits: [demo.ship.OrderShipped]\n  payload: {demo.ship.OrderShipped: {order_id: input.order_id}}\n\
         - name: kept\n  when_subject_state: {in: [Shipped, Delivered]}\n  preserves: demo.ship.Order\n  instance: order_id\n\
         - name: gone\n  when_subject_state: Cancelled\n  error: demo.ship.Gone\n",
    );
    let error = assembled(&text).expect_err("`{in: [...]}` is not a held-state guard");
    assert!(
        error.contains("`when_subject_state` is one state, such as `Shipped`, or a list of states"),
        "the refusal names both admitted spellings: {error}"
    );
}
