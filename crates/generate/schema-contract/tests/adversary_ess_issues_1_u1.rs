//! Adversarial cases against the types-only realization of the invariant keywords of open-issue
//! batch 1, unit U1 (beyond10x/ess#685).
//!
//! A `String` newtype's `value.count` bounds are published as `minLength` and `maxLength`, and the
//! realizer adds no obligation for a length keyword that only restates the text constraint it
//! enforces. The mapping form `value.count: {gte: 1, lte: 8}` is the same pair of bounds; the
//! report must not count it twice either.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use schema_contract::realize::Plan;

const LENGTHS: &str = r"format: ess/20
system: probe
version: v1
domains: [probe.units]
domain: probe.units
types:
  - name: probe.units.Paired
    kind: newtype
    of: String
    invariants: [value.count >= 1, value.count <= 8]
  - name: probe.units.Mapped
    kind: newtype
    of: String
    invariants:
      - value.count: {gte: 1, lte: 8}
  - name: probe.units.Measure
    kind: struct
    fields:
      - {name: paired, type: probe.units.Paired}
      - {name: mapped, type: probe.units.Mapped}
";

/// The rules a report leaves at a pointer naming `name`.
fn rules_at(report: &schema_contract::realize::Report, name: &str) -> Vec<String> {
    let report = serde_json::to_value(report).expect("report");
    report["obligations"]
        .as_array()
        .expect("obligations")
        .iter()
        .filter(|item| {
            item["pointer"]
                .as_str()
                .unwrap_or_default()
                .contains(&format!("/{name}"))
        })
        .map(|item| item["rule"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// The mapping form and the pair publish the same keywords and leave the same obligations.
#[test]
fn the_mapping_form_length_is_reported_as_the_pair_is() {
    let selection = model::selection(LENGTHS, &["probe.units.Measure"]);
    let definitions: serde_json::Value = serde_json::from_str(&selection.to_json()).expect("JSON");
    for name in ["probe.units.Paired", "probe.units.Mapped"] {
        let node = &definitions["$defs"][name];
        assert_eq!(node["minLength"], serde_json::json!(1), "{name}: {node:#}");
        assert_eq!(node["maxLength"], serde_json::json!(8), "{name}: {node:#}");
    }

    let plan = Plan::from_model(&selection).expect("plan");
    let reports = [
        ("typescript", plan.typescript().report),
        ("rust", plan.rust("probe-units").expect("rust").report),
        (
            "go",
            plan.go("units", "example.invalid/units")
                .expect("go")
                .report,
        ),
    ];
    for (target, report) in &reports {
        let paired = rules_at(report, "probe.units.Paired");
        let mapped = rules_at(report, "probe.units.Mapped");
        assert_eq!(
            mapped, paired,
            "{target}: the mapping form leaves other obligations than the pair; x-ess-invariants \
             {}",
            definitions["$defs"]["probe.units.Mapped"]["x-ess-invariants"]
        );
    }
}
