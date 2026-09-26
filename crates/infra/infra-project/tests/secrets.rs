//! The one thing a generated patch must never contain.
//!
//! The hard rule of the infrastructure family, third enforcement. The scanner sanitizes before it
//! writes; `infra-domain` refuses an unsanitized bundle (`INFRA-SECRET-001`) so a secret value
//! cannot enter the IR even through a bundle the scanner never touched; and this asserts the
//! consequence at the far end — that nothing which *did* enter the IR reaches a file this crate
//! writes.
//!
//! It is a byte scan on purpose. A structural argument ("the projection only reads gaps, and gaps
//! carry no secret values") is exactly the kind of reasoning that stays true until somebody adds a
//! field to a gap. Reading the emitted bytes needs no argument.

mod support;

use infra_compiler::InfraIr;

/// Every legacy digest the fixture's secrets carry, and every key name they are stored under.
///
/// A current IR holds no digest at all — a Secret key is only ever present. A legacy
/// `infra-ir/1` document still holds the scanner's old digests, so those are the strongest
/// available proxy for a leak: a patch that carried a secret's content would almost certainly
/// have carried the digest beside it.
fn secret_material(ir: &InfraIr) -> (Vec<String>, Vec<String>) {
    let mut digests = Vec::new();
    let mut keys = Vec::new();
    for secret in ir.model().secrets.values() {
        for (key, value) in &secret.keys {
            keys.push(key.clone());
            if let infra_domain::SecretValue::LegacyDigest(digest) = value {
                digests.push(digest.sha256.clone());
            }
        }
    }
    (digests, keys)
}

const LEGACY_IR: &str =
    "crates/infra/infra-compiler/tests/fixtures/legacy-k3d-dev-cluster.ir-1.json";

/// The committed example as a pre-presence compiler wrote it, digests and all, read.
fn legacy_ir() -> InfraIr {
    infra_compiler::read_document(&serde_json::from_str(&support::read(LEGACY_IR)).expect("JSON"))
        .expect("the legacy document is still read")
}

/// The legacy file itself: its Secret digests, and its model digest with each reduced to
/// presence. Read from the file, not from the reader, which strips them.
fn legacy_file() -> (Vec<String>, String) {
    let frozen: serde_json::Value = serde_json::from_str(&support::read(LEGACY_IR)).expect("JSON");
    let mut model = frozen["model"].clone();
    let mut digests = Vec::new();
    for secret in model["secrets"]
        .as_object_mut()
        .expect("secrets")
        .values_mut()
    {
        for value in secret["keys"].as_object_mut().expect("keys").values_mut() {
            digests.push(
                value["sha256"]
                    .as_str()
                    .expect("a legacy digest")
                    .to_owned(),
            );
            *value = serde_json::json!({ "present": true });
        }
    }
    let stripped = infra_compiler::digest_of_canonical(&serde_json::to_vec(&model).expect("model"));
    assert_ne!(
        stripped, frozen["digest"],
        "the legacy fixture carries no Secret digest"
    );
    (digests, stripped)
}

#[test]
fn a_projection_of_a_legacy_ir_names_the_stripped_model_not_the_one_holding_secret_digests() {
    let (_, stripped) = legacy_file();
    let projection = infra_project::project(&support::example_spec(), &legacy_ir())
        .expect("the projected candidate is admitted");
    assert_eq!(projection.provenance.snapshot_digest, stripped);
    let summary = &projection.artifacts()["SUMMARY.md"];
    assert!(
        summary.contains(&stripped),
        "SUMMARY.md names another snapshot"
    );
}

#[test]
fn the_current_ir_holds_secret_key_names_and_presence_and_nothing_else() {
    let ir = support::example_ir();
    let (digests, keys) = secret_material(&ir);
    assert!(
        !keys.is_empty(),
        "the committed observation carries no secret keys, so this proves nothing"
    );
    assert!(
        digests.is_empty(),
        "the IR holds a Secret digest: {digests:?}"
    );
    assert!(ir
        .model()
        .secrets
        .values()
        .flat_map(|secret| secret.keys.values())
        .all(|value| *value == infra_domain::SecretValue::Present));
}

#[test]
fn no_emitted_byte_carries_a_secrets_digest_or_key_name() {
    let legacy = legacy_ir();
    let (digests, _) = legacy_file();
    // The legacy document has to hold digests for the byte scan below to be a test of anything.
    assert!(
        !digests.is_empty() && digests.iter().all(|digest| digest.len() == 64),
        "the frozen legacy IR carries no `{{sha256, length}}` digest: {digests:?}"
    );

    for ir in [support::example_ir(), legacy] {
        let (_, keys) = secret_material(&ir);
        let projection = infra_project::project(&support::example_spec(), &ir)
            .expect("the projected candidate is admitted");
        let mut leaks = Vec::new();
        for (path, contents) in projection.artifacts() {
            for digest in &digests {
                if contents.contains(digest.as_str()) {
                    leaks.push(format!("{path} carries the digest {digest}"));
                }
            }
            for key in &keys {
                // A key *name* is allowed to be named in an obligation — "create the secret
                // `agent-credentials`" is the whole point of that sentence — so the scan is about the
                // `data` shape a value would arrive in, not about the word appearing at all.
                if contents.contains(&format!("\"{key}\":")) && contents.contains("\"data\"") {
                    leaks.push(format!("{path} carries a `data` block keyed by {key}"));
                }
            }
            assert!(
                !contents.contains("stringData"),
                "{path} writes a `stringData` block, which is where a plain secret value would go"
            );
        }
        assert!(leaks.is_empty(), "{}", leaks.join("\n"));
    }
}

#[test]
fn a_dangling_secret_reference_is_owed_and_the_obligation_says_why_nothing_can_write_it() {
    // The one place the projection is *asked* for a secret: the fixture's `flaky-agent` reads a
    // secret nobody observed. The honest answer is an obligation that names the reason — the
    // snapshot holds a secret's key names, never its value — rather than an empty secret manifest
    // that would look like progress and break the pod differently.
    let projection = infra_project::project(&support::example_spec(), &support::example_ir())
        .expect("the projected candidate is admitted");
    let entry = projection
        .entries
        .iter()
        .find(|entry| entry.expectation == "shop-config-refs")
        .expect("the fixture has a dangling required reference");
    let infra_project::Disposition::Obligation(obligation) = &entry.disposition else {
        panic!("a dangling secret reference must never be generated: {entry:?}");
    };
    assert!(
        obligation.decision.contains("the names of its keys"),
        "the obligation says why nothing here can write the secret: {}",
        obligation.decision
    );
}
