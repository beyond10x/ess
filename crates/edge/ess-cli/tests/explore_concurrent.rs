//! The concurrent mode of the emitted Go and TypeScript explorers
//! (`story:concurrent-explorer-runner`).
//!
//! Each package is emitted by the `ess` binary built from this tree, exactly as an adopter emits
//! it, and run against the explore fixture target (`explore.yaml`, switched by its mutant name) and
//! against a small billing target (`examples/billing`). The runner writes one `ess-history/1`
//! document per seed and hands it to `ess verify conform check-history`; the `ess` it finds is this
//! tree's, placed first on `PATH`. This file lives in `ess-cli` rather than beside
//! `crates/verify/ess-conformance/tests/explore.rs` because it needs `CARGO_BIN_EXE_ess`.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping: a lane that ran nothing and exited
//! zero is the failure this repository already knows by name.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

const EXPLORE: &str = "crates/verify/ess-conformance/tests/fixtures/explore.yaml";
const BILLING: &str = "examples/billing";
const RETRY: &str = "crates/verify/ess-conformance/tests/fixtures/explore-retry/retry.yaml";
const SEEDS: u64 = 200;
const BILLING_SEEDS: u64 = 40;
const RETRY_SEEDS: u64 = 20;
const PAIR: &str = "crates/verify/ess-conformance/tests/fixtures/explore-pair/pair.yaml";
const PAIR_SEEDS: u64 = 40;

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

/// `PATH` with this tree's `ess` in front of whatever is installed.
fn path_with_ess() -> String {
    let ess = Path::new(env!("CARGO_BIN_EXE_ess"));
    let directory = ess.parent().expect("the binary has a directory");
    match std::env::var("PATH") {
        Ok(path) => format!("{}:{path}", directory.display()),
        Err(_) => directory.display().to_string(),
    }
}

/// Emits the package for `spec` in `language` under `out`, through the `ess` binary.
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

/// What one lane wrote: each case's result or refusal, its histories by seed, and the draws.
#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Result<Value, String>>,
    /// What `CheckConcurrent` / `concurrentProblem` said of each result, or `none`.
    problems: BTreeMap<String, String>,
    histories: BTreeMap<String, BTreeMap<u64, Vec<u8>>>,
    splitmix64: String,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut problems = BTreeMap::new();
    let mut histories = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let result = match std::fs::read_to_string(out.join(format!("{name}.json"))) {
            Ok(text) => Ok(serde_json::from_str(&text).expect("the result is JSON")),
            Err(_) => Err(std::fs::read_to_string(out.join(format!("{name}.refused")))
                .unwrap_or_else(|_| panic!("`{name}` wrote neither a result nor a refusal:\n{log}"))
                .trim_end()
                .to_owned()),
        };
        let mut written = BTreeMap::new();
        if let Ok(entries) = std::fs::read_dir(out.join(&name)) {
            for entry in entries {
                let path = entry.expect("an entry").path();
                let file = path.file_name().unwrap().to_string_lossy().into_owned();
                let seed = file
                    .strip_prefix("history-")
                    .and_then(|rest| rest.strip_suffix(".json"))
                    .unwrap_or_else(|| panic!("`{file}` is not a history file"))
                    .parse::<u64>()
                    .expect("a seed");
                written.insert(seed, std::fs::read(&path).expect("readable"));
            }
        }
        if let Ok(problem) = std::fs::read_to_string(out.join(format!("{name}.problem"))) {
            problems.insert(name.clone(), problem.trim_end().to_owned());
        }
        results.insert(name.clone(), result);
        histories.insert(name, written);
    }
    let splitmix64 = std::fs::read_to_string(out.join("splitmix64"))
        .unwrap_or_else(|_| panic!("the lane wrote no draws:\n{log}"))
        .trim_end()
        .to_owned();
    Lane {
        results,
        problems,
        histories,
        splitmix64,
        log,
    }
}

/// The Go lane: the emitted package as a module of its own, run with `go test`.
fn go(root: &Path, spec: &str, cases: &Value) -> Lane {
    let module = root.join("go");
    let package = emit(spec, "go", &module);
    std::fs::write(
        module.join("go.mod"),
        "module example.invalid/concurrent\n\ngo 1.24\n",
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
        (
            "explore_concurrent_pair_target.go",
            "explore_concurrent_pair_target_test.go",
        ),
        (
            "explore_concurrent_driver_test.go",
            "explore_concurrent_driver_test.go",
        ),
    ] {
        std::fs::copy(fixture(from), package.join(to)).expect("the fixture copies");
    }
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args(["test", "./essconform", "-run", "TestExploreConcurrent"])
        .args(["-count=1", "-v", "-timeout", "30m"])
        .env("ESS_CONCURRENT_CASES", cases.to_string())
        .env("ESS_CONCURRENT_OUT", &out)
        .env("ESS_CONCURRENT_SPEC", root_of(spec))
        .env("PATH", path_with_ess())
        .env("GOWORK", "off")
        .env_remove("ESS_EXPLORE_MUTANT")
        .env_remove("ESS_EXPLORE_GRADE")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH: the concurrent lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for case in cases.as_array().unwrap() {
        let line = format!(
            "--- PASS: TestExploreConcurrent/{}",
            case["name"].as_str().unwrap()
        );
        assert!(
            log.lines().any(|it| it.trim_start().starts_with(&line)),
            "`{line}` is not in the go log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

/// The TypeScript lane: the emitted package, compiled with `tsc --noCheck`, run with `node --test`.
fn typescript(root: &Path, spec: &str, cases: &Value) -> Lane {
    let package = emit(spec, "typescript", &root.join("typescript"));
    for name in [
        "explore-target.mjs",
        "explore-concurrent-billing-target.mjs",
        "explore-concurrent-retry-target.mjs",
        "explore-concurrent-pair-target.mjs",
        "explore-concurrent-driver.mjs",
    ] {
        std::fs::copy(fixture(name), package.join(name)).expect("the fixture copies");
    }
    std::fs::write(
        package.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&package)
        .output()
        .expect("`tsc` is on PATH: the concurrent lane does not skip");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let out = root.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "explore-concurrent-driver.mjs"])
        .env("ESS_CONCURRENT_CASES", cases.to_string())
        .env("ESS_CONCURRENT_OUT", &out)
        .env("ESS_CONCURRENT_SPEC", root_of(spec))
        .env("PATH", path_with_ess())
        .env_remove("ESS_EXPLORE_MUTANT")
        .env_remove("ESS_EXPLORE_GRADE")
        .current_dir(&package)
        .output()
        .expect("`node` is on PATH: the concurrent lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let line = format!("ok {} - {}", index + 1, case["name"].as_str().unwrap());
        assert!(
            log.lines().any(|it| it == line),
            "`{line}` is not in the node log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

fn root_of(spec: &str) -> PathBuf {
    root().join(spec)
}

/// An empty directory, standing for a `PATH` on which there is no `ess`.
fn no_ess(scratch: &Path) -> PathBuf {
    let empty = scratch.join("empty-path");
    std::fs::create_dir_all(&empty).unwrap();
    empty
}

fn explore_cases(scratch: &Path) -> Value {
    serde_json::json!([
        {"name": "correct", "target": "explore", "options": {"seeds": SEEDS}},
        {"name": "correct-again", "target": "explore", "options": {"seeds": SEEDS}},
        {"name": "lost-update", "target": "explore", "mutant": "lost-update", "options": {"seeds": SEEDS}},
        {"name": "no-ess", "target": "explore", "pathEnv": no_ess(scratch), "options": {"seeds": 1}},
        {"name": "answer-lost", "target": "explore", "mutant": "answer-lost", "options": {"seeds": SEEDS}},
        {"name": "stale-read", "target": "explore", "mutant": "stale-read", "options": {"seeds": SEEDS}},
        {"name": "close-unsupported", "target": "explore", "mutant": "close-unsupported", "options": {"seeds": 20}},
        {"name": "close-unsupported-allowed", "target": "explore", "mutant": "close-unsupported", "options": {"seeds": 20, "allowExcluded": true}},
        {"name": "clients-five", "target": "explore", "options": {"seeds": 1, "clients": 5}},
        {"name": "clients-one", "target": "explore", "options": {"seeds": 1, "clients": 1}},
        {"name": "seeds-negative", "target": "explore", "options": {"seeds": -1}},
        {"name": "seed-negative", "target": "explore", "options": {"seed": -2}},
        {"name": "calls-negative", "target": "explore", "options": {"seeds": 1, "calls": -1}},
    ])
}

fn billing_cases() -> Value {
    serde_json::json!([
        {"name": "billing", "target": "billing", "options": {"seeds": BILLING_SEEDS}},
        {"name": "billing-injected", "target": "billing", "options": {"seeds": BILLING_SEEDS, "inject": true}},
        {"name": "double-apply", "target": "billing", "mutant": "double-apply", "options": {"seeds": SEEDS}},
        {"name": "double-apply-injected", "target": "billing", "mutant": "double-apply", "options": {"seeds": SEEDS, "inject": true}},
    ])
}

fn retry_cases() -> Value {
    serde_json::json!([
        {"name": "retry", "target": "retry", "options": {"seeds": RETRY_SEEDS}},
        {"name": "retry-injected", "target": "retry", "options": {"seeds": RETRY_SEEDS, "inject": true}},
        {"name": "unretained", "target": "retry", "mutant": "unretained", "options": {"seeds": RETRY_SEEDS}},
        {"name": "unretained-injected", "target": "retry", "mutant": "unretained", "options": {"seeds": RETRY_SEEDS, "inject": true}},
    ])
}

/// Two entities, each created by its own command (`explore-pair/pair.yaml`).
fn pair_cases() -> Value {
    serde_json::json!([
        {"name": "pair", "target": "pair", "options": {"seeds": PAIR_SEEDS}},
        {"name": "box-lost", "target": "pair", "mutant": "box-lost", "options": {"seeds": PAIR_SEEDS}},
        {"name": "box-lost-phantom-crate", "target": "pair", "mutant": "box-lost-phantom-crate", "options": {"seeds": PAIR_SEEDS}},
    ])
}

/// Every lane pair — explore, billing, retry and pair — run once for every test below.
struct Lanes {
    explore: (Lane, Lane),
    billing: (Lane, Lane),
    retry: (Lane, Lane),
    pair: (Lane, Lane),
}

fn lanes() -> &'static Lanes {
    static LANES: OnceLock<Lanes> = OnceLock::new();
    LANES.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("a scratch directory");
        let cases = explore_cases(scratch.path());
        let explore = (
            typescript(&scratch.path().join("explore"), EXPLORE, &cases),
            go(&scratch.path().join("explore"), EXPLORE, &cases),
        );
        let cases = billing_cases();
        let billing = (
            typescript(&scratch.path().join("billing"), BILLING, &cases),
            go(&scratch.path().join("billing"), BILLING, &cases),
        );
        let cases = retry_cases();
        let retry = (
            typescript(&scratch.path().join("retry"), RETRY, &cases),
            go(&scratch.path().join("retry"), RETRY, &cases),
        );
        let cases = pair_cases();
        let pair = (
            typescript(&scratch.path().join("pair"), PAIR, &cases),
            go(&scratch.path().join("pair"), PAIR, &cases),
        );
        Lanes {
            explore,
            billing,
            retry,
            pair,
        }
    })
}

fn result<'a>(lane: &'a Lane, name: &str) -> &'a Value {
    match &lane.results[name] {
        Ok(value) => value,
        Err(refusal) => panic!("`{name}` was refused: {refusal}\n{}", lane.log),
    }
}

