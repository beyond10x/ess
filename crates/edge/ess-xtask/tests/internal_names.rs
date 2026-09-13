//! No file this repository tracks names the organisation it was written inside.
//!
//! `beyond10x/ess` is public and goes through GitHub pull requests. Its planning store, its design
//! pages and its stories were written while adopting the specification format inside a company, and
//! the adoption's own vocabulary came with them: the employer's name, its product's name, the
//! repositories the measurements were taken in. None of that is a secret in the sense a credential
//! is, and every one of them is a disclosure — the kind a reader assembles from a name here and a
//! repository path there.
//!
//! On 2026-09-12 the organisation's name and two of its repositories were found 89 times across
//! fifteen tracked files, of which forty were already in merged history. They were not found by
//! anybody reading; they were found because a commit hook refused a commit, and the hook exempts
//! content that reached the baseline before it was installed. This lane is what the hook cannot be:
//! a check that holds the *whole* tree rather than the new half of it, and that fails in this
//! repository's own test run rather than at a push somebody may not be making.
//!
//! ## What it scans, and why that is everything
//!
//! Every file `git ls-files` reports, with one exemption named below. Not a prefix list.
//! [`host_paths.rs`](../host_paths.rs) scans four published trees and deliberately leaves
//! `.engineering/` alone, on the argument that the planning store is engineering record rather than
//! published source. That argument does not transfer. A store document is a file in a public
//! repository whichever tree it sits in, and in this case the store held sixty-four of the eighty-nine
//! occurrences. The one lane that excluded the store is the lane that would have missed this.
//!
//! ## The list is hand-written, and this sentence is the reason that is said out loud
//!
//! [`FORBIDDEN`] is a table somebody maintains. Nothing derives it, nothing can: an organisation's
//! name is not a shape a program recognises, and a scanner that guessed at proper nouns would refuse
//! this file's own prose. So this lane catches a name that is already known and nothing else, and a
//! second employer, a customer or an acquired product arrives unguarded until somebody adds a row.
//!
//! Four separate claims of exhaustiveness in this repository turned out to rest on a hand-written
//! table during waves 22 to 24, each time discovered by an adversary rather than by the claim's
//! author. This one says it before being asked.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

/// This file, which is scanned like every other and carries no exemption.
///
/// The first draft exempted it, on the reasoning that a lane looking for names has to contain them.
/// That reasoning was wrong: [`forbidden`] assembles each name from fragments, so this file never
/// holds one, and an exemption for a file that does not need one is a hole with no purpose.
/// `this_lane_is_held_to_its_own_rule` is what replaced it.
const LANE: &str = "crates/edge/ess-xtask/tests/internal_names.rs";

/// The names no tracked file may contain, as a stem and the tails that follow it.
///
/// Split this way because a name is not only ever written the way it was registered. A stem and a
/// tail run together, hyphenated, underscored, spaced or capitalised are one disclosure in five
/// dresses, and a check that matched a single dress would read as covering all of them.
/// [`occurrences`] joins a stem to a tail across up to three non-alphanumeric characters, which is
/// what those five have in common. The dresses are not written out here — this file is scanned like
/// every other, so an example in its prose is a finding, which is how the first draft of this
/// comment failed its own lane.
///
/// Assembled from fragments rather than written whole. A literal would make the constant itself an
/// occurrence, and then either the scan reports its own table or the table has to be exempted from
/// a search it defines — and an exemption inside the definition is how a check comes to pass for a
/// reason nobody intended.
///
/// **What this cannot see**, said here rather than discovered later: a name split across more than
/// three separators, one spelt with a Unicode homoglyph, one abbreviated, and any name not in this
/// table. The table is hand-maintained and nothing can derive it — an organisation's name is not a
/// shape a program recognises.
fn forbidden() -> Vec<(String, Vec<String>, &'static str)> {
    vec![(
        "babel".to_owned(),
        vec!["connect".to_owned(), "force".to_owned()],
        "the employer this format was adopted inside, its product, and the prefix of two of its \
         repositories",
    )]
}

/// How many characters of `text` from `at` are a separator rather than part of a word.
///
/// Bounded at three. Unbounded, a stem ending one sentence and a tail opening the next would join
/// into a finding, and a disclosure check that cries wolf is one somebody switches off.
fn separator_run(text: &[u8], at: usize) -> Option<usize> {
    let mut run = 0;
    while run < 3 && at + run < text.len() && !text[at + run].is_ascii_alphanumeric() {
        run += 1;
    }
    (at + run <= text.len()).then_some(run)
}

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

/// Every file the repository tracks. No prefix filter, by the argument in the module documentation.
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

/// Each forbidden name `text` contains, with how many times, in any case and any separator spelling.
///
/// Lowercasing the haystack rather than matching twice: a name capitalised in prose and the same
/// name lowercased inside a repository path are one disclosure. The spellings are not written out
/// here for the reason [`forbidden`] gives — a literal in this file's prose is an occurrence like
/// any other, and there is no exemption to fall back on.
fn occurrences(text: &str) -> BTreeMap<String, usize> {
    let lowered = text.to_lowercase();
    let bytes = lowered.as_bytes();
    let mut found = BTreeMap::new();
    for (stem, tails, _) in forbidden() {
        for (start, _) in lowered.match_indices(stem.as_str()) {
            let after = start + stem.len();
            let Some(run) = separator_run(bytes, after) else {
                continue;
            };
            for tail in &tails {
                if lowered[after + run..].starts_with(tail.as_str()) {
                    *found.entry(format!("{stem}{tail}")).or_insert(0) += 1;
                    break;
                }
            }
        }
    }
    found
}

