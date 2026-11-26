//! The pull-request gate is split into parallel jobs, and the split is held to `task check`.
//!
//! `.github/workflows/ci.yml` does not run `task check` as one job. A few jobs each run Taskfile
//! tasks as steps: `checks` runs the static, smoke, xtask and fuzz tasks one after another; one job
//! compiles the test binaries once into nextest archives; a matrix of test shards downloads them and
//! runs one partition each; and a job named `Gate` aggregates the result. A split like that has one
//! silent failure mode — a step of `task check` that no job calls exits nowhere, so nothing turns
//! red when it goes missing. The coverage case below reads both files and names the step instead.
//!
//! The release workflow runs the full gate on a tag unless the required checks passed on the
//! tagged tree: `main`'s own push run on the tagged commit, or the pull-request run of the queue
//! head the tagged commit merged, which `ci.yml` runs in full (a pull request from any other head
//! and a merge-group run archive the narrowed feature-off build and never stand in). It also
//! reuses the archives `package.yml` built from that head; the release cases below hold both, and
//! run the gate-reuse and archive-reuse scripts themselves against real Git histories and a
//! stand-in for `gh`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
#[path = "../../ess-cli/tests/support/executable.rs"]
mod executable;

use ess_yaml::Value;
use serde_json::json;

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

fn yaml(relative: &str) -> Value {
    let path = workspace_root().join(relative);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("reading {relative}: {error}"));
    ess_yaml::from_str(&text).unwrap_or_else(|error| panic!("parsing {relative}: {error}"))
}

fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}

fn scalar(value: &Value) -> String {
    match value {
        Value::String(string) => string.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        other => panic!("a matrix value is not a scalar: {other:?}"),
    }
}

/// The job ids of `ci.yml` whose result the aggregate `Gate` carries.
///
/// Everything but the aggregate itself and the native macOS witnesses, which were never part of
/// `Gate` and are skipped when the release workflow calls this one for a backfill.
fn gated_jobs(ci: &Value) -> BTreeSet<String> {
    ci["jobs"]
        .as_mapping()
        .expect("ci.yml has jobs")
        .keys()
        .map(|key| text(key).to_owned())
        .filter(|id| id != "gate" && id != "output-ownership-macos")
        .collect()
}

/// The rows a job's matrix expands to, each mapping a matrix key to its value. A job without a
/// matrix is one row with no keys.
fn matrix_rows(job: &Value) -> Vec<BTreeMap<String, String>> {
    let Some(matrix) = job["strategy"]["matrix"].as_mapping() else {
        return vec![BTreeMap::new()];
    };
    if let Some(include) = matrix.get("include") {
        assert_eq!(
            matrix.len(),
            1,
            "a matrix mixes `include` with axes, and this reader expands only one of the two"
        );
        return include
            .as_sequence()
            .expect("`include` is a list")
            .iter()
            .map(|row| {
                row.as_mapping()
                    .expect("an `include` row is a mapping")
                    .iter()
                    .map(|(key, value)| (text(key).to_owned(), scalar(value)))
                    .collect()
            })
            .collect();
    }
    let mut rows = vec![BTreeMap::new()];
    for (key, values) in matrix {
        let key = text(key).to_owned();
        let values = values
            .as_sequence()
            .unwrap_or_else(|| panic!("matrix axis `{key}` is not a list"));
        rows = rows
            .iter()
            .flat_map(|row| {
                values.iter().map(|value| {
                    let mut row = row.clone();
                    row.insert(key.clone(), scalar(value));
                    row
                })
            })
            .collect();
    }
    rows
}

/// One Taskfile task one step of one gated job runs, for one row of that job's matrix.
#[derive(Debug)]
struct Lane {
    job: String,
    /// The index of the matrix row the step runs for.
    row: usize,
    step: usize,
    task: String,
    vars: Vec<String>,
}

/// Every Taskfile task a `run: task …` step of a gated job invokes, each with the variable
/// assignments of that invocation and once per matrix row. `task a b VAR=x` runs `a` then `b`,
/// both with `VAR`, so one step can name several.
fn lane_invocations(ci: &Value) -> Vec<Lane> {
    let mut lanes = Vec::new();
    for id in gated_jobs(ci) {
        let job = &ci["jobs"][id.as_str()];
        for (row_index, row) in matrix_rows(job).into_iter().enumerate() {
            for (step, entry) in job["steps"].as_sequence().into_iter().flatten().enumerate() {
                let run = text(&entry["run"]).trim();
                let Some(command) = run.strip_prefix("task ") else {
                    continue;
                };
                assert!(
                    !command.contains('\n'),
                    "`{id}` runs a multi-line task step, which this reader cannot split: {run}"
                );
                let mut command = command.to_owned();
                for (key, value) in &row {
                    command = command.replace(&format!("${{{{ matrix.{key} }}}}"), value);
                }
                assert!(
                    !command.contains("${{"),
                    "`{id}` runs a task with an expression this reader cannot expand: {command}"
                );
                let (vars, tasks): (Vec<String>, Vec<String>) = command
                    .split_whitespace()
                    .map(str::to_owned)
                    .partition(|word| word.contains('='));
                assert!(!tasks.is_empty(), "`{id}` has a task step naming no task");
                for task in tasks {
                    lanes.push(Lane {
                        job: id.clone(),
                        row: row_index,
                        step,
                        task,
                        vars: vars.clone(),
                    });
                }
            }
        }
    }
    lanes
}

/// The `SHARD=<m>/<n>` values the lanes pass to `task`, one per invocation.
fn shards_of(ci: &Value, task: &str) -> Vec<String> {
    lane_invocations(ci)
        .into_iter()
        .filter(|lane| lane.task == task)
        .map(|lane| {
            let [assignment] = lane.vars.as_slice() else {
                panic!(
                    "a `{task}` lane passes exactly `SHARD=<m>/<n>`, not {:?}",
                    lane.vars
                )
            };
            assignment
                .strip_prefix("SHARD=")
                .unwrap_or_else(|| panic!("`{assignment}` is not a SHARD assignment"))
                .to_owned()
        })
        .collect()
}

/// `task`'s lanes are one complete partition: `1/n` through `n/n`, each exactly once.
fn assert_complete_partition(ci: &Value, task: &str) -> BTreeSet<String> {
    let shards = shards_of(ci, task);
    let total = shards.len();
    assert!(total > 1, "`{task}` runs in {total} shard(s)");
    let expected: BTreeSet<String> = (1..=total).map(|m| format!("{m}/{total}")).collect();
    let found: BTreeSet<String> = shards.iter().cloned().collect();
    assert_eq!(
        found, expected,
        "the `{task}` shards of ci.yml leave a partition unrun or run one twice"
    );
    found
}

/// Every task reached from `name` through `task:` entries in its `cmds`, including `name`.
fn reached(taskfile: &Value, name: &str, into: &mut BTreeSet<String>) {
    if !into.insert(name.to_owned()) {
        return;
    }
    let task = &taskfile["tasks"][name];
    assert!(
        !task.is_null(),
        "a lane calls `{name}`, which Taskfile.yml does not define"
    );
    for command in task["cmds"].as_sequence().into_iter().flatten() {
        if let Some(inner) = command["task"].as_str() {
            reached(taskfile, inner, into);
        }
    }
}

fn direct_subtasks(taskfile: &Value, name: &str) -> Vec<String> {
    taskfile["tasks"][name]["cmds"]
        .as_sequence()
        .into_iter()
        .flatten()
        .filter_map(|command| command["task"].as_str().map(str::to_owned))
        .collect()
}

fn shell_commands(taskfile: &Value, name: &str) -> Vec<String> {
    taskfile["tasks"][name]["cmds"]
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

fn needs_of(job: &Value) -> BTreeSet<String> {
    match &job["needs"] {
        Value::String(one) => BTreeSet::from([one.clone()]),
        Value::Sequence(many) => many.iter().map(|need| text(need).to_owned()).collect(),
        _ => BTreeSet::new(),
    }
}

fn step_using<'a>(job: &'a Value, action: &str) -> Option<(usize, &'a Value)> {
    job["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .enumerate()
        .find(|(_, step)| text(&step["uses"]).starts_with(&format!("{action}@")))
}

#[test]
fn every_step_of_task_check_runs_in_some_pull_request_lane() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let lanes = lane_invocations(&ci);
    let jobs: BTreeSet<&String> = lanes.iter().map(|lane| &lane.job).collect();
    assert!(
        jobs.len() > 1,
        "ci.yml runs its tasks in {} job(s); the gate is not split",
        jobs.len()
    );

    let mut covered = BTreeSet::new();
    for lane in &lanes {
        reached(&taskfile, &lane.task, &mut covered);
    }

    // `test` is the one step CI replaces rather than calls: its three commands run as nextest
    // archives run in shards, the xtask task and the doc-tests nextest skips.
    let check = direct_subtasks(&taskfile, "check");
    assert!(
        check.iter().any(|step| step == "test"),
        "`task check` no longer runs `test`"
    );
    let missing: Vec<&String> = check
        .iter()
        .filter(|step| *step != "test" && !covered.contains(*step))
        .collect();
    assert!(
        missing.is_empty(),
        "these steps of `task check` run in no job of ci.yml, so a pull request never runs them: \
         {missing:?}"
    );
    for replacement in [
        "test-archive-workspace",
        "test-archive-feature-off",
        "test-shard",
        "test-feature-off",
        "test-feature-off-doc",
        "test-xtask",
        "test-doc",
    ] {
        assert!(
            covered.contains(replacement),
            "no job of ci.yml runs `{replacement}`, one of the parts `task test` is split into"
        );
    }
    // The xtask half is shared, not restated: `task test` calls the same task the job does.
    assert!(
        direct_subtasks(&taskfile, "test")
            .iter()
            .any(|step| step == "test-xtask"),
        "`task test` restates the xtask command instead of calling `test-xtask`"
    );
}

/// Consumer coverage is parked by `ESS-EVOLUTION.md` revision 3: `task check` runs
/// `consumer-check` only when `CONSUMER_CHECKS=true`, the task itself stays defined, no lane
/// reaches it, and the retired `SKIP_CONSUMER_CHECKS` switch — which would now mean nothing — is
/// spelled in no file a gate reads.
#[test]
fn consumer_check_runs_in_task_check_only_when_opted_in() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    assert!(
        !taskfile["tasks"]["consumer-check"].is_null(),
        "`task consumer-check` is no longer defined; parking keeps it runnable"
    );
    assert!(
        !direct_subtasks(&taskfile, "check")
            .iter()
            .any(|step| step == "consumer-check"),
        "`task check` calls `consumer-check` unconditionally"
    );
    let guarded: Vec<String> = shell_commands(&taskfile, "check")
        .into_iter()
        .filter(|command| command.contains("consumer-check"))
        .collect();
    let [guard] = guarded.as_slice() else {
        panic!(
            "`task check` has {} consumer-check commands, not one: {guarded:?}",
            guarded.len()
        )
    };
    let lines: Vec<&str> = guard.lines().map(str::trim).collect();
    let condition = lines
        .iter()
        .position(|line| {
            line.starts_with("if [ \"{{.CONSUMER_CHECKS | default \"false\"}}\" = \"true\" ]")
                && line.ends_with("then")
        })
        .unwrap_or_else(|| {
            panic!("consumer-check is not guarded by CONSUMER_CHECKS=true: {guard}")
        });
    assert_eq!(
        lines.get(condition + 1).copied(),
        Some("task consumer-check"),
        "the opted-in branch does not run `task consumer-check`: {guard}"
    );

    let mut covered = BTreeSet::new();
    for lane in lane_invocations(&ci) {
        reached(&taskfile, &lane.task, &mut covered);
    }
    assert!(
        !covered.contains("consumer-check"),
        "a CI lane runs the parked consumer-check"
    );

    let root = workspace_root();
    let mut files = vec![root.join("Taskfile.yml")];
    for entry in fs::read_dir(root.join(".github/workflows")).expect("workflows") {
        files.push(entry.unwrap().path());
    }
    for action in fs::read_dir(root.join(".github/actions"))
        .expect("actions")
        .flatten()
    {
        files.push(action.path().join("action.yml"));
    }
    // Split, so that this file does not contain what it scans for.
    let retired = ["SKIP_", "CONSUMER_CHECKS"].concat();
    let spelled: Vec<String> = files
        .iter()
        .filter(|file| file.is_file())
        .filter(|file| fs::read_to_string(file).unwrap().contains(&retired))
        .map(|file| file.display().to_string())
        .collect();
    assert!(
        spelled.is_empty(),
        "these gate files still spell the retired `{retired}`: {spelled:?}"
    );
}

/// The filterset of workspace group a; group b is its complement.
///
/// The workspace test binaries no longer fit a hosted runner extracted all at once. Queue runs
/// 41997613644 and 42017687798 stopped Test 11/20 and Test 4/20 at 4.9 GB free on `/` while
/// nextest extracted the one workspace archive: its 1817 binaries take 45.6 GB extracted, already
/// built with `debug = 0` and `strip = symbols`, on runners with 52 GB free. Split by binary name
/// on the real archive, the ess-conformance binaries `a`..`m` (its lib, `ess_conformance`, among
/// them) are 20.43 GB in 493 binaries and everything else is 25.22 GB in 1319, so each shard
/// extracts one of two halves.
const WORKSPACE_GROUP_A: &str = "package(ess-conformance) & binary(/^[a-m]/)";

/// The workspace groups `test-shard` accepts, read from its `requires` enum.
fn workspace_groups(taskfile: &Value) -> BTreeSet<String> {
    let vars = taskfile["tasks"]["test-shard"]["requires"]["vars"]
        .as_sequence()
        .expect("`test-shard` requires its variables");
    let group = vars
        .iter()
        .find(|var| text(&var["name"]) == "GROUP")
        .expect("`test-shard` does not require GROUP");
    group["enum"]
        .as_sequence()
        .expect("`test-shard` does not restrict GROUP to the archive groups")
        .iter()
        .map(scalar)
        .collect()
}

/// The archives select what `task test` selects, one task each, so that two jobs can build them
/// side by side; local `task test-archive` still builds both. The workspace task writes two
/// archives from one build, a filterset and its complement over the same selection, so together
/// they hold every workspace test binary exactly once.
fn assert_each_archive_selects_what_task_test_selects(taskfile: &Value) {
    assert_eq!(
        direct_subtasks(taskfile, "test-archive"),
        ["test-archive-workspace", "test-archive-feature-off"],
        "`task test-archive` no longer builds both archives"
    );
    assert_eq!(
        text(&taskfile["vars"]["WORKSPACE_GROUP_A"]),
        WORKSPACE_GROUP_A,
        "workspace group a is no longer the ess-conformance binaries a..m"
    );
    let commands = shell_commands(taskfile, "test-archive-workspace");
    let selection = "cargo nextest archive --workspace --exclude ess-xtask --locked ";
    let mut archives: BTreeMap<String, String> = BTreeMap::new();
    for command in commands
        .iter()
        .filter(|command| command.contains("cargo nextest archive"))
    {
        let rest = command.strip_prefix(selection).unwrap_or_else(|| {
            panic!("a workspace archive selects other than what `task test` selects: {command}")
        });
        let rest = rest.strip_prefix("-E '").unwrap_or_else(|| {
            panic!("a workspace archive is not filtered, so a shard extracts all of it: {command}")
        });
        let (filterset, rest) = rest
            .split_once("' ")
            .unwrap_or_else(|| panic!("an unterminated filterset: {command}"));
        let group = rest
            .strip_prefix("--archive-file {{.NEXTEST_ARCHIVES}}/workspace-")
            .and_then(|rest| rest.strip_suffix(".tar.zst"))
            .unwrap_or_else(|| panic!("a workspace archive names no group archive: {command}"));
        assert!(
            archives
                .insert(group.to_owned(), filterset.to_owned())
                .is_none(),
            "two archives of workspace group `{group}`"
        );
    }
    assert_eq!(
        archives,
        BTreeMap::from([
            ("a".to_owned(), "{{.WORKSPACE_GROUP_A}}".to_owned()),
            ("b".to_owned(), "not ({{.WORKSPACE_GROUP_A}})".to_owned()),
        ]),
        "the workspace archives are not group a's filterset and its complement"
    );
    assert_eq!(
        archives.keys().cloned().collect::<BTreeSet<_>>(),
        workspace_groups(taskfile),
        "`test-shard` accepts other groups than the archives `test-archive-workspace` writes"
    );
    let archive = commands.join("\n");
    assert!(
        !archive.contains("/workspace.tar.zst") && !archive.contains("/feature-off.tar.zst"),
        "the workspace archive task also builds an unsplit or a feature-off archive: {archive}"
    );
    // Its pull-request narrowing is held by
    // `pull_requests_run_feature_off_on_number_semantics_and_every_other_run_runs_all_of_it`.
    let archive = shell_commands(taskfile, "test-archive-feature-off").join("\n");
    let feature_off = archive
        .lines()
        .find(|line| line.contains("/feature-off.tar.zst"))
        .unwrap_or_else(|| {
            panic!("`test-archive-feature-off` builds no feature-off archive: {archive}")
        });
    assert!(
        feature_off.starts_with("cargo nextest archive --locked {{.FEATURE_OFF_PACKAGES}} ")
            && feature_off.ends_with("--archive-file {{.NEXTEST_ARCHIVES}}/feature-off.tar.zst"),
        "{feature_off}"
    );
}

/// The `(GROUP, PART)` each `test-shard` lane passes, with the job and matrix row that runs it.
fn workspace_parts(ci: &Value) -> Vec<(String, String, (String, usize))> {
    lane_invocations(ci)
        .into_iter()
        .filter(|lane| lane.task == "test-shard")
        .map(|lane| {
            let (mut group, mut part) = (None, None);
            for assignment in &lane.vars {
                let slot = if let Some(value) = assignment.strip_prefix("GROUP=") {
                    group.replace(value.to_owned())
                } else if let Some(value) = assignment.strip_prefix("PART=") {
                    part.replace(value.to_owned())
                } else {
                    panic!("a `test-shard` lane passes `{assignment}`, not GROUP or PART")
                };
                assert!(slot.is_none(), "a `test-shard` lane passes {:?}", lane.vars);
            }
            (
                group.unwrap_or_else(|| panic!("a `test-shard` lane passes no GROUP")),
                part.unwrap_or_else(|| panic!("a `test-shard` lane passes no PART")),
                (lane.job, lane.row),
            )
        })
        .collect()
}

/// Each workspace group's parts are one complete partition `1/n` through `n/n`, every group the
/// Taskfile archives runs, and every shard job runs one part of one group and one feature-off
/// partition, which is one complete partition of its own.
fn assert_workspace_groups_are_complete_partitions(ci: &Value, taskfile: &Value) {
    let groups = workspace_groups(taskfile);
    assert_eq!(
        groups,
        BTreeSet::from(["a".to_owned(), "b".to_owned()]),
        "the workspace archive is not split into groups a and b"
    );
    let mut by_group: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (group, part, _) in workspace_parts(ci) {
        by_group.entry(group).or_default().push(part);
    }
    assert_eq!(
        by_group.keys().cloned().collect::<BTreeSet<_>>(),
        groups,
        "the shards of ci.yml leave a workspace group unrun or run one with no archive"
    );
    for (group, parts) in &by_group {
        let total = parts.len();
        assert!(
            total > 1,
            "workspace group `{group}` runs in {total} shard(s)"
        );
        let expected: BTreeSet<String> = (1..=total).map(|m| format!("{m}/{total}")).collect();
        let found: BTreeSet<String> = parts.iter().cloned().collect();
        assert_eq!(
            found, expected,
            "the shards of workspace group `{group}` leave a part unrun or run one twice"
        );
    }

    assert_complete_partition(ci, "test-feature-off");
    let mut per_row: BTreeMap<(String, usize), Vec<String>> = BTreeMap::new();
    for lane in lane_invocations(ci)
        .into_iter()
        .filter(|lane| lane.task == "test-shard" || lane.task == "test-feature-off")
    {
        per_row
            .entry((lane.job, lane.row))
            .or_default()
            .push(lane.task);
    }
    for ((job, row), mut tasks) in per_row {
        tasks.sort();
        assert_eq!(
            tasks,
            ["test-feature-off", "test-shard"],
            "row {row} of `{job}` does not run one workspace part and one feature-off partition"
        );
    }
}

/// A workspace shard extracts its group's archive into the checkout, deletes both archives, and
/// only then runs its part from the extracted metadata, compiling nothing. An archive left on disk
/// beside its extracted binaries is what ran the runners out of space (see `WORKSPACE_GROUP_A`).
fn assert_workspace_shard_extracts_deletes_and_runs(taskfile: &Value) {
    assert_eq!(
        text(&taskfile["tasks"]["test-shard"]["vars"]["OTHER_GROUP"]),
        "{{if eq .GROUP \"a\"}}b{{else}}a{{end}}",
        "`test-shard` does not name the other group's archive"
    );
    let commands = shell_commands(taskfile, "test-shard");
    let [drop_other, extract, drop_own, run] = commands.as_slice() else {
        panic!("`test-shard` does not drop, extract, delete and run in four commands: {commands:?}")
    };
    let own = "{{.NEXTEST_ARCHIVES}}/workspace-{{.GROUP}}.tar.zst";
    assert_eq!(
        drop_other, "rm -f {{.NEXTEST_ARCHIVES}}/workspace-{{.OTHER_GROUP}}.tar.zst",
        "`test-shard` keeps the other group's archive on disk"
    );
    assert!(
        extract.starts_with(&format!("cargo nextest list --archive-file {own} "))
            && extract.ends_with(" > /dev/null"),
        "`test-shard` does not extract its own group's archive first: {extract}"
    );
    for (flag, value) in [("--workspace-remap", "."), ("--extract-to", ".")] {
        assert!(
            passes(extract, flag, value),
            "`test-shard` extracts without `{flag} {value}`: {extract}"
        );
    }
    assert!(
        extract
            .split_whitespace()
            .any(|word| word == "--extract-overwrite"),
        "`test-shard` lacks `--extract-overwrite`: {extract}"
    );
    assert_eq!(
        drop_own,
        &format!("rm {own}"),
        "`test-shard` runs with its archive still on disk"
    );
    assert!(
        run.starts_with("cargo nextest run ")
            && !run.contains("--archive-file")
            && !run.contains(".tar.zst"),
        "`test-shard` runs from an archive, not from what it extracted: {run}"
    );
    for (flag, value) in [
        (
            "--binaries-metadata",
            "target/nextest/binaries-metadata.json",
        ),
        ("--cargo-metadata", "target/nextest/cargo-metadata.json"),
        ("--workspace-remap", "."),
        ("--target-dir-remap", "target"),
        ("--partition", "slice:{{.PART}}"),
    ] {
        assert!(
            passes(run, flag, value),
            "`test-shard` lacks `{flag} {value}`: {run}"
        );
    }
    assert!(
        run.split_whitespace().any(|word| word == "--no-fail-fast"),
        "`test-shard` lacks `--no-fail-fast`: {run}"
    );
}

#[test]
fn the_workspace_shards_are_one_complete_partition() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    assert_workspace_groups_are_complete_partitions(&ci, &taskfile);

    // Local `task test` is unchanged: cargo test over the same two selections.
    let local = shell_commands(&taskfile, "test");
    assert!(
        local[0].contains("cargo test --workspace --exclude ess-xtask"),
        "{local:?}"
    );
    assert!(
        local.join("\n").contains("{{.FEATURE_OFF_PACKAGES}}"),
        "{local:?}"
    );

    assert_each_archive_selects_what_task_test_selects(&taskfile);
    assert_workspace_shard_extracts_deletes_and_runs(&taskfile);

    // A shard runs one archive's partition and compiles nothing. It extracts into the checkout,
    // because test binaries bake `env!("CARGO_BIN_EXE_…")` and `env!("CARGO_TARGET_TMPDIR")` in
    // at build time: extracted anywhere else, those paths name nothing and the tests that spawn
    // the CLI fail with `NotFound`.
    //
    // The partition is `slice:`, not `count:`. `count:` deals each binary's tests out on its own,
    // starting at shard 1, so a binary of one test always lands on shard 1 and shard n gets only
    // what is left over: queue run 36233791835 ran 1,072 / 960 / 875 / 780 workspace tests for
    // 584 / 575 / 414 / 273 test-seconds. `slice:` deals the whole sorted list out once, which
    // on that run's own timings is 407 / 520 / 432 / 489 and 147 / 154 / 148 / 125 feature-off
    // test-seconds against 185 / 202 / 124 / 64.
    let commands = shell_commands(&taskfile, "test-feature-off");
    let [run] = commands.as_slice() else {
        panic!(
            "`test-feature-off` runs {} commands, not one: {commands:?}",
            commands.len()
        )
    };
    assert!(
        run.starts_with(
            "cargo nextest run --archive-file {{.NEXTEST_ARCHIVES}}/feature-off.tar.zst"
        ),
        "`test-feature-off` does not run the feature-off archive: {run}"
    );
    for (flag, value) in [
        ("--workspace-remap", "."),
        ("--extract-to", "."),
        ("--partition", "slice:{{.SHARD}}"),
    ] {
        assert!(
            passes(run, flag, value),
            "`test-feature-off` lacks `{flag} {value}`: {run}"
        );
    }
    for flag in ["--extract-overwrite", "--no-fail-fast"] {
        assert!(
            run.split_whitespace().any(|word| word == flag),
            "`test-feature-off` lacks `{flag}`: {run}"
        );
    }

    // Nextest does not run doc-tests, so both selections' doc-tests run beside it — once each.
    let doc = shell_commands(&taskfile, "test-doc").join("\n");
    assert!(
        doc.contains("cargo test --workspace --exclude ess-xtask") && doc.contains("--doc"),
        "{doc}"
    );
    let doc = shell_commands(&taskfile, "test-feature-off-doc").join("\n");
    assert!(
        doc.contains("cargo test --locked --doc {{.FEATURE_OFF_PACKAGES}}"),
        "{doc}"
    );
    let mut covered_by: BTreeMap<String, usize> = BTreeMap::new();
    for lane in lane_invocations(&ci) {
        let mut reach = BTreeSet::new();
        reached(&taskfile, &lane.task, &mut reach);
        for task in reach {
            *covered_by.entry(task).or_default() += 1;
        }
    }
    for doc in ["test-doc", "test-feature-off-doc"] {
        assert_eq!(
            covered_by.get(doc).copied().unwrap_or(0),
            1,
            "`{doc}` runs in {} lanes, not exactly one",
            covered_by.get(doc).copied().unwrap_or(0)
        );
    }
}

