use serde_json::{json, Value};
use std::collections::BTreeMap;

fn baseline() -> Value {
    json!({
        "format":"ess-consumer-initial-eligibility/1",
        "status":"ACCEPTED_INITIAL_ELIGIBILITY",
        "eligible_pairs":6,
        "mandatory_pairs_excluded":0,
        "stage1_source_commit":"old-source",
        "stage1_binding_sha256":"old-binding",
        "source_checkpoint_sha256":"old-checkpoint",
        "extraction_profile":"fixture",
        "models":{"changed":"old-shape","removed":"removed-shape","stable":"stable-shape"},
        "groups":[
            {"consumer":"stable-consumer","profile":"stable-profile","exact_model_ids":["changed","removed","stable"],"owner":"owner","follow_up":"epic:unknowns","reason":"visible unknown"},
            {"consumer":"acquisition-authored-manifest","profile":"old-acquisition-profile","exact_model_ids":["changed","removed","stable"],"owner":"owner","follow_up":"epic:unknowns","reason":"misclassified acquisition"}
        ]
    })
}

fn old(model: &str, shape: &str, consumer: &str, profile: &str) -> Value {
    json!({"model":model,"shape":shape,"consumer":consumer,"profile":profile})
}

fn reconciliation() -> Value {
    json!({
        "format":"ess-consumer-reconciliation/1",
        "baseline_sha256":"fixture-baseline",
        "source_evidence":"local-evidence:fixture",
        "review_reference":"review-result:fixture",
        "decisions":[
            {"old":old("changed","old-shape","stable-consumer","stable-profile"),"source_evidence":"source","review_reference":"review","action":{"kind":"Replace","current":old("changed","new-shape","stable-consumer","stable-profile"),"claim":{"kind":"Supported","cases":["exact-case"],"reason":"exact current behavior"}}},
            {"old":old("removed","removed-shape","stable-consumer","stable-profile"),"source_evidence":"source","review_reference":"review","action":{"kind":"RetireModel"}},
            {"old":old("changed","old-shape","acquisition-authored-manifest","old-acquisition-profile"),"source_evidence":"source","review_reference":"review","action":{"kind":"RetireConsumerProfile"}},
            {"old":old("removed","removed-shape","acquisition-authored-manifest","old-acquisition-profile"),"source_evidence":"source","review_reference":"review","action":{"kind":"RetireConsumerProfile"}},
            {"old":old("stable","stable-shape","acquisition-authored-manifest","old-acquisition-profile"),"source_evidence":"source","review_reference":"review","action":{"kind":"RetireConsumerProfile"}}
        ]
    })
}

#[test]
fn historical_v1_readers_reject_v2_and_every_new_surface() {
    let v2 = json!({"format":"ess-consumer-accounting/2","stage":"candidate","cells":[],"reconciliation":{},"scenario_acquisition":[]});
    assert!(super::account::read_candidates(&v2).is_err());
    let mut v1 = json!({"format":"ess-consumer-accounting/1","stage":"candidate","cells":[]});
    v1["scenario_acquisition"] = json!([]);
    assert!(super::account::read_candidates(&v1).is_err());
    v1.as_object_mut().unwrap().remove("scenario_acquisition");
    v1["cells"] = json!([{"model":"m","shape":"s","consumer":"c","profile":"p","status":"UNACCEPTED","disposition":{"kind":"AggregateClosure","closure":"x"}}]);
    assert!(super::account::read_candidates(&v1).is_err());
}

#[test]
fn v2_readers_are_closed_by_format_stage_disposition_and_fields() {
    let candidate = json!({"format":"ess-consumer-accounting/2","stage":"candidate","cells":[],"reconciliation_sha256":"r","scenario_acquisition_format":"ess-consumer-scenario-acquisition/2","acquisition_rows":8,"aggregate_rows":0});
    super::account::read_candidates_v2(&candidate).unwrap();
    for pointer in ["/format", "/stage", "/scenario_acquisition_format"] {
        let mut bad = candidate.clone();
        *bad.pointer_mut(pointer).unwrap() = json!("unknown");
        assert!(
            super::account::read_candidates_v2(&bad).is_err(),
            "{pointer}"
        );
    }
    let mut bad = candidate;
    bad["extra"] = json!(true);
    assert!(super::account::read_candidates_v2(&bad).is_err());
}

