//! `ess` manages its own toolchain (ess#147).
//!
//! An exact `requires: ess X.Y.Z` found by walking up from the working directory, or
//! `ESS_TOOLCHAIN=X.Y.Z`, makes any `ess` exec that release from its cache with the same arguments
//! and environment. `ess specify toolchain install` fills the cache from a release's archive and
//! `SHA256SUMS`, `list` shows it and `which` says what would run here and why.
//!
//! Every case is offline: `ESS_TOOLCHAIN_BASE_URL` points at a fixture directory holding a fake
//! `ess` script archive and its `SHA256SUMS`, and `ESS_TOOLCHAIN_DIR` at a private cache.

#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use sha2::{Digest, Sha256};

const THIS: &str = env!("CARGO_PKG_VERSION");
/// The release the fixture publishes. No real `ess` release carries this number.
const FAKE: &str = "9.9.9";
/// A release the fixture does not publish.
const ABSENT: &str = "9.9.8";

/// The release target this `ess` downloads for, as the release workflow names its archives.
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

/// A release directory, a cache and a project, all private to one case.
struct Fixture {
    _root: tempfile::TempDir,
    releases: PathBuf,
    cache: PathBuf,
    project: PathBuf,
}

/// The fake release's `ess`: prints a marker, the arguments one per line, and the recursion guard.
fn fake_script(version: &str) -> String {
    format!(
        "#!/bin/sh\necho \"FAKE-ESS {version} delegated=$ESS_TOOLCHAIN_DELEGATED\"\n\
         for argument in \"$@\"; do echo \"arg:$argument\"; done\n"
    )
}

