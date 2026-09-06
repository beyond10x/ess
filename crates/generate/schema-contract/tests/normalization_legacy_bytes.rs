//! Complete legacy output snapshots, recorded before each successor format.

#[allow(dead_code)]
#[path = "fixtures/normalization_v1.rs"]
mod v1;
#[allow(dead_code)]
#[path = "fixtures/normalization_numeric.rs"]
mod v2;
use v4::model as v3;
#[allow(dead_code)]
#[path = "fixtures/normalization_raw.rs"]
mod v4;

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;

#[test]
fn complete_legacy_file_maps_are_preserved() {
    let (bundle, recipe) = v2::fixture();
    let plans = [
        v1::plan(),
        Plan::read(&recipe.to_string(), &[bundle]).unwrap(),
        v3::plan(),
        v4::plan(),
    ];
    let mut snapshots = BTreeMap::new();
    for (index, plan) in plans.iter().enumerate() {
        for (language, target) in [
            ("rust", plan.rust("normalization_adapter").unwrap()),
            (
                "go",
                plan.go(
                    "normalization_adapter",
                    "example.invalid/normalization-adapter",
                )
                .unwrap(),
            ),
        ] {
            let mut files = BTreeMap::new();
            for (path, mut source) in target.files {
                if path == "normalization-report.json" {
                    let mut report: Value = serde_json::from_str(&source).unwrap();
                    assert_eq!(report["generator_version"], env!("CARGO_PKG_VERSION"));
                    report["generator_version"] = json!("<current-generator-version>");
                    source = format!("{}\n", serde_json::to_string_pretty(&report).unwrap());
                }
                let mut digest = String::new();
                for byte in Sha256::digest(source.as_bytes()) {
                    write!(digest, "{byte:02x}").unwrap();
                }
                files.insert(path, digest);
            }
            snapshots.insert(format!("v{}-{language}", index + 1), files);
        }
    }
    let actual = format!("{}\n", serde_json::to_string_pretty(&snapshots).unwrap());
    let expected = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/normalization_legacy_maps.json"
    ))
    .unwrap();
    assert_eq!(actual, expected);
}
