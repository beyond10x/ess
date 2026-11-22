//! `cargo xtask pr-watch <number>`: wait for a pull request's check runs and say how they ended.
//!
//! The command reads the head commit of the pull request once, then polls that commit's check
//! runs until every one has completed. Exit statuses:
//!
//! - **0**: every check run ended `success`, `neutral` or `skipped`;
//! - **1**: some check run ended any other way; each is printed with its conclusion, its URL and
//!   the failing task its annotations name;
//! - **2**: a timeout, a `gh` failure or unreadable JSON.
//!
//! It reads GitHub through `gh` and writes nothing, to GitHub or to disk.

use std::fmt::Write as _;
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Duration;

/// Arguments of `cargo xtask pr-watch`.
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// The pull request number.
    number: u64,
    /// Seconds between two reads of the check runs.
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
    interval: u64,
    /// Seconds to wait in all before giving up with exit status 2.
    #[arg(long, default_value_t = 5400)]
    timeout: u64,
}

/// One check run of the watched commit, as `GET /repos/{owner}/{repo}/commits/{sha}/check-runs` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckRun {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) conclusion: Option<String>,
    pub(crate) url: String,
}

/// What one read of the check runs says.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// No run yet, or at least one run not `completed`.
    Pending { completed: usize, total: usize },
    /// Every run completed with a passing conclusion.
    Passed { total: usize },
    /// Every run completed and these did not pass, sorted by name.
    Failed(Vec<CheckRun>),
}

/// A run that did not pass, with the tasks its annotations name.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FailedRun {
    pub(crate) run: CheckRun,
    pub(crate) tasks: Result<Vec<String>, String>,
}

/// How the watch ended.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Passed { total: usize },
    Failed(Vec<FailedRun>),
    TimedOut { completed: usize, total: usize },
    Error(String),
}

/// The conclusions that pass. Any other conclusion, or none, fails.
const PASSING: &[&str] = &["success", "neutral", "skipped"];

/// Reads the check-runs document of one commit.
pub(crate) fn parse_check_runs(json: &str) -> Result<Vec<CheckRun>, String> {
    let value = serde_json::from_str::<serde_json::Value>(json)
        .map_err(|error| format!("the check runs are not JSON: {error}"))?;
    let total = value
        .get("total_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or("the check runs have no numeric `total_count`")?;
    let array = value
        .get("check_runs")
        .and_then(serde_json::Value::as_array)
        .ok_or("the check runs have no `check_runs` array")?;
    let mut runs = Vec::with_capacity(array.len());
    for item in array {
        let text = |key: &str| item.get(key).and_then(serde_json::Value::as_str);
        let id = item
            .get("id")
            .and_then(serde_json::Value::as_u64)
            .ok_or("a check run has no numeric `id`")?;
        let name = text("name").ok_or("a check run has no string `name`")?;
        let status = text("status").ok_or("a check run has no string `status`")?;
        let conclusion = match item.get("conclusion") {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(text)) => Some(text.clone()),
            Some(_) => return Err("a check run has a mistyped `conclusion`".to_owned()),
        };
        runs.push(CheckRun {
            id,
            name: name.to_owned(),
            status: status.to_owned(),
            conclusion,
            url: text("html_url").unwrap_or("").to_owned(),
        });
    }
    let len = runs.len();
    if usize::try_from(total).map_or(true, |total| total > len) {
        return Err(format!(
            "the commit has {total} check runs and one page holds {len}; pr-watch reads one page of 100"
        ));
    }
    Ok(runs)
}

/// Decides what one read of the check runs says.
pub(crate) fn decide(runs: &[CheckRun]) -> Verdict {
    let completed = runs.iter().filter(|run| run.status == "completed").count();
    if runs.is_empty() || completed < runs.len() {
        return Verdict::Pending {
            completed,
            total: runs.len(),
        };
    }
    let mut failed: Vec<CheckRun> = runs
        .iter()
        .filter(|run| {
            run.conclusion
                .as_deref()
                .is_none_or(|conclusion| !PASSING.contains(&conclusion))
        })
        .cloned()
        .collect();
    if failed.is_empty() {
        return Verdict::Passed { total: runs.len() };
    }
    failed.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    Verdict::Failed(failed)
}

