//! The stand-in for the admitted Helm artifact, and the synthetic target it acts on.
//!
//! This is the `ess-recovery-fake` executable. Every engine-level case installs a byte-exact copy
//! of it as the admitted artifact, and the engine reads, hashes and probes that copy exactly as it
//! would a real one, so every byte here is paid by every case. It therefore speaks only the
//! bounded protocol it answers and links none of the product: its whole dependency set is `std`
//! and `serde_json`.
//!
//! It is also a module of `fake_recovery.rs`, which re-exports the target reader and writer and
//! the profile, so the fixture that *builds* an arrangement and the executable that *acts* on it
//! still agree byte for byte about where everything is.
//!
//! Compatible output is not proof of stock Helm's semantics, and nothing here claims it is.

// As a module of `fake_recovery.rs`, `main` and part of the protocol are unused.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// --- The synthetic target and the stand-in for the admitted Helm artifact ---------------------
//
// The target is a directory of JSON files, one per release, holding the release's stored manifest
// inventory, its description marker and the direct objects that exist. The driver never reads it;
// only the fake Helm and the fake API do, and only through their own process boundary.

/// One release's durable state in the synthetic target.
#[derive(Debug, Clone, Default)]
pub struct ReleaseState {
    /// The Helm description marker, absent when release storage is absent.
    pub description: Option<String>,
    /// The stored manifest inventory as `Kind/name` strings.
    pub manifest: Vec<String>,
    /// The direct objects that exist, as `Kind/name` to their projection digest.
    pub objects: BTreeMap<String, String>,
}

