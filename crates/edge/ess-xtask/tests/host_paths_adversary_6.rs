//! Adversarial pass 2 against `story:planning-store-carries-workstation-paths`.
//!
//! Pass 1 attacked the decision the unit wrote. Correction round 1 answered it by widening
//! `documented_unread_trees` from "the first backtick token" to `(tree, unread, files, lines)`,
//! and by delimiting the list with two anchors instead of one. This pass attacks **that
//! correction**, and it attacks it against the correction's own prose rather than against the
//! behaviour the unit built — the two were written by the same hand in the same commit, so a
//! suite that agrees with the code proves nothing about whether the code agrees with the document.
//!
//! Three claims are driven here, each the lane's own words:
//!
//! * `host_paths.rs`'s doc on `documented_unread_trees` says the first version's defect was that
//!   *"every number in the bullet was unread prose. One of those numbers was wrong."* The parse
//!   reads the two counts before the bullet's colon. Everything after it — six more numbers and
//!   the comparison that was the one that was wrong — is still unread prose.
//! * `host_paths.rs`'s doc on `UNREAD_TREE_SECTION` says that with an explicit closing line
//!   *"every bullet between the anchors is read whatever its continuations are indented with"*.
//!   A bullet between the anchors written with any other list marker is dropped without a word.
//! * the same doc says *"a missing closing anchor is a panic rather than a short list"*. It says
//!   nothing about a **second** closing anchor, and a second one is a short list with no panic —
//!   which is the failure the two anchors were chosen to eliminate.
//!
//! Everything here drives the transcribed copy in `host_paths_lane/mod.rs` and calls
//! `assert_current()` first, so a lane edit turns these into a named complaint rather than a
//! measurement of a fossil. Nothing here spells a home-directory marker as a literal.

mod host_paths_lane;

use host_paths_lane::{assert_current, documented_unread_trees, LANE};

/// A module documentation built from one bullet head and whatever reasons follow its colon.
///
/// The head is fixed at the lane's own, so the only thing that varies between two documents built
/// this way is the text the parse stops before.
fn document_with_reasons(reasons: &[&str]) -> String {
    let mut text = String::from(
        "//! No file this repository tracks under `crates/` names a workstation path.\n\
         //!\n\
         //! What it does not read at all, tree by tree:\n\
         //!\n\
         //! * `.engineering/` \u{2014} unread, 60 files, 32806 lines: the planning store.\n",
    );
    for line in reasons {
        text.push_str("//!   ");
        text.push_str(line);
        text.push('\n');
    }
    text.push_str("//!\n//! That is the whole of that list.\n");
    text
}

/// The numbers the shipped bullet states after its colon are read by nothing.
///
/// `host_paths.rs`'s doc on `documented_unread_trees` gives the reason the parse was widened:
/// the first version *"took the bullet's first backtick token and stopped … and it meant every
/// number in the bullet was unread prose. One of those numbers was wrong."* The widened parse
/// stops at the bullet's first colon — its own comment says so, *"up to the first colon, so a
/// bullet's reasons can hold anything at all"* — and the bullet the unit shipped states two
/// numbers before that colon and 87, 0.27%, 28730, 59 of 60, 99.7% and 32163 of 65838 after it.
///
/// The number that was wrong was not a count in the head. It was the comparison in the reasons.
/// So this case hands the parse the corrected reasons and the falsified ones, differing in
/// nothing but the text after the colon, and asks it to tell them apart. The falsified document
/// also contradicts its own head outright — it says the tree holds one file and two lines while
/// the head says sixty and 32806 — so the case does not rest on the parse understanding English.
#[test]
#[ignore = "story:the-unread-tree-bullet-is-read-whole — the widened parse stops at the bullet's first colon, so the corrected bullet and the falsified one parse alike"]
fn the_parse_cannot_tell_the_corrected_bullet_from_the_one_the_correction_removed() {
    assert_current();
    let corrected = document_with_reasons(&[
        "The journal holds 87 of those 32806 carrying lines, which is 0.27%, and one",
        "review-result holds 28730. A carve-out would cover 59 of the 60 files.",
    ]);
    let falsified = document_with_reasons(&[
        "Carving the journal out of a widened scan would report the store covered while",
        "exempting the file holding the largest single share of the defect. The tree holds 1",
        "file and 2 lines and the journal holds all of them.",
    ]);

    assert_ne!(
        documented_unread_trees(&falsified),
        documented_unread_trees(&corrected),
        "`{LANE}` widened this parse because the old one left `every number in the bullet` as \
         `unread prose` and `one of those numbers was wrong`. The number that was wrong was the \
         comparison in the bullet's reasons, and the widened parse stops at the bullet's first \
         colon, so the reasons are still unread: the corrected bullet and the falsified one the \
         correction round removed parse to the same answer, and so does a bullet whose reasons \
         contradict its own head. The shipped bullet states six numbers and this parse reads two"
    );
}

