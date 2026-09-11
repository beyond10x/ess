//! Admission of bounded projections over declared event payloads.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::fmt::Write;

fn model(format: u32, payload: &str, target: &str, path: &str) -> String {
    format!(
        r"format: ess/{format}
system: example
version: v1
domain: example.flow
types:
  - name: example.flow.Body
    kind: struct
    fields:
      - name: status
        type: String
  - name: example.flow.Wrapper
    kind: struct
    fields:
      - name: body
        type: example.flow.Body
events:
  - name: example.flow.Arrived
    fields:
      - name: data
        type: {payload}
  - name: example.flow.Done
    fields:
      - name: result
        type: {target}
commands:
  - name: example.flow.Receive
    input:
      - name: result
        type: {target}
    outcomes:
      - name: accepted
        emits: [example.flow.Done]
bindings:
  - id: receive
    when:
      event: example.flow.Arrived
    invoke:
      command: example.flow.Receive
    mapping:
      result: {path}
    delivery: at_most_once
    on_failure: drop
"
    )
}

fn admit(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("accessor.yaml"), raw)])
        .map_err(|errors| errors.to_string())
}

#[test]
fn existing_flat_field_stays_in_its_original_format() {
    let spec = admit(&model(1, "String", "String", "event.data")).expect("flat mapping");
    let json = serde_json::to_value(spec.bindings().values().next().unwrap()).unwrap();
    assert_eq!(json["mapping"]["result"]["kind"], "event_field");
    assert_eq!(json["mapping"]["result"]["field"], "data");
}

#[test]
fn persisted_plans_reject_forged_shapes_cycles_and_excess_nodes_before_admission() {
    use ess_domain::accessor::{AccessorPlan, RawAccessorPlan, MAX_NODES};
    let spec = admit(&model(
        3,
        "example.flow.Body",
        "String",
        "event.data.status",
    ))
    .unwrap();
    let original = serde_json::to_value(plan(&spec, "event.data.status")).unwrap();
    for key in ["shape", "depth", "operation"] {
        let mut value = original.clone();
        value["nodes"][0][key] = match key {
            "shape" => serde_json::json!({"kind":"sequence"}),
            "depth" => serde_json::json!(127),
            _ => serde_json::json!({"kind":"optional", "next":0}),
        };
        assert!(
            serde_json::from_value::<AccessorPlan>(value).is_err(),
            "{key}"
        );
    }
    let mut value = original;
    value["nodes"] = serde_json::Value::Array(vec![value["nodes"][0].clone(); MAX_NODES + 1]);
    assert!(serde_json::from_value::<RawAccessorPlan>(value)
        .unwrap_err()
        .to_string()
        .contains("AccessorResource"));
}

#[test]
fn plan_and_model_byte_accounts_refuse_finite_growth() {
    use ess_domain::accessor::{Account, MAX_BYTES};
    let spec = admit(&model(
        3,
        "example.flow.Body",
        "String",
        "event.data.status",
    ))
    .unwrap();
    let mut original = plan(&spec, "event.data.status");
    original.root.naming.summary = Some("x".repeat(1_000));
    let mut oversized = original.clone();
    oversized.root.naming.summary = Some("x".repeat(MAX_BYTES));
    assert!(oversized
        .bytes()
        .unwrap_err()
        .to_string()
        .contains("accessor_resource"));
    let mut account = Account::default();
    let error = (0..=65_536)
        .find_map(|_| account.add(&original).err())
        .expect("bounded aggregate");
    assert!(error.to_string().contains("model bytes"), "{error}");
}

#[test]
fn two_and_three_declared_segments_are_admitted_under_ess3() {
    for (payload, path) in [
        ("example.flow.Body", "event.data.status"),
        ("example.flow.Wrapper", "event.data.body.status"),
    ] {
        let spec = admit(&model(3, payload, "String", path)).expect("declared accessor");
        let json = serde_json::to_value(spec.bindings().values().next().unwrap()).unwrap();
        assert_eq!(json["mapping"]["result"]["kind"], "event_accessor");
    }
}

#[test]
fn traversal_presence_requires_an_optional_target() {
    let text = model(
        3,
        "Optional<example.flow.Body>",
        "Optional<String>",
        "event.data.status",
    );
    admit(&text).expect("optional target represents unavailable traversal");
    let required = text.replace("type: Optional<String>", "type: String");
    let error = admit(&required).expect_err("partial projection into required target");
    assert!(error.contains("partial_accessor"), "{error}");
}

#[test]
fn source_typos_and_unsupported_depth_remain_distinct() {
    let typo = admit(&model(3, "example.flow.Body", "String", "event.data.typo"))
        .expect_err("missing declared member");
    assert!(typo.contains("unobservable_fact"), "{typo}");
    let deep = admit(&model(
        3,
        "example.flow.Wrapper",
        "String",
        "event.data.body.status.extra",
    ))
    .expect_err("four segments");
    assert!(deep.contains("accessor_depth"), "{deep}");
}

#[test]
fn older_source_formats_do_not_reinterpret_accessor_syntax() {
    for format in [1, 2] {
        let error = admit(&model(
            format,
            "example.flow.Body",
            "String",
            "event.data.status",
        ))
        .expect_err("requires source format 3");
        assert!(error.contains("ess/3"), "{error}");
    }
}

