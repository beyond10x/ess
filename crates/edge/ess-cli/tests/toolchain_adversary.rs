//! Adversary cases for ess#147 (`ess` manages its own toolchain).
//!
//! Offline: `ESS_TOOLCHAIN_BASE_URL` points at a fixture directory, and the cache is private to
//! each case.

#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use sha2::{Digest, Sha256};

const THIS: &str = env!("CARGO_PKG_VERSION");
const FAKE: &str = "9.9.9";
const ABSENT: &str = "9.9.8";

fn target() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        other => panic!("no release target for {other:?}"),
    }
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn archive(version: &str) -> Vec<u8> {
    let package = format!("ess-{version}-{}", target());
    let script = format!("#!/bin/sh\necho \"FAKE-ESS {version}\"\n");
    let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    ));
    let mut header = tar::Header::new_gnu();
    header.set_size(script.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    builder
        .append_data(&mut header, format!("{package}/ess"), script.as_bytes())
        .unwrap();
    builder.into_inner().unwrap().finish().unwrap()
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut text, byte| {
            use std::fmt::Write as _;
            let _ = write!(text, "{byte:02x}");
            text
        })
}

struct Fixture {
    root: tempfile::TempDir,
    releases: PathBuf,
    cache: PathBuf,
    project: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let parent = repo().join("target/toolchain-tests");
        fs::create_dir_all(&parent).unwrap();
        let root = tempfile::Builder::new()
            .prefix("adversary-")
            .tempdir_in(parent)
            .unwrap();
        let releases = root.path().join("releases");
        let cache = root.path().join("cache");
        let project = root.path().join("project");
        fs::create_dir_all(&releases).unwrap();
        fs::create_dir_all(&project).unwrap();
        let fixture = Self {
            root,
            releases,
            cache,
            project,
        };
        fixture.publish(FAKE);
        fixture
    }

    fn publish(&self, version: &str) {
        let directory = self.releases.join(version);
        fs::create_dir_all(&directory).unwrap();
        let name = format!("ess-{version}-{}.tar.gz", target());
        let bytes = archive(version);
        fs::write(directory.join(&name), &bytes).unwrap();
        fs::write(
            directory.join("SHA256SUMS"),
            format!("{}  {name}\n", hex(&bytes)),
        )
        .unwrap();
    }

    fn manifest(&self) -> PathBuf {
        self.project.join("ess-inputs.yaml")
    }

    fn command(&self, directory: &Path, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .current_dir(directory)
            .args(args)
            .env_remove("ESS_TOOLCHAIN")
            .env_remove("ESS_TOOLCHAIN_DELEGATED")
            .env("ESS_TOOLCHAIN_DIR", &self.cache)
            .env("ESS_TOOLCHAIN_BASE_URL", &self.releases);
        command
    }

    fn ess(&self, directory: &Path, args: &[&str]) -> Output {
        self.command(directory, args).output().unwrap()
    }
}

