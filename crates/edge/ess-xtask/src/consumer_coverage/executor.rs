//! Exact native case execution; parsers are bound to measured stable libtest output.
use anyhow::{bail, Context, Result};
use serde_json::Value;

pub(super) fn listing(stdout: &str, name: &str, ignored: bool) -> Result<()> {
    let expected = if ignored {
        "0 tests, 0 benchmarks\n".to_owned()
    } else {
        format!("{name}: test\n\n1 test, 0 benchmarks\n")
    };
    if stdout != expected {
        bail!("missing, renamed, duplicate or ignored exact case {name}; unrecognized stable libtest listing");
    }
    Ok(())
}
pub(super) fn result(stdout: &str, stderr: &str, name: &str, exit: Option<i32>) -> Result<()> {
    if exit != Some(0) || !stderr.is_empty() {
        bail!("exact case {name} failed, signaled, or emitted unexpected stderr: {exit:?}");
    }
    let prefix=format!("\nrunning 1 test\ntest {name} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; ");
    let rest = stdout
        .strip_prefix(&prefix)
        .context("not one exact successful stable libtest case")?;
    let (filtered, duration) = rest
        .split_once(" filtered out; finished in ")
        .context("unknown stable libtest summary")?;
    let count = filtered
        .parse::<usize>()
        .context("unknown filtered case count")?;
    if count.to_string() != filtered {
        bail!("noncanonical filtered case count");
    }
    let duration = duration
        .strip_suffix("s\n\n")
        .context("extra or missing stable libtest result text")?;
    let (seconds, decimal) = duration
        .split_once('.')
        .context("unknown libtest duration")?;
    if seconds.is_empty()
        || decimal.is_empty()
        || !seconds.bytes().all(|b| b.is_ascii_digit())
        || !decimal.bytes().all(|b| b.is_ascii_digit())
    {
        bail!("unknown stable libtest duration");
    }
    Ok(())
}
pub(super) fn source_contract(case: &Value) -> Result<()> {
    for flag in ["ignored", "contains_catch_unwind", "contains_early_return"] {
        if case[flag] != false {
            bail!("unqualified source behavior boundary: {flag}");
        }
    }
    if !case["nested_command_candidates"]
        .as_array()
        .context("nested command inventory")?
        .is_empty()
    {
        bail!("nested runtime requires a separately measured execution contract; initial cases are direct Rust only");
    }
    Ok(())
}

pub(super) struct Captured {
    pub stdout: String,
    pub stderr: String,
    pub exit: Option<i32>,
}
pub(super) trait CaseRunner {
    fn authority(&mut self) -> Result<()>;
    fn command(&mut self, args: &[String]) -> Result<Captured>;
}
pub(super) fn execute_case(runner: &mut impl CaseRunner, name: &str) -> Result<()> {
    runner.authority()?;
    for ignored in [false, true] {
        let mut args = vec!["--list".to_owned()];
        if ignored {
            args.push("--ignored".into());
        }
        args.extend(["--exact", name, "--color", "never"].map(str::to_owned));
        let output = runner.command(&args)?;
        if output.exit != Some(0) || !output.stderr.is_empty() {
            bail!("case listing subprocess refused: {name}");
        }
        listing(&output.stdout, name, ignored)?;
        runner.authority()?;
    }
    let output = runner.command(
        &["--exact", name, "--test-threads", "1", "--color", "never"].map(str::to_owned),
    )?;
    runner.authority()?;
    result(&output.stdout, &output.stderr, name, output.exit)
}

pub(super) fn artifact(stdout: &str, package: &str, target: &str, source: &str) -> Result<Value> {
    let mut selected = None;
    let mut finished = false;
    for line in stdout.lines() {
        if finished {
            bail!("Cargo emitted data after build-finished");
        }
        let message: Value = serde_json::from_str(line).context("unknown Cargo message format")?;
        match message["reason"].as_str().context("Cargo message reason")? {
            "compiler-artifact" => {
                if message["package_id"] == package
                    && message["target"]["name"] == target
                    && message["target"]["kind"] == serde_json::json!(["test"])
                {
                    if selected.is_some() {
                        bail!("duplicate selected Cargo artifact {package} {target}");
                    }
                    let executable = message["executable"]
                        .as_str()
                        .context("selected Cargo target has no native executable")?;
                    if message["target"]["src_path"] != source
                        || message["target"]["test"] != true
                        || message["target"]["crate_types"] != serde_json::json!(["bin"])
                        || message["features"] != serde_json::json!([])
                        || message["filenames"] != serde_json::json!([executable])
                        || message["profile"]
                            != serde_json::json!({"opt_level":"0","debuginfo":0,"debug_assertions":true,"overflow_checks":true,"test":true})
                    {
                        bail!("selected Cargo owner artifact has an unreviewed source/profile: {package} {target}");
                    }
                    selected = Some(message);
                }
            }
            "compiler-message" | "build-script-executed" => {}
            "build-finished" => {
                if message != serde_json::json!({"reason":"build-finished","success":true}) {
                    bail!("Cargo build did not finish successfully");
                }
                finished = true;
            }
            reason => bail!("unknown Cargo producer message {reason}"),
        }
    }
    if !finished {
        bail!("missing Cargo build-finished observation");
    }
    selected.with_context(|| format!("missing exact Cargo artifact {package} {target}"))
}
