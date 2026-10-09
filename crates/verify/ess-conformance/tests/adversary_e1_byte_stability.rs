//! Adversary, wave 2026-10-08e unit e1 (beyond10x/ess#499): a specification with no constrained
//! response type keeps the suite format and bytes it had, and `constraints` is emitted only when a
//! reachable response type is constrained (story Cost, item 6).

mod adversary_e1_support;
mod support_go;

use adversary_e1_support::*;
use ess_conformance::synthesize::synthesize;

const CODE: &str = "  - name: catalog.items.Code
    kind: newtype
    of: String
    alphabet: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-'
";

fn format_and_bytes(text: &str) -> (String, String, Vec<String>) {
    let result = synthesize(&ir(text));
    (
        result.suite.provenance.suite_version.to_string(),
        result.suite.to_canonical_json().unwrap(),
        refusals(&result),
    )
}

#[test]
fn a_constraint_on_a_request_type_only_changes_nothing() {
    let text = model(CODE, "      - {name: code, type: String}\n", "", "").replace(
        "{name: label, type: String}",
        "{name: label, type: catalog.items.Code}",
    );
    let (format, bytes, refused) = format_and_bytes(&text);
    assert_eq!(format, "ess-conformance/34", "{refused:#?}");
    assert!(!bytes.contains("\"constraints\""), "{bytes}");
}

#[test]
fn a_one_time_outcome_keeps_its_format_and_carries_no_response_constraints() {
    let text = model(
        CODE,
        "      - {name: code, type: catalog.items.Code}\n",
        "",
        "",
    )
    .replace(
        "        returns: true\n",
        "        returns: true\n        one_time_response: [code]\n",
    );
    let (format, bytes, refused) = format_and_bytes(&text);
    assert_eq!(format, "ess-conformance/34", "{refused:#?}");
    let json: serde_json::Value = serde_json::from_str(&bytes).unwrap();
    for scenario in json["scenarios"].as_object().unwrap().values() {
        for step in scenario["steps"].as_array().unwrap() {
            if matches!(
                step["step"].as_str(),
                Some("expect_direct_response" | "expect_response_payload")
            ) {
                assert!(step["response"].get("constraints").is_none(), "{step:#}");
            }
        }
    }
}
