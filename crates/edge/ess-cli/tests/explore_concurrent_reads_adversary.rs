//! Adversarial pass against the view reads of the concurrent Go and TypeScript explorers
//! (`story:explorers-record-view-reads`).
//!
//! Each package is emitted by this tree's `ess`, the fixture targets and a driver written by this
//! file are placed in it, and each case writes `<name>.json` (a result), `<name>.refused` (an
//! error) and `<name>.problem`, and its histories under `<name>/`. A missing `go`, `node` or `tsc`
//! panics rather than skipping.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

const EXPLORE: &str = "crates/verify/ess-conformance/tests/fixtures/explore.yaml";
const BILLING: &str = "examples/billing";
const RETRY: &str = "crates/verify/ess-conformance/tests/fixtures/explore-retry/retry.yaml";
const OPEN_TICKETS: &str = "explore.desk.OpenTickets";
const BY_ITEMS: &str = "explore.desk.TicketsByItems";

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
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

type advrCase struct {
	Name    string            `json:"name"`
	Target  string            `json:"target"`
	Options ConcurrentOptions `json:"options"`
}

// advrTarget is the explore fixture target under one of these modes:
//
//	stale-lost-rejected  the `stale-read` mutant, and the answer of every `OpenTicket` the target
//	                     rejected (which created nothing) is lost
//	lagging              `OpenTickets` answers a snapshot that is refreshed only when a read demands
//	                     a token newer than it: correct for `read_your_writes`; every answer
//	                     carries a token, a refusal the current one
//	lagging-deaf         the same, ignoring the token a read demands
//	unsupported-view     `TicketsByItems` is not exposed
//	read-indeterminate   every fourth view read times out
type advrTarget struct {
	*exploreFixtureTarget
	mode     string
	current  int
	snapshot []Row
	snapAt   int
	reads    int
}

func advrVersion(token string) int {
	var n int
	if _, err := fmt.Sscanf(token, "v%d", &n); err != nil {
		return 0
	}
	return n
}

func (t *advrTarget) BeginScenario(scenario ScenarioContext) error {
	t.current, t.snapshot, t.snapAt, t.reads = 0, []Row{}, 0, 0
	return t.exploreFixtureTarget.BeginScenario(scenario)
}

type advrPending struct {
	target  *advrTarget
	inner   PendingCommand
	command string
}

func (p advrPending) Complete() (CommandResult, error) {
	result, err := p.inner.Complete()
	if err != nil {
		return result, err
	}
	if n := advrVersion(result.Consistency); n > p.target.current {
		p.target.current = n
	}
	switch p.target.mode {
	case "lagging", "lagging-deaf":
		if result.Consistency == "" {
			result.Consistency = fmt.Sprintf("v%d", p.target.current)
		}
	case "stale-lost-rejected":
		if p.command == "explore.desk.OpenTicket" && result.Outcome == "rejected" {
			return CommandResult{}, fmt.Errorf("adversary: the answer was lost: %w", ErrIndeterminate)
		}
	}
	return result, nil
}

func (t *advrTarget) InvokeCommand(request CommandRequest) PendingCommand {
	return advrPending{target: t, inner: t.exploreFixtureTarget.InvokeCommand(request), command: request.Command}
}

func (t *advrTarget) QueryView(request ViewRequest) (ViewResult, error) {
	switch t.mode {
	case "lagging", "lagging-deaf":
		if request.View == "explore.desk.OpenTickets" {
			demanded := advrVersion(request.AtLeast)
			if t.mode == "lagging-deaf" {
				demanded = 0
			}
			if demanded > t.snapAt {
				current, err := t.exploreFixtureTarget.QueryView(ViewRequest{View: request.View, Params: request.Params, Correlation: request.Correlation, Deadline: request.Deadline})
				if err != nil {
					return current, err
				}
				t.snapshot, t.snapAt = current.Rows, t.current
			}
			return ViewResult{Rows: append([]Row{}, t.snapshot...)}, nil
		}
	case "unsupported-view":
		if request.View == "explore.desk.TicketsByItems" {
			return ViewResult{}, exploreFixtureUnsupported{"adversary: TicketsByItems is not exposed"}
		}
	case "read-indeterminate":
		t.reads++
		if t.reads%4 == 0 {
			return ViewResult{}, fmt.Errorf("adversary: the read timed out: %w", ErrIndeterminate)
		}
	}
	return t.exploreFixtureTarget.QueryView(request)
}

