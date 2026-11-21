//! The child process behind `ess verify conform run --target process -- <program> [args…]`.
//!
//! `docs/design/conformance-target-process.md`, "The process lifecycle", is the binding design.
//! [`ChildPeer`] is the [`Peer`] `ess_conformance::process::ProcessTarget` talks to: it starts the
//! program in its own process group with standard input and output piped and standard error
//! inherited, hands request lines to a writer thread, and reads answer lines, each bounded by
//! [`MAX_LINE`], on a reader thread. Neither thread can block a request past its limit.
//!
//! Every way a process ends here signals its whole process group before the child is reaped, so the
//! group id cannot have been reused and a server the program started in its group dies with it.

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::time::{Duration, Instant};

use ess_conformance::process::{Exit, Peer, PeerFault, MAX_LINE};
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions};

/// How long `ess` waits for the program to exit after closing its standard input at the end of a
/// run, before it kills the group.
const END_GRACE: Duration = Duration::from_secs(5);

/// How long, once standard output has ended or the program has exited, `ess` waits for the other
/// half: the exit status, or a last line still in the pipe.
const SETTLE: Duration = Duration::from_millis(500);

/// How often a wait looks at the child.
const POLL: Duration = Duration::from_millis(10);

/// What the program is and where it starts.
#[derive(Debug, Clone)]
pub struct Program {
    path: PathBuf,
    args: Vec<OsString>,
    dir: Option<PathBuf>,
}

impl Program {
    /// The argv after `--`, started in `dir` (the current directory when `None`).
    ///
    /// A relative program containing `/` is joined to `dir` and made absolute, so it names a file
    /// in the implementation's tree whatever directory `ess` runs in; a bare name is looked up on
    /// `PATH`; an absolute path is used as given. `None` for an empty argv.
    pub fn new(argv: &[OsString], dir: Option<&Path>) -> Option<Self> {
        let (program, arguments) = argv.split_first()?;
        let named = Path::new(program);
        let path = if named.is_relative() && program.as_encoded_bytes().contains(&b'/') {
            let base = dir.map_or_else(|| PathBuf::from("."), Path::to_path_buf);
            std::path::absolute(base.join(named)).unwrap_or_else(|_| base.join(named))
        } else {
            named.to_path_buf()
        };
        Some(Self {
            path,
            args: arguments.to_vec(),
            dir: dir.map(Path::to_path_buf),
        })
    }

    /// The program as it is spawned, for a refusal to name.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn spawn(&self) -> std::io::Result<Running> {
        use std::os::unix::process::CommandExt as _;

        let mut command = Command::new(&self.path);
        command
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .process_group(0);
        if let Some(dir) = &self.dir {
            command.current_dir(dir);
        }
        let mut child = command.spawn()?;
        let pid = Pid::from_child(&child);
        let (stdin, stdout) = (child.stdin.take(), child.stdout.take());
        let (Some(stdin), Some(stdout)) = (stdin, stdout) else {
            let mut running = Running::bare(child, pid);
            running.kill();
            return Err(std::io::Error::other("the child's pipes were not opened"));
        };
        let (requests, pending) = mpsc::channel::<Vec<u8>>();
        std::thread::spawn(move || {
            let mut stdin = stdin;
            for line in pending {
                if stdin.write_all(&line).and_then(|()| stdin.flush()).is_err() {
                    break;
                }
            }
        });
        let (deliver, lines) = mpsc::sync_channel::<Line>(2);
        std::thread::spawn(move || read_lines(stdout, &deliver));
        Ok(Running {
            child,
            pid,
            requests: Some(requests),
            lines: Some(lines),
        })
    }
}

/// What the reader thread delivers.
enum Line {
    /// One complete line, without its `\n`.
    Complete(Vec<u8>),
    /// More than [`MAX_LINE`] bytes with no `\n`; nothing more is read.
    TooLong,
    /// End of file, or a read error; nothing more is read.
    Ended,
}

fn read_lines(stdout: ChildStdout, deliver: &SyncSender<Line>) {
    let mut reader = BufReader::new(stdout);
    let limit = u64::try_from(MAX_LINE)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    loop {
        let mut line = Vec::new();
        let read = (&mut reader).take(limit).read_until(b'\n', &mut line);
        let delivered = match read {
            Ok(0) | Err(_) => {
                let _ = deliver.send(Line::Ended);
                return;
            }
            Ok(_) if line.last() == Some(&b'\n') => {
                line.pop();
                deliver.send(Line::Complete(line))
            }
            Ok(_) if line.len() > MAX_LINE => {
                let _ = deliver.send(Line::TooLong);
                return;
            }
            // A last line with no `\n` is not a complete line.
            Ok(_) => {
                let _ = deliver.send(Line::Ended);
                return;
            }
        };
        if delivered.is_err() {
            return;
        }
    }
}

