//! Second adversarial pass against the view reads of the concurrent Go and TypeScript explorers
//! (`story:explorers-record-view-reads`): the narrowed rule that writes a read without `rows` when
//! one of its rows is an identity no written operation names and a creation of the view's entity,
//! invoked before the read returned, never answered.
//!
//! Each package is emitted by this tree's `ess`, the fixture targets and a driver written by this
//! file are placed in it. The driver's wrapper targets log what each view answered (or how many
//! instances nobody created a read showed), so a case can tell what the rule withheld from the
//! checker. A missing `go`, `node` or `tsc` panics rather than skipping.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

const EXPLORE: &str = "crates/verify/ess-conformance/tests/fixtures/explore.yaml";
const BILLING: &str = "examples/billing";

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
	"strings"
	"testing"
)

type advr2Case struct {
	Name    string            `json:"name"`
	Target  string            `json:"target"`
	Options ConcurrentOptions `json:"options"`
}

const advr2Prefix = "explore/concurrent/seed-"

func advr2Seed(scenario ScenarioContext) string {
	if strings.HasPrefix(scenario.Scenario, advr2Prefix) {
		return strings.TrimPrefix(scenario.Scenario, advr2Prefix)
	}
	return ""
}

func advr2Append(path string, lines []string) {
	file, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		panic(err)
	}
	defer file.Close()
	for _, line := range lines {
		if _, err := file.WriteString(line + "\n"); err != nil {
			panic(err)
		}
	}
}

// advr2Explore is the explore fixture target whose third and fifth `OpenTicket` that opened a
// ticket lose their answer after the ticket was created. It logs every view answer, in order,
// as `rows` with the tickets of those lost answers removed and as `answered` whole.
type advr2Explore struct {
	*exploreFixtureTarget
	log    string
	seed   string
	opened int
	lost   map[string]bool
	lines  []string
}

func (t *advr2Explore) BeginScenario(scenario ScenarioContext) error {
	t.seed, t.opened, t.lost, t.lines = advr2Seed(scenario), 0, map[string]bool{}, nil
	return t.exploreFixtureTarget.BeginScenario(scenario)
}

func (t *advr2Explore) EndScenario(scenario ScenarioContext) error {
	if t.seed != "" {
		advr2Append(t.log, t.lines)
	}
	t.lines = nil
	return t.exploreFixtureTarget.EndScenario(scenario)
}

type advr2Pending struct {
	target  *advr2Explore
	inner   PendingCommand
	command string
}

func (p advr2Pending) Complete() (CommandResult, error) {
	result, err := p.inner.Complete()
	if err != nil || p.command != "explore.desk.OpenTicket" || result.Outcome != "opened" {
		return result, err
	}
	p.target.opened++
	if p.target.opened == 3 || p.target.opened == 5 {
		p.target.lost[fmt.Sprintf("00000000-0000-4000-8000-%012d", p.target.opened)] = true
		return CommandResult{}, fmt.Errorf("adversary: the answer was lost: %w", ErrIndeterminate)
	}
	return result, nil
}

func (t *advr2Explore) InvokeCommand(request CommandRequest) PendingCommand {
	return advr2Pending{target: t, inner: t.exploreFixtureTarget.InvokeCommand(request), command: request.Command}
}

func (t *advr2Explore) QueryView(request ViewRequest) (ViewResult, error) {
	answer, err := t.exploreFixtureTarget.QueryView(request)
	if err != nil {
		return answer, err
	}
	rows := []string{}
	answered := []string{}
	for _, row := range answer.Rows {
		id, _ := row["ticket_id"].(string)
		answered = append(answered, id)
		if !t.lost[id] {
			rows = append(rows, id)
		}
	}
	line, _ := json.Marshal(map[string]any{"seed": t.seed, "view": request.View, "rows": rows, "answered": answered})
	t.lines = append(t.lines, string(line))
	return answer, nil
}

// advr2Billing is the billing target under `double-apply`, whose first `CreateInvoice` the target
// rejected (so it created nothing) loses its answer. It logs, per seed, the most invoices nobody
// created (second applications) an `InvoiceById` read showed after that answer was lost.
type advr2Billing struct {
	*exploreBillingTarget
	log      string
	seed     string
	lost     bool
	phantoms int
	shown    int
}

