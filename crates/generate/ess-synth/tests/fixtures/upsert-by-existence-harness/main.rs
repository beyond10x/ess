//! In-memory storage ports for the behaviours synthesized from the upsert-by-existence model, and
//! the line protocol `tests/upsert_by_existence.rs` drives them through.
//!
//! No command behaviour is written here: every command reaches
//! `demo_types::behaviour::Generated`, which the generator wrote, so the lookup that selects a
//! branch by whether the addressed record exists is the generated one. This file supplies only
//! where an item and a slot are stored.
//!
//! One JSON object per line each way: `{"op":"reset"}`; `{"op":"command","command":…,
//! "actor":…,"body":…}`; `{"op":"view","view":…}`. Each command or view goes through the generated
//! HTTP dispatcher and answers `{"status":…,"answer":<served body>}`. The arguments are the route
//! table as `name method path` triples.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::rc::Rc;

use demo_server::items_service as served;
use demo_server::{http, json};
use demo_types::actor::{Actor, Caller};
use demo_types::behaviour::{Generated, ItemStorage, SlotStorage};
use demo_types::items;

/// One scenario's rows.
#[derive(Default)]
struct Shared {
    items: BTreeMap<String, items::ItemSnapshot>,
    slots: BTreeMap<String, items::SlotSnapshot>,
}

/// The ports, over one shared scenario.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl ItemStorage for Ports {
    fn get(&self, identity: &items::ItemId) -> Option<items::ItemSnapshot> {
        self.0.borrow().items.get(&identity.0).cloned()
    }

    fn put(&mut self, snapshot: items::ItemSnapshot) {
        let key = snapshot.data.item_id.0.clone();
        self.0.borrow_mut().items.insert(key, snapshot);
    }

    fn delete(&mut self, identity: &items::ItemId) {
        self.0.borrow_mut().items.remove(&identity.0);
    }

    fn list(&self) -> Vec<items::ItemSnapshot> {
        self.0.borrow().items.values().cloned().collect()
    }
}

impl SlotStorage for Ports {
    fn get(&self, identity: &items::ItemId) -> Option<items::SlotSnapshot> {
        self.0.borrow().slots.get(&identity.0).cloned()
    }

    fn put(&mut self, snapshot: items::SlotSnapshot) {
        let key = snapshot.data.slot_id.0.clone();
        self.0.borrow_mut().slots.insert(key, snapshot);
    }

    fn delete(&mut self, identity: &items::ItemId) {
        self.0.borrow_mut().slots.remove(&identity.0);
    }

    fn list(&self) -> Vec<items::SlotSnapshot> {
        self.0.borrow().slots.values().cloned().collect()
    }
}

type System = demo_system::System<Generated<Ports>>;

/// A fresh system over the ports.
fn assemble(ports: &Ports) -> System {
    demo_system::System::new(items_service::ItemsService::new(Generated::new(ports.clone())))
}

/// Runs one command or view through the generated HTTP dispatcher.
fn serve(
    system: &mut System,
    routes: &[(String, String, String)],
    caller: Option<&Caller>,
    name: &str,
    input: &str,
) -> Result<String, String> {
    let (_, method, path) = routes
        .iter()
        .find(|(declared, _, _)| declared == name)
        .ok_or_else(|| format!("no route for `{name}`"))?;
    let request = http::Request {
        method: method.clone(),
        path: path.clone(),
        query: String::new(),
        headers: vec![("content-type".to_owned(), http::JSON.to_owned())],
        body: if method == "GET" {
            Vec::new()
        } else {
            input.as_bytes().to_vec()
        },
    };
    let answered = served::dispatch(system, caller, &request);
    Ok(format!(
        "{{\"status\":{},\"answer\":{}}}",
        answered.status, answered.body
    ))
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
    let mut system = assemble(&ports);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("the test writes lines");
        let request = json::parse(&line).expect("the test writes JSON");
        let answer = match text(&request, "op") {
            Some("reset") => {
                *ports.0.borrow_mut() = Shared::default();
                system = assemble(&ports);
                Ok("{\"ok\":true}".to_owned())
            }
            Some("command") => {
                let name = text(&request, "command").unwrap_or_default().to_owned();
                let body = text(&request, "body").unwrap_or_default().to_owned();
                let caller = text(&request, "actor").and_then(|actor| {
                    Actor::ALL
                        .iter()
                        .find(|declared| declared.name() == actor)
                        .map(|declared| Caller { actor: *declared })
                });
                serve(&mut system, &routes, caller.as_ref(), &name, &body)
            }
            Some("view") => serve(
                &mut system,
                &routes,
                None,
                text(&request, "view").unwrap_or_default(),
                "",
            ),
            _ => Err(format!("no operation in `{line}`")),
        };
        let line = match answer {
            Ok(line) => line,
            Err(why) => {
                let mut out = String::from("{\"failure\":");
                json::push_text(&mut out, &why);
                out.push('}');
                out
            }
        };
        writeln!(stdout, "{line}").expect("the test reads lines");
        stdout.flush().expect("the test reads lines");
    }
}