#[test]
fn reconciliation_consumes_the_complete_stale_set_once_and_preserves_one_unknown() {
    let result = super::reconciliation::validate(
        &baseline(),
        "fixture-baseline",
        &json!({"changed":"new-shape","stable":"stable-shape"}),
        &json!({"stable-consumer":"stable-profile"}),
        &json!({"acquisition-authored-manifest":"new-acquisition-profile"}),
        &reconciliation(),
    )
    .unwrap();
    assert_eq!(result["unchanged_unknowns"], 1);
    assert_eq!(result["replacements"], 1);
    assert_eq!(result["retirements"], 4);
    assert_eq!(result["consumed_decisions"], 5);
}

#[test]
fn reconciliation_refuses_missing_extra_duplicate_wrong_and_revived_decisions() {
    let args = || {
        (
            baseline(),
            json!({"changed":"new-shape","stable":"stable-shape"}),
            json!({"stable-consumer":"stable-profile"}),
            json!({"acquisition-authored-manifest":"new-acquisition-profile"}),
        )
    };
    for mutate in 0..9 {
        let (baseline, models, profiles, acquisitions) = args();
        let mut authority = reconciliation();
        match mutate {
            0 => { authority["decisions"].as_array_mut().unwrap().pop(); }
            1 => authority["decisions"].as_array_mut().unwrap().push(json!({"old":old("ghost","shape","stable-consumer","stable-profile"),"source_evidence":"source","review_reference":"review","action":{"kind":"RetireModel"}})),
            2 => { let row = authority["decisions"][0].clone(); authority["decisions"].as_array_mut().unwrap().push(row); }
            3 => authority["baseline_sha256"] = json!("wrong"),
            4 => authority["decisions"][0]["old"]["shape"] = json!("wrong"),
            5 => authority["decisions"][0]["action"]["current"]["shape"] = json!("wrong"),
            6 => authority["decisions"][1]["action"] = json!({"kind":"Replace","current":old("removed","revived","stable-consumer","stable-profile"),"claim":{"kind":"Supported","cases":["case"],"reason":"reason"}}),
            7 => {
                authority["decisions"][0]["action"]
                    .as_object_mut()
                    .unwrap()
                    .remove("claim");
            }
            8 => authority["decisions"][0]["action"]["claim"]["cases"] = json!([]),
            _ => unreachable!(),
        }
        assert!(
            super::reconciliation::validate(
                &baseline,
                "fixture-baseline",
                &models,
                &profiles,
                &acquisitions,
                &authority
            )
            .is_err(),
            "mutation {mutate}"
        );
    }
}

#[test]
fn replacement_binding_requires_the_exact_kind_cases_reason_refusal_or_closure() {
    let supported = json!({"kind":"Supported","cases":["case-a"],"reason":"reviewed"});
    assert!(super::enforce::test_replacement_matches(&supported, &supported).unwrap());
    for changed in [
        json!({"kind":"Supported","cases":["case-b"],"reason":"reviewed"}),
        json!({"kind":"Supported","cases":["case-a"],"reason":"changed"}),
        json!({"kind":"Refused","cases":["case-a"],"reason":"reviewed","refusal":"boundary"}),
        json!({"kind":"BaselineUnknown","owner":"owner","follow_up":"epic:x","reason":"unknown"}),
    ] {
        assert!(!super::enforce::test_replacement_matches(&supported, &changed).unwrap());
    }

    let refused =
        json!({"kind":"Refused","cases":["case-a"],"reason":"reviewed","refusal":"boundary"});
    assert!(super::enforce::test_replacement_matches(&refused, &refused).unwrap());
    let changed_refusal =
        json!({"kind":"Refused","cases":["case-a"],"reason":"reviewed","refusal":"other"});
    assert!(!super::enforce::test_replacement_matches(&refused, &changed_refusal).unwrap());

    let aggregate = json!({"kind":"AggregateClosure","closure":"closure-a"});
    assert!(super::enforce::test_replacement_matches(&aggregate, &aggregate).unwrap());
    assert!(!super::enforce::test_replacement_matches(
        &aggregate,
        &json!({"kind":"AggregateClosure","closure":"closure-b"})
    )
    .unwrap());
}

