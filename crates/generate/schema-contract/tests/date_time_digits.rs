//! `format: date-time` refuses a non-digit at a digit position.
//!
//! The dependency's own checker takes `+`, `-` and `.` for digits at some positions. The schema
//! validation behind `ess generate schema validate` must not.

use schema_contract::validate::{validate, IssueCode, JsonDocument};
use serde_json::{json, Value};

fn run(at: &str) -> (bool, Vec<(IssueCode, String)>) {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "urn:example:stamp",
        "type": "object",
        "properties": {
            "schema": {"const": "urn:example:stamp"},
            "at": {"type": "string", "format": "date-time"}
        }
    });
    let instance: Value = json!({"schema": "urn:example:stamp", "at": at});
    let report = validate(
        &[JsonDocument::new("schemas/stamp.json", &schema)],
        &[JsonDocument::new("instances/stamp.json", &instance)],
    );
    let issues = report
        .issues
        .iter()
        .map(|issue| (issue.code, issue.instance_path.clone()))
        .collect();
    (report.is_valid(), issues)
}

#[test]
fn schema_validate_refuses_a_non_digit_in_a_date_time() {
    for bad in [
        "202+-10-29T12:00:00Z",
        "2026-10-29T1-:00:00Z",
        "2026-1.-29T12:00:00Z",
    ] {
        let (valid, issues) = run(bad);
        assert!(!valid, "{bad} was admitted");
        assert_eq!(
            issues,
            vec![(IssueCode::InvalidInstance, "/at".to_string())],
            "{bad}"
        );
    }
    let (valid, issues) = run("2026-10-29T12:00:00Z");
    assert!(valid, "{issues:?}");
}
