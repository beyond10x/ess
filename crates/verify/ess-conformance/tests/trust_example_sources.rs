//! Observable example facts have explicit model sources, independent of target counters.
use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{
    execute_generating, Externals, Generated, GeneratedSlot, Store,
};
use ess_conformance::reference::Billing;
use ess_conformance::target::{
    ConformanceTarget, Deadline, SemanticCommandRequest, SemanticViewRequest,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};

fn model(example: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(example);
    let mut pending = vec![base.clone()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|value| value == "yaml") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut sources = SourceMap::new();
    let mut files = Vec::new();
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let text = std::fs::read_to_string(path).unwrap();
        files.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        sources.insert(label, text);
    }
    compile(&Specification::assemble(files).unwrap(), &sources).unwrap()
}

#[test]
fn oracle_creation_stores_the_contact_its_views_publish() {
    let ir = model("oracle-fixture");
    let input = BTreeMap::from([
        ("contact".into(), Node::Text("actual@example.test".into())),
        (
            "alternate_contact".into(),
            Node::Text("decoy@example.test".into()),
        ),
        ("weight_grams".into(), Node::Number(1_i64.into())),
    ]);
    // Supply legacy undetermined event slots, without claiming they determine stored entity fields.
    let event: ess_domain::name::QualifiedName = "oracle.order.OrderPlaced".parse().unwrap();
    let generated = Generated::Given(BTreeMap::from([
        (
            GeneratedSlot::new(event.clone(), "order_id"),
            Node::Text("00000000-0000-4000-8000-000000000001".into()),
        ),
        (
            GeneratedSlot::new(event.clone(), "contact"),
            input["contact"].clone(),
        ),
        (
            GeneratedSlot::new(event, "alternate_contact"),
            input["alternate_contact"].clone(),
        ),
    ]));
    let steps = execute_generating(
        &ir,
        &Store::default(),
        &"oracle.order.PlaceOrder".parse().unwrap(),
        &input,
        &Externals::Open,
        &generated,
    )
    .unwrap();
    assert_eq!(steps.len(), 1);
    let (_, _, held) = steps[0].next.instances().next().unwrap();
    assert_eq!(held.fields.get("contact"), input.get("contact"));
    assert_ne!(held.fields.get("contact"), input.get("alternate_contact"));
}

#[test]
fn billing_orders_by_supplied_issuance_time_even_when_commands_arrive_in_reverse_order() {
    let target = Billing::new();
    let correlation = CorrelationId::new("explicit-time").unwrap();
    let mut identities = Vec::new();
    // The first command carries the later timestamp: a hidden target counter gets this wrong.
    for at in ["2026-01-05T09:00:03Z", "2026-01-05T10:00:01+02:00"] {
        let created = target.execute_command(SemanticCommandRequest {
            command: "billing.invoice.CreateInvoice".parse().unwrap(), actor: Some("billing.invoice.Customer".parse().unwrap()), caller: None,
            input: serde_json::from_value(serde_json::json!({"account_id":"3f1d5b7e-0000-4000-8000-000000000001","customer_email":"test@example.test","amount":{"amount":1,"currency":"EUR"}})).unwrap(), correlation: correlation.clone(),
        }).unwrap();
        let id = created.direct_events[0].payload["invoice_id"].clone();
        target
            .execute_command(SemanticCommandRequest {
                command: "billing.invoice.IssueInvoice".parse().unwrap(),
                actor: None,
                caller: None,
                input: BTreeMap::from([
                    ("invoice_id".into(), id.clone()),
                    ("issued_at".into(), Node::Text(at.into())),
                ]),
                correlation: correlation.clone(),
            })
            .unwrap();
        identities.push(id);
    }
    let rows = target
        .query_view(SemanticViewRequest {
            view: "billing.invoice.OutstandingInvoices".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(u64::MAX)),
        })
        .unwrap()
        .rows;
    assert_eq!(rows.len(), 2);
    for (index, at) in ["2026-01-05T09:00:03Z", "2026-01-05T10:00:01+02:00"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(rows[index]["issued_at"], Node::Text(at.into()));
        assert_eq!(rows[index]["invoice_id"], identities[index]);
    }
}
