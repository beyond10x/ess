//! Guarded faults remain external and construct only eligible inputs.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::ScenarioStep;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = r#"
format: ess/6
system: delivery
version: v1
domain: delivery.mail
events:
  - name: delivery.mail.Sent
    fields: []
errors:
  - name: delivery.mail.Rejected
    fields: []
commands:
  - name: delivery.mail.Send
    input:
      - {name: retry, type: Boolean}
      - {name: recipient, type: String}
    outcomes:
      - name: sent
        emits: [delivery.mail.Sent]
      - name: rejected
        when:
          all: [retry == false, recipient != ""]
        external: the provider rejects the initial delivery
        error: delivery.mail.Rejected
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("guarded.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn eligibility_constrains_input_without_removing_fault_arrangement() {
    let ir = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&ir);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    let (_, scenario) = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "delivery.mail.Send/outcome/rejected")
        .unwrap();
    assert!(matches!(
        scenario.steps.first(),
        Some(ScenarioStep::ConfigureExternalOutcome { .. })
    ));
    let input = scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(input),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        input["retry"],
        ess_conformance::ScenarioValue::literal(ess_primitives::node::Node::Bool(false))
    );
    assert_ne!(
        input["recipient"],
        ess_conformance::ScenarioValue::literal(ess_primitives::node::Node::Text(String::new()))
    );
}

#[test]
fn legacy_formats_do_not_admit_guarded_external_semantics() {
    for version in 1..=5 {
        let text = MODEL.replace("ess/6", &format!("ess/{version}"));
        let raw = RawSpecFile::parse(&text);
        assert!(
            raw.is_err()
                || Specification::assemble([(Source::new("old.yaml"), raw.unwrap())]).is_err()
        );
    }
}

#[test]
fn a_finite_partition_does_not_hide_external_witness_fields() {
    let model = MODEL.replace("events:\n", "types:\n  - name: delivery.mail.Mode\n    kind: enum\n    variants: [A, B]\nevents:\n")
        .replace("      - {name: retry,", "      - {name: mode, type: delivery.mail.Mode}\n      - {name: retry,")
        .replace("      - name: sent\n        emits:", "      - name: sent-b\n        when: mode == B\n        emits: [delivery.mail.Sent]\n      - name: sent\n        when: mode == A\n        emits:");
    let result = ess_conformance::synthesize::synthesize(&ir(&model));
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
}

#[test]
fn impossible_external_eligibility_refuses_a_scenario() {
    let model = MODEL.replace(
        "retry == false, recipient != \"\"",
        "retry == false, retry == true",
    );
    let result = ess_conformance::synthesize::synthesize(&ir(&model));
    assert_eq!(result.refusals.len(), 1, "{:?}", result.refusals);
    assert!(format!("{:?}", result.refusals[0]).contains("GuardUnsatisfiable"));
}

#[test]
fn guarded_external_rejects_untyped_guards_and_empty_causes() {
    for model in [
        MODEL.replace("retry == false", "missing == false"),
        MODEL.replace("the provider rejects the initial delivery", "\"\""),
    ] {
        let raw = RawSpecFile::parse(&model);
        assert!(
            raw.is_err()
                || Specification::assemble([(Source::new("bad.yaml"), raw.unwrap())]).is_err()
        );
    }
}

#[test]
fn emitted_runtimes_fail_when_the_observed_outcome_ignores_the_fault() {
    let suite = ess_conformance::synthesize::synthesize(&ir(MODEL)).suite;
    let root = std::env::temp_dir().join(format!("ess-guarded-external-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/guarded\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/guarded_test.go"),
        include_str!("fixtures/guarded-external-runtime.go"),
    )
    .unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap() {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("guarded.mjs"),
        include_str!("fixtures/guarded-external-runtime.mjs"),
    )
    .unwrap();
    std::fs::write(
        ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = std::process::Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .expect("required TypeScript compiler");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    for ignore in [false, true] {
        for (tool, args, directory) in [
            ("go", vec!["test", "./essconform", "-count=1", "-v"], &root),
            ("node", vec!["--test", "guarded.mjs"], &ts),
        ] {
            let output = std::process::Command::new(tool)
                .args(args)
                .env("ESS_IGNORE_FAULT", if ignore { "1" } else { "0" })
                .env("ESS_REPORT_FORMAT", "2")
                .env("GOWORK", "off")
                .current_dir(directory)
                .output()
                .expect("required emitted runtime toolchain");
            let log = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.status.success(),
                !ignore,
                "{tool} ignore={ignore}: {log}"
            );
            if ignore {
                assert!(log.contains("delivery.mail.Send/outcome/rejected"), "{log}");
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
