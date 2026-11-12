//! `story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path`.
//!
//! A branch's `when:` guard on the identity input of the row a command addresses is decided on
//! the identity of the row sent, as the subject-addressed paths decide it
//! (`story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent`). Two paths sent a
//! row created under a placeholder identity and so wrote scenarios the reference interpreter
//! fails: `addressed_row`, which arranges the row a complete input-guarded refusal (or a row-set
//! accepting branch) is sent for, and `prepare_state_input`, which arranges the held state of a
//! branch beside subject-state guards. Every model is inline and neutral.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

/// A pass is opened by its identity; `ClosePass` refuses the pass `"frozen"` as a complete
/// refusal, and otherwise closes (deletes) it.
const COMPLETE: &str = r#"format: ess/36
system: demo
version: v1
domain: demo.passes
summary: A pass is opened and closed; the pass frozen cannot be closed.
types:
  - {name: demo.passes.PassId, kind: newtype, of: String}
entities:
  - name: demo.passes.Pass
    identity: {name: pass_id, type: demo.passes.PassId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
views:
  - name: demo.passes.Passes
    source: demo.passes.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
actors:
  - name: demo.passes.User
    may: [demo.passes.OpenPass, demo.passes.ClosePass]
errors:
  - name: demo.passes.Frozen
    fields:
      - {name: pass_id, type: demo.passes.PassId}
commands:
  - name: demo.passes.OpenPass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassOpened]
        payload:
          demo.passes.PassOpened: {pass_id: input.pass_id}
  - name: demo.passes.ClosePass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: reason, type: String}
    outcomes:
      - name: frozen
        when: 'pass_id == "frozen"'
        error: demo.passes.Frozen
        payload:
          demo.passes.Frozen: {pass_id: input.pass_id}
      - name: closed
        deletes: demo.passes.Pass
        instance: pass_id
        emits: [demo.passes.PassClosed]
        payload:
          demo.passes.PassClosed: {pass_id: input.pass_id}
events:
  - name: demo.passes.PassOpened
    fields:
      - {name: pass_id, type: demo.passes.PassId}
  - name: demo.passes.PassClosed
    fields:
      - {name: pass_id, type: demo.passes.PassId}
"#;

/// The same entity with a lifecycle; `ClosePass` refuses every pass but `"main"`, then closes an
/// open pass and refuses a closed one.
const STATE: &str = r#"format: ess/36
system: demo
version: v1
domain: demo.passes
summary: A pass is opened and closed; only the pass main can be closed.
types:
  - {name: demo.passes.PassId, kind: newtype, of: String}
entities:
  - name: demo.passes.Pass
    identity: {name: pass_id, type: demo.passes.PassId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
views:
  - name: demo.passes.Passes
    source: demo.passes.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: state, type: demo.passes.Pass.State}
      - {name: note, type: String}
actors:
  - name: demo.passes.User
    may: [demo.passes.OpenPass, demo.passes.ClosePass]
errors:
  - name: demo.passes.Frozen
    fields:
      - {name: pass_id, type: demo.passes.PassId}
  - name: demo.passes.NotOpen
    fields:
      - {name: pass_id, type: demo.passes.PassId}
commands:
  - name: demo.passes.OpenPass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassOpened]
        payload:
          demo.passes.PassOpened: {pass_id: input.pass_id}
  - name: demo.passes.ClosePass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: reason, type: String}
    outcomes:
      - name: frozen
        when: 'pass_id != "main"'
        error: demo.passes.Frozen
        payload:
          demo.passes.Frozen: {pass_id: input.pass_id}
      - name: closed
        when_subject_state: Open
        moves: demo.passes.Pass.close
        instance: pass_id
        emits: [demo.passes.PassClosed]
        payload:
          demo.passes.PassClosed: {pass_id: input.pass_id}
      - name: not-open
        when_subject_state: Closed
        error: demo.passes.NotOpen
        payload:
          demo.passes.NotOpen: {pass_id: input.pass_id}
events:
  - name: demo.passes.PassOpened
    fields:
      - {name: pass_id, type: demo.passes.PassId}
  - name: demo.passes.PassClosed
    fields:
      - {name: pass_id, type: demo.passes.PassId}
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

struct Run {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run(text: &str) -> Run {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Run { synthesis, results }
}

/// The scenario `wanted` as the suite serializes it, where one is written.
fn written(run: &Run, wanted: &str) -> Option<Value> {
    run.synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .map(|(_, scenario)| serde_json::to_value(scenario).unwrap())
}

/// Every scenario passes on the interpreted target.
fn assert_all_pass(run: &Run) {
    let failed: Vec<String> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| {
            format!(
                "{id}: {:?} {:#?}",
                result.status,
                result.diagnostics().collect::<Vec<_>>()
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes on the interpreted target:\n{}",
        failed.join("\n")
    );
}

/// The identity literals every `OpenPass` step of the scenario `wanted` sends, in order.
fn opened(run: &Run, wanted: &str) -> Vec<String> {
    let scenario = written(run, wanted).unwrap_or_else(|| panic!("{wanted} is written"));
    let steps: Vec<String> = scenario["steps"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|step| {
            step["step"] == "execute_command" && step["command"] == "demo.passes.OpenPass"
        })
        .filter_map(|step| {
            step["input"]["pass_id"]["value"]
                .as_str()
                .map(str::to_owned)
        })
        .collect();
    assert!(!steps.is_empty(), "{wanted} opens a pass: {scenario:#}");
    steps
}

#[test]
fn addressed_row_sends_the_row_its_identity_guard_selects() {
    let run = run(COMPLETE);
    assert_eq!(
        opened(&run, "demo.passes.ClosePass/outcome/frozen"),
        ["frozen"],
        "the refusal's scenario opens the pass its guard selects"
    );
    assert_all_pass(&run);
}

#[test]
fn prepare_state_input_sends_the_row_its_identity_guard_selects() {
    let run = run(STATE);
    let wrong: Vec<String> = [
        "demo.passes.ClosePass/outcome/closed",
        "demo.passes.ClosePass/outcome/not-open",
        "demo.passes.Pass/state/Closed/refuses/demo.passes.ClosePass",
        "demo.passes.Pass/transition/close/by/demo.passes.ClosePass/closed",
    ]
    .into_iter()
    .filter(|scenario| written(&run, scenario).is_none_or(|_| opened(&run, scenario) != ["main"]))
    .map(|scenario| {
        format!(
            "{scenario}: {:?}",
            written(&run, scenario).map(|_| opened(&run, scenario))
        )
    })
    .collect();
    assert!(
        wrong.is_empty(),
        "each scenario opens the pass its guard selects:\n{}",
        wrong.join("\n")
    );
    assert_all_pass(&run);
}