fn acquisition_rows() -> Vec<Value> {
    let roles = [("authored", "Authored"), ("coverage", "Coverage")];
    let modes = [
        ("manifest", "Manifest"),
        ("legacy-directory", "LegacyDirectory"),
        ("direct-file", "DirectFile"),
        ("omitted-scenarios", "Omitted"),
    ];
    let mut rows = Vec::new();
    for (role_id, role) in roles {
        for (mode_id, mode) in modes {
            let id = format!("acquisition-{role_id}-{mode_id}");
            rows.push(json!({
                "profile":id,
                "profile_sha256":format!("profile-{role_id}-{mode_id}"),
                "entrypoint":"ess_cli::bin(ess)::input_discovery::fn::authored",
                "entrypoint_sha256":"entrypoint-shape",
                "entrypoint_source":"crates/edge/ess-cli/src/input_discovery.rs",
                "entrypoint_ast_sha256":"entrypoint-body",
                "entrypoint_source_file_sha256":"entrypoint-source-file",
                "role":role,
                "mode":mode,
                "case":{
                    "id":format!("{role_id}-{mode_id}"),
                    "package":"ess-cli","target_kind":"bin","target_name":"ess",
                    "target_source":"crates/edge/ess-cli/src/main.rs",
                    "case_source":"crates/edge/ess-cli/src/input_discovery_accounting_tests.rs",
                    "full_name":format!("input_discovery::accounting_tests::acquisition_{role_id}_{}",mode_id.replace('-',"_")),
                    "case_ast_sha256":format!("ast-{role_id}-{mode_id}"),
                    "source_file_sha256":"source-file",
                    "features":[],"target_profile":"x86_64-unknown-linux-gnu/default",
                    "tool_requirements":["frozen Rust 1.98.1; locked offline owner-target build"],
                    "nested_runtime":"None claimed; direct Rust assertions only"
                },
                "source_reference":"crates/edge/ess-cli/src/input_discovery.rs",
                "review_reference":"review-result:fixture"
            }));
        }
    }
    rows
}

fn acquisition_candidate_for_authored_source(source_text: &str) -> Value {
    const ENTRYPOINT: &str = "ess_cli::bin(ess)::input_discovery::fn::authored";
    const ENTRYPOINT_SOURCE: &str = "crates/edge/ess-cli/src/input_discovery.rs";
    const CASE_SOURCE: &str = "crates/edge/ess-cli/src/input_discovery_accounting_tests.rs";
    let extracted = super::consumer::fixture_at(ENTRYPOINT_SOURCE, source_text).unwrap();
    let entry = extracted["entries"]["fixture::fn::authored"].clone();
    let entries = BTreeMap::from([(ENTRYPOINT.to_owned(), entry)]);
    let mut profiles = BTreeMap::new();
    let mut cases = BTreeMap::new();
    for row in acquisition_rows() {
        let profile = row["profile"].as_str().unwrap();
        profiles.insert(
            profile.to_owned(),
            json!({
                "profile_sha256":row["profile_sha256"],
                "definition":{
                    "classification":"scenario-acquisition",
                    "entrypoints":[ENTRYPOINT]
                }
            }),
        );
        let case = &row["case"];
        let case_id = format!("ess-cli:bin:ess:{}", case["full_name"].as_str().unwrap());
        cases.insert(
            case_id,
            json!({
                "source":CASE_SOURCE,
                "package":"ess-cli",
                "target_kind":"bin",
                "target_name":"ess",
                "full_name":case["full_name"],
                "ignored":false,
                "contains_early_return":false,
                "contains_catch_unwind":false,
                "nested_command_candidates":[],
                "assertion_candidates":["assert_eq"] ,
                "case_ast_sha256":case["case_ast_sha256"]
            }),
        );
    }
    let sources = BTreeMap::from([
        (
            ENTRYPOINT_SOURCE.to_owned(),
            super::hash_bytes(source_text.as_bytes()),
        ),
        (CASE_SOURCE.to_owned(), "case-source".to_owned()),
    ]);
    super::scenario_acquisition::candidates(&profiles, &entries, &cases, &sources).unwrap()
}

