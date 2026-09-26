//! Installs an executable a case is about to run, so that the kernel never refuses to run it.
//!
//! `cargo test` runs the cases of one binary as threads of one process. Writing a file opens it
//! for writing, and a child that any other thread forks while that descriptor is open inherits it.
//! `O_CLOEXEC` closes it only when the child executes, and until then the kernel refuses to
//! execute the file (`ETXTBSY`, "Text file busy"). Closing the file before running it does not
//! help, and neither does writing it under another name and renaming it into place: the child's
//! descriptor is on the inode, and a rename keeps the inode.
//!
//! So this process never opens an installed executable for writing at all. `cp`, a separate
//! process that forks nothing, creates and fills it, and has exited before these functions return.
//! From then on no process holds a descriptor on the file that a fork could inherit, and none ever
//! will, so there is no window to wait out, retry or probe.
use std::{
    fs,
    io::{self, Write as _},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

/// Installs `bytes` at `destination` with mode `0o755`.
///
/// `cp` reads the bytes from its standard input, so no staging file is left anywhere. Another
/// fork may inherit this process's end of that pipe, which delays nothing an `exec` depends on.
pub fn install_bytes(destination: &Path, bytes: &[u8]) -> io::Result<()> {
    copy_in_a_child(Path::new("/dev/stdin"), destination, Some(bytes))?;
    fs::set_permissions(destination, fs::Permissions::from_mode(0o755))
}

/// Installs a copy of `source` at `destination` with `source`'s permissions, as `fs::copy` would.
pub fn install_copy(source: &Path, destination: &Path) -> io::Result<()> {
    copy_in_a_child(source, destination, None)?;
    fs::set_permissions(destination, fs::metadata(source)?.permissions())
}

/// Copies `source` to `destination` in a `cp` process, feeding it `input`, and waits for it.
fn copy_in_a_child(source: &Path, destination: &Path, input: Option<&[u8]>) -> io::Result<()> {
    let mut child = Command::new("/bin/cp")
        .arg("--")
        .arg(source)
        .arg(destination)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    if let (Some(bytes), Some(mut stdin)) = (input, child.stdin.take()) {
        stdin.write_all(bytes)?;
    }
    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "cp {} {} exited {}: {}",
            source.display(),
            destination.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim_end()
        )))
    }
}

/// Installs and executes `rounds` fresh executables while three threads keep forking children, and
/// returns every round that did not execute, with the reason; a busy refusal reads
/// `Text file busy`.
///
/// `install` places round `n`'s executable and returns the command that runs it; that command
/// must exit promptly. The spawners fork `/bin/sh -c :`, which is what any other case in the binary
/// does when it starts a child.
pub fn unlaunched_rounds(rounds: usize, install: impl Fn(usize) -> Command) -> Vec<String> {
    let stop = Arc::new(AtomicBool::new(false));
    let spawners: Vec<_> = (0..3)
        .map(|_| {
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let _ = Command::new("/bin/sh")
                        .args(["-c", ":"])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
            })
        })
        .collect();
    let mut unlaunched = Vec::new();
    for round in 0..rounds {
        let mut command = install(round);
        let status = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if let Err(error) = status {
            unlaunched.push(format!("round {round}: {error}"));
        }
    }
    stop.store(true, Ordering::Relaxed);
    for spawner in spawners {
        spawner.join().expect("a spawner thread finishes");
    }
    unlaunched
}
