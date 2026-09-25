use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn digest(character: char) -> String {
    std::iter::repeat_n(character, 64).collect()
}

fn model_candidate() -> serde_json::Value {
    let id = "ess-cli:bin:ess:load::accounting_tests::reading";
    json!({
        "format":"ess-consumer-model-behavior-execution/1",
        "stage":"candidate",
        "authority_sha256":digest('a'),
        "source_sha256":digest('b'),
        "provider_profile_sha256":digest('c'),
        "selected_cases":[id],
        "cases":{id:{
            "identity":{"package":"ess-cli","target_kind":"bin","target_name":"ess","full_name":"load::accounting_tests::reading","features":[],"target_profile":"x86_64-unknown-linux-gnu/default","tool_requirements":["frozen Rust 1.98.1; locked offline owner-target build"],"nested_runtime":"None claimed; direct Rust assertions only"},
            "target_source":"crates/edge/ess-cli/src/main.rs","target_source_file_sha256":digest('d'),
            "source":"crates/edge/ess-cli/src/load_accounting_tests.rs","source_file_sha256":digest('e'),"case_ast_sha256":digest('f'),
            "reviewed_assertion":"reading survives","attribution_limit":"loader only","source_evidence":"source","review_reference":"review"
        }},
        "claims":[{
            "identity":{"model":"model","shape":digest('1'),"consumer":"consumer","profile":digest('2')},
            "cases":[id],"entrypoint":"entry","entrypoint_sha256":digest('3'),"entrypoint_source":"source.rs","entrypoint_ast_sha256":digest('4'),"entrypoint_source_file_sha256":digest('5'),
            "behavior":{"kind":"Supported","reason":"observed"},"source_evidence":"source","review_reference":"review"
        }],
        "qualified_cells":0
    })
}

#[test]
fn historical_accounting_readers_reject_v3_without_changing_v1_or_v2() {
    let v1 = json!({"format":"ess-consumer-accounting/1","stage":"candidate","cells":[]});
    let v2 = json!({"format":"ess-consumer-accounting/2","stage":"candidate","cells":[],"reconciliation_sha256":"r","scenario_acquisition_format":"ess-consumer-scenario-acquisition/2","acquisition_rows":8,"aggregate_rows":0});
    let v3 = json!({"format":"ess-consumer-accounting/3","stage":"candidate","cells":[],"reconciliation_sha256":"r","scenario_acquisition_format":"ess-consumer-scenario-acquisition/2","acquisition_rows":8,"aggregate_rows":0,"model_behavior_candidate":model_candidate()});

    super::account::read_candidates(&v1).unwrap();
    super::account::read_candidates_v2(&v2).unwrap();
    assert!(super::account::read_candidates(&v3).is_err());
    assert!(super::account::read_candidates_v2(&v3).is_err());
    assert!(super::account::read_candidates_v3(&v1).is_err());
    assert!(super::account::read_candidates_v3(&v2).is_err());
    assert!(super::account::read_candidates_v3(&v3).is_ok());
}

#[test]
fn v3_candidate_reader_is_closed_and_requires_model_behavior_evidence() {
    let candidate = json!({"format":"ess-consumer-accounting/3","stage":"candidate","cells":[],"reconciliation_sha256":"r","scenario_acquisition_format":"ess-consumer-scenario-acquisition/2","acquisition_rows":8,"aggregate_rows":0,"model_behavior_candidate":model_candidate()});
    super::account::read_candidates_v3(&candidate).unwrap();
    for pointer in ["/format", "/stage", "/scenario_acquisition_format"] {
        let mut changed = candidate.clone();
        *changed.pointer_mut(pointer).unwrap() = json!("unknown");
        assert!(
            super::account::read_candidates_v3(&changed).is_err(),
            "{pointer}"
        );
    }
    let mut missing = candidate.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("model_behavior_candidate");
    assert!(super::account::read_candidates_v3(&missing).is_err());
    let mut incomplete = candidate.clone();
    incomplete["model_behavior_candidate"] = json!({});
    assert!(super::account::read_candidates_v3(&incomplete).is_err());
    let mut nested_extra = candidate.clone();
    nested_extra["model_behavior_candidate"]["extra"] = json!(true);
    assert!(super::account::read_candidates_v3(&nested_extra).is_err());
    let mut extra = candidate;
    extra["extra"] = json!(true);
    assert!(super::account::read_candidates_v3(&extra).is_err());
}

