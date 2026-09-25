use serde_json::{json, Value};

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn authority() -> Value {
    let id = "ess-cli:bin:ess:load::accounting_tests::reading";
    json!({
        "format": "ess-consumer-model-behavior/1",
        "cases": {
            id: {
                "identity": {
                    "package": "ess-cli",
                    "target_kind": "bin",
                    "target_name": "ess",
                    "full_name": "load::accounting_tests::reading",
                    "features": [],
                    "target_profile": "x86_64-unknown-linux-gnu/default",
                    "tool_requirements": ["frozen Rust 1.98.1; locked offline owner-target build"],
                    "nested_runtime": "None claimed; direct Rust assertions only"
                },
                "target_source": "crates/edge/ess-cli/src/main.rs",
                "target_source_file_sha256": digest('a'),
                "source": "crates/edge/ess-cli/src/load_accounting_tests.rs",
                "source_file_sha256": digest('b'),
                "case_ast_sha256": digest('c'),
                "reviewed_assertion": "the returned IR preserves a declared reading contract",
                "attribution_limit": "the direct loader call establishes no runtime behavior",
                "source_evidence": "native-case-schema-scope-result.md#reading",
                "review_reference": "docs/design/consumer-model-behavior-native-cases.md"
            }
        },
        "claims": [{
            "identity": {
                "model": "rust:type:ResolvedType.field.reading",
                "shape": digest('d'),
                "consumer": "ess_cli::bin(ess)::load::fn::specification",
                "profile": digest('e')
            },
            "cases": [id],
            "entrypoint": "ess_cli::bin(ess)::load::fn::specification",
            "entrypoint_sha256": digest('f'),
            "entrypoint_source": "crates/edge/ess-cli/src/load.rs",
            "entrypoint_ast_sha256": digest('1'),
            "entrypoint_source_file_sha256": digest('2'),
            "behavior": {
                "kind": "Supported",
                "reason": "the resolved type retains the authored reading contract"
            },
            "source_evidence": "native-case-schema-scope-result.md#reading",
            "review_reference": "docs/design/consumer-model-behavior-native-cases.md"
        }]
    })
}

#[test]
fn closed_authority_accepts_one_exact_case_and_claim() {
    let bytes = serde_json::to_vec(&authority()).unwrap();
    assert!(super::model_behavior::read_authority(&bytes).is_ok());
}

