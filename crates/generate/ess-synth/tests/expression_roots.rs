//! A value read its branch established (`ess/27`,
//! `story:expression-roots-a-branch-establishes-are-readable`, beyond10x/ess#555, #556) in the
//! generated Rust and Go targets: each command is generated where the behaviour can read what the
//! value reads, and otherwise stays an obligation named by the read; what is generated builds.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/expression-roots.yaml");

const COMPONENT: &str = "components:
  - component: review-service
    owns: {domains: [demo.review]}
    accepts:
      commands:
        - demo.review.OpenIncident
        - demo.review.JoinCustomer
        - demo.review.OpenReview
        - demo.review.CloseReview
        - demo.review.FileComplaint
        - demo.review.SettleIncident
    publishes:
      events:
        - demo.review.IncidentOpened
        - demo.review.IncidentSettled
        - demo.review.ReviewOpened
        - demo.review.ReviewClosed
        - demo.review.ComplaintFiled
        - demo.review.CustomerJoined
    reached_by: network
";

fn model() -> EssIr {
    let text = format!("{MODEL}{COMPONENT}");
    let spec = Specification::assemble([(
        Source::new("review.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn disposition(target: Target, command: &str) -> SynthesisDisposition {
    let synthesis = synthesize_for(&model(), target).expect("a realizable target");
    synthesis
        .plan
        .disposition_of(CapabilityKind::CommandBehavior, command)
        .cloned()
        .unwrap_or_else(|| panic!("{target:?}: `{command}` is planned"))
}

#[test]
fn each_read_is_generated_or_owed_by_name() {
    for target in [Target::Rust, Target::Go] {
        for command in [
            "demo.review.CloseReview",
            "demo.review.FileComplaint",
            "demo.review.SettleIncident",
        ] {
            match disposition(target, command) {
                SynthesisDisposition::Generated => {}
                SynthesisDisposition::Obligation(owed) => {
                    let reason = owed.reason.describes();
                    assert!(
                        reason.contains("subject")
                            || reason.contains("narrowed")
                            || reason.contains("related"),
                        "{target:?}: `{command}` is owed naming the read: {reason}"
                    );
                }
                SynthesisDisposition::Refused(refused) => {
                    panic!("{target:?}: `{command}`: {refused:?}")
                }
            }
        }
    }
}

fn emit(target: Target) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-roots-555-{}-{}",
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
fn what_is_generated_builds_in_rust_and_go() {
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
