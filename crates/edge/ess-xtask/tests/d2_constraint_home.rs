//! Gap register D-2 has one home, and nothing points at the register that is gone.
//!
//! This is the check `story:d2-constraint-has-a-home` is accepted against. The story's acceptance
//! is two mechanical clauses and this module is one case per clause:
//!
//! > D-2 is stated once, in a document that is not a plan file, and every place that quotes it
//! > links there instead of restating it; no reference to `docs/plan/gap-register.md` remains that
//! > does not say the file is gone.
//!
//! `docs/plan/gap-register.md` was deleted and nothing noticed, because a Markdown link to a
//! missing sibling renders as a link. D-2 — the linker never chooses — survived only as a sentence
//! quoted inside a wave plan, and a constraint whose home is missing is one the next reader
//! re-litigates. Restoring the sentence by hand fixes today's copy and leaves tomorrow's; the
//! defect is that nothing asserts the home exists, so the assertion is the fix.
//!
//! A scan has two ways of being worthless — it can look at the wrong files, and it can look for
//! the wrong shape — and both were live here, so each has a case of its own below rather than
//! being assumed. The register reference that survived longest was in `.gitignore`, which is not
//! Markdown and not under `docs/`; and the wave plan's copy of D-2 wrapped "an ambiguity / error"
//! across a line break, so a scan matching the clause literally read straight past the one file
//! the story was written about.
//!
//! # What this deliberately does not reach
//!
//! [`restatements`] reads `docs/` only — the engineering record — and that bound is the one thing
//! here a reader should not take on trust, so it is written down rather than implied.
//!
//! D-2 is also restated in full outside `docs/` — in linkers, which is deliberate and correct,
//! because a linker stating the rule it implements is the rule in the place it binds rather than a
//! second-best copy of a document. That is why the bound here is `docs/` and why the home page
//! claims to be the only full statement *under `docs/`* rather than in the repository.
//!
//! No list of those copies is kept here. An earlier version of this comment named three files and
//! called them "the three linkers that enforce it", which is a claim about the whole tree made in
//! a place that checks one directory: it was falsifiable, it was not checked by anything, and
//! keeping it correct would have been somebody's unpaid job forever. What is enforced is enforced
//! by a case below; what is not is not asserted.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, str};

/// Root-relative prefixes the scans drop from what Git reports.
///
/// Only one entry is needed, and it is not build output: `.engineering/` is the plan store, whose
/// journal is append-only, so a register path named in a journal line cannot be corrected by a
/// later edit even in principle.
///
/// Build output needs no entry because [`scanned_files`] asks Git rather than walking the
/// filesystem, and Git already knows what is ignored. An earlier version of this module walked the
/// tree with a hand-written prefix list and claimed to mirror `tests/layout.rs`. It did not:
/// `layout.rs:38` also excludes `docs/design/` and `docs/reviews/` as the dated record, which these
/// scans must read, and `layout.rs:102` asks `git ls-files` rather than walking. Walking was the
/// worse half of that mistake — the prefix list matched root-relative paths only, so
/// `generated/rust/*/target/`, `examples/billing-web/target/` and `website/node_modules` were read
/// whenever they happened to exist, and the verdict depended on whether somebody had built
/// recently. Asking Git fixes every member of that class at once instead of adding a prefix per
/// discovery.
const UNSCANNED_PREFIXES: &[&str] = &[".engineering/"];

/// The file that is gone, as every reference to it spells the name.
const DELETED_REGISTER: &str = "gap-register.md";

/// Every wording the tree uses for D-2's zero case.
///
/// This is a list for the same reason [`TWO_CLAUSE_SPELLINGS`] is, and it was left a single
/// literal when that one became a list — the one-spelling defect closed on one half of a two-half
/// rule, which is not closing it. A passage saying "two is an ambiguity naming both" and "the
/// obligation is unsatisfied" has told the reader D-2, and a check keyed to the noun phrase
/// `unsatisfied obligation` would read it as a citation and let a second copy stand.
const ZERO_CLAUSE_SPELLINGS: &[&str] = &[
    "unsatisfied obligation",
    "unfilled obligation",
    "obligation is unsatisfied",
    "obligation remains unsatisfied",
    "obligation nobody filled",
];

/// Every wording the tree uses for D-2's two case.
///
/// One spelling is not the definition of "in full", and treating it as one is how a second copy of
/// the rule survived this story's first pass: `docs/design/…-interpretation-design-v0.1.md` wrote
/// "two remains an ambiguity naming both", the shipped check required the literal `ambiguity
/// error`, and the check reported the page clean while the page restated the rule. A passage that
/// tells the reader both refusals has told them the rule, whichever words it picks.
const TWO_CLAUSE_SPELLINGS: &[&str] = &[
    "ambiguity error",
    "ambiguity naming both",
    "ambiguity naming every claimant",
];

/// Wording that admits the register is gone, any one of which clears a reference to it.
const SAYS_IT_IS_GONE: &[&str] = &[
    "does not exist",
    "no longer exists",
    "is gone",
    "was deleted",
    "has been deleted",
];