/// `ess-<version>-<target>.tar.gz`, laid out as the release workflow packs it.
fn archive(version: &str) -> Vec<u8> {
    let package = format!("ess-{version}-{}", target());
    let script = fake_script(version);
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

impl Fixture {
    fn new() -> Self {
        let parent = repo().join("target/toolchain-tests");
        fs::create_dir_all(&parent).unwrap();
        let root = tempfile::Builder::new()
            .prefix("case-")
            .tempdir_in(parent)
            .unwrap();
        let releases = root.path().join("releases");
        let cache = root.path().join("cache");
        let project = root.path().join("project");
        fs::create_dir_all(&releases).unwrap();
        fs::create_dir_all(project.join("spec/deeper")).unwrap();
        let fixture = Self {
            _root: root,
            releases,
            cache,
            project,
        };
        fixture.publish(FAKE, None);
        fixture
    }

    /// Put `version` under the release base URL; `lie` replaces the digest `SHA256SUMS` states.
    fn publish(&self, version: &str, lie: Option<&str>) {
        let directory = self.releases.join(version);
        fs::create_dir_all(&directory).unwrap();
        let name = format!("ess-{version}-{}.tar.gz", target());
        let bytes = archive(version);
        let digest = lie.map_or_else(|| hex(&bytes), str::to_owned);
        fs::write(directory.join(&name), &bytes).unwrap();
        fs::write(directory.join("SHA256SUMS"), format!("{digest}  {name}\n")).unwrap();
    }

    fn pin(&self, requires: &str) {
        fs::write(
            self.project.join("ess-inputs.yaml"),
            format!(
                "format: ess-inputs/2\nrequires: {requires}\nspecification: [system.yaml]\n\
                 scenarios: []\n"
            ),
        )
        .unwrap();
    }

    fn command(&self, directory: &Path, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .current_dir(directory)
            .args(args)
            .env_remove("ESS_TOOLCHAIN")
            .env_remove("ESS_TOOLCHAIN_DELEGATED")
            .env_remove("ESS_TOOLCHAIN_QUIET")
            .env("ESS_TOOLCHAIN_DIR", &self.cache)
            .env("ESS_TOOLCHAIN_BASE_URL", &self.releases);
        command
    }

    fn ess(&self, directory: &Path, args: &[&str]) -> Output {
        self.command(directory, args).output().unwrap()
    }

    fn install(&self, version: &str) -> Output {
        self.ess(&self.project, &["specify", "toolchain", "install", version])
    }

    fn cached(&self, version: &str) -> PathBuf {
        self.cache.join(version).join("ess")
    }
}

fn out(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn err(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn install_verifies_the_archive_and_caches_its_ess() {
    let fixture = Fixture::new();
    let output = fixture.install(FAKE);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let cached = fixture.cached(FAKE);
    let mode = fs::metadata(&cached).unwrap().permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "the cached ess is executable");
    assert_eq!(fs::read_to_string(&cached).unwrap(), fake_script(FAKE));
    assert!(out(&output).contains(FAKE), "{}", out(&output));

    // A second install of a cached release is a no-op that succeeds.
    let again = fixture.install(FAKE);
    assert_eq!(again.status.code(), Some(0), "{}", err(&again));

    let list = fixture.ess(&fixture.project, &["specify", "toolchain", "list"]);
    assert_eq!(list.status.code(), Some(0), "{}", err(&list));
    assert!(out(&list).contains(FAKE), "{}", out(&list));
}

#[test]
fn install_accepts_a_file_url_base() {
    let fixture = Fixture::new();
    let output = fixture
        .command(&fixture.project, &["specify", "toolchain", "install", FAKE])
        .env(
            "ESS_TOOLCHAIN_BASE_URL",
            format!("file://{}", fixture.releases.display()),
        )
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert!(fixture.cached(FAKE).is_file());
}

#[test]
fn a_checksum_mismatch_is_refused_and_nothing_is_cached() {
    let fixture = Fixture::new();
    fixture.publish(FAKE, Some(&"0".repeat(64)));
    let output = fixture.install(FAKE);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("SHA256SUMS"), "{}", err(&output));
    assert!(!fixture.cache.join(FAKE).exists(), "nothing cached");
    let list = fixture.ess(&fixture.project, &["specify", "toolchain", "list"]);
    assert!(!out(&list).contains(FAKE), "{}", out(&list));
}

#[test]
fn a_pin_to_a_cached_release_delegates_with_the_same_arguments() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    fixture.pin(&format!("ess {FAKE}"));
    // Found by walking up from a directory below the manifest.
    let output = fixture.ess(
        &fixture.project.join("spec/deeper"),
        &["specify", "validate", "--path", "a b", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert_eq!(
        out(&output),
        format!(
            "FAKE-ESS {FAKE} delegated=1\narg:specify\narg:validate\narg:--path\narg:a b\n\
             arg:--format\narg:json\n"
        )
    );
}

#[test]
fn a_pin_to_a_release_not_yet_cached_is_installed_and_delegated_to() {
    let fixture = Fixture::new();
    fixture.pin(&format!("ess {FAKE}"));
    let output = fixture.ess(&fixture.project, &["compile"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert!(
        out(&output).starts_with(&format!("FAKE-ESS {FAKE}")),
        "{}",
        out(&output)
    );
    assert!(fixture.cached(FAKE).is_file());
}

#[test]
fn the_override_delegates_without_a_pin_and_beats_one() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    let output = fixture
        .command(&fixture.project, &["graph"])
        .env("ESS_TOOLCHAIN", FAKE)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert!(
        out(&output).starts_with(&format!("FAKE-ESS {FAKE}")),
        "{}",
        out(&output)
    );

    // The override names this release, so a pin to another one does not delegate.
    fixture.pin(&format!("ess {FAKE}"));
    let output = fixture
        .command(&fixture.project, &["--version"])
        .env("ESS_TOOLCHAIN", THIS)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert_eq!(out(&output), format!("ess {THIS}\n"));
}

#[test]
fn a_malformed_override_refuses() {
    let fixture = Fixture::new();
    for value in ["latest", "9.9", "v9.9.9"] {
        let output = fixture
            .command(&fixture.project, &["--version"])
            .env("ESS_TOOLCHAIN", value)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{value}: {}", err(&output));
        assert!(err(&output).contains("ESS_TOOLCHAIN"), "{}", err(&output));
    }
}

#[test]
fn the_delegated_marker_stops_recursion() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    fixture.pin(&format!("ess {FAKE}"));
    for guard in [
        vec![("ESS_TOOLCHAIN_DELEGATED", "1")],
        vec![("ESS_TOOLCHAIN_DELEGATED", "1"), ("ESS_TOOLCHAIN", FAKE)],
    ] {
        let output = fixture
            .command(&fixture.project, &["--version"])
            .envs(guard.clone())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{guard:?}: {}", err(&output));
        assert_eq!(out(&output), format!("ess {THIS}\n"), "{guard:?}");
    }
}

#[test]
fn a_line_pin_or_no_pin_runs_this_release() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    for pin in [None, Some("ess 9.9")] {
        if let Some(pin) = pin {
            fixture.pin(pin);
        }
        let output = fixture.ess(&fixture.project, &["--version"]);
        assert_eq!(output.status.code(), Some(0), "{pin:?}: {}", err(&output));
        assert_eq!(out(&output), format!("ess {THIS}\n"), "{pin:?}");
    }
}

#[test]
fn version_names_the_dispatcher_and_the_delegated_release() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    fixture.pin(&format!("ess {FAKE}"));
    let output = fixture.ess(&fixture.project, &["--version"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    // stdout stays release lines only, for `awk '{print $NF}'`; the reason goes to stderr.
    assert_eq!(
        out(&output),
        format!("ess {THIS}\nFAKE-ESS {FAKE} delegated=1\narg:--version\n")
    );
    let note = err(&output);
    assert!(note.contains("dispatcher"), "{note}");
    assert!(note.contains(&format!("ess {FAKE}")), "{note}");
    assert!(
        note.contains(
            &fixture
                .project
                .join("ess-inputs.yaml")
                .display()
                .to_string()
        ),
        "{note}"
    );
}

/// beyond10x/ess#261: a newer `ess` under an older pin used to delegate every other command in
/// silence, so a reader could not tell which release wrote the output.
#[test]
fn every_delegated_command_names_the_delegation_once_on_stderr() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    fixture.pin(&format!("ess {FAKE}"));
    for args in [&["specify", "validate"][..], &["--version"][..]] {
        let output = fixture.ess(&fixture.project, args);
        assert_eq!(output.status.code(), Some(0), "{args:?}: {}", err(&output));
        let note = err(&output);
        assert_eq!(note.matches("delegating to").count(), 1, "{args:?}: {note}");
        assert!(
            note.contains(&format!(
                "ess {THIS} is the dispatcher; delegating to ess {FAKE}"
            )),
            "{args:?}: {note}"
        );
    }
    // Stdout is the delegated release's alone.
    let output = fixture.ess(&fixture.project, &["specify", "validate"]);
    assert_eq!(
        out(&output),
        format!("FAKE-ESS {FAKE} delegated=1\narg:specify\narg:validate\n")
    );
}

#[test]
fn the_delegation_note_can_be_silenced() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    fixture.pin(&format!("ess {FAKE}"));
    let output = fixture
        .command(&fixture.project, &["specify", "validate"])
        .env("ESS_TOOLCHAIN_QUIET", "1")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert!(!err(&output).contains("delegating to"), "{}", err(&output));
}

