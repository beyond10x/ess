//! `cargo xtask synth-jobs`: one synthesis measured at one, two and four jobs.
//!
//! Runs `ess verify conform synthesize --target ir` once per job count (`ESS_SYNTH_JOBS`) on one
//! model, prints wall time, CPU time and peak memory of each run, and refuses when the suites
//! differ. A measurement is not a test: it takes minutes and runs outside the test step.

use anyhow::{anyhow, bail, Context, Result};
use clap::Args as ClapArgs;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Flags of the generated card, each set by a command of its own.
    #[arg(long, default_value_t = 48)]
    flags: usize,
    /// How many of those commands also read the caller.
    #[arg(long, default_value_t = 24)]
    reading: usize,
    /// Measure this specification instead of the generated model.
    #[arg(long)]
    path: Option<PathBuf>,
    /// Authored scenarios, passed to `ess` as given.
    #[arg(long)]
    scenarios: Option<PathBuf>,
    /// The `ess` binary to run.
    #[arg(long, default_value = "target/release/ess")]
    ess: PathBuf,
    /// The job counts to measure.
    #[arg(long, value_delimiter = ',', default_values_t = [1_usize, 2, 4])]
    jobs: Vec<usize>,
}

/// The card model of the caller-readings tests with `flags` flags. Commands `Mark1..=Mark<reading>`
/// refuse a caller who is not the owner, so each is a focused reading of its own.
pub fn model(flags: usize, reading: usize) -> String {
    let mut text = String::from(
        "format: ess/37\nsystem: demo\nversion: v1\ndomain: demo.board\n\
         summary: Cards whose flags several commands set, each refusing a flag already set.\n\
         types:\n  - {name: demo.board.CardId, kind: newtype, of: String}\n\
         \x20 - {name: demo.board.OwnerId, kind: newtype, of: String}\n\
         entities:\n  - name: demo.board.Card\n    identity: {name: card_id, type: demo.board.CardId}\n\
         \x20   fields:\n      - {name: owner_id, type: demo.board.OwnerId}\n",
    );
    for i in 1..=flags {
        let _ = writeln!(text, "      - {{name: flag_{i}, type: Boolean}}");
    }
    text.push_str(
        "    lifecycle:\n      initial: Open\n      states: [Open, Closed]\n      terminal: [Closed]\n\
         \x20     transitions:\n",
    );
    for i in 1..=flags {
        let _ = writeln!(text, "        - {{name: mark_{i}, from: [Open], to: Open}}");
    }
    text.push_str(
        "        - {name: close, from: [Open], to: Closed}\nactors:\n  - name: demo.board.Clerk\n\
         \x20   attributes:\n      - {name: owner_id, type: demo.board.OwnerId}\n\
         \x20   may: [demo.board.Open, demo.board.Close",
    );
    for i in 1..=flags {
        let _ = write!(text, ", demo.board.Mark{i}");
    }
    text.push_str("]\nerrors:\n");
    for error in ["NoSuchCard", "AlreadyMarked", "StateConflict", "NotOwner"] {
        let _ = writeln!(
            text,
            "  - name: demo.board.{error}\n    fields:\n      - {{name: card_id, type: demo.board.CardId}}"
        );
    }
    text.push_str(
        "commands:\n  - name: demo.board.Open\n    input:\n      - {name: card_id, type: demo.board.CardId}\n\
         \x20   outcomes:\n      - name: opened\n        creates: demo.board.Card\n\
         \x20       instance: card_id\n        sets:\n          owner_id: {caller: owner_id}\n",
    );
    for i in 1..=flags {
        let _ = writeln!(text, "          flag_{i}: false");
    }
    text.push_str(
        "        emits: [demo.board.Opened]\n        payload:\n          demo.board.Opened: {card_id: input.card_id}\n",
    );
    for i in 1..=flags {
        let not_owner = if i <= reading {
            "      - name: not-owner\n        when_subject:\n          predicate: owner_id != caller.owner_id\n\
             \x20       error: demo.board.NotOwner\n"
        } else {
            ""
        };
        let _ = write!(
            text,
            "  - name: demo.board.Mark{i}\n    input:\n      - {{name: card_id, type: demo.board.CardId}}\n\
             \x20     - {{name: on, type: Boolean}}\n    outcomes:\n      - name: no-card\n\
             \x20       unknown_instance: true\n        error: demo.board.NoSuchCard\n\
             \x20     - name: already-marked\n        when_subject:\n          predicate: flag_{i} == true\n\
             \x20       error: demo.board.AlreadyMarked\n{not_owner}      - name: marked\n\
             \x20       moves: demo.board.Card.mark_{i}\n        instance: card_id\n        sets:\n\
             \x20         flag_{i}: input.on\n        emits: [demo.board.Marked]\n        payload:\n\
             \x20         demo.board.Marked: {{card_id: input.card_id}}\n      - name: wrong-state\n\
             \x20       wrong_state: true\n        error: demo.board.StateConflict\n"
        );
    }
    text.push_str(
        "  - name: demo.board.Close\n    input:\n      - {name: card_id, type: demo.board.CardId}\n\
         \x20   outcomes:\n      - name: no-card\n        unknown_instance: true\n\
         \x20       error: demo.board.NoSuchCard\n      - name: not-owner\n        when_subject:\n\
         \x20         predicate: owner_id != caller.owner_id\n        error: demo.board.NotOwner\n\
         \x20     - name: closed\n        moves: demo.board.Card.close\n        instance: card_id\n\
         \x20       emits: [demo.board.Closed]\n        payload:\n\
         \x20         demo.board.Closed: {card_id: input.card_id}\n      - name: wrong-state\n\
         \x20       wrong_state: true\n        error: demo.board.StateConflict\nevents:\n",
    );
    for event in ["Opened", "Marked", "Closed"] {
        let _ = writeln!(
            text,
            "  - name: demo.board.{event}\n    fields:\n      - {{name: card_id, type: demo.board.CardId}}"
        );
    }
    text.push_str(
        "views:\n  - name: demo.board.Cards\n    source: demo.board.Card\n    consistency: read_your_writes\n\
         \x20   fields:\n      - {name: card_id, type: demo.board.CardId}\n\
         \x20     - {name: state, type: demo.board.Card.State}\n\
         \x20     - {name: owner_id, type: demo.board.OwnerId}\n",
    );
    for i in 1..=flags {
        let _ = writeln!(text, "      - {{name: flag_{i}, type: Boolean}}");
    }
    text
}

