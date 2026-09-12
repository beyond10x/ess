//! Adversarial cases against the correction round of `story:host-path-lane-detector-bounds`.
//!
//! The correction replaced a panic with two things that now stand in for judgement: a **shape
//! guard** over the labels the parse produces, and a **control over this repository's own
//! `.github/workflows/ci.yml`** asserting that the parse sees every runner the workflow names.
//! Both are documents the unit wrote about its own code, and nothing else compares them to it, so
//! that is what these cases do: each drives the lane's parse against a sentence the lane's own
//! source states, not against the behaviour it shipped.
//!
//! Every shape below was first put through a YAML parser rather than supposed to be legal; the
//! three the parse cannot read are ordinary documents that `yaml.safe_load` resolves to exactly
//! the list a reader would expect. One shape that looked like an attack — a tab before a `#`
//! comment — is **not** legal YAML, and is therefore not here.
//!
//! Like the three adversarial targets beside it, every case drives the copy in
//! `host_paths_lane/mod.rs` and calls `assert_current()` first, so a lane edit turns these into a
//! named complaint rather than a measurement of a fossil. Nothing here spells a home-directory
//! marker as a literal.

mod host_paths_lane;

use host_paths_lane::{assert_current, runner_labels, LANE, RUNNER_HOME_ROOTS};
use std::collections::BTreeSet;

/// The lane's own shape guard, `host_paths.rs:965`, copied as a predicate.
///
/// In the lane it is an assertion inside
/// `the_workflow_parse_reads_every_runner_the_workflow_names`, applied to the nine hand-written
/// workflows in that case's own table and to nothing else. It never sees a label read out of
/// `.github/workflows/ci.yml`, and it is not consulted by `runner_labels`, `ci_runner_labels` or
/// `the_markers_cover_the_home_root_of_every_platform_ci_runs_on`. Here it is a function so the
/// cases below can say, of a label the parse invented, whether the guard would have objected.
fn passes_the_shape_guard(label: &str) -> bool {
    !label.is_empty()
        && !label.contains(char::is_whitespace)
        && !label.contains(['$', '{', '}', '#', '[', ']', ',', '"', '\''])
}

/// Whether the lane can look a home root up for `label`, as `RUNNER_HOME_ROOTS` is consumed.
///
/// A label this answers `false` for is one
/// `the_markers_cover_the_home_root_of_every_platform_ci_runs_on` panics on by name, saying it
/// runs CI — so a label the parse invents and this refuses is a red gate on a clean repository.
fn resolvable(label: &str) -> bool {
    RUNNER_HOME_ROOTS
        .resolve()
        .iter()
        .any(|(family, _)| label.starts_with(family.as_str()))
}

/// Every label the parse attributes to each `runs-on:` line of `workflow`, keyed by line number.
///
/// One line at a time, by blinding the parse to the *other* `runs-on:` lines and leaving the rest
/// of the document alone — so a matrix defined elsewhere still resolves, which is the whole reason
/// `matrix_values` reads the file rather than the job. This is the measurement the sentence at
/// `host_paths.rs:981` describes, "every `runs-on:` line in it has to contribute at least one
/// label", and the one the assertion at `host_paths.rs:990` does not make.
fn labels_per_runs_on_line(workflow: &str) -> Vec<(usize, BTreeSet<String>)> {
    let lines: Vec<&str> = workflow.lines().collect();
    let targets: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("runs-on:"))
        .map(|(number, _)| number)
        .collect();
    targets
        .iter()
        .map(|target| {
            let alone: Vec<String> = lines
                .iter()
                .enumerate()
                .map(|(number, line)| {
                    if number == *target || !targets.contains(&number) {
                        (*line).to_owned()
                    } else {
                        line.replace("runs-on:", "not-the-line-under-test:")
                    }
                })
                .collect();
            (target + 1, runner_labels(&alone.join("\n")))
        })
        .collect()
}

