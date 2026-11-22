//! One rustc, pinned in `rust-toolchain.toml` and read by every workflow that builds this
//! repository. Three exceptions are named: the `msrv` job, which reads `rust-version`, the
//! `planning.yml` build of the pinned `aep` checkout and the generated `b10x-docs-bundle.yml`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PIN_OUTPUT: &str = "${{ steps.pin.outputs.toolchain }}";
const MSRV_OUTPUT: &str = "${{ steps.msrv.outputs.toolchain }}";
const DTOLNAY: &str = "dtolnay/rust-toolchain@";

/// Workflow file and job id; `*` means every job.
const EXCEPTIONS: &[(&str, &str)] = &[
    ("b10x-docs-bundle.yml", "*"),
    ("ci.yml", "msrv"),
    ("planning.yml", "*"),
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.contains("[workspace]"))
        })
        .expect("a workspace manifest stands above this crate")
        .to_path_buf()
}

fn workflow_names() -> Vec<String> {
    let directory = workspace_root().join(".github/workflows");
    let mut names: Vec<String> = fs::read_dir(&directory)
        .expect("the workflows directory is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| {
            Path::new(name)
                .extension()
                .is_some_and(|extension| extension == "yml")
        })
        .collect();
    names.sort();
    names
}

fn workflow(name: &str) -> ess_yaml::Value {
    let path = workspace_root().join(".github/workflows").join(name);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()));
    ess_yaml::from_str(&text).unwrap_or_else(|error| panic!("{name} parses: {error}"))
}

fn toolchain_file() -> String {
    let path = workspace_root().join("rust-toolchain.toml");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()))
}

fn pinned_channel() -> String {
    let text = toolchain_file();
    let channels: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("channel = \""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .collect();
    assert_eq!(
        channels.len(),
        1,
        "rust-toolchain.toml has exactly one channel line: {channels:?}"
    );
    channels[0].to_string()
}

fn text_of<'a>(value: &'a ess_yaml::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(|inner| inner.as_str())
}

/// Every `(file, job id, steps)` of every workflow.
fn jobs() -> Vec<(String, String, Vec<ess_yaml::Value>)> {
    let mut found = Vec::new();
    for name in workflow_names() {
        let document = workflow(&name);
        let Some(jobs) = document.get("jobs").and_then(|jobs| jobs.as_mapping()) else {
            continue;
        };
        for (id, job) in jobs {
            let id = id.as_str().expect("a job id is text").to_string();
            let steps = job
                .get("steps")
                .and_then(|steps| steps.as_sequence())
                .cloned()
                .unwrap_or_default();
            found.push((name.clone(), id, steps));
        }
    }
    found
}

fn is_exception(file: &str, job: &str) -> bool {
    EXCEPTIONS
        .iter()
        .any(|(f, j)| *f == file && (*j == "*" || *j == job))
}

fn is_install(step: &ess_yaml::Value) -> bool {
    text_of(step, "uses").is_some_and(|uses| uses.starts_with(DTOLNAY))
}

fn the_pin_run() -> String {
    for (file, job, steps) in jobs() {
        if file == "ci.yml" && job == "checks" {
            for step in steps {
                if text_of(&step, "id") == Some("pin") {
                    return text_of(&step, "run")
                        .expect("the pin step runs a script")
                        .to_string();
                }
            }
        }
    }
    panic!("ci.yml job checks has no step with id pin");
}

#[test]
fn the_toolchain_file_pins_one_exact_rustc() {
    let text = toolchain_file();
    assert!(
        text.lines().any(|line| line == "[toolchain]"),
        "a [toolchain] table"
    );
    let channel = pinned_channel();
    let parts: Vec<&str> = channel.split('.').collect();
    assert!(
        parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())),
        "the channel is an exact X.Y.Z: {channel}"
    );
    let line_with = |key: &str| {
        text.lines()
            .find(|line| line.starts_with(key))
            .unwrap_or_else(|| panic!("a {key} line"))
            .to_string()
    };
    let components = line_with("components");
    assert!(components.contains("\"rustfmt\"") && components.contains("\"clippy\""));
    assert!(line_with("targets").contains("\"wasm32-unknown-unknown\""));
}

#[test]
fn every_workflow_reads_the_pinned_rustc() {
    let mut excepted = BTreeSet::new();
    for (file, job, steps) in jobs() {
        for (index, step) in steps.iter().enumerate() {
            if !is_install(step) {
                continue;
            }
            let toolchain = step.get("with").and_then(|with| text_of(with, "toolchain"));
            if is_exception(&file, &job) {
                excepted.insert((file.clone(), job.clone()));
                if file == "ci.yml" && job == "msrv" {
                    assert_eq!(
                        toolchain,
                        Some(MSRV_OUTPUT),
                        "ci.yml msrv reads rust-version"
                    );
                }
                continue;
            }
            assert_eq!(
                toolchain,
                Some(PIN_OUTPUT),
                "{file} job {job}: the install step reads the pin"
            );
            let pin = steps[..index]
                .iter()
                .find(|earlier| text_of(earlier, "id") == Some("pin"))
                .unwrap_or_else(|| panic!("{file} job {job}: no earlier step with id pin"));
            assert!(
                text_of(pin, "run").is_some_and(|run| run.contains("rust-toolchain.toml")),
                "{file} job {job}: the pin step reads rust-toolchain.toml"
            );
            assert_eq!(
                text_of(pin, "if"),
                text_of(step, "if"),
                "{file} job {job}: the pin step and the install step share one condition"
            );
        }
    }
    let expected: BTreeSet<(String, String)> = [
        ("b10x-docs-bundle.yml", "bundle"),
        ("ci.yml", "msrv"),
        ("planning.yml", "validate"),
    ]
    .iter()
    .map(|(file, job)| (file.to_string(), job.to_string()))
    .collect();
    assert_eq!(
        excepted, expected,
        "the exceptions are exactly the named three"
    );
}