#[test]
fn acquisition_refuses_a_discarded_default_probe_in_the_actual_authored_body() {
    let control = r"
        pub(crate) fn authored(path: Option<&Path>, coverage: bool) -> Result<Vec<Input>> {
            path.map_or_else(
                || Ok(Vec::new()),
                |path| acquire(path, if coverage { Kind::Coverage } else { Kind::Authored }),
            )
        }
    ";
    let mutated = r#"
        pub(crate) fn authored(path: Option<&Path>, coverage: bool) -> Result<Vec<Input>> {
            let _discarded_default_probe = std::fs::read_dir(".");
            path.map_or_else(
                || Ok(Vec::new()),
                |path| acquire(path, if coverage { Kind::Coverage } else { Kind::Authored }),
            )
        }
    "#;
    let control_candidate = acquisition_candidate_for_authored_source(control);
    let authority = json!({
        "format":"ess-consumer-scenario-acquisition/2",
        "rows":control_candidate["rows"]
    });
    super::scenario_acquisition::plan(&control_candidate, &authority).unwrap();

    let mutated_candidate = acquisition_candidate_for_authored_source(mutated);
    assert_eq!(
        control_candidate["rows"][0]["entrypoint_sha256"],
        mutated_candidate["rows"][0]["entrypoint_sha256"],
        "the discarded probe must leave the signature identity unchanged"
    );
    assert_ne!(
        control_candidate["rows"][0]["entrypoint_ast_sha256"],
        mutated_candidate["rows"][0]["entrypoint_ast_sha256"],
        "the production extractor must observe the changed body"
    );
    assert_ne!(
        control_candidate["rows"][0]["entrypoint_source_file_sha256"],
        mutated_candidate["rows"][0]["entrypoint_source_file_sha256"],
        "the production source authority must observe the changed bytes"
    );
    assert!(
        super::scenario_acquisition::plan(&mutated_candidate, &authority).is_err(),
        "reviewed acquisition authority must reject an actual discarded default-directory probe"
    );
}

#[test]
fn acquisition_v1_signature_only_family_is_refused() {
    let rows = acquisition_rows();
    let candidates = json!({
        "format":"ess-consumer-scenario-acquisition-candidate/2",
        "rows":rows
    });
    let mut signature_only = acquisition_rows();
    for row in &mut signature_only {
        let fields = row.as_object_mut().unwrap();
        fields.remove("entrypoint_source");
        fields.remove("entrypoint_ast_sha256");
        fields.remove("entrypoint_source_file_sha256");
    }
    let v1_authority = json!({
        "format":"ess-consumer-scenario-acquisition/1",
        "rows":signature_only
    });
    assert!(super::scenario_acquisition::plan(&candidates, &v1_authority).is_err());

    let mut v1_candidates = candidates.clone();
    v1_candidates["format"] = json!("ess-consumer-scenario-acquisition-candidate/1");
    let authority = json!({
        "format":"ess-consumer-scenario-acquisition/2",
        "rows":acquisition_rows()
    });
    assert!(super::scenario_acquisition::plan(&v1_candidates, &authority).is_err());

    let plan = super::scenario_acquisition::plan(&candidates, &authority).unwrap();
    let mut proof = super::scenario_acquisition::test_proof(&plan).unwrap();
    proof["format"] = json!("ess-consumer-scenario-acquisition-proof/1");
    assert!(super::scenario_acquisition::qualify(&plan, &proof).is_err());
    let mut v1_plan = plan;
    v1_plan["format"] = json!("ess-consumer-scenario-acquisition-plan/1");
    assert!(super::scenario_acquisition::required_cases(&v1_plan).is_err());
}

#[test]
fn acquisition_authority_is_exactly_eight_closed_rows_and_planned_equals_proved() {
    let rows = acquisition_rows();
    let candidates = json!({"format":"ess-consumer-scenario-acquisition-candidate/2","rows":rows});
    let authority =
        json!({"format":"ess-consumer-scenario-acquisition/2","rows":acquisition_rows()});
    let plan = super::scenario_acquisition::plan(&candidates, &authority).unwrap();
    assert_eq!(plan["required_rows"], 8);
    assert_eq!(plan["qualified_rows"], 0);
    let proof = super::scenario_acquisition::test_proof(&plan).unwrap();
    let qualified = super::scenario_acquisition::qualify(&plan, &proof).unwrap();
    assert_eq!(qualified["qualified_rows"], 8);
}