/// This file, root-relative, taken from the compiler rather than written down.
///
/// A scan must not read its own negative fixtures. The cases below name the deleted register and
/// quote D-2's clauses — that is what they assert — and a scan that reported them would be
/// reporting itself.
fn this_file() -> &'static str {
    file!()
}

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

/// Text with its Markdown emphasis removed and every run of whitespace collapsed to one space.
///
/// The reason this exists rather than a literal match: the wave plan wrapped D-2 mid-clause, so
/// "an ambiguity\nerror naming both" contains no line holding `ambiguity error`. A per-line scan
/// for the clause read past the single most important file in the story. Emphasis is stripped for
/// the same reason — a clause is free to arrive as `an **ambiguity error**`.
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

/// Every file the scans read, root-relative and sorted, as Git reports it.
///
/// `--cached --others --exclude-standard` is the repository's own answer to "what is a source file
/// here": everything tracked, plus everything untracked that is not ignored. It is the same
/// question `tests/layout.rs:102` asks, and asking it has two consequences this module depends on.
///
/// Ignored build trees are gone without naming any of them, so the verdict does not change
/// depending on whether somebody has run `cargo build` or `npm install`. And a file about to be
/// committed is read before it lands, which is the only moment a check like this can still be
/// cheap to act on.
///
/// Files are read as UTF-8 lossily rather than filtered by extension: a scan that trusts suffixes
/// is a scan that misses `.gitignore`, which is exactly what happened here.
fn scanned_files(root: &Path) -> Vec<String> {
    let listed = Command::new("git")
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
    assert!(
        listed.status.success(),
        "git ls-files refused: {}",
        String::from_utf8_lossy(&listed.stderr)
    );

    let mut found: Vec<String> = listed
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).into_owned())
        .filter(|relative| {
            !UNSCANNED_PREFIXES
                .iter()
                .any(|prefix| relative.starts_with(prefix))
        })
        .filter(|relative| relative != this_file())
        .collect();
    found.sort();
    found
}

/// The contents of `relative` under `root`, lossily decoded, or `None` if it cannot be read.
fn read(root: &Path, relative: &str) -> Option<String> {
    fs::read(root.join(relative))
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Whether `text` states D-2 in full — both refusals, in any wording the tree uses.
///
/// "In full" means the passage has told the reader the rule, so a citation that names D-2 and
/// links onwards is free to exist anywhere. Both halves must be present: a page discussing only
/// the zero case has not restated the rule.
fn states_d2_in_full(text: &str) -> bool {
    let normalized = normalize(text);
    ZERO_CLAUSE_SPELLINGS
        .iter()
        .any(|spelling| normalized.contains(spelling))
        && TWO_CLAUSE_SPELLINGS
            .iter()
            .any(|spelling| normalized.contains(spelling))
}

/// Every file under `docs/` whose text states D-2 in full, root-relative and sorted.
fn restatements(root: &Path) -> Vec<String> {
    scanned_files(root)
        .into_iter()
        .filter(|relative| relative.starts_with("docs/"))
        .filter(|relative| read(root, relative).is_some_and(|text| states_d2_in_full(&text)))
        .collect()
}

/// D-2 is stated once in the engineering record, and not in a plan file.
///
/// The "once" half is what stops the next reader finding two copies and having to decide which is
/// authoritative — the situation this repository refuses everywhere else. The "not a plan file"
/// half is why the story exists: a wave plan describes a wave and is finished when the wave is, so
/// a constraint that outlives the wave cannot live in one.
#[test]
fn d2_is_stated_once_in_the_record_and_not_in_a_plan_file() {
    let root = workspace_root();
    let homes = restatements(&root);

    assert_eq!(
        homes.len(),
        1,
        "D-2 must be stated exactly once under docs/, and these files state it in full: {homes:?}"
    );

    let home = &homes[0];
    assert!(
        !home.starts_with("docs/plan/"),
        "D-2's home must not be a plan file, and {home} is one"
    );
}

/// No reference to the deleted register survives without admitting it is gone.
///
/// Scoped to the whole tree rather than to `docs/`, because the reference that survived the
/// longest was a comment in `.gitignore`. A reader who follows a reference to a file that is not
/// there learns nothing about whether the decision was reversed, moved or never written down, and
/// that is the state this case exists to keep the tree out of.
#[test]
fn no_reference_to_the_deleted_register_omits_that_it_is_gone() {
    let root = workspace_root();
    let mut silent = Vec::new();

    for relative in scanned_files(&root) {
        let Some(text) = read(&root, &relative) else {
            continue;
        };
        if !text.contains(DELETED_REGISTER) {
            continue;
        }
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if !line.contains(DELETED_REGISTER) {
                continue;
            }
            let low = index.saturating_sub(4);
            let high = (index + 5).min(lines.len());
            let window = normalize(&lines[low..high].join(" ")).to_lowercase();
            if !SAYS_IT_IS_GONE.iter().any(|phrase| window.contains(phrase)) {
                silent.push(format!("{relative}:{}: {}", index + 1, line.trim()));
            }
        }
    }

    assert!(
        silent.is_empty(),
        "every reference to the deleted {DELETED_REGISTER} must say it is gone; these do not:\n{}",
        silent.join("\n")
    );
}

