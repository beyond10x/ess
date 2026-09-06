//! Admission and compatibility controls for the versioned `OpenAPI` import result.

use ess_openapi::{import, project, project_import, read_import, read_interface};
use serde_json::{json, Value};

// Exact /1 DTOs, validation and reader extracted from opening 21eac63d lib.rs.
#[allow(dead_code)]
#[path = "fixtures/legacy_v1_reader.rs"]
mod legacy_v1;

const SOURCE: &str = include_str!("fixtures/supported.openapi.yaml");
const LEGACY: &str = include_str!("fixtures/supported.interface-v1.json");
const PROJECTED: &str = include_str!("fixtures/supported.projected.yaml");

#[test]
fn checked_envelope_preserves_exact_source_and_legacy_bytes() {
    let report = import(SOURCE).unwrap();
    assert_eq!(report.source().text, SOURCE);
    assert_eq!(
        report.source().sha256,
        "2eded720b5a5869accb3b0646bbdbcb49805ca5c207e917ce761fa7db2fd22a3"
    );
    assert_eq!(report.interface().to_canonical_json(), LEGACY);
    assert_eq!(
        legacy_v1::read_interface(LEGACY)
            .unwrap()
            .to_canonical_json(),
        LEGACY
    );
    assert_eq!(read_interface(LEGACY).unwrap().to_canonical_json(), LEGACY);
    assert_eq!(project(report.interface()).unwrap(), PROJECTED);
    assert_eq!(project_import(&report).unwrap(), PROJECTED);
    let bytes = report.to_canonical_json();
    let value: Value = serde_json::from_str(&bytes).unwrap();
    assert_eq!(value["format"], "ess-openapi-import/1");
    assert_eq!(value["normalization"], "ess-openapi-service-subset/1");
    assert_eq!(
        value["schema_dialect"],
        "https://spec.openapis.org/oas/3.1/dialect/base"
    );
    assert!(value["accounting"]["coverage_gaps"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!value["accounting"]["normalizations"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(read_import(&bytes).unwrap().to_canonical_json(), bytes);
    assert_eq!(
        read_import(&serde_yaml::to_string(&value).unwrap()).unwrap(),
        report
    );
    assert!(legacy_v1::read_interface(&bytes).is_err());
    assert!(read_interface(&bytes).is_err());
    let refusal = read_import(LEGACY).unwrap_err();
    assert!(refusal
        .iter()
        .any(|item| item.message.contains("accounting unavailable")
            && item.message.contains("reimport")));
}

#[test]
fn raw_source_identity_includes_comments_and_line_endings() {
    let unix = import(SOURCE).unwrap();
    let changed = format!(
        "# same schema, distinct source\r\n{}",
        SOURCE.replace('\n', "\r\n")
    );
    let windows = import(&changed).unwrap();
    assert_eq!(windows.interface(), unix.interface());
    assert_ne!(windows.source().sha256, unix.source().sha256);
    assert_eq!(windows.source().text, changed);
    assert_eq!(read_import(&windows.to_canonical_json()).unwrap(), windows);
}

#[test]
fn partial_accounting_survives_reload_and_blocks_projection() {
    let source = SOURCE.replace(
        "type: number",
        "type: number\n          enum: [1, 2]\n          const: 1",
    );
    let report = import(&source).unwrap();
    assert_eq!(report.accounting().coverage_gaps.len(), 2);
    let reloaded = read_import(&report.to_canonical_json()).unwrap();
    assert_eq!(reloaded.accounting(), report.accounting());
    assert!(project_import(&reloaded).is_err());
    assert!(project(reloaded.interface()).is_ok());
}

#[test]
fn unresolved_references_keep_distinct_sites_and_escaped_target_identity() {
    let source = SOURCE
        .replace(
            "#/components/schemas/CreateInvoice",
            "#/components/schemas/Miss~0~1",
        )
        .replace(
            "#/components/schemas/Invoice",
            "#/components/schemas/Miss~0~1",
        );
    let report = import(&source).unwrap();
    let sites = &report.accounting().unresolved_references;
    assert_eq!(sites.len(), 2);
    assert!(sites
        .iter()
        .all(|site| site.target == "Miss~/" && site.pointer.ends_with("/$ref")));
    assert_ne!(sites[0].pointer, sites[1].pointer);
    let reloaded = read_import(&report.to_canonical_json()).unwrap();
    assert_eq!(reloaded.accounting(), report.accounting());
    assert!(project_import(&reloaded).is_err());
}

#[test]
fn replay_refuses_tampered_identity_interface_and_accounting() {
    let source = SOURCE
        .replace("type: number", "type: number\n          enum: [1]")
        .replace(
            "#/components/schemas/Invoice",
            "#/components/schemas/Missing",
        );
    let value: Value = serde_json::from_str(&import(&source).unwrap().to_canonical_json()).unwrap();
    for (pointer, replacement) in [
        ("/format", json!("ess-openapi-import/2")),
        ("/normalization", json!("ess-openapi-service-subset/2")),
        ("/schema_dialect", json!("https://example.invalid/dialect")),
        ("/source/text", json!(SOURCE)),
        ("/source/sha256", json!("0".repeat(64))),
        ("/source/sha256", json!("bad")),
        ("/interface/service/name", json!("different")),
        ("/accounting/coverage_gaps", json!([])),
        ("/accounting/normalizations", json!([])),
        ("/accounting/unresolved_references", json!([])),
        (
            "/accounting/unresolved_references/0/pointer",
            json!("/elsewhere"),
        ),
    ] {
        let mut changed = value.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            read_import(&changed.to_string()).is_err(),
            "accepted tampering at {pointer}"
        );
    }
    for pointer in [
        "",
        "/source",
        "/accounting",
        "/interface",
        "/interface/service",
        "/accounting/coverage_gaps/0",
        "/accounting/unresolved_references/0",
    ] {
        let mut changed = value.clone();
        changed
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected".to_owned(), json!(true));
        assert!(
            read_import(&changed.to_string()).is_err(),
            "accepted unknown field at {pointer}"
        );
    }
    for field in [
        "format",
        "normalization",
        "source",
        "schema_dialect",
        "interface",
        "accounting",
    ] {
        let mut changed = value.clone();
        changed.as_object_mut().unwrap().remove(field);
        assert!(
            read_import(&changed.to_string()).is_err(),
            "accepted missing {field}"
        );
    }
}

#[test]
fn repeated_fields_and_nested_map_keys_refuse_before_erasure() {
    let bytes = import(SOURCE).unwrap().to_canonical_json();
    for (needle, replacement) in [
        (
            "\"format\": \"ess-openapi-import/1\",",
            "\"format\": \"ess-openapi-import/1\", \"format\": \"ess-openapi-import/1\",",
        ),
        (
            "\"name\": \"invoice-service\",",
            "\"name\": \"invoice-service\", \"name\": \"invoice-service\",",
        ),
        (
            "\"amount\": {",
            "\"amount\": {\"kind\": \"number\"}, \"amount\": {",
        ),
        (
            "\"normalizations\": [",
            "\"normalizations\": [], \"normalizations\": [",
        ),
    ] {
        assert!(bytes.contains(needle));
        let changed = bytes.replacen(needle, replacement, 1);
        let errors = read_import(&changed).unwrap_err();
        assert!(
            errors.iter().any(|item| item.message.contains("duplicate")),
            "{errors:?}"
        );
    }
    let yaml = serde_yaml::to_string(&serde_json::from_str::<Value>(&bytes).unwrap()).unwrap();
    let changed = yaml.replacen(
        "name: invoice-service",
        "name: invoice-service\n    name: invoice-service",
        1,
    );
    assert_ne!(yaml, changed);
    assert!(read_import(&changed).is_err());
}