#[test]
fn every_pin_step_is_the_same_script() {
    let reference = the_pin_run();
    let mut seen = 0;
    for (file, job, steps) in jobs() {
        for step in steps {
            if text_of(&step, "id") != Some("pin") {
                continue;
            }
            seen += 1;
            assert_eq!(
                text_of(&step, "run"),
                Some(reference.as_str()),
                "{file} job {job}: the pin script differs from ci.yml checks"
            );
            assert_eq!(
                text_of(&step, "shell"),
                Some("bash"),
                "{file} job {job}: the pin step runs in bash"
            );
        }
    }
    assert!(seen > 1, "more than one pin step exists");
}

fn run_pin(script: &str, directory: &Path) -> (bool, String) {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("toolchain-pin-out-{}", std::process::id()));
    fs::create_dir_all(&scratch).expect("the output directory is created");
    let output_file = scratch.join("github-output");
    let _ = fs::remove_file(&output_file);
    let status = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(directory)
        .env("GITHUB_OUTPUT", &output_file)
        .stderr(std::process::Stdio::null())
        .status()
        .expect("bash runs");
    let written = fs::read_to_string(&output_file).unwrap_or_default();
    fs::remove_dir_all(&scratch).expect("the output directory is removed");
    (status.success(), written)
}

#[test]
fn the_pin_step_reads_the_file_and_refuses_anything_else() {
    let script = the_pin_run();
    let cases: &[(&str, Option<&str>)] = &[
        (
            "[toolchain]\nchannel = \"1.98.1\"\n",
            Some("toolchain=1.98.1\n"),
        ),
        ("[toolchain]\nchannel = \"stable\"\n", None),
        ("[toolchain]\nchannel = \"1.98\"\n", None),
        ("[toolchain]\n", None),
        (
            "[toolchain]\nchannel = \"1.98.1\"\nchannel = \"1.99.0\"\n",
            None,
        ),
    ];
    for (index, (content, expected)) in cases.iter().enumerate() {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("toolchain-pin-{}-{index}", std::process::id()));
        fs::create_dir_all(&directory).expect("the case directory is created");
        fs::write(directory.join("rust-toolchain.toml"), content).expect("the file is written");
        let (success, written) = run_pin(&script, &directory);
        if let Some(line) = expected {
            assert!(success, "case {index} succeeds");
            assert_eq!(written, *line, "case {index} writes the channel");
        } else {
            assert!(!success, "case {index} fails");
            assert!(written.is_empty(), "case {index} writes nothing: {written}");
        }
        fs::remove_dir_all(&directory).expect("the case directory is removed");
    }
    let (success, written) = run_pin(&script, &workspace_root());
    assert!(success, "the real file passes");
    assert_eq!(written, format!("toolchain={}\n", pinned_channel()));
}

#[test]
fn no_workflow_names_a_rustc_version() {
    for name in workflow_names() {
        if name == "planning.yml" || name == "b10x-docs-bundle.yml" {
            continue;
        }
        let text = fs::read_to_string(workspace_root().join(".github/workflows").join(&name))
            .expect("the workflow is readable");
        for (index, line) in text.lines().enumerate() {
            let at = format!("{name}:{}: {line}", index + 1);
            let trimmed = line.trim_start().trim_start_matches("- ");
            if let Some(value) = trimmed.strip_prefix("toolchain:") {
                let value = value.trim();
                assert!(value == PIN_OUTPUT || value == MSRV_OUTPUT, "{at}");
            }
            if let Some((_, value)) = line.split_once("RUSTUP_TOOLCHAIN") {
                let value = value.trim_start_matches(['=', ':']).trim();
                assert!(value == MSRV_OUTPUT, "{at}");
            }
            if let Some((_, value)) = line.split_once("cargo +") {
                assert!(value.starts_with(MSRV_OUTPUT), "{at}");
            }
            assert!(
                !line.contains("rustup default") && !line.contains("rustup toolchain install"),
                "{at}"
            );
        }
    }
}

#[test]
fn the_consumer_profile_measures_the_pinned_rustc() {
    let path = workspace_root().join("crates/edge/ess-xtask/src/consumer_coverage/mod.rs");
    let source = fs::read_to_string(&path).expect("the consumer coverage module is readable");
    let expected = format!("const MEASURED_TOOLCHAIN: &str = \"{}\";", pinned_channel());
    assert!(
        source.contains(&expected),
        "consumer_coverage/mod.rs holds `{expected}`"
    );
}
