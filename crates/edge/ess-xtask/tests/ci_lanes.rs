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

/// Every Taskfile task a lane's `task` field invokes, each with the variable assignments of that
/// invocation. `task a b VAR=x` runs `a` then `b`, both with `VAR`, so one lane can name several.
fn lane_invocations(ci: &Value) -> Vec<(String, Vec<String>)> {
    let include = ci["jobs"]["lane"]["strategy"]["matrix"]["include"]
        .as_sequence()
        .expect("the lane job is a matrix of explicit lanes");
    include
        .iter()
        .flat_map(|lane| {
            let (vars, tasks): (Vec<String>, Vec<String>) = text(&lane["task"])
                .split_whitespace()
                .map(str::to_owned)
                .partition(|word| word.contains('='));
            assert!(!tasks.is_empty(), "a lane names no task: {lane:?}");
            tasks
                .into_iter()
                .map(move |task| (task, vars.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The `SHARD=<m>/<n>` values the lanes pass to `task`, one per lane that runs it.
fn shards_of(ci: &Value, task: &str) -> Vec<String> {
    lane_invocations(ci)
        .into_iter()
        .filter(|(name, _)| name == task)
        .map(|(_, vars)| {
            let [assignment] = vars.as_slice() else {
                panic!("a `{task}` lane passes exactly `SHARD=<m>/<n>`, not {vars:?}")
            };
            assignment
                .strip_prefix("SHARD=")
                .unwrap_or_else(|| panic!("`{assignment}` is not a SHARD assignment"))
                .to_owned()
        })
        .collect()
}

/// `task`'s lanes are one complete partition: `1/n` through `n/n`, each exactly once.
fn assert_complete_partition(ci: &Value, task: &str) {
    let shards = shards_of(ci, task);
    let total = shards.len();
    assert!(total > 1, "`{task}` runs in {total} shard(s)");
    let expected: BTreeSet<String> = (1..=total).map(|m| format!("{m}/{total}")).collect();
    assert_eq!(
        shards.iter().cloned().collect::<BTreeSet<_>>(),
        expected,
        "the `{task}` shards of ci.yml leave a partition unrun or run one twice"
    );
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
    for replacement in [
        "test-shard",
        "test-feature-off",
        "test-feature-off-doc",
        "test-xtask",
        "test-doc",
    ] {
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
    for (task, _) in lane_invocations(&ci) {
        reached(&taskfile, &task, &mut covered);
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

#[test]
fn the_workspace_shards_are_one_complete_partition() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    assert_complete_partition(&ci, "test-shard");
    assert_complete_partition(&ci, "test-feature-off");

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
    // The feature-off shards select the packages `task test` selects, by the same variable, and
    // hand the partition to nextest.
    let local = shell_commands(&taskfile, "test").join("\n");
    let lane = shell_commands(&taskfile, "test-feature-off").join("\n");
    assert!(local.contains("{{.FEATURE_OFF_PACKAGES}}"), "{local}");
    assert!(lane.contains("cargo nextest run") && lane.contains("{{.FEATURE_OFF_PACKAGES}}"));
    assert!(lane.contains("--partition count:{{.SHARD}}"), "{lane}");
    // Their doc-tests, which nextest does not run, run once: in exactly one lane, by themselves.
    let doc = shell_commands(&taskfile, "test-feature-off-doc").join("\n");
    assert!(
        doc.contains("cargo test --locked --doc {{.FEATURE_OFF_PACKAGES}}"),
        "{doc}"
    );
    let doc_lanes = lane_invocations(&ci)
        .into_iter()
        .filter(|(task, _)| task == "test-feature-off-doc")
        .count();
    assert_eq!(
        doc_lanes, 1,
        "the feature-off doc-tests run in {doc_lanes} lanes, not exactly one"
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

/// The Cargo workspaces a lane's tasks build: the root, and every `--manifest-path` directory.
fn lane_workspaces(ci: &Value, taskfile: &Value) -> BTreeSet<String> {
    let mut reached_tasks = BTreeSet::new();
    for (task, _) in lane_invocations(ci) {
        reached(taskfile, &task, &mut reached_tasks);
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

/// Every lockfile a lane builds from is fetched before the lane's task runs.
///
/// The gate's tasks pass `--offline` (`fuzz-check`), set `CARGO_NET_OFFLINE` (`support-check`) or
/// spawn `cargo run --offline` from inside a test (`support::tests::adversary_*` in ess-xtask).
/// The single-job gate never noticed, because `cargo clippy --workspace` ran first and populated
/// the registry. A lane starts from an empty one, so the fetch has to be explicit, and it has to
/// cover every workspace a lane builds, not only the ones whose commands say `--offline` — the
/// xtask case is invisible from the Taskfile.
#[test]
fn every_lockfile_a_lane_builds_is_fetched_before_the_lane_runs() {
    let ci = yaml(".github/workflows/ci.yml");
    let taskfile = yaml("Taskfile.yml");
    let steps = ci["jobs"]["lane"]["steps"]
        .as_sequence()
        .expect("lane steps");
    let run_at = steps
        .iter()
        .position(|step| text(&step["run"]).starts_with("task "))
        .expect("the lane runs its task");
    let fetched: String = steps[..run_at]
        .iter()
        .map(|step| text(&step["run"]))
        .collect::<Vec<_>>()
        .join("\n");
    let root = workspace_root();
    for workspace in lane_workspaces(&ci, &taskfile) {
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
            "a lane builds `{workspace}` and no step before the lane's task fetches its lockfile"
        );
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
/// Two such mechanisms exist, both found by the first nextest run (Actions run 36126201626):
/// `support/browser.rs`'s `STARTUP` mutex, which admits one Firefox start at a time, and a
/// `Once` that prunes every fixture directory not carrying this process's id — which, with one
/// process per test, deletes the fixtures of every test running beside it. Both are found here by
/// what the source says, so a new binary that includes the browser fixture or copies the prune is
/// serialised or named.
#[test]
fn nextest_serialises_every_binary_that_coordinates_through_process_state() {
    let config = fs::read_to_string(workspace_root().join(".config/nextest.toml"))
        .expect("the nextest configuration is readable");
    let serialised = serialised_binaries(&config);
    // Split, so that this file does not contain what it scans for.
    let include = ["#[path = \"support/", "browser.rs\"]"].concat();
    let once = ["std::sync::Once", "::new()"].concat();
    let prune = ["remove_dir_all(", "entry.path())"].concat();
    let browser = binaries_where(|source| source.contains(&include));
    let pruning = binaries_where(|source| source.contains(&once) && source.contains(&prune));
    assert!(
        browser.contains("ess-cli::coverage_browser"),
        "the browser scan found {browser:?}; it no longer sees the fixture it exists to find"
    );
    assert!(
        pruning.contains("ess-cli::execution_recovery"),
        "the prune scan found {pruning:?}; it no longer sees the binary it exists to find"
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
}
