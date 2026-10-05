//! In-memory ports for the Rust system synthesized from
//! `crates/verify/ess-conformance/tests/fixtures/typed-text-operands.yaml`, and the line protocol
//! `tests/typed_text_operands.rs` drives it through.
//!
//! No view query and no command behaviour is written here: every command and view reaches
//! `directory_types::behaviour::Generated` through the generated server's in-process dispatcher —
//! the route a socket reaches, query string and all, so a view's parameters are decoded by the
//! generated query decoder. This file supplies only what the specification leaves to the
//! implementor: where a contact is stored, in which order the store lists them, and which
//! identities are minted.
//!
//! The protocol is one JSON object per line each way:
//!
//! - `{"op":"reset"}` opens an empty scenario and answers `{"status":0,"body":{}}`;
//! - `{"op":"command","command":…,"body":…}` runs one command with the request body `body`, its
//!   input as JSON text, and answers the served status and body;
//! - `{"op":"view","view":…,"query":…}` reads one view with the query string `query` and answers
//!   the served status and body.
//!
//! Its arguments are the route table, as `name method path` triples.

use std::cell::RefCell;
use std::io::{BufRead, Write};
use std::rc::Rc;

use directory_server::directory_service as surface;
use directory_server::{http, json};
use directory_types::behaviour::{ContactStorage, Context, Generated};
use directory_types::people;
use directory_types::primitives::Uuid;

/// One scenario's state: the contacts in the order they were first stored.
#[derive(Default)]
struct Shared {
    contacts: Vec<people::ContactSnapshot>,
    minted: u64,
}

/// The ports, over one shared scenario.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl ContactStorage for Ports {
    fn get(&self, identity: &people::ContactId) -> Option<people::ContactSnapshot> {
        self.0
            .borrow()
            .contacts
            .iter()
            .find(|held| held.data.contact_id == *identity)
            .cloned()
    }

    fn put(&mut self, snapshot: people::ContactSnapshot) {
        let mut shared = self.0.borrow_mut();
        match shared
            .contacts
            .iter_mut()
            .find(|held| held.data.contact_id == snapshot.data.contact_id)
        {
            Some(held) => *held = snapshot,
            None => shared.contacts.push(snapshot),
        }
    }

    fn delete(&mut self, identity: &people::ContactId) {
        self.0
            .borrow_mut()
            .contacts
            .retain(|held| held.data.contact_id != *identity);
    }

    fn list(&self) -> Vec<people::ContactSnapshot> {
        self.0.borrow().contacts.clone()
    }
}

impl Context for Ports {
    fn generate_directory_people_contact_id(&mut self) -> people::ContactId {
        let mut shared = self.0.borrow_mut();
        shared.minted += 1;
        people::ContactId(Uuid(format!(
            "00000000-0000-4000-8000-{:012}",
            shared.minted
        )))
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
    let mut system = directory_system::System::new(
        directory_service::DirectoryService::new(Generated::new(ports.clone())),
    );
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
