//! Scripted command ports for the system synthesized from `model.yaml.in` beside this file, and
//! the line protocol `tests/refusal_policy_runtime.rs` drives it through.
//!
//! No binding and no failure policy is written here. Every binding is delivered by
//! `demo_system::System`'s pump, its policy selected by the generated dispatch: this file supplies
//! only what the command ports answer, the escalation's payload, and an account of what happened.
//!
//! One occurrence per line in: `place <order> <answer>…` runs `Place`, whose `OrderPlaced` the
//! `notify-ledger` binding delivers to `Record`; `close <order> <answer>…` runs `Close`, whose
//! `OrderClosed` the `archive-order` binding delivers to `Archive`; `audit <order> <answer>…` runs
//! `Audit`, whose `OrderAudited` the `note-order` binding delivers to `Note` through its owed
//! transformation, which fails for an order whose id starts with `noinput`. The answers are what
//! the port gives that order's attempts, in order: an outcome name of the command, or `unmet` for
//! an unmet obligation. Past the script the port accepts.
//!
//! The harness pumps until a pump makes no attempt and asks no transformation (at most ten pumps),
//! then prints, per attempt, `attempt <command> <order> <n> <answer> <policy> <published>`: the
//! attempt's number for that order, the answer it got, the policy the system applied — `accepted`,
//! `unmet` (the pump that made it failed), `escalate` (that pump published the escalation),
//! `retry` (a later pump attempted the order again) or `drop` — and the events that pump published
//! besides the triggering one, comma-separated, or `-`. For `audit` it then prints
//! `inputs <order> <asked> reported <failed>`: how often the transformation was asked for that
//! order's input, and how many pumps failed. Then `end <order>`.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, Write};
use std::rc::Rc;

use demo_system::SystemEvent;
use demo_types::ledger;
use demo_types::obligation::UnmetObligation;

const RECORD: &str = "demo.ledger.Record";
const ARCHIVE: &str = "demo.ledger.Archive";
const ESCALATED: &str = "demo.ledger.RecordEscalated";
const NOTE: &str = "demo.ledger.Note";

/// One call of a scripted command port.
struct Call {
    command: &'static str,
    order: String,
    answer: String,
}