#[test]
fn acquisition_profile_inventory_refuses_a_ninth_scenario_acquisition_row() {
    let mut profiles = serde_json::Map::new();
    for row in acquisition_rows() {
        profiles.insert(
            row["profile"].as_str().unwrap().to_owned(),
            json!({
                "profile_sha256":row["profile_sha256"],
                "definition":{"classification":"scenario-acquisition"}
            }),
        );
    }
    profiles.insert(
        "acquisition-authored-shadow".to_owned(),
        json!({
            "profile_sha256":"shadow-profile",
            "definition":{"classification":"scenario-acquisition"}
        }),
    );

    assert!(
        super::scenario_acquisition::profile_identities(&Value::Object(profiles)).is_err(),
        "a ninth scenario-acquisition profile must not disappear from both the model matrix and the exact eight-case acquisition plan"
    );
}

#[test]
fn acquisition_refuses_ninth_missing_stale_wrong_role_source_or_execution() {
    let candidates =
        json!({"format":"ess-consumer-scenario-acquisition-candidate/2","rows":acquisition_rows()});
    for mutate in 0..15 {
        let mut authority =
            json!({"format":"ess-consumer-scenario-acquisition/2","rows":acquisition_rows()});
        match mutate {
            0 => {
                authority["rows"].as_array_mut().unwrap().pop();
            }
            1 => {
                let row = authority["rows"][0].clone();
                authority["rows"].as_array_mut().unwrap().push(row);
            }
            2 => authority["rows"][0]["profile"] = json!("acquisition-authored-ninth"),
            3 => authority["rows"][0]["profile_sha256"] = json!("stale"),
            4 => authority["rows"][0]["role"] = json!("Coverage"),
            5 => {
                authority["rows"][0]["case"]["target_source"] =
                    json!("crates/edge/ess-cli/src/lib.rs");
            }
            6 => {
                authority["rows"][0]["case"]["case_source"] =
                    json!("crates/edge/ess-cli/src/main.rs");
            }
            7 => authority["rows"][0]["case"]["target_kind"] = json!("test"),
            8 => authority["rows"][0]["mode"] = json!("DirectFile"),
            9 => authority["rows"][0]["case"]["case_ast_sha256"] = json!("stale-ast"),
            10 => authority["rows"][0]["case"]["source_file_sha256"] = json!("stale-source"),
            11 => authority["rows"][0]["case"]["full_name"] = json!("wrong::case"),
            12 => authority["rows"][0]["entrypoint_source"] = json!("wrong.rs"),
            13 => authority["rows"][0]["entrypoint_ast_sha256"] = json!("stale-ast"),
            14 => authority["rows"][0]["entrypoint_source_file_sha256"] = json!("stale-source"),
            _ => unreachable!(),
        }
        assert!(
            super::scenario_acquisition::plan(&candidates, &authority).is_err(),
            "mutation {mutate}"
        );
    }
    let authority =
        json!({"format":"ess-consumer-scenario-acquisition/2","rows":acquisition_rows()});
    let plan = super::scenario_acquisition::plan(&candidates, &authority).unwrap();
    let mut proof = super::scenario_acquisition::test_proof(&plan).unwrap();
    proof["executed_cases"].as_array_mut().unwrap().pop();
    assert!(super::scenario_acquisition::qualify(&plan, &proof).is_err());
}

