//! Adversarial cases against `story:planning-store-carries-workstation-paths`.
//!
//! The unit took the story's *out of scope* branch and shipped one case,
//! `every_unread_tree_that_carries_the_class_is_named_in_the_module_documentation`, to hold the
//! decision. That case is now the only thing standing between the lane's documentation and the
//! repository's state, which is the agreement the story's acceptance statement requires **either
//! way**. So the cases here drive the *paragraph the unit wrote* against the repository it
//! describes, rather than against the behaviour the unit built.
//!
//! Three things the shipped case does not look at, each measured below rather than asserted:
//!
//! * the module documentation's **summary line** — the first paragraph, which is the whole of what
//!   rustdoc renders beside the item in an index, divorced from every qualification seventy lines
//!   below it;
//! * the **counted claims inside the new bullet**. The parse the unit added takes the first token a
//!   bullet quotes in backticks and nothing else, so every number, comparison and citation in that
//!   bullet is prose the lane does not read;
//! * whether the parse can tell a bullet that says a tree is **unread** from one that says it is
//!   **read**. It cannot, and the bullet is where the decision lives.
//!
//! Like the four adversarial targets beside it, everything here drives the transcribed copy in
//! `host_paths_lane/mod.rs` and calls `assert_current()` first, so a lane edit turns these into a
//! named complaint rather than a measurement of a fossil. Nothing here spells a home-directory
//! marker as a literal; every marker used is resolved out of the lane's own `HOME_MARKERS`.

mod host_paths_lane;

use host_paths_lane::{
    assert_current, documented_unread_trees, home_paths_in, lane_source, scanned_files,
    workspace_root,
};
use host_paths_lane::{LANE, SCANNED_PREFIXES};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

/// The anchor line the lane's own `UNREAD_TREE_SECTION` holds, copied.
///
/// Copied rather than imported because the lane declares it as a private `const` in a test target
/// and `host_paths_lane/mod.rs` does not transcribe it — which is the first thing worth saying
/// about it: the drift guard that protects every other copy in that module does not cover the
/// three helpers this story added.
const UNREAD_TREE_SECTION: &str = "What it does not read at all, tree by tree:";

/// Every tracked file this repository holds, root-relative, whatever tree it is under.
///
/// The lane's new `tracked_files`, copied. It is not in `host_paths_lane::TRANSCRIBED`, so unlike
/// `scanned_files` beside it this copy is not held to the lane's text by `assert_current`.
fn tracked_files(root: &Path) -> Vec<String> {
    let listed = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--cached", "-z"])
        .output()
        .expect("git lists the repository's files");
    assert!(listed.status.success(), "`git ls-files` failed");
    let mut files: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .split('\0')
        .filter(|file| !file.is_empty())
        .map(str::to_owned)
        .collect();
    files.sort();
    files.dedup();
    files
}

/// The trees `source` claims are unread, through the lane's own transcribed parse.
///
/// No longer a hand copy. The lane's `documented_unread_trees` is in `TRANSCRIBED` now, so
/// `assert_current()` holds this to the lane's text the way it holds every other copy — which is
/// what the finding against the unguarded copy asked for.
fn documented_unread_trees_in(source: &str) -> BTreeSet<String> {
    documented_unread_trees(source)
        .into_iter()
        .filter(|(_, unread, _, _)| *unread)
        .map(|(tree, _, _, _)| tree)
        .collect()
}

/// The first paragraph of the lane's module documentation, as rustdoc renders it on its own.
///
/// Rustdoc's short description is everything up to the first blank documentation line, and it is
/// what appears beside the item in a module index, in search results and in a re-export listing —
/// on its own, with nothing after it. A qualification seventy lines down is not part of it.
fn module_summary(source: &str) -> String {
    source
        .lines()
        .map_while(|line| line.strip_prefix("//!"))
        .take_while(|content| !content.trim().is_empty())
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join(" ")
}

/// The bullet under [`UNREAD_TREE_SECTION`] whose first backtick token is `tree`, as one string.
fn unread_bullet(source: &str, tree: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let anchor = lines
        .iter()
        .position(|line| line.strip_prefix("//!").map(str::trim) == Some(UNREAD_TREE_SECTION))
        .expect("the lane's module documentation carries the unread-tree anchor");
    let mut collected: Vec<String> = Vec::new();
    let mut inside = false;
    for line in &lines[anchor + 1..] {
        let Some(content) = line.strip_prefix("//!") else {
            break;
        };
        let trimmed = content.trim();
        if trimmed.is_empty() {
            if inside {
                break;
            }
            continue;
        }
        if let Some(bullet) = trimmed.strip_prefix("* ") {
            if inside {
                break;
            }
            inside = bullet.split('`').nth(1) == Some(tree);
            if inside {
                collected.push(trimmed.to_owned());
            }
            continue;
        }
        if inside {
            collected.push(trimmed.to_owned());
            continue;
        }
        if content.starts_with("   ") {
            continue;
        }
        break;
    }
    collected.join(" ")
}

