//! Second adversarial pass against the concurrent mode of the emitted Go and TypeScript explorers
//! (`story:concurrent-explorer-runner`), after the correction that made only a genuinely unknown
//! result `Indeterminate` and left unsupported commands out.
//!
//! Each package is emitted by this tree's `ess`, a driver written by this file is placed in it, and
//! each case writes `<name>.json` (a result), `<name>.refused` (an error) and `<name>.problem` (what
//! `CheckConcurrent` / `concurrentProblem` said, or `none`), and its histories under `<name>/`.
//! A missing `go`, `node` or `tsc` panics rather than skipping.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

const EXPLORE: &str = "crates/verify/ess-conformance/tests/fixtures/explore.yaml";
const CLOSE: &str = "explore.desk.CloseTicket";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn spec_path() -> PathBuf {
    root().join(EXPLORE)
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
	"strconv"
	"strings"
	"testing"
)

type adv2Case struct {
	Name    string            `json:"name"`
	Target  string            `json:"target"`
	Options ConcurrentOptions `json:"options"`
}

// adv2Target is the explore fixture target with CloseTicket unsupported from seed `closeFrom` on
// (0: never), and, under `mode`, every third completed call answered with an error.
type adv2Target struct {
	*exploreFixtureTarget
	closeFrom int
	mode      string
	seed      int
	calls     int
}

func (t *adv2Target) BeginScenario(scenario ScenarioContext) error {
	if at := strings.LastIndex(scenario.Scenario, "seed-"); at >= 0 {
		t.seed, _ = strconv.Atoi(scenario.Scenario[at+len("seed-"):])
	}
	return t.exploreFixtureTarget.BeginScenario(scenario)
}

func (t *adv2Target) blocked(request CommandRequest) bool {
	return t.closeFrom > 0 && t.seed >= t.closeFrom && request.Command == "explore.desk.CloseTicket"
}

func (t *adv2Target) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if t.blocked(request) {
		return CommandResult{}, exploreFixtureUnsupported{"CloseTicket is not exposed"}
	}
	return t.exploreFixtureTarget.ExecuteCommand(request)
}

type adv2Pending struct {
	target *adv2Target
	inner  PendingCommand
	err    error
}

func (p adv2Pending) Complete() (CommandResult, error) {
	if p.err != nil {
		return CommandResult{}, p.err
	}
	result, err := p.inner.Complete()
	if err != nil {
		return result, err
	}
	p.target.calls++
	if p.target.calls%3 == 0 {
		switch p.target.mode {
		case "join":
			return CommandResult{}, errors.Join(errors.New("adversary: the first attempt timed out"), ErrIndeterminate)
		case "plain":
			return CommandResult{}, errors.New("adversary: timed out")
		}
	}
	return result, nil
}

func (t *adv2Target) InvokeCommand(request CommandRequest) PendingCommand {
	if t.blocked(request) {
		return adv2Pending{err: exploreFixtureUnsupported{"CloseTicket is not exposed"}}
	}
	return adv2Pending{target: t, inner: t.exploreFixtureTarget.InvokeCommand(request)}
}

func adv2New(name string) func() Target {
	switch name {
	case "late-close":
		return func() Target { return &adv2Target{exploreFixtureTarget: newExploreFixtureTarget("", "low"), closeFrom: 5} }
	case "lost-close":
		return func() Target {
			return &adv2Target{exploreFixtureTarget: newExploreFixtureTarget("lost-update", "low"), closeFrom: 1}
		}
	case "join-indeterminate":
		return func() Target { return &adv2Target{exploreFixtureTarget: newExploreFixtureTarget("", "low"), mode: "join"} }
	case "plain-error":
		return func() Target { return &adv2Target{exploreFixtureTarget: newExploreFixtureTarget("", "low"), mode: "plain"} }
	default:
		return func() Target { return newExploreFixtureTarget("", "low") }
	}
}

