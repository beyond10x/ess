//! Existing generated runtime transport, backed by this test's independent Rust target.
use super::Backend;
use ess_conformance::{target::*, AdmittedSuite};
use ess_primitives::ids::CorrelationId;
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
    sync::OnceLock,
    thread,
};

pub struct Host {
    pub address: String,
    handle: Option<thread::JoinHandle<Backend>>,
}
impl Host {
    pub fn start(target: Backend) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut line = String::new();
                BufReader::new(&mut stream).read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                if request["method"] == "stop" {
                    break;
                }
                if request["method"] != "execute" {
                    target.callbacks.set(target.callbacks.get() + 1);
                }
                let result = match request["method"].as_str().unwrap() {
                    "identity" => json!({"Name":"nested-response-fixture","Version":"1"}),
                    "begin" | "end" => Value::Null,
                    "execute" => {
                        let result = target
                            .execute_command(SemanticCommandRequest {
                                command: request["args"]["Command"]
                                    .as_str()
                                    .unwrap()
                                    .parse()
                                    .unwrap(),
                                actor: None,
                                caller: None,
                                input: std::collections::BTreeMap::new(),
                                correlation: CorrelationId::new("nested-live").unwrap(),
                            })
                            .unwrap();
                        json!({"Outcome": result.outcome.unwrap().outcome, "Response":result.response,
                            "DirectEvents":result.direct_events.iter().map(|e|json!({"Event":e.event,"Payload":e.payload})).collect::<Vec<_>>()})
                    }
                    other => panic!("unexpected live callback {other}"),
                };
                writeln!(stream, "{}", json!({"ok": result})).unwrap();
            }
            target
        });
        Self {
            address,
            handle: Some(handle),
        }
    }
    pub fn stop(mut self) -> Backend {
        writeln!(
            TcpStream::connect(&self.address).unwrap(),
            "{{\"method\":\"stop\"}}"
        )
        .unwrap();
        self.handle.take().unwrap().join().unwrap()
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            if let Ok(mut stream) = TcpStream::connect(&self.address) {
                let _ = writeln!(stream, "{{\"method\":\"stop\"}}");
            }
            let _ = handle.join();
        }
    }
}

pub fn typescript_package(suite: &AdmittedSuite) -> &'static PathBuf {
    static PACKAGE: OnceLock<PathBuf> = OnceLock::new();
    PACKAGE.get_or_init(|| {
        let root = super::super::support_go::directory("nested-response-ts");
        for artifact in ess_conformance::ts::emit(suite.suite()).unwrap() {
            let path = root.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        let package = root.join("essconform");
        let mut command = Command::new("tsc");
        if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
            command
                .arg("--typeRoots")
                .arg(PathBuf::from(modules).join("@types"));
        }
        let output = command
            .args(["--project", "tsconfig.json"])
            .current_dir(&package)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        // Reuse the existing runtime's live adapter rather than adding another executable asset.
        let driver = include_str!("../runtime_parity_typescript_28_35.rs")
            .split_once("const LIVE_DRIVER: &str = r\"")
            .unwrap()
            .1
            .split_once("\";")
            .unwrap()
            .0;
        std::fs::write(package.join("live.mjs"), driver).unwrap();
        package
    })
}

pub fn typescript_run(
    suite: &AdmittedSuite,
    document: &str,
    target: Backend,
    label: &str,
) -> (std::process::Output, Option<Value>, Backend) {
    let package = typescript_package(suite);
    let input = package.join(format!("{label}.json"));
    let report = package.join(format!("{label}-report.json"));
    std::fs::write(&input, document).unwrap();
    let _ = std::fs::remove_file(&report);
    let host = Host::start(target);
    let output = Command::new("node")
        .arg(package.join("live.mjs"))
        .arg(input)
        .arg(&host.address)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .unwrap();
    let target = host.stop();
    let report = std::fs::read(report)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap());
    (output, report, target)
}

pub fn go_run(
    suite: &AdmittedSuite,
    document: &str,
    target: Backend,
    label: &str,
) -> (super::super::support_go::GoRun, Backend) {
    let client = include_str!("../support_initial_state/mod.rs")
        .split_once("pub const GO: &str = r#\"")
        .unwrap()
        .1
        .split_once("\"#;")
        .unwrap()
        .0;
    let directory =
        super::super::support_go::package(label, suite.suite(), &[("live_test.go", client)]);
    std::fs::write(directory.join("essconform/suite.json"), document).unwrap();
    let host = Host::start(target);
    let run = super::super::support_go::go_test(
        &directory,
        "TestLive",
        &[("PARITY_ADDRESS", &host.address)],
    );
    let target = host.stop();
    std::fs::remove_dir_all(directory).unwrap();
    (run, target)
}