/// Every tracked file that carries the class, as `path -> (lines carrying, findings)`.
///
/// Both units, because the bullet under attack uses one of them and its comparison rests on the
/// other. The lane's own `home_paths_in` does the reading, so this is the lane's detector and not
/// a second one.
fn carried_by_file(root: &Path, files: &[String]) -> BTreeMap<String, (usize, usize)> {
    let mut carried = BTreeMap::new();
    for file in files {
        let full = root.join(file);
        if !full.exists() {
            continue;
        }
        let found = home_paths_in(&full);
        if found.is_empty() {
            continue;
        }
        let lines: BTreeSet<usize> = found.iter().map(|(line, _)| *line).collect();
        carried.insert(file.clone(), (lines.len(), found.len()));
    }
    carried
}

/// The sentence rustdoc shows on its own is a claim about this repository, and it is false.
///
/// The story is accepted on *"either way the lane's documentation and the repository's state agree
/// afterwards"*. The unit's answer to that was a paragraph under a new anchor plus a closing
/// sentence declaring that *"what the opening sentence of this file claims, therefore, is a claim
/// about `SCANNED_PREFIXES` and about nothing else"* — a redefinition of the opening sentence
/// rather than a correction of it.
///
/// A redefinition seventy lines below a summary line does not travel with that summary line.
/// Rustdoc's short description is the first paragraph and nothing else; it is what a reader of the
/// module index, of a search result or of the crate's own documentation sees, and it there reads
/// as an unqualified claim about every tracked file. This case asks for exactly one thing: if the
/// summary names none of the trees the lane actually reads, it has to be true of the repository.
/// Naming one — *"under the trees it reads"*, `crates/`, anything — turns this green, and that is
/// the fix.
#[test]
fn the_module_summary_line_is_true_of_the_repository_or_names_the_trees_it_is_true_of() {
    assert_current();
    let root = workspace_root();
    let summary = module_summary(lane_source());
    assert!(
        !summary.is_empty(),
        "`{LANE}` has no module documentation summary line at all"
    );

    let scoped = SCANNED_PREFIXES
        .iter()
        .any(|prefix| summary.contains(prefix.as_str()));
    if scoped {
        return;
    }

    let carried = carried_by_file(&root, &tracked_files(&root));
    let findings: usize = carried.values().map(|(_, found)| found).sum();
    let worst: Vec<String> = carried
        .iter()
        .map(|(file, (lines, found))| format!("{file}: {lines} lines, {found} findings"))
        .take(5)
        .collect();
    assert!(
        carried.is_empty(),
        "the summary line of `{LANE}` is the whole of what rustdoc renders beside this module, and \
         it names no tree it is scoped to, so it reads as a claim about every tracked file — it \
         says\n\n    {summary}\n\nand {} tracked files carry {findings} findings of the very class \
         it names. The qualification the unit added lives seventy lines further down and does not \
         travel with the summary; the story is accepted on the lane's documentation and the \
         repository's state agreeing. Naming a scope in this sentence is the fix, not restating it \
         later. First five:\n{}",
        carried.len(),
        worst.join("\n")
    );
}

/// The bullet's load-bearing comparison, measured in the bullet's own unit.
///
/// The decision rests on one sentence: a scan widened to `.engineering/` with the journal carved
/// out *"would report the store covered while exempting the file holding the largest single share
/// of the defect"*. That is the argument against the remedy the story offers, and the parse the
/// unit added does not read it — it takes the bullet's first backtick token and stops.
///
/// The bullet states its own unit two clauses earlier: the journal *"carries 87 of those lines"*.
/// Measured in that unit with the lane's own detector, the journal is not the largest share and is
/// not close to it. This case asserts the bullet's comparison, in the bullet's unit, and names the
/// file that actually holds the largest share.
#[test]
fn the_journal_holds_the_largest_share_of_the_defect_the_bullet_says_it_holds() {
    assert_current();
    let root = workspace_root();
    let tree = ".engineering/";
    let journal = ".engineering/planning/journal.jsonl";
    let bullet = unread_bullet(lane_source(), tree);
    // The bullet is allowed to stop making this claim — that was the correction. What is not
    // allowed is making it while the measurement says otherwise, so the premise is a condition
    // and not an assertion. A bullet that drops the superlative leaves nothing here to refuse.
    if !bullet.contains("largest single share") {
        return;
    }

    let under_tree: Vec<String> = tracked_files(&root)
        .into_iter()
        .filter(|file| file.starts_with(tree))
        .collect();
    let carried = carried_by_file(&root, &under_tree);
    assert!(
        carried.contains_key(journal),
        "`{journal}` does not carry the class at all, so the bullet's premise is gone with it"
    );
    let total_lines: usize = carried.values().map(|(lines, _)| lines).sum();
    let (journal_lines, journal_findings) = carried[journal];
    let mut ranked: Vec<(&String, &(usize, usize))> = carried.iter().collect();
    ranked.sort_by_key(|(file, (lines, _))| (std::cmp::Reverse(*lines), (*file).clone()));
    let (largest, (largest_lines, largest_findings)) = ranked[0];

    assert_eq!(
        largest,
        journal,
        "the `{tree}` bullet in `{LANE}` argues the store must stay unscanned because carving the \
         journal out of a widened scan would exempt \"the file holding the largest single share of \
         the defect\". In the unit the same bullet counts in — lines — the journal holds \
         {journal_lines} of {total_lines} carrying lines across {} files, and `{largest}` holds \
         {largest_lines}. A carve-out would therefore cover {} of the carrying lines and \
         {} of the carrying files, not exempt the largest share. \
         (By a second unit, findings per line, the journal is first with {journal_findings} \
         against {largest_findings} — the bullet does not say which unit it means, and the two \
         answer the decision differently, which is the defect: the sentence the decision rests on \
         is not measured by anything.)",
        carried.len(),
        total_lines - journal_lines,
        carried.len() - 1
    );
}

