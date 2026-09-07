//! An explicit empty selection cannot masquerade as an authored conformance run.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = root().join(format!(
            "target/review-boundaries-6/authored-scenarios-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn directory(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn scenario(directory: &Path, file: &str, name: &str) -> PathBuf {
        let path = directory.join(file);
        let text = fs::read_to_string(
            root().join("examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml"),
        )
        .unwrap()
        .replace(
            "scenario: outstanding-invoices-rank-latest-first",
            &format!("scenario: {name}"),
        );
        fs::write(&path, text).unwrap();
        path
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Ir,
    Go,
    Author,
    Web,
    Run,
}

const OPERATIONS: [Operation; 5] = [
    Operation::Ir,
    Operation::Go,
    Operation::Author,
    Operation::Web,
    Operation::Run,
];

impl Operation {
    fn command(self, scenarios: Option<&Path>, out: Option<&Path>) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command.current_dir(root()).args(["verify", "conform"]);
        match self {
            Self::Ir => command.args(["synthesize", "--target", "ir"]),
            Self::Go => command.args(["synthesize", "--target", "go"]),
            Self::Author => command.arg("author"),
            Self::Web => command.arg("web"),
            Self::Run => command.args(["run", "--target", "billing"]),
        };
        command
            .arg("--path")
            .arg(root().join("examples/billing"))
            .args(["--format", "json"]);
        if let Some(scenarios) = scenarios {
            command.arg("--scenarios").arg(scenarios);
        }
        if let Some(out) = out {
            command
                .arg(if matches!(self, Self::Run) {
                    "--report-out"
                } else {
                    "--out"
                })
                .arg(out);
        }
        command
    }

    fn destination(self, fixture: &Fixture, existing: bool) -> PathBuf {
        let out = fixture.0.join("out");
        if existing {
            if matches!(self, Self::Go | Self::Web) {
                fs::create_dir_all(out.join("essconform")).unwrap();
                fs::write(out.join("essconform/suite.json"), "owned suite\n").unwrap();
                fs::write(out.join("model.json"), "owned model\n").unwrap();
                fs::write(out.join("sentinel"), "owned sentinel\n").unwrap();
            } else {
                fs::write(&out, "owned output\n").unwrap();
            }
        }
        out
    }
}

/// Include directories, so a refusal cannot leave an empty output tree behind either.
fn inventory(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, path: &Path, found: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        if path.is_dir() {
            found.insert(relative, None);
            for entry in fs::read_dir(path).unwrap() {
                visit(root, &entry.unwrap().path(), found);
            }
        } else {
            found.insert(relative, Some(fs::read(path).unwrap()));
        }
    }
    let mut found = BTreeMap::new();
    if root.exists() {
        visit(root, root, &mut found);
    }
    found
}

fn successful(command: &mut Command) -> Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[derive(Clone, Copy, Debug)]
enum EmptySelection {
    Empty,
    Nested,
    Nonmatching,
}

fn rejects_empty(operation: Operation, selection: EmptySelection) {
    for destination in ["absent", "existing", "omitted"] {
        let fixture = Fixture::new();
        let selected = fixture.directory("selected");
        match selection {
            EmptySelection::Empty => {}
            EmptySelection::Nested => {
                let child = fixture.directory("selected/child");
                Fixture::scenario(&child, "scenario.yaml", "nested");
            }
            EmptySelection::Nonmatching => {
                fs::write(selected.join("README.md"), "No immediate YAML files.\n").unwrap();
                Fixture::scenario(&selected, "scenario.json", "unselected-json");
                Fixture::scenario(&selected, "scenario.YAML", "unselected-uppercase");
            }
        }
        let out = operation.destination(&fixture, destination == "existing");
        let before = inventory(&fixture.0);
        let mut command = operation.command(
            Some(&selected),
            (destination != "omitted").then_some(out.as_path()),
        );
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{command:?}: {output:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("refus"), "{command:?}: {error}");
        assert!(error.contains(selected.to_str().unwrap()), "{error}");
        assert!(error.contains(".yaml") && error.contains(".yml"), "{error}");
        assert!(error.contains("directly"), "{error}");
        assert!(error.contains("--scenarios"), "{error}");
        if matches!(selection, EmptySelection::Nested) {
            assert!(error.contains("subdirector"), "{error}");
        }
        assert!(
            output.stdout.is_empty(),
            "no suite or runner report: {output:?}"
        );
        assert_eq!(inventory(&fixture.0), before, "{command:?}");
    }
}

