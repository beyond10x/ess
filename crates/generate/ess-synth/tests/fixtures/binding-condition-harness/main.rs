//! The context port for the behaviours synthesized from
//! `ess-conformance/tests/fixtures/binding-condition.yaml` and `binding-condition-selected.yaml`
//! (each with one component added), and the line
//! protocol `tests/binding_condition_runtime.rs` drives the generated system through.
//!
//! No command behaviour and no binding is written here. Every command reaches
//! `demo_types::behaviour::Generated`, and every binding is delivered by `demo_system::System`'s
//! pump, condition included: both are what the generator wrote. This file supplies only which
//! external branch the order log takes, and renders what the system recorded with the generated
//! server's own `wire` encoders.
//!
//! One JSON object per line each way:
//!
//! - `{"op":"reset"}` opens an empty scenario;
//! - `{"op":"force","command":…,"outcome":…}` makes the next decision of that external branch
//!   take it, once;
//! - `{"op":"command","command":…,"input":{…}}` runs one command through its component's port and
//!   pumps, answering `{"outcome":…,"unmet":…}`: the outcome as `wire` writes it, and the first
//!   unmet obligation the pump reported, or `null`;
//! - `{"op":"pump"}` pumps again, answering `{"unmet":…}`;
//! - `{"op":"redeliver","event":…}` redelivers the most recent occurrence of that event on the
//!   log, answering `{"unmet":…}`;
//! - `{"op":"observe"}` answers `{"published":[…],"invocations":[…]}`: the whole log and every
//!   invocation a binding made, in order.

use std::cell::RefCell;
use std::io::{BufRead, Write};
use std::rc::Rc;

use demo_server::{json, wire};
use demo_system::{BindingInvocation, SystemEvent};
use demo_types::behaviour::{Context, ExternalCommand, Generated};

/// The external branches forced and not yet taken.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Vec<(String, String)>>>);

impl Context for Ports {
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        let mut forced = self.0.borrow_mut();
        let taken = forced
            .iter()
            .position(|(name, branch)| name == command.name() && branch == outcome);
        taken.map(|at| forced.remove(at)).is_some()
    }
}

type System = demo_system::System<Generated<Ports>>;

fn assemble(ports: &Ports) -> System {
    demo_system::System::new(messages_service::MessagesService::new(Generated::new(
        ports.clone(),
    )))
}

fn text<'v>(value: &'v json::Value, name: &str) -> &'v str {
    match value.member(name) {
        Some(json::Value::Text(text)) => text,
        _ => "",
    }
}

/// The pump's answer as the protocol writes it: the unmet obligation and the binding that reported
/// it, or `null`. `demo_system::harness_unmet` reads the pump's own failure type, which is
/// `TransportFailure` where a binding selects occurrences; the test adds it beside `fault`.
fn unmet(result: Result<(), demo_system::HarnessFailure>) -> String {
    match result.err().as_ref().and_then(demo_system::harness_unmet) {
        None => "null".to_owned(),
        Some((capability, source)) => {
            let mut out = String::from("{");
            json::member(&mut out, "capability");
            json::push_text(&mut out, capability);
            json::member(&mut out, "source");
            json::push_text(&mut out, source);
            out.push('}');
            out
        }
    }
}

/// Runs one command through its port, then pumps.
fn command(system: &mut System, name: &str, input: &json::Value) -> Result<String, String> {
    let mut outcome = String::new();
    let ran = match name {
        "demo.messages.ReceiveMessage" => {
            let input = wire::decode_command_demo_messages_receive_message(input, "input")
                .map_err(|error| error.to_string())?;
            system.messages_service.receive_message(input).map(|answer| {
                wire::encode_outcome_demo_messages_receive_message(&answer, &mut outcome);
            })
        }
        "demo.messages.MessageEvent" => {
            let input = wire::decode_command_demo_messages_message_event(input, "input")
                .map_err(|error| error.to_string())?;
            system.messages_service.message_event(input).map(|answer| {
                wire::encode_outcome_demo_messages_message_event(&answer, &mut outcome);
            })
        }
        "demo.messages.LogMessage" => {
            let input = wire::decode_command_demo_messages_log_message(input, "input")
                .map_err(|error| error.to_string())?;
            system.messages_service.log_message(input).map(|answer| {
                wire::encode_outcome_demo_messages_log_message(&answer, &mut outcome);
            })
        }
        other => return Err(format!("no command `{other}`")),
    };
    if let Err(refused) = ran {
        return Err(format!("the port refused: {refused}"));
    }
    let pumped = unmet(system.pump());
    Ok(format!("{{\"outcome\":{outcome},\"unmet\":{pumped}}}"))
}

fn observe(system: &System) -> String {
    let published: Vec<String> = system
        .published()
        .iter()
        .map(wire::encode_system_event)
        .collect();
    let invocations: Vec<String> = system
        .invocations()
        .iter()
        .map(|invocation| {
            let mut input = String::new();
            let (binding, command) = match invocation {
                BindingInvocation::Received(passed) => {
                    wire::encode_command_demo_messages_message_event(passed, &mut input);
                    ("received", "demo.messages.MessageEvent")
                }
                BindingInvocation::Logged(passed) => {
                    wire::encode_command_demo_messages_log_message(passed, &mut input);
                    ("logged", "demo.messages.LogMessage")
                }
            };
            format!("{{\"binding\":\"{binding}\",\"command\":\"{command}\",\"input\":{input}}}")
        })
        .collect();
    format!(
        "{{\"published\":[{}],\"invocations\":[{}]}}",
        published.join(","),
        invocations.join(",")
    )
}

fn main() {
    let ports = Ports::default();
    let mut system = assemble(&ports);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("the test writes lines");
        let request = json::parse(&line).expect("the test writes JSON");
        let answer = match text(&request, "op") {
            "reset" => {
                ports.0.borrow_mut().clear();
                system = assemble(&ports);
                Ok("{\"ok\":true}".to_owned())
            }
            "force" => {
                ports.0.borrow_mut().push((
                    text(&request, "command").to_owned(),
                    text(&request, "outcome").to_owned(),
                ));
                Ok("{\"ok\":true}".to_owned())
            }
            "command" => match request.member("input") {
                Some(input) => command(&mut system, text(&request, "command"), input),
                None => Err("no input".to_owned()),
            },
            "pump" => Ok(format!("{{\"unmet\":{}}}", unmet(system.pump()))),
            "redeliver" => {
                let name = text(&request, "event");
                let latest: Option<SystemEvent> = system
                    .published()
                    .iter()
                    .rev()
                    .find(|event| event.name() == name)
                    .cloned();
                match latest {
                    Some(event) => Ok(format!("{{\"unmet\":{}}}", unmet(system.redeliver(&event)))),
                    None => Err(format!("no `{name}` on the log to redeliver")),
                }
            }
            "observe" => Ok(observe(&system)),
            other => Err(format!("no operation `{other}`")),
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
