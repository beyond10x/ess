//! The generated library must be usable from an adopter's enclosing workspace.
use ess_cli_contract::{compile, Binding};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

#[test]
fn generated_library_is_a_path_dependency_of_an_enclosing_workspace() {
    let model = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(include_str!("fixtures/model.yaml")).unwrap(),
    )])
    .unwrap();
    let ir = ess_compiler::compile(&model, &ess_compiler::source::SourceMap::new()).unwrap();
    let binding = Binding::from_yaml(include_str!("fixtures/cli.yaml")).unwrap();
    let compiled = compile(&ir, &binding).unwrap();
    let temporary = tempfile::Builder::new()
        .prefix("ess-cli-adopter-")
        .tempdir()
        .unwrap();
    let root = temporary.path();
    for (path, contents) in ess_cli_project::project(&compiled) {
        let path = root.join("generated").join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"runtime\", \"consumer\"]\nexclude = [\"generated\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("runtime/src")).unwrap();
    std::fs::write(
        root.join("runtime/Cargo.toml"),
        "[package]\nname = \"existing-runtime\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("runtime/src/lib.rs"),
        "pub fn existing_runtime() {}\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("consumer/src")).unwrap();
    std::fs::write(
        root.join("consumer/Cargo.toml"),
        "[package]\nname = \"consumer\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\ngenerated_cli = { package = \"demo-cli-contract\", path = \"../generated\" }\n",
    )
    .unwrap();
    std::fs::write(
        root.join("consumer/src/lib.rs"),
        "#[test]\nfn links_generated_contract() {\nassert_eq!(generated_cli::plan().binary, \"demo\");\nassert_eq!(generated_cli::command().get_name(), \"demo\");\n}\n",
    )
    .unwrap();
    let metadata = std::process::Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "metadata",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout).unwrap();
    assert_eq!(metadata["workspace_members"].as_array().unwrap().len(), 2);
    let output = std::process::Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["test", "--offline", "--workspace"])
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_TARGET_DIR", root.join("target"))
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
    assert!(output.status.success(), "{log}");
    assert!(log.contains("1 passed; 0 failed"), "{log}");
}
