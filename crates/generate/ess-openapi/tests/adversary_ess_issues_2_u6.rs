//! Adversary pass on open objects, empty schemas and nullable pairs (beyond10x/ess#647): replay
//! against envelopes the `ess 0.76.0` release binary wrote, nullable-pair edges, the
//! `additionalProperties` matrix, and re-projection.

use std::fmt::Write as _;

use ess_openapi::{import, project_import, read_import, ImportReport, Refusal};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Documents 0.76.0 accepted, with the SHA-256 of the envelope `ess 0.76.0 infra import openapi
/// --out` wrote for each.
const STORED_0_76: [(&str, &str, &str); 5] = [
    (
        "wide 3.1",
        include_str!("fixtures/adversary-ess-issues-2-u6.wide31.yaml"),
        "0e9065699f2c8e983eee3d5477dee36701a047d52cf32af3bf824ddf87173937",
    ),
    (
        "wide 3.0",
        include_str!("fixtures/adversary-ess-issues-2-u6.wide30.yaml"),
        "03de6e9a8eb8c246b0577614f71239f8bee62436e34b6e8d88a31ca5e69bcaac",
    ),
    (
        "supported",
        include_str!("fixtures/supported.openapi.yaml"),
        "bcc835dd0b8c711ad62fe25811fa4b6639f5abf0ecfb5de7fdf5002955908433",
    ),
    (
        "supported projected",
        include_str!("fixtures/supported.projected.yaml"),
        "df525058720b37b4d33c9f0f7722e4bf39b8350f923ef06d7a7b97433d0c1e44",
    ),
    (
        "items service",
        include_str!("../../ess-gen/tests/fixtures/openapi-http/items-service.stage-1.yaml"),
        "ee1aaec7846d9d8fab1a4fbe72a843f81aa567a52039fd70d2f06f39ebdea9c9",
    ),
];

fn sha256(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        })
}

#[allow(clippy::needless_pass_by_value)]
fn document(schemas: Value) -> Value {
    json!({
        "openapi": "3.1.0", "info": {"title": "demo.adversary", "version": "v1"},
        "paths": {}, "components": {"schemas": schemas}
    })
}

/// `schemas` with `A` holding one required property `p` of `schema`.
#[allow(clippy::needless_pass_by_value)]
fn property(schema: Value, mut schemas: Value) -> Value {
    schemas["A"] = json!({
        "type": "object", "additionalProperties": false,
        "properties": {"p": schema}, "required": ["p"]
    });
    document(schemas)
}

const P: &str = "/components/schemas/A/properties/p";

fn imported(source: &Value) -> ImportReport {
    import(&source.to_string()).unwrap_or_else(|refusals| panic!("{source}: {refusals:?}"))
}

fn refused(source: &Value) -> Vec<Refusal> {
    import(&source.to_string())
        .map(|report| report.to_canonical_json())
        .expect_err("refused")
}

fn refused_at(refusals: &[Refusal], pointer: &str) {
    assert!(
        refusals.iter().any(|refusal| refusal.pointer == pointer),
        "no refusal at {pointer}: {refusals:?}"
    );
}

fn node(report: &ImportReport, pointer: &str) -> Value {
    let wire: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    wire.pointer(pointer)
        .unwrap_or_else(|| panic!("no {pointer} in {wire}"))
        .clone()
}

fn gaps(report: &ImportReport) -> Vec<String> {
    report
        .accounting()
        .coverage_gaps
        .iter()
        .map(|gap| gap.pointer.clone())
        .collect()
}

/// Replay, checked projection and reimport of the projection give the same IR.
fn round_trips(report: &ImportReport) -> String {
    assert_eq!(&read_import(&report.to_canonical_json()).unwrap(), report);
    let projected = project_import(report).unwrap_or_else(|refusals| panic!("{refusals:?}"));
    let reimported = import(&projected).unwrap_or_else(|refusals| panic!("{refusals:?}"));
    assert_eq!(reimported.interface(), report.interface(), "{projected}");
    projected
}