func (t *advr2Billing) BeginScenario(scenario ScenarioContext) error {
	t.seed, t.lost, t.phantoms, t.shown = advr2Seed(scenario), false, 0, 0
	return t.exploreBillingTarget.BeginScenario(scenario)
}

func (t *advr2Billing) EndScenario(scenario ScenarioContext) error {
	if t.seed != "" {
		line, _ := json.Marshal(map[string]any{"seed": t.seed, "phantoms": t.shown})
		advr2Append(t.log, []string{string(line)})
	}
	return t.exploreBillingTarget.EndScenario(scenario)
}

func (t *advr2Billing) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	result, err := t.exploreBillingTarget.ExecuteCommand(request)
	if err == nil && request.Command == "billing.invoice.CreateInvoice" && result.Outcome == "rejected" && !t.lost {
		t.lost = true
		return CommandResult{}, fmt.Errorf("adversary: the answer was lost: %w", ErrIndeterminate)
	}
	return result, err
}

func (t *advr2Billing) RedeliverEvent(request RedeliveryRequest) error {
	err := t.exploreBillingTarget.RedeliverEvent(request)
	if err == nil && request.Event == "billing.invoice.InvoiceCreated" {
		t.phantoms++
	}
	return err
}

func (t *advr2Billing) QueryView(request ViewRequest) (ViewResult, error) {
	answer, err := t.exploreBillingTarget.QueryView(request)
	if err == nil && request.View == "billing.invoice.InvoiceById" && t.lost && t.phantoms > t.shown {
		t.shown = t.phantoms
	}
	return answer, err
}

func advr2New(name, target, out string) func() Target {
	log := filepath.Join(out, name+".log")
	switch target {
	case "stale-lost-opened":
		return func() Target { return &advr2Explore{exploreFixtureTarget: newExploreFixtureTarget("stale-read", "low"), log: log} }
	case "correct-lost-opened":
		return func() Target { return &advr2Explore{exploreFixtureTarget: newExploreFixtureTarget("", "low"), log: log} }
	case "double-apply":
		return func() Target { return newExploreBillingTarget("double-apply") }
	case "double-apply-lost-rejected":
		return func() Target { return &advr2Billing{exploreBillingTarget: newExploreBillingTarget("double-apply"), log: log} }
	}
	panic("unknown target " + target)
}

func TestAdversaryReads2(t *testing.T) {
	out := os.Getenv("ESS_ADVR_OUT")
	var cases []advr2Case
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
			result, err := ExploreConcurrent(advr2New(one.Name, one.Target, out), options)
			if err != nil {
				write(".refused", err.Error())
				return
			}
			bytes, err := json.Marshal(result)
			if err != nil {
				t.Fatal(err)
			}
			write(".json", string(bytes))
		})
	}
}
"#;

const TS_DRIVER: &str = r"import { appendFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { exploreConcurrent, goMarshal, indeterminate } from './dist/index.js';
import { newTarget } from './explore-target.mjs';
import { newBillingTarget } from './explore-concurrent-billing-target.mjs';

const PREFIX = 'explore/concurrent/seed-';
const seedOf = (scenario) =>
  scenario.scenario.startsWith(PREFIX) ? scenario.scenario.slice(PREFIX.length) : '';
const append = (path, lines) => {
  if (lines.length > 0) appendFileSync(path, lines.map((line) => `${line}\n`).join(''));
};

