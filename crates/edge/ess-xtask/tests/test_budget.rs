//! The runner half of the test budget (the organization rule `Tests stay fast`): nextest names
//! every test that runs over 60 s, and each task that runs nextest prints its ten slowest tests
//! from the ``JUnit`` report of the `ci` profile, also when the run failed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_yaml::Value;

/// Where nextest writes the `ci` profile's ``JUnit`` report, relative to the workspace root.
const REPORT: &str = "target/nextest/ci/junit.xml";

/// The tasks that run `cargo nextest run` today.
const NEXTEST_TASKS: [&str; 5] = [
    "test-feature-off",
    "test-packages",
    "test-shard",
    "test-slow",
    "test-xtask",
];

/// The task that runs the slow lane: nextest in a loop, one run per package, in one command.
const SLOW_TASK: &str = "test-slow";

const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run" tests="12" skipped="0" failures="1" errors="0" uuid="x" timestamp="2026-11-22T00:12:28.633+01:00" time="99.0">
    <testsuite name="crate-a::alpha" tests="12" skipped="0" errors="0" failures="1">
        <testcase name="t01" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="1.500"/>
        <testcase name="t02" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.009"/>
        <testcase name="t03" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="12.250"/>
        <testcase name="t04" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="3.000"/>
        <testcase name="t05" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="61.042">
            <failure type="test failure">killed</failure>
        </testcase>
        <testcase name="t06" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="9.900"/>
        <testcase name="t07" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.100"/>
        <testcase name="t08" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="10.000"/>
        <testcase name="t09" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="2.000"/>
        <testcase name="t10" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.500"/>
        <testcase name="t11" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="4.000"/>
        <testcase name="t12" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.200"/>
    </testsuite>
</testsuites>
"#;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.contains("[workspace]"))
        })
        .expect("a workspace manifest stands above this crate")
        .to_path_buf()
}

fn taskfile() -> Value {
    let text = fs::read_to_string(workspace_root().join("Taskfile.yml")).expect("Taskfile.yml");
    ess_yaml::from_str(&text).expect("Taskfile.yml parses")
}

fn nextest_config() -> String {
    fs::read_to_string(workspace_root().join(".config/nextest.toml")).expect(".config/nextest.toml")
}