fn count(value: &Value, key: &str) -> u64 {
    value[key]
        .as_u64()
        .unwrap_or_else(|| panic!("`{key}` is a count: {value}"))
}

// ---- the correct target: 200 seeds, 0 violations, 0 unknown ------------------------------------

#[test]
fn the_correct_explore_target_gives_no_violation_and_no_unknown_over_200_seeds() {
    for lane in [&lanes().explore.0, &lanes().explore.1] {
        let correct = result(lane, "correct");
        println!(
            "correct: {} violation(s), {} unknown, {} linearizable of {SEEDS}",
            count(correct, "violations"),
            count(correct, "unknown"),
            count(correct, "linearizable")
        );
        assert_eq!(count(correct, "histories"), SEEDS, "{correct}");
        assert_eq!(count(correct, "violations"), 0, "{correct}");
        assert_eq!(count(correct, "unknown"), 0, "{correct}");
        assert_eq!(count(correct, "linearizable"), SEEDS, "{correct}");
        assert!(correct.get("failure").is_none(), "{correct}");
        assert_eq!(lane.histories["correct"].len() as u64, SEEDS);
    }
}

// ---- the lost-update mutant is caught by check-history -----------------------------------------

#[test]
fn the_lost_update_mutant_gives_a_violation_under_at_least_one_of_200_seeds() {
    for lane in [&lanes().explore.0, &lanes().explore.1] {
        let lost = result(lane, "lost-update");
        println!(
            "lost-update: {} violation(s), {} unknown, {} linearizable of {SEEDS}; first failing seed {}",
            count(lost, "violations"),
            count(lost, "unknown"),
            count(lost, "linearizable"),
            lost["failure"]["seed"]
        );
        assert_eq!(count(lost, "histories"), SEEDS, "{lost}");
        assert!(count(lost, "violations") >= 1, "{lost}");
        let failure = &lost["failure"];
        let seed = failure["seed"].as_u64().expect("the failing seed");
        assert_eq!(failure["verdict"], "Violation", "{lost}");
        assert_eq!(failure["history"], format!("history-{seed}.json"), "{lost}");
        let report = failure["report"].as_str().expect("the checker's report");
        println!("{report}");
        assert!(report.starts_with("verdict: Violation\n"), "{report}");
        assert!(report.contains("shrunk history:"), "{report}");
        assert_eq!(
            lost["verdicts"][usize::try_from(seed - 1).unwrap()],
            "Violation",
            "{lost}"
        );
    }
}

#[test]
fn the_lost_update_violations_are_checked_again_by_ess_and_are_violations() {
    let lane = &lanes().explore.1;
    let lost = result(lane, "lost-update");
    let seed = lost["failure"]["seed"].as_u64().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let file = scratch.path().join("history.json");
    std::fs::write(&file, &lane.histories["lost-update"][&seed]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "check-history", "--path", EXPLORE])
        .arg("--history")
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", printed(&output));
}

// ---- one seed, one history ---------------------------------------------------------------------

#[test]
fn the_same_seed_writes_the_same_history_bytes_on_two_runs_in_each_language() {
    for lane in [&lanes().explore.0, &lanes().explore.1] {
        let first = &lane.histories["correct"];
        let second = &lane.histories["correct-again"];
        assert_eq!(first.len() as u64, SEEDS);
        assert_eq!(
            first, second,
            "a seed wrote different bytes on a second run"
        );
        assert_eq!(
            result(lane, "correct"),
            result(lane, "correct-again"),
            "the same seeds gave another result"
        );
    }
}

#[test]
fn both_languages_write_the_same_explore_histories_and_verdicts() {
    let (typescript, go) = &lanes().explore;
    for name in ["correct", "lost-update"] {
        assert_eq!(
            typescript.histories[name], go.histories[name],
            "`{name}`: the TypeScript and Go histories differ"
        );
        assert_eq!(result(typescript, name), result(go, name), "`{name}`");
    }
}

#[test]
fn go_and_typescript_write_equal_bytes_from_the_same_seed_over_examples_billing() {
    let (typescript, go) = &lanes().billing;
    let ts = &typescript.histories["billing"];
    assert_eq!(ts.len() as u64, BILLING_SEEDS);
    assert_eq!(ts, &go.histories["billing"]);
    for lane in [typescript, go] {
        let billing = result(lane, "billing");
        assert_eq!(count(billing, "linearizable"), BILLING_SEEDS, "{billing}");
    }
    // Not a vacuous equality: the histories hold concurrent calls of more than one command, some
    // of them creations whose identity the target minted and some on those identities.
    let commands: std::collections::BTreeSet<String> = ts
        .values()
        .flat_map(|bytes| {
            let document: Value = serde_json::from_slice(bytes).unwrap();
            document["operations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|operation| operation["command"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        })
        .collect();
    for command in [
        "billing.invoice.CreateInvoice",
        "billing.invoice.IssueInvoice",
        "billing.invoice.PayInvoice",
        "billing.invoice.CancelInvoice",
    ] {
        assert!(commands.contains(command), "{command} in {commands:?}");
    }
    let overlapping = ts.values().any(|bytes| {
        let document: Value = serde_json::from_slice(bytes).unwrap();
        let operations = document["operations"].as_array().unwrap();
        operations.iter().any(|a| {
            operations.iter().any(|b| {
                a["client"] != b["client"]
                    && a["invoked_at"].as_u64() < b["returned_at"].as_u64()
                    && b["invoked_at"].as_u64() < a["returned_at"].as_u64()
            })
        })
    });
    assert!(overlapping, "no two clients' calls overlapped in any seed");
}

// ---- what the runner writes is the one spelling of an admitted history -------------------------

#[test]
fn every_written_history_is_admitted_and_is_the_readers_own_spelling() {
    let lanes = lanes();
    for (lane, name, spec) in [
        (&lanes.explore.0, "correct", EXPLORE),
        (&lanes.explore.1, "lost-update", EXPLORE),
        (&lanes.explore.0, "answer-lost", EXPLORE),
        (&lanes.explore.1, "close-unsupported", EXPLORE),
        (&lanes.explore.0, "stale-read", EXPLORE),
        (&lanes.explore.1, "stale-read", EXPLORE),
        (&lanes.billing.0, "billing", BILLING),
        (&lanes.billing.1, "double-apply-injected", BILLING),
        (&lanes.retry.0, "retry", RETRY),
        (&lanes.billing.1, "billing-injected", BILLING),
        (&lanes.retry.0, "retry-injected", RETRY),
        (&lanes.retry.1, "unretained-injected", RETRY),
    ] {
        let digest = spec_digest(spec);
        for (seed, bytes) in &lane.histories[name] {
            let history = ess_conformance::history::read(bytes, &digest)
                .unwrap_or_else(|refusal| panic!("`{name}` seed {seed} is refused: {refusal}"));
            assert_eq!(history.seed, *seed);
            assert!((2..=4).contains(&history.clients), "{name} seed {seed}");
            assert_eq!(
                &serde_json::to_vec(&history).unwrap(),
                bytes,
                "`{name}` seed {seed} is not written in the reader's own spelling"
            );
        }
    }
}

fn spec_digest(spec: &str) -> ess_conformance::history::SpecDigest {
    let scratch = tempfile::tempdir().unwrap();
    let package = emit(spec, "go", scratch.path());
    let suite: Value =
        serde_json::from_str(&std::fs::read_to_string(package.join("suite.json")).unwrap())
            .unwrap();
    ess_conformance::history::SpecDigest::new(suite["provenance"]["spec_digest"].as_str().unwrap())
        .unwrap()
}

// ---- the scheduler's draws are SplitMix64, as `src/record.rs` draws them -----------------------

fn splitmix64(seed: u64, count: usize) -> Vec<u64> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        })
        .collect()
}