/// The split holds only while both halves run: a dropped group, a filterset that is no longer the
/// complement, a shard that runs before deleting its archive, or a part left unrun each fail.
#[test]
fn the_workspace_split_fails_on_a_dropped_group_a_broken_complement_or_a_kept_archive() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let holds = |ci: &Value, taskfile: &Value| {
        assert_workspace_groups_are_complete_partitions(ci, taskfile);
        assert_each_archive_selects_what_task_test_selects(taskfile);
        assert_workspace_shard_extracts_deletes_and_runs(taskfile);
    };
    holds(&ci, &taskfile);

    let caught = |ci: &Value, taskfile: &Value, what: &str| {
        assert!(
            std::panic::catch_unwind(|| holds(ci, taskfile)).is_err(),
            "{what} went unnoticed"
        );
    };
    let command_index = |taskfile: &Value, task: &str, needle: &str| {
        taskfile["tasks"][task]["cmds"]
            .as_sequence()
            .unwrap()
            .iter()
            .position(|command| command.as_str().is_some_and(|c| c.contains(needle)))
            .unwrap()
    };

    // Group b's archive dropped.
    let mut mutated = taskfile.clone();
    let index = command_index(&mutated, "test-archive-workspace", "/workspace-b.tar.zst");
    mutated["tasks"]["test-archive-workspace"]["cmds"]
        .as_sequence_mut()
        .unwrap()
        .remove(index);
    caught(&ci, &mutated, "a dropped workspace-b archive");

    // Group b no longer the complement of group a.
    let mut mutated = taskfile.clone();
    let index = command_index(&mutated, "test-archive-workspace", "/workspace-b.tar.zst");
    mutated["tasks"]["test-archive-workspace"]["cmds"][index] = Value::String(
        "cargo nextest archive --workspace --exclude ess-xtask --locked \
         -E 'not package(ess-conformance)' --archive-file {{.NEXTEST_ARCHIVES}}/workspace-b.tar.zst"
            .to_owned(),
    );
    caught(&ci, &mutated, "a group b that is not group a's complement");

    // Group a's filterset changed underneath both archives.
    let mut mutated = taskfile.clone();
    mutated["vars"]["WORKSPACE_GROUP_A"] = Value::String("package(ess-conformance)".to_owned());
    caught(&ci, &mutated, "a changed group a filterset");

    // The shard runs with its own archive still on disk.
    let mut mutated = taskfile.clone();
    let index = command_index(
        &mutated,
        "test-shard",
        "rm {{.NEXTEST_ARCHIVES}}/workspace-{{.GROUP}}",
    );
    mutated["tasks"]["test-shard"]["cmds"]
        .as_sequence_mut()
        .unwrap()
        .remove(index);
    caught(&ci, &mutated, "a shard that keeps its archive");

    // The shard runs from the archive again instead of from what it extracted.
    let mut mutated = taskfile.clone();
    let index = command_index(&mutated, "test-shard", "cargo nextest run");
    mutated["tasks"]["test-shard"]["cmds"][index] = Value::String(
        "cargo nextest run --archive-file {{.NEXTEST_ARCHIVES}}/workspace-{{.GROUP}}.tar.zst \
         --workspace-remap . --extract-to . --extract-overwrite --no-fail-fast \
         --partition slice:{{.PART}}"
            .to_owned(),
    );
    caught(&ci, &mutated, "a shard that runs from its archive");

    // Every group-b shard removed from the matrix.
    let mut mutated = ci.clone();
    mutated["jobs"]["test"]["strategy"]["matrix"]["include"]
        .as_sequence_mut()
        .unwrap()
        .retain(|row| text(&row["group"]) == "a");
    caught(&mutated, &taskfile, "a matrix without group b");

    // One part of a group run twice and another left unrun.
    let mut mutated = ci.clone();
    let rows = mutated["jobs"]["test"]["strategy"]["matrix"]["include"]
        .as_sequence_mut()
        .unwrap();
    let last = rows.len() - 1;
    let duplicate = rows[last - 1]["part"].clone();
    rows[last]["part"] = duplicate;
    caught(&mutated, &taskfile, "a workspace part run twice");
}

/// The test binaries are compiled once, each archive by a job of its own so that the two compile
/// side by side: in queue run 36233791835 one job built them one after the other, the workspace
/// archive in 3:11 and then the feature-off one in 1:44, and every shard waited for both. Each
/// build job is unmatrixed, runs its one archive task and uploads the directory the Taskfile
/// names under an artifact name of its own; every shard job waits for both, downloads both
/// artifacts into that directory before its first task, and runs nothing but the two
/// archive-running tasks.
#[test]
fn the_test_shards_run_the_archives_two_jobs_build_side_by_side_and_compile_nothing() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let lanes = lane_invocations(&ci);
    let directory = text(&taskfile["vars"]["NEXTEST_ARCHIVES"]);
    assert!(
        !directory.is_empty(),
        "Taskfile.yml names no NEXTEST_ARCHIVES directory"
    );
    assert!(
        !lanes.iter().any(|lane| lane.task == "test-archive"),
        "a job of ci.yml builds both archives one after the other"
    );

    let mut builders: BTreeMap<String, String> = BTreeMap::new();
    for task in ["test-archive-workspace", "test-archive-feature-off"] {
        let builds: Vec<&Lane> = lanes.iter().filter(|lane| lane.task == task).collect();
        let [build] = builds.as_slice() else {
            panic!(
                "`{task}` runs {} times across ci.yml, not once: {builds:?}",
                builds.len()
            )
        };
        let builder = &ci["jobs"][build.job.as_str()];
        let (upload_at, upload) = step_using(builder, "actions/upload-artifact")
            .unwrap_or_else(|| panic!("`{}` uploads no archive", build.job));
        assert!(
            upload_at > build.step,
            "`{}` uploads before it builds",
            build.job
        );
        assert_eq!(text(&upload["with"]["path"]), directory);
        assert_eq!(text(&upload["with"]["if-no-files-found"]), "error");
        let artifact = text(&upload["with"]["name"]).to_owned();
        assert!(!artifact.is_empty(), "`{}` names no artifact", build.job);
        assert!(
            !builders.values().any(|job| job == &build.job),
            "`{}` builds both archives one after the other",
            build.job
        );
        assert!(
            builders
                .insert(artifact.clone(), build.job.clone())
                .is_none(),
            "both archive jobs upload the artifact `{artifact}`"
        );
    }
    for job in builders.values() {
        let waits_for: Vec<&String> = builders
            .values()
            .filter(|other| needs_of(&ci["jobs"][job.as_str()]).contains(*other))
            .collect();
        assert!(
            waits_for.is_empty(),
            "`{job}` waits for {waits_for:?}, so the archives do not build side by side"
        );
    }

    let runners: BTreeSet<&String> = lanes
        .iter()
        .filter(|lane| lane.task == "test-shard" || lane.task == "test-feature-off")
        .map(|lane| &lane.job)
        .collect();
    assert!(!runners.is_empty(), "no job runs the test archives");
    for runner in runners {
        assert!(
            !builders.values().any(|job| job == runner),
            "an archive job also runs a shard"
        );
        let job = &ci["jobs"][runner.as_str()];
        let own: Vec<&Lane> = lanes.iter().filter(|lane| &lane.job == runner).collect();
        let first = own.iter().map(|lane| lane.step).min().unwrap();
        for (artifact, builder) in &builders {
            assert!(
                needs_of(job).contains(builder),
                "`{runner}` does not wait for `{builder}`"
            );
            let download = job["steps"]
                .as_sequence()
                .into_iter()
                .flatten()
                .take(first)
                .find(|step| {
                    text(&step["uses"]).starts_with("actions/download-artifact@")
                        && text(&step["with"]["name"]) == artifact
                })
                .unwrap_or_else(|| {
                    panic!("`{runner}` does not download `{artifact}` before its first task")
                });
            assert_eq!(text(&download["with"]["path"]), directory);
        }
        for lane in own {
            assert!(
                lane.task == "test-shard" || lane.task == "test-feature-off",
                "`{runner}` also runs `{}`, which compiles",
                lane.task
            );
        }
    }
}

/// Every archive a shard downloads, with the step id of its first attempt and the task that runs
/// it.
const ARCHIVE_DOWNLOADS: [(&str, &str, &str); 2] = [
    ("nextest-archive-workspace", "archives", "test-shard"),
    (
        "nextest-archive-feature-off",
        "feature-off-archive",
        "test-feature-off",
    ),
];

/// Attempts per archive download, and the minutes each attempt may take.
const ARCHIVE_ATTEMPTS: usize = 3;
const ARCHIVE_ATTEMPT_MINUTES: u64 = 40;

/// The step ids of one archive's download attempts, in order: `archives`, `archives-retry`,
/// `archives-retry-2`.
fn archive_attempt_ids(first: &str) -> Vec<String> {
    (0..ARCHIVE_ATTEMPTS)
        .map(|attempt| match attempt {
            0 => first.to_owned(),
            1 => format!("{first}-retry"),
            n => format!("{first}-retry-{n}"),
        })
        .collect()
}

fn assert_archive_downloads_survive_slow_throughput(ci: &Value) {
    let job = &ci["jobs"]["test"];
    assert!(
        job["continue-on-error"].is_null(),
        "a shard that never got its archive would not fail the job"
    );
    let steps: Vec<&Value> = job["steps"].as_sequence().into_iter().flatten().collect();
    for (artifact, first, task) in ARCHIVE_DOWNLOADS {
        let attempts: Vec<&Value> = steps
            .iter()
            .copied()
            .filter(|step| {
                text(&step["uses"]).starts_with("actions/download-artifact@")
                    && text(&step["with"]["name"]) == artifact
            })
            .collect();
        let ids = archive_attempt_ids(first);
        assert_eq!(
            attempts
                .iter()
                .map(|step| text(&step["id"]))
                .collect::<Vec<_>>(),
            ids,
            "`{artifact}` is not downloaded in {ARCHIVE_ATTEMPTS} attempts"
        );
        for (index, attempt) in attempts.iter().enumerate() {
            let id = &ids[index];
            assert_eq!(
                attempt["timeout-minutes"].as_u64(),
                Some(ARCHIVE_ATTEMPT_MINUTES),
                "`{id}` is not bounded at {ARCHIVE_ATTEMPT_MINUTES} minutes"
            );
            // Only the last attempt's failure fails the shard; the ones before it are retried.
            let last = index + 1 == ARCHIVE_ATTEMPTS;
            if last {
                assert!(
                    attempt["continue-on-error"].is_null(),
                    "`{id}` is the last attempt and its failure is ignored, so a shard \
                     without its archive skips its tests and passes"
                );
            } else {
                assert_eq!(
                    attempt["continue-on-error"].as_bool(),
                    Some(true),
                    "`{id}` is not the last attempt and its failure fails the shard"
                );
            }
            let guard = if index == 0 {
                "${{ !cancelled() }}".to_owned()
            } else {
                let earlier: Vec<String> = ids[..index]
                    .iter()
                    .map(|earlier| format!("steps.{earlier}.outcome != 'success'"))
                    .collect();
                format!("${{{{ !cancelled() && {} }}}}", earlier.join(" && "))
            };
            assert!(
                text(&attempt["if"]) == guard || (index == 0 && attempt["if"].is_null()),
                "`{id}` does not run exactly when every earlier attempt failed: {:?}",
                attempt["if"]
            );
        }
        let any = ids
            .iter()
            .map(|id| format!("steps.{id}.outcome == 'success'"))
            .collect::<Vec<_>>()
            .join(" || ");
        let runs: Vec<&Value> = steps
            .iter()
            .copied()
            .filter(|step| {
                text(&step["run"])
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .windows(2)
                    .any(|words| words == ["task", task])
            })
            .collect();
        let [run] = runs.as_slice() else {
            panic!("`test` runs `{task}` {} times, not once", runs.len())
        };
        assert_eq!(
            text(&run["if"]),
            format!("${{{{ !cancelled() && steps.go-task.outcome == 'success' && ({any}) }}}}"),
            "`{task}` does not run after any successful attempt of `{artifact}`, or runs without \
             go-task"
        );
        assert!(run["continue-on-error"].is_null());
    }
}

/// An archive download that is slow but progressing must not fail a shard whose tests pass. On
/// release run 38588707717 and main run 38599096240 (2026-10-14) the 8985 MiB workspace and
/// 5662 MiB feature-off archives ran into the 12-minute timeout of both attempts on 13 and 5 of 20
/// shards, while shards that got them ran green. Each archive then got three 30-minute attempts;
/// each runs only when every earlier one failed, the first two may fail and the last may not, so a
/// shard that never got its archive still fails rather than silently skipping its tests.
///
/// By the 0.85.0 queue run 41635526432 (2026-11-03) the workspace archive was 13,916 MiB, and one
/// runner fetched at 6.2 to 7.3 MB/s: shard 2's 9,421 MiB feature-off archive took 26.7 and
/// 22.6 minutes in the run's two attempts, steady, so its workspace archive needed 33 to 39 and
/// timed out in all three 30-minute attempts both times, while the same shard's tests then passed
/// 348 of 348. A slow download restarts from nothing, so an attempt has to hold the whole archive
/// at the slowest rate seen: 40 minutes, with the stripped archives
/// (`the_archive_jobs_build_the_test_binaries_without_debug_information`) about a quarter smaller.
#[test]
fn each_nextest_archive_gets_three_forty_minute_attempts_and_a_missing_one_fails_the_shard() {
    let ci = yaml(".github/workflows/ci.yml");
    assert_archive_downloads_survive_slow_throughput(&ci);

    let mutate = |change: &dyn Fn(&mut Value)| {
        let mut mutated = ci.clone();
        change(&mut mutated);
        let caught =
            std::panic::catch_unwind(|| assert_archive_downloads_survive_slow_throughput(&mutated));
        assert!(caught.is_err(), "a mutation of ci.yml went unnoticed");
    };
    let step_index = |ci: &Value, id: &str| {
        ci["jobs"]["test"]["steps"]
            .as_sequence()
            .unwrap()
            .iter()
            .position(|step| text(&step["id"]) == id)
            .unwrap()
    };
    // The last attempt's failure ignored: a shard without its archive skips its tests.
    mutate(&|ci: &mut Value| {
        let index = step_index(ci, "archives-retry-2");
        ci["jobs"]["test"]["steps"][index]["continue-on-error"] = Value::Bool(true);
    });
    // A third attempt dropped.
    mutate(&|ci: &mut Value| {
        let index = step_index(ci, "feature-off-archive-retry-2");
        ci["jobs"]["test"]["steps"]
            .as_sequence_mut()
            .unwrap()
            .remove(index);
    });
    // A tests step that ignores the last attempt's success.
    mutate(&|ci: &mut Value| {
        let index = step_index(ci, "archives-retry-2") + 1;
        let steps = ci["jobs"]["test"]["steps"].as_sequence_mut().unwrap();
        let run = steps
            .iter_mut()
            .skip(index)
            .find(|step| text(&step["run"]).contains("task test-shard"))
            .unwrap();
        run["if"] = Value::String(
            "${{ !cancelled() && steps.go-task.outcome == 'success' && \
             (steps.archives.outcome == 'success' || \
             steps.archives-retry.outcome == 'success') }}"
                .to_owned(),
        );
    });
    // A retry that runs after an earlier attempt already succeeded.
    mutate(&|ci: &mut Value| {
        let index = step_index(ci, "archives-retry-2");
        ci["jobs"]["test"]["steps"][index]["if"] = Value::String(
            "${{ !cancelled() && steps.archives-retry.outcome != 'success' }}".to_owned(),
        );
    });
}

fn assert_transport_network_controls(ci: &Value, taskfile: &Value) {
    let job = &ci["jobs"]["transport-network"];
    assert!(
        job["if"].is_null(),
        "required transport execution cannot be conditional"
    );
    assert!(
        job["continue-on-error"].is_null(),
        "transport failure cannot be ignored"
    );
    assert_eq!(text(&job["env"]["ESS_TRANSPORT_PREFETCH_DEPS"]), "1");
    let image = text(&job["env"]["ESS_TEST_NATS_IMAGE"]);
    let digest = image
        .strip_prefix("nats@sha256:")
        .expect("NATS image is digest pinned");
    assert!(digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_ne!(
        text(&job["env"]["ESS_ASYNCAPI_CLI"]).len(),
        0,
        "the network job pins the AsyncAPI CLI"
    );
    let commands = shell_commands(taskfile, "test-transport-network");
    assert_eq!(commands.len(), 2, "both external controls are required");
    let steps = job["steps"].as_sequence().expect("transport job has steps");
    for (command, required_test) in commands.iter().zip([
        "the_official_asyncapi_cli_accepts_the_parameterized_document",
        "both_generated_adapters_publish_parameterized_batches_to_actual_nats",
    ]) {
        assert!(command.split_whitespace().any(|word| word == required_test));
        assert!(command.contains("-- --ignored --exact --nocapture"));
        let matching: Vec<_> = steps
            .iter()
            .filter(|step| text(&step["run"]) == command)
            .collect();
        let [step] = matching.as_slice() else {
            panic!("required transport control must execute exactly once: {required_test}");
        };
        assert!(step["if"].is_null() && step["continue-on-error"].is_null());
    }
}

#[test]
fn transport_ci_executes_both_ignored_external_controls_without_skipping() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    assert_transport_network_controls(&ci, &taskfile);
    let mut missing = ci.clone();
    missing["jobs"]["transport-network"]["steps"] = Value::Sequence(Vec::new());
    assert!(
        std::panic::catch_unwind(|| assert_transport_network_controls(&missing, &taskfile))
            .is_err()
    );
    let mut skipped = ci.clone();
    skipped["jobs"]["transport-network"]["if"] = Value::Bool(false);
    assert!(
        std::panic::catch_unwind(|| assert_transport_network_controls(&skipped, &taskfile))
            .is_err()
    );
    let mut ignored = ci;
    ignored["jobs"]["transport-network"]["continue-on-error"] = Value::Bool(true);
    assert!(
        std::panic::catch_unwind(|| assert_transport_network_controls(&ignored, &taskfile))
            .is_err()
    );
}

#[test]
fn the_gate_check_aggregates_every_lane_and_cannot_be_skipped() {
    let ci = yaml(".github/workflows/ci.yml");
    let gate = &ci["jobs"]["gate"];
    // The required status check and the release reuse query both name it by this string.
    assert_eq!(text(&gate["name"]), "Gate");
    // A skipped required check reads as passing, so the aggregate runs whatever its lanes did.
    assert!(text(&gate["if"]).contains("always()"), "{:?}", gate["if"]);
    assert_eq!(gate["timeout-minutes"].as_u64(), Some(5));

    let needs = needs_of(gate);
    assert_eq!(
        needs,
        gated_jobs(&ci),
        "Gate does not carry every gated job's result"
    );

    let checked: String = gate["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .map(|step| ess_yaml::to_string(step).unwrap())
        .collect();
    for need in &needs {
        assert!(
            checked.contains(&format!("needs.{need}.result")),
            "Gate never reads the result of `{need}`"
        );
    }

    // A test shard downloads its archives before its first test: on main's 0.53.0 run one
    // download took 18m42s against a 94 s median, and on run 37615777010 (2026-10-07) the
    // downloads of shards 19 and 20 took 1,512 s and 1,730 s, the first reaching its 12-minute
    // timeout and retrying. The shard's own tests take up to 23 minutes, so 45 minutes left the
    // retry no room and cancelled two shards near their last test; 70 covers about 29 minutes of
    // downloads, 23 of tests and the setup. On 2026-10-14 downloads ran past two 12-minute
    // attempts on most shards, so each archive got three attempts, now of 40 minutes each (see
    // `each_nextest_archive_gets_three_forty_minute_attempts_and_a_missing_one_fails_the_shard`),
    // and the job holds every attempt of both archives plus 45 minutes for the setup and the
    // tests. Every other job compiles or checks and stays at 30.
    let downloads: u64 = ci["jobs"]["test"]["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .filter(|step| text(&step["uses"]).starts_with("actions/download-artifact@"))
        .map(|step| step["timeout-minutes"].as_u64().unwrap_or(360))
        .sum();
    assert_eq!(
        downloads,
        2 * ARCHIVE_ATTEMPTS as u64 * ARCHIVE_ATTEMPT_MINUTES,
        "the shards' downloads are not {ARCHIVE_ATTEMPTS} {ARCHIVE_ATTEMPT_MINUTES}-minute attempts per archive"
    );
    for id in gated_jobs(&ci) {
        let bound = if id == "test" { downloads + 45 } else { 30 };
        assert_eq!(
            ci["jobs"][id.as_str()]["timeout-minutes"].as_u64(),
            Some(bound),
            "`{id}` is not bounded at {bound} minutes"
        );
    }
}

#[test]
fn nextest_and_every_action_are_pinned_by_commit() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");

    // Every job whose tasks call nextest installs it, at one exact version.
    let mut versions = BTreeSet::new();
    let jobs: BTreeSet<String> = lane_invocations(&ci)
        .into_iter()
        .filter(|lane| {
            let mut reach = BTreeSet::new();
            reached(&taskfile, &lane.task, &mut reach);
            reach.iter().any(|task| {
                shell_commands(&taskfile, task)
                    .iter()
                    .any(|command| command.contains("cargo nextest"))
            })
        })
        .map(|lane| lane.job)
        .collect();
    assert!(!jobs.is_empty(), "no job of ci.yml runs nextest");
    for id in &jobs {
        let (_, install) = step_using(&ci["jobs"][id.as_str()], "taiki-e/install-action")
            .unwrap_or_else(|| panic!("`{id}` runs nextest and does not install it"));
        let tool = text(&install["with"]["tool"]);
        let version = tool.strip_prefix("cargo-nextest@").unwrap_or_else(|| {
            panic!("`{id}`'s install step names `{tool}`, not an exact cargo-nextest")
        });
        assert!(
            version.split('.').count() == 3
                && version.split('.').all(|part| part.parse::<u32>().is_ok()),
            "cargo-nextest is not pinned to an exact version in `{id}`: `{tool}`"
        );
        versions.insert(version.to_owned());
    }
    assert_eq!(
        versions.len(),
        1,
        "the archive is built and run by different nextest versions: {versions:?}"
    );

    // The class, not the instance: every `uses:` in every workflow and action is a local path or
    // a full commit.
    let root = workspace_root();
    let mut files: Vec<PathBuf> = fs::read_dir(root.join(".github/workflows"))
        .expect("the workflow directory is readable")
        .map(|entry| entry.unwrap().path())
        .collect();
    for action in fs::read_dir(root.join(".github/actions"))
        .expect("actions")
        .flatten()
    {
        files.push(action.path().join("action.yml"));
    }
    let mut unpinned = Vec::new();
    for file in files.iter().filter(|file| file.is_file()) {
        for (index, line) in fs::read_to_string(file).unwrap().lines().enumerate() {
            let Some(target) = line
                .trim_start()
                .trim_start_matches("- ")
                .strip_prefix("uses:")
            else {
                continue;
            };
            let target = target.trim();
            if target.starts_with("./") {
                continue;
            }
            let pinned = target.split_once('@').is_some_and(|(_, reference)| {
                let sha = reference.split_whitespace().next().unwrap_or("");
                sha.len() == 40 && sha.bytes().all(|byte| byte.is_ascii_hexdigit())
            });
            if !pinned {
                unpinned.push(format!("{}:{}", file.display(), index + 1));
            }
        }
    }
    assert!(
        unpinned.is_empty(),
        "actions not pinned by commit: {unpinned:?}"
    );
}

/// The prefixes of `binary(/^(a|b|…)/)` in a nextest filterset.
fn binary_prefixes(filterset: &str) -> BTreeSet<String> {
    let start = filterset
        .find("binary(/^(")
        .unwrap_or_else(|| panic!("`{filterset}` selects no binaries by name prefix"));
    let rest = &filterset[start + "binary(/^(".len()..];
    let end = rest
        .find(")/)")
        .unwrap_or_else(|| panic!("`{filterset}` does not close its binary prefix group"));
    rest[..end].split('|').map(str::to_owned).collect()
}

/// A pull request from an ordinary head, and a merge-group run, run the feature-off build — the
/// packages without `arbitrary_precision` — on the code whose number semantics the feature
/// changes, and every other run of ci.yml (the `main` push, the nightly schedule, the release gate,
/// and the pull request of a `queue/` head, which the release reuses in place of its gate) runs
/// all of it. Running every ess-cli case twice was about 1,900 of the 3,935 test-seconds of one
/// pull-request run. A queue head runs once, as its pull request: a `queue/**` push run beside it
/// put two 27-job runs of one commit on a 20-job plan (2026-10-19, runs 39285366577 and
/// 39285374142), and shards waited 1,500–3,900 s.
///
/// The narrowing is a nextest archive filterset, so it is data a case can read: it must keep
/// every package other than ess-cli, ess-cli's own unit tests, and every ess-cli test binary whose
/// source names `arbitrary_precision`. That last class is found from the source, not listed.
#[test]
fn pull_requests_run_feature_off_on_number_semantics_and_every_other_run_runs_all_of_it() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");

    let on = &ci["on"];
    assert!(
        on.get("pull_request").is_some(),
        "ci.yml no longer runs on pull requests"
    );
    assert_eq!(
        on["merge_group"]["types"],
        ess_yaml::from_str::<Value>("[checks_requested]").unwrap(),
        "ci.yml does not run in the merge queue, so the queue can never see a Gate"
    );
    assert_eq!(
        on["push"]["branches"],
        ess_yaml::from_str::<Value>("[main]").unwrap(),
        "ci.yml runs on a push besides main's; a queue head runs once, as its pull request"
    );
    let crons: Vec<&str> = on["schedule"]
        .as_sequence()
        .into_iter()
        .flatten()
        .map(|entry| text(&entry["cron"]))
        .collect();
    assert!(
        crons.len() == 1 && !crons[0].is_empty(),
        "ci.yml has no nightly schedule to run the full feature-off build: {crons:?}"
    );

    let builder = lane_invocations(&ci)
        .into_iter()
        .find(|lane| lane.task == "test-archive-feature-off")
        .expect("a job builds the feature-off archive")
        .job;
    assert_eq!(
        text(&ci["jobs"][builder.as_str()]["env"]["FEATURE_OFF"]),
        "${{ inputs.checkout-ref == '' && (github.event_name == 'merge_group' || (github.event_name == 'pull_request' && !(startsWith(github.head_ref, 'queue/') && github.event.pull_request.head.repo.full_name == github.repository))) && 'number-semantics' || 'full' }}",
        "only a pull request from an ordinary head or a merge-group run may narrow the feature-off \
         archive; a queue head's pull request must not, because the release reuses it"
    );

    let archive = shell_commands(&taskfile, "test-archive-feature-off").join("\n");
    assert!(
        archive.contains(
            "cargo nextest archive --locked {{.FEATURE_OFF_PACKAGES}} \
             {{if eq .FEATURE_OFF \"number-semantics\"}}-E '{{.FEATURE_OFF_NUMBER_SEMANTICS}}'{{end}} \
             --archive-file {{.NEXTEST_ARCHIVES}}/feature-off.tar.zst"
        ),
        "{archive}"
    );
    // Local `task test` keeps the full feature-off run: ess-cli is still in the selection.
    let packages = text(&taskfile["vars"]["FEATURE_OFF_PACKAGES"]);
    assert!(passes(packages, "--package", "ess-cli"), "{packages}");

    let filterset = text(&taskfile["vars"]["FEATURE_OFF_NUMBER_SEMANTICS"]);
    for clause in ["not package(ess-cli)", "kind(lib)", "kind(bin)"] {
        assert!(
            filterset.split(" | ").any(|part| part.trim() == clause),
            "the pull-request feature-off filterset drops `{clause}`: {filterset}"
        );
    }
    let prefixes = binary_prefixes(filterset);
    let sensitive: BTreeSet<String> =
        binaries_where(|source| source.contains("arbitrary_precision"))
            .into_iter()
            .filter_map(|binary| binary.strip_prefix("ess-cli::").map(str::to_owned))
            .collect();
    assert!(
        sensitive.contains("execution_recovery"),
        "the scan found {sensitive:?}; it no longer sees the binary it exists to find"
    );
    let dropped: Vec<&String> = sensitive
        .iter()
        .filter(|binary| {
            !prefixes
                .iter()
                .any(|prefix| binary.starts_with(prefix.as_str()))
        })
        .collect();
    assert!(
        dropped.is_empty(),
        "these ess-cli test binaries exercise `arbitrary_precision` and a pull request's \
         feature-off run leaves them out: {dropped:?}"
    );
}

