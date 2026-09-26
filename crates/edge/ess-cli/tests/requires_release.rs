//! A specification names the `ess` release it is maintained with (ess#106).
//!
//! `requires:` in an `ess-inputs/2` manifest: an older `ess` refuses and says how to upgrade, a
//! newer one warns once per command and continues unless `--strict-requires`, and a manifest without
//! it behaves exactly as before. Generated output records the producing release in its
//! `.ess-output` checkpoint, and a changing regeneration by another release says so.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const THIS: &str = env!("CARGO_PKG_VERSION");
const MODEL: [&str; 3] = ["system.yaml", "domains/invoice.yaml", "domains/email.yaml"];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

/// This release's (major, minor, patch).
fn this() -> (u64, u64, u64) {
    let mut parts = THIS
        .split(['.', '-', '+'])
        .map(|part| part.parse::<u64>().unwrap());
    (
        parts.next().unwrap(),
        parts.next().unwrap(),
        parts.next().unwrap(),
    )
}

struct Fixture(PathBuf);

impl Fixture {
    /// The billing model under `spec/`, with `ess-inputs.yaml` carrying `head` above its lists.
    fn new(head: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = repo().join(format!(
            "target/requires-release/{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("spec/domains")).unwrap();
        for file in MODEL {
            fs::copy(
                repo().join("examples/billing").join(file),
                root.join("spec").join(file),
            )
            .unwrap();
        }
        let fixture = Self(root);
        fixture.manifest(head);
        fixture
    }

    fn manifest(&self, head: &str) {
        let listed = MODEL
            .iter()
            .map(|file| format!("\"{file}\""))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            self.0.join("spec/ess-inputs.yaml"),
            format!("{head}specification: [{listed}]\nscenarios: []\n"),
        )
        .unwrap();
    }

    /// The billing example's authored scenario, listed under `scenarios`.
    fn list_a_scenario(&self, head: &str) {
        fs::copy(
            repo().join("examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml"),
            self.0.join("spec/scenario.yaml"),
        )
        .unwrap();
        self.manifest(head);
        let path = self.0.join("spec/ess-inputs.yaml");
        let text = fs::read_to_string(&path).unwrap();
        fs::write(
            &path,
            text.replace("scenarios: []", "scenarios: [\"scenario.yaml\"]"),
        )
        .unwrap();
    }

    fn ess(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const VALIDATE: &[&str] = &["specify", "validate", "--path", "spec"];

#[test]
fn an_older_ess_refuses_an_exact_or_line_requirement_and_says_how_to_upgrade() {
    let (major, minor, patch) = this();
    for required in [
        format!("{major}.{minor}.{}", patch + 1),
        format!("{major}.{}", minor + 1),
        format!("{}.0.0", major + 1),
    ] {
        let fixture = Fixture::new(&format!("format: ess-inputs/2\nrequires: ess {required}\n"));
        let output = fixture.ess(VALIDATE);
        let err = stderr(&output);
        assert_eq!(output.status.code(), Some(1), "{required}: {err}");
        assert!(
            err.contains(&format!("requires ess {required}")),
            "{required}: {err}"
        );
        assert!(err.contains(&format!("this is ess {THIS}")), "{err}");
        assert!(err.contains("b10x upgrade"), "{err}");
        assert!(err.contains("/ess:upgrade"), "{err}");
        assert!(output.stdout.is_empty(), "{required}: nothing validated");
    }
}

#[test]
fn a_newer_ess_warns_once_per_command_and_continues() {
    let (major, minor, patch) = this();
    let mut older = vec![format!("{major}.{minor}.{}", patch.saturating_sub(1))];
    if minor > 0 {
        older.push(format!("{major}.{}", minor - 1));
    }
    if patch == 0 {
        older.remove(0);
    }
    assert!(!older.is_empty(), "no older release to require");
    for required in older {
        // A nonempty `scenarios` list makes `validate` acquire through the manifest twice: once
        // for the specification and once for the scenarios.
        let fixture = Fixture::new(&format!("format: ess-inputs/2\nrequires: ess {required}\n"));
        fixture.list_a_scenario(&format!("format: ess-inputs/2\nrequires: ess {required}\n"));
        let output = fixture.ess(VALIDATE);
        let err = stderr(&output);
        assert_eq!(output.status.code(), Some(0), "{required}: {err}");
        assert_eq!(
            err.matches(&format!("requires ess {required}")).count(),
            1,
            "{required}: {err}"
        );
        assert!(err.contains("warning:"), "{err}");
        assert!(err.contains("--strict-requires"), "{err}");
        assert!(
            String::from_utf8_lossy(&output.stdout).contains("valid"),
            "{required}"
        );

        for position in [
            &["--strict-requires", "specify", "validate", "--path", "spec"][..],
            &["specify", "validate", "--path", "spec", "--strict-requires"][..],
        ] {
            let strict = fixture.ess(position);
            let err = stderr(&strict);
            assert_eq!(strict.status.code(), Some(1), "{position:?}: {err}");
            assert!(err.contains(&format!("requires ess {required}")), "{err}");
            assert!(strict.stdout.is_empty());
        }
    }
}

#[test]
fn a_satisfied_requirement_or_none_at_all_says_nothing() {
    let (major, minor, _) = this();
    for head in [
        format!("format: ess-inputs/2\nrequires: ess {THIS}\n"),
        format!("format: ess-inputs/2\nrequires: ess {major}.{minor}\n"),
        "format: ess-inputs/2\n".to_owned(),
        "format: ess-inputs/1\n".to_owned(),
    ] {
        let fixture = Fixture::new(&head);
        for args in [
            VALIDATE,
            &["--strict-requires", "specify", "validate", "--path", "spec"],
        ] {
            let output = fixture.ess(args);
            assert_eq!(output.status.code(), Some(0), "{head}: {}", stderr(&output));
            assert!(output.stderr.is_empty(), "{head}: {}", stderr(&output));
        }
    }
}

#[test]
fn a_malformed_or_misplaced_requirement_refuses() {
    for (head, expected) in [
        ("format: ess-inputs/2\nrequires: ess latest\n", "ess X.Y.Z"),
        ("format: ess-inputs/2\nrequires: ess >=0.30\n", "ess X.Y.Z"),
        ("format: ess-inputs/2\nrequires: aep 0.32.0\n", "ess X.Y.Z"),
        (
            "format: ess-inputs/2\nrequires: ess 0.32.0-rc1\n",
            "ess X.Y.Z",
        ),
        ("format: ess-inputs/2\nrequires: ess 00.32\n", "ess X.Y.Z"),
        ("format: ess-inputs/2\nrequires: 32\n", "expected a string"),
        (
            "format: ess-inputs/1\nrequires: ess 0.1.0\n",
            "ess-inputs/2",
        ),
    ] {
        let fixture = Fixture::new(head);
        let output = fixture.ess(VALIDATE);
        let err = stderr(&output);
        assert_eq!(output.status.code(), Some(1), "{head}: {err}");
        assert!(err.contains(expected), "{head}: {err}");
    }
}

fn state(root: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join(".ess-output/state.json")).unwrap()).unwrap()
}

/// Rewrite the checkpoint's payload and its checksum, as a different release would have left it.
fn rewrite_state(root: &Path, edit: impl FnOnce(&mut serde_json::Value)) {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;
    let mut value = state(root);
    edit(&mut value["payload"]);
    let mut payload = serde_json::to_vec(&value["payload"]).unwrap();
    payload.push(b'\n');
    let mut checksum = String::new();
    for byte in Sha256::digest(&payload) {
        write!(&mut checksum, "{byte:02x}").unwrap();
    }
    value["checksum"] = checksum.into();
    let mut bytes = serde_json::to_vec(&value).unwrap();
    bytes.push(b'\n');
    fs::write(root.join(".ess-output/state.json"), bytes).unwrap();
}

/// Change what the docs projection renders, so the next publication is not a no-op.
fn edit_summary(fixture: &Fixture, marker: &str) {
    let path = fixture.0.join("spec/system.yaml");
    let text = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        text.replacen("summary: >-\n", &format!("summary: >-\n  {marker}\n"), 1),
    )
    .unwrap();
}

