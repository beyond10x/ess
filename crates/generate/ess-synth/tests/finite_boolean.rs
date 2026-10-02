//! The Boolean partition admitted by the finite proof reaches generated target behavior.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
use std::process::Command;

const MODEL: &str = "format: ess/1
system: reporting
version: v1
domain: reporting.core
events:
  - {name: reporting.core.Observed, fields: []}
commands:
  - name: reporting.core.Report
    input:
      - {name: pause, type: Boolean}
    outcomes:
      - name: paused
        when: pause == true
        emits: [reporting.core.Observed]
      - name: resumed
        when: pause == false
        emits: [reporting.core.Observed]
components:
  - component: reporting-service
    owns: {domains: [reporting.core]}
    accepts: {commands: [reporting.core.Report]}
    publishes: {events: [reporting.core.Observed]}
";

#[test]
fn boolean_partition_reaches_generated_targets() {
    let spec = Specification::assemble([(
        Source::new("boolean.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("finite-boolean-{}", std::process::id()));
    for target in [Target::Rust, Target::Go, Target::Web] {
        let synthesis = synthesize_for(&ir, target).unwrap();
        assert!(synthesis.plan.is_generated(
            ess_synth::CapabilityKind::CommandBehavior,
            "reporting.core.Report"
        ));
        let directory = root.join(target.name());
        for (relative, artifact) in synthesis.artifacts {
            let path = directory.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        eprintln!("emitted {}", directory.display());
        if target == Target::Rust {
            let tests = directory.join("crates/reporting-types/tests");
            std::fs::create_dir_all(&tests).unwrap();
            std::fs::write(tests.join("partition.rs"), r"
use reporting_types::{behaviour::Generated, core::{Report, ReportOutcome, obligations::ReportBehavior}};
#[test]
fn both_boolean_values_select_their_declared_outcome() {
    let mut generated = Generated::new(());
    assert!(matches!(generated.report(Report { pause: true }).unwrap(), ReportOutcome::Paused { .. }));
    assert!(matches!(generated.report(Report { pause: false }).unwrap(), ReportOutcome::Resumed { .. }));
}
").unwrap();
            let output = Command::new(env!("CARGO"))
                .args([
                    "test",
                    "--offline",
                    "-p",
                    "reporting-types",
                    "--test",
                    "partition",
                ])
                .env_remove("CARGO_TARGET_DIR")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .env("RUSTFLAGS", "-D warnings")
                .current_dir(&directory)
                .output()
                .unwrap();
            let log = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!("{log}");
            assert!(output.status.success(), "{log}");
            assert!(log.contains("1 passed; 0 failed"), "{log}");
        }
        if target == Target::Go {
            let tests = directory.join("types/behaviour/partition_test.go");
            std::fs::write(
                tests,
                r#"
package behaviour
import (
    "testing"
    "example.invalid/reporting/types/core"
)
func TestBothBooleanValues(t *testing.T) {
    generated := New(Ports{})
    paused, err := generated.Report(core.Report{Pause: true})
    if err != nil { t.Fatal(err) }
    if _, ok := paused.(core.ReportOutcomePaused); !ok { t.Fatalf("true: %T", paused) }
    resumed, err := generated.Report(core.Report{Pause: false})
    if err != nil { t.Fatal(err) }
    if _, ok := resumed.(core.ReportOutcomeResumed); !ok { t.Fatalf("false: %T", resumed) }
}
"#,
            )
            .unwrap();
            let output = Command::new("go")
                .args(["test", "-count=1", "-v", "./types/behaviour"])
                .current_dir(&directory)
                .output()
                .unwrap();
            let log = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!("{log}");
            assert!(output.status.success(), "{log}");
            assert!(log.contains("--- PASS: TestBothBooleanValues"), "{log}");
        }
    }
}