/// go-task is installed from its pinned release archive, checked against its published SHA-256,
/// in every job of the gate and release workflows that runs a task — compiling it with
/// `go install` cost each job about a minute. `pages.yml` is Atlas-classified validation that
/// names its task-toolchain gate as part of what it preserves, so it is left as it is.
#[test]
fn every_job_that_runs_task_installs_the_pinned_release_by_checksum() {
    let mut pins = BTreeSet::new();
    for file in [".github/workflows/ci.yml", ".github/workflows/release.yml"] {
        let workflow = yaml(file);
        let source = fs::read_to_string(workspace_root().join(file)).unwrap();
        assert!(
            !source.contains("go install github.com/go-task/task"),
            "{file} still compiles go-task from source"
        );
        for (id, job) in workflow["jobs"].as_mapping().expect("jobs") {
            let id = text(id);
            let steps: Vec<&Value> = job["steps"].as_sequence().into_iter().flatten().collect();
            let Some(first_task) = steps
                .iter()
                .position(|step| text(&step["run"]).trim_start().starts_with("task "))
            else {
                continue;
            };
            let install = steps[..first_task]
                .iter()
                .find(|step| text(&step["run"]).contains("task_linux_amd64.tar.gz"))
                .unwrap_or_else(|| panic!("{file} `{id}` runs a task before installing go-task"));
            let run = text(&install["run"]);
            assert!(
                run.contains(
                    "https://github.com/go-task/task/releases/download/v$TASK_VERSION/\
                     task_linux_amd64.tar.gz"
                ),
                "{file} `{id}`: {run}"
            );
            assert!(
                run.contains("sha256sum --check --strict"),
                "{file} `{id}` does not check the archive: {run}"
            );
            let version = text(&install["env"]["TASK_VERSION"]);
            let digest = text(&install["env"]["TASK_SHA256"]);
            assert!(
                digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "{file} `{id}` pins no SHA-256: `{digest}`"
            );
            pins.insert((version.to_owned(), digest.to_owned()));
        }
    }
    assert_eq!(
        pins.len(),
        1,
        "the workflows install different go-task releases: {pins:?}"
    );
    let (version, _) = pins.into_iter().next().unwrap();
    assert_eq!(version, "3.52.0");
}

/// The workflows whose jobs install go-task from its release archive.
const GO_TASK_WORKFLOWS: [&str; 2] = [".github/workflows/ci.yml", ".github/workflows/release.yml"];

/// How often, at least, a go-task download is retried before its job fails.
const GO_TASK_RETRIES: u32 = 5;

/// Whether `step` runs go-task: a line of its script starts with `task `.
fn runs_task(step: &Value) -> bool {
    text(&step["run"])
        .lines()
        .any(|line| line.trim_start().starts_with("task "))
}

/// The words of the `curl` command in `script`, its `\` line continuations joined.
fn curl_words(script: &str) -> Vec<String> {
    script
        .replace("\\\n", " ")
        .lines()
        .find(|line| line.trim_start().starts_with("curl "))
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// Whether a step guarded by `condition` still runs once an earlier step has failed its job, with
/// each earlier step's outcome under `steps.<id>.outcome` in `outcomes`. GitHub prefixes a
/// condition that calls no status function with `success()`, so such a step, like one with no
/// condition, is skipped.
fn runs_after_a_failed_step(condition: &Value, outcomes: &BTreeMap<String, &'static str>) -> bool {
    let condition = text(condition).trim();
    let status = ["success()", "failure()", "always()", "cancelled()"];
    if !status.iter().any(|function| condition.contains(function)) {
        return false;
    }
    let condition = condition
        .replace("success()", "''")
        .replace("failure()", "'true'")
        .replace("always()", "'true'")
        .replace("cancelled()", "''");
    let condition = if condition.starts_with("${{") {
        condition
    } else {
        format!("${{{{ {condition} }}}}")
    };
    let context: BTreeMap<&str, &str> = outcomes
        .iter()
        .map(|(name, outcome)| (name.as_str(), *outcome))
        .collect();
    evaluate(&condition, &context).truthy()
}

/// Every job of `workflow` that runs go-task downloads it with `curl --fail`, retried at least
/// [`GO_TASK_RETRIES`] times on any error with curl's own doubling backoff, and an install that
/// still fails stops the job there: the install step is neither conditional nor allowed to fail,
/// and no later step that runs `task` runs once it has failed. Returns how many installs it read.
fn assert_go_task_install_survives_a_download_error(file: &str, workflow: &Value) -> usize {
    let mut installs = 0;
    for (id, job) in workflow["jobs"].as_mapping().expect("jobs") {
        let id = text(id);
        let steps: Vec<&Value> = job["steps"].as_sequence().into_iter().flatten().collect();
        if !steps.iter().any(|step| runs_task(step)) {
            continue;
        }
        let at = steps
            .iter()
            .position(|step| text(&step["run"]).contains("task_linux_amd64.tar.gz"))
            .unwrap_or_else(|| panic!("{file} `{id}` runs go-task and never installs it"));
        let install = steps[at];
        let words = curl_words(text(&install["run"]));
        assert!(
            words.iter().any(|word| word == "--fail"),
            "{file} `{id}`: an HTTP error on the go-task download does not fail curl: {words:?}"
        );
        let retries = words
            .iter()
            .position(|word| word == "--retry")
            .and_then(|at| words.get(at + 1))
            .and_then(|count| count.parse::<u32>().ok());
        assert!(
            retries.is_some_and(|count| count >= GO_TASK_RETRIES),
            "{file} `{id}`: the go-task download is retried {retries:?} times, not at least \
             {GO_TASK_RETRIES}, so one transient error fails the job: {words:?}"
        );
        assert!(
            words.iter().any(|word| word == "--retry-all-errors"),
            "{file} `{id}`: the go-task download is not retried on every error: {words:?}"
        );
        assert!(
            !words.iter().any(|word| word.starts_with("--retry-delay")),
            "{file} `{id}`: a fixed `--retry-delay` replaces curl's doubling backoff: {words:?}"
        );
        assert!(
            install["continue-on-error"].is_null() && install["if"].is_null(),
            "{file} `{id}`: a failed go-task install must fail the job at the install step"
        );
        let mut outcomes: BTreeMap<String, &'static str> = steps
            .iter()
            .filter_map(|step| step["id"].as_str())
            .map(|step| (format!("steps.{step}.outcome"), "skipped"))
            .collect();
        for (index, step) in steps.iter().enumerate().take(at + 1) {
            if let Some(step) = step["id"].as_str() {
                let outcome = if index == at { "failure" } else { "success" };
                outcomes.insert(format!("steps.{step}.outcome"), outcome);
            }
        }
        for step in &steps[at + 1..] {
            if !runs_after_a_failed_step(&step["if"], &outcomes) {
                continue;
            }
            assert!(
                !runs_task(step),
                "{file} `{id}`: `{}` still runs once the go-task install has failed, so the job \
                 fails there as `task: command not found` rather than at the install",
                text(&step["name"])
            );
            if let Some(step) = step["id"].as_str() {
                outcomes.insert(format!("steps.{step}.outcome"), "success");
            }
        }
        installs += 1;
    }
    installs
}

/// The go-task install in `job` of `workflow`.
fn go_task_install<'a>(workflow: &'a mut Value, job: &str) -> &'a mut Value {
    workflow["jobs"][job]["steps"]
        .as_sequence_mut()
        .unwrap()
        .iter_mut()
        .find(|step| text(&step["run"]).contains("task_linux_amd64.tar.gz"))
        .unwrap_or_else(|| panic!("`{job}` installs go-task"))
}

/// A transient error on the go-task release download must not fail a job. In the 0.87.0 queue run
/// 42043226338 the download returned HTTP 504 on two of twenty test shards; `task` was missing, and
/// the shards' test steps, which run under `!cancelled()` so that every archive attempt is read,
/// exited 127 with `task: command not found`. Every install now retries the download on any error
/// with curl's doubling backoff, and a job whose install still fails stops there. `pages.yml`
/// compiles go-task with `go install` and is left as it is
/// (`every_job_that_runs_task_installs_the_pinned_release_by_checksum`).
#[test]
fn a_go_task_install_retries_its_download_and_a_failed_one_stops_the_job_there() {
    for file in GO_TASK_WORKFLOWS {
        let installs = assert_go_task_install_survives_a_download_error(file, &yaml(file));
        assert!(installs > 0, "{file} installs go-task in no job");
    }

    for file in GO_TASK_WORKFLOWS {
        let workflow = yaml(file);
        let jobs: Vec<String> = workflow["jobs"]
            .as_mapping()
            .unwrap()
            .iter()
            .filter(|(_, job)| {
                job["steps"]
                    .as_sequence()
                    .into_iter()
                    .flatten()
                    .any(runs_task)
            })
            .map(|(id, _)| text(id).to_owned())
            .collect();
        let mutate = |job: &str, change: &dyn Fn(&mut Value)| {
            let mut mutated = workflow.clone();
            change(go_task_install(&mut mutated, job));
            let caught = std::panic::catch_unwind(|| {
                assert_go_task_install_survives_a_download_error(file, &mutated)
            });
            assert!(
                caught.is_err(),
                "a mutation of {file} `{job}` went unnoticed"
            );
        };
        let edit = |from: &'static str, to: &'static str| {
            move |install: &mut Value| {
                let run = text(&install["run"]).to_owned();
                assert!(run.contains(from), "the install has no `{from}`: {run}");
                install["run"] = Value::String(run.replace(from, to));
            }
        };
        for job in &jobs {
            // The retry dropped: one 504 fails the job.
            mutate(job, &edit("--retry 5 ", ""));
            // Too few retries.
            mutate(job, &edit("--retry 5 ", "--retry 1 "));
            // Retried only on the errors curl calls transient.
            mutate(job, &edit(" --retry-all-errors", ""));
            // A fixed delay instead of the backoff.
            mutate(
                job,
                &edit("--retry-all-errors", "--retry-all-errors --retry-delay 5"),
            );
            // A failed install that lets the job go on.
            mutate(job, &|install: &mut Value| {
                install["continue-on-error"] = Value::Bool(true);
            });
        }
    }

    // The test steps as they were in run 42043226338: guarded by the archive attempts alone, they
    // run without go-task.
    let ci = yaml(".github/workflows/ci.yml");
    for task in ["task test-shard", "task test-feature-off"] {
        let mut mutated = ci.clone();
        let step = mutated["jobs"]["test"]["steps"]
            .as_sequence_mut()
            .unwrap()
            .iter_mut()
            .find(|step| text(&step["run"]).contains(task))
            .unwrap();
        let guard = text(&step["if"]).replace(" && steps.go-task.outcome == 'success'", "");
        assert_ne!(guard, text(&step["if"]), "`{task}` is guarded on go-task");
        step["if"] = Value::String(guard);
        let caught = std::panic::catch_unwind(|| {
            assert_go_task_install_survives_a_download_error("ci.yml", &mutated)
        });
        assert!(
            caught.is_err(),
            "`{task}` runs after a failed go-task install unnoticed"
        );
    }
}

/// The test binaries carry line tables, not full debug information: the archive every shard
/// downloads is built from them, and its compile and link time scale with it. Hot hashing and
/// archive crates are optimised even in debug builds, because the tests spend their time there.
/// The Linux gate links with lld from the workflow environment, so the WASM target, the local
/// `task check` and the shipped release binaries keep the linker they had.
#[test]
fn test_builds_are_cheap_to_compile_link_and_run() {
    let manifest: toml_lines::Manifest =
        toml_lines::parse(&fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap());
    assert_eq!(
        manifest.get("profile.dev", "debug").as_deref(),
        Some("\"line-tables-only\""),
        "the dev profile, which the test profile inherits, carries full debug information"
    );
    let lock = fs::read_to_string(workspace_root().join("Cargo.lock")).unwrap();
    for package in ["sha2", "flate2", "tar"] {
        if !lock.contains(&format!("name = \"{package}\"\n")) {
            continue;
        }
        assert_eq!(
            manifest
                .get(&format!("profile.dev.package.{package}"), "opt-level")
                .as_deref(),
            Some("3"),
            "`{package}` is in the lockfile and is not optimised in test builds"
        );
    }

    let ci = yaml(".github/workflows/ci.yml");
    assert_eq!(
        text(&ci["env"]["CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS"]),
        "-C link-arg=-fuse-ld=lld",
        "the Linux gate does not link with lld"
    );
    let config = fs::read_to_string(workspace_root().join(".cargo/config.toml")).unwrap();
    assert!(
        !config.contains("fuse-ld"),
        "`.cargo/config.toml` changes the linker for local builds and release binaries too"
    );
}

/// The CI archive jobs compile the test binaries with no debug information at all, which Cargo
/// then strips, standard library included. Even line tables were about half of every binary the
/// shards download: the 0.34.0 queue run's archives (run 36239205801, artifact 10904603287) held
/// 16.7 GB of workspace and 7.1 GB of feature-off binaries in 4.15 GB of zstd, and
/// `objcopy --strip-debug` took a sample of three of them from 449 MB to 244 MB, and from 77 MB to
/// 42 MB compressed. The shards spent 0:35 to 4:12 downloading that artifact and 1:00 extracting
/// it. A failing test still names its file and line; only a backtrace loses its line numbers.
/// Local builds keep the manifest's line tables, held by `test_builds_are_cheap_to_compile_link_and_run`.
///
/// They strip the symbol tables too. By 0.85.0 the workspace archive was 13,916 MiB of zstd and
/// shard downloads ran past 30 minutes (run 41635526432): twelve `debug = 0` test binaries of
/// `ess-domain` went from 271 MB to 157 MB under `strip --strip-all`, and from 50.7 MB to 39.3 MB
/// at zstd level 3, 23 % less to download. A panic still names its file and line; a backtrace
/// loses its function names.
#[test]
fn the_archive_jobs_build_the_test_binaries_without_debug_information() {
    let ci = yaml(".github/workflows/ci.yml");
    let archivers: BTreeSet<String> = lane_invocations(&ci)
        .into_iter()
        .filter(|lane| lane.task.starts_with("test-archive"))
        .map(|lane| lane.job)
        .collect();
    assert!(
        !archivers.is_empty(),
        "no job of ci.yml builds a test archive"
    );
    // The slow lane compiles its own test binaries and archives none.
    let mut builders = archivers.clone();
    builders.extend(
        lane_invocations(&ci)
            .into_iter()
            .filter(|lane| lane.task == "test-slow")
            .map(|lane| lane.job),
    );
    assert!(
        builders.len() > archivers.len(),
        "no job of ci.yml runs `task test-slow`"
    );
    for id in &builders {
        let env = &ci["jobs"][id.as_str()]["env"];
        for variable in ["CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG"] {
            assert_eq!(
                scalar_or_empty(&env[variable]),
                "0",
                "`{id}` builds test binaries with debug information: `{variable}` is not \"0\""
            );
        }
    }
    for id in &archivers {
        let env = &ci["jobs"][id.as_str()]["env"];
        for variable in ["CARGO_PROFILE_DEV_STRIP", "CARGO_PROFILE_TEST_STRIP"] {
            assert_eq!(
                scalar_or_empty(&env[variable]),
                "symbols",
                "`{id}` archives test binaries with their symbol tables: `{variable}` is not \"symbols\""
            );
        }
    }
    assert!(
        ci["env"]["CARGO_PROFILE_DEV_DEBUG"].is_null()
            && ci["env"]["CARGO_PROFILE_TEST_DEBUG"].is_null(),
        "ci.yml drops debug information for every job, not only the archive builds"
    );
}

fn scalar_or_empty(value: &Value) -> String {
    if value.is_null() {
        String::new()
    } else {
        scalar(value)
    }
}

/// Just enough TOML to read `key = value` lines under `[section]` headers, which is all the
/// profile case needs; the workspace carries no TOML parser as a test dependency.
mod toml_lines {
    use std::collections::BTreeMap;

    pub struct Manifest(BTreeMap<(String, String), String>);

    pub fn parse(source: &str) -> Manifest {
        let mut section = String::new();
        let mut entries = BTreeMap::new();
        for line in source.lines().map(str::trim) {
            if let Some(header) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                header.trim().clone_into(&mut section);
            } else if let Some((key, value)) = line.split_once('=') {
                if !line.starts_with('#') {
                    entries.insert(
                        (section.clone(), key.trim().to_owned()),
                        value.split('#').next().unwrap().trim().to_owned(),
                    );
                }
            }
        }
        Manifest(entries)
    }

    impl Manifest {
        pub fn get(&self, section: &str, key: &str) -> Option<String> {
            self.0.get(&(section.to_owned(), key.to_owned())).cloned()
        }
    }
}

/// A public workflow may inspect GitHub to reuse exact prior archives, but release publication is
/// an organization-bot operation from the trusted coordinator. The preparation run therefore has
/// no write permission and carries no `gh release` mutation that would use `github.token` as
/// `github-actions[bot]`.
#[test]
fn release_preparation_has_no_publication_authority_or_command() {
    let release = yaml(".github/workflows/release.yml");
    let source =
        fs::read_to_string(workspace_root().join(".github/workflows/release.yml")).unwrap();
    let mut write_permissions = Vec::new();
    let mut inspect = |scope: &str, permissions: &Value| {
        if let Some(access) = permissions.as_str() {
            if access == "write-all" {
                write_permissions.push(format!("{scope}: {access}"));
            }
        }
        for (name, access) in permissions.as_mapping().into_iter().flatten() {
            let access = text(access);
            if access == "write" || access == "write-all" {
                write_permissions.push(format!("{scope}.{}: {access}", text(name)));
            }
        }
    };
    inspect("workflow", &release["permissions"]);
    for (id, job) in release["jobs"].as_mapping().into_iter().flatten() {
        inspect(&format!("job.{}", text(id)), &job["permissions"]);
    }
    assert!(
        write_permissions.is_empty(),
        "the public release workflow retains write permissions: {write_permissions:?}"
    );
    let mutations: Vec<&str> = ["gh release create", "gh release edit", "gh release upload"]
        .into_iter()
        .filter(|command| source.contains(command))
        .collect();
    assert!(
        mutations.is_empty(),
        "the public release workflow still publishes with the Actions token: {mutations:?}"
    );
}

/// `publish.yml` is the one place a release is published: after a successful `Release
/// preparation` run, with the organization App's token and never the Actions token, refusing to
/// run without the App's secrets, and reading the release back as the bot's.
#[test]
fn a_successful_release_run_is_published_by_the_bot_and_only_by_it() {
    let publish = yaml(".github/workflows/publish.yml");
    let run = &publish["on"]["workflow_run"];
    assert_eq!(
        run["workflows"]
            .as_sequence()
            .map(|names| names.iter().map(text).collect::<Vec<_>>()),
        Some(vec!["Release preparation"])
    );
    assert_eq!(text(&publish["permissions"]["contents"]), "read");
    let job = &publish["jobs"]["publish"];
    assert!(
        text(&job["if"]).contains("github.event.workflow_run.conclusion == 'success'"),
        "publication runs after an unsuccessful release run"
    );
    let steps = job["steps"].as_sequence().expect("publish steps");
    let index = |name: &str| {
        steps
            .iter()
            .position(|step| text(&step["name"]) == name)
            .unwrap_or_else(|| panic!("publish.yml has no step `{name}`"))
    };
    let token = index("Bot token");
    assert!(index("Require the bot App secrets") < token);
    assert!(text(&steps[token]["uses"]).starts_with("actions/create-github-app-token@"));
    let create = index("Publish as the bot");
    assert_eq!(
        text(&steps[create]["env"]["GH_TOKEN"]),
        "${{ steps.app-token.outputs.token }}",
        "the release is created with a token besides the bot's"
    );
    assert!(text(&steps[create]["run"]).contains("gh release create"));
    for (position, step) in steps.iter().enumerate() {
        if position != create {
            assert!(
                !text(&step["run"]).contains("gh release create")
                    && !text(&step["run"]).contains("gh release edit"),
                "step {position} of publish.yml writes a release"
            );
        }
    }
    let read_back = text(&steps[index("Read the release back")]["run"]);
    assert!(read_back.contains("b10x-bot[bot]"));
}

/// The trusted coordinator consumes one successful artifact, so its name must bind the resolved
/// tag and commit and its contents must be exactly the checked archives, checksums and notes. The
/// upload action is the same pinned revision already admitted by `package.yml`.
#[test]
fn release_preparation_retains_an_exact_tag_and_commit_artifact() {
    let release = yaml(".github/workflows/release.yml");
    assert_eq!(text(&release["name"]), "Release preparation");
    let steps = release["jobs"]["release"]["steps"]
        .as_sequence()
        .expect("release preparation steps");
    let upload = steps
        .iter()
        .find(|step| {
            text(&step["uses"])
                == "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"
        })
        .expect("release preparation uploads no retained artifact");
    assert_eq!(
        text(&upload["with"]["name"]),
        "ess-release-${{ needs.resolve.outputs.tag }}-${{ needs.resolve.outputs.commit }}"
    );
    let paths = text(&upload["with"]["path"]);
    let actual: BTreeSet<&str> = paths
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .collect();
    assert_eq!(
        actual,
        BTreeSet::from([
            "dist/ess-${{ needs.resolve.outputs.tag }}-*.tar.gz",
            "dist/SHA256SUMS",
            "notes.md",
        ]),
        "the retained release artifact does not contain exactly the verified files"
    );
    assert_eq!(text(&upload["with"]["if-no-files-found"]), "error");
}

/// What a GitHub Actions expression evaluates to, as far as the readers in this suite need.
#[derive(Clone, Debug, PartialEq)]
enum ExpressionValue {
    Text(String),
    Flag(bool),
}

impl ExpressionValue {
    fn truthy(&self) -> bool {
        match self {
            Self::Text(text) => !text.is_empty(),
            Self::Flag(flag) => *flag,
        }
    }
}

/// The tokens of a GitHub Actions expression: operators, parentheses, commas, quoted literals and
/// names (a context path or a function).
fn expression_tokens(source: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut rest = source.trim_start();
    while !rest.is_empty() {
        let length = if let Some(operator) = ["&&", "||", "==", "!="]
            .into_iter()
            .find(|operator| rest.starts_with(operator))
        {
            operator.len()
        } else if rest.starts_with(['(', ')', ',', '!']) {
            1
        } else if let Some(after) = rest.strip_prefix('\'') {
            after
                .find('\'')
                .unwrap_or_else(|| panic!("an unterminated literal at `{rest}`"))
                + 2
        } else {
            rest.find(|character: char| {
                !(character.is_ascii_alphanumeric() || "._-".contains(character))
            })
            .unwrap_or(rest.len())
        };
        assert!(
            length > 0,
            "this reader cannot read the expression at `{rest}`"
        );
        tokens.push(&rest[..length]);
        rest = rest[length..].trim_start();
    }
    tokens
}

/// Reads one GitHub Actions expression by precedence: `||`, then `&&`, then `==` and `!=`, then
/// `!`, then a parenthesised expression, a literal, `startsWith(…, …)` or a context name.
struct ExpressionReader<'a> {
    tokens: Vec<&'a str>,
    at: usize,
    context: &'a BTreeMap<&'a str, &'a str>,
}