const GENERATE: &[&str] = &[
    "generate", "--path", "spec", "--kind", "docs", "--out", "out",
];

#[test]
fn generated_output_records_its_producer_and_a_changing_regeneration_by_another_release_notes_it() {
    let fixture = Fixture::new("format: ess-inputs/1\n");
    let out = fixture.0.join("out");
    let first = fixture.ess(GENERATE);
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    assert!(first.stderr.is_empty(), "{}", stderr(&first));
    let payload = state(&out)["payload"].clone();
    assert_eq!(payload["format"], "ess-output-state/2");
    assert_eq!(payload["producer"], format!("ess {THIS}"));

    // The same release regenerating a change says nothing.
    edit_summary(&fixture, "First edit.");
    let same = fixture.ess(GENERATE);
    assert_eq!(same.status.code(), Some(0), "{}", stderr(&same));
    assert!(same.stderr.is_empty(), "{}", stderr(&same));

    // Another release left it: a changing regeneration names both, and records this one.
    rewrite_state(&out, |payload| payload["producer"] = "ess 0.0.1".into());
    edit_summary(&fixture, "Second edit.");
    let other = fixture.ess(GENERATE);
    let err = stderr(&other);
    assert_eq!(other.status.code(), Some(0), "{err}");
    assert!(err.contains("note:"), "{err}");
    assert!(err.contains("last generated by ess 0.0.1"), "{err}");
    assert!(err.contains(&format!("this is ess {THIS}")), "{err}");
    assert_eq!(state(&out)["payload"]["producer"], format!("ess {THIS}"));

    // A regeneration that changes nothing writes nothing and says nothing, whoever left it.
    rewrite_state(&out, |payload| payload["producer"] = "ess 0.0.1".into());
    let unchanged = fixture.ess(GENERATE);
    assert_eq!(unchanged.status.code(), Some(0), "{}", stderr(&unchanged));
    assert!(unchanged.stderr.is_empty(), "{}", stderr(&unchanged));
    assert_eq!(state(&out)["payload"]["producer"], "ess 0.0.1");
}

