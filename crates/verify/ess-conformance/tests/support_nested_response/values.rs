//! Source-admitted path shapes and complete independently supplied response values.
use super::{suite, Backend, Fault, MODEL};
use ess_conformance::{report::Status, AdmittedSuite, Runner};
use serde_json::{json, Value};

fn backend(response: Value, payload: Value) -> Backend {
    let mut target = Backend::new(Fault::None);
    target.values = Some((
        serde_json::from_value(response).unwrap(),
        serde_json::from_value(payload).unwrap(),
    ));
    target
}
fn all_runtimes(
    label: &str,
    admitted: &AdmittedSuite,
    response: Value,
    payload: Value,
    passed: bool,
) {
    let target = backend(response.clone(), payload.clone());
    let run = Runner::for_suite(admitted.suite()).run_admitted(admitted, &target);
    assert_eq!(
        run.scenarios[0].status,
        if passed {
            Status::Passed
        } else {
            Status::Failed
        },
        "{label}: {run:?}"
    );
    let (run, target) = super::foreign::go_run(
        admitted,
        admitted.original_json(),
        backend(response.clone(), payload.clone()),
        label,
    );
    assert_eq!(target.calls.get(), 1, "{label}: {}", run.log);
    assert_eq!(run.success, passed, "{label}: {}", run.log);
    let (output, report, target) = super::foreign::typescript_run(
        admitted,
        admitted.original_json(),
        backend(response, payload),
        label,
    );
    assert_eq!(
        target.calls.get(),
        1,
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.success(),
        passed,
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        report.unwrap()["counts"]["failed"],
        usize::from(!passed),
        "{label}"
    );
}

#[test]
fn exact_integer_and_complete_aggregate_leaves_across_actual_runtimes() {
    let admitted = suite(MODEL);
    for (label, event, passed) in [
        ("exact-integer", 9_007_199_254_740_993_u64, true),
        ("rounded-integer", 9_007_199_254_740_992, false),
    ] {
        all_runtimes(
            label,
            &admitted,
            json!({"value":9_007_199_254_740_993_u64}),
            json!({"packet":{"value":event},"receipt":"generated"}),
            passed,
        );
    }
    let admitted = suite(&MODEL.replace("type: Integer", "type: List<Integer>"));
    for (label, event, passed) in [
        ("aggregate-equal", json!([1, 2, 2]), true),
        ("aggregate-reordered", json!([2, 1, 2]), false),
    ] {
        all_runtimes(
            label,
            &admitted,
            json!({"value":[1,2,2]}),
            json!({"packet":{"value":event},"receipt":"generated"}),
            passed,
        );
    }
    let admitted = suite(&MODEL.replace("type: Integer", "type: 'Map<String, Integer>'"));
    for (label, event, passed) in [
        ("map-equal", json!({"a":1,"b":2}), true),
        ("map-missing", json!({"a":1}), false),
    ] {
        all_runtimes(
            label,
            &admitted,
            json!({"value":{"a":1,"b":2}}),
            json!({"packet":{"value":event},"receipt":"generated"}),
            passed,
        );
    }
    let text = MODEL.replace("type: Integer", "type: demo.api.Item").replace("events:\n", "  - name: demo.api.Item\n    kind: struct\n    fields:\n      - {name: number, type: Integer}\n      - {name: label, type: Optional<String>}\nevents:\n");
    let admitted = suite(&text);
    for (label, event, passed) in [
        ("struct-equal", json!({"number":37,"label":"actual"}), true),
        ("struct-wrong", json!({"number":38,"label":"actual"}), false),
    ] {
        all_runtimes(
            label,
            &admitted,
            json!({"value":{"number":37,"label":"actual"}}),
            json!({"packet":{"value":event},"receipt":"generated"}),
            passed,
        );
    }
}

