//! Open objects, empty schemas and nullable unions import (beyond10x/ess#647): `{}` and `true` as
//! `json`, an open object with its extra-member schema, a nullable `anyOf`/`oneOf` pair as
//! `optional`. Every input accepted before keeps its bytes; the new nodes arise only from inputs
//! that were refused.

use ess_openapi::{import, project_import, read_import, AccountingCode, ImportReport, Refusal};
use serde_json::{json, Value};

#[allow(dead_code)]
#[path = "fixtures/interface_schema_0_76.rs"]
mod interface_schema_0_76;

fn document(version: &str) -> Value {
    json!({
        "openapi": version, "info": {"title": "demo.open", "version": "v1"},
        "paths": {}, "components": {"schemas": {"B": {"type": "integer"}}}
    })
}

/// The schema placed at one of five sites: a component, a required property, a request, a
/// response, and array items. Returns the document, the source pointer and the interface pointer.
fn at_version(version: &str, schema: Value, site: usize) -> (Value, &'static str, &'static str) {
    let mut source = document(version);
    match site {
        0 => {
            source["components"]["schemas"]["A"] = schema;
            (source, "/components/schemas/A", "/interface/types/A")
        }
        1 => {
            source["components"]["schemas"]["A"] = json!({
                "type": "object", "additionalProperties": false,
                "properties": {"p~/": schema}, "required": ["p~/"]
            });
            (
                source,
                "/components/schemas/A/properties/p~0~1",
                "/interface/types/A/properties/p~0~1",
            )
        }
        2 | 3 => {
            source["paths"] = json!({"/probe": {"post": {
                "operationId": "probe", "responses": {"200": {"description": "ok"}}
            }}});
            if site == 2 {
                source["paths"]["/probe"]["post"]["requestBody"] = json!({
                    "content": {"application/json": {"schema": schema}}
                });
                (
                    source,
                    "/paths/~1probe/post/requestBody/content/application~1json/schema",
                    "/interface/operations/probe/request/schema",
                )
            } else {
                source["paths"]["/probe"]["post"]["responses"]["200"]["content"] =
                    json!({"application/json": {"schema": schema}});
                (
                    source,
                    "/paths/~1probe/post/responses/200/content/application~1json/schema",
                    "/interface/operations/probe/responses/200/schema",
                )
            }
        }
        4 => {
            source["components"]["schemas"]["A"] = json!({"type": "array", "items": schema});
            (
                source,
                "/components/schemas/A/items",
                "/interface/types/A/items",
            )
        }
        _ => unreachable!(),
    }
}

fn at_site(schema: Value, site: usize) -> (Value, &'static str, &'static str) {
    at_version("3.1.0", schema, site)
}

fn imported(source: &Value) -> ImportReport {
    import(&source.to_string()).unwrap_or_else(|refusals| panic!("{source}: {refusals:?}"))
}

fn refused(source: &Value) -> Vec<Refusal> {
    import(&source.to_string()).expect_err("the source is refused")
}

fn node(report: &ImportReport, wire_pointer: &str) -> Value {
    let wire: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    wire.pointer(wire_pointer)
        .unwrap_or_else(|| panic!("no {wire_pointer} in {wire}"))
        .clone()
}

fn gap_pointers(report: &ImportReport) -> Vec<String> {
    report
        .accounting()
        .coverage_gaps
        .iter()
        .map(|gap| gap.pointer.clone())
        .collect()
}

/// The envelope replays, checked projection succeeds, and the projection reimports to an equal IR.
fn round_trips(report: &ImportReport) -> String {
    let checked = read_import(&report.to_canonical_json()).unwrap();
    assert_eq!(&checked, report);
    let projected = project_import(&checked).unwrap_or_else(|refusals| panic!("{refusals:?}"));
    let reimported = import(&projected).unwrap_or_else(|refusals| panic!("{refusals:?}"));
    assert_eq!(reimported.interface(), report.interface(), "{projected}");
    projected
}

fn refused_at(refusals: &[Refusal], pointer: &str) {
    assert!(
        refusals.iter().any(|refusal| refusal.pointer == pointer),
        "no refusal at {pointer}: {refusals:?}"
    );
}

#[test]
fn an_empty_schema_imports_as_json() {
    for schema in [json!({}), json!(true), json!({"description": "any value"})] {
        for site in 0..5 {
            let (source, pointer, wire_pointer) = at_site(schema.clone(), site);
            let report = imported(&source);
            assert_eq!(
                node(&report, wire_pointer),
                json!({"kind": "json"}),
                "{schema}"
            );
            assert!(gap_pointers(&report).is_empty(), "{report:?}");
            if schema.get("description").is_some() {
                assert!(report.accounting().normalizations.iter().any(|entry| {
                    entry.pointer == format!("{pointer}/description")
                        && entry.code == AccountingCode::AnnotationOmitted
                }));
            }
            round_trips(&report);
        }
    }
}

