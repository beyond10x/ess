//! Adversary, pass 2, on `{related: {via: <field>, field: <field>}}` (ess/16, beyond10x/ess#166,
//! `docs/design/value-expressions.md` E8): what adding `related` to the source keywords does to
//! documents written before `ess/16`, and whether the refusal's hint names a remedy that validates.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("links.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// An `ess/15` document whose event carries a struct `Meta` with one field, `related`, itself a
/// struct, filled by a nested mapping — valid before this unit, where `related` was no source
/// keyword and `{related: {count: 1}}` was a nested mapping for `Meta`.
const ESS_15_STRUCT_FIELD_NAMED_RELATED: &str = "format: ess/15
system: demo
version: v1
domain: demo.links
types:
  - {name: demo.links.LinkId, kind: newtype, of: String}
  - name: demo.links.Counted
    kind: struct
    fields:
      - {name: count, type: Integer}
  - name: demo.links.Meta
    kind: struct
    fields:
      - {name: related, type: demo.links.Counted}
entities:
  - name: demo.links.Link
    identity: {name: link_id, type: demo.links.LinkId}
    fields: []
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.links.LinkOpened
    fields:
      - {name: link_id, type: demo.links.LinkId}
      - {name: meta, type: demo.links.Meta}
actors:
  - name: demo.links.Clerk
    may: [demo.links.Open]
commands:
  - name: demo.links.Open
    input: []
    outcomes:
      - name: opened
        creates: demo.links.Link
        instance: link_id
        emits: [demo.links.LinkOpened]
        payload:
          demo.links.LinkOpened:
            link_id: {generated: true}
            meta: {related: {count: 1}}
";

/// `related` became a source keyword in every format, not only from `ess/16`, so an `ess/15`
/// document that was valid before the unit no longer parses. The design records the trade for the
/// new keyword (`value-expressions.md` E8, "Spelling"), not a retroactive break of earlier formats.
#[test]
fn adversary_related_an_ess_15_struct_field_named_related_still_parses() {
    if let Err(error) = spec(ESS_15_STRUCT_FIELD_NAMED_RELATED) {
        panic!("an ess/15 document valid before ess/16 existed is refused:\n{error}");
    }
}

/// Two entities identified by `CustomerId`, and a `creates:` branch reading
/// `{related: {via: input.customer_id, …}}`. `{shipment_relations}` is the shipment's relations
/// block and `{via}` the `via:` written.
fn creating(shipment_relations: &str, via: &str) -> String {
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
{shipment_relations}
    lifecycle: {{initial: Packed, states: [Packed], terminal: [Packed]}}
events:
  - name: demo.shipping.ShipmentPacked
    fields:
      - {{name: shipment_id, type: demo.shipping.ShipmentId}}
      - {{name: region, type: demo.shipping.Region}}
actors:
  - name: demo.shipping.Clerk
    may: [demo.shipping.Pack]
commands:
  - name: demo.shipping.Pack
    input:
      - {{name: customer_id, type: demo.shipping.CustomerId}}
    outcomes:
      - name: packed
        creates: demo.shipping.Shipment
        instance: shipment_id
        emits: [demo.shipping.ShipmentPacked]
        payload:
          demo.shipping.ShipmentPacked:
            shipment_id: {{generated: true}}
            region: {{related: {{via: {via}, field: region}}}}
        sets:
          customer_id: input.customer_id
"
    )
}

/// On a `creates:` branch the ambiguous-input refusal's hint says to "read the identity through a
/// field of the subject that a `references` relation carries". Doing exactly that is refused: a
/// `creates:` branch has no existing subject to read. The hint names a remedy that cannot validate
/// here — the defect pass 1 found for the subject hint, still standing for the input one.
#[test]
fn adversary_related_the_input_hint_on_a_creating_branch_names_a_remedy_that_validates() {
    let error = spec(&creating("", "input.customer_id"))
        .expect_err("an input of a type two entities share is refused");
    assert!(
        error.contains("read the identity through a field of the subject"),
        "the hint under test is the one given:\n{error}"
    );
    let remedy = creating(
        "    relations:
      - {name: customer, kind: references, target: demo.shipping.Customer, cardinality: one, via: customer_id}",
        "customer_id",
    );
    if let Err(refused) = spec(&remedy) {
        panic!("the hint's own remedy is refused:\n{refused}\n\nfirst refusal:\n{error}");
    }
}