#[test]
fn seed_one_schedules_with_the_same_splitmix64_outputs_in_rust_typescript_and_go() {
    let expected = splitmix64(1, 5)
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    for lane in [
        &lanes().explore.0,
        &lanes().explore.1,
        &lanes().billing.0,
        &lanes().billing.1,
    ] {
        assert_eq!(lane.splitmix64, expected);
    }
}

// ---- no `ess`, no concurrent mode --------------------------------------------------------------

#[test]
fn without_ess_on_path_the_concurrent_mode_fails_naming_ess() {
    for lane in [&lanes().explore.0, &lanes().explore.1] {
        let refusal = lane.results["no-ess"]
            .as_ref()
            .expect_err("the concurrent mode fails without `ess`");
        assert!(refusal.contains("`ess`"), "{refusal}");
        assert!(refusal.contains("PATH"), "{refusal}");
        assert!(
            lane.histories["no-ess"].is_empty(),
            "nothing ran: {refusal}"
        );
    }
    let (typescript, go) = &lanes().explore;
    assert_eq!(typescript.results["no-ess"], go.results["no-ess"]);
}

// ---- the emitted packages depend on the standard library only ----------------------------------

#[test]
fn the_emitted_go_module_names_no_module_outside_the_standard_library() {
    let scratch = tempfile::tempdir().unwrap();
    let package = emit(EXPLORE, "go", scratch.path());
    assert!(
        !scratch.path().join("go.mod").exists() && !package.join("go.mod").exists(),
        "the package names no module of its own to require anything from"
    );
    let mut files = 0;
    for entry in std::fs::read_dir(&package).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|it| it != "go") {
            continue;
        }
        files += 1;
        let source = std::fs::read_to_string(&path).unwrap();
        for import in go_imports(&source) {
            let first = import.split('/').next().unwrap_or_default();
            assert!(
                !first.contains('.'),
                "{} imports `{import}`, which is not in the Go standard library",
                path.display()
            );
        }
    }
    assert!(files >= 4, "the package's Go files were read");
    let explore = std::fs::read_to_string(package.join("explore.go")).unwrap();
    assert!(go_imports(&explore).contains(&"os/exec".to_owned()));
}

/// Every import path of one Go file, from its `import` declarations.
fn go_imports(source: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let mut block = false;
    for line in source.lines() {
        let line = line.trim();
        if block {
            if line == ")" {
                block = false;
            } else if let Some(path) = quoted(line) {
                imports.push(path);
            }
        } else if line == "import (" {
            block = true;
        } else if let Some(rest) = line.strip_prefix("import ") {
            if let Some(path) = quoted(rest) {
                imports.push(path);
            }
        }
    }
    imports
}

fn quoted(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let end = line[start + 1..].find('"')? + start + 1;
    Some(line[start + 1..end].to_owned())
}

#[test]
fn the_emitted_typescript_package_declares_no_dependency_outside_the_node_standard_library() {
    let scratch = tempfile::tempdir().unwrap();
    let package = emit(EXPLORE, "typescript", scratch.path());
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(package.join("package.json")).unwrap())
            .unwrap();
    for key in [
        "dependencies",
        "peerDependencies",
        "optionalDependencies",
        "bundleDependencies",
        "bundledDependencies",
    ] {
        assert!(manifest.get(key).is_none(), "package.json declares `{key}`");
    }
    // What building it takes: the compiler and the Node standard library's own declarations.
    let development: Vec<&str> = manifest["devDependencies"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(development, ["@types/node", "typescript"]);
    let mut files = 0;
    for entry in std::fs::read_dir(package.join("src")).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        files += 1;
        for line in source.lines() {
            let line = line.trim_start();
            if !(line.starts_with("import ") || line.starts_with("} from ")) {
                continue;
            }
            let specifier = line.rsplit_once(" from ").map_or(line, |(_, it)| it);
            let Some(start) = specifier.find('\'') else {
                continue;
            };
            let end = specifier[start + 1..].find('\'').unwrap() + start + 1;
            let module = &specifier[start + 1..end];
            assert!(
                module.starts_with("./") || module.starts_with("node:"),
                "{} imports `{module}`, which is neither the package's own nor Node's",
                path.display()
            );
        }
    }
    assert!(files >= 7, "the package's sources were read");
    let explore = std::fs::read_to_string(package.join("src/explore.ts")).unwrap();
    assert!(
        explore.contains("from 'node:child_process'"),
        "explore.ts runs `ess`"
    );
}

// ---- what a call that did not answer is written as ---------------------------------------------

/// Every operation of every history of `name`, as JSON.
fn operations(lane: &Lane, name: &str) -> Vec<Value> {
    lane.histories[name]
        .values()
        .flat_map(|bytes| {
            let document: Value = serde_json::from_slice(bytes).unwrap();
            document["operations"].as_array().unwrap().clone()
        })
        .collect()
}

#[test]
fn a_call_whose_answer_was_lost_is_written_indeterminate_counted_and_passes() {
    for lane in [&lanes().explore.0, &lanes().explore.1] {
        let lost = result(lane, "answer-lost");
        let written = operations(lane, "answer-lost")
            .iter()
            .filter(|operation| operation["completion"] == "Indeterminate")
            .count() as u64;
        println!(
            "answer-lost: {} indeterminate call(s), {} violation(s), {} unknown, {} linearizable of {SEEDS}",
            count(lost, "indeterminate"),
            count(lost, "violations"),
            count(lost, "unknown"),
            count(lost, "linearizable")
        );
        assert!(written > 0, "{lost}");
        assert_eq!(count(lost, "indeterminate"), written, "{lost}");
        // A call that took effect and never answered is not a fault: the checker may place it.
        assert_eq!(count(lost, "violations"), 0, "{lost}");
        assert_eq!(count(lost, "unknown"), 0, "{lost}");
        assert_eq!(lane.problems["answer-lost"], "none");
    }
    let (typescript, go) = &lanes().explore;
    assert_eq!(
        typescript.histories["answer-lost"],
        go.histories["answer-lost"]
    );
    assert_eq!(result(typescript, "answer-lost"), result(go, "answer-lost"));
}

// ---- a command the target does not expose ------------------------------------------------------

#[test]
fn an_unsupported_command_is_left_out_and_fails_the_check_unless_accepted() {
    let reason = "the target does not expose it: CloseTicket is not exposed";
    for lane in [&lanes().explore.0, &lanes().explore.1] {
        let refused = result(lane, "close-unsupported");
        assert_eq!(
            refused["excluded"],
            serde_json::json!([{"subject": "explore.desk.CloseTicket", "reason": reason}]),
            "{refused}"
        );
        assert_eq!(count(refused, "indeterminate"), 0, "{refused}");
        for name in ["close-unsupported", "close-unsupported-allowed"] {
            assert!(
                operations(lane, name)
                    .iter()
                    .all(|operation| operation["command"] != "explore.desk.CloseTicket"),
                "`{name}` wrote a call the target does not expose"
            );
            assert_eq!(count(result(lane, name), "violations"), 0);
        }
        let problem = &lane.problems["close-unsupported"];
        assert!(
            problem.starts_with(
                "explore: 1 command(s) the target does not expose were left out of concurrent exploration:"
            ) && problem.contains(&format!("explore.desk.CloseTicket: {reason}")),
            "{problem}"
        );
        assert_eq!(lane.problems["close-unsupported-allowed"], "none");
    }
    let (typescript, go) = &lanes().explore;
    assert_eq!(
        typescript.histories["close-unsupported"],
        go.histories["close-unsupported"]
    );
}

// ---- 2 to 4 clients ----------------------------------------------------------------------------

#[test]
fn a_client_count_outside_two_to_four_is_refused_with_one_message_in_both_languages() {
    let (typescript, go) = &lanes().explore;
    for (name, clients) in [("clients-five", 5), ("clients-one", 1)] {
        let refusal = typescript.results[name]
            .as_ref()
            .expect_err("the client count is refused");
        assert_eq!(
            refusal,
            &format!(
                "explore: `Clients` is {clients}; concurrent exploration runs 2 to 4 clients, or \
                 draws how many from each seed when it is 0"
            )
        );
        assert_eq!(typescript.results[name], go.results[name]);
        assert!(typescript.histories[name].is_empty() && go.histories[name].is_empty());
    }
}

