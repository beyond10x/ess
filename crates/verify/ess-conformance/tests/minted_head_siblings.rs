//! A branch that reads the head of an ordered selection over a key the target mints is refused by
//! name, and takes no other branch of its command with it
//! (<https://github.com/beyond10x/ess/issues/552>, `docs/design/ordered-related-head.md`).
//!
//! Elements join a slot; each is scored by the target (`element_score: {generated: true}`). A pick
//! answers `empty` where the slot holds nothing, and otherwise pairs the best-scored element. No
//! scenario can steer a minted score, so the head is not arranged and `picked` is refused. `empty`
//! reads no head: it keeps its scenario.
//!
//! The issue's shape is [`decided_by_input`]: `empty` is the default after a `picked` its input
//! guards, and a binding follows `Paired`. Before the fix `empty` was routed through the row-set
//! arrangement only because `picked` reads a head, and refused there because a binding makes the
//! rows a scenario leaves unfollowable; with `picked` reading its key from the input instead, the
//! same model kept `empty`.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioId;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// The slot, with the score written as `score` on the creating command.
fn model(score: &str, input: &str) -> String {
    format!(
        "format: ess/27
system: demo
version: v1
domain: demo.slots
types:
  - {{name: demo.slots.ElementId, kind: newtype, of: Uuid}}
  - {{name: demo.slots.PairId, kind: newtype, of: Uuid}}
entities:
  - name: demo.slots.Element
    identity: {{name: element_id, type: demo.slots.ElementId}}
    fields:
      - {{name: slot_id, type: String}}
      - {{name: element_key, type: String}}
      - {{name: element_score, type: Integer}}
    lifecycle: {{initial: Waiting, states: [Waiting], terminal: [Waiting]}}
  - name: demo.slots.Pair
    identity: {{name: pair_id, type: demo.slots.PairId}}
    fields:
      - {{name: slot_id, type: String}}
      - {{name: element_key, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - {{name: demo.slots.Empty, summary: No element waits in the slot., fields: []}}
events:
  - name: demo.slots.Joined
    fields:
      - {{name: element_id, type: demo.slots.ElementId}}
  - name: demo.slots.Paired
    fields:
      - {{name: pair_id, type: demo.slots.PairId}}
      - {{name: element_key, type: String}}
views:
  - name: demo.slots.Pairs
    source: demo.slots.Pair
    consistency: read_your_writes
    fields:
      - {{name: pair_id, type: demo.slots.PairId}}
      - {{name: slot_id, type: String}}
      - {{name: element_key, type: String}}
commands:
  - name: demo.slots.Join
    input:
      - {{name: element_id, type: demo.slots.ElementId}}
      - {{name: slot_id, type: String}}
      - {{name: element_key, type: String}}
{input}    outcomes:
      - name: joined
        creates: demo.slots.Element
        instance: element_id
        sets:
          slot_id: input.slot_id
          element_key: input.element_key
          element_score: {score}
        emits: [demo.slots.Joined]
        payload:
          demo.slots.Joined: {{element_id: input.element_id}}
  - name: demo.slots.Pick
    input:
      - {{name: pair_id, type: demo.slots.PairId}}
      - {{name: slot_id, type: String}}
    outcomes:
      - name: empty
        when_related:
          entity: demo.slots.Element
          where: {{all: [slot_id == input.slot_id]}}
          exists: false
        error: demo.slots.Empty
      - name: picked
        creates: demo.slots.Pair
        instance: pair_id
        sets:
          slot_id: input.slot_id
          element_key:
            related:
              entity: demo.slots.Element
              where: {{all: [slot_id == input.slot_id]}}
              order_by: [element_score desc]
              ties: unordered
              first: element_key
        emits: [demo.slots.Paired]
        payload:
          demo.slots.Paired:
            pair_id: input.pair_id
            element_key:
              related:
                entity: demo.slots.Element
                where: {{all: [slot_id == input.slot_id]}}
                order_by: [element_score desc]
                ties: unordered
                first: element_key
"
    )
}

/// The score minted by the target.
fn minted() -> String {
    model("{generated: true}", "")
}

/// The score sent by the caller.
fn steered() -> String {
    model(
        "input.element_score",
        "      - {name: element_score, type: Integer}\n",
    )
}

/// [`minted`], with `empty` the default after a `picked` its input guards, and a binding that
/// notes each pair.
fn decided_by_input() -> String {
    let edits = [
        (
            "      - name: empty
        when_related:
          entity: demo.slots.Element
          where: {all: [slot_id == input.slot_id]}
          exists: false
        error: demo.slots.Empty
      - name: picked
",
            "      - name: picked\n        when: slot_id != \"\"\n",
        ),
        (
            "  - name: demo.slots.Paired\n",
            "  - name: demo.slots.Noted\n    fields:\n      - {name: pair_id, type: demo.slots.PairId}\n  - name: demo.slots.Paired\n",
        ),
    ];
    let mut text = minted();
    for (from, to) in edits {
        assert!(text.contains(from), "the edit lands: {from}");
        text = text.replacen(from, to, 1);
    }
    text.push_str(
        "      - {name: empty, error: demo.slots.Empty}
  - name: demo.slots.Notify
    input:
      - {name: pair_id, type: demo.slots.PairId}
    outcomes:
      - name: noted
        emits: [demo.slots.Noted]
        payload:
          demo.slots.Noted: {pair_id: input.pair_id}
bindings:
  - id: notify
    when: {event: demo.slots.Paired}
    invoke: {command: demo.slots.Notify}
    mapping:
      pair_id: event.pair_id
    delivery: at_least_once
    on_failure: retry
",
    );
    text
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("slots.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn id(value: &str) -> ScenarioId {
    value.parse().expect("a scenario id")
}

fn refusals(synthesis: &Synthesis) -> BTreeMap<String, String> {
    synthesis
        .refusals
        .iter()
        .filter_map(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(|scenario| (scenario.to_string(), refusal.to_string()))
        })
        .collect()
}

fn verdicts(ir: EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("the synthesized suite is admitted: {error}"));
    let target = Interpreted::for_model(ir);
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

const EMPTY: &str = "demo.slots.Pick/outcome/empty";
const PICKED: &str = "demo.slots.Pick/outcome/picked";

#[test]
fn a_branch_reading_no_head_keeps_its_scenario_beside_a_minted_head() {
    let ir = ir_of(&decided_by_input());
    let synthesis = synthesize(&ir);
    assert_empty_stands(ir, &synthesis);
    let refused = refusals(&synthesis);
    let text = refused
        .get(PICKED)
        .unwrap_or_else(|| panic!("the head-reading branch is refused by name: {refused:#?}"));
    assert!(text.contains("mints"), "{text}");
}

/// The empty side guarded by the rows themselves: it reads no head either, and stands.
#[test]
fn a_row_set_guarded_branch_keeps_its_scenario_beside_a_minted_head() {
    let ir = ir_of(&minted());
    let synthesis = synthesize(&ir);
    assert_empty_stands(ir, &synthesis);
}

fn assert_empty_stands(ir: EssIr, synthesis: &Synthesis) {
    let refused = refusals(synthesis);
    assert!(
        synthesis.suite.scenario(&id(EMPTY)).is_some(),
        "the empty side reads no head: {refused:#?}"
    );
    assert!(
        !refused.contains_key(EMPTY),
        "nothing refuses the empty side: {refused:#?}"
    );
    let verdicts = verdicts(ir, synthesis);
    assert_eq!(verdicts.get(EMPTY), Some(&Status::Passed), "{verdicts:#?}");
}

#[test]
fn a_head_over_a_minted_key_is_refused_by_name_toward_an_authored_scenario() {
    let synthesis = synthesize(&ir_of(&minted()));
    assert!(synthesis.suite.scenario(&id(PICKED)).is_none());
    let refused = refusals(&synthesis);
    let text = refused
        .get(PICKED)
        .unwrap_or_else(|| panic!("the head-reading branch is refused by name: {refused:#?}"));
    assert!(
        text.contains("element_score") && text.contains("mint"),
        "the refusal names the minted key: {text}"
    );
    assert!(
        text.contains("authored scenario"),
        "the help points at an authored scenario: {text}"
    );
}

/// The steered model is the control: both branches are synthesized, and both pass.
#[test]
fn a_steered_key_keeps_both_branches() {
    let ir = ir_of(&steered());
    let synthesis = synthesize(&ir);
    for scenario in [EMPTY, PICKED] {
        assert!(
            synthesis.suite.scenario(&id(scenario)).is_some(),
            "{scenario}: {:#?}",
            refusals(&synthesis)
        );
    }
    let verdicts = verdicts(ir, &synthesis);
    for scenario in [EMPTY, PICKED] {
        assert_eq!(
            verdicts.get(scenario),
            Some(&Status::Passed),
            "{verdicts:#?}"
        );
    }
}
