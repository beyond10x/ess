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
            Self::Run => command.args(["run", "--target", "billing", "--report-format", "2"]),
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
        assert_eq!(output.stdout.len(), 0);
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
            assert_eq!(output.stdout.len(), 0);
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
        // A run states a fresh suite's initial-state requirement before the target runs
        // (docs/design/scenario-initial-state-and-cross-caller-witnesses.md, #312); nothing else.
        if matches!(operation, Operation::Run) {
            assert_eq!(
                String::from_utf8_lossy(&output.stderr),
                "Requires an empty logical modeled-instance/event/invocation namespace before each \
                 scenario setup; unrelated physical data need not be deleted.\n"
            );
        } else {
            assert_eq!(output.stderr.len(), 0);
        }
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

const CHECKPOINT: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/nested-response-checkpoint.yaml");
const CHECKPOINT_SCENARIO: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/nested-response-checkpoint-scenario.yaml"
);

/// A brand-free project: the stop/checkpoint model, its `ess-inputs.yaml` and one scenario.
fn checkpoint_project(scenario: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("scenarios")).unwrap();
    fs::write(dir.path().join("system.yaml"), CHECKPOINT).unwrap();
    fs::write(
        dir.path().join("ess-inputs.yaml"),
        "format: ess-inputs/1\nspecification:\n  - system.yaml\nscenarios:\n  - scenarios/defer.yaml\n",
    )
    .unwrap();
    fs::write(dir.path().join("scenarios/defer.yaml"), scenario).unwrap();
    dir
}

fn ess_in(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(directory)
        .output()
        .unwrap()
}

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// story:an-authored-claim-may-compare-a-nested-field. A response claim naming one member of a
/// nested struct by its path validates with no refusal, synthesizes beside the generated
/// scenarios, and every scenario passes on the interpreted target, which carries members the claim
/// leaves out. The same claim written as a partial map is refused ESS-AUTHOR-014 naming the
/// members it leaves out.
#[test]
fn a_nested_response_member_claim_validates_and_synthesizes() {
    let project = checkpoint_project(CHECKPOINT_SCENARIO);
    let root = project.path();

    let validated = ess_in(root, &["specify", "validate", "--format", "json"]);
    let said = shown(&validated);
    assert_eq!(validated.status.code(), Some(0), "{said}");
    let report: serde_json::Value = serde_json::from_slice(&validated.stdout).unwrap();
    assert_eq!(report["valid"], true, "{said}");
    assert_eq!(report["scenarios"], 1, "{said}");
    assert!(report.get("scenario_refusals").is_none(), "{said}");

    let synthesized = ess_in(
        root,
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
    assert_eq!(synthesized.status.code(), Some(0), "{said}");
    let suite = fs::read_to_string(root.join("suite.json")).unwrap();
    assert!(suite.contains("\"ess-conformance/62\""), "{suite}");
    let compact = serde_json::from_str::<serde_json::Value>(&suite)
        .unwrap()
        .to_string();
    assert!(
        compact.contains("\"expected\":{\"checkpoint.checkpoint_id\":\"cp-1\"}"),
        "{suite}"
    );

    let ran = ess_in(
        root,
        &[
            "verify",
            "conform",
            "run",
            "--target",
            "interpreted",
            "--path",
            "system.yaml",
            "--report-format",
            "2",
            "--suite",
            "suite.json",
        ],
    );
    let said = shown(&ran);
    assert_eq!(ran.status.code(), Some(0), "{said}");
    assert!(
        said.contains("passed relay.loop/authored/defer-returns-its-checkpoint"),
        "{said}"
    );
    println!("{said}");

    let partial = checkpoint_project(&CHECKPOINT_SCENARIO.replace(
        "response: {checkpoint.checkpoint_id: cp-1}",
        "response: {checkpoint: {checkpoint_id: cp-1}}",
    ));
    let validated = ess_in(partial.path(), &["specify", "validate"]);
    let said = shown(&validated);
    assert_eq!(validated.status.code(), Some(1), "{said}");
    assert_eq!(said.matches("refusal[ESS-AUTHOR-014]").count(), 2, "{said}");
    for member in ["`format`", "`call`", "at `checkpoint`"] {
        assert!(said.contains(member), "{member}: {said}");
    }
    assert!(!said.contains("invalid_input"), "{said}");
}

/// A command whose input is a code over a three-letter alphabet.
const TYPED_INPUT_MODEL: &str = r#"format: ess/32
system: demo
version: v1
domain: demo.codes
types:
  - name: demo.codes.Code
    kind: newtype
    of: String
    alphabet: "abc"
events:
  - name: demo.codes.Recorded
    fields: [{name: code, type: demo.codes.Code}]
commands:
  - name: demo.codes.Record
    input:
      - {name: code, type: demo.codes.Code}
    outcomes:
      - name: recorded
        emits: [demo.codes.Recorded]
        payload:
          demo.codes.Recorded: {code: input.code}
"#;

/// A brand-free project: the code model, its `ess-inputs.yaml` and one scenario sending `code`.
fn typed_input_project(format: &str, code: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("scenarios")).unwrap();
    fs::write(dir.path().join("system.yaml"), TYPED_INPUT_MODEL).unwrap();
    fs::write(
        dir.path().join("ess-inputs.yaml"),
        "format: ess-inputs/1\nspecification:\n  - system.yaml\nscenarios:\n  - scenarios/record.yaml\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("scenarios/record.yaml"),
        format!(
            "type: {format}\ndomain: demo.codes\nscenario: record-a-code\nsummary: One code is \
             recorded.\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    command: demo.codes.Record\n    \
             input: {{code: {code}}}\n    outcome: recorded\n"
        ),
    )
    .unwrap();
    dir
}

/// story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type. From
/// `ess-scenario/6`, `ess specify validate` refuses an authored act whose literal input its declared
/// type refuses as ESS-AUTHOR-015 naming the field and the alphabet; the same file below `/6`, and
/// an in-type input at `/6`, validate.
#[test]
fn an_out_of_type_authored_input_is_refused_by_author() {
    let story = "story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type";
    let refused = typed_input_project("ess-scenario/6", "abd");
    let validated = ess_in(refused.path(), &["specify", "validate"]);
    let said = shown(&validated);
    assert_eq!(validated.status.code(), Some(1), "{story}: {said}");
    assert_eq!(
        said.matches("refusal[ESS-AUTHOR-015]").count(),
        1,
        "{story}: {said}"
    );
    for named in ["`code`", "not in the alphabet of demo.codes.Code"] {
        assert!(said.contains(named), "{story}: {named}: {said}");
    }
    for (format, code) in [("ess-scenario/5", "abd"), ("ess-scenario/6", "cab")] {
        let project = typed_input_project(format, code);
        let validated = ess_in(project.path(), &["specify", "validate"]);
        let said = shown(&validated);
        assert_eq!(
            validated.status.code(),
            Some(0),
            "{story}: {format} {code}: {said}"
        );
    }
}

#[path = "support/input_discovery_cases.rs"]
mod input_discovery;