#[test]
fn optional_terminal_presence_and_constructed_ancestors_across_actual_runtimes() {
    let text = MODEL
        .replace("type: Integer", "type: Optional<Integer>")
        .replace(
            "{name: packet, type: demo.api.Packet}",
            "{name: packet, type: Optional<demo.api.Packet>}",
        );
    let admitted = suite(&text);
    for (label, response, packet, passed) in [
        (
            "optional-null",
            json!({"value":null}),
            json!({"value":null}),
            true,
        ),
        ("optional-missing", json!({}), json!({}), true),
        (
            "optional-equivalent",
            json!({}),
            json!({"value":null}),
            true,
        ),
        ("optional-null-ancestor", json!({}), Value::Null, false),
        ("optional-scalar-ancestor", json!({}), json!(37), false),
    ] {
        all_runtimes(
            label,
            &admitted,
            response,
            json!({"packet":packet,"receipt":"generated"}),
            passed,
        );
    }
    let presence = text.replacen(
        "    response:\n      - {name: value, type: Optional<Integer>}",
        "    response:\n      - {name: value, type: Optional<Integer>, presence: null_when_absent}",
        1,
    );
    let admitted = suite(&presence.replace("ess/14", "ess/15"));
    all_runtimes(
        "presence-before-equality",
        &admitted,
        json!({}),
        json!({"packet":{"value":null},"receipt":"generated"}),
        false,
    );
}

#[test]
fn newtype_ancestor_and_multiple_flat_nested_relationships_across_actual_runtimes() {
    let text=MODEL.replace("events:\n","  - name: demo.api.Wrapped\n    kind: newtype\n    of: Optional<demo.api.Packet>\nevents:\n")
        .replace("{name: packet, type: demo.api.Packet}","{name: packet, type: demo.api.Wrapped}");
    let admitted = suite(&text);
    all_runtimes(
        "newtype-ancestor",
        &admitted,
        json!({"value":37}),
        json!({"packet":{"value":37},"receipt":"generated"}),
        true,
    );
    let text=MODEL.replace("      - {name: receipt, type: String}","      - {name: mirror, type: demo.api.Packet}\n      - {name: flat, type: Integer}\n      - {name: receipt, type: String}")
        .replace("            receipt: {generated: true}","            mirror:\n              value: {response: value}\n            flat: {response: value}\n            receipt: {generated: true}");
    let admitted = suite(&text);
    for (label, mirror, passed) in [("mixed-equal", 37, true), ("mixed-second-root", 38, false)] {
        all_runtimes(
            label,
            &admitted,
            json!({"value":37}),
            json!({"packet":{"value":37},"mirror":{"value":mirror},"flat":37,"receipt":"generated"}),
            passed,
        );
    }
}

#[test]
fn owned_special_names_are_compared_without_prototype_lookup() {
    for name in ["__proto__", "toString"] {
        let text = MODEL.replace("value", name);
        let admitted = suite(&text);
        let response = json!({name:37});
        let payload = json!({"packet":{name:37},"receipt":"generated"});
        all_runtimes(&format!("owned-{name}"), &admitted, response, payload, true);
    }
}

pub fn deep_model(depth: usize) -> String {
    use std::fmt::Write;
    let mut text =
        "format: ess/14\nsystem: demo\nversion: v1\ndomain: demo.api\ntypes:\n".to_owned();
    for level in 0..depth {
        let ty = if level + 1 == depth {
            "Integer".into()
        } else {
            format!("demo.api.Level{}", level + 1)
        };
        writeln!(text,"  - name: demo.api.Level{level}\n    kind: struct\n    fields:\n      - {{name: value, type: {ty}}}").unwrap();
    }
    text.push_str("events:\n  - name: demo.api.Returned\n    fields:\n      - {name: packet, type: demo.api.Level0}\ncommands:\n  - name: demo.api.Read\n    response:\n      - {name: value, type: Integer}\n    outcomes:\n      - name: returned\n        emits: [demo.api.Returned]\n        payload:\n          demo.api.Returned:\n            packet:\n");
    for level in 0..depth {
        write!(text, "{}value:", " ".repeat(14 + 2 * level)).unwrap();
        text.push_str(if level + 1 == depth {
            " {response: value}\n"
        } else {
            "\n"
        });
    }
    text
}
#[test]
fn source_boundary_allows_33_segments_and_refuses_34() {
    let admitted = suite(&deep_model(32));
    let mut packet = json!(37);
    for _ in 0..32 {
        packet = json!({"value":packet});
    }
    all_runtimes(
        "depth-33",
        &admitted,
        json!({"value":37}),
        json!({"packet":packet}),
        true,
    );
    let spec = ess_domain::Specification::assemble([(
        ess_domain::system::Source::new("depth.yaml"),
        ess_domain::spec::RawSpecFile::parse(&deep_model(33)).unwrap(),
    )]);
    assert!(
        spec.is_err(),
        "34-segment source must be refused at its construction depth boundary"
    );
}

