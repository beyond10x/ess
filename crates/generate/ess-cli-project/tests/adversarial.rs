//! Adversarial boundary cases for the admitted CLI projection and native sources.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, OsSources, ProtectedSource, Sources,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

const MODEL: &str = include_str!("fixtures/model.yaml");
const BINDING: &str = include_str!("fixtures/cli.yaml");

fn resolved(model: &str, binding: &str) -> Result<CompiledBinding, ess_cli_contract::Error> {
    let spec = Specification::assemble(vec![(
        Source::new("model.yaml"),
        RawSpecFile::parse(model).unwrap(),
    )])
    .unwrap();
    let ir = ess_compiler::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap();
    compile(&ir, &Binding::from_yaml(binding).unwrap())
}

#[derive(Default)]
struct Calls(Vec<Value>);

impl Handler for Calls {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.0.push(invocation.input.clone());
        HandlerReply::Success(json!({"profile":"safe"}))
    }
}

#[derive(Default)]
struct Reads(usize);

impl Sources for Reads {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        self.0 += 1;
        Ok("protected-canary".to_owned())
    }
}

#[test]
fn admitted_binary_names_emit_cargo_accepted_targets() {
    for binary in ["build", "deps", "examples", "incremental"] {
        let binding = BINDING.replace("binary: demo", &format!("binary: {binary}"));
        let Ok(compiled) = resolved(MODEL, &binding) else {
            // Refusing unsupported projection names at compilation is permitted.
            continue;
        };
        let directory = tempfile::tempdir().unwrap();
        for (name, text) in ess_cli_project::project(&compiled) {
            let path = directory.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let output = std::process::Command::new(env!("CARGO"))
            .args([
                "metadata",
                "--offline",
                "--no-deps",
                "--format-version",
                "1",
            ])
            .current_dir(directory.path())
            .env("CARGO_TARGET_DIR", directory.path().join("target"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "compiler admitted binary {binary}, but its emitted Cargo package is invalid:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[cfg(unix)]
#[test]
fn document_file_refuses_a_fifo_without_waiting_for_an_external_writer() {
    use rustix::fs::{Mode, OFlags, CWD};
    use std::sync::mpsc;
    use std::time::Duration;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("input.fifo");
    rustix::fs::mkfifoat(CWD, &path, Mode::RUSR | Mode::WUSR).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let reader_path = path.clone();
    let reader = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        result_tx
            .send(OsSources.acquire(ProtectedSource::DocumentFile(reader_path)))
            .unwrap();
    });
    started_rx.recv().unwrap();
    let early = result_rx.recv_timeout(Duration::from_secs(1));
    if early.is_err() {
        // Release the blocked open before failing; do not leave a live probe thread.
        let writer =
            rustix::fs::open(&path, OFlags::RDWR | OFlags::NONBLOCK, Mode::empty()).unwrap();
        assert_eq!(
            result_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            Err(AcquireError::Unavailable)
        );
        drop(writer);
    }
    reader.join().unwrap();
    assert_eq!(
        early,
        Ok(Err(AcquireError::Unavailable)),
        "a document source must reject the non-regular file without an external writer releasing open"
    );
}

#[cfg(unix)]
#[test]
fn protected_hardlinks_are_refused_while_regular_documents_remain_admitted() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("secret");
    std::fs::write(&path, "protected-canary").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::hard_link(&path, directory.path().join("alias")).unwrap();
    assert_eq!(
        OsSources.acquire(ProtectedSource::File(path.clone())),
        Err(AcquireError::Unavailable)
    );
    assert_eq!(
        OsSources.acquire(ProtectedSource::DocumentFile(path)),
        Ok("protected-canary".to_owned())
    );
}

#[test]
fn invalid_ordinary_values_are_refused_before_protected_acquisition() {
    let model = MODEL.replace(
        "name: profile, type: String",
        "name: profile, type: Integer",
    );
    let binding = resolved(&model, BINDING).unwrap();
    let mut sources = Reads::default();
    let mut handler = Calls::default();
    let output = runtime::run(
        binding.plan(),
        [
            "demo",
            "credential",
            "store",
            "--profile",
            "wrong",
            "--secret-stdin",
            "--output=json",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
        &mut sources,
        &mut handler,
        None,
    );
    assert_eq!(output.exit_code, 2);
    assert_eq!(sources.0, 0);
    assert!(handler.0.is_empty());
    assert!(!output.stderr.contains("wrong"));
    assert!(output.stdout.is_empty());
}

#[test]
fn authored_field_names_dispatch_using_resolved_wire_names() {
    let model = MODEL.replacen(
        "{name: profile, type: String}",
        "{name: profile, type: String, wire: account}",
        1,
    );
    let binding = resolved(&model, BINDING).unwrap();
    let mut sources = Reads::default();
    let mut handler = Calls::default();
    let output = runtime::run(
        binding.plan(),
        [
            "demo",
            "credential",
            "set",
            "--profile",
            "p",
            "--secret-stdin",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
        &mut sources,
        &mut handler,
        None,
    );
    assert_eq!(output.exit_code, 0);
    assert_eq!(sources.0, 1);
    assert_eq!(
        handler.0,
        [json!({"account":"p", "secret":"protected-canary"})]
    );
}
