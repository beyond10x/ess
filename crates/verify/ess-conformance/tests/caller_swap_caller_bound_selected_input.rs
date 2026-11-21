//! beyond10x/ess#805: an input an input guard compares with a caller attribute
//! (`writer != caller.principal`) keeps that binding in the run appended with the callers swapped,
//! also where a row-set selector reads the same input (`writer == input.writer`). The swapped run
//! drew such an input afresh, as it draws every input a selector reads (beyond10x/ess#580), and
//! took the fresh value because the guard, read without the caller, was undecided before and
//! after; the target then answered `not-yours` where the suite expected another branch.
//!
//! A fresh value must now decide every input guard as before over the step's own caller. Where none
//! does, the swapped run keeps the value, its caller's attribute, unless a selector reading it would
//! then find the rows the first run left; only then does the scenario run once, with a note naming
//! that selector.
//!
//! Each model's suite is run against the interpreted target of the same model, and every step that
//! expects a branch past the caller guard sends the input as its own caller's attribute.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_conformance::{report::Status, AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The reproduction of beyond10x/ess#805, in one file.
const MODEL: &str = r#"format: ess/37
system: depot
version: v1
domain: depot.slips
types:
  - name: depot.slips.SlipId
    kind: newtype
    of: String
entities:
  - name: depot.slips.Slip
    identity:
      name: slip_id
      type: depot.slips.SlipId
    fields:
      - {name: writer, type: String}
      - {name: tag, type: String}
    lifecycle:
      initial: Posted
      states: [Posted]
      terminal: [Posted]
      transitions: []
actors:
  - name: depot.slips.Clerk
    attributes:
      - {name: principal, type: String}
    may:
      - depot.slips.PostSlip
errors:
  - name: depot.slips.NotYours
  - name: depot.slips.Duplicate
  - name: depot.slips.TagTaken
commands:
  - name: depot.slips.PostSlip
    input:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: writer, type: String}
      - {name: tag, type: String}
    outcomes:
      - name: not-yours
        when: "writer != caller.principal"
        error: depot.slips.NotYours
      - {name: duplicate, existing_instance: true, error: depot.slips.Duplicate}
      - name: tag-taken
        when_related:
          entity: depot.slips.Slip
          where: {all: ["tag == input.tag", "writer == input.writer"]}
          exists: true
        error: depot.slips.TagTaken
      - name: posted
        creates: depot.slips.Slip
        instance: slip_id
        sets:
          writer: input.writer
          tag: input.tag
        emits: [depot.slips.SlipPosted]
        payload:
          depot.slips.SlipPosted: {slip_id: input.slip_id}
events:
  - name: depot.slips.SlipPosted
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
views:
  - name: depot.slips.Slips
    source: depot.slips.Slip
    consistency: read_your_writes
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: writer, type: String}
      - {name: tag, type: String}
"#;

const SELECTOR: &str = r#"where: {all: ["tag == input.tag", "writer == input.writer"]}"#;
const GUARD: &str = r#"when: "writer != caller.principal""#;

