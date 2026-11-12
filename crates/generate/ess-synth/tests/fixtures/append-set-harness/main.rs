//! In-memory ports for the Rust system `tests/append_set.rs` synthesizes from its `demo` model, and
//! the line protocol that test drives it through.
//!
//! No view query and no command behaviour is written here: every command and view reaches
//! `demo_types::behaviour::Generated` through the generated server's in-process dispatcher, so a
//! request body is read by the generated JSON decoder. This file supplies only where a job is
//! stored.
//!
//! The protocol is one JSON object per line each way:
//!
//! - `{"op":"reset"}` opens an empty scenario and answers `{"status":0,"body":{}}`;
//! - `{"op":"command","command":…,"body":…}` runs one command with the request body `body`, its
//!   input as JSON text, and answers the served status and body;
//! - `{"op":"view","view":…,"query":…}` reads one view and answers the served status and body.
//!
//! Its arguments are the route table, as `name method path` triples.

use std::cell::RefCell;
use std::io::{BufRead, Write};
use std::rc::Rc;

use demo_server::jobs_service as surface;
use demo_server::{http, json};
use demo_types::behaviour::{Generated, JobStorage};
use demo_types::jobs;

#[derive(Default)]
struct Shared {
    jobs: Vec<jobs::JobSnapshot>,
}

#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl JobStorage for Ports {
    fn get(&self, identity: &jobs::JobId) -> Option<jobs::JobSnapshot> {
        self.0
            .borrow()
            .jobs
            .iter()
            .find(|held| held.data.job_id == *identity)
            .cloned()
    }

    fn put(&mut self, snapshot: jobs::JobSnapshot) {
        let mut shared = self.0.borrow_mut();
        match shared
            .jobs
            .iter_mut()
            .find(|held| held.data.job_id == snapshot.data.job_id)
        {
            Some(held) => *held = snapshot,
            None => shared.jobs.push(snapshot),
        }
    }

    fn delete(&mut self, identity: &jobs::JobId) {
        self.0
            .borrow_mut()
            .jobs
            .retain(|held| held.data.job_id != *identity);
    }

    fn list(&self) -> Vec<jobs::JobSnapshot> {
        self.0.borrow().jobs.clone()
    }
}

fn text<'v>(value: &'v json::Value, name: &str) -> Option<&'v str> {
    match value.member(name) {
        Some(json::Value::Text(text)) => Some(text),
        _ => None,
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let routes: Vec<(String, String, String)> = arguments
        .chunks(3)
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect();
    let ports = Ports::default();
    let mut system = demo_system::System::new(jobs_service::JobsService::new(Generated::new(
        ports.clone(),
    )));
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("the test writes lines");
        let request = json::parse(&line).expect("the test writes JSON");
        let answer = match text(&request, "op") {
            Some("reset") => {
                *ports.0.borrow_mut() = Shared::default();
                "{\"status\":0,\"body\":{}}".to_owned()
            }
            Some(op @ ("command" | "view")) => {
                let key = if op == "command" { "command" } else { "view" };
                let name = text(&request, key).unwrap_or_default();
                let (_, method, path) = routes
                    .iter()
                    .find(|(declared, _, _)| declared == name)
                    .unwrap_or_else(|| panic!("`{name}` is a route the model declares"));
                let body = text(&request, "body").unwrap_or_default().to_owned();
                let request = http::Request {
                    method: method.clone(),
                    path: path.clone(),
                    query: text(&request, "query").unwrap_or_default().to_owned(),
                    headers: vec![("content-type".to_owned(), "application/json".to_owned())],
                    body: body.into_bytes(),
                };
                let answered = surface::dispatch(&mut system, &request);
                format!(
                    "{{\"status\":{},\"body\":{}}}",
                    answered.status, answered.body
                )
            }
            _ => panic!("no operation in `{line}`"),
        };
        writeln!(stdout, "{answer}").expect("the test reads lines");
        stdout.flush().expect("the test reads lines");
    }
}
