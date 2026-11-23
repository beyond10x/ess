//! The Rust target carries a finite `Binary64` in every position it can, and refuses the rest with
//! one detail.
//!
//! The generated server is not built here: a test that runs cargo compiles a crate graph inside
//! its lane (`crates/edge/ess-xtask/tests/no_test_builds.rs`). `task binary64-harness` generates
//! it from the fixture model with `ess generate synthesize` and builds the harness once, and this
//! test runs the binary [`HARNESS`] names. Without it the server cases fail, naming the task.

#[path = "binary64_served/support.rs"]
mod support;

use std::path::PathBuf;

use ess_synth::{synthesize_for, Target};

const HARNESS: &str = "ESS_BINARY64_HARNESS";

/// The harness `task binary64-harness` built against the generated Rust server, from the path
/// [`HARNESS`] names.
///
/// A file that is not executable is made so: an uploaded CI artifact loses its mode bits, and the
/// test shards receive the harness beside the nextest archives.
fn rust_harness() -> PathBuf {
    let Some(path) = std::env::var_os(HARNESS).map(PathBuf::from) else {
        panic!(
            "{HARNESS} is not set: run `task binary64-harness` and set it to the binary \
             that task names, or run the tests through `task test`, which does both"
        );
    };
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "{HARNESS}={} names no harness ({error}): run `task binary64-harness`",
            path.display()
        )
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            std::fs::set_permissions(&path, permissions).expect("the harness is made executable");
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    path
}

fn artifact<'a>(synthesis: &'a ess_synth::Synthesis, path: &str) -> &'a str {
    synthesis.artifacts.get(path).map_or_else(
        || panic!("no artifact {path}"),
        |artifact| artifact.contents.as_str(),
    )
}

#[test]
fn the_rust_target_synthesizes_every_carried_binary64_position() {
    let synthesis = synthesize_for(&support::model(), Target::Rust).expect("Rust carries Binary64");
    let counts = synthesis.plan.counts();
    assert_eq!(
        (counts.generated, counts.obligations, counts.refused),
        (14, 0, 0)
    );
    assert!(synthesis.target.is_none());
    let primitives = artifact(&synthesis, "crates/demo-types/src/primitives.rs");
    assert!(primitives.contains("pub struct Binary64(f64);"));
    assert!(primitives.contains("impl Eq for Binary64 {}"));
    let wire = artifact(&synthesis, "crates/demo-server/src/wire.rs");
    assert!(wire.contains("fn binary64_at("));
    assert!(wire.matches("binary64_at(").count() >= 2);
    assert!(wire.contains(".token());"));
}

#[test]
fn a_model_without_binary64_keeps_its_bytes() {
    let source = support::MODEL.replace("Binary64", "Decimal");
    let synthesis =
        synthesize_for(&support::compile(&source), Target::Rust).expect("Rust carries Decimal");
    assert!(!artifact(&synthesis, "crates/demo-types/src/primitives.rs").contains("Binary64"));
    assert!(!artifact(&synthesis, "crates/demo-server/src/wire.rs").contains("binary64_at"));
}

#[test]
fn a_binary64_identity_is_still_refused_with_the_uncarried_detail() {
    let source = support::MODEL.replace(
        "{name: demo.search.DocId, kind: newtype, of: Uuid}",
        "{name: demo.search.DocId, kind: newtype, of: Binary64}",
    );
    assert_ne!(source, support::MODEL);
    let Err(failure) = synthesize_for(&support::compile(&source), Target::Rust) else {
        panic!("a Binary64 identity is not carried");
    };
    let causes = failure.causes();
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert_eq!(causes[0].sources(), ["types.demo.search.DocId.of"]);
    assert_eq!(
        causes[0].detail(),
        "a Binary64 here is a key, a lookup or a delivery route, which this target does not \
         carry; it carries one in entity fields, command input and response, event and error \
         fields, and view fields and row"
    );
    assert!(format!("{:?}", causes[0].code()).contains("MissingRepresentation"));
}

#[test]
fn the_rust_server_writes_every_binary64_token_in_the_wire_layout() {
    support::writes_every_token("rust", &rust_harness());
}

#[test]
fn the_rust_server_refuses_what_no_finite_binary64_holds() {
    support::refuses_what_no_binary64_holds("rust", &rust_harness());
}

#[test]
fn the_rust_server_compares_binary64_by_value() {
    support::compares_by_value("rust", &rust_harness());
}

#[test]
fn the_synthesized_suite_passes_against_the_rust_server() {
    support::passes_the_synthesized_suite("rust", &rust_harness());
}
