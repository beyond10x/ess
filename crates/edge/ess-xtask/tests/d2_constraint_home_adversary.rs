//! Adversary pass over `story:d2-constraint-has-a-home`.
//!
//! Three cases, each driving the tree from a document the unit wrote about itself in this same
//! change. Nothing here rewrites, weakens or duplicates a case in `d2_constraint_home.rs`; each
//! case below asserts something that module's scans do not look at.
//!
//! 1. [`the_home_page_names_only_linkers_that_ship_the_tests_it_claims`] reads the new home page's
//!    "Where it is enforced" section as the specification it says it is, and holds the tree to it.
//! 2. [`d2_is_stated_once_under_docs_however_the_second_clause_is_worded`] asserts the story's
//!    acceptance ("D-2 is stated once") against a clause match that is not tied to one spelling of
//!    the ambiguity clause.
//! 3. [`every_markdown_page_stating_d2_in_full_links_to_its_home`] asserts the home page's own
//!    opening claim — the only full statement, every other page links here — over Markdown pages.
//!
//! The suffix tests below are written as `ends_with`, and clippy's
//! `case_sensitive_file_extension_comparisons` is allowed for the module rather than each of them
//! rewritten, because the rewrite it suggests would change what two of them mean. `_test.go` is a
//! filename convention the Go toolchain enforces, not an extension, and asking for the extension
//! `go` would match every Go file rather than the test files; and the paths being filtered are
//! literal spans quoted inside a Markdown page, where an upper-case suffix is a typo to catch
//! rather than a case to tolerate. No assertion here is relaxed by the attribute.
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use std::fs;
use std::path::{Path, PathBuf};

/// The document under attack: the home this unit created for the constraint.
const HOME: &str = "docs/design/linker-never-chooses.md";

/// Root-relative prefixes no scan here reads: build output, the append-only plan store, fetched
/// dependencies, and git's own storage.
const UNSCANNED_PREFIXES: &[&str] = &["target/", ".engineering/", "node_modules/", ".git/"];

/// The workspace root, found by walking up rather than by counting `..`.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

/// Text with Markdown emphasis removed and every run of whitespace collapsed to one space, so a
/// clause that wrapped across a line break still reads as one clause.
fn normalize(text: &str) -> String {
    let stripped: String = text
        .chars()
        .map(|character| match character {
            '*' | '`' | '_' | '#' | '>' => ' ',
            other => other,
        })
        .collect();
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// This file, root-relative, so no scan reads its own fixtures.
fn this_file() -> &'static str {
    file!()
}

/// Every file under `root` these scans read, root-relative and sorted.
fn scanned_files(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut queue = vec![root.to_path_buf()];
    while let Some(directory) = queue.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            if UNSCANNED_PREFIXES.iter().any(|prefix| {
                relative == prefix.trim_end_matches('/') || relative.starts_with(prefix)
            }) {
                continue;
            }
            if path.is_dir() {
                queue.push(path);
            } else if relative != this_file() {
                found.push(relative);
            }
        }
    }
    found.sort();
    found
}

/// Whether `text` states D-2 in full — both refusals, in either wording the tree uses.
///
/// The zero clause is spelled one way everywhere. The two clause is not: the home page writes "an
/// ambiguity error naming both" and
/// `docs/design/ess-model-driven-interpretation-design-v0.1.md:60` writes "an ambiguity naming
/// both". Both tell the reader the same rule, which is what makes a passage a restatement rather
/// than a citation, so both are matched here.
fn states_d2_in_full(text: &str) -> bool {
    let normalized = normalize(text);
    normalized.contains("unsatisfied obligation")
        && (normalized.contains("ambiguity naming both")
            || normalized.contains("ambiguity error naming both"))
}

/// The contents of `relative` under `root`, lossily decoded, or `None` if it cannot be read.
fn read(root: &Path, relative: &str) -> Option<String> {
    fs::read(root.join(relative))
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// The home page's "Where it is enforced" section, without the sections around it.
fn where_it_is_enforced(home: &str) -> &str {
    home.split("## Where it is enforced")
        .nth(1)
        .expect("the home page must carry a 'Where it is enforced' section")
        .split("\n## ")
        .next()
        .expect("a section ends at the next heading or at the end of the page")
}

/// Every repository path the "Where it is enforced" section quotes, whether or not it claims a
/// test for it — a file or a directory, in document order.
fn paths_the_home_page_names(home: &str) -> Vec<String> {
    where_it_is_enforced(home)
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| span.ends_with(".rs") || span.ends_with(".go") || span.ends_with('/'))
        .map(ToString::to_string)
        .collect()
}

