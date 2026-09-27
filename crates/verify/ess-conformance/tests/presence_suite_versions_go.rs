//! The Go runtime refuses suite/24 and /25 by version (beyond10x/ess#139).
//!
//! Its leaf unmarshal drops a key the type does not name, so a `presence` policy would be read as
//! no policy and the swap it forbids would pass. The runtime is compiled beside a Go test, the shape
//! `tests/optional_shape_go.rs` established.
#[test]
fn go_refuses_suites_carrying_presence_policies_by_version() {
    let directory = std::env::temp_dir().join(format!(
        "ess-presence-refusal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(
        directory.join("go.mod"),
        "module presencerefusalproof\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("runtime.go"),
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            include_str!("../src/go/runtime.go"),
            include_str!("../src/go/reading.go"),
            include_str!("../src/go/response.go"),
            include_str!("../src/go/replay.go"),
            include_str!("../src/go/fixtures.go"),
            include_str!("../../../specify/ess-domain/src/reading/coordinate.go")
        ),
    )
    .unwrap();
    std::fs::write(
        directory.join("predicate.go"),
        include_str!("../src/go/predicate.go"),
    )
    .unwrap();
    std::fs::write(
        directory.join("presence_refusal_test.go"),
        include_str!("fixtures/presence-refusal.go"),
    )
    .unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "-p",
            "2",
            "-count=1",
            "-run",
            "^TestPresenceSuitesAreRefusedByVersion$",
            ".",
        ])
        .env("GOWORK", "off")
        .env("GOMAXPROCS", "2")
        .current_dir(&directory)
        .output()
        .expect("required Go toolchain executes");
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