#[test]
fn an_open_object_imports_with_any_other_member() {
    for schema in [
        json!({"type": "object", "properties": {"a": {"type": "string"}}, "required": ["a"]}),
        json!({
            "type": "object", "properties": {"a": {"type": "string"}}, "required": ["a"],
            "additionalProperties": true
        }),
    ] {
        for site in 0..5 {
            let (source, _, wire_pointer) = at_site(schema.clone(), site);
            let report = imported(&source);
            assert_eq!(
                node(&report, wire_pointer),
                json!({
                    "kind": "object", "properties": {"a": {"kind": "string"}}, "required": ["a"],
                    "additional_properties": {"kind": "json"}
                })
            );
            assert!(gap_pointers(&report).is_empty(), "{report:?}");
            round_trips(&report);
        }
    }
}

#[test]
fn a_typed_map_imports_its_member_schema() {
    for site in 0..5 {
        let (source, _, wire_pointer) = at_site(
            json!({"type": "object", "additionalProperties": {"type": "array", "items": {"$ref": "#/components/schemas/B"}}}),
            site,
        );
        let report = imported(&source);
        assert_eq!(
            node(&report, wire_pointer),
            json!({
                "kind": "object", "properties": {},
                "additional_properties": {"kind": "array", "items": {"kind": "reference", "name": "B"}}
            })
        );
        assert!(gap_pointers(&report).is_empty(), "{report:?}");
        round_trips(&report);
    }
}

#[test]
fn an_object_without_properties_imports() {
    for (schema, expected) in [
        (
            json!({"type": "object"}),
            json!({"kind": "object", "properties": {}, "additional_properties": {"kind": "json"}}),
        ),
        (
            json!({"type": "object", "additionalProperties": false}),
            json!({"kind": "object", "properties": {}}),
        ),
    ] {
        for site in 0..5 {
            let (source, _, wire_pointer) = at_site(schema.clone(), site);
            let report = imported(&source);
            assert_eq!(node(&report, wire_pointer), expected, "{schema}");
            round_trips(&report);
        }
    }
}

#[test]
fn an_array_without_items_imports_as_an_array_of_json() {
    for site in 0..5 {
        let (source, _, wire_pointer) = at_site(json!({"type": "array"}), site);
        let report = imported(&source);
        assert_eq!(
            node(&report, wire_pointer),
            json!({"kind": "array", "items": {"kind": "json"}})
        );
        assert!(gap_pointers(&report).is_empty(), "{report:?}");
        round_trips(&report);
    }
}

#[test]
fn a_nullable_any_of_imports_as_optional() {
    let null_branch = json!({"type": "null"});
    for keyword in ["anyOf", "oneOf"] {
        for (member, expected) in [
            (
                json!({"$ref": "#/components/schemas/B"}),
                json!({"kind": "reference", "name": "B"}),
            ),
            (
                json!({"type": "string", "format": "date"}),
                json!({"kind": "string", "format": "date"}),
            ),
        ] {
            for branches in [
                json!([member.clone(), null_branch.clone()]),
                json!([null_branch.clone(), member.clone()]),
            ] {
                for site in 0..5 {
                    let mut schema = json!({});
                    schema[keyword] = branches.clone();
                    let (source, pointer, wire_pointer) = at_site(schema.clone(), site);
                    let report = imported(&source);
                    assert_eq!(
                        node(&report, wire_pointer),
                        json!({"kind": "optional", "schema": expected}),
                        "{schema}"
                    );
                    assert!(gap_pointers(&report).is_empty(), "{report:?}");
                    assert_eq!(report.accounting().unresolved_references, []);
                    let projected = round_trips(&report);
                    let value: Value = ess_yaml::from_str(&projected).unwrap();
                    let written = value.pointer(pointer).unwrap();
                    assert_eq!(written["anyOf"][1], null_branch, "{projected}");
                }
            }
        }
    }
}