/// The `runs-on:` lines of `workflow` that contribute no label, by line number.
///
/// Retired here: this file used to carry a hand copy of the lane's old `labels.len() >= written`
/// arithmetic and compare the lane against it. That copy is not in `TRANSCRIBED`, so it froze while
/// the lane moved, and the comparison became one no implementation could satisfy — a `BTreeSet` of
/// one distinct runner name can never have two elements, however correct the parse is. What the
/// case was arguing for is what the lane now does, so the case asks for that directly.
fn blind_runs_on_lines(workflow: &str) -> Vec<usize> {
    labels_per_runs_on_line(workflow)
        .iter()
        .filter(|(_, labels)| labels.is_empty())
        .map(|(line, _)| *line)
        .collect()
}

/// A block sequence is one YAML shape, and the lane reads one of the three ways it is written.
///
/// `block_sequence` at `host_paths.rs:468` was added by the correction round to close the bound
/// the first adversary pass named, and its doc says why: a block sequence is "the ordinary
/// spelling for a runner named by several labels — `- self-hosted` and then the platform, which is
/// exactly how a platform with no row in `RUNNER_HOME_ROOTS` arrives. Read it or the job
/// contributes no label at all, which is this gate narrowing itself in silence on the one platform
/// it has never heard of."
///
/// It reads a block sequence whose items are indented *deeper* than the key, whose items are
/// uninterrupted, and only those. Three ordinary documents fall outside that, and each was put
/// through a YAML parser before it was written here:
///
/// * items at the **same** indentation as the key. This is legal YAML — a sequence entry may sit
///   at its parent mapping's own indentation — and it is what several formatters emit. The lane
///   reads **no label at all** from it, which is the exact silence the function was added to end.
/// * a **comment line** between two items. `.github/workflows/ci.yml` carries a comment above a
///   value line in five places, so this is this repository's own habit; the lane stops at the
///   comment and every item after it is invisible.
/// * a **blank line** between two items, which stops it the same way.
///
/// Silence is the failure this bound exists to remove: a platform with no home root in
/// `HOME_MARKERS` arrives through exactly these shapes, and a parse that returns nothing for it
/// narrows the gate without saying so.
#[test]
fn a_block_sequence_is_read_however_yaml_allows_it_to_be_written() {
    assert_current();
    let mut unread = Vec::new();
    for (spelling, workflow) in [
        (
            "items at the key's own indentation",
            "jobs:\n  native:\n    runs-on:\n    - self-hosted\n    - freebsd-14\n",
        ),
        (
            "a comment line between two items",
            "jobs:\n  native:\n    runs-on:\n      - self-hosted\n      # the ARM fleet\n      \
             - freebsd-14\n",
        ),
        (
            "a blank line between two items",
            "jobs:\n  native:\n    runs-on:\n      - self-hosted\n\n      - freebsd-14\n",
        ),
    ] {
        let labels = runner_labels(workflow);
        for expected in ["self-hosted", "freebsd-14"] {
            if !labels.iter().any(|label| label == expected) {
                unread.push(format!(
                    "  {spelling}: `{expected}` is one of the two labels this job runs on, and \
                     the parse read {labels:?}"
                ));
            }
        }
    }
    assert!(
        unread.is_empty(),
        "each workflow above is a document a YAML parser resolves to the two labels named, and \
         `block_sequence` in `{LANE}` was added so that a `runs-on:` written as a block sequence \
         is not silence — its own doc says a job that contributes no label is \"this gate \
         narrowing itself in silence on the one platform it has never heard of\". It reads the \
         deeper-indented, uninterrupted spelling only:\n{}",
        unread.join("\n")
    );
}

/// A runner group is a `runs-on:` GitHub Actions documents, and the lane reads it as nothing.
///
/// `runs-on:` takes a mapping of `group:` and `labels:` as well as a scalar and a sequence; it is
/// how a repository addresses a self-hosted runner group, and it is legal YAML. The lane's parse
/// finds the value empty, hands the line to `block_sequence`, which finds no `- ` item and stops —
/// so the job contributes no label.
///
/// This is the same silence as the case above and it costs twice, because it is also the shape
/// that turns the repository control red. Written into `.github/workflows/ci.yml`, a `runs-on:`
/// mapping contributes nothing while still counting as a `runs-on:` line at `host_paths.rs:985` —
/// the failure the correction round was opened to remove, in the mechanism that replaced it.
#[test]
fn a_runner_group_mapping_is_a_runs_on_value_the_parse_can_see() {
    assert_current();
    let workflow = "jobs:\n  gate:\n    runs-on: ubuntu-latest\n  fleet:\n    runs-on:\n      \
                    group: ubuntu-runners\n      labels: ubuntu-22.04-16core\n";
    let labels = runner_labels(workflow);
    assert!(
        labels.iter().any(|label| label == "ubuntu-latest"),
        "the parse did not find the runner it does know, so this case measures nothing: {labels:?}"
    );
    assert!(
        labels.len() > 1,
        "`runs-on:` takes a `group:`/`labels:` mapping as well as a scalar and a sequence, and \
         this workflow runs two jobs on two different runners; the parse in `{LANE}` read \
         {labels:?}, so the second job's platform is invisible to it — and in \
         `.github/workflows/ci.yml` the same shape is a `runs-on:` line contributing no label, \
         which is what the control at `host_paths.rs:990` is there to refuse"
    );
}

