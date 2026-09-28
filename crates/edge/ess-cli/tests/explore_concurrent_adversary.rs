//! Adversarial cases against the concurrent mode of the emitted Go and TypeScript explorers
//! (`story:concurrent-explorer-runner`).
//!
//! Each package is emitted by this tree's `ess`, a driver written by this file is placed in it, and
//! each case writes `<name>.json` (a result), `<name>.refused` (an error) and `<name>.problem` (what
//! `CheckConcurrent` / `concurrentProblem` or `CheckExplored` / `assertExplored` said, or `none`).
//! A missing `go`, `node` or `tsc` panics rather than skipping.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

const EXPLORE: &str = "crates/verify/ess-conformance/tests/fixtures/explore.yaml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn fixture(name: &str) -> PathBuf {
    root()
        .join("crates/verify/ess-conformance/tests/fixtures")
        .join(name)
}

fn printed(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn path_with_ess() -> String {
    let ess = Path::new(env!("CARGO_BIN_EXE_ess"));
    let directory = ess.parent().expect("the binary has a directory");
    match std::env::var("PATH") {
        Ok(path) => format!("{}:{path}", directory.display()),
        Err(_) => directory.display().to_string(),
    }
}

fn emit(spec: &str, language: &str, out: &Path) -> PathBuf {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "synthesize", "--path", spec])
        .args(["--target", language, "--out"])
        .arg(out)
        .output()
        .expect("the `ess` binary runs");
    assert!(output.status.success(), "{}", printed(&output));
    out.join("essconform")
}

const GO_DRIVER: &str = r#"package essconform

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

type advCase struct {
	Name    string            `json:"name"`
	Target  string            `json:"target"`
	Mode    string            `json:"mode"`
	PathEnv *string           `json:"pathEnv"`
	Options ConcurrentOptions `json:"options"`
}

type advFailing struct{ Target }

func (advFailing) ExecuteCommand(CommandRequest) (CommandResult, error) {
	return CommandResult{}, errors.New("adversary: the call timed out")
}

type advFlaky struct {
	Target
	calls *int
}

func (f advFlaky) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	*f.calls++
	if *f.calls%3 == 0 {
		return CommandResult{}, errors.New("adversary: the call timed out")
	}
	return f.Target.ExecuteCommand(request)
}

func advTarget(name string) func() Target {
	switch name {
	case "failing":
		return func() Target { return advFailing{newExploreFixtureTarget("", "low")} }
	case "flaky":
		return func() Target { n := 0; return advFlaky{newExploreFixtureTarget("", "low"), &n} }
	case "close-unsupported", "lost-update":
		return func() Target { return newExploreFixtureTarget(name, "low") }
	default:
		return func() Target { return newExploreFixtureTarget("", "low") }
	}
}

func TestAdversary(t *testing.T) {
	out := os.Getenv("ESS_ADV_OUT")
	var cases []advCase
	if err := json.Unmarshal([]byte(os.Getenv("ESS_ADV_CASES")), &cases); err != nil {
		t.Fatal(err)
	}
	for _, one := range cases {
		one := one
		t.Run(one.Name, func(t *testing.T) {
			write := func(suffix, text string) {
				if err := os.WriteFile(filepath.Join(out, one.Name+suffix), []byte(text+"\n"), 0o644); err != nil {
					t.Fatal(err)
				}
			}
			if one.PathEnv != nil {
				t.Setenv("PATH", *one.PathEnv)
			}
			if one.Mode == "sequential" {
				result, err := Explore(advTarget(one.Target), ExploreOptions{})
				if err != nil {
					write(".refused", err.Error())
					return
				}
				write(".json", "{}")
				if problem := CheckExplored(result, AssertOptions{}); problem != nil {
					write(".problem", problem.Error())
				} else {
					write(".problem", "none")
				}
				return
			}
			options := one.Options
			options.Path = os.Getenv("ESS_ADV_SPEC")
			options.Out = filepath.Join(out, one.Name)
			var result ConcurrentResult
			var err error
			func() {
				defer func() {
					if r := recover(); r != nil {
						err = fmt.Errorf("panicked: %v", r)
					}
				}()
				result, err = ExploreConcurrent(advTarget(one.Target), options)
			}()
			if err != nil {
				write(".refused", err.Error())
				return
			}
			bytes, err := json.Marshal(result)
			if err != nil {
				t.Fatal(err)
			}
			write(".json", string(bytes))
			if problem := CheckConcurrent(result); problem != nil {
				write(".problem", problem.Error())
			} else {
				write(".problem", "none")
			}
		})
	}
}
"#;