#[test]
fn the_other_null_spellings_keep_their_gap() {
    let detail = "the type admits `null` (a 3.1 union with `null`, or 3.0 `nullable: true`); \
                  this variant does not preserve the `null` member";
    for (version, schema, keyword) in [
        ("3.1.0", json!({"type": ["integer", "null"]}), "type"),
        ("3.1.0", json!({"type": ["null", "integer"]}), "type"),
        (
            "3.0.3",
            json!({"type": "integer", "nullable": true}),
            "nullable",
        ),
    ] {
        for site in 0..5 {
            let (source, pointer, wire_pointer) = at_version(version, schema.clone(), site);
            let report = imported(&source);
            assert_eq!(node(&report, wire_pointer), json!({"kind": "integer"}));
            let accounting = serde_json::to_value(report.accounting()).unwrap();
            assert_eq!(
                accounting["coverage_gaps"],
                json!([{
                    "pointer": format!("{pointer}/{keyword}"),
                    "code": "feature-unpreserved",
                    "detail": detail
                }])
            );
            assert!(project_import(&report).is_err());
        }
    }
}

#[test]
fn an_untagged_object_one_of_stays_refused() {
    let object = |name: &str| {
        json!({
            "type": "object", "properties": {name: {"type": "string"}}, "required": [name],
            "additionalProperties": false
        })
    };
    for (schema, keyword) in [
        (json!({"oneOf": [object("a"), object("b")]}), "oneOf"),
        (json!({"allOf": [object("a"), object("b")]}), "allOf"),
        (json!({"not": {"type": "string"}}), "not"),
        (
            json!({"anyOf": [{"type": "string"}, {"type": "integer"}, {"type": "null"}]}),
            "anyOf",
        ),
        (
            json!({"anyOf": [{"type": "string"}, {"type": "integer"}]}),
            "anyOf",
        ),
        (
            json!({"oneOf": [{"type": "null"}, {"type": "null"}]}),
            "oneOf",
        ),
    ] {
        for site in 0..5 {
            let (source, pointer, _) = at_site(schema.clone(), site);
            let refusals = refused(&source);
            let at = format!("{pointer}/{keyword}");
            refused_at(&refusals, &at);
            assert!(
                refusals
                    .iter()
                    .filter(|refusal| refusal.pointer == at)
                    .all(|refusal| refusal.message.contains(&format!("`{keyword}`"))),
                "the refusal does not name the union: {refusals:?}"
            );
        }
    }
}

#[test]
fn a_one_of_whose_other_branch_admits_null_stays_refused() {
    for (version, member) in [
        ("3.1.0", json!({"type": ["string", "null"]})),
        (
            "3.1.0",
            json!({"anyOf": [{"type": "string"}, {"type": "null"}]}),
        ),
        ("3.0.3", json!({"type": "string", "nullable": true})),
    ] {
        for site in 0..5 {
            let (source, pointer, _) = at_version(
                version,
                json!({"oneOf": [member.clone(), {"type": "null"}]}),
                site,
            );
            refused_at(&refused(&source), &format!("{pointer}/oneOf"));
        }
    }
}

#[test]
fn a_required_member_outside_properties_imports_on_an_open_object() {
    for (schema, additional) in [
        (
            json!({"type": "object", "properties": {"a": {"type": "string"}}, "required": ["a", "extra"]}),
            json!({"kind": "json"}),
        ),
        (
            json!({"type": "object", "additionalProperties": {"type": "integer"}, "required": ["n"]}),
            json!({"kind": "integer"}),
        ),
    ] {
        for site in 0..5 {
            let (source, _, wire_pointer) = at_site(schema.clone(), site);
            let report = imported(&source);
            let node = node(&report, wire_pointer);
            assert_eq!(node["required"], schema["required"]);
            assert_eq!(node["additional_properties"], additional);
            round_trips(&report);
        }
    }
}

#[test]
fn a_required_member_outside_properties_stays_refused_on_a_closed_object() {
    for site in 0..5 {
        let (source, pointer, _) = at_site(
            json!({
                "type": "object", "properties": {"a": {"type": "string"}},
                "required": ["a", "extra"], "additionalProperties": false
            }),
            site,
        );
        let refusals = refused(&source);
        refused_at(&refusals, &format!("{pointer}/required"));
        assert!(refusals
            .iter()
            .any(|refusal| refusal.message.contains("`extra`")));
    }
}

#[test]
fn a_refused_property_is_not_also_reported_as_undeclared() {
    for closed in [false, true] {
        for site in 0..5 {
            let mut schema = json!({
                "type": "object", "properties": {"b": {"type": ["string", "integer"]}},
                "required": ["b"]
            });
            if closed {
                schema["additionalProperties"] = json!(false);
            }
            let (source, pointer, _) = at_site(schema, site);
            let refusals = refused(&source);
            refused_at(&refusals, &format!("{pointer}/properties/b/type"));
            assert!(
                refusals
                    .iter()
                    .all(|refusal| refusal.pointer != format!("{pointer}/required")),
                "reported twice: {refusals:?}"
            );
        }
    }
}