func TestAdversaryPass2(t *testing.T) {
	out := os.Getenv("ESS_ADV2_OUT")
	var cases []adv2Case
	if err := json.Unmarshal([]byte(os.Getenv("ESS_ADV2_CASES")), &cases); err != nil {
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
			options := one.Options
			options.Path = os.Getenv("ESS_ADV2_SPEC")
			options.Out = filepath.Join(out, one.Name)
			var result ConcurrentResult
			var err error
			func() {
				defer func() {
					if r := recover(); r != nil {
						err = fmt.Errorf("panicked: %v", r)
					}
				}()
				result, err = ExploreConcurrent(adv2New(one.Target), options)
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

import {
  concurrentProblem,
  exploreConcurrent,
  goMarshal,
  indeterminate,
  unsupported,
} from './dist/index.js';
import { newTarget } from './explore-target.mjs';

const CLOSE = 'explore.desk.CloseTicket';

// The explore fixture target with CloseTicket unsupported from seed `closeFrom` on (0: never), and,
// under `mode`, every third call answered with a throw, at its invoke or at its completion.
function wrapped(mutant, { closeFrom = 0, mode = '' } = {}) {
  return () => {
    const target = newTarget(mutant, 'low');
    let seed = 0;
    let calls = 0;
    let invokes = 0;
    const begin = target.beginScenario.bind(target);
    const execute = target.executeCommand.bind(target);
    const invoke = target.invokeCommand.bind(target);
    const blocked = (request) => closeFrom > 0 && seed >= closeFrom && request.command === CLOSE;
    target.beginScenario = (scenario) => {
      const found = /seed-(-?\d+)$/.exec(scenario.scenario);
      seed = found === null ? 0 : Number(found[1]);
      return begin(scenario);
    };
    target.executeCommand = (request) => {
      if (blocked(request)) throw unsupported('CloseTicket is not exposed');
      return execute(request);
    };
    target.invokeCommand = (request) => {
      if (mode === 'invoke-unsupported' && request.command === CLOSE)
        throw unsupported('CloseTicket is not exposed');
      if (mode === 'invoke-indeterminate') {
        invokes += 1;
        if (invokes % 3 === 0) throw indeterminate('adversary: the request timed out at its invoke');
      }
      if (blocked(request))
        return {
          complete: () => {
            throw unsupported('CloseTicket is not exposed');
          },
        };
      const pending = invoke(request);
      return {
        complete: async () => {
          const result = await pending.complete();
          calls += 1;
          if (calls % 3 === 0) {
            if (mode === 'join')
              throw new AggregateError(
                [
                  new Error('adversary: the first attempt timed out'),
                  indeterminate('the call did not answer, and may have taken effect'),
                ],
                'adversary: every attempt failed',
              );
            if (mode === 'plain') throw 'adversary: timed out';
          }
          return result;
        },
      };
    };
    return target;
  };
}

function adv2New(name) {
  switch (name) {
    case 'late-close':
      return wrapped('', { closeFrom: 5 });
    case 'lost-close':
      return wrapped('lost-update', { closeFrom: 1 });
    case 'join-indeterminate':
      return wrapped('', { mode: 'join' });
    case 'plain-error':
      return wrapped('', { mode: 'plain' });
    case 'invoke-indeterminate':
      return wrapped('', { mode: 'invoke-indeterminate' });
    case 'invoke-unsupported':
      return wrapped('', { mode: 'invoke-unsupported' });
    default:
      return () => newTarget('', 'low');
  }
}

const out = process.env.ESS_ADV2_OUT ?? '.';
const cases = JSON.parse(process.env.ESS_ADV2_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    try {
      const result = await exploreConcurrent(adv2New(one.target), {
        ...(one.options ?? {}),
        path: process.env.ESS_ADV2_SPEC,
        out: join(out, one.name),
      });
      write('.json', goMarshal(result));
      write('.problem', concurrentProblem(result) ?? 'none');
    } catch (error) {
      write('.refused', error instanceof Error ? error.message : String(error));
    }
  });
}
";

#[derive(Debug)]
struct Case {
    result: Result<Value, String>,
    problem: Option<String>,
    histories: BTreeMap<u64, Vec<u8>>,
}

#[derive(Debug)]
struct Lane {
    cases: BTreeMap<String, Case>,
}

fn read_lane(out: &Path, cases: &[Value], log: &str) -> Lane {
    let mut read = BTreeMap::new();
    for case in cases {
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
        let mut histories = BTreeMap::new();
        if let Ok(entries) = std::fs::read_dir(out.join(&name)) {
            for entry in entries {
                let path = entry.unwrap().path();
                let file = path.file_name().unwrap().to_string_lossy().into_owned();
                if let Some(seed) = file
                    .strip_prefix("history-")
                    .and_then(|rest| rest.strip_suffix(".json"))
                    .and_then(|seed| seed.parse::<u64>().ok())
                {
                    histories.insert(seed, std::fs::read(&path).unwrap());
                }
            }
        }
        read.insert(
            name,
            Case {
                result,
                problem,
                histories,
            },
        );
    }
    Lane { cases: read }
}

fn go(root: &Path, cases: &[Value]) -> Lane {
    let module = root.join("go");
    let package = emit(EXPLORE, "go", &module);
    std::fs::write(
        module.join("go.mod"),
        "module example.invalid/adversary2\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::copy(
        fixture("explore_target.go"),
        package.join("explore_target_test.go"),
    )
    .unwrap();
    std::fs::write(package.join("adversary2_driver_test.go"), GO_DRIVER).unwrap();
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args(["test", "./essconform", "-run", "TestAdversaryPass2"])
        .args(["-count=1", "-v", "-timeout", "30m"])
        .env("ESS_ADV2_CASES", Value::Array(cases.to_vec()).to_string())
        .env("ESS_ADV2_OUT", &out)
        .env("ESS_ADV2_SPEC", spec_path())
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

fn typescript(root: &Path, cases: &[Value]) -> Lane {
    let package = emit(EXPLORE, "typescript", &root.join("typescript"));
    std::fs::copy(
        fixture("explore-target.mjs"),
        package.join("explore-target.mjs"),
    )
    .unwrap();
    std::fs::write(package.join("adversary2-driver.mjs"), TS_DRIVER).unwrap();
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
        .args(["--test", "adversary2-driver.mjs"])
        .env("ESS_ADV2_CASES", Value::Array(cases.to_vec()).to_string())
        .env("ESS_ADV2_OUT", &out)
        .env("ESS_ADV2_SPEC", spec_path())
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

/// Seeds replayed one at a time against the batch run of `late-close`.
const REPLAYED: [u64; 3] = [6, 9, 12];

/// Every case, and whether only the TypeScript lane runs it (Go has no throw at an invoke).
fn cases() -> Vec<(Value, bool)> {
    let mut cases = vec![
        (
            serde_json::json!({"name": "seeds-negative", "target": "explore", "options": {"seeds": -1}}),
            false,
        ),
        (
            serde_json::json!({"name": "late-close", "target": "late-close", "options": {"seeds": 12, "allowExcluded": true}}),
            false,
        ),
        (
            serde_json::json!({"name": "lost-close", "target": "lost-close", "options": {"seeds": 200}}),
            false,
        ),
        (
            serde_json::json!({"name": "join-indeterminate", "target": "join-indeterminate", "options": {"seeds": 20}}),
            false,
        ),
        (
            serde_json::json!({"name": "plain-error", "target": "plain-error", "options": {"seeds": 3}}),
            false,
        ),
        (
            serde_json::json!({"name": "invoke-indeterminate", "target": "invoke-indeterminate", "options": {"seeds": 20}}),
            true,
        ),
        (
            serde_json::json!({"name": "invoke-unsupported", "target": "invoke-unsupported", "options": {"seeds": 20}}),
            true,
        ),
    ];
    for seed in REPLAYED {
        cases.push((
            serde_json::json!({"name": format!("late-close-seed-{seed}"), "target": "late-close", "options": {"seed": seed, "allowExcluded": true}}),
            false,
        ));
    }
    cases
}

/// (TypeScript, Go)
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("a scratch directory");
        let all: Vec<Value> = cases().into_iter().map(|(case, _)| case).collect();
        let go_only: Vec<Value> = cases()
            .into_iter()
            .filter(|(_, ts_only)| !ts_only)
            .map(|(case, _)| case)
            .collect();
        (
            typescript(scratch.path(), &all),
            go(scratch.path(), &go_only),
        )
    })
}