// ---- every count below its meaning is refused, not a pass over nothing -------------------------

#[test]
fn a_negative_seed_count_seed_or_call_count_is_refused_with_one_message_in_both_languages() {
    let (typescript, go) = &lanes().explore;
    for (name, message) in [
        (
            "seeds-negative",
            "explore: `Seeds` is -1; concurrent exploration records at least one history, or 200 \
             when it is 0",
        ),
        (
            "seed-negative",
            "explore: `Seed` is -2; a replayed seed is at least 1, or none when it is 0",
        ),
        (
            "calls-negative",
            "explore: `Calls` is -1; each client makes at least one call, or 3 when it is 0",
        ),
    ] {
        assert_eq!(
            typescript.results[name].as_ref().err().map(String::as_str),
            Some(message)
        );
        assert_eq!(typescript.results[name], go.results[name], "{name}");
        assert!(typescript.histories[name].is_empty() && go.histories[name].is_empty());
    }
}

// ---- the faults the specification declares, injected (`story:declared-fault-injection`) ---------

/// `result[injected][kind][key]`, or 0.
fn injected(result: &Value, kind: &str, key: &str) -> u64 {
    result["injected"][kind][key].as_u64().unwrap_or(0)
}

/// The sum of `result[injected][kind]`.
fn injected_total(result: &Value, kind: &str) -> u64 {
    result["injected"][kind]
        .as_object()
        .unwrap_or_else(|| panic!("`injected.{kind}` is a map: {result}"))
        .values()
        .map(|count| count.as_u64().expect("a count"))
        .sum()
}

#[test]
fn injection_over_examples_billing_writes_equal_bytes_in_go_and_typescript_and_stays_linearizable()
{
    let (typescript, go) = &lanes().billing;
    let ts = &typescript.histories["billing-injected"];
    assert_eq!(ts.len() as u64, BILLING_SEEDS);
    assert_eq!(ts, &go.histories["billing-injected"]);
    assert_eq!(
        result(typescript, "billing-injected"),
        result(go, "billing-injected")
    );
    assert_ne!(
        ts, &typescript.histories["billing"],
        "injection changed no history, so it injected nothing"
    );
    for lane in [typescript, go] {
        let billing = result(lane, "billing-injected");
        println!("billing-injected: {}", billing["injected"]);
        assert_eq!(count(billing, "linearizable"), BILLING_SEEDS, "{billing}");
        // What billing declares: `notify-on-invoice-created` is `at_least_once`, and
        // `SendEmail/failed` is `external:`. It declares no `replays:`, so nothing is retried.
        assert!(
            injected(billing, "redeliveries", "notify-on-invoice-created") > 0,
            "{billing}"
        );
        assert_eq!(injected_total(billing, "retries"), 0, "{billing}");
        assert_eq!(injected_total(billing, "refused"), 0, "{billing}");
        let lost = injected_total(billing, "delayed") + injected_total(billing, "unanswered");
        assert!(lost > 0, "{billing}");
        let written = operations(lane, "billing-injected")
            .iter()
            .filter(|operation| operation["completion"] == "Indeterminate")
            .count() as u64;
        assert_eq!(count(billing, "indeterminate"), written, "{billing}");
        assert_eq!(
            written, lost,
            "every injected delay or loss, and nothing else"
        );
        // Without injection nothing is counted.
        let plain = result(lane, "billing");
        assert_eq!(
            plain["injected"],
            serde_json::json!({"redeliveries": {}, "refused": {}, "retries": {}, "delayed": {}, "unanswered": {}, "reached": {}})
        );
    }
}

#[test]
fn a_client_retry_of_a_command_declaring_replays_is_written_equally_and_catches_an_unretained_target(
) {
    let (typescript, go) = &lanes().retry;
    for name in [
        "retry",
        "retry-injected",
        "unretained",
        "unretained-injected",
    ] {
        assert_eq!(
            typescript.histories[name], go.histories[name],
            "`{name}`: the TypeScript and Go histories differ"
        );
        assert_eq!(result(typescript, name), result(go, name), "`{name}`");
    }
    for lane in [typescript, go] {
        let correct = result(lane, "retry-injected");
        println!("retry-injected: {}", correct["injected"]);
        assert_eq!(count(correct, "linearizable"), RETRY_SEEDS, "{correct}");
        assert!(
            injected(correct, "retries", "retry.core.Seed") > 0,
            "{correct}"
        );
        assert!(
            injected(correct, "reached", "retry.core.Seed/replayed") > 0,
            "{correct}"
        );
        assert!(operations(lane, "retry-injected")
            .iter()
            .any(|operation| operation.get("retry_of").is_some()));
        // The target that retains a request only once it answered: no violation without a retry,
        // and at least one with.
        let without = result(lane, "unretained");
        assert_eq!(count(without, "violations"), 0, "{without}");
        assert!(operations(lane, "unretained")
            .iter()
            .all(|operation| operation.get("retry_of").is_none()));
        let with = result(lane, "unretained-injected");
        println!(
            "unretained-injected: {} violation(s) of {RETRY_SEEDS}; first failing seed {}",
            count(with, "violations"),
            with["failure"]["seed"]
        );
        assert!(count(with, "violations") >= 1, "{with}");
        // The violation is the request applied twice: its retry answered the origin branch.
        let report = with["failure"]["report"]
            .as_str()
            .expect("the checker's report");
        println!("{report}");
        assert!(report.contains("-> seeded (a retry of "), "{report}");
        assert_eq!(
            result(lane, "retry")["verdicts"].as_array().unwrap().len() as u64,
            RETRY_SEEDS
        );
    }
}

// ---- view reads (`story:explorers-record-view-reads`) -------------------------------------------

/// The read operations of `name`: those whose `command` is one of `views`.
fn reads(lane: &Lane, name: &str, views: &[&str]) -> Vec<Value> {
    operations(lane, name)
        .into_iter()
        .filter(|operation| views.contains(&operation["command"].as_str().unwrap()))
        .collect()
}

const EXPLORE_VIEWS: [&str; 2] = ["explore.desk.OpenTickets", "explore.desk.TicketsByItems"];
const BILLING_VIEWS: [&str; 2] = [
    "billing.invoice.InvoiceById",
    "billing.invoice.OutstandingInvoices",
];

#[test]
fn the_explorers_read_every_view_and_write_rows_only_on_an_answered_read() {
    let lanes = lanes();
    for (lane, name, views) in [
        (&lanes.explore.0, "correct", &EXPLORE_VIEWS[..]),
        (&lanes.explore.1, "correct", &EXPLORE_VIEWS[..]),
        (&lanes.billing.0, "billing", &BILLING_VIEWS[..]),
        (&lanes.billing.1, "billing", &BILLING_VIEWS[..]),
        (&lanes.retry.0, "retry", &["retry.core.Records"][..]),
        (&lanes.retry.1, "retry", &["retry.core.Records"][..]),
    ] {
        let read = reads(lane, name, views);
        for view in views {
            assert!(
                read.iter().any(|operation| operation["command"] == *view
                    && operation["rows"]
                        .as_array()
                        .is_some_and(|rows| !rows.is_empty())),
                "`{name}` never read a row of `{view}`"
            );
        }
        for operation in &read {
            assert_eq!(operation["subject_key"], "", "{operation}");
            assert_eq!(operation["completion"], "Returned", "{operation}");
            assert_eq!(operation["outcome"], "read", "{operation}");
            assert!(operation["rows"].is_array(), "{operation}");
        }
        assert!(
            operations(lane, name)
                .iter()
                .filter(|operation| operation.get("rows").is_some())
                .all(|operation| views.contains(&operation["command"].as_str().unwrap())),
            "`{name}` wrote rows on a command"
        );
    }
}

#[test]
fn a_go_history_and_a_typescript_history_from_one_seed_are_equal_bytes_reads_included() {
    let lanes = lanes();
    for (pair, name, views) in [
        (&lanes.explore, "correct", &EXPLORE_VIEWS[..]),
        (&lanes.explore, "stale-read", &EXPLORE_VIEWS[..]),
        (&lanes.billing, "double-apply", &BILLING_VIEWS[..]),
        (&lanes.billing, "double-apply-injected", &BILLING_VIEWS[..]),
        (&lanes.retry, "retry-injected", &["retry.core.Records"][..]),
    ] {
        let (typescript, go) = pair;
        assert!(
            !reads(typescript, name, views).is_empty(),
            "`{name}` recorded no read, so the equality says nothing about reads"
        );
        assert_eq!(
            typescript.histories[name], go.histories[name],
            "`{name}`: the TypeScript and Go histories differ"
        );
        assert_eq!(result(typescript, name), result(go, name), "`{name}`");
    }
}

