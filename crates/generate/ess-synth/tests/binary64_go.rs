//! The Go target carries a finite `Binary64` in every carried position, and still refuses an
//! uncarried one (an identity), with the detail the Rust target gives.
//!
//! The generated Go server is built once, in one test, beside a harness that holds only storage and
//! identities, and is held to the shared wire table and the synthesized suite. The test is skipped,
//! and said out loud, where this machine has no `go`.

#[path = "binary64_served/support.rs"]
mod support;

use std::path::Path;
use std::process::{Command, Output};
use std::time::Instant;

use ess_synth::{synthesize_for, Target, TargetFailureCode};

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn write_tree(root: &Path, synthesis: &ess_synth::Synthesis) {
    for artifact in synthesis.artifacts.values() {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
}

fn has_go() -> bool {
    let found = Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success());
    if !found {
        println!("skipped: no `go` on PATH, so the generated Go server was not run");
    }
    found
}

fn go(directory: &Path, arguments: &[&str]) -> Output {
    Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("go runs")
}

fn artifact<'a>(synthesis: &'a ess_synth::Synthesis, path: &str) -> &'a str {
    &synthesis
        .artifacts
        .values()
        .find(|artifact| artifact.path == path)
        .unwrap_or_else(|| panic!("no artifact {path}"))
        .contents
}

#[test]
fn the_go_target_synthesizes_every_carried_binary64_position() {
    let synthesis =
        synthesize_for(&support::model(), Target::Go).expect("the Go target carries it");
    let counts = synthesis.plan.counts();
    assert_eq!(
        (counts.generated, counts.obligations, counts.refused),
        (14, 0, 0)
    );
    let primitives = artifact(&synthesis, "types/primitives/primitives.go");
    assert!(primitives.contains("type Binary64 struct {"));
    assert!(primitives.contains("func (v Binary64) Token() string {"));
    assert!(primitives.contains("The five below have no"));
    let wire = artifact(&synthesis, "server/wire.go");
    assert!(wire.contains("func binary64At("));
    assert!(wire.matches("binary64At(").count() >= 2);
    assert!(wire.contains("json.Number("));
    assert!(wire.contains(".Token())"));
}

#[test]
fn a_go_model_without_binary64_keeps_its_bytes() {
    let source = support::MODEL.replace("Binary64", "Decimal");
    let synthesis = synthesize_for(&support::compile(&source), Target::Go).expect("synthesizes");
    let primitives = artifact(&synthesis, "types/primitives/primitives.go");
    assert!(!primitives.contains("Binary64"));
    assert!(primitives.contains("The four below have no"));
    assert!(!artifact(&synthesis, "server/wire.go").contains("binary64At"));
}

#[test]
fn a_binary64_identity_is_still_refused_by_the_go_target() {
    let source = support::MODEL.replace(
        "{name: demo.search.DocId, kind: newtype, of: Uuid}",
        "{name: demo.search.DocId, kind: newtype, of: Binary64}",
    );
    assert_ne!(source, support::MODEL);
    let Err(failure) = synthesize_for(&support::compile(&source), Target::Go) else {
        panic!("a Binary64 identity is refused");
    };
    let causes = failure.causes();
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert_eq!(causes[0].code(), TargetFailureCode::MissingRepresentation);
    assert_eq!(causes[0].sources(), ["types.demo.search.DocId.of"]);
    assert!(
        causes[0]
            .detail()
            .starts_with("a Binary64 here is a key, a lookup or a delivery route"),
        "{}",
        causes[0].detail()
    );
}

#[test]
fn the_go_server_carries_a_finite_binary64() {
    if !has_go() {
        return;
    }
    let synthesis =
        synthesize_for(&support::model(), Target::Go).expect("the Go target carries it");
    let root =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("binary64-go-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("go-demo");
    write_tree(&tree, &synthesis);
    let formatted = Command::new("gofmt")
        .arg("-l")
        .arg(&tree)
        .output()
        .expect("gofmt runs");
    assert!(formatted.status.success(), "{}", shown(&formatted));
    assert_eq!(
        String::from_utf8_lossy(&formatted.stdout),
        "",
        "the generated module is gofmt-clean"
    );

    let kept = root.join("kept");
    let harness = root.join("go-harness");
    std::fs::create_dir_all(&kept).expect("mkdir");
    std::fs::create_dir_all(&harness).expect("mkdir");
    std::fs::write(
        harness.join("go.mod"),
        "module binary64goharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
         example.invalid/demo => ../go-demo\n",
    )
    .expect("write");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/binary64-served-go-harness/main.go"),
        harness.join("main.go"),
    )
    .expect("the harness copies");
    let binary = kept.join("go");
    let started = Instant::now();
    let output = go(
        &harness,
        &["build", "-o", binary.to_str().expect("a UTF-8 path"), "."],
    );
    assert!(
        output.status.success(),
        "the Go harness builds: {}",
        shown(&output)
    );
    eprintln!("the Go harness built in {:?}", started.elapsed());

    support::writes_every_token("go", &binary);
    support::refuses_what_no_binary64_holds("go", &binary);
    support::compares_by_value("go", &binary);
    support::passes_the_synthesized_suite("go", &binary);
}