// The Go driver's advr2Explore, line for line.
function lostOpened(mutant, log) {
  return () => {
    const target = newTarget(mutant, 'low');
    let seed = '';
    let opened = 0;
    let lost = new Set();
    let lines = [];
    const begin = target.beginScenario.bind(target);
    const end = target.endScenario.bind(target);
    const invoke = target.invokeCommand.bind(target);
    const query = target.queryView.bind(target);
    target.beginScenario = (scenario) => {
      seed = seedOf(scenario);
      opened = 0;
      lost = new Set();
      lines = [];
      return begin(scenario);
    };
    target.endScenario = (scenario) => {
      if (seed !== '') append(log, lines);
      lines = [];
      return end(scenario);
    };
    target.invokeCommand = (request) => {
      const pending = invoke(request);
      return {
        complete: async () => {
          const result = await pending.complete();
          if (request.command !== 'explore.desk.OpenTicket' || result.outcome !== 'opened') {
            return result;
          }
          opened += 1;
          if (opened === 3 || opened === 5) {
            lost.add(`00000000-0000-4000-8000-${String(opened).padStart(12, '0')}`);
            throw indeterminate('adversary: the answer was lost');
          }
          return result;
        },
      };
    };
    target.queryView = async (request) => {
      const answer = await query(request);
      const answered = answer.rows.map((row) => row.ticket_id);
      const rows = answered.filter((id) => !lost.has(id));
      lines.push(JSON.stringify({ seed, view: request.view, rows, answered }));
      return answer;
    };
    return target;
  };
}

// The Go driver's advr2Billing, line for line.
function lostRejected(log) {
  return () => {
    const target = newBillingTarget('double-apply');
    let seed = '';
    let lost = false;
    let phantoms = 0;
    let shown = 0;
    const begin = target.beginScenario.bind(target);
    const end = target.endScenario.bind(target);
    const execute = target.executeCommand.bind(target);
    const redeliver = target.redeliverEvent.bind(target);
    const query = target.queryView.bind(target);
    target.beginScenario = (scenario) => {
      seed = seedOf(scenario);
      lost = false;
      phantoms = 0;
      shown = 0;
      return begin(scenario);
    };
    target.endScenario = (scenario) => {
      if (seed !== '') append(log, [JSON.stringify({ seed, phantoms: shown })]);
      return end(scenario);
    };
    target.executeCommand = async (request) => {
      const result = await execute(request);
      if (request.command === 'billing.invoice.CreateInvoice' && result.outcome === 'rejected' && !lost) {
        lost = true;
        throw indeterminate('adversary: the answer was lost');
      }
      return result;
    };
    target.redeliverEvent = async (request) => {
      await redeliver(request);
      if (request.event === 'billing.invoice.InvoiceCreated') phantoms += 1;
    };
    target.queryView = async (request) => {
      const answer = await query(request);
      if (request.view === 'billing.invoice.InvoiceById' && lost && phantoms > shown) shown = phantoms;
      return answer;
    };
    return target;
  };
}

function advrNew(name, target, out) {
  const log = join(out, `${name}.log`);
  switch (target) {
    case 'stale-lost-opened':
      return lostOpened('stale-read', log);
    case 'correct-lost-opened':
      return lostOpened('', log);
    case 'double-apply':
      return () => newBillingTarget('double-apply');
    case 'double-apply-lost-rejected':
      return lostRejected(log);
  }
  throw new Error(`unknown target ${target}`);
}

const out = process.env.ESS_ADVR_OUT ?? '.';
const cases = JSON.parse(process.env.ESS_ADVR_CASES ?? '[]');
for (const one of cases) {
  await test(one.name, async () => {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    try {
      const result = await exploreConcurrent(advrNew(one.name, one.target, out), {
        ...(one.options ?? {}),
        path: process.env.ESS_ADVR_SPEC,
        out: join(out, one.name),
      });
      write('.json', goMarshal(result));
    } catch (error) {
      write('.refused', error instanceof Error ? error.message : String(error));
    }
  });
}
";

#[derive(Debug)]
struct Case {
    result: Result<Value, String>,
    histories: BTreeMap<u64, Vec<u8>>,
    log: Vec<Value>,
}

type Lane = BTreeMap<String, Case>;

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
        let lines = std::fs::read_to_string(out.join(format!("{name}.log"))).unwrap_or_default();
        let log = lines
            .lines()
            .map(|line| serde_json::from_str(line).expect("a log line is JSON"))
            .collect();
        read.insert(
            name,
            Case {
                result,
                histories,
                log,
            },
        );
    }
    read
}

