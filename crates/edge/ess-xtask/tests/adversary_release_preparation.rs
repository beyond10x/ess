//! Adversarial release-preparation and release-record workflow checks.

use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ess-publisher-adversary-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("scratch directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove scratch directory");
    }
}

fn git(cwd: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args(args)
        .output()
        .expect("git must run")
}

fn git_ok(cwd: &Path, args: &[&str]) {
    let output = git(cwd, args);
    assert!(
        output.status.success(),
        "git {args:?} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn checkout_step(workflow: &Value) -> &Value {
    workflow["jobs"]["record"]["steps"]
        .as_sequence()
        .expect("record steps")
        .iter()
        .find(|step| {
            step["uses"]
                .as_str()
                .is_some_and(|uses| uses.starts_with("actions/checkout@"))
        })
        .expect("release-record checkout step")
}

#[test]
fn release_record_fetches_historical_tag_objects_before_checking_off_main() {
    let workflow: Value = serde_yaml::from_str(include_str!(
        "../../../../.github/workflows/release-record.yml"
    ))
    .expect("release-record workflow parses");
    let checkout = checkout_step(&workflow);

    // Reproduce the object visibility that actions/checkout's shallow, tagless defaults provide.
    // The remote advertises the historical tag, while the local checker cannot peel it to a
    // commit and therefore omits it from its off-main result.
    let scratch = Scratch::new();
    let origin = scratch.path().join("origin.git");
    let source = scratch.path().join("source");
    let runner = scratch.path().join("runner");
    fs::create_dir(&source).expect("source directory");
    fs::create_dir(&runner).expect("runner directory");

    git_ok(
        scratch.path(),
        &["init", "--bare", origin.to_str().unwrap()],
    );
    git_ok(&source, &["init", "--initial-branch=main"]);
    fs::write(source.join("main"), "main\n").expect("main fixture");
    git_ok(&source, &["add", "main"]);
    git_ok(
        &source,
        &[
            "-c",
            "user.name=Adversary",
            "-c",
            "user.email=adversary@example.invalid",
            "commit",
            "-m",
            "main",
        ],
    );
    git_ok(
        &source,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    git_ok(&source, &["push", "origin", "main"]);
    git_ok(&source, &["checkout", "--orphan", "off-main"]);
    git_ok(&source, &["rm", "-rf", "."]);
    fs::write(source.join("release"), "release\n").expect("release fixture");
    git_ok(&source, &["add", "release"]);
    git_ok(
        &source,
        &[
            "-c",
            "user.name=Adversary",
            "-c",
            "user.email=adversary@example.invalid",
            "commit",
            "-m",
            "off-main release",
        ],
    );
    git_ok(
        &source,
        &[
            "-c",
            "user.name=Adversary",
            "-c",
            "user.email=adversary@example.invalid",
            "tag",
            "-a",
            "9.9.9",
            "-m",
            "off-main tag",
        ],
    );
    git_ok(&source, &["push", "origin", "refs/tags/9.9.9"]);

    git_ok(&runner, &["init"]);
    let origin_url = format!("file://{}", origin.display());
    git_ok(&runner, &["remote", "add", "origin", &origin_url]);
    git_ok(
        &runner,
        &["fetch", "--depth=1", "--no-tags", "origin", "main"],
    );
    git_ok(&runner, &["checkout", "--detach", "FETCH_HEAD"]);
    let advertised = git(&runner, &["ls-remote", "--tags", "origin"]);
    assert!(advertised.status.success(), "remote tags must be queryable");
    assert!(
        String::from_utf8_lossy(&advertised.stdout).contains("refs/tags/9.9.9"),
        "the fixture tag must be advertised by the remote"
    );
    assert!(
        !git(&runner, &["rev-parse", "--verify", "9.9.9^{commit}"])
            .status
            .success(),
        "the shallow, tagless checkout must lack the historical tag object"
    );

    assert_eq!(
        checkout["with"]["fetch-depth"].as_u64(),
        Some(0),
        "release status skips any remote tag whose object is absent locally, so the record job must fetch full history"
    );
    assert_eq!(
        checkout["with"]["fetch-tags"].as_bool(),
        Some(true),
        "release status must receive historical tag objects before checking whether they reached main"
    );
}