/// The authority reader holds a number as `serde_json` does without `arbitrary_precision`.
///
/// `entity-core` enables that feature and Cargo unifies it into any build holding both, which
/// hands this reader's visitor a number as a one-entry map, and `-0` as the integer `0`.
#[test]
fn authority_numbers_read_the_same_in_every_serde_json_build() {
    let read = |text: &str| super::model_behavior::unique_value(text.as_bytes());
    assert_eq!(
        read("[1.50,1E2,-0,18446744073709551616,7,-7]")
            .unwrap()
            .to_string(),
        "[1.5,100.0,-0.0,1.8446744073709552e+19,7,-7]"
    );
    assert_eq!(
        read(r#"{"n":{"m":2.50}}"#).unwrap().to_string(),
        r#"{"n":{"m":2.5}}"#
    );
    assert!(read("[1e400]").is_err());
    assert!(read(r#"{"a":1,"a":1.5}"#).is_err());
}

#[test]
fn authority_rejects_duplicate_keys_before_a_json_value_exists() {
    let duplicate = br#"{"format":"ess-consumer-model-behavior/1","format":"ess-consumer-model-behavior/1","cases":{},"claims":[]}"#;
    let error = super::model_behavior::read_authority(duplicate).unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate JSON object key `format`"));
}

#[test]
fn authority_rejects_unknown_members_and_unclaimed_cases() {
    let mut unknown = authority();
    unknown["unexpected"] = json!(true);
    assert!(super::model_behavior::read_authority(&serde_json::to_vec(&unknown).unwrap()).is_err());

    let mut missing = authority();
    missing["claims"] = json!([]);
    let error =
        super::model_behavior::read_authority(&serde_json::to_vec(&missing).unwrap()).unwrap_err();
    assert!(error.to_string().contains("case/claim partition"));
}

#[test]
fn authority_rejects_inexact_hashes_and_execution_contracts() {
    let mut invalid = authority();
    invalid["claims"][0]["entrypoint_sha256"] = json!("ABC");
    let error =
        super::model_behavior::read_authority(&serde_json::to_vec(&invalid).unwrap()).unwrap_err();
    assert!(error.to_string().contains("64 lowercase hexadecimal"));

    let mut invalid = authority();
    invalid["cases"]["ess-cli:bin:ess:load::accounting_tests::reading"]["identity"]
        ["nested_runtime"] = json!("cargo test");
    let error =
        super::model_behavior::read_authority(&serde_json::to_vec(&invalid).unwrap()).unwrap_err();
    assert!(error
        .to_string()
        .contains("unreviewed executable case contract"));
}

fn bound_inputs() -> (
    super::model_behavior::Authority,
    Value,
    Value,
    Value,
    Value,
    Value,
) {
    let authority =
        super::model_behavior::read_authority(&serde_json::to_vec(&authority()).unwrap()).unwrap();
    let id = "ess-cli:bin:ess:load::accounting_tests::reading";
    let source_profile = json!({
        "source": {
            "crates/edge/ess-cli/src/main.rs": digest('a'),
            "crates/edge/ess-cli/src/load_accounting_tests.rs": digest('b'),
            "crates/edge/ess-cli/src/load.rs": digest('2')
        },
        "compiled_provider_source": {},
        "provider_executable_sha256": digest('9'),
        "compiled_build": {}
    });
    let profiles = json!({
        "ess_cli::bin(ess)::load::fn::specification": {
            "profile_sha256": digest('e'),
            "definition": {"entrypoints":["ess_cli::bin(ess)::load::fn::specification"]}
        }
    });
    let models = json!({
        "rust:type:ResolvedType.field.reading": digest('d')
    });
    let inventory = json!({
        "packages": {
            "ess-cli": {
                "entries": {
                    "ess_cli::bin(ess)::load::fn::specification": {
                        "source":"crates/edge/ess-cli/src/load.rs",
                        "declaration_sha256":digest('f'),
                        "source_item_sha256":digest('1')
                    }
                },
                "cases": {
                    id: {
                        "package":"ess-cli",
                        "target_kind":"bin",
                        "target_name":"ess",
                        "full_name":"load::accounting_tests::reading",
                        "source":"crates/edge/ess-cli/src/load_accounting_tests.rs",
                        "case_ast_sha256":digest('c'),
                        "required_features":[],
                        "ignored":false,
                        "contains_early_return":false,
                        "contains_catch_unwind":false,
                        "nested_command_candidates":[],
                        "assertion_candidates":["assert"]
                    }
                }
            }
        }
    });
    let metadata = json!({"packages":[{
        "name":"ess-cli",
        "targets":[{
            "name":"ess",
            "kind":["bin"],
            "required-features":[],
            "src_path":"/workspace/crates/edge/ess-cli/src/main.rs"
        }]
    }]});
    (
        authority,
        models,
        source_profile,
        profiles,
        inventory,
        metadata,
    )
}

#[test]
fn candidate_distinguishes_extractor_and_cargo_no_feature_encodings() {
    let (authority, models, source_profile, profiles, mut inventory, mut metadata) = bound_inputs();
    let id = "ess-cli:bin:ess:load::accounting_tests::reading";
    inventory["packages"]["ess-cli"]["cases"][id]["required_features"] = Value::Null;
    metadata["packages"][0]["targets"][0]
        .as_object_mut()
        .unwrap()
        .remove("required-features");
    assert!(super::model_behavior::candidate(
        &authority,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .is_ok());

    inventory["packages"]["ess-cli"]["cases"][id]["required_features"] = json!([]);
    metadata["packages"][0]["targets"][0]["required-features"] = json!([]);
    assert!(super::model_behavior::candidate(
        &authority,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .is_ok());

    for invalid in [json!(["unexpected"]), json!({})] {
        inventory["packages"]["ess-cli"]["cases"][id]["required_features"] = invalid;
        assert!(super::model_behavior::candidate(
            &authority,
            &models,
            &source_profile,
            &profiles,
            &inventory,
            &metadata,
        )
        .is_err());
    }
    inventory["packages"]["ess-cli"]["cases"][id]
        .as_object_mut()
        .unwrap()
        .remove("required_features");
    assert!(super::model_behavior::candidate(
        &authority,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .is_err());

    let (_, _, _, _, fresh_inventory, _) = bound_inputs();
    inventory = fresh_inventory;
    inventory["packages"]["ess-cli"]["cases"][id]["required_features"] = Value::Null;
    for invalid in [Value::Null, json!(["unexpected"]), json!({})] {
        metadata["packages"][0]["targets"][0]["required-features"] = invalid;
        assert!(super::model_behavior::candidate(
            &authority,
            &models,
            &source_profile,
            &profiles,
            &inventory,
            &metadata,
        )
        .is_err());
    }
}

#[test]
fn candidate_plan_and_execution_bind_every_digest_and_exact_receipt() {
    let (authority, models, source_profile, profiles, inventory, metadata) = bound_inputs();
    let candidate = super::model_behavior::candidate(
        &authority,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .unwrap();
    assert_eq!(candidate["stage"], "candidate");
    assert_eq!(candidate["qualified_cells"], 0);
    let plan = super::model_behavior::plan(
        &candidate,
        &authority,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .unwrap();
    let (id, case) = super::model_behavior::required_cases(&plan)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let receipt = super::model_behavior::Receipt {
        format: super::model_behavior::CASE_FORMAT.to_owned(),
        authority_sha256: plan["authority_sha256"].as_str().unwrap().to_owned(),
        plan_sha256: super::hash_json(&plan),
        identity: case.identity,
        target_source: case.target_source,
        target_source_file_sha256: case.target_source_file_sha256,
        source: case.source,
        source_file_sha256: case.source_file_sha256,
        case_ast_sha256: case.case_ast_sha256,
        native: json!({"copy":"/copy","original_device":1,"original_inode":2,"original_mode":493,"sha256":digest('3'),"size":1,"source":"/source"}),
        source_sha256: plan["source_sha256"].as_str().unwrap().to_owned(),
        provider_profile_sha256: plan["provider_profile_sha256"].as_str().unwrap().to_owned(),
        executed: 1,
        passed: 1,
        failed: 0,
        ignored: 0,
        nested_runtime: "none claimed".to_owned(),
    };
    let execution =
        super::model_behavior::executed(&plan, std::collections::BTreeMap::from([(id, receipt)]))
            .unwrap();
    let claims = super::model_behavior::verify_execution(&execution, &plan, &candidate).unwrap();
    assert_eq!(claims.len(), 1);

    let mut changed = execution.clone();
    changed["receipts"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()["passed"] = json!(0);
    assert!(super::model_behavior::verify_execution(&changed, &plan, &candidate).is_err());
    let mut stale = candidate.clone();
    stale["source_sha256"] = json!(digest('4'));
    assert!(super::model_behavior::plan(
        &stale,
        &authority,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .is_err());

    let mut changed_models = models.clone();
    changed_models["rust:type:ResolvedType.field.reading"] = json!(digest('6'));
    assert!(super::model_behavior::plan(
        &candidate,
        &authority,
        &changed_models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .is_err());
}

#[test]
fn candidate_rejects_a_stale_model_shape_claim() {
    let (original, models, source_profile, profiles, inventory, metadata) = bound_inputs();
    let mut stale = super::model_behavior::authority_value(&original);
    stale["claims"][0]["identity"]["shape"] = json!(digest('6'));
    let stale =
        super::model_behavior::read_authority(&serde_json::to_vec(&stale).unwrap()).unwrap();

    assert!(
        super::model_behavior::candidate(
            &stale,
            &models,
            &source_profile,
            &profiles,
            &inventory,
            &metadata,
        )
        .is_err(),
        "the diagnostic candidate must reject a reviewed claim whose model shape no longer matches the current extracted model inventory"
    );
}

#[test]
fn candidate_rejects_an_unknown_model_claim() {
    let (original, models, source_profile, profiles, inventory, metadata) = bound_inputs();
    let mut stale = super::model_behavior::authority_value(&original);
    stale["claims"][0]["identity"]["model"] = json!("rust:missing::Model");
    let stale =
        super::model_behavior::read_authority(&serde_json::to_vec(&stale).unwrap()).unwrap();

    assert!(
        super::model_behavior::candidate(
            &stale,
            &models,
            &source_profile,
            &profiles,
            &inventory,
            &metadata,
        )
        .is_err(),
        "the diagnostic candidate must reject a reviewed claim whose model is absent from the current extracted model inventory"
    );
}

#[test]
fn candidate_binds_a_wire_model_to_its_current_shape() {
    let (original, _, source_profile, profiles, inventory, metadata) = bound_inputs();
    let mut wire = super::model_behavior::authority_value(&original);
    wire["claims"][0]["identity"]["model"] = json!("wire:RawSpecFile#/definitions/Reading");
    wire["claims"][0]["identity"]["shape"] = json!(digest('7'));
    let wire = super::model_behavior::read_authority(&serde_json::to_vec(&wire).unwrap()).unwrap();
    let models = json!({"wire:RawSpecFile#/definitions/Reading": digest('7')});

    assert!(super::model_behavior::candidate(
        &wire,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .is_ok());

    let stale_models = json!({"wire:RawSpecFile#/definitions/Reading": digest('8')});
    assert!(super::model_behavior::candidate(
        &wire,
        &stale_models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .is_err());
}

#[test]
fn candidate_rejects_a_stale_claim_after_a_current_claim() {
    let (original, mut models, source_profile, profiles, inventory, metadata) = bound_inputs();
    let mut mixed = super::model_behavior::authority_value(&original);
    let mut stale = mixed["claims"][0].clone();
    stale["identity"]["model"] = json!("rust:type:ResolvedType.field.later");
    stale["identity"]["shape"] = json!(digest('7'));
    mixed["claims"].as_array_mut().unwrap().push(stale);
    let mixed =
        super::model_behavior::read_authority(&serde_json::to_vec(&mixed).unwrap()).unwrap();
    models["rust:type:ResolvedType.field.later"] = json!(digest('8'));

    let error = super::model_behavior::candidate(
        &mixed,
        &models,
        &source_profile,
        &profiles,
        &inventory,
        &metadata,
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("rust:type:ResolvedType.field.later"),
        "every reviewed claim must be checked against the current model inventory: {error:#}"
    );
}
