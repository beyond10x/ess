//! Additional authority and ambiguity probes for corrected provenance admission.

use ess_gen::Provenance;

fn schema() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../generated/schema/types/billing.invoice.InvoiceId.schema.json"
    ))
    .unwrap()
}

/// A well-formed whole-model digest, as an earlier release stamped beside the contract digest.
fn model() -> String {
    "a".repeat(64)
}

#[test]
fn duplicate_digest_aliases_and_duplicate_system_keys_are_unreadable() {
    let mut schema = schema();
    let expected = Provenance::read_digests(&schema.to_string()).unwrap();
    schema["x-ess-provenance"]["source_digest"] = serde_json::json!(model());
    assert_eq!(
        Provenance::read_digests(&schema.to_string()),
        Some(expected.clone()),
        "the profile an earlier release wrote still reads"
    );
    schema["x-ess-provenance"]["spec_digest"] = serde_json::json!(model());
    assert!(Provenance::read_digests(&schema.to_string()).is_none());
    let mut clean = self::schema();
    clean["x-ess-provenance"]["spec_digest"] = serde_json::json!(model());
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
    let expected = Provenance::read_digests(source).unwrap();
    // The profile an earlier release wrote: a model digest in the comment and in the copy.
    let legacy = source
        .replacen(
            "\n# contract digest ",
            &format!("\n# model digest {}\n# contract digest ", model()),
            1,
        )
        .replacen(
            "\n    contract_digest: ",
            &format!("\n    source_digest: {}\n    contract_digest: ", model()),
            1,
        );
    assert_ne!(source, legacy);
    assert_eq!(Provenance::read_digests(&legacy), Some(expected));
    let duplicate = legacy.replace(
        &format!("    source_digest: {}", model()),
        &format!(
            "    source_digest: {}\n    spec_digest: {}",
            model(),
            model()
        ),
    );
    assert_ne!(legacy, duplicate);
    assert!(Provenance::read_digests(&duplicate).is_none());
}