#[test]
fn legacy_native_receipt_shape_remains_byte_for_byte_unchanged() {
    let identity = json!({
        "package":"fixture",
        "target_kind":"Test",
        "target_name":"owner",
        "full_name":"case",
        "features":[],
        "target_profile":"x86_64-unknown-linux-gnu/default",
        "tool_requirements":["frozen Rust 1.98.1; locked offline owner-target build"],
        "nested_runtime":"None claimed; direct Rust assertions only"
    });
    let native = json!({"source":"/source","copy":"/copy","sha256":"digest","size":1,"original_device":2,"original_inode":3,"original_mode":493});
    let receipt = super::native::test_legacy_receipt(&identity, &native, "source", "profile");
    assert_eq!(
        serde_json::to_string(&receipt).unwrap(),
        r#"{"executed":1,"failed":0,"identity":{"features":[],"full_name":"case","nested_runtime":"None claimed; direct Rust assertions only","package":"fixture","target_kind":"Test","target_name":"owner","target_profile":"x86_64-unknown-linux-gnu/default","tool_requirements":["frozen Rust 1.98.1; locked offline owner-target build"]},"ignored":0,"native":{"copy":"/copy","original_device":2,"original_inode":3,"original_mode":493,"sha256":"digest","size":1,"source":"/source"},"nested_runtime":"none claimed","passed":1,"provider_profile_sha256":"profile","source_sha256":"source"}"#
    );
}

#[test]
fn v3_requires_an_exact_disjoint_legacy_and_model_case_partition() {
    super::enforce::test_case_partition(&["legacy", "model"], &["legacy"], &["model"]).unwrap();
    assert!(super::enforce::test_case_partition(&["shared"], &["shared"], &["shared"]).is_err());
    assert!(super::enforce::test_case_partition(
        &["legacy", "model", "extra"],
        &["legacy"],
        &["model"]
    )
    .is_err());
}

struct V3Fixture {
    plan: Value,
    model_candidate: Value,
    model_execution: Value,
    models: Value,
    profiles: Value,
}

