//! In-memory ports for the behaviours synthesized from `tests/fixtures/declared-behaviour/`, and the
//! line protocol `tests/declared_behaviour.rs` drives them through.
//!
//! No command behaviour is written here. Every command reaches `desk_types::behaviour::Generated`,
//! which the generator wrote; this file supplies what the specification leaves to the implementor
//! and says so: where a ticket is stored, who the caller is, which identities are minted, which
//! external branch the provider takes. The one view query is generated too, over `list`.
//!
//! The protocol is one JSON object per line each way:
//!
//! - `{"op":"reset"}` opens an empty scenario;
//! - `{"op":"command","command":…,"input":…,"caller":…,"force":…}` runs one command, `force`
//!   naming the external branch the provider takes on this invocation, and answers the outcome as
//!   the generated server's `wire` module writes it, or `{"undeclared":…}` where the behaviour
//!   answered no declared outcome;
//! - `{"op":"view","view":…}` answers `{"rows":[…]}`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::rc::Rc;

use desk_server::{json, wire};
use desk_types::behaviour::{Context, Generated, TicketStorage};
use desk_types::primitives::Uuid;
use desk_types::ticket;

/// One scenario's state.
#[derive(Default)]
struct Shared {
    tickets: BTreeMap<String, ticket::TicketSnapshot>,
    minted: u64,
    caller: Option<ticket::AgentId>,
    forced: Option<(String, String)>,
}

/// The ports, over one shared scenario.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl Ports {
    fn mint(&self) -> Uuid {
        let mut shared = self.0.borrow_mut();
        shared.minted += 1;
        Uuid(format!("00000000-0000-4000-8000-{:012}", shared.minted))
    }
}

impl TicketStorage for Ports {
    fn get(&self, identity: &ticket::TicketId) -> Option<ticket::TicketSnapshot> {
        self.0.borrow().tickets.get(&identity.0 .0).cloned()
    }

    fn put(&mut self, snapshot: ticket::TicketSnapshot) {
        let key = snapshot.data.ticket_id.0 .0.clone();
        self.0.borrow_mut().tickets.insert(key, snapshot);
    }

    fn delete(&mut self, identity: &ticket::TicketId) {
        self.0.borrow_mut().tickets.remove(&identity.0 .0);
    }

    fn list(&self) -> Vec<ticket::TicketSnapshot> {
        self.0.borrow().tickets.values().cloned().collect()
    }
}

impl Context for Ports {
    fn caller_agent_id(&self) -> Option<ticket::AgentId> {
        self.0.borrow().caller.clone()
    }

    fn generate_desk_ticket_escalation_id(&mut self) -> ticket::EscalationId {
        ticket::EscalationId(self.mint())
    }

    fn generate_desk_ticket_ticket_id(&mut self) -> ticket::TicketId {
        ticket::TicketId(self.mint())
    }

    fn generate_desk_ticket_ticket_ref(&mut self) -> ticket::TicketRef {
        ticket::TicketRef(self.mint())
    }

    fn external(&mut self, command: &'static str, outcome: &'static str) -> bool {
        self.0
            .borrow()
            .forced
            .as_ref()
            .is_some_and(|(forced_command, forced_outcome)| {
                forced_command == command && forced_outcome == outcome
            })
    }
}

type Service = desk_service::DeskService<Generated<Ports>>;

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
        "desk.ticket.OpenTicket" => run!(
            decode_command_desk_ticket_open_ticket,
            open_ticket,
            encode_outcome_desk_ticket_open_ticket
        ),
        "desk.ticket.CloseTicket" => run!(
            decode_command_desk_ticket_close_ticket,
            close_ticket,
            encode_outcome_desk_ticket_close_ticket
        ),
        "desk.ticket.ReopenTicket" => run!(
            decode_command_desk_ticket_reopen_ticket,
            reopen_ticket,
            encode_outcome_desk_ticket_reopen_ticket
        ),
        "desk.ticket.Reprioritize" => run!(
            decode_command_desk_ticket_reprioritize,
            reprioritize,
            encode_outcome_desk_ticket_reprioritize
        ),
        "desk.ticket.Comment" => run!(
            decode_command_desk_ticket_comment,
            comment,
            encode_outcome_desk_ticket_comment
        ),
        "desk.ticket.DeleteTicket" => run!(
            decode_command_desk_ticket_delete_ticket,
            delete_ticket,
            encode_outcome_desk_ticket_delete_ticket
        ),
        "desk.ticket.Escalate" => run!(
            decode_command_desk_ticket_escalate,
            escalate,
            encode_outcome_desk_ticket_escalate
        ),
        other => return Err(format!("no command `{other}`")),
    }
    Ok(out)
}

/// Reads the one view the fixture declares.
fn view(service: &Service, name: &str) -> Result<String, String> {
    if name != "desk.ticket.Tickets" {
        return Err(format!("no view `{name}`"));
    }
    let rows = service.tickets().map_err(|unmet| unmet.to_string())?;
    let mut out = String::from("{\"rows\":[");
    for (position, row) in rows.iter().enumerate() {
        if position > 0 {
            out.push(',');
        }
        wire::encode_view_desk_ticket_tickets(row, &mut out);
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
    let mut service: Service = desk_service::DeskService::new(Generated::new(ports.clone()));
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
                {
                    let mut shared = ports.0.borrow_mut();
                    shared.caller = request
                        .member("caller")
                        .and_then(|caller| text(caller, "agent_id"))
                        .map(|agent| ticket::AgentId(Uuid(agent.to_owned())));
                    shared.forced = text(&request, "force")
                        .map(|outcome| (name.clone(), outcome.to_owned()));
                }
                let input = request.member("input").cloned().unwrap_or(json::Value::Null);
                let answer = command(&mut service, &name, &input);
                ports.0.borrow_mut().forced = None;
                answer
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