/// The parse cannot tell a bullet that says *unread* from one that says *read*.
///
/// The module documentation claims of its own list that it is *"not prose to be taken on trust"*,
/// and that the case below it holds the list to the repository *"both ways"*. What the parse
/// actually reads is the first token a bullet quotes in backticks. Everything that makes the
/// bullet a decision — the counts, the comparison, the citation of `AGENTS.md`, the appeal to
/// `layout.rs`, and whether the sentence says the tree is read or unread — is prose the lane
/// passes over.
///
/// Shown rather than argued: the parse is handed the real lane source and a synthetic module
/// documentation whose bullet asserts the exact opposite of the decision, and is asked to tell
/// them apart. What reaches this is the ordinary edit the case exists to catch — somebody changing
/// the decision, rewriting the sentence and leaving the tree name where it was.
#[test]
fn the_documentation_parse_distinguishes_a_tree_called_unread_from_one_called_read() {
    assert_current();
    let real = documented_unread_trees_in(lane_source());
    assert_eq!(
        real,
        BTreeSet::from([".engineering/".to_owned()]),
        "this copy of the lane's `documented_unread_trees` no longer reproduces the lane's own \
         parse, so the comparison below measures the copy and not `{LANE}`"
    );

    let inverted = concat!(
        "//! No file this repository tracks names a path inside somebody's home directory.\n",
        "//!\n",
        "//! What it does not read at all, tree by tree:\n",
        "//!\n",
        "//! * `.engineering/` — read, 0 files, 0 lines: this lane reads it in full on every run\n",
        "//!   and it is clean; it is named here only so a reader who wondered whether the store\n",
        "//!   was covered can see that it is. There is no tree this scan does not read.\n",
        "//!\n",
        "//! That is the whole of that list.\n",
        "//!\n",
        "//! A scan has two ways of being worthless.\n",
    );
    let parsed = documented_unread_trees_in(inverted);

    assert_ne!(
        parsed, real,
        "the parse the unit added reads the first token a bullet quotes in backticks and nothing \
         else, so a bullet asserting `.engineering/` is read in full and clean parses to exactly \
         the same set as the bullet asserting it is unread and carries sixty files of the defect. \
         `{LANE}` says of this list that it is \"not prose to be taken on trust\" and that the \
         case below it holds the list \"both ways\"; both directions are directions of the tree \
         *name*. The decision itself — the counts, the comparison against `.engineering/planning/\
         journal.jsonl`, the citation of `AGENTS.md`, the appeal to `layout.rs` — is unread, and \
         an editor who changes the decision and leaves the name keeps this lane green"
    );
}

