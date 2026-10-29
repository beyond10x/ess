//! In-memory ports for the Rust system `tests/integer_spellings.rs` synthesizes from its `demo`
//! model, and the line protocol that test drives it through.
//!
//! No view query and no command behaviour is written here: every command and view reaches
//! `demo_types::behaviour::Generated` through the generated server's in-process dispatcher, so a
//! request body is read by the generated JSON decoder. This file supplies only where a board is
//! stored and which identities are minted.
//!
//! The protocol is one JSON object per line each way:
//!
//! - `{"op":"reset"}` opens an empty scenario and answers `{"status":0,"body":{}}`;
//! - `{"op":"command","command":…,"actor":…,"body":…}` runs one command as the declared actor
//!   `actor` (or as no one) with the request body `body`, its input as JSON text, and answers the
//!   served status and body;
//! - `{"op":"view","view":…,"query":…}` reads one view and answers the served status and body.
//!
//! Its arguments are the route table, as `name method path` triples.

use std::cell::RefCell;
use std::io::{BufRead, Write};
use std::rc::Rc;

use demo_server::board_service as surface;
use demo_server::{http, json};
use demo_types::actor::{Actor, Caller};
use demo_types::behaviour::{BoardStorage, Context, Generated};
use demo_types::board;
use demo_types::primitives::Uuid;

/// One scenario's state: the boards in the order they were first stored.
#[derive(Default)]
struct Shared {
    boards: Vec<board::BoardSnapshot>,
    minted: u64,
}

/// The ports, over one shared scenario.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl BoardStorage for Ports {
    fn get(&self, identity: &board::BoardId) -> Option<board::BoardSnapshot> {
        self.0
            .borrow()
            .boards
            .iter()
            .find(|held| held.data.board_id == *identity)
            .cloned()
    }

    fn put(&mut self, snapshot: board::BoardSnapshot) {
        let mut shared = self.0.borrow_mut();
        match shared
            .boards
            .iter_mut()
            .find(|held| held.data.board_id == snapshot.data.board_id)
        {
            Some(held) => *held = snapshot,
            None => shared.boards.push(snapshot),
        }
    }

    fn delete(&mut self, identity: &board::BoardId) {
        self.0
            .borrow_mut()
            .boards
            .retain(|held| held.data.board_id != *identity);
    }

    fn list(&self) -> Vec<board::BoardSnapshot> {
        self.0.borrow().boards.clone()
    }
}

impl Context for Ports {
    fn generate_demo_board_board_id(&mut self) -> board::BoardId {
        let mut shared = self.0.borrow_mut();
        shared.minted += 1;
        board::BoardId(Uuid(format!(
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
    let mut system = demo_system::System::new(board_service::BoardService::new(Generated::new(
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
                let caller = text(&request, "actor").and_then(|actor| {
                    Actor::ALL
                        .iter()
                        .find(|declared| declared.name() == actor)
                        .map(|declared| Caller { actor: *declared })
                });
                let request = http::Request {
                    method: method.clone(),
                    path: path.clone(),
                    query: text(&request, "query").unwrap_or_default().to_owned(),
                    headers: vec![("content-type".to_owned(), "application/json".to_owned())],
                    body: body.into_bytes(),
                };
                let answered = surface::dispatch(&mut system, caller.as_ref(), &request);
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