/// The task names a check run's annotations report, in first-seen order.
pub(crate) fn failing_tasks(json: &str) -> Result<Vec<String>, String> {
    const MARK: &str = "Failed to run task \"";
    let value = serde_json::from_str::<serde_json::Value>(json)
        .map_err(|_| "the annotations are not a JSON array".to_owned())?;
    let items = value
        .as_array()
        .ok_or("the annotations are not a JSON array")?;
    let mut tasks: Vec<String> = Vec::new();
    for message in items
        .iter()
        .filter_map(|item| item.get("message").and_then(serde_json::Value::as_str))
    {
        for line in message.lines() {
            if let Some(start) = line.find(MARK) {
                let rest = &line[start + MARK.len()..];
                if let Some(end) = rest.find('"') {
                    let name = &rest[..end];
                    if !tasks.iter().any(|seen| seen == name) {
                        tasks.push(name.to_owned());
                    }
                }
            }
        }
    }
    Ok(tasks)
}

/// Polls until the check runs end or `polls` reads are used up.
pub(crate) fn watch(
    fetch_runs: &mut dyn FnMut() -> Result<String, String>,
    fetch_annotations: &mut dyn FnMut(u64) -> Result<String, String>,
    polls: u64,
    progress: &mut dyn FnMut(usize, usize),
    wait: &mut dyn FnMut(),
) -> Outcome {
    let (mut completed, mut total) = (0, 0);
    for poll in 1..=polls {
        let json = match fetch_runs() {
            Ok(json) => json,
            Err(error) => return Outcome::Error(error),
        };
        let runs = match parse_check_runs(&json) {
            Ok(runs) => runs,
            Err(error) => return Outcome::Error(error),
        };
        match decide(&runs) {
            Verdict::Passed { total } => return Outcome::Passed { total },
            Verdict::Failed(red) => {
                let failed = red
                    .into_iter()
                    .map(|run| {
                        let tasks = fetch_annotations(run.id)
                            .and_then(|annotations| failing_tasks(&annotations));
                        FailedRun { run, tasks }
                    })
                    .collect();
                return Outcome::Failed(failed);
            }
            Verdict::Pending {
                completed: done,
                total: all,
            } => {
                (completed, total) = (done, all);
                progress(completed, total);
                if poll < polls {
                    wait();
                }
            }
        }
    }
    Outcome::TimedOut { completed, total }
}

/// The process exit status of an outcome.
pub(crate) fn exit_status(outcome: &Outcome) -> u8 {
    match outcome {
        Outcome::Passed { .. } => 0,
        Outcome::Failed(_) => 1,
        Outcome::TimedOut { .. } | Outcome::Error(_) => 2,
    }
}

/// The text printed for an outcome, each line ending in a newline.
pub(crate) fn report(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Passed { total } => format!("pr-watch: all {total} check runs passed\n"),
        Outcome::Failed(runs) => {
            let mut text = String::new();
            for failed in runs {
                let run = &failed.run;
                let conclusion = run.conclusion.as_deref().unwrap_or("no conclusion");
                let _ = writeln!(text, "red: {} ({conclusion}) {}", run.name, run.url);
                match &failed.tasks {
                    Ok(tasks) if tasks.is_empty() => {
                        text.push_str("  failing task: not named in the annotations\n");
                    }
                    Ok(tasks) => {
                        for task in tasks {
                            let _ = writeln!(text, "  failing task: {task}");
                        }
                    }
                    Err(error) => {
                        let _ = writeln!(text, "  failing task: annotations unreadable: {error}");
                    }
                }
            }
            text
        }
        Outcome::TimedOut { completed, total } => {
            format!("pr-watch: timed out with {completed} of {total} check runs completed\n")
        }
        Outcome::Error(error) => format!("pr-watch: {error}\n"),
    }
}

