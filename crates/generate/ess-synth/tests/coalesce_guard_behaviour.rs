//! The generated Rust and Go behaviours decide a guard that says what an absent `Optional`
//! compares as (`ess/29`, beyond10x/ess#620): `coalesce(is_error, true) == false` reads the fact
//! where present and the literal where absent, so the command is generated rather than owed, and
//! it decides every record the shared evaluator decides, absent ones included. Each lane is
//! compiled and run healthy, then once with the default dropped from the emitted seam, which must
//! fail: absence is Unknown again there, and the behaviour answers with an unmet obligation.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL: &str = "format: ess/29
system: closing
version: v1
domain: closing.desk
events:
  - {name: closing.desk.Budget, fields: []}
  - {name: closing.desk.Completed, fields: []}
  - {name: closing.desk.Retry, fields: []}
  - {name: closing.desk.Failed, fields: []}
commands:
  - name: closing.desk.Classify
    input:
      - {name: terminal, type: Boolean}
      - {name: is_error, type: Optional<Boolean>}
      - {name: stop_reason, type: Optional<String>}
      - {name: attempts, type: Optional<Integer>}
    outcomes:
      - name: budget-stop
        when: coalesce(stop_reason, \"none\") == \"budget\"
        emits: [closing.desk.Budget]
      - name: completed
        when:
          all:
            - terminal == true
            - coalesce(is_error, true) == false
        emits: [closing.desk.Completed]
      - name: retry
        when: coalesce(attempts, 0) < 3
        emits: [closing.desk.Retry]
      - name: failed
        emits: [closing.desk.Failed]
components:
  - component: closing-service
    owns: {domains: [closing.desk]}
    accepts: {commands: [closing.desk.Classify]}
    publishes: {events: [closing.desk.Budget, closing.desk.Completed, closing.desk.Retry, closing.desk.Failed]}
";

fn ir_of(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("closing.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emit(target: Target) -> PathBuf {
    let synthesis = synthesize_for(&ir_of(MODEL), target)
        .unwrap_or_else(|error| panic!("{target:?}: the model is generated: {error:?}"));
    assert!(
        synthesis
            .plan
            .is_generated(CapabilityKind::CommandBehavior, "closing.desk.Classify"),
        "{target:?}: the coalesced guards are generated, not owed: {:?}",
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "closing.desk.Classify")
    );
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "coalesce-guard-{}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    directory
}

/// Runs `program` in `directory`; its output and whether it succeeded.
fn run(directory: &Path, program: &str, arguments: &[&str]) -> (bool, String) {
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
    (output.status.success(), log)
}

/// Replaces every `before` with `after` in the one emitted file that holds it.
fn patch(directory: &Path, file: &str, before: &str, after: &str) {
    let path = directory.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(before), "{file} holds `{before}`:\n{text}");
    std::fs::write(&path, text.replace(before, after)).unwrap();
}

const RUST_CASES: &str = r#"
use closing_types::{behaviour::Generated, desk::{Classify, ClassifyOutcome, obligations::ClassifyBehavior}};
fn classify(terminal: bool, is_error: Option<bool>, stop_reason: Option<&str>, attempts: Option<i64>) -> &'static str {
    match Generated::new(()).classify(Classify {
        terminal,
        is_error,
        stop_reason: stop_reason.map(str::to_owned),
        attempts,
    }) {
        Ok(ClassifyOutcome::BudgetStop { .. }) => "budget-stop",
        Ok(ClassifyOutcome::Completed { .. }) => "completed",
        Ok(ClassifyOutcome::Retry { .. }) => "retry",
        Ok(ClassifyOutcome::Failed { .. }) => "failed",
        Err(_) => "unknown",
    }
}
#[test]
fn guards() {
    assert_eq!(classify(true, Some(false), None, Some(5)), "completed", "an explicit success");
    assert_eq!(classify(true, Some(true), None, Some(5)), "failed", "an explicit failure");
    assert_eq!(classify(true, None, None, Some(5)), "failed", "an absent verdict is no success");
    assert_eq!(classify(true, None, Some("budget"), Some(5)), "budget-stop", "the stop reason");
    assert_eq!(classify(true, None, Some("other"), Some(5)), "failed", "another stop reason");
    assert_eq!(classify(false, Some(false), None, None), "retry", "absent attempts are none");
    assert_eq!(classify(false, Some(false), None, Some(2)), "retry", "two attempts");
    assert_eq!(classify(false, Some(false), None, Some(3)), "failed", "three attempts");
}
"#;