/// The clause scan survives a line break inside a clause.
///
/// Not a hypothetical. `docs/plan/ess-wave-6-structural-synthesis.md` wrapped D-2 as "two is an
/// ambiguity / error naming both", so the first version of this scan — a `grep` for the clause,
/// one line at a time — reported that file clean. Had it shipped, the story's own subject would
/// have been the one file the check could not see.
#[test]
fn the_clause_scan_survives_a_line_break_inside_a_clause() {
    let wrapped = "Zero implementations for an obligation is an unsatisfied\n\
                   obligation; two is an **ambiguity\n\
                   error** naming both.";

    assert!(
        wrapped
            .lines()
            .all(|line| !line.contains("ambiguity error")),
        "the fixture must be wrapped mid-clause, or it is testing nothing"
    );

    assert!(
        states_d2_in_full(wrapped),
        "a wrapped, emphasised restatement must still read as one: {}",
        normalize(wrapped)
    );
}

/// "In full" is not one spelling of either clause.
///
/// D-2 is a two-half rule and "stated in full" has to survive a rewording of *either* half. The
/// first version of this case pinned three wordings of the ambiguity clause and left the zero
/// clause a single literal, which closed the defect on one half and left the other exactly as it
/// had been — the same mistake, one clause along. Every wording below is checked in both
/// directions, and a passage carrying only one of the two refusals is checked to read as a
/// citation rather than a restatement.
#[test]
fn in_full_is_not_tied_to_one_spelling_of_either_clause() {
    let full = [
        "Zero implementations for an obligation is an unsatisfied obligation; two is an \
         ambiguity error naming both.",
        "Zero implementations for an obligation remains an unsatisfied obligation; two remains \
         an ambiguity naming both.",
        "Zero offers is an unsatisfied obligation; two is an ambiguity naming every claimant.",
        "With nothing offered the obligation is unsatisfied; with two offered it is an ambiguity \
         naming both.",
        "Nothing offered leaves an unfilled obligation, and two offered is an ambiguity error.",
    ];
    for passage in full {
        assert!(
            states_d2_in_full(passage),
            "this states both refusals and must read as a full statement: {passage}"
        );
    }

    let not_full = [
        "Gap register D-2 governs the linker; see the page that states it.",
        // The zero half alone — the tree's own commonest citation shape.
        "Zero implementations for an obligation is an unsatisfied obligation.",
        // The two half alone.
        "Two implementations for one obligation is an ambiguity error naming both.",
    ];
    for passage in not_full {
        assert!(
            !states_d2_in_full(passage),
            "one refusal of the two is not the whole rule: {passage}"
        );
    }
}

/// The reference scan reads files that are neither Markdown nor under `docs/`.
///
/// The register reference in `.gitignore` outlived every earlier attempt to find them all, because
/// each one looked at `docs/**/*.md`. A dotfile at the repository root is the shape that scan
/// cannot see, so the scan is defined by exclusion and this case pins that it stays that way.
#[test]
fn the_reference_scan_reads_files_that_are_neither_markdown_nor_under_docs() {
    let root = workspace_root();
    let scanned = scanned_files(&root);

    assert!(
        scanned.iter().any(|relative| relative == ".gitignore"),
        "the scan must read .gitignore, which is neither Markdown nor under docs/"
    );
    assert!(
        scanned.iter().any(|relative| relative == "Taskfile.yml"),
        "the scan must read Taskfile.yml, which is not Markdown"
    );
    assert!(
        !scanned
            .iter()
            .any(|relative| relative.starts_with(".engineering/")),
        "the scan must not read the append-only plan store"
    );
    assert!(
        !scanned.iter().any(|relative| relative == this_file()),
        "the scan must not read its own negative fixtures"
    );
}

/// The scan's verdict does not depend on what has been built locally.
///
/// `generated/rust/*/target/`, `examples/billing-web/target/` and `website/node_modules` are
/// ignored, they exist on a machine that has built recently and not on one that has not, and they
/// contain vendored Markdown and source by the thousand. A scan that walks the filesystem reads
/// them and one that asks Git does not, so the same tree gives two answers depending on which
/// commands the reader happened to run first — and the cheap fix, a prefix per directory as each
/// is discovered, leaves the next one to the next adversary.
#[test]
fn the_scan_reads_no_ignored_build_output_whether_or_not_it_exists() {
    let root = workspace_root();
    let scanned = scanned_files(&root);

    let ignored: Vec<&String> = scanned
        .iter()
        .filter(|relative| {
            relative.contains("/target/")
                || relative.starts_with("target/")
                || relative.contains("node_modules/")
        })
        .collect();
    assert!(
        ignored.is_empty(),
        "the scan must read no ignored build output; it read: {ignored:?}"
    );

    // The guarantee is Git's, not a prefix list's: every scanned path is one Git reports.
    let tracked = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(&root)
        .output()
        .expect("git reports this repository's source files");
    let known: Vec<String> = tracked
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).into_owned())
        .collect();
    assert!(
        scanned.iter().all(|relative| known.contains(relative)),
        "every scanned path must be one Git reports as source"
    );
}
