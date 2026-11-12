//! In-memory ports for the Rust system synthesized from the workbench report's examples 25 and 26
//! (`.engineering/repro/workbench-2026-11-09/examples/`), and the line protocol
//! `tests/generated_view_queries.rs` drives them through.
//!
//! No view query and no command behaviour is written here: every command and view reaches
//! `gap_types::behaviour::Generated` through the generated server's in-process dispatcher — the
//! route a socket reaches, query string and all, so a view's parameters are decoded by the
//! generated query decoder. This file supplies only what the specification leaves to the
//! implementor: where an item and a parent are stored, and in which order the store lists them.
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

use gap_server::gap_service as surface;
use gap_server::{http, json};
use gap_types::actor::{Actor, Caller};
use gap_types::behaviour::{Generated, ItemStorage, ParentStorage};
use gap_types::demo;

/// One scenario's state: the items and the parents in the order they were first stored.
#[derive(Default)]
struct Shared {
    items: Vec<demo::ItemSnapshot>,
    parents: Vec<demo::ParentSnapshot>,
}

/// The ports, over one shared scenario.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl ItemStorage for Ports {
    fn get(&self, identity: &String) -> Option<demo::ItemSnapshot> {
        self.0
            .borrow()
            .items
            .iter()
            .find(|held| held.data.id == *identity)
            .cloned()
    }

    fn put(&mut self, snapshot: demo::ItemSnapshot) {
        let mut shared = self.0.borrow_mut();
        match shared
            .items
            .iter_mut()
            .find(|held| held.data.id == snapshot.data.id)
        {
            Some(held) => *held = snapshot,
            None => shared.items.push(snapshot),
        }
    }

    fn delete(&mut self, identity: &String) {
        self.0
            .borrow_mut()
            .items
            .retain(|held| held.data.id != *identity);
    }

    fn list(&self) -> Vec<demo::ItemSnapshot> {
        self.0.borrow().items.clone()
    }
}

impl ParentStorage for Ports {
    fn get(&self, identity: &String) -> Option<demo::ParentSnapshot> {
        self.0
            .borrow()
            .parents
            .iter()
            .find(|held| held.data.id == *identity)
            .cloned()
    }

    fn put(&mut self, snapshot: demo::ParentSnapshot) {
        let mut shared = self.0.borrow_mut();
        match shared
            .parents
            .iter_mut()
            .find(|held| held.data.id == snapshot.data.id)
        {
            Some(held) => *held = snapshot,
            None => shared.parents.push(snapshot),
        }
    }

    fn delete(&mut self, identity: &String) {
        self.0
            .borrow_mut()
            .parents
            .retain(|held| held.data.id != *identity);
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
    let anyone = Caller {
        actor: Actor::Anyone,
    };
    let mut system =
        gap_system::System::new(gap_service::GapService::new(Generated::new(ports.clone())));
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
                let caller = match request.member("actor") {
                    Some(json::Value::Text(actor)) if actor == "gap.demo.Anyone" => Some(&anyone),
                    _ => None,
                };
                let request = http::Request {
                    method: method.clone(),
                    path: path.clone(),
                    query: text(&request, "query").unwrap_or_default().to_owned(),
                    headers: vec![("content-type".to_owned(), "application/json".to_owned())],
                    body: text(&request, "body").unwrap_or_default().as_bytes().to_vec(),
                };
                let answered = surface::dispatch(&mut system, caller, &request);
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
