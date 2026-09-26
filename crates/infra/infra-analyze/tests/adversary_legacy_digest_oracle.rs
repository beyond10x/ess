//! Adversarial case: a derived document plus the `infra-ir/3` this build writes must not confirm
//! a Secret guess.
//!
//! `infra-ir/3` is presented as safe to hold: nothing derived from a Secret value. A persisted
//! legacy `infra-ir/1` still reads, and every document derived from it — `infra-graph`'s
//! `source_digest`, `infra-drift`'s `from`/`to`, `infra-simulation`'s snapshot, the projection's
//! `snapshot_digest` — names it by its model digest, which is SHA-256 over a model holding each
//! value's unsalted `{sha256, length}`. `ess infra import kubernetes --path <legacy> --out` writes
//! the IR/3 of that same model, which differs only in those values. Together the two are a guess
//! oracle: substitute a guess's digest into the IR/3 model, rehash, compare.

use infra_analyze::{GraphDocument, InfraGraph};

const LEGACY: &str =
    include_str!("../../infra-compiler/tests/fixtures/legacy-k3d-dev-cluster.ir-1.json");

fn digest_of(value: &str) -> serde_json::Value {
    serde_json::json!({
        "sha256": infra_compiler::digest_of_canonical(value.as_bytes()),
        "length": value.len(),
    })
}

fn restamp(document: &mut serde_json::Value) {
    let canonical = serde_json::to_vec(&document["model"]).expect("the model serializes");
    document["digest"] = serde_json::json!(infra_compiler::digest_of_canonical(&canonical));
}

#[test]
fn a_legacy_graph_digest_and_the_converted_ir_three_do_not_confirm_a_secret_guess() {
    // A legacy IR/1 of a cluster whose three Secret keys hold low-entropy values, exactly as an
    // `infra-observation/1` scanner would have digested them.
    let truth = [
        ("shop/devspace-cache-acd", "cache", "hunter2"),
        ("shop/sa-token-legacy", "token", "letmein"),
        ("shop/storefront-server", "turn-secret", "changeme"),
    ];
    let mut legacy: serde_json::Value = serde_json::from_str(LEGACY).expect("frozen legacy IR");
    assert_eq!(legacy["format"], "infra-ir/1");
    for (secret, key, value) in truth {
        legacy["model"]["secrets"][secret]["keys"][key] = digest_of(value);
    }
    restamp(&mut legacy);
    let legacy = infra_compiler::read_document(&legacy).expect("a legacy IR/1 still reads");

    // What `ess infra graph --path <legacy> --format json` prints: holds no Secret key digest.
    let graph = GraphDocument::of(&InfraGraph::of(&legacy), &legacy, None);
    let published = graph.source_digest.clone();

    // What `ess infra import kubernetes --path <legacy> --out` writes: presence only.
    let converted = serde_json::to_value(legacy.without_secret_digests().document())
        .expect("the IR/3 serializes");
    assert_eq!(converted["format"], "infra-ir/3");

    // The holder of those two files, and a three-word dictionary.
    let dictionary = ["password", "hunter2", "letmein", "changeme"];
    let mut confirmed = Vec::new();
    for a in dictionary {
        for b in dictionary {
            for c in dictionary {
                let mut model = converted["model"].clone();
                for ((secret, key, _), guess) in truth.iter().zip([a, b, c]) {
                    model["secrets"][secret]["keys"][key] = digest_of(guess);
                }
                let candidate = infra_compiler::digest_of_canonical(
                    &serde_json::to_vec(&model).expect("the model serializes"),
                );
                if candidate == published {
                    confirmed.push([a, b, c]);
                }
            }
        }
    }
    assert!(
        confirmed.is_empty(),
        "infra-graph source_digest {published} of a legacy IR, with the infra-ir/3 written from \
         it, confirms the Secret values {confirmed:?}"
    );
}
