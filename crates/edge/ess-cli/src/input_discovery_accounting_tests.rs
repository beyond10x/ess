use super::authored;
use std::{fs, path::Path};
use tempfile::TempDir;

fn write(root: &Path, name: &str, bytes: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().expect("fixture parent")).unwrap();
    fs::write(path, bytes).unwrap();
}

fn manifest_fixture() -> TempDir {
    let tree = tempfile::tempdir().unwrap();
    write(tree.path(), "scenario/z.yaml", "scenario: z\n");
    write(tree.path(), "scenario/a.input", "scenario: a\n");
    write(tree.path(), "inactive.yaml", "must not be read\n");
    write(
        tree.path(),
        "ess-inputs.yaml",
        "format: ess-inputs/1\nspecification: [inactive.yaml]\nscenarios: [scenario/z.yaml, scenario/a.input]\n",
    );
    tree
}

fn assert_manifest(coverage: bool) -> Vec<super::Input> {
    let tree = manifest_fixture();
    let rows = authored(Some(tree.path()), coverage).unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row.identity.as_str())
            .collect::<Vec<_>>(),
        ["scenario/a.input", "scenario/z.yaml"]
    );
    assert_eq!(rows[0].origin, tree.path().join("scenario/a.input"));
    assert_eq!(rows[0].text, "scenario: a\n");
    assert_eq!(rows[1].text, "scenario: z\n");

    write(
        tree.path(),
        "ess-inputs.yaml",
        "format: ess-inputs/1\nspecification: []\nscenarios: [../escape.yaml]\n",
    );
    assert!(authored(Some(tree.path()), coverage)
        .err()
        .expect("root traversal must be refused")
        .to_string()
        .contains("root-relative"));

    write(
        tree.path(),
        "ess-inputs.yaml",
        "format: ess-inputs/1\nspecification: []\nscenarios: [linked.yaml]\n",
    );
    std::os::unix::fs::symlink("scenario/a.input", tree.path().join("linked.yaml")).unwrap();
    assert!(authored(Some(tree.path()), coverage)
        .err()
        .expect("symlink must be refused")
        .to_string()
        .contains("contains a symlink"));
    rows
}

fn assert_legacy_directory(coverage: bool) -> Vec<super::Input> {
    let tree = tempfile::tempdir().unwrap();
    write(tree.path(), "z.yml", "scenario: z\n");
    write(tree.path(), "a.yaml", "scenario: a\n");
    write(tree.path(), "nested/ignored.yaml", "must not be read\n");
    let rows = authored(Some(tree.path()), coverage).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].origin, tree.path().join("a.yaml"));
    assert_eq!(rows[0].text, "scenario: a\n");
    assert_eq!(rows[1].origin, tree.path().join("z.yml"));
    assert_eq!(rows[1].text, "scenario: z\n");
    if coverage {
        assert_eq!(rows[0].identity, "a.yaml");
        assert_eq!(rows[1].identity, "z.yml");
    } else {
        assert_eq!(
            rows[0].identity,
            tree.path().join("a.yaml").display().to_string()
        );
        assert_eq!(
            rows[1].identity,
            tree.path().join("z.yml").display().to_string()
        );
    }
    rows
}

fn assert_direct_file(coverage: bool) -> Vec<super::Input> {
    let tree = tempfile::tempdir().unwrap();
    let path = tree.path().join("scenario.other");
    fs::write(&path, "scenario: direct\n").unwrap();
    let rows = authored(Some(&path), coverage).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].origin, path);
    assert_eq!(rows[0].text, "scenario: direct\n");
    assert_eq!(
        rows[0].identity,
        if coverage {
            "scenario.other".to_owned()
        } else {
            path.display().to_string()
        }
    );
    rows
}

fn omitted(coverage: bool) -> Vec<super::Input> {
    authored(None, coverage).unwrap()
}

#[test]
fn acquisition_authored_manifest() {
    assert_eq!(assert_manifest(false).len(), 2);
}

#[test]
fn acquisition_authored_legacy_directory() {
    assert_eq!(assert_legacy_directory(false).len(), 2);
}

#[test]
fn acquisition_authored_direct_file() {
    assert_eq!(assert_direct_file(false)[0].text, "scenario: direct\n");
}

#[test]
fn acquisition_authored_omitted_scenarios() {
    assert!(omitted(false).is_empty());
}

#[test]
fn acquisition_coverage_manifest() {
    assert_eq!(assert_manifest(true).len(), 2);
}

#[test]
fn acquisition_coverage_legacy_directory() {
    assert_eq!(assert_legacy_directory(true).len(), 2);
}

#[test]
fn acquisition_coverage_direct_file() {
    assert_eq!(assert_direct_file(true)[0].identity, "scenario.other");
}

#[test]
fn acquisition_coverage_omitted_scenarios() {
    assert!(omitted(true).is_empty());
}
