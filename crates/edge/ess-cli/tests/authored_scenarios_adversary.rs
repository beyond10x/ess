//! Adversarial filesystem and dispatch boundaries for explicit authored selection.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = root().join(format!(
            "target/review-boundaries-6/adversary-fixture-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn directory(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn run(&self, command: &mut Command) -> Output {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let record = self.directory(&format!("command-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::write(record.join("command.txt"), format!("{command:?}\n")).unwrap();
        let output = command.output().unwrap();
        fs::write(record.join("stdout"), &output.stdout).unwrap();
        fs::write(record.join("stderr"), &output.stderr).unwrap();
        fs::write(record.join("exit"), format!("{}\n", output.status)).unwrap();
        output
    }
}

fn scenario(path: &Path) {
    fs::copy(
        root().join("examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml"),
        path,
    )
    .unwrap();
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Ir,
    Go,
    Author,
    Web,
    Billing,
    Oracle,
}

const OPERATIONS: [Operation; 6] = [
    Operation::Ir,
    Operation::Go,
    Operation::Author,
    Operation::Web,
    Operation::Billing,
    Operation::Oracle,
];

fn command(operation: Operation, flat: bool, format: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command.current_dir(root());
    if !flat {
        command.arg("verify");
    }
    command.arg("conform");
    match operation {
        Operation::Ir => command.args(["synthesize", "--target", "ir"]),
        Operation::Go => command.args(["synthesize", "--target", "go"]),
        Operation::Author => command.arg("author"),
        Operation::Web => command.arg("web"),
        Operation::Billing => command.args(["run", "--target", "billing"]),
        Operation::Oracle => command.args(["run", "--target", "oracle-fixture"]),
    };
    command
        .arg("--path")
        .arg(root().join("examples/billing"))
        .args(["--format", format]);
    command
}

fn with_out(command: &mut Command, operation: Operation, out: &Path) {
    command
        .arg(
            if matches!(operation, Operation::Billing | Operation::Oracle) {
                "--report-out"
            } else {
                "--out"
            },
        )
        .arg(out);
}

fn refused(output: &Output, required: &[&str]) {
    let error = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "stderr: {error}");
    for word in required {
        assert!(error.contains(word), "expected {word:?} in {error:?}");
    }
    assert!(
        output.stdout.is_empty(),
        "unexpected suite or runner output"
    );
}

fn same_output(left: &Output, right: &Output) {
    assert_eq!(left.status, right.status);
    assert_eq!(left.stderr, right.stderr);
    assert_eq!(left.stdout, right.stdout);
}

#[test]
fn empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner() {
    let fixture = Fixture::new();
    let empty = fixture.directory("empty selection");
    for operation in OPERATIONS {
        for flat in [false, true] {
            for format in ["text", "json", "yaml"] {
                let mut command = command(operation, flat, format);
                command.arg("--scenarios").arg(&empty);
                // File writers would refuse this directory; tree writers could populate it.
                with_out(&mut command, operation, &empty);
                let output = fixture.run(&mut command);
                refused(
                    &output,
                    &["refused --scenarios", "directly", ".yaml", ".yml"],
                );
                assert!(fs::read_dir(&empty).unwrap().next().is_none());
            }
        }
    }
}

#[test]
#[cfg(unix)]
fn selected_directory_symlink_reports_the_requested_path_and_stays_shallow() {
    let fixture = Fixture::new();
    let child = fixture.directory("corpus/child");
    scenario(&child.join("valid.yaml"));
    let selected = fixture.0.join("chosen corpus");
    std::os::unix::fs::symlink(fixture.0.join("corpus"), &selected).unwrap();
    for operation in OPERATIONS {
        let output = fixture.run(
            command(operation, false, "text")
                .arg("--scenarios")
                .arg(&selected),
        );
        refused(
            &output,
            &[
                "refused --scenarios",
                selected.to_str().unwrap(),
                "subdirectories",
                "not searched",
                "child directory",
            ],
        );
        assert_eq!(fs::read_dir(&child).unwrap().count(), 1);
    }
}

#[test]
#[cfg(unix)]
fn empty_selection_cannot_follow_or_replace_an_output_symlink() {
    let fixture = Fixture::new();
    let empty = fixture.directory("empty");
    let sentinel = fixture.0.join("sentinel");
    fs::write(&sentinel, b"owned bytes\0\xff").unwrap();
    let out = fixture.0.join("out");
    std::os::unix::fs::symlink(&sentinel, &out).unwrap();
    for operation in OPERATIONS {
        let mut command = command(operation, true, "json");
        command.arg("--scenarios").arg(&empty);
        with_out(&mut command, operation, &out);
        refused(
            &fixture.run(&mut command),
            &["refused --scenarios", "directly"],
        );
        assert_eq!(fs::read(&sentinel).unwrap(), b"owned bytes\0\xff");
        assert_eq!(fs::read_link(&out).unwrap(), sentinel);
    }
}