#[test]
fn generated_member_beside_mapped_leaf_keeps_its_ordinary_shape_obligation() {
    let text = MODEL
        .replacen(
            "      - {name: value, type: Integer}",
            "      - {name: value, type: Integer}\n      - {name: valueExtra, type: String}",
            1,
        )
        .replace(
            "              value: {response: value}",
            "              value: {response: value}\n              valueExtra: {generated: true}",
        );
    let admitted = suite(&text);
    for (label, sibling, passed) in [
        ("nested-sibling-healthy", json!("generated"), true),
        ("nested-sibling-wrong", Value::Null, false),
    ] {
        all_runtimes(
            label,
            &admitted,
            json!({"value":37}),
            json!({"packet":{"value":37,"valueExtra":sibling},"receipt":"generated"}),
            passed,
        );
    }
}

#[test]
fn response_conversion_on_leaf_or_ancestor_is_an_explicit_synthesis_refusal() {
    use ess_compiler::ir::ResolvedPayloadValue;
    let ir = super::model(MODEL);
    let command = ir.commands().values().next().unwrap();
    for ancestor in [false, true] {
        let mut outcome = command.outcomes[0].clone();
        let root = &mut outcome.payload[0].fields[0];
        if ancestor {
            root.conversion = Some("approved representation crossing".into());
        } else if let ResolvedPayloadValue::Struct { fields } = &mut root.value {
            fields[0].conversion = Some("approved representation crossing".into());
        }
        let error = ess_conformance::response::Observation::of(&ir, command, &outcome).unwrap_err();
        assert!(error.contains("conversion"), "{error}");
    }
}

#[test]
fn source_binary64_sibling_keeps_the_existing_explicit_model_refusal() {
    let text = MODEL
        .replacen(
            "      - {name: value, type: Integer}",
            "      - {name: value, type: Integer}\n      - {name: real, type: Binary64}",
            1,
        )
        .replace(
            "              value: {response: value}",
            "              value: {response: value}\n              real: {generated: true}",
        );
    let ir = super::model(&text.replace("ess/14", "ess/15"));
    assert!(ess_conformance::admission::model(&ir).is_err());
}

#[test]
fn observation_cannot_use_a_different_invocation_outcome_or_event() {
    for (label, member, value) in [
        ("stale-command", "command", "demo.api.Other"),
        ("stale-outcome", "outcome", "other"),
        ("stale-event", "event", "demo.api.Other"),
    ] {
        let mut document = super::admission::document();
        let response = super::admission::observation(&mut document);
        match member {
            "command" => {
                response["command"] = json!(value);
                response["outcome"]["command"] = json!(value);
            }
            "outcome" => response["outcome"]["outcome"] = json!(value),
            _ => response["event"] = json!(value),
        }
        let admitted = AdmittedSuite::from_json(&document.to_string()).unwrap();
        all_runtimes(
            label,
            &admitted,
            json!({"value":37}),
            json!({"packet":{"value":37},"receipt":"generated"}),
            false,
        );
    }
}
