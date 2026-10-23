//! An append to a stored list, `sets: {tags: {append: input.tag}}` (`ess/31`), in the generated
//! Rust and Go behaviours: the commands stay generated, the row they write holds a copy of the
//! previous list (an absent optional one read as empty) with the input element last, and the Go
//! module builds.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

const JOBS: &str = r"
format: ess/31
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: tags, type: List<String>}
      - {name: notes, type: Optional<List<String>>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
events:
  - name: demo.jobs.Changed
    fields: [{name: job_id, type: demo.jobs.JobId}]
errors:
  - name: demo.jobs.NoSuchJob
commands:
  - name: demo.jobs.AddJob
    input: [{name: tags, type: List<String>}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {tags: input.tags}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: {generated: true}}}
  - name: demo.jobs.AddTag
    input: [{name: job_id, type: demo.jobs.JobId}, {name: tag, type: String}]
    outcomes:
      - name: tagged
        updates: demo.jobs.Job
        instance: job_id
        sets: {tags: {append: input.tag}}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: input.job_id}}
      - {name: unknown, unknown_instance: true, error: demo.jobs.NoSuchJob}
  - name: demo.jobs.AddNote
    input: [{name: job_id, type: demo.jobs.JobId}, {name: note, type: String}]
    outcomes:
      - name: noted
        updates: demo.jobs.Job
        instance: job_id
        sets: {notes: {append: input.note}}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: input.job_id}}
      - {name: unknown, unknown_instance: true, error: demo.jobs.NoSuchJob}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: tags, type: List<String>}
      - {name: notes, type: Optional<List<String>>}
components:
  - component: jobs-service
    owns: {domains: [demo.jobs]}
    accepts: {commands: [demo.jobs.AddJob, demo.jobs.AddTag, demo.jobs.AddNote]}
    publishes: {events: [demo.jobs.Changed]}
    reached_by: network
";

fn model() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(JOBS).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// Every artifact of `target`'s synthesis, by path, with both appending behaviours generated.
fn artifacts(target: Target) -> BTreeMap<String, String> {
    let synthesis = synthesize_for(&model(), target).expect("a realizable target");
    for command in ["demo.jobs.AddTag", "demo.jobs.AddNote"] {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, command),
            Some(&SynthesisDisposition::Generated),
            "{target:?}: the behaviour of {command} is generated"
        );
    }
    synthesis
        .artifacts
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

fn artifact<'a>(artifacts: &'a BTreeMap<String, String>, path: &str) -> &'a str {
    artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    })
}

/// The Go `artifacts`, written under a fresh directory and built offline; the log, and whether it
/// built. Go only: a test here spawns no cargo (`ess-xtask/tests/no_test_builds.rs`).
fn go_build(artifacts: BTreeMap<String, String>) -> (String, bool) {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("append-set-go-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, contents) in artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    let output = Command::new("go")
        .args(["build", "-buildvcs=false", "./..."])
        .current_dir(&directory)
        .env("GOWORK", "off")
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    (log, output.status.success())
}

#[test]
fn generated_rust_appends_to_the_previous_list() {
    let rust = artifacts(Target::Rust);
    let behaviour = artifact(&rust, "crates/demo-types/src/behaviour.rs");
    assert!(
        behaviour.contains(".clone(); items.push(") && behaviour.contains("items }"),
        "the Rust row holds the previous tags with the element last:\n{behaviour}"
    );
    assert!(
        behaviour.contains(".clone().unwrap_or_default(); items.push(")
            && behaviour.contains("Some(items) }"),
        "an absent optional list reads as empty:\n{behaviour}"
    );
}

#[test]
fn generated_go_appends_to_a_fresh_slice_and_builds() {
    let go = artifacts(Target::Go);
    let behaviour = artifact(&go, "types/behaviour/behaviour.go");
    assert!(
        behaviour.contains("append(append([]string{}, "),
        "the Go row holds a fresh slice with the element last:\n{behaviour}"
    );
    assert!(
        behaviour.contains("items := []string{}; if "),
        "an absent optional list reads as empty:\n{behaviour}"
    );
    let (log, built) = go_build(go);
    assert!(built, "{log}");
}