fn go(dir: &Path, spec: &str, cases: &[Value]) -> Lane {
    let module = dir.join("go");
    let package = emit(spec, "go", &module);
    std::fs::write(
        module.join("go.mod"),
        "module example.invalid/adversaryreads2\n\ngo 1.24\n",
    )
    .unwrap();
    for (from, to) in [
        ("explore_target.go", "explore_target_test.go"),
        (
            "explore_concurrent_billing_target.go",
            "explore_concurrent_billing_target_test.go",
        ),
    ] {
        std::fs::copy(fixture(from), package.join(to)).unwrap();
    }
    std::fs::write(package.join("adversary_reads2_driver_test.go"), GO_DRIVER).unwrap();
    let out = dir.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args(["test", "./essconform", "-run", "TestAdversaryReads2"])
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
    ] {
        std::fs::copy(fixture(name), package.join(name)).unwrap();
    }
    std::fs::write(package.join("adversary-reads2-driver.mjs"), TS_DRIVER).unwrap();
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
        .args(["--test", "adversary-reads2-driver.mjs"])
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
        {"name": "stale-lost-opened", "target": "stale-lost-opened", "options": {"seeds": 200}},
        {"name": "stale-lost-opened-again", "target": "stale-lost-opened", "options": {"seeds": 200}},
        {"name": "correct-lost-opened", "target": "correct-lost-opened", "options": {"seeds": 200}},
    ])
    .as_array()
    .unwrap()
    .clone()
}

fn billing_cases() -> Vec<Value> {
    serde_json::json!([
        {"name": "double-apply", "target": "double-apply", "options": {"seeds": 200, "inject": true}},
        {"name": "double-apply-lost-rejected", "target": "double-apply-lost-rejected", "options": {"seeds": 200, "inject": true}},
    ])
    .as_array()
    .unwrap()
    .clone()
}

/// (TypeScript, Go) for each specification.
struct Lanes {
    explore: (Lane, Lane),
    billing: (Lane, Lane),
}

fn lanes() -> &'static Lanes {
    static LANES: OnceLock<Lanes> = OnceLock::new();
    LANES.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("a scratch directory");
        let explore = explore_cases();
        let billing = billing_cases();
        Lanes {
            explore: (
                typescript(&scratch.path().join("explore"), EXPLORE, &explore),
                go(&scratch.path().join("explore"), EXPLORE, &explore),
            ),
            billing: (
                typescript(&scratch.path().join("billing"), BILLING, &billing),
                go(&scratch.path().join("billing"), BILLING, &billing),
            ),
        }
    })
}

fn both(pair: &'static (Lane, Lane)) -> [(&'static str, &'static Lane); 2] {
    [("typescript", &pair.0), ("go", &pair.1)]
}

fn ok<'a>(language: &str, lane: &'a Lane, name: &str) -> &'a Value {
    match &lane[name].result {
        Ok(value) => value,
        Err(refusal) => panic!("{language}: `{name}` was refused: {refusal}"),
    }
}

fn verdicts(value: &Value) -> Vec<String> {
    value["verdicts"]
        .as_array()
        .expect("verdicts")
        .iter()
        .map(|it| it.as_str().unwrap().to_owned())
        .collect()
}

fn log_seed(line: &Value) -> u64 {
    line["seed"].as_str().unwrap().parse().unwrap()
}

/// `check-history`'s exit code for `history` against `spec`.
fn check(spec: &str, history: &Path) -> i32 {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args([
            "verify",
            "conform",
            "check-history",
            "--path",
            spec,
            "--history",
        ])
        .arg(history)
        .output()
        .expect("the `ess` binary runs");
    output.status.code().expect("`ess` exited")
}

/// The history of `seed` with each answered read the explorer wrote with fewer `rows` than the view
/// answered (or none) given the rows the view answered minus the instances of the lost creations;
/// `None` when no read was withheld a row.
fn with_rows_restored(bytes: &[u8], answers: &[&Value]) -> Option<Value> {
    let mut document: Value = serde_json::from_slice(bytes).unwrap();
    let operations = document["operations"].as_array_mut().unwrap();
    let mut reads: Vec<&mut Value> = operations
        .iter_mut()
        .filter(|it| it["outcome"] == "read")
        .collect();
    reads.sort_by_key(|it| it["returned_at"].as_u64().unwrap());
    assert_eq!(
        reads.len(),
        answers.len(),
        "every answered read is one logged view answer"
    );
    let mut restored = false;
    for (read, answer) in reads.into_iter().zip(answers) {
        assert_eq!(read["command"], answer["view"]);
        if read.get("rows") != Some(&answer["answered"]) {
            read["rows"] = answer["rows"].clone();
            restored = true;
        }
    }
    restored.then_some(document)
}

