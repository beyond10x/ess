//! Adversary, pass 1, on `{related: {via: <field>, field: <field>}}` (ess/16, beyond10x/ess#166,
//! `docs/design/value-expressions.md` E8): which entity `via` names.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("shipping.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// Two entities identified by `CustomerId` (a customer and its profile), and a shipment whose
/// `customer_id` the customer declares it `owns`. `{relations}` is the `Shipment` entity's block.
fn document(relations: &str) -> String {
    format!(
        "format: ess/16
system: demo
version: v1
domain: demo.shipping
types:
  - {{name: demo.shipping.CustomerId, kind: newtype, of: String}}
  - {{name: demo.shipping.ShipmentId, kind: newtype, of: String}}
  - {{name: demo.shipping.Region, kind: newtype, of: String}}
entities:
  - name: demo.shipping.Customer
    identity: {{name: customer_id, type: demo.shipping.CustomerId}}
    fields:
      - {{name: region, type: demo.shipping.Region}}
    relations:
      - {{name: shipments, kind: owns, target: demo.shipping.Shipment, cardinality: many, via: customer_id}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: demo.shipping.Profile
    identity: {{name: customer_id, type: demo.shipping.CustomerId}}
    fields:
      - {{name: region, type: demo.shipping.Region}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: demo.shipping.Shipment
    identity: {{name: shipment_id, type: demo.shipping.ShipmentId}}
    fields:
      - {{name: customer_id, type: demo.shipping.CustomerId}}
{relations}
    lifecycle:
      initial: Packed
      states: [Packed, Dispatched]
      terminal: [Dispatched]
      transitions:
        - {{name: dispatch, from: [Packed], to: Dispatched}}
events:
  - name: demo.shipping.ShipmentDispatched
    fields:
      - {{name: shipment_id, type: demo.shipping.ShipmentId}}
      - {{name: region, type: demo.shipping.Region}}
actors:
  - name: demo.shipping.Clerk
    may: [demo.shipping.Dispatch]
commands:
  - name: demo.shipping.Dispatch
    input:
      - {{name: shipment_id, type: demo.shipping.ShipmentId}}
    outcomes:
      - name: dispatched
        moves: demo.shipping.Shipment.dispatch
        instance: shipment_id
        emits: [demo.shipping.ShipmentDispatched]
        payload:
          demo.shipping.ShipmentDispatched:
            shipment_id: input.shipment_id
            region: {{related: {{via: customer_id, field: region}}}}
"
    )
}

/// The `owns` relation already says which entity `Shipment.customer_id` names, and the hint's own
/// remedy — a `references` relation on the same field — is refused because a field carries one
/// relation. So a shipment owned by a customer can never read its customer's region.
#[test]
fn adversary_related_an_owns_relation_carried_by_via_names_the_entity() {
    let body = document("");
    if let Err(error) = spec(&body) {
        let remedy = document(
            "    relations:
      - {name: customer, kind: references, target: demo.shipping.Customer, cardinality: one, via: customer_id}",
        );
        panic!(
            "refused although `owns` names the entity:\n{error}\n\nand the hint's remedy:\n{}",
            spec(&remedy).err().unwrap_or_else(|| "accepted".to_owned())
        );
    }
}
