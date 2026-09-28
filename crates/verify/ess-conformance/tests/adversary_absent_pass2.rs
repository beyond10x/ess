//! Adversary pass 2 against `input_absent:` (beyond10x/ess#170): the new
//! `execute_command_without_input` step through admission, replay validation, coverage and
//! synthesis beside `unknown_instance:`.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{AdmittedSuite, ConformanceSuite, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const ABSENT: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/absent-input.yaml");
const REPLAY: &str = include_str!("fixtures/retained-replay.yaml");

fn ir_of(name: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new(name), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

/// The synthesized retained-replay scenario, pinned at suite/26 where the new step is admitted.
fn replay_suite_at_26() -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize::synthesize(&ir_of("replay.yaml", REPLAY)).suite;
    suite
        .scenarios
        .retain(|id, _| id.to_string().ends_with("/outcome/replayed"));
    suite.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/26").unwrap();
    suite
}

fn steps_of(suite: &mut ConformanceSuite) -> &mut Vec<ScenarioStep> {
    &mut suite.scenarios.values_mut().next().unwrap().steps
}

fn without_input_of_retry(steps: &[ScenarioStep], before: usize) -> ScenarioStep {
    let (command, actor) = steps[..before]
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, actor, .. } => {
                Some((command.clone(), actor.clone()))
            }
            _ => None,
        })
        .expect("a command runs before");
    ScenarioStep::ExecuteCommandWithoutInput {
        command,
        actor,
        caller: std::collections::BTreeMap::new(),
    }
}

/// Replay admission refuses a second invocation between the retry and the replay comparison
/// ("retry substituted invocation"). The input-less invocation is an invocation too: the runner
/// compares *its* result against the capture, so admitting it lets a suite claim a replay of the
/// original request while comparing a different request.
#[test]
fn an_input_less_invocation_cannot_stand_in_for_the_replayed_retry() {
    let suite = replay_suite_at_26();
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| {
        panic!("precondition: the replay suite is admitted at /26: {error}")
    });

    let at = suite
        .scenarios
        .values()
        .next()
        .unwrap()
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExpectReplayResult { .. }))
        .expect("the scenario compares a replay");

    // Control: a second `execute_command` there is refused today.
    let mut control = suite.clone();
    let steps = steps_of(&mut control);
    let retry = steps[..at]
        .iter()
        .rev()
        .find(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .unwrap()
        .clone();
    steps.insert(at, retry);
    assert!(
        AdmittedSuite::from_suite(&control).is_err(),
        "precondition: a substituted `execute_command` before the replay comparison is refused"
    );

    let mut bad = suite.clone();
    let steps = steps_of(&mut bad);
    let step = without_input_of_retry(steps, at);
    steps.insert(at, step);
    assert!(
        AdmittedSuite::from_suite(&bad).is_err(),
        "an `execute_command_without_input` between the retry and `expect_replay_result` is \
         admitted, so the replay compares the input-less request's result"
    );
}

/// The same substitution on the original side: an input-less invocation between the original
/// `execute_command` and `capture_command_result` makes the capture hold the input-less result,
/// while `validate_bindings` still binds it to the original invocation's command, outcome and input.
#[test]
fn an_input_less_invocation_cannot_stand_in_for_the_captured_original() {
    let suite = replay_suite_at_26();
    let at = suite
        .scenarios
        .values()
        .next()
        .unwrap()
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::CaptureCommandResult { .. }))
        .expect("the scenario captures the original");
    let mut bad = suite.clone();
    let steps = steps_of(&mut bad);
    let step = without_input_of_retry(steps, at);
    steps.insert(at, step);
    assert!(
        AdmittedSuite::from_suite(&bad).is_err(),
        "an `execute_command_without_input` between the original invocation and \
         `capture_command_result` is admitted"
    );
}

fn absent_suite() -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir_of("absent-input.yaml", ABSENT)).suite
}

/// The JSON reader, not only `from_suite`: a /25 document carrying the step is refused before it
/// is deserialized, and a /26 step carrying an `input` key (even `{}`) is refused as unknown.
#[test]
fn the_json_reader_refuses_the_step_below_26_and_an_input_on_it() {
    let json = absent_suite().to_canonical_json().unwrap();
    assert!(json.contains("\"ess-conformance/26\""), "{json}");
    AdmittedSuite::from_json(&json).unwrap_or_else(|error| panic!("{error}"));
    for older in 1..26 {
        assert!(
            AdmittedSuite::from_json(
                &json.replace("ess-conformance/26", &format!("ess-conformance/{older}"))
            )
            .is_err(),
            "ess-conformance/{older} carrying the step is admitted"
        );
    }
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let mut patched = false;
    for scenario in value["scenarios"]
        .as_object_mut()
        .into_iter()
        .flat_map(|s| s.values_mut())
    {
        for step in scenario["steps"].as_array_mut().into_iter().flatten() {
            if step["step"] == "execute_command_without_input" {
                step["input"] = serde_json::json!({});
                patched = true;
            }
        }
    }
    assert!(patched, "the step is in the canonical JSON: {json}");
    assert!(
        AdmittedSuite::from_json(&serde_json::to_string(&value).unwrap()).is_err(),
        "an input on the input-less step is admitted"
    );
}

