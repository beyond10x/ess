//! `sets: {tags: {append: input.tag}}` (`ess/31`): an outcome storing the list its subject held
//! before it, absent read as empty, with one input element last.
//!
//! An adopter's command added one element to a stored list and could not say so; a target that
//! dropped the earlier elements passed. From `ess/31` the interpreted target appends, the
//! synthesized suite asserts the whole list the row holds afterwards, and a model that does not
//! use the source compiles to the bytes it had.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted,
    report::Status,
    target::{
        ConformanceTarget, Deadline, EntitySetupRequest, ScenarioContext, SemanticCommandRequest,
        SemanticViewRequest, ViewRow,
    },
    AdmittedSuite, Runner,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const ID: &str = "00000000-0000-4000-8000-000000000001";

/// A job opened with a required list and holding an optional one, grown by `AddTag` and
/// `AddNote` from `tag_source` and `note_source`.
fn jobs(format: &str, tag_source: &str, note_source: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
entities:
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: tags, type: List<String>}}
      - {{name: notes, type: Optional<List<String>>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.Changed
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.OpenJob
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: opened
        creates: demo.jobs.Job
        instance: job_id
        sets: {{tags: input.tags}}
        emits: [demo.jobs.Changed]
        payload: {{demo.jobs.Changed: {{job_id: input.job_id}}}}
  - name: demo.jobs.AddTag
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: tag, type: String}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: tagged
        updates: demo.jobs.Job
        instance: job_id
        sets: {{tags: {tag_source}}}
        emits: [demo.jobs.Changed]
        payload: {{demo.jobs.Changed: {{job_id: input.job_id}}}}
  - name: demo.jobs.AddNote
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: note, type: String}}, {{name: notes, type: List<String>}}]
    outcomes:
      - name: noted
        updates: demo.jobs.Job
        instance: job_id
        sets: {{notes: {note_source}}}
        emits: [demo.jobs.Changed]
        payload: {{demo.jobs.Changed: {{job_id: input.job_id}}}}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {{name: job_id, type: demo.jobs.JobId}}
      - {{name: tags, type: List<String>}}
      - {{name: notes, type: Optional<List<String>>}}
"
    )
}

fn appending() -> String {
    jobs("ess/31", "{append: input.tag}", "{append: input.note}")
}

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn correlation() -> CorrelationId {
    CorrelationId::new("jobs").unwrap()
}

fn texts(items: &[&str]) -> Node {
    Node::Seq(
        items
            .iter()
            .map(|item| Node::Text((*item).into()))
            .collect(),
    )
}

fn run(target: &Interpreted, command: &str, field: &str, value: &str) {
    target
        .execute_command(SemanticCommandRequest {
            command: command.parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([
                ("job_id".into(), Node::Text(ID.into())),
                (field.into(), Node::Text(value.into())),
                // The whole list the ess/30 form stores; the append ignores it.
                (format!("{field}s"), Node::Seq(Vec::new())),
            ]),
            correlation: correlation(),
        })
        .unwrap_or_else(|error| panic!("{command}: {error:?}"));
}

fn row(target: &Interpreted) -> ViewRow {
    let rows = target
        .query_view(SemanticViewRequest {
            view: "demo.jobs.Jobs".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows;
    assert_eq!(rows.len(), 1, "{rows:?}");
    rows[0].clone()
}

#[test]
fn the_interpreter_appends_the_input_last() {
    let target = Interpreted::for_model(model(&appending()));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.jobs/authored/append".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    target
        .establish_entity(EntitySetupRequest {
            entity: "demo.jobs.Job".parse().unwrap(),
            identity: Node::Text(ID.into()),
            fields: BTreeMap::from([("tags".into(), texts(&["first"]))]),
            state: "Open".parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();

    run(&target, "demo.jobs.AddTag", "tag", "second");
    run(&target, "demo.jobs.AddTag", "tag", "third");
    assert_eq!(row(&target)["tags"], texts(&["first", "second", "third"]));

    // An absent optional list reads as empty: the first append stores a list of one.
    run(&target, "demo.jobs.AddNote", "note", "seen");
    assert_eq!(row(&target)["notes"], texts(&["seen"]));
    run(&target, "demo.jobs.AddNote", "note", "again");
    let after = row(&target);
    assert_eq!(after["notes"], texts(&["seen", "again"]));
    assert_eq!(after["tags"], texts(&["first", "second", "third"]));
}

#[test]
fn the_synthesized_suite_passes_on_the_interpreted_target() {
    let ir = model(&appending());
    let synthesis = ess_conformance::synthesize(&ir);
    for command in ["demo.jobs.AddTag", "demo.jobs.AddNote"] {
        assert!(
            synthesis
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string().contains(command)),
            "{command} is witnessed: {:?}",
            synthesis.refusals
        );
    }
    // The appending branch's scenario asserts the whole list the row holds afterwards.
    let suite = format!("{:?}", synthesis.suite);
    assert!(
        suite.contains("tags"),
        "the suite names the appended list:\n{suite}"
    );
    let input = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report =
        Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(ir.clone()));
    let unpassed: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert!(unpassed.is_empty(), "{unpassed:#?}");

    // A target that overwrites the list with the element alone fails the suite.
    let overwriting = model(&jobs("ess/31", "[]", "{cleared: true}"));
    let report =
        Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(overwriting));
    assert!(
        report
            .scenarios
            .iter()
            .any(|scenario| scenario.status == Status::Failed),
        "a target that drops the earlier elements fails"
    );
}

/// The IR of a model that sets both lists from its input at `ess/30`, as compiled before
/// `{append: …}` existed: its SHA-256 and length.
const BEFORE_APPEND: (&str, usize) = (
    "4c1fe8d0e2d72e14fa5ea3f31fa2359f7b0721c78ba1d84230c81724879c36c2",
    11462,
);

#[test]
fn models_without_it_keep_their_ir_bytes() {
    let at_30 = model(&jobs("ess/30", "input.tags", "input.notes")).to_canonical_json();
    let digest: String =
        Sha256::digest(at_30.as_bytes())
            .iter()
            .fold(String::new(), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            });
    assert_eq!(
        (digest.as_str(), at_30.len()),
        BEFORE_APPEND,
        "the ess/30 IR keeps its bytes:\n{at_30}"
    );
    let at_31 = model(&jobs("ess/31", "input.tags", "input.notes")).to_canonical_json();
    assert_eq!(at_31, at_30);
}
