use super::{check_report, command, files, fixtures, hash, list, text, transport, write_json};
use ess_conformance::coverage::AdmittedInput;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn generate(directory: &Path, input: &AdmittedInput) -> PathBuf {
    transport(directory, input);
    for artifact in ess_conformance::go::emit_input(input).unwrap() {
        let path = directory.join(artifact.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, artifact.contents).unwrap();
    }
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/go-billing/target.go"),
        directory.join("target.go"),
    )
    .unwrap();
    fs::copy(
        fixtures().join("producer_test.go"),
        directory.join("producer_test.go"),
    )
    .unwrap();
    fs::copy(
        fixtures().join("fixture_clock.go"),
        directory.join("essconform/fixture_clock.go"),
    )
    .unwrap();
    fs::write(directory.join("go.mod"), "module essbilling\n\ngo 1.24\n").unwrap();
    let sources: BTreeMap<_, _> = files(directory)
        .iter()
        .map(|path| {
            (
                path.strip_prefix(directory)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                hash(path),
            )
        })
        .collect();
    write_json(
        &directory.join("generated-source-manifest.json"),
        &json!(sources),
    );
    let binary = directory.join("producer.test");
    let output = command(
        directory,
        "go-build",
        Command::new("go")
            .args(["test", "-c", "-o"])
            .arg(&binary)
            .arg(".")
            .current_dir(directory),
    );
    assert!(output.status.success(), "{output:?}");
    fs::write(directory.join("producer.test.sha256"), hash(&binary) + "\n").unwrap();
    binary
}
fn configured(directory: &Path, binary: &Path, instance: &Value) -> Command {
    let mut command = Command::new(binary);
    command
        .args(["-test.v", "-test.count=1"])
        .current_dir(directory)
        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
        .env_remove("ESS_CONFORMANCE_STRICT")
        .env_remove("ESS_REPORT_FORMAT")
        .env_remove("ESS_REPORT_OUT")
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", directory.join("report.json"))
        .env("ESS_FIXTURE_CALLBACKS", directory.join("callbacks.jsonl"))
        .env("ESS_FIXTURE_CLOCK_LOG", directory.join("clock.jsonl"))
        .env("ESS_FIXTURE_CLOCK", instance["clock"].to_string())
        .env("ESS_FIXTURE_MODE", text(&instance["target_mode"]))
        .env(
            "ESS_FIXTURE_TARGET",
            if instance["target"] == "single_scenario_controls" {
                "control"
            } else {
                "billing"
            },
        );
    if instance["strict"] == true {
        command.env("ESS_CONFORMANCE_STRICT", "1");
    }
    command
}
fn callbacks(directory: &Path, expected: &Value) {
    let path = directory.join("callbacks.jsonl");
    let original = if path.exists() {
        fs::read_to_string(path).unwrap()
    } else {
        String::new()
    };
    let values: Vec<Value> = original
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for method in [
        "target_factory",
        "identity",
        "begin",
        "execute_command",
        "end",
    ] {
        assert_eq!(
            json!(values
                .iter()
                .filter(|call| call["method"] == method)
                .count()),
            expected[method],
            "{}/{method}",
            directory.display()
        );
    }
}
pub(super) fn export(directory: &Path, plan: &Value, structures: &BTreeMap<String, AdmittedInput>) {
    let mut executed = 0;
    for instance in list(&plan["requested_report_instances"])
        .iter()
        .filter(|i| i["producer_profile"] == "go-scenario-status/1")
    {
        let input = &structures[text(&instance["structure"])];
        let out = directory.join(text(&instance["id"]));
        fs::create_dir_all(&out).unwrap();
        let binary = generate(&out, input);
        let output = command(&out, "go-run", &mut configured(&out, &binary, instance));
        assert_eq!(
            output.status.code().map(i64::from),
            instance["expected_diagnostic_producer_exit"].as_i64(),
            "{}: {output:?}",
            instance["id"]
        );
        callbacks(&out, &instance["callbacks"]);
        let clock = instance["clock"].as_u64().unwrap();
        assert_eq!(
            fs::read_to_string(out.join("clock.jsonl")).unwrap(),
            format!("{clock}\n")
        );
        let original = fs::read_to_string(out.join("report.json")).unwrap();
        check_report(instance, &original, input, clock);
        write_json(
            &out.join("fixture.json"),
            &json!({
                "id":instance["id"],"structure":instance["structure"],"producer_profile":instance["producer_profile"],
                "report":"report.json","suite":"suite.json","input":"input.json","transport":"transport.json",
                "callbacks":"callbacks.jsonl","clock_transcript":"clock.jsonl",
                "independent_expected":instance,"independent_completed_at":clock,
                "diagnostic_producer_exit":output.status.code(),
                "source_manifest":"../source-manifest.json","producer_receipt":"../producer.json",
                "generated_source_manifest":"generated-source-manifest.json","binary_sha256":hash(&binary),
                "command_receipt":"go-run.receipt.json"
            }),
        );
        println!(
            "{}: actual Go process {:?}",
            instance["id"],
            output.status.code()
        );
        executed += 1;
    }
    assert_eq!(executed, 15);
    println!("actual Go producer exports: {executed}");
}

pub(super) fn refuse(out: &Path, input: &AdmittedInput, control: &Value) {
    let binary = generate(out, input);
    let mut instance = json!({"clock":0,"target_mode":"passed","target":"single_scenario_controls","strict":false});
    if control.get("clock").is_some() {
        instance["clock"] = control["clock"].clone();
    }
    let mut invocation = configured(out, &binary, &instance);
    invocation.env("ESS_CONFORMANCE_ALLOW_INCOMPLETE", "1");
    match text(&control["id"]) {
        "R01-go-host-filter" => {
            invocation.arg("-test.run=^TestConformance$/^does-not-match$");
        }
        "R02-go-negative-clock" => {}
        "R03-go-suite5-report1-no-destination" => {
            invocation
                .env("ESS_REPORT_FORMAT", "1")
                .env_remove("ESS_REPORT_OUT");
        }
        _ => panic!("unrecognized independent refusal control"),
    }
    let output = command(out, "go-run", &mut invocation);
    assert!(!output.status.success(), "{output:?}");
    callbacks(out, &control["callbacks"]);
    assert!(!out.join("report.json").exists());
    if control["id"] == "R02-go-negative-clock" {
        assert_eq!(fs::read_to_string(out.join("clock.jsonl")).unwrap(), "-1\n");
    }
    write_json(
        &out.join("fixture.json"),
        &json!({
            "independent_expected":control,"report_expected":false,"command_receipt":"go-run.receipt.json",
            "callbacks":"callbacks.jsonl","producer_receipt":"../producer.json","transport":"transport.json",
            "binary_sha256":hash(&binary),"generated_source_manifest":"generated-source-manifest.json"
        }),
    );
}
