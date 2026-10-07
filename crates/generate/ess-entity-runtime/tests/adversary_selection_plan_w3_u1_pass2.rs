//! Adversary pass 2 against `story:entity-runtime-lowering-reads-selection-plan` (wave 3, U1).
//!
//! The correction for pass 1's F3 adds `tests/fixtures/held-state-after-disjoint-accepting.yaml`
//! and pins it in this crate's lowered-definitions table. The file is a repository model, and
//! `crates/specify/ess-compiler/tests/selection_precedence_table.rs` walks every ESS source file of
//! the tree (`models`: each `.yaml` / `.yml` carrying a `format: ess/` line, labelled by its path
//! from the repository root) and fails on any it does not pin ("the tree holds models the table
//! does not pin; in the change that adds them, re-pin"). The unit's gate is package-scoped, so that
//! test is not run before CI.
//!
//! The case below reads that table and applies its rule to the ESS fixtures of this crate.
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
}

/// Every ESS source file under this crate's `tests/fixtures/` is pinned by the compiler's
/// selection-precedence table, as that table's own test requires of every model the tree holds.
/// Red where a fixture this unit adds is not re-pinned there.
#[test]
fn adv_w3u1_p2_every_ess_fixture_of_this_crate_is_pinned_by_the_compiler_selection_table() {
    let root = root();
    let table = std::fs::read_to_string(
        root.join("crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv"),
    )
    .expect("the compiler's selection-precedence table is readable");
    let pinned: Vec<&str> = table
        .lines()
        .skip(1)
        .filter_map(|line| line.split('\t').next())
        .collect();

    let mut files = Vec::new();
    walk(
        &root.join("crates/generate/ess-entity-runtime/tests/fixtures"),
        &mut files,
    );
    files.sort();
    let mut read = 0usize;
    let mut unpinned = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        read += 1;
        let label = path.strip_prefix(&root).unwrap().display().to_string();
        if !pinned.contains(&label.as_str()) {
            unpinned.push(label);
        }
    }
    assert!(
        read >= 1,
        "the walk found this crate's ESS fixtures: {read}"
    );
    assert_eq!(
        unpinned,
        Vec::<String>::new(),
        "ESS fixtures of this crate the compiler's selection-precedence table does not pin; its \
         test `every_repository_command_keeps_its_plan_and_every_model_its_ir_bytes` \
         (ess-compiler) fails on each"
    );
}