#[test]
fn a_version_one_checkpoint_is_read_and_its_first_changing_publication_records_the_producer() {
    let fixture = Fixture::new("format: ess-inputs/1\n");
    let out = fixture.0.join("out");
    assert_eq!(fixture.ess(GENERATE).status.code(), Some(0));
    rewrite_state(&out, |payload| {
        payload["format"] = "ess-output-state/1".into();
        payload.as_object_mut().unwrap().remove("producer");
    });
    edit_summary(&fixture, "An edit.");
    let output = fixture.ess(GENERATE);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(output.stderr.is_empty(), "{}", stderr(&output));
    let payload = state(&out)["payload"].clone();
    assert_eq!(payload["format"], "ess-output-state/2");
    assert_eq!(payload["producer"], format!("ess {THIS}"));
}

#[test]
fn a_checkpoint_whose_producer_contradicts_its_version_is_refused() {
    for (format, producer) in [
        ("ess-output-state/1", Some("ess 0.0.1")),
        ("ess-output-state/2", None),
        ("ess-output-state/2", Some("")),
    ] {
        let fixture = Fixture::new("format: ess-inputs/1\n");
        let out = fixture.0.join("out");
        assert_eq!(fixture.ess(GENERATE).status.code(), Some(0));
        rewrite_state(&out, |payload| {
            payload["format"] = format.into();
            match producer {
                Some(producer) => payload["producer"] = producer.into(),
                None => {
                    payload.as_object_mut().unwrap().remove("producer");
                }
            }
        });
        edit_summary(&fixture, "An edit.");
        let output = fixture.ess(GENERATE);
        let err = stderr(&output);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{format} {producer:?}: {err}"
        );
        assert!(err.contains("producer"), "{format} {producer:?}: {err}");
    }
}
