//! Adversary pass on `sets: {field: []}` (ess/31) in the generated Go target, served: the model is
//! emitted, built as its generated network entry point and driven over a real socket by the suite
//! the same model synthesizes, so the view row is read back through Go's own encoder.
//!
//! The unit's own Go test asserts the text `Tags: []string{}` and that the module builds; nothing
//! there reads the row back. Go's `nil` slice and its empty slice are the two candidates for
//! `null` versus `[]`. Go only: a test here spawns no cargo.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use ess_synth::Target;

/// A job created with `tags: []` (or from `created`), retagged by an update writing `retagged`,
/// and a view publishing the list.
fn jobs(created: &str, retagged: &str) -> String {
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
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
errors:
  - {{name: demo.jobs.UnknownJob, summary: No job carries the identity., fields: []}}
events:
  - name: demo.jobs.JobAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
  - name: demo.jobs.JobRetagged
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.AddJob
    input: [{{name: label, type: String}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {{label: input.label, tags: {created}}}
        emits: [demo.jobs.JobAdded]
        payload: {{demo.jobs.JobAdded: {{job_id: {{generated: true}}}}}}
  - name: demo.jobs.Retag
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: unknown-job
        unknown_instance: true
        error: demo.jobs.UnknownJob
      - name: retagged
        updates: demo.jobs.Job
        instance: job_id
        sets: {{tags: {retagged}}}
        emits: [demo.jobs.JobRetagged]
        payload: {{demo.jobs.JobRetagged: {{job_id: input.job_id}}}}
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

/// The served Go target passes the suite on a creator and an update that store `[]`.
#[test]
fn adversary_served_go_passes_the_suite_on_empty_list_writes() {
    assert!(served::go_available(), "Go is required");
    let ir = served::model(&served::served(&jobs("[]", "[]"), "jobs"));
    let (root, _) = served::emit(&ir, Target::Go, "empty-list-adversary");
    let binary = served::build(&root, Target::Go, "demo", "jobs", false);
    let statuses = served::run(&ir, &binary);
    let _ = std::fs::remove_dir_all(&root);
    served::assert_passes(
        "Go empty list",
        &statuses,
        &["AddJob/outcome/added", "Retag/outcome/retagged"],
    );
}

/// Faulty control: the creator's `[]string{}` replaced by a one-element slice. The suite reads the
/// row back through the view and must fail it. (Go's `nil` is not a usable control: the served
/// view answers `"tags":[]` for a `nil` slice too, so that edit changes nothing observable.)
#[test]
fn adversary_served_go_suite_fails_a_creator_storing_a_stray_element() {
    assert!(served::go_available(), "Go is required");
    let ir = served::model(&served::served(&jobs("[]", "input.tags"), "jobs"));
    let (root, _) = served::emit(&ir, Target::Go, "empty-list-adversary-stray");
    served::mutate(
        &served::behaviour_path(&root, Target::Go, "demo"),
        "Tags: []string{}",
        "Tags: []string{\"stray\"}",
    );
    let binary = served::build(&root, Target::Go, "demo", "jobs", false);
    let statuses = served::run(&ir, &binary);
    let _ = std::fs::remove_dir_all(&root);
    served::assert_fails("Go creator storing a stray element", &statuses);
}