/// The scripts not yet answered, every call made and every order whose `Note` input the owed
/// transformation was asked for, shared by the ports, the obligations and the harness.
#[derive(Default)]
struct Shared {
    scripts: BTreeMap<(&'static str, String), VecDeque<String>>,
    calls: Vec<Call>,
    asks: Vec<String>,
}

/// The behaviours of `ledger-service`: `Place`, `Close` and `Audit` accept, `Record`, `Archive`
/// and `Note` answer from their script.
#[derive(Clone, Default)]
struct Ports(Rc<RefCell<Shared>>);

impl Ports {
    /// The next scripted answer of `command` for `order`, recorded as a call.
    fn answer(&self, command: &'static str, order: &ledger::OrderId, accepted: &str) -> String {
        let mut shared = self.0.borrow_mut();
        let answer = shared
            .scripts
            .get_mut(&(command, order.0.clone()))
            .and_then(VecDeque::pop_front)
            .unwrap_or_else(|| accepted.to_owned());
        shared.calls.push(Call {
            command,
            order: order.0.clone(),
            answer: answer.clone(),
        });
        answer
    }
}

fn unmet(source: &'static str) -> UnmetObligation {
    UnmetObligation {
        capability: "scripted",
        source,
    }
}

impl ledger::obligations::PlaceBehavior for Ports {
    fn place(&mut self, input: ledger::Place) -> Result<ledger::PlaceOutcome, UnmetObligation> {
        Ok(ledger::PlaceOutcome::Placed {
            order_placed: ledger::OrderPlaced {
                order_id: input.order_id,
            },
        })
    }
}

impl ledger::obligations::CloseBehavior for Ports {
    fn close(&mut self, input: ledger::Close) -> Result<ledger::CloseOutcome, UnmetObligation> {
        Ok(ledger::CloseOutcome::Closed {
            order_closed: ledger::OrderClosed {
                order_id: input.order_id,
            },
        })
    }
}

impl ledger::obligations::RecordBehavior for Ports {
    fn record(&mut self, input: ledger::Record) -> Result<ledger::RecordOutcome, UnmetObligation> {
        Ok(
            match self.answer(RECORD, &input.order_id, "recorded").as_str() {
                "recorded" => ledger::RecordOutcome::Recorded {
                    recorded: ledger::Recorded {
                        order_id: input.order_id,
                    },
                },
                "unavailable" => ledger::RecordOutcome::Unavailable {
                    error: ledger::Unavailable,
                },
                "busy" => ledger::RecordOutcome::Busy {
                    error: ledger::Unavailable,
                },
                "rejected" => ledger::RecordOutcome::Rejected {
                    error: ledger::Unknown,
                },
                "at-limit" => ledger::RecordOutcome::AtLimit {
                    error: ledger::AtLimit,
                },
                "wrong-state" => ledger::RecordOutcome::WrongState {
                    error: ledger::WrongState,
                },
                "unmet" => return Err(unmet(RECORD)),
                other => panic!("`{RECORD}` has no answer `{other}`"),
            },
        )
    }
}

impl ledger::obligations::ArchiveBehavior for Ports {
    fn archive(
        &mut self,
        input: ledger::Archive,
    ) -> Result<ledger::ArchiveOutcome, UnmetObligation> {
        Ok(
            match self.answer(ARCHIVE, &input.order_id, "archived").as_str() {
                "archived" => ledger::ArchiveOutcome::Archived {
                    archived: ledger::Archived {
                        order_id: input.order_id,
                    },
                },
                "unavailable" => ledger::ArchiveOutcome::Unavailable {
                    error: ledger::Unavailable,
                },
                "rejected" => ledger::ArchiveOutcome::Rejected {
                    error: ledger::Unknown,
                },
                "unmet" => return Err(unmet(ARCHIVE)),
                other => panic!("`{ARCHIVE}` has no answer `{other}`"),
            },
        )
    }
}

impl ledger::obligations::AuditBehavior for Ports {
    fn audit(&mut self, input: ledger::Audit) -> Result<ledger::AuditOutcome, UnmetObligation> {
        Ok(ledger::AuditOutcome::Audited {
            order_audited: ledger::OrderAudited {
                order_id: input.order_id,
            },
        })
    }
}

impl ledger::obligations::NoteBehavior for Ports {
    fn note(&mut self, input: ledger::Note) -> Result<ledger::NoteOutcome, UnmetObligation> {
        Ok(match self.answer(NOTE, &input.order_id, "noted").as_str() {
            "noted" => ledger::NoteOutcome::Noted {
                noted: ledger::Noted {
                    order_id: input.order_id,
                },
            },
            "unavailable" => ledger::NoteOutcome::Unavailable {
                error: ledger::Unavailable,
            },
            "unmet" => return Err(unmet(NOTE)),
            other => panic!("`{NOTE}` has no answer `{other}`"),
        })
    }
}

/// The system's obligations: the escalation and the owed transformation, each recording nothing
/// but what it was asked.
///
/// For an order whose id starts with `nobuild` the escalation builder fails (adversary pass 1,
/// `adversary_rust_refusal_policy_pass1.rs`): the design page's escalation-builder failure. For an
/// order whose id starts with `noinput` the transformation fails: a failure before the port is
/// invoked.
struct Escalations(Ports);

impl demo_system::obligations::NoteOrderTransformation for Escalations {
    fn note_order_input(
        &self,
        event: &ledger::OrderAudited,
    ) -> Result<ledger::Note, UnmetObligation> {
        self.0 .0.borrow_mut().asks.push(event.order_id.0.clone());
        if event.order_id.0.starts_with("noinput") {
            return Err(unmet("note-order"));
        }
        Ok(ledger::Note {
            order_id: event.order_id.clone(),
            entry: ledger::EntryRef(1),
        })
    }
}

impl demo_system::obligations::NotifyLedgerEscalation for Escalations {
    fn notify_ledger_escalation(
        &self,
        failed: &ledger::Record,
    ) -> Result<ledger::RecordEscalated, UnmetObligation> {
        if failed.order_id.0.starts_with("nobuild") {
            return Err(unmet(ESCALATED));
        }
        Ok(ledger::RecordEscalated {
            order_id: failed.order_id.clone(),
        })
    }
}

type System = demo_system::System<Ports, Escalations>;

/// What one pump did: the calls it made, the orders whose input it asked the transformation for,
/// the events it published besides the triggers, and whether it failed.
struct Pumped {
    calls: Vec<Call>,
    asks: Vec<String>,
    published: Vec<&'static str>,
    failed: bool,
}

/// Pumps until a pump makes no call and asks no transformation, at most ten times.
fn pump_until_quiet(system: &mut System, ports: &Ports) -> Vec<Pumped> {
    let mut pumps = Vec::new();
    for _ in 0..10 {
        let logged = system.published().len();
        let failed = system.pump().is_err();
        let calls = core::mem::take(&mut ports.0.borrow_mut().calls);
        let asks = core::mem::take(&mut ports.0.borrow_mut().asks);
        let published = system.published()[logged..]
            .iter()
            .filter(|event| {
                !matches!(
                    event,
                    SystemEvent::OrderPlaced(_)
                        | SystemEvent::OrderClosed(_)
                        | SystemEvent::OrderAudited(_)
                )
            })
            .map(SystemEvent::name)
            .collect();
        if calls.is_empty() && asks.is_empty() {
            break;
        }
        pumps.push(Pumped {
            calls,
            asks,
            published,
            failed,
        });
    }
    pumps
}

/// Runs one occurrence and prints what happened to it.
fn occurrence(system: &mut System, ports: &Ports, line: &str, out: &mut impl Write) {
    let mut words = line.split_whitespace();
    let operation = words.next().expect("an operation");
    let order = words.next().expect("an order").to_owned();
    let command = match operation {
        "place" => RECORD,
        "close" => ARCHIVE,
        "audit" => NOTE,
        other => panic!("no operation `{other}`"),
    };
    ports
        .0
        .borrow_mut()
        .scripts
        .insert((command, order.clone()), words.map(str::to_owned).collect());
    let order_id = ledger::OrderId(order.clone());
    let ran = match command {
        RECORD => system
            .ledger_service
            .place(ledger::Place { order_id })
            .map(drop),
        ARCHIVE => system
            .ledger_service
            .close(ledger::Close { order_id })
            .map(drop),
        _ => system
            .ledger_service
            .audit(ledger::Audit { order_id })
            .map(drop),
    };
    ran.expect("the triggering command accepts");
    let pumps = pump_until_quiet(system, ports);
    let attempts: Vec<(&Call, &Pumped)> = pumps
        .iter()
        .flat_map(|pumped| pumped.calls.iter().map(move |call| (call, pumped)))
        .filter(|(call, _)| call.command == command && call.order == order)
        .collect();
    for (at, (call, pumped)) in attempts.iter().enumerate() {
        let policy = if pumped.failed {
            "unmet"
        } else if matches!(call.answer.as_str(), "recorded" | "archived" | "noted") {
            "accepted"
        } else if pumped.published.contains(&ESCALATED) {
            "escalate"
        } else if at + 1 < attempts.len() {
            "retry"
        } else {
            "drop"
        };
        let published = if pumped.published.is_empty() {
            "-".to_owned()
        } else {
            pumped.published.join(",")
        };
        writeln!(
            out,
            "attempt {} {order} {} {} {policy} {published}",
            call.command,
            at + 1,
            call.answer
        )
        .expect("the test reads lines");
    }
    if command == NOTE {
        let asked = pumps
            .iter()
            .flat_map(|pumped| &pumped.asks)
            .filter(|asked| **asked == order)
            .count();
        let failed = pumps.iter().filter(|pumped| pumped.failed).count();
        writeln!(out, "inputs {order} {asked} reported {failed}").expect("the test reads lines");
    }
    writeln!(out, "end {order}").expect("the test reads lines");
}

fn main() {
    let ports = Ports::default();
    let mut system: System = demo_system::System::new(
        ledger_service::LedgerService::new(ports.clone()),
        Escalations(ports.clone()),
    );
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("the test writes lines");
        if line.trim().is_empty() {
            continue;
        }
        occurrence(&mut system, &ports, &line, &mut stdout);
        stdout.flush().expect("the test reads lines");
    }
}