func advrNew(name string) func() Target {
	switch name {
	case "billing":
		return func() Target { return newExploreBillingTarget("") }
	case "retry":
		return func() Target { return newExploreRetryTarget("") }
	case "stale":
		return func() Target { return newExploreFixtureTarget("stale-read", "low") }
	case "stale-lost-rejected":
		return func() Target { return &advrTarget{exploreFixtureTarget: newExploreFixtureTarget("stale-read", "low"), mode: name} }
	case "correct":
		return func() Target { return newExploreFixtureTarget("", "low") }
	default:
		return func() Target { return &advrTarget{exploreFixtureTarget: newExploreFixtureTarget("", "low"), mode: name} }
	}
}

func TestAdversaryReads(t *testing.T) {
	out := os.Getenv("ESS_ADVR_OUT")
	var cases []advrCase
	if err := json.Unmarshal([]byte(os.Getenv("ESS_ADVR_CASES")), &cases); err != nil {
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
			options.Path = os.Getenv("ESS_ADVR_SPEC")
			options.Out = filepath.Join(out, one.Name)
			var result ConcurrentResult
			var err error
			func() {
				defer func() {
					if r := recover(); r != nil {
						err = fmt.Errorf("panicked: %v", r)
					}
				}()
				result, err = ExploreConcurrent(advrNew(one.Target), options)
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
import { newBillingTarget } from './explore-concurrent-billing-target.mjs';
import { newRetryTarget } from './explore-concurrent-retry-target.mjs';

const version = (token) => {
  const found = /^v(\d+)$/.exec(typeof token === 'string' ? token : '');
  return found === null ? 0 : Number(found[1]);
};

// The explore fixture target under one of the modes the Go driver lists.
function wrapped(mode) {
  return () => {
    const target = newTarget(mode === 'stale-lost-rejected' ? 'stale-read' : '', 'low');
    let current = 0;
    let snapshot = [];
    let snapAt = 0;
    let reads = 0;
    const begin = target.beginScenario.bind(target);
    const invoke = target.invokeCommand.bind(target);
    const query = target.queryView.bind(target);
    target.beginScenario = (scenario) => {
      current = 0;
      snapshot = [];
      snapAt = 0;
      reads = 0;
      return begin(scenario);
    };
    target.invokeCommand = (request) => {
      const pending = invoke(request);
      return {
        complete: async () => {
          const result = await pending.complete();
          current = Math.max(current, version(result.consistency));
          if ((mode === 'lagging' || mode === 'lagging-deaf') && !result.consistency) {
            return { ...result, consistency: `v${current}` };
          }
          if (
            mode === 'stale-lost-rejected' &&
            request.command === 'explore.desk.OpenTicket' &&
            result.outcome === 'rejected'
          ) {
            throw indeterminate('adversary: the answer was lost');
          }
          return result;
        },
      };
    };
    target.queryView = async (request) => {
      if ((mode === 'lagging' || mode === 'lagging-deaf') && request.view === 'explore.desk.OpenTickets') {
        const demanded = mode === 'lagging-deaf' ? 0 : version(request.atLeast);
        if (demanded > snapAt) {
          snapshot = (await query({ ...request, atLeast: '' })).rows;
          snapAt = current;
        }
        return { rows: [...snapshot] };
      }
      if (mode === 'unsupported-view' && request.view === 'explore.desk.TicketsByItems') {
        throw unsupported('adversary: TicketsByItems is not exposed');
      }
      if (mode === 'read-indeterminate') {
        reads += 1;
        if (reads % 4 === 0) throw indeterminate('adversary: the read timed out');
      }
      return query(request);
    };
    return target;
  };
}

function advrNew(name) {
  switch (name) {
    case 'billing':
      return () => newBillingTarget('');
    case 'retry':
      return () => newRetryTarget('');
    case 'stale':
      return () => newTarget('stale-read', 'low');
    case 'correct':
      return () => newTarget('', 'low');
    default:
      return wrapped(name);
  }
}

const out = process.env.ESS_ADVR_OUT ?? '.';
const cases = JSON.parse(process.env.ESS_ADVR_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    try {
      const result = await exploreConcurrent(advrNew(one.target), {
        ...(one.options ?? {}),
        path: process.env.ESS_ADVR_SPEC,
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

fn go(dir: &Path, spec: &str, cases: &[Value]) -> Lane {
    let module = dir.join("go");
    let package = emit(spec, "go", &module);
    std::fs::write(
        module.join("go.mod"),
        "module example.invalid/adversaryreads\n\ngo 1.24\n",
    )
    .unwrap();
    for (from, to) in [
        ("explore_target.go", "explore_target_test.go"),
        (
            "explore_concurrent_billing_target.go",
            "explore_concurrent_billing_target_test.go",
        ),
        (
            "explore_concurrent_retry_target.go",
            "explore_concurrent_retry_target_test.go",
        ),
    ] {
        std::fs::copy(fixture(from), package.join(to)).unwrap();
    }
    std::fs::write(package.join("adversary_reads_driver_test.go"), GO_DRIVER).unwrap();
    let out = dir.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args(["test", "./essconform", "-run", "TestAdversaryReads"])
        .args(["-count=1", "-v", "-timeout", "60m"])
        .env("ESS_ADVR_CASES", Value::Array(cases.to_vec()).to_string())
        .env("ESS_ADVR_OUT", &out)
        .env("ESS_ADVR_SPEC", root().join(spec))
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

fn typescript(dir: &Path, spec: &str, cases: &[Value]) -> Lane {
    let package = emit(spec, "typescript", &dir.join("typescript"));
    for name in [
        "explore-target.mjs",
        "explore-concurrent-billing-target.mjs",
        "explore-concurrent-retry-target.mjs",
    ] {
        std::fs::copy(fixture(name), package.join(name)).unwrap();
    }
    std::fs::write(package.join("adversary-reads-driver.mjs"), TS_DRIVER).unwrap();
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
    let out = dir.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "adversary-reads-driver.mjs"])
        .env("ESS_ADVR_CASES", Value::Array(cases.to_vec()).to_string())
        .env("ESS_ADVR_OUT", &out)
        .env("ESS_ADVR_SPEC", root().join(spec))
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

fn explore_cases() -> Vec<Value> {
    serde_json::json!([
        {"name": "stale", "target": "stale", "options": {"seeds": 200}},
        {"name": "stale-lost-rejected", "target": "stale-lost-rejected", "options": {"seeds": 200}},
        {"name": "lagging", "target": "lagging", "options": {"seeds": 200}},
        {"name": "lagging-deaf", "target": "lagging-deaf", "options": {"seeds": 200}},
        {"name": "unsupported-view", "target": "unsupported-view", "options": {"seeds": 20, "allowExcluded": true}},
        {"name": "unsupported-view-strict", "target": "unsupported-view", "options": {"seeds": 5}},
        {"name": "read-indeterminate", "target": "read-indeterminate", "options": {"seeds": 50}},
        {"name": "correct-wide", "target": "correct", "options": {"seeds": 200, "clients": 4, "calls": 6}},
    ])
    .as_array()
    .unwrap()
    .clone()
}

fn billing_cases() -> Vec<Value> {
    serde_json::json!([
        {"name": "billing-injected-wide", "target": "billing", "options": {"seeds": 200, "inject": true}},
    ])
    .as_array()
    .unwrap()
    .clone()
}

fn retry_cases() -> Vec<Value> {
    serde_json::json!([
        {"name": "retry-injected-wide", "target": "retry", "options": {"seeds": 200, "inject": true}},
    ])
    .as_array()
    .unwrap()
    .clone()
}

/// (TypeScript, Go) for each specification.
struct Lanes {
    explore: (Lane, Lane),
    billing: (Lane, Lane),
    retry: (Lane, Lane),
}

fn lanes() -> &'static Lanes {
    static LANES: OnceLock<Lanes> = OnceLock::new();
    LANES.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("a scratch directory");
        let explore = explore_cases();
        let billing = billing_cases();
        let retry = retry_cases();
        Lanes {
            explore: (
                typescript(&scratch.path().join("explore"), EXPLORE, &explore),
                go(&scratch.path().join("explore"), EXPLORE, &explore),
            ),
            billing: (
                typescript(&scratch.path().join("billing"), BILLING, &billing),
                go(&scratch.path().join("billing"), BILLING, &billing),
            ),
            retry: (
                typescript(&scratch.path().join("retry"), RETRY, &retry),
                go(&scratch.path().join("retry"), RETRY, &retry),
            ),
        }
    })
}

fn both(pair: &'static (Lane, Lane)) -> [(&'static str, &'static Lane); 2] {
    [("typescript", &pair.0), ("go", &pair.1)]
}

fn ok<'a>(language: &str, lane: &'a Lane, name: &str) -> &'a Value {
    match &lane.cases[name].result {
        Ok(value) => value,
        Err(refusal) => panic!("{language}: `{name}` was refused: {refusal}"),
    }
}

fn count(value: &Value, key: &str) -> u64 {
    value[key]
        .as_u64()
        .unwrap_or_else(|| panic!("`{key}` is a count: {value}"))
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

fn verdicts(value: &Value) -> Vec<String> {
    value["verdicts"]
        .as_array()
        .expect("verdicts")
        .iter()
        .map(|it| it.as_str().unwrap().to_owned())
        .collect()
}

// ---- the "no rows when a creation never answered" rule hides real stale reads ------------------

/// A lost answer of an `OpenTicket` the target **rejected** created nothing, so no read can show
/// an instance nobody named; the same target state, schedule and reads as the plain `stale-read`
/// run follow. Every seed that run found a violation in is still one.
#[test]
fn a_lost_answer_of_a_rejected_creation_does_not_hide_a_stale_read() {
    for (language, lane) in both(&lanes().explore) {
        let stale = ok(language, lane, "stale");
        let lost = ok(language, lane, "stale-lost-rejected");
        let lost_creations = operations(&lane.cases["stale-lost-rejected"])
            .iter()
            .filter(|operation| {
                operation["command"] == "explore.desk.OpenTicket"
                    && operation["completion"] == "Indeterminate"
            })
            .count();
        assert!(
            lost_creations > 0,
            "{language}: no rejected creation's answer was lost, so this case says nothing"
        );
        let before = verdicts(stale);
        let after = verdicts(lost);
        let hidden: Vec<usize> = before
            .iter()
            .zip(&after)
            .enumerate()
            .filter(|(_, (was, now))| *was == "Violation" && *now != "Violation")
            .map(|(index, _)| index + 1)
            .collect();
        let unjudged = operations(&lane.cases["stale-lost-rejected"])
            .iter()
            .filter(|operation| {
                operation["command"] == OPEN_TICKETS
                    && operation["completion"] == "Returned"
                    && operation.get("rows").is_none()
            })
            .count();
        assert!(
            hidden.is_empty(),
            "{language}: `stale-read` found {} violation(s) over 200 seeds; with the answers of {lost_creations} \
             rejected (nothing-creating) `OpenTicket` calls lost it finds {}. {} seed(s) that were a \
             violation are no longer one ({:?}...), and {unjudged} answered `OpenTickets` read(s) were written \
             without `rows`",
            count(stale, "violations"),
            count(lost, "violations"),
            hidden.len(),
            &hidden[..hidden.len().min(10)],
        );
    }
}

// ---- `read_your_writes` tokens -----------------------------------------------------------------

/// A target that answers `OpenTickets` from a snapshot, refreshed only when a read demands a token
/// newer than it, keeps `read_your_writes` exactly when the reader sends its last token: 0
/// violations. The same target deaf to the token is caught, so a recorder that sent no token would
/// be caught by the first case — which no unit fixture can do, since none reads `AtLeast`.
#[test]
fn a_target_lagging_until_a_token_is_demanded_is_clean_and_a_deaf_one_is_caught() {
    let (typescript, go) = &lanes().explore;
    for (language, lane) in both(&lanes().explore) {
        let lagging = ok(language, lane, "lagging");
        assert_eq!(count(lagging, "histories"), 200, "{language}: {lagging}");
        assert_eq!(count(lagging, "violations"), 0, "{language}: {lagging}");
        assert_eq!(count(lagging, "unknown"), 0, "{language}: {lagging}");
        let deaf = ok(language, lane, "lagging-deaf");
        assert!(count(deaf, "violations") >= 1, "{language}: {deaf}");
        assert!(
            deaf["failure"]["report"]
                .as_str()
                .unwrap_or("")
                .contains("declared read_your_writes"),
            "{language}: {deaf}"
        );
    }
    assert_eq!(
        typescript.cases["lagging"].histories,
        go.cases["lagging"].histories
    );
}

// ---- a view the target does not expose ---------------------------------------------------------

#[test]
fn a_view_the_target_does_not_expose_is_excluded_and_never_written() {
    let (typescript, go) = &lanes().explore;
    for (language, lane) in both(&lanes().explore) {
        let allowed = ok(language, lane, "unsupported-view");
        assert!(
            allowed["excluded"]
                .as_array()
                .is_some_and(|excluded| excluded.iter().any(|it| it["subject"] == BY_ITEMS)),
            "{language}: {allowed}"
        );
        assert_eq!(
            lane.cases["unsupported-view"].problem.as_deref(),
            Some("none")
        );
        let written = operations(&lane.cases["unsupported-view"]);
        assert!(written.iter().all(|it| it["command"] != BY_ITEMS));
        assert!(written.iter().any(|it| it["command"] == OPEN_TICKETS));
        ok(language, lane, "unsupported-view-strict");
        assert_ne!(
            lane.cases["unsupported-view-strict"].problem.as_deref(),
            Some("none"),
            "{language}"
        );
    }
    assert_eq!(
        typescript.cases["unsupported-view"].histories,
        go.cases["unsupported-view"].histories
    );
    assert_eq!(
        ok("typescript", typescript, "unsupported-view"),
        ok("go", go, "unsupported-view")
    );
}

// ---- a read that never answered -----------------------------------------------------------------

#[test]
fn a_read_that_timed_out_is_written_indeterminate_without_rows_and_equally() {
    let (typescript, go) = &lanes().explore;
    for (language, lane) in both(&lanes().explore) {
        let result = ok(language, lane, "read-indeterminate");
        assert_eq!(count(result, "violations"), 0, "{language}: {result}");
        let reads: Vec<Value> = operations(&lane.cases["read-indeterminate"])
            .into_iter()
            .filter(|it| it["command"] == OPEN_TICKETS || it["command"] == BY_ITEMS)
            .collect();
        let lost: Vec<&Value> = reads
            .iter()
            .filter(|it| it["completion"] == "Indeterminate")
            .collect();
        assert!(!lost.is_empty(), "{language}");
        for read in lost {
            assert!(read.get("rows").is_none(), "{read}");
            assert!(read.get("outcome").is_none(), "{read}");
            assert!(read.get("returned_at").is_none(), "{read}");
        }
    }
    assert_eq!(
        typescript.cases["read-indeterminate"].histories,
        go.cases["read-indeterminate"].histories
    );
    assert_eq!(
        ok("typescript", typescript, "read-indeterminate"),
        ok("go", go, "read-indeterminate")
    );
}

// ---- a correct target over many seeds is never a violation --------------------------------------

#[test]
fn correct_targets_over_200_seeds_wider_and_injected_give_no_violation() {
    for (pair, name) in [
        (&lanes().explore, "correct-wide"),
        (&lanes().billing, "billing-injected-wide"),
        (&lanes().retry, "retry-injected-wide"),
    ] {
        for (language, lane) in both(pair) {
            let result = ok(language, lane, name);
            assert_eq!(
                count(result, "histories"),
                200,
                "{language} {name}: {result}"
            );
            assert_eq!(
                count(result, "violations"),
                0,
                "{language} {name}: {}",
                result["failure"]["report"]
            );
        }
        assert_eq!(
            pair.0.cases[name].histories, pair.1.cases[name].histories,
            "{name}"
        );
    }
}
