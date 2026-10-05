//! The emitted Go and TypeScript packages describe what their runner does (beyond10x/ess#186).
//!
//! Three things an adopter follows literally, and all three were wrong:
//!
//! - `CommandResult.Outcome` said "empty when it refused", while the runner compares `Outcome` for
//!   refusals exactly as for any other branch, so a target that followed it failed every refusal
//!   scenario.
//! - The README put `ESS_REPORT_FORMAT=2` under "The report", as if it chose a report shape, while
//!   the runner stops before the first scenario without it.
//! - A package was emitted, run instructions and all, for suite versions its own runner refuses
//!   at admission.
//!
//! The README and the emitter answer from one Rust rule (`go::requires_report_format_2` and
//! `go::NEWEST_ADMITTED_SUITE_MAJOR`). These cases hold that rule against what both emitted
//! runners *do*: each runner is compiled once and run over a suite document of every supported
//! major, with and without `ESS_REPORT_FORMAT=2`, so a runtime that changes either gate turns this
//! red until the emitter and the README say the same.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_conformance::scenario::{
    ScenarioInitialState, SuiteFormat, SuiteProvenance, SUPPORTED_SUITE_FORMATS,
};
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
        scenario_initial_state: (major >= 34).then_some(ScenarioInitialState::Empty),
        // The seed-bearing pair requires its record (beyond10x/ess#413): one selection, no use.
        synthesis_seeds: matches!(major, 42 | 43).then(seed_record),
    })
}

/// The smallest valid `synthesis_seeds` record: one source and one unused selection.
fn seed_record() -> ess_conformance::synthesis_seeds::SynthesisSeeds {
    use ess_conformance::coverage::SourceIdentity;
    use ess_conformance::synthesis_seeds::{SeedRecord, SynthesisSeeds};
    let source = SourceIdentity::new("seed.yaml").expect("a source identity");
    SynthesisSeeds {
        sources: [(source.clone(), format!("sha256:{}", "0".repeat(64)))].into(),
        selections: vec![SeedRecord {
            source,
            instance: ess_conformance::InstanceName::new("row").expect("an instance"),
            entity: "billing.invoice.Invoice".parse().expect("an entity"),
            identity: ess_primitives::node::Node::Text(
                "00000000-0000-4000-8000-000000000001".into(),
            ),
            fields: std::collections::BTreeMap::new(),
            state: "Draft".parse().expect("a state"),
        }],
        applications: Vec::new(),
    }
}

/// One emitted package as `(path, contents)` pairs, or the emitter's refusal.
type Package = Result<Vec<(String, String)>, String>;

fn go(major: u32) -> Package {
    ess_conformance::go::emit(&suite(major))
        .map(|files| files.into_iter().map(|f| (f.path, f.contents)).collect())
        .map_err(|error| error.to_string())
}

fn typescript(major: u32) -> Package {
    ess_conformance::ts::emit(&suite(major))
        .map(|files| files.into_iter().map(|f| (f.path, f.contents)).collect())
        .map_err(|error| error.to_string())
}

fn file<'a>(package: &'a [(String, String)], suffix: &str) -> &'a str {
    let Some((_, contents)) = package.iter().find(|(path, _)| path.ends_with(suffix)) else {
        panic!("the package carries `{suffix}`")
    };
    contents
}

fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("generated-docs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn write_all(root: &Path, files: &[(String, String)]) {
    for (path, contents) in files {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

/// The empty coverage inventory an odd major from `/5` carries: nothing generated, nothing
/// authored, the whole system selected. Its shape is the released `ess-conformance/27` fixture's
/// (`tests/fixtures/released-0.38/suite27.json`) with every list emptied.
fn empty_coverage() -> serde_json::Value {
    serde_json::json!({
        "authored": [],
        "authored_sources": {},
        "counts": {"authored": 0, "generated": 0, "outside": 0, "refused": 0},
        "generated": [],
        "knowledge": "complete_inventory",
        "outside": [],
        "refused": [],
        "selection": {"filter": {"kind": "all"}, "origins": "generated", "scope": {"kind": "system"}}
    })
}

/// A suite document for every supported major, named so the probes read them in order. The odd
/// majors from `/5` carry an empty coverage inventory, which both runners require of them.
fn documents(root: &Path) -> PathBuf {
    let directory = root.join("documents");
    std::fs::create_dir_all(&directory).unwrap();
    for &major in SUPPORTED_SUITE_FORMATS {
        let json = suite(major)
            .to_canonical_json()
            .expect("an empty suite serialises");
        let json = if major >= 5 && major % 2 == 1 {
            let mut document: serde_json::Value = serde_json::from_str(&json).unwrap();
            document["coverage"] = empty_coverage();
            serde_json::to_string(&document).unwrap()
        } else {
            json
        };
        std::fs::write(directory.join(format!("m{major:02}.json")), json).unwrap();
    }
    directory
}

fn log(output: &Output) -> String {
    format!(
        "exit {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// What one runner did with one suite document.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Ran {
    Passed,
    RefusedVersion,
    RequiredFormat2,
    Other(String),
}

impl Ran {
    fn of(message: &str) -> Self {
        if message.contains("unsupported suite version") {
            Self::RefusedVersion
        } else if message.contains("require explicit ESS_REPORT_FORMAT=2 before execution") {
            Self::RequiredFormat2
        } else {
            Self::Other(message.to_owned())
        }
    }
}

/// `(major, with ESS_REPORT_FORMAT=2)` to what the runner did.
type Observed = BTreeMap<(u32, bool), Ran>;

/// A Go target for an empty suite: only `Identity` is ever asked.
const GO_PROBE: &str = r#"package essconform

import (
	"os"
	"path/filepath"
	"sort"
	"testing"
)

type probeTarget struct{}

func (probeTarget) Identity() (Identity, error)         { return Identity{Name: "probe", Version: "1"}, nil }
func (probeTarget) BeginScenario(ScenarioContext) error { return ErrUnsupported }
func (probeTarget) EndScenario(ScenarioContext) error   { return ErrUnsupported }
func (probeTarget) ExecuteCommand(CommandRequest) (CommandResult, error) {
	return CommandResult{}, ErrUnsupported
}
func (probeTarget) QueryView(ViewRequest) (ViewResult, error) { return ViewResult{}, ErrUnsupported }
func (probeTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func (probeTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (probeTarget) RedeliverEvent(RedeliveryRequest) error               { return ErrUnsupported }
func (probeTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestProbe(t *testing.T) {
	directory := os.Getenv("ESS_PROBE_DOCUMENTS")
	entries, err := os.ReadDir(directory)
	if err != nil {
		t.Fatal(err)
	}
	names := []string{}
	for _, entry := range entries {
		names = append(names, entry.Name())
	}
	sort.Strings(names)
	for _, name := range names {
		raw, err := os.ReadFile(filepath.Join(directory, name))
		if err != nil {
			t.Fatal(err)
		}
		for _, format := range []string{"unset", "2"} {
			suiteJSON = string(raw)
			os.Unsetenv("ESS_REPORT_OUT")
			if format == "unset" {
				os.Unsetenv("ESS_REPORT_FORMAT")
			} else {
				os.Setenv("ESS_REPORT_FORMAT", format)
			}
			t.Run(name[:3]+"-"+format, func(t *testing.T) {
				Run(t, func() Target { return probeTarget{} })
			})
		}
	}
}
"#;

fn observe_go() -> Observed {
    let root = scratch("go");
    let documents = documents(&root);
    write_all(&root, &go(4).expect("suite/4 emits"));
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/docs\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(root.join("essconform/probe_test.go"), GO_PROBE).unwrap();
    let output = Command::new("go")
        .args([
            "test",
            "-json",
            "-count=1",
            "-run",
            "^TestProbe$",
            "./essconform",
        ])
        .env("GOWORK", "off")
        .env("ESS_PROBE_DOCUMENTS", &documents)
        .current_dir(&root)
        .output()
        .expect("the Go toolchain runs");
    let mut outputs: BTreeMap<String, String> = BTreeMap::new();
    let mut verdicts: BTreeMap<String, String> = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(test) = event["Test"].as_str() else {
            continue;
        };
        let Some(name) = test.strip_prefix("TestProbe/") else {
            continue;
        };
        match event["Action"].as_str() {
            Some("output") => outputs
                .entry(name.to_owned())
                .or_default()
                .push_str(event["Output"].as_str().unwrap_or_default()),
            Some(action @ ("pass" | "fail" | "skip")) => {
                verdicts.insert(name.to_owned(), action.to_owned());
            }
            _ => {}
        }
    }
    let mut observed = Observed::new();
    for &major in SUPPORTED_SUITE_FORMATS {
        for format_2 in [false, true] {
            let name = format!("m{major:02}-{}", if format_2 { "2" } else { "unset" });
            let ran = match verdicts.get(&name).map(String::as_str) {
                Some("pass") => Ran::Passed,
                Some(_) => Ran::of(outputs.get(&name).map_or("", String::as_str)),
                None => panic!("the Go probe did not run `{name}`:\n{}", log(&output)),
            };
            observed.insert((major, format_2), ran);
        }
    }
    std::fs::remove_dir_all(&root).unwrap();
    observed
}

const TS_PROBE: &str = r"import test from 'node:test';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { runWith } from './dist/runtime.js';

const target = { identity() { return { name: 'probe', version: '1' }; } };
const directory = process.env.ESS_PROBE_DOCUMENTS;
await test('probe', async (t) => {
  for (const name of readdirSync(directory).sort()) {
    const raw = readFileSync(join(directory, name), 'utf8');
    for (const format of ['unset', '2']) {
      delete process.env.ESS_REPORT_OUT;
      if (format === 'unset') delete process.env.ESS_REPORT_FORMAT;
      else process.env.ESS_REPORT_FORMAT = format;
      let result = 'pass';
      try {
        await runWith(t, () => target, raw);
      } catch (error) {
        result = String(error instanceof Error ? error.message : error);
      }
      console.log('PROBE ' + JSON.stringify({ name: name.slice(0, 3) + '-' + format, result }));
    }
  }
});
";

fn observe_typescript() -> Observed {
    let root = scratch("ts");
    let documents = documents(&root);
    write_all(&root, &typescript(4).expect("suite/4 emits"));
    let package = root.join("essconform");
    std::fs::write(
        package.join("probe.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "probe.tsconfig.json"])
        .current_dir(&package)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(compiled.status.success(), "{}", log(&compiled));
    std::fs::write(package.join("probe.mjs"), TS_PROBE).unwrap();
    let output = Command::new("node")
        .arg("probe.mjs")
        .env("ESS_PROBE_DOCUMENTS", &documents)
        .current_dir(&package)
        .output()
        .expect("node runs");
    let mut results: BTreeMap<String, String> = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some(json) = line.trim_start().strip_prefix("PROBE ") else {
            continue;
        };
        let event: serde_json::Value = serde_json::from_str(json).expect("a probe line");
        results.insert(
            event["name"].as_str().unwrap().to_owned(),
            event["result"].as_str().unwrap().to_owned(),
        );
    }
    let mut observed = Observed::new();
    for &major in SUPPORTED_SUITE_FORMATS {
        for format_2 in [false, true] {
            let name = format!("m{major:02}-{}", if format_2 { "2" } else { "unset" });
            let ran = match results.get(&name).map(String::as_str) {
                Some("pass") => Ran::Passed,
                Some(message) => Ran::of(message),
                None => panic!(
                    "the TypeScript probe did not run `{name}`:\n{}",
                    log(&output)
                ),
            };
            observed.insert((major, format_2), ran);
        }
    }
    std::fs::remove_dir_all(&root).unwrap();
    observed
}

/// Everything before `## The report`: what an adopter reads to run the suite.
fn instructions(readme: &str) -> &str {
    readme
        .split_once("## The report")
        .unwrap_or_else(|| panic!("the README has a report section:\n{readme}"))
        .0
}

/// The emitter and the README against the runner, for every supported major.
fn check(target: &str, emit: fn(u32) -> Package, run: &str, observed: &Observed) {
    for &major in SUPPORTED_SUITE_FORMATS {
        let (without, with) = (&observed[&(major, false)], &observed[&(major, true)]);
        let admitted = *without != Ran::RefusedVersion && *with != Ran::RefusedVersion;
        let package = emit(major);
        match (&package, admitted) {
            (Err(error), true) => panic!(
                "{target} suite/{major}: the emitter refuses a version its runner admits: {error}"
            ),
            (Ok(_), false) => panic!(
                "{target} suite/{major}: the emitter writes a package, run instructions and all, \
                 for a version its runner refuses at admission"
            ),
            (Err(error), false) => {
                assert!(
                    error.contains(&format!("ess-conformance/{major}")),
                    "{target} suite/{major}: the refusal does not name the version: {error}"
                );
                continue;
            }
            (Ok(_), true) => {}
        }
        let package = package.unwrap();
        let readme = file(&package, "README.md");
        let instructions = instructions(readme);
        assert_eq!(
            *with,
            Ran::Passed,
            "{target} suite/{major}: an empty suite does not pass with ESS_REPORT_FORMAT=2"
        );
        let requires = match without {
            Ran::Passed => false,
            Ran::RequiredFormat2 => true,
            other => {
                panic!("{target} suite/{major}: without the variable the runner said {other:?}")
            }
        };
        let claims = instructions.contains(&format!("ESS_REPORT_FORMAT=2 {run}"));
        assert_eq!(
            claims,
            requires,
            "{target} suite/{major}: the run instructions {} `ESS_REPORT_FORMAT=2` and the runner \
             {} without it:\n{readme}",
            if claims { "require" } else { "do not require" },
            if requires { "stops" } else { "runs" },
        );
        if requires {
            assert!(
                instructions.contains("stops before the first"),
                "{target} suite/{major}: the run instructions do not say what happens without \
                 it:\n{readme}"
            );
            assert!(
                instructions.contains(&format!("`ess-conformance/{major}`")),
                "{target} suite/{major}: the run instructions do not name this suite's \
                 version:\n{readme}"
            );
            assert_eq!(
                instructions.matches("## Running it").count(),
                1,
                "{target} suite/{major}: one `## Running it`, not two:\n{readme}"
            );
            assert!(
                !readme.contains("Select `ESS_REPORT_FORMAT=2` explicitly"),
                "{target} suite/{major}: the requirement still reads as a report option:\n{readme}"
            );
        } else {
            assert!(
                !instructions.contains("ESS_REPORT_FORMAT"),
                "{target} suite/{major}: the run instructions require a variable the runtime does \
                 not:\n{readme}"
            );
        }
    }
}

#[test]
fn generated_docs_go_emitter_and_readme_agree_with_what_the_go_runner_does() {
    check("Go", go, "go test ./...", &observe_go());
}

#[test]
fn generated_docs_typescript_emitter_and_readme_agree_with_what_the_typescript_runner_does() {
    check("TypeScript", typescript, "npm test", &observe_typescript());
}

#[test]
fn generated_docs_go_command_result_outcome_says_a_refusal_returns_its_outcome() {
    let package = go(4).expect("suite/4 emits");
    let runtime = file(&package, "/runtime.go");
    assert!(
        !runtime.contains("empty when it refused"),
        "the Go `CommandResult.Outcome` comment still says a refusal leaves it empty"
    );
    assert!(
        runtime.contains("A refusal takes a branch too: return the refusing outcome's name"),
        "the Go `CommandResult.Outcome` comment does not say what a refusal returns"
    );
}

#[test]
fn generated_docs_typescript_command_result_outcome_says_a_refusal_returns_its_outcome() {
    let package = typescript(4).expect("suite/4 emits");
    let runtime = file(&package, "/runtime.ts");
    assert!(
        !runtime.contains("empty when it refused"),
        "the TypeScript `CommandResult.outcome` doc still says a refusal leaves it empty"
    );
    assert!(
        runtime.contains("A refusal takes a branch too: return the refusing outcome's name"),
        "the TypeScript `CommandResult.outcome` doc does not say what a refusal returns"
    );
}

/// The TypeScript runner's refusal names the rule it applies, not the two versions it was first
/// written for (beyond10x/ess#186).
#[test]
fn generated_docs_typescript_format_refusal_names_the_rule() {
    let package = typescript(4).expect("suite/4 emits");
    let runtime = file(&package, "/runtime.ts");
    assert!(
        !runtime.contains("suite/8 and /9 require"),
        "the TypeScript runner still says only suite/8 and /9 need ESS_REPORT_FORMAT=2"
    );
    assert!(
        runtime.contains(&format!(
            "suite/8 through /{} require explicit ESS_REPORT_FORMAT=2",
            SUPPORTED_SUITE_FORMATS.iter().max().unwrap()
        )),
        "the TypeScript runner's refusal does not name the versions it refuses"
    );
}
