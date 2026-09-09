//! Projection and actual generated-package process acceptance cases.
use ess_cli_contract::{compile, Binding};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

#[test]
fn service_commands_and_parameterized_views_dispatch_to_the_declared_owner() {
    use ess_cli_project::runtime::{
        AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
    };
    use ess_cli_project::wire::Target;
    use serde_json::{json, Value};
    struct NoSources;
    impl Sources for NoSources {
        fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
            panic!("this command has no external input source")
        }
    }
    #[derive(Default)]
    struct Recorder(Vec<(String, String, Value)>);
    impl Handler for Recorder {
        fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
            let Target::ServiceForward { owner, operation } = invocation.target else {
                panic!("expected service target")
            };
            self.0
                .push((owner.clone(), operation.clone(), invocation.input.clone()));
            HandlerReply::Success(if invocation.callable == "query" {
                json!([{"name":"a"}])
            } else {
                json!({"name":"a"})
            })
        }
    }
    let specification = Specification::assemble(vec![
        (
            Source::new("service.yaml"),
            RawSpecFile::parse(include_str!(
                "../../../specify/ess-cli-contract/tests/fixtures/service-model.yaml"
            ))
            .unwrap(),
        ),
        (
            Source::new("foreign.yaml"),
            RawSpecFile::parse("domain: demo.foreign\n").unwrap(),
        ),
    ])
    .unwrap();
    let model =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    let binding = Binding::from_yaml(include_str!(
        "../../../specify/ess-cli-contract/tests/fixtures/service-cli.yaml"
    ))
    .unwrap();
    let compiled = compile(&model, &binding).unwrap();
    let mut recorder = Recorder::default();
    for args in [
        vec!["records", "rename", "--name", "a", "--output=json"],
        vec![
            "records",
            "records",
            "find",
            "--prefix",
            "a",
            "--output=json",
        ],
    ] {
        let result = ess_cli_project::runtime::run(
            compiled.plan(),
            args.into_iter().map(Into::into).collect(),
            &mut NoSources,
            &mut recorder,
            None,
        );
        assert_eq!(result.exit_code, 0);
        assert!(result.stderr.is_empty());
    }
    assert_eq!(
        recorder.0,
        vec![
            (
                "owner-service".to_owned(),
                "demo.records.Rename".to_owned(),
                json!({"name":"a"})
            ),
            (
                "owner-service".to_owned(),
                "demo.records.NamedRecords".to_owned(),
                json!({"prefix":"a"})
            ),
        ]
    );
    let refusal = ess_cli_project::runtime::run(
        compiled.plan(),
        ["records", "records", "find", "--output=json"]
            .into_iter()
            .map(Into::into)
            .collect(),
        &mut NoSources,
        &mut recorder,
        None,
    );
    assert_eq!(refusal.exit_code, 2);
    assert_eq!(recorder.0.len(), 2);
}

#[test]
fn emits_help_and_reference_from_the_resolved_binding() {
    let model = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(include_str!("fixtures/model.yaml")).unwrap(),
    )])
    .unwrap();
    let ir = ess_compiler::compile(&model, &ess_compiler::source::SourceMap::new()).unwrap();
    let binding = Binding::from_yaml(include_str!("fixtures/cli.yaml")).unwrap();
    let compiled = compile(&ir, &binding).unwrap();
    let artifacts = ess_cli_project::project(&compiled);
    let help = artifacts.get("help.txt").expect("generated process help");
    assert!(help.contains("credential"));
    assert!(help.contains("--output"));
    let reference = artifacts.get("README.md").expect("generated reference");
    for content in [
        "credential store",
        "credential set",
        "demo.Input",
        "demo.Stored",
        "store_failed",
        "handler:store:",
        "dynamic-validator:invoke:",
        "unavailable handler",
    ] {
        assert!(
            reference.contains(content),
            "missing reference fact {content}"
        );
    }
    assert_eq!(artifacts, ess_cli_project::project(&compiled));
    let manifest: serde_json::Value = serde_json::from_str(&artifacts["manifest.json"]).unwrap();
    for path in ["help.txt", "README.md"] {
        assert!(manifest["files"].as_array().unwrap().contains(&path.into()));
    }
}

#[test]
fn emits_a_deterministic_standalone_parser_process_package() {
    let model = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(include_str!(
            "../../../specify/ess-cli-contract/tests/fixtures/model.yaml"
        ))
        .unwrap(),
    )])
    .unwrap();
    let ir = ess_compiler::compile(&model, &ess_compiler::source::SourceMap::new()).unwrap();
    let binding = Binding::from_yaml(include_str!(
        "../../../specify/ess-cli-contract/tests/fixtures/cli.yaml"
    ))
    .unwrap();
    let compiled = compile(&ir, &binding).unwrap();
    let artifacts = ess_cli_project::project(&compiled);
    assert_eq!(artifacts, ess_cli_project::project(&compiled));
    for path in [
        "Cargo.toml",
        "src/lib.rs",
        "src/main.rs",
        "src/runtime.rs",
        "binding.json",
        "manifest.json",
        "completions/demo.bash",
    ] {
        assert!(artifacts.contains_key(path), "missing {path}");
    }
    assert!(artifacts["Cargo.toml"].contains("clap"));
    assert!(artifacts["manifest.json"].contains("handler"));
}

#[test]
fn generated_package_compiles_offline_and_executes_process_fixtures() {
    let model = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(include_str!("fixtures/model.yaml")).unwrap(),
    )])
    .unwrap();
    let ir = ess_compiler::compile(&model, &ess_compiler::source::SourceMap::new()).unwrap();
    let binding = Binding::from_yaml(include_str!("fixtures/cli.yaml")).unwrap();
    let compiled = compile(&ir, &binding).unwrap();
    let artifacts = ess_cli_project::project(&compiled);
    let temporary = tempfile::Builder::new()
        .prefix("ess-cli-fixture-")
        .tempdir()
        .unwrap();
    let directory = temporary.path();
    for (path, contents) in &artifacts {
        let path = directory.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    std::fs::create_dir_all(directory.join("tests")).unwrap();
    std::fs::write(
        directory.join("tests/process.rs"),
        include_str!("fixtures/process.rs"),
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO"))
        .args(["test", "--offline", "--manifest-path"])
        .arg(directory.join("Cargo.toml"))
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_TARGET_DIR", directory.join("target"))
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .env("CARGO_PROFILE_TEST_DEBUG", "0")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{log}");
    assert!(
        output.status.success(),
        "generated fixture exited {}:\n{log}",
        output.status
    );
    assert!(
        log.contains("12 passed; 0 failed"),
        "fixture test count must select the authored cases"
    );
    let binary = directory.join("target/debug/demo");
    let output = std::process::Command::new(&binary)
        .args(["show", "--output=json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("cli_handler_unavailable"));
    let output = std::process::Command::new(&binary)
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("credential"));
}