fn both() -> [(&'static str, &'static Lane); 2] {
    let (typescript, go) = lanes();
    [("typescript", typescript), ("go", go)]
}

fn ok<'a>(language: &str, lane: &'a Lane, name: &str) -> &'a Value {
    match &lane.cases[name].result {
        Ok(value) => value,
        Err(refusal) => panic!("{language}: `{name}` was refused: {refusal}"),
    }
}

fn operations(case: &Case) -> Vec<Value> {
    case.histories
        .values()
        .flat_map(|bytes| {
            let document: Value = serde_json::from_slice(bytes).unwrap();
            document["operations"].as_array().unwrap().clone()
        })
        .collect()
}

fn indeterminate_written(case: &Case) -> u64 {
    operations(case)
        .iter()
        .filter(|operation| operation["completion"] == "Indeterminate")
        .count() as u64
}

// ---- boundaries: a negative seed count is not a vacuous pass ----------------------------------

#[test]
fn a_negative_seed_count_is_refused_not_a_pass_over_zero_histories() {
    for (language, lane) in both() {
        let case = &lane.cases["seeds-negative"];
        let passed = case.result.is_ok() && case.problem.as_deref() == Some("none");
        assert!(
            !passed,
            "{language}: `seeds: -1` passed concurrent exploration: {:?}",
            case.result
        );
    }
}

