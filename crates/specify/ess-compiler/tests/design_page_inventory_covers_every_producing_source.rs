//! The design page's inventory against *every* `ess-domain` source, not only the top level.
//!
//! `typed_diagnostics.rs::the_inventory_on_the_design_page_is_the_count_in_the_tree` is the check
//! that keeps `docs/design/review-typed-diagnostics.md`'s inventory block honest, and its own
//! comment states the contract: "a file with sites that the block omits, a count that has drifted,
//! and a file the block lists that has no sites, are each a failure. Migrating another family
//! cannot go green without moving the page."
//!
//! It walks `crates/specify/ess-domain/src` with a single `std::fs::read_dir`, and skips every
//! entry whose extension is not `rs` — which is every **directory**. `ess-domain/src` has two, and
//! both contain refusal producers, so the sentence "in both directions" holds only for the top
//! level of one directory. A family migrated into a subdirectory, or moved into one, goes green
//! without moving the page: exactly the hand-maintained census the check was written to replace.
//!
//! This case walks the same tree recursively and asks the same question. It is deliberately weaker
//! than the shipped check — it asks only that a producing file be *named* in the block, not that
//! its counts match — so that what it reports is the hole and not a count nobody has written yet.

use std::path::{Path, PathBuf};

/// The repository root, from this crate's manifest directory.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The file names the design page's machine-readable inventory block lists.
fn inventoried() -> Vec<String> {
    let page =
        std::fs::read_to_string(repository_root().join("docs/design/review-typed-diagnostics.md"))
            .expect("the design page exists");
    let block = page
        .split_once("<!-- inventory:begin -->")
        .expect("the page carries a machine-readable inventory")
        .1
        .split_once("<!-- inventory:end -->")
        .expect("the inventory block is closed")
        .0;
    block
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("```"))
        .map(|line| {
            line.split_whitespace()
                .next()
                .expect("a file name")
                .to_owned()
        })
        .collect()
}

/// Every `.rs` file under `root`, at any depth, relative to `root`.
fn sources(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("ess-domain sources are readable") {
            let path = entry.expect("a readable entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|kind| kind == "rs") {
                found.push(
                    path.strip_prefix(root)
                        .expect("under the root")
                        .to_path_buf(),
                );
            }
        }
    }
    found.sort();
    found
}

/// Every `ess-domain` source that produces a refusal is in the inventory the page carries.
///
/// Red here means the inventory's completeness claim is bounded by a directory walk that does not
/// recurse, and the count on the page is not the count in the tree. It is red now, for
/// `binding/periodic.rs` and `command/subject_state.rs`, and it is red for the same two at
/// `e5a97603` — the walk it measures is untouched by
/// `story:a-masked-first-declaration-hides-a-duplicate-name`, whose diff adds one `Cited` row to
/// `typed_diagnostics.rs` and does not go near `the_inventory_on_the_design_page_is_the_count_in_
/// the_tree`. So it is `#[ignore]`d at the attribute rather than deleted, relaxed, or answered by
/// hand-listing the two files on the page: the defect is the non-recursive walk, and the two
/// missing entries are its symptom. The `#[ignore]` comes off when the walk recurses, and nothing
/// else has to change here for it to.
#[test]
#[ignore = "pre-existing and not this story's to fix: `typed_diagnostics.rs`'s \
            `the_inventory_on_the_design_page_is_the_count_in_the_tree` walks `ess-domain/src` \
            with one non-recursive `read_dir`, so every source in a subdirectory is invisible to \
            the check that claims the page's inventory is the count in the tree in both \
            directions. Red at `e5a97603` for the same two files; filed by the coordinator out of \
            the wave-25 unit-2 pass-2 review as G5"]
fn the_inventory_names_every_producing_source_at_every_depth() {
    let root = repository_root().join("crates/specify/ess-domain/src");
    let listed = inventoried();

    let mut missing: Vec<String> = Vec::new();
    for relative in sources(&root) {
        let text = std::fs::read_to_string(root.join(&relative)).expect("readable");
        let untyped = text.matches("ValidationError::new").count();
        let typed = text.matches("ValidationError::at").count();
        if untyped == 0 && typed == 0 {
            continue;
        }
        let name = relative
            .file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned();
        if !listed.contains(&name) {
            missing.push(format!(
                "{} ({untyped} `ValidationError::new`, {typed} `ValidationError::at`)",
                relative.display()
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "`docs/design/review-typed-diagnostics.md`'s inventory claims to be the count in the tree \
         in both directions, and \
         `typed_diagnostics.rs::the_inventory_on_the_design_page_is_the_count_in_the_tree` cannot \
         see a source in a subdirectory: its `read_dir` does not recurse and its extension filter \
         drops every directory. Producing sources the block omits: {missing:?}"
    );
}
