//! An expression value, `{expr: …}`, and a computed view field, `expr:` (`ess/27`,
//! beyond10x/ess#560, `docs/design/generic-language-features.md` § G1), in the generated Rust and Go
//! targets: neither computes an expression yet, so the command's behaviour and the view's query stay
//! obligations named by the expression they would compute, rather than code that computes it
//! otherwise than the shared evaluator does. The rest of the model is still generated, and builds.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/expression-arithmetic-and-time.yaml"
);

const COMPONENT: &str = "components:
  - component: ledger-service
    owns: {domains: [demo.ledger]}
    accepts: {commands: [demo.ledger.RecordPastIncident]}
    publishes: {events: [demo.ledger.IncidentRecorded]}
    reached_by: network
";

fn model() -> EssIr {
    let text = format!("{MODEL}{COMPONENT}");
    let spec = Specification::assemble([(
        Source::new("ledger.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn owed(target: Target, kind: CapabilityKind, source: &str) -> String {
    let synthesis = synthesize_for(&model(), target).expect("a realizable target");
    match synthesis.plan.disposition_of(kind, source) {
        Some(SynthesisDisposition::Obligation(owed)) => owed.reason.describes(),
        other => panic!("{target:?}: `{source}` is owed, not {other:?}"),
    }
}

#[test]
fn an_expression_value_keeps_its_command_an_obligation_named_by_it() {
    for target in [Target::Rust, Target::Go] {
        let reason = owed(
            target,
            CapabilityKind::CommandBehavior,
            "demo.ledger.RecordPastIncident",
        );
        assert!(
            reason.contains("an expression value") && reason.contains("input.ended_at"),
            "{target:?}: {reason}"
        );
    }
}

#[test]
fn a_computed_view_field_keeps_its_query_an_obligation_named_by_it() {
    for target in [Target::Rust, Target::Go] {
        let reason = owed(target, CapabilityKind::ViewQuery, "demo.ledger.Incidents");
        assert!(
            reason.contains("computed field `elapsed`") && reason.contains("ended_at - started_at"),
            "{target:?}: {reason}"
        );
    }
}

/// Every artifact of `target`'s synthesis, written under a fresh directory.
fn emit(target: Target) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-values-560-{}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    let synthesis = synthesize_for(&model(), target).expect("a realizable target");
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    directory
}

fn run(directory: &Path, program: &str, arguments: &[&str]) -> (String, bool) {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(directory);
    if program == env!("CARGO") {
        command
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("RUSTC_WRAPPER")
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
    (log, output.status.success())
}

#[test]
fn the_rest_of_the_model_is_generated_and_builds_in_rust_and_go() {
    let directory = emit(Target::Rust);
    let (log, passed) = run(
        &directory,
        env!("CARGO"),
        &["check", "--offline", "-p", "demo-types"],
    );
    assert!(passed, "{log}");
    let _ = std::fs::remove_dir_all(&directory);
    let directory = emit(Target::Go);
    let (log, passed) = run(&directory, "go", &["build", "-buildvcs=false", "./..."]);
    assert!(passed, "{log}");
    let _ = std::fs::remove_dir_all(&directory);
}

const DIGEST: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/digest-guard.yaml");

const DIGEST_COMPONENT: &str = "components:
  - component: pkce-service
    owns: {domains: [demo.pkce]}
    accepts: {commands: [demo.pkce.IssueCode, demo.pkce.Exchange]}
    publishes: {events: [demo.pkce.CodeIssued, demo.pkce.CodeRedeemed]}
    reached_by: network
";

/// A guard reading a fact through a digest or encoding function (`ess/27`, beyond10x/ess#584):
/// neither generated behaviour computes one yet, so the command stays an obligation named by the
/// read, and the command issuing the code is still generated.
#[test]
fn a_guard_reading_a_fact_through_a_function_keeps_its_command_an_obligation() {
    let text = format!("{DIGEST}{DIGEST_COMPONENT}");
    let spec = Specification::assemble([(
        Source::new("pkce.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let model = compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"));
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&model, target).expect("a realizable target");
        match synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.pkce.Exchange")
        {
            Some(SynthesisDisposition::Obligation(owed)) => {
                let reason = owed.reason.describes();
                assert!(
                    reason.contains("through a function") && reason.contains("sha256"),
                    "{target:?}: {reason}"
                );
            }
            other => panic!("{target:?}: `Exchange` is owed, not {other:?}"),
        }
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "demo.pkce.IssueCode"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}