#[test]
fn toolchain_commands_run_in_the_dispatcher_even_under_a_pin() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));
    fixture.pin(&format!("ess {FAKE}"));
    for args in [
        &["specify", "toolchain", "list"][..],
        &["--strict-requires", "specify", "toolchain", "list"][..],
        &["toolchain", "list"][..],
    ] {
        let output = fixture.ess(&fixture.project, args);
        assert_eq!(output.status.code(), Some(0), "{args:?}: {}", err(&output));
        assert!(
            !out(&output).contains("FAKE-ESS"),
            "{args:?}: {}",
            out(&output)
        );
        assert!(out(&output).contains(FAKE), "{args:?}: {}", out(&output));
    }
}

#[test]
fn which_explains_the_choice() {
    let fixture = Fixture::new();
    let which = &["specify", "toolchain", "which"][..];

    let output = fixture.ess(&fixture.project, which);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    assert!(text.starts_with(&format!("ess {THIS}\n")), "{text}");
    assert!(text.contains("self"), "{text}");

    fixture.pin(&format!("ess {FAKE}"));
    let output = fixture.ess(&fixture.project.join("spec"), which);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    assert!(text.starts_with(&format!("ess {FAKE}\n")), "{text}");
    assert!(text.contains("pin"), "{text}");
    assert!(
        text.contains(
            &fixture
                .project
                .join("ess-inputs.yaml")
                .display()
                .to_string()
        ),
        "{text}"
    );
    assert!(text.contains("not cached"), "{text}");

    let output = fixture
        .command(&fixture.project, which)
        .env("ESS_TOOLCHAIN", ABSENT)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    assert!(text.starts_with(&format!("ess {ABSENT}\n")), "{text}");
    assert!(text.contains("override"), "{text}");
    assert!(text.contains("ESS_TOOLCHAIN"), "{text}");
}