/// Runs `gh` in `root` and returns its standard output.
fn gh(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| {
            format!(
                "running gh: {error}. pr-watch reads GitHub through gh and is not part of the offline gate"
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "`gh {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("gh printed non-UTF-8 text: {error}"))
}

/// Reads the head commit of a pull request.
fn head_sha(root: &Path, number: &str) -> Result<String, String> {
    let json = gh(root, &["pr", "view", number, "--json", "headRefOid"])?;
    let value = serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|error| format!("the pull request is not JSON: {error}"))?;
    match value.get("headRefOid").and_then(serde_json::Value::as_str) {
        Some(sha) if !sha.is_empty() => Ok(sha.to_owned()),
        _ => Err("the pull request has no `headRefOid`".to_owned()),
    }
}

/// Runs the command.
pub(crate) fn main(args: &Args) -> ExitCode {
    let root = match crate::workspace_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("pr-watch: {error}");
            return ExitCode::from(2);
        }
    };
    let number = args.number.to_string();
    let sha = match head_sha(&root, &number) {
        Ok(sha) => sha,
        Err(error) => {
            let outcome = Outcome::Error(error);
            eprint!("{}", report(&outcome));
            return ExitCode::from(exit_status(&outcome));
        }
    };
    eprintln!("pr-watch: watching pull request {number} at {sha}");
    let polls = args.timeout.div_ceil(args.interval).max(1);
    let mut last = None;
    let outcome = watch(
        &mut || {
            gh(
                &root,
                &[
                    "api",
                    &format!(
                        "repos/{{owner}}/{{repo}}/commits/{sha}/check-runs?per_page=100&filter=latest"
                    ),
                ],
            )
        },
        &mut |id| {
            gh(
                &root,
                &[
                    "api",
                    &format!("repos/{{owner}}/{{repo}}/check-runs/{id}/annotations?per_page=100"),
                ],
            )
        },
        polls,
        &mut |completed, total| {
            if last != Some((completed, total)) {
                eprintln!("pr-watch: {completed} of {total} check runs completed");
                last = Some((completed, total));
            }
        },
        &mut || std::thread::sleep(Duration::from_secs(args.interval)),
    );
    match outcome {
        Outcome::Passed { .. } | Outcome::Failed(_) => print!("{}", report(&outcome)),
        Outcome::TimedOut { .. } | Outcome::Error(_) => eprint!("{}", report(&outcome)),
    }
    ExitCode::from(exit_status(&outcome))
}

#[cfg(test)]
mod tests {
    use super::super::{Cli, Command};
    use super::*;
    use clap::Parser;

    const TASK_ANNOTATION: &str = r#"[{"path":".github","start_line":1,"annotation_level":"failure","message":"task: Failed to run task \"test-shard\": exit status 100"}]"#;

    fn run(id: u64, name: &str, status: &str, conclusion: Option<&str>) -> String {
        let conclusion = conclusion.map_or("null".to_owned(), |c| format!("\"{c}\""));
        format!(
            r#"{{"id":{id},"name":"{name}","status":"{status}","conclusion":{conclusion},"html_url":"https://github.com/o/r/runs/{id}"}}"#
        )
    }

    fn runs_json(runs: &[String]) -> String {
        format!(
            r#"{{"total_count":{},"check_runs":[{}]}}"#,
            runs.len(),
            runs.join(",")
        )
    }

    fn decided(runs: &[String]) -> Verdict {
        decide(&parse_check_runs(&runs_json(runs)).unwrap())
    }

