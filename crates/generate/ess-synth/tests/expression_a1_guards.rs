//! The generated Rust and Go behaviours decide a guard comparing two facts by bare words
//! (`docs/design/expression-family-source22.md`, A1): `task_id == depends_on` by the values sent, and
//! `valid_until < valid_from` by the instants the two Timestamps name, never their spellings
//! (decision 2). Each is compiled and run.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
use std::process::Command;

const MODEL: &str = "format: ess/22
system: graph
version: v1
domain: graph.core
events:
  - {name: graph.core.SelfEdge, fields: []}
  - {name: graph.core.Early, fields: []}
  - {name: graph.core.Linked, fields: []}
commands:
  - name: graph.core.Link
    input:
      - {name: task_id, type: String}
      - {name: depends_on, type: String}
      - {name: valid_from, type: Timestamp}
      - {name: valid_until, type: Timestamp}
    outcomes:
      - name: self-edge
        when: task_id == depends_on
        emits: [graph.core.SelfEdge]
      - name: early
        when: valid_until < valid_from
        emits: [graph.core.Early]
      - name: linked
        emits: [graph.core.Linked]
components:
  - component: graph-service
    owns: {domains: [graph.core]}
    accepts: {commands: [graph.core.Link]}
    publishes: {events: [graph.core.SelfEdge, graph.core.Early, graph.core.Linked]}
";

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("graph.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emit(target: Target) -> std::path::PathBuf {
    let synthesis = synthesize_for(&ir(), target).unwrap();
    assert!(
        synthesis.plan.is_generated(
            ess_synth::CapabilityKind::CommandBehavior,
            "graph.core.Link"
        ),
        "{target:?}: the guard is generated, not owed"
    );
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-a1-guards-{}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    eprintln!("emitted {}", directory.display());
    directory
}

fn run(directory: &std::path::Path, program: &str, arguments: &[&str]) -> String {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(directory);
    if program == env!("CARGO") {
        command
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("RUSTFLAGS", "-D warnings");
    } else {
        command
            .env("GOWORK", "off")
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off");
    }
    let output = command.output().unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!("{log}");
    assert!(output.status.success(), "{log}");
    log
}

/// `12:00Z` spelled lower than `11:00Z`, and `11:00Z` spelled higher than `11:30Z`: a guard
/// ordering spellings answers both the wrong way round.
#[test]
fn a1_timestamp_sibling_instant_order_in_the_generated_rust_guard() {
    let directory = emit(Target::Rust);
    let tests = directory.join("crates/graph-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(
        tests.join("guards.rs"),
        r#"
use graph_types::{behaviour::Generated, core::{Link, LinkOutcome, obligations::LinkBehavior}, primitives::Timestamp};
fn link(task: &str, depends: &str, from: &str, until: &str) -> LinkOutcome {
    Generated::new(())
        .link(Link {
            task_id: task.to_owned(),
            depends_on: depends.to_owned(),
            valid_from: Timestamp(from.to_owned()),
            valid_until: Timestamp(until.to_owned()),
        })
        .unwrap()
}
#[test]
fn the_guards_read_both_facts_and_compare_instants() {
    let (early, late) = ("2020-01-01T11:00:00Z", "2020-01-01T12:00:00Z");
    assert!(matches!(link("a", "a", early, late), LinkOutcome::SelfEdge { .. }));
    assert!(matches!(link("a", "b", early, late), LinkOutcome::Linked { .. }));
    assert!(matches!(link("depends_on", "x", early, late), LinkOutcome::Linked { .. }));
    assert!(matches!(
        link("a", "b", "2020-01-01T11:00:00Z", "2020-01-01T10:00:00-02:00"),
        LinkOutcome::Linked { .. }
    ));
    assert!(matches!(
        link("a", "b", "2020-01-01T11:30:00Z", "2020-01-01T12:00:00+01:00"),
        LinkOutcome::Early { .. }
    ));
}
"#,
    )
    .unwrap();
    let log = run(
        &directory,
        env!("CARGO"),
        &["test", "--offline", "-p", "graph-types", "--test", "guards"],
    );
    assert!(log.contains("1 passed; 0 failed"), "{log}");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a1_timestamp_sibling_instant_order_in_the_generated_go_guard() {
    let directory = emit(Target::Go);
    std::fs::write(
        directory.join("types/behaviour/guards_test.go"),
        r#"
package behaviour

import (
	"testing"

	"example.invalid/graph/types/core"
	"example.invalid/graph/types/primitives"
)

func link(t *testing.T, task, depends, from, until string) core.LinkOutcome {
	outcome, err := New(Ports{}).Link(core.Link{
		TaskId:     task,
		DependsOn:  depends,
		ValidFrom:  primitives.NewTimestamp(from),
		ValidUntil: primitives.NewTimestamp(until),
	})
	if err != nil {
		t.Fatal(err)
	}
	return outcome
}

func TestGuards(t *testing.T) {
	early, late := "2020-01-01T11:00:00Z", "2020-01-01T12:00:00Z"
	if _, ok := link(t, "a", "a", early, late).(core.LinkOutcomeSelfEdge); !ok {
		t.Fatal("equal identities are a self-edge")
	}
	if _, ok := link(t, "a", "b", early, late).(core.LinkOutcomeLinked); !ok {
		t.Fatal("different identities link")
	}
	if _, ok := link(t, "depends_on", "x", early, late).(core.LinkOutcomeLinked); !ok {
		t.Fatal("the word is never the text it is spelled like")
	}
	if _, ok := link(t, "a", "b", "2020-01-01T11:00:00Z", "2020-01-01T10:00:00-02:00").(core.LinkOutcomeLinked); !ok {
		t.Fatal("a later instant spelled lower is not early")
	}
	if _, ok := link(t, "a", "b", "2020-01-01T11:30:00Z", "2020-01-01T12:00:00+01:00").(core.LinkOutcomeEarly); !ok {
		t.Fatal("an earlier instant spelled higher is early")
	}
}
"#,
    )
    .unwrap();
    let log = run(
        &directory,
        "go",
        &["test", "-count=1", "-v", "./types/behaviour"],
    );
    assert!(log.contains("--- PASS: TestGuards"), "{log}");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn below_ess22_a_timestamp_pair_stays_owed() {
    let old = MODEL
        .replace("format: ess/22", "format: ess/21")
        .replace("when: task_id == depends_on", "when: task_id == \"x\"")
        .replace("    input:\n", "    input:\n      - {name: window, type: graph.core.Window}\n")
        .replace("when: valid_until < valid_from", "when: window.end < window.start")
        .replace(
            "\nevents:\n",
            "\ntypes:\n  - name: graph.core.Window\n    kind: struct\n    fields:\n      - {name: start, type: Timestamp}\n      - {name: end, type: Timestamp}\nevents:\n",
        );
    let spec =
        Specification::assemble([(Source::new("graph.yaml"), RawSpecFile::parse(&old).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).unwrap();
        assert!(
            !synthesis.plan.is_generated(
                ess_synth::CapabilityKind::CommandBehavior,
                "graph.core.Link"
            ),
            "{target:?}: an ess/21 guard over two Timestamps keeps its obligation"
        );
    }
}