/// `MODEL` with each `(from, to)` replaced, each `from` occurring once.
fn edited(edits: &[(&str, &str)]) -> String {
    let mut text = MODEL.to_owned();
    for (from, to) in edits {
        assert_eq!(
            text.matches(from).count(),
            1,
            "the anchor occurs once: {from:?}"
        );
        text = text.replacen(from, to, 1);
    }
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("slips.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every scenario of `synthesis` run against the interpreted target of `ir`, by id.
fn statuses(synthesis: &Synthesis, ir: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Every `PostSlip` step expecting a branch other than `refused`, as `(scenario, step, writer,
/// caller principal)`, where the two differ.
fn unbound(synthesis: &Synthesis, input: &str, refused: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut last: Option<(usize, Option<&Node>, Option<&Node>)> = None;
        for (at, step) in scenario.steps.iter().enumerate() {
            match step {
                ScenarioStep::ExecuteCommand {
                    command,
                    caller,
                    input: sent,
                    ..
                } if command.to_string() == "depot.slips.PostSlip" => {
                    let value = match sent.get(input) {
                        Some(ScenarioValue::Literal { value }) => Some(value),
                        _ => None,
                    };
                    last = Some((at, value, caller.get("principal")));
                }
                ScenarioStep::ExpectOutcome { outcome }
                    if outcome.command.to_string() == "depot.slips.PostSlip"
                        && outcome.outcome.to_string() != refused =>
                {
                    if let Some((at, value, principal)) = last.take() {
                        if value != principal {
                            found.push(format!(
                                "{id} step {at}: {input} {value:?}, caller principal {principal:?}"
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    found
}

/// `edits`, then a second command `FileSlip` creating a slip with no guard reading the caller or a
/// row: a way to arrange rows `PostSlip`'s own guards keep it from creating.
fn with_filing<'e>(edits: &[(&'e str, &'e str)]) -> Vec<(&'e str, &'e str)> {
    let mut all = edits.to_vec();
    all.extend([
        (
            "      - depot.slips.PostSlip\n",
            "      - depot.slips.PostSlip\n      - depot.slips.FileSlip\n",
        ),
        (
            "events:\n",
            "  - name: depot.slips.FileSlip
    input:
      - {name: slip_id, type: depot.slips.SlipId}
      - {name: writer, type: String}
      - {name: tag, type: String}
    outcomes:
      - {name: filed-twice, existing_instance: true, error: depot.slips.Duplicate}
      - name: filed
        creates: depot.slips.Slip
        instance: slip_id
        sets: {writer: input.writer, tag: input.tag}
        emits: [depot.slips.SlipFiled]
        payload:
          depot.slips.SlipFiled: {slip_id: input.slip_id}
events:
  - name: depot.slips.SlipFiled
    fields:
      - {name: slip_id, type: depot.slips.SlipId}
",
        ),
    ]);
    all
}

/// The suite of `text` is synthesized without a refusal and the interpreted target of the same
/// model passes every scenario.
fn all_pass(label: &str, text: &str) -> Synthesis {
    let ir = ir(text);
    let synthesis = synthesize(&ir);
    assert!(
        synthesis.refusals.is_empty(),
        "{label}: synthesis refuses nothing: {:#?}",
        synthesis.refusals
    );
    let statuses = statuses(&synthesis, &ir);
    let failed: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(
        failed.is_empty(),
        "{label}: the interpreted target passes every scenario of its own suite; failed {failed:?}"
    );
    synthesis
}

/// The scenarios `ids` of the suite of `text` are written, the interpreted target of the same
/// model passes each, and each sends `writer` as its caller's `principal` past `not-yours`.
fn named_pass(label: &str, text: &str, ids: &[&str]) {
    let ir = ir(text);
    let mut synthesis = synthesize(&ir);
    let once = run_once(&synthesis);
    for id in ids {
        assert!(
            !once.contains_key(*id) && appended(&synthesis, id),
            "{label}: `{id}` keeps its run with the callers swapped; notes {once:#?}"
        );
    }
    synthesis
        .suite
        .scenarios
        .retain(|id, _| ids.contains(&id.to_string().as_str()));
    let statuses = statuses(&synthesis, &ir);
    for id in ids {
        assert_eq!(
            statuses.get(*id),
            Some(&Status::Passed),
            "{label}: the interpreted target passes `{id}`: {statuses:?}"
        );
    }
    let unbound = unbound(&synthesis, "writer", "not-yours");
    assert!(unbound.is_empty(), "{label}:\n{}", unbound.join("\n"));
}

/// Every note saying a scenario runs once, without a run appended with the callers swapped or
/// reversed, by scenario.
fn run_once(synthesis: &Synthesis) -> BTreeMap<String, String> {
    synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnswappedCallers { scenario, .. }
            | Note::CrossCallerUnswapped { scenario, .. } => {
                Some((scenario.to_string(), note.to_string()))
            }
            _ => None,
        })
        .collect()
}

/// Whether the scenario `id` carries a run appended to its first, which binds its instances under
/// the `-swapped` suffix.
fn appended(synthesis: &Synthesis, id: &str) -> bool {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .is_some_and(|(_, scenario)| {
            serde_json::to_string(&scenario.steps)
                .unwrap()
                .contains("-swapped\"")
        })
}

const SCENARIOS: [&str; 4] = [
    "depot.slips.PostSlip/outcome/duplicate",
    "depot.slips.PostSlip/outcome/not-yours",
    "depot.slips.PostSlip/outcome/posted",
    "depot.slips.PostSlip/outcome/tag-taken",
];

/// Every scenario of `PostSlip` keeps its appended run and no note says it runs once.
fn every_run_kept(label: &str, synthesis: &Synthesis) {
    let once = run_once(synthesis);
    for id in SCENARIOS {
        assert!(
            !once.contains_key(id) && appended(synthesis, id),
            "{label}: `{id}` keeps its run with the callers swapped; notes {once:#?}"
        );
    }
}

/// [`all_pass`], with `scenarios` scenarios.
fn passes(label: &str, text: &str, scenarios: usize) -> Synthesis {
    let synthesis = all_pass(label, text);
    assert_eq!(
        synthesis.suite.scenarios.len(),
        scenarios,
        "{label}: scenarios {:?}",
        synthesis.suite.scenarios.keys()
    );
    synthesis
}

/// The reproduction: the interpreted target passes every scenario, every scenario keeps its run
/// with the callers swapped, and every step past the caller guard sends `writer` as its own
/// caller's `principal`, in the swapped run too. The selector also reads `tag`, drawn afresh, so
/// the swapped run's selector finds none of the first run's rows.
#[test]
fn a_swapped_run_keeps_an_input_bound_to_the_caller_that_a_selector_reads() {
    let synthesis = passes("issue 805", MODEL, 4);
    let unbound = unbound(&synthesis, "writer", "not-yours");
    assert!(
        unbound.is_empty(),
        "a step past `not-yours` sends `writer` as its caller's principal:\n{}",
        unbound.join("\n")
    );
    every_run_kept("issue 805", &synthesis);
}

/// Control: the selector does not read `writer`, so the swapped run never drew it.
#[test]
fn control_a_selector_not_reading_the_caller_bound_input_passes() {
    let text = edited(&[(SELECTOR, r#"where: "tag == input.tag""#)]);
    let synthesis = passes("selector without writer", &text, 4);
    assert_eq!(
        unbound(&synthesis, "writer", "not-yours"),
        Vec::<String>::new()
    );
}

/// Control: no guard reads the caller, so a fresh `writer` decides nothing.
#[test]
fn control_no_caller_guard_passes() {
    let text = edited(&[(
        &format!("      - name: not-yours\n        {GUARD}\n        error: depot.slips.NotYours\n"),
        "",
    )]);
    passes("no caller guard", &text, 3);
}

/// The selector reads only the caller-bound input: no other input it reads is drawn afresh.
///
/// A scenario whose first run sends `PostSlip` as both callers (the mixed-caller witnesses of
/// `duplicate`, `not-yours` and `posted`) leaves rows written by each caller, so a swapped run
/// keeping its caller's `writer` would select a row the first run wrote: it runs once, and the
/// note names that selector rather than claiming `String` has too few values. `tag-taken` sends
/// every step as one caller, so its swapped run sends the other caller's `principal`, a value the
/// first run never sends, and keeps that run.
#[test]
fn a_selector_reading_only_the_caller_bound_input_passes() {
    let text = edited(&[(SELECTOR, r#"where: "writer == input.writer""#)]);
    let synthesis = passes("selector on writer alone", &text, 4);
    assert_eq!(
        unbound(&synthesis, "writer", "not-yours"),
        Vec::<String>::new()
    );
    let once = run_once(&synthesis);
    for id in &SCENARIOS[..3] {
        let note = once
            .get(*id)
            .unwrap_or_else(|| panic!("`{id}` runs once: {once:#?}"));
        assert!(!appended(&synthesis, id), "`{id}` runs once");
        assert!(
            !note.contains("too few values") && note.contains("`writer == input.writer`"),
            "`{id}`: the note names the selector that would find the first run's row: {note}"
        );
    }
    let tag_taken = SCENARIOS[3];
    assert!(
        !once.contains_key(tag_taken) && appended(&synthesis, tag_taken),
        "`{tag_taken}` keeps its swapped run: {once:#?}"
    );
}

/// The guard written with `==`: the refusal is the caller's own `writer`, every other branch is
/// sent one that is not the caller's.
#[test]
fn a_guard_with_equality_passes() {
    let text = edited(&[(GUARD, r#"when: "writer == caller.principal""#)]);
    let synthesis = passes("guard with ==", &text, 4);
    every_run_kept("guard with ==", &synthesis);
}

/// The scenarios of `PostSlip` a swapped run is appended to in the `FileSlip` models below.
///
/// Not `not-yours`, which synthesis refuses there (`its row set is True`), nor `duplicate`, whose
/// mixed-caller witness sends its arranging `posted` step as one caller with the other caller's
/// `writer` before any run is appended: both as in 0.97.0, and neither the swapped run's draw.
const SWAPPED_IN_FILING: [&str; 2] = [
    "depot.slips.PostSlip/outcome/posted",
    "depot.slips.PostSlip/outcome/tag-taken",
];

/// The selector reading the caller-bound input refuses where no row matches (`exists: false`):
/// the matching row is arranged by `FileSlip`, which no guard reads.
#[test]
fn a_selector_refusing_where_no_row_matches_passes() {
    let edit = (
        format!("          {SELECTOR}\n          exists: true\n"),
        format!("          {SELECTOR}\n          exists: false\n"),
    );
    let text = edited(&with_filing(&[(&edit.0, &edit.1)]));
    named_pass("exists: false", &text, &SWAPPED_IN_FILING);
}

/// The guard written with the caller on the left.
#[test]
fn a_guard_with_the_caller_on_the_left_passes() {
    let text = edited(&[(GUARD, r#"when: "caller.principal != writer""#)]);
    let synthesis = passes("caller on the left", &text, 4);
    assert_eq!(
        unbound(&synthesis, "writer", "not-yours"),
        Vec::<String>::new()
    );
    every_run_kept("caller on the left", &synthesis);
}

/// Two inputs the selector reads, one bound to the caller and one free: the free one is still
/// drawn afresh, the bound one keeps its binding.
#[test]
fn a_free_input_beside_the_caller_bound_one_passes() {
    let text = edited(&[
        (
            "      - {name: tag, type: String}\n    outcomes:",
            "      - {name: tag, type: String}\n      - {name: shelf, type: String}\n    outcomes:",
        ),
        (
            SELECTOR,
            r#"where: {all: ["tag == input.tag", "writer == input.writer", "shelf == input.shelf"]}"#,
        ),
        (
            "      - {name: tag, type: String}\n    lifecycle:",
            "      - {name: tag, type: String}\n      - {name: shelf, type: String}\n    lifecycle:",
        ),
        (
            "          tag: input.tag\n",
            "          tag: input.tag\n          shelf: input.shelf\n",
        ),
    ]);
    let synthesis = passes("a free input beside", &text, 4);
    assert_eq!(
        unbound(&synthesis, "writer", "not-yours"),
        Vec::<String>::new()
    );
    every_run_kept("a free input beside", &synthesis);
}

/// A `forall:` test reading the caller-bound input, which `selected` serves like a selector: the
/// rows are arranged by `FileSlip`, which no guard reads.
#[test]
fn a_forall_test_reading_the_caller_bound_input_passes() {
    let from = format!("          {SELECTOR}\n          exists: true\n");
    let text = edited(&with_filing(&[(
        &from,
        "          where: \"tag == input.tag\"\n          forall: writer != input.writer\n",
    )]));
    named_pass("forall", &text, &SWAPPED_IN_FILING);
}