impl<'a> ExpressionReader<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.tokens.get(self.at).copied()
    }

    fn take(&mut self) -> &'a str {
        let token = self
            .peek()
            .unwrap_or_else(|| panic!("the expression {:?} ends early", self.tokens));
        self.at += 1;
        token
    }

    fn expect(&mut self, token: &str) {
        let found = self.take();
        assert_eq!(
            found, token,
            "the expression {:?} is malformed",
            self.tokens
        );
    }

    // `||` and `&&` yield one of their operands, not a boolean, as GitHub's do: that is what makes
    // `cond && 'a' || 'b'` the language's conditional.
    fn or(&mut self) -> ExpressionValue {
        let mut value = self.and();
        while self.peek() == Some("||") {
            self.at += 1;
            let right = self.and();
            if !value.truthy() {
                value = right;
            }
        }
        value
    }

    fn and(&mut self) -> ExpressionValue {
        let mut value = self.comparison();
        while self.peek() == Some("&&") {
            self.at += 1;
            let right = self.comparison();
            if value.truthy() {
                value = right;
            }
        }
        value
    }

    fn comparison(&mut self) -> ExpressionValue {
        let left = self.unary();
        match self.peek() {
            Some(operator @ ("==" | "!=")) => {
                self.at += 1;
                let right = self.unary();
                ExpressionValue::Flag((left == right) == (operator == "=="))
            }
            _ => left,
        }
    }

    fn unary(&mut self) -> ExpressionValue {
        if self.peek() == Some("!") {
            self.at += 1;
            ExpressionValue::Flag(!self.unary().truthy())
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ExpressionValue {
        let token = self.take();
        if token == "(" {
            let value = self.or();
            self.expect(")");
            value
        } else if let Some(literal) = token.strip_prefix('\'') {
            ExpressionValue::Text(literal.strip_suffix('\'').unwrap().to_owned())
        } else if token == "always" {
            self.expect("(");
            self.expect(")");
            ExpressionValue::Flag(true)
        } else if token == "startsWith" {
            self.expect("(");
            let whole = self.or();
            self.expect(",");
            let prefix = self.or();
            self.expect(")");
            match (whole, prefix) {
                (ExpressionValue::Text(whole), ExpressionValue::Text(prefix)) => {
                    ExpressionValue::Flag(whole.starts_with(&prefix))
                }
                other => panic!("startsWith over {other:?} is not text"),
            }
        } else {
            let value = self.context.get(token).unwrap_or_else(|| {
                panic!("the expression names `{token}`, which this case gives no value")
            });
            ExpressionValue::Text((*value).to_owned())
        }
    }
}

/// Evaluates `${{ … }}` over `context`. A name the context does not hold panics, so a term added
/// to the expression cannot pass a case unread.
fn evaluate(expression: &str, context: &BTreeMap<&str, &str>) -> ExpressionValue {
    let inner = expression
        .trim()
        .strip_prefix("${{")
        .and_then(|rest| rest.strip_suffix("}}"))
        .unwrap_or_else(|| panic!("`{expression}` is not one expression"));
    let mut reader = ExpressionReader {
        tokens: expression_tokens(inner),
        at: 0,
        context,
    };
    let value = reader.or();
    assert_eq!(
        reader.at,
        reader.tokens.len(),
        "the expression {:?} has tokens after its end",
        reader.tokens
    );
    value
}

/// A release runs its full gate unless a full gate already passed on the tagged tree. The only
/// stand-in is the `prior-gate` step's answer, which is `true` only when every check the ruleset
/// requires passed on the tagged commit, or on the queue head it merged, in this repository's own
/// full run (`a_release_reuses_a_full_gate_on_the_tagged_tree_and_otherwise_runs_it`); a `Gate`
/// from a pull request of an ordinary head or a fork ran the narrowed feature-off build and never
/// counts.
/// `release.yml` calls the reusable gate with nothing but the commit, publication requires that
/// call to have succeeded or that step to have said `true`, and `ci.yml`'s feature-off selection is
/// `full` for every run a release makes or reuses: a tag push, a backfill dispatched from any
/// branch, `main`'s push, and the pull request of a `queue/` head.
#[test]
// Both workflows' halves of one contract, which reads best whole.
#[allow(clippy::too_many_lines)]
fn a_release_runs_its_full_gate_unless_a_full_gate_passed_the_tagged_tree() {
    let ci = yaml(".github/workflows/ci.yml");
    let release = yaml(".github/workflows/release.yml");
    let jobs = &release["jobs"];

    let gate = &jobs["gate"];
    assert_eq!(text(&gate["uses"]), "./.github/workflows/ci.yml");
    assert_eq!(needs_of(gate), BTreeSet::from(["resolve".to_owned()]));
    assert_eq!(
        text(&gate["if"]),
        "${{ needs.resolve.outputs.gate-passed != 'true' && \
         needs.resolve.outputs.scope != 'targeted' }}",
        "the release gate is skipped on something besides the prior-gate step's answer and a \
         targeted hotfix"
    );
    let passed: BTreeMap<String, String> = gate["with"]
        .as_mapping()
        .into_iter()
        .flatten()
        .map(|(name, value)| (text(name).to_owned(), scalar(value)))
        .collect();
    assert_eq!(
        passed,
        BTreeMap::from([(
            "checkout-ref".to_owned(),
            "${{ needs.resolve.outputs.commit }}".to_owned()
        )]),
        "the release passes its gate something besides the tagged commit"
    );
    let declared: BTreeSet<&str> = ci["on"]["workflow_call"]["inputs"]
        .as_mapping()
        .into_iter()
        .flatten()
        .map(|(name, _)| text(name))
        .collect();
    assert_eq!(
        declared,
        BTreeSet::from(["checkout-ref"]),
        "ci.yml takes an input a caller could narrow the gate with"
    );

    let resolve = &jobs["resolve"];
    assert_eq!(
        text(&resolve["outputs"]["gate-passed"]),
        "${{ steps.prior-gate.outputs.passed }}"
    );
    assert_eq!(
        text(&resolve["outputs"]["head"]),
        "${{ steps.prior-gate.outputs.head }}",
        "resolve no longer reports the merged head the prebuilt step is keyed on"
    );
    // Check-runs, their workflow runs and the pull requests a commit closed are read through
    // these scopes; `release_preparation_has_no_publication_authority_or_command` holds them to
    // reads.
    for scope in ["actions", "checks", "contents", "pull-requests"] {
        assert_eq!(
            text(&resolve["permissions"][scope]),
            "read",
            "resolve cannot read {scope}"
        );
    }
    let step = prior_gate_step(&release);
    assert_eq!(text(&step["shell"]), "bash");
    assert_eq!(
        text(&step["env"]["COMMIT"]),
        "${{ steps.release.outputs.commit }}"
    );
    assert_eq!(text(&step["env"]["EVENT"]), "${{ github.event_name }}");
    let script = text(&step["run"]);
    let mut needed = vec![
        "/check-runs?check_name=".to_owned(),
        "&filter=all&per_page=100".to_owned(),
        ".app.slug == \"github-actions\"".to_owned(),
        "\"completed/success\"".to_owned(),
        // A second page of check-runs is never read; it runs the gate instead.
        "total_count > 100".to_owned(),
        // A check-run's own workflow run says whether it is this repository's full run of that
        // commit.
        "/actions/runs/".to_owned(),
        ".head_repository.full_name".to_owned(),
        ".head_sha".to_owned(),
        "pull_request".to_owned(),
        // The pull request a merge commit closed, from this repository into main, and the Git
        // facts about its head.
        "/pulls?per_page=100".to_owned(),
        ".head.repo.full_name == env.REPOSITORY".to_owned(),
        ".base.ref == \"main\"".to_owned(),
        "^{tree}".to_owned(),
        "merge-base --is-ancestor".to_owned(),
        "decide false".to_owned(),
    ];
    // Every check the ruleset requires is read, by its exact name: `Gate`, and the ownership
    // witness of each macOS runner, which a skipped tag would otherwise never run.
    needed.extend(
        required_checks(&ci)
            .iter()
            .map(|name| format!("\"{name}\"")),
    );
    for needed in needed {
        assert!(
            script.contains(&needed),
            "the prior-gate step no longer reads `{needed}`: {script}"
        );
    }
    let publish = text(&jobs["release"]["if"]);
    assert!(
        publish.contains(
            "(needs.gate.result == 'success' || (needs.gate.result == 'skipped' && \
             needs.resolve.outputs.gate-passed == 'true') || (needs.gate.result == 'skipped' && \
             needs.resolve.outputs.scope == 'targeted' && needs.hotfix-checks.result == 'success' \
             && needs.hotfix-tests.result == 'success'))"
        ),
        "publication accepts a skipped gate without the prior-gate step's `true` or a passed \
         targeted hotfix gate: {publish}"
    );
    // A skipped gate is accepted for a targeted hotfix only when both of its jobs passed, and
    // those jobs run exactly when the full gate does not.
    let release_needs = needs_of(&jobs["release"]);
    for job in ["gate", "hotfix-checks", "hotfix-tests"] {
        assert!(
            release_needs.contains(job),
            "the release does not wait for {job}"
        );
    }
    for job in ["hotfix-checks", "hotfix-tests"] {
        assert_eq!(
            text(&jobs[job]["if"]),
            "${{ needs.resolve.outputs.gate-passed != 'true' && \
             needs.resolve.outputs.scope == 'targeted' }}",
            "{job} runs on something besides a targeted hotfix without a prior gate"
        );
    }
    let context =
        |gate: &'static str, scope: &'static str, checks: &'static str, tests: &'static str| {
            BTreeMap::from([
                ("needs.resolve.result", "success"),
                ("needs.gate.result", gate),
                ("needs.resolve.outputs.gate-passed", "false"),
                ("needs.resolve.outputs.scope", scope),
                ("needs.hotfix-checks.result", checks),
                ("needs.hotfix-tests.result", tests),
                ("needs.wasm.result", "success"),
                ("needs.package.result", "success"),
                ("needs.resolve.outputs.prebuilt-run", ""),
            ])
        };
    let cases = [
        (context("success", "full", "skipped", "skipped"), true),
        (context("skipped", "targeted", "success", "success"), true),
        (context("skipped", "targeted", "success", "failure"), false),
        (context("skipped", "targeted", "failure", "success"), false),
        (context("skipped", "full", "success", "success"), false),
        (context("failure", "full", "skipped", "skipped"), false),
    ];
    for (context, expected) in cases {
        assert_eq!(
            evaluate(publish, &context).truthy(),
            expected,
            "the release job decides {context:?} wrongly under `{publish}`"
        );
    }

    let builder = lane_invocations(&ci)
        .into_iter()
        .find(|lane| lane.task == "test-archive-feature-off")
        .expect("a job builds the feature-off archive")
        .job;
    let feature_off = text(&ci["jobs"][builder.as_str()]["env"]["FEATURE_OFF"]);
    let commit = "3ec2deb977bb19c292a611dcf92a3e676352864a";
    let own = "owner/repository";
    // The event, `github.ref`, `github.head_ref` and the head repository (a pull request's, empty
    // elsewhere), `inputs.checkout-ref`, and the feature-off selection expected.
    let runs = [
        // The release, which always passes the tagged commit.
        ("push", "refs/tags/0.60.0", "", "", commit, "full"),
        (
            "workflow_dispatch",
            "refs/heads/main",
            "",
            "",
            commit,
            "full",
        ),
        (
            "workflow_dispatch",
            "refs/heads/queue/pr-119",
            "",
            "",
            commit,
            "full",
        ),
        // The runs of ci.yml's own, which this reader must tell apart from it.
        ("push", "refs/heads/main", "", "", "", "full"),
        ("schedule", "refs/heads/main", "", "", "", "full"),
        // The queue head's pull request, which the release reuses in place of its gate.
        (
            "pull_request",
            "refs/pull/7/merge",
            "queue/pr-119",
            own,
            "",
            "full",
        ),
        // A fork can name its branch anything, and runs the workflow file of its own merge ref.
        (
            "pull_request",
            "refs/pull/8/merge",
            "queue/pr-119",
            "fork/repository",
            "",
            "number-semantics",
        ),
        (
            "pull_request",
            "refs/pull/7/merge",
            "feat/x",
            own,
            "",
            "number-semantics",
        ),
        (
            "merge_group",
            "refs/heads/gh-readonly-queue/main/pr-7",
            "",
            "",
            "",
            "number-semantics",
        ),
    ];
    // The macOS ownership witnesses run for a tag too: only a backfill, dispatched to gate an
    // older commit that may predate their test target, leaves them out.
    let ownership = text(&ci["jobs"]["output-ownership-macos"]["if"]);
    let mut wrong = Vec::new();
    for (event, reference, head_ref, head_repository, checkout, expected) in runs {
        let context = BTreeMap::from([
            ("github.event_name", event),
            ("github.ref", reference),
            ("github.head_ref", head_ref),
            (
                "github.event.pull_request.head.repo.full_name",
                head_repository,
            ),
            ("github.repository", own),
            ("inputs.checkout-ref", checkout),
        ]);
        let selected = evaluate(feature_off, &context);
        if selected != ExpressionValue::Text(expected.to_owned()) {
            wrong.push(format!(
                "{event} on {reference} (checkout-ref `{checkout}`): {selected:?}, expected {expected}"
            ));
        }
        let witnessed = evaluate(ownership, &context).truthy();
        let backfill = event == "workflow_dispatch" && !checkout.is_empty();
        if witnessed == backfill {
            wrong.push(format!(
                "{event} on {reference} (checkout-ref `{checkout}`): output-ownership-macos runs \
                 {witnessed} under `{ownership}`, expected {}",
                !backfill
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "ci.yml runs these wrongly (feature-off `{feature_off}`):\n{}",
        wrong.join("\n")
    );
}

/// A scratch directory under this target's test scratch, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("ci-lanes-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `program` in `directory`, and any Git it runs with no user or system configuration, so no host
/// hook, signing key or default branch reaches the fixture, and with fixed identities and dates,
/// so its commits are reproducible.
fn isolated(program: &str, directory: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .current_dir(directory)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z");
    command
}

/// A `gh` stub this suite installs runs at once, even while other cases fork children.
///
/// `prebuilt` installs a `gh` stub and the step it runs executes it through
/// `PATH`, as threads of one process with every other case here. A child another thread forks
/// while the stub is open for writing inherits that descriptor until it executes, and until then
/// the kernel refuses to execute the stub (`ETXTBSY`). This installs forty and runs each once
/// while three threads spawn.
#[test]
fn a_freshly_installed_gh_stub_runs_while_other_cases_spawn() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("ci-lanes-fresh-stub-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let unlaunched = executable::unlaunched_rounds(40, |round| {
        let stub = root.join(format!("gh-{round}"));
        executable::install_bytes(&stub, b"#!/bin/sh\nexit 0\n").unwrap();
        Command::new(stub)
    });
    let _ = fs::remove_dir_all(&root);
    assert!(
        unlaunched.is_empty(),
        "{} of 40 freshly installed stubs never ran: {unlaunched:#?}",
        unlaunched.len()
    );
}

/// The targets a release ships one archive each for, `ess-<tag>-<target>.tar.gz`: beside each
/// Linux gnu target a static musl one, which runs on a host whose glibc is older than the
/// runner's.
const RELEASE_TARGETS: [&str; 6] = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

/// The `package.yml` artifact of each of [`RELEASE_TARGETS`].
const RELEASE_ARTIFACTS: [&str; 6] = [
    "release-x86_64-unknown-linux-gnu",
    "release-aarch64-unknown-linux-gnu",
    "release-x86_64-unknown-linux-musl",
    "release-aarch64-unknown-linux-musl",
    "release-x86_64-apple-darwin",
    "release-aarch64-apple-darwin",
];

/// Intel macOS is cross-compiled on the Apple Silicon runner — the Intel runner took 16.9 of the
/// 0.32.0 release's 19.2 minutes — and smoke-run there under Rosetta. What ships: one archive per
/// target named `ess-<tag>-<target>.tar.gz`, one `SHA256SUMS` over them.
#[test]
fn the_release_builds_intel_macos_on_apple_silicon_and_names_each_archive_by_its_target() {
    let release = yaml(".github/workflows/release.yml");
    let packaging = yaml(".github/workflows/package.yml");
    let package = &packaging["jobs"]["package"];
    let runners: BTreeMap<String, String> = matrix_rows(package)
        .into_iter()
        .map(|row| (row["target"].clone(), row["runner"].clone()))
        .collect();
    let targets = RELEASE_TARGETS;
    assert_eq!(
        runners.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from(targets),
        "the release packages a different set of targets"
    );
    assert_eq!(runners["aarch64-apple-darwin"], "macos-15");
    assert_eq!(
        runners["x86_64-apple-darwin"], "macos-15",
        "the Intel archive is built on {} rather than cross-compiled on Apple Silicon",
        runners["x86_64-apple-darwin"]
    );

    let steps: Vec<&Value> = package["steps"]
        .as_sequence()
        .expect("package steps")
        .iter()
        .collect();
    let toolchain = steps
        .iter()
        .find(|step| text(&step["uses"]).starts_with("dtolnay/rust-toolchain@"))
        .expect("package installs a toolchain");
    assert_eq!(text(&toolchain["with"]["targets"]), "${{ matrix.target }}");
    let rosetta = steps
        .iter()
        .find(|step| text(&step["run"]).contains("--install-rosetta"))
        .expect("the Intel smoke run has no Rosetta step");
    assert!(
        text(&rosetta["if"]).contains("matrix.target == 'x86_64-apple-darwin'"),
        "{:?}",
        rosetta["if"]
    );
    let build = steps
        .iter()
        .map(|step| text(&step["run"]))
        .find(|run| run.contains("cargo build --locked --release --bin ess"))
        .expect("package builds the binary");
    assert!(build.contains("--target \"$TARGET\""), "{build}");
    // The smoke run proves which architecture ran, not only that something did.
    assert!(build.contains("lipo -archs \"$bin\""), "{build}");
    assert!(
        build.contains("x86_64-apple-darwin) expected=x86_64"),
        "{build}"
    );
    assert!(
        build.contains("aarch64-apple-darwin) expected=arm64"),
        "{build}"
    );
    assert!(build.contains("\"$bin\" --help"), "{build}");
    assert!(build.contains("\"$bin\" --version"), "{build}");
    let archive = steps
        .iter()
        .map(|step| text(&step["run"]))
        .find(|run| run.contains("tar -C dist"))
        .expect("package makes an archive");
    assert!(
        archive.contains("package=\"ess-$TAG-$TARGET\""),
        "{archive}"
    );
    assert!(
        archive.contains("tar -C dist -czf \"dist/$package.tar.gz\" \"$package\""),
        "{archive}"
    );

    let publish = release["jobs"]["release"]["steps"]
        .as_sequence()
        .expect("release steps")
        .iter()
        .map(|step| text(&step["run"]))
        .find(|run| run.contains("SHA256SUMS"))
        .expect("the release writes SHA256SUMS");
    assert!(publish.contains("\"$archive_count\" != 6"), "{publish}");
    assert!(
        publish.contains("-type f -name \"*.tar.gz\""),
        "the release counts only expected-looking names and could retain another archive: {publish}"
    );
    assert!(
        publish.contains("sha256sum *.tar.gz > SHA256SUMS"),
        "{publish}"
    );
    for target in targets {
        assert!(
            publish.contains(target),
            "SHA256SUMS no longer checks {target}"
        );
    }
}

/// Beside each Linux gnu archive a release ships a static musl one, built on the same runner, so
/// the binary runs on a host whose glibc is older than the runner's. The package job proves the
/// binary static (no interpreter, no needed library) and runs it in a bare busybox container; the
/// preparation and the publication check its name and count it, and the publication uploads it.
#[test]
// One contract across three workflows, which reads best whole.
#[allow(clippy::too_many_lines)]
fn the_release_ships_a_static_musl_archive_beside_each_linux_gnu_archive() {
    let packaging = yaml(".github/workflows/package.yml");
    let release = yaml(".github/workflows/release.yml");
    let publication = yaml(".github/workflows/publish.yml");
    let package = &packaging["jobs"]["package"];
    let runners: BTreeMap<String, String> = matrix_rows(package)
        .into_iter()
        .map(|row| (row["target"].clone(), row["runner"].clone()))
        .collect();
    for (gnu, musl) in [
        ("x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"),
        ("aarch64-unknown-linux-gnu", "aarch64-unknown-linux-musl"),
    ] {
        assert_eq!(
            runners.get(musl),
            runners.get(gnu),
            "{musl} is not packaged on the runner that packages {gnu}"
        );
    }

    let steps = package["steps"].as_sequence().expect("package steps");
    let musl_only = "${{ endsWith(matrix.target, '-musl') }}";
    let position = |needle: &str| {
        steps
            .iter()
            .position(|step| text(&step["run"]).contains(needle))
            .unwrap_or_else(|| panic!("package.yml has no step that runs `{needle}`"))
    };
    let tools = position("musl-tools");
    let build = position("cargo build --locked --release --bin ess");
    let proof = position("docker run");
    assert!(
        tools < build && build < proof,
        "package.yml does not install musl-tools, build, then prove the binary static"
    );
    for index in [tools, proof] {
        assert_eq!(text(&steps[index]["if"]), musl_only, "{:?}", steps[index]);
    }
    let tools = text(&steps[tools]["run"]);
    assert!(
        tools.contains("=musl-gcc\" >> \"$GITHUB_ENV\""),
        "the C code a dependency builds is not compiled with musl-gcc: {tools}"
    );
    let proof = text(&steps[proof]["run"]);
    for needle in [
        "file --brief \"$bin\"",
        "*\"statically linked\"*",
        "*\"static-pie linked\"*",
        "readelf --program-headers --wide \"$bin\"",
        "INTERP",
        "readelf --dynamic --wide \"$bin\"",
        "(NEEDED)",
        "--network none",
        "busybox:",
        "/opt/ess/ess --version",
        "\"$printed\" != \"$TAG\"",
    ] {
        assert!(
            proof.contains(needle),
            "the static proof lacks `{needle}`: {proof}"
        );
    }

    let prebuilt = text(&prebuilt_step(&release)["run"]);
    let preparation = release["jobs"]["release"]["steps"]
        .as_sequence()
        .expect("release steps")
        .iter()
        .map(|step| text(&step["run"]))
        .find(|run| run.contains("sha256sum *.tar.gz > SHA256SUMS"))
        .expect("the release writes SHA256SUMS");
    let smoke = release["jobs"]["release"]["steps"]
        .as_sequence()
        .expect("release steps")
        .iter()
        .map(|step| text(&step["run"]))
        .find(|run| run.contains("--version"))
        .expect("the release smoke-runs no archive");
    let published = publication["jobs"]["publish"]["steps"]
        .as_sequence()
        .expect("publish steps");
    let named = |name: &str| {
        let step = published
            .iter()
            .find(|step| text(&step["name"]) == name)
            .unwrap_or_else(|| panic!("publish.yml has no step `{name}`"));
        text(&step["run"])
    };
    let verify = named("Verify the prepared files");
    let create = named("Publish as the bot");
    let read_back = named("Read the release back");
    for target in RELEASE_TARGETS {
        assert!(
            prebuilt.contains(target),
            "a prebuilt run is reused without release-{target}"
        );
        assert!(
            preparation.contains(target),
            "the preparation does not check {target}"
        );
        assert!(
            verify.contains(target),
            "publish.yml does not check {target}"
        );
        assert!(
            create.contains(&format!("\"ess-$TAG-{target}.tar.gz\"")),
            "publish.yml does not upload {target}: {create}"
        );
    }
    assert!(
        preparation.contains("\"$archive_count\" != 6"),
        "{preparation}"
    );
    assert!(
        verify.contains("wc -l | tr -d ' ')\" = 6"),
        "publish.yml does not count six archives: {verify}"
    );
    assert!(
        read_back.contains("\"$assets\" != 7"),
        "publish.yml does not read back six archives and SHA256SUMS: {read_back}"
    );
    for target in ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"] {
        assert!(
            smoke.contains(target),
            "the preparation does not smoke-run the {target} archive: {smoke}"
        );
    }
}

/// The one rule every compile cache follows: saved by a `main` or pull-request run, whose scope a
/// later run can restore, and never by a queue branch or a tag, whose scope nothing restores. The
/// 0.33.0 queue run saved 1.1 GB under `queue/pr-119` and its release saved four `release-*`
/// keys under the tag, while the repository sat at 10.77 GB of its 10 GB cache allowance. A manual
/// dispatch saves nothing either: one from `main` runs at `main`'s ref, and a release backfill so
/// dispatched builds an old tag's source.
const CACHE_SAVE_IF: &str = "${{ github.ref == 'refs/heads/main' && github.event_name != 'workflow_dispatch' || github.event_name == 'pull_request' }}";

#[test]
fn compile_caches_are_saved_only_where_a_later_run_restores_them() {
    let root = workspace_root();
    let mut files: Vec<PathBuf> = fs::read_dir(root.join(".github/workflows"))
        .expect("the workflow directory is readable")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "yml"))
        .collect();
    files.sort();
    let mut seen = 0;
    let mut wrong = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&root).unwrap().display().to_string();
        let workflow = yaml(&relative);
        for (id, job) in workflow["jobs"].as_mapping().into_iter().flatten() {
            for step in job["steps"].as_sequence().into_iter().flatten() {
                if !text(&step["uses"]).starts_with("Swatinem/rust-cache@") {
                    continue;
                }
                seen += 1;
                let save_if = text(&step["with"]["save-if"]);
                if save_if != CACHE_SAVE_IF {
                    wrong.push(format!("{relative} `{}`: save-if `{save_if}`", text(id)));
                }
            }
        }
    }
    assert!(seen > 0, "no workflow restores a compile cache");
    assert!(
        wrong.is_empty(),
        "these compile caches save under a scope no later run restores: {wrong:?}"
    );
}

/// The release archives of a queue commit are built while that commit's required checks run, so
/// the tag's release publishes them instead of compiling them again: 0.33.0's four package jobs
/// took 6:20 of its 7:45 release, all four from a cold cache. A queue run packages only a commit
/// whose workspace version has a dated changelog section and no tag yet; the release publishes a
/// prebuilt run only when `prebuilt` found one for the exact tagged commit, and builds otherwise.
#[test]
// Both workflows' halves of one contract, which reads best whole.
#[allow(clippy::too_many_lines)]
fn a_queue_run_prebuilds_the_archives_the_tag_publishes_and_the_release_builds_them_otherwise() {
    let packaging = yaml(".github/workflows/package.yml");
    let release = yaml(".github/workflows/release.yml");

    let on = &packaging["on"];
    assert_eq!(
        on["push"]["branches"],
        ess_yaml::from_str::<Value>("['queue/**']").unwrap(),
        "package.yml does not run on exactly the queue branches"
    );
    for input in ["ref", "version"] {
        assert!(
            !on["workflow_call"]["inputs"][input].is_null(),
            "the release cannot pass `{input}` to package.yml"
        );
    }
    let candidate = &packaging["jobs"]["candidate"];
    let decide: String = candidate["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .map(|step| text(&step["run"]).to_owned())
        .collect();
    assert!(decide.contains("cargo xtask release verify"), "{decide}");
    assert!(decide.contains("refs/tags/"), "{decide}");
    assert_eq!(
        text(&candidate["outputs"]["version"]),
        "${{ steps.version.outputs.version }}"
    );
    let package = &packaging["jobs"]["package"];
    assert!(needs_of(package).contains("candidate"));
    assert_eq!(
        text(&package["if"]),
        "${{ needs.candidate.outputs.version != '' }}"
    );
    let build = package["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .map(|step| text(&step["run"]))
        .find(|run| run.contains("cargo build --locked --release --bin ess"))
        .expect("package builds the binary");
    assert!(
        build.contains("cargo xtask release verify \"$TAG\""),
        "{build}"
    );
    assert!(
        build.contains("git cat-file -e \"refs/tags/$TAG^{tag}\""),
        "a release build no longer checks the annotated tag: {build}"
    );

    let jobs = &release["jobs"];
    let resolve = &jobs["resolve"];
    assert_eq!(text(&resolve["permissions"]["actions"]), "read");
    assert_eq!(
        text(&resolve["outputs"]["prebuilt-run"]),
        "${{ steps.prebuilt.outputs.run }}"
    );
    let fallback = &jobs["package"];
    assert_eq!(text(&fallback["uses"]), "./.github/workflows/package.yml");
    assert_eq!(
        text(&fallback["if"]),
        "${{ needs.resolve.outputs.prebuilt-run == '' }}"
    );
    assert_eq!(
        text(&fallback["with"]["ref"]),
        "${{ needs.resolve.outputs.commit }}"
    );
    assert_eq!(
        text(&fallback["with"]["version"]),
        "${{ needs.resolve.outputs.tag }}"
    );

    let publish = &jobs["release"];
    assert!(
        text(&publish["if"]).contains(
            "(needs.package.result == 'success' || (needs.package.result == 'skipped' && \
             needs.resolve.outputs.prebuilt-run != ''))"
        ),
        "publication accepts skipped packaging without a prebuilt run: {:?}",
        publish["if"]
    );
    assert_eq!(text(&publish["permissions"]["actions"]), "read");
    let downloads: Vec<&Value> = publish["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .filter(|step| text(&step["uses"]).starts_with("actions/download-artifact@"))
        .collect();
    let [own, prebuilt] = downloads.as_slice() else {
        panic!("publish downloads {} times, not twice", downloads.len())
    };
    assert_eq!(
        text(&own["if"]),
        "${{ needs.resolve.outputs.prebuilt-run == '' }}"
    );
    assert!(own["with"]["run-id"].is_null());
    assert_eq!(
        text(&prebuilt["if"]),
        "${{ needs.resolve.outputs.prebuilt-run != '' }}"
    );
    assert_eq!(
        text(&prebuilt["with"]["run-id"]),
        "${{ needs.resolve.outputs.prebuilt-run }}"
    );
    for download in [own, prebuilt] {
        assert_eq!(text(&download["with"]["pattern"]), "release-*");
    }
    let smoke = publish["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .map(|step| text(&step["run"]))
        .find(|run| run.contains("--version"))
        .expect("publish never runs a published binary");
    assert!(
        smoke.contains("tar -xzf \"dist/ess-$TAG-$target.tar.gz\""),
        "{smoke}"
    );
}

/// The step of release.yml's `resolve` job with `id`.
fn step_with_id<'a>(release: &'a Value, id: &str) -> &'a Value {
    release["jobs"]["resolve"]["steps"]
        .as_sequence()
        .expect("resolve steps")
        .iter()
        .find(|step| text(&step["id"]) == id)
        .unwrap_or_else(|| panic!("resolve has no `{id}` step"))
}

fn prebuilt_step(release: &Value) -> &Value {
    step_with_id(release, "prebuilt")
}

fn prior_gate_step(release: &Value) -> &Value {
    step_with_id(release, "prior-gate")
}

/// Stands in for `gh api <path?query> --jq <filter>`: answers from `<path?query, / as _>.json` in
/// `$GH_STUB_RESPONSES` through the real `jq`, and fails like an HTTP error when there is none.
/// The query is part of the name, so a step that asks with other parameters gets no answer.
const GH_PATH_STUB: &str = r#"#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -ne 4 ] || [ "$1" != api ] || [ "$3" != --jq ]; then
  echo "gh stub: unexpected arguments: $*" >&2
  exit 2
fi
file="$GH_STUB_RESPONSES/${2//\//_}.json"
if [ ! -f "$file" ]; then
  echo "gh: HTTP 404: Not Found (https://api.github.com/$2)" >&2
  exit 1
fi
exec jq -r "$4" "$file"
"#;

/// The API paths, queries included, that the resolve steps ask for.
fn check_runs_path(sha: &str, name: &str) -> String {
    format!(
        "repos/owner/repository/commits/{sha}/check-runs?check_name={}&filter=all&per_page=100",
        name.replace(' ', "%20")
    )
}

fn pulls_path(sha: &str) -> String {
    format!("repos/owner/repository/commits/{sha}/pulls?per_page=100")
}

fn run_path(id: &str) -> String {
    format!("repos/owner/repository/actions/runs/{id}")
}

fn package_runs_path(head: &str) -> String {
    format!(
        "repos/owner/repository/actions/workflows/package.yml/runs\
         ?head_sha={head}&event=push&status=success&per_page=100"
    )
}

fn artifacts_path(id: u64) -> String {
    format!("repos/owner/repository/actions/runs/{id}/artifacts?per_page=100")
}

/// The canned API answers one step run serves: each API path, with its query, and the JSON it
/// returns.
type Answers = Vec<(String, serde_json::Value)>;

/// What one step wrote to `$GITHUB_OUTPUT`, by name, and its log.
struct StepRun {
    outputs: BTreeMap<String, String>,
    log: String,
}

impl StepRun {
    fn output(&self, name: &str) -> &str {
        self.outputs
            .get(name)
            .unwrap_or_else(|| panic!("the step wrote no `{name}` output: {}", self.log))
    }
}

/// Runs one `shell: bash` step of release.yml's `resolve` job the way Actions does. `env` is the
/// step's environment and must name exactly the variables the step declares; `gh` on `PATH` is a
/// stub answering `responses`; the step runs in `directory`, and `case` holds the stub, the
/// answers and the output file. The step must exit 0 and write each output once.
fn run_release_step(
    case: &Path,
    id: &str,
    directory: &Path,
    env: &[(&str, &str)],
    responses: &[(String, serde_json::Value)],
) -> StepRun {
    let release = yaml(".github/workflows/release.yml");
    let step = step_with_id(&release, id);
    assert_eq!(
        text(&step["shell"]),
        "bash",
        "the {id} step is not a bash step"
    );
    let declared: BTreeSet<&str> = step["env"]
        .as_mapping()
        .into_iter()
        .flatten()
        .map(|(name, _)| text(name))
        .collect();
    let supplied: BTreeSet<&str> = env.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        declared, supplied,
        "the {id} step's environment is not what this harness supplies"
    );
    let bin = case.join("bin");
    let answers = case.join("responses");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&answers).unwrap();
    executable::install_bytes(&bin.join("gh"), GH_PATH_STUB.as_bytes()).unwrap();
    for (path, body) in responses {
        fs::write(
            answers.join(format!("{}.json", path.replace('/', "_"))),
            body.to_string(),
        )
        .unwrap();
    }
    let script = case.join(format!("{id}.sh"));
    fs::write(&script, text(&step["run"])).unwrap();
    let output_file = case.join("github-output");
    fs::write(&output_file, "").unwrap();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut command = isolated("bash", directory);
    command
        .args(["--noprofile", "--norc", "-eo", "pipefail"])
        .arg(&script)
        .env("PATH", path)
        .env("GITHUB_OUTPUT", &output_file)
        .env("GH_STUB_RESPONSES", &answers);
    for (name, value) in env {
        command.env(name, value);
    }
    let output = command.output().unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "the {id} step failed the resolve job instead of deciding: {log}"
    );
    let written = fs::read_to_string(&output_file).unwrap();
    let mut outputs = BTreeMap::new();
    for line in written.lines() {
        let (name, value) = line
            .split_once('=')
            .unwrap_or_else(|| panic!("the {id} step wrote `{line}`, not name=value: {log}"));
        assert!(
            outputs.insert(name.to_owned(), value.to_owned()).is_none(),
            "the {id} step wrote `{name}` twice: {log}"
        );
    }
    StepRun { outputs, log }
}

