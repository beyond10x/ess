//! Compile generated native data libraries and exercise their actual wire serialization.

#[path = "support/model.rs"]
mod model;

#[test]
fn model_wire_mapping_builds_with_native_roundtrips() {
    let plan = model::plan();
    let result = plan.rust("model_types").unwrap();
    assert_eq!(result, plan.rust("model_types").unwrap());
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        include_str!("fixtures/model_wire_tests.rs.txt"),
    );
}

/// The model types of a union mixing a unit variant with a payload variant (ess/22,
/// beyond10x/ess#418), selected through the event that carries it.
fn unit_variant_plan() -> Plan {
    let text =
        include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("work.yaml"),
        ess_domain::spec::RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap();
    let selected = ess_gen::schema::ModelTypes::select(
        &ir,
        &BTreeSet::from(["demo.work.StatusReported".to_owned()]),
    )
    .unwrap();
    Plan::from_model(&selected).unwrap()
}

/// A unit variant is the tag alone: both variants round-trip, and a unit variant written with a
/// content member, a payload variant written without one, and an undeclared tag are refused.
const UNIT_VARIANT_WIRE: &str = r#"
#[cfg(test)]
mod unit_variant_wire {
    use model_types::*;
    use serde_json::json;

    #[test]
    fn a_unit_variant_round_trips_as_its_tag_alone() {
        for status in [json!({"kind": "Open"}), json!({"kind": "Complete", "value": {"outcome": "shipped"}})] {
            let decoded: DemoWorkStatus = serde_json::from_value(status.clone()).unwrap();
            assert_eq!(serde_json::to_value(&decoded).unwrap(), status);
            let event = json!({"status": status});
            let decoded: DemoWorkStatusReported = serde_json::from_value(event.clone()).unwrap();
            assert_eq!(serde_json::to_value(&decoded).unwrap(), event);
        }
        for malformed in [
            json!({"kind": "Open", "value": {"outcome": "shipped"}}),
            json!({"kind": "Open", "value": null}),
            json!({"kind": "Complete"}),
            json!({"kind": "Closed"}),
        ] {
            assert!(serde_json::from_value::<DemoWorkStatus>(malformed.clone()).is_err(), "{malformed}");
        }
    }
}
"#;

#[test]
fn model_unit_variant_round_trips_natively_and_refuses_the_malformed_shapes() {
    let plan = unit_variant_plan();
    let result = plan.rust("model_types").unwrap();
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        UNIT_VARIANT_WIRE,
    );
}

/// A command's input and response and a view's row selected as roots (beyond10x/ess#656): the
/// generated library decodes the bodies the `OpenAPI` projection accepts for them, the input's
/// being the one its published examples spell, and refuses the bodies that projection refuses.
const HTTP_BODY_WIRE: &str = r#"
#[cfg(test)]
mod http_body_wire {
    use model_types::*;
    use serde_json::{json, Value};

    fn round_trips<T: serde::de::DeserializeOwned + serde::Serialize>(body: &Value) {
        let decoded: T = serde_json::from_value(body.clone()).unwrap();
        assert_eq!(&serde_json::to_value(&decoded).unwrap(), body);
        for (key, bad) in [("extra", json!(1)), ("note_id", json!(7))] {
            let mut refused = body.clone();
            refused[key] = bad;
            assert!(serde_json::from_value::<T>(refused).is_err(), "{key}");
        }
        let mut missing = body.clone();
        missing.as_object_mut().unwrap().remove("note_id");
        assert!(serde_json::from_value::<T>(missing).is_err());
    }

    #[test]
    fn the_types_match_the_http_projection() {
        round_trips::<DemoNotesCreateInput>(&json!({"note_id": "n-1", "title": "Buy milk", "isPinned": true}));
        round_trips::<DemoNotesCreateResponse>(&json!({"note_id": "n-1", "titleLength": 8}));
        round_trips::<DemoNotesNotesRow>(&json!({"note_id": "n-1", "title": "Buy milk"}));
        round_trips::<DemoNotesSummariesRow>(&json!({"note_id": "n-1", "title": "Buy milk"}));
    }
}
"#;