/// `VmHWM` of a `/proc/<pid>/status` text, in KiB.
pub fn peak_kib(status: &str) -> Option<u64> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|number| number.parse().ok())
}

/// `cutime` and `cstime` (fields 16 and 17) of a `/proc/self/stat` line, in clock ticks. The
/// command name may hold spaces and parentheses, so the line is split after its last `)`.
pub fn children_ticks(stat: &str) -> Option<(u64, u64)> {
    let after = &stat[stat.rfind(')')? + 1..];
    // After the name, the first field is field 3 (`state`).
    let mut fields = after.split_whitespace();
    let user = fields.nth(16 - 3)?.parse().ok()?;
    let system = fields.next()?.parse().ok()?;
    Some((user, system))
}

/// One measured run: jobs, suite digest, scenario count.
pub type Row = (usize, String, usize);

/// Refuse when any two runs wrote a different suite.
pub fn compare(rows: &[Row]) -> Result<(), String> {
    let Some((first_jobs, first_digest, first_count)) = rows.first() else {
        return Ok(());
    };
    for (jobs, digest, count) in &rows[1..] {
        if digest != first_digest {
            return Err(format!(
                "the suites differ: {first_jobs} jobs wrote sha256 {first_digest}, {jobs} jobs wrote {digest}"
            ));
        }
        if count != first_count {
            return Err(format!(
                "the suites differ: {first_jobs} jobs wrote {first_count} scenarios, {jobs} jobs wrote {count}"
            ));
        }
    }
    Ok(())
}

