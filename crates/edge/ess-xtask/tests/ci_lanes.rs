//! The pull-request gate is split into parallel lanes, and the split is held to `task check`.
//!
//! `.github/workflows/ci.yml` no longer runs `task check` as one job: a matrix of lanes each calls
//! one Taskfile task, and a job named `Gate` aggregates them. A split like that has one silent
//! failure mode — a step of `task check` that no lane calls exits nowhere, so nothing turns red when
//! it goes missing. The coverage case below reads both files and names the step instead.
//!
//! The release workflow reuses a green `Gate` on the exact tagged commit rather than running the
//! gate a second time. That reuse is a condition in two jobs and a query in a third, and the query
//! names the check by the same string the aggregate job carries; the last case ties them together.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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

/// The Taskfile task a lane's `task` field invokes, and the variable assignments after it.
fn lane_invocations(ci: &Value) -> Vec<(String, Vec<String>)> {
    let include = ci["jobs"]["lane"]["strategy"]["matrix"]["include"]
        .as_sequence()
        .expect("the lane job is a matrix of explicit lanes");
    include
        .iter()
        .map(|lane| {
            let mut words = text(&lane["task"]).split_whitespace().map(str::to_owned);
            let task = words.next().expect("every lane names a task");
            (task, words.collect())
        })
        .collect()
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

#[test]
fn every_step_of_task_check_runs_in_some_pull_request_lane() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let lanes = lane_invocations(&ci);
    assert!(
        lanes.len() > 1,
        "ci.yml runs {} lane(s); the gate is not split",
        lanes.len()
    );

    let mut covered = BTreeSet::new();
    for (task, _) in &lanes {
        reached(&taskfile, task, &mut covered);
    }

    // `test` is the one step CI replaces rather than calls: its three commands run as sharded
    // nextest partitions, the feature-off lane, the xtask lane and the doc-tests nextest skips.
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
        "these steps of `task check` run in no lane of ci.yml, so a pull request never runs them: \
         {missing:?}"
    );
    for replacement in ["test-shard", "test-feature-off", "test-xtask", "test-doc"] {
        assert!(
            covered.contains(replacement),
            "no lane of ci.yml runs `{replacement}`, one of the parts `task test` is split into"
        );
    }
    // The xtask half is shared, not restated: `task test` calls the same task the lane does.
    assert!(
        direct_subtasks(&taskfile, "test")
            .iter()
            .any(|step| step == "test-xtask"),
        "`task test` restates the xtask command instead of calling `test-xtask`"
    );
}

#[test]
fn the_workspace_shards_are_one_complete_partition() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let shards: Vec<String> = lane_invocations(&ci)
        .into_iter()
        .filter(|(task, _)| task == "test-shard")
        .map(|(_, vars)| {
            let [assignment] = vars.as_slice() else {
                panic!("a `test-shard` lane passes exactly `SHARD=<m>/<n>`, not {vars:?}")
            };
            assignment
                .strip_prefix("SHARD=")
                .unwrap_or_else(|| panic!("`{assignment}` is not a SHARD assignment"))
                .to_owned()
        })
        .collect();
    let total = shards.len();
    assert!(total > 1, "the workspace tests run in {total} shard(s)");
    let expected: BTreeSet<String> = (1..=total).map(|m| format!("{m}/{total}")).collect();
    assert_eq!(
        shards.iter().cloned().collect::<BTreeSet<_>>(),
        expected,
        "the shards of ci.yml leave a partition unrun or run one twice"
    );

    // The shard selects what `task test` selects, and hands the partition to nextest.
    let workspace = &shell_commands(&taskfile, "test")[0];
    assert!(
        workspace.contains("cargo test --workspace --exclude ess-xtask"),
        "{workspace}"
    );
    let shard = shell_commands(&taskfile, "test-shard").join("\n");
    assert!(
        shard.contains("cargo nextest run --workspace --exclude ess-xtask"),
        "{shard}"
    );
    assert!(shard.contains("--partition count:{{.SHARD}}"), "{shard}");
    // Nextest does not run doc-tests, so the workspace doc-tests are a lane of their own.
    let doc = shell_commands(&taskfile, "test-doc").join("\n");
    assert!(
        doc.contains("cargo test --workspace --exclude ess-xtask"),
        "{doc}"
    );
    assert!(doc.contains("--doc"), "{doc}");
    // The feature-off lane selects the packages `task test` selects, by the same variable.
    let local = shell_commands(&taskfile, "test").join("\n");
    let lane = shell_commands(&taskfile, "test-feature-off").join("\n");
    assert!(local.contains("{{.FEATURE_OFF_PACKAGES}}"), "{local}");
    assert!(lane.contains("cargo nextest run") && lane.contains("{{.FEATURE_OFF_PACKAGES}}"));
    assert!(
        lane.contains("--doc"),
        "the feature-off lane skips its doc-tests: {lane}"
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

    let needs: BTreeSet<String> = gate["needs"]
        .as_sequence()
        .expect("Gate needs a list of jobs")
        .iter()
        .map(|need| text(need).to_owned())
        .collect();
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
    let steps = ci["jobs"]["lane"]["steps"]
        .as_sequence()
        .expect("lane steps");
    let install = steps
        .iter()
        .find(|step| text(&step["uses"]).starts_with("taiki-e/install-action@"))
        .expect("the lane installs nextest with taiki-e/install-action");
    let tool = text(&install["with"]["tool"]);
    let version = tool
        .strip_prefix("cargo-nextest@")
        .unwrap_or_else(|| panic!("the install step names `{tool}`, not an exact cargo-nextest"));
    assert!(
        version.split('.').count() == 3
            && version.split('.').all(|part| part.parse::<u32>().is_ok()),
        "cargo-nextest is not pinned to an exact version: `{tool}`"
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

#[test]
fn a_release_reuses_a_green_gate_on_the_exact_tagged_commit_and_otherwise_runs_it() {
    let ci = yaml(".github/workflows/ci.yml");
    let release = yaml(".github/workflows/release.yml");
    let jobs = &release["jobs"];
    let check = text(&ci["jobs"]["gate"]["name"]);

    let resolve = &jobs["resolve"];
    assert_eq!(text(&resolve["permissions"]["checks"]), "read");
    assert_eq!(text(&resolve["permissions"]["contents"]), "read");
    assert_eq!(
        text(&resolve["outputs"]["gate-passed"]),
        "${{ steps.prior-gate.outputs.passed }}"
    );
    let query = resolve["steps"]
        .as_sequence()
        .expect("resolve steps")
        .iter()
        .find(|step| text(&step["id"]) == "prior-gate")
        .expect("resolve looks for a prior Gate");
    let run = text(&query["run"]);
    assert_eq!(
        text(&query["env"]["COMMIT"]),
        "${{ steps.release.outputs.commit }}"
    );
    assert!(run.contains("commits/$COMMIT/check-runs"), "{run}");
    assert!(run.contains(&format!("check_name={check}")), "{run}");
    assert!(run.contains(&format!(".name == \"{check}\"")), "{run}");
    assert!(run.contains(".app.slug == \"github-actions\""), "{run}");
    // Only a completed success counts; every other answer, including an API failure, runs it.
    assert!(run.contains("\"completed/success\""), "{run}");
    assert!(
        run.contains("passed=false") || run.contains("passed=\"false\""),
        "{run}"
    );

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