/// Runs the `prebuilt` step with `head` as the merged queue head `prior-gate` reported (empty when
/// it reported none), and returns the `run` value it wrote, then its log.
fn prebuilt(
    root: &Path,
    index: usize,
    head: &str,
    responses: &[(String, serde_json::Value)],
) -> String {
    let release = yaml(".github/workflows/release.yml");
    assert_eq!(
        text(&prebuilt_step(&release)["env"]["HEAD"]),
        "${{ steps.prior-gate.outputs.head }}",
        "the prebuilt step is no longer keyed on the head prior-gate resolved"
    );
    let case = root.join(format!("prebuilt-{index}"));
    fs::create_dir_all(&case).unwrap();
    let run = run_release_step(
        &case,
        "prebuilt",
        &case,
        &[
            ("GH_TOKEN", "fixture"),
            ("REPOSITORY", "owner/repository"),
            ("HEAD", head),
        ],
        responses,
    );
    format!("{}\n{}", run.output("run"), run.log)
}

fn package_run(id: u64, sha: &str, branch: &str, path: &str, started: &str) -> serde_json::Value {
    json!({
        "id": id, "head_sha": sha, "head_branch": branch, "path": path, "event": "push",
        "status": "completed", "conclusion": "success", "run_started_at": started,
    })
}

fn artifacts(names: &[&str], expired: &[&str]) -> serde_json::Value {
    json!({ "artifacts": names
        .iter()
        .map(|name| json!({ "name": name, "expired": expired.contains(name) }))
        .collect::<Vec<_>>() })
}

/// A prebuilt run counts only when `prior-gate` reported a merged queue head with the tagged tree,
/// the run is a successful `package.yml` push run of a queue branch at exactly that head, and it
/// still holds every unexpired archive; of several such runs the newest that holds every one
/// is used. Every other answer — no head, and every API failure — builds the archives in the
/// release. `package.yml` runs on the queue push of the head and never at the merge commit a tag
/// names: keyed on the tagged commit, 0.61.0 and 0.62.0 both built again.
#[test]
// One table of cases, which reads best whole.
#[allow(clippy::too_many_lines)]
fn a_release_publishes_only_a_queue_run_of_the_merged_head_that_holds_every_archive() {
    let root = Scratch::new("release-prebuilt");
    // The merged head `prior-gate` reported; `prebuilt` never sees the tagged commit.
    let head = "3ec2deb977bb19c292a611dcf92a3e676352864a";
    let other = "5090405eeef545e9731ff362911ed5707e4b9293";
    let runs = package_runs_path(head);
    let workflow = ".github/workflows/package.yml";
    let all = || artifacts(&RELEASE_ARTIFACTS, &[]);
    let queued = |id| package_run(id, head, "queue/pr-119", workflow, "2026-01-01T00:00:05Z");
    let older = |id| package_run(id, head, "queue/pr-119", workflow, "2026-01-01T00:00:01Z");
    let listing = |list: Vec<serde_json::Value>| json!({ "workflow_runs": list });
    let artifacts_of = artifacts_path;

    let cases: Vec<(&str, Answers, &str)> = vec![
        (
            "a queue run of the head holds every archive",
            vec![(runs.clone(), listing(vec![queued(7)])), (artifacts_of(7), all())],
            "7",
        ),
        (
            "the newest of two queue runs is the one published",
            vec![
                (runs.clone(), listing(vec![queued(9), older(8)])),
                (artifacts_of(9), all()),
                (artifacts_of(8), all()),
            ],
            "9",
        ),
        (
            "the newest run holds five archives, so the older one that holds every one is published",
            vec![
                (runs.clone(), listing(vec![queued(9), older(8)])),
                (artifacts_of(9), artifacts(&RELEASE_ARTIFACTS[..5], &[])),
                (artifacts_of(8), all()),
            ],
            "8",
        ),
        (
            "the newest run's artifacts are unavailable, so the older one is published",
            vec![
                (runs.clone(), listing(vec![queued(9), older(8)])),
                (artifacts_of(8), all()),
            ],
            "8",
        ),
        (
            "no package run built the head",
            vec![(runs.clone(), listing(vec![]))],
            "",
        ),
        ("the package runs are unavailable", vec![], ""),
        (
            "the only run built another commit",
            vec![
                (
                    runs.clone(),
                    listing(vec![package_run(
                        7,
                        other,
                        "queue/pr-119",
                        workflow,
                        "2026-01-01T00:00:05Z",
                    )]),
                ),
                (artifacts_of(7), all()),
            ],
            "",
        ),
        (
            "the only run was not a queue branch's",
            vec![
                (
                    runs.clone(),
                    listing(vec![package_run(
                        7,
                        head,
                        "feature",
                        workflow,
                        "2026-01-01T00:00:05Z",
                    )]),
                ),
                (artifacts_of(7), all()),
            ],
            "",
        ),
        (
            "the only run came from another workflow file",
            vec![
                (
                    runs.clone(),
                    listing(vec![package_run(
                        7,
                        head,
                        "queue/pr-119",
                        ".github/workflows/ci.yml",
                        "2026-01-01T00:00:05Z",
                    )]),
                ),
                (artifacts_of(7), all()),
            ],
            "",
        ),
        (
            "the run holds every archive but the musl ones",
            vec![
                (runs.clone(), listing(vec![queued(7)])),
                (artifacts_of(7), artifacts(&[RELEASE_ARTIFACTS[0], RELEASE_ARTIFACTS[1], RELEASE_ARTIFACTS[4], RELEASE_ARTIFACTS[5]], &[])),
            ],
            "",
        ),
        (
            "one of the six archives has expired",
            vec![
                (runs.clone(), listing(vec![queued(7)])),
                (artifacts_of(7), artifacts(&RELEASE_ARTIFACTS, &["release-aarch64-unknown-linux-musl"])),
            ],
            "",
        ),
        (
            "the run's artifacts are unavailable",
            vec![(runs.clone(), listing(vec![queued(7)]))],
            "",
        ),
    ];

    let mut wrong = Vec::new();
    for (index, (name, responses, expected)) in cases.into_iter().enumerate() {
        let answer = prebuilt(&root.0, index, head, &responses);
        let (decision, log) = answer.split_once('\n').unwrap();
        if decision != expected {
            wrong.push(format!(
                "{name}: run={decision}, expected {expected:?}\n{log}"
            ));
        }
    }
    // No head reported — a squash, a stale merge, a backfill — builds, whatever package runs exist.
    let answer = prebuilt(
        &root.0,
        usize::MAX,
        "",
        &[
            (runs.clone(), listing(vec![queued(7)])),
            (artifacts_of(7), all()),
        ],
    );
    let (decision, log) = answer.split_once('\n').unwrap();
    if !decision.is_empty() {
        wrong.push(format!(
            "no merged head, and a queue run of some commit: run={decision}, expected \"\"\n{log}"
        ));
    }
    assert!(
        wrong.is_empty(),
        "the prebuilt step decided these wrongly:\n{}",
        wrong.join("\n")
    );
}

fn git(directory: &Path, args: &[&str]) -> String {
    let output = isolated("git", directory).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn commit_file(directory: &Path, name: &str, contents: &str) -> String {
    commit_file_as(directory, name, contents, name)
}

fn commit_file_as(directory: &Path, name: &str, contents: &str, message: &str) -> String {
    fs::write(directory.join(name), contents).unwrap();
    git(directory, &["add", name]);
    git(directory, &["commit", "--quiet", "--message", message]);
    git(directory, &["rev-parse", "HEAD"])
}

fn merge(directory: &Path, branch: &str) -> String {
    git(directory, &["checkout", "--quiet", "main"]);
    git(
        directory,
        &["merge", "--quiet", "--no-ff", "--message", branch, branch],
    );
    let parents = git(
        directory,
        &["rev-list", "--parents", "--max-count=1", "HEAD"],
    );
    assert_eq!(
        parents.split_whitespace().count(),
        3,
        "merging `{branch}` made no merge commit, so the fixture is not the history it names"
    );
    git(directory, &["rev-parse", "HEAD"])
}

/// The histories the reuse rule has to tell apart: each the commit a tag would name, paired with
/// the head of the pull request it merged.
struct History {
    /// `main` merged an up-to-date head: the merge's second parent is the head, its tree is the
    /// head's, and `main` before the merge is in the head's history.
    fresh: (String, String),
    /// `main` moved after the branch left it: the merge carries a change the head never had.
    stale: (String, String),
    /// `main` and the branch made the same change independently: the trees agree, but `main`
    /// before the merge is not in the head's history, so no pull-request run tested this tree.
    twin: (String, String),
    /// A squash of an up-to-date head: one parent and the head's tree, with the head still on its
    /// branch, so only the second-parent rule refuses it.
    squash: (String, String),
    /// A merge commit whose parents are `main` and an up-to-date head but whose tree carries a
    /// file the head never had: the parents hold and only the tree comparison refuses it.
    forged: (String, String),
    origin: PathBuf,
}

fn history(root: &Path) -> History {
    let origin = root.join("origin");
    fs::create_dir_all(&origin).unwrap();
    git(&origin, &["init", "--quiet", "--initial-branch", "main"]);
    commit_file(&origin, "base.txt", "base\n");

    git(&origin, &["checkout", "--quiet", "-b", "fresh"]);
    let fresh_head = commit_file(&origin, "fresh.txt", "fresh\n");
    let fresh = merge(&origin, "fresh");

    git(&origin, &["checkout", "--quiet", "-b", "stale"]);
    let stale_head = commit_file(&origin, "stale.txt", "stale\n");
    git(&origin, &["checkout", "--quiet", "main"]);
    commit_file(&origin, "moved.txt", "moved\n");
    let stale = merge(&origin, "stale");

    git(&origin, &["checkout", "--quiet", "-b", "twin"]);
    let twin_head = commit_file(&origin, "twin.txt", "twin\n");
    git(&origin, &["checkout", "--quiet", "main"]);
    // Another message, or with fixed dates the two commits would be one and nothing would merge.
    commit_file_as(&origin, "twin.txt", "twin\n", "twin.txt on main");
    assert_ne!(git(&origin, &["rev-parse", "HEAD"]), twin_head);
    let twin = merge(&origin, "twin");

    git(&origin, &["checkout", "--quiet", "-b", "squash"]);
    let squash_head = commit_file(&origin, "squash.txt", "squash\n");
    git(&origin, &["checkout", "--quiet", "main"]);
    git(&origin, &["merge", "--quiet", "--squash", "squash"]);
    git(&origin, &["commit", "--quiet", "--message", "squash"]);
    let squash = git(&origin, &["rev-parse", "HEAD"]);
    // The squashed branch stays, as a queue branch does until the bot deletes it, so the release
    // checkout (every branch, `fetch-depth: 0`) holds the head and its tree.
    git(&origin, &["update-ref", "refs/pull/4/head", &squash_head]);

    git(&origin, &["checkout", "--quiet", "-b", "forged"]);
    let forged_head = commit_file(&origin, "forged.txt", "forged\n");
    commit_file(&origin, "smuggled.txt", "smuggled\n");
    let smuggled = git(&origin, &["rev-parse", "HEAD^{tree}"]);
    let forged = git(
        &origin,
        &[
            "commit-tree",
            &smuggled,
            "-p",
            &squash,
            "-p",
            &forged_head,
            "-m",
            "forged",
        ],
    );
    git(&origin, &["checkout", "--quiet", "main"]);
    git(&origin, &["update-ref", "refs/heads/main", &forged]);
    git(&origin, &["reset", "--quiet", "--hard", "main"]);
    git(&origin, &["branch", "--quiet", "-D", "forged"]);

    History {
        fresh: (fresh, fresh_head),
        stale: (stale, stale_head),
        twin: (twin, twin_head),
        squash: (squash, squash_head),
        forged: (forged, forged_head),
        origin,
    }
}

/// The check names the ruleset requires: `Gate`, and the ownership witness of each macOS runner.
fn required_checks(ci: &Value) -> Vec<String> {
    let ownership = &ci["jobs"]["output-ownership-macos"];
    let template = text(&ownership["name"]);
    let mut names = vec![text(&ci["jobs"]["gate"]["name"]).to_owned()];
    for row in matrix_rows(ownership) {
        names.push(template.replace("${{ matrix.runner }}", &row["runner"]));
    }
    assert_eq!(names.len(), 3, "the required checks are {names:?}");
    names
}

/// One check-run as a listing shows it, without its name: status and conclusion, the app that
/// recorded it, the workflow run its details URL names, and when it started.
fn check_run(
    status: &str,
    conclusion: Option<&str>,
    app: &str,
    run: &str,
    started: &str,
) -> serde_json::Value {
    json!({
        "app": { "slug": app }, "status": status, "conclusion": conclusion,
        "started_at": started,
        "details_url": format!("https://github.com/owner/repository/actions/runs/{run}/job/30"),
    })
}

/// A listing of the check-runs named `name` on a commit, as `/check-runs?check_name=` returns
/// it, with a check-run of another name beside them that the name filter must drop; `total`
/// overrides the `total_count` the listing reports.
fn listing(name: &str, runs: &[serde_json::Value], total: Option<u64>) -> serde_json::Value {
    let mut named: Vec<serde_json::Value> = runs
        .iter()
        .map(|run| {
            let mut run = run.clone();
            run["name"] = json!(name);
            run
        })
        .collect();
    named.push(json!({
        "name": "Other", "app": { "slug": "github-actions" }, "status": "completed",
        "conclusion": "success", "started_at": "2026-01-01T00:09:00Z",
        "details_url": "https://github.com/owner/repository/actions/runs/1/job/10",
    }));
    json!({
        "total_count": total.unwrap_or(named.len() as u64),
        "check_runs": named,
    })
}

/// One workflow run as `GET /actions/runs/{id}` describes it.
fn workflow_run(
    path: &str,
    event: &str,
    head_branch: Option<&str>,
    head_repository: &str,
    head_sha: &str,
) -> serde_json::Value {
    json!({
        "path": path, "event": event, "head_branch": head_branch,
        "head_repository": { "full_name": head_repository }, "head_sha": head_sha,
    })
}

fn pull(
    number: u64,
    merged: bool,
    merge_commit: &str,
    head: &str,
    head_repository: &str,
    base: &str,
) -> serde_json::Value {
    json!({
        "number": number,
        "merged_at": if merged { json!("2026-01-01T00:00:00Z") } else { json!(null) },
        "merge_commit_sha": merge_commit,
        "head": { "sha": head, "repo": { "full_name": head_repository } },
        "base": { "ref": base },
    })
}

/// What release.yml's `prior-gate` step wrote: `passed`, `head`, then its log.
struct PriorGate {
    passed: String,
    head: String,
    log: String,
}

/// Runs the `prior-gate` step in a fresh clone of `main` as the resolve job's checkout.
fn prior_gate(
    root: &Path,
    index: usize,
    history: &History,
    commit: &str,
    event: &str,
    responses: &[(String, serde_json::Value)],
) -> PriorGate {
    let release = yaml(".github/workflows/release.yml");
    let step = prior_gate_step(&release);
    assert_eq!(
        text(&step["env"]["COMMIT"]),
        "${{ steps.release.outputs.commit }}",
        "the prior-gate step is no longer keyed on the commit the tag resolved to"
    );
    assert_eq!(
        text(&step["env"]["EVENT"]),
        "${{ github.event_name }}",
        "the prior-gate step no longer knows whether it is a backfill"
    );
    let case = root.join(format!("prior-gate-{index}"));
    fs::create_dir_all(&case).unwrap();
    // Every branch, as `actions/checkout` with `fetch-depth: 0` fetches, and `main` checked out.
    let checkout = case.join("checkout");
    git(
        root,
        &[
            "clone",
            "--quiet",
            "--no-local",
            "--branch",
            "main",
            "--no-tags",
            history.origin.to_str().unwrap(),
            checkout.to_str().unwrap(),
        ],
    );
    let run = run_release_step(
        &case,
        "prior-gate",
        &checkout,
        &[
            ("GH_TOKEN", "fixture"),
            ("REPOSITORY", "owner/repository"),
            ("EVENT", event),
            ("COMMIT", commit),
        ],
        responses,
    );
    PriorGate {
        passed: run.output("passed").to_owned(),
        head: run.output("head").to_owned(),
        log: run.log,
    }
}

/// One history the rule is asked about: its name, the tagged commit, the event, the answers, the
/// `passed` decision, the `head` the prebuilt step is keyed on.
type Case<'a> = (&'a str, &'a str, &'a str, Answers, &'a str, &'a str);