struct Run {
    jobs: usize,
    wall: f64,
    user: f64,
    system: f64,
    peak_kib: u64,
    digest: String,
    scenarios: usize,
}

fn load() -> String {
    fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|text| text.split_whitespace().next().map(str::to_owned))
        .unwrap_or_else(|| "?".to_owned())
}

fn ticks() -> Result<(u64, u64)> {
    let stat = fs::read_to_string("/proc/self/stat")?;
    children_ticks(&stat).ok_or_else(|| anyhow!("cannot read /proc/self/stat"))
}

fn measure(
    ess: &Path,
    model: &Path,
    scenarios: Option<&Path>,
    out: &Path,
    jobs: usize,
) -> Result<Run> {
    let mut command = std::process::Command::new(ess);
    command
        .args(["verify", "conform", "synthesize", "--path"])
        .arg(model)
        .args(["--target", "ir", "--compact", "--out"])
        .arg(out)
        .env("ESS_SYNTH_JOBS", jobs.to_string())
        .stdout(std::process::Stdio::null())
        .stderr(fs::File::create(out.with_extension("log"))?);
    if let Some(scenarios) = scenarios {
        command.arg("--scenarios").arg(scenarios);
    }
    let (user_before, system_before) = ticks()?;
    let start = std::time::Instant::now();
    let mut child = command
        .spawn()
        .with_context(|| format!("cannot start {}", ess.display()))?;
    let status_path = format!("/proc/{}/status", child.id());
    let mut peak = 0;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if let Some(kib) = fs::read_to_string(&status_path)
            .ok()
            .as_deref()
            .and_then(peak_kib)
        {
            peak = peak.max(kib);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let wall = start.elapsed().as_secs_f64();
    let (user_after, system_after) = ticks()?;
    if !status.success() {
        bail!(
            "{jobs} jobs: ess exited with {status}; stderr in {}",
            out.with_extension("log").display()
        );
    }
    let bytes = fs::read(out).with_context(|| format!("cannot read {}", out.display()))?;
    let document: serde_json::Value = serde_json::from_slice(&bytes)?;
    let scenarios = document
        .get("scenarios")
        .and_then(serde_json::Value::as_object)
        .map(serde_json::Map::len)
        .ok_or_else(|| anyhow!("{}: no `scenarios` object", out.display()))?;
    let digest = Sha256::digest(&bytes)
        .iter()
        .fold(String::new(), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        });
    #[allow(clippy::cast_precision_loss)] // Tick counts are far below 2^52.
    let seconds = |ticks: u64| ticks as f64 / 100.0;
    Ok(Run {
        jobs,
        wall,
        user: seconds(user_after - user_before),
        system: seconds(system_after - system_before),
        peak_kib: peak,
        digest,
        scenarios,
    })
}

