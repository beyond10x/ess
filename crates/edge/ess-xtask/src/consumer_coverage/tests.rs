use super::{eligibility, model, wire};
use serde_json::json;

#[test]
fn private_optional_tuple_and_enum_members_have_separate_identities() {
    let a = model("pub struct Root { hidden: Option<Inner> } enum Inner { Unit, Tuple(u32), Named { x: bool } }").unwrap();
    for id in [
        "rust:fixture::Root",
        "rust:fixture::Root/field/hidden",
        "rust:fixture::Inner/variant/Tuple/field/0",
        "rust:fixture::Inner/variant/Named/field/x",
    ] {
        assert!(a["obligations"].get(id).is_some(), "{id}");
    }
}
#[test]
fn comments_and_whitespace_do_not_change_rust_identity_or_shape() {
    assert_eq!(
        model("pub struct Root { x: u32 }").unwrap(),
        model("// comment\npub struct Root { /* comment */ x : u32, }").unwrap()
    );
}
#[test]
fn changed_string_backed_alternative_is_not_hidden_by_wire_string() {
    assert_ne!(
        model("enum Root { Text(String) }").unwrap(),
        model("enum Root { Text(String), Number(u32) }").unwrap()
    );
}
#[test]
fn local_aliases_inline_modules_reexports_and_cycles_are_resolved() {
    let a = model("mod child { pub struct Item { pub parent: Option<Box<super::Root>> } } use child::Item as Alias; pub struct Root { x: Alias }").unwrap();
    assert!(a["obligations"]
        .get("rust:fixture::child::Item/field/parent")
        .is_some());
}
#[test]
fn unknown_type_owner_is_a_named_refusal() {
    assert!(model("struct Root { x: other::Unknown }")
        .unwrap_err()
        .to_string()
        .contains("other::Unknown"));
}
#[test]
fn unknown_production_macro_cannot_hide_a_declaration() {
    assert!(model("unknown! { struct Secret; } struct Root;")
        .unwrap_err()
        .to_string()
        .contains("unknown"));
}
#[test]
fn production_cfg_and_alias_ambiguity_fail_closed() {
    for text in [
        r#"#[cfg(feature = "hidden")] struct Root;"#,
        "type Root<T> = Vec<T>;",
    ] {
        assert!(model(text).is_err());
    }
}
#[test]
fn cfg_test_is_excluded_without_hiding_production() {
    assert_eq!(
        model("struct Root; #[cfg(test)] mod tests { unknown!(); }").unwrap(),
        model("struct Root;").unwrap()
    );
}
#[test]
fn representation_attributes_invalidate_shape_and_unknown_attributes_refuse() {
    assert_ne!(
        model("struct Root { x: u32 }").unwrap(),
        model(r#"struct Root { #[serde(rename = "y")] x: u32 }"#).unwrap()
    );
    assert!(model("#[mystery] struct Root;").is_err());
}
#[test]
fn wire_definitions_references_and_escaped_property_names_are_separate() {
    let a=wire(&json!({"type":"object","properties":{"a/b~c":{"$ref":"#/definitions/Thing"}},"definitions":{"Thing":{"type":"string"}}})).unwrap();
    assert!(a["obligations"]
        .get("wire:RawSpecFile#/properties/a~1b~0c/$ref")
        .is_some());
    assert_eq!(
        a["references"][0]["target"],
        "wire:RawSpecFile#/definitions/Thing"
    );
}
#[test]
fn wire_annotations_are_excluded_only_at_schema_positions() {
    assert_eq!(
        wire(&json!({"type":"string","description":"a"})).unwrap(),
        wire(&json!({"type":"string","title":"b"})).unwrap()
    );
    assert_ne!(
        wire(&json!({"const":{"description":"a"}})).unwrap(),
        wire(&json!({"const":{"description":"b"}})).unwrap()
    );
}
#[test]
fn wire_boolean_schemas_and_ordered_literals_are_concrete() {
    assert_ne!(wire(&json!(true)).unwrap(), wire(&json!(false)).unwrap());
    assert_ne!(
        wire(&json!({"enum":[1,2]})).unwrap(),
        wire(&json!({"enum":[2,1]})).unwrap()
    );
}
#[test]
fn wire_unknown_keyword_dialect_and_external_references_refuse() {
    for x in [
        json!({"mystery":true}),
        json!({"$defs":{}}),
        json!({"$ref":"https://example.invalid/s"}),
        json!({"$ref":"#/definitions/Missing"}),
    ] {
        assert!(wire(&x).is_err());
    }
}
#[test]
fn wire_reference_cycle_is_finite_and_descendants_invalidate_parents() {
    let x = json!({"definitions":{"X":{"properties":{"next":{"$ref":"#/definitions/X"}}}}});
    let mut y = x.clone();
    y["definitions"]["X"]["properties"]["new"] = json!(false);
    let a = wire(&x).unwrap();
    let b = wire(&y).unwrap();
    assert_ne!(
        a["obligations"]["wire:RawSpecFile#"],
        b["obligations"]["wire:RawSpecFile#"]
    );
    assert_eq!(a["references"].as_array().unwrap().len(), 1);
}
fn rows() -> (serde_json::Value, serde_json::Value, serde_json::Value) {
    (
        json!({"m":"shape"}),
        json!({"c":"profile"}),
        json!([{"model":"m","shape":"shape","consumer":"c","profile":"profile","status":"UNACCEPTED","disposition":{"kind":"UnknownProposal","owner":"fixture-owner","follow_up":"story:fixture-independent-follow-up","reason":"fixture behavior unproven"}}]),
    )
}
#[test]
fn finite_unaccepted_eligibility_has_exact_complete_accounting() {
    let (m, c, r) = rows();
    eligibility(&m, &c, &r).unwrap();
}
#[test]
fn new_member_or_consumer_cannot_inherit_eligibility() {
    let (mut m, mut c, r) = rows();
    m["new"] = json!("shape");
    assert!(eligibility(&m, &c, &r).is_err());
    m.as_object_mut().unwrap().remove("new");
    c["new"] = json!("p");
    assert!(eligibility(&m, &c, &r).is_err());
}
#[test]
fn stale_duplicate_ownerless_accepted_and_mandatory_unknown_rows_refuse() {
    let (m, c, r) = rows();
    for (key, value) in [
        ("shape", json!("stale")),
        ("status", json!("ACCEPTED")),
        ("mandatory", json!(true)),
        ("model", json!("unknown")),
    ] {
        let mut bad = r.clone();
        bad[0][key] = value;
        assert!(eligibility(&m, &c, &bad).is_err());
    }
    for (key, value) in [
        ("owner", ""),
        ("follow_up", ""),
        ("follow_up", "story:review-consumer-coverage"),
    ] {
        let mut bad = r.clone();
        bad[0]["disposition"][key] = json!(value);
        assert!(eligibility(&m, &c, &bad).is_err());
    }
    let mut bad = r.clone();
    bad.as_array_mut().unwrap().push(r[0].clone());
    assert!(eligibility(&m, &c, &bad).is_err());
}

#[test]
fn concrete_consumer_entries_and_full_case_names_are_not_package_aliases() {
    let a=super::consumer::fixture("pub fn consume() {} fn helper() {} #[cfg(test)] mod tests { #[test] fn case() { assert!(true); } }").unwrap();
    assert!(a["entries"].get("fixture::fn::consume").is_some());
    assert!(a["entries"].get("fixture::fn::helper").is_some());
    assert!(a["cases"].get("tests::case").is_some());
    assert_eq!(
        a["cases"]["tests::case"]["evidence_status"],
        "SOURCE_CANDIDATE_UNEXECUTED"
    );
}
#[test]
fn new_callable_and_target_alternatives_change_exact_inventory() {
    let a = super::consumer::fixture("pub enum Target { Rust, Go } pub fn consume() {}").unwrap();
    let b = super::consumer::fixture(
        "pub enum Target { Rust, Go, New } pub fn consume() {} pub fn added() {}",
    )
    .unwrap();
    assert_ne!(a, b);
    assert!(b["entries"]
        .get("fixture::enum::Target/variant/New")
        .is_some());
}
#[test]
fn ignored_case_and_nested_execution_are_visible_candidates_only() {
    let a=super::consumer::fixture(r#"#[cfg(test)] mod tests { #[test] #[ignore] fn skipped() { if unavailable() { return; } std::process::Command::new("go").status().unwrap(); } }"#).unwrap();
    let c = &a["cases"]["tests::skipped"];
    assert_eq!(c["ignored"], true);
    assert_eq!(c["contains_early_return"], true);
    assert_eq!(c["nested_command_candidates"], serde_json::json!(["go"]));
}
#[test]
fn consumer_unknown_cfg_is_not_a_default_profile() {
    assert!(
        super::consumer::fixture(r#"#[cfg(feature = "unreviewed")] pub fn added() {}"#).is_err()
    );
}
fn production_models() -> std::collections::BTreeMap<String, String> {
    let root = crate::workspace_root().unwrap();
    super::source_files(&root)
        .unwrap()
        .keys()
        .filter(|p| {
            p.starts_with("crates/specify/")
                && std::path::Path::new(p)
                    .extension()
                    .is_some_and(|ext| ext == "rs")
        })
        .map(|p| (p.clone(), std::fs::read_to_string(root.join(p)).unwrap()))
        .collect()
}
#[test]
fn actual_selected_roots_resolve_without_diagnostic_generated_declarations() {
    let a = super::rust::extract(&production_models()).unwrap();
    assert!(a["obligations"]
        .get("rust:ess_domain::spec::RawSpecFile")
        .is_some());
    assert!(a["obligations"]
        .get("rust:ess_compiler::ir::EssIrParts")
        .is_some());
    assert!(a["obligations"]
        .get("rust:ess_primitives::error::ValidationCode")
        .is_none());
    assert_eq!(a["diagnostic_macros"].as_object().unwrap().len(), 2);
}
#[test]
fn diagnostic_macro_definition_and_invocation_changes_refuse() {
    for (old, new) in [
        ("pub enum ValidationCode", "pub enum OtherCode"),
        (
            "EmptyWorkflow => \"empty_workflow\"",
            "EmptyWorkflow => \"changed\"",
        ),
    ] {
        let mut s = production_models();
        let p = s
            .get_mut("crates/specify/ess-primitives/src/error.rs")
            .unwrap();
        assert!(p.contains(old));
        *p = p.replace(old, new);
        assert!(super::rust::extract(&s)
            .unwrap_err()
            .to_string()
            .contains("guard"));
    }
}
#[test]
fn selected_root_reaching_guarded_diagnostic_type_is_not_an_opaque_leaf() {
    let mut s = production_models();
    let p = s.get_mut("crates/specify/ess-domain/src/spec.rs").unwrap();
    assert!(p.contains("pub struct RawSpecFile {"));
    *p = p.replace(
        "pub struct RawSpecFile {",
        "pub struct RawSpecFile { pub probe: ess_primitives::error::ValidationCode,",
    );
    let e = super::rust::extract(&s).unwrap_err().to_string();
    assert!(
        e.contains("ValidationCode") && e.contains("diagnostic"),
        "{e}"
    );
}

#[test]
fn generated_consumer_owner_uses_the_checked_invocation_grammar() {
    let a=super::consumer::fixture("checked_deserialize! { First { pub x: u32, } } checked_deserialize! { Second { y: String, } }").unwrap();
    assert!(a["entries"]
        .get("fixture::macro::invocation/checked_deserialize/for/First")
        .is_some());
    assert!(super::consumer::fixture(
        "checked_deserialize! { First this_is_not_the_declared_grammar }"
    )
    .is_err());
}

#[test]
fn declaration_order_and_unknown_representation_grammar_are_not_erased() {
    assert_ne!(
        model("enum Root { A, B }").unwrap(),
        model("enum Root { B, A }").unwrap()
    );
    assert_ne!(
        model("struct Root { a: bool, b: bool }").unwrap(),
        model("struct Root { b: bool, a: bool }").unwrap()
    );
    assert!(model("#[serde(unknown_representation)] struct Root;").is_err());
    assert!(model("#[derive(UnreviewedDerive)] struct Root;").is_err());
}

#[test]
fn pending_owner_checkpoint_is_complete_but_never_eligible() {
    let (m, c, _) = rows();
    let rows = json!([{"model":"m","shape":"shape","consumer":"c","profile":"profile","status":"UNACCEPTED","disposition":{"kind":"PendingOwner","reason":"No exact behavior assertion has been attributed."}}]);
    let checkpoint = super::account::checkpoint(&m, &c, &rows).unwrap();
    assert_eq!(checkpoint["accounting_complete"], true);
    assert_eq!(checkpoint["eligibility_valid"], false);
    assert!(super::account::checkpoint(&m, &c, &json!([])).is_err());
    let mut bad = rows.clone();
    bad[0]["disposition"]["kind"] = json!("BaselineUnknown");
    assert!(super::account::checkpoint(&m, &c, &bad).is_err());
    let mut bad = rows.clone();
    bad[0]["unexpected"] = json!(true);
    assert!(super::account::checkpoint(&m, &c, &bad).is_err());
}
#[test]
fn measured_build_profile_refuses_unsupported_target_features_and_wrappers() {
    // The measured boundary is injected: ordinary Cargo settings are not this
    // command's selected production profile, and tests must not mutate the host.
    let bytes = b"reviewed tool or configuration bytes";
    let identity =
        json!({"path":"measured-tool", "version":"tool 1.98.1", "sha256":super::hash_bytes(bytes)});
    let actual = json!({
        "environment": {
            "TARGET":"x86_64-unknown-linux-gnu", "HOST":"x86_64-unknown-linux-gnu",
            "CARGO_CFG_TARGET_ARCH":"x86_64", "CARGO_CFG_TARGET_OS":"linux",
            "CARGO_CFG_TARGET_FAMILY":"unix", "CARGO_CFG_FEATURE":"", "CARGO_ENCODED_RUSTFLAGS":"-C\u{1f}link-arg=-fuse-ld=lld",
            "PROFILE":"debug", "OPT_LEVEL":"0", "DEBUG":"false", "NUM_JOBS":"2",
            "RUSTC_WRAPPER":"", "RUSTC_WORKSPACE_WRAPPER":"",
            "CARGO_BUILD_RUSTC_WRAPPER":"", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER":""
        },
        "tools":{"CARGO":identity, "RUSTC":identity, "RUST_LLD":identity, "LD_LLD":identity},
        "cargo_configuration":{"measured-config":super::hash_bytes(bytes)}
    });
    let empty_wrapper = |_: &str| Some(String::new());
    let read = |_: &str| Ok(bytes.to_vec());
    super::validate_build(&actual, empty_wrapper, read).unwrap();
    for (key, value) in [
        ("TARGET", "aarch64-unknown-linux-gnu"),
        ("CARGO_FEATURE_HIDDEN", "1"),
        ("RUSTC_WRAPPER", "unreviewed-wrapper"),
        ("CARGO_ENCODED_RUSTFLAGS", ""),
        (
            "CARGO_ENCODED_RUSTFLAGS",
            "-C\u{1f}link-arg=-fuse-ld=lld\u{1f}-Copt-level=3",
        ),
    ] {
        let mut p = actual.clone();
        p["environment"][key] = json!(value);
        assert!(
            super::validate_build(&p, empty_wrapper, read).is_err(),
            "{key}"
        );
    }
    assert!(super::validate_build(&actual, |_| None, read).is_err());
    assert!(super::validate_build(&actual, empty_wrapper, |_| Ok(b"changed".to_vec())).is_err());
    for key in ["RUST_LLD", "LD_LLD"] {
        let mut changed = actual.clone();
        changed["tools"][key]["sha256"] = json!("changed linker bytes");
        assert!(
            super::validate_build(&changed, empty_wrapper, read).is_err(),
            "{key}"
        );
        let mut missing = actual.clone();
        missing["tools"].as_object_mut().unwrap().remove(key);
        assert!(
            super::validate_build(&missing, empty_wrapper, read).is_err(),
            "{key}"
        );
    }
    let mut changed_config = actual.clone();
    changed_config["cargo_configuration"]["measured-config"] = json!("changed");
    assert!(super::validate_build(&changed_config, empty_wrapper, read).is_err());
    for version in ["tool 1.99.0", "tool 1.98.10"] {
        let mut changed_compiler = actual.clone();
        changed_compiler["tools"]["RUSTC"]["version"] = json!(version);
        assert!(super::validate_build(&changed_compiler, empty_wrapper, read).is_err());
    }
}
#[test]
fn file_level_production_cfg_cannot_hide_from_the_graph() {
    let mut sources = production_models();
    let file = sources
        .get_mut("crates/specify/ess-domain/src/spec.rs")
        .unwrap();
    *file = format!("#![cfg(feature=\"hidden\")]\n{file}");
    assert!(super::rust::extract(&sources).is_err());
}
#[test]
fn external_module_inside_inline_module_uses_its_semantic_directory() {
    let mut sources = production_models();
    let file = sources
        .get_mut("crates/specify/ess-domain/src/spec.rs")
        .unwrap();
    *file = file.replace(
        "pub struct RawSpecFile {",
        "mod probe { pub mod leaf; }\npub struct RawSpecFile { pub probe: probe::leaf::Item,",
    );
    let virtual_module = std::path::Path::new("crates/specify/ess-domain/src/spec.rs")
        .with_extension("")
        .join("probe/leaf.rs");
    sources.insert(
        virtual_module.to_str().unwrap().into(),
        "pub struct Item;".into(),
    );
    let graph = super::rust::extract(&sources).unwrap();
    assert!(graph["obligations"]
        .get("rust:ess_domain::spec::probe::leaf::Item")
        .is_some());
}

#[test]
fn local_declarations_shadow_prelude_leaves_and_generics_refuse() {
    let graph = model("struct String { surprise: u8 } struct Root { text: String }").unwrap();
    assert!(graph["obligations"]
        .get("rust:fixture::String/field/surprise")
        .is_some());
    assert!(model("struct Option<T> { hidden: T } struct Root { value: Option<u8> }").is_err());
    assert!(model("struct Root { text: std::string::String }").is_ok());
}

fn profile_for(source: &str, target: &str) -> String {
    let entries = super::consumer::fixture(source).unwrap()["entries"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    super::proposal::fingerprint(
        &json!({"id":"fixture-profile","execution":"host-rust"}),
        &["fixture::fn::consume".into()],
        &entries,
        &json!({"compiled_build":{"environment":{"TARGET":target}}}),
    )
    .unwrap()
}
#[test]
fn consumer_identity_separates_body_comments_and_unrelated_entries_from_signatures() {
    let method_shape = |source: &str| {
        let scan = super::consumer::fixture(source).unwrap();
        scan["entries"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(id, _)| id.ends_with("::consume"))
            .unwrap()
            .1["declaration_sha256"]
            .clone()
    };
    assert_ne!(
        method_shape(
            "struct Owner<T>(T); impl<T> Owner<T> where T: Clone { pub fn consume(&self) {} }"
        ),
        method_shape(
            "struct Owner<T>(T); impl<T> Owner<T> where T: Copy { pub fn consume(&self) {} }"
        )
    );
    let base = profile_for("pub fn consume(x:u8)->u8 {x}", "linux");
    for source in [
        "pub fn consume(x:u8)->u8 {x+1}",
        "/// A comment.\npub fn consume(x:u8)->u8 {x}",
        "pub fn consume(x:u8)->u8 {x}\nfn unrelated() {}",
    ] {
        assert_eq!(base, profile_for(source, "linux"));
    }
    assert_ne!(base, profile_for("pub fn consume(x:u16)->u16 {x}", "linux"));
    assert_ne!(base, profile_for("fn consume(x:u8)->u8 {x}", "linux"));
    assert_ne!(
        base,
        profile_for("pub fn consume(x:u8)->u8 {x}", "other-target")
    );
}

#[test]
fn public_reexport_additions_and_alias_changes_have_concrete_identities() {
    let a = super::consumer::fixture("pub use owner::Original;").unwrap();
    let b = super::consumer::fixture("pub use owner::{Original, Added as Alias};").unwrap();
    assert!(a["entries"].get("fixture::reexport::Original").is_some());
    assert!(b["entries"].get("fixture::reexport::Alias").is_some());
    let c = super::consumer::fixture("pub use owner::Original as Renamed;").unwrap();
    assert!(c["entries"].get("fixture::reexport::Original").is_none());
    assert!(c["entries"].get("fixture::reexport::Renamed").is_some());
}
#[test]
fn classified_nonmodel_macro_definition_and_invocation_drift_refuse() {
    let source = "macro_rules! api { () => { pub struct Original; } } api! { Original; }";
    let entries = super::consumer::fixture(source).unwrap()["entries"].clone();
    let classes=serde_json::Value::Object(entries.as_object().unwrap().iter().map(|(id,row)|(id.clone(),json!({"class":"OwnedHelper","reason":"Explicit fixture macro guard", "macro_ast_sha256":if id.contains("::macro::"){row["source_item_sha256"].clone()}else{serde_json::Value::Null}}))).collect());
    super::proposal::classification_fixture(&entries, &classes).unwrap();
    for changed in [
        source.replacen("pub struct Original", "pub struct Added", 1),
        source.replace("api! { Original; }", "api! { Added; }"),
    ] {
        let new = super::consumer::fixture(&changed).unwrap()["entries"].clone();
        assert!(super::proposal::classification_fixture(&new, &classes).is_err());
    }
}

fn stage2_rows() -> (
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
) {
    let models = json!({"m":"shape", "n":"other-shape"});
    let profiles = json!({"c":"profile"});
    let baseline = json!({
        "format":"ess-consumer-initial-eligibility/1", "status":"ACCEPTED_INITIAL_ELIGIBILITY",
        "eligible_pairs":1, "mandatory_pairs_excluded":1,
        "stage1_source_commit":"source-commit", "stage1_binding_sha256":"binding",
        "source_checkpoint_sha256":"checkpoint", "extraction_profile":"closed fixture",
        "models":models,
        "groups":[{"consumer":"c", "profile":"profile", "exact_model_ids":["m"],
            "owner":"model-owner", "follow_up":"epic:independent-follow-up", "reason":"Unproven exact behavior"}]
    });
    let claims = json!([{"model":"n", "consumer":"c", "cases":["owner:test:target:case"],
        "kind":"Supported", "reason":"Exact value assertion"}]);
    (models, profiles, baseline, claims)
}
#[test]
fn stage2_exact_finite_baseline_plus_behavioral_case_covers_two_pairs() {
    let (m, p, b, c) = stage2_rows();
    let plan = super::enforce::plan(&m, &p, &b, &c).unwrap();
    assert_eq!(plan["cells"].as_array().unwrap().len(), 2);
}
#[test]
fn stage2_new_optional_or_string_backed_member_cannot_inherit_unknown() {
    let (mut m, p, b, c) = stage2_rows();
    m["new_optional"] = json!("new-shape");
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
    let (mut m, p, b, c) = stage2_rows();
    m["m"] = json!("changed-string-backed-shape");
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
}
#[test]
fn stage2_new_consumer_and_changed_profile_cannot_inherit_unknown() {
    let (m, mut p, b, c) = stage2_rows();
    p["new-target"] = json!("new-profile");
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
    let (m, mut p, b, c) = stage2_rows();
    p["c"] = json!("changed-profile");
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
}
#[test]
fn stage2_refusal_counts_only_current_eligible_unknown_cells() {
    let (mut m, p, b, c) = stage2_rows();
    m["new_optional"] = json!("new-shape");
    let error = super::enforce::plan(&m, &p, &b, &c).unwrap_err();
    let report: serde_json::Value = serde_json::from_str(&error.to_string()).unwrap();
    assert_eq!(report["BaselineUnknown"], 1);
    m["m"] = json!("changed-shape");
    let error = super::enforce::plan(&m, &p, &b, &c).unwrap_err();
    let report: serde_json::Value = serde_json::from_str(&error.to_string()).unwrap();
    assert_eq!(report["BaselineUnknown"], 0);
    assert_eq!(report["Supported"], 0);
    assert_eq!(report["Refused"], 0);
}
#[test]
fn stage2_classification_refusal_preserves_actual_accounting_diagnostics() {
    let (mut models, profiles, baseline, claims) = stage2_rows();
    models["new_optional"] = json!("shape");
    let accounting = super::enforce::plan(&models, &profiles, &baseline, &claims);
    let refusal =
        super::reconcile_extraction(Err(anyhow::anyhow!("unclassified new API")), accounting)
            .unwrap_err()
            .to_string();
    assert!(refusal.contains("unclassified new API"));
    assert!(refusal.contains("unaccounted new_optional c"));
    let (m, p, b, c) = stage2_rows();
    assert!(super::reconcile_extraction(
        Err(anyhow::anyhow!("classification failed")),
        super::enforce::plan(&m, &p, &b, &c)
    )
    .is_err());
    assert!(
        super::reconcile_extraction(Ok(String::new()), super::enforce::plan(&m, &p, &b, &c))
            .is_ok()
    );
}
#[test]
fn stage2_removed_duplicate_and_missing_pairs_refuse() {
    let (mut m, p, b, c) = stage2_rows();
    m.as_object_mut().unwrap().remove("m");
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
    let (m, p, mut b, c) = stage2_rows();
    b["groups"][0]["exact_model_ids"] = json!(["m", "m"]);
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
    let (m, p, b, c) = stage2_rows();
    assert!(super::enforce::plan(&m, &p, &b, &json!([])).is_err());
    let duplicate = json!([c[0], c[0]]);
    assert!(super::enforce::plan(&m, &p, &b, &duplicate).is_err());
}
#[test]
fn stage2_unknown_requires_independent_owner_and_closed_manifest() {
    for (key, value) in [
        ("owner", ""),
        ("follow_up", "story:review-consumer-coverage"),
        ("reason", ""),
    ] {
        let (m, p, mut b, c) = stage2_rows();
        b["groups"][0][key] = json!(value);
        assert!(super::enforce::plan(&m, &p, &b, &c).is_err(), "{key}");
    }
    let (m, p, mut b, c) = stage2_rows();
    b["unreviewed_extension"] = json!(true);
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
}
#[test]
fn stage2_claims_need_cases_and_named_refusal_without_contradictions() {
    let (m, p, b, mut c) = stage2_rows();
    c[0]["cases"] = json!([]);
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
    let (m, p, b, mut c) = stage2_rows();
    c[0]["kind"] = json!("Refused");
    c[0]["refusal"] = json!("");
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
    let (m, p, b, mut c) = stage2_rows();
    c[0]["model"] = json!("m");
    assert!(super::enforce::plan(&m, &p, &b, &c).is_err());
}
fn stage2_success_output() -> &'static str {
    "\nrunning 1 test\ntest case ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 65 filtered out; finished in 0.00s\n\n"
}
#[test]
fn stage2_measured_stable_listing_requires_exact_nonignored_case() {
    super::executor::listing("case: test\n\n1 test, 0 benchmarks\n", "case", false).unwrap();
    super::executor::listing("0 tests, 0 benchmarks\n", "case", true).unwrap();
    for output in [
        "0 tests, 0 benchmarks\n",
        "renamed: test\n\n1 test, 0 benchmarks\n",
        "case: test\ncase: test\n\n2 tests, 0 benchmarks\n",
    ] {
        assert!(super::executor::listing(output, "case", false).is_err());
    }
    assert!(
        super::executor::listing("case: test\n\n1 test, 0 benchmarks\n", "case", true).is_err()
    );
}
#[test]
fn stage2_one_actual_pass_requires_successful_direct_exit() {
    let output = stage2_success_output();
    super::executor::result(output, "", "case", Some(0)).unwrap();
    for exit in [Some(1), Some(101), None] {
        assert!(super::executor::result(output, "", "case", exit).is_err());
    }
    assert!(super::executor::result(output, "panic escaped capture", "case", Some(0)).is_err());
}
#[test]
fn stage2_zero_selected_ignored_failing_and_forged_results_refuse() {
    let output = stage2_success_output();
    for bad in [
        output.replace("1 passed", "0 passed"),
        output.replace("0 ignored", "1 ignored"),
        output.replace("0 failed", "1 failed"),
        output.replace("case ... ok", "other ... ok"),
        format!("{output}{output}"),
        format!("forged prelude\n{output}"),
        output.replace("0.00s", "unknown-time"),
        output.replace("running 1 test", "running 0 tests"),
    ] {
        assert!(
            super::executor::result(&bad, "", "case", Some(0)).is_err(),
            "{bad}"
        );
    }
}
#[test]
fn stage2_swallowed_panic_and_skipped_nested_branches_do_not_qualify() {
    let good = json!({"ignored":false,"contains_catch_unwind":false,"contains_early_return":false,"nested_command_candidates":[]});
    super::executor::source_contract(&good).unwrap();
    for key in ["ignored", "contains_catch_unwind", "contains_early_return"] {
        let mut bad = good.clone();
        bad[key] = json!(true);
        assert!(super::executor::source_contract(&bad).is_err());
    }
    let mut bad = good;
    bad["nested_command_candidates"] = json!(["unobserved Go branch"]);
    assert!(super::executor::source_contract(&bad).is_err());
}

#[test]
fn stage2_new_obligation_can_gain_behavior_without_expanding_initial_unknowns() {
    let (mut m, p, b, mut c) = stage2_rows();
    m["new_optional"] = json!("new-shape");
    c.as_array_mut().unwrap().push(json!({"model":"new_optional","consumer":"c","cases":["owner:test:target:new_case"],"kind":"Supported","reason":"New field is actually asserted"}));
    let plan = super::enforce::plan(&m, &p, &b, &c).unwrap();
    assert_eq!(plan["cells"].as_array().unwrap().len(), 3);
    assert_eq!(plan["BaselineUnknown"], 1);
}

struct Stage2FakeRunner {
    outputs: std::collections::VecDeque<super::executor::Captured>,
    commands: Vec<Vec<String>>,
    checks: usize,
    drift_at: Option<usize>,
}
impl super::executor::CaseRunner for Stage2FakeRunner {
    fn authority(&mut self) -> anyhow::Result<()> {
        self.checks += 1;
        if self.drift_at == Some(self.checks) {
            anyhow::bail!("source/profile drift");
        }
        Ok(())
    }
    fn command(&mut self, args: &[String]) -> anyhow::Result<super::executor::Captured> {
        self.commands.push(args.to_vec());
        self.outputs
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("missing actual result"))
    }
}
fn stage2_fake_runner() -> Stage2FakeRunner {
    Stage2FakeRunner {
        outputs: [
            "case: test\n\n1 test, 0 benchmarks\n",
            "0 tests, 0 benchmarks\n",
            stage2_success_output(),
        ]
        .map(|stdout| super::executor::Captured {
            stdout: stdout.into(),
            stderr: String::new(),
            exit: Some(0),
        })
        .into(),
        commands: Vec::new(),
        checks: 0,
        drift_at: None,
    }
}
#[test]
fn stage2_case_engine_lists_filters_and_executes_before_qualifying() {
    let mut runner = stage2_fake_runner();
    super::executor::execute_case(&mut runner, "case").unwrap();
    assert_eq!(runner.commands.len(), 3);
    assert_eq!(
        runner.commands[0],
        ["--list", "--exact", "case", "--color", "never"]
    );
    assert_eq!(
        runner.commands[1],
        ["--list", "--ignored", "--exact", "case", "--color", "never"]
    );
    assert_eq!(
        runner.commands[2],
        ["--exact", "case", "--test-threads", "1", "--color", "never"]
    );
    assert_eq!(runner.checks, 4);
    assert!(runner.outputs.is_empty());
}
#[test]
fn stage2_case_engine_refuses_source_drift_or_missing_actual_result() {
    for checkpoint in 1..=4 {
        let mut runner = stage2_fake_runner();
        runner.drift_at = Some(checkpoint);
        assert!(super::executor::execute_case(&mut runner, "case").is_err());
    }
    let mut runner = stage2_fake_runner();
    runner.outputs.pop_back();
    assert!(super::executor::execute_case(&mut runner, "case").is_err());
    let mut runner = stage2_fake_runner();
    runner.outputs[1].exit = Some(1);
    assert!(super::executor::execute_case(&mut runner, "case").is_err());
}

#[test]
fn stage2_current_flags_target_tools_and_complete_configuration_set_are_bound() {
    let build = json!({"environment":{"TARGET":"x86_64-unknown-linux-gnu"},"tools":{"CARGO":{"sha256":"cargo"},"RUSTC":{"sha256":"rustc"}},"cargo_configuration":{"config":"bytes"}});
    let invocation = json!({"environment":{},"tool_sha256":{"CARGO":"cargo","RUSTC":"rustc"},"cargo_configuration":{"config":"bytes"}});
    super::validate_invocation(&build, &invocation).unwrap();
    for (key, flags) in [
        ("RUSTFLAGS", "-C link-arg=-fuse-ld=lld"),
        ("CARGO_ENCODED_RUSTFLAGS", "-C\u{1f}link-arg=-fuse-ld=lld"),
        ("CARGO_BUILD_RUSTFLAGS", "-C link-arg=-fuse-ld=lld"),
        (
            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS",
            "-C link-arg=-fuse-ld=lld",
        ),
    ] {
        let mut good = invocation.clone();
        good["environment"][key] = json!(flags);
        super::validate_invocation(&build, &good).unwrap();
        good["environment"][key] = json!(format!("{flags} --cfg unreviewed"));
        assert!(super::validate_invocation(&build, &good).is_err(), "{key}");
        good["environment"][key] = json!("");
        assert!(
            super::validate_invocation(&build, &good).is_err(),
            "empty explicit {key}"
        );
    }
    for key in [
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS",
    ] {
        let mut bad = invocation.clone();
        bad["environment"][key] = json!("unreviewed flags");
        assert!(super::validate_invocation(&build, &bad).is_err(), "{key}");
    }
    let mut bad = invocation.clone();
    bad["environment"]["CARGO_BUILD_TARGET"] = json!("aarch64-unknown-linux-gnu");
    assert!(super::validate_invocation(&build, &bad).is_err());
    let mut bad = invocation.clone();
    bad["tool_sha256"]["RUSTC"] = json!("another compiler");
    assert!(super::validate_invocation(&build, &bad).is_err());
    let mut bad = invocation.clone();
    bad["cargo_configuration"]["added config"] = json!("bytes");
    assert!(super::validate_invocation(&build, &bad).is_err());
    let mut bad = invocation;
    bad["cargo_configuration"].as_object_mut().unwrap().clear();
    assert!(super::validate_invocation(&build, &bad).is_err());
}

fn stage2_artifact_fixture() -> serde_json::Value {
    json!({"reason":"compiler-artifact","package_id":"package","manifest_path":"Cargo.toml",
        "target":{"kind":["test"],"crate_types":["bin"],"name":"target","src_path":"source.rs","edition":"2021","doc":false,"doctest":false,"test":true},
        "profile":{"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":true},
        "features":[],"filenames":["fixture.bin"],"executable":"fixture.bin","fresh":true})
}
#[test]
fn stage2_cargo_artifact_requires_exact_owner_target_profile_and_completed_build() {
    let artifact = stage2_artifact_fixture();
    let finished = json!({"reason":"build-finished","success":true});
    let text = format!("{artifact}\n{finished}\n");
    assert_eq!(
        super::executor::artifact(&text, "package", "target", "source.rs").unwrap()["executable"],
        "fixture.bin"
    );
    for (path, value) in [
        ("/package_id", json!("other")),
        ("/target/name", json!("other")),
        ("/target/src_path", json!("other.rs")),
        ("/profile/debuginfo", json!(2)),
        ("/features", json!(["hidden"])),
        ("/executable", serde_json::Value::Null),
    ] {
        let mut bad = artifact.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(
            super::executor::artifact(
                &format!("{bad}\n{finished}\n"),
                "package",
                "target",
                "source.rs"
            )
            .is_err(),
            "{path}"
        );
    }
    for bad in [
        format!("{artifact}\n{artifact}\n{finished}\n"),
        format!("{artifact}\n"),
        format!("{artifact}\n{{\"reason\":\"build-finished\",\"success\":false}}\n"),
        format!("{text}{{\"reason\":\"unreviewed\"}}\n"),
    ] {
        assert!(super::executor::artifact(&bad, "package", "target", "source.rs").is_err());
    }
}