fn plan(spec: &Specification, path: &str) -> ess_domain::accessor::AccessorPlan {
    let event = spec
        .events()
        .get(&"example.flow.Arrived".parse().unwrap())
        .unwrap();
    let segments = path
        .strip_prefix("event.")
        .unwrap()
        .split('.')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    ess_domain::accessor::resolve(
        event,
        &segments,
        &spec.system().types,
        "binding.receive.mapping.result",
    )
    .unwrap()
}

#[test]
fn a_union_diamond_retains_shared_nodes_instead_of_expanding_branches() {
    let mut declarations = String::new();
    for depth in 0..20 {
        let next = if depth == 19 {
            "example.flow.Body".to_owned()
        } else {
            format!("example.flow.U{}", depth + 1)
        };
        write!(declarations, "  - name: example.flow.U{depth}\n    kind: union\n    tag: kind\n    variants:\n      first: {next}\n      second: {next}\n").unwrap();
    }
    let text = model(
        3,
        "example.flow.U0",
        "Optional<String>",
        "event.data.status",
    )
    .replace("events:\n", &format!("{declarations}events:\n"));
    let spec = admit(&text).unwrap();
    let plan = plan(&spec, "event.data.status");
    assert_eq!(plan.nodes.len(), 22);
    assert!(plan.bytes().unwrap() < 20_000);
    assert!(plan.may_miss());
    let json = serde_json::to_string(&plan).unwrap();
    let raw = serde_json::from_str(&json).unwrap();
    assert_eq!(
        ess_domain::accessor::AccessorPlan::admit(raw).unwrap(),
        plan
    );
}

#[test]
fn unavailable_union_branch_is_distinct_from_missing_everywhere() {
    let text = model(3, "example.flow.Choice", "Optional<String>", "event.data.status")
        .replace("events:\n", "  - name: example.flow.Choice\n    kind: union\n    tag: kind\n    variants:\n      ready: example.flow.Body\n      gone: String\nevents:\n");
    let spec = admit(&text).unwrap();
    assert!(plan(&spec, "event.data.status")
        .nodes
        .iter()
        .any(|n| matches!(n.operation, ess_domain::accessor::Operation::Missing)));
    let error = admit(&text.replace("event.data.status", "event.data.typo")).unwrap_err();
    assert!(error.contains("unobservable_fact"), "{error}");
}

#[test]
fn nominal_unwrap_and_wire_names_are_retained_as_distinct_operations() {
    let text = model(3, "example.flow.Wrapped", "String", "event.data.status")
        .replace("events:\n", "  - name: example.flow.Wrapped\n    kind: newtype\n    of: example.flow.Body\nevents:\n")
        .replacen("name: status\n        type: String", "name: status\n        type: String\n        wire: upstream_status", 1);
    let spec = admit(&text).unwrap();
    let plan = plan(&spec, "event.data.status");
    assert!(plan
        .nodes
        .iter()
        .any(|n| matches!(n.operation, ess_domain::accessor::Operation::Newtype { .. })));
    let field = plan
        .nodes
        .iter()
        .find_map(|n| match &n.operation {
            ess_domain::accessor::Operation::Field { field, .. } => Some(field),
            _ => None,
        })
        .unwrap();
    assert_eq!(field.name, "status");
    assert_eq!(field.naming.wire.as_deref(), Some("upstream_status"));
}

#[test]
fn whole_optional_leaf_uses_its_explicit_conversion_without_partiality() {
    let text = model(3, "example.flow.Body", "String", "event.data.status")
        .replacen("name: status\n        type: String", "name: status\n        type: Optional<String>", 1)
        + "conversions:\n  - from: Optional<String>\n    to: String\n    because: the host maps None to an empty string\n";
    let spec = admit(&text).unwrap();
    let plan = plan(&spec, "event.data.status");
    assert!(!plan.may_miss());
    assert_eq!(plan.effective_type().to_string(), "Optional<String>");
}

#[test]
fn long_acyclic_alias_chains_refuse_resource_exhaustion() {
    let mut declarations = String::new();
    for depth in 0..129 {
        let next = if depth == 128 {
            "example.flow.Body".to_owned()
        } else {
            format!("example.flow.N{}", depth + 1)
        };
        write!(
            declarations,
            "  - name: example.flow.N{depth}\n    kind: newtype\n    of: {next}\n"
        )
        .unwrap();
    }
    let text = model(3, "example.flow.N0", "String", "event.data.status")
        .replace("events:\n", &format!("{declarations}events:\n"));
    let error = admit(&text).unwrap_err();
    assert!(error.contains("accessor_resource"), "{error}");
    assert!(error.contains("operations"), "{error}");
}

#[test]
fn collection_traversal_is_refused_even_under_an_optional_target() {
    for payload in ["List<example.flow.Body>", "Map<String, example.flow.Body>"] {
        let error = admit(&model(3, payload, "Optional<String>", "event.data.status")).unwrap_err();
        assert!(error.contains("accessor_traversal"), "{error}");
    }
}
