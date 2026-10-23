//! The generated Rust crate's `Cargo.toml` takes compatible version ranges at the tested minimum,
//! so it never downgrades a crate a consumer workspace already locks (beyond10x/ess#660), and its
//! default-on `exact-numbers` feature is the only route to `serde_json/arbitrary_precision`
//! (beyond10x/ess#483).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use semver::{Version, VersionReq};

use super::plan;

/// Every dependency the generated manifest names, at the version this repository builds and
/// tests the generated crate against.
const TESTED: [(&str, &str); 3] = [
    ("serde", "1.0.229"),
    ("serde_json", "1.0.151"),
    ("time", "0.3.45"),
];

fn manifest() -> String {
    plan().rust("probe-meter").expect("rust").supporting["Cargo.toml"].clone()
}

/// The `[dependencies]` table of a generated manifest, name to version requirement.
fn dependencies(manifest: &str) -> BTreeMap<String, String> {
    let table = manifest
        .split("\n[dependencies]\n")
        .nth(1)
        .unwrap_or_else(|| panic!("no dependencies table:\n{manifest}"));
    let table = table.split("\n[").next().unwrap_or(table);
    table
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (name, value) = line
                .split_once(" = ")
                .unwrap_or_else(|| panic!("unreadable dependency line {line:?}"));
            let requirement = match value.strip_prefix("{ version = \"") {
                Some(rest) => rest.split('"').next().unwrap(),
                None => value.trim_matches('"'),
            };
            (name.to_owned(), requirement.to_owned())
        })
        .collect()
}

#[test]
fn the_manifest_uses_caret_ranges() {
    let manifest = manifest();
    let dependencies = dependencies(&manifest);
    let tested = TESTED
        .iter()
        .map(|(name, version)| ((*name).to_owned(), format!("^{version}")))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(dependencies, tested, "{manifest}");
    assert!(
        !manifest.contains("\"="),
        "an exact pin remains:\n{manifest}"
    );
}

/// Offline, so at the manifest: a consumer whose lock already holds a newer compatible release of
/// each dependency keeps it, because the generated requirement admits it; the tested minimum is
/// admitted and nothing older is.
#[test]
fn a_consumer_lock_with_newer_patches_resolves() {
    let dependencies = dependencies(&manifest());
    for (name, tested) in TESTED {
        let requirement = VersionReq::parse(&dependencies[name]).unwrap();
        let tested = Version::parse(tested).unwrap();
        let mut newer = vec![Version::new(tested.major, tested.minor, tested.patch + 1)];
        let mut incompatible = vec![Version::new(tested.major + 1, 0, 0)];
        if tested.major == 0 {
            incompatible.push(Version::new(0, tested.minor + 1, 0));
        } else {
            newer.push(Version::new(tested.major, tested.minor + 1, 0));
        }
        assert!(requirement.matches(&tested), "{name} {requirement}");
        for version in newer {
            assert!(
                requirement.matches(&version),
                "{name} {requirement} would downgrade a consumer locking {version}"
            );
        }
        for version in incompatible {
            assert!(
                !requirement.matches(&version),
                "{name} {requirement} {version}"
            );
        }
        if tested.patch > 0 {
            let older = Version::new(tested.major, tested.minor, tested.patch - 1);
            assert!(!requirement.matches(&older), "{name} {requirement} {older}");
        }
    }
}

/// `exact-numbers` stays default-on (the decision on beyond10x/ess#483 and #660), and the
/// dependency line never names `arbitrary_precision`, so `default-features = false` drops it.
#[test]
fn the_number_feature_default_is_decided() {
    let manifest = manifest();
    assert!(
        manifest.ends_with(
            "\n[features]\ndefault = [\"exact-numbers\"]\nexact-numbers = [\"serde_json/arbitrary_precision\"]\n\n[workspace]\n"
        ),
        "{manifest}"
    );
    assert_eq!(
        manifest.matches("arbitrary_precision").count(),
        1,
        "only the exact-numbers feature may enable arbitrary_precision:\n{manifest}"
    );
    let changelog =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../CHANGELOG.md"))
            .unwrap();
    assert!(
        changelog.lines().any(|line| line.contains("exact-numbers")
            && line.contains("default-on")
            && line.contains("default-features = false")),
        "the changelog names the exact-numbers default"
    );
}