/// Every envelope 0.76.0 wrote for these documents is written again byte for byte, and replays.
#[test]
fn every_0_76_envelope_is_rewritten_byte_for_byte_and_replays() {
    for (label, source, frozen) in STORED_0_76 {
        let report = import(source).unwrap_or_else(|refusals| panic!("{label}: {refusals:?}"));
        let envelope = report.to_canonical_json();
        assert_eq!(sha256(&envelope), frozen, "{label}");
        assert_eq!(read_import(&envelope).unwrap(), report, "{label}");
    }
}

/// Under `oneOf`, a `null` that both branches admit fails the union, so a pair whose `$ref`
/// member admits `null` refuses `null` where `optional` admits it. The decision refuses a pair
/// whose other member admits `null`, naming "another `optional`"; a `$ref` hides it here.
#[test]
fn a_one_of_whose_ref_member_admits_null_stays_refused() {
    let mut imported_anyway = Vec::new();
    for (label, target) in [
        (
            "an optional component",
            json!({"anyOf": [{"type": "string"}, {"type": "null"}]}),
        ),
        ("a json component", json!({})),
        (
            "a [string, null] component",
            json!({"type": ["string", "null"]}),
        ),
    ] {
        let source = property(
            json!({"oneOf": [{"$ref": "#/components/schemas/M"}, {"type": "null"}]}),
            json!({ "M": target }),
        );
        match import(&source.to_string()) {
            Err(refusals) => refused_at(&refusals, &format!("{P}/oneOf")),
            Ok(report) => {
                let projected = project_import(&report).unwrap_or_default();
                imported_anyway.push(format!(
                    "{label}: imported as {} and projected as {}",
                    node(&report, "/interface/types/A/properties/p"),
                    projected
                        .lines()
                        .skip_while(|line| !line.contains("p:"))
                        .take(6)
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
            }
        }
    }
    assert!(imported_anyway.is_empty(), "{imported_anyway:#?}");
}

/// Three members, two nulls, `null` beside an empty or `true` member, a lone member, and a
/// `type: ['null']` spelling stay refused at the keyword.
#[test]
fn nullable_pair_edges_stay_refused_at_the_keyword() {
    for (keyword, members) in [
        (
            "anyOf",
            json!([{"type": "string"}, {"type": "integer"}, {"type": "null"}]),
        ),
        ("oneOf", json!([{"type": "null"}, {"type": "null"}])),
        ("anyOf", json!([{}, {"type": "null"}])),
        ("oneOf", json!([true, {"type": "null"}])),
        ("anyOf", json!([{"type": "string"}])),
        ("anyOf", json!([{"type": "string"}, {"type": ["null"]}])),
        ("oneOf", json!([{"type": "string"}, {"const": null}])),
        (
            "anyOf",
            json!([{"type": "string", "nullable": true}, {"type": "null"}]),
        ),
    ] {
        let source = property(json!({ keyword: members }), json!({}));
        refused_at(&refused(&source), &format!("{P}/{keyword}"));
    }
}

/// Assertions beside a nullable pair are not dropped silently: each is a gap, which checked
/// projection then refuses.
#[test]
fn keywords_beside_a_nullable_pair_are_gaps_not_silent_drops() {
    for (sibling, value) in [
        ("discriminator", json!({"propertyName": "kind"})),
        ("minLength", json!(1)),
        ("properties", json!({"q": {"type": "string"}})),
        ("nullable", json!(true)),
    ] {
        let mut schema = json!({"oneOf": [{"$ref": "#/components/schemas/B"}, {"type": "null"}]});
        schema[sibling] = value;
        let report = imported(&property(schema, json!({"B": {"type": "string"}})));
        let pointer = format!("{P}/{sibling}");
        assert!(
            gaps(&report).contains(&pointer),
            "{sibling}: {:?}",
            gaps(&report)
        );
        assert!(project_import(&report).is_err(), "{sibling}");
    }
}

/// A nullable pair nested in array items, in a typed map and in a tuple round-trips through
/// projection, and both keywords and both orders give one IR.
#[test]
fn nested_nullable_pairs_round_trip() {
    let inner = json!({"anyOf": [{"type": "string"}, {"type": "null"}]});
    let mut interfaces = Vec::new();
    for (keyword, members) in [
        (
            "anyOf",
            json!([{"type": "array", "items": inner}, {"type": "null"}]),
        ),
        (
            "oneOf",
            json!([{"type": "null"}, {"type": "array", "items": inner}]),
        ),
    ] {
        let report = imported(&property(json!({ keyword: members }), json!({})));
        round_trips(&report);
        interfaces.push(node(&report, "/interface/types/A/properties/p"));
    }
    assert_eq!(interfaces[0], interfaces[1]);
    assert_eq!(
        interfaces[0],
        json!({"kind": "optional", "schema": {"kind": "array", "items": {
            "kind": "optional", "schema": {"kind": "string"}
        }}})
    );
    for schema in [
        json!({"type": "object", "additionalProperties": inner}),
        json!({"type": "array", "prefixItems": [inner, true], "items": false}),
    ] {
        round_trips(&imported(&property(schema, json!({}))));
    }
}

/// `additionalProperties` absent, `true`, `{}` and an annotation-only schema give one IR; a
/// schema types the extra members beside declared ones; `required` beside each.
#[test]
fn the_additional_properties_matrix() {
    let open = |additional: Option<Value>| {
        let mut schema = json!({
            "type": "object", "properties": {"a": {"type": "string"}}, "required": ["a", "x"]
        });
        if let Some(additional) = additional {
            schema["additionalProperties"] = additional;
        }
        let report = imported(&document(json!({ "A": schema })));
        round_trips(&report);
        node(&report, "/interface/types/A")
    };
    let expected = json!({
        "kind": "object", "properties": {"a": {"kind": "string"}}, "required": ["a", "x"],
        "additional_properties": {"kind": "json"}
    });
    for additional in [
        None,
        Some(json!(true)),
        Some(json!({})),
        Some(json!({"description": "anything"})),
    ] {
        assert_eq!(open(additional.clone()), expected, "{additional:?}");
    }
    assert_eq!(
        open(Some(json!({"type": "integer"}))),
        json!({
            "kind": "object", "properties": {"a": {"kind": "string"}}, "required": ["a", "x"],
            "additional_properties": {"kind": "integer"}
        })
    );
    refused_at(
        &refused(&document(json!({"A": {
            "type": "object", "properties": {"a": {"type": "string"}}, "required": ["x"],
            "additionalProperties": false
        }}))),
        "/components/schemas/A/required",
    );
    refused_at(
        &refused(&document(json!({"A": {
            "type": "object", "additionalProperties": {"type": ["string"]}
        }}))),
        "/components/schemas/A/additionalProperties/type",
    );
    refused_at(
        &refused(&document(json!({"A": {
            "type": "object", "additionalProperties": false, "required": ["x"]
        }}))),
        "/components/schemas/A/required",
    );
    let closed = imported(&document(json!({"A": {
        "type": "object", "additionalProperties": false
    }})));
    assert_eq!(
        node(&closed, "/interface/types/A"),
        json!({"kind": "object", "properties": {}})
    );
    round_trips(&closed);
}

/// The projection of an open object and of an array without `items` is the source's meaning:
/// `additionalProperties: {}`, `items: {}`; reimport gives the same IR.
#[test]
fn projection_spells_open_members_and_items_as_the_empty_schema() {
    let report = imported(&document(json!({
        "A": {"type": "object", "properties": {"m": {"type": "array"}}}
    })));
    let projected = round_trips(&report);
    let value: Value = ess_yaml::from_str(&projected).unwrap();
    let a = &value["components"]["schemas"]["A"];
    assert_eq!(a["additionalProperties"], json!({}), "{projected}");
    assert_eq!(a["properties"]["m"]["items"], json!({}), "{projected}");
}
