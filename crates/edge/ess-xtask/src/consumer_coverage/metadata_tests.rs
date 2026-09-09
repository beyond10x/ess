use super::*;

fn fixture() -> (Value, Value, Value, Value, Value) {
    let schema =
        serde_json::to_value(schemars::schema_for!(ess_domain::spec::RawSpecFile)).unwrap();
    let inventory = super::super::wire::extract(&schema).unwrap();
    let manifest: Value =
        serde_json::from_slice(include_bytes!("reviewed-schema-metadata.json")).unwrap();
    let profiles = manifest["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["consumer"].as_str().unwrap().to_owned(),
                r["profile"].clone(),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let definitions = serde_json::from_str(include_str!("profiles.json")).unwrap();
    (schema, inventory, manifest, json!(profiles), definitions)
}
fn checked(
    manifest: &Value,
    inventory: &Value,
    profiles: &Value,
    definitions: &Value,
) -> Result<Candidates> {
    validate(
        &serde_json::to_vec(manifest)?,
        &inventory["obligations"],
        profiles,
        definitions,
        &guard_digest(),
    )
}
#[test]
fn fresh_provider_and_exact_six_relationships_share_the_actual_wire_inventory() {
    let (schema, wire, manifest, profiles, definitions) = fixture();
    let (fresh, actual) = observe_schema(&schema, &wire).unwrap();
    assert_eq!(fresh, schema);
    assert_eq!(actual, wire);
    let candidate = checked(&manifest, &wire, &profiles, &definitions).unwrap();
    candidate.verify_shapes(&actual).unwrap();
    assert_eq!(candidate.rows.len(), 6);
    assert_eq!(
        candidate
            .rows
            .iter()
            .map(|r| &r.guard)
            .collect::<BTreeSet<_>>()
            .len(),
        1
    );
    assert_eq!(
        candidate
            .rows
            .iter()
            .map(|r| (r.model.clone(), r.consumer.clone(), r.relationship.clone()))
            .collect::<BTreeSet<_>>(),
        expected()
    );
    assert!(schema["properties"].get("$schema").is_none());
    assert!(schema["properties"].get("definitions").is_none());
}
#[test]
fn manifest_rows_reject_missing_stale_hashes_wrong_roles_and_behavior_fields() {
    let (_, wire, manifest, profiles, definitions) = fixture();
    for field in [
        "model",
        "shape",
        "consumer",
        "profile",
        "relationship",
        "guard",
        "guard_source_sha256",
        "reason",
        "decision",
    ] {
        let mut bad = manifest.clone();
        bad["rows"][0].as_object_mut().unwrap().remove(field);
        assert!(
            checked(&bad, &wire, &profiles, &definitions).is_err(),
            "{field}"
        );
    }
    for (field, value) in [
        ("shape", json!("0".repeat(64))),
        ("profile", json!("0".repeat(64))),
        ("guard_source_sha256", json!("0".repeat(64))),
        ("guard_source_sha256", json!("")),
        ("guard", json!("another-guard")),
        ("reason", json!("")),
        ("decision", json!("another-decision")),
        ("relationship", json!("RootDefinitionsContainer")),
        ("relationship", json!("Other")),
        ("cases", json!(["passed"])),
        ("owner", json!("owner")),
        ("refusal", json!("not behavior")),
    ] {
        let mut bad = manifest.clone();
        bad["rows"][0][field] = value;
        assert!(
            checked(&bad, &wire, &profiles, &definitions).is_err(),
            "{field}"
        );
    }
    for format in [Value::Null, json!("ess-consumer-schema-metadata/2")] {
        let mut bad = manifest.clone();
        bad["format"] = format;
        assert!(checked(&bad, &wire, &profiles, &definitions).is_err());
    }
    let mut bad = manifest.clone();
    bad.as_object_mut().unwrap().remove("format");
    assert!(checked(&bad, &wire, &profiles, &definitions).is_err());
}
#[test]
fn manifest_cannot_drop_duplicate_add_descendants_or_add_a_fourth_profile() {
    let (_, wire, manifest, profiles, definitions) = fixture();
    let mut missing = manifest.clone();
    missing["rows"].as_array_mut().unwrap().pop();
    assert!(checked(&missing, &wire, &profiles, &definitions).is_err());
    let mut duplicate = manifest.clone();
    duplicate["rows"]
        .as_array_mut()
        .unwrap()
        .push(manifest["rows"][0].clone());
    assert!(checked(&duplicate, &wire, &profiles, &definitions).is_err());
    let mut descendant = manifest.clone();
    descendant["rows"][0]["model"] = json!("wire:RawSpecFile#/definitions/Field");
    descendant["rows"][0]["shape"] =
        wire["obligations"]["wire:RawSpecFile#/definitions/Field"].clone();
    assert!(checked(&descendant, &wire, &profiles, &definitions).is_err());
    let mut fourth = manifest.clone();
    fourth["rows"][0]["consumer"] = json!("fourth");
    let mut expanded = profiles;
    expanded["fourth"] = fourth["rows"][0]["profile"].clone();
    assert!(checked(&fourth, &wire, &expanded, &definitions).is_err());
    let mut root = manifest;
    root["rows"][0]["model"] = json!("wire:RawSpecFile#");
    root["rows"][0]["shape"] = wire["obligations"]["wire:RawSpecFile#"].clone();
    assert!(checked(&root, &wire, &expanded, &definitions).is_err());
}
#[test]
fn profile_declaration_boundary_is_exact_and_does_not_follow_a_manifest_allowlist() {
    let (_, wire, manifest, profiles, definitions) = fixture();
    for field in ["entrypoints", "classification", "execution_profile"] {
        let mut bad = definitions.clone();
        bad[0][field] = if field == "entrypoints" {
            json!(COMMON[..3])
        } else {
            json!("other")
        };
        assert!(
            checked(&manifest, &wire, &profiles, &bad).is_err(),
            "{field}"
        );
    }
    let mut absent = definitions.clone();
    absent.as_array_mut().unwrap().remove(0);
    assert!(checked(&manifest, &wire, &profiles, &absent).is_err());
    let mut duplicate = definitions;
    let row = duplicate[0].clone();
    duplicate.as_array_mut().unwrap().push(row);
    assert!(checked(&manifest, &wire, &profiles, &duplicate).is_err());
}
#[test]
fn fresh_provider_refuses_retained_drift_dialect_container_and_reference_inventory_changes() {
    let (schema, wire, _, _, _) = fixture();
    let mut retained = schema.clone();
    retained["description"] = json!("different retained provider");
    assert!(observe_schema(&retained, &wire).is_err());
    for (field, value) in [
        (
            "$schema",
            json!("https://json-schema.org/draft/2020-12/schema"),
        ),
        ("definitions", json!([])),
    ] {
        let mut bad = schema.clone();
        bad[field] = value;
        assert!(verify_schema(&bad, &bad, &wire).is_err(), "{field}");
    }
    for field in ["$schema", "definitions"] {
        let mut bad = schema.clone();
        bad["properties"][field] = json!({"type":"string"});
        let observed = super::super::wire::extract(&bad).unwrap();
        assert!(verify_schema(&bad, &bad, &observed).is_err(), "{field}");
    }
    let mut wrong_refs = wire.clone();
    wrong_refs["references"] = json!([]);
    assert_ne!(wrong_refs, wire);
    assert!(observe_schema(&schema, &wrong_refs).is_err());
    let mut wrong_obligations = wire.clone();
    wrong_obligations["obligations"]["wire:RawSpecFile#"] = json!("stale");
    assert!(observe_schema(&schema, &wrong_obligations).is_err());
    let mut unresolved = schema.clone();
    unresolved["properties"]["probe"] = json!({"$ref":"#/definitions/Missing"});
    assert!(verify_schema(&unresolved, &unresolved, &wire).is_err());
    let mut unsupported = schema;
    unsupported["unevaluatedProperties"] = json!(false);
    assert!(verify_schema(&unsupported, &unsupported, &wire).is_err());
}
#[test]
fn changed_definition_and_forged_model_dictionary_cannot_reuse_the_container_row() {
    let (mut schema, wire, manifest, profiles, definitions) = fixture();
    let candidate = checked(&manifest, &wire, &profiles, &definitions).unwrap();
    schema["definitions"]["Field"]["properties"]["metadata_probe"] = json!({"type":"string"});
    let changed = super::super::wire::extract(&schema).unwrap();
    assert!(changed["obligations"]
        .get("wire:RawSpecFile#/definitions/Field/properties/metadata_probe")
        .is_some());
    assert_ne!(
        wire["obligations"]["wire:RawSpecFile#/definitions"],
        changed["obligations"]["wire:RawSpecFile#/definitions"]
    );
    assert!(candidate.verify_shapes(&changed).is_err());
    assert!(checked(&manifest, &changed, &profiles, &definitions).is_err());
    // Even mutually agreeing source-row/model-dictionary lies cannot replace fresh wire facts.
    let mut forged = manifest;
    let mut dictionary = wire.clone();
    let model = forged["rows"][0]["model"].as_str().unwrap().to_owned();
    for row in forged["rows"].as_array_mut().unwrap() {
        if row["model"] == model {
            row["shape"] = json!("0".repeat(64));
        }
    }
    dictionary["obligations"][&model] = json!("0".repeat(64));
    let forged = checked(&forged, &dictionary, &profiles, &definitions).unwrap();
    assert!(forged.verify_shapes(&wire).is_err());
}

#[test]
fn current_compiled_provider_executes_one_guard_and_binds_its_opaque_proof_to_this_run() {
    let (schema, wire, _, profiles, _) = fixture();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap();
    let source: Value = serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/consumer-source.json"
    )))
    .unwrap();
    let build: Value = serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/consumer-build.json"
    )))
    .unwrap();
    let invocation = super::super::invocation(root, &build).unwrap();
    let profile = json!({"source":source,"compiled_provider_source":source,"compiled_build":build,"current_invocation":invocation,
        "provider_executable_sha256":super::super::hash_bytes(&fs::read(std::env::current_exe().unwrap()).unwrap())});
    let authority = Authority::capture(root, &profile).unwrap();
    let proof = execute(&authority, &schema, &wire, &wire["obligations"], &profiles).unwrap();
    let candidate = candidates(&wire["obligations"], &profiles).unwrap();
    proof
        .check(&authority, candidate.rows(), candidate.digest())
        .unwrap();
    assert_eq!(proof.receipt()["executed_guards"], 1);
    assert_eq!(proof.receipt()["metadata_cells"], 6);
    let other = Authority::capture(root, &profile).unwrap();
    assert!(proof
        .check(&other, candidate.rows(), candidate.digest())
        .is_err());
    let mut forged = profile;
    forged["source"][SOURCE] = json!("0".repeat(64));
    forged["compiled_provider_source"] = forged["source"].clone();
    assert!(Authority::capture(root, &forged).is_err());
}
#[test]
fn opaque_proof_checks_reject_different_runs_pairs_guard_sources_and_manifests() {
    let (_, wire, manifest, profiles, definitions) = fixture();
    let candidate = checked(&manifest, &wire, &profiles, &definitions).unwrap();
    let authority = Authority {
        root: PathBuf::from("unmeasured-test-root"),
        profile: json!({"source":"fixture"}),
    };
    let other = Authority {
        root: authority.root.clone(),
        profile: authority.profile.clone(),
    };
    // Internal proof fixture exercises equality checks; it is not provider execution evidence.
    let proof = Verified {
        authority: &authority,
        rows: candidate.rows.clone(),
        manifest: candidate.digest.clone(),
        receipt: json!({"diagnostic":"fixture only"}),
    };
    proof
        .check(&authority, &candidate.rows, &candidate.digest)
        .unwrap();
    assert!(proof
        .check(&other, &candidate.rows, &candidate.digest)
        .is_err());
    assert!(proof
        .check(&authority, &candidate.rows, "forged receipt")
        .is_err());
    let mut missing = candidate.rows.clone();
    missing.pop_first();
    assert!(proof
        .check(&authority, &missing, &candidate.digest)
        .is_err());
    for field in [
        "shape",
        "profile",
        "guard_source_sha256",
        "consumer",
        "model",
    ] {
        let mut row = serde_json::to_value(candidate.rows.first().unwrap()).unwrap();
        row[field] = json!("changed");
        let mut altered = candidate.rows.clone();
        altered.pop_first();
        altered.insert(serde_json::from_value(row).unwrap());
        assert!(
            proof
                .check(&authority, &altered, &candidate.digest)
                .is_err(),
            "{field}"
        );
    }
    assert!(authority.verify().is_err());
    assert!(serde_json::from_value::<Manifest>(proof.receipt.clone()).is_err());
}
