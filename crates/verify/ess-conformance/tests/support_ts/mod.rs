//! Running one suite through the generated TypeScript package against a recorded transcript
//! (`fixtures/transcript-target-ts.mjs`), and reading back one verdict per scenario.
//!
//! The transcript is the one `support_go::Recorder` writes while the Rust reference runner asks a
//! Rust target, so a verdict that differs is a difference in how the two runners read the suite.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::{AdmittedSuite, ConformanceSuite};

/// The compiled TypeScript runtime package for `suite`, with the transcript driver beside it.
pub struct Package {
    root: PathBuf,
    package: PathBuf,
}

impl Package {
    /// Emits and compiles the package for `suite` under the build's temporary directory.
    pub fn new(label: &str, suite: &ConformanceSuite) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("ts-transcript-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for artifact in ess_conformance::ts::emit(suite).unwrap_or_else(|error| panic!("{error}")) {
            let path = root.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        let package = root.join("essconform");
        let mut compile = Command::new("tsc");
        if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
            compile
                .arg("--typeRoots")
                .arg(Path::new(&modules).join("@types"));
        }
        let output = compile
            .args(["--project", "tsconfig.json", "--noCheck"])
            .current_dir(&package)
            .output()
            .expect("the TypeScript compiler runs");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(
            package.join("transcript.mjs"),
            include_str!("../fixtures/transcript-target-ts.mjs"),
        )
        .unwrap();
        Self { root, package }
    }

    /// Runs `admitted` against `transcript`, hiding every optional target method where `bare`.
    pub fn replay(
        &self,
        admitted: &AdmittedSuite,
        transcript: &serde_json::Value,
        bare: bool,
    ) -> Replayed {
        let suite_file = self.package.join("suite.json");
        std::fs::write(&suite_file, admitted.original_json()).unwrap();
        let transcript_file = self.package.join("transcript.json");
        std::fs::write(&transcript_file, transcript.to_string()).unwrap();
        let divergence = self.package.join("divergence.txt");
        let _ = std::fs::remove_file(&divergence);
        let report = self.package.join("report.json");
        let _ = std::fs::remove_file(&report);
        let output = Command::new("node")
            .arg(self.package.join("transcript.mjs"))
            .arg(&suite_file)
            .arg(&transcript_file)
            .arg(&divergence)
            .arg(if bare { "1" } else { "0" })
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .output()
            .expect("node runs");
        let log = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let mut outcomes = BTreeMap::new();
        if let Ok(text) = std::fs::read_to_string(&report) {
            let document: serde_json::Value = serde_json::from_str(&text).unwrap();
            for (status, ids) in document["outcomes"].as_object().expect("report/2 outcomes") {
                for id in ids.as_array().expect("ids") {
                    outcomes.insert(id.as_str().unwrap().to_owned(), status.clone());
                }
            }
        }
        Replayed {
            outcomes,
            divergences: std::fs::read_to_string(&divergence)
                .unwrap_or_default()
                .lines()
                .map(str::to_owned)
                .collect(),
            exit: output.status.code(),
            log,
        }
    }
}

impl Drop for Package {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// What one replay through the TypeScript runtime came to.
pub struct Replayed {
    /// Every scenario's precise status, read from the run's report/2.
    pub outcomes: BTreeMap<String, String>,
    /// Every request the runtime sent that the recorded run did not, one line each.
    pub divergences: Vec<String>,
    /// The driver's exit code; `Some(2)` where the runtime refused the suite before any scenario.
    pub exit: Option<i32>,
    /// What the driver printed.
    pub log: String,
}
