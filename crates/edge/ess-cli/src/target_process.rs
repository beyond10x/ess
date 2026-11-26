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
//!
//! # A signalled `ess`
//!
//! The child's own process group does not receive the interrupt a terminal sends to `ess`'s, and
//! `ess` dying of a signal would run no `Drop`. So from the first spawn on, a thread of its own
//! waits for `SIGINT`, `SIGTERM` and `SIGHUP`, and is the only thread that takes them
//! ([`watch_signals`]). On the first it kills the group of
//! the target alive then, reaps it within [`SIGNAL_REAP`], and ends `ess` by that same signal,
//! writing no report ([`end_on`]). `SIGKILL` cannot be handled; a target whose `ess` was killed
//! exits when its standard input closes, as the design requires of every target.
//!
//! The group alive is [`ALIVE`], and nothing changes it outside its lock: a spawn registers its
//! child before the lock is released, and a reap clears it before the lock is released. The
//! signal thread takes the same lock and never releases it, so it sees either no target or one
//! that is registered and not yet reaped (its group id therefore not reusable), and no target is
//! spawned after it.

use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use ess_conformance::process::{Exit, Peer, PeerFault, MAX_LINE};
use nix::sys::signal::{SigSet, Signal as Sig};
use rustix::process::{Pid, Signal, WaitId, WaitIdOptions};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};

/// The signals that end `ess` and, with it, the target.
const ENDING: [i32; 3] = [SIGINT, SIGTERM, SIGHUP];

/// The leader of the target's process group while one is alive: registered by a spawn and cleared
/// by a reap, each under this lock (see the [module documentation](self)).
static ALIVE: Mutex<Option<Pid>> = Mutex::new(None);

fn alive() -> MutexGuard<'static, Option<Pid>> {
    ALIVE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// [`ENDING`] as a signal set.
fn ending() -> SigSet {
    let mut set = SigSet::empty();
    for signal in [Sig::SIGINT, Sig::SIGTERM, Sig::SIGHUP] {
        set.add(signal);
    }
    set
}

/// Starts the thread that ends the target when `ess` is signalled, once per process; before the
/// first spawn, so no target is ever alive without it.
///
/// Only that thread takes an ending signal. A process-directed signal goes to any thread that does
/// not block it, so with every thread open a SIGINT and a SIGTERM sent right after it can run their
/// handlers on two threads at once, and whichever records first decides. So the calling thread,
/// `main`'s, which spawns the target and its pipe threads, blocks [`ENDING`] before the handlers
/// exist, every thread started from it later inherits the block, and the signal thread alone
/// unblocks it. Its handlers then run one at a time, in the order the kernel hands the signals
/// out, and the first decides. A signal sent in between stays pending until the signal thread
/// unblocks it. `std::process::Command` resets a child's mask.
fn watch_signals() -> std::io::Result<()> {
    static WATCHING: OnceLock<Result<(), std::io::ErrorKind>> = OnceLock::new();
    let watching = WATCHING.get_or_init(|| {
        let set = ending();
        set.thread_block()
            .map_err(|errno| std::io::Error::from(errno).kind())?;
        let started = signal_hook::iterator::Signals::new(ENDING).and_then(|mut signals| {
            std::thread::Builder::new()
                .name("ess-target-signals".to_owned())
                .spawn(move || {
                    // Unblocking a valid set in the calling thread cannot fail.
                    let _ = set.thread_unblock();
                    if let Some(signal) = signals.forever().next() {
                        end_on(signal);
                    }
                })
                .map(drop)
        });
        if started.is_err() {
            // No thread takes them: open the calling thread again rather than swallow them.
            let _ = set.thread_unblock();
        }
        started.map_err(|error| error.kind())
    });
    watching.map_err(std::io::Error::from)
}

/// How long a signalled `ess` waits to reap the target it has killed. A child killed with
/// `SIGKILL` is reaped within milliseconds; one that is not reaped in 2 s will not be (it is stuck
/// in an uninterruptible wait, or the kill was refused because it changed its user), and every
/// later signal is swallowed while `ess` waits, so the person who pressed Ctrl-C waits at most
/// this long before `ess` ends anyway.
const SIGNAL_REAP: Duration = Duration::from_secs(2);

/// Kills the target alive and reaps it within [`SIGNAL_REAP`], then ends `ess` by `signal` itself:
/// a parent sees a process terminated by that signal, as it would without the handler, so a shell
/// looping over runs stops on Ctrl-C. `signal_hook`'s emulation restores the default action and
/// raises the signal again; should that return, `ess` exits `128 + signal`, the status a shell
/// shows for it. The lock is never released, so nothing is spawned after this.
fn end_on(signal: i32) -> ! {
    let alive = alive();
    if let Some(pid) = *alive {
        kill_group(pid);
        reap_within(pid, SIGNAL_REAP);
    }
    let _ = signal_hook::low_level::emulate_default_handler(signal);
    std::process::exit(128 + signal);
}

/// Waits up to `limit` for the child `pid` to exit and reaps it; whether it was reaped.
fn reap_within(pid: Pid, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    loop {
        match rustix::process::waitid(
            WaitId::Pid(pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG,
        ) {
            Ok(Some(_)) => return true,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL),
            Ok(None) | Err(_) => return false,
        }
    }
}

/// `SIGKILL` to the group `leader` leads, and to `leader` itself, which may have left its group;
/// either way the reap that follows cannot wait forever. `leader` must not have been reaped yet.
fn kill_group(leader: Pid) {
    // ESRCH: the whole group has already exited, and the child waits to be reaped.
    let _ = rustix::process::kill_process_group(leader, Signal::KILL);
    let _ = rustix::process::kill_process(leader, Signal::KILL);
}

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
        watch_signals()?;
        // Spawned and registered under one lock: a signal in between waits for the registration
        // and then kills this child's group.
        let mut alive = alive();
        let mut child = command.spawn()?;
        let pid = Pid::from_child(&child);
        *alive = Some(pid);
        drop(alive);
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
    /// child's when it is signalled. Under [`ALIVE`]'s lock, cleared before it is released.
    fn kill(&mut self) {
        self.requests = None;
        self.lines = None;
        let mut alive = alive();
        kill_group(self.pid);
        let _ = self.child.wait();
        if *alive == Some(self.pid) {
            *alive = None;
        }
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

#[cfg(test)]
mod tests;