#[test]
fn the_stale_read_mutant_gives_a_read_violation_under_at_least_one_of_200_seeds() {
    for lane in [&lanes().explore.0, &lanes().explore.1] {
        let stale = result(lane, "stale-read");
        println!(
            "stale-read: {} violation(s), {} unknown, {} linearizable of {SEEDS}; first failing seed {}",
            count(stale, "violations"),
            count(stale, "unknown"),
            count(stale, "linearizable"),
            stale["failure"]["seed"]
        );
        assert_eq!(count(stale, "histories"), SEEDS, "{stale}");
        assert!(count(stale, "violations") >= 1, "{stale}");
        assert_eq!(stale["failure"]["verdict"], "Violation", "{stale}");
        let report = stale["failure"]["report"]
            .as_str()
            .expect("the checker's report");
        println!("{report}");
        assert!(
            report.contains("read violation: client ")
                && report.contains("of `explore.desk.OpenTickets`, declared read_your_writes"),
            "{report}"
        );
        // The same seeds against the unmutated target.
        assert_eq!(count(result(lane, "correct"), "violations"), 0);
    }
}

#[test]
fn double_apply_on_redelivery_is_caught_through_a_view_read_with_injection_and_missed_without() {
    let (typescript, go) = &lanes().billing;
    for lane in [typescript, go] {
        let without = result(lane, "double-apply");
        assert_eq!(count(without, "histories"), SEEDS, "{without}");
        assert_eq!(count(without, "violations"), 0, "{without}");
        assert_eq!(count(without, "unknown"), 0, "{without}");
        let with = result(lane, "double-apply-injected");
        println!(
            "double-apply-injected: {} violation(s) of {SEEDS}; first failing seed {}; injected {}",
            count(with, "violations"),
            with["failure"]["seed"],
            with["injected"]
        );
        assert!(count(with, "violations") >= 1, "{with}");
        assert!(
            injected(with, "redeliveries", "notify-on-invoice-created") > 0,
            "{with}"
        );
        let report = with["failure"]["report"]
            .as_str()
            .expect("the checker's report");
        println!("{report}");
        assert!(
            report.contains("read violation: client ") && report.contains("of `billing.invoice."),
            "{report}"
        );
    }
}

/// A creation that never answered may account for one row of its own entity nobody names, never
/// for a row of another entity's view. `explore-pair/pair.yaml` has two creatable entities: under
/// `box-lost` every `OpenBox` opens its box and loses its answer, which is no fault; under
/// `box-lost-phantom-crate` it also opens a crate nobody asked for, which only a read of `Crates`
/// shows, and the lost box creations must not hide it.
#[test]
fn a_lost_creation_of_one_entity_never_withholds_a_row_of_another_entitys_view() {
    let (typescript, go) = &lanes().pair;
    for name in ["pair", "box-lost", "box-lost-phantom-crate"] {
        assert_eq!(
            typescript.histories[name], go.histories[name],
            "`{name}`: the TypeScript and Go histories differ"
        );
        assert_eq!(result(typescript, name), result(go, name), "`{name}`");
    }
    for lane in [typescript, go] {
        for name in ["pair", "box-lost"] {
            let clean = result(lane, name);
            assert_eq!(count(clean, "histories"), PAIR_SEEDS, "{clean}");
            assert_eq!(count(clean, "violations"), 0, "{clean}");
            assert_eq!(count(clean, "unknown"), 0, "{clean}");
        }
        let lost = result(lane, "box-lost");
        assert!(count(lost, "indeterminate") > 0, "{lost}");
        // Not vacuous: boxes whose creation lost its answer were read, and crates were read.
        assert!(
            reads(lane, "box-lost", &["pair.core.Boxes"])
                .iter()
                .any(|read| read["rows"].is_array())
                && reads(lane, "box-lost", &["pair.core.Crates"])
                    .iter()
                    .any(|read| read["rows"].as_array().is_some_and(|rows| !rows.is_empty())),
            "`box-lost` read no box and no crate"
        );
        let phantom = result(lane, "box-lost-phantom-crate");
        println!(
            "box-lost-phantom-crate: {} violation(s) of {PAIR_SEEDS}; first failing seed {}",
            count(phantom, "violations"),
            phantom["failure"]["seed"]
        );
        assert!(count(phantom, "violations") >= 1, "{phantom}");
        let report = phantom["failure"]["report"]
            .as_str()
            .expect("the checker's report");
        println!("{report}");
        assert!(
            report.contains("of `pair.core.Crates`, declared read_your_writes: future-read"),
            "{report}"
        );
    }
}

// ---- concurrent exploration checks what the sequential walk checks (beyond10x/ess#576) ----------

/// `explore-external.yaml` with one more command, `Note`, whose guard reads an `Optional` input: a
/// draw that leaves `memo` out leaves the guard unknown, and the walk excludes `Note` there and
/// then. With the `cannot-arrange` double, the first external branch drawn is unarrangeable too.
/// Both are exclusions an earlier seed of a run makes before a later seed is drawn.
fn noted_external_spec(scratch: &Path) -> PathBuf {
    let source = std::fs::read_to_string(fixture("explore-external.yaml"))
        .expect("the external fixture reads")
        .replace(
            "      - exploreext.pay.Capture\n",
            "      - exploreext.pay.Capture\n      - exploreext.pay.Note\n",
        )
        .replace(
            "\nevents:\n",
            "
  - name: exploreext.pay.Note
    input:
      - name: memo
        type: Optional<Integer>
    outcomes:
      - name: noted
        when: memo >= 1
        emits:
          - exploreext.pay.Noted
        payload:
          exploreext.pay.Noted:
            memo: input.memo
      - name: ignored
        error: exploreext.pay.InvalidAmount

events:
  - name: exploreext.pay.Noted
    fields:
      - name: memo
        type: Optional<Integer>
",
        );
    assert!(
        source.contains("exploreext.pay.Note\n    input"),
        "the command was added"
    );
    let path = scratch.join("noted-external.yaml");
    std::fs::write(&path, source).unwrap();
    path
}

/// The Go half of the walk-parity lane: `driver` as the test `run` of the package emitted for
/// `spec`, with each of `copies` (a fixture, and its name in the package), under `env` and
/// `ESS_WALK_SPEC`. Returns the log.
fn walk_go(
    scratch: &Path,
    spec: &Path,
    copies: &[(&str, &str)],
    driver: &str,
    run: &str,
    env: &[(&str, String)],
) -> String {
    let module = scratch.join("go");
    let package = emit(spec.to_str().unwrap(), "go", &module);
    std::fs::write(
        module.join("go.mod"),
        "module example.invalid/walk\n\ngo 1.24\n",
    )
    .unwrap();
    for (from, to) in copies {
        std::fs::copy(fixture(from), package.join(to)).expect("the fixture copies");
    }
    std::fs::write(package.join("walk_parity_driver_test.go"), driver).unwrap();
    let output = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            run,
            "-count=1",
            "-v",
            "-timeout",
            "30m",
        ])
        .envs(env.iter().map(|(key, value)| (*key, value)))
        .env("ESS_WALK_SPEC", spec)
        .env("PATH", path_with_ess())
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH: the lane does not skip");
    let log = printed(&output);
    assert!(output.status.success(), "{log}");
    assert!(log.contains(&format!("--- PASS: {run}")), "{log}");
    log
}

/// The TypeScript half of the walk-parity lane, as [`walk_go`]: `driver` run with `node --test`.
fn walk_typescript(
    scratch: &Path,
    spec: &Path,
    copies: &[&str],
    driver: &str,
    env: &[(&str, String)],
) -> String {
    let package = emit(
        spec.to_str().unwrap(),
        "typescript",
        &scratch.join("typescript"),
    );
    for name in copies {
        std::fs::copy(fixture(name), package.join(name)).expect("the fixture copies");
    }
    std::fs::write(package.join("walk-parity-driver.mjs"), driver).unwrap();
    std::fs::write(
        package.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&package)
        .output()
        .expect("`tsc` is on PATH: the lane does not skip");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let output = Command::new("node")
        .args(["--test", "walk-parity-driver.mjs"])
        .envs(env.iter().map(|(key, value)| (*key, value)))
        .env("ESS_WALK_SPEC", spec)
        .env("PATH", path_with_ess())
        .current_dir(&package)
        .output()
        .expect("`node` is on PATH: the lane does not skip");
    let log = printed(&output);
    assert!(output.status.success(), "{log}");
    assert!(log.lines().any(|line| line.starts_with("ok 1 - ")), "{log}");
    log
}

/// What a walk-parity driver wrote to `out`, as JSON.
fn walk_json(out: &Path, log: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(out).unwrap_or_else(|_| panic!("{log}")))
        .expect("the driver wrote JSON")
}

/// Every command a seed sends, by scenario, in a run of six seeds and in six runs of one seed each.
const SEED_REPLAY_GO: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

type seedReplayLog struct {
	Target
	scenario string
	log      map[string][]string
}

func (t *seedReplayLog) BeginScenario(s ScenarioContext) error {
	t.scenario = s.Scenario
	return t.Target.BeginScenario(s)
}

func (t *seedReplayLog) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	input, _ := json.Marshal(r.Input)
	t.log[t.scenario] = append(t.log[t.scenario], r.Command+" "+string(input))
	if r.Command == "exploreext.pay.Note" {
		if memo, ok := r.Input["memo"].(float64); ok && memo >= 1 {
			return CommandResult{Outcome: "noted", DirectEvents: []ObservedEvent{{Event: "exploreext.pay.Noted", Payload: map[string]Node{"memo": memo}}}}, nil
		}
		return CommandResult{Outcome: "ignored", Error: "exploreext.pay.InvalidAmount", DirectEvents: []ObservedEvent{}}, nil
	}
	return t.Target.ExecuteCommand(r)
}