#[test]
fn binary_unit_artifact_binds_target_kind_root_and_case_source_separately() {
    let artifact = json!({"reason":"compiler-artifact","package_id":"package","manifest_path":"Cargo.toml",
        "target":{"kind":["bin"],"crate_types":["bin"],"name":"ess","src_path":"main.rs","edition":"2021","doc":true,"doctest":false,"test":true},
        "profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":true},
        "features":[],"filenames":["ess"],"executable":"ess","fresh":true});
    let finished = json!({"reason":"build-finished","success":true});
    let text = format!("{artifact}\n{finished}\n");
    super::executor::binary_unit_artifact(&text, "package", "ess", "main.rs").unwrap();
    for (pointer, value) in [
        ("/target/kind", json!(["test"])),
        ("/target/src_path", json!("case.rs")),
        ("/target/name", json!("other")),
    ] {
        let mut bad = artifact.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            super::executor::binary_unit_artifact(
                &format!("{bad}\n{finished}\n"),
                "package",
                "ess",
                "main.rs"
            )
            .is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn aggregate_modes_require_complete_nonoverlapping_same_consumer_frontiers() {
    let old_structure = json!({"obligations":{"wire:RawSpecFile#":"old-parent","wire:RawSpecFile#/properties":"old-children","wire:RawSpecFile#/properties/a":"old-a"},"references":[]});
    let current = json!({"obligations":{"wire:RawSpecFile#":"new-parent","wire:RawSpecFile#/properties":"new-children","wire:RawSpecFile#/properties/a":"new-a","wire:RawSpecFile#/properties/b":"new-b"},"references":[]});
    let row = json!({
        "id":"fixture-closure","mode":"ShapeDelta",
        "old":old("wire:RawSpecFile#","old-parent","consumer","profile"),
        "current":old("wire:RawSpecFile#","new-parent","consumer","profile"),
        "baseline_sha256":"baseline","reconciliation_sha256":"reconciliation",
        "residual":{"owner":"owner","follow_up":"epic:unknowns","reason":"unchanged residual remains visible"},
        "frontier":[{"path":"wire:RawSpecFile#/properties","disposition":{"kind":"Supported","cases":["case"]}}],
        "source_evidence":"source","review_reference":"review"
    });
    super::aggregate::test_validate(&json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]}),&old_structure,&current,&json!([{"model":"wire:RawSpecFile#/properties","consumer":"consumer","profile":"profile","kind":"Supported","cases":["case"]}]),&json!(["case"])).unwrap();
    assert!(super::aggregate::claims(
        &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]})
    )
    .is_err());
    for mutate in 0..6 {
        let mut changed = row.clone();
        match mutate {
            0 => changed["frontier"] = json!([]),
            1 => {
                let child = changed["frontier"][0].clone();
                changed["frontier"].as_array_mut().unwrap().push(child);
            }
            2 => changed["frontier"][0]["path"] = json!("wire:RawSpecFile#/unrelated"),
            3 => changed["current"]["consumer"] = json!("other"),
            4 => changed["mode"] = json!("CompleteCurrentSubtree"),
            5 => changed["frontier"][0]["disposition"]["cases"] = json!(["unexecuted"]),
            _ => unreachable!(),
        }
        assert!(super::aggregate::test_validate(&json!({"format":"ess-consumer-aggregate-closure/1","rows":[changed]}),&old_structure,&current,&json!([{"model":"wire:RawSpecFile#/properties","consumer":"consumer","profile":"profile","kind":"Supported","cases":["case"]}]),&json!(["case"])).is_err(), "mutation {mutate}");
    }
}

#[test]
fn aggregate_reference_deltas_require_both_local_endpoints_in_the_frontier() {
    let old_structure = json!({"obligations":{"parent":"old-parent","parent/a":"old-a","parent/b":"stable-b"},"references":[]});
    let current = json!({"obligations":{"parent":"new-parent","parent/a":"new-a","parent/b":"stable-b"},"references":[{"source":"parent/a","target":"parent/b"}]});
    let mut row = json!({
        "id":"reference-closure","mode":"ShapeDelta",
        "old":old("parent","old-parent","consumer","profile"),
        "current":old("parent","new-parent","consumer","profile"),
        "baseline_sha256":"baseline","reconciliation_sha256":"reconciliation",
        "residual":{"owner":"owner","follow_up":"epic:unknowns","reason":"residual"},
        "frontier":[{"path":"parent/a","disposition":{"kind":"Supported","cases":["case-a"]}}],
        "source_evidence":"source","review_reference":"review"
    });
    let cells = json!([
        {"model":"parent/a","consumer":"consumer","profile":"profile","kind":"Supported","cases":["case-a"]},
        {"model":"parent/b","consumer":"consumer","profile":"profile","kind":"Supported","cases":["case-b"]}
    ]);
    assert!(super::aggregate::test_validate(
        &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]}),
        &old_structure,
        &current,
        &cells,
        &json!(["case-a", "case-b"])
    )
    .is_err());
    row["frontier"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"parent/b","disposition":{"kind":"Supported","cases":["case-b"]}}));
    super::aggregate::test_validate(
        &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row]}),
        &old_structure,
        &current,
        &cells,
        &json!(["case-a", "case-b"]),
    )
    .unwrap();
}

