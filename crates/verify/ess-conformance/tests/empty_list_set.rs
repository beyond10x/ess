//! A required list a view publishes, filled by its creator with `sets: {tags: []}` (`ess/31`).
//!
//! An adopter's view published a required `List<…>` field no command could set, since a list
//! literal in `sets:` was refused: the interpreted target answered the view `unsupported` for every
//! row. From `ess/31` the creator stores the empty list, the interpreted target holds it, and the
//! synthesized suite passes on it. A model that does not use it compiles to the bytes it had.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted,
    report::Status,
    target::{ConformanceTarget, Deadline, SemanticCommandRequest, SemanticViewRequest},
    AdmittedSuite, Runner,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A job whose creator stores `tags` from `tags_source`, and a view publishing the list.
fn jobs(format: &str, tags_source: &str, input: &str) -> String {
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
      - {{name: label, type: String}}
      - {{name: tags, type: List<String>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.JobAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.AddJob
    input: [{{name: label, type: String}}{input}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {{label: input.label, tags: {tags_source}}}
        emits: [demo.jobs.JobAdded]
        payload: {{demo.jobs.JobAdded: {{job_id: {{generated: true}}}}}}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {{name: job_id, type: demo.jobs.JobId}}
      - {{name: label, type: String}}
      - {{name: tags, type: List<String>}}
"
    )
}

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn a_row_created_with_an_empty_list_fills_the_view() {
    let ir = model(&jobs("ess/31", "[]", ""));

    // The whole synthesized suite passes on the interpreted target: nothing is unsupported.
    let synthesis = ess_conformance::synthesize(&ir);
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().contains("demo.jobs.AddJob")),
        "the creator is witnessed: {:?}",
        synthesis.refusals
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

    // And the row the creator wrote reads back with the empty list.
    let target = Interpreted::for_model(ir);
    let correlation = CorrelationId::new("jobs").unwrap();
    target
        .execute_command(SemanticCommandRequest {
            command: "demo.jobs.AddJob".parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([("label".into(), Node::Text("first".into()))]),
            correlation: correlation.clone(),
        })
        .unwrap();
    let rows = target
        .query_view(SemanticViewRequest {
            view: "demo.jobs.Jobs".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows;
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["tags"], Node::Seq(Vec::new()), "{rows:?}");
    assert_eq!(rows[0]["label"], Node::Text("first".into()), "{rows:?}");
}

/// The IR of a model that fills `tags` from its input at `ess/30`, as compiled before `ess/31`
/// existed: its SHA-256 and length.
const BEFORE_ESS_31: (&str, usize) = (
    "972cdbd33e827bca61b55865a0c2491336d58bd04f8829bf26d2d0692a1e0dc9",
    5955,
);

#[test]
fn models_without_it_keep_their_ir_bytes() {
    let input = ", {name: tags, type: List<String>}";
    let at_30 = model(&jobs("ess/30", "input.tags", input)).to_canonical_json();
    let digest: String =
        Sha256::digest(at_30.as_bytes())
            .iter()
            .fold(String::new(), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            });
    assert_eq!(
        (digest.as_str(), at_30.len()),
        BEFORE_ESS_31,
        "the ess/30 IR keeps its bytes:\n{at_30}"
    );
    // The IR does not name the source format, so at ess/31 the same model is the same bytes.
    let at_31 = model(&jobs("ess/31", "input.tags", input)).to_canonical_json();
    assert_eq!(at_31, at_30);
}
