use super::*;

fn fixture() -> (Value, metadata::Candidates) {
    let schema =
        serde_json::to_value(schemars::schema_for!(ess_domain::spec::RawSpecFile)).unwrap();
    let wire = super::super::wire::extract(&schema).unwrap();
    let manifest: Value =
        serde_json::from_slice(include_bytes!("reviewed-schema-metadata.json")).unwrap();
    let mut models = json!({"fixture:m":"shape-m","fixture:n":"shape-n"});
    let mut profiles = serde_json::Map::new();
    for row in manifest["rows"].as_array().unwrap() {
        let id = row["model"].as_str().unwrap();
        models[id] = wire["obligations"][id].clone();
        profiles.insert(
            row["consumer"].as_str().unwrap().to_owned(),
            row["profile"].clone(),
        );
    }
    let candidates = metadata::candidates(&models, &json!(profiles)).unwrap();
    let groups=profiles.iter().map(|(consumer,profile)|json!({
        "consumer":consumer,"profile":profile,"exact_model_ids":["fixture:m"],
        "owner":"fixture-owner","follow_up":"story:fixture-follow-up","reason":"Existing unknown remains unproven."
    })).collect::<Vec<_>>();
    let baseline = json!({
        "format":"ess-consumer-initial-eligibility/1","status":"ACCEPTED_INITIAL_ELIGIBILITY",
        "eligible_pairs":3,"mandatory_pairs_excluded":9,"stage1_source_commit":"fixture",
        "stage1_binding_sha256":"fixture","source_checkpoint_sha256":"fixture","extraction_profile":"fixture",
        "models":models,"groups":groups
    });
    let claims=profiles.keys().enumerate().map(|(i,consumer)| {
        let mut row=json!({"model":"fixture:n","consumer":consumer,"cases":[if i==0{"case-refusal"}else{"case-success"}],
            "kind":if i==0{"Refused"}else{"Supported"},"reason":"Exact fixture behavior"});
        if i==0 {row["refusal"]=json!("named fixture refusal");}
        row
    }).collect::<Vec<_>>();
    let plan = plan_with_metadata(
        &models,
        &json!(profiles),
        &baseline,
        &json!(claims),
        Some(&candidates),
    )
    .unwrap();
    (plan, candidates)
}
#[test]
fn metadata_planning_and_qualified_counts_conserve_every_cell_and_keep_cases_separate() {
    let (value, candidates) = fixture();
    let plan = read_plan(&value).unwrap();
    assert_eq!(value["format"], account::FORMAT);
    assert_eq!(value["stage"], "execution-plan");
    assert_eq!(value["SchemaDocumentMetadata"], 0);
    assert_eq!(value["pending_metadata_candidates"], 6);
    assert_eq!(plan.required_cases.len(), 2);
    assert_eq!(plan.required_metadata_guards, candidates.rows().clone());
    // Accounting-only fixture: actual guard execution has its own current-provider test.
    let result = qualify_cells(
        &plan,
        &plan.required_cases,
        &json!({"fixture":"two exact cases"}),
        &json!({"fixture":"one guard"}),
    )
    .unwrap();
    assert_eq!(result["stage"], "qualified");
    assert_eq!(
        result["counts"],
        json!({"Supported":2,"Refused":1,"BaselineUnknown":3,"SchemaDocumentMetadata":6})
    );
    assert_eq!(result["cells"].as_array().unwrap().len(), 12);
    assert_eq!(result["executed_case_count"], 2);
    assert_eq!(result["executed_metadata_guards"], 1);
    assert!(qualify_cells(&plan, &BTreeSet::new(), &Value::Null, &Value::Null).is_err());
}
#[test]
fn accounting_readers_reject_legacy_unknown_envelopes_and_wrong_stages() {
    let (value, _) = fixture();
    let mut absent = value.clone();
    absent.as_object_mut().unwrap().remove("format");
    assert!(read_plan(&absent).is_err());
    for (field, replacement) in [
        ("format", json!("ess-consumer-accounting/2")),
        ("stage", json!("candidate")),
        ("stage", json!("qualified")),
        ("stage", json!("unknown")),
        ("unreviewed", json!(true)),
    ] {
        let mut bad = value.clone();
        bad[field] = replacement;
        assert!(read_plan(&bad).is_err(), "{field}");
    }
    let candidate = json!({"format":account::FORMAT,"stage":"candidate","cells":[]});
    account::read_candidates(&candidate).unwrap();
    for bad in [
        json!([]),
        json!({"stage":"candidate","cells":[]}),
        json!({"format":"unknown","stage":"candidate","cells":[]}),
        json!({"format":account::FORMAT,"stage":"execution-plan","cells":[]}),
        json!({"format":account::FORMAT,"stage":"qualified","cells":[]}),
    ] {
        assert!(account::read_candidates(&bad).is_err());
    }
}
#[test]
fn execution_plan_rejects_missing_duplicate_changed_or_premature_guard_claims() {
    let (value, _) = fixture();
    for key in [
        "required_metadata_guards",
        "metadata_manifest_sha256",
        "pending_metadata_candidates",
    ] {
        let mut bad = value.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(read_plan(&bad).is_err(), "{key}");
    }
    let mut missing = value.clone();
    missing["required_metadata_guards"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(read_plan(&missing).is_err());
    let mut duplicate = value.clone();
    duplicate["required_metadata_guards"]
        .as_array_mut()
        .unwrap()
        .push(value["required_metadata_guards"][0].clone());
    assert!(read_plan(&duplicate).is_err());
    for field in [
        "shape",
        "profile",
        "guard",
        "guard_source_sha256",
        "consumer",
        "model",
    ] {
        let mut bad = value.clone();
        bad["required_metadata_guards"][0][field] = json!("changed");
        assert!(read_plan(&bad).is_err(), "{field}");
    }
    for key in ["Supported", "Refused", "SchemaDocumentMetadata"] {
        let mut bad = value.clone();
        bad[key] = json!(1);
        assert!(read_plan(&bad).is_err());
    }
    let mut bad = value.clone();
    bad["required_cases"]
        .as_array_mut()
        .unwrap()
        .push(value["required_cases"][0].clone());
    assert!(read_plan(&bad).is_err());
}
#[test]
fn metadata_cells_reject_behavior_fields_and_old_closed_disposition_readers_reject_metadata() {
    #[derive(Deserialize)]
    #[serde(tag = "kind", deny_unknown_fields)]
    enum OldDisposition {
        Supported {
            cases: Vec<String>,
            reason: String,
        },
        Refused {
            cases: Vec<String>,
            reason: String,
            refusal: String,
        },
        BaselineUnknown {
            owner: String,
            follow_up: String,
            reason: String,
        },
    }
    let (value, _) = fixture();
    let index = value["cells"]
        .as_array()
        .unwrap()
        .iter()
        .position(|cell| cell["disposition"]["kind"] == "SchemaDocumentMetadata")
        .unwrap();
    let disposition = value["cells"][index]["disposition"].clone();
    assert!(serde_json::from_value::<OldDisposition>(disposition).is_err());
    // Exercise old reader fields with old values so this test does not add dead-code exceptions.
    for (input, expected) in [
        (
            json!({"kind":"Supported","cases":["case"],"reason":"reason"}),
            3,
        ),
        (
            json!({"kind":"Refused","cases":["case"],"reason":"reason","refusal":"refusal"}),
            10,
        ),
        (
            json!({"kind":"BaselineUnknown","owner":"owner","follow_up":"follow","reason":"reason"}),
            17,
        ),
    ] {
        let count = match serde_json::from_value::<OldDisposition>(input).unwrap() {
            OldDisposition::Supported { cases, reason } => cases.len() + reason.len() - 4,
            OldDisposition::Refused {
                cases,
                reason,
                refusal,
            } => cases.len() + reason.len() + refusal.len() - 4,
            OldDisposition::BaselineUnknown {
                owner,
                follow_up,
                reason,
            } => owner.len() + follow_up.len() + reason.len(),
        };
        assert_eq!(count, expected);
    }
    for field in ["cases", "refusal", "owner"] {
        let mut bad = value.clone();
        bad["cells"][index]["disposition"][field] = json!("unsupported");
        assert!(read_plan(&bad).is_err(), "{field}");
    }
}
#[test]
fn a_metadata_container_cannot_absorb_a_new_descendant_obligation() {
    let (value, _) = fixture();
    let mut bad = value.clone();
    bad["discovered_models"] = json!(5);
    assert!(read_plan(&bad).is_err());
    let mut mismatched = value;
    let index = mismatched["cells"]
        .as_array()
        .unwrap()
        .iter()
        .position(|cell| cell["disposition"]["kind"] == "SchemaDocumentMetadata")
        .unwrap();
    mismatched["cells"][index]["model"] =
        json!("wire:RawSpecFile#/definitions/Field/properties/new_child");
    assert!(read_plan(&mismatched).is_err());
}