// ---- an exclusion found mid-run: Go and TypeScript write the same bytes -----------------------

#[test]
fn an_exclusion_found_at_seed_five_leaves_go_and_typescript_histories_equal() {
    let (typescript, go) = lanes();
    let ts = &typescript.cases["late-close"];
    assert_eq!(ts.histories.len(), 12);
    assert_eq!(ts.histories, go.cases["late-close"].histories);
    assert_eq!(
        ok("typescript", typescript, "late-close"),
        ok("go", go, "late-close")
    );
    let result = ok("typescript", typescript, "late-close");
    assert_eq!(result["excluded"][0]["subject"], CLOSE, "{result}");
    // Not vacuous: CloseTicket was called before seed 5, and never written after it.
    let closes = |seed: u64| {
        let document: Value = serde_json::from_slice(&ts.histories[&seed]).unwrap();
        document["operations"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|operation| operation["command"] == CLOSE)
            .count()
    };
    assert!((1..5).map(closes).sum::<usize>() > 0);
    assert_eq!((5..=12).map(closes).sum::<usize>(), 0);
}

// ---- replaying one seed reproduces its history ------------------------------------------------

#[test]
fn replaying_a_seed_with_seed_writes_the_history_the_batch_wrote_for_it() {
    for (language, lane) in both() {
        let batch = &lane.cases["late-close"].histories;
        let mut differ = Vec::new();
        for seed in REPLAYED {
            let alone = &lane.cases[&format!("late-close-seed-{seed}")].histories;
            ok(language, lane, &format!("late-close-seed-{seed}"));
            if alone.get(&seed) != batch.get(&seed) {
                differ.push(seed);
            }
        }
        assert!(
            differ.is_empty(),
            "{language}: `Seed: n` did not reproduce the history the batch run wrote for seeds {differ:?}"
        );
    }
}

// ---- a violation is still reported when some calls were left out ------------------------------