#[expect(
    clippy::too_many_lines,
    reason = "the finite v3 fixture keeps its exact nine-cell conservation visible"
)]
fn v3_fixture() -> V3Fixture {
    let metadata_manifest: Value =
        serde_json::from_str(include_str!("reviewed-schema-metadata.json")).unwrap();
    let mut models = serde_json::Map::new();
    let mut profiles = serde_json::Map::new();
    for row in metadata_manifest["rows"].as_array().unwrap() {
        models.insert(
            row["model"].as_str().unwrap().to_owned(),
            row["shape"].clone(),
        );
        profiles.insert(
            row["consumer"].as_str().unwrap().to_owned(),
            row["profile"].clone(),
        );
    }
    let aggregate_model = "rust:ess_domain::command::RawOutcome";
    let aggregate_shape = "f167bcf3a4fd9ea3f40d41885bf457433f1601dd39353efb7883626a38c4b160";
    let aggregate_old_shape = "a2072ea0863f3bfdaa92b6ab4b30eb1857bf23af0538db9747ea3b3efc3505b0";
    models.insert(aggregate_model.to_owned(), json!(aggregate_shape));
    let models = Value::Object(models);
    let profiles = Value::Object(profiles);
    let metadata = super::metadata::candidates(&models, &profiles).unwrap();

    let aggregate_consumer = "cli-binding-resolution";
    let model_consumer = "cli-binding-rust-emission";
    let second_model_consumer = "cli-binding-process-execution";
    let aggregate_profile = profiles[aggregate_consumer].as_str().unwrap();
    let model_profile = profiles[model_consumer].as_str().unwrap();
    let old = super::reconciliation::Identity {
        model: aggregate_model.to_owned(),
        shape: aggregate_old_shape.to_owned(),
        consumer: aggregate_consumer.to_owned(),
        profile: aggregate_profile.to_owned(),
    };
    let current = super::reconciliation::Identity {
        model: aggregate_model.to_owned(),
        shape: aggregate_shape.to_owned(),
        consumer: aggregate_consumer.to_owned(),
        profile: aggregate_profile.to_owned(),
    };
    let unknown = super::reconciliation::Unknown {
        identity: old.clone(),
        owner: "test-owner".to_owned(),
        follow_up: "test-follow-up".to_owned(),
        reason: "test retained aggregate residual".to_owned(),
    };
    let replacement = super::reconciliation::Replacement {
        current: current.clone(),
        claim: super::reconciliation::ReplacementClaim::AggregateClosure {
            closure: "raw-outcome-closure".to_owned(),
        },
        unknown,
    };
    let reconciliation = super::reconciliation::Resolution {
        unchanged: Vec::new(),
        replacements: BTreeMap::from([(old.clone(), replacement)]),
        retired: BTreeSet::new(),
        digest: "test-reconciliation".to_owned(),
        receipt: json!({
            "reconciliation_sha256":"test-reconciliation",
            "retirements":[],
        }),
    };
    let aggregate_authority = json!({
        "format":super::aggregate::FORMAT,
        "rows":[{
            "id":"raw-outcome-closure",
            "mode":"ShapeDelta",
            "old":old,
            "current":current,
            "baseline_sha256":"test-baseline",
            "reconciliation_sha256":"test-reconciliation",
            "residual":{
                "owner":"test-owner",
                "follow_up":"test-follow-up",
                "reason":"test retained aggregate residual",
            },
            "frontier":[{
                "path":"rust:ess_domain::command::RawOutcome/field/when_subject_state",
                "disposition":{"kind":"Supported","cases":["aggregate-case"]},
            }],
            "source_evidence":"test-only aggregate fixture",
            "review_reference":"test-only aggregate fixture",
        }],
    });

    let id = "ess-cli:bin:ess:load::accounting_tests::reading";
    let model_case = model_candidate()["cases"][id].clone();
    let model_claim = json!({
        "identity":{
            "model":aggregate_model,
            "shape":aggregate_shape,
            "consumer":model_consumer,
            "profile":model_profile,
        },
        "cases":[id],
        "entrypoint":"entry",
        "entrypoint_sha256":digest('3'),
        "entrypoint_source":"source.rs",
        "entrypoint_ast_sha256":digest('4'),
        "entrypoint_source_file_sha256":digest('5'),
        "behavior":{"kind":"Supported","reason":"test model behavior"},
        "source_evidence":"test-only model fixture",
        "review_reference":"test-only model fixture",
    });
    let mut second_model_claim = model_claim.clone();
    second_model_claim["identity"]["consumer"] = json!(second_model_consumer);
    second_model_claim["identity"]["profile"] = profiles[second_model_consumer].clone();
    second_model_claim["behavior"]["reason"] = json!("second test model behavior");
    let model_candidate = json!({
        "format":"ess-consumer-model-behavior-execution/1",
        "stage":"candidate",
        "authority_sha256":digest('a'),
        "source_sha256":digest('b'),
        "provider_profile_sha256":digest('c'),
        "selected_cases":[id],
        "cases":{id:model_case},
        "claims":[model_claim,second_model_claim],
        "qualified_cells":0,
    });
    let mut model_plan = model_candidate.clone();
    model_plan["stage"] = json!("execution-plan");
    model_plan["candidate_sha256"] = json!(super::hash_json(&model_candidate));
    let (case_id, case) = super::model_behavior::required_cases(&model_plan)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let receipt = super::model_behavior::Receipt {
        format: super::model_behavior::CASE_FORMAT.to_owned(),
        authority_sha256: model_plan["authority_sha256"].as_str().unwrap().to_owned(),
        plan_sha256: super::hash_json(&model_plan),
        identity: case.identity,
        target_source: case.target_source,
        target_source_file_sha256: case.target_source_file_sha256,
        source: case.source,
        source_file_sha256: case.source_file_sha256,
        case_ast_sha256: case.case_ast_sha256,
        native: json!({
            "copy":"/test-copy",
            "original_device":1,
            "original_inode":2,
            "original_mode":493,
            "sha256":digest('6'),
            "size":1,
            "source":"/test-source",
        }),
        source_sha256: model_plan["source_sha256"].as_str().unwrap().to_owned(),
        provider_profile_sha256: model_plan["provider_profile_sha256"]
            .as_str()
            .unwrap()
            .to_owned(),
        executed: 1,
        passed: 1,
        failed: 0,
        ignored: 0,
        nested_runtime: "none claimed".to_owned(),
    };
    let model_execution =
        super::model_behavior::executed(&model_plan, BTreeMap::from([(case_id, receipt)])).unwrap();
    let legacy_claims = json!([]);
    let acquisition_plan = json!({
        "required_rows":8,
        "qualified_rows":0,
        "status":"PENDING_CASE_AND_GUARD_EXECUTION",
        "authority_sha256":"test-acquisition",
    });
    let plan = super::enforce::plan_v3(super::enforce::PlanV3Inputs {
        accounting: super::enforce::PlanV2Inputs {
            models: &models,
            profiles: &profiles,
            claims: &legacy_claims,
            metadata: &metadata,
            reconciliation: &reconciliation,
            baseline_sha256: "test-baseline",
            acquisition_plan: &acquisition_plan,
            aggregate_authority: &aggregate_authority,
        },
        model_behavior_plan: &model_plan,
    })
    .unwrap();
    V3Fixture {
        plan,
        model_candidate,
        model_execution,
        models,
        profiles,
    }
}