/// A bullet between the anchors written with any other list marker is dropped in silence.
///
/// `UNREAD_TREE_SECTION`'s doc gives the two-anchor design its reason: a section that runs until
/// "the first line that is not a bullet" has to decide what a continuation looks like, and
/// *"ended at an explicit line instead, every bullet between the anchors is read whatever its
/// continuations are indented with"*. It is the word **every** that is under attack. The parse
/// keeps only lines whose trimmed text begins `* `, and a continuation is recognised by being
/// none of those — so a line that is a bullet and is not spelt with an asterisk is indistinguish-
/// able from prose and leaves no trace.
///
/// The direction that matters is the quiet one, and it is the one pass 1 named: a dropped bullet
/// for a tree that still carries turns the `undocumented` half red loudly, while a dropped bullet
/// for a tree that has gone clean cannot appear in the `stale` difference at all. Documentation
/// that says a tree is unread and carrying, a repository that disagrees, and a green lane.
#[test]
#[ignore = "story:the-unread-tree-bullet-is-read-whole — UNREAD_TREE_SECTION's doc claims every bullet between the anchors is read; a dash-marked one is dropped without a word"]
fn a_bullet_between_the_anchors_is_dropped_unless_it_is_spelt_with_an_asterisk() {
    assert_current();
    let head = "//! s\n//!\n//! What it does not read at all, tree by tree:\n//!\n\
                //! * `.engineering/` \u{2014} unread, 60 files, 32806 lines: the planning store.\n";
    let tail = "//!\n//! That is the whole of that list.\n";
    let second = "`fuzz/` \u{2014} unread, 1 files, 1 lines: a second tree, added the way the \
                  lane prescribes.";

    let asterisk = format!("{head}//! * {second}\n{tail}");
    let dashed = format!("{head}//! - {second}\n{tail}");

    assert_eq!(
        documented_unread_trees(&dashed),
        documented_unread_trees(&asterisk),
        "`{LANE}` says of its two anchors that `every bullet between the anchors is read`. A \
         bullet written with a dash rather than an asterisk is between the anchors, renders as a \
         bullet in rustdoc, and is read by nothing: the parse keeps a line only if its trimmed \
         text begins `* `, and every other line in the section is silently taken for a \
         continuation. The remedy this lane prescribes for a tree that starts carrying the class \
         is `adding a bullet`, and a bullet added this way leaves the tree undocumented with no \
         complaint at all"
    );
}

/// A second closing anchor is a short list, and the two-anchor design was chosen against exactly
/// that.
///
/// `UNREAD_TREE_SECTION`'s doc states the asymmetry it fixed — *"a missing closing anchor is a
/// panic rather than a short list"* — and stops there. The parse takes the first line matching
/// the closing anchor **after** the opening one, compared on `line.trim()`, so a continuation
/// line whose text is the closing sentence closes the section where it stands. Every bullet after
/// it is dropped, `claimed` is still non-empty so the emptiness guard does not fire, and the run
/// is green.
///
/// This is the same silent truncation the tab-indented continuation produced before the
/// correction, reached through the anchor the correction added rather than through the
/// indentation it removed.
#[test]
#[ignore = "story:the-unread-tree-bullet-is-read-whole — a second closing anchor truncates the list silently and neither guard floor notices"]
fn a_closing_anchor_inside_a_bullet_truncates_the_list_without_a_panic() {
    assert_current();
    let head = "//! s\n//!\n//! What it does not read at all, tree by tree:\n//!\n\
                //! * `.engineering/` \u{2014} unread, 60 files, 32806 lines: the planning store.\n";
    let tail = "//! * `fuzz/` \u{2014} unread, 1 files, 1 lines: a second tree.\n\
                //!\n//! That is the whole of that list.\n";

    let plain =
        format!("{head}//!   Its reasons continue here, and that is the whole of them.\n{tail}");
    let anchored = format!("{head}//!   That is the whole of that list.\n{tail}");

    assert_eq!(
        documented_unread_trees(&anchored),
        documented_unread_trees(&plain),
        "`{LANE}` chose two anchors so that `a missing closing anchor is a panic rather than a \
         short list`, and said nothing about a second one. A continuation line whose trimmed text \
         is the closing sentence closes the section where it stands: every bullet after it is \
         dropped, the `claimed.is_empty()` guard does not fire because the first bullet survived, \
         and the lane is green on a list it read half of. This is the pre-correction truncation \
         reached through the anchor rather than through the indentation"
    );
}
