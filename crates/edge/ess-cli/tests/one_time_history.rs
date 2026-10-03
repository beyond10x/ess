//! Model-aware history import refuses disclosure policy before reading caller log bytes.
use std::{fs, process::Command};

const MODEL: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";

#[test]
fn marked_history_import_refuses_before_input_io_and_preserves_output() {
    for existing in [false, true] {
        let directory = std::env::temp_dir().join(format!(
            "ess-one-time-history-{existing}-{}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let specification = directory.join("system.yaml");
        fs::write(&specification, MODEL).unwrap();
        let adapter = directory.join("PRIVATE-INPUT-SENTINEL.adapter");
        let log = directory.join("PRIVATE-INPUT-SENTINEL.log");
        let destination = directory.join("history.json");
        if existing {
            fs::write(&adapter, "PRIVATE-CONTENT-SENTINEL").unwrap();
            fs::write(&log, "PRIVATE-CONTENT-SENTINEL").unwrap();
            fs::write(&destination, "preserve-existing-output\n").unwrap();
        }
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .args(["verify", "conform", "import-history", "--path"])
            .arg(&specification)
            .arg("--adapter")
            .arg(&adapter)
            .arg("--log")
            .arg(&log)
            .arg("--output")
            .arg(&destination)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let diagnostics = String::from_utf8(output.stderr).unwrap();
        assert!(
            diagnostics.contains("UnsupportedOneTimeDisclosure"),
            "{diagnostics}"
        );
        assert!(diagnostics.contains("one_time_response"), "{diagnostics}");
        assert!(!diagnostics.contains("PRIVATE-"), "{diagnostics}");
        if existing {
            assert_eq!(
                fs::read_to_string(&destination).unwrap(),
                "preserve-existing-output\n"
            );
        } else {
            assert!(!destination.exists());
        }
        assert!(!directory.join("history.json.gaps.json").exists());
    }
}

#[test]
fn unmarked_history_import_keeps_its_existing_input_refusal() {
    let directory =
        std::env::temp_dir().join(format!("ess-ordinary-history-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let specification = directory.join("system.yaml");
    fs::write(
        &specification,
        MODEL.replace(", one_time_response: [secret]", ""),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "import-history", "--path"])
        .arg(&specification)
        .arg("--adapter")
        .arg(directory.join("absent.adapter"))
        .arg("--log")
        .arg(directory.join("absent.log"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let diagnostics = String::from_utf8(output.stderr).unwrap();
    assert!(
        diagnostics.contains("import-history.adapter-unreadable"),
        "{diagnostics}"
    );
    assert!(!diagnostics.contains("UnsupportedOneTimeDisclosure"));
}