#[test]
fn v3_plan_reader_consumes_model_claims_once_and_keeps_case_sets_disjoint() {
    let fixture = v3_fixture();
    super::enforce::test_read_plan_v3(&fixture.plan).unwrap();

    let mut missing = fixture.plan.clone();
    missing["model_behavior_plan"]["claims"]
        .as_array_mut()
        .unwrap()
        .remove(1);
    assert!(super::enforce::test_read_plan_v3(&missing).is_err());

    let mut unconsumed = fixture.plan.clone();
    unconsumed["cells"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|cell| {
            cell["model"] == "rust:ess_domain::command::RawOutcome"
                && cell["consumer"] == "cli-binding-rust-emission"
        })
        .unwrap()["disposition"]["reason"] = json!("changed model reason");
    assert!(super::enforce::test_read_plan_v3(&unconsumed).is_err());

    let mut mixed = fixture.plan.clone();
    mixed["cells"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|cell| {
            cell["model"] == "rust:ess_domain::command::RawOutcome"
                && cell["consumer"] == "cli-binding-rust-emission"
        })
        .unwrap()["disposition"]["cases"]
        .as_array_mut()
        .unwrap()
        .push(json!("aggregate-case"));
    assert!(super::enforce::test_read_plan_v3(&mixed).is_err());

    let mut extra = fixture.plan.clone();
    let mut claim = extra["model_behavior_plan"]["claims"][0].clone();
    claim["identity"]["consumer"] = json!("unaccounted-consumer");
    extra["model_behavior_plan"]["claims"]
        .as_array_mut()
        .unwrap()
        .push(claim);
    assert!(super::enforce::test_read_plan_v3(&extra).is_err());

    let mut overlapping_model_plan = fixture.plan["model_behavior_plan"].clone();
    let model_case = overlapping_model_plan["cases"]
        .as_object_mut()
        .unwrap()
        .remove("ess-cli:bin:ess:load::accounting_tests::reading")
        .unwrap();
    overlapping_model_plan["cases"] = json!({"aggregate-case":model_case});
    overlapping_model_plan["selected_cases"] = json!(["aggregate-case"]);
    for claim in overlapping_model_plan["claims"].as_array_mut().unwrap() {
        claim["cases"] = json!(["aggregate-case"]);
    }
    let mut overlapping = fixture.plan;
    overlapping["model_behavior_plan"] = overlapping_model_plan;
    assert!(super::enforce::test_read_plan_v3(&overlapping).is_err());
}

