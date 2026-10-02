//! Adversary pass 1 on `story:generated-server-publishes-and-reads-headers`: the committed gatepass
//! servers, Rust and Go, each linked with its hand-written realization, driven over real sockets
//! through every command and every branch the example reaches.
//!
//! Both must answer the same status and the same JSON for the same request sequence, and every
//! answer must validate against the contract the answering server serves. The Go server is also
//! built with the race detector and sent concurrent commands: `http.Serve` answers each connection
//! on its own goroutine, and every served command now pumps and takes from the one shared log.

use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde_json::{json, Value};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-gatepass-{name}-{}", std::process::id()))
}

fn rust_server() -> PathBuf {
    let target = scratch("rust-target");
    let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args([
            "build",
            "--offline",
            "--quiet",
            "--bin",
            "gatepass-server",
            "--manifest-path",
        ])
        .arg(root().join("examples/gatepass-realization/Cargo.toml"))
        .arg("--target-dir")
        .arg(&target)
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    assert!(
        built.status.success(),
        "the Rust gatepass server builds: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    target.join("debug/gatepass-server")
}

fn go_server(race: bool) -> PathBuf {
    let binary = scratch(if race { "go-race" } else { "go" });
    let mut command = Command::new("go");
    command.arg("build");
    if race {
        command.arg("-race");
    }
    let built = command
        .arg("-o")
        .arg(&binary)
        .arg("./cmd/gatepass-server")
        .current_dir(root().join("examples/gatepass-go-realization"))
        .env("GOPROXY", "off")
        .env("CGO_ENABLED", if race { "1" } else { "0" })
        .output()
        .expect("go build runs");
    assert!(
        built.status.success(),
        "the Go gatepass server builds: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    binary
}

struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn serve(binary: &Path) -> (Served, u16) {
    let mut child = Command::new(binary)
        .env("PORT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let mut lines = std::io::BufReader::new(stdout).lines();
    let first = lines.next().expect("a startup record").expect("text");
    let record: Value = serde_json::from_str(&first).expect("the record is JSON");
    let port = record["runtime"]["port"]
        .as_u64()
        .and_then(|port| u16::try_from(port).ok())
        .expect("the record names the port");
    std::thread::spawn(move || lines.for_each(drop));
    (Served(child), port)
}

fn request(port: u16, method: &str, path: &str, body: &str) -> (u16, Value) {
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("the server accepts");
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n\
         Authorization: Actor {RECEPTIONIST}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("the request writes");
    let mut answer = String::new();
    stream
        .read_to_string(&mut answer)
        .expect("the answer reads");
    let (head, payload) = answer.split_once("\r\n\r\n").expect("a head and a body");
    let status = head
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .expect("a status");
    let payload =
        serde_json::from_str(payload).unwrap_or_else(|_| Value::String(payload.to_owned()));
    (status, payload)
}

/// The actor both realizations' `authenticate` reads from `Authorization: Actor <name>`, and the one
/// the specification grants every command: without it every command is the standard `403`.
const RECEPTIONIST: &str = "gatepass.visit.Receptionist";

const REGISTER: &str = "/visits/commands/register-visit";
const ADMIT: &str = "/visits/commands/admit-visitor";
const SIGN_OUT: &str = "/visits/commands/sign-out-visitor";

fn register(minutes: i64) -> String {
    json!({
        "visitor": "Ada",
        "building": "North",
        "host": {"kind": "employee", "value": "E1"},
        "expected_minutes": minutes,
        "expected_stay": "PT30M",
        "deposit": {"amount": "10.50", "currency": "EUR"},
        "escorts": ["Bo"],
        "notes": {"k": "v"},
        "on_watchlist": false,
    })
    .to_string()
}

/// The sequence both servers are sent; each step is `(label, path, body)` and the body may name the
/// first or second registered visit as `{V1}` / `{V2}`.
fn steps() -> Vec<(&'static str, &'static str, String)> {
    let badge = r#"{"serial":"s-1","printed_at":"2026-09-30T08:00:00Z","signature":"AAEC"}"#;
    let bare = r#"{"serial":"s-2","signature":"AA=="}"#;
    vec![
        ("register", REGISTER, register(30)),
        ("register-refused", REGISTER, register(0)),
        (
            "admit",
            ADMIT,
            format!(r#"{{"visit_id":"{{V1}}","badge":{badge}}}"#),
        ),
        (
            "admit-again",
            ADMIT,
            format!(r#"{{"visit_id":"{{V1}}","badge":{badge}}}"#),
        ),
        ("sign-out", SIGN_OUT, r#"{"visit_id":"{V1}"}"#.to_owned()),
        (
            "sign-out-again",
            SIGN_OUT,
            r#"{"visit_id":"{V1}"}"#.to_owned(),
        ),
        (
            "sign-out-unknown",
            SIGN_OUT,
            r#"{"visit_id":"00000000-0000-4000-8000-0000000000ff"}"#.to_owned(),
        ),
        ("register-second", REGISTER, register(5)),
        (
            "sign-out-expected",
            SIGN_OUT,
            r#"{"visit_id":"{V2}"}"#.to_owned(),
        ),
        (
            "admit-bare-badge",
            ADMIT,
            format!(r#"{{"visit_id":"{{V2}}","badge":{bare}}}"#),
        ),
    ]
}

/// Runs the sequence; every answer with the visit ids replaced by `{V1}` / `{V2}`, and the contract
/// the server served.
fn drive(port: u16) -> (Vec<(&'static str, u16, Value)>, Value) {
    let mut ids: Vec<String> = Vec::new();
    let mut answers = Vec::new();
    for (label, path, body) in steps() {
        let mut body = body;
        for (index, id) in ids.iter().enumerate() {
            body = body.replace(&format!("{{V{}}}", index + 1), id);
        }
        let (status, answer) = request(port, "POST", path, &body);
        if path == REGISTER && status == 202 {
            ids.push(
                answer["published"][0]["payload"]["visit_id"]
                    .as_str()
                    .expect("the created identity is in the payload")
                    .to_owned(),
            );
        }
        let mut text = answer.to_string();
        for (index, id) in ids.iter().enumerate() {
            text = text.replace(id, &format!("{{V{}}}", index + 1));
        }
        answers.push((label, status, serde_json::from_str(&text).expect("JSON")));
    }
    let (_, contract) = request(port, "GET", "/openapi.json", "");
    (answers, contract)
}

fn validate(contract: &Value, path: &str, status: u16, answer: &Value) -> Result<(), String> {
    let schema_at = &contract["paths"][path]["post"]["responses"][status.to_string()]["content"]
        ["application/json"]["schema"];
    if !schema_at.is_object() {
        return Err(format!("no {status} declared at {path}"));
    }
    let mut schema = contract.clone();
    for key in ["$ref", "oneOf"] {
        if let Some(value) = schema_at.get(key) {
            schema[key] = value.clone();
        }
    }
    let validator = jsonschema::draft202012::new(&schema).expect("the contract compiles");
    let errors: Vec<String> = validator
        .iter_errors(answer)
        .map(|error| format!("{error} at {}", error.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// The ids are substituted back in before validation, so a `uuid` pattern sees a real identity.
fn concrete(answer: &Value) -> Value {
    let text = answer
        .to_string()
        .replace("{V1}", "00000000-0000-4000-8000-000000000001")
        .replace("{V2}", "00000000-0000-4000-8000-000000000002");
    serde_json::from_str(&text).expect("JSON")
}

#[test]
fn rust_and_go_gatepass_servers_answer_every_command_identically_and_to_their_contract() {
    let rust = rust_server();
    let go = go_server(false);
    let (rust_answers, rust_contract) = {
        let (_served, port) = serve(&rust);
        drive(port)
    };
    let (go_answers, go_contract) = {
        let (_served, port) = serve(&go);
        drive(port)
    };
    let _ = std::fs::remove_file(&go);
    let _ = std::fs::remove_dir_all(scratch("rust-target"));

    let paths: Vec<&str> = steps().iter().map(|(_, path, _)| *path).collect();
    let mut problems = Vec::new();
    for (index, ((label, rust_status, rust_answer), (_, go_status, go_answer))) in
        rust_answers.iter().zip(&go_answers).enumerate()
    {
        eprintln!(
            "{label}: rust {rust_status} {rust_answer}\n{label}:   go {go_status} {go_answer}"
        );
        if (rust_status, rust_answer) != (go_status, go_answer) {
            problems.push(format!(
                "`{label}`: Rust {rust_status} {rust_answer} but Go {go_status} {go_answer}"
            ));
        }
        for (language, contract, status, answer) in [
            ("Rust", &rust_contract, rust_status, rust_answer),
            ("Go", &go_contract, go_status, go_answer),
        ] {
            if let Err(error) = validate(contract, paths[index], *status, &concrete(answer)) {
                problems.push(format!("`{label}` {language} {status} {answer}: {error}"));
            }
        }
    }
    assert_eq!(rust_contract, go_contract, "both serve one contract");
    assert!(problems.is_empty(), "{problems:#?}");
    // The parity is about commands that ran: no step is the standard refusal of an ungranted
    // caller, and both registrations were accepted.
    for (language, answers) in [("Rust", &rust_answers), ("Go", &go_answers)] {
        let refused: Vec<&str> = answers
            .iter()
            .filter(|(_, status, _)| *status == 403)
            .map(|(label, _, _)| *label)
            .collect();
        assert!(
            refused.is_empty(),
            "{language} refused {refused:?} as ungranted"
        );
        let registered = answers
            .iter()
            .filter(|(label, status, _)| label.starts_with("register") && *status == 202)
            .count();
        assert_eq!(registered, 2, "{language} accepts both registrations");
    }
}

#[test]
fn the_go_gatepass_server_answers_concurrent_commands_without_a_data_race() {
    let binary = go_server(true);
    let mut child = Command::new(&binary)
        .env("PORT", "0")
        .env("GORACE", "halt_on_error=0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let mut stderr = child.stderr.take().expect("piped");
    let mut lines = std::io::BufReader::new(stdout).lines();
    let first = lines.next().expect("a startup record").expect("text");
    let record: Value = serde_json::from_str(&first).expect("JSON");
    let port = u16::try_from(record["runtime"]["port"].as_u64().expect("a port")).expect("u16");
    std::thread::spawn(move || lines.for_each(drop));
    let collected = std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        text
    });

    // Every request carries a read timeout: a goroutine that never answers is a finding, not a
    // test that never ends.
    let workers: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(move || {
                let mut unanswered = 0;
                for _ in 0..10 {
                    let body = register(30);
                    let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) else {
                        // The server is gone; what killed it is on its stderr.
                        unanswered += 1;
                        continue;
                    };
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_secs(20)))
                        .expect("a timeout");
                    let _ = write!(
                        stream,
                        "POST {REGISTER} HTTP/1.1\r\nHost: x\r\nAuthorization: Actor \
                         {RECEPTIONIST}\r\nContent-Length: {}\r\n\
                         Connection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let mut answer = String::new();
                    if stream.read_to_string(&mut answer).is_err() || answer.is_empty() {
                        unanswered += 1;
                    } else if !answer.starts_with("HTTP/1.1 202 ") {
                        // Answered, but not accepted: the race would not have been run.
                        unanswered += 1;
                    }
                }
                unanswered
            })
        })
        .collect();
    let unanswered: usize = workers
        .into_iter()
        .map(|worker| worker.join().expect("a worker"))
        .sum();
    // SIGQUIT makes a Go program print every goroutine's stack before it exits.
    let _ = Command::new("kill")
        .args(["-QUIT", &child.id().to_string()])
        .status();
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(&binary);
    let stderr = collected.join().expect("stderr");
    let spinning = stderr
        .split("\n\n")
        .filter(|goroutine| {
            goroutine.starts_with("goroutine ") && goroutine.contains("(*System).Pump")
        })
        .count();
    eprintln!("{}", stderr.chars().take(20000).collect::<String>());
    for line in stderr
        .lines()
        .filter(|line| line.starts_with("fatal error") || line.starts_with("panic:"))
    {
        eprintln!("--- the server died: {line}");
    }
    for goroutine in stderr
        .split("\n\n")
        .filter(|goroutine| {
            goroutine.starts_with("goroutine ") && goroutine.contains("(*System).Pump")
        })
        .take(3)
    {
        eprintln!("--- a goroutine inside Pump when stopped ---\n{goroutine}");
    }
    assert!(
        unanswered == 0 && !stderr.contains("DATA RACE"),
        "{unanswered} of 80 concurrent commands were never answered or not accepted; {spinning} \
         goroutines were \
         inside System.Pump when the server was stopped; data race reported: {}",
        stderr.contains("DATA RACE")
    );
}