#[test]
fn no_tracked_file_names_the_organisation_this_repository_was_written_inside() {
    let root = workspace_root();
    let mut carrying: Vec<(String, BTreeMap<String, usize>)> = Vec::new();
    for file in tracked_files(&root) {
        let Ok(text) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        let found = occurrences(&text);
        if !found.is_empty() {
            carrying.push((file, found));
        }
    }
    assert!(
        carrying.is_empty(),
        "this repository is public and these tracked files name the organisation it was written \
         inside. Rewrite the sentence so it still says what it measured without naming who \
         measured it — `an internal application repository` and `a second internal adopter` are \
         what the 2026-09-12 redaction used, and they kept every claim true. \
         `.engineering/planning/journal.jsonl` is append-only and has no amend, so a name that \
         reaches it can only be removed by editing it by hand, which is why this lane exists to \
         stop one arriving: {carrying:?}"
    );
}

#[test]
fn the_detector_finds_every_spelling_a_name_is_written_in() {
    for (stem, tails, reason) in forbidden() {
        assert!(
            !reason.is_empty(),
            "`{stem}` is forbidden and the table does not say why, so the next reader cannot tell \
             whether it is still a disclosure or a word that has since become public"
        );
        for tail in &tails {
            let whole = format!("{stem}{tail}");
            // Every separator run this check admits, and the case forms each can be written in.
            // Enumerated rather than sampled: a one-pattern search is what let 89 occurrences of
            // this name sit in a public repository until a commit hook refused them.
            for joiner in ["", "-", "_", " ", ".", "::", " - "] {
                let joined = format!("{stem}{joiner}{tail}");
                for spelling in [
                    joined.clone(),
                    joined.to_uppercase(),
                    format!("{}{}", joined[..1].to_uppercase(), &joined[1..]),
                ] {
                    let sentence = format!("adopted inside {spelling}, measured on 2026-09-11");
                    assert_eq!(
                        occurrences(&sentence).get(&whole),
                        Some(&1),
                        "`{spelling}` is the same disclosure as `{whole}` and the detector missed it"
                    );
                }
            }
            let suffixed = format!("{stem}{tail}-specs and {stem}{tail}-app");
            assert_eq!(
                occurrences(&suffixed).get(&whole),
                Some(&2),
                "a forbidden name is still one when a repository suffix is attached, and both \
                 `-specs` and `-app` were real repositories"
            );
        }
    }
}

#[test]
fn the_separator_run_is_bounded_so_two_sentences_do_not_join_into_a_finding() {
    for (stem, tails, _) in forbidden() {
        for tail in &tails {
            let apart = format!("the {stem} format. Four words later, {tail} happened.");
            assert!(
                occurrences(&apart).is_empty(),
                "`{stem}` and `{tail}` four characters apart are two words, not a name. An \
                 unbounded join makes ordinary prose a finding, and a check that cries wolf is one \
                 somebody switches off: {apart}"
            );
            let edge = format!("{stem}...{tail}");
            assert_eq!(
                occurrences(&edge).get(&format!("{stem}{tail}")),
                Some(&1),
                "three separators is the documented bound and must still join"
            );
            let over = format!("{stem}....{tail}");
            assert!(
                occurrences(&over).is_empty(),
                "four separators is past the documented bound and must not join, or the bound is \
                 not the bound the doc comment states"
            );
        }
    }
}

#[test]
fn the_detector_does_not_fire_on_a_file_that_names_nobody() {
    let innocent = "The compiler refuses a duplicate declaration and cites the file it was read \
                    from. Measured in an internal application repository, 2026-09-11.";
    assert!(
        occurrences(innocent).is_empty(),
        "the redaction's own replacement text must not itself be a finding, or every redacted \
         sentence becomes a new one"
    );
}

#[test]
fn this_lane_is_held_to_its_own_rule() {
    let root = workspace_root();
    assert!(
        root.join(LANE).is_file(),
        "`LANE` names this file and must resolve, or the check below reads nothing and passes"
    );
    let source = fs::read_to_string(root.join(LANE)).expect("this lane reads its own source");
    assert!(
        occurrences(&source).is_empty(),
        "this file is scanned like every other and must not contain a forbidden name. \
         `forbidden` assembles each one from fragments so that this holds; writing a name whole \
         anywhere here — in the table, in a doc comment, in an assertion message — makes the lane \
         report itself, and the fix somebody reaches for next is an exemption. There is no \
         exemption: {:?}",
        occurrences(&source)
    );
}

#[test]
fn the_scan_reads_the_whole_repository_and_not_a_list_of_trees() {
    let root = workspace_root();
    let files = tracked_files(&root);
    for tree in [
        ".engineering/",
        "crates/",
        "docs/",
        "website/",
        "models/",
        "examples/",
    ] {
        assert!(
            files.iter().any(|file| file.starts_with(tree)),
            "`{tree}` is tracked and this scan must reach it. The 2026-09-12 disclosure put \
             sixty-four of its eighty-nine occurrences under `.engineering/`, which the \
             host-path lane excludes by design — this lane excludes nothing"
        );
    }
    assert!(
        files.len() > 500,
        "`git ls-files` returned {} paths, which is too few for this repository; a scan that \
         reads almost nothing passes for the wrong reason",
        files.len()
    );
}
