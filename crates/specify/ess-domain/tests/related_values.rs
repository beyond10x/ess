//! `{related: {via: <field>, field: <field>}}` (source format `ess/16`, beyond10x/ess#166,
//! `docs/design/value-expressions.md` E8): a payload or `sets:` value read from a field of the row
//! the subject (or the input) references.

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("shipping.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str, expected: &[&str]) {
    let error = spec(body)
        .err()
        .unwrap_or_else(|| panic!("must not compile:\n{body}"));
    for needle in expected {
        assert!(error.contains(needle), "expected {needle:?} in:\n{error}");
    }
}

/// The #166 repro. `{region}` is the source of `ShipmentDispatched.region`, `{pack}` extra lines of
/// the `packed` outcome, `{extra_types}` extra type declarations and `{relations}` the `Shipment`
/// entity's `relations:` block.
fn shipping(format: &str, region: &str, pack: &str, extra_types: &str, relations: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domains: [demo.shipping]
domain: demo.shipping
types:
  - {{name: demo.shipping.CustomerId, kind: newtype, of: String}}
  - {{name: demo.shipping.ShipmentId, kind: newtype, of: String}}
  - {{name: demo.shipping.Region, kind: newtype, of: String}}
{extra_types}
entities:
  - name: demo.shipping.Customer
    identity: {{name: customer_id, type: demo.shipping.CustomerId}}
    fields:
      - {{name: region, type: demo.shipping.Region}}
      - {{name: rank, type: Integer}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: demo.shipping.Shipment
    identity: {{name: shipment_id, type: demo.shipping.ShipmentId}}
    fields:
      - {{name: customer_id, type: demo.shipping.CustomerId}}
      - {{name: backup, type: Optional<demo.shipping.CustomerId>}}
      - {{name: region, type: Optional<demo.shipping.Region>}}
      - {{name: zone, type: demo.shipping.Region}}
{relations}
    lifecycle:
      initial: Packed
      states: [Packed, Dispatched]
      terminal: [Dispatched]
      transitions:
        - {{name: dispatch, from: [Packed], to: Dispatched}}
events:
  - name: demo.shipping.CustomerRegistered
    fields:
      - {{name: customer_id, type: demo.shipping.CustomerId}}
  - name: demo.shipping.ShipmentPacked
    fields:
      - {{name: shipment_id, type: demo.shipping.ShipmentId}}
      - {{name: region, type: demo.shipping.Region}}
  - name: demo.shipping.ShipmentDispatched
    fields:
      - {{name: shipment_id, type: demo.shipping.ShipmentId}}
      - {{name: region, type: demo.shipping.Region}}
actors:
  - name: demo.shipping.Clerk
    may: [demo.shipping.Register, demo.shipping.Pack, demo.shipping.Dispatch]
commands:
  - name: demo.shipping.Register
    input:
      - {{name: region, type: demo.shipping.Region}}
    outcomes:
      - name: registered
        creates: demo.shipping.Customer
        instance: customer_id
        emits: [demo.shipping.CustomerRegistered]
        payload:
          demo.shipping.CustomerRegistered: {{customer_id: {{generated: true}}}}
        sets:
          region: input.region
          rank: 1
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
{pack}
        sets:
          customer_id: input.customer_id
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
            region: {region}
"
    )
}

const PACKED_REGION: &str =
    "            region: {related: {via: input.customer_id, field: region}}";
const THE_REPRO: &str = "{related: {via: customer_id, field: region}}";

fn repro(format: &str) -> String {
    shipping(format, THE_REPRO, PACKED_REGION, "", "")
}

#[test]
fn issue_166_the_repro_validates_under_ess_16() {
    spec(&repro("ess/16")).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn a_related_source_needs_ess_16() {
    for format in ["ess/14", "ess/15"] {
        refused(
            &repro(format),
            &["unsupported_format_version", "ess/16", "{related: …}"],
        );
    }
}

/// The issue's repro as filed, with the wanted source written and the format raised.
const ISSUE_166: &str = "format: ess/16
system: demo
version: v1
domains: [demo.shipping]
domain: demo.shipping
types:
  - {name: demo.shipping.CustomerId, kind: newtype, of: String}
  - {name: demo.shipping.ShipmentId, kind: newtype, of: String}
  - {name: demo.shipping.Region, kind: newtype, of: String}
entities:
  - name: demo.shipping.Customer
    identity: {name: customer_id, type: demo.shipping.CustomerId}
    fields:
      - {name: region, type: demo.shipping.Region}
    lifecycle:
      initial: Active
      states: [Active, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Active], to: Closed}
  - name: demo.shipping.Shipment
    identity: {name: shipment_id, type: demo.shipping.ShipmentId}
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
    lifecycle:
      initial: Packed
      states: [Packed, Dispatched]
      terminal: [Dispatched]
      transitions:
        - {name: dispatch, from: [Packed], to: Dispatched}
