//! The pull-request gate is split into parallel jobs, and the split is held to `task check`.
//!
//! `.github/workflows/ci.yml` does not run `task check` as one job. A few jobs each run Taskfile
//! tasks as steps: `checks` runs the static, smoke, xtask and fuzz tasks one after another; one job
//! compiles the test binaries once into nextest archives; a matrix of test shards downloads them and
//! runs one partition each; and a job named `Gate` aggregates the result. A split like that has one
//! silent failure mode — a step of `task check` that no job calls exits nowhere, so nothing turns
//! red when it goes missing. The coverage case below reads both files and names the step instead.
//!
//! The release workflow reuses a green `Gate` rather than running the gate a second time: the one
//! on the exact tagged commit, or the one on the head of the pull request whose merge commit is the
//! tagged commit when that head has the tagged tree. The reuse is a condition in two jobs and a
//! script in a third; the release cases below hold the conditions, and run the script itself
//! against a real Git history and a stand-in for `gh`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(dead_code)]
#[path = "../../ess-cli/tests/support/executable.rs"]
mod executable;

use serde_json::json;
use serde_yaml::Value;

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
    serde_yaml::from_str(&text).unwrap_or_else(|error| panic!("parsing {relative}: {error}"))
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
        for row in matrix_rows(job) {
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

/// The archives select what `task test` selects, one task each, so that two jobs can build them
/// side by side; local `task test-archive` still builds both.
fn assert_each_archive_selects_what_task_test_selects(taskfile: &Value) {
    assert_eq!(
        direct_subtasks(taskfile, "test-archive"),
        ["test-archive-workspace", "test-archive-feature-off"],
        "`task test-archive` no longer builds both archives"
    );
    let archive = shell_commands(taskfile, "test-archive-workspace").join("\n");
    assert!(
        archive.contains(
            "cargo nextest archive --workspace --exclude ess-xtask --locked \
             --archive-file {{.NEXTEST_ARCHIVES}}/workspace.tar.zst"
        ),
        "{archive}"
    );
    assert!(
        !archive.contains("/feature-off.tar.zst"),
        "the workspace archive task also builds the feature-off archive: {archive}"
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

#[test]
fn the_workspace_shards_are_one_complete_partition() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let workspace = assert_complete_partition(&ci, "test-shard");
    let feature_off = assert_complete_partition(&ci, "test-feature-off");
    assert_eq!(
        workspace, feature_off,
        "the feature-off partitions are not the workspace partitions, so one shard job runs a \
         partition of one archive and not of the other"
    );

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
    for (task, file) in [
        ("test-shard", "workspace"),
        ("test-feature-off", "feature-off"),
    ] {
        let commands = shell_commands(&taskfile, task);
        let [run] = commands.as_slice() else {
            panic!(
                "`{task}` runs {} commands, not one: {commands:?}",
                commands.len()
            )
        };
        assert!(
            run.starts_with(&format!(
                "cargo nextest run --archive-file {{{{.NEXTEST_ARCHIVES}}}}/{file}.tar.zst"
            )),
            "`{task}` does not run the {file} archive: {run}"
        );
        for (flag, value) in [
            ("--workspace-remap", "."),
            ("--extract-to", "."),
            ("--partition", "slice:{{.SHARD}}"),
        ] {
            assert!(
                passes(run, flag, value),
                "`{task}` lacks `{flag} {value}`: {run}"
            );
        }
        for flag in ["--extract-overwrite", "--no-fail-fast"] {
            assert!(
                run.split_whitespace().any(|word| word == flag),
                "`{task}` lacks `{flag}`: {run}"
            );
        }
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
        .map(|step| serde_yaml::to_string(step).unwrap())
        .collect();
    for need in &needs {
        assert!(
            checked.contains(&format!("needs.{need}.result")),
            "Gate never reads the result of `{need}`"
        );
    }

    for id in gated_jobs(&ci) {
        assert_eq!(
            ci["jobs"][id.as_str()]["timeout-minutes"].as_u64(),
            Some(30),
            "`{id}` is not bounded at 30 minutes"
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

/// A pull request runs the feature-off build — the packages without `arbitrary_precision` —
/// on the code whose number semantics the feature changes, and every other run of ci.yml (the
/// `main` push, the nightly schedule, the release gate) runs all of it. Running every ess-cli
/// case twice was about 1,900 of the 3,935 test-seconds of one pull-request run.
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
        serde_yaml::from_str::<Value>("[checks_requested]").unwrap(),
        "ci.yml does not run in the merge queue, so the queue can never see a Gate"
    );
    assert_eq!(
        on["push"]["branches"],
        serde_yaml::from_str::<Value>("[main, 'queue/**']").unwrap(),
        "ci.yml no longer runs on every push to main and to the bot merge queue"
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
        "${{ (github.event_name == 'pull_request' || github.event_name == 'merge_group' || startsWith(github.ref, 'refs/heads/queue/')) && 'number-semantics' || 'full' }}",
        "only a pull request or its merge-queue run may narrow the feature-off archive"
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
#[test]
fn the_archive_jobs_build_the_test_binaries_without_debug_information() {
    let ci = yaml(".github/workflows/ci.yml");
    let builders: BTreeSet<String> = lane_invocations(&ci)
        .into_iter()
        .filter(|lane| lane.task.starts_with("test-archive"))
        .map(|lane| lane.job)
        .collect();
    assert!(
        !builders.is_empty(),
        "no job of ci.yml builds a test archive"
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

fn prior_gate_step(release: &Value) -> &Value {
    release["jobs"]["resolve"]["steps"]
        .as_sequence()
        .expect("resolve steps")
        .iter()
        .find(|step| text(&step["id"]) == "prior-gate")
        .expect("resolve looks for a prior Gate")
}

#[test]
fn a_release_reuses_a_green_gate_on_the_exact_tagged_commit_and_otherwise_runs_it() {
    let ci = yaml(".github/workflows/ci.yml");
    let release = yaml(".github/workflows/release.yml");
    let jobs = &release["jobs"];
    let check = text(&ci["jobs"]["gate"]["name"]);

    let resolve = &jobs["resolve"];
    assert_eq!(text(&resolve["permissions"]["checks"]), "read");
    assert_eq!(text(&resolve["permissions"]["contents"]), "read");
    // The pull requests a commit belongs to are read through the pull-requests scope.
    assert_eq!(text(&resolve["permissions"]["pull-requests"]), "read");
    assert_eq!(
        text(&resolve["outputs"]["gate-passed"]),
        "${{ steps.prior-gate.outputs.passed }}"
    );
    let query = prior_gate_step(&release);
    let run = text(&query["run"]);
    assert_eq!(text(&query["shell"]), "bash");
    assert_eq!(
        text(&query["env"]["COMMIT"]),
        "${{ steps.release.outputs.commit }}"
    );
    assert!(run.contains("/check-runs?"), "{run}");
    assert!(run.contains(&format!("check_name={check}")), "{run}");
    assert!(run.contains(&format!(".name == \"{check}\"")), "{run}");
    assert!(run.contains(".app.slug == \"github-actions\""), "{run}");
    // Only a completed success counts; every other answer, including an API failure, runs it.
    assert!(run.contains("\"completed/success\""), "{run}");
    assert!(run.contains("decide false"), "{run}");

    assert_eq!(
        text(&jobs["gate"]["if"]),
        "${{ needs.resolve.outputs.gate-passed != 'true' }}"
    );
    let publish = text(&jobs["release"]["if"]);
    assert!(
        publish.contains(
            "(needs.gate.result == 'success' || (needs.gate.result == 'skipped' && \
             needs.resolve.outputs.gate-passed == 'true'))"
        ),
        "publication accepts a skipped gate without a green prior Gate: {publish}"
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

/// The histories the reuse rule has to tell apart, each ending in the commit a tag would name.
struct History {
    /// `main` merged an up-to-date branch: the merge's tree is the head's, and `main` before the
    /// merge is in the head's history.
    fresh: (String, String),
    /// `main` moved after the branch left it: the merge carries a change the head never had.
    stale: (String, String),
    /// `main` and the branch made the same change independently: the trees agree, but `main`
    /// before the merge is not in the head's history, so no pull-request run tested this tree.
    twin: (String, String),
    /// A squash merge of an up-to-date branch: one parent, the head's tree, and a head that only
    /// the pull request's ref holds, so the checkout has to fetch it.
    squash: (String, String),
    /// A merge commit whose parents are `main` and an up-to-date head but whose tree carries a
    /// file the head never had: the ancestry holds and only the tree comparison refuses it.
    forged: (String, String),
    origin: PathBuf,
}

fn history(root: &Path) -> History {
    let origin = root.join("origin");
    fs::create_dir_all(&origin).unwrap();
    git(&origin, &["init", "--quiet", "--initial-branch", "main"]);
    // GitHub serves any reachable commit by id; a local upload-pack does only when told to.
    git(
        &origin,
        &["config", "uploadpack.allowAnySHA1InWant", "true"],
    );
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
    // The squashed branch survives only as a ref the release checkout does not clone.
    git(&origin, &["update-ref", "refs/pull/4/head", &squash_head]);
    git(&origin, &["branch", "--quiet", "-D", "squash"]);

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

fn gate_run(status: &str, conclusion: Option<&str>, app: &str) -> serde_json::Value {
    json!({ "check_runs": [
        { "name": "Other", "app": { "slug": "github-actions" }, "status": "completed",
          "conclusion": "success", "started_at": "2026-01-01T00:00:09Z",
          "html_url": "https://example.invalid/other" },
        { "name": "Gate", "app": { "slug": "github-actions" }, "status": "completed",
          "conclusion": "failure", "started_at": "2026-01-01T00:00:00Z",
          "html_url": "https://example.invalid/older" },
        { "name": "Gate", "app": { "slug": app }, "status": status, "conclusion": conclusion,
          "started_at": "2026-01-01T00:00:05Z", "html_url": "https://example.invalid/newest" },
    ]})
}

fn pull(number: u64, merged: bool, merge_commit: &str, head: &str) -> serde_json::Value {
    json!({
        "number": number,
        "merged_at": if merged { json!("2026-01-01T00:00:00Z") } else { json!(null) },
        "merge_commit_sha": merge_commit,
        "head": { "sha": head },
    })
}

/// Stands in for `gh api <path> --jq <filter>`: answers from `<sha>.<endpoint>.json` in
/// `$GH_STUB_RESPONSES` through the real `jq`, and fails like an HTTP error when there is none.
const GH_STUB: &str = r#"#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -ne 4 ] || [ "$1" != api ] || [ "$3" != --jq ]; then
  echo "gh stub: unexpected arguments: $*" >&2
  exit 2
fi
IFS=/ read -r _ _ _ _ sha endpoint <<< "$2"
file="$GH_STUB_RESPONSES/$sha.${endpoint%%\?*}.json"
if [ ! -f "$file" ]; then
  echo "gh: HTTP 404: Not Found (https://api.github.com/$2)" >&2
  exit 1
fi
exec jq -r "$4" "$file"
"#;

/// A `gh` stub this suite installs runs at once, even while other cases fork children.
///
/// `prior_gate` and `prebuilt` install a `gh` stub and the step they run executes it through
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

/// Runs release.yml's `prior-gate` step the way Actions runs a `shell: bash` step, in a fresh
/// clone of `main`, and returns the `passed` value it wrote.
fn prior_gate(
    root: &Path,
    index: usize,
    history: &History,
    commit: &str,
    responses: &[(&str, &str, serde_json::Value)],
) -> String {
    let release = yaml(".github/workflows/release.yml");
    let step = prior_gate_step(&release);
    for name in ["GH_TOKEN", "REPOSITORY", "COMMIT"] {
        assert!(
            !step["env"][name].is_null(),
            "the prior-gate step no longer receives {name}; this harness supplies it"
        );
    }
    let case = &root.join(format!("case-{index}"));
    let bin = case.join("bin");
    let answers = case.join("responses");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&answers).unwrap();
    let stub = bin.join("gh");
    executable::install_bytes(&stub, GH_STUB.as_bytes()).unwrap();
    for (sha, endpoint, body) in responses {
        fs::write(
            answers.join(format!("{sha}.{endpoint}.json")),
            body.to_string(),
        )
        .unwrap();
    }
    let checkout = case.join("checkout");
    let origin = history.origin.to_str().unwrap();
    git(
        root,
        &[
            "clone",
            "--quiet",
            "--no-local",
            "--single-branch",
            "--branch",
            "main",
            "--no-tags",
            origin,
            checkout.to_str().unwrap(),
        ],
    );
    let script = case.join("prior-gate.sh");
    fs::write(&script, text(&step["run"])).unwrap();
    let output_file = case.join("github-output");
    fs::write(&output_file, "").unwrap();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = isolated("bash", &checkout)
        .args(["--noprofile", "--norc", "-eo", "pipefail"])
        .arg(&script)
        .env("PATH", path)
        .env("GH_TOKEN", "fixture")
        .env("REPOSITORY", "owner/repository")
        .env("COMMIT", commit)
        .env("GITHUB_OUTPUT", &output_file)
        .env("GH_STUB_RESPONSES", &answers)
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "the prior-gate step failed the resolve job instead of deciding: {log}"
    );
    let written = fs::read_to_string(&output_file).unwrap();
    let decisions: Vec<&str> = written
        .lines()
        .filter_map(|line| line.strip_prefix("passed="))
        .collect();
    let [decision] = decisions.as_slice() else {
        panic!("the prior-gate step wrote {decisions:?}, not one decision: {log}")
    };
    format!("{decision}\n{log}")
}

/// Canned `gh` answers: the commit, the endpoint, the JSON body.
type Responses<'a> = Vec<(&'a str, &'a str, serde_json::Value)>;
/// One history the rule is asked about: its name, the tagged commit, the answers, the decision.
type Case<'a> = (&'a str, &'a str, Responses<'a>, &'a str);

/// The gate counts as passed when the tagged commit carries a green `Gate`, or when it is the
/// merge commit of exactly one merged pull request whose head has the tagged tree, whose head
/// already contained the `main` it merged into, and whose head carries a green `Gate`. Every
/// other answer — and every failure on the way — runs the gate.
#[test]
// One table of cases, which reads best whole.
#[allow(clippy::too_many_lines)]
fn a_release_reuses_the_gate_of_a_merged_pull_request_only_when_it_tested_the_tagged_tree() {
    let root = Scratch::new("release-reuse");
    let history = history(&root.0);
    let (fresh, fresh_head) = (history.fresh.0.as_str(), history.fresh.1.as_str());
    let (stale, stale_head) = (history.stale.0.as_str(), history.stale.1.as_str());
    let (twin, twin_head) = (history.twin.0.as_str(), history.twin.1.as_str());
    let (squash, squash_head) = (history.squash.0.as_str(), history.squash.1.as_str());
    let (forged, forged_head) = (history.forged.0.as_str(), history.forged.1.as_str());
    let absent = "0123456789abcdef0123456789abcdef01234567";
    let green = || gate_run("completed", Some("success"), "github-actions");
    let none = || json!({ "check_runs": [] });

    let cases: Vec<Case> = vec![
        (
            "a green Gate on the tagged commit itself",
            fresh,
            vec![(fresh, "check-runs", green())],
            "true",
        ),
        (
            "an up-to-date merged pull request whose head's Gate is green",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh, "pulls", json!([pull(1, true, fresh, fresh_head)])),
                (fresh_head, "check-runs", green()),
            ],
            "true",
        ),
        (
            "the tagged commit's check-runs are unavailable, and the head's Gate is green",
            fresh,
            vec![
                (fresh, "pulls", json!([pull(1, true, fresh, fresh_head)])),
                (fresh_head, "check-runs", green()),
            ],
            "true",
        ),
        (
            "a squash merge whose head only the pull request ref holds",
            squash,
            vec![
                (squash, "check-runs", none()),
                (squash, "pulls", json!([pull(4, true, squash, squash_head)])),
                (squash_head, "check-runs", green()),
            ],
            "true",
        ),
        (
            "the head's newest Gate failed",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh, "pulls", json!([pull(1, true, fresh, fresh_head)])),
                (
                    fresh_head,
                    "check-runs",
                    gate_run("completed", Some("failure"), "github-actions"),
                ),
            ],
            "false",
        ),
        (
            "the head's newest Gate is still running",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh, "pulls", json!([pull(1, true, fresh, fresh_head)])),
                (
                    fresh_head,
                    "check-runs",
                    gate_run("in_progress", None, "github-actions"),
                ),
            ],
            "false",
        ),
        (
            "the head's green Gate was recorded by another app",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh, "pulls", json!([pull(1, true, fresh, fresh_head)])),
                (
                    fresh_head,
                    "check-runs",
                    gate_run("completed", Some("success"), "impostor"),
                ),
            ],
            "false",
        ),
        (
            "main moved after the branch left it, so the merge's tree is not the head's",
            stale,
            vec![
                (stale, "check-runs", none()),
                (stale, "pulls", json!([pull(2, true, stale, stale_head)])),
                (stale_head, "check-runs", green()),
            ],
            "false",
        ),
        (
            "the trees agree but main before the merge is not in the head's history",
            twin,
            vec![
                (twin, "check-runs", none()),
                (twin, "pulls", json!([pull(3, true, twin, twin_head)])),
                (twin_head, "check-runs", green()),
            ],
            "false",
        ),
        (
            "two merged pull requests name the tagged commit",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (
                    fresh,
                    "pulls",
                    json!([
                        pull(1, true, fresh, fresh_head),
                        pull(5, true, fresh, fresh_head)
                    ]),
                ),
                (fresh_head, "check-runs", green()),
            ],
            "false",
        ),
        (
            "the pull request was never merged",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh, "pulls", json!([pull(1, false, fresh, fresh_head)])),
                (fresh_head, "check-runs", green()),
            ],
            "false",
        ),
        (
            "the pull request merged as another commit",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh, "pulls", json!([pull(1, true, stale, fresh_head)])),
                (fresh_head, "check-runs", green()),
            ],
            "false",
        ),
        (
            "the tagged commit's pull requests are unavailable",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh_head, "check-runs", green()),
            ],
            "false",
        ),
        (
            "the head cannot be fetched",
            fresh,
            vec![
                (fresh, "check-runs", none()),
                (fresh, "pulls", json!([pull(1, true, fresh, absent)])),
                (absent, "check-runs", green()),
            ],
            "false",
        ),
        (
            "the merge commit's tree carries a file its up-to-date head never had",
            forged,
            vec![
                (forged, "check-runs", none()),
                (forged, "pulls", json!([pull(6, true, forged, forged_head)])),
                (forged_head, "check-runs", green()),
            ],
            "false",
        ),
    ];

    let mut wrong = Vec::new();
    for (index, (name, commit, responses, expected)) in cases.into_iter().enumerate() {
        let answer = prior_gate(&root.0, index, &history, commit, &responses);
        let (decision, log) = answer.split_once('\n').unwrap();
        if decision != expected {
            wrong.push(format!(
                "{name}: passed={decision}, expected {expected}\n{log}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "the prior-gate step decided these wrongly:\n{}",
        wrong.join("\n")
    );
}

/// Intel macOS is cross-compiled on the Apple Silicon runner — the Intel runner took 16.9 of the
/// 0.32.0 release's 19.2 minutes — and smoke-run there under Rosetta. What ships does not move:
/// four archives named `ess-<tag>-<target>.tar.gz`, one `SHA256SUMS` over them.
#[test]
fn the_release_builds_intel_macos_on_apple_silicon_and_keeps_its_four_archives() {
    let release = yaml(".github/workflows/release.yml");
    let packaging = yaml(".github/workflows/package.yml");
    let package = &packaging["jobs"]["package"];
    let runners: BTreeMap<String, String> = matrix_rows(package)
        .into_iter()
        .map(|row| (row["target"].clone(), row["runner"].clone()))
        .collect();
    let targets = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
    ];
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
    assert!(publish.contains("\"$archive_count\" != 4"), "{publish}");
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
        serde_yaml::from_str::<Value>("['queue/**']").unwrap(),
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
        smoke.contains("ess-$TAG-x86_64-unknown-linux-gnu.tar.gz"),
        "{smoke}"
    );
}

