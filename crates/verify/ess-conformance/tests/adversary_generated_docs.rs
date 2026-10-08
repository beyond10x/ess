//! Adversary cases for beyond10x/ess#186: does the emitted README say what the emitted runner
//! actually does?
//!
//! The unit's own test (`generated_docs.rs`) compares the README with the version gate *as text*
//! read out of the runtime source. These cases compare the README with the runner's *observed
//! behaviour*: each package is emitted, compiled and run with and without `ESS_REPORT_FORMAT`, and
//! what the README tells an adopter to type is held against what happened.

mod support_scratch;

use std::path::Path;
use std::process::{Command, Output};

use ess_conformance::scenario::{SuiteFormat, SuiteProvenance};
use ess_conformance::ConformanceSuite;
use ess_primitives::evidence::SpecDigest;

fn suite(major: u32) -> ConformanceSuite {
    ConformanceSuite::new(SuiteProvenance {
        suite_version: SuiteFormat::parse(&format!("ess-conformance/{major}")).expect("a version"),
        system: "billing".to_owned(),
        specification_version: "v3".to_owned(),
        spec_digest: SpecDigest::new("ab".repeat(32)).expect("a digest"),
        contract_digest: SpecDigest::new("cd".repeat(32)).expect("a digest"),
        component: None,
        scenario_initial_state: None,
        synthesis_seeds: None,
    })
}

fn go_readme(major: u32) -> String {
    ess_conformance::go::emit(&suite(major))
        .expect("an empty suite emits")
        .into_iter()
        .find(|file| file.path.ends_with("/README.md"))
        .expect("a README")
        .contents
}

fn typescript_readme(major: u32) -> String {
    ess_conformance::ts::emit(&suite(major))
        .expect("an empty suite emits")
        .into_iter()
        .find(|file| file.path.ends_with("README.md"))
        .expect("a README")
        .contents
}

/// Everything before `## The report`: what an adopter reads to run the suite.
fn instructions(readme: &str) -> &str {
    readme
        .split_once("## The report")
        .map_or(readme, |(before, _)| before)
}

/// The bodies of every fenced `console` block in `text`.
fn console_blocks(text: &str) -> Vec<String> {
    text.split("```console\n")
        .skip(1)
        .map(|rest| {
            rest.split_once("```")
                .map_or(rest, |(body, _)| body)
                .to_owned()
        })
        .collect()
}

fn scratch(name: &str) -> support_scratch::Scratch {
    let directory = support_scratch::Scratch::adopt(std::env::temp_dir().join(format!(
        "ess-adversary-generated-docs-{name}-{}",
        std::process::id()
    )));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn write_all(root: &Path, files: Vec<(String, String)>) {
    for file in files {
        let path = root.join(file.0);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, file.1).unwrap();
    }
}

