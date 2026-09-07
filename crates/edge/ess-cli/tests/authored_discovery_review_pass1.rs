//! Regression controls derived from the published directory configuration contract.
use ess_conformance::AdmittedSuite;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

const MODEL_FILES: [&str; 5] = [
    "system.yaml",
    "domains/invoice.yaml",
    "domains/email.yaml",
    "components.yaml",
    "topology.yaml",
];
const SCENARIOS: [&str; 2] = [
    "authored/routing/first.yaml",
    "authored/e2e/second.scenario",
];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = repo().join(format!(
            "target/review-boundaries-17/adversary-pass-1/fixtures/{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let f = Self(root);
        for path in MODEL_FILES {
            f.write(
                &format!("model/{path}"),
                fs::read(repo().join("examples/billing").join(path)).unwrap(),
            );
        }
        let scenario = fs::read_to_string(
            repo().join("examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml"),
        )
        .unwrap();
        for (i, path) in SCENARIOS.iter().enumerate() {
            f.write(
                path,
                scenario.replace(
                    "scenario: outstanding-invoices-rank-latest-first",
                    &format!("scenario: local-review-{i}"),
                ),
            );
        }
        f.write("generated/openapi/poison.yaml", "[:");
        f.write("generated/asyncapi/poison.yaml", "[:");
        f.write("output/sentinel", "owned");
        f.manifest(&Self::configuration());
        f
    }

    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn configuration() -> Value {
        json!({"format":"ess-inputs/1", "specification":MODEL_FILES.map(|p|format!("model/{p}")), "scenarios":SCENARIOS})
    }

    fn manifest(&self, value: &Value) {
        self.write("ess-inputs.yaml", serde_yaml::to_string(value).unwrap());
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_ess"));
        c.current_dir(&self.0).args(args);
        c
    }

    fn run(&self, command: &mut Command) -> Output {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = self
            .0
            .join(format!("commands/{}", NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("command.txt"), format!("{command:?}\n")).unwrap();
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let start = Instant::now();
        let child = command.spawn().unwrap();
        let pid = child.id();
        let output = child.wait_with_output().unwrap();
        fs::write(directory.join("stdout"), &output.stdout).unwrap();
        fs::write(directory.join("stderr"), &output.stderr).unwrap();
        fs::write(
            directory.join("direct.json"),
            json!({"pid":pid,"exit":output.status.code(),"seconds":start.elapsed().as_secs_f64()})
                .to_string(),
        )
        .unwrap();
        output
    }

    fn good(&self, command: &mut Command) -> Output {
        let output = self.run(command);
        assert!(output.status.success(), "{command:?}: {output:?}");
        output
    }
}

fn identities(out: &Output) -> Vec<String> {
    AdmittedSuite::from_json(std::str::from_utf8(&out.stdout).unwrap())
        .unwrap()
        .coverage()
        .unwrap()
        .authored_sources
        .keys()
        .map(|name| name.as_str().to_owned())
        .collect()
}

#[test]
fn manifest_requires_named_fields_before_model_or_scenario_outputs() {
    let f = Fixture::new();
    f.good(&mut f.command(&["specify", "validate", "--format", "json"]));
    let valid = f.good(&mut f.command(&[
        "conform",
        "author",
        "--scenarios",
        ".",
        "--suite-format",
        "5",
        "--format",
        "json",
    ]));
    assert_eq!(identities(&valid), [SCENARIOS[1], SCENARIOS[0]]);
    let configuration = Fixture::configuration();
    f.manifest(&json!([
        configuration["format"],
        configuration["specification"],
        configuration["scenarios"]
    ]));
    let mut outputs = vec![f.run(&mut f.command(&["specify", "validate", "--format", "json"]))];
    for version in ["4", "5"] {
        outputs.push(
            f.run(
                f.command(&[
                    "conform",
                    "author",
                    "--path",
                    "model",
                    "--scenarios",
                    ".",
                    "--suite-format",
                    version,
                    "--format",
                    "json",
                    "--out",
                ])
                .arg(format!("output/should-not-exist-{version}.json")),
            ),
        );
    }
    for output in outputs {
        assert!(
            !output.status.success(),
            "a positional sequence is not the named-field manifest: {output:?}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("ess-inputs.yaml"),
            "{output:?}"
        );
        assert!(output.stdout.is_empty(), "{output:?}");
    }
    assert_eq!(fs::read_dir(f.0.join("output")).unwrap().count(), 1);
    assert_eq!(fs::read(f.0.join("output/sentinel")).unwrap(), b"owned");
}

#[test]
fn documented_mixed_root_preserves_default_model_and_omitted_scenarios() {
    let f = Fixture::new();
    let explicit =
        f.good(&mut f.command(&["specify", "compile", "--path", ".", "--format", "json"]));
    let default = f.good(&mut f.command(&["specify", "compile", "--format", "json"]));
    assert_eq!(explicit.stdout, default.stdout);
    let mut snapshots = Vec::new();
    for version in ["4", "5"] {
        let omitted = f.good(&mut f.command(&[
            "conform",
            "author",
            "--suite-format",
            version,
            "--format",
            "json",
        ]));
        let selected = f.good(&mut f.command(&[
            "verify",
            "conform",
            "author",
            "--path",
            ".",
            "--scenarios",
            ".",
            "--suite-format",
            version,
            "--format",
            "json",
        ]));
        assert_ne!(omitted.stdout, selected.stdout);
        let default = f.good(&mut f.command(&[
            "conform",
            "author",
            "--scenarios",
            ".",
            "--suite-format",
            version,
            "--format",
            "json",
        ]));
        assert_eq!(selected.stdout, default.stdout);
        if version == "5" {
            assert!(identities(&omitted).is_empty());
            assert_eq!(identities(&selected), [SCENARIOS[1], SCENARIOS[0]]);
        }
        snapshots.push(omitted.stdout);
    }
    assert_after_generation(&f, &explicit, snapshots);
}

fn assert_after_generation(f: &Fixture, explicit: &Output, snapshots: Vec<Vec<u8>>) {
    // Generated artifacts are co-located exactly as the public guide permits.
    f.good(&mut f.command(&[
        "verify",
        "conform",
        "author",
        "--path",
        ".",
        "--scenarios",
        ".",
        "--suite-format",
        "5",
        "--out",
        "output/authored.json",
    ]));
    f.write("generated/model-copy.yaml", &explicit.stdout);
    f.write("scenarios/poison.yaml", "[:");
    let mut manifest = Fixture::configuration();
    manifest["scenarios"] = json!(["missing/inactive"]);
    f.manifest(&manifest);
    assert_eq!(
        explicit.stdout,
        f.good(&mut f.command(&["specify", "compile", "--format", "json"]))
            .stdout
    );
    for (version, expected) in ["4", "5"].into_iter().zip(snapshots) {
        assert_eq!(
            expected,
            f.good(&mut f.command(&[
                "conform",
                "author",
                "--suite-format",
                version,
                "--format",
                "json",
            ]))
            .stdout
        );
    }
    let refused = f.run(&mut f.command(&[
        "conform",
        "author",
        "--scenarios",
        ".",
        "--suite-format",
        "5",
        "--out",
        "output/sentinel",
        "--format",
        "json",
    ]));
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("missing/inactive"));
    assert!(refused.stdout.is_empty());
    assert_eq!(fs::read(f.0.join("output/sentinel")).unwrap(), b"owned");
}