func TestSeedReplay(t *testing.T) {
	logged := func(log map[string][]string) func() Target {
		return func() Target { return &seedReplayLog{Target: newExploreExtTarget("cannot-arrange"), log: log} }
	}
	run := map[string][]string{}
	result, err := Explore(logged(run), ExploreOptions{Seeds: 6, Steps: 30})
	if err != nil {
		t.Fatal(err)
	}
	alone := map[string][]string{}
	for seed := 1; seed <= 6; seed++ {
		if _, err := Explore(logged(alone), ExploreOptions{Seed: seed, Steps: 30}); err != nil {
			t.Fatal(err)
		}
	}
	bytes, err := json.Marshal(map[string]any{"run": run, "alone": alone, "result": result})
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("ESS_WALK_OUT"), bytes, 0o644); err != nil {
		t.Fatal(err)
	}
}
"#;

const SEED_REPLAY_TS: &str = r#"import { writeFileSync } from 'node:fs';
import test from 'node:test';

import { explore, goMarshal } from './dist/index.js';
import { newTarget } from './explore-external-target.mjs';

const logged = (log) => () => {
  const inner = newTarget('cannot-arrange');
  let scenario = '';
  return {
    ...inner,
    beginScenario(context) {
      scenario = context.scenario;
      return inner.beginScenario(context);
    },
    executeCommand(request) {
      (log[scenario] ??= []).push(`${request.command} ${JSON.stringify(request.input)}`);
      if (request.command === 'exploreext.pay.Note') {
        const memo = request.input.memo;
        if (typeof memo === 'number' && memo >= 1) {
          return { outcome: 'noted', directEvents: [{ event: 'exploreext.pay.Noted', payload: { memo } }] };
        }
        return { outcome: 'ignored', error: 'exploreext.pay.InvalidAmount', directEvents: [] };
      }
      return inner.executeCommand(request);
    },
  };
};

await test('seed replay', async () => {
  const run = {};
  const result = await explore(logged(run), { seeds: 6, steps: 30 });
  const alone = {};
  for (let seed = 1; seed <= 6; seed += 1) await explore(logged(alone), { seed, steps: 30 });
  writeFileSync(
    process.env.ESS_WALK_OUT,
    `{"run":${JSON.stringify(run)},"alone":${JSON.stringify(alone)},"result":${goMarshal(result)}}`,
  );
});
"#;

/// beyond10x/ess#576, finding 3: the sequence a seed draws inside a run is the sequence it draws
/// alone, after an earlier seed excluded a command (`Note`, its guard unknown over a draw) and an
/// external branch the target cannot arrange.
#[test]
fn a_seed_drawn_inside_a_multi_seed_run_replays_alone_the_same_sequence() {
    let scratch = tempfile::tempdir().unwrap();
    let spec = noted_external_spec(scratch.path());
    let out = scratch.path().join("seed-go.json");
    let log = walk_go(
        &scratch.path().join("seed-go"),
        &spec,
        &[(
            "explore_external_target.go",
            "explore_external_target_test.go",
        )],
        SEED_REPLAY_GO,
        "TestSeedReplay",
        &[("ESS_WALK_OUT", out.display().to_string())],
    );
    let go = walk_json(&out, &log);
    let out = scratch.path().join("seed-typescript.json");
    let log = walk_typescript(
        &scratch.path().join("seed-typescript"),
        &spec,
        &["explore-external-target.mjs"],
        SEED_REPLAY_TS,
        &[("ESS_WALK_OUT", out.display().to_string())],
    );
    let typescript = walk_json(&out, &log);
    for (language, lane) in [("go", &go), ("typescript", &typescript)] {
        let result = &lane["result"];
        // Not vacuous: an earlier seed excluded a command and found a branch unarrangeable.
        assert!(
            result["excluded"]
                .as_array()
                .unwrap()
                .iter()
                .any(|exclusion| exclusion["subject"] == "exploreext.pay.Note"),
            "{language}: no seed excluded `Note`: {result}"
        );
        assert!(
            result["external"]
                .as_array()
                .unwrap()
                .iter()
                .any(|branch| branch["reach"] == "unarrangeable"),
            "{language}: no branch was unarrangeable: {result}"
        );
        assert_eq!(result["sequences"], 6, "{language}: {result}");
        for seed in 1..=6 {
            let scenario = format!("explore/seed-{seed}");
            let inside = &lane["run"][&scenario];
            assert!(
                inside.as_array().is_some_and(|sent| !sent.is_empty()),
                "{language} {scenario}"
            );
            assert_eq!(
                inside, &lane["alone"][&scenario],
                "{language}: seed {seed} sent another sequence alone than inside the run"
            );
        }
    }
    assert_eq!(
        go["run"], typescript["run"],
        "Go and TypeScript drew different sequences"
    );
}

/// Runs every concurrent case of `cases` against the package emitted for `spec`, in Go and in
/// TypeScript, through [`WALK_CONCURRENT_GO`] and [`WALK_CONCURRENT_TS`].
fn walk_concurrent(scratch: &Path, spec: &Path, cases: &Value) -> (Lane, Lane) {
    let go_out = scratch.join("out-go");
    let typescript_out = scratch.join("out-typescript");
    for out in [&go_out, &typescript_out] {
        std::fs::create_dir_all(out).unwrap();
    }
    let log = walk_go(
        &scratch.join("go-lane"),
        spec,
        &[(
            "explore_external_target.go",
            "explore_external_target_test.go",
        )],
        WALK_CONCURRENT_GO,
        "TestWalkConcurrent",
        &[
            ("ESS_WALK_CASES", cases.to_string()),
            ("ESS_WALK_DIR", go_out.display().to_string()),
        ],
    );
    let go = read_lane(&go_out, cases, log);
    let log = walk_typescript(
        &scratch.join("typescript-lane"),
        spec,
        &["explore-external-target.mjs"],
        WALK_CONCURRENT_TS,
        &[
            ("ESS_WALK_CASES", cases.to_string()),
            ("ESS_WALK_DIR", typescript_out.display().to_string()),
        ],
    );
    let typescript = read_lane(&typescript_out, cases, log);
    (typescript, go)
}

/// Runs `ExploreConcurrent` once per case in `ESS_WALK_CASES` against the external double or the
/// entity-less calculator, and writes what [`read_lane`] reads into `ESS_WALK_DIR`.
const WALK_CONCURRENT_GO: &str = r#"package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

type walkCase struct {
	Name    string            `json:"name"`
	Target  string            `json:"target"`
	Mutant  string            `json:"mutant"`
	Options ConcurrentOptions `json:"options"`
}

// walkCalcTarget answers `calc.core.Check`: `big` from 10 up, `small` below. `wrong-outcome`
// answers `small` from 10 up, a declared branch the input does not take; `wrong-events` answers
// `big` from 10 up and publishes `Small`, an event the branch does not declare.
type walkCalcTarget struct{ mutant string }

func (t *walkCalcTarget) Identity() (Identity, error) {
	return Identity{Name: "walk-calc", Version: "1"}, nil
}
func (t *walkCalcTarget) BeginScenario(ScenarioContext) error { return nil }
func (t *walkCalcTarget) EndScenario(ScenarioContext) error   { return nil }
func (t *walkCalcTarget) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	n, _ := r.Input["n"].(float64)
	outcome, event := "small", "calc.core.Small"
	if n >= 10 {
		outcome, event = "big", "calc.core.Big"
		if t.mutant == "wrong-outcome" {
			outcome, event = "small", "calc.core.Small"
		}
		if t.mutant == "wrong-events" {
			event = "calc.core.Small"
		}
	}
	return CommandResult{Outcome: outcome, DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{"n": r.Input["n"]}}}}, nil
}
func (t *walkCalcTarget) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreExtUnsupported{"no view"}
}
func (t *walkCalcTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}
func (t *walkCalcTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreExtUnsupported{"no external outcome"}
}
func (t *walkCalcTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExtUnsupported{"no redelivery"}
}
func (t *walkCalcTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExtUnsupported{"no bindings"}
}