#[test]
fn a_missing_release_is_refused_naming_the_newest_cached_one() {
    let fixture = Fixture::new();
    assert_eq!(fixture.install(FAKE).status.code(), Some(0));

    let output = fixture.install(ABSENT);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains(ABSENT), "{}", err(&output));
    assert!(
        err(&output).contains(&format!("newest cached release is {FAKE}")),
        "{}",
        err(&output)
    );
    assert!(!fixture.cache.join(ABSENT).exists());

    // Delegation to a pin that cannot be installed refuses the same way and runs nothing.
    fixture.pin(&format!("ess {ABSENT}"));
    let output = fixture.ess(&fixture.project, &["specify", "validate"]);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(
        err(&output).contains(&format!("newest cached release is {FAKE}")),
        "{}",
        err(&output)
    );
    assert!(output.stdout.is_empty(), "{}", out(&output));
}

#[test]
fn nothing_cached_and_no_source_says_so() {
    let fixture = Fixture::new();
    let output = fixture
        .command(&fixture.project, &["specify", "toolchain", "install", FAKE])
        .env("ESS_TOOLCHAIN_BASE_URL", fixture.releases.join("gone"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(
        err(&output).contains("no release is cached"),
        "{}",
        err(&output)
    );
}

#[test]
fn install_pin_writes_the_nearest_manifest() {
    let fixture = Fixture::new();
    let manifest = fixture.project.join("ess-inputs.yaml");
    fs::write(
        &manifest,
        "format: ess-inputs/1\nspecification: [system.yaml]\nscenarios: []\n",
    )
    .unwrap();
    let output = fixture.ess(
        &fixture.project.join("spec"),
        &["specify", "toolchain", "install", FAKE, "--pin"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert_eq!(
        fs::read_to_string(&manifest).unwrap(),
        format!(
            "format: ess-inputs/2\nrequires: ess {FAKE}\nspecification: [system.yaml]\n\
             scenarios: []\n"
        )
    );

    // Re-pinning replaces the requirement rather than adding a second one.
    fixture.publish("9.9.10", None);
    let output = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", "--pin", "9.9.10"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = fs::read_to_string(&manifest).unwrap();
    assert_eq!(text.matches("requires:").count(), 1, "{text}");
    assert!(text.contains("requires: ess 9.9.10\n"), "{text}");
}

#[test]
fn install_pin_without_a_manifest_refuses_after_caching() {
    let fixture = Fixture::new();
    let output = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", FAKE, "--pin"],
    );
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("ess-inputs.yaml"), "{}", err(&output));
}

#[test]
fn install_help_says_a_git_revision_is_out_of_scope() {
    let fixture = Fixture::new();
    let output = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", "--help"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert!(out(&output).contains("git revision"), "{}", out(&output));
}

#[test]
fn a_plain_http_base_url_is_refused() {
    let fixture = Fixture::new();
    let output = fixture
        .command(&fixture.project, &["specify", "toolchain", "install", FAKE])
        .env("ESS_TOOLCHAIN_BASE_URL", "http://127.0.0.1:9/releases")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("https"), "{}", err(&output));
    assert!(!fixture.cache.join(FAKE).exists());
}

#[test]
fn version_names_the_dispatcher_first_even_when_the_pin_cannot_be_installed() {
    let fixture = Fixture::new();
    fixture.pin(&format!("ess {ABSENT}"));
    let output = fixture.ess(&fixture.project, &["--version"]);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert_eq!(out(&output), format!("ess {THIS}\n"));
    assert!(err(&output).contains(ABSENT), "{}", err(&output));

    // Delegating, the dispatcher's line still comes before the release's own.
    fixture.pin(&format!("ess {FAKE}"));
    let output = fixture.ess(&fixture.project, &["--version"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    assert!(text.starts_with(&format!("ess {THIS}\n")), "{text}");
    assert!(
        text.ends_with(&format!("FAKE-ESS {FAKE} delegated=1\narg:--version\n")),
        "{text}"
    );
}

#[test]
fn pin_keeps_an_unchanged_requires_line_byte_for_byte() {
    let fixture = Fixture::new();
    let manifest = fixture.project.join("ess-inputs.yaml");
    let before = format!(
        "format: ess-inputs/2 # kept\nrequires: ess {FAKE}   # chosen by release\n\
         specification: [system.yaml]\nscenarios: []\n"
    );
    fs::write(&manifest, &before).unwrap();
    let output = fixture.ess(
        &fixture.project,
        &["specify", "toolchain", "install", FAKE, "--pin"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert_eq!(fs::read_to_string(&manifest).unwrap(), before);
}

#[test]
fn pin_refuses_to_write_through_a_symlinked_nearest_manifest() {
    let fixture = Fixture::new();
    let real = fixture.project.join("real.yaml");
    let text = "format: ess-inputs/2\nspecification: [system.yaml]\nscenarios: []\n";
    fs::write(&real, text).unwrap();
    std::os::unix::fs::symlink(&real, fixture.project.join("spec/ess-inputs.yaml")).unwrap();
    let output = fixture.ess(
        &fixture.project.join("spec"),
        &["specify", "toolchain", "install", FAKE, "--pin"],
    );
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("symlink"), "{}", err(&output));
    assert_eq!(fs::read_to_string(&real).unwrap(), text);

    // For choosing, the symlinked manifest is still the nearest one: it has no pin, so this runs.
    let which = fixture.ess(
        &fixture.project.join("spec"),
        &["specify", "toolchain", "which"],
    );
    assert!(
        out(&which).contains("spec/ess-inputs.yaml"),
        "{}",
        out(&which)
    );
}

#[test]
fn a_pin_or_override_before_0_34_0_is_refused_naming_0_34_0() {
    let fixture = Fixture::new();
    fixture.publish("0.33.0", None);
    fixture.pin("ess 0.33.0");
    let output = fixture.ess(&fixture.project, &["specify", "validate"]);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("0.34.0"), "{}", err(&output));
    assert!(output.stdout.is_empty(), "{}", out(&output));

    fs::remove_file(fixture.project.join("ess-inputs.yaml")).unwrap();
    let output = fixture
        .command(&fixture.project, &["graph"])
        .env("ESS_TOOLCHAIN", "0.33.0")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("0.34.0"), "{}", err(&output));
    assert!(!fixture.cache.join("0.33.0").exists());
}

#[test]
fn which_prints_absolute_paths_and_ignores_a_relative_cache_dir() {
    let fixture = Fixture::new();
    fixture.pin(&format!("ess {FAKE}"));
    let xdg = fixture.project.join("xdg");
    let output = fixture
        .command(
            &fixture.project.join("spec"),
            &["specify", "toolchain", "which"],
        )
        .env("ESS_TOOLCHAIN_DIR", "relative-cache")
        .env("XDG_CACHE_HOME", &xdg)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    let binary = text
        .lines()
        .find_map(|line| line.strip_prefix("binary: "))
        .unwrap_or_else(|| panic!("{text}"));
    let path = binary.split(" (").next().unwrap();
    assert!(Path::new(path).is_absolute(), "{text}");
    assert!(path.starts_with(&xdg.display().to_string()), "{text}");
}

#[test]
fn a_stale_cache_entry_is_not_cached_and_install_replaces_it() {
    for stale in ["empty", "not executable", "no ess"] {
        let fixture = Fixture::new();
        let entry = fixture.cache.join(FAKE);
        fs::create_dir_all(&entry).unwrap();
        match stale {
            "empty" => {
                fs::write(entry.join("ess"), b"").unwrap();
                fs::set_permissions(entry.join("ess"), fs::Permissions::from_mode(0o755)).unwrap();
            }
            "not executable" => fs::write(entry.join("ess"), fake_script(FAKE)).unwrap(),
            _ => fs::write(entry.join("leftover"), b"").unwrap(),
        }
        let list = fixture.ess(&fixture.project, &["specify", "toolchain", "list"]);
        assert!(!out(&list).contains(FAKE), "{stale}: {}", out(&list));

        let output = fixture.install(FAKE);
        assert_eq!(output.status.code(), Some(0), "{stale}: {}", err(&output));
        assert!(
            !out(&output).contains("already cached"),
            "{stale}: {}",
            out(&output)
        );
        assert_eq!(
            fs::read_to_string(entry.join("ess")).unwrap(),
            fake_script(FAKE)
        );
        assert!(!entry.join("leftover").exists(), "{stale}");
        let mode = fs::metadata(entry.join("ess"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o111, 0o111, "{stale}");
    }
}

/// `THIS` with its patch moved by one: a release that is not this one, either side of it.
fn other_release(newer: bool) -> String {
    let mut parts = THIS.split(['.', '-', '+']).map(str::parse::<u64>);
    let (major, minor, patch) = (
        parts.next().unwrap().unwrap(),
        parts.next().unwrap().unwrap(),
        parts.next().unwrap().unwrap(),
    );
    match (newer, patch) {
        (true, _) => format!("{major}.{minor}.{}", patch + 1),
        (false, 1..) => format!("{major}.{minor}.{}", patch - 1),
        (false, 0) => format!("{major}.{}.0", minor - 1),
    }
}

impl Fixture {
    /// A manifest in `directory` of the project, pinning `requires`.
    fn pin_in(&self, directory: &str, requires: &str) {
        let directory = self.project.join(directory);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("ess-inputs.yaml"),
            format!(
                "format: ess-inputs/2\nrequires: {requires}\nspecification: [system.yaml]\n\
                 scenarios: []\n"
            ),
        )
        .unwrap();
    }
}

const VALIDATE_SPEC: &[&str] = &["specify", "validate", "--path", "spec"];

fn assert_names_the_way_to_the_pin(text: &str, release: &str) {
    assert!(text.contains("spec/ess-inputs.yaml"), "{text}");
    assert!(text.contains(&format!("ESS_TOOLCHAIN={release}")), "{text}");
    assert!(text.contains("cd spec"), "{text}");
    assert!(!text.contains("b10x upgrade"), "{text}");
}

#[test]
fn a_pin_below_newer_than_this_ess_is_refused_by_name() {
    let fixture = Fixture::new();
    let release = other_release(true);
    fixture.pin_in("spec", &format!("ess {release}"));
    let output = fixture.ess(&fixture.project, VALIDATE_SPEC);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert_names_the_way_to_the_pin(&err(&output), &release);
}

#[test]
fn a_pin_below_older_than_this_ess_is_refused_by_name() {
    let fixture = Fixture::new();
    let release = other_release(false);
    fixture.pin_in("spec", &format!("ess {release}"));
    let output = fixture.ess(&fixture.project, VALIDATE_SPEC);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert_names_the_way_to_the_pin(&err(&output), &release);
}

#[test]
fn a_minor_line_pin_or_an_override_keeps_the_existing_checks_below() {
    let fixture = Fixture::new();
    fixture.pin_in("spec", "ess 99.0");
    let output = fixture.ess(&fixture.project, VALIDATE_SPEC);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("b10x upgrade"), "{}", err(&output));

    let release = other_release(true);
    fixture.pin_in("spec", &format!("ess {release}"));
    let output = fixture
        .command(&fixture.project, VALIDATE_SPEC)
        .env("ESS_TOOLCHAIN", THIS)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert!(err(&output).contains("b10x upgrade"), "{}", err(&output));
}

#[test]
fn toolchain_which_names_a_single_pin_below() {
    let fixture = Fixture::new();
    let release = other_release(false);
    fixture.pin_in("spec", &format!("ess {release}"));
    let output = fixture.ess(&fixture.project, &["toolchain", "which"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    assert!(text.starts_with(&format!("ess {THIS}\n")), "{text}");
    let reason = text
        .lines()
        .find(|line| line.starts_with("reason: "))
        .unwrap_or_else(|| panic!("{text}"));
    assert!(reason.contains("spec/ess-inputs.yaml"), "{reason}");
    assert!(
        reason.contains(&format!("requires: ess {release}")),
        "{reason}"
    );
    assert!(
        reason.contains(&format!("ESS_TOOLCHAIN={release}")),
        "{reason}"
    );
}

#[test]
fn toolchain_which_refuses_two_disagreeing_pins_below() {
    let fixture = Fixture::new();
    let (older, newer) = (other_release(false), other_release(true));
    fixture.pin_in("a", &format!("ess {older}"));
    fixture.pin_in("b", &format!("ess {newer}"));
    let output = fixture.ess(&fixture.project, &["toolchain", "which"]);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    let text = err(&output);
    for needle in ["a/ess-inputs.yaml", "b/ess-inputs.yaml", &older, &newer] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

/// The `reason: ` line of a `toolchain which` run from the project.
fn which_reason(fixture: &Fixture) -> String {
    let output = fixture.ess(&fixture.project, &["toolchain", "which"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    text.lines()
        .find(|line| line.starts_with("reason: "))
        .unwrap_or_else(|| panic!("{text}"))
        .to_owned()
}

#[test]
fn adversary_an_exact_pin_of_this_release_below_is_not_refused() {
    let fixture = Fixture::new();
    fixture.pin_in("spec", &format!("ess {THIS}"));
    let output = fixture.ess(&fixture.project, VALIDATE_SPEC);
    let text = err(&output);
    assert!(!text.contains("did not switch"), "{text}");
    assert!(!text.contains("ESS_TOOLCHAIN="), "{text}");
    let reason = which_reason(&fixture);
    assert!(reason.contains("spec/ess-inputs.yaml"), "{reason}");
    assert!(
        reason.contains(&format!("ESS_TOOLCHAIN={THIS}")),
        "{reason}"
    );
}

#[test]
fn adversary_two_pins_below_of_one_release_are_named_not_refused() {
    let fixture = Fixture::new();
    let release = other_release(true);
    fixture.pin_in("a", &format!("ess {release}"));
    fixture.pin_in("b", &format!("ess {release}"));
    let reason = which_reason(&fixture);
    for needle in ["a/ess-inputs.yaml", "b/ess-inputs.yaml", "cd a"] {
        assert!(reason.contains(needle), "{needle}: {reason}");
    }
}

#[test]
fn adversary_which_looks_six_directories_down_and_not_seven() {
    let release = other_release(true);
    let six = Fixture::new();
    six.pin_in("a/b/c/d/e/f", &format!("ess {release}"));
    let reason = which_reason(&six);
    assert!(reason.contains("a/b/c/d/e/f/ess-inputs.yaml"), "{reason}");

    let seven = Fixture::new();
    seven.pin_in("a/b/c/d/e/f/g", &format!("ess {release}"));
    let reason = which_reason(&seven);
    assert!(reason.contains("no ess-inputs.yaml in"), "{reason}");
}

#[test]
fn adversary_which_skips_hidden_target_node_modules_and_symlinked_directories() {
    let fixture = Fixture::new();
    let release = other_release(true);
    for directory in [".hidden", "target", "node_modules", "spec/target"] {
        fixture.pin_in(directory, &format!("ess {release}"));
    }
    let outside = fixture.project.parent().unwrap().join("outside/sub");
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        outside.join("ess-inputs.yaml"),
        format!(
            "format: ess-inputs/2\nrequires: ess {release}\nspecification: [system.yaml]\n\
             scenarios: []\n"
        ),
    )
    .unwrap();
    std::os::unix::fs::symlink(&outside, fixture.project.join("link")).unwrap();
    std::os::unix::fs::symlink(".", fixture.project.join("loop")).unwrap();
    let reason = which_reason(&fixture);
    assert!(reason.contains("no ess-inputs.yaml in"), "{reason}");
}

#[test]
fn adversary_which_output_stays_three_lines_each_with_its_field() {
    let fixture = Fixture::new();
    fixture.pin_in("spec", &format!("ess {}", other_release(false)));
    let output = fixture.ess(&fixture.project, &["toolchain", "which"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = out(&output);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert_eq!(lines[0], format!("ess {THIS}"), "{text}");
    assert_eq!(lines[0].split_whitespace().last(), Some(THIS), "{text}");
    assert!(lines[1].starts_with("reason: self: "), "{text}");
    assert!(
        lines[2].starts_with("binary: ") && lines[2].ends_with("(this ess)"),
        "{text}"
    );
}

#[test]
fn adversary_a_delegated_run_keeps_the_existing_checks_below() {
    let fixture = Fixture::new();
    fixture.pin_in("spec", &format!("ess {}", other_release(true)));
    let output = fixture
        .command(&fixture.project, VALIDATE_SPEC)
        .env("ESS_TOOLCHAIN_DELEGATED", "1")
        .output()
        .unwrap();
    let text = err(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("b10x upgrade"), "{text}");
    assert!(!text.contains("did not switch"), "{text}");
}

#[test]
fn adversary_a_manifest_outside_the_working_directory_keeps_the_existing_checks() {
    let fixture = Fixture::new();
    fixture.pin_in("spec", &format!("ess {}", other_release(true)));
    let other = fixture.project.join("other");
    fs::create_dir_all(&other).unwrap();
    let output = fixture.ess(&other, &["specify", "validate", "--path", "../spec"]);
    let text = err(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("b10x upgrade"), "{text}");
    assert!(!text.contains("did not switch"), "{text}");
}

#[test]
fn adversary_a_roundabout_or_absolute_path_into_the_pin_still_names_cd_spec() {
    let fixture = Fixture::new();
    let release = other_release(false);
    fixture.pin_in("spec", &format!("ess {release}"));
    let absolute = fixture.project.join("spec");
    for path in ["./spec/deeper/..", absolute.to_str().unwrap()] {
        let output = fixture.ess(&fixture.project, &["specify", "validate", "--path", path]);
        let text = err(&output);
        assert_eq!(output.status.code(), Some(1), "{path}: {text}");
        assert!(text.contains("cd spec"), "{path}: {text}");
        assert!(
            text.contains(&format!("ESS_TOOLCHAIN={release}")),
            "{path}: {text}"
        );
    }
}

#[test]
fn adversary_an_unpinned_manifest_above_does_not_hide_a_pin_below() {
    let fixture = Fixture::new();
    fs::write(
        fixture.project.join("ess-inputs.yaml"),
        "format: ess-inputs/2\nspecification: [system.yaml]\nscenarios: []\n",
    )
    .unwrap();
    let release = other_release(false);
    fixture.pin_in("spec", &format!("ess {release}"));
    let output = fixture.ess(&fixture.project, VALIDATE_SPEC);
    let text = err(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains(&format!("ESS_TOOLCHAIN={release}")), "{text}");
    assert!(text.contains("cd spec"), "{text}");
}