/// One started process.
struct Running {
    child: Child,
    pid: Pid,
    requests: Option<mpsc::Sender<Vec<u8>>>,
    lines: Option<Receiver<Line>>,
}

impl Running {
    fn bare(child: Child, pid: Pid) -> Self {
        Self {
            child,
            pid,
            requests: None,
            lines: None,
        }
    }

    /// How the child ended, if it has, without reaping it.
    fn exited(&self) -> Option<Exit> {
        let status = rustix::process::waitid(
            WaitId::Pid(self.pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOWAIT | WaitIdOptions::NOHANG,
        )
        .ok()??;
        Some(if let Some(code) = status.exit_status() {
            Exit::Code(code)
        } else if let Some(signal) = status.terminating_signal() {
            Exit::Signal(signal)
        } else {
            Exit::Unknown
        })
    }

    /// Waits up to `within` for the child to exit, without reaping it.
    fn exit_within(&self, within: Duration) -> Option<Exit> {
        let deadline = Instant::now() + within;
        loop {
            if let Some(exit) = self.exited() {
                return Some(exit);
            }
            if Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(POLL);
        }
    }

    /// The fault for a child whose output ended or whose input is gone: its exit when it exits
    /// soon, otherwise a closed standard output.
    fn ended(&self) -> PeerFault {
        self.exit_within(SETTLE)
            .map_or(PeerFault::ClosedStdout, PeerFault::Exited)
    }

    /// Kills the process group, then reaps the child: in this order, so the group id is still the
    /// child's when it is signalled.
    fn kill(&mut self) {
        self.requests = None;
        self.lines = None;
        // ESRCH: the whole group has already exited, and the child waits to be reaped.
        let _ = rustix::process::kill_process_group(self.pid, Signal::KILL);
        let _ = self.child.wait();
    }
}

/// The [`Peer`] over a child process. See the [module documentation](self).
pub struct ChildPeer {
    program: Program,
    running: Option<Running>,
}

impl ChildPeer {
    /// Starts the program.
    pub fn start(program: Program) -> std::io::Result<Self> {
        let running = program.spawn()?;
        Ok(Self {
            program,
            running: Some(running),
        })
    }

    /// Ends the run: closes the child's standard input, waits up to five seconds for it to exit,
    /// kills its process group and reaps it. The exit status is not reported.
    pub fn finish(mut self) {
        if let Some(mut running) = self.running.take() {
            running.requests = None;
            let _ = running.exit_within(END_GRACE);
            running.kill();
        }
    }
}

impl Peer for ChildPeer {
    fn send(&mut self, line: &str) -> Result<(), PeerFault> {
        let running = self.running.as_ref().ok_or(PeerFault::ClosedStdout)?;
        let mut bytes = Vec::with_capacity(line.len() + 1);
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
        let sent = running
            .requests
            .as_ref()
            .is_some_and(|requests| requests.send(bytes).is_ok());
        if sent {
            Ok(())
        } else {
            Err(running.ended())
        }
    }

    fn receive(&mut self, limit_ms: u64) -> Result<Vec<u8>, PeerFault> {
        let running = self.running.as_ref().ok_or(PeerFault::ClosedStdout)?;
        let lines = running.lines.as_ref().ok_or(PeerFault::ClosedStdout)?;
        let deliver = |line: Line| match line {
            Line::Complete(bytes) => Ok(bytes),
            Line::TooLong => Err(PeerFault::LineTooLong),
            Line::Ended => Err(running.ended()),
        };
        let deadline = Instant::now() + Duration::from_millis(limit_ms);
        loop {
            let now = Instant::now();
            if now >= deadline {
                return Err(PeerFault::Timeout);
            }
            match lines.recv_timeout((deadline - now).min(Duration::from_millis(50))) {
                Ok(line) => return deliver(line),
                Err(RecvTimeoutError::Disconnected) => return Err(running.ended()),
                Err(RecvTimeoutError::Timeout) => {
                    // A child that exited while another process holds its output open would
                    // otherwise run the request to its limit.
                    if let Some(exit) = running.exited() {
                        return match lines.recv_timeout(SETTLE) {
                            Ok(Line::Complete(bytes)) => Ok(bytes),
                            Ok(Line::TooLong) => Err(PeerFault::LineTooLong),
                            _ => Err(PeerFault::Exited(exit)),
                        };
                    }
                }
            }
        }
    }

    fn kill(&mut self) {
        if let Some(mut running) = self.running.take() {
            running.kill();
        }
    }

    fn respawn(&mut self) -> Result<(), PeerFault> {
        self.kill();
        let running = self
            .program
            .spawn()
            .map_err(|error| PeerFault::NotStarted(error.kind()))?;
        self.running = Some(running);
        Ok(())
    }
}

impl Drop for ChildPeer {
    fn drop(&mut self) {
        self.kill();
    }
}