#[test]
fn complete_current_subtree_requires_changed_profile_and_every_current_child() {
    let structure = json!({"obligations":{"parent":"parent-shape","parent/a":"a","parent/b":"b"},"references":[]});
    let mut row = json!({
        "id":"complete-closure","mode":"CompleteCurrentSubtree",
        "old":old("parent","parent-shape","consumer","old-profile"),
        "current":old("parent","parent-shape","consumer","new-profile"),
        "baseline_sha256":"baseline","reconciliation_sha256":"reconciliation",
        "residual":null,
        "frontier":[
            {"path":"parent/a","disposition":{"kind":"Supported","cases":["case-a"]}},
            {"path":"parent/b","disposition":{"kind":"Refused","cases":["case-b"],"refusal":"boundary"}}
        ],
        "source_evidence":"source","review_reference":"review"
    });
    let cells = json!([
        {"model":"parent/a","consumer":"consumer","profile":"new-profile","kind":"Supported","cases":["case-a"]},
        {"model":"parent/b","consumer":"consumer","profile":"new-profile","kind":"Refused","cases":["case-b"],"refusal":"boundary"}
    ]);
    super::aggregate::test_validate(
        &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]}),
        &structure,
        &structure,
        &cells,
        &json!(["case-a", "case-b"]),
    )
    .unwrap();
    row["current"]["profile"] = json!("old-profile");
    assert!(super::aggregate::test_validate(
        &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row]}),
        &structure,
        &structure,
        &cells,
        &json!(["case-a", "case-b"])
    )
    .is_err());
}

#[test]
fn aggregate_refused_child_requires_the_exact_qualified_refusal_boundary() {
    let old_structure =
        json!({"obligations":{"parent":"old-parent","parent/a":"old-a"},"references":[]});
    let current_structure =
        json!({"obligations":{"parent":"new-parent","parent/a":"new-a"},"references":[]});
    let mut row = json!({
        "id":"refused-child-closure","mode":"ShapeDelta",
        "old":old("parent","old-parent","consumer","profile"),
        "current":old("parent","new-parent","consumer","profile"),
        "baseline_sha256":"baseline","reconciliation_sha256":"reconciliation",
        "residual":{"owner":"owner","follow_up":"epic:unknowns","reason":"residual"},
        "frontier":[{"path":"parent/a","disposition":{"kind":"Refused","cases":["case-a"],"refusal":"reviewed-boundary"}}],
        "source_evidence":"source","review_reference":"review"
    });
    let cells = json!([
        {"model":"parent/a","consumer":"consumer","profile":"profile","kind":"Refused","cases":["case-a"],"refusal":"reviewed-boundary"}
    ]);
    super::aggregate::test_validate(
        &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]}),
        &old_structure,
        &current_structure,
        &cells,
        &json!(["case-a"]),
    )
    .unwrap();

    let mut missing_refusal = cells.clone();
    missing_refusal[0]
        .as_object_mut()
        .unwrap()
        .remove("refusal");
    assert!(
        super::aggregate::test_validate(
            &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]}),
            &old_structure,
            &current_structure,
            &missing_refusal,
            &json!(["case-a"]),
        )
        .is_err(),
        "a missing qualified refusal must not act as a wildcard"
    );

    row["frontier"][0]["disposition"]["refusal"] = json!("different-boundary");
    assert!(
        super::aggregate::test_validate(
            &json!({"format":"ess-consumer-aggregate-closure/1","rows":[row]}),
            &old_structure,
            &current_structure,
            &cells,
            &json!(["case-a"]),
        )
        .is_err(),
        "the aggregate authority must not replace the qualified child's named refusal boundary"
    );
}

