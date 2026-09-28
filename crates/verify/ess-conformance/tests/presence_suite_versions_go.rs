//! The Go runtime reads suite/24 and /25 and holds a payload leaf to its presence policy
//! (beyond10x/ess#139, beyond10x/ess#188).
//!
//! Until #188 this runtime refused both majors by version: its leaf unmarshal dropped the
//! `presence` key, so a policy would have been read as no policy and the swap it forbids would
//! have passed. `Held` now carries the key and `holds` decides by it as `LeafShape::admits` does,
//! so the refusal is replaced by the check. The runtime is compiled beside a Go test, the shape
//! `tests/optional_shape_go.rs` established.
#[test]
fn go_admits_presence_suites_and_decides_by_the_policy() {
    let directory = std::env::temp_dir().join(format!(
        "ess-presence-policy-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(
        directory.join("go.mod"),
        "module presencepolicyproof\n\ngo 1.24\n",
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
        directory.join("presence_policy_test.go"),
        include_str!("fixtures/presence-policy-go.go"),
    )
    .unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "-p",
            "2",
            "-count=1",
            "-v",
            "-run",
            "^TestPresence",
            ".",
        ])
        .env("GOWORK", "off")
        .env("GOMAXPROCS", "2")
        .current_dir(&directory)
        .output()
        .expect("required Go toolchain executes");
    std::fs::remove_dir_all(&directory).unwrap();
    let log = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{log}");
    for case in [
        "TestPresenceSuitesAreAdmitted",
        "TestPresencePolicyDecidesNullAndAbsence",
        "TestPresenceBelowSuite24IsRefused",
    ] {
        assert!(
            log.contains(&format!("--- PASS: {case}")),
            "{case} ran: {log}"
        );
    }
}