    fn annotation(task: &str) -> String {
        format!(r#"{{"message":"task: Failed to run task \"{task}\": exit status 1"}}"#)
    }

    #[test]
    fn every_run_successful_neutral_or_skipped_passes() {
        let verdict = decided(&[
            run(1, "Gate", "completed", Some("success")),
            run(2, "Docs", "completed", Some("neutral")),
            run(3, "Test 1/20", "completed", Some("skipped")),
        ]);
        assert_eq!(verdict, Verdict::Passed { total: 3 });
    }

    #[test]
    fn a_failed_run_fails_once_every_run_completed() {
        let verdict = decided(&[
            run(1, "Gate", "completed", Some("success")),
            run(2, "Test 11/20", "completed", Some("failure")),
        ]);
        let Verdict::Failed(red) = verdict else {
            panic!("expected Failed, got {verdict:?}");
        };
        assert_eq!(red.len(), 1);
        assert_eq!(red[0].id, 2);
    }

    #[test]
    fn cancelled_timed_out_and_missing_conclusions_fail() {
        let verdict = decided(&[
            run(1, "A", "completed", Some("cancelled")),
            run(2, "B", "completed", Some("timed_out")),
            run(3, "C", "completed", None),
        ]);
        let Verdict::Failed(red) = verdict else {
            panic!("expected Failed, got {verdict:?}");
        };
        let names: Vec<&str> = red.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["A", "B", "C"]);
    }

    #[test]
    fn an_unfinished_run_keeps_the_watch_pending_beside_a_failure() {
        let verdict = decided(&[
            run(1, "Test 2/20", "completed", Some("failure")),
            run(2, "Test 3/20", "in_progress", None),
        ]);
        assert_eq!(
            verdict,
            Verdict::Pending {
                completed: 1,
                total: 2
            }
        );
    }