/// The source paths the home page promises carry a zero-case and a many-case test.
///
/// Scoped to the paragraphs that make the promise, not to the section, because the section also
/// carries a paragraph that says the opposite — the Go realization has no test lane, stated
/// plainly. A check over the whole section would demand tests the page explicitly disclaims; a
/// check over a fixed count would pass whatever the page said. The promise is the thing to locate.
fn paths_the_home_page_says_are_tested(home: &str) -> Vec<String> {
    where_it_is_enforced(home)
        .split("\n\n")
        .filter(|paragraph| paragraph.contains("a test for the zero case"))
        .flat_map(|paragraph| {
            paragraph
                .split('`')
                .skip(1)
                .step_by(2)
                .filter(|span| span.ends_with(".rs") || span.ends_with(".go"))
                .map(ToString::to_string)
        })
        .collect()
}

/// Whether a test exercising `relative` exists — in the file itself for Rust, or in a sibling
/// `_test.go` for Go, which is the only place the Go toolchain will look.
fn has_an_executable_test(root: &Path, relative: &str) -> bool {
    if relative.ends_with(".rs") {
        return read(root, relative).is_some_and(|text| text.contains("#[test]"));
    }

    let directory = Path::new(relative)
        .parent()
        .map(|parent| parent.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default();
    scanned_files(root)
        .iter()
        .filter(|candidate| candidate.starts_with(&directory) && candidate.ends_with("_test.go"))
        .any(|candidate| {
            read(root, candidate.as_str()).is_some_and(|text| text.contains("func Test"))
        })
}

/// The home page's "Where it is enforced" section is true of the tree it names.
///
/// > The rule is executed, not asserted. Each realization ships a linker that resolves one
/// > obligation at a time and refuses both degenerate cases, **with a test for the zero case and a
/// > test for the many case**: …
///
/// That sentence is the only evidence the new home offers that D-2 is enforced rather than
/// asserted, and it is the sentence a reader who wants to check the rule follows. A page that
/// names a file as carrying two tests, when that file's package carries none, sends its reader to
/// look at nothing — which is the failure mode this whole story exists to end.
#[test]
fn the_home_page_names_only_linkers_that_ship_the_tests_it_claims() {
    let root = workspace_root();
    let home = read(&root, HOME).expect("the home page must exist");
    let named = paths_the_home_page_says_are_tested(&home);

    // Not `named.len() == 3`. A count is a fact about today's page, and the page has already been
    // rewritten once under this story: the first version named three linkers, the second names two
    // linkers and a synthesis test and moves the Go realization to a paragraph that claims no test
    // at all. Both pass a check for three. What has to hold is a property — the sentence that
    // promises tests is about files that have them — so the promise is located in the page rather
    // than assumed to be the whole section, and the count is asserted to be non-zero only, which is
    // what stops a page that promises nothing from passing vacuously.
    assert!(
        !named.is_empty(),
        "the section must promise a test for at least one named file, or this case proves nothing"
    );

    let absent: Vec<String> = paths_the_home_page_names(&home)
        .into_iter()
        .filter(|relative| !root.join(relative).exists())
        .collect();
    assert!(
        absent.is_empty(),
        "{HOME} sends its reader to paths that are not in this tree: {absent:?}"
    );

    let missing: Vec<&String> = named
        .iter()
        .filter(|relative| !has_an_executable_test(&root, relative.as_str()))
        .collect();

    assert!(
        missing.is_empty(),
        "{HOME} says each named linker ships a test for the zero case and a test for the many \
         case; these ship no executable test at all: {missing:?}"
    );
}

/// D-2 is stated once under `docs/`, whichever way the ambiguity clause is worded.
///
/// This is the story's acceptance — *D-2 is stated once, in a document that is not a plan file, and
/// every place that quotes it links there instead of restating it*. The shipped check reads "in
/// full" as the literal pair `unsatisfied obligation` and `ambiguity error`, and one spelling is
/// not the rule: a passage that says "two remains an ambiguity naming both" has told the reader
/// the whole rule and is a second copy of it, which is the state "stated once" forbids.
#[test]
fn d2_is_stated_once_under_docs_however_the_second_clause_is_worded() {
    let root = workspace_root();
    let homes: Vec<String> = scanned_files(&root)
        .into_iter()
        .filter(|relative| relative.starts_with("docs/"))
        .filter(|relative| read(&root, relative).is_some_and(|text| states_d2_in_full(&text)))
        .collect();

    assert_eq!(
        homes.len(),
        1,
        "D-2 must be stated exactly once under docs/; these state it in full: {homes:?}"
    );
    assert_eq!(homes[0], HOME, "the one statement must be the home page");
}

/// Every Markdown page that states D-2 in full links to its home.
///
/// The home page opens by claiming this of itself: *"It is the only full statement of the rule in
/// the engineering record; every other page cites it and links here."* A reader has no way to know
/// which of two full statements is authoritative except by that claim being true, and a page that
/// states the rule without linking home is the second-best copy this story was written to remove.
#[test]
fn every_markdown_page_stating_d2_in_full_links_to_its_home() {
    let root = workspace_root();
    let unlinked: Vec<String> = scanned_files(&root)
        .into_iter()
        .filter(|relative| relative.ends_with(".md") && relative != HOME)
        .filter(|relative| {
            read(&root, relative).is_some_and(|text| {
                states_d2_in_full(&text) && !text.contains("linker-never-chooses")
            })
        })
        .collect();

    assert!(
        unlinked.is_empty(),
        "{HOME} claims it is the only full statement of D-2 and that every other page links here; \
         these state it in full and link nowhere: {unlinked:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Pass two. The corrections above were made against pass one's cases; these attack the
// corrections, and they ask Git what the source files are rather than walking the filesystem.
// ---------------------------------------------------------------------------------------------

/// Every file Git reports as source here: tracked, plus untracked and not ignored.
///
/// `-z` and split on NUL, because a path is allowed to contain a newline and a check that splits
/// on one is a check with a hole in it exactly where somebody would put a file to hide it.
fn git_listed_files(root: &Path) -> Vec<String> {
    let listed = std::process::Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(root)
        .output()
        .expect("git reports this repository's source files");
    assert!(listed.status.success(), "git ls-files refused");

    let mut found: Vec<String> = listed
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).into_owned())
        .filter(|relative| !relative.starts_with(".engineering/"))
        .collect();
    found.sort();
    found
}

/// The two modules that quote D-2 as a test fixture, which no scan for copies of D-2 may read.
///
/// Both `d2_constraint_home.rs` and this file spell the rule out — that is what they assert
/// against — and a scan that reported them would be reporting itself.
fn is_a_negative_fixture(relative: &str) -> bool {
    relative.starts_with("crates/edge/ess-xtask/tests/")
}

/// Whether `text`, at `relative`, carries a Markdown link whose target is the home page.
///
/// A link, not a mention. `[text](path)` is a thing a reader clicks; `` `path` `` is a string they
/// retype. The target is resolved against the linking file's own directory and required to be the
/// home, so a link that is correct in form and wrong in depth is caught rather than counted.
fn links_to_the_home(root: &Path, relative: &str, text: &str) -> bool {
    let directory = Path::new(relative).parent().unwrap_or(Path::new(""));
    text.split("](")
        .skip(1)
        .filter_map(|rest| rest.split(')').next())
        .map(|target| target.split('#').next().unwrap_or(target))
        .filter(|target| target.ends_with("linker-never-chooses.md"))
        .any(|target| {
            let resolved = root.join(directory).join(target);
            resolved
                .canonicalize()
                .ok()
                .zip(root.join(HOME).canonicalize().ok())
                .is_some_and(|(left, right)| left == right)
        })
}

/// Under `docs/` a document that states D-2 in full links to the home; outside it, naming is enough.
///
/// Coordinator decision, 2026-09-11, recorded in `review-result:adversary-d2-constraint-pass-2`.
/// The acceptance says every place that quotes D-2 links there. `docs/` is never published, so a
/// relative link from `website/blog/` would be a broken link in the Docusaurus build — the
/// implementor established that and it is why the blog names the home as inline code instead.
/// That distinction is a decision, not a rewording, so it is asserted here rather than left to
/// prose: inside `docs/` the link is required, outside it a citation naming the home is the
/// obligation. Narrowing this later is then a change somebody makes on purpose.
#[test]
fn docs_pages_link_to_the_home_and_pages_outside_docs_at_least_name_it() {
    let root = workspace_root();
    let mut unlinked: Vec<String> = Vec::new();
    let mut unnamed: Vec<String> = Vec::new();

    for relative in git_listed_files(&root)
        .into_iter()
        .filter(|relative| relative.ends_with(".md") && relative != HOME)
        .filter(|relative| !is_a_negative_fixture(relative))
    {
        let Some(text) = read(&root, &relative) else {
            continue;
        };
        if !states_d2_in_full(&text) {
            continue;
        }
        if relative.starts_with("docs/") {
            if !links_to_the_home(&root, &relative, &text) {
                unlinked.push(relative);
            }
        } else if !text.contains(HOME) {
            unnamed.push(relative);
        }
    }

    assert!(
        unlinked.is_empty(),
        "a document under docs/ states D-2 in full and does not link to {HOME}: {unlinked:?}"
    );
    assert!(
        unnamed.is_empty(),
        "a document outside docs/ states D-2 in full and does not even name {HOME}; a link is not \
         required there because docs/ is unpublished, but the citation is: {unnamed:?}"
    );
}

/// The home page makes no completeness claim about copies of D-2 anywhere in the tree.
///
/// Coordinator decision, 2026-09-11, recorded in `review-result:adversary-d2-constraint-pass-2`.
/// Three findings across two adversary passes were one defect: the page kept asserting properties
/// of the whole repository — that every document stating the rule links here, that the code copies
/// are the ones it names, that tests it names check them — and each was falsified by a scan. The
/// decision was to delete the claims rather than narrow them, because an audit of every file in
/// the tree cannot stay true and a reader who trusts a stale one stops looking.
///
/// The page may still *name examples*; `the_home_page_names_only_linkers_that_ship_the_tests_it_claims`
/// requires it to, and that case checks the paths it names exist. What it may not do is claim the
/// examples are all of them. This case pins the removal by its exact wording: if a completeness
/// sentence returns, this goes red and somebody revisits the decision deliberately rather than
/// reintroducing a claim nobody is auditing.
#[test]
fn the_home_page_carries_no_completeness_claim_about_copies_of_d2() {
    const REMOVED_CLAIMS: &[&str] = &[
        "every document that states the rule in full links here",
        "those copies are checked by the tests named below",
        "It is the only full statement of the rule in the engineering record",
        "every other page cites it and links here",
    ];

    let root = workspace_root();
    let home = read(&root, HOME).expect("the home page must exist");
    let returned: Vec<&&str> = REMOVED_CLAIMS
        .iter()
        .filter(|claim| home.contains(**claim))
        .collect();

    assert!(
        returned.is_empty(),
        "{HOME} has regained a completeness claim about the whole tree: {returned:?}. These were \
         removed on 2026-09-11 because each was falsified by a scan. Reinstating one means \
         auditing it, and changing this case on purpose."
    );
}

/// The Go realization names a test that does not exist, and that is pinned until somebody fixes it.
///
/// `examples/gatepass-go-realization/linker.go:14` tells its reader that
/// `TestTheLinkersObligationListIsExactlyThePlans` holds the obligation list equal to
/// `generated/go/gatepass/plan.json`. No such identifier is defined anywhere in this repository,
/// and `git show 1a2effd6` has the comment byte-identical at the wave base — so this is a
/// pre-existing defect, filed as its own story rather than fixed under a documentation unit.
/// Asserting today's state keeps it visible: the case goes red the moment the test is written or
/// the comment is removed, and that is exactly when the story should close.
#[test]
fn the_go_realizations_named_test_is_still_missing_and_is_filed_as_its_own_story() {
    const NAMED: &str = "TestTheLinkersObligationListIsExactlyThePlans";

    let root = workspace_root();
    let listed = git_listed_files(&root);

    let comment_still_names_it = read(&root, "examples/gatepass-go-realization/linker.go")
        .is_some_and(|text| text.contains(NAMED));
    let definition = format!("func {NAMED}(");
    let defined_somewhere = listed.iter().any(|candidate| {
        candidate.ends_with(".go")
            && read(&root, candidate).is_some_and(|text| text.contains(&definition))
    });

    assert!(
        comment_still_names_it && !defined_somewhere,
        "state changed: linker.go names the test = {comment_still_names_it}, it is defined = \
         {defined_somewhere}. Both halves were true when this was pinned on 2026-09-11. Close the \
         story for the missing Go test lane and delete this case."
    );
}
