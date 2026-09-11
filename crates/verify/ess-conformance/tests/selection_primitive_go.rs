//! Canonical source type names must reach the Go observer's primitive grammar.
#[test]
fn go_selection_validates_primitives_and_every_list_member() {
    let directory = std::env::temp_dir().join(format!(
        "ess-selection-primitives-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(
        directory.join("go.mod"),
        "module selectionproof\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("runtime.go"),
        format!(
            "{}\n{}\n{}\n{}",
            include_str!("../src/go/runtime.go"),
            include_str!("../src/go/reading.go"),
            include_str!("../src/go/response.go"),
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
        directory.join("selection_test.go"),
        include_str!("fixtures/selection-primitives.go"),
    )
    .unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "-p",
            "2",
            "-count=1",
            "-run",
            "^TestSelectionPrimitiveValues$",
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
