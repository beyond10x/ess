//! `ess specify validate` and `ess verify conform synthesize` refuse an authored scenario that
//! repeats a mapping key alike, as `ESS-AUTHOR-047`, and synthesize writes no suite for it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MODEL: &str = "\
format: ess/25
system: demo
version: v1
domain: demo.board
summary: A board holds a label and a size limit.
types:
  - {name: demo.board.BoardId, kind: newtype, of: Uuid}
entities:
  - name: demo.board.Board
    identity: {name: board_id, type: demo.board.BoardId}
    fields:
      - {name: label, type: String}
      - {name: limit, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.board.BoardOpened
    fields: [{name: board_id, type: demo.board.BoardId}]
actors:
  - name: demo.board.Operator
    may: [demo.board.Open]
commands:
  - name: demo.board.Open
    input:
      - {name: label, type: String}
      - {name: limit, type: Integer}
    outcomes:
      - name: opened
        creates: demo.board.Board
        instance: board_id
        emits: [demo.board.BoardOpened]
        payload: {demo.board.BoardOpened: {board_id: {generated: true}}}
        sets: {label: input.label, limit: input.limit}
views:
  - name: demo.board.Boards
    source: demo.board.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.board.BoardId}
      - {name: label, type: String}
      - {name: limit, type: Integer}
";

const INPUTS: &str = "\
format: ess-inputs/1
specification:
  - system.yaml
scenarios:
  - scenarios/open-board.yaml
";

/// A step input naming `label` twice, with two values.
const SCENARIO: &str = "\
type: ess-scenario/1
domain: demo.board
scenario: open-board
summary: Opening a board lists it with its limit.
arrange:
  - instance: board
    entity: demo.board.Board
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.board.Open
    actor: demo.board.Operator
    input: {label: morning, limit: 3, label: evening}
    outcome: opened
    capture: {instance: board, event: demo.board.BoardOpened, field: board_id}
assert:
  - view: demo.board.Boards
    contains: {board_id: {$instance: board}, limit: 3}
";

fn project() -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("authored-repeated-key-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("scenarios")).expect("mkdir");
    std::fs::write(root.join("system.yaml"), MODEL).expect("write");
    std::fs::write(root.join("ess-inputs.yaml"), INPUTS).expect("write");
    std::fs::write(root.join("scenarios/open-board.yaml"), SCENARIO).expect("write");
    root
}

fn ess(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(directory)
        .output()
        .expect("ess runs")
}

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn validate_and_synthesize_refuse_a_repeated_key_alike() {
    let root = project();

    let validated = ess(&root, &["specify", "validate"]);
    let said = shown(&validated);
    assert_eq!(validated.status.code(), Some(1), "{said}");
    assert!(said.contains("refusal[ESS-AUTHOR-047]"), "{said}");
    assert!(said.contains("timeline[0].input"), "{said}");
    assert!(said.contains("\"label\""), "{said}");

    let out = root.join("suite.json");
    let synthesized = ess(
        &root,
        &[
            "verify",
            "conform",
            "synthesize",
            "--scenarios",
            "scenarios",
            "--out",
            "suite.json",
        ],
    );
    let said = shown(&synthesized);
    assert_eq!(synthesized.status.code(), Some(1), "{said}");
    assert!(said.contains("refusal[ESS-AUTHOR-047]"), "{said}");
    assert!(said.contains("timeline[0].input"), "{said}");
    assert!(!out.exists(), "no suite is written: {said}");

    let _ = std::fs::remove_dir_all(&root);
}
