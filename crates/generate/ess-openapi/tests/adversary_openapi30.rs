//! Adversarial cases against the `OpenAPI` 3.0 -> 3.1 rewrite (beyond10x/ess#73).

use ess_openapi::import;
use serde_json::{json, Value};

const POINTER: &str = "/components/schemas/A";

fn document(version: &str, schema: &Value) -> String {
    json!({
        "openapi": version, "info": {"title": "test", "version": "v1"},
        "paths": {"/items": {"post": {"operationId": "create", "responses": {"200": {"description": "ok"}}}}},
        "components": {"schemas": {"A": schema, "B": {"type": "string"}}}
    })
    .to_string()
}

fn gaps(source: &str) -> Vec<String> {
    import(source)
        .expect("imports")
        .accounting()
        .coverage_gaps
        .iter()
        .map(|gap| gap.pointer.clone())
        .collect()
}

/// `const` is not an `OpenAPI` 3.0 Schema Object keyword, so in a 3.0 document it cannot exclude the
/// `null` that `nullable: true` admits. The claim is that only an `enum` lacking null makes 3.0
/// `nullable` exact; reading 3.0 through 3.1's `const` imports a narrower type with no gap.
#[test]
fn openapi30_const_does_not_silently_cancel_nullable() {
    let source = document(
        "3.0.3",
        &json!({"type": "string", "nullable": true, "const": "a"}),
    );
    match import(&source) {
        Err(_) => {}
        Ok(imported) => {
            let pointers: Vec<&str> = imported
                .accounting()
                .coverage_gaps
                .iter()
                .map(|gap| gap.pointer.as_str())
                .collect();
            assert!(
                pointers.contains(&format!("{POINTER}/nullable").as_str())
                    || pointers.contains(&format!("{POINTER}/const").as_str()),
                "a 3.0 nullable string with a (3.1-only) const imported exact: {:?}",
                imported.interface().types["A"]
            );
        }
    }
}

/// The commit refuses "a numeric bound in a 3.0 document" at its own pointer. Beside `$ref` the
/// rewrite returns early, so the numeric (3.1-only) bound is not refused there.
#[test]
fn openapi30_numeric_bound_beside_ref_is_refused_at_its_pointer() {
    let source = document(
        "3.0.3",
        &json!({"$ref": "#/components/schemas/B", "exclusiveMinimum": 3}),
    );
    let refusals = import(&source).expect_err("a numeric bound in a 3.0 document refuses");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.pointer == format!("{POINTER}/exclusiveMinimum")),
        "{refusals:?}"
    );
}

/// A 3.0 nullable whose enum is exactly `[null]` admits only null. Whatever the adapter does, it
/// must not tell the author their enum is empty.
#[test]
fn openapi30_null_only_enum_is_not_reported_as_an_empty_enum() {
    for (version, schema) in [
        (
            "3.0.3",
            json!({"type": "string", "nullable": true, "enum": [null]}),
        ),
        ("3.1.0", json!({"type": ["string", "null"], "enum": [null]})),
    ] {
        if let Err(refusals) = import(&document(version, &schema)) {
            assert!(
                refusals
                    .iter()
                    .all(|refusal| !refusal.message.contains("empty enumerations")),
                "{version} {schema}: {refusals:?}"
            );
        }
    }
}

/// Nullable nested in items and properties is accounted at the nested pointer.
#[test]
fn openapi30_nullable_in_nested_items_is_accounted_at_its_own_pointer() {
    let source = document(
        "3.0.3",
        &json!({"type": "array", "items": {"type": "array", "items": {"type": "integer", "nullable": true}}}),
    );
    assert_eq!(gaps(&source), [format!("{POINTER}/items/items/nullable")]);
}

/// Every patch of 3.0 and 3.1 imports; 3.2 and malformed versions refuse at `/openapi`.
#[test]
fn openapi_version_boundaries() {
    for version in ["3.0.0", "3.0.4", "3.0.10", "3.1.1"] {
        assert!(
            import(&document(version, &json!({"type": "string"}))).is_ok(),
            "{version}"
        );
    }
    for version in [
        "3.0",
        "3.0.",
        "3.0.0-rc1",
        "3.0.x",
        " 3.0.3",
        "3.2.0",
        "3.00.0",
    ] {
        let refusals = import(&document(version, &json!({"type": "string"}))).expect_err("refuses");
        assert!(
            refusals.iter().any(|refusal| refusal.pointer == "/openapi"),
            "{version}: {refusals:?}"
        );
    }
}

/// A 3.0 document cannot use a 3.1 type array; it is refused rather than read as a null union.
#[test]
fn openapi30_type_array_is_refused() {
    let refusals = import(&document("3.0.3", &json!({"type": ["string", "null"]})))
        .expect_err("3.0 has no type arrays");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.pointer == format!("{POINTER}/type")),
        "{refusals:?}"
    );
}