fn out(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn err(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// An empty `ESS_TOOLCHAIN=` is how a shell unsets a variable for one command; the other two
/// toolchain variables already read empty as unset. It must not refuse every command.
#[test]
fn adversary_an_empty_override_is_no_override() {
    let fixture = Fixture::new();
    let output = fixture
        .command(&fixture.project, &["--version"])
        .env("ESS_TOOLCHAIN", "")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert_eq!(out(&output), format!("ess {THIS}\n"));
}

/// `ess --version` worked in a working directory that has since been removed before ess#147; the
/// pin lookup must not turn that into a refusal of every command.
#[test]
fn adversary_version_still_answers_in_a_removed_working_directory() {
    let fixture = Fixture::new();
    let gone = fixture.root.path().join("gone");
    fs::create_dir_all(&gone).unwrap();
    let output = Command::new("sh")
        .arg("-c")
        .arg("cd \"$1\" && rmdir \"$1\" && exec \"$2\" --version")
        .arg("sh")
        .arg(&gone)
        .arg(env!("CARGO_BIN_EXE_ess"))
        .env_remove("ESS_TOOLCHAIN")
        .env_remove("ESS_TOOLCHAIN_DELEGATED")
        .env("ESS_TOOLCHAIN_DIR", &fixture.cache)
        .env("ESS_TOOLCHAIN_BASE_URL", &fixture.releases)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert_eq!(out(&output), format!("ess {THIS}\n"));
}

/// ess#147: "`ess --version` names both the dispatcher and the delegated release". When the
/// pinned release cannot be installed, the dispatcher can still name itself.
#[test]
fn adversary_version_under_an_uninstallable_pin_still_names_the_dispatcher() {
    let fixture = Fixture::new();
    fs::write(
        fixture.manifest(),
        format!("format: ess-inputs/2\nrequires: ess {ABSENT}\nspecification: [s.yaml]\nscenarios: []\n"),
    )
    .unwrap();
    let output = fixture.ess(&fixture.project, &["--version"]);
    assert!(
        out(&output).contains(&format!("ess {THIS}")),
        "stdout: {:?}\nstderr: {}",
        out(&output),
        err(&output)
    );
}

/// The design promises "the rest of the document keeps its bytes". An `ess-inputs/2` manifest
/// needs no format change, so its commented `format:` line has no reason to lose its comment.
#[test]
fn adversary_pin_keeps_a_comment_on_an_unchanged_format_line() {
    let fixture = Fixture::new();
    let before = "format: ess-inputs/2  # schema owned by platform\nspecification: [s.yaml]\nscenarios: []\n";
    fs::write(fixture.manifest(), before).unwrap();
    let output = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", FAKE, "--pin"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let after = fs::read_to_string(fixture.manifest()).unwrap();
    assert!(after.contains("# schema owned by platform"), "{after}");
}

/// A CRLF manifest comes back with mixed line endings: the two lines `--pin` writes end in `\n`.
#[test]
fn adversary_pin_keeps_crlf_line_endings() {
    let fixture = Fixture::new();
    fs::write(
        fixture.manifest(),
        "format: ess-inputs/1\r\nspecification: [s.yaml]\r\nscenarios: []\r\n",
    )
    .unwrap();
    let output = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", FAKE, "--pin"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let after = fs::read_to_string(fixture.manifest()).unwrap();
    let bare = after.matches('\n').count() - after.matches("\r\n").count();
    assert_eq!(bare, 0, "{after:?}");
}

/// `requires` exists only in `ess-inputs/2`, which ess 0.34.0 introduced (CHANGELOG). Pinning
/// 0.33.0 writes a manifest the pinned release refuses as an unknown format, so every delegated
/// command in the project fails. `--pin` must refuse and leave the manifest alone.
#[test]
fn adversary_pin_refuses_a_release_that_cannot_read_the_pin() {
    let fixture = Fixture::new();
    fixture.publish("0.33.0");
    let before = "format: ess-inputs/1\nspecification: [s.yaml]\nscenarios: []\n";
    fs::write(fixture.manifest(), before).unwrap();
    let output = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", "0.33.0", "--pin"],
    );
    assert_ne!(output.status.code(), Some(0), "{}", out(&output));
    assert_eq!(fs::read_to_string(fixture.manifest()).unwrap(), before);
}

/// The nearest manifest wins (design page). A symlinked `ess-inputs.yaml`, which every command
/// refuses, is skipped instead, and `--pin` then rewrites a manifest in a directory above it.
#[test]
fn adversary_pin_does_not_rewrite_a_manifest_above_a_symlinked_one() {
    let fixture = Fixture::new();
    let outer = fixture.root.path().join("ess-inputs.yaml");
    let outer_text = "format: ess-inputs/2\nspecification: [s.yaml]\nscenarios: []\n";
    fs::write(&outer, outer_text).unwrap();
    let real = fixture.root.path().join("real.yaml");
    fs::write(
        &real,
        "format: ess-inputs/2\nspecification: [s.yaml]\nscenarios: []\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(&real, fixture.manifest()).unwrap();
    let _ = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", FAKE, "--pin"],
    );
    assert_eq!(fs::read_to_string(&outer).unwrap(), outer_text);
}

/// Every other case sets `ESS_TOOLCHAIN_DIR`, so the documented default cache location is
/// otherwise untested: `$XDG_CACHE_HOME/ess/toolchains`, and `~/.cache/…` for a relative one.
#[test]
fn adversary_the_default_cache_follows_xdg_cache_home() {
    let fixture = Fixture::new();
    let xdg = fixture.root.path().join("xdg");
    let output = fixture
        .command(&fixture.project, &["specify", "toolchain", "install", FAKE])
        .env_remove("ESS_TOOLCHAIN_DIR")
        .env("XDG_CACHE_HOME", &xdg)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert!(xdg.join("ess/toolchains").join(FAKE).join("ess").is_file());

    let home = fixture.root.path().join("home");
    let output = fixture
        .command(&fixture.project, &["specify", "toolchain", "install", FAKE])
        .env_remove("ESS_TOOLCHAIN_DIR")
        .env("XDG_CACHE_HOME", "relative/cache")
        .env("HOME", &home)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert!(home
        .join(".cache/ess/toolchains")
        .join(FAKE)
        .join("ess")
        .is_file());
    assert!(!fixture.project.join("relative").exists());
}
