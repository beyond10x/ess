//! Complete output maps frozen before TypeScript; only the truthful producer line is projected.

// Historical fixtures own private helper modules. Import them intact so this
// byte witness exercises the original plans without rewriting frozen fixtures.
#![allow(clippy::duplicate_mod)]

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
#[allow(dead_code)]
#[path = "fixtures/normalization_binary64.rs"]
mod v5;
#[allow(dead_code)]
#[path = "fixtures/normalization_positional.rs"]
mod v6;

#[path = "support/generator_version.rs"]
mod generator_version;

use schema_contract::realize::normalize::Plan;

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
        v5::plan(),
        v6::plan(),
    ];
    let capture_root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("normalization-typescript-legacy-current");
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
            for (path, source) in &target.files {
                let destination = capture_root
                    .join(format!("v{}-{language}", index + 1))
                    .join(path);
                std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
                std::fs::write(destination, source).unwrap();
            }
            let mut files = BTreeMap::new();
            for (path, mut source) in target.files {
                if path == "normalization-report.json" {
                    source =
                        generator_version::report_at_baseline(&source, env!("CARGO_PKG_VERSION"))
                            .unwrap();
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
    std::fs::write(capture_root.join("canonical-maps.json"), &actual).unwrap();
    let expected = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/normalization_typescript_legacy_maps.json"
    ))
    .unwrap();
    assert_eq!(actual, expected);
}
