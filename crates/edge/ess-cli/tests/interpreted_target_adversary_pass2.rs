//! Adversary pass 2 against `story:interpreted-target-selection`.
//!
//! Round 1 found that `interpret.rs` claimed, without qualification, that a run against
//! `--target interpreted` can never be green. Round 2's correction qualified that claim in the Rust
//! module doc — `interpret.rs:24-27` now scopes it to "a suite holding **at least one** scenario",
//! and `interpret.rs:29-35` names the empty-suite verdict as the runner's, target-independent, and
//! cites `ess verify conform select --ids` documenting `[]` as "a selection somebody can actually
//! ask for".
//!
//! The correction to the *other* round-1 finding wrote the unqualified version of the same claim
//! into the published adopter guide. `website/docs/guides/verify-conformance.md:149-151` now says of
//! this target: "it executes nothing, so every scenario comes back as an unsatisfied obligation and
//! the run fails." There is no qualification, and the same guide documents the `[]` selection at
//! line 238 — the exact input for which the run does not fail. The claim the round-1 case removed
//! from a doc comment is now in a public page, where it is worth more.
//!
//! This case is about the document, because the behaviour is settled: the empty-suite verdict is
//! `report.rs:566-578`'s and changing it would change `billing` and `oracle-fixture` too, which
//! `interpreted_target_adversary.rs` already pins across targets. What is not settled is a guide
//! sentence that states one outcome for this target and does not say when it holds.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io::Write};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Whitespace collapsed, so a claim wrapped across source lines reads as one sentence.
fn unwrapped(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// What the guide actually reports for a suite holding no scenarios, measured rather than argued.
fn empty_suite_outcome(target: &str) -> (String, Option<i32>) {
    let text = fs::read_to_string(root().join("suites/generated/billing/suite.json"))
        .expect("the committed billing suite");
    let mut suite: serde_json::Value =
        serde_json::from_str(&text).expect("the committed suite is JSON");
    suite["scenarios"] = serde_json::json!({});

    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary-pass2-empty-suite.json");
    let mut file = fs::File::create(&path).expect("a suite file under the test target directory");
    file.write_all(format!("{suite:#}\n").as_bytes())
        .expect("writing the suite");
    drop(file);

    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "run", "--target", target])
        .arg("--suite")
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("the `ess` binary runs");
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("report/1 is rendered as JSON");
    (
        report["status"].as_str().expect("a run status").to_owned(),
        output.status.code(),
    )
}

/// A run-level outcome the guide states for `--target interpreted` must say when it holds.
///
/// The guide describes this target in prose with no generator behind it, exactly as its target list
/// was before `interpreted_target.rs:70` started checking that list. This checks the other half of
/// the same paragraph: not *which* targets it names but *what it promises* one of them does.
///
/// The rule is the one `interpret.rs:24-35` already follows — a sentence that states how a run comes
/// out has to name the suites it is true of, because there is one it is not true of and the same
/// guide documents how to build it (`verify-conformance.md:238-240`, `[]` explicitly selects none).
/// Qualifying the sentence turns this green; so does dropping the run-level promise and keeping only
/// the per-scenario one, which is unconditionally true.
#[test]
fn the_guide_qualifies_the_run_outcome_it_promises_for_the_interpreted_target() {
    let guide = fs::read_to_string(root().join("website/docs/guides/verify-conformance.md"))
        .expect("the conformance guide is committed");
    let guide = unwrapped(&guide);

    let start = guide
        .find("`interpreted` selects")
        .expect("the guide describes what `--target interpreted` selects");
    // The claim, and the sentence after it, so a qualification carried in either one counts.
    let description: String = guide[start..]
        .split(". ")
        .take(2)
        .collect::<Vec<_>>()
        .join(". ");
    let lowered = description.to_lowercase();

    let promise = [
        "the run fails",
        "run fails",
        "exits non-zero",
        "exit non-zero",
        "cannot be green",
        "never green",
    ]
    .into_iter()
    .find(|phrase| lowered.contains(phrase));

    let Some(promise) = promise else {
        // No run-level promise is made, so there is nothing here that needs a scope.
        return;
    };

    let qualified = [
        "at least one",
        "holding a scenario",
        "holds a scenario",
        "holding at least",
        "non-empty",
        "nonempty",
        "empty",
        "no scenarios",
        "any scenario",
    ]
    .into_iter()
    .any(|phrase| lowered.contains(phrase));

    let (status, code) = empty_suite_outcome("interpreted");

    assert!(
        qualified,
        "website/docs/guides/verify-conformance.md promises `{promise}` for `--target interpreted` \
         and does not say for which suites. For a suite holding no scenarios — the selection the \
         same guide documents at verify-conformance.md:238 as `[]` — that run reports `{status}` \
         and exits {code:?}. interpret.rs:24-35 states the same claim with the scope attached; this \
         sentence is the unqualified form the round-1 case removed from that module. The guide \
         says:\n  {description}"
    );
}