/// A bullet after a continuation the parse does not recognise is dropped without a word.
///
/// The section ends at "the first unindented line that is not a bullet", and *indented* is spelled
/// as exactly three spaces of `content`. The lane's one bullet happens to wrap that way. Nothing
/// holds it there: `rustfmt` does not reflow documentation comments and `clippy` has no lint for
/// the indentation of a continuation line, so a tab, or two spaces, is an ordinary thing for the
/// next editor to write — and it ends the section early, taking every bullet after it.
///
/// The direction that matters is the quiet one. A dropped bullet naming a tree that *does* carry
/// the class turns the lane red on the undocumented half, loudly. A dropped bullet naming a tree
/// that has gone clean is a stale sentence the `stale` half exists to refuse, and dropping it is
/// exactly how it stops being refused: the documentation keeps saying a tree is unread and
/// carrying, the repository disagrees, and the lane is green.
///
/// What reaches it: the remedy the lane's own case prescribes. A tree that starts carrying the
/// class is answered by adding a second bullet, and a second bullet is all this needs.
#[test]
fn a_second_documented_tree_survives_a_continuation_line_the_parse_does_not_recognise() {
    assert_current();
    let spaced = concat!(
        "//! s\n//!\n//! What it does not read at all, tree by tree:\n//!\n",
        "//! * `.engineering/` — unread, 1 files, 1 lines: the reason continues on\n",
        "//!   the next line, wrapped with three spaces.\n",
        "//! * `fuzz/` — unread, 1 files, 1 lines: named as unread and carrying.\n",
        "//!\n//! That is the whole of that list.\n",
    );
    let tabbed = concat!(
        "//! s\n//!\n//! What it does not read at all, tree by tree:\n//!\n",
        "//! * `.engineering/` — unread, 1 files, 1 lines: the reason continues on\n",
        "//!\tthe next line, wrapped with a tab.\n",
        "//! * `fuzz/` — unread, 1 files, 1 lines: named as unread and carrying.\n",
        "//!\n//! That is the whole of that list.\n",
    );
    let both = BTreeSet::from([".engineering/".to_owned(), "fuzz/".to_owned()]);
    assert_eq!(
        documented_unread_trees_in(spaced),
        both,
        "the three-space wrapping is not the one under attack here; if this fails the copy is wrong"
    );
    assert_eq!(
        documented_unread_trees_in(tabbed),
        both,
        "one continuation line wrapped with a tab instead of three spaces ends the section, and \
         every bullet after it is dropped with no complaint. `{LANE}` calls the half below it the \
         guard against \"documentation outliving its reason\" — and a stale bullet naming a tree \
         that has gone clean is precisely what this drop hides, because a tree the parse never \
         saw cannot appear in the `stale` difference. The undocumented half fails loudly in the \
         other direction, so the failure mode this leaves is the silent one: documentation that \
         says a tree is unread and carrying, a repository that disagrees, and a green lane"
    );
}

/// `tracked_files` is what it says: `git ls-files --cached`, unfiltered, and it partitions.
///
/// The lane added this beside `scanned_files` rather than widening it, and a selection that
/// quietly drops a file is how a scan reports nothing about a tree it never opened. Extensionless
/// paths, deeply nested ones and dotted trees are asked for by name, because those are the three
/// shapes a filter written against `SCANNED_PREFIXES` would have been most likely to lose.
#[test]
fn tracked_files_is_git_ls_files_unfiltered_and_partitions_against_the_scanned_selection() {
    assert_current();
    let root = workspace_root();
    let tracked = tracked_files(&root);
    let scanned = scanned_files(&root);

    let listed = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "--cached", "-z"])
        .output()
        .expect("git lists the repository's files");
    let mut direct: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .split('\0')
        .filter(|file| !file.is_empty())
        .map(str::to_owned)
        .collect();
    direct.sort();
    direct.dedup();
    assert_eq!(
        tracked, direct,
        "`tracked_files` is not `git ls-files --cached`"
    );

    let tracked_set: BTreeSet<&String> = tracked.iter().collect();
    let scanned_set: BTreeSet<&String> = scanned.iter().collect();
    assert!(
        scanned_set.is_subset(&tracked_set),
        "the scanned selection holds a path the tracked selection does not"
    );
    assert_eq!(
        tracked.len(),
        scanned.len() + tracked.iter().filter(|f| !scanned.contains(*f)).count(),
        "the two selections do not partition the repository"
    );

    for shape in [
        "an extensionless path",
        "a path four levels deep",
        "a dotted tree",
    ] {
        let found = tracked.iter().any(|file| match shape {
            "an extensionless path" => !file.rsplit('/').next().unwrap_or("").contains('.'),
            "a path four levels deep" => file.matches('/').count() >= 4,
            _ => file.starts_with('.'),
        });
        assert!(
            found,
            "the repository holds no example of {shape}, so this case measures nothing about it"
        );
        let unreadable = tracked
            .iter()
            .filter(|file| match shape {
                "an extensionless path" => !file.rsplit('/').next().unwrap_or("").contains('.'),
                "a path four levels deep" => file.matches('/').count() >= 4,
                _ => file.starts_with('.'),
            })
            .filter(|file| !root.join(file).exists())
            .count();
        assert_eq!(
            unreadable, 0,
            "`tracked_files` lists paths of shape `{shape}` the working tree cannot open, and the \
             lane's new case skips those silently rather than accounting for them as \
             `no_tracked_file_under_the_published_trees_names_a_home_directory_path` does"
        );
    }
}