#[test]
fn coalesced_guards_in_the_generated_rust_behaviour_and_the_dropped_default_fails() {
    let directory = emit(Target::Rust);
    let tests = directory.join("crates/closing-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("coalesce.rs"), RUST_CASES).unwrap();
    let cargo = env!("CARGO");
    let arguments = [
        "test",
        "--offline",
        "-p",
        "closing-types",
        "--test",
        "coalesce",
    ];
    let (healthy, log) = run(&directory, cargo, &arguments);
    assert!(healthy && log.contains("1 passed; 0 failed"), "{log}");
    let behaviour = "crates/closing-types/src/behaviour.rs";
    patch(&directory, behaviour, ".or(Some(true))", "");
    let (passed, log) = run(&directory, cargo, &arguments);
    assert!(
        !passed && log.contains("FAILED"),
        "the dropped default passed\n{log}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

const GO_CASES: &str = r#"
package behaviour

import (
	"testing"

	"example.invalid/closing/types/desk"
)

func classify(terminal bool, isError *bool, stopReason *string, attempts *int64) string {
	outcome, err := New(Ports{}).Classify(desk.Classify{Terminal: terminal, IsError: isError, StopReason: stopReason, Attempts: attempts})
	if err != nil {
		return "unknown"
	}
	switch outcome.(type) {
	case desk.ClassifyOutcomeBudgetStop:
		return "budget-stop"
	case desk.ClassifyOutcomeCompleted:
		return "completed"
	case desk.ClassifyOutcomeRetry:
		return "retry"
	}
	return "failed"
}

func flag(value bool) *bool       { return &value }
func text(value string) *string   { return &value }
func count(value int64) *int64    { return &value }

func TestGuards(t *testing.T) {
	cases := []struct {
		name       string
		terminal   bool
		isError    *bool
		stopReason *string
		attempts   *int64
		want       string
	}{
		{"an explicit success", true, flag(false), nil, count(5), "completed"},
		{"an explicit failure", true, flag(true), nil, count(5), "failed"},
		{"an absent verdict is no success", true, nil, nil, count(5), "failed"},
		{"the stop reason", true, nil, text("budget"), count(5), "budget-stop"},
		{"another stop reason", true, nil, text("other"), count(5), "failed"},
		{"absent attempts are none", false, flag(false), nil, nil, "retry"},
		{"two attempts", false, flag(false), nil, count(2), "retry"},
		{"three attempts", false, flag(false), nil, count(3), "failed"},
	}
	for _, c := range cases {
		if got := classify(c.terminal, c.isError, c.stopReason, c.attempts); got != c.want {
			t.Errorf("%s: got %s, want %s", c.name, got, c.want)
		}
	}
}
"#;

#[test]
fn coalesced_guards_in_the_generated_go_behaviour_and_the_dropped_default_fails() {
    let directory = emit(Target::Go);
    std::fs::write(directory.join("types/behaviour/coalesce_test.go"), GO_CASES).unwrap();
    let arguments = [
        "test",
        "-count=1",
        "-run",
        "TestGuards",
        "./types/behaviour",
    ];
    let (healthy, log) = run(&directory, "go", &arguments);
    assert!(healthy && log.contains("ok"), "{log}");
    let behaviour = "types/behaviour/behaviour.go";
    patch(&directory, behaviour, "= some(true)", "= nil");
    let (passed, log) = run(&directory, "go", &arguments);
    assert!(
        !passed && log.contains("FAIL"),
        "the dropped default passed\n{log}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_coalesced_default_of_another_kind_stays_owed_by_name() {
    // A Decimal default for an Integer fact: the shared evaluator reads it as a Decimal, which the
    // generated read of an Integer does not compare.
    let text = MODEL.replace("coalesce(attempts, 0) < 3", "coalesce(attempts, 0.5) < 3");
    let synthesis = synthesize_for(&ir_of(&text), Target::Rust).expect("synthesizes");
    match synthesis
        .plan
        .disposition_of(CapabilityKind::CommandBehavior, "closing.desk.Classify")
    {
        Some(SynthesisDisposition::Obligation(owed)) => {
            let reason = owed.reason.describes();
            assert!(reason.contains("coalesce(attempts, 0.5)"), "{reason}");
        }
        other => panic!("owed, named by the expression: {other:?}"),
    }
}