events:
  - name: demo.shipping.CustomerRegistered
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
  - name: demo.shipping.CustomerClosed
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
  - name: demo.shipping.ShipmentPacked
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
  - name: demo.shipping.ShipmentDispatched
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
      - {name: region, type: demo.shipping.Region}
actors:
  - name: demo.shipping.Clerk
    may: [demo.shipping.Register, demo.shipping.CloseCustomer, demo.shipping.Pack, demo.shipping.Dispatch]
commands:
  - name: demo.shipping.Register
    input:
      - {name: region, type: demo.shipping.Region}
    outcomes:
      - name: registered
        creates: demo.shipping.Customer
        instance: customer_id
        emits: [demo.shipping.CustomerRegistered]
        payload:
          demo.shipping.CustomerRegistered: {customer_id: {generated: true}}
        sets:
          region: input.region
  - name: demo.shipping.CloseCustomer
    input:
      - {name: customer_id, type: demo.shipping.CustomerId}
    outcomes:
      - name: closed
        moves: demo.shipping.Customer.close
        instance: customer_id
        emits: [demo.shipping.CustomerClosed]
        payload:
          demo.shipping.CustomerClosed: {customer_id: input.customer_id}
  - name: demo.shipping.Pack
    input:
      - {name: customer_id, type: demo.shipping.CustomerId}
    outcomes:
      - name: packed
        creates: demo.shipping.Shipment
        instance: shipment_id
        emits: [demo.shipping.ShipmentPacked]
        payload:
          demo.shipping.ShipmentPacked: {shipment_id: {generated: true}}
        sets:
          customer_id: input.customer_id
  - name: demo.shipping.Dispatch
    input:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
    outcomes:
      - name: dispatched
        moves: demo.shipping.Shipment.dispatch
        instance: shipment_id
        emits: [demo.shipping.ShipmentDispatched]
        payload:
          demo.shipping.ShipmentDispatched:
            shipment_id: input.shipment_id
            region: {related: {via: customer_id, field: region}}
";

#[test]
fn issue_166_the_filed_repro_validates_with_the_related_source() {
    spec(ISSUE_166).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn a_related_source_is_read_only_in_its_own_shape() {
    for (source, needle) in [
        ("{related: {via: customer_id}}", "`{related: …}` takes"),
        ("{related: {field: region}}", "`{related: …}` takes"),
        (
            "{related: {via: customer_id, field: region, entity: demo.shipping.Customer}}",
            "`{related: …}` takes",
        ),
        ("{related: customer_id}", "`{related: …}` takes"),
        (
            "{related: {via: customer_id.region, field: region}}",
            "one field",
        ),
        ("{related: {via: customer_id, field: a.b}}", "one field"),
        (
            "{related: {via: customer_id, field: region}, generated: true}",
            "is written alone",
        ),
    ] {
        refused(
            &shipping("ess/16", source, PACKED_REGION, "", ""),
            &[needle],
        );
    }
}

#[test]
fn the_via_field_must_be_the_subjects_or_the_inputs_and_name_one_entity() {
    // Not a field of the shipment.
    refused(
        &shipping(
            "ess/16",
            "{related: {via: owner_id, field: region}}",
            PACKED_REGION,
            "",
            "",
        ),
        &["undeclared_reference", "`owner_id` is not a field"],
    );
    // Not an input of the command.
    refused(
        &shipping(
            "ess/16",
            "{related: {via: input.owner_id, field: region}}",
            PACKED_REGION,
            "",
            "",
        ),
        &["undeclared_reference", "`owner_id` is not an input"],
    );
    // A subject field read on `creates:` that the branch does not set from its input: no row before.
    refused(
        &shipping(
            "ess/16",
            THE_REPRO,
            "            region: {related: {via: backup, field: region}}",
            "",
            "",
        ),
        &["conflicting_declaration", "no row before it"],
    );
    // An identity that may be absent.
    refused(
        &shipping(
            "ess/16",
            "{related: {via: backup, field: region}}",
            PACKED_REGION,
            "",
            "",
        ),
        &["type_mismatch", "`backup`", "Optional"],
    );
    // A field whose type is no entity's identity.
    refused(
        &shipping(
            "ess/16",
            "{related: {via: zone, field: region}}",
            PACKED_REGION,
            "",
            "",
        ),
        &["type_mismatch", "no entity's identity"],
    );
}

#[test]
fn the_read_field_must_be_held_by_the_referenced_entity_and_fit_the_target() {
    refused(
        &shipping(
            "ess/16",
            "{related: {via: customer_id, field: nope}}",
            PACKED_REGION,
            "",
            "",
        ),
        &[
            "undeclared_reference",
            "`nope` is not a field of `demo.shipping.Customer`",
        ],
    );
    refused(
        &shipping(
            "ess/16",
            "{related: {via: customer_id, field: rank}}",
            PACKED_REGION,
            "",
            "",
        ),
        &["type_mismatch", "Customer.rank`"],
    );
}

#[test]
fn a_related_source_is_admitted_in_sets() {
    let body = shipping(
        "ess/16",
        THE_REPRO,
        PACKED_REGION,
        "",
        "",
    )
    .replace(
        "            region: {related: {via: customer_id, field: region}}\n",
        "            region: {related: {via: customer_id, field: region}}\n        sets:\n          region: {related: {via: customer_id, field: region}}\n",
    );
    spec(&body).unwrap_or_else(|error| panic!("{error}\n{body}"));
}

/// A second entity identified by `CustomerId` makes the type alone ambiguous; a `references`
/// relation carried by the field says which one it names.
const TWIN: &str = "  - name: demo.shipping.Twin
    identity: {name: customer_id, type: demo.shipping.CustomerId}
    fields:
      - {name: region, type: demo.shipping.Region}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}";