/// The repository control counts labels where its own sentence counts lines.
///
/// `host_paths.rs:981` states the criterion: "every `runs-on:` line in it has to contribute at
/// least one label". `host_paths.rs:990` asserts something else — that the *number of distinct
/// labels* is at least the number of `runs-on:` lines — and the two are not the same predicate in
/// either direction. `labels` is a `BTreeSet`, so two jobs on one runner are one label.
///
/// **It refuses a clean workflow.** Two jobs that both run on `ubuntu-latest` is the most ordinary
/// shape a workflow has. Every line resolves, nothing is unseen, and the criterion rejects it.
/// That is the failure mode the correction round was opened for — a legitimate edit to `ci.yml`
/// turning the gate red on a repository with nothing wrong in it — reproduced in the assertion
/// that replaced the panic. The slack that hides it today is an accident of arithmetic: `ci.yml`
/// has two `runs-on:` lines and three labels, so it survives one duplicate and not two, and a
/// single edit reaches it — retiring one of the two macOS architectures from the matrix, or
/// writing that job's runners as a block sequence, leaves two lines and one label.
///
/// **It passes a workflow with a job it cannot see.** One `runs-on:` naming an input the parse
/// cannot resolve, beside one naming three labels, counts three against two and is accepted —
/// while the job whose platform is invisible is exactly what the control exists to name.
#[test]
fn the_repository_control_measures_the_sentence_it_is_written_to_mean() {
    assert_current();
    let mut disagreed = Vec::new();
    for (shape, workflow, expected) in [
        (
            "two jobs on one runner, every line resolved",
            "jobs:\n  gate:\n    runs-on: ubuntu-latest\n  docs:\n    runs-on: ubuntu-latest\n",
            Vec::new(),
        ),
        (
            "one line the parse cannot resolve, beside one naming three labels",
            "jobs:\n  called:\n    runs-on: ${{ inputs.runner }}\n  fleet:\n    runs-on: \
             [self-hosted, linux, x64]\n",
            vec![3usize],
        ),
    ] {
        let blind = blind_runs_on_lines(workflow);
        if blind != expected {
            disagreed.push(format!(
                "  {shape}: the `runs-on:` lines contributing no label are {blind:?}, and the \
                 sentence the control is written to mean makes them {expected:?}"
            ));
        }
    }
    assert!(
        disagreed.is_empty(),
        "the control is documented as holding the workflow to \"every `runs-on:` line in it has to \
         contribute at least one label\", which is a claim about each line and not about a count. \
         Two jobs sharing a runner is one distinct label across two lines and must be accepted; a \
         line the parse cannot resolve must be named however many labels the other lines \
         contribute:\n{}",
        disagreed.join("\n")
    );
}