/// The non-blank, non-comment lines of the TOML table `header` (such as `[profile.ci]`), up to
/// the next line that starts a table.
fn table(config: &str, header: &str) -> Vec<String> {
    let mut lines = config.lines().map(str::trim);
    assert!(
        lines.by_ref().any(|line| line == header),
        ".config/nextest.toml has no `{header}` table"
    );
    lines
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// The string commands of `task`, in order (a `task:` or `defer:` entry is a mapping and is left out).
fn string_commands(taskfile: &Value, task: &str) -> Vec<String> {
    taskfile["tasks"][task]["cmds"]
        .as_sequence()
        .into_iter()
        .flatten()
        .filter_map(|command| command.as_str().map(str::to_owned))
        .collect()
}

/// Whether `command` passes `flag` followed by exactly `value` as the next word.
fn passes(command: &str, flag: &str, value: &str) -> bool {
    let words: Vec<&str> = command.split_whitespace().collect();
    words
        .windows(2)
        .any(|pair| pair[0] == flag && pair[1] == value)
}

/// Every task whose string commands run `cargo nextest run`, by name.
fn nextest_tasks(taskfile: &Value) -> Vec<String> {
    taskfile["tasks"]
        .as_mapping()
        .expect("Taskfile.yml has tasks")
        .keys()
        .filter_map(|key| key.as_str())
        .filter(|name| {
            string_commands(taskfile, name)
                .iter()
                .any(|command| command.contains("cargo nextest run"))
        })
        .map(str::to_owned)
        .collect()
}

fn slowest_command(taskfile: &Value) -> String {
    let commands = string_commands(taskfile, "test-slowest");
    let [command] = commands.as_slice() else {
        panic!("Taskfile.yml defines no single `test-slowest` command: {commands:?}")
    };
    command.clone()
}

/// Runs `test-slowest`'s command with `sh` in a fresh directory under `CARGO_TARGET_TMPDIR`,
/// holding `report` at `REPORT` when given. Returns whether it succeeded, and its stdout.
fn run_slowest(directory: &str, report: Option<&str>) -> (bool, String) {
    fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("test_budget")
        .join(directory);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    if let Some(report) = report {
        let path = root.join(REPORT);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, report).unwrap();
    }
    let output = Command::new("sh")
        .arg("-c")
        .arg(slowest_command(&taskfile()))
        .current_dir(&root)
        .output()
        .expect("sh runs");
    (
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

#[test]
fn the_ci_profile_reports_every_test_over_sixty_seconds() {
    let lines = table(&nextest_config(), "[profile.ci]");
    let reports = lines.len() == 2
        && lines
            .iter()
            .any(|line| line == "final-status-level = \"slow\"")
        && lines.iter().any(|line| {
            line == "slow-timeout = { period = \"60s\" }"
                || line == "slow-timeout = { period = \"60s\", terminate-after = 1 }"
        });
    assert!(
        reports,
        "the nextest profile `ci` does not report every test over 60 s: {lines:?}"
    );
}

#[test]
fn the_ci_profile_writes_the_junit_report_test_slowest_reads() {
    assert_eq!(
        table(&nextest_config(), "[profile.ci.junit]"),
        ["path = \"junit.xml\""]
    );
    let command = slowest_command(&taskfile());
    assert!(
        command.contains(&format!("report=\"${{SLOWEST_REPORT:-{REPORT}}}\"")),
        "test-slowest does not read ${{SLOWEST_REPORT}}, defaulting to {REPORT}"
    );
    let taskfile = taskfile();
    let slow = string_commands(&taskfile, SLOW_TASK);
    assert!(
        slow.iter().any(|command| command
            .contains("SLOWEST_REPORT=target/nextest/slow/junit.xml task test-slowest")),
        "`{SLOW_TASK}` does not call `task test-slowest` with SLOWEST_REPORT=target/nextest/slow/junit.xml: {slow:?}"
    );
}

#[test]
fn the_slow_profile_kills_a_test_after_twenty_minutes_and_writes_junit() {
    let config = nextest_config();
    assert_eq!(
        table(&config, "[profile.slow]"),
        [
            "slow-timeout = { period = \"120s\", terminate-after = 10 }",
            "final-status-level = \"slow\""
        ]
    );
    assert_eq!(
        table(&config, "[profile.slow.junit]"),
        ["path = \"junit.xml\""]
    );
}

#[test]
fn every_nextest_run_in_the_taskfile_uses_the_ci_profile() {
    let taskfile = taskfile();
    let tasks = nextest_tasks(&taskfile);
    for expected in NEXTEST_TASKS {
        assert!(
            tasks.iter().any(|task| task == expected),
            "`{expected}` no longer runs nextest: {tasks:?}"
        );
    }
    for task in &tasks {
        for command in string_commands(&taskfile, task) {
            if command.contains("cargo nextest run") {
                let ci = passes(&command, "--profile", "ci");
                let slow = passes(&command, "--profile", "slow")
                    && passes(&command, "--run-ignored", "only")
                    && command.split_whitespace().any(|word| word == "-E");
                assert!(
                    ci || slow,
                    "`{task}` runs nextest without `--profile ci`, or `--profile slow` with `--run-ignored only` and `-E`: {command}"
                );
            }
        }
    }
}

#[test]
fn every_nextest_run_prints_its_ten_slowest_tests_even_when_it_fails() {
    let taskfile = taskfile();
    for task in nextest_tasks(&taskfile) {
        let commands = taskfile["tasks"][task.as_str()]["cmds"]
            .as_sequence()
            .expect("cmds is a sequence");
        if task == SLOW_TASK {
            let command = commands
                .iter()
                .filter_map(|entry| entry.as_str())
                .find(|command| command.contains("cargo nextest run"))
                .expect("the task runs nextest");
            let run = command.find("cargo nextest run").unwrap();
            let Some(failed) = command[run..].find("|| status=1") else {
                panic!("`{task}` does not record a failed run with `|| status=1`")
            };
            assert!(
                command[run + failed..].contains("task test-slowest"),
                "`{task}` does not run `task test-slowest` after a failed run"
            );
            continue;
        }
        let deferred = commands
            .iter()
            .position(|entry| entry["defer"]["task"].as_str() == Some("test-slowest"))
            .unwrap_or_else(|| panic!("`{task}` does not defer `test-slowest`"));
        let run = commands
            .iter()
            .position(|entry| {
                entry
                    .as_str()
                    .is_some_and(|command| command.contains("cargo nextest run"))
            })
            .expect("the task runs nextest");
        assert!(
            deferred < run,
            "`{task}` defers `test-slowest` only after nextest runs, so a failed run prints nothing"
        );
    }
}

#[test]
fn test_slowest_prints_the_ten_slowest_tests_slowest_first() {
    let (success, stdout) = run_slowest("ten-slowest", Some(SAMPLE));
    assert!(success);
    let expected = "\
the ten slowest tests of this run (target/nextest/ci/junit.xml):
    61.042s  crate-a::alpha t05
    12.250s  crate-a::alpha t03
    10.000s  crate-a::alpha t08
     9.900s  crate-a::alpha t06
     4.000s  crate-a::alpha t11
     3.000s  crate-a::alpha t04
     2.000s  crate-a::alpha t09
     1.500s  crate-a::alpha t01
     0.500s  crate-a::alpha t10
     0.200s  crate-a::alpha t12
";
    assert_eq!(stdout, expected);
}

#[test]
fn test_slowest_reports_a_missing_junit_report_and_succeeds() {
    let (success, stdout) = run_slowest("no-report", None);
    assert!(success);
    assert_eq!(
        stdout,
        "test-slowest: no JUnit report at target/nextest/ci/junit.xml\n"
    );
}