pub fn run(root: &Path, args: &Args) -> Result<String> {
    if !cfg!(target_os = "linux") {
        bail!("synth-jobs reads /proc and runs on Linux only");
    }
    let work = root.join("target/synth-jobs");
    fs::create_dir_all(&work)?;
    let (model_path, described) = if let Some(path) = &args.path {
        (path.clone(), format!("model {}", path.display()))
    } else {
        let path = work.join("board.yaml");
        fs::write(&path, model(args.flags, args.reading))?;
        (
            path,
            format!(
                "generated model: {} flags, {} reading, {} commands",
                args.flags,
                args.reading,
                args.flags + 2
            ),
        )
    };
    let ess = if args.ess.is_absolute() {
        args.ess.clone()
    } else {
        root.join(&args.ess)
    };
    let load_before = load();
    let mut runs = Vec::new();
    for &jobs in &args.jobs {
        let out = work.join(format!("suite-{jobs}.json"));
        runs.push(measure(
            &ess,
            &model_path,
            args.scenarios.as_deref(),
            &out,
            jobs,
        )?);
    }
    let load_after = load();
    let rows: Vec<Row> = runs
        .iter()
        .map(|run| (run.jobs, run.digest.clone(), run.scenarios))
        .collect();
    let base = runs.first().map_or(0.0, |run| run.wall);
    let mut text = String::from(
        "| jobs | wall s | user s | sys s | peak MiB | speed-up | scenarios | suite sha256 |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    for run in &runs {
        #[allow(clippy::cast_precision_loss)] // KiB counts are far below 2^52.
        let mib = run.peak_kib as f64 / 1024.0;
        let _ = writeln!(
            text,
            "| {} | {:.1} | {:.1} | {:.1} | {:.0} | {:.2}x | {} | {} |",
            run.jobs,
            run.wall,
            run.user,
            run.system,
            mib,
            base / run.wall,
            run.scenarios,
            run.digest
        );
    }
    let cores = std::thread::available_parallelism().map_or(0, std::num::NonZeroUsize::get);
    let _ = writeln!(
        text,
        "\n{described}; {cores} cores; load average {load_before} before, {load_after} after."
    );
    text.push_str(
        "Peak memory is polled every 20 ms, so it is a lower bound by at most the last 20 ms.\n",
    );
    compare(&rows).map_err(|message| anyhow!("{text}\n{message}"))?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
    use ess_domain::{
        spec::{RawSpecFile, Specification},
        system::Source,
    };

    fn ir(text: &str) -> EssIr {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
        let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
        compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
    }

    #[test]
    fn the_generated_model_compiles_and_reads_the_caller_in_its_marks() {
        let text = model(2, 1);
        let ir = ir(&text);
        let names: Vec<String> = ir.commands().keys().map(ToString::to_string).collect();
        assert_eq!(names.len(), 4, "{names:#?}");
        for name in ["Open", "Mark1", "Mark2", "Close"] {
            assert!(names.iter().any(|n| n.contains(name)), "{name}");
        }
        let mark = |name: &str| {
            let at = text.find(&format!("demo.board.{name}\n")).unwrap();
            let rest = &text[at..];
            rest[..rest[1..]
                .find("\n  - name:")
                .map_or(rest.len(), |end| end + 1)]
                .to_owned()
        };
        assert!(mark("Mark1").contains("owner_id != caller.owner_id"));
        assert!(!mark("Mark2").contains("owner_id != caller.owner_id"));
    }

    #[test]
    fn the_generated_model_grows_with_its_flags() {
        assert_eq!(ir(&model(24, 12)).commands().len(), 26);
    }

    #[test]
    fn a_status_line_gives_the_peak_resident_memory() {
        assert_eq!(peak_kib("Name:\tess\nVmHWM:\t  123456 kB\n"), Some(123_456));
        assert_eq!(peak_kib("Name:\tess\nVmRSS:\t 1 kB\n"), None);
    }

    #[test]
    fn a_stat_line_gives_the_childrens_cpu_ticks() {
        let line = "12 (a b) c) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18";
        // After the name: S=3, 1=4 ... so field 16 is the 14th number after "S".
        assert_eq!(children_ticks(line), Some((13, 14)));
    }

    #[test]
    fn differing_suites_are_refused_with_both_counts() {
        let a = || "a".to_owned();
        let b = || "b".to_owned();
        let error = compare(&[(1, a(), 7), (2, a(), 7), (4, b(), 7)]).unwrap_err();
        assert!(error.contains('1') && error.contains('4'), "{error}");
        assert!(compare(&[(1, a(), 7), (2, a(), 8)]).is_err());
        assert!(compare(&[(1, a(), 7), (2, a(), 7), (4, a(), 7)]).is_ok());
    }
}
