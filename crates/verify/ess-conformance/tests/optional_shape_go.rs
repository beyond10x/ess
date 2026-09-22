//! A declared `optional` leaf must reach the Go runtime's own decision, and mean "may be absent"
//! rather than "unchecked".
//!
//! The runtime is a checked-in Go file copied into every emitted package, so the only way to hold it
//! to a claim is to compile it beside a Go test — the shape
//! `tests/selection_primitive_go.rs` already established for the primitive grammar.
#[test]
fn go_shape_leaves_honour_optional_without_going_unchecked() {
    let directory = std::env::temp_dir().join(format!(
        "ess-optional-shape-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(
        directory.join("go.mod"),
        "module optionalshapeproof\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("runtime.go"),
        format!(
            "{}\n{}\n{}\n{}\n{}",
            include_str!("../src/go/runtime.go"),
            include_str!("../src/go/reading.go"),
            include_str!("../src/go/response.go"),
            include_str!("../src/go/replay.go"),
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
        directory.join("optional_shape_test.go"),
        include_str!("fixtures/optional-shape.go"),
    )
    .unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "-p",
            "2",
            "-count=1",
            "-run",
            "^TestShapeLeafKeepsItsOptionalFlag|TestOptionalLeaf",
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
