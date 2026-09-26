//! A `creates` outcome copies declared command input into the created entity's fields where its
//! `sets:` mapping says so, and a view filtered on such a field is decided from what was sent.
//!
//! The prerequisite beyond10x/ess#96 reports (SYNTH-005 on 26 of 39 refusals in a probe of a
//! per-session layer) and `docs/design/aggregate-views.md` builds on: aggregate scenarios choose
//! the creating command's input through its `sets:` mappings, which is the goal-directed choice of
//! `cross-record-and-stored-field-guards.md`.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{scenario::ViewExpectation, ScenarioStep, ScenarioValue};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SESSIONS: &str = "format: ess/7
system: metrics
version: v1
domain: metrics.session
types:
  - name: metrics.session.Ending
    kind: enum
    variants: [Answered, Abandoned]
entities:
  - name: metrics.session.Session
    identity: {name: session_id, type: Uuid}
    fields:
      - {name: ending, type: metrics.session.Ending}
      - {name: talk_seconds, type: Integer}
      - {name: note, type: Optional<String>}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: metrics.session.Opened
    fields:
      - {name: session_id, type: Uuid}
  - name: metrics.session.Closed
    fields: []
commands:
  - name: metrics.session.Open
    input:
      - {name: ending, type: metrics.session.Ending}
      - {name: talk_seconds, type: Integer}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: opened
        creates: metrics.session.Session
        instance: session_id
        sets: {ending: input.ending, talk_seconds: input.talk_seconds, note: input.note}
        emits: [metrics.session.Opened]
        payload:
          metrics.session.Opened:
            session_id: {generated: true}
  - name: metrics.session.Close
    input:
      - {name: session_id, type: Uuid}
    outcomes:
      - name: closed
        moves: metrics.session.Session.close
        instance: session_id
        emits: [metrics.session.Closed]
views:
  - name: metrics.session.AnsweredSessions
    source: metrics.session.Session
    consistency: read_your_writes
    filter:
      all:
        - ending == Answered
        - talk_seconds > 0
    fields:
      - {name: session_id, type: Uuid}
      - {name: ending, type: metrics.session.Ending}
      - {name: talk_seconds, type: Integer}
      - {name: note, type: Optional<String>}
";

#[test]
fn a_view_filtered_on_fields_the_create_copied_from_input_is_decided_without_synth_005() {
    let raw = RawSpecFile::parse(SESSIONS).unwrap();
    let spec = Specification::assemble([(Source::new("sessions.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        !result
            .refusals
            .iter()
            .any(|refusal| refusal.cause.code().to_string() == "ESS-SYNTH-005"),
        "{:?}",
        result.refusals
    );
    let (_, opened) = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "metrics.session.Open/outcome/opened")
        .unwrap();
    let ScenarioStep::ExecuteCommand { input, .. } = &opened.steps[0] else {
        panic!("{:?}", opened.steps)
    };
    let expectation = opened
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectView { expectation, .. } => Some(expectation.clone()),
            _ => None,
        })
        .expect("the filtered view is asserted");
    let ViewExpectation::Contains { fields } = &expectation else {
        panic!("the plain witness is admitted by the filter: {expectation:?}")
    };
    for copied in ["ending", "talk_seconds", "note"] {
        assert_eq!(
            fields.get(copied),
            input.get(copied),
            "{copied} is copied from input"
        );
    }
    assert!(
        input
            .get("note")
            .is_some_and(|note| note != &ScenarioValue::literal(Node::Null)),
        "{input:?}"
    );
}