fn log(output: &Output) -> String {
    format!(
        "exit {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// What the emitted runner did with this suite version, without and with `ESS_REPORT_FORMAT=2`.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Observed {
    /// The runner refused the suite document itself (`suite admission`).
    admitted: bool,
    /// The runner refused the suite version itself (`unsupported suite version`).
    version_supported: bool,
    /// Without the variable it stopped with the "require explicit" refusal.
    refused_without_format_2: bool,
    /// With `ESS_REPORT_FORMAT=2` and nothing else it ran to a pass.
    runs_with_format_2: bool,
    logs: String,
}

fn observe(program: &str, args: &[&str], directory: &Path) -> Observed {
    let without = Command::new(program)
        .args(args)
        .env_remove("ESS_REPORT_FORMAT")
        .env_remove("ESS_REPORT_OUT")
        .env("GOWORK", "off")
        .env("GOFLAGS", "-count=1")
        .current_dir(directory)
        .output()
        .expect("the toolchain runs");
    let with = Command::new(program)
        .args(args)
        .env("ESS_REPORT_FORMAT", "2")
        .env_remove("ESS_REPORT_OUT")
        .env("GOWORK", "off")
        .env("GOFLAGS", "-count=1")
        .current_dir(directory)
        .output()
        .expect("the toolchain runs");
    let (without, with) = (log(&without), log(&with));
    Observed {
        admitted: !without.contains("suite admission"),
        version_supported: !without.contains("unsupported suite version"),
        refused_without_format_2: without
            .contains("require explicit ESS_REPORT_FORMAT=2 before execution"),
        runs_with_format_2: with.contains("exit Some(0)"),
        logs: format!(
            "--- without ESS_REPORT_FORMAT\n{without}\n--- with ESS_REPORT_FORMAT=2\n{with}"
        ),
    }
}

/// A target for an empty suite: only `Identity` is ever asked.
const GO_STUB_TARGET: &str = r#"package essconform

import "testing"

type stub struct{}

func (stub) Identity() (Identity, error)             { return Identity{Name: "stub", Version: "1"}, nil }
func (stub) BeginScenario(ScenarioContext) error     { return ErrUnsupported }
func (stub) EndScenario(ScenarioContext) error       { return ErrUnsupported }
func (stub) ExecuteCommand(CommandRequest) (CommandResult, error) {
	return CommandResult{}, ErrUnsupported
}
func (stub) QueryView(ViewRequest) (ViewResult, error) { return ViewResult{}, ErrUnsupported }
func (stub) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func (stub) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (stub) RedeliverEvent(RedeliveryRequest) error               { return ErrUnsupported }
func (stub) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestConformance(t *testing.T) {
	Run(t, func() Target { return stub{} })
}
"#;

fn observe_go(major: u32) -> Observed {
    let root = scratch(&format!("go-{major}"));
    write_all(
        &root,
        ess_conformance::go::emit(&suite(major))
            .unwrap()
            .into_iter()
            .map(|file| (file.path, file.contents))
            .collect(),
    );
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/docs\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(root.join("essconform/conformance_test.go"), GO_STUB_TARGET).unwrap();
    let observed = observe(
        "go",
        &["test", "./essconform", "-run", "^TestConformance$"],
        &root,
    );
    std::fs::remove_dir_all(&root).unwrap();
    observed
}

fn observe_typescript(major: u32) -> Observed {
    let root = scratch(&format!("ts-{major}"));
    write_all(
        &root,
        ess_conformance::ts::emit(&suite(major))
            .unwrap()
            .into_iter()
            .map(|file| (file.path, file.contents))
            .collect(),
    );
    let package = root.join("essconform");
    std::fs::write(
        package.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&package)
        .output()
        .expect("required TypeScript compiler");
    assert!(compiled.status.success(), "{}", log(&compiled));
    std::fs::write(
        package.join("conformance.mjs"),
        "import test from 'node:test';\nimport { run } from './dist/runtime.js';\n\
         const stub = { identity() { return { name: 'stub', version: '1' }; } };\n\
         await test('conformance', (t) => run(t, () => stub));\n",
    )
    .unwrap();
    let observed = observe("node", &["--test", "conformance.mjs"], &package);
    std::fs::remove_dir_all(&root).unwrap();
    observed
}

/// The README's run instructions against the run: they tell the adopter to set the variable
/// exactly when the runner stops without it, and the command they give runs.
fn check(target: &str, major: u32, readme: &str, command: &str, observed: &Observed) {
    let instructions = instructions(readme);
    let claims_required = instructions.contains(&format!("ESS_REPORT_FORMAT=2 {command}"));
    assert_eq!(
        claims_required,
        observed.refused_without_format_2,
        "{target} suite/{major}: the README's run instructions {} `ESS_REPORT_FORMAT=2`, and the \
         runner {} without it.\n{}\n--- README\n{readme}",
        if claims_required {
            "require"
        } else {
            "do not require"
        },
        if observed.refused_without_format_2 {
            "stops"
        } else {
            "runs"
        },
        observed.logs,
    );
    assert!(
        observed.runs_with_format_2,
        "{target} suite/{major}: `ESS_REPORT_FORMAT=2 {command}`, what the README says to run, \
         does not pass on an empty suite:\n{}",
        observed.logs
    );
}

/// Real packages, real toolchains, at the versions the brief names. A version the emitted runner
/// does not admit is left to `*_is_not_emitted_for_a_suite_its_runner_refuses`; once a runtime
/// admits it (beyond10x/ess#188), it is checked here.
#[test]
fn adversary_go_readme_run_instructions_match_what_run_does() {
    for major in [4, 5, 10, 21, 26] {
        let observed = observe_go(major);
        if !observed.admitted {
            continue;
        }
        check("Go", major, &go_readme(major), "go test ./...", &observed);
    }
}

#[test]
fn adversary_typescript_readme_run_instructions_match_what_run_does() {
    for major in [4, 5, 10, 12, 21, 26] {
        let observed = observe_typescript(major);
        if !observed.admitted {
            continue;
        }
        check(
            "TypeScript",
            major,
            &typescript_readme(major),
            "npm test",
            &observed,
        );
    }
}

/// A README is a promise that the package runs. The generator emits a package for every suite
/// version the synthesizer supports, and the emitted runner refuses some of them at admission; the
/// README for those still gives run instructions (and, for `>= 5`, "select
/// `ESS_REPORT_FORMAT=2` explicitly before execution"), describing a run that cannot happen.
#[test]
fn adversary_go_readme_is_not_emitted_for_a_suite_its_runner_refuses() {
    for major in [22, 26, 29] {
        // Not emitted is the outcome this case asks for: the emitter refusing the version
        // (beyond10x/ess#186, correction 1 F3) writes no README to disagree with the runner.
        if ess_conformance::go::emit(&suite(major)).is_err() {
            continue;
        }
        let observed = observe_go(major);
        assert!(
            observed.version_supported,
            "Go suite/{major}: the package is emitted with run instructions, and its own runner \
             refuses the suite at admission:\n{}\n--- README run instructions\n{}",
            observed.logs,
            instructions(&go_readme(major)),
        );
    }
}

#[test]
fn adversary_typescript_readme_is_not_emitted_for_a_suite_its_runner_refuses() {
    for major in [12, 17, 26] {
        let observed = observe_typescript(major);
        assert!(
            observed.version_supported,
            "TypeScript suite/{major}: the package is emitted with run instructions, and its own \
             runner refuses the suite at admission:\n{}\n--- README run instructions\n{}",
            observed.logs,
            instructions(&typescript_readme(major)),
        );
    }
}

/// The TypeScript README already had a `## Running it` section (`npm install` / `npm test`). The
/// unit inserts a second one further down, so a suite the runner refuses without the variable
/// is introduced by a first "Running it" that says to run bare `npm test` — which stops before the
/// first scenario — and a second "Running it" that says the opposite.
#[test]
fn adversary_typescript_readme_never_tells_a_bare_npm_test_the_runner_refuses() {
    for major in [5, 10, 21] {
        let readme = typescript_readme(major);
        let instructions = instructions(&readme);
        assert_eq!(
            instructions.matches("## Running it").count(),
            1,
            "TypeScript suite/{major}: the README has more than one `## Running it`:\n{readme}"
        );
        for block in console_blocks(instructions) {
            for line in block.lines().filter(|line| line.contains("npm test")) {
                assert!(
                    line.contains("ESS_REPORT_FORMAT=2"),
                    "TypeScript suite/{major}: the run instructions say `{line}`, which the \
                     runner refuses for this version:\n{readme}"
                );
            }
        }
    }
}

/// The public guide's run step for a generated package gives `ESS_REPORT_OUT=… go test ./...` and
/// `… npm test` with no `ESS_REPORT_FORMAT=2` and no word that most suite versions stop without
/// it, while the generated README now says they do.
#[test]
fn adversary_verify_conformance_guide_run_step_agrees_with_the_generated_readme() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let guide =
        std::fs::read_to_string(root.join("website/docs/guides/verify/runners.md")).unwrap();
    let start = guide
        .find("Then run the language's own test command.")
        .expect("the guide's run step");
    let end = start + guide[start..].find("## ").expect("the next section");
    let step = &guide[start..end];
    assert!(
        go_readme(10).contains("ESS_REPORT_FORMAT=2 go test ./..."),
        "precondition: the README requires the variable for suite/10"
    );
    assert!(
        step.contains("ESS_REPORT_FORMAT=2"),
        "the guide's run step tells an adopter to run a generated suite without \
         `ESS_REPORT_FORMAT=2`, which the generated README (and the runner) says stops a /5 to /21 \
         suite before the first scenario:\n{step}"
    );
}