fn prebuilt_step(release: &Value) -> &Value {
    release["jobs"]["resolve"]["steps"]
        .as_sequence()
        .expect("resolve steps")
        .iter()
        .find(|step| text(&step["id"]) == "prebuilt")
        .expect("resolve looks for prebuilt archives")
}

/// Stands in for `gh api <path> --jq <filter>`: answers from `<path without query, / as _>.json`
/// in `$GH_STUB_RESPONSES` through the real `jq`, and fails like an HTTP error when there is none.
const GH_PATH_STUB: &str = r#"#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -ne 4 ] || [ "$1" != api ] || [ "$3" != --jq ]; then
  echo "gh stub: unexpected arguments: $*" >&2
  exit 2
fi
path="${2%%\?*}"
file="$GH_STUB_RESPONSES/${path//\//_}.json"
if [ ! -f "$file" ]; then
  echo "gh: HTTP 404: Not Found (https://api.github.com/$2)" >&2
  exit 1
fi
exec jq -r "$4" "$file"
"#;

/// The canned API answers one `prebuilt` case serves: each API path with the JSON it returns.
type Answers = Vec<(String, serde_json::Value)>;

/// Runs release.yml's `prebuilt` step the way Actions runs a `shell: bash` step and returns the
/// `run` value it wrote, then its log.
fn prebuilt(
    root: &Path,
    index: usize,
    commit: &str,
    responses: &[(&str, serde_json::Value)],
) -> String {
    let release = yaml(".github/workflows/release.yml");
    let step = prebuilt_step(&release);
    for name in ["GH_TOKEN", "REPOSITORY", "COMMIT"] {
        assert!(
            !step["env"][name].is_null(),
            "the prebuilt step no longer receives {name}; this harness supplies it"
        );
    }
    let case = &root.join(format!("prebuilt-{index}"));
    let bin = case.join("bin");
    let answers = case.join("responses");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&answers).unwrap();
    let stub = bin.join("gh");
    executable::install_bytes(&stub, GH_PATH_STUB.as_bytes()).unwrap();
    for (path, body) in responses {
        fs::write(
            answers.join(format!("{}.json", path.replace('/', "_"))),
            body.to_string(),
        )
        .unwrap();
    }
    let script = case.join("prebuilt.sh");
    fs::write(&script, text(&step["run"])).unwrap();
    let output_file = case.join("github-output");
    fs::write(&output_file, "").unwrap();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = isolated("bash", case)
        .args(["--noprofile", "--norc", "-eo", "pipefail"])
        .arg(&script)
        .env("PATH", path)
        .env("GH_TOKEN", "fixture")
        .env("REPOSITORY", "owner/repository")
        .env("COMMIT", commit)
        .env("GITHUB_OUTPUT", &output_file)
        .env("GH_STUB_RESPONSES", &answers)
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "the prebuilt step failed the resolve job instead of deciding: {log}"
    );
    let written = fs::read_to_string(&output_file).unwrap();
    let decisions: Vec<&str> = written
        .lines()
        .filter_map(|line| line.strip_prefix("run="))
        .collect();
    let [decision] = decisions.as_slice() else {
        panic!("the prebuilt step wrote {decisions:?}, not one decision: {log}")
    };
    format!("{decision}\n{log}")
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