/// The gate counts as passed when every check the ruleset requires — `Gate` and the two ownership
/// witnesses — passed on the tagged tree in a qualifying run. Of a commit's check-runs of one name,
/// newest first, the first whose workflow run qualifies decides, and only a completed success
/// passes; a run qualifies when it is this repository's `ci.yml` at that commit and archived every
/// feature-off case: a push or schedule run of `main`, or a pull-request run of a `queue/` head.
/// That holds on the tagged commit itself, or on the head of exactly one merged pull request of
/// this repository into `main` that is the tagged commit's second parent, has its tree, and
/// already contained the `main` it merged into. Every other answer — a backfill, a squash, a stale
/// or forged merge, a fork, a narrowed run, more than one page of check-runs, no qualifying run
/// among the newest ten, a failure on the way — runs the gate. The head is reported whenever its
/// tree is the tagged tree, so the prebuilt step can be keyed on it.
#[test]
// One table of cases, which reads best whole.
#[allow(clippy::too_many_lines)]
fn a_release_reuses_a_full_gate_on_the_tagged_tree_and_otherwise_runs_it() {
    let root = Scratch::new("release-reuse");
    let history = history(&root.0);
    let required = required_checks(&yaml(".github/workflows/ci.yml"));
    let (fresh, fresh_head) = (history.fresh.0.as_str(), history.fresh.1.as_str());
    let (stale, stale_head) = (history.stale.0.as_str(), history.stale.1.as_str());
    let (twin, twin_head) = (history.twin.0.as_str(), history.twin.1.as_str());
    let (squash, squash_head) = (history.squash.0.as_str(), history.squash.1.as_str());
    let (forged, forged_head) = (history.forged.0.as_str(), history.forged.1.as_str());
    let absent = "0123456789abcdef0123456789abcdef01234567";
    let ci = ".github/workflows/ci.yml";
    let repository = "owner/repository";
    let gate = required[0].as_str();
    let intel = required[2].as_str();
    let green = |run: &str, started: &str| {
        check_run("completed", Some("success"), "github-actions", run, started)
    };
    let newest = "2026-01-01T00:00:05Z";
    let checks = |sha: &str, name: &str, runs: Vec<serde_json::Value>| {
        (check_runs_path(sha, name), listing(name, &runs, None))
    };
    // Every required check green on `sha`, from workflow run `run`.
    let passed = |sha: &str, run: &str| -> Answers {
        required
            .iter()
            .map(|name| checks(sha, name, vec![green(run, newest)]))
            .collect()
    };
    // `passed`, with the check named `name` answered by `runs` instead.
    let passed_but = |sha: &str, run: &str, name: &str, runs: Vec<serde_json::Value>| -> Answers {
        passed(sha, run)
            .into_iter()
            .map(|(path, body)| {
                if path == check_runs_path(sha, name) {
                    (path, listing(name, &runs, None))
                } else {
                    (path, body)
                }
            })
            .collect()
    };
    let pulls = |sha: &str, list: serde_json::Value| (pulls_path(sha), list);
    let run = |id: &str, body: serde_json::Value| (run_path(id), body);
    let merged = |merge_commit: &str, head: &str| {
        json!([pull(1, true, merge_commit, head, repository, "main")])
    };
    let queue_pull =
        |sha: &str| workflow_run(ci, "pull_request", Some("queue/pr-119"), repository, sha);
    let queue_push = |sha: &str| workflow_run(ci, "push", Some("queue/pr-119"), repository, sha);
    let main_push = |sha: &str| workflow_run(ci, "push", Some("main"), repository, sha);
    // The usual rule-2 history: the tagged commit's own check-runs unavailable, the pull request
    // merged, the head's run described.
    let head_run = |body: serde_json::Value| -> Answers {
        vec![pulls(fresh, merged(fresh, fresh_head)), run("50", body)]
    };

    let cases: Vec<Case> =
        vec![
        (
            "the tagged commit passed every required check in main's push run",
            fresh,
            "push",
            [
                passed(fresh, "60"),
                vec![
                    run("60", main_push(fresh)),
                    pulls(fresh, merged(fresh, fresh_head)),
                ],
            ]
            .concat(),
            "true",
            fresh_head,
        ),
        (
            "the tagged commit passed every required check in the nightly run",
            fresh,
            "push",
            [
                passed(fresh, "65"),
                vec![
                    run("65", workflow_run(ci, "schedule", Some("main"), repository, fresh)),
                    pulls(fresh, merged(fresh, fresh_head)),
                ],
            ]
            .concat(),
            "true",
            fresh_head,
        ),
        (
            "an up-to-date merged pull request whose head passed every check in its queue run",
            fresh,
            "push",
            [head_run(queue_pull(fresh_head)), passed(fresh_head, "50")].concat(),
            "true",
            fresh_head,
        ),
        (
            "the tagged commit's check-runs are an empty listing, and the head passed",
            fresh,
            "push",
            [
                vec![checks(fresh, gate, vec![])],
                head_run(queue_pull(fresh_head)),
                passed(fresh_head, "50"),
            ]
            .concat(),
            "true",
            fresh_head,
        ),
        (
            "the head's newest Gate is from its queue push run, an older one from its pull request",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                vec![run("80", queue_push(fresh_head))],
                passed_but(
                    fresh_head,
                    "50",
                    gate,
                    vec![green("80", newest), green("50", "2026-01-01T00:00:01Z")],
                ),
            ]
            .concat(),
            "true",
            fresh_head,
        ),
        (
            "a qualifying green Gate tenth among the head's newest check-runs",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                vec![run("80", queue_push(fresh_head))],
                passed_but(fresh_head, "50", gate, {
                    let mut runs: Vec<serde_json::Value> = (1..=9)
                        .map(|minute| green("80", &format!("2026-01-01T00:{minute:02}:00Z")))
                        .collect();
                    runs.push(green("50", "2026-01-01T00:00:00Z"));
                    runs
                }),
            ]
            .concat(),
            "true",
            fresh_head,
        ),
        (
            "the head's newest qualifying Gate failed, though an older one is green",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                vec![run("51", queue_pull(fresh_head))],
                passed_but(
                    fresh_head,
                    "50",
                    gate,
                    vec![
                        check_run("completed", Some("failure"), "github-actions", "51", newest),
                        green("50", "2026-01-01T00:00:01Z"),
                    ],
                ),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "a qualifying green Gate eleventh among the head's newest check-runs",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                vec![run("80", queue_push(fresh_head))],
                passed_but(fresh_head, "50", gate, {
                    let mut runs: Vec<serde_json::Value> = (1..=10)
                        .map(|minute| green("80", &format!("2026-01-01T00:{minute:02}:00Z")))
                        .collect();
                    runs.push(green("50", "2026-01-01T00:00:00Z"));
                    runs
                }),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but an ownership witness failed",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                passed_but(
                    fresh_head,
                    "50",
                    intel,
                    vec![check_run("completed", Some("failure"), "github-actions", "50", newest)],
                ),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but an ownership witness has no check-run",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                passed_but(fresh_head, "50", intel, vec![]),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but an ownership witness's check-runs are unavailable",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                passed(fresh_head, "50")
                    .into_iter()
                    .filter(|(path, _)| path != &check_runs_path(fresh_head, intel))
                    .collect(),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head has more than one page of Gate check-runs",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                passed(fresh_head, "50")
                    .into_iter()
                    .map(|(path, body)| {
                        if path == check_runs_path(fresh_head, gate) {
                            (path, listing(gate, &[green("50", newest)], Some(101)))
                        } else {
                            (path, body)
                        }
                    })
                    .collect(),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but only from its queue push run, which was narrowed",
            fresh,
            "push",
            [head_run(queue_push(fresh_head)), passed(fresh_head, "50")].concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but from a pull request of an ordinary branch",
            fresh,
            "push",
            [
                head_run(workflow_run(ci, "pull_request", Some("feat/x"), repository, fresh_head)),
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but from a merge-group run",
            fresh,
            "push",
            [
                head_run(workflow_run(
                    ci,
                    "merge_group",
                    Some("gh-readonly-queue/main/pr-7"),
                    repository,
                    fresh_head,
                )),
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but from another workflow",
            fresh,
            "push",
            [
                head_run(workflow_run(
                    ".github/workflows/package.yml",
                    "pull_request",
                    Some("queue/pr-119"),
                    repository,
                    fresh_head,
                )),
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but from a run of a fork",
            fresh,
            "push",
            [
                head_run(workflow_run(
                    ci,
                    "pull_request",
                    Some("queue/pr-119"),
                    "fork/repository",
                    fresh_head,
                )),
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate is green but from a run at another commit",
            fresh,
            "push",
            [head_run(queue_pull(stale_head)), passed(fresh_head, "50")].concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's Gate names no workflow run",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                passed_but(fresh_head, "50", gate, vec![green("", newest)]),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the workflow run of the head's Gate is unavailable",
            fresh,
            "push",
            [
                vec![pulls(fresh, merged(fresh, fresh_head))],
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's check-runs are unavailable",
            fresh,
            "push",
            head_run(queue_pull(fresh_head)),
            "false",
            fresh_head,
        ),
        (
            "the head's newest Gate is still running",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                passed_but(
                    fresh_head,
                    "50",
                    gate,
                    vec![check_run("in_progress", None, "github-actions", "50", newest)],
                ),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the head's green Gate was recorded by another app",
            fresh,
            "push",
            [
                head_run(queue_pull(fresh_head)),
                passed_but(
                    fresh_head,
                    "50",
                    gate,
                    vec![check_run("completed", Some("success"), "impostor", "50", newest)],
                ),
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the tagged commit's own Gate is green but from a pull request of an ordinary branch",
            fresh,
            "push",
            [
                passed(fresh, "70"),
                vec![
                    run("70", workflow_run(ci, "pull_request", Some("feat/x"), repository, fresh)),
                    pulls(fresh, merged(fresh, fresh_head)),
                    checks(fresh_head, gate, vec![]),
                ],
            ]
            .concat(),
            "false",
            fresh_head,
        ),
        (
            "the pull request's head is a fork's",
            fresh,
            "push",
            [
                vec![
                    pulls(
                        fresh,
                        json!([pull(1, true, fresh, fresh_head, "fork/repository", "main")]),
                    ),
                    run("50", queue_pull(fresh_head)),
                ],
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "the pull request targets another branch",
            fresh,
            "push",
            [
                vec![
                    pulls(
                        fresh,
                        json!([pull(1, true, fresh, fresh_head, repository, "release/1")]),
                    ),
                    run("50", queue_pull(fresh_head)),
                ],
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "a backfill, whatever passed",
            fresh,
            "workflow_dispatch",
            [
                passed(fresh, "60"),
                vec![run("60", main_push(fresh))],
                head_run(queue_pull(fresh_head)),
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "a schedule event, whatever passed",
            fresh,
            "schedule",
            [passed(fresh, "60"), vec![run("60", main_push(fresh))]].concat(),
            "false",
            "",
        ),
        (
            "the tagged commit is not in the checkout",
            absent,
            "push",
            [
                passed(absent, "60"),
                vec![run("60", main_push(absent)), pulls(absent, merged(absent, fresh_head))],
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "main moved after the branch left it, so the merge's tree is not the head's",
            stale,
            "push",
            [
                vec![
                    pulls(stale, merged(stale, stale_head)),
                    run("50", queue_pull(stale_head)),
                ],
                passed(stale_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "the trees agree but main before the merge is not in the head's history",
            twin,
            "push",
            [
                vec![
                    pulls(twin, merged(twin, twin_head)),
                    run("50", queue_pull(twin_head)),
                ],
                passed(twin_head, "50"),
            ]
            .concat(),
            "false",
            twin_head,
        ),
        (
            "a squash, whose head is not the tagged commit's second parent",
            squash,
            "push",
            [
                vec![
                    pulls(squash, merged(squash, squash_head)),
                    run("50", queue_pull(squash_head)),
                ],
                passed(squash_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "the merge commit's tree carries a file its up-to-date head never had",
            forged,
            "push",
            [
                vec![
                    pulls(forged, merged(forged, forged_head)),
                    run("50", queue_pull(forged_head)),
                ],
                passed(forged_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "two merged pull requests name the tagged commit",
            fresh,
            "push",
            [
                vec![
                    pulls(
                        fresh,
                        json!([
                            pull(1, true, fresh, fresh_head, repository, "main"),
                            pull(5, true, fresh, fresh_head, repository, "main")
                        ]),
                    ),
                    run("50", queue_pull(fresh_head)),
                ],
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "the pull request was never merged",
            fresh,
            "push",
            [
                vec![
                    pulls(
                        fresh,
                        json!([pull(1, false, fresh, fresh_head, repository, "main")]),
                    ),
                    run("50", queue_pull(fresh_head)),
                ],
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "the pull request merged as another commit",
            fresh,
            "push",
            [
                vec![
                    pulls(fresh, merged(stale, fresh_head)),
                    run("50", queue_pull(fresh_head)),
                ],
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "the pull request names no head commit",
            fresh,
            "push",
            [
                vec![
                    pulls(fresh, merged(fresh, "not-a-commit")),
                    run("50", queue_pull(fresh_head)),
                ],
                passed(fresh_head, "50"),
            ]
            .concat(),
            "false",
            "",
        ),
        (
            "the tagged commit's pull requests are unavailable",
            fresh,
            "push",
            [vec![run("50", queue_pull(fresh_head))], passed(fresh_head, "50")].concat(),
            "false",
            "",
        ),
    ];

    let mut wrong = Vec::new();
    for (index, (name, commit, event, responses, expected, head)) in cases.into_iter().enumerate() {
        let answer = prior_gate(&root.0, index, &history, commit, event, &responses);
        if answer.passed != expected || answer.head != head {
            wrong.push(format!(
                "{name}: passed={} head={}, expected passed={expected} head={head:?}\n{}",
                answer.passed, answer.head, answer.log
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "the prior-gate step decided these wrongly:\n{}",
        wrong.join("\n")
    );
}

/// The Cargo workspaces a job's tasks build: the root, and every `--manifest-path` directory.
fn job_workspaces(taskfile: &Value, tasks: &BTreeSet<String>) -> BTreeSet<String> {
    let mut reached_tasks = BTreeSet::new();
    for task in tasks {
        reached(taskfile, task, &mut reached_tasks);
    }
    let mut workspaces = BTreeSet::from([".".to_owned()]);
    for task in &reached_tasks {
        for command in shell_commands(taskfile, task) {
            let mut words = command.split_whitespace();
            while let Some(word) = words.next() {
                if word == "--manifest-path" {
                    let manifest = words.next().expect("--manifest-path names a manifest");
                    let directory = Path::new(manifest).parent().unwrap().to_str().unwrap();
                    workspaces
                        .insert(if directory.is_empty() { "." } else { directory }.to_owned());
                }
            }
        }
    }
    workspaces
}

/// Every lockfile a job builds from is fetched before the job's first task runs.
///
/// The gate's tasks pass `--offline` (`fuzz-check`), set `CARGO_NET_OFFLINE` (`support-check`) or
/// spawn `cargo run --offline` from inside a test (`support::tests::adversary_*` in ess-xtask, and
/// the generated-project cases the test shards run). The single-job gate never noticed, because
/// `cargo clippy --workspace` ran first and populated the registry. A job starts from an empty
/// one, so the fetch has to be explicit, and it has to cover every workspace the job builds, not
/// only the ones whose commands say `--offline` — the xtask case is invisible from the Taskfile.
#[test]
fn every_lockfile_a_lane_builds_is_fetched_before_the_lane_runs() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let lanes = lane_invocations(&ci);
    let root = workspace_root();
    let jobs: BTreeSet<&String> = lanes.iter().map(|lane| &lane.job).collect();
    for id in jobs {
        let own: Vec<&Lane> = lanes.iter().filter(|lane| &lane.job == id).collect();
        let first = own.iter().map(|lane| lane.step).min().unwrap();
        let tasks: BTreeSet<String> = own.iter().map(|lane| lane.task.clone()).collect();
        let steps = ci["jobs"][id.as_str()]["steps"]
            .as_sequence()
            .expect("job steps");
        let fetched: String = steps[..first]
            .iter()
            .map(|step| text(&step["run"]))
            .collect::<Vec<_>>()
            .join("\n");
        for workspace in job_workspaces(&taskfile, &tasks) {
            if !root.join(&workspace).join("Cargo.lock").is_file() {
                continue;
            }
            let spelled = if workspace == "." {
                fetched
                    .lines()
                    .any(|line| line.trim() == "cargo fetch --locked")
            } else {
                fetched.lines().any(|line| {
                    line.trim()
                        == format!("cargo fetch --locked --manifest-path {workspace}/Cargo.toml")
                })
            };
            assert!(
                spelled,
                "`{id}` builds `{workspace}` and no step before its first task fetches that \
                 lockfile"
            );
        }
    }
}

/// Integration-test binaries of `crates/<area>/<crate>/tests/` whose source matches `matches`.
fn binaries_where(matches: impl Fn(&str) -> bool) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let crates = workspace_root().join("crates");
    for area in fs::read_dir(&crates).unwrap().flatten() {
        for krate in fs::read_dir(area.path()).into_iter().flatten().flatten() {
            let tests = krate.path().join("tests");
            let package = krate.file_name().to_string_lossy().into_owned();
            for file in fs::read_dir(&tests).into_iter().flatten().flatten() {
                let path = file.path();
                if path.extension().is_some_and(|extension| extension == "rs")
                    && matches(&fs::read_to_string(&path).unwrap())
                {
                    let binary = path.file_stem().unwrap().to_string_lossy().into_owned();
                    found.insert(format!("{package}::{binary}"));
                }
            }
        }
    }
    found
}

/// The binary ids nextest runs one test at a time: every `binary_id(...)` in a filter whose
/// override assigns a test group declared with `max-threads = 1`.
fn serialised_binaries(config: &str) -> BTreeSet<String> {
    let serial_groups: BTreeSet<&str> = config
        .lines()
        .filter(|line| line.replace(' ', "").contains("={max-threads=1}"))
        .map(|line| line.split('=').next().unwrap().trim())
        .collect();
    let mut serialised = BTreeSet::new();
    for block in config.split("[[profile.default.overrides]]").skip(1) {
        let field = |name: &str| {
            block
                .lines()
                .find_map(|line| line.trim().strip_prefix(name))
                .and_then(|rest| rest.trim().strip_prefix('='))
                .map(|value| value.trim().trim_matches('"').to_owned())
                .unwrap_or_default()
        };
        if !serial_groups.contains(field("test-group").as_str()) {
            continue;
        }
        let filter = field("filter");
        for piece in filter.split("binary_id(").skip(1) {
            serialised.insert(piece.split(')').next().unwrap().to_owned());
        }
    }
    serialised
}

/// Under nextest every test is its own process, so a binary whose tests coordinate through state
/// shared by every test *in one process* loses that coordination, and runs one test at a time.
///
/// Two such mechanisms have existed, both found by the first nextest run (Actions run
/// 36126201626): `support/browser.rs`'s `STARTUP` mutex, which admits one Firefox start at a time,
/// and a `Once` that prunes every fixture directory not carrying this process's id — which, with
/// one process per test, deletes the fixtures of every test running beside it. Both are found here
/// by what the source says, so a new binary that includes the browser fixture or copies the prune
/// is serialised or named.
///
/// The other direction holds too: a binary serialised for a reason no scan finds any more runs one
/// test at a time for nothing. `execution_recovery` was the longest item on the CI critical path
/// for exactly that, after its prune became safe for neighbouring processes.
#[test]
fn nextest_serialises_exactly_the_binaries_that_coordinate_through_process_state() {
    let config = fs::read_to_string(workspace_root().join(".config/nextest.toml"))
        .expect("the nextest configuration is readable");
    let serialised = serialised_binaries(&config);
    // Split, so that this file does not contain what it scans for.
    let include = ["#[path = \"support/", "browser.rs\"]"].concat();
    let browser = binaries_where(|source| source.contains(&include));
    let pruning = binaries_where(prunes_from_a_once_per_process);
    assert!(
        browser.contains("ess-cli::coverage_browser"),
        "the browser scan found {browser:?}; it no longer sees the fixture it exists to find"
    );
    let missing: Vec<&String> = browser
        .iter()
        .chain(&pruning)
        .filter(|binary| !serialised.contains(*binary))
        .collect();
    assert!(
        missing.is_empty(),
        "these binaries coordinate their tests through process-global state and nextest runs \
         them in parallel processes: {missing:?}"
    );
    let causeless: Vec<&String> = serialised
        .iter()
        .filter(|binary| !browser.contains(*binary) && !pruning.contains(*binary))
        .collect();
    assert!(
        causeless.is_empty(),
        "nextest runs these binaries one test at a time and no scan finds a reason to: \
         {causeless:?}"
    );
}

/// The binary ids nextest runs with the whole shard to themselves: every `binary_id(...)` in a
/// filter whose override sets `threads-required = "num-test-threads"`.
fn whole_shard_binaries(config: &str) -> BTreeSet<String> {
    let mut whole = BTreeSet::new();
    for block in config.split("[[profile.default.overrides]]").skip(1) {
        let field = |name: &str| {
            block
                .lines()
                .find_map(|line| line.trim().strip_prefix(name))
                .and_then(|rest| rest.trim().strip_prefix('='))
                .map(|value| value.trim().trim_matches('"').to_owned())
                .unwrap_or_default()
        };
        if field("threads-required") != "num-test-threads" {
            continue;
        }
        for piece in field("filter").split("binary_id(").skip(1) {
            whole.insert(piece.split(')').next().unwrap().to_owned());
        }
    }
    whole
}

/// One Firefox at a time is not enough on a busy runner: with only the one-at-a-time group, a
/// Firefox start still competed with three neighbouring test processes and overran its budget
/// (ess#85 on 2026-09-25; again on `main` a1cf7233f, job 108296632588, 30.010s at the announce
/// stage, after ess#107 dropped the whole-shard setting). So every binary that includes the
/// browser fixture also takes the whole shard, and a later speed pass that removes it fails here
/// rather than on the next loaded runner.
#[test]
fn every_browser_binary_takes_the_whole_shard_while_it_runs() {
    let config = fs::read_to_string(workspace_root().join(".config/nextest.toml"))
        .expect("the nextest configuration is readable");
    let whole = whole_shard_binaries(&config);
    // Split, so that this file does not contain what it scans for.
    let include = ["#[path = \"support/", "browser.rs\"]"].concat();
    let browser = binaries_where(|source| source.contains(&include));
    assert!(
        browser.contains("ess-cli::coverage_browser"),
        "the browser scan found {browser:?}; it no longer sees the fixture it exists to find"
    );
    let sharing: Vec<&String> = browser
        .iter()
        .filter(|binary| !whole.contains(*binary))
        .collect();
    assert!(
        sharing.is_empty(),
        "these binaries start a real Firefox and share their shard with other test processes \
         while it starts: {sharing:?}"
    );
}

/// Whether one test source prunes fixture directories from a `Once`, once per process.
fn prunes_from_a_once_per_process(source: &str) -> bool {
    // Split, so that this file does not contain what it scans for.
    let once = ["std::sync::Once", "::new()"].concat();
    let prune = ["remove_dir_all(", "entry.path())"].concat();
    source.contains(&once) && source.contains(&prune)
}

/// The prune scan still sees the shape it exists to find, now that no binary carries it.
#[test]
fn the_prune_scan_recognises_a_once_per_process_prune() {
    let once = ["std::sync::Once", "::new()"].concat();
    let prune = ["remove_dir_all(", "entry.path())"].concat();
    let copied =
        format!("static PRUNED: {once};\nPRUNED.call_once(|| {{ let _ = std::fs::{prune}; }});\n");
    assert!(prunes_from_a_once_per_process(&copied));
    assert!(
        !prunes_from_a_once_per_process(&format!("let _ = std::fs::{prune};\n")),
        "a prune that is not once per process is not the shape"
    );
    assert!(
        !prunes_from_a_once_per_process(&format!("static STARTED: {once};\n")),
        "a `Once` that prunes nothing is not the shape"
    );
}

/// A pull request can edit `package.yml` to run on `pull_request`, and the lookup is keyed on
/// that pull request's head, so such a run of the head is reachable. So is a queue run that
/// failed or was cancelled after uploading. The prebuilt step refuses all three twice: in the API
/// query and in its `jq` filter. The stub answers by path and query, so the query half is held by
/// the answer's name and as text, and the filter half by these runs.
#[test]
fn a_release_publishes_no_pull_request_or_unsuccessful_package_run_of_the_merged_head() {
    let release = yaml(".github/workflows/release.yml");
    let query = text(&prebuilt_step(&release)["run"]);
    for pin in ["event=push", "status=success"] {
        assert!(
            query.contains(&format!("&{pin}&")),
            "the package-run query no longer asks for `{pin}`: {query}"
        );
    }

    let root = Scratch::new("release-prebuilt-review");
    let commit = "3ec2deb977bb19c292a611dcf92a3e676352864a";
    let runs = package_runs_path(commit);
    let held = artifacts_path(7);
    let every = artifacts(&RELEASE_ARTIFACTS, &[]);
    let run = |field: &str, value: &str| {
        let mut changed = package_run(
            7,
            commit,
            "queue/pr-119",
            ".github/workflows/package.yml",
            "2026-01-01T00:00:05Z",
        );
        changed[field] = json!(value);
        json!({ "workflow_runs": [changed] })
    };
    let cases = [
        (
            "a pull_request run of package.yml",
            run("event", "pull_request"),
        ),
        ("a failed queue run", run("conclusion", "failure")),
        ("a cancelled queue run", run("conclusion", "cancelled")),
    ];
    let mut wrong = Vec::new();
    for (index, (name, listing)) in cases.into_iter().enumerate() {
        let answer = prebuilt(
            &root.0,
            index,
            commit,
            &[(runs.clone(), listing), (held.clone(), every.clone())],
        );
        let (decision, log) = answer.split_once('\n').unwrap();
        if !decision.is_empty() {
            wrong.push(format!("{name}: run={decision}\n{log}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "the prebuilt step published these:\n{}",
        wrong.join("\n")
    );
}

/// Whether a `save-if` of the shape `${{ <a> == '<x>' && <b> != '<y>' || <c> == '<z>' }}` holds
/// for a run whose `github.ref` is `reference` and whose `github.event_name` is `event`. `&&` binds
/// tighter than `||`, as in GitHub's expression language.
fn save_if_holds(expression: &str, reference: &str, event: &str) -> bool {
    let inner = expression
        .strip_prefix("${{")
        .and_then(|rest| rest.strip_suffix("}}"))
        .unwrap_or_else(|| panic!("save-if `{expression}` is not one expression"));
    inner.split("||").any(|conjunction| {
        conjunction.split("&&").all(|term| {
            let (name, literal, equal) = term
                .split_once("!=")
                .map(|(name, literal)| (name, literal, false))
                .or_else(|| {
                    term.split_once("==")
                        .map(|(name, literal)| (name, literal, true))
                })
                .unwrap_or_else(|| panic!("save-if term `{term}` is not a comparison"));
            let literal = literal.trim().trim_matches('\'');
            let matches = match name.trim() {
                "github.ref" => literal == reference,
                "github.event_name" => literal == event,
                other => {
                    panic!("save-if term names `{other}`, which this reader does not evaluate")
                }
            };
            matches == equal
        })
    })
}

/// `release.yml`'s header documents a manual dispatch that backfills an existing tag, and a
/// dispatch from `main` runs with `github.ref` at `refs/heads/main`. `save-if` keys on that ref, so
/// the backfill's release job, its `package.yml` call and its `ci.yml` gate save compile caches
/// built from the old tag's source under `main`'s scope — the one every later run restores, and
/// the allowance the `save-if` rule was introduced to protect.
#[test]
fn a_release_backfill_dispatched_from_main_saves_no_compile_cache() {
    let root = workspace_root();
    let mut files: Vec<PathBuf> = fs::read_dir(root.join(".github/workflows"))
        .expect("the workflow directory is readable")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "yml"))
        .collect();
    files.sort();
    let mut saving = Vec::new();
    for file in files {
        let relative = &file.strip_prefix(&root).unwrap().display().to_string();
        let workflow = yaml(relative);
        for (id, job) in workflow["jobs"].as_mapping().into_iter().flatten() {
            for step in job["steps"].as_sequence().into_iter().flatten() {
                if !text(&step["uses"]).starts_with("Swatinem/rust-cache@") {
                    continue;
                }
                let save_if = text(&step["with"]["save-if"]);
                if save_if_holds(save_if, "refs/heads/main", "workflow_dispatch") {
                    saving.push(format!("{relative} `{}`", text(id)));
                }
            }
        }
    }
    for (reference, event) in [
        ("refs/heads/main", "push"),
        ("refs/heads/main", "schedule"),
        ("refs/pull/7/merge", "pull_request"),
    ] {
        assert!(
            save_if_holds(CACHE_SAVE_IF, reference, event),
            "the rule no longer saves the cache of a {event} run on {reference}"
        );
    }
    for (reference, event) in [
        ("refs/heads/main", "workflow_dispatch"),
        ("refs/heads/queue/pr-119", "push"),
        ("refs/tags/0.33.0", "push"),
    ] {
        assert!(
            !save_if_holds(CACHE_SAVE_IF, reference, event),
            "the rule saves the cache of a {event} run on {reference}"
        );
    }
    assert!(
        saving.is_empty(),
        "a release backfill dispatched from main saves these compile caches: {saving:?}"
    );
}

/// Runs the `msrv` job's `Read the declared rust-version` step in `directory` the way Actions
/// does, returning its exit status, the `toolchain` it wrote, if any, and its log.
fn run_msrv_read_step(case: &Path, directory: &Path) -> (bool, Option<String>, String) {
    let ci = yaml(".github/workflows/ci.yml");
    let step = ci["jobs"]["msrv"]["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .find(|step| text(&step["id"]) == "msrv")
        .expect("the msrv job has a step `msrv` that reads the declared rust-version");
    let script = case.join("read-rust-version.sh");
    fs::write(&script, text(&step["run"])).unwrap();
    let output_file = case.join("github-output");
    fs::write(&output_file, "").unwrap();
    let output = isolated("bash", directory)
        .args(["--noprofile", "--norc", "-eo", "pipefail"])
        .arg(&script)
        .env("GITHUB_OUTPUT", &output_file)
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let written = fs::read_to_string(&output_file).unwrap();
    let toolchain = written
        .lines()
        .find_map(|line| line.strip_prefix("toolchain="))
        .map(str::to_owned);
    (output.status.success(), toolchain, log)
}

/// The workspace declares a `rust-version`, and downstream repositories compile ESS with exactly
/// that compiler. Nothing checked it, so a line only newer compilers accept shipped (E0716 in
/// `ess-conformance` on 1.85). The `msrv` job checks every target on that compiler on every pull
/// request, reads the version from `Cargo.toml` instead of naming it a second time, and `Gate`
/// carries its result (`the_gate_check_aggregates_every_lane_and_cannot_be_skipped`).
#[test]
fn the_msrv_job_checks_every_target_on_the_declared_rust_version() {
    let manifest: toml_lines::Manifest =
        toml_lines::parse(&fs::read_to_string(workspace_root().join("Cargo.toml")).unwrap());
    let declared = manifest
        .get("workspace.package", "rust-version")
        .expect("the workspace declares a rust-version")
        .trim_matches('"')
        .to_owned();

    let ci = yaml(".github/workflows/ci.yml");
    let job = &ci["jobs"]["msrv"];
    assert!(job.is_mapping(), "ci.yml has no `msrv` job");
    // The workflow-wide lld flag reaches the linker through gcc only on newer toolchains; on the
    // declared one gcc finds no `ld` (the 0.75.0 queue run), so the job links with the system one.
    assert_eq!(
        text(&job["env"]["CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS"]),
        "",
        "the msrv job inherits the workflow's lld flag, which the declared toolchain cannot link with"
    );
    assert!(
        !job["env"]["CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS"].is_null(),
        "the msrv job does not clear the workflow's lld flag"
    );
    assert!(
        job["if"].is_null(),
        "the msrv job does not run on every pull request: if `{}`",
        text(&job["if"])
    );
    let source = ess_yaml::to_string(job).unwrap();
    assert!(
        !source.contains(&declared),
        "the msrv job names the declared rust-version `{declared}` a second time"
    );

    let steps: Vec<&Value> = job["steps"].as_sequence().into_iter().flatten().collect();
    let read = steps
        .iter()
        .position(|step| text(&step["id"]) == "msrv")
        .expect("the msrv job reads the declared rust-version");
    let (install, toolchain) = step_using(job, "dtolnay/rust-toolchain")
        .expect("the msrv job installs its toolchain with dtolnay/rust-toolchain");
    assert_eq!(
        text(&toolchain["with"]["toolchain"]),
        "${{ steps.msrv.outputs.toolchain }}",
        "the msrv job installs a toolchain other than the one it read"
    );
    let check = steps
        .iter()
        .position(|step| {
            text(&step["run"]).trim()
                == "cargo +${{ steps.msrv.outputs.toolchain }} check --workspace --all-targets --locked"
        })
        .expect(
            "the msrv job runs `cargo +${{ steps.msrv.outputs.toolchain }} check --workspace --all-targets --locked`",
        );
    assert!(
        read < install && install < check,
        "the msrv job does not read, install and then check, in that order"
    );

    // The step itself, against this workspace and against fixtures.
    let case = Scratch::new("msrv");
    let (ok, written, log) = run_msrv_read_step(&case.0, &workspace_root());
    assert!(ok, "the read step failed on this workspace: {log}");
    let expected = if declared.split('.').count() == 2 {
        format!("{declared}.0")
    } else {
        declared.clone()
    };
    assert_eq!(written.as_deref(), Some(expected.as_str()), "{log}");

    for (index, (manifest, expected)) in [
        (
            "[workspace.package]\nversion = \"9.9.9\"\nrust-version = \"1.90.1\"\n",
            Some("1.90.1"),
        ),
        (
            "[workspace.package]\nversion = \"9.9.9\"\nrust-version = \"1.91\"\n",
            Some("1.91.0"),
        ),
        ("[workspace.package]\nversion = \"9.9.9\"\n", None),
        (
            "[workspace.package]\nrust-version = \"1.90\"\n[package]\nrust-version = \"1.91\"\n",
            None,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let directory = case.0.join(format!("fixture-{index}"));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("Cargo.toml"), manifest).unwrap();
        let (ok, written, log) = run_msrv_read_step(&case.0, &directory);
        match expected {
            Some(expected) => {
                assert!(ok, "the read step failed on {manifest:?}: {log}");
                assert_eq!(written.as_deref(), Some(expected), "{manifest:?}: {log}");
            }
            None => assert!(
                !ok && written.is_none(),
                "the read step chose a toolchain from {manifest:?}: {written:?} {log}"
            ),
        }
    }
}

/// The jobs a plan-only pull request does not start: the two archive builds and the test shards.
const PLAN_ONLY_SKIPS: [&str; 3] = ["build-tests", "build-feature-off", "test"];

/// The job of `ci.yml` that decides whether a pull request's diff is plan-only.
const PLAN_ONLY_JOB: &str = "diff";

/// The condition each job of `PLAN_ONLY_SKIPS` carries, and no other job.
const PLAN_ONLY_IF: &str = "${{ needs.diff.outputs.plan-only != 'true' }}";

/// The condition every step of the deciding job carries: it decides only on a run that may skip.
const PLAN_ONLY_STEP_IF: &str = "${{ env.PLAN_ONLY_CANDIDATE == 'true' }}";

/// The part of the plan store a test binary reads: the reproductions committed with issues. A
/// diff that touches it is not plan-only.
const PLAN_ONLY_EXCEPT: &str = ".engineering/repro/";

/// The tasks that build or run the test archives.
const TEST_ARCHIVE_TASKS: [&str; 4] = [
    "test-archive-workspace",
    "test-archive-feature-off",
    "test-shard",
    "test-feature-off",
];

fn assert_plan_only_skips_exactly_the_test_jobs(ci: &Value) {
    let skips: BTreeSet<String> = PLAN_ONLY_SKIPS.iter().map(|id| (*id).to_owned()).collect();
    let testing: BTreeSet<String> = lane_invocations(ci)
        .into_iter()
        .filter(|lane| TEST_ARCHIVE_TASKS.contains(&lane.task.as_str()))
        .map(|lane| lane.job)
        .collect();
    assert_eq!(
        testing, skips,
        "the jobs that build or run the test archives are not the jobs a plan-only diff skips"
    );
    for (id, job) in ci["jobs"].as_mapping().expect("ci.yml has jobs") {
        let id = text(id);
        let condition = text(&job["if"]);
        if skips.contains(id) {
            assert_eq!(
                condition, PLAN_ONLY_IF,
                "`{id}` is not skipped on a plan-only diff, or on something besides it"
            );
            assert!(
                needs_of(job).contains(PLAN_ONLY_JOB),
                "`{id}` does not wait for the plan-only decision of `{PLAN_ONLY_JOB}`"
            );
        } else {
            assert!(
                !condition.contains("plan-only"),
                "`{id}` is skipped on a plan-only diff too: `{condition}`"
            );
        }
    }
}

/// A pull request whose every changed path lies in the plan store, outside its reproductions,
/// builds no test archive and starts no test shard. Exactly the jobs that build or run the
/// archives carry the condition, and each carries exactly it; a job added to build or run them
/// without it, or another job skipped by it, fails here.
#[test]
fn a_plan_only_pull_request_skips_exactly_the_test_archives_and_the_test_shards() {
    let ci = yaml(".github/workflows/ci.yml");
    assert_plan_only_skips_exactly_the_test_jobs(&ci);
    for (output, runs) in [("true", false), ("false", true), ("", true)] {
        let context = BTreeMap::from([("needs.diff.outputs.plan-only", output)]);
        assert_eq!(
            evaluate(PLAN_ONLY_IF, &context).truthy(),
            runs,
            "a decision of `{output}` runs the test jobs {}",
            !runs
        );
    }

    let mut dropped = ci.clone();
    dropped["jobs"]["test"]["if"] = Value::Null;
    let mut other = ci.clone();
    other["jobs"]["build-feature-off"]["if"] =
        Value::String("${{ needs.diff.outputs.plan-only == 'false' }}".to_owned());
    let mut widened = ci.clone();
    widened["jobs"]["checks"]["if"] = Value::String(PLAN_ONLY_IF.to_owned());
    for (name, mutant) in [("dropped", dropped), ("other", other), ("widened", widened)] {
        assert!(
            std::panic::catch_unwind(|| assert_plan_only_skips_exactly_the_test_jobs(&mutant))
                .is_err(),
            "the {name} plan-only condition passes"
        );
    }
}

/// Whether the deciding job may decide at all for a run of `ci.yml` in `context`.
fn plan_only_candidate(ci: &Value, context: &BTreeMap<&str, &str>) -> bool {
    let job = &ci["jobs"][PLAN_ONLY_JOB];
    let candidate = text(&job["env"]["PLAN_ONLY_CANDIDATE"]);
    assert!(
        !candidate.is_empty(),
        "`{PLAN_ONLY_JOB}` sets no PLAN_ONLY_CANDIDATE"
    );
    match evaluate(candidate, context) {
        ExpressionValue::Flag(flag) => flag,
        ExpressionValue::Text(other) => {
            panic!("PLAN_ONLY_CANDIDATE `{candidate}` is `{other}`, not true or false")
        }
    }
}

/// The expression context of a run of `ci.yml` in `owner/repository`: its event, `github.head_ref`
/// and head repository (a pull request's, empty elsewhere), and `inputs.checkout-ref`.
fn ci_context<'a>(
    event: &'a str,
    head_ref: &'a str,
    head_repository: &'a str,
    checkout: &'a str,
) -> BTreeMap<&'static str, &'a str> {
    BTreeMap::from([
        ("github.event_name", event),
        ("github.head_ref", head_ref),
        (
            "github.event.pull_request.head.repo.full_name",
            head_repository,
        ),
        ("github.repository", "owner/repository"),
        ("inputs.checkout-ref", checkout),
    ])
}

/// The `qualifies` function of release.yml's `prior-gate` step, verbatim.
fn prior_gate_qualifies(release: &Value) -> String {
    let script = text(&prior_gate_step(release)["run"]);
    let start = script
        .find("qualifies() {")
        .expect("the prior-gate step defines `qualifies`");
    let end = script[start..]
        .find("\n}\n")
        .expect("the prior-gate step closes `qualifies`");
    script[start..start + end + 3].to_owned()
}

/// Runs `function`, the `qualifies` function of `prior-gate`, on workflow run `id`, which `gh`
/// describes as `run`, for `commit`. Returns whether the run qualifies; a run `qualifies` cannot
/// read fails the case.
fn run_prior_gate_qualifies(
    case: &Path,
    function: &str,
    id: usize,
    run: &serde_json::Value,
    commit: &str,
) -> bool {
    let bin = case.join("bin");
    let answers = case.join("responses");
    fs::create_dir_all(&answers).unwrap();
    if !bin.join("gh").exists() {
        fs::create_dir_all(&bin).unwrap();
        executable::install_bytes(&bin.join("gh"), GH_PATH_STUB.as_bytes()).unwrap();
    }
    let id = id.to_string();
    fs::write(
        answers.join(format!("{}.json", run_path(&id).replace('/', "_"))),
        run.to_string(),
    )
    .unwrap();
    let script = case.join(format!("qualifies-{id}.sh"));
    fs::write(
        &script,
        format!("REPOSITORY=owner/repository\n{function}\nqualifies {id} {commit}\n"),
    )
    .unwrap();
    let output = isolated("bash", case)
        .args(["--noprofile", "--norc"])
        .arg(&script)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("GH_STUB_RESPONSES", &answers)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    let code = output.status.code();
    assert!(
        matches!(code, Some(0 | 1)),
        "`qualifies` could not read run {id} ({run}): {code:?} {log}"
    );
    code == Some(0)
}

const RUN_OWN: &str = "owner/repository";
const RUN_COMMIT: &str = "3ec2deb977bb19c292a611dcf92a3e676352864a";

/// The runs of `ci.yml` the skip decisions are held to: the event, `github.head_ref`, the pull
/// request's head repository, `inputs.checkout-ref`, and whether the run may skip.
fn candidate_runs() -> [(&'static str, &'static str, &'static str, &'static str, bool); 11] {
    let (own, commit) = (RUN_OWN, RUN_COMMIT);
    [
        ("push", "", "", "", false),
        ("schedule", "", "", "", false),
        ("pull_request", "queue/pr-119", own, "", false),
        ("merge_group", "", "", "", false),
        // A call from release.yml: a tag push, and a backfill dispatched from any branch.
        ("push", "", "", commit, false),
        ("workflow_dispatch", "", "", commit, false),
        ("workflow_dispatch", "", "", "", false),
        ("pull_request", "feat/x", own, commit, false),
        // A pull request from an ordinary head, and from a fork, which can name a branch anything.
        ("pull_request", "feat/x", own, "", true),
        ("pull_request", "queue/pr-119", "fork/repository", "", true),
        ("pull_request", "main", "fork/repository", "", true),
    ]
}

/// The workflow runs `prior-gate` reads, each the ci.yml run it describes: event, head branch,
/// head repository.
fn described_runs() -> [(&'static str, &'static str, &'static str); 8] {
    let own = RUN_OWN;
    [
        ("push", "main", own),
        ("schedule", "main", own),
        ("pull_request", "queue/pr-119", own),
        ("pull_request", "feat/x", own),
        ("pull_request", "queue/pr-119", "fork/repository"),
        ("merge_group", "gh-readonly-queue/main/pr-7", own),
        ("workflow_dispatch", "main", own),
        ("push", "queue/pr-119", own),
    ]
}

/// A plan-only skip is never possible on a run that has to test everything: a `main` push, the
/// nightly run, the pull request of a `queue/` head of this repository, a merge-group run, a
/// backfill, and a call from `release.yml` (`checkout-ref` set). And no run that may skip ever
/// stands in for a release's gate: the `qualifies` function of release.yml's `prior-gate` step,
/// run here, accepts none of them, so a later widening of either side fails this case.
#[test]
fn no_run_that_must_test_everything_or_stands_in_for_a_release_can_skip_for_a_plan_only_diff() {
    let ci = yaml(".github/workflows/ci.yml");
    let release = yaml(".github/workflows/release.yml");
    let job = &ci["jobs"][PLAN_ONLY_JOB];
    assert!(job.is_mapping(), "ci.yml has no `{PLAN_ONLY_JOB}` job");
    assert!(
        job["if"].is_null(),
        "`{PLAN_ONLY_JOB}` is conditional, so the jobs that wait for it would be skipped with it"
    );
    assert_eq!(
        text(&job["outputs"]["plan-only"]),
        "${{ steps.plan-only.outputs.plan-only }}",
        "`{PLAN_ONLY_JOB}` reports a decision its `plan-only` step did not make"
    );
    let steps: Vec<&Value> = job["steps"].as_sequence().into_iter().flatten().collect();
    assert!(
        steps.iter().any(|step| text(&step["id"]) == "plan-only"),
        "`{PLAN_ONLY_JOB}` has no `plan-only` step"
    );
    for step in &steps {
        assert_eq!(
            text(&step["if"]),
            PLAN_ONLY_STEP_IF,
            "a step of `{PLAN_ONLY_JOB}` runs on a run that may not skip: {step:?}"
        );
    }

    let commit = RUN_COMMIT;
    let wrong: Vec<String> = candidate_runs()
        .iter()
        .filter(|&&(event, head_ref, repository, checkout, may)| {
            plan_only_candidate(&ci, &ci_context(event, head_ref, repository, checkout)) != may
        })
        .map(|run| format!("{run:?}"))
        .collect();
    assert!(
        wrong.is_empty(),
        "the plan-only decision may run wrongly on: {wrong:?}"
    );

    let case = Scratch::new("plan-only-qualifies");
    let function = prior_gate_qualifies(&release);
    let mut qualifying = 0;
    let mut skippable = 0;
    let mut stands_in = Vec::new();
    for (index, (event, branch, repository)) in described_runs().into_iter().enumerate() {
        let run = workflow_run(
            ".github/workflows/ci.yml",
            event,
            Some(branch),
            repository,
            commit,
        );
        let qualifies = run_prior_gate_qualifies(&case.0, &function, 100 + index, &run, commit);
        let (head_ref, head_repository) = if event == "pull_request" {
            (branch, repository)
        } else {
            ("", "")
        };
        let may_skip = plan_only_candidate(&ci, &ci_context(event, head_ref, head_repository, ""));
        qualifying += usize::from(qualifies);
        skippable += usize::from(may_skip);
        if qualifies && may_skip {
            stands_in.push(format!("{event} {branch} {repository}"));
        }
    }
    assert!(
        qualifying > 0 && skippable > 0,
        "the case reads nothing: {qualifying} qualifying runs, {skippable} that may skip"
    );
    assert!(
        stands_in.is_empty(),
        "a run that may skip its tests for a plan-only diff stands in for a release's gate: \
         {stands_in:?}"
    );
}

/// Runs `Gate`'s step with the results `results` gives each job it needs, and `plan_only` as the
/// deciding job's output, `slow` as its `slow` output. Returns whether it passed, and its log.
fn run_gate(
    case: &Path,
    results: &BTreeMap<String, String>,
    plan_only: &str,
    slow: &str,
) -> (bool, String) {
    let ci = yaml(".github/workflows/ci.yml");
    let gate = &ci["jobs"]["gate"];
    let steps: Vec<&Value> = gate["steps"].as_sequence().into_iter().flatten().collect();
    let [step] = steps.as_slice() else {
        panic!("Gate runs {} steps, not one", steps.len())
    };
    let script = case.join("gate.sh");
    fs::write(&script, text(&step["run"])).unwrap();
    let mut command = isolated("bash", case);
    command
        .args(["--noprofile", "--norc", "-eo", "pipefail"])
        .arg(&script);
    for (name, value) in step["env"].as_mapping().into_iter().flatten() {
        let value = text(value);
        let supplied = if value == "${{ needs.diff.outputs.plan-only }}" {
            plan_only.to_owned()
        } else if value == "${{ needs.diff.outputs.slow }}" {
            slow.to_owned()
        } else {
            let need = value
                .strip_prefix("${{ needs.")
                .and_then(|rest| rest.strip_suffix(".result }}"))
                .unwrap_or_else(|| panic!("Gate reads `{value}`, which this case cannot supply"));
            results
                .get(need)
                .unwrap_or_else(|| {
                    panic!("Gate reads the result of `{need}`, which it does not need")
                })
                .clone()
        };
        command.env(text(name), supplied);
    }
    let output = command.output().unwrap();
    let log = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    (output.status.success(), log)
}

/// `Gate` passes the archive builds and the test shards skipped only when the deciding job
/// succeeded and said the diff is plan-only, and says so in its log. It fails on a failed or
/// cancelled job whatever the decision, on any other job skipped, on a skipped test job without
/// that decision, and when the deciding job itself did not succeed.
#[test]
fn the_gate_passes_skipped_test_jobs_only_for_a_plan_only_diff() {
    let ci = yaml(".github/workflows/ci.yml");
    let gate = &ci["jobs"]["gate"];
    let needs = needs_of(gate);
    assert!(
        needs.contains(PLAN_ONLY_JOB),
        "Gate does not wait for the plan-only decision"
    );
    let case = Scratch::new("plan-only-gate");
    let all = |result: &str| -> BTreeMap<String, String> {
        needs
            .iter()
            .map(|need| (need.clone(), result.to_owned()))
            .collect()
    };
    let with = |changes: &[(&str, &str)]| {
        let mut results = all("success");
        for (job, result) in changes {
            assert!(results.contains_key(*job), "Gate does not need `{job}`");
            results.insert((*job).to_owned(), (*result).to_owned());
        }
        results
    };
    let skipped: Vec<(&str, &str)> = PLAN_ONLY_SKIPS
        .iter()
        .map(|job| (*job, "skipped"))
        .collect();

    let mut wrong = Vec::new();
    let mut expect =
        |name: String, results: BTreeMap<String, String>, plan_only: &str, pass: bool| {
            let (passed, log) = run_gate(&case.0, &results, plan_only, "");
            if passed != pass {
                wrong.push(format!("{name}: passed {passed}, expected {pass}\n{log}"));
            }
            log
        };

    expect("every job passed".into(), all("success"), "", true);
    let log = expect("a plan-only diff".into(), with(&skipped), "true", true);
    for job in PLAN_ONLY_SKIPS {
        assert!(
            log.contains(&format!(
                "the {job} job was skipped because the diff is plan-only"
            )),
            "Gate does not say why `{job}` was skipped: {log}"
        );
    }
    for job in needs.iter().filter(|job| job.as_str() != PLAN_ONLY_JOB) {
        let skippable = PLAN_ONLY_SKIPS.contains(&job.as_str());
        expect(
            format!("{job} skipped on a plan-only diff"),
            with(&[(job.as_str(), "skipped")]),
            "true",
            skippable,
        );
        expect(
            format!("{job} skipped without a plan-only diff"),
            with(&[(job.as_str(), "skipped")]),
            "",
            false,
        );
        expect(
            format!("{job} skipped on a `false` decision"),
            with(&[(job.as_str(), "skipped")]),
            "false",
            false,
        );
        for result in ["failure", "cancelled"] {
            let mut changes = skipped.clone();
            changes.retain(|(skip, _)| *skip != job.as_str());
            changes.push((job.as_str(), result));
            expect(
                format!("{job} {result} on a plan-only diff"),
                with(&changes),
                "true",
                false,
            );
        }
    }
    for result in ["failure", "cancelled", "skipped"] {
        let mut changes = skipped.clone();
        changes.push((PLAN_ONLY_JOB, result));
        expect(
            format!("the deciding job {result}"),
            with(&changes),
            "true",
            false,
        );
    }
    assert!(
        wrong.is_empty(),
        "Gate decides wrongly:\n{}",
        wrong.join("\n")
    );
}

/// Runs the deciding job's step `id` (`plan-only` or `slow`) in `checkout` the way Actions does,
/// and returns the decision it wrote and its log. The step must exit 0 whatever it finds: an
/// error decides, it never fails the job.
fn run_plan_only_step(case: &Path, checkout: &Path, id: &str) -> (String, String) {
    let ci = yaml(".github/workflows/ci.yml");
    let step = ci["jobs"][PLAN_ONLY_JOB]["steps"]
        .as_sequence()
        .into_iter()
        .flatten()
        .find(|step| text(&step["id"]) == id)
        .unwrap_or_else(|| panic!("the deciding job has no `{id}` step"));
    assert_eq!(text(&step["shell"]), "bash", "the {id} step is not bash");
    assert!(
        step["env"].is_null(),
        "the {id} step reads an environment this harness does not supply"
    );
    let temp = case.join("runner-temp");
    fs::create_dir_all(&temp).unwrap();
    let script = case.join(format!("{id}.sh"));
    fs::write(&script, text(&step["run"])).unwrap();
    let output_file = case.join("github-output");
    fs::write(&output_file, "").unwrap();
    let output = isolated("bash", checkout)
        .args(["--noprofile", "--norc", "-eo", "pipefail"])
        .arg(&script)
        .env("GITHUB_OUTPUT", &output_file)
        .env("RUNNER_TEMP", &temp)
        // The checkout this test runs from can itself be a plan-only merge, so `git` must never
        // look for a repository above the case directory.
        .env("GIT_CEILING_DIRECTORIES", case)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the {id} step failed the job instead of deciding: {log}"
    );
    let written = fs::read_to_string(&output_file).unwrap();
    let decisions: Vec<&str> = written
        .lines()
        .filter_map(|line| line.strip_prefix(&format!("{id}=")))
        .collect();
    let [decision] = decisions.as_slice() else {
        panic!("the {id} step wrote {decisions:?}, not one decision: {log}")
    };
    assert!(
        matches!(*decision, "true" | "false"),
        "the {id} step decided `{decision}`: {log}"
    );
    ((*decision).to_owned(), log)
}

/// A plan-only pull request, as the deciding job checks it out: the merge commit of the pull
/// request at depth `depth`, `refs/pull/<n>/merge` in a real run.
fn plan_only_checkout(root: &Path, origin: &Path, reference: &str, depth: u32) -> PathBuf {
    let checkout = root.join(format!("checkout-{reference}-{depth}"));
    fs::create_dir_all(&checkout).unwrap();
    git(&checkout, &["init", "--quiet"]);
    git(
        &checkout,
        &[
            "fetch",
            "--quiet",
            "--no-tags",
            &format!("--depth={depth}"),
            &format!("file://{}", origin.display()),
            reference,
        ],
    );
    git(
        &checkout,
        &["checkout", "--quiet", "--detach", "FETCH_HEAD"],
    );
    checkout
}

/// What one pull request of the plan-only histories changes on its branch.
type PlanChange<'a> = Box<dyn Fn() + 'a>;

/// The plan-only decision is "every path the pull request changes lies under `.engineering/`,
/// none under `.engineering/repro/`", read from the merge commit Actions tests against its first
/// parent, the base it merges into. The step is run against real Git histories: a branch that
/// left `main` before `main` moved on, merged with `--no-ff` as `refs/pull/<n>/merge` is. A rename
/// counts with both of its paths, an empty change is not plan-only, and anything the step cannot
/// read decides `false`.
#[test]
// One table of histories, which reads best whole.
#[allow(clippy::too_many_lines)]
fn the_plan_only_decision_is_every_changed_path_under_the_plan_store() {
    let case = Scratch::new("plan-only-decision");
    let origin = case.0.join("origin");
    fs::create_dir_all(&origin).unwrap();
    git(&origin, &["init", "--quiet", "--initial-branch=main"]);
    // `crates/lib.rs` stays where it is, so moving `crates/a.rs` out is no directory rename that
    // would carry `main`'s new `crates/b.rs` with it and stop the merge.
    for (path, contents) in [
        ("crates/a.rs", "a"),
        ("crates/lib.rs", "lib"),
        ("AGENTS.md", "agents"),
        ("docs/x.md", "docs"),
        (".engineering/planning/story/old.md", "old"),
        (".engineering/repro/501/system.yaml", "repro"),
    ] {
        let file = origin.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, contents).unwrap();
    }
    git(&origin, &["add", "--all"]);
    git(&origin, &["commit", "--quiet", "--message", "base"]);
    let base = git(&origin, &["rev-parse", "HEAD"]);
    // `main` moves on after every branch below left it: its change is not the pull request's.
    let moved = origin.join("crates/b.rs");
    fs::write(&moved, "b").unwrap();
    git(&origin, &["add", "--all"]);
    git(
        &origin,
        &["commit", "--quiet", "--message", "main moves on"],
    );

    let write = |path: &str, contents: &str| {
        let file = origin.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, contents).unwrap();
    };
    let many: Vec<String> = (0..300)
        .map(|n| format!(".engineering/evidence/run-{n:03}.md"))
        .collect();
    let cases: Vec<(&str, PlanChange<'_>, &str)> = vec![
        (
            "one story",
            Box::new(|| write(".engineering/planning/story/x.md", "x")),
            "true",
        ),
        (
            "a story and AGENTS.md",
            Box::new(|| {
                write(".engineering/planning/story/x.md", "x");
                write("AGENTS.md", "changed");
            }),
            "false",
        ),
        (
            "docs only",
            Box::new(|| write("docs/x.md", "changed")),
            "false",
        ),
        (
            "a sibling of the plan store",
            Box::new(|| write(".engineeringx/y", "y")),
            "false",
        ),
        (
            "a rename into the plan store",
            Box::new(|| {
                git(&origin, &["mv", "crates/a.rs", ".engineering/a.rs"]);
            }),
            "false",
        ),
        (
            "a reproduction a test reads",
            Box::new(|| write(".engineering/repro/501/system.yaml", "changed")),
            "false",
        ),
        (
            "a story deleted",
            Box::new(|| {
                git(
                    &origin,
                    &["rm", "--quiet", ".engineering/planning/story/old.md"],
                );
            }),
            "true",
        ),
        (
            "a story with a non-ASCII name",
            Box::new(|| write(".engineering/planning/story/\u{e4}.md", "x")),
            "true",
        ),
        (
            "300 changed files in the plan store",
            Box::new(|| {
                for path in &many {
                    write(path, "evidence");
                }
            }),
            "true",
        ),
        ("no change", Box::new(|| {}), "false"),
    ];

    let mut wrong = Vec::new();
    for (index, (name, change, expected)) in cases.iter().enumerate() {
        let branch = format!("case-{index}");
        git(&origin, &["checkout", "--quiet", "-b", &branch, &base]);
        change();
        git(&origin, &["add", "--all"]);
        git(
            &origin,
            &["commit", "--quiet", "--allow-empty", "--message", name],
        );
        git(&origin, &["checkout", "--quiet", "--detach", "main"]);
        git(
            &origin,
            &["merge", "--quiet", "--no-ff", "--message", name, &branch],
        );
        git(&origin, &["branch", &format!("merge-{index}"), "HEAD"]);
        let checkout = plan_only_checkout(&case.0, &origin, &format!("merge-{index}"), 2);
        let (decision, log) = run_plan_only_step(&case.0, &checkout, "plan-only");
        if decision != *expected {
            wrong.push(format!("{name}: {decision}, expected {expected}\n{log}"));
        }
    }

    // What the step cannot read decides `false`: a checkout too shallow to hold the base, a head
    // that is no merge commit, and no repository at all.
    let shallow = plan_only_checkout(&case.0, &origin, "merge-0", 1);
    let plain = plan_only_checkout(&case.0, &origin, "case-0", 2);
    let empty = case.0.join("no-repository");
    fs::create_dir_all(&empty).unwrap();
    for (name, checkout) in [
        ("a checkout without the base", shallow),
        ("a head that is no merge commit", plain),
        ("no repository", empty),
    ] {
        let (decision, log) = run_plan_only_step(&case.0, &checkout, "plan-only");
        if decision != "false" {
            wrong.push(format!("{name}: {decision}, expected false\n{log}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "the plan-only step decides wrongly:\n{}",
        wrong.join("\n")
    );
}

/// A plan-only diff skips the test archives, so no test binary in them may read the plan store
/// outside `.engineering/repro/`, the one part of it a plan-only diff never touches. `ess-xtask`'s
/// tests run in `Checks`, which a plan-only diff does not skip, and are left out. A line that
/// names the store only to skip it in a tree walk (beside `"target"`) is not a read.
#[test]
fn no_archived_test_reads_the_plan_store_outside_its_reproductions() {
    fn walk(directory: &Path, into: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                if !matches!(name.as_str(), "target" | "node_modules" | ".git") {
                    walk(&path, into);
                }
            } else {
                into.push(path);
            }
        }
    }
    let root = workspace_root();
    let xtask = root.join("crates/edge/ess-xtask");
    let mut files = Vec::new();
    walk(&root.join("crates"), &mut files);
    let mut reproductions = 0;
    let mut readers = Vec::new();
    for file in files.iter().filter(|file| !file.starts_with(&xtask)) {
        let Ok(source) = fs::read_to_string(file) else {
            continue;
        };
        for (number, line) in source.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || code.starts_with('#') {
                continue;
            }
            for (at, _) in line.match_indices(".engineering/") {
                if line[at..].starts_with(PLAN_ONLY_EXCEPT) {
                    reproductions += 1;
                } else {
                    readers.push(format!("{}:{}", file.display(), number + 1));
                }
            }
            if line.contains("\".engineering\"") && !line.contains("\"target\"") {
                readers.push(format!("{}:{}", file.display(), number + 1));
            }
        }
    }
    assert!(
        reproductions > 0,
        "the scan found no test reading a reproduction; it no longer sees what it exists to find"
    );
    assert!(
        readers.is_empty(),
        "these archived tests read the plan store outside {PLAN_ONLY_EXCEPT}, so a plan-only \
         diff would skip the tests of what it changed: {readers:#?}"
    );
}

/// The commands of `test-xtask` with each `{{.NAME}}` of the task's `vars` expanded.
fn test_xtask_commands(taskfile: &Value) -> Vec<String> {
    let task = &taskfile["tasks"]["test-xtask"];
    let vars: Vec<(String, String)> = task["vars"]
        .as_mapping()
        .into_iter()
        .flatten()
        .map(|(name, value)| (text(name).to_owned(), text(value).to_owned()))
        .collect();
    shell_commands(taskfile, "test-xtask")
        .into_iter()
        .map(|mut command| {
            for (name, value) in &vars {
                command = command.replace(&format!("{{{{.{name}}}}}"), value);
            }
            command
        })
        .collect()
}

#[test]
fn test_xtask_sets_no_wrapper_and_no_build_jobs() {
    let taskfile = yaml("Taskfile.yml");
    let task = &taskfile["tasks"]["test-xtask"];
    let mut texts = shell_commands(&taskfile, "test-xtask");
    texts.extend(
        task["vars"]
            .as_mapping()
            .into_iter()
            .flatten()
            .map(|(_, value)| text(value).to_owned()),
    );
    let mut found = Vec::new();
    for command in &texts {
        for word in command.split_whitespace() {
            let name = word.split('=').next().unwrap_or("");
            let forbidden = [
                "RUSTC_WRAPPER",
                "RUSTC_WORKSPACE_WRAPPER",
                "CARGO_BUILD_RUSTC_WRAPPER",
                "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
            ]
            .contains(&name)
                || word.starts_with("CARGO_BUILD_JOBS")
                || word.starts_with("SCCACHE_")
                || word == "-j"
                || word == "--jobs";
            if forbidden {
                found.push(word.to_owned());
            }
        }
    }
    assert!(
        found.is_empty(),
        "`test-xtask` empties the host's compile wrapper or sets build jobs: {found:?}"
    );
}

#[test]
fn test_xtask_builds_before_it_runs() {
    let taskfile = yaml("Taskfile.yml");
    let commands = test_xtask_commands(&taskfile);
    assert_eq!(
        commands.len(),
        3,
        "`test-xtask` is a build line of `ess`, a test build line and a nextest run line: \
         {commands:?}"
    );
    // The unit tests run the `ess` built here, beside their executable: a test running
    // `cargo run` compiled the CLI inside the test, over the 60 s test budget.
    assert!(
        commands[0].contains("cargo build --package ess-cli --bin ess --locked"),
        "the first command does not build `ess`: {}",
        commands[0]
    );
    assert!(
        commands[1].trim_end().ends_with("--no-run"),
        "the second command does not end `--no-run`: {}",
        commands[1]
    );
    assert!(
        commands[2].contains("cargo nextest run") && commands[2].contains("--profile ci"),
        "the third command is not a nextest run under the `ci` profile: {}",
        commands[2]
    );
    assert!(
        !commands[2].contains("--no-run"),
        "the third command is the run and does not build only: {}",
        commands[2]
    );
    for command in &commands[1..] {
        assert!(
            command.contains("--package ess-xtask --locked"),
            "`--package ess-xtask --locked` is missing from: {command}"
        );
    }
    for command in &commands {
        for needed in [
            "RUSTUP_TOOLCHAIN=1.98.1",
            "-fuse-ld=lld",
            "CARGO_INCREMENTAL=0",
            "CARGO_PROFILE_DEV_DEBUG=0",
            "CARGO_PROFILE_TEST_DEBUG=0",
        ] {
            assert!(
                command.contains(needed),
                "`{needed}` is missing from: {command}"
            );
        }
    }
}

/// The words of the `task …` lines of `step`: a one-line step or a line of a script.
fn task_words(step: &Value) -> Vec<String> {
    text(&step["run"])
        .lines()
        .filter_map(|line| line.trim().strip_prefix("task "))
        .flat_map(|rest| {
            rest.split_whitespace()
                .take_while(|word| {
                    !word.contains('=') && !word.contains('$') && !word.starts_with('-')
                })
                .map(|word| word.trim_matches('"').to_owned())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The jobs of `workflow` that run a task reaching `target`, each with the index of its first
/// such step.
fn jobs_reaching(workflow: &Value, taskfile: &Value, target: &str) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    let Some(jobs) = workflow["jobs"].as_mapping() else {
        return found;
    };
    for (id, job) in jobs {
        for (index, step) in job["steps"].as_sequence().into_iter().flatten().enumerate() {
            let mut covered = BTreeSet::new();
            for task in task_words(step) {
                reached(taskfile, &task, &mut covered);
            }
            if covered.contains(target) {
                found.push((text(id).to_owned(), index));
                break;
            }
        }
    }
    found
}

/// Every job of `ci.yml` and `release.yml` that reaches `body-cap-go-server` installs Go with the
/// `test` job's pinned action and version, after the step that frees the runner's disk (which
/// deletes the hosted tool cache) and before the step that runs the task; and the task builds.
fn assert_the_go_server_has_its_go(ci: &Value, release: &Value, taskfile: &Value) {
    let builds = shell_commands(taskfile, "body-cap-go-server");
    assert!(
        builds.iter().any(|command| command.contains("go build")),
        "`body-cap-go-server` runs no `go build`: {builds:?}"
    );
    let (_, reference) = step_using(&ci["jobs"]["test"], "actions/setup-go")
        .expect("the `test` job of ci.yml installs Go");
    let mut found = 0;
    for (file, workflow) in [("ci.yml", ci), ("release.yml", release)] {
        for (id, task_step) in jobs_reaching(workflow, taskfile, "body-cap-go-server") {
            found += 1;
            let job = &workflow["jobs"][id.as_str()];
            let (go_step, go) = step_using(job, "actions/setup-go").unwrap_or_else(|| {
                panic!("`{id}` of {file} builds the Go server and installs no Go")
            });
            let pinned = text(&go["uses"]);
            let (_, commit) = pinned.split_once('@').expect("a ref");
            assert!(
                commit.len() >= 40 && commit[..40].chars().all(|c| c.is_ascii_hexdigit()),
                "`{id}` of {file} does not pin setup-go to a commit: {pinned}"
            );
            assert_eq!(
                pinned,
                text(&reference["uses"]),
                "`{id}` of {file} pins another setup-go than the `test` job"
            );
            assert_eq!(
                text(&go["with"]["go-version"]),
                text(&reference["with"]["go-version"]),
                "`{id}` of {file} installs another Go than the `test` job"
            );
            assert_eq!(
                go["with"]["cache"],
                Value::Bool(false),
                "`{id}` of {file} caches Go"
            );
            for (index, step) in job["steps"].as_sequence().unwrap().iter().enumerate() {
                if text(&step["name"]) == "Free runner disk" {
                    assert!(
                        go_step > index,
                        "`{id}` of {file} installs Go before it frees the disk, which deletes it"
                    );
                }
            }
            assert!(
                go_step < task_step,
                "`{id}` of {file} installs Go after the step that builds the Go server"
            );
        }
    }
    assert!(
        found >= 2,
        "{found} job(s) reach `body-cap-go-server`; build-tests and hotfix-tests are expected"
    );
}

#[test]
fn a_job_that_builds_the_go_server_installs_go_after_it_frees_the_disk() {
    let ci = yaml(".github/workflows/ci.yml");
    let release = yaml(".github/workflows/release.yml");
    let taskfile = yaml("Taskfile.yml");
    assert_the_go_server_has_its_go(&ci, &release, &taskfile);
    let jobs: Vec<String> = [&ci, &release]
        .iter()
        .flat_map(|workflow| jobs_reaching(workflow, &taskfile, "body-cap-go-server"))
        .map(|(id, _)| id)
        .collect();
    assert!(
        jobs.contains(&"build-tests".to_owned()) && jobs.contains(&"hotfix-tests".to_owned()),
        "{jobs:?}"
    );

    let caught = |ci: &Value, release: &Value, what: &str| {
        assert!(
            std::panic::catch_unwind(|| assert_the_go_server_has_its_go(ci, release, &taskfile))
                .is_err(),
            "{what} went unnoticed"
        );
    };
    let go_index = |job: &Value| step_using(job, "actions/setup-go").unwrap().0;

    let mut mutated = ci.clone();
    let at = go_index(&ci["jobs"]["build-tests"]);
    let removed = mutated["jobs"]["build-tests"]["steps"]
        .as_sequence_mut()
        .unwrap()
        .remove(at);
    caught(&mutated, &release, "build-tests without Go");

    let steps = mutated["jobs"]["build-tests"]["steps"]
        .as_sequence_mut()
        .unwrap();
    let disk = steps
        .iter()
        .position(|step| text(&step["name"]) == "Free runner disk")
        .unwrap();
    steps.insert(disk, removed);
    caught(&mutated, &release, "Go installed before the disk is freed");

    let mut mutated = release.clone();
    let at = go_index(&release["jobs"]["hotfix-tests"]);
    mutated["jobs"]["hotfix-tests"]["steps"]
        .as_sequence_mut()
        .unwrap()
        .remove(at);
    caught(&ci, &mutated, "hotfix-tests without Go");
}

#[test]
fn the_tutorial_network_is_set_by_the_slow_lane_and_by_no_test_step() {
    const VARIABLE: &str = "ESS_TUTORIAL_NETWORK";
    let sets = |env: &Value| {
        env.as_mapping()
            .is_some_and(|env| env.keys().any(|key| key.as_str() == Some(VARIABLE)))
    };
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let mut offenders = Vec::new();
    for (name, job) in ci["jobs"].as_mapping().expect("ci.yml has jobs") {
        if sets(&job["env"]) {
            offenders.push(format!("{}: job env", text(name)));
        }
        for step in job["steps"].as_sequence().into_iter().flatten() {
            if sets(&step["env"]) {
                offenders.push(format!("{}: {}", text(name), text(&step["name"])));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "{VARIABLE} is set outside the slow lane: {offenders:?}"
    );
    assert_eq!(
        taskfile["tasks"]["test-slow"]["env"][VARIABLE].as_str(),
        Some("1"),
        "`test-slow` does not set {VARIABLE}: \"1\""
    );
}

/// The condition of the `slow` job: it runs unless the deciding job said no changed path reaches a
/// slow test.
const SLOW_IF: &str = "${{ needs.diff.outputs.slow != 'false' }}";

fn job_steps(job: &Value) -> Vec<&Value> {
    job["steps"].as_sequence().into_iter().flatten().collect()
}

fn assert_the_slow_job_is_gated_bounded_and_runs_the_slow_task(ci: &Value) {
    let job = &ci["jobs"]["slow"];
    assert!(job.is_mapping(), "ci.yml has no slow job");
    assert_eq!(
        needs_of(job),
        BTreeSet::from(["diff".to_owned()]),
        "`slow` waits for something besides `diff`"
    );
    assert_eq!(text(&job["if"]), SLOW_IF, "`slow` has the wrong condition");
    assert!(
        !text(&job["if"]).contains("plan-only"),
        "`slow` is skipped on a plan-only diff"
    );
    assert_eq!(job["timeout-minutes"].as_u64(), Some(30));
    assert!(
        job_steps(job)
            .iter()
            .any(|step| text(&step["run"]).trim() == "task test-slow"),
        "`slow` runs no `task test-slow` step"
    );
    for variable in ["CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_DEBUG"] {
        assert_eq!(scalar_or_empty(&job["env"][variable]), "0", "{variable}");
    }
    let gate = &ci["jobs"]["gate"];
    assert!(needs_of(gate).contains("slow"), "Gate does not need `slow`");
    let read: String = job_steps(gate)
        .iter()
        .map(|step| ess_yaml::to_string(step).unwrap())
        .collect();
    assert!(
        read.contains("needs.slow.result"),
        "Gate never reads the result of `slow`"
    );
}

/// The `slow` job runs `task test-slow` inside its bound and is a gated job `Gate` reads; losing
/// the condition, the bound, the step or the place in `Gate`'s `needs` fails here.
#[test]
fn the_slow_job_is_gated_bounded_and_runs_the_slow_task() {
    let ci = yaml(".github/workflows/ci.yml");
    assert_the_slow_job_is_gated_bounded_and_runs_the_slow_task(&ci);
    for (output, runs) in [("false", false), ("true", true), ("", true)] {
        let context = BTreeMap::from([("needs.diff.outputs.slow", output)]);
        assert_eq!(
            evaluate(SLOW_IF, &context).truthy(),
            runs,
            "a decision of `{output}`"
        );
    }

    let mut no_if = ci.clone();
    no_if["jobs"]["slow"]["if"] = Value::Null;
    let mut no_bound = ci.clone();
    no_bound["jobs"]["slow"]["timeout-minutes"] = Value::Null;
    let mut no_step = ci.clone();
    for step in no_step["jobs"]["slow"]["steps"].as_sequence_mut().unwrap() {
        if text(&step["run"]) == "task test-slow" {
            step["run"] = Value::String("task test-xtask".to_owned());
        }
    }
    let mut no_need = ci.clone();
    no_need["jobs"]["gate"]["needs"] = Value::Sequence(
        needs_of(&ci["jobs"]["gate"])
            .into_iter()
            .filter(|need| need != "slow")
            .map(Value::String)
            .collect(),
    );
    for (name, mutant) in [
        ("no condition", no_if),
        ("no bound", no_bound),
        ("no task step", no_step),
        ("not in Gate", no_need),
    ] {
        assert!(
            std::panic::catch_unwind(|| {
                assert_the_slow_job_is_gated_bounded_and_runs_the_slow_task(&mutant);
            })
            .is_err(),
            "a slow job with {name} passes"
        );
    }
}

fn slow_decision_step(ci: &Value) -> &Value {
    job_steps(&ci["jobs"][PLAN_ONLY_JOB])
        .into_iter()
        .find(|step| text(&step["id"]) == "slow")
        .expect("the deciding job has no `slow` step")
}

/// The `slow` decision is made on the same runs as the plan-only one, a pull request of an
/// ordinary head or a fork, and on no other.
#[test]
fn the_slow_decision_may_skip_only_a_pull_request_of_an_ordinary_head() {
    let ci = yaml(".github/workflows/ci.yml");
    assert_eq!(
        text(&ci["jobs"][PLAN_ONLY_JOB]["outputs"]["slow"]),
        "${{ steps.slow.outputs.slow }}",
        "`{PLAN_ONLY_JOB}` reports a `slow` decision its `slow` step did not make"
    );
    assert_eq!(
        text(&slow_decision_step(&ci)["if"]),
        PLAN_ONLY_STEP_IF,
        "the `slow` step runs on a run that may not skip"
    );
    let wrong: Vec<String> = candidate_runs()
        .iter()
        .filter(|&&(event, head_ref, repository, checkout, may)| {
            plan_only_candidate(&ci, &ci_context(event, head_ref, repository, checkout)) != may
        })
        .map(|run| format!("{run:?}"))
        .collect();
    assert!(
        wrong.is_empty(),
        "the slow decision may run wrongly on: {wrong:?}"
    );
}

/// No run that `prior-gate` accepts as a release's gate is a run that may skip the slow lane.
#[test]
fn no_run_that_stands_in_for_a_release_can_skip_the_slow_lane() {
    let ci = yaml(".github/workflows/ci.yml");
    let release = yaml(".github/workflows/release.yml");
    assert_eq!(
        text(&slow_decision_step(&ci)["if"]),
        PLAN_ONLY_STEP_IF,
        "the `slow` step runs on a run that may not skip"
    );
    let case = Scratch::new("slow-qualifies");
    let function = prior_gate_qualifies(&release);
    let (mut qualifying, mut skippable) = (0, 0);
    let mut stands_in = Vec::new();
    for (index, (event, branch, repository)) in described_runs().into_iter().enumerate() {
        let run = workflow_run(
            ".github/workflows/ci.yml",
            event,
            Some(branch),
            repository,
            RUN_COMMIT,
        );
        let qualifies = run_prior_gate_qualifies(&case.0, &function, 200 + index, &run, RUN_COMMIT);
        let (head_ref, head_repository) = if event == "pull_request" {
            (branch, repository)
        } else {
            ("", "")
        };
        let may_skip = plan_only_candidate(&ci, &ci_context(event, head_ref, head_repository, ""));
        qualifying += usize::from(qualifies);
        skippable += usize::from(may_skip);
        if qualifies && may_skip {
            stands_in.push(format!("{event} {branch} {repository}"));
        }
    }
    assert!(
        qualifying > 0 && skippable > 0,
        "the case reads nothing: {qualifying} qualifying runs, {skippable} that may skip"
    );
    assert!(
        stands_in.is_empty(),
        "a run that may skip the slow lane stands in for a release's gate: {stands_in:?}"
    );
}

/// The slow decision is `false` only when every changed path is in the skippable list and none is
/// one a slow test reads, read from the merge commit against its first parent, as plan-only is.
/// Anything the step cannot read, an empty change included, runs the lane.
#[test]
#[allow(clippy::too_many_lines)]
fn the_slow_decision_is_every_changed_path_in_the_skippable_list() {
    let case = Scratch::new("slow-decision");
    let origin = case.0.join("origin");
    fs::create_dir_all(&origin).unwrap();
    git(&origin, &["init", "--quiet", "--initial-branch=main"]);
    for (path, contents) in [
        ("crates/lib.rs", "lib"),
        ("AGENTS.md", "agents"),
        ("docs/x.md", "docs"),
        ("website/docs/other.md", "other"),
        ("website/docs/start/install.md", "install"),
        (".engineering/repro/501/system.yaml", "repro"),
    ] {
        let file = origin.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, contents).unwrap();
    }
    git(&origin, &["add", "--all"]);
    git(&origin, &["commit", "--quiet", "--message", "base"]);
    let base = git(&origin, &["rev-parse", "HEAD"]);
    fs::write(origin.join("crates/b.rs"), "b").unwrap();
    git(&origin, &["add", "--all"]);
    git(
        &origin,
        &["commit", "--quiet", "--message", "main moves on"],
    );

    let cases: &[(&str, &[&str], &str)] = &[
        ("docs only", &["docs/x.md"], "false"),
        ("AGENTS.md only", &["AGENTS.md"], "false"),
        (
            "a UI path and docs",
            &["crates/ui/ess-ui/src/lib.rs", "docs/x.md"],
            "false",
        ),
        (
            "an infrastructure path",
            &["crates/infra/infra-spec/src/lib.rs"],
            "false",
        ),
        (
            "an ess-xtask test",
            &["crates/edge/ess-xtask/tests/layout.rs"],
            "false",
        ),
        (
            "an ess-xtask source slow-list does not run on",
            &["crates/edge/ess-xtask/src/support.rs"],
            "false",
        ),
        (
            "the dispatcher of slow-list",
            &["crates/edge/ess-xtask/src/main.rs"],
            "true",
        ),
        (
            "the manifest that builds slow-list",
            &["crates/edge/ess-xtask/Cargo.toml"],
            "true",
        ),
        ("a release note", &["release-notes/0.99.0.md"], "false"),
        (
            "a plan story",
            &[".engineering/planning/story/x.md"],
            "false",
        ),
        (
            "a verifier path",
            &["crates/verify/ess-diff/src/lib.rs"],
            "true",
        ),
        (
            "a reproduction a test reads",
            &[".engineering/repro/501/system.yaml"],
            "true",
        ),
        ("a start page", &["website/docs/start/install.md"], "true"),
        ("another site page", &["website/docs/other.md"], "false"),
        (
            "the slow-list reader",
            &["crates/edge/ess-xtask/src/slow_lane.rs"],
            "true",
        ),
        ("the workflow", &[".github/workflows/ci.yml"], "true"),
        ("the Taskfile", &["Taskfile.yml"], "true"),
        ("the lock file", &["Cargo.lock"], "true"),
        (
            "a generator path",
            &["crates/generate/ess-gen/src/lib.rs"],
            "true",
        ),
        ("an example", &["examples/a.yaml"], "true"),
        (
            "a skippable and another path",
            &["docs/x.md", "crates/lib.rs"],
            "true",
        ),
        ("no change", &[], "true"),
    ];

    let mut wrong = Vec::new();
    for (index, (name, paths, expected)) in cases.iter().enumerate() {
        let branch = format!("case-{index}");
        git(&origin, &["checkout", "--quiet", "-b", &branch, &base]);
        for path in *paths {
            let file = origin.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, format!("changed by {name}")).unwrap();
        }
        git(&origin, &["add", "--all"]);
        git(
            &origin,
            &["commit", "--quiet", "--allow-empty", "--message", name],
        );
        git(&origin, &["checkout", "--quiet", "--detach", "main"]);
        git(
            &origin,
            &["merge", "--quiet", "--no-ff", "--message", name, &branch],
        );
        git(&origin, &["branch", &format!("merge-{index}"), "HEAD"]);
        let checkout = plan_only_checkout(&case.0, &origin, &format!("merge-{index}"), 2);
        let (decision, log) = run_plan_only_step(&case.0, &checkout, "slow");
        if decision != *expected {
            wrong.push(format!("{name}: {decision}, expected {expected}\n{log}"));
        }
    }

    let shallow = plan_only_checkout(&case.0, &origin, "merge-0", 1);
    let plain = plan_only_checkout(&case.0, &origin, "case-0", 2);
    let empty = case.0.join("no-repository");
    fs::create_dir_all(&empty).unwrap();
    for (name, checkout) in [
        ("a checkout without the base", shallow),
        ("a head that is no merge commit", plain),
        ("no repository", empty),
    ] {
        let (decision, log) = run_plan_only_step(&case.0, &checkout, "slow");
        if decision != "true" {
            wrong.push(format!("{name}: {decision}, expected true\n{log}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "the slow step decides wrongly:\n{}",
        wrong.join("\n")
    );
}

/// `Gate` passes a skipped `slow` job only when `diff` succeeded and said `false`, and says so.
#[test]
fn the_gate_passes_a_skipped_slow_job_only_when_the_diff_said_so() {
    let ci = yaml(".github/workflows/ci.yml");
    let needs = needs_of(&ci["jobs"]["gate"]);
    assert!(needs.contains("slow"), "Gate does not need `slow`");
    let case = Scratch::new("slow-gate");
    let with = |changes: &[(&str, &str)]| {
        let mut results: BTreeMap<String, String> = needs
            .iter()
            .map(|need| (need.clone(), "success".to_owned()))
            .collect();
        for (job, result) in changes {
            results.insert((*job).to_owned(), (*result).to_owned());
        }
        results
    };
    let mut wrong = Vec::new();
    let mut expect = |name: &str, changes: &[(&str, &str)], slow: &str, pass: bool| {
        let (passed, log) = run_gate(&case.0, &with(changes), "", slow);
        if passed != pass {
            wrong.push(format!("{name}: passed {passed}, expected {pass}\n{log}"));
        }
        log
    };
    expect("every job passed", &[], "false", true);
    expect("slow ran on a `true` decision", &[], "true", true);
    let log = expect(
        "slow skipped on `false`",
        &[("slow", "skipped")],
        "false",
        true,
    );
    assert!(
        log.contains("the slow job was skipped because no changed path can reach a slow test"),
        "Gate does not say why `slow` was skipped: {log}"
    );
    expect(
        "slow skipped without a decision",
        &[("slow", "skipped")],
        "",
        false,
    );
    expect(
        "slow skipped on `true`",
        &[("slow", "skipped")],
        "true",
        false,
    );
    for result in ["failure", "cancelled"] {
        expect(
            &format!("slow {result} on `false`"),
            &[("slow", result)],
            "false",
            false,
        );
    }
    for result in ["failure", "cancelled", "skipped"] {
        expect(
            &format!("diff {result} and slow skipped on `false`"),
            &[("slow", "skipped"), ("diff", result)],
            "false",
            false,
        );
    }
    assert!(
        wrong.is_empty(),
        "Gate decides wrongly:\n{}",
        wrong.join("\n")
    );
}

/// `task check` runs the slow lane directly after the tests, as the forge does.
#[test]
fn the_check_task_runs_the_slow_lane_after_the_tests() {
    let taskfile = yaml("Taskfile.yml");
    let check = direct_subtasks(&taskfile, "check");
    let test = check
        .iter()
        .position(|step| step == "test")
        .expect("`task check` runs no `test`");
    assert_eq!(
        check.get(test + 1).map(String::as_str),
        Some("test-slow"),
        "`task check` does not run `test-slow` directly after `test`: {check:?}"
    );
}

// ---- hotfix-scope.sh: how much of the gate a patch tag runs ------------------------------------

/// What `.github/scripts/hotfix-scope.sh` prints for the patch tag `1.0.1` of a small workspace
/// whose change since `1.0.0` is `changes`: crates `low`, `high` (depending on `low`), `ess-ui`,
/// `ess-conformance`, `ess-gen` with its dependent `ess-cli`, and `ess-service-contract` with its
/// dependent `ess-entity-runtime`, with a `cargo` stub on `PATH` that answers `metadata` with their
/// JSON, so no cargo runs.
fn hotfix_scope(case: &str, changes: &[(&str, &str)]) -> BTreeMap<String, String> {
    let scratch = Scratch::new(&format!("hotfix-scope-{case}"));
    let repo = scratch.0.join("repo");
    let bin = scratch.0.join("bin");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&bin).unwrap();
    let crates = [
        ("low", "crates/low", None),
        ("high", "crates/high", Some("low")),
        ("ess-ui", "crates/ui/ess-ui", None),
        ("ess-conformance", "crates/verify/ess-conformance", None),
        ("ess-gen", "crates/generate/ess-gen", None),
        ("ess-cli", "crates/edge/ess-cli", Some("ess-gen")),
        (
            "ess-service-contract",
            "crates/specify/ess-service-contract",
            None,
        ),
        (
            "ess-entity-runtime",
            "crates/generate/ess-entity-runtime",
            Some("ess-service-contract"),
        ),
    ];
    let write = |path: &str, contents: &str| {
        let file = repo.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, contents).unwrap();
    };
    let members: Vec<String> = crates
        .iter()
        .map(|(_, dir, _)| format!("\"{dir}\""))
        .collect();
    write(
        "Cargo.toml",
        &format!("[workspace]\nmembers = [{}]\n", members.join(", ")),
    );
    write("CHANGELOG.md", "# Changelog\n");
    write("schemas/ui/ess-ui.schema.yaml", "type: object\n");
    let mut packages = Vec::new();
    for (name, dir, depends) in crates {
        write(
            &format!("{dir}/Cargo.toml"),
            &format!("[package]\nname = \"{name}\"\nedition = \"2021\"\n"),
        );
        write(&format!("{dir}/src/lib.rs"), "pub fn one() -> u8 { 1 }\n");
        let dependencies: Vec<serde_json::Value> = depends
            .map(|on| {
                let path = crates.iter().find(|(name, _, _)| *name == on).unwrap().1;
                json!({"name": on, "path": repo.join(path).display().to_string()})
            })
            .into_iter()
            .collect();
        packages.push(json!({
            "name": name,
            "manifest_path": repo.join(dir).join("Cargo.toml").display().to_string(),
            "dependencies": dependencies,
        }));
    }
    let metadata = json!({"workspace_root": repo.display().to_string(), "packages": packages});
    let answer = scratch.0.join("metadata.json");
    fs::write(&answer, metadata.to_string()).unwrap();
    executable::install_bytes(
        &bin.join("cargo"),
        format!("#!/bin/sh\nexec cat '{}'\n", answer.display()).as_bytes(),
    )
    .unwrap();
    git(&repo, &["init", "--quiet", "--initial-branch", "main"]);
    git(&repo, &["add", "--all"]);
    git(&repo, &["commit", "--quiet", "--message", "1.0.0"]);
    git(&repo, &["tag", "1.0.0"]);
    for (path, contents) in changes {
        write(path, contents);
    }
    git(&repo, &["add", "--all"]);
    git(&repo, &["commit", "--quiet", "--message", "1.0.1"]);
    git(&repo, &["tag", "1.0.1"]);
    let script = workspace_root().join(".github/scripts/hotfix-scope.sh");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = isolated("bash", &repo)
        .arg(&script)
        .arg("1.0.1")
        .env("PATH", path)
        .env_remove("GITHUB_OUTPUT")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "hotfix-scope.sh {case}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

/// Every hotfix carries `CHANGELOG.md` and `release-notes/<version>.md` (`AGENTS.md` § Hotfix): a
/// patch tag of one crate's code with those two runs the targeted gate, the crate and its direct
/// dependents; a file outside every crate and outside the prose list still runs the full gate.
#[test]
fn a_patch_tag_with_its_release_notes_runs_the_targeted_gate() {
    let fix = [
        ("crates/low/src/lib.rs", "pub fn one() -> u8 { 2 }\n"),
        ("CHANGELOG.md", "# Changelog\n\n## [1.0.1]\n\n- Fixed.\n"),
        ("release-notes/1.0.1.md", "A fix.\n"),
    ];
    let targeted = hotfix_scope("notes", &fix);
    assert_eq!(targeted["scope"], "targeted", "{targeted:?}");
    assert_eq!(targeted["packages"], "high low", "{targeted:?}");
    let mut stray = fix.to_vec();
    stray.push(("tools/stray.txt", "a file of no crate\n"));
    let full = hotfix_scope("stray", &stray);
    assert_eq!(full["scope"], "full", "{full:?}");
    assert!(full["reason"].contains("tools/stray.txt"), "{full:?}");
}

/// `ess-ui` embeds `schemas/ui/ess-ui.schema.yaml`, so a patch tag changing the schema runs the
/// targeted gate of `ess-ui` and its dependents, not the full gate.
#[test]
fn a_patch_tag_of_the_ess_ui_schema_runs_the_targeted_gate_of_ess_ui() {
    let scope = hotfix_scope(
        "schema",
        &[
            (
                "schemas/ui/ess-ui.schema.yaml",
                "type: object\nrequired: []\n",
            ),
            ("CHANGELOG.md", "# Changelog\n\n## [1.0.1]\n\n- Fixed.\n"),
            ("release-notes/1.0.1.md", "A fix.\n"),
        ],
    );
    assert_eq!(scope["scope"], "targeted", "{scope:?}");
    assert!(
        scope["packages"]
            .split(' ')
            .any(|package| package == "ess-ui"),
        "{scope:?}"
    );
}

/// 0.99.3 changed `ess-conformance`; its targeted job selected `ess-cli`, `ess-conformance` and
/// `ess-synth` among seven crates and was cancelled at its 60-minute limit with 2,734 of 8,685
/// tests run. A patch tag whose selected set holds one of those three runs the full gate, and the
/// reason names the crate, whether the tag changed it or it was selected as a direct dependent.
#[test]
fn a_patch_tag_that_selects_a_large_crate_runs_the_full_gate() {
    let notes = [
        ("CHANGELOG.md", "# Changelog\n\n## [1.0.1]\n\n- Fixed.\n"),
        ("release-notes/1.0.1.md", "A fix.\n"),
    ];
    let mut changed = notes.to_vec();
    changed.push((
        "crates/verify/ess-conformance/src/lib.rs",
        "pub fn one() -> u8 { 2 }\n",
    ));
    let scope = hotfix_scope("large-changed", &changed);
    assert_eq!(scope["scope"], "full", "{scope:?}");
    assert!(scope["reason"].contains("ess-conformance"), "{scope:?}");
    let mut dependent = notes.to_vec();
    dependent.push((
        "crates/generate/ess-gen/src/lib.rs",
        "pub fn one() -> u8 { 2 }\n",
    ));
    let scope = hotfix_scope("large-dependent", &dependent);
    assert_eq!(scope["scope"], "full", "{scope:?}");
    assert!(scope["reason"].contains("ess-cli"), "{scope:?}");
}

/// A patch tag of a crate whose selected set holds none of the three large crates keeps the
/// targeted gate: `ess-service-contract` and its dependent `ess-entity-runtime`.
#[test]
fn a_patch_tag_that_selects_no_large_crate_stays_targeted() {
    let scope = hotfix_scope(
        "small",
        &[
            (
                "crates/specify/ess-service-contract/src/lib.rs",
                "pub fn one() -> u8 { 2 }\n",
            ),
            ("CHANGELOG.md", "# Changelog\n\n## [1.0.1]\n\n- Fixed.\n"),
            ("release-notes/1.0.1.md", "A fix.\n"),
        ],
    );
    assert_eq!(scope["scope"], "targeted", "{scope:?}");
    assert_eq!(
        scope["packages"], "ess-entity-runtime ess-service-contract",
        "{scope:?}"
    );
}