/// What was decided for a read showing the instance of a creation that never answered, checked on
/// every seed of `name` against the logged answers: every answered row whose identity some written
/// operation names is written, and the rows left out are no more than the `OpenTicket` calls that
/// never answered, name nothing, and were invoked before the read returned. The number of reads
/// that were written with a row left out.
fn decided_rows(language: &str, lane: &Lane, name: &str) -> usize {
    let case = &lane[name];
    let mut left_out = 0;
    for (seed, bytes) in &case.histories {
        let answers: Vec<&Value> = case.log.iter().filter(|it| log_seed(it) == *seed).collect();
        let document: Value = serde_json::from_slice(bytes).unwrap();
        let operations = document["operations"].as_array().unwrap();
        let named: std::collections::BTreeSet<&str> = operations
            .iter()
            .map(|it| it["subject_key"].as_str().unwrap())
            .filter(|key| !key.is_empty())
            .collect();
        let mut reads: Vec<&Value> = operations
            .iter()
            .filter(|it| it["outcome"] == "read")
            .collect();
        reads.sort_by_key(|it| it["returned_at"].as_u64().unwrap());
        assert_eq!(reads.len(), answers.len(), "{language} {name} seed {seed}");
        for (read, answer) in reads.into_iter().zip(answers) {
            let written: Vec<&str> = read["rows"]
                .as_array()
                .unwrap_or_else(|| panic!("{language} {name} seed {seed}: a read without rows"))
                .iter()
                .map(|it| it.as_str().unwrap())
                .collect();
            let answered: Vec<&str> = answer["answered"]
                .as_array()
                .unwrap()
                .iter()
                .map(|it| it.as_str().unwrap())
                .collect();
            for row in &answered {
                assert!(
                    !named.contains(row) || written.contains(row),
                    "{language} {name} seed {seed}: the named row `{row}` was withheld from {read}"
                );
            }
            let returned = read["returned_at"].as_u64().unwrap();
            let eligible = operations
                .iter()
                .filter(|it| {
                    it["command"] == "explore.desk.OpenTicket"
                        && it["completion"] == "Indeterminate"
                        && it["subject_key"] == ""
                        && it["invoked_at"].as_u64().unwrap() < returned
                })
                .count();
            let omitted = answered.len() - written.len();
            assert!(
                omitted <= eligible,
                "{language} {name} seed {seed}: {omitted} row(s) left out of {read}, and only \
                 {eligible} creation(s) that never answered could account for them"
            );
            if omitted > 0 {
                left_out += 1;
            }
        }
    }
    left_out
}

/// Restores the withheld reads of every seed of `name` and checks the result: (seeds with a
/// withheld read, seeds whose restored history is a violation while the written one is not).
fn restored_verdicts(language: &str, lane: &Lane, name: &str) -> (usize, Vec<u64>) {
    let case = &lane[name];
    let written = verdicts(ok(language, lane, name));
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let mut withheld = 0;
    let mut hidden = Vec::new();
    for (seed, bytes) in &case.histories {
        let answers: Vec<&Value> = case.log.iter().filter(|it| log_seed(it) == *seed).collect();
        let Some(document) = with_rows_restored(bytes, &answers) else {
            continue;
        };
        withheld += 1;
        let path = scratch.path().join(format!("restored-{seed}.json"));
        std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        let code = check(EXPLORE, &path);
        assert_ne!(
            code, 2,
            "{language} {name} seed {seed}: the restored history was refused"
        );
        if code == 1 && written[usize::try_from(*seed - 1).unwrap()] != "Violation" {
            hidden.push(*seed);
        }
    }
    (withheld, hidden)
}

// ---- the whole read is withheld, and the stale part with it ----------------------------------------

