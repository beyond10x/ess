//! A creator storing the empty list, `sets: {tags: []}` (`ess/31`), in the generated Rust and Go
//! behaviours: the command stays generated, the row it writes holds an empty list (`Vec::new()`,
//! `[]string{}` in Go, never Go's `nil`, which encodes as `null`), and the Go module builds.

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
      - {name: label, type: String}
      - {name: tags, type: List<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
events:
  - name: demo.jobs.JobAdded
    fields: [{name: job_id, type: demo.jobs.JobId}]
commands:
  - name: demo.jobs.AddJob
    input: [{name: label, type: String}]
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {label: input.label, tags: []}
        emits: [demo.jobs.JobAdded]
        payload: {demo.jobs.JobAdded: {job_id: {generated: true}}}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: label, type: String}
      - {name: tags, type: List<String>}
components:
  - component: jobs-service
    owns: {domains: [demo.jobs]}
    accepts: {commands: [demo.jobs.AddJob]}
    publishes: {events: [demo.jobs.JobAdded]}
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

/// Every artifact of `target`'s synthesis, by path, with the behaviour generated.
fn artifacts(target: Target) -> BTreeMap<String, String> {
    let synthesis = synthesize_for(&model(), target).expect("a realizable target");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.jobs.AddJob"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}: the behaviour is generated"
    );
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
        .join(format!("empty-list-set-go-{}", std::process::id()));
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
fn generated_rust_stores_the_empty_list() {
    let rust = artifacts(Target::Rust);
    let behaviour = artifact(&rust, "crates/demo-types/src/behaviour.rs");
    assert!(
        behaviour.contains("tags: Vec::new()"),
        "the Rust row holds an empty list:\n{behaviour}"
    );
}

#[test]
fn generated_go_stores_an_empty_slice_and_builds() {
    let go = artifacts(Target::Go);
    let behaviour = artifact(&go, "types/behaviour/behaviour.go");
    assert!(
        behaviour.contains("Tags: []string{}"),
        "the Go row holds an empty list, not nil:\n{behaviour}"
    );
    let (log, built) = go_build(go);
    assert!(built, "{log}");
}