macro_rules! empty_cases {
    ($operation:ident, $empty:ident, $nested:ident, $nonmatching:ident) => {
        #[test]
        fn $empty() {
            rejects_empty(Operation::$operation, EmptySelection::Empty);
        }
        #[test]
        fn $nested() {
            rejects_empty(Operation::$operation, EmptySelection::Nested);
        }
        #[test]
        fn $nonmatching() {
            rejects_empty(Operation::$operation, EmptySelection::Nonmatching);
        }
    };
}

empty_cases!(Ir, ir_empty, ir_nested_only, ir_nonmatching_only);
empty_cases!(Go, go_empty, go_nested_only, go_nonmatching_only);
empty_cases!(
    Author,
    author_empty,
    author_nested_only,
    author_nonmatching_only
);
empty_cases!(Web, web_empty, web_nested_only, web_nonmatching_only);
empty_cases!(Run, run_empty, run_nested_only, run_nonmatching_only);

#[test]
fn a_yaml_named_subdirectory_alone_retains_its_read_refusal() {
    for operation in OPERATIONS {
        let fixture = Fixture::new();
        let selected = fixture.directory("selected");
        fixture.directory("selected/child.yaml");
        let output = operation.command(Some(&selected), None).output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("reading") && error.contains("child.yaml"),
            "{error}"
        );
        assert!(output.stdout.is_empty());
    }
}

fn matching_directory_refusal(symlink: bool) {
    for operation in OPERATIONS {
        for existing in [false, true] {
            let fixture = Fixture::new();
            let selected = fixture.directory("selected");
            Fixture::scenario(&selected, "one.yml", "selected");
            let directory = selected.join("child.yaml");
            if symlink {
                #[cfg(unix)]
                std::os::unix::fs::symlink(fixture.directory("child-target"), &directory).unwrap();
            } else {
                fs::create_dir(&directory).unwrap();
            }
            let out = operation.destination(&fixture, existing);
            let before = inventory(&out);
            let output = operation
                .command(Some(&selected), Some(&out))
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1), "{operation:?}: {output:?}");
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains("reading") && error.contains("child.yaml"),
                "{error}"
            );
            assert!(output.stdout.is_empty());
            assert_eq!(inventory(&out), before);
        }
    }
}

#[test]
fn a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal() {
    matching_directory_refusal(false);
}

#[test]
#[cfg(unix)]
fn a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal() {
    matching_directory_refusal(true);
}

#[test]
fn explicit_files_and_shallow_directories_select_the_same_scenario() {
    for operation in OPERATIONS {
        let fixture = Fixture::new();
        let selected = fixture.directory("selected");
        let file = Fixture::scenario(&selected, "one.yaml", "selected");
        let out = operation.destination(&fixture, false);
        let baseline = successful(&mut operation.command(Some(&file), Some(&out)));
        let baseline_files = inventory(&out);
        let json = Fixture::scenario(&fixture.0, "one.json", "selected");
        let yml = Fixture::scenario(&fixture.0, "one.yml", "selected");
        // An invalid nested document detects accidental recursion more strongly than a valid one.
        let child = fixture.directory("selected/child");
        fs::write(child.join("invalid.yaml"), "not a scenario\n").unwrap();
        for path in [&selected, &json, &yml] {
            let output = successful(&mut operation.command(Some(path), Some(&out)));
            assert_eq!(output.stdout, baseline.stdout, "{operation:?}, {path:?}");
            assert_eq!(output.stderr, baseline.stderr, "{operation:?}, {path:?}");
            assert_eq!(inventory(&out), baseline_files, "{operation:?}, {path:?}");
        }
    }
}

#[test]
fn shallow_yaml_and_yml_selection_is_independent_of_creation_order() {
    let fixture = Fixture::new();
    let first = fixture.directory("first");
    let second = fixture.directory("second");
    for (directory, entries) in [
        (&first, [("b.yml", "second"), ("a.yaml", "first")]),
        (&second, [("a.yaml", "first"), ("b.yml", "second")]),
    ] {
        for (file, name) in entries {
            Fixture::scenario(directory, file, name);
        }
    }
    for operation in OPERATIONS {
        let operation_fixture = Fixture::new();
        let out = operation.destination(&operation_fixture, false);
        let first_output = successful(&mut operation.command(Some(&first), Some(&out)));
        let first_files = inventory(&out);
        let second_output = successful(&mut operation.command(Some(&second), Some(&out)));
        assert_eq!(second_output.stdout, first_output.stdout, "{operation:?}");
        assert_eq!(second_output.stderr, first_output.stderr, "{operation:?}");
        assert_eq!(inventory(&out), first_files, "{operation:?}");
    }
}

