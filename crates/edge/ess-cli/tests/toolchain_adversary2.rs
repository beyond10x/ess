//! Adversary pass 2 cases for ess#147 (`ess` manages its own toolchain).
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

const FAKE: &str = "9.9.9";

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
            .prefix("adversary2-")
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

impl Fixture {
    fn pin(&self, requires: &str) {
        fs::write(
            self.manifest(),
            format!(
                "format: ess-inputs/2\nrequires: {requires}\nspecification: [system.yaml]\n\
                 scenarios: []\n"
            ),
        )
        .unwrap();
    }
}

fn is_release(word: &str) -> bool {
    let parts: Vec<_> = word.split('.').collect();
    parts.len() == 3 && parts.iter().all(|part| part.parse::<u64>().is_ok())
}

/// The release workflow's smoke check reads `ess --version | awk '{print $NF}'`. Under a pin that
/// check must still read releases, not a manifest path: every stdout line must end in `X.Y.Z`.
#[test]
fn adversary2_version_under_a_pin_still_reads_as_releases_to_the_smoke_check() {
    let fixture = Fixture::new();
    fixture.pin(&format!("ess {FAKE}"));
    let output = fixture.ess(&fixture.project, &["--version"]);
    assert!(output.status.success(), "{}", err(&output));
    let stdout = out(&output);
    assert!(stdout.contains(&format!("FAKE-ESS {FAKE}")), "{stdout}");
    for line in stdout.lines() {
        let last = line.split_whitespace().last().unwrap_or("");
        assert!(
            is_release(last),
            "awk '{{print $NF}}' reads `{last}` from `{line}`, not a release"
        );
    }
}

/// `--pin` refuses a release before 0.34.0 because it cannot read `ess-inputs/2`. A pin written
/// by hand is the same pin: the dispatcher must refuse it too, rather than download that release
/// and hand it a manifest it cannot read.
#[test]
fn adversary2_a_hand_written_pin_before_0_34_0_is_refused_not_delegated() {
    let fixture = Fixture::new();
    fixture.publish("0.33.0");
    fixture.pin("ess 0.33.0");
    let output = fixture.ess(&fixture.project, &["specify", "validate"]);
    assert!(
        !out(&output).contains("FAKE-ESS 0.33.0"),
        "delegated to 0.33.0, which cannot read ess-inputs/2:\n{}\n{}",
        out(&output),
        err(&output)
    );
    assert!(
        !fixture.cache.join("0.33.0").exists(),
        "downloaded a release the pin can never run"
    );
}

/// The pin is found by walking up from the working directory, so one project must use one cache.
/// A relative `ESS_TOOLCHAIN_DIR` resolves against the working directory instead, and every
/// subdirectory installs its own copy (`XDG_CACHE_HOME` and `HOME` already ignore relative values).
#[test]
fn adversary2_a_relative_toolchain_dir_is_one_cache_for_the_project() {
    let fixture = Fixture::new();
    fixture.pin(&format!("ess {FAKE}"));
    let deeper = fixture.project.join("spec");
    fs::create_dir_all(&deeper).unwrap();
    let first = fixture
        .command(&fixture.project, &["specify", "validate"])
        .env("ESS_TOOLCHAIN_DIR", "relative-cache")
        .env("XDG_CACHE_HOME", fixture.root.path().join("xdg-cache"))
        .output()
        .unwrap();
    assert!(out(&first).contains("FAKE-ESS"), "{}", err(&first));
    let second = fixture
        .command(&deeper, &["specify", "validate"])
        .env("ESS_TOOLCHAIN_DIR", "relative-cache")
        .env("XDG_CACHE_HOME", fixture.root.path().join("xdg-cache"))
        .output()
        .unwrap();
    assert!(out(&second).contains("FAKE-ESS"), "{}", err(&second));
    assert!(
        !err(&second).contains("installing"),
        "installed a second time from a subdirectory of the same project: {}",
        err(&second)
    );
    let _ = fixture.root.path();
}

/// A cache entry whose `ess` is gone (removed by hand, or lost in a crash between write and
/// rename) must not block installing that release again.
#[test]
fn adversary2_a_cache_entry_without_its_ess_is_installed_again() {
    let fixture = Fixture::new();
    fixture.pin(&format!("ess {FAKE}"));
    let entry = fixture.cache.join(FAKE);
    fs::create_dir_all(&entry).unwrap();
    fs::write(entry.join("leftover"), b"").unwrap();
    let output = fixture.ess(&fixture.project, &["specify", "validate"]);
    assert!(
        out(&output).contains(&format!("FAKE-ESS {FAKE}")),
        "{}",
        err(&output)
    );
}

/// Twelve commands installing the same release at once all run it, and one entry is cached.
#[test]
fn adversary2_concurrent_installs_of_one_release_all_succeed() {
    let fixture = Fixture::new();
    fixture.pin(&format!("ess {FAKE}"));
    let children: Vec<_> = (0..12)
        .map(|_| {
            fixture
                .command(&fixture.project, &["specify", "validate"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{}", err(&output));
        assert!(out(&output).contains(&format!("FAKE-ESS {FAKE}")));
    }
    let entries: Vec<_> = fs::read_dir(&fixture.cache)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(entries, [FAKE.to_owned()], "{entries:?}");
}
