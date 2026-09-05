//! Independent probes of the authoritative stamp admission boundary.

use ess_gen::Provenance;

fn schema() -> String {
    include_str!("../../../../generated/schema/types/billing.invoice.InvoiceId.schema.json").into()
}

#[test]
fn structured_stamp_requires_the_complete_emitted_envelope() {
    let source = schema();
    assert!(Provenance::read_digests(&source).is_some());
    for removed in ["system", "specification_version", "regenerate"] {
        let mut document: serde_json::Value = serde_json::from_str(&source).unwrap();
        document["x-ess-provenance"]
            .as_object_mut()
            .unwrap()
            .remove(removed);
        let damaged = serde_json::to_string_pretty(&document).unwrap();
        let read = Provenance::read_digests(&damaged);
        println!("removed {removed}: {read:?}");
        assert!(
            read.is_none(),
            "an incomplete authoritative envelope missing {removed} must be unreadable"
        );
    }
}

#[test]
fn conflicting_locations_and_every_malformed_profile_are_refused() {
    let source = schema();
    let current = Provenance::read_digests(&source).unwrap();
    assert!(current.contract_digest.starts_with("slice-sha256/2:"));
    for contract in [
        format!("slice-sha256/3:{}", "a".repeat(64)),
        format!("slice-sha256/2:{}", "A".repeat(64)),
        format!("slice-sha256/2:{}", "a".repeat(63)),
        format!("slice-sha256/2:{} ", "a".repeat(64)),
        format!("{}:slice-sha256/2", "a".repeat(64)),
    ] {
        let damaged = source.replace(&current.contract_digest, &contract);
        assert!(Provenance::read_digests(&damaged).is_none(), "{contract}");
    }
    let mut duplicate: serde_json::Value = serde_json::from_str(&source).unwrap();
    duplicate["provenance"] = duplicate["x-ess-provenance"].clone();
    assert!(Provenance::read_digests(&duplicate.to_string()).is_none());
    duplicate["provenance"]["contract_digest"] = serde_json::json!("a".repeat(64));
    assert!(Provenance::read_digests(&duplicate.to_string()).is_none());
}

#[test]
fn complete_comments_do_not_admit_an_unknown_profile_via_body_markers() {
    let encoded: serde_json::Value = serde_json::from_str(&schema()).unwrap();
    let provenance: Provenance =
        serde_json::from_value(encoded["x-ess-provenance"].clone()).unwrap();
    for header in [provenance.html_comment(), provenance.commented("//")] {
        assert!(Provenance::read_digests(&header).is_some());
        let invalid = header.replace("slice-sha256/2:", "slice-sha256/99:");
        let body = format!(
            "{invalid}\n{}",
            serde_json::to_string_pretty(&provenance).unwrap()
        );
        assert!(Provenance::read_digests(&body).is_none());
    }
}
