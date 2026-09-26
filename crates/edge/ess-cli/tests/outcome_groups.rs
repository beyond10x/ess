//! `ess specify validate` on the reproduction in beyond10x/ess#105, written as an outcome group.
//!
//! `docs/design/outcome-groups.md`, C6: the issue's minimal `keypad` specification declares one
//! outcome for the commands an actor may invoke. Written as `outcome_groups:` it validates; written
//! the way the issue first tried, as `actors[].outcomes`, it is still an unknown field.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const SYSTEM: &str = "format: ess/12
system: keypad
version: v1
summary: A minimal keypad.
domains:
  - keypad.dial
";

const DIAL: &str = "domain: keypad.dial
summary: Sending key presses on an established session.
types:
  - name: keypad.dial.SessionId
    kind: newtype
    of: Uuid
  - name: keypad.dial.KeySequence
    kind: newtype
    of: String
actors:
  - name: keypad.dial.Caller
    may: [keypad.dial.SendKeys]
errors:
  - name: keypad.dial.UnsupportedKey
    summary: The sequence contains a character that has no key.
events:
  - name: keypad.dial.KeysSent
    fields:
      - {name: session_id, type: keypad.dial.SessionId}
      - {name: keys, type: keypad.dial.KeySequence}
commands:
  - name: keypad.dial.SendKeys
    input:
      - {name: session_id, type: keypad.dial.SessionId}
      - {name: keys, type: keypad.dial.KeySequence}
    outcomes:
      - name: sent
        emits: [keypad.dial.KeysSent]
        payload:
          keypad.dial.KeysSent:
            session_id: input.session_id
            keys: input.keys
";

const GROUP: &str = "outcome_groups:
  - name: remote-backed
    actor: keypad.dial.Caller
    outcomes:
      - name: credential-rejected
        external: the service rejects the session credential
        error: keypad.dial.UnsupportedKey
";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

/// A `keypad/` directory holding `system.yaml` and `domains/dial.yaml`.
fn keypad(dial: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = repo().join(format!(
        "target/outcome-groups-cli/{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(root.join("keypad/domains")).unwrap();
    fs::write(root.join("keypad/system.yaml"), SYSTEM).unwrap();
    fs::write(root.join("keypad/domains/dial.yaml"), dial).unwrap();
    root
}

fn validate(root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(["specify", "validate", "--path", "keypad"])
        .output()
        .unwrap()
}

#[test]
fn c6_the_issues_reproduction_written_as_a_group_validates() {
    let root = keypad(&format!("{DIAL}{GROUP}"));
    let output = validate(&root);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("valid"), "{stdout}");

    // And the actor's command can now end that way: the suite carries the expanded branch.
    let synthesized = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(&root)
        .args([
            "verify",
            "conform",
            "synthesize",
            "--path",
            "keypad",
            "--target",
            "ir",
            "--out",
            "suite.json",
        ])
        .output()
        .unwrap();
    assert!(
        synthesized.status.success(),
        "{}",
        String::from_utf8_lossy(&synthesized.stderr)
    );
    let suite = fs::read_to_string(root.join("suite.json")).unwrap();
    assert!(
        suite.contains("keypad.dial.SendKeys/outcome/credential-rejected"),
        "{suite}"
    );
}

#[test]
fn c6_outcomes_on_an_actor_are_still_an_unknown_field() {
    let dial = DIAL.replace(
        "    may: [keypad.dial.SendKeys]\n",
        "    may: [keypad.dial.SendKeys]\n    outcomes:\n      - name: credential-rejected\n        \
         external: the service rejects the session credential\n        \
         error: keypad.dial.UnsupportedKey\n",
    );
    let output = validate(&keypad(&dial));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stdout}");
    assert!(
        format!("{stdout}{stderr}").contains("unknown field `outcomes`"),
        "{stdout}\n{stderr}"
    );
}