#[test]
fn a_lost_update_is_still_a_violation_when_close_ticket_is_left_out() {
    for (language, lane) in both() {
        let result = ok(language, lane, "lost-close");
        assert!(
            result["violations"].as_u64().unwrap() >= 1,
            "{language}: {result}"
        );
        assert_eq!(result["excluded"][0]["subject"], CLOSE, "{result}");
        let report = result["failure"]["report"].as_str().unwrap();
        assert!(report.contains("shrunk history:"), "{report}");
        let problem = lane.cases["lost-close"].problem.as_deref().unwrap();
        assert!(
            problem.contains("concurrent histories were violations")
                && problem.contains("shrunk history:")
                && problem.contains("left out of concurrent exploration"),
            "{language}: {problem}"
        );
        assert!(operations(&lane.cases["lost-close"])
            .iter()
            .all(|operation| operation["command"] != CLOSE));
    }
    let (typescript, go) = lanes();
    assert_eq!(
        typescript.cases["lost-close"].histories,
        go.cases["lost-close"].histories
    );
}

// ---- an indeterminate cause joined with another error ------------------------------------------

#[test]
fn an_indeterminate_error_joined_with_another_is_indeterminate_in_both_languages() {
    for (language, lane) in both() {
        let case = &lane.cases["join-indeterminate"];
        let result = ok(language, lane, "join-indeterminate");
        assert!(indeterminate_written(case) > 0, "{language}: {result}");
        assert_eq!(
            result["indeterminate"].as_u64().unwrap(),
            indeterminate_written(case),
            "{language}: {result}"
        );
    }
    let (typescript, go) = lanes();
    assert_eq!(
        typescript.cases["join-indeterminate"].histories,
        go.cases["join-indeterminate"].histories
    );
}

// ---- a plain error, and a thrown value that is not an Error, stop the run ----------------------

#[test]
fn a_plain_error_or_a_thrown_string_stops_the_run_with_the_same_message() {
    let (typescript, go) = lanes();
    let ts = typescript.cases["plain-error"]
        .result
        .as_ref()
        .expect_err("a thrown string stops the run");
    let go = go.cases["plain-error"]
        .result
        .as_ref()
        .expect_err("a plain error stops the run");
    assert!(ts.contains("failed: adversary: timed out"), "{ts}");
    assert_eq!(ts, go);
}

// ---- TypeScript: a throw at the invoke of an interleaved call ----------------------------------

#[test]
fn an_indeterminate_throw_at_invoke_is_written_indeterminate_not_a_failed_run() {
    let (typescript, _) = lanes();
    let case = &typescript.cases["invoke-indeterminate"];
    let result = ok("typescript", typescript, "invoke-indeterminate");
    assert!(indeterminate_written(case) > 0, "{result}");
    assert_eq!(
        result["indeterminate"].as_u64().unwrap(),
        indeterminate_written(case)
    );
}

#[test]
fn an_unsupported_throw_at_invoke_excludes_the_command_not_a_failed_run() {
    let (typescript, _) = lanes();
    let result = ok("typescript", typescript, "invoke-unsupported");
    assert_eq!(result["excluded"][0]["subject"], CLOSE, "{result}");
}

// ---- the emitted API documents the corrected rule ----------------------------------------------

#[test]
fn the_pending_command_docs_do_not_say_every_error_is_indeterminate() {
    let scratch = tempfile::tempdir().unwrap();
    let go =
        std::fs::read_to_string(emit(EXPLORE, "go", &scratch.path().join("go")).join("explore.go"))
            .unwrap();
    let ts = std::fs::read_to_string(
        emit(EXPLORE, "typescript", &scratch.path().join("ts")).join("src/explore.ts"),
    )
    .unwrap();
    let mut stale = Vec::new();
    if go.contains("An error is a call that never answered") {
        stale.push("explore.go: PendingCommand.Complete says an error is a call that never answered, written `Indeterminate`");
    }
    if ts.contains("A throw is a call that never answered") {
        stale.push("explore.ts: PendingCommand.complete says a throw is a call that never answered, written `Indeterminate`");
    }
    assert!(stale.is_empty(), "{}", stale.join("\n"));
}