#[test]
#[cfg(unix)]
fn matching_broken_links_are_read_errors_even_beside_a_valid_source() {
    for with_valid in [false, true] {
        let fixture = Fixture::new();
        let selected = fixture.directory("selected");
        if with_valid {
            scenario(&selected.join("valid.yaml"));
        }
        std::os::unix::fs::symlink("missing", selected.join("broken.yml")).unwrap();
        for operation in OPERATIONS {
            let out = fixture.0.join("new-output");
            let mut command = command(operation, false, "json");
            command.arg("--scenarios").arg(&selected);
            with_out(&mut command, operation, &out);
            refused(&fixture.run(&mut command), &["reading", "broken.yml"]);
            assert!(!out.exists());
        }
    }
}

#[test]
#[cfg(unix)]
fn lexical_selection_order_decides_the_first_read_error() {
    let fixture = Fixture::new();
    for (name, order) in [
        ("reverse", ["z-last.yaml", "a-first.yml"]),
        ("forward", ["a-first.yml", "z-last.yaml"]),
    ] {
        let selected = fixture.directory(name);
        for file in order {
            std::os::unix::fs::symlink("missing", selected.join(file)).unwrap();
        }
        for operation in OPERATIONS {
            let output = fixture.run(
                command(operation, false, "text")
                    .arg("--scenarios")
                    .arg(&selected),
            );
            refused(&output, &["reading", "a-first.yml"]);
            assert!(!String::from_utf8_lossy(&output.stderr).contains("z-last.yaml"));
        }
    }
}

#[test]
fn non_utf8_matching_content_is_refused_before_any_output_write() {
    let fixture = Fixture::new();
    let selected = fixture.directory("selected");
    scenario(&selected.join("a-valid.yaml"));
    fs::write(selected.join("z-invalid.yml"), [0xff, 0xfe]).unwrap();
    for operation in OPERATIONS {
        let out = fixture.0.join("owned-output");
        fs::write(&out, "previous output").unwrap();
        let mut command = command(operation, false, "json");
        command.arg("--scenarios").arg(&selected);
        with_out(&mut command, operation, &out);
        refused(&fixture.run(&mut command), &["reading", "z-invalid.yml"]);
        assert_eq!(fs::read_to_string(&out).unwrap(), "previous output");
    }
}

#[test]
#[cfg(unix)]
fn valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links() {
    let fixture = Fixture::new();
    let source = fixture.0.join("extensionless");
    scenario(&source);
    let direct = fixture.0.join("chosen.scenario");
    std::os::unix::fs::symlink(&source, &direct).unwrap();
    let selected = fixture.directory("selected");
    std::os::unix::fs::symlink(&source, selected.join("one.yml")).unwrap();
    std::os::unix::fs::symlink("loop", selected.join("loop")).unwrap();
    let nested = fixture.directory("nested");
    fs::write(nested.join("bad.yaml"), "invalid scenario").unwrap();
    std::os::unix::fs::symlink(&nested, selected.join("child")).unwrap();
    fs::write(selected.join("ignored.YML"), [0xff, 0xfe]).unwrap();
    for operation in OPERATIONS {
        let expected = fixture.run(
            command(operation, false, "json")
                .arg("--scenarios")
                .arg(&source),
        );
        if !matches!(operation, Operation::Oracle) {
            assert!(expected.status.success());
        }
        for path in [&direct, &selected] {
            let actual = fixture.run(
                command(operation, false, "json")
                    .arg("--scenarios")
                    .arg(path),
            );
            same_output(&actual, &expected);
        }
    }
}

#[test]
fn omitted_scenarios_ignore_a_poisoned_working_directory_default() {
    let fixture = Fixture::new();
    let poisoned = fixture.directory("scenarios");
    fs::write(poisoned.join("bad.yaml"), "type: [unterminated").unwrap();
    for operation in OPERATIONS {
        let expected = fixture.run(&mut command(operation, false, "json"));
        let actual = fixture.run(command(operation, false, "json").current_dir(&fixture.0));
        same_output(&actual, &expected);
    }
}

#[test]
#[cfg(unix)]
fn committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners() {
    let fixture = Fixture::new();
    let suite = fixture.0.join("suite.json");
    let generated = fixture.run(
        command(Operation::Ir, false, "json")
            .arg("--out")
            .arg(&suite),
    );
    assert!(generated.status.success());
    let malformed = fixture.0.join("bad.yaml");
    fs::write(&malformed, [0xff, 0xfe]).unwrap();
    let loop_path = fixture.0.join("loop");
    std::os::unix::fs::symlink("loop", &loop_path).unwrap();
    let directory = fixture.directory("bad-directory");
    fs::create_dir(directory.join("child.yml")).unwrap();
    for operation in [Operation::Billing, Operation::Oracle] {
        let expected = fixture.run(command(operation, false, "json").arg("--suite").arg(&suite));
        for path in [&malformed, &loop_path, &directory] {
            let mut actual = Command::new(env!("CARGO_BIN_EXE_ess"));
            actual
                .args(["conform", "run", "--target"])
                .arg(if matches!(operation, Operation::Billing) {
                    "billing"
                } else {
                    "oracle-fixture"
                })
                .args(["--format", "json", "--path"])
                .arg(fixture.0.join("missing-model"))
                .arg("--suite")
                .arg(&suite)
                .arg("--scenarios")
                .arg(path);
            same_output(&fixture.run(&mut actual), &expected);
        }
    }
}