/// An expression the parse cannot resolve contributes a label after all, one level down.
///
/// `runner_labels` at `host_paths.rs:530` documents the rule without qualification: "**An
/// expression this parse cannot resolve contributes no label**, deliberately: the alternative is
/// to hand the case that consumes these labels the expression itself, which panics naming a
/// platform nobody runs and turns the gate red on a workflow that is not wrong."
///
/// The rule is applied to the `runs-on:` value and to nothing else. A matrix entry that is itself
/// an expression — a repository variable or a workflow input naming a fleet, which is how a
/// reusable workflow parameterises its runners — is read by `matrix_values`, unquoted by
/// `list_items` and returned as a runner label. `the_markers_cover_the_home_root_of_every_platform_ci_runs_on`
/// then panics on it by name.
///
/// The shape guard at `host_paths.rs:965` would have objected to this string, and does not see it:
/// it is applied to the nine workflows in that case's own table and to no label the parse reads
/// out of a file.
#[test]
fn an_expression_the_parse_cannot_resolve_contributes_no_label_wherever_it_is_written() {
    assert_current();
    let workflow = "jobs:\n  matrixed:\n    runs-on: ${{ matrix.runner }}\n    strategy:\n      \
                    matrix:\n        runner: [ubuntu-latest, \"${{ vars.FLEET }}\"]\n";
    let labels = runner_labels(workflow);
    assert!(
        labels.iter().any(|label| label == "ubuntu-latest"),
        "the parse did not find the runner it does know, so this case measures nothing: {labels:?}"
    );
    let invented: Vec<&String> = labels.iter().filter(|label| !resolvable(label)).collect();
    assert!(
        invented.is_empty(),
        "`runner_labels` in `{LANE}` states that an expression it cannot resolve contributes no \
         label, because the alternative \"panics naming a platform nobody runs and turns the gate \
         red on a workflow that is not wrong\"; it applies that rule to the `runs-on:` value only, \
         and a matrix entry that is an expression comes back as a runner: {invented:?} out of \
         {labels:?}"
    );
}

/// A string the shape guard accepts is not thereby a runner label.
///
/// The correction round replaced a hand-maintained expectation list with a machine-checkable form
/// — `host_paths.rs:965`, every label "must contain no whitespace and none of `$ { } # [ ] , " '`"
/// — and that form is what now stands in for knowing whether the parse read a runner or a piece of
/// the document. It is a weak property, and `matrix_values` produces strings that satisfy it and
/// are not runners.
///
/// `matrix_values` at `host_paths.rs:492` looks a key up "across the whole file rather than inside
/// the job that names the expression", and its doc says why that is safe: "a parse that guesses
/// wrong about scope here should over-collect: an extra label is a platform the lane checks a home
/// root for, and a missing one is a platform it knows nothing about." The first half is false. An
/// extra label is not checked, it is looked up in `RUNNER_HOME_ROOTS` and, when no row matches,
/// **panicked on** — so over-collection is not the safe direction, it is a red gate on a clean
/// workflow.
///
/// `os` is the key GitHub's own documentation uses for a runner matrix, and it is also an ordinary
/// key for a matrix of container images or target platforms in another job of the same file. Every
/// value of every `os:` in the file becomes a runner label for the first job. `alpine` and
/// `debian` carry no whitespace and none of the nine characters, so the guard has nothing to say
/// about them; they are still not runners.
#[test]
fn a_matrix_key_another_job_defines_is_not_a_runner_this_workflow_runs_on() {
    assert_current();
    let workflow = "jobs:\n  gate:\n    runs-on: ${{ matrix.os }}\n    strategy:\n      \
                    matrix:\n        os: [ubuntu-latest]\n  images:\n    runs-on: \
                    ubuntu-latest\n    strategy:\n      matrix:\n        os: [alpine, debian]\n    \
                    container: ${{ matrix.os }}\n";
    let labels = runner_labels(workflow);
    assert!(
        labels.iter().any(|label| label == "ubuntu-latest"),
        "the parse did not find the runner it does know, so this case measures nothing: {labels:?}"
    );
    let invented: Vec<String> = labels
        .iter()
        .filter(|label| !resolvable(label))
        .map(|label| {
            format!(
                "`{label}` (the shape guard {} it)",
                if passes_the_shape_guard(label) {
                    "accepts"
                } else {
                    "refuses"
                }
            )
        })
        .collect();
    assert!(
        invented.is_empty(),
        "this workflow runs both its jobs on `ubuntu-latest` and the second matrixes over \
         container images; `matrix_values` in `{LANE}` reads every `os:` in the file and its doc \
         calls that over-collection harmless — \"an extra label is a platform the lane checks a \
         home root for\". It is not checked, it is panicked on by \
         `the_markers_cover_the_home_root_of_every_platform_ci_runs_on`, and the shape guard that \
         replaced the expectation list has nothing to say about a string that is shaped like a \
         label and is not one: {}",
        invented.join(", ")
    );
}
