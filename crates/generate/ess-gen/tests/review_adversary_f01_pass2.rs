//! Additional authority and ambiguity probes for corrected provenance admission.

use ess_gen::Provenance;

fn schema() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../generated/schema/types/billing.invoice.InvoiceId.schema.json"
    ))
    .unwrap()
}

#[test]
fn duplicate_digest_aliases_and_duplicate_system_keys_are_unreadable() {
    let mut schema = schema();
    let expected = Provenance::read_digests(&schema.to_string()).unwrap();
    schema["x-ess-provenance"]["spec_digest"] = serde_json::json!(expected.source_digest);
    assert!(Provenance::read_digests(&schema.to_string()).is_none());
    let mut clean = self::schema();
    clean["x-ess-provenance"]
        .as_object_mut()
        .unwrap()
        .remove("source_digest");
    clean["x-ess-provenance"]["spec_digest"] = serde_json::json!(expected.source_digest);
    assert_eq!(Provenance::read_digests(&clean.to_string()), Some(expected));
    let text = clean.to_string().replace(
        "\"system\":\"billing\"",
        "\"system\":\"billing\",\"system\":\"billing\"",
    );
    assert_ne!(text, clean.to_string());
    assert!(
        Provenance::read_digests(&text).is_none(),
        "duplicate attribution remains ambiguous even when values match"
    );
}

#[test]
fn incomplete_authoritative_stamp_cannot_fall_back_to_model_provenance() {
    let mut document = schema();
    let valid = document["x-ess-provenance"].clone();
    assert!(Provenance::read_digests(&document.to_string()).is_some());
    document["description"] = serde_json::json!(format!(
        "An example: {}",
        serde_json::to_string_pretty(&valid).unwrap()
    ));
    document["example"] = serde_json::json!({"provenance": valid});
    document["x-ess-provenance"]
        .as_object_mut()
        .unwrap()
        .remove("system");
    assert!(Provenance::read_digests(&document.to_string()).is_none());
}

#[test]
fn paired_yaml_refuses_mixed_digest_aliases_even_when_hashes_agree() {
    let source = include_str!("../../../../generated/openapi/invoice-service.yaml");
    assert!(Provenance::read_digests(source).is_some());
    let expected = Provenance::read_digests(source).unwrap();
    let duplicate = source.replace(
        &format!("    source_digest: {}", expected.source_digest),
        &format!(
            "    source_digest: {}\n    spec_digest: {}",
            expected.source_digest, expected.source_digest
        ),
    );
    assert_ne!(source, duplicate);
    assert!(Provenance::read_digests(&duplicate).is_none());
}