/// A read that shows the ticket of a lost `OpenTicket` answer may leave that row out, and only that
/// row: the control checks, on the correct target, that every named row is written and no more
/// rows are left out than lost creations can account for, and that restoring the rows of the
/// tickets everyone knows never makes a violation. Then no `stale-read` history judged clean is a
/// violation once those rows are restored.
#[test]
fn a_stale_read_beside_the_row_of_a_lost_creation_is_not_hidden() {
    for (language, lane) in both(&lanes().explore) {
        let left_out = decided_rows(language, lane, "correct-lost-opened");
        assert!(
            left_out > 0,
            "{language}: no read of the correct target left a row out, so the control says nothing"
        );
        decided_rows(language, lane, "stale-lost-opened");
        let (control_withheld, control_hidden) =
            restored_verdicts(language, lane, "correct-lost-opened");
        assert!(
            control_withheld > 0,
            "{language}: no read of the correct target was withheld, so the control says nothing"
        );
        assert!(
            control_hidden.is_empty(),
            "{language}: restoring the known rows of a correct target made violations in seeds \
             {control_hidden:?}, so the restoration is not sound and the next assertion proves nothing"
        );
        let (withheld, hidden) = restored_verdicts(language, lane, "stale-lost-opened");
        let written = ok(language, lane, "stale-lost-opened");
        assert!(
            hidden.is_empty(),
            "{language}: over 200 seeds the `stale-read` target with two `OpenTicket` answers lost gives \
             {} violation(s); {withheld} seed(s) had a read written without `rows`, and in {} of them the \
             rows of the named tickets alone are a violation the written history does not show (seeds \
             {:?})",
            written["violations"],
            hidden.len(),
            &hidden[..hidden.len().min(10)],
        );
    }
}

// ---- more instances nobody created than creations that never answered ------------------------------

/// Under `double-apply` each second delivery of `InvoiceCreated` makes an invoice nobody created.
/// When one `CreateInvoice` that created nothing loses its answer, a read showing two or more such
/// invoices shows at least one no lost creation can explain: a violation whatever the lost call did.
#[test]
fn more_phantoms_than_lost_creations_are_still_a_violation() {
    for (language, lane) in both(&lanes().billing) {
        let plain = ok(language, lane, "double-apply");
        let lost = ok(language, lane, "double-apply-lost-rejected");
        let written = verdicts(lost);
        let definite: Vec<u64> = lane["double-apply-lost-rejected"]
            .log
            .iter()
            .filter(|it| it["phantoms"].as_u64().unwrap() >= 2)
            .map(log_seed)
            .collect();
        assert!(
            !definite.is_empty(),
            "{language}: no read showed two invoices nobody created after a lost answer; the case says nothing"
        );
        let missed: Vec<u64> = definite
            .iter()
            .copied()
            .filter(|seed| written[usize::try_from(*seed - 1).unwrap()] != "Violation")
            .collect();
        assert!(
            missed.is_empty(),
            "{language}: `double-apply` with injection gives {} violation(s) over 200 seeds; with the answer \
             of one rejected (nothing-creating) `CreateInvoice` lost it gives {}. In {} of the {} seed(s) where \
             an `InvoiceById` read showed two or more invoices nobody created after that one lost answer, \
             the history is not a violation (seeds {:?})",
            plain["violations"],
            lost["violations"],
            missed.len(),
            definite.len(),
            &missed[..missed.len().min(10)],
        );
    }
}

// ---- Go and TypeScript agree on the rule's edge cases, and a seed is one history --------------------

#[test]
fn the_rule_writes_equal_bytes_in_go_and_typescript_and_again() {
    for (pair, names) in [
        (
            &lanes().explore,
            &["stale-lost-opened", "correct-lost-opened"][..],
        ),
        (
            &lanes().billing,
            &["double-apply", "double-apply-lost-rejected"][..],
        ),
    ] {
        for name in names {
            let (typescript, go) = pair;
            assert_eq!(typescript[*name].histories.len(), 200, "{name}");
            assert!(
                typescript[*name].histories == go[*name].histories,
                "{name}: the TypeScript and Go histories differ"
            );
            assert_eq!(
                ok("typescript", typescript, name),
                ok("go", go, name),
                "{name}"
            );
        }
    }
    for (language, lane) in both(&lanes().explore) {
        assert!(
            lane["stale-lost-opened"].histories == lane["stale-lost-opened-again"].histories,
            "{language}: one seed recorded twice is two histories"
        );
    }
}
