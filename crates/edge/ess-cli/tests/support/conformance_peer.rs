//! Test-only. A protocol peer over the billing reference that misbehaves on request.
//!
//! `ess verify conform serve --target billing` is the public, well-behaved peer of
//! `ess-conformance-target/1`. The fault cases of `tests/process_target.rs` need one that exits,
//! writes garbage, answers another request, stalls, refuses to restart, declines an operation or
//! speaks another version, which a public verb should not offer
//! (`docs/design/conformance-target-process.md`, "Test peer"). Every answer it does give is
//! `ess_conformance::process::serve`'s over `Billing`, one request at a time.
//!
//! Options, each misbehaviour bound to the first `execute_command` of the named scenario:
//!
//! - `--wrong-outcome <scenario>`: answers that command with the outcome `rejected`;
//! - `--exit-in <scenario>`: exits 1 instead of answering;
//! - `--garbage-in <scenario>`: answers with a line that is not JSON;
//! - `--wrong-id-in <scenario>`: answers under another request's `id`;
//! - `--stall-in <scenario>`: never answers;
//! - `--unsupported <op>`: answers every request of that operation `unsupported`;
//! - `--speaks <format>`: names that version in its handshake;
//! - `--pid-file <file>`: starts a grandchild in the peer's process group and appends
//!   `<own pid> <grandchild pid>` to the file;
//! - `--fail-restart`: with `--pid-file`, exits 3 before answering anything when the file already
//!   names a process, so every restart fails;
//! - `--record <file>`: appends each request's `op` to the file, one per line.

use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

use ess_conformance::process::{serve, TARGET_FORMAT};
use ess_conformance::reference::Billing;
use ess_conformance::ConformanceTarget;

/// The reason every declined operation carries, so a test can find it in a report.
const DECLINES: &str = "the conformance peer declines";

#[derive(Default)]
struct Options {
    wrong_outcome: Option<String>,
    exit_in: Option<String>,
    garbage_in: Option<String>,
    wrong_id_in: Option<String>,
    stall_in: Option<String>,
    unsupported: Vec<String>,
    speaks: Option<String>,
    pid_file: Option<PathBuf>,
    fail_restart: bool,
    record: Option<PathBuf>,
    linger: bool,
}

fn options() -> Result<Options, String> {
    let mut options = Options::default();
    let mut arguments = std::env::args().skip(1);
    while let Some(flag) = arguments.next() {
        let mut value = || {
            arguments
                .next()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--wrong-outcome" => options.wrong_outcome = Some(value()?),
            "--exit-in" => options.exit_in = Some(value()?),
            "--garbage-in" => options.garbage_in = Some(value()?),
            "--wrong-id-in" => options.wrong_id_in = Some(value()?),
            "--stall-in" => options.stall_in = Some(value()?),
            "--unsupported" => options.unsupported.push(value()?),
            "--speaks" => options.speaks = Some(value()?),
            "--pid-file" => options.pid_file = Some(PathBuf::from(value()?)),
            "--record" => options.record = Some(PathBuf::from(value()?)),
            "--fail-restart" => options.fail_restart = true,
            "--linger" => options.linger = true,
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(options)
}

fn append(file: &PathBuf, text: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .expect("the peer's record file opens");
    file.write_all(text.as_bytes())
        .expect("the peer's record file is writable");
}

fn stall() -> ! {
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}

/// `serve`'s answer to one request line.
fn served(billing: &Billing, line: &str) -> String {
    let mut out = Vec::new();
    let mut input = line.as_bytes().to_vec();
    input.push(b'\n');
    if let Err(error) = serve(billing, input.as_slice(), &mut out) {
        eprintln!("conformance peer: {error}");
        std::process::exit(2);
    }
    String::from_utf8(out)
        .expect("serve writes UTF-8")
        .trim_end_matches('\n')
        .to_owned()
}

fn main() -> ExitCode {
    let options = match options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("conformance peer: {message}");
            return ExitCode::from(64);
        }
    };
    if options.linger {
        stall();
    }
    if let Some(pid_file) = &options.pid_file {
        let restart = fs::metadata(pid_file).is_ok_and(|meta| meta.len() > 0);
        if restart && options.fail_restart {
            append(pid_file, &format!("{}\n", std::process::id()));
            return ExitCode::from(3);
        }
        // Stays in this process's group, as a server an adapter starts would. Never waited for on
        // purpose: `ess` must end it, by killing the group, and the case checks that it did.
        #[allow(clippy::zombie_processes)]
        let grandchild = Command::new(std::env::current_exe().expect("the peer knows its path"))
            .arg("--linger")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the grandchild starts");
        append(
            pid_file,
            &format!("{} {}\n", std::process::id(), grandchild.id()),
        );
    }
    let billing = Billing::new();
    let mut stdout = io::stdout().lock();
    let mut scenario = String::new();
    let mut commanded = false;
    for line in io::stdin().lock().lines() {
        let Ok(line) = line else {
            return ExitCode::from(2);
        };
        let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) else {
            // `serve` refuses it with its own fault kind.
            served(&billing, &line);
            return ExitCode::from(2);
        };
        let op = request["op"].as_str().unwrap_or_default().to_owned();
        let id = request["id"].as_u64().unwrap_or_default();
        if let Some(record) = &options.record {
            append(record, &format!("{op}\n"));
        }
        if op == "begin_scenario" {
            request["request"]["scenario"]
                .as_str()
                .unwrap_or_default()
                .clone_into(&mut scenario);
            commanded = false;
        }
        let first_command = op == "execute_command" && !commanded;
        if op == "execute_command" {
            commanded = true;
        }
        let here = |chosen: &Option<String>| first_command && chosen.as_deref() == Some(&scenario);
        let answer = if op == "handshake" && options.speaks.is_some() {
            let implementation = billing.identity().expect("billing names itself");
            serde_json::json!({"id": id, "ok": {
                "speaks": options.speaks.as_deref().unwrap_or(TARGET_FORMAT),
                "implementation": implementation,
            }})
            .to_string()
        } else if options.unsupported.contains(&op) {
            serde_json::json!({"id": id, "target_error": {
                "kind": "unsupported", "observation": op, "why": format!("{DECLINES} {op}"),
            }})
            .to_string()
        } else if here(&options.exit_in) {
            return ExitCode::from(1);
        } else if here(&options.stall_in) {
            stall();
        } else if here(&options.garbage_in) {
            "this line is not a protocol message".to_owned()
        } else if here(&options.wrong_id_in) {
            let mut answer: serde_json::Value =
                serde_json::from_str(&served(&billing, &line)).expect("serve writes JSON");
            answer["id"] = serde_json::json!(id + 1000);
            answer.to_string()
        } else if here(&options.wrong_outcome) {
            let mut answer: serde_json::Value =
                serde_json::from_str(&served(&billing, &line)).expect("serve writes JSON");
            answer["ok"]["outcome"]["outcome"] = serde_json::json!("rejected");
            answer.to_string()
        } else {
            served(&billing, &line)
        };
        if writeln!(stdout, "{answer}")
            .and_then(|()| stdout.flush())
            .is_err()
        {
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}