/// A prebuilt run counts only when it is a successful `package.yml` push run of a queue branch at
/// the exact tagged commit, and the newest such run still holds all four unexpired archives. Every
/// other answer — and every API failure — builds the archives in the release.
#[test]
// One table of cases, which reads best whole.
#[allow(clippy::too_many_lines)]
fn a_release_publishes_only_a_queue_run_of_the_tagged_commit_that_holds_all_four_archives() {
    let root = Scratch::new("release-prebuilt");
    let commit = "3ec2deb977bb19c292a611dcf92a3e676352864a";
    let other = "5090405eeef545e9731ff362911ed5707e4b9293";
    let runs = "repos/owner/repository/actions/workflows/package.yml/runs";
    let workflow = ".github/workflows/package.yml";
    let four = [
        "release-x86_64-unknown-linux-gnu",
        "release-aarch64-unknown-linux-gnu",
        "release-x86_64-apple-darwin",
        "release-aarch64-apple-darwin",
    ];
    let all = || artifacts(&four, &[]);
    let queued = |id| package_run(id, commit, "queue/pr-119", workflow, "2026-01-01T00:00:05Z");
    let listing = |list: Vec<serde_json::Value>| json!({ "workflow_runs": list });
    let held = |id: u64| format!("repos/owner/repository/actions/runs/{id}/artifacts");

    let cases: Vec<(&str, Answers, &str)> = vec![
        (
            "a queue run of the tagged commit holds all four archives",
            vec![
                (runs.to_owned(), listing(vec![queued(7)])),
                (held(7), all()),
            ],
            "7",
        ),
        (
            "the newest of two queue runs is the one published",
            vec![
                (
                    runs.to_owned(),
                    listing(vec![
                        queued(9),
                        package_run(8, commit, "queue/pr-119", workflow, "2026-01-01T00:00:01Z"),
                    ]),
                ),
                (held(9), all()),
                (held(8), all()),
            ],
            "9",
        ),
        (
            "no package run built the tagged commit",
            vec![(runs.to_owned(), listing(vec![]))],
            "",
        ),
        ("the package runs are unavailable", vec![], ""),
        (
            "the only run built another commit",
            vec![
                (
                    runs.to_owned(),
                    listing(vec![package_run(
                        7,
                        other,
                        "queue/pr-119",
                        workflow,
                        "2026-01-01T00:00:05Z",
                    )]),
                ),
                (held(7), all()),
            ],
            "",
        ),
        (
            "the only run was not a queue branch's",
            vec![
                (
                    runs.to_owned(),
                    listing(vec![package_run(
                        7,
                        commit,
                        "feature",
                        workflow,
                        "2026-01-01T00:00:05Z",
                    )]),
                ),
                (held(7), all()),
            ],
            "",
        ),
        (
            "the only run came from another workflow file",
            vec![
                (
                    runs.to_owned(),
                    listing(vec![package_run(
                        7,
                        commit,
                        "queue/pr-119",
                        ".github/workflows/ci.yml",
                        "2026-01-01T00:00:05Z",
                    )]),
                ),
                (held(7), all()),
            ],
            "",
        ),
        (
            "the run holds three of the four archives",
            vec![
                (runs.to_owned(), listing(vec![queued(7)])),
                (held(7), artifacts(&four[..3], &[])),
            ],
            "",
        ),
        (
            "one of the four archives has expired",
            vec![
                (runs.to_owned(), listing(vec![queued(7)])),
                (held(7), artifacts(&four, &["release-x86_64-apple-darwin"])),
            ],
            "",
        ),
        (
            "the run's artifacts are unavailable",
            vec![(runs.to_owned(), listing(vec![queued(7)]))],
            "",
        ),
    ];

    let mut wrong = Vec::new();
    for (index, (name, responses, expected)) in cases.into_iter().enumerate() {
        let borrowed: Vec<(&str, serde_json::Value)> = responses
            .iter()
            .map(|(path, body)| (path.as_str(), body.clone()))
            .collect();
        let answer = prebuilt(&root.0, index, commit, &borrowed);
        let (decision, log) = answer.split_once('\n').unwrap();
        if decision != expected {
            wrong.push(format!(
                "{name}: run={decision}, expected {expected:?}\n{log}"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "the prebuilt step decided these wrongly:\n{}",
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

/// A pull request can edit `package.yml` to run on `pull_request`, and with fast-forward merges its
/// head is the commit later tagged, so such a run of the tagged commit is reachable. So is a queue
/// run that failed or was cancelled after uploading. The prebuilt step refuses all three twice: in
/// the API query and in its `jq` filter. The stub answers by path alone, so the query half is held
/// as text and the filter half by these runs.
#[test]
fn a_release_publishes_no_pull_request_or_unsuccessful_package_run_of_the_tagged_commit() {
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
    let runs = "repos/owner/repository/actions/workflows/package.yml/runs";
    let held = "repos/owner/repository/actions/runs/7/artifacts";
    let four = artifacts(
        &[
            "release-x86_64-unknown-linux-gnu",
            "release-aarch64-unknown-linux-gnu",
            "release-x86_64-apple-darwin",
            "release-aarch64-apple-darwin",
        ],
        &[],
    );
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
            &[(runs, listing), (held, four.clone())],
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
