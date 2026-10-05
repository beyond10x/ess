//! In-memory ports for the query synthesized from `tests/fixtures/conditional-measures-generated.yaml`
//! (beyond10x/ess#363), and the line protocol `tests/generated_view_queries.rs` drives them through.
//!
//! No view query and no command behaviour is written here. Every view reaches
//! `ledger_types::behaviour::Generated`, which the generator wrote; this file supplies only what the
//! specification leaves to the implementor: where a task is stored (and in which order the store
//! lists them) and which identities are minted.
//!
//! The protocol is one JSON object per line each way:
//!
//! - `{"op":"reset"}` opens an empty scenario;
//! - `{"op":"command","command":…,"input":…}` runs one command and answers the outcome as the
//!   generated server's `wire` module writes it, or `{"undeclared":…}` where the behaviour answered
//!   no declared outcome;
//! - `{"op":"view","view":…}` answers `{"rows":[…]}`.

use std::cell::RefCell;
use std::io::{BufRead, Write};
use std::rc::Rc;

use ledger_server::{json, wire};
use ledger_types::behaviour::{Context, Generated, TaskStorage};
use ledger_types::primitives::Uuid;
use ledger_types::work;

/// One scenario's state: the tasks in the order they were first stored.
#[derive(Default)]
struct Shared {
    tasks: Vec<work::TaskSnapshot>,
    minted: u64,
}

/// The ports, over one shared scenario.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl TaskStorage for Ports {
    fn get(&self, identity: &work::TaskId) -> Option<work::TaskSnapshot> {
        self.0
            .borrow()
            .tasks
            .iter()
            .find(|held| held.data.task_id == *identity)
            .cloned()
    }

    fn put(&mut self, snapshot: work::TaskSnapshot) {
        let mut shared = self.0.borrow_mut();
        match shared
            .tasks
            .iter_mut()
            .find(|held| held.data.task_id == snapshot.data.task_id)
        {
            Some(held) => *held = snapshot,
            None => shared.tasks.push(snapshot),
        }
    }

    fn delete(&mut self, identity: &work::TaskId) {
        self.0
            .borrow_mut()
            .tasks
            .retain(|held| held.data.task_id != *identity);
    }

    fn list(&self) -> Vec<work::TaskSnapshot> {
        self.0.borrow().tasks.clone()
    }
}

impl Context for Ports {
    fn generate_ledger_work_task_id(&mut self) -> work::TaskId {
        let mut shared = self.0.borrow_mut();
        shared.minted += 1;
        work::TaskId(Uuid(format!("00000000-0000-4000-8000-{:012}", shared.minted)))
    }
}

type Service = ledger_service::LedgerService<Generated<Ports>>;

/// Runs one command through the generated component, answering its outcome as JSON.
fn command(service: &mut Service, name: &str, input: &json::Value) -> Result<String, String> {
    let mut out = String::new();
    macro_rules! run {
        ($decode:ident, $method:ident, $encode:ident) => {{
            let input = wire::$decode(input, "input").map_err(|error| error.to_string())?;
            match service.$method(input) {
                Ok(outcome) => wire::$encode(&outcome, &mut out),
                Err(unmet) => {
                    out.push_str("{\"undeclared\":");
                    json::push_text(&mut out, &unmet.to_string());
                    out.push('}');
                }
            }
        }};
    }
    match name {
        "ledger.work.CreateTask" => run!(
            decode_command_ledger_work_create_task,
            create_task,
            encode_outcome_ledger_work_create_task
        ),
        "ledger.work.FinishTask" => run!(
            decode_command_ledger_work_finish_task,
            finish_task,
            encode_outcome_ledger_work_finish_task
        ),
        other => return Err(format!("no command `{other}`")),
    }
    Ok(out)
}

/// Reads one view through the generated component.
fn view(service: &Service, name: &str) -> Result<String, String> {
    let mut out = String::from("{\"rows\":[");
    macro_rules! read {
        ($method:ident, $encode:ident) => {{
            let rows = service.$method().map_err(|unmet| unmet.to_string())?;
            for (position, row) in rows.iter().enumerate() {
                if position > 0 {
                    out.push(',');
                }
                wire::$encode(row, &mut out);
            }
        }};
    }
    match name {
        "ledger.work.Scorecard" => read!(scorecard, encode_view_ledger_work_scorecard),
        other => return Err(format!("no view `{other}`")),
    }
    out.push_str("]}");
    Ok(out)
}

fn text<'v>(value: &'v json::Value, name: &str) -> Option<&'v str> {
    match value.member(name) {
        Some(json::Value::Text(text)) => Some(text),
        _ => None,
    }
}

fn main() {
    let ports = Ports::default();
    let mut service: Service = ledger_service::LedgerService::new(Generated::new(ports.clone()));
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("the test writes lines");
        let request = json::parse(&line).expect("the test writes JSON");
        let answer = match text(&request, "op") {
            Some("reset") => {
                *ports.0.borrow_mut() = Shared::default();
                Ok("{\"ok\":true}".to_owned())
            }
            Some("command") => {
                let name = text(&request, "command").unwrap_or_default().to_owned();
                let input = request.member("input").cloned().unwrap_or(json::Value::Null);
                command(&mut service, &name, &input)
            }
            Some("view") => view(&service, text(&request, "view").unwrap_or_default()),
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