#[test]
fn aggregate_closure_is_bound_to_the_exact_reconciliation_and_frozen_residual() {
    let mut reconciliation_authority = reconciliation();
    reconciliation_authority["decisions"][0]["action"]["claim"] =
        json!({"kind":"AggregateClosure","closure":"fixture-closure"});
    let resolution = super::reconciliation::resolve(
        &baseline(),
        "fixture-baseline",
        &json!({"changed":"new-shape","stable":"stable-shape"}),
        &json!({"stable-consumer":"stable-profile"}),
        &json!({"acquisition-authored-manifest":"new-acquisition-profile"}),
        &reconciliation_authority,
    )
    .unwrap();
    let row = json!({
        "id":"fixture-closure","mode":"ShapeDelta",
        "old":old("changed","old-shape","stable-consumer","stable-profile"),
        "current":old("changed","new-shape","stable-consumer","stable-profile"),
        "baseline_sha256":"fixture-baseline","reconciliation_sha256":resolution.digest,
        "residual":{"owner":"owner","follow_up":"epic:unknowns","reason":"visible unknown"},
        "frontier":[{"path":"changed/child","disposition":{"kind":"Supported","cases":["case"]}}],
        "source_evidence":"source","review_reference":"review"
    });
    let authority = json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]});
    super::aggregate::verify_reconciliation(&authority, &resolution, "fixture-baseline").unwrap();
    for (pointer, value) in [
        ("/rows/0/baseline_sha256", json!("wrong")),
        ("/rows/0/reconciliation_sha256", json!("wrong")),
        ("/rows/0/id", json!("other-closure")),
        ("/rows/0/residual/owner", json!("other-owner")),
    ] {
        let mut changed = authority.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert!(
            super::aggregate::verify_reconciliation(&changed, &resolution, "fixture-baseline")
                .is_err()
        );
    }
}

#[test]
fn pinned_historical_source_reproduces_all_five_old_aggregate_hashes() {
    let witness = super::aggregate::historical_witness().unwrap();
    for (id, hash) in [
        (
            "rust:ess_domain::command::RawOutcome",
            "a2072ea0863f3bfdaa92b6ab4b30eb1857bf23af0538db9747ea3b3efc3505b0",
        ),
        (
            "wire:RawSpecFile#",
            "687cebb230483729bb9de45eeec08e14b3f8961cfc977c1f470001cc1b41d6a5",
        ),
        (
            "wire:RawSpecFile#/definitions",
            "3515c009028136bab26f2c1e31a6a5a15f8a44eba3f3ae6444e47a3aa5aab244",
        ),
        (
            "wire:RawSpecFile#/definitions/RawOutcome",
            "eed5afcaac752aae6182c13a784afdc777ff207dd5404c20b320a619d7683722",
        ),
        (
            "wire:RawSpecFile#/definitions/RawOutcome/properties",
            "c72e70a9700e2b90445a761f78283670c50659dffa3bee63ff0bb5878c7297ab",
        ),
    ] {
        assert_eq!(witness["obligations"][id], hash, "{id}");
    }
}

#[test]
fn aggregate_proof_binds_the_exact_reviewed_manifest() {
    let old_structure =
        json!({"obligations":{"parent":"old-parent","parent/a":"old-a"},"references":[]});
    let current_structure =
        json!({"obligations":{"parent":"new-parent","parent/a":"new-a"},"references":[]});
    let row = json!({
        "id":"manifest-bound-closure","mode":"ShapeDelta",
        "old":old("parent","old-parent","consumer","profile"),
        "current":old("parent","new-parent","consumer","profile"),
        "baseline_sha256":"baseline","reconciliation_sha256":"reconciliation",
        "residual":{"owner":"owner","follow_up":"epic:unknowns","reason":"residual"},
        "frontier":[{"path":"parent/a","disposition":{"kind":"Supported","cases":["case-a"]}}],
        "source_evidence":"source-a","review_reference":"review-a"
    });
    let cells = json!([
        {"model":"parent/a","consumer":"consumer","profile":"profile","kind":"Supported","cases":["case-a"]}
    ]);
    let authority = json!({"format":"ess-consumer-aggregate-closure/1","rows":[row.clone()]});
    let first = super::aggregate::test_validate(
        &authority,
        &old_structure,
        &current_structure,
        &cells,
        &json!(["case-a"]),
    )
    .unwrap();
    assert_eq!(
        first["aggregate_manifest_sha256"],
        super::hash_json(&authority),
        "aggregate proof must preserve the exact reviewed manifest digest"
    );

    let mut changed_authority = authority;
    changed_authority["rows"][0]["review_reference"] = json!("review-b");
    let second = super::aggregate::test_validate(
        &changed_authority,
        &old_structure,
        &current_structure,
        &cells,
        &json!(["case-a"]),
    )
    .unwrap();
    assert_eq!(
        second["aggregate_manifest_sha256"],
        super::hash_json(&changed_authority),
        "aggregate proof must bind the manifest that was actually verified"
    );

    assert_ne!(
        first, second,
        "aggregate proof must bind the exact reviewed manifest, including its evidence authority"
    );
}