#[test]
fn string_filenames_keep_their_yaml_scalar_spelling_and_inactive_types_are_checked() {
    let f = Fixture::new();
    for spelling in ["42", "true", "null", "0x2a"] {
        f.write(spelling, fs::read(f.0.join(SCENARIOS[0])).unwrap());
        let mut manifest = Fixture::configuration();
        manifest["scenarios"] = json!([spelling]);
        f.manifest(&manifest);
        let out = f.good(&mut f.command(&[
            "conform",
            "author",
            "--scenarios",
            ".",
            "--suite-format",
            "5",
            "--format",
            "json",
        ]));
        assert_eq!(identities(&out), [spelling]);
        // YAML's actual scalar kind, not whether a same-spelled file exists, owns admission.
        f.write(
            "ess-inputs.yaml",
            format!("format: ess-inputs/1\nspecification: [missing]\nscenarios: [{spelling}]\n"),
        );
        let out = f.run(&mut f.command(&["specify", "validate", "--format", "json"]));
        assert!(!out.status.success());
        let diagnostic = String::from_utf8_lossy(&out.stderr);
        assert!(
            diagnostic.contains("expected a string list entry"),
            "{out:?}"
        );
        assert!(!diagnostic.contains("reading missing"), "{out:?}");
        assert!(out.stdout.is_empty());
    }
}
