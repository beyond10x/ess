//! The interpreted target answers an identity no record carries with the command's declared
//! not-found refusal before `wrong_state` (beyond10x/ess#291): an `external:` refusal naming no
//! subject whose error carries the identity's type, which synthesis reads as the not-found answer
//! and sends a never-created identity for.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{synthesize::synthesize, AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = "format: ess/18
system: demo
version: v1
domain: demo.sessions
types:
  - {name: demo.sessions.SessionId, kind: newtype, of: Uuid}
entities:
  - name: demo.sessions.Session
    identity: {name: session_id, type: demo.sessions.SessionId}
    fields: []
    lifecycle:
      initial: Active
      states: [Active, Ended]
      terminal: [Ended]
      transitions:
        - {name: end, from: [Active], to: Ended}
errors:
  - name: demo.sessions.NoSession
    summary: No session carries the identity.
    fields:
      - {name: session_id, type: demo.sessions.SessionId}
  - {name: demo.sessions.NotActive, summary: The session has ended., fields: []}
events:
  - name: demo.sessions.SessionStarted
    fields:
      - {name: session_id, type: demo.sessions.SessionId}
  - name: demo.sessions.SessionEnded
    fields:
      - {name: session_id, type: demo.sessions.SessionId}
actors:
  - name: demo.sessions.User
    may: [demo.sessions.StartSession, demo.sessions.EndSession]
commands:
  - name: demo.sessions.StartSession
    input: []
    outcomes:
      - name: started
        creates: demo.sessions.Session
        instance: session_id
        emits: [demo.sessions.SessionStarted]
        payload:
          demo.sessions.SessionStarted: {session_id: {generated: true}}
  - name: demo.sessions.EndSession
    input:
      - {name: session_id, type: demo.sessions.SessionId}
    outcomes:
      - name: ended
        moves: demo.sessions.Session.end
        instance: session_id
        emits: [demo.sessions.SessionEnded]
        payload:
          demo.sessions.SessionEnded: {session_id: input.session_id}
      - name: no-session
        external: the session store holds no session with this id
        error: demo.sessions.NoSession
      - name: not-active
        wrong_state: true
        error: demo.sessions.NotActive
views:
  - name: demo.sessions.Sessions
    source: demo.sessions.Session
    consistency: read_your_writes
    fields:
      - {name: session_id, type: demo.sessions.SessionId}
      - {name: state, type: demo.sessions.Session.State}
";

const NO_SESSION: &str = "demo.sessions.EndSession/outcome/no-session";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("demo.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The synthesized not-found scenario passes against the interpreted target, and no scenario of
/// the suite fails.
#[test]
fn issue_291_the_interpreter_answers_the_declared_not_found_refusal() {
    let model = ir();
    let suite = synthesize(&model).suite;
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let statuses: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .map(|run| {
            (
                run.scenario.to_string(),
                format!("{:?} {:?}", run.status, run.checks),
            )
        })
        .collect();
    let found = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == NO_SESSION)
        .unwrap_or_else(|| panic!("{NO_SESSION} is synthesized: {statuses:#?}"));
    assert_eq!(found.status, Status::Passed, "{statuses:#?}");
    for run in &report.scenarios {
        assert_eq!(
            run.status,
            Status::Passed,
            "{}: {statuses:#?}",
            run.scenario
        );
    }
}
