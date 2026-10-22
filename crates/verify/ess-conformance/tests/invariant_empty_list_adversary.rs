//! Adversary pass on `sets: {field: []}` (ess/31): what a validated model whose creator stores the
//! empty list under an invariant reading the list does on the interpreted target.
//!
//! The property: a specification `ess-domain` admits synthesizes a suite its own interpreted
//! target passes. A creator writing `tags: []` beside `invariants: [tags.count >= 1]` can never
//! succeed; either validation refuses it, as it refuses a literal doing the same, or the suite says
//! nothing is wrong. Both cannot be true of a model no run can satisfy.

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn jobs(invariant: &str, tags: &str) -> String {
    format!(
        "format: ess/31
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
    invariants:
      - {invariant}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.JobAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.AddJob
    input: [{{name: label, type: String}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {{label: input.label, tags: {tags}}}
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

/// `None` when validation refuses the model; otherwise every scenario that did not pass, and
/// whether the creator was witnessed at all.
fn unpassed(text: &str) -> Option<(Vec<String>, bool, String)> {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)]).ok()?;
    let ir = compile(&spec, &SourceMap::new()).expect("a validated model compiles");
    let synthesis = ess_conformance::synthesize(&ir);
    let witnessed = synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string().contains("AddJob/outcome/added"));
    let input = AdmittedSuite::from_suite(&synthesis.suite).expect("admitted");
    let report = Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(ir));
    let unpassed = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect();
    Some((unpassed, witnessed, format!("{:?}", synthesis.refusals)))
}

/// Control: the same invariant over a list the input supplies validates, and the suite passes.
#[test]
fn adversary_control_an_input_list_under_the_invariant_passes() {
    let (unpassed, witnessed, refusals) =
        unpassed(&jobs("tags.count >= 1", "input.tags")).expect("the control validates");
    assert!(witnessed, "{refusals}");
    assert!(unpassed.is_empty(), "{unpassed:#?}");
}

/// `tags: []` under `tags.count >= 1`: refused at validation, or the suite holds no failure and
/// still witnesses the creator. A creator that cannot succeed and a suite that is green on it, or
/// that silently drops the creator, are both a gap.
#[test]
fn adversary_an_admitted_empty_list_creator_under_an_invariant_is_held_by_the_suite() {
    let text = jobs("tags.count >= 1", "[]");
    let Some((unpassed, witnessed, refusals)) = unpassed(&text) else {
        return;
    };
    panic!(
        "validation admitted a creator writing `tags: []` under `tags.count >= 1`, which no run \
         satisfies; the creator witnessed: {witnessed}; scenarios not passed on the interpreted \
         target: {unpassed:#?}; synthesis refusals: {refusals}"
    );
}