#[test]
fn omitted_scenarios_preserve_intentional_generated_and_authored_selections() {
    for operation in OPERATIONS {
        let fixture = Fixture::new();
        let out = operation.destination(&fixture, false);
        let output = successful(&mut operation.command(None, Some(&out)));
        assert!(output.stderr.is_empty());
        let files = inventory(&out);
        assert!(!files.is_empty(), "{operation:?}");
        if matches!(operation, Operation::Ir | Operation::Author) {
            let suite = ess_conformance::ConformanceSuite::from_json(
                std::str::from_utf8(&output.stdout).unwrap(),
            )
            .unwrap();
            assert_eq!(suite.is_empty(), matches!(operation, Operation::Author));
            assert!(!String::from_utf8_lossy(&output.stdout).contains("/authored/"));
        }
        let repeated = successful(&mut operation.command(None, Some(&out)));
        assert_eq!(repeated.stdout, output.stdout, "{operation:?}");
        assert_eq!(inventory(&out), files, "{operation:?}");
    }
}

#[test]
fn supplied_suite_bypasses_empty_and_nonexistent_scenario_paths() {
    let fixture = Fixture::new();
    let suite = fixture.0.join("suite.json");
    successful(&mut Operation::Ir.command(None, Some(&suite)));
    let empty = fixture.directory("empty");
    let missing = fixture.0.join("missing");
    let baseline = successful(
        Operation::Run
            .command(None, None)
            .arg("--suite")
            .arg(&suite),
    );
    for path in [&empty, &missing] {
        let output = successful(
            Operation::Run
                .command(Some(path), None)
                .arg("--suite")
                .arg(&suite),
        );
        assert_eq!(output.status, baseline.status);
        assert_eq!(output.stdout, baseline.stdout);
        assert_eq!(output.stderr, baseline.stderr);
    }
}

#[test]
fn missing_paths_and_malformed_matching_sources_remain_failures() {
    let fixture = Fixture::new();
    let missing = fixture.0.join("missing");
    let malformed = fixture.directory("malformed");
    fs::write(malformed.join("bad.yaml"), "type: [unterminated\n").unwrap();
    for operation in OPERATIONS {
        for path in [&missing, &malformed, &malformed.join("bad.yaml")] {
            let output = operation.command(Some(path), None).output().unwrap();
            assert_eq!(output.status.code(), Some(1), "{operation:?}: {output:?}");
        }
    }
}

#[test]
fn empty_selection_refusal_is_identical_through_flat_and_area_spellings() {
    let fixture = Fixture::new();
    let empty = fixture.directory("empty");
    let mut outputs = Vec::new();
    for prefix in [&["conform"][..], &["verify", "conform"][..]] {
        outputs.push(
            Command::new(env!("CARGO_BIN_EXE_ess"))
                .args(prefix)
                .args(["synthesize", "--path"])
                .arg(root().join("examples/billing"))
                .arg("--scenarios")
                .arg(&empty)
                .output()
                .unwrap(),
        );
    }
    assert_eq!(outputs[0].status.code(), Some(1));
    assert_eq!(outputs[0].status, outputs[1].status);
    assert_eq!(outputs[0].stdout, outputs[1].stdout);
    assert_eq!(outputs[0].stderr, outputs[1].stderr);
}

#[test]
fn synthesize_help_describes_explicit_shallow_selection_without_a_default() {
    let output = successful(Command::new(env!("CARGO_BIN_EXE_ess")).args([
        "verify",
        "conform",
        "synthesize",
        "--help",
    ]));
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(!help.contains("Defaults to `scenarios/`"), "{help}");
    assert!(help.contains("no authored scenarios"), "{help}");
    assert!(help.contains("subdirectories"), "{help}");
}

#[path = "support/input_discovery_cases.rs"]
mod input_discovery;
