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
//!   naming the external branch the provider takes on this invocation, and answers
//!   `{"answer":…,"log":[…]}`: the outcome as the generated server's `wire` module writes it, or
//!   `{"undeclared":…}` where the behaviour answered no declared outcome, and every event the
//!   system then pumped and took off its log (`take_published`), each named by
//!   `SystemEvent::name` and encoded by `wire::encode_system_event`;
//! - `{"op":"view","view":…}` answers `{"rows":[…]}`;
//! - `{"op":"headers"}` sends one request over a real socket to the generated `http::read` and
//!   answers the headers it read, as `{"headers":[[name,value],…]}`.
//!
//! Its first argument chooses the path a command takes. `port` calls the component's port and
//! encodes the outcome with the generated `wire::encode_outcome_*`. `served`, followed by the route
//! table as `name method path` triples, hands the generated HTTP dispatcher a request and answers
//! `{"status":…,"answer":<body>,"kept":…}`: the served body verbatim, and how many events the
//! dispatch left on the system's log and in the component's outbox — which must be none, or a
//! long-running server grows with every request.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::rc::Rc;

use desk_server::desk_service as served;
use desk_server::{http, json, wire};
use desk_types::actor::{Actor, Caller};
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

type System = desk_system::System<Generated<Ports>>;

/// A fresh system over the ports.
fn assemble(ports: &Ports) -> System {
    desk_system::System::new(desk_service::DeskService::new(Generated::new(ports.clone())))
}

/// Runs one command or view through the generated HTTP dispatcher, answering the status, the
/// served body and what the system's log gained.
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
        headers: vec![
            ("authorization".to_owned(), "Bearer harness".to_owned()),
            ("content-type".to_owned(), http::JSON.to_owned()),
        ],
        body: if method == "GET" {
            Vec::new()
        } else {
            input.as_bytes().to_vec()
        },
    };
    let answered = served::dispatch(system, caller, &request);
    // What the served dispatch left behind: the log, and the component's outbox. Draining the
    // outbox here would hide a leak rather than measure one, but an outbox the dispatch left
    // non-empty is counted before it is dropped.
    let kept = system.published().len() + system.desk_service.drain_outbox().len();
    Ok(format!(
        "{{\"status\":{},\"answer\":{},\"kept\":{kept}}}",
        answered.status, answered.body
    ))
}

/// Every event the system's log holds once pumped, taken off it, as `[{"name":…,"encoded":…}]`.
fn taken(system: &mut System) -> Result<String, String> {
    system.pump().map_err(|unmet| unmet.to_string())?;
    let mut out = String::from("[");
    for (position, event) in system.take_published().iter().enumerate() {
        if position > 0 {
            out.push(',');
        }
        out.push_str("{\"name\":");
        json::push_text(&mut out, event.name());
        out.push_str(",\"encoded\":");
        out.push_str(&wire::encode_system_event(event));
        out.push('}');
    }
    out.push(']');
    if !system.published().is_empty() {
        return Err("`take_published` left delivered events on the log".to_owned());
    }
    Ok(out)
}

/// One request over a real socket, read by the generated `http::read`; answers its headers.
fn headers() -> Result<String, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let client = std::thread::spawn(move || {
        let mut stream = std::net::TcpStream::connect(address).expect("the listener accepts");
        stream
            .write_all(
                b"POST /tickets/commands/open-ticket?trace=1 HTTP/1.1\r\n\
                  Host: desk.example\r\n\
                  Authorization: Bearer token-1\r\n\
                  X-Request-ID:   abc-123  \r\n\
                  Content-Length: 2\r\n\
                  \r\n\
                  {}",
            )
            .expect("the request is written");
        stream
    });
    let (connection, _) = listener.accept().map_err(|error| error.to_string())?;
    let mut reader = std::io::BufReader::new(connection);
    let request = http::read(&mut reader).map_err(|refusal| refusal.body)?;
    let _ = client.join();
    let mut out = String::from("{\"method\":");
    json::push_text(&mut out, &request.method);
    out.push_str(",\"path\":");
    json::push_text(&mut out, &request.path);
    out.push_str(",\"headers\":[");
    for (position, (name, value)) in request.headers.iter().enumerate() {
        if position > 0 {
            out.push(',');
        }
        out.push('[');
        json::push_text(&mut out, name);
        out.push(',');
        json::push_text(&mut out, value);
        out.push(']');
    }
    out.push_str("],\"bounded\":");
    json::push_integer(&mut out, i64::from(too_many_headers()?));
    out.push('}');
    Ok(out)
}

/// One request carrying one header more than `http::MAX_HEADERS`; answers the refusal's status.
fn too_many_headers() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let client = std::thread::spawn(move || {
        let mut stream = std::net::TcpStream::connect(address).expect("the listener accepts");
        let mut request = String::from("GET /openapi.json HTTP/1.1\r\n");
        for position in 0..=http::MAX_HEADERS {
            request.push_str(&format!("X-Filler-{position}: {position}\r\n"));
        }
        request.push_str("\r\n");
        stream
            .write_all(request.as_bytes())
            .expect("the request is written");
        stream
    });
    let (connection, _) = listener.accept().map_err(|error| error.to_string())?;
    let mut reader = std::io::BufReader::new(connection);
    let status = match http::read(&mut reader) {
        Ok(_) => 200,
        Err(refusal) => refusal.status,
    };
    let _ = client.join();
    Ok(status)
}

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
        "desk.ticket.ForgetStats" => run!(
            decode_command_desk_ticket_forget_stats,
            forget_stats,
            encode_outcome_desk_ticket_forget_stats
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
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let via_served = arguments.first().is_some_and(|mode| mode == "served");
    let routes: Vec<(String, String, String)> = arguments
        .get(1..)
        .unwrap_or_default()
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
            Some("headers") => headers(),
            Some("command") if via_served => {
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
                let body = text(&request, "body").unwrap_or_default().to_owned();
                // Who the request was authenticated as: the step's actor, where it names one.
                let caller = text(&request, "actor").and_then(|actor| {
                    Actor::ALL
                        .iter()
                        .find(|declared| declared.name() == actor)
                        .map(|declared| Caller { actor: *declared })
                });
                let answer = serve(&mut system, &routes, caller.as_ref(), &name, &body);
                ports.0.borrow_mut().forced = None;
                answer
            }
            Some("view") if via_served => serve(
                &mut system,
                &routes,
                None,
                text(&request, "view").unwrap_or_default(),
                "",
            ),
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
                let answer = command(&mut system.desk_service, &name, &input).and_then(|answer| {
                    Ok(format!("{{\"answer\":{answer},\"log\":{}}}", taken(&mut system)?))
                });
                ports.0.borrow_mut().forced = None;
                answer
            }
            Some("view") => view(&system.desk_service, text(&request, "view").unwrap_or_default()),
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
