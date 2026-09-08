//! Per-bundle child watchdog and complete stream admission; a process status alone never qualifies.
use crate::{
    observation::{self, Entry, Outcome, Summary},
    pipeline::Control,
    Result,
};
use serde::Serialize;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
pub const DEADLINE: Duration = Duration::from_secs(10);
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    Completed,
    LaunchFailure,
    Timeout,
    Crash,
    ObservationFailure,
}
#[derive(Debug, Serialize)]
pub struct Execution {
    pub disposition: Disposition,
    pub pid: Option<u32>,
    pub status: Option<i32>,
    pub reaped: bool,
    pub elapsed_millis: u128,
    pub detail: String,
    pub summary: Option<Summary>,
}
struct Reap(Option<Child>);
impl Drop for Reap {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
pub fn fresh_default() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("target/replay/{}-{nonce}", std::process::id()))
}
pub fn child(
    executable: &Path,
    entry: Entry,
    input: &[u8],
    root: &Path,
    control: Control,
    deadline: Duration,
) -> Result<Execution> {
    supervised(
        executable,
        &["--child", entry.name()],
        input,
        root,
        control.name(),
        deadline,
    )
}
pub fn supervised(
    executable: &Path,
    prefix: &[&str],
    input: &[u8],
    root: &Path,
    control: &str,
    deadline: Duration,
) -> Result<Execution> {
    if let Some(parent) = root.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(root)?;
    let input_path = root.join("original-input");
    fs::write(&input_path, input)?;
    let output_path = root.join("observations");
    let mut command = Command::new(executable);
    command
        .args(prefix)
        .arg(&input_path)
        .arg(&output_path)
        .arg(control)
        .stdout(Stdio::from(File::create(root.join("stdout"))?))
        .stderr(Stdio::from(File::create(root.join("stderr"))?));
    fs::write(
        root.join("command.txt"),
        format!(
            "{command:?}\noriginal_sha256={}\ndeadline_millis={}\n",
            crate::digest(input),
            deadline.as_millis()
        ),
    )?;
    let start = Instant::now();
    let mut child = match command.spawn() {
        Ok(child) => Reap(Some(child)),
        Err(error) => {
            let result = Execution {
                disposition: Disposition::LaunchFailure,
                pid: None,
                status: None,
                reaped: false,
                elapsed_millis: start.elapsed().as_millis(),
                detail: error.to_string(),
                summary: None,
            };
            fs::write(
                root.join("execution.json"),
                serde_json::to_vec_pretty(&result)?,
            )?;
            return Ok(result);
        }
    };
    let pid = child.0.as_ref().unwrap().id();
    let mut timeout = false;
    let status = loop {
        if let Some(status) = child.0.as_mut().unwrap().try_wait()? {
            break status;
        }
        if start.elapsed() >= deadline {
            child.0.as_mut().unwrap().kill()?;
            timeout = true;
            break child.0.as_mut().unwrap().wait()?;
        }
        thread::sleep(Duration::from_millis(5));
    };
    // Save the actual observed child status before a stream-read or later I/O can fail.
    fs::write(
        root.join("direct-status.txt"),
        format!(
            "pid={pid}\nstatus={status:?}\ncode={:?}\ntimeout={timeout}\nreaped=true\n",
            status.code()
        ),
    )?;
    child.0.take();
    let (disposition, detail, summary) = if timeout {
        (
            Disposition::Timeout,
            "monotonic deadline expired; child killed and reaped".into(),
            None,
        )
    } else if status.code() == Some(74) {
        (
            Disposition::ObservationFailure,
            "child reported operational observation failure".into(),
            None,
        )
    } else if !status.success() {
        (
            Disposition::Crash,
            format!("child failed: {status:?}"),
            None,
        )
    } else {
        match observation::admit(&output_path) {
            Ok(summary) if summary.attempts == 1 => {
                (Disposition::Completed, String::new(), Some(summary))
            }
            Ok(_) => (
                Disposition::ObservationFailure,
                "one bundle child must complete exactly one attempt".into(),
                None,
            ),
            Err(error) => (Disposition::ObservationFailure, error.to_string(), None),
        }
    };
    let execution = Execution {
        disposition,
        pid: Some(pid),
        status: status.code(),
        reaped: true,
        elapsed_millis: start.elapsed().as_millis(),
        detail,
        summary,
    };
    fs::write(
        root.join("execution.json"),
        serde_json::to_vec_pretty(&execution)?,
    )?;
    Ok(execution)
}
fn control_bundle(text: &str) -> Vec<u8> {
    crate::carrier::Bundle {
        documents: vec![crate::carrier::Document {
            label: "control.yaml".into(),
            text: text.into(),
        }],
    }
    .encode()
    .unwrap()
}
pub fn run(executable: &Path, root: &Path) -> Result<Vec<Execution>> {
    crate::regressions::verify()?;
    crate::structured::verify_vectors()?;
    fs::create_dir_all(root)?;
    let mut cases = Vec::new();
    for name in crate::regressions::NAMES {
        cases.push((
            name.to_owned(),
            Entry::ByteCarrier,
            crate::regressions::encoded(name).to_vec(),
            Outcome::Compiled,
        ));
    }
    let mut identities = std::collections::BTreeSet::new();
    for (i, input) in crate::structured::vectors().into_iter().enumerate() {
        identities.insert(crate::digest(&crate::structured::render(&input)?.encode()?));
        cases.push((
            format!("structured-{i:02}"),
            Entry::Structured,
            input,
            Outcome::Compiled,
        ));
    }
    if identities.len() < 16 {
        return Err("structured source diversity below sixteen distinct bundles".into());
    }
    cases.extend([
        ("decode-refusal".into(),Entry::ByteCarrier,b"{".to_vec(),Outcome::InputRefused),
        ("parse-refusal".into(),Entry::ByteCarrier,control_bundle("{"),Outcome::ParseRefused),
        ("validation-refusal".into(),Entry::ByteCarrier,control_bundle("format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\ndomain: demo.core\ntypes:\n  - {name: demo.core.Bad, kind: enum, variants: []}\n"),Outcome::ValidationRefused),
        ("undeclared-type-validation-refusal".into(),Entry::ByteCarrier,control_bundle("format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\ndomain: demo.core\ntypes:\n  - {name: demo.core.Bad, kind: newtype, of: demo.core.Missing}\n"),Outcome::ValidationRefused),
        ("outside-domain".into(),Entry::ByteCarrier,control_bundle("format: ess/2\nsystem: demo\nversion: v1\ndomains: [demo.core]\ndomain: demo.core\n"),Outcome::OutsideDomain{major:2}),
        ("default-format".into(),Entry::ByteCarrier,control_bundle("system: demo\nversion: v1\ndomains: [demo.core]\ndomain: demo.core\n"),Outcome::Compiled),
    ]);
    let mut outcomes = Vec::new();
    let mut errors = Vec::new();
    for (name, entry, input, expected) in cases {
        let result = child(
            executable,
            entry,
            &input,
            &root.join(&name),
            Control::None,
            DEADLINE,
        )?;
        println!("case {name}: {:?}", result.disposition);
        if result.disposition != Disposition::Completed {
            errors.push(format!("{name}: {}", result.detail));
        } else {
            let summary = result.summary.as_ref().unwrap();
            let expected_key = match expected {
                Outcome::Compiled => "compiled",
                Outcome::InputRefused => "input-refused",
                Outcome::ParseRefused => "parse-refused",
                Outcome::ValidationRefused => "validation-refused",
                Outcome::CompileRefused => "compile-refused",
                Outcome::OutsideDomain { .. } => "outside-domain",
            };
            if summary.outcomes.get(expected_key) != Some(&1) {
                errors.push(format!(
                    "{name}: expected {expected:?}, got {:?}",
                    summary.outcomes
                ));
            }
        }
        outcomes.push(result);
    }
    fs::write(
        root.join("executions.json"),
        serde_json::to_vec_pretty(&outcomes)?,
    )?;
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    Ok(outcomes)
}
