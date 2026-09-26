//! Adversarial cases: a refusal must never quote a Secret value back.
//!
//! `INFRA-SECRET-001`'s contract is that no message echoes a value. The per-key checks honour
//! it; these cases drive the item-level shape refusal, whose text is serde's, with a Secret whose
//! `data` or `stringData` is the value itself rather than a map of keys.

use infra_domain::observation::Observation;
use infra_domain::raw::RawBundle;
use infra_domain::InfraCode;

const GUESS: &str = "hunter2";

fn observation() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../../examples/k3d-dev-cluster/observation.json"
    ))
    .expect("the committed observation is JSON")
}

#[test]
fn an_observation_refusal_never_quotes_a_secret_data_block_that_is_a_bare_value() {
    for field in ["data", "stringData"] {
        let mut bundle = observation();
        assert_eq!(bundle["format"], "infra-observation/3");
        let item = &mut bundle["kinds"]["secrets"]["items"][0];
        item.as_object_mut().expect("a secret item").remove("data");
        item[field] = serde_json::json!(GUESS);
        let raw: RawBundle = serde_json::from_value(bundle).expect("the bundle parses");
        let errors =
            Observation::try_from(raw).expect_err("a secret whose data is not a map is refused");
        assert!(errors.contains(InfraCode::MalformedObject), "{errors}");
        assert!(
            !errors.to_string().contains(GUESS),
            "the refusal of `{field}` quotes the secret value back: {errors}"
        );
    }
}

#[test]
fn an_ir_refusal_never_quotes_a_secret_keys_block_that_is_a_bare_value() {
    let raw: RawBundle = serde_json::from_value(observation()).expect("the bundle parses");
    let ir = infra_compiler::compile(&Observation::try_from(raw).expect("the example reads"));
    let mut document = serde_json::to_value(ir.document()).expect("the document serializes");
    assert_eq!(document["format"], "infra-ir/3");
    document["model"]["secrets"]["shop/storefront-server"]["keys"] = serde_json::json!(GUESS);
    let canonical = serde_json::to_vec(&document["model"]).expect("the model serializes");
    document["digest"] = serde_json::json!(infra_compiler::digest_of_canonical(&canonical));
    let errors = infra_compiler::read_document(&document)
        .expect_err("a secret whose keys are not a map is refused");
    assert!(errors.contains(InfraCode::IrMalformed), "{errors}");
    assert!(
        !errors.to_string().contains(GUESS),
        "the IR refusal quotes the secret value back: {errors}"
    );
}