    #[test]
    fn no_check_run_yet_is_pending() {
        let runs = parse_check_runs(r#"{"total_count":0,"check_runs":[]}"#).unwrap();
        assert_eq!(
            decide(&runs),
            Verdict::Pending {
                completed: 0,
                total: 0
            }
        );
    }

    #[test]
    fn more_runs_than_one_page_is_an_error() {
        let json = format!(
            r#"{{"total_count":150,"check_runs":[{}]}}"#,
            run(1, "A", "completed", Some("success"))
        );
        assert!(parse_check_runs(&json)
            .unwrap_err()
            .contains("150 check runs"));
    }

    #[test]
    fn a_run_without_an_id_is_an_error() {
        let json = r#"{"total_count":1,"check_runs":[{"name":"x","status":"completed","conclusion":"success"}]}"#;
        assert!(parse_check_runs(json).unwrap_err().contains("`id`"));
    }

    #[test]
    fn the_failing_task_is_read_from_the_annotations() {
        assert_eq!(
            failing_tasks(TASK_ANNOTATION),
            Ok(vec!["test-shard".to_owned()])
        );
        let json = format!(
            "[{},{},{}]",
            annotation("test-shard"),
            annotation("test-shard"),
            annotation("ci-lint")
        );
        assert_eq!(
            failing_tasks(&json),
            Ok(vec!["test-shard".to_owned(), "ci-lint".to_owned()])
        );
    }

    #[test]
    fn annotations_without_a_task_name_none_and_a_non_array_is_an_error() {
        assert_eq!(
            failing_tasks(r#"[{"message":"Process completed with exit code 1."}]"#),
            Ok(vec![])
        );
        assert!(failing_tasks(r#"{"message":"x"}"#).is_err());
    }

    #[test]
    fn watch_names_the_red_run_and_its_task_and_exits_1() {
        let mut replies = vec![
            runs_json(&[run(5, "Test 11/20", "in_progress", None)]),
            runs_json(&[
                run(5, "Test 11/20", "completed", Some("failure")),
                run(6, "Gate", "completed", Some("success")),
            ]),
        ]
        .into_iter();
        let mut waits = 0;
        let outcome = watch(
            &mut || Ok(replies.next().unwrap()),
            &mut |id| {
                assert_eq!(id, 5);
                Ok(TASK_ANNOTATION.to_owned())
            },
            5,
            &mut |_, _| {},
            &mut || waits += 1,
        );
        assert_eq!(waits, 1);
        assert_eq!(exit_status(&outcome), 1);
        assert_eq!(
            report(&outcome),
            "red: Test 11/20 (failure) https://github.com/o/r/runs/5\n  failing task: test-shard\n"
        );
        let Outcome::Failed(failed) = outcome else {
            panic!("expected Failed");
        };
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].run.id, 5);
        assert_eq!(failed[0].tasks, Ok(vec!["test-shard".to_owned()]));
    }

    #[test]
    fn watch_passes_and_exits_0() {
        let json = runs_json(&[
            run(1, "A", "completed", Some("success")),
            run(2, "B", "completed", Some("success")),
        ]);
        let mut waits = 0;
        let outcome = watch(
            &mut || Ok(json.clone()),
            &mut |_| unreachable!(),
            3,
            &mut |_, _| {},
            &mut || waits += 1,
        );
        assert_eq!(outcome, Outcome::Passed { total: 2 });
        assert_eq!(exit_status(&outcome), 0);
        assert_eq!(report(&outcome), "pr-watch: all 2 check runs passed\n");
        assert_eq!(waits, 0);
    }

    #[test]
    fn watch_times_out_while_pending_and_exits_2() {
        let json = runs_json(&[
            run(1, "A", "completed", Some("success")),
            run(2, "B", "queued", None),
        ]);
        let (mut waits, mut seen) = (0, Vec::new());
        let outcome = watch(
            &mut || Ok(json.clone()),
            &mut |_| unreachable!(),
            3,
            &mut |completed, total| seen.push((completed, total)),
            &mut || waits += 1,
        );
        assert_eq!(
            outcome,
            Outcome::TimedOut {
                completed: 1,
                total: 2
            }
        );
        assert_eq!(waits, 2);
        assert_eq!(seen, [(1, 2), (1, 2), (1, 2)]);
        assert_eq!(exit_status(&outcome), 2);
        assert_eq!(
            report(&outcome),
            "pr-watch: timed out with 1 of 2 check runs completed\n"
        );
    }

    #[test]
    fn a_gh_failure_exits_2() {
        let text = "`gh api …` failed: HTTP 404";
        let outcome = watch(
            &mut || Err(text.to_owned()),
            &mut |_| unreachable!(),
            3,
            &mut |_, _| {},
            &mut || {},
        );
        assert_eq!(outcome, Outcome::Error(text.to_owned()));
        assert_eq!(exit_status(&outcome), 2);
    }

    #[test]
    fn an_annotation_failure_keeps_the_red_outcome() {
        let json = runs_json(&[run(1, "A", "completed", Some("failure"))]);
        let outcome = watch(
            &mut || Ok(json.clone()),
            &mut |_| Err("HTTP 403".to_owned()),
            3,
            &mut |_, _| {},
            &mut || {},
        );
        assert!(matches!(outcome, Outcome::Failed(_)));
        assert_eq!(exit_status(&outcome), 1);
        assert!(report(&outcome).contains("  failing task: annotations unreadable: HTTP 403\n"));
    }

    #[test]
    fn the_command_line_parses() {
        let cli = Cli::try_parse_from([
            "ess-xtask",
            "pr-watch",
            "806",
            "--interval",
            "10",
            "--timeout",
            "60",
        ])
        .unwrap();
        let Command::PrWatch(args) = cli.command else {
            panic!("expected PrWatch");
        };
        assert_eq!((args.number, args.interval, args.timeout), (806, 10, 60));
        let cli = Cli::try_parse_from(["ess-xtask", "pr-watch", "806"]).unwrap();
        let Command::PrWatch(args) = cli.command else {
            panic!("expected PrWatch");
        };
        assert_eq!((args.interval, args.timeout), (30, 5400));
        assert!(Cli::try_parse_from(["ess-xtask", "pr-watch", "806", "--interval", "0"]).is_err());
    }
}