const TS_DRIVER: &str = r"import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { assertExplored, concurrentProblem, explore, exploreConcurrent, goMarshal } from './dist/index.js';
import { newTarget } from './explore-target.mjs';

function advTarget(name) {
  switch (name) {
    case 'failing':
      return () => {
        const target = newTarget('', 'low');
        target.invokeCommand = undefined;
        target.executeCommand = () => {
          throw new Error('adversary: the call timed out');
        };
        return target;
      };
    case 'flaky':
      return () => {
        const target = newTarget('', 'low');
        const original = target.executeCommand.bind(target);
        let calls = 0;
        target.invokeCommand = undefined;
        target.executeCommand = (request) => {
          calls += 1;
          if (calls % 3 === 0) throw new Error('adversary: the call timed out');
          return original(request);
        };
        return target;
      };
    case 'close-unsupported':
    case 'lost-update':
      return () => newTarget(name, 'low');
    default:
      return () => newTarget('', 'low');
  }
}

const out = process.env.ESS_ADV_OUT ?? '.';
const cases = JSON.parse(process.env.ESS_ADV_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    const path = process.env.PATH;
    if (one.pathEnv !== undefined) process.env.PATH = one.pathEnv;
    try {
      if (one.mode === 'sequential') {
        const result = await explore(advTarget(one.target), {});
        write('.json', '{}');
        try {
          assertExplored(result, {});
          write('.problem', 'none');
        } catch (error) {
          write('.problem', error.message);
        }
        return;
      }
      const result = await exploreConcurrent(advTarget(one.target), {
        ...(one.options ?? {}),
        path: process.env.ESS_ADV_SPEC,
        out: join(out, one.name),
      });
      write('.json', goMarshal(result));
      write('.problem', concurrentProblem(result) ?? 'none');
    } catch (error) {
      write('.refused', error.message);
    } finally {
      process.env.PATH = path;
    }
  });
}
";

#[derive(Debug)]
struct Case {
    result: Result<Value, String>,
    problem: Option<String>,
}

#[derive(Debug)]
struct Lane {
    cases: BTreeMap<String, Case>,
}

fn read_lane(out: &Path, cases: &Value, log: &str) -> Lane {
    let mut read = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().unwrap().to_owned();
        let result = match std::fs::read_to_string(out.join(format!("{name}.json"))) {
            Ok(text) => Ok(serde_json::from_str(&text).expect("the result is JSON")),
            Err(_) => Err(std::fs::read_to_string(out.join(format!("{name}.refused")))
                .unwrap_or_else(|_| panic!("`{name}` wrote neither a result nor a refusal:\n{log}"))
                .trim_end()
                .to_owned()),
        };
        let problem = std::fs::read_to_string(out.join(format!("{name}.problem")))
            .ok()
            .map(|it| it.trim_end().to_owned());
        read.insert(name, Case { result, problem });
    }
    Lane { cases: read }
}

fn go(root: &Path, cases: &Value) -> Lane {
    let module = root.join("go");
    let package = emit(EXPLORE, "go", &module);
    std::fs::write(
        module.join("go.mod"),
        "module example.invalid/adversary\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::copy(
        fixture("explore_target.go"),
        package.join("explore_target_test.go"),
    )
    .unwrap();
    std::fs::write(package.join("adversary_driver_test.go"), GO_DRIVER).unwrap();
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args(["test", "./essconform", "-run", "TestAdversary"])
        .args(["-count=1", "-v", "-timeout", "30m"])
        .env("ESS_ADV_CASES", cases.to_string())
        .env("ESS_ADV_OUT", &out)
        .env("ESS_ADV_SPEC", root_of(EXPLORE))
        .env("PATH", path_with_ess())
        .env("GOWORK", "off")
        .env_remove("ESS_EXPLORE_MUTANT")
        .env_remove("ESS_EXPLORE_GRADE")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, &log)
}