fn current_source_profile(root: &Path) -> Value {
    let source = serde_json::to_value(super::source_files(root).unwrap()).unwrap();
    let compiled_source: Value = serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/consumer-source.json"
    )))
    .unwrap();
    let compiled_build: Value = serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/consumer-build.json"
    )))
    .unwrap();
    assert_eq!(source, compiled_source);
    json!({
        "source":source,
        "compiled_provider_source":compiled_source,
        "provider_executable_sha256":super::hash_bytes(&std::fs::read(std::env::current_exe().unwrap()).unwrap()),
        "compiled_build":compiled_build,
        "current_invocation":super::invocation(root, &compiled_build).unwrap(),
    })
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one qualification test mutates each independently retained proof"
)]
fn v3_qualification_consumes_exact_model_and_retained_v2_proofs() {
    let fixture = v3_fixture();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap();
    let source_profile = current_source_profile(root);
    let authority = super::metadata::Authority::capture(root, &source_profile).unwrap();
    let schema =
        serde_json::to_value(schemars::schema_for!(ess_domain::spec::RawSpecFile)).unwrap();
    let wire = super::wire::extract(&schema).unwrap();
    let metadata = super::metadata::execute(
        &authority,
        &schema,
        &wire,
        &fixture.models,
        &fixture.profiles,
    )
    .unwrap();
    let legacy_cases: BTreeSet<String> =
        serde_json::from_value(fixture.plan["required_legacy_cases"].clone()).unwrap();
    let verified = super::native::test_verified_cases(
        legacy_cases.clone(),
        json!({"test_only_synthetic_legacy_receipts":true}),
    );
    let acquisition = json!({
        "format":super::scenario_acquisition::FORMAT,
        "qualified_rows":8,
        "guard_executed":1,
    });
    let aggregate = json!({
        "format":super::aggregate::FORMAT,
        "aggregate_manifest_sha256":fixture.plan["aggregate_manifest_sha256"],
        "qualified_closures":1,
    });
    let qualify = |verified, acquisition: &Value, aggregate: &Value, execution: &Value| {
        super::enforce::qualify_v3(
            &fixture.plan,
            verified,
            &authority,
            &metadata,
            acquisition,
            aggregate,
            &fixture.model_candidate,
            execution,
        )
    };
    let qualified = qualify(
        &verified,
        &acquisition,
        &aggregate,
        &fixture.model_execution,
    )
    .unwrap();
    assert_eq!(qualified["format"], "ess-consumer-accounting/3");
    assert_eq!(qualified["counts"]["AggregateClosure"], 1);
    assert_eq!(qualified["executed_case_count"], legacy_cases.len() + 1);

    let missing_legacy = super::native::test_verified_cases(
        legacy_cases.iter().skip(1).cloned().collect(),
        json!({"test_only_synthetic_legacy_receipts":true}),
    );
    assert!(qualify(
        &missing_legacy,
        &acquisition,
        &aggregate,
        &fixture.model_execution,
    )
    .is_err());

    let mut changed_execution = fixture.model_execution.clone();
    changed_execution["receipts"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap()["passed"] = json!(0);
    assert!(qualify(&verified, &acquisition, &aggregate, &changed_execution).is_err());

    let mut incomplete_acquisition = acquisition.clone();
    incomplete_acquisition["qualified_rows"] = json!(7);
    assert!(qualify(
        &verified,
        &incomplete_acquisition,
        &aggregate,
        &fixture.model_execution,
    )
    .is_err());

    let mut incomplete_aggregate = aggregate.clone();
    incomplete_aggregate["qualified_closures"] = json!(0);
    assert!(qualify(
        &verified,
        &acquisition,
        &incomplete_aggregate,
        &fixture.model_execution,
    )
    .is_err());
    let mut wrong_aggregate = aggregate;
    wrong_aggregate["aggregate_manifest_sha256"] = json!("wrong");
    assert!(qualify(
        &verified,
        &acquisition,
        &wrong_aggregate,
        &fixture.model_execution,
    )
    .is_err());
}
