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

/// Both explore lanes, both billing lanes and both retry lanes, run once for every test below.
struct Lanes {
    explore: (Lane, Lane),
    billing: (Lane, Lane),
    retry: (Lane, Lane),
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
        Lanes {
            explore,
            billing,
            retry,
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
        (&lanes.billing.0, "billing", BILLING),
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
