//! A closed tuple imports: the served answer's `published` list is one (`prefixItems` in emit
//! order with `items: false`, or `maxItems: 0` for a branch that publishes nothing).

use ess_openapi::{import, project_import, read_import};
use serde_json::{json, Value};

fn source(schema: &Value) -> String {
    json!({
        "openapi": "3.1.0", "info": {"title": "tuples", "version": "v1"},
        "paths": {}, "components": {"schemas": {"A": schema}}
    })
    .to_string()
}

fn event(name: &str) -> Value {
    json!({
        "type": "object", "additionalProperties": false,
        "properties": {"event": {"type": "string", "const": name}, "payload": {"type": "integer"}},
        "required": ["event", "payload"]
    })
}

fn imports(schema: &Value) {
    let report = import(&source(schema)).unwrap_or_else(|errors| panic!("{errors:?}"));
    assert!(
        report.accounting().coverage_gaps.is_empty(),
        "{:?}",
        report.accounting().coverage_gaps
    );
    let checked = read_import(&report.to_canonical_json()).unwrap();
    assert_eq!(checked, report);
    assert!(project_import(&checked).is_ok());
}

#[test]
fn a_closed_tuple_in_emit_order_imports() {
    imports(&json!({
        "type": "array", "prefixItems": [event("Opened"), event("Stamped")],
        "items": false, "minItems": 2, "maxItems": 2
    }));
}

#[test]
fn an_empty_tuple_imports() {
    imports(&json!({"type": "array", "maxItems": 0}));
}

#[test]
fn a_tuple_that_is_not_closed_or_whose_bounds_disagree_is_refused() {
    for schema in [
        json!({"type": "array", "prefixItems": [event("Opened")]}),
        json!({"type": "array", "prefixItems": [event("Opened")], "items": {"type": "integer"}}),
        json!({"type": "array", "prefixItems": [event("Opened")], "items": false, "maxItems": 2}),
        json!({"type": "array", "prefixItems": [event("Opened")], "items": false, "minItems": 0}),
        json!({"type": "array", "prefixItems": {"a": 1}, "items": false}),
    ] {
        let errors = import(&source(&schema)).expect_err("refused");
        assert!(
            errors
                .iter()
                .any(|error| error.pointer.starts_with("/components/schemas/A")),
            "{schema}: {errors:?}"
        );
    }
}

#[test]
fn a_refused_tuple_item_refuses_the_tuple() {
    let errors = import(&source(&json!({
        "type": "array", "prefixItems": [true], "items": false
    })))
    .expect_err("refused");
    assert!(
        errors
            .iter()
            .any(|error| error.pointer == "/components/schemas/A/prefixItems/0"),
        "{errors:?}"
    );
}
