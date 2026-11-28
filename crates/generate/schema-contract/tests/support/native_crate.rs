//! A generated Rust crate whose own tests a test of this package runs, built once, before the
//! tests, by `task schema-contract-native-crates` (the organization rule `Tests stay fast`;
//! `no_test_builds.rs`). No test builds one.
//!
//! The test binary's ignored test `write_the_sources_of_every_native_crate` writes the sources
//! of every crate it runs under `ESS_SCHEMA_CONTRACT_NATIVE_SOURCES`, one directory per digest of
//! those sources. The task builds each crate's test binaries once per `serde_json` number mode,
//! without and with `serde_json/arbitrary_precision`, into
//! `ESS_SCHEMA_CONTRACT_NATIVE_CRATES/<digest>/<mode>/`. A test generates its sources again,
//! which is cheap, and runs the binaries built from exactly those bytes.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

/// The `serde_json` number modes every crate is built and run in, as the task names them.
const MODES: [&str; 2] = ["default", "arbitrary_precision"];

/// One generated crate: its files, relative to the crate root, and a label for messages.
pub struct NativeCrate {
    label: String,
    files: BTreeMap<String, String>,
}

impl NativeCrate {
    pub fn new(label: &str, files: BTreeMap<String, String>) -> Self {
        Self {
            label: label.to_owned(),
            files,
        }
    }

    /// The SHA-256 of every file, path and bytes, as lowercase hex: the name the built crate
    /// goes under.
    pub fn digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"schema-contract-native-crate/1\0");
        for (path, contents) in &self.files {
            hasher.update(path.as_bytes());
            hasher.update([0]);
            hasher.update((contents.len() as u64).to_le_bytes());
            hasher.update(contents.as_bytes());
        }
        hasher
            .finalize()
            .iter()
            .fold(String::new(), |mut out, byte| {
                let _ = write!(out, "{byte:02x}");
                out
            })
    }

    /// Writes the sources under `root/<digest>/`: the crate under `src/` and the file `label`
    /// the task reads. A crate written twice, by two binaries at once too, is written once.
    pub fn write_sources(&self, root: &Path) {
        let digest = self.digest();
        let directory = root.join(&digest);
        if directory.exists() {
            return;
        }
        let partial = root.join(format!(".{digest}.{}", std::process::id()));
        for (relative, contents) in &self.files {
            let destination = partial.join("src").join(relative);
            std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
            std::fs::write(&destination, contents).expect("write");
        }
        std::fs::write(partial.join("label"), &self.label).expect("write");
        if std::fs::rename(&partial, &directory).is_err() {
            assert!(directory.is_dir(), "{}: rename into place", self.label);
            std::fs::remove_dir_all(&partial).expect("remove a partial copy");
        }
    }

    /// The test binaries the task built from these sources in `mode`, or a panic saying what to
    /// run.
    fn built(&self, mode: &str) -> Vec<PathBuf> {
        let label = &self.label;
        let Some(directory) = std::env::var_os("ESS_SCHEMA_CONTRACT_NATIVE_CRATES") else {
            panic!(
                "ESS_SCHEMA_CONTRACT_NATIVE_CRATES is not set: run `task \
                 schema-contract-native-crates` and set it to the directory that task names, or \
                 run the tests through `task test`, which does both"
            )
        };
        let directory = PathBuf::from(directory);
        let digest = self.digest();
        if let Ok(log) = std::fs::read_to_string(directory.join(format!("{digest}.{mode}.failed")))
        {
            panic!("{label}: the generated crate does not build in mode {mode}:\n{log}");
        }
        let mut binaries: Vec<PathBuf> = std::fs::read_dir(directory.join(&digest).join(mode))
            .map(|entries| {
                entries
                    .map(|entry| entry.expect("a directory entry").path())
                    .collect()
            })
            .unwrap_or_default();
        binaries.sort();
        assert!(
            !binaries.is_empty(),
            "{label}: no {mode} build of crate {digest} in ESS_SCHEMA_CONTRACT_NATIVE_CRATES={}: \
             run `task schema-contract-native-crates` again, since its sources changed; a crate \
             that task does not build is missing from `native_crates()` in its test file",
            directory.display()
        );
        binaries
    }

    /// Runs every test binary of the crate with `arguments`, in both number modes, and returns
    /// what they printed; a failing one fails the calling test with its output.
    pub fn run(&self, arguments: &[&str]) -> String {
        let mut log = String::new();
        for mode in MODES {
            for binary in self.built(mode) {
                let output = Command::new(&binary)
                    .args(arguments)
                    .output()
                    .unwrap_or_else(|error| panic!("{} runs: {error}", binary.display()));
                let text = format!(
                    "{} {mode} {}: {}\n{}\n{}",
                    self.label,
                    binary.display(),
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(output.status.success(), "{text}");
                log.push_str(&text);
            }
        }
        log
    }
}

/// Writes the sources of every crate of `crates` under `ESS_SCHEMA_CONTRACT_NATIVE_SOURCES`.
pub fn write_sources(crates: Vec<NativeCrate>) {
    let root = std::env::var_os("ESS_SCHEMA_CONTRACT_NATIVE_SOURCES")
        .map(PathBuf::from)
        .expect(
            "ESS_SCHEMA_CONTRACT_NATIVE_SOURCES names the directory the sources are written to",
        );
    std::fs::create_dir_all(&root).expect("mkdir");
    for native in crates {
        native.write_sources(&root);
    }
}