fn typescript(root: &Path, cases: &Value) -> Lane {
    let package = emit(EXPLORE, "typescript", &root.join("typescript"));
    std::fs::copy(
        fixture("explore-target.mjs"),
        package.join("explore-target.mjs"),
    )
    .unwrap();
    std::fs::write(package.join("adversary-driver.mjs"), TS_DRIVER).unwrap();
    std::fs::write(
        package.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&package)
        .output()
        .expect("`tsc` is on PATH");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let out = root.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "adversary-driver.mjs"])
        .env("ESS_ADV_CASES", cases.to_string())
        .env("ESS_ADV_OUT", &out)
        .env("ESS_ADV_SPEC", root_of(EXPLORE))
        .env("PATH", path_with_ess())
        .env_remove("ESS_EXPLORE_MUTANT")
        .env_remove("ESS_EXPLORE_GRADE")
        .current_dir(&package)
        .output()
        .expect("`node` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, &log)
}

fn root_of(spec: &str) -> PathBuf {
    root().join(spec)
}

fn cases() -> Value {
    serde_json::json!([
        {"name": "failing", "target": "failing", "options": {"seeds": 200}},
        {"name": "close-unsupported", "target": "close-unsupported", "options": {"seeds": 200}},
        {"name": "clients-one", "target": "explore", "options": {"seeds": 3, "clients": 1}},
        {"name": "clients-negative", "target": "explore", "options": {"seeds": 1, "clients": -1}},
    ])
}

/// (TypeScript, Go)
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("a scratch directory");
        let cases = cases();
        (
            typescript(scratch.path(), &cases),
            go(scratch.path(), &cases),
        )
    })
}

fn both() -> [(&'static str, &'static Lane); 2] {
    let (typescript, go) = lanes();
    [("typescript", typescript), ("go", go)]
}

/// Whether the run said something was wrong: an error, or a problem from the check.
fn flagged(case: &Case) -> bool {
    case.result.is_err() || case.problem.as_deref().is_some_and(|it| it != "none")
}

/// What each language did with `name`, in one line each, when it did not say anything was wrong.
fn passed(name: &str) -> Vec<String> {
    both()
        .into_iter()
        .filter(|(_, lane)| !flagged(&lane.cases[name]))
        .map(|(language, lane)| {
            let result = lane.cases[name].result.as_ref().expect("not flagged");
            format!(
                "{language}: {} histories, {} linearizable, {} violations, {} unknown, check: {}",
                result["histories"],
                result["linearizable"],
                result["violations"],
                result["unknown"],
                lane.cases[name].problem.as_deref().unwrap_or("-")
            )
        })
        .collect()
}

// ---- a target that answers nothing must not pass ------------------------------------------------

#[test]
fn a_target_whose_every_call_fails_does_not_pass_concurrent_exploration() {
    let passed = passed("failing");
    assert!(
        passed.is_empty(),
        "a target that answered none of its calls passed concurrent exploration:\n{}",
        passed.join("\n")
    );
}

#[test]
fn a_target_that_does_not_support_close_ticket_does_not_pass_concurrent_exploration() {
    let passed = passed("close-unsupported");
    assert!(
        passed.is_empty(),
        "`close-unsupported` (CloseTicket answers unsupported) passed concurrent exploration:\n{}",
        passed.join("\n")
    );
}

// ---- the documented range of clients, 2 to 4 ----------------------------------------------------

#[test]
fn one_client_is_refused_as_outside_two_to_four() {
    let accepted: Vec<String> = both()
        .into_iter()
        .filter(|(_, lane)| lane.cases["clients-one"].result.is_ok())
        .map(|(language, lane)| format!("{language}: {:?}", lane.cases["clients-one"].result))
        .collect();
    assert!(
        accepted.is_empty(),
        "`clients: 1` was accepted:\n{}",
        accepted.join("\n")
    );
}

#[test]
fn a_negative_client_count_is_an_error_not_a_panic() {
    for (language, lane) in both() {
        let refusal = lane.cases["clients-negative"]
            .result
            .as_ref()
            .expect_err("a negative client count is refused");
        assert!(
            !refusal.starts_with("panicked"),
            "{language}: a negative client count panicked: {refusal}"
        );
    }
}
