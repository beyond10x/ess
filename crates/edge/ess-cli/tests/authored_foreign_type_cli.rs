//! `ess verify conform author` over a document in an adopter's own trace format: the refusal names
//! the foreign `type:` (ESS-AUTHOR-002), not the first key that format uses (ESS-AUTHOR-001).
use std::{fs, path::PathBuf, process::Command};

const FOREIGN: &str = "\
type: mcp-outbound-lifecycle/1
story: cancellation closes the response stream
document: outbound
state: open
outcome: closed
wire_code: 499
given: an open stream
when: the caller cancels
then: the stream closes
never: a late frame arrives
";

#[test]
fn a_foreign_scenario_type_is_refused_by_format_through_the_cli() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("authored-foreign-type-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let scenario = root.join("cancellation-closes-the-response-stream.yaml");
    fs::write(&scenario, FOREIGN).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "author", "--path"])
        .arg(repository.join("examples/billing"))
        .arg("--scenarios")
        .arg(&scenario)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&root);
    let shown = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1), "{shown}");
    assert!(shown.contains("refusal[ESS-AUTHOR-002]"), "{shown}");
    assert!(shown.contains("`mcp-outbound-lifecycle/1`"), "{shown}");
    assert!(shown.contains("`--scenarios`"), "{shown}");
    assert!(!shown.contains("ESS-AUTHOR-001"), "{shown}");
    assert!(!shown.contains("unknown field"), "{shown}");
}
