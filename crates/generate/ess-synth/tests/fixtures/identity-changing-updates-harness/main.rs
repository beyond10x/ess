//! An in-memory storage port for the behaviour synthesized from the identity-changing-updates
//! model (ess/23, beyond10x/ess#429), and the line protocol `tests/identity_changing_updates.rs`
//! drives it through.
//!
//! No command behaviour is written here: every command reaches
//! `demo_types::behaviour::Generated`, which the generator wrote, so the collision lookup and the
//! re-key — the old row removed, the new one inserted — are the generated ones. This file supplies
//! only where a secret is stored, keyed by the identity its snapshot carries.
//!
//! One JSON object per line each way: `{"op":"reset"}`; `{"op":"command","command":…,
//! "actor":…,"body":…}`; `{"op":"view","view":…}`. Each command or view goes through the generated
//! HTTP dispatcher and answers `{"status":…,"answer":<served body>}`. The arguments are the route
//! table as `name method path` triples.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::rc::Rc;

use demo_server::vault_service as served;
use demo_server::{http, json};
use demo_types::actor::{Actor, Caller};
use demo_types::behaviour::{Generated, SecretStorage};
use demo_types::vault;

/// One scenario's rows, keyed by the identity each snapshot carries.
#[derive(Default)]
struct Shared {
    secrets: BTreeMap<String, vault::SecretSnapshot>,
}

/// The port, over one shared scenario.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl SecretStorage for Ports {
    fn get(&self, identity: &vault::SecretName) -> Option<vault::SecretSnapshot> {
        self.0.borrow().secrets.get(&identity.0).cloned()
    }

    fn put(&mut self, snapshot: vault::SecretSnapshot) {
        let key = snapshot.data.name.0.clone();
        self.0.borrow_mut().secrets.insert(key, snapshot);
    }

    fn delete(&mut self, identity: &vault::SecretName) {
        self.0.borrow_mut().secrets.remove(&identity.0);
    }

    fn list(&self) -> Vec<vault::SecretSnapshot> {
        self.0.borrow().secrets.values().cloned().collect()
    }
}

type System = demo_system::System<Generated<Ports>>;

/// A fresh system over the port.
fn assemble(ports: &Ports) -> System {
    demo_system::System::new(vault_service::VaultService::new(Generated::new(
        ports.clone(),
    )))
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
