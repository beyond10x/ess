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
            "CARGO_CFG_TARGET_FAMILY":"unix", "CARGO_CFG_FEATURE":"",
            "PROFILE":"debug", "OPT_LEVEL":"0", "DEBUG":"false", "NUM_JOBS":"2",
            "RUSTC_WRAPPER":"", "RUSTC_WORKSPACE_WRAPPER":"",
            "CARGO_BUILD_RUSTC_WRAPPER":"", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER":""
        },
        "tools":{"CARGO":identity, "RUSTC":identity},
        "cargo_configuration":{"measured-config":super::hash_bytes(bytes)}
    });
    let empty_wrapper = |_: &str| Some(String::new());
    let read = |_: &str| Ok(bytes.to_vec());
    super::validate_build(&actual, empty_wrapper, read).unwrap();
    for (key, value) in [
        ("TARGET", "aarch64-unknown-linux-gnu"),
        ("CARGO_FEATURE_HIDDEN", "1"),
        ("RUSTC_WRAPPER", "unreviewed-wrapper"),
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
    let mut changed_config = actual.clone();
    changed_config["cargo_configuration"]["measured-config"] = json!("changed");
    assert!(super::validate_build(&changed_config, empty_wrapper, read).is_err());
    let mut changed_compiler = actual;
    changed_compiler["tools"]["RUSTC"]["version"] = json!("tool 1.99.0");
    assert!(super::validate_build(&changed_compiler, empty_wrapper, read).is_err());
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