#[test]
fn an_ambiguous_identity_type_is_refused_unless_a_references_relation_names_the_entity() {
    // `Pack` emits a generated region here, so only `Dispatch`'s subject-field read is in question.
    let twin = |pack: &str, relations: &str| {
        shipping("ess/16", THE_REPRO, pack, "", relations).replace(
            "  - name: demo.shipping.Shipment\n",
            &format!("{TWIN}\n  - name: demo.shipping.Shipment\n"),
        )
    };
    let generated = "            region: {generated: true}";
    refused(
        &twin(generated, ""),
        &[
            "conflicting_declaration",
            "demo.shipping.Customer",
            "demo.shipping.Twin",
            "references",
        ],
    );
    let related = "    relations:
      - {name: customer, kind: references, target: demo.shipping.Customer, cardinality: one, via: customer_id}";
    let body = twin(generated, related);
    spec(&body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    // An input read is settled by the relation on the subject field the branch sets from it
    // (`Pack` sets `customer_id: input.customer_id`), and refused without one.
    let body = twin(PACKED_REGION, related);
    spec(&body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    refused(
        &twin(PACKED_REGION, ""),
        &["conflicting_declaration", "demo.shipping.Twin"],
    );
}

/// Below `ess/16` the exact `{related: {via, field}}` shape is the nested mapping it always was:
/// a struct field `related` whose own fields `via` and `field` take the texts written.
#[test]
fn below_ess_16_the_related_shape_fills_a_struct_as_before() {
    let body = "format: ess/15
system: demo
version: v1
domain: demo.links
types:
  - {name: demo.links.LinkId, kind: newtype, of: String}
  - name: demo.links.Pair
    kind: struct
    fields:
      - {name: via, type: String}
      - {name: field, type: String}
  - name: demo.links.Meta
    kind: struct
    fields:
      - {name: related, type: demo.links.Pair}
entities:
  - name: demo.links.Link
    identity: {name: link_id, type: demo.links.LinkId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.links.LinkOpened
    fields:
      - {name: link_id, type: demo.links.LinkId}
      - {name: meta, type: demo.links.Meta}
actors:
  - {name: demo.links.Clerk, may: [demo.links.Open]}
commands:
  - name: demo.links.Open
    outcomes:
      - name: opened
        creates: demo.links.Link
        instance: link_id
        emits: [demo.links.LinkOpened]
        payload:
          demo.links.LinkOpened:
            link_id: {generated: true}
            meta: {related: {via: customer_id, field: region}}
";
    let spec = spec(body).unwrap_or_else(|error| panic!("{error}"));
    let command = spec.commands().values().next().expect("one command");
    let meta = command.outcomes[0]
        .payload
        .values()
        .next()
        .and_then(|fields| fields.get("meta"))
        .expect("meta is filled");
    assert!(
        matches!(meta, ess_domain::command::PayloadSource::Struct { .. }),
        "a nested mapping: {meta:?}"
    );
}