#[test]
fn the_types_match_the_http_projection() {
    let text = include_str!("../../ess-gen/tests/fixtures/command-view-roots.yaml");
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("notes.yaml"),
        ess_domain::spec::RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap();
    let selected = ess_gen::schema::ModelTypes::select(
        &ir,
        &BTreeSet::from(
            [
                "demo.notes.Create.input",
                "demo.notes.Create.response",
                "demo.notes.Notes.row",
                "demo.notes.Summaries.row",
            ]
            .map(str::to_owned),
        ),
    )
    .unwrap();
    let result = Plan::from_model(&selected)
        .unwrap()
        .rust("model_types")
        .unwrap();
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        HTTP_BODY_WIRE,
    );
}

#[path = "support/empty_struct.rs"]
mod empty_struct;

/// The generated decoders of a struct with `fields: []` (ess/32, beyond10x/ess#649).
const EMPTY_STRUCT_WIRE: &str = r#"
#[cfg(test)]
mod empty_struct_wire {
    use model_types::*;
    use serde_json::{json, Value};

    fn round_trips<T: serde::de::DeserializeOwned + serde::Serialize>(body: Value) {
        let decoded: T = serde_json::from_value(body.clone()).unwrap();
        assert_eq!(serde_json::to_value(&decoded).unwrap(), body);
    }

    #[test]
    fn an_open_empty_struct_decodes_any_object() {
        round_trips::<DemoAckOpen>(json!({}));
        round_trips::<DemoAckOpen>(json!({"x": 1}));
        assert!(serde_json::from_value::<DemoAckOpen>(json!("x")).is_err());
    }

    #[test]
    fn a_closed_empty_struct_decodes_only_the_empty_object() {
        round_trips::<DemoAckClosed>(json!({}));
        assert!(serde_json::from_value::<DemoAckClosed>(json!({"x": 1})).is_err());
        assert!(serde_json::from_value::<DemoAckClosed>(json!("x")).is_err());
    }
}
"#;

#[test]
fn an_open_empty_struct_decodes_any_object() {
    let result = empty_struct::plan().rust("model_types").unwrap();
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        EMPTY_STRUCT_WIRE,
    );
}

#[test]
fn a_closed_empty_struct_decodes_only_the_empty_object() {
    let result = empty_struct::plan().rust("model_types").unwrap();
    assert!(
        result
            .declarations
            .contains("#[serde(deny_unknown_fields)]\npub struct DemoAckClosed {"),
        "{}",
        result.declarations
    );
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        EMPTY_STRUCT_WIRE,
    );
}

#[allow(dead_code)]
#[path = "../../ess-gen/tests/fixtures/declaration_order.rs"]
mod declaration_order;

/// The fixture model's plan, every root selected (beyond10x/ess#661).
fn declaration_order_plan() -> Plan {
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("demo.yaml"),
        ess_domain::spec::RawSpecFile::parse(declaration_order::MODEL).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap();
    let selected = ess_gen::schema::ModelTypes::select(
        &ir,
        &declaration_order::ROOTS.map(str::to_owned).into(),
    )
    .unwrap();
    Plan::from_model(&selected).unwrap()
}

/// With both options: `Outer` encodes its members in declaration order and decodes them in any
/// order to the same value, and the recursive types build with a `Box` only where a cycle of
/// by-value fields closes.
const DECLARED_ORDER_WIRE: &str = r##"
#[cfg(test)]
mod declared_order_wire {
    use model_types::*;

    const DECLARED: &str = r#"{"zeta":{"label":"z"},"alpha":"a","mid":{"label":"m"}}"#;
    const DEFAULT: &str = r#"{"alpha":"a","mid":{"label":"m"},"zeta":{"label":"z"}}"#;

    fn inner(label: &str) -> DemoGInner {
        DemoGInner { label: label.to_owned() }
    }

    #[test]
    fn members_encode_in_declaration_order_and_decode_in_any() {
        let outer = DemoGOuter { zeta: inner("z"), alpha: "a".to_owned(), mid: EssPresence::Present(inner("m")) };
        assert_eq!(serde_json::to_string(&outer).unwrap(), DECLARED);
        let declared: DemoGOuter = serde_json::from_str(DECLARED).unwrap();
        let default: DemoGOuter = serde_json::from_str(DEFAULT).unwrap();
        assert_eq!(declared, outer);
        assert_eq!(default, outer);
    }