/// Coverage for the fixture is ess-conformance/27, round-trips through the JSON reader, and
/// carries the absent-input scenario.
#[test]
fn coverage_for_the_fixture_is_27_and_carries_the_scenario() {
    let input = ess_conformance::coverage_build::build(
        &ir_of("absent-input.yaml", ABSENT),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let original = input.selected().original_json().to_owned();
    assert!(original.contains("\"ess-conformance/27\""), "{original}");
    assert!(
        original.contains("execute_command_without_input"),
        "{original}"
    );
    AdmittedSuite::from_json(&original).unwrap_or_else(|error| panic!("{error}"));
    assert!(AdmittedSuite::from_json(
        &original.replace("ess-conformance/27", "ess-conformance/25")
    )
    .is_err());
}

/// `input_absent:` beside `unknown_instance:` on a command that names an existing record: both
/// branches synthesize, the absent one sends no input and the unknown-instance one sends one.
#[test]
fn input_absent_beside_unknown_instance_synthesizes_both() {
    let model = ABSENT
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.notes.NoteUnknown\n    summary: No such note.\n    fields: []\n",
        )
        .replace(
            "      - name: reworded\n",
            "      - name: reword-body-missing\n        input_absent: true\n        error: demo.notes.BodyMissing\n      - name: no-such-note\n        unknown_instance: true\n        error: demo.notes.NoteUnknown\n      - name: reworded\n",
        );
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of("absent-input.yaml", &model));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    let find = |id: &str| {
        synthesis
            .suite
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == id)
            .map_or_else(
                || {
                    panic!(
                        "no scenario {id}: {:#?}",
                        synthesis.suite.scenarios.keys().collect::<Vec<_>>()
                    )
                },
                |(_, scenario)| scenario,
            )
    };
    let absent = find("demo.notes.RewordNote/outcome/reword-body-missing");
    assert!(matches!(
        absent.steps[0],
        ScenarioStep::ExecuteCommandWithoutInput { .. }
    ));
    assert!(!absent
        .steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::ExecuteCommand { .. })));
    let unknown = find("demo.notes.RewordNote/outcome/no-such-note");
    assert!(unknown
        .steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::ExecuteCommand { .. })));
    assert!(!unknown
        .steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::ExecuteCommandWithoutInput { .. })));
}

/// Pass 2 found validation admitting `missing(body.text)` over a required struct input with a
/// required `text` while synthesis could not witness the branch (the #170 disagreement one level
/// down). Coordinator decision, correction round 2: validation refuses it from ess/16, so the model
/// that was evidence of the gap is now refused before synthesis — asserted here, so the gap cannot
/// reopen at either end.
#[test]
fn a_missing_over_a_required_nested_field_is_refused_before_synthesis() {
    let model = "format: ess/16
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - name: demo.notes.Body
    kind: struct
    fields:
      - {name: text, type: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - {name: demo.notes.Service, may: [demo.notes.SubmitNote]}
errors:
  - name: demo.notes.Refused
    summary: Refused.
    fields: []
commands:
  - name: demo.notes.SubmitNote
    input:
      - {name: body, type: demo.notes.Body}
    outcomes:
      - name: refused
        when: missing(body.text)
        error: demo.notes.Refused
      - name: submitted
        creates: demo.notes.Note
        instance: note_id
        emits: [demo.notes.NoteSubmitted]
        payload:
          demo.notes.NoteSubmitted: {note_id: {generated: true}}
events:
  - name: demo.notes.NoteSubmitted
    fields:
      - {name: note_id, type: demo.notes.NoteId}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
";
    let raw = RawSpecFile::parse(model).expect("the model parses");
    let errors = Specification::assemble([(Source::new("nested.yaml"), raw)])
        .err()
        .map(|errors| errors.to_string())
        .unwrap_or_default();
    assert!(
        errors.contains("type_mismatch") && errors.contains("body.text"),
        "`missing(body.text)` over a required nested field is admitted at ess/16 (story:absent-command-input-outcome): {errors}"
    );
}