func TestWalkConcurrent(t *testing.T) {
	out := os.Getenv("ESS_WALK_DIR")
	draws := NewSplitMix64(1)
	outputs := []string{}
	for index := 0; index < 5; index++ {
		outputs = append(outputs, fmt.Sprint(draws.Next()))
	}
	if err := os.WriteFile(filepath.Join(out, "splitmix64"), []byte(strings.Join(outputs, " ")+"\n"), 0o644); err != nil {
		t.Fatal(err)
	}
	var cases []walkCase
	if err := json.Unmarshal([]byte(os.Getenv("ESS_WALK_CASES")), &cases); err != nil {
		t.Fatal(err)
	}
	for _, one := range cases {
		write := func(suffix, text string) {
			if err := os.WriteFile(filepath.Join(out, one.Name+suffix), []byte(text+"\n"), 0o644); err != nil {
				t.Fatal(err)
			}
		}
		options := one.Options
		options.Path = os.Getenv("ESS_WALK_SPEC")
		options.Out = filepath.Join(out, one.Name)
		mutant := one.Mutant
		newTarget := func() Target { return newExploreExtTarget(mutant) }
		if one.Target == "calc" {
			newTarget = func() Target { return &walkCalcTarget{mutant: mutant} }
		}
		result, err := ExploreConcurrent(newTarget, options)
		if err != nil {
			write(".refused", err.Error())
			continue
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
	}
}
"#;

const WALK_CONCURRENT_TS: &str = r"import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { concurrentProblem, exploreConcurrent, goMarshal, SplitMix64, unsupported } from './dist/index.js';
import { newTarget } from './explore-external-target.mjs';

// The calculator `walk_concurrent_go` describes, in TypeScript.
const calcTarget = (mutant) => ({
  identity: () => ({ name: 'walk-calc', version: '1' }),
  beginScenario: () => {},
  endScenario: () => {},
  executeCommand({ input }) {
    let outcome = 'small';
    let event = 'calc.core.Small';
    if (input.n >= 10) {
      outcome = 'big';
      event = 'calc.core.Big';
      if (mutant === 'wrong-outcome') {
        outcome = 'small';
        event = 'calc.core.Small';
      }
      if (mutant === 'wrong-events') event = 'calc.core.Small';
    }
    return { outcome, directEvents: [{ event, payload: { n: input.n } }] };
  },
  queryView: () => {
    throw unsupported('no view');
  },
  observeEvents: () => [],
  configureExternalOutcome: () => {
    throw unsupported('no external outcome');
  },
  redeliverEvent: () => {
    throw unsupported('no redelivery');
  },
  observeInvocations: () => {
    throw unsupported('no bindings');
  },
});

const out = process.env.ESS_WALK_DIR ?? '.';
const draws = new SplitMix64(1);
const outputs = [];
for (let index = 0; index < 5; index += 1) outputs.push(draws.next().toString());
writeFileSync(join(out, 'splitmix64'), `${outputs.join(' ')}\n`);

await test('walk concurrent', async () => {
  for (const one of JSON.parse(process.env.ESS_WALK_CASES ?? '[]')) {
    const write = (suffix, text) => writeFileSync(join(out, `${one.name}${suffix}`), `${text}\n`);
    const make =
      one.target === 'calc' ? () => calcTarget(one.mutant ?? '') : () => newTarget(one.mutant ?? '');
    try {
      const result = await exploreConcurrent(make, {
        ...(one.options ?? {}),
        path: process.env.ESS_WALK_SPEC,
        out: join(out, one.name),
      });
      write('.json', goMarshal(result));
      write('.problem', concurrentProblem(result) ?? 'none');
    } catch (error) {
      write('.refused', error.message);
    }
  }
});
";

fn external_cases() -> Value {
    serde_json::json!([
        {"name": "external", "target": "external", "options": {"seeds": SEEDS}},
        {"name": "fail-keeps-pending", "target": "external", "mutant": "fail-keeps-pending", "options": {"seeds": SEEDS}},
        {"name": "cannot-arrange", "target": "external", "mutant": "cannot-arrange", "options": {"seeds": 20}},
        {"name": "cannot-arrange-allowed", "target": "external", "mutant": "cannot-arrange", "options": {"seeds": 20, "allowExcluded": true}},
    ])
}

/// `explore-external.yaml` with `Authorize/declined` declared before `authorized`.
///
/// As the fixture declares them, `authorized` (`amount >= 1`) is the accepting branch declared
/// first, so the model never admits `declined` (`amount >= 1000`, `external:`) and synthesis
/// refuses it; declared first, it is a branch an input from 1000 up may take, which is what a
/// concurrent history must be able to show.
fn reachable_external_spec(scratch: &Path) -> PathBuf {
    let declined = "      - name: declined
        when: amount >= 1000
        external: the card network declines
        error: exploreext.pay.Declined
";
    let source = std::fs::read_to_string(fixture("explore-external.yaml"))
        .expect("the external fixture reads");
    assert!(source.contains(declined), "the fixture declares `declined`");
    let source = source.replace(declined, "").replacen(
        "      - name: authorized\n",
        &format!("{declined}      - name: authorized\n"),
        1,
    );
    let path = scratch.join("reachable-external.yaml");
    std::fs::write(&path, source).unwrap();
    path
}

/// The external lane pair: [`reachable_external_spec`] explored concurrently, once for every test
/// below.
fn external_lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let scratch = tempfile::tempdir().expect("a scratch directory");
        let spec = reachable_external_spec(scratch.path());
        walk_concurrent(scratch.path(), &spec, &external_cases())
    })
}

/// beyond10x/ess#576, finding 2: concurrent exploration arranges an `external:` cause through
/// `ConfigureExternalOutcome` wherever the sequential walk would, so a defect that only an external
/// answer reaches — a capture failure that leaves its payment pending — is caught.
#[test]
fn concurrent_exploration_arranges_external_branches_and_catches_a_backend_dependent_defect() {
    let (typescript, go) = external_lanes();
    for name in ["external", "fail-keeps-pending", "cannot-arrange"] {
        assert_eq!(
            typescript.histories[name], go.histories[name],
            "`{name}`: the TypeScript and Go histories differ"
        );
        assert_eq!(result(typescript, name), result(go, name), "`{name}`");
    }
    for lane in [typescript, go] {
        let planted = result(lane, "fail-keeps-pending");
        println!(
            "fail-keeps-pending: {} violation(s) of {SEEDS}; first failing seed {}",
            count(planted, "violations"),
            planted["failure"]["seed"]
        );
        assert!(count(planted, "violations") >= 1, "{planted}");
        let correct = result(lane, "external");
        println!("external: arranged {}", correct["arranged"]);
        assert_eq!(count(correct, "linearizable"), SEEDS, "{correct}");
        for branch in [
            "exploreext.pay.Authorize/declined",
            "exploreext.pay.Capture/failed",
        ] {
            assert!(
                correct["arranged"][branch]
                    .as_u64()
                    .is_some_and(|count| count > 0),
                "`{branch}` was never arranged: {correct}"
            );
        }
        assert!(correct.get("unarrangeable").is_none(), "{correct}");
        assert_eq!(lane.problems["external"], "none");
        let refused = result(lane, "cannot-arrange");
        assert_eq!(count(refused, "violations"), 0, "{refused}");
        let reason = "the target cannot arrange it: this double arranges no external outcome";
        let branches = refused["unarrangeable"]
            .as_array()
            .expect("unarrangeable branches");
        let subjects: std::collections::BTreeSet<&str> = branches
            .iter()
            .map(|branch| branch["subject"].as_str().unwrap())
            .collect();
        assert_eq!(
            subjects,
            [
                "exploreext.pay.Authorize/declined",
                "exploreext.pay.Capture/failed"
            ]
            .into(),
            "{refused}"
        );
        assert_eq!(branches.len(), 2, "{refused}");
        assert!(
            branches.iter().all(|branch| branch["reason"] == reason),
            "{refused}"
        );
        assert!(
            lane.problems["cannot-arrange"].contains(
                "external outcome(s) the target could not arrange were never arranged in concurrent exploration"
            ),
            "{}",
            lane.problems["cannot-arrange"]
        );
        assert_eq!(lane.problems["cannot-arrange-allowed"], "none");
    }
}

// ---- the input and events a history records (`ess-history/3`, beyond10x/ess#576) ---------------

/// The digest the committed `history2/` documents carry.
fn history2_digest() -> ess_conformance::history::SpecDigest {
    ess_conformance::history::SpecDigest::new("ab".repeat(32)).unwrap()
}

fn history2(name: &str) -> Vec<u8> {
    std::fs::read(fixture("history2").join(name)).expect("the history2 fixture is committed")
}