#[test]
fn a_typeless_schema_with_assertions_stays_refused() {
    for schema in [
        json!({"minimum": 1}),
        json!({"properties": {"a": {"type": "string"}}}),
        json!({"required": ["a"]}),
        json!({"items": {"type": "string"}}),
        json!({"format": "int64", "description": "annotated"}),
        json!({"x-extension": true}),
    ] {
        for site in 0..5 {
            let (source, pointer, _) = at_site(schema.clone(), site);
            let refusals = refused(&source);
            assert!(
                refusals.iter().any(|refusal| refusal.pointer == pointer
                    && refusal.message == "schema must declare `$ref` or one supported `type`"),
                "{schema}: {refusals:?}"
            );
        }
    }
    for site in 0..5 {
        let (source, pointer, _) = at_site(json!(false), site);
        refused_at(&refused(&source), pointer);
    }
}

#[test]
fn an_openapi30_null_branch_stays_refused() {
    let null_branch = json!({"type": "null"});
    let member = json!({"$ref": "#/components/schemas/B"});
    for keyword in ["anyOf", "oneOf"] {
        for branches in [
            json!([member.clone(), null_branch.clone()]),
            json!([null_branch.clone(), member.clone()]),
        ] {
            for site in 0..5 {
                let mut schema = json!({});
                schema[keyword] = branches.clone();
                let (source, pointer, _) = at_version("3.0.3", schema, site);
                let refusals = refused(&source);
                let at = format!("{pointer}/{keyword}");
                assert!(
                    refusals
                        .iter()
                        .any(|refusal| refusal.pointer.starts_with(&at)),
                    "{refusals:?}"
                );
            }
        }
    }
}

/// The reported shape: an open `Meta` object, an untyped `data` member, and a nullable
/// reference.
const REPORTED: &str = include_str!("fixtures/open.openapi.yaml");

#[test]
fn the_reported_document_imports() {
    let report = import(REPORTED).unwrap_or_else(|refusals| panic!("{refusals:?}"));
    assert_eq!(
        gap_pointers(&report),
        ["/components/schemas/PingResponse/properties/count/format"]
    );
    assert_eq!(report.accounting().unresolved_references, []);
    assert_eq!(read_import(&report.to_canonical_json()).unwrap(), report);
    let types = serde_json::to_value(&report.interface().types).unwrap();
    assert_eq!(
        types["PingResponse"]["properties"]["audio_codec"],
        json!({"kind": "optional", "schema": {"kind": "reference", "name": "Codec"}})
    );
    assert_eq!(
        types["PingRequest"]["properties"]["data"],
        json!({"kind": "json"})
    );
    assert_eq!(
        types["Meta"],
        json!({"kind": "object", "properties": {}, "additional_properties": {"kind": "json"}})
    );
}

#[test]
fn a_0_76_reader_refuses_the_new_nodes_by_name() {
    let report = import(REPORTED).unwrap_or_else(|refusals| panic!("{refusals:?}"));
    let types = serde_json::to_value(&report.interface().types).unwrap();
    for (node, name) in [
        (
            &types["PingRequest"]["properties"]["data"],
            "unknown variant `json`",
        ),
        (
            &types["PingResponse"]["properties"]["audio_codec"],
            "unknown variant `optional`",
        ),
        (&types["Meta"], "unknown field `additional_properties`"),
    ] {
        let error = serde_json::from_value::<interface_schema_0_76::InterfaceSchema>(node.clone())
            .expect_err("the 0.76.0 reader refuses the node");
        assert!(error.to_string().contains(name), "{node}: {error}");
    }
}

#[test]
fn closed_import_rejects_unknown_json_variant_fields() {
    for (schema, field) in [
        (json!({}), "extra"),
        (
            json!({"anyOf": [{"type": "integer"}, {"type": "null"}]}),
            "extra",
        ),
    ] {
        for site in 0..5 {
            let (source, _, wire_pointer) = at_site(schema.clone(), site);
            let report = imported(&source);
            let mut wire: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
            wire.pointer_mut(wire_pointer).unwrap()[field] = json!(1);
            for bytes in [wire.to_string(), ess_yaml::to_string(&wire).unwrap()] {
                assert!(
                    read_import(&bytes).is_err(),
                    "admitted {wire_pointer}/{field}"
                );
            }
        }
    }
}