    #[test]
    fn recursive_types_box_only_the_cycle() {
        let leaf = || DemoGNode { next: EssPresence::Absent, kids: ::std::vec::Vec::new() };
        let node = DemoGNode { next: EssPresence::Present(::std::boxed::Box::new(leaf())), kids: vec![leaf(), leaf()] };
        let holder = DemoGHolder { tree: node };
        let text = serde_json::to_string(&holder).unwrap();
        assert_eq!(text, r#"{"tree":{"next":{"kids":[]},"kids":[{"kids":[]},{"kids":[]}]}}"#);
        assert_eq!(serde_json::from_str::<DemoGHolder>(&text).unwrap(), holder);
        let a = DemoGA { b: EssPresence::Present(::std::boxed::Box::new(DemoGB { a: EssPresence::Present(::std::boxed::Box::new(DemoGA { b: EssPresence::Absent })) })) };
        let text = serde_json::to_string(&a).unwrap();
        assert_eq!(text, r#"{"b":{"a":{}}}"#);
        assert_eq!(serde_json::from_str::<DemoGA>(&text).unwrap(), a);
    }
}
"##;

/// Without the options: the same value encodes in wire-name order, and both texts decode to it.
const DEFAULT_ORDER_WIRE: &str = r##"
#[cfg(test)]
mod default_order_wire {
    use model_types::*;

    const DECLARED: &str = r#"{"zeta":{"label":"z"},"alpha":"a","mid":{"label":"m"}}"#;
    const DEFAULT: &str = r#"{"alpha":"a","mid":{"label":"m"},"zeta":{"label":"z"}}"#;

    #[test]
    fn members_encode_in_wire_name_order_and_decode_in_any() {
        let inner = |label: &str| ::std::boxed::Box::new(DemoGInner { label: label.to_owned() });
        let outer = DemoGOuter { zeta: inner("z"), alpha: "a".to_owned(), mid: EssPresence::Present(inner("m")) };
        assert_eq!(serde_json::to_string(&outer).unwrap(), DEFAULT);
        assert_eq!(serde_json::from_str::<DemoGOuter>(DECLARED).unwrap(), outer);
        assert_eq!(serde_json::from_str::<DemoGOuter>(DEFAULT).unwrap(), outer);
    }
}
"##;

#[test]
fn declared_order_and_box_only_recursion_build_and_round_trip() {
    let plan = declaration_order_plan();
    let default = plan.rust("model_types").unwrap();
    compile(
        &default.supporting["Cargo.toml"],
        &default.declarations,
        DEFAULT_ORDER_WIRE,
    );
    let options = schema_contract::realize::RustOptions {
        box_only_recursion: true,
        ..schema_contract::realize::RustOptions::default()
    };
    let ordered = plan
        .with_declared_field_order()
        .unwrap()
        .rust_with("model_types", &options)
        .unwrap();
    compile(
        &ordered.supporting["Cargo.toml"],
        &ordered.declarations,
        DECLARED_ORDER_WIRE,
    );
}

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod normalization_model;

#[path = "fixtures/cycle_shapes.rs"]
mod cycle_shapes;

/// The cycle shapes and records the fixture above leaves out (beyond10x/ess#661): a three-type
/// cycle, recursion through a union payload, a map and an optional list, a type reached from
/// inside and outside its cycle, an open struct and a shorthand struct.
fn cycle_shapes_plan() -> Plan {
    Plan::from_model(&normalization_model::selection(
        cycle_shapes::SOURCE,
        &cycle_shapes::ROOTS,
    ))
    .unwrap()
}

const CYCLE_SHAPES_WIRE: &str = r##"
use demo_r::*;

fn round_trip<T>(text: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize + PartialEq + std::fmt::Debug,
{
    let value: T = serde_json::from_str(text).unwrap();
    assert_eq!(serde_json::to_string(&value).unwrap(), text);
    let again: T = serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap();
    assert_eq!(again, value);
}

#[test]
fn every_cycle_shape_round_trips() {
    round_trip::<DemoRC1>(r#"{"c2":{"c3":{"back":{"c2":{"c3":{}},"label":"in"}}},"label":"out"}"#);
    round_trip::<DemoRExpr>(r#"{"kind":"neg","value":{"inner":{"kind":"neg","value":{"inner":{"kind":"lit","value":"x"}}}}}"#);
    round_trip::<DemoRDir>(r#"{"entries":{"a":{"entries":{},"kids":[]}},"kids":[{"entries":{},"kids":[],"parent":[]}],"parent":[{"entries":{},"kids":[]}]}"#);
    round_trip::<DemoROutside>(r#"{"entry":{"next":{"leaf":{"zeta":"z","alpha":"a"}},"leaf":{"zeta":"y","alpha":"b"}},"also":{"leaf":{"zeta":"x","alpha":"c"}},"expr":{"kind":"lit","value":"v"}}"#);
    // The open record drops its unknown member and writes the declared order.
    let open: DemoROpen = serde_json::from_str(r#"{"aaa":"r","extra":1,"alpha":null,"zeta":"z"}"#).unwrap();
    assert_eq!(serde_json::to_string(&open).unwrap(), r#"{"zeta":"z","aaa":"r"}"#);
    let short: DemoRShort = serde_json::from_str(r#""z""#).unwrap();
    assert_eq!(serde_json::to_string(&short).unwrap(), r#"{"zeta":"z"}"#);
    round_trip::<DemoRShort>(r#"{"zeta":"z","alpha":"a"}"#);
}

#[test]
fn a_derived_ord_compares_in_declaration_order() {
    let first = DemoRLeaf { zeta: "a".to_owned(), alpha: "z".to_owned() };
    let second = DemoRLeaf { zeta: "b".to_owned(), alpha: "a".to_owned() };
    assert!(first < second);
    let mut set = std::collections::BTreeSet::new();
    set.insert(serde_json::from_str::<DemoRC1>(r#"{"c2":{"c3":{}},"label":"x"}"#).unwrap());
    set.insert(serde_json::from_str::<DemoRC1>(r#"{"c2":{"c3":{}},"label":"x"}"#).unwrap());
    assert_eq!(set.len(), 1);
}
"##;

/// Both options with every other Rust option on: the crate builds (no E0072 on any cycle shape),
/// each shape round-trips, and a derived `Ord` compares fields in declaration order, as the
/// option's documentation says.
#[test]
fn every_cycle_shape_builds_and_round_trips_with_every_option() {
    use schema_contract::realize::{RustDerive, RustOptions};
    let options = RustOptions {
        derives: BTreeSet::from([
            RustDerive::Eq,
            RustDerive::Hash,
            RustDerive::PartialOrd,
            RustDerive::Ord,
        ]),
        fold_null: true,
        drop_unknown_members: true,
        box_only_recursion: true,
        ..RustOptions::default()
    };
    let realization = cycle_shapes_plan()
        .with_declared_field_order()
        .unwrap()
        .rust_with("demo_r", &options)
        .unwrap();
    compile(
        &realization.supporting["Cargo.toml"],
        &realization.declarations,
        CYCLE_SHAPES_WIRE,
    );
}

const OPEN_RECORD_WIRE: &str = r##"
use demo_r::*;

#[test]
fn the_open_record_writes_declared_members_first_and_keeps_the_rest() {
    let open: DemoROpen = serde_json::from_str(r#"{"extra":1,"aaa":"r","alpha":"a","zeta":"z"}"#).unwrap();
    assert_eq!(serde_json::to_string(&open).unwrap(), r#"{"zeta":"z","alpha":"a","aaa":"r","extra":1}"#);
}
"##;

/// An open record under declared order alone: declared members in declaration order, then the
/// members it keeps.
#[test]
fn an_open_record_builds_and_encodes_in_declaration_order() {
    let options = schema_contract::realize::RustOptions {
        box_only_recursion: true,
        ..schema_contract::realize::RustOptions::default()
    };
    let realization = cycle_shapes_plan()
        .with_declared_field_order()
        .unwrap()
        .rust_with("demo_r", &options)
        .unwrap();
    compile(
        &realization.supporting["Cargo.toml"],
        &realization.declarations,
        OPEN_RECORD_WIRE,
    );
}

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::Plan;
use serde_json::{json, Value};

fn plan(schemas: &Value, selected: &[&str]) -> Plan {
    let roots = selected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    let bundle = import(
        &json!({"components": {"schemas": schemas}}).to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap();
    Plan::from_bundle(&bundle, &roots).unwrap()
}

#[test]
fn generated_library_preserves_presence_open_data_unions_and_exact_numbers() {
    let plan = plan(
        &json!({
            "Record": {"type": "object", "required": ["requiredNullable", "requiredValue"], "properties": {
                "requiredNullable": {"type": ["string", "null"]}, "requiredValue": true,
                "optionalNullable": {"type": ["string", "null"]}, "defaultValue": {"type": "string", "default": "seed"}
            }},
            "Closed": {"type": "object", "additionalProperties": false, "required": ["type"], "properties": {"type": {"type": "boolean"}}},
            "Choice": {"anyOf": [{"$ref": "#/components/schemas/Record"}, {"$ref": "#/components/schemas/Closed"}]},
            "Tag": {"type": "string", "enum": ["ready", "paused"]},
            "Narrow": {"$ref": "#/components/schemas/Tag", "const": "ready", "type": "string"},
            "Tuple": {"type": "array", "prefixItems": [{"$ref": "#/components/schemas/Tag"}, true, {"type": "string"}], "minItems": 3, "maxItems": 3},
            "Node": {"type": "object", "additionalProperties": false, "properties": {"children": {"type": "array", "items": {"$ref": "#/components/schemas/Node"}}}},
            "Dictionary": {"type": "object", "additionalProperties": {"type": "integer"}},
            "Impossible": false
        }),
        &[
            "Record",
            "Closed",
            "Choice",
            "Tag",
            "Narrow",
            "Tuple",
            "Node",
            "Dictionary",
            "Impossible",
        ],
    );
    let result = plan.rust("contract_types").unwrap();
    assert_eq!(result, plan.rust("contract_types").unwrap());
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        report["configuration"],
        json!({"language": "rust", "package": "contract_types"})
    );
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        include_str!("fixtures/native_wire_tests.rs.txt"),
    );
}

#[test]
fn unsupported_native_shapes_and_names_refuse_before_emission() {
    let plan = plan(
        &json!({
            "Intersection": {"allOf": [{"type": "string"}, {"type": "boolean"}]},
            "Tuple": {"type": "array", "prefixItems": [{"type": "string"}]},
            "Collision": {"type": "object", "properties": {"a-b": true, "a_b": true}},
            "Literal": {"const": 3}
        }),
        &["Intersection", "Tuple", "Collision", "Literal"],
    );
    let errors = plan.rust("contract_types").unwrap_err();
    for rule in [
        "rust_intersection_or_literal",
        "rust_prefix_layout",
        "rust_field_collision",
    ] {
        assert!(errors.0.iter().any(|error| error.rule == rule), "{rule}");
    }
    for package in ["", "Upper", "../escape", "a\"\n", "serde", "self"] {
        assert!(plan.rust(package).is_err());
    }
}

#[test]
fn hostile_source_names_remain_comments_and_wire_strings() {
    let name = "name\ncompile_error!(\"injected\");";
    let plan = plan(
        &json!({name: {"type": "object", "properties": {"quote\"\nkey": {"type": "string"}}}}),
        &[name],
    );
    let result = plan.rust("quoted_types").unwrap();
    assert!(!result.declarations.contains("\ncompile_error!"));
    compile(&result.supporting["Cargo.toml"], &result.declarations, "");
}

fn compile(manifest: &str, source: &str, tests: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "native-types-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    fs::write(root.join("types.rs"), source).unwrap();
    if !tests.is_empty() {
        fs::write(root.join("tests/wire.rs"), tests).unwrap();
    }
    let result = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-types-target"),
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