/// Reads one release's durable state out of the synthetic target.
pub fn read_release(target: &Path, namespace: &str, release: &str) -> ReleaseState {
    let path = target.join(format!("{namespace}--{release}.json"));
    let Ok(text) = std::fs::read_to_string(path) else {
        return ReleaseState::default();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return ReleaseState::default();
    };
    ReleaseState {
        description: value
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        manifest: value
            .get("manifest")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        objects: value
            .get("objects")
            .and_then(serde_json::Value::as_object)
            .map(|map| {
                map.iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|value| (key.clone(), value.to_owned()))
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// Writes one release's durable state into the synthetic target.
pub fn write_release(
    target: &Path,
    namespace: &str,
    release: &str,
    state: &ReleaseState,
) -> std::io::Result<()> {
    let path = target.join(format!("{namespace}--{release}.json"));
    if state.description.is_none() && state.objects.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    let document = serde_json::json!({
        "description": state.description,
        "manifest": state.manifest,
        "objects": state.objects,
    });
    std::fs::write(
        path,
        serde_json::to_string(&document).expect("state serializes"),
    )
}

/// The fixture profile, read from a file beside the installed artifact.
///
/// A file rather than an environment variable, deliberately. The production child environment is
/// *constructed*, not inherited — `process::environment` clears everything — so an environment
/// switch would be invisible to this executable when it runs as the admitted artifact, and a
/// production path that did let one through would be the defect. The profile lives beside the
/// binary, so it changes no byte of the binary the authority pinned.
#[derive(Debug, Default)]
pub struct Profile {
    /// What the version probe reports.
    pub version: Option<String>,
    /// A flag to omit from one operation's help output, as `operation:--flag`.
    pub omit: Option<String>,
    /// Which mutation fault to inject.
    pub fault: Option<String>,
    /// The stored manifest inventory an apply records, comma separated `Kind/name`.
    pub inventory: Option<String>,
    /// The independently controlled synthetic target's directory.
    pub target: Option<String>,
}

/// The profile file's name, beside the installed artifact.
pub const PROFILE: &str = "fixture-profile";

/// Writes a fixture profile beside an installed artifact.
pub fn write_profile(artifact: &Path, lines: &[(&str, &str)]) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let mut text = String::new();
    for (key, value) in lines {
        let _ = writeln!(text, "{key}={value}");
    }
    std::fs::write(
        artifact
            .parent()
            .expect("an installed artifact has a directory")
            .join(PROFILE),
        text,
    )
}

fn profile() -> Profile {
    let Ok(exe) = std::env::current_exe() else {
        return Profile::default();
    };
    let Some(directory) = exe.parent() else {
        return Profile::default();
    };
    let Ok(text) = std::fs::read_to_string(directory.join(PROFILE)) else {
        return Profile::default();
    };
    let mut profile = Profile::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = Some(value.to_owned());
        match key {
            "version" => profile.version = value,
            "omit" => profile.omit = value,
            "fault" => profile.fault = value,
            "inventory" => profile.inventory = value,
            "target" => profile.target = value,
            _ => {}
        }
    }
    profile
}

fn main() -> std::process::ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let profile = profile();
    let target = PathBuf::from(profile.target.clone().unwrap_or_default());
    match helm(&arguments, &target, &profile) {
        Ok(output) => {
            print!("{output}");
            std::process::ExitCode::SUCCESS
        }
        Err(message) => {
            eprint!("{message}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// The bounded, closed subset of Helm's surface this fixture answers.
///
/// It answers exactly the probes and operations C06 admits and nothing else. An argument outside
/// that set is an error here, so a production path that started passing one would be caught by the
/// fixture rather than tolerated by it. Compatible output is not proof of stock Helm's semantics,
/// and nothing here claims it is.
fn helm(arguments: &[String], target: &Path, profile: &Profile) -> Result<String, String> {
    match arguments.first().map(String::as_str) {
        Some("version") => Ok(format!(
            "{}\n",
            profile
                .version
                .clone()
                .unwrap_or_else(|| "v3.16.2".to_owned())
        )),
        Some(operation) if arguments.get(1).map(String::as_str) == Some("--help") => {
            Ok(help(operation, profile.omit.as_deref()))
        }
        Some("upgrade") => mutate(arguments, target, profile, false),
        Some("uninstall") => mutate(arguments, target, profile, true),
        Some(other) => Err(format!("unsupported operation {other}\n")),
        None => Err("no operation\n".to_owned()),
    }
}

fn help(operation: &str, omit: Option<&str>) -> String {
    use std::fmt::Write as _;
    let flags: &[&str] = match operation {
        "upgrade" => &[
            "--install",
            "--atomic",
            "--wait",
            "--timeout",
            "--description",
            "--no-hooks",
            "--skip-crds",
            "--values",
            "--namespace",
            "--kubeconfig",
            "--kube-context",
        ],
        "uninstall" => &[
            "--no-hooks",
            "--wait",
            "--timeout",
            "--namespace",
            "--kubeconfig",
            "--kube-context",
        ],
        "template" => &["--values", "--namespace", "--no-hooks", "--skip-crds"],
        "status" => &["--output", "--namespace"],
        "get" => &["--revision", "--namespace"],
        _ => &[],
    };
    let omitted = omit
        .and_then(|omit| omit.split_once(':'))
        .filter(|(named, _)| *named == operation)
        .map(|(_, flag)| flag);
    let mut text = format!("Usage: helm {operation} [flags]\n\nFlags:\n");
    for flag in flags {
        if omitted == Some(flag) {
            continue;
        }
        let _ = writeln!(text, "  {flag}");
    }
    text
}

fn value(arguments: &[String], flag: &str) -> Option<String> {
    arguments
        .iter()
        .position(|argument| argument == flag)
        .and_then(|position| arguments.get(position + 1))
        .cloned()
}

/// Applies or removes one release against the durable synthetic target.
///
/// The fault selects between the barriers a mutation can fail at, and each one is a different fact
/// about the target: `no-effect` fails before touching it, `effect-then-fail` changes it and then
/// fails, `lost-ack` changes it and never returns, `timeout` hangs. The target is a directory of
/// its own, so what it says survives this process either way.
fn mutate(
    arguments: &[String],
    target: &Path,
    profile: &Profile,
    remove: bool,
) -> Result<String, String> {
    let fault = profile.fault.clone().unwrap_or_default();
    let namespace = value(arguments, "--namespace").ok_or("no namespace\n")?;
    let release = arguments.get(1).cloned().ok_or("no release\n")?;
    if fault == "no-effect" {
        return Err("the injected control failed this call before any effect\n".to_owned());
    }
    if fault == "timeout" {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
    let mut state = read_release(target, &namespace, &release);
    if remove {
        state = ReleaseState::default();
    } else {
        state.description = value(arguments, "--description");
        state.manifest = profile
            .inventory
            .clone()
            .unwrap_or_default()
            .split(',')
            .filter(|entry| !entry.is_empty())
            .map(str::to_owned)
            .collect();
        state.objects = state
            .manifest
            .iter()
            .map(|address| (address.clone(), format!("live:{address}")))
            .collect();
    }
    write_release(target, &namespace, &release, &state)
        .map_err(|error| format!("the synthetic target could not be written: {error}\n"))?;
    if fault == "effect-then-fail" {
        return Err("the injected control failed this call after its effect\n".to_owned());
    }
    if fault == "lost-ack" {
        // The effect happened and the acknowledgement never arrives. Terminating rather than
        // returning is the point: a caller that saw an exit code would have evidence it must not
        // have.
        std::process::exit(101);
    }
    Ok(format!("Release \"{release}\" has been upgraded.\n"))
}