/// The old-reader compatibility case: an `ess-history/1` or `ess-history/2` document a writer wrote
/// before `ess-history/3` existed reads, selects its own format again and writes back byte for
/// byte, and the retained format 1 reader refuses every later format by its `format` alone.
#[test]
fn an_older_history_reads_and_writes_back_unchanged_and_the_format1_reader_refuses_later_formats() {
    use ess_conformance::history::{read, read_format1, HistoryFormat, HistoryRefusal};
    let digest = history2_digest();
    for (name, format) in [
        ("written-format1.json", HistoryFormat::EssHistory1),
        ("written.json", HistoryFormat::EssHistory2),
    ] {
        let bytes = history2(name);
        let bytes = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
        let history = read(bytes, &digest).unwrap_or_else(|refusal| panic!("{name}: {refusal}"));
        assert_eq!(history.format, format, "{name}");
        assert_eq!(
            HistoryFormat::for_operations(&history.operations),
            format,
            "{name}"
        );
        assert_eq!(serde_json::to_vec(&history).unwrap(), bytes, "{name}");
    }
    let format1 = history2("written-format1.json");
    let text = String::from_utf8(format1).unwrap();
    let text = text.trim_end();
    assert!(read_format1(text.as_bytes(), &history2_digest()).is_ok());
    for later in ["ess-history/2", "ess-history/3"] {
        let relabelled = text.replace(
            "\"format\":\"ess-history/1\"",
            &format!("\"format\":\"{later}\""),
        );
        assert_ne!(relabelled, text);
        assert_eq!(
            read_format1(relabelled.as_bytes(), &digest).unwrap_err(),
            HistoryRefusal::UnsupportedFormat {
                found: Some(later.to_owned())
            },
            "{later}"
        );
    }
}
/// `calc.core`: one command and no entity, the shape of the model beyond10x/ess#576 was found on.
fn calc_spec(scratch: &Path) -> PathBuf {
    let path = scratch.join("calc.yaml");
    std::fs::write(
        &path,
        "format: ess/6
system: calc
version: v1
domains:
  - calc.core

domain: calc.core

summary: A check that answers big or small, over no entity at all.

actors:
  - name: calc.core.Caller
    may:
      - calc.core.Check

commands:
  - name: calc.core.Check
    input:
      - name: n
        type: Integer
    outcomes:
      - name: big
        when: n >= 10
        emits:
          - calc.core.Big
        payload:
          calc.core.Big:
            n: input.n
      - name: small
        emits:
          - calc.core.Small
        payload:
          calc.core.Small:
            n: input.n

events:
  - name: calc.core.Big
    fields:
      - name: n
        type: Integer
  - name: calc.core.Small
    fields:
      - name: n
        type: Integer
",
    )
    .unwrap();
    path
}

fn calc_cases() -> Value {
    serde_json::json!([
        {"name": "calc", "target": "calc", "options": {"seeds": SEEDS}},
        {"name": "wrong-outcome", "target": "calc", "mutant": "wrong-outcome", "options": {"seeds": SEEDS}},
        {"name": "wrong-events", "target": "calc", "mutant": "wrong-events", "options": {"seeds": SEEDS}},
    ])
}

/// beyond10x/ess#576, finding 1: on a model without entities, a history that records each call's
/// input and published events lets `check-history` refuse an answer that is a declared outcome but
/// not the one the input takes, and one that publishes events its branch does not declare.
#[test]
fn check_history_holds_each_answer_to_its_recorded_input_and_events_on_a_model_without_entities() {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let spec = calc_spec(scratch.path());
    let (typescript, go) = walk_concurrent(scratch.path(), &spec, &calc_cases());
    for name in ["calc", "wrong-outcome", "wrong-events"] {
        assert_eq!(
            typescript.histories[name], go.histories[name],
            "`{name}`: the TypeScript and Go histories differ"
        );
        assert_eq!(result(&typescript, name), result(&go, name), "`{name}`");
    }
    let digest = ess_conformance::history::SpecDigest::new(
        serde_json::from_str::<Value>(
            &std::fs::read_to_string(
                emit(spec.to_str().unwrap(), "go", &scratch.path().join("digest"))
                    .join("suite.json"),
            )
            .unwrap(),
        )
        .unwrap()["provenance"]["spec_digest"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    for lane in [&typescript, &go] {
        for name in ["wrong-outcome", "wrong-events"] {
            let planted = result(lane, name);
            println!(
                "{name}: {} violation(s) of {SEEDS}; first failing seed {}",
                count(planted, "violations"),
                planted["failure"]["seed"]
            );
            assert!(count(planted, "violations") >= 1, "{planted}");
        }
        let correct = result(lane, "calc");
        assert_eq!(count(correct, "linearizable"), SEEDS, "{correct}");
        assert_eq!(lane.problems["calc"], "none");
        // Every history is `ess-history/3`, every call records its input, every answered call its
        // events, and each is written in the reader's own spelling.
        assert_eq!(lane.histories["calc"].len() as u64, SEEDS);
        for (seed, bytes) in &lane.histories["calc"] {
            let history = ess_conformance::history::read(bytes, &digest)
                .unwrap_or_else(|refusal| panic!("seed {seed} is refused: {refusal}"));
            assert_eq!(&serde_json::to_vec(&history).unwrap(), bytes, "seed {seed}");
            let document: Value = serde_json::from_slice(bytes).unwrap();
            assert_eq!(document["format"], "ess-history/3", "seed {seed}");
            for operation in document["operations"].as_array().unwrap() {
                assert!(operation["input"].is_object(), "seed {seed}: {operation}");
                assert_eq!(
                    operation["events"].is_array(),
                    operation["completion"] == "Returned",
                    "seed {seed}: {operation}"
                );
            }
        }
    }
}

/// An operation of `ess-history/3` carries its input and its events, reads and writes back byte for
/// byte, and is held to the format: neither field in a lower format, neither written `null`, and no
/// events on a call that never answered.
#[test]
#[allow(clippy::too_many_lines)]
fn an_ess_history_3_operation_records_its_input_and_events_and_the_reader_holds_them_to_the_format()
{
    use ess_conformance::history::{read, read_format1, HistoryFormat, HistoryRefusal, Node};
    let digest = history2_digest();
    let document = |format: &str, operation: &str| {
        format!(
            r#"{{"format":"{format}","history_id":"00000000-0000-4000-8000-000000000001","spec_digest":"{}","seed":1,"clients":1,"operations":[{operation}]}}"#,
            "ab".repeat(32)
        )
    };
    let returned = r#"{"operation_id":"00000000-0000-4000-8000-000000000002","client":0,"command":"calc.core.Check","subject_key":"","invoked_at":1,"returned_at":2,"completion":"Returned","outcome":"big","input":{"n":100,"note":null,"tag":{"a":"x","b":[true]}},"events":["calc.core.Big"]}"#;
    let bytes = document("ess-history/3", returned);
    let history = read(bytes.as_bytes(), &digest).unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(history.format, HistoryFormat::EssHistory3);
    assert_eq!(
        HistoryFormat::for_operations(&history.operations),
        HistoryFormat::EssHistory3
    );
    let operation = &history.operations[0];
    let input = operation.input.as_ref().expect("the input is read");
    assert_eq!(input["note"], Node::Null);
    assert_eq!(input["n"], serde_json::from_str::<Node>("100").unwrap());
    assert_eq!(
        operation
            .events
            .as_ref()
            .expect("the events are read")
            .iter()
            .map(ess_conformance::history::QualifiedName::as_str)
            .collect::<Vec<_>>(),
        ["calc.core.Big"]
    );
    assert_eq!(serde_json::to_vec(&history).unwrap(), bytes.as_bytes());

    // A lower format carries neither field, on the wire or written.
    for format in ["ess-history/1", "ess-history/2"] {
        let refused = read(document(format, returned).as_bytes(), &digest).unwrap_err();
        assert_eq!(refused.code(), "history.malformed", "{format}: {refused}");
        assert!(
            refused.to_string().contains("write `ess-history/3`"),
            "{format}: {refused}"
        );
        let mut lowered = history.clone();
        lowered.format = if format == "ess-history/1" {
            HistoryFormat::EssHistory1
        } else {
            HistoryFormat::EssHistory2
        };
        assert!(
            serde_json::to_vec(&lowered).is_err(),
            "{format} was written"
        );
    }
    // Absent where nothing was recorded, never `null`, and of its one shape.
    for (from, to, needle) in [
        (
            r#""input":{"n":100,"note":null,"tag":{"a":"x","b":[true]}}"#,
            r#""input":null"#,
            "`operations[0].input` is `null`",
        ),
        (
            r#""events":["calc.core.Big"]"#,
            r#""events":null"#,
            "`operations[0].events` is `null`",
        ),
        (
            r#""input":{"n":100,"note":null,"tag":{"a":"x","b":[true]}}"#,
            r#""input":[100]"#,
            "",
        ),
        (r#""events":["calc.core.Big"]"#, r#""events":[""]"#, ""),
        (
            r#""events":["calc.core.Big"]"#,
            r#""events":"calc.core.Big""#,
            "",
        ),
    ] {
        let changed = returned.replace(from, to);
        assert_ne!(changed, returned);
        let refused = read(document("ess-history/3", &changed).as_bytes(), &digest).unwrap_err();
        assert_eq!(refused.code(), "history.malformed", "{to}: {refused}");
        assert!(refused.to_string().contains(needle), "{to}: {refused}");
    }
    // A call that never answered published nothing anyone saw; it still sent its input.
    let indeterminate = returned.replace(
        r#""returned_at":2,"completion":"Returned","outcome":"big","#,
        r#""completion":"Indeterminate","#,
    );
    assert_eq!(
        read(
            document("ess-history/3", &indeterminate).as_bytes(),
            &digest
        )
        .unwrap_err(),
        HistoryRefusal::IndeterminateWithEvents {
            operation_id: "00000000-0000-4000-8000-000000000002".to_owned()
        }
    );
    let sent = indeterminate.replace(r#","events":["calc.core.Big"]"#, "");
    let read_back = read(document("ess-history/3", &sent).as_bytes(), &digest).unwrap();
    assert!(read_back.operations[0].input.is_some());
    // The retained format 1 reader refuses it by its format.
    assert_eq!(
        read_format1(bytes.as_bytes(), &digest).unwrap_err(),
        HistoryRefusal::UnsupportedFormat {
            found: Some("ess-history/3".to_owned())
        }
    );
}
