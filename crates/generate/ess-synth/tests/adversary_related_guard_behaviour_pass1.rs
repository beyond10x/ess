//! Adversary pass 1 against story:related-guard-behaviour (beyond10x/ess#319): generated Rust and
//! Go `when_related:` behaviour, driven request by request beside the conformance interpreter.
//!
//! Every model here is one the domain admits and the plan generates. Each request sequence is
//! answered by the interpreter (`ess_conformance::interpret::execute::execute`, the reference
//! semantics) and by the generated network entry point, built from the emitted tree with no
//! hand-written code; every step's outcome and error must agree. The sequences reach what the
//! synthesized suite does not: a stored reference rewritten, cleared, pointed at its own row or at
//! a cancelled row; a branch that `sets:` the reference it read; held state beside a dangling
//! reference; an Optional input omitted and sent as `null`; both declaration orders.

#![allow(dead_code, clippy::too_many_lines)]

mod related_guard_served;
use related_guard_served as related;

use std::collections::BTreeMap;
use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ess_compiler::ir::EssIr;
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_primitives::node::Node;

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const DANGLING: &str = "00000000-0000-4000-8000-777777777777";
const UNKNOWN: &str = "00000000-0000-4000-8000-888888888888";

// ---- models ----------------------------------------------------------------------------------

/// A stored, Optional reference read by `CompleteTask`, whose accepting branch clears the field it
/// read; `SetBlocker` rewrites or clears it; `CancelTask` moves a task out of `Open` without
/// reading any related row.
const STORED: &str = "format: ess/22
system: demo
version: v1
domain: demo.tasks
summary: Adversary model, stored reference.
types:
  - {name: demo.tasks.TaskId, kind: newtype, of: Uuid}
entities:
  - name: demo.tasks.Task
    identity: {name: task_id, type: demo.tasks.TaskId}
    fields:
      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}
    lifecycle:
      initial: Open
      states: [Open, Done, Cancelled]
      terminal: [Done, Cancelled]
      transitions:
        - {name: complete, from: [Open], to: Done}
        - {name: cancel, from: [Open], to: Cancelled}
errors:
  - {name: demo.tasks.BlockerMissing, summary: No task carries the stored blocker., fields: []}
  - {name: demo.tasks.Blocked, summary: The blocker is not done., fields: []}
  - {name: demo.tasks.TaskStateConflict, summary: The task cannot move., fields: []}
  - {name: demo.tasks.NoTask, summary: No task carries the identity., fields: []}
events:
  - name: demo.tasks.TaskAdded
    fields: [{name: task_id, type: demo.tasks.TaskId}]
  - name: demo.tasks.TaskCompleted
    fields: [{name: task_id, type: demo.tasks.TaskId}]
  - name: demo.tasks.TaskCancelled
    fields: [{name: task_id, type: demo.tasks.TaskId}]
  - name: demo.tasks.BlockerSet
    fields: [{name: task_id, type: demo.tasks.TaskId}]
actors:
  - name: demo.tasks.Planner
    may: [demo.tasks.AddTask, demo.tasks.CompleteTask, demo.tasks.CancelTask, demo.tasks.SetBlocker]
commands:
  - name: demo.tasks.AddTask
    input:
      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}
    outcomes:
      - name: added
        creates: demo.tasks.Task
        instance: task_id
        sets: {blocked_by: input.blocked_by}
        emits: [demo.tasks.TaskAdded]
        payload: {demo.tasks.TaskAdded: {task_id: {generated: true}}}
  - name: demo.tasks.SetBlocker
    input:
      - {name: task_id, type: demo.tasks.TaskId}
      - {name: blocker, type: Optional<demo.tasks.TaskId>}
    outcomes:
      - name: set
        updates: demo.tasks.Task
        instance: task_id
        sets: {blocked_by: input.blocker}
        emits: [demo.tasks.BlockerSet]
        payload: {demo.tasks.BlockerSet: {task_id: input.task_id}}
      - {name: no-task, unknown_instance: true, error: demo.tasks.NoTask}
  - name: demo.tasks.CancelTask
    input:
      - {name: task_id, type: demo.tasks.TaskId}
    outcomes:
      - name: cancelled
        moves: demo.tasks.Task.cancel
        instance: task_id
        emits: [demo.tasks.TaskCancelled]
        payload: {demo.tasks.TaskCancelled: {task_id: input.task_id}}
      - {name: cancel-conflict, wrong_state: true, error: demo.tasks.TaskStateConflict}
  - name: demo.tasks.CompleteTask
    input:
      - {name: task_id, type: demo.tasks.TaskId}
    outcomes:
      - name: blocker-missing
        when_related: {via: blocked_by, exists: false}
        error: demo.tasks.BlockerMissing
      - name: blocked
        when_related: {via: blocked_by, predicate: state != Done}
        error: demo.tasks.Blocked
      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}
      - name: completed
        moves: demo.tasks.Task.complete
        instance: task_id
        sets: {blocked_by: {cleared: true}}
        emits: [demo.tasks.TaskCompleted]
        payload: {demo.tasks.TaskCompleted: {task_id: input.task_id}}
views:
  - name: demo.tasks.Tasks
    source: demo.tasks.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: demo.tasks.TaskId}
      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}
      - {name: state, type: demo.tasks.Task.State}
";

/// [`STORED`] with `CompleteTask`'s branches in the other declaration order, `wrong_state:` first.
fn stored_reordered() -> String {
    let from = "      - name: blocker-missing
        when_related: {via: blocked_by, exists: false}
        error: demo.tasks.BlockerMissing
      - name: blocked
        when_related: {via: blocked_by, predicate: state != Done}
        error: demo.tasks.Blocked
      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}
";
    let to = "      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}
      - name: blocked
        when_related: {via: blocked_by, predicate: state != Done}
        error: demo.tasks.Blocked
      - name: blocker-missing
        when_related: {via: blocked_by, exists: false}
        error: demo.tasks.BlockerMissing
";
    assert!(STORED.contains(from));
    STORED.replacen(from, to, 1)
}

/// The Optional input reference of the release fixture, with an input-guarded refusal on the
/// guarded command and a `wrong_state:` on `AcceptCandidate`.
const OPTIONAL: &str = "format: ess/22
system: demo
version: v1
domain: demo.release
summary: Adversary model, Optional input reference.
types:
  - {name: demo.release.ReleaseId, kind: newtype, of: Uuid}
  - {name: demo.release.CandidateId, kind: newtype, of: Uuid}
entities:
  - name: demo.release.Candidate
    identity: {name: candidate_id, type: demo.release.CandidateId}
    fields: []
    lifecycle:
      initial: Proposed
      states: [Proposed, Accepted]
      terminal: [Accepted]
      transitions:
        - {name: accept, from: [Proposed], to: Accepted}
  - name: demo.release.Release
    identity: {name: release_id, type: demo.release.ReleaseId}
    fields: []
    lifecycle:
      initial: Draft
      states: [Draft, Published]
      terminal: [Published]
      transitions:
        - {name: publish, from: [Draft], to: Published}
errors:
  - {name: demo.release.NoCandidate, summary: No candidate carries that identity., fields: []}
  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}
  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move., fields: []}
  - {name: demo.release.CandidateStateConflict, summary: The candidate cannot move., fields: []}
  - {name: demo.release.Forced, summary: Forcing is refused., fields: []}
events:
  - name: demo.release.CandidateProposed
    fields: [{name: candidate_id, type: demo.release.CandidateId}]
  - name: demo.release.CandidateAccepted
    fields: [{name: candidate_id, type: demo.release.CandidateId}]
  - name: demo.release.ReleaseDrafted
    fields: [{name: release_id, type: demo.release.ReleaseId}]
  - name: demo.release.ReleasePublished
    fields: [{name: release_id, type: demo.release.ReleaseId}]
actors:
  - name: demo.release.Maintainer
    may:
      - demo.release.ProposeCandidate
      - demo.release.AcceptCandidate
      - demo.release.DraftRelease
      - demo.release.PublishRelease
commands:
  - name: demo.release.ProposeCandidate
    input: []
    outcomes:
      - name: proposed
        creates: demo.release.Candidate
        instance: candidate_id
        emits: [demo.release.CandidateProposed]
        payload: {demo.release.CandidateProposed: {candidate_id: {generated: true}}}
  - name: demo.release.AcceptCandidate
    input:
      - {name: candidate_id, type: demo.release.CandidateId}
    outcomes:
      - name: accepted
        moves: demo.release.Candidate.accept
        instance: candidate_id
        emits: [demo.release.CandidateAccepted]
        payload: {demo.release.CandidateAccepted: {candidate_id: input.candidate_id}}
      - {name: candidate-conflict, wrong_state: true, error: demo.release.CandidateStateConflict}
  - name: demo.release.DraftRelease
    input: []
    outcomes:
      - name: drafted
        creates: demo.release.Release
        instance: release_id
        emits: [demo.release.ReleaseDrafted]
        payload: {demo.release.ReleaseDrafted: {release_id: {generated: true}}}
  - name: demo.release.PublishRelease
    input:
      - {name: release_id, type: demo.release.ReleaseId}
      - {name: candidate, type: Optional<demo.release.CandidateId>}
      - {name: force, type: Boolean}
    outcomes:
      - name: forced
        when: force == true
        error: demo.release.Forced
      - name: no-candidate
        when_related: {via: input.candidate, exists: false}
        error: demo.release.NoCandidate
      - name: not-accepted
        when_related: {via: input.candidate, predicate: state != Accepted}
        error: demo.release.CandidateNotAccepted
      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}
      - name: published
        moves: demo.release.Release.publish
        instance: release_id
        emits: [demo.release.ReleasePublished]
        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}
views:
  - name: demo.release.Releases
    source: demo.release.Release
    consistency: read_your_writes
    fields:
      - {name: release_id, type: demo.release.ReleaseId}
      - {name: state, type: demo.release.Release.State}
";

/// The ess/18 owner link, where an accepting `when:` branch is declared before the present-row
/// refusal: below ess/22 and with no `wrong_state:`, declaration order selects.
const DECLARED_ORDER: &str = "format: ess/18
system: mini
version: v1
domain: mini.m
summary: Adversary model, declaration order.
types:
  - {name: mini.m.OwnerId, kind: newtype, of: Uuid}
  - {name: mini.m.ItemId, kind: newtype, of: Uuid}
  - {name: mini.m.RunId, kind: newtype, of: Uuid}
entities:
  - name: mini.m.Item
    identity: {name: item_id, type: mini.m.ItemId}
    fields:
      - {name: owner_id, type: mini.m.OwnerId}
    lifecycle: {initial: Listed, states: [Listed], terminal: [Listed]}
  - name: mini.m.Run
    identity: {name: run_id, type: mini.m.RunId}
    fields:
      - {name: item_id, type: mini.m.ItemId}
    lifecycle: {initial: Running, states: [Running], terminal: [Running]}
errors:
  - {name: mini.m.NoItem, summary: No item carries the identity., fields: []}
  - {name: mini.m.OtherOwner, summary: The item belongs to another owner., fields: []}
events:
  - name: mini.m.ItemAdded
    fields:
      - {name: item_id, type: mini.m.ItemId}
  - name: mini.m.Started
    fields:
      - {name: run_id, type: mini.m.RunId}
  - name: mini.m.Quick
    fields:
      - {name: run_id, type: mini.m.RunId}
actors:
  - name: mini.m.Operator
    may: [mini.m.AddItem, mini.m.Start]
commands:
  - name: mini.m.AddItem
    input:
      - {name: owner_id, type: mini.m.OwnerId}
    outcomes:
      - name: added
        creates: mini.m.Item
        instance: item_id
        emits: [mini.m.ItemAdded]
        payload:
          mini.m.ItemAdded: {item_id: {generated: true}}
        sets:
          owner_id: input.owner_id
  - name: mini.m.Start
    input:
      - {name: owner_id, type: mini.m.OwnerId}
      - {name: item_id, type: mini.m.ItemId}
      - {name: fast, type: Boolean}
    outcomes:
      - name: no-item
        when_related: {via: input.item_id, exists: false}
        error: mini.m.NoItem
      - name: quick
        when: fast == true
        creates: mini.m.Run
        instance: run_id
        emits: [mini.m.Quick]
        payload:
          mini.m.Quick: {run_id: {generated: true}}
        sets:
          item_id: input.item_id
      - name: other-owner
        when_related: {via: input.item_id, predicate: owner_id != input.owner_id}
        error: mini.m.OtherOwner
      - name: started
        creates: mini.m.Run
        instance: run_id
        emits: [mini.m.Started]
        payload:
          mini.m.Started: {run_id: {generated: true}}
        sets:
          item_id: input.item_id
views:
  - name: mini.m.Runs
    source: mini.m.Run
    consistency: read_your_writes
    fields:
      - {name: run_id, type: mini.m.RunId}
";

// ---- one request, two answers ----------------------------------------------------------------

/// One input value of a request: a literal, the identity step `n` produced, `null`, or omitted.
#[derive(Clone, Copy)]
enum Arg {
    Text(&'static str),
    Made(usize),
    Bool(bool),
    Int(i64),
    Null,
    Omit,
}

struct Req {
    command: &'static str,
    input: Vec<(&'static str, Arg)>,
}

fn req(command: &'static str, input: &[(&'static str, Arg)]) -> Req {
    Req {
        command,
        input: input.to_vec(),
    }
}

/// What one step answered: the outcome name, the error name, and the identity it created.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Answer {
    outcome: String,
    error: Option<String>,
    /// The error payload, by field, with absent and `null` fields dropped.
    fields: BTreeMap<String, String>,
}

fn node_json(node: &Node) -> serde_json::Value {
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(value) => serde_json::from_str(&value.to_string()).expect("a JSON number"),
        Node::Text(value) => serde_json::Value::String(value.clone()),
        Node::Seq(values) => serde_json::Value::Array(values.iter().map(node_json).collect()),
        Node::Map(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), node_json(value)))
                .collect(),
        ),
    }
}

fn node_of(arg: Arg, made: &[Option<String>]) -> Option<Node> {
    match arg {
        Arg::Text(text) => Some(Node::Text(text.to_owned())),
        Arg::Made(step) => Some(Node::Text(
            made[step].clone().expect("that step created an identity"),
        )),
        Arg::Bool(value) => Some(Node::Bool(value)),
        Arg::Int(value) => Some(Node::Number(ess_primitives::facts::Number::from(value))),
        Arg::Null => Some(Node::Null),
        Arg::Omit => None,
    }
}

fn json_of(arg: Arg, made: &[Option<String>]) -> Option<serde_json::Value> {
    match arg {
        Arg::Text(text) => Some(serde_json::Value::String(text.to_owned())),
        Arg::Made(step) => Some(serde_json::Value::String(
            made[step].clone().expect("that step created an identity"),
        )),
        Arg::Bool(value) => Some(serde_json::Value::Bool(value)),
        Arg::Int(value) => Some(serde_json::Value::from(value)),
        Arg::Null => Some(serde_json::Value::Null),
        Arg::Omit => None,
    }
}

/// The interpreter's answers to the sequence, from an empty store.
fn interpreted(ir: &EssIr, sequence: &[Req]) -> Vec<Answer> {
    let mut store = Store::default();
    let mut made: Vec<Option<String>> = Vec::new();
    let mut answers = Vec::new();
    for request in sequence {
        let input: BTreeMap<String, Node> = request
            .input
            .iter()
            .filter_map(|(name, arg)| node_of(*arg, &made).map(|node| ((*name).to_owned(), node)))
            .collect();
        let mut steps = execute(
            ir,
            &store,
            &request.command.parse().expect("a command name"),
            &input,
            &Externals::Withheld,
        )
        .unwrap_or_else(|error| panic!("{} {input:?} is interpreted: {error}", request.command));
        assert_eq!(steps.len(), 1, "{} selects one outcome", request.command);
        let step = steps.remove(0);
        let outcome = step.outcome.as_ref().map_or_else(
            || "none".to_owned(),
            |outcome| {
                outcome
                    .to_string()
                    .rsplit('/')
                    .next()
                    .expect("a name")
                    .to_owned()
            },
        );
        let error = step.error.as_ref().map(|error| error.error.to_string());
        let created = step
            .events
            .first()
            .filter(|_| step.error.is_none())
            .and_then(|event| event.payload.values().find_map(Node::as_text))
            .map(ToOwned::to_owned);
        let fields = step
            .error
            .as_ref()
            .map(|error| {
                error
                    .fields
                    .iter()
                    .filter(|(_, value)| **value != Node::Null)
                    .map(|(key, value)| (key.clone(), node_json(value).to_string()))
                    .collect()
            })
            .unwrap_or_default();
        made.push(created);
        answers.push(Answer {
            outcome,
            error,
            fields,
        });
        store = step.next;
    }
    answers
}

struct Live {
    child: std::process::Child,
    address: String,
}

impl Live {
    fn start(binary: &Path) -> Self {
        let mut child = Command::new(binary)
            .args(["--listen", "127.0.0.1:0", "--callers", "actor-header"])
            .env("GORACE", "halt_on_error=1 exitcode=66")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("the entry point starts");
        let stdout = child.stdout.take().expect("piped");
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    if value["event"] == "system.ready" {
                        let _ = sender.send(
                            value["runtime"]["address"]
                                .as_str()
                                .expect("an address")
                                .to_owned(),
                        );
                    }
                }
            }
        });
        let Ok(address) = receiver.recv_timeout(Duration::from_secs(20)) else {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the entry point did not become ready");
        };
        Self { child, address }
    }

    fn post(&self, path: &str, actor: &str, body: &serde_json::Value) -> (u16, serde_json::Value) {
        post(&self.address, path, actor, body)
    }

    /// Stops the process; `true` where the Go race detector reported a race (exit 66).
    fn stop(mut self) -> bool {
        // A race the detector found exits with 66 before it is killed.
        std::thread::sleep(Duration::from_millis(200));
        let raced = matches!(self.child.try_wait(), Ok(Some(status)) if status.code() == Some(66));
        let _ = self.child.kill();
        let _ = self.child.wait();
        raced
    }
}

fn post(
    address: &str,
    path: &str,
    actor: &str,
    body: &serde_json::Value,
) -> (u16, serde_json::Value) {
    let body = body.to_string();
    let mut stream = std::net::TcpStream::connect(address).expect("connects");
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .expect("timeout");
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nAuthorization: Actor {actor}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .expect("writes");
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).expect("reads");
    let boundary = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("a head");
    let head = std::str::from_utf8(&bytes[..boundary]).expect("UTF-8 head");
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("a status");
    let answer = serde_json::from_slice(&bytes[boundary + 4..]).unwrap_or_else(|_| {
        serde_json::Value::String(String::from_utf8_lossy(&bytes[boundary + 4..]).into_owned())
    });
    (status, answer)
}

/// One generated entry point of a model, ready to start.
struct Built {
    ir: EssIr,
    binary: PathBuf,
    root: PathBuf,
    routes: BTreeMap<String, String>,
    actor: String,
}

impl Drop for Built {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn build(text: &str, target: Target, label: &str) -> Built {
    build_ir(
        related::model(&related::served(text, "guarded")),
        target,
        label,
    )
}

fn build_ir(ir: EssIr, target: Target, label: &str) -> Built {
    let (root, synthesis) = related::emit(&ir, target, label);
    // Keep a copy of the emitted behaviour for the report, outside the build tree.
    if let Some(scratch) = std::env::var_os("ADVERSARY_SCRATCH") {
        let system = ir.system().to_string();
        let from = related::behaviour_path(&root, target, &system);
        let to = Path::new(&scratch).join(format!(
            "{label}-{}",
            from.file_name().unwrap().to_string_lossy()
        ));
        let _ = std::fs::copy(&from, &to);
    }
    // Every command of the model is generated: nothing below is answered by an obligation stub.
    for command in ir.commands().keys() {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, &command.to_string()),
            Some(&SynthesisDisposition::Generated),
            "{label}: `{command}` is generated:\n{}",
            synthesis.plan.to_markdown()
        );
    }
    let system = ir.system().to_string();
    let component = ir
        .components()
        .keys()
        .next()
        .expect("one served component")
        .to_string();
    let binary = related::build(&root, target, &system, &component, target == Target::Go);
    let routes = ir
        .components()
        .values()
        .flat_map(|component| ess_gen::http::routes(&ir, component))
        .filter_map(|route| match route.serves {
            ess_gen::http::Served::Command(command) => {
                Some((command.name().to_string(), route.path))
            }
            ess_gen::http::Served::View(_) => None,
        })
        .collect();
    // A model declaring no actor grants every caller; the header names none then.
    let actor = ir
        .actors()
        .keys()
        .next()
        .map_or_else(|| "anyone".to_owned(), ToString::to_string);
    Built {
        ir,
        binary,
        root,
        routes,
        actor,
    }
}

/// The generated entry point's answers to the sequence, from a fresh process.
fn served(built: &Built, sequence: &[Req]) -> Vec<Answer> {
    let live = Live::start(&built.binary);
    let mut made: Vec<Option<String>> = Vec::new();
    let mut answers = Vec::new();
    for request in sequence {
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .filter_map(|(name, arg)| {
                    json_of(*arg, &made).map(|value| ((*name).to_owned(), value))
                })
                .collect(),
        );
        let path = &built.routes[request.command];
        let (status, answer) = live.post(path, &built.actor, &body);
        let outcome = answer["outcome"]
            .as_str()
            .map_or_else(|| format!("http {status}: {answer}"), ToOwned::to_owned);
        let error = answer
            .get("error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        let created = answer["published"]
            .as_array()
            .and_then(|published| published.first())
            .filter(|_| error.is_none())
            .and_then(|event| {
                event["payload"]
                    .as_object()
                    .and_then(|payload| payload.values().find_map(serde_json::Value::as_str))
            })
            .map(ToOwned::to_owned);
        let fields = answer
            .get("payload")
            .filter(|_| error.is_some())
            .and_then(serde_json::Value::as_object)
            .map(|payload| {
                payload
                    .iter()
                    .filter(|(_, value)| !value.is_null())
                    .map(|(key, value)| (key.clone(), value.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        made.push(created);
        answers.push(Answer {
            outcome,
            error,
            fields,
        });
    }
    assert!(!live.stop(), "the race detector reported a data race");
    answers
}

/// Every sequence answered alike by the interpreter and by `built`; the disagreements otherwise.
fn disagreements(label: &str, built: &Built, sequences: &[(&str, Vec<Req>)]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, sequence) in sequences {
        let reference = interpreted(&built.ir, sequence);
        let generated = served(built, sequence);
        for (index, (expected, actual)) in reference.iter().zip(&generated).enumerate() {
            // An error field the interpreter does not carry (no declared source, `ess/19`) is not
            // determined by the model; every field it carries must be answered alike.
            let agrees = expected.outcome == actual.outcome
                && expected.error == actual.error
                && expected
                    .fields
                    .iter()
                    .all(|(key, value)| actual.fields.get(key) == Some(value));
            if !agrees {
                found.push(format!(
                    "{label} `{name}` step {index} `{}`: interpreter {expected:?}, generated \
                     {actual:?}",
                    sequence[index].command
                ));
            }
        }
        eprintln!("{label} `{name}`: interpreter {reference:?}");
    }
    found
}

// ---- sequences ---------------------------------------------------------------------------------

const ADD: &str = "demo.tasks.AddTask";
const SET: &str = "demo.tasks.SetBlocker";
const CANCEL: &str = "demo.tasks.CancelTask";
const COMPLETE: &str = "demo.tasks.CompleteTask";

fn task(step: usize) -> [(&'static str, Arg); 1] {
    [("task_id", Arg::Made(step))]
}

fn stored_sequences() -> Vec<(&'static str, Vec<Req>)> {
    use Arg::{Made, Null, Omit, Text};
    vec![
        (
            "no blocker, twice",
            vec![
                req(ADD, &[("blocked_by", Omit)]),
                req(COMPLETE, &task(0)),
                req(COMPLETE, &task(0)),
            ],
        ),
        (
            "unknown task",
            vec![req(COMPLETE, &[("task_id", Text(UNKNOWN))])],
        ),
        (
            "blocked, then done",
            vec![
                req(ADD, &[("blocked_by", Null)]),
                req(ADD, &[("blocked_by", Made(0))]),
                req(COMPLETE, &task(1)),
                req(COMPLETE, &task(0)),
                req(COMPLETE, &task(1)),
                req(COMPLETE, &task(1)),
            ],
        ),
        (
            "dangling, then cancelled",
            vec![
                req(ADD, &[("blocked_by", Text(DANGLING))]),
                req(COMPLETE, &task(0)),
                req(CANCEL, &task(0)),
                req(COMPLETE, &task(0)),
            ],
        ),
        (
            "blocker cancelled",
            vec![
                req(ADD, &[("blocked_by", Omit)]),
                req(ADD, &[("blocked_by", Made(0))]),
                req(CANCEL, &task(0)),
                req(COMPLETE, &task(1)),
            ],
        ),
        (
            "blocked by itself",
            vec![
                req(ADD, &[("blocked_by", Omit)]),
                req(SET, &[("task_id", Made(0)), ("blocker", Made(0))]),
                req(COMPLETE, &task(0)),
            ],
        ),
        (
            "rewritten, then cleared",
            vec![
                req(ADD, &[("blocked_by", Omit)]),
                req(ADD, &[("blocked_by", Made(0))]),
                req(SET, &[("task_id", Made(1)), ("blocker", Text(DANGLING))]),
                req(COMPLETE, &task(1)),
                req(SET, &[("task_id", Made(1)), ("blocker", Null)]),
                req(COMPLETE, &task(1)),
            ],
        ),
        (
            "blocker done, then rewritten to an open one",
            vec![
                req(ADD, &[("blocked_by", Omit)]),
                req(ADD, &[("blocked_by", Omit)]),
                req(ADD, &[("blocked_by", Made(0))]),
                req(COMPLETE, &task(0)),
                req(SET, &[("task_id", Made(2)), ("blocker", Made(1))]),
                req(COMPLETE, &task(2)),
            ],
        ),
    ]
}

const DRAFT: &str = "demo.release.DraftRelease";
const PROPOSE: &str = "demo.release.ProposeCandidate";
const ACCEPT: &str = "demo.release.AcceptCandidate";
const PUBLISH: &str = "demo.release.PublishRelease";

fn optional_sequences() -> Vec<(&'static str, Vec<Req>)> {
    use Arg::{Bool, Made, Null, Omit, Text};
    vec![
        (
            "missing candidate before the input refusal and the unknown release",
            vec![req(
                PUBLISH,
                &[
                    ("release_id", Text(UNKNOWN)),
                    ("candidate", Text(DANGLING)),
                    ("force", Bool(true)),
                ],
            )],
        ),
        (
            "input refusal before the unknown release",
            vec![req(
                PUBLISH,
                &[
                    ("release_id", Text(UNKNOWN)),
                    ("candidate", Omit),
                    ("force", Bool(true)),
                ],
            )],
        ),
        (
            "unknown release, null candidate",
            vec![req(
                PUBLISH,
                &[
                    ("release_id", Text(UNKNOWN)),
                    ("candidate", Null),
                    ("force", Bool(false)),
                ],
            )],
        ),
        (
            "unknown release, proposed candidate",
            vec![
                req(PROPOSE, &[]),
                req(
                    PUBLISH,
                    &[
                        ("release_id", Text(UNKNOWN)),
                        ("candidate", Made(0)),
                        ("force", Bool(false)),
                    ],
                ),
            ],
        ),
        (
            "proposed, published, null, omitted",
            vec![
                req(DRAFT, &[]),
                req(PROPOSE, &[]),
                req(
                    PUBLISH,
                    &[
                        ("release_id", Made(0)),
                        ("candidate", Made(1)),
                        ("force", Bool(false)),
                    ],
                ),
                req(
                    PUBLISH,
                    &[
                        ("release_id", Made(0)),
                        ("candidate", Null),
                        ("force", Bool(false)),
                    ],
                ),
                req(
                    PUBLISH,
                    &[
                        ("release_id", Made(0)),
                        ("candidate", Made(1)),
                        ("force", Bool(false)),
                    ],
                ),
                req(
                    PUBLISH,
                    &[
                        ("release_id", Made(0)),
                        ("candidate", Omit),
                        ("force", Bool(false)),
                    ],
                ),
            ],
        ),
        (
            "accepted",
            vec![
                req(DRAFT, &[]),
                req(PROPOSE, &[]),
                req(ACCEPT, &[("candidate_id", Made(1))]),
                req(ACCEPT, &[("candidate_id", Made(1))]),
                req(
                    PUBLISH,
                    &[
                        ("release_id", Made(0)),
                        ("candidate", Made(1)),
                        ("force", Bool(false)),
                    ],
                ),
            ],
        ),
    ]
}

const ADD_ITEM: &str = "mini.m.AddItem";
const START: &str = "mini.m.Start";
const OWNER: &str = "00000000-0000-4000-8000-000000000001";
const OTHER: &str = "00000000-0000-4000-8000-000000000002";

fn declared_order_sequences() -> Vec<(&'static str, Vec<Req>)> {
    use Arg::{Bool, Made, Text};
    vec![(
        "quick for another owner",
        vec![
            req(ADD_ITEM, &[("owner_id", Text(OWNER))]),
            req(
                START,
                &[
                    ("owner_id", Text(OTHER)),
                    ("item_id", Made(0)),
                    ("fast", Bool(true)),
                ],
            ),
            req(
                START,
                &[
                    ("owner_id", Text(OTHER)),
                    ("item_id", Made(0)),
                    ("fast", Bool(false)),
                ],
            ),
            req(
                START,
                &[
                    ("owner_id", Text(OWNER)),
                    ("item_id", Made(0)),
                    ("fast", Bool(false)),
                ],
            ),
            req(
                START,
                &[
                    ("owner_id", Text(OWNER)),
                    ("item_id", Text(DANGLING)),
                    ("fast", Bool(true)),
                ],
            ),
        ],
    )]
}

fn swap(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model carries {from:?}");
    text.replacen(from, to, 1)
}

/// [`STORED`] where `wrong_state:` describes the held row (its state and stored blocker), and an
/// accepting `when:` branch addressing no row is declared before the moving default.
fn stored_checked() -> String {
    let text = swap(
        STORED,
        "  - {name: demo.tasks.TaskStateConflict, summary: The task cannot move., fields: []}\n",
        "  - name: demo.tasks.TaskStateConflict\n    summary: The task cannot move.\n    fields:\n      - {name: state, type: demo.tasks.Task.State}\n      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}\n",
    );
    let text = swap(
        &text,
        "  - name: demo.tasks.BlockerSet\n",
        "  - name: demo.tasks.TaskChecked\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n  - name: demo.tasks.BlockerSet\n",
    );
    let text = swap(
        &text,
        "  - name: demo.tasks.CompleteTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n",
        "  - name: demo.tasks.CompleteTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n      - {name: check, type: Boolean}\n",
    );
    swap(
        &text,
        "      - name: completed\n        moves: demo.tasks.Task.complete\n",
        "      - name: checked\n        when: check == true\n        emits: [demo.tasks.TaskChecked]\n        payload: {demo.tasks.TaskChecked: {task_id: input.task_id}}\n      - name: completed\n        moves: demo.tasks.Task.complete\n",
    )
}

/// [`STORED`] with a required reference: every task stores a blocker, which blocks while open.
fn stored_required() -> String {
    let text = swap(
        STORED,
        "      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}\n    lifecycle:",
        "      - {name: blocked_by, type: demo.tasks.TaskId}\n    lifecycle:",
    );
    let text = swap(
        &text,
        "      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}\n    outcomes:\n      - name: added",
        "      - {name: blocked_by, type: demo.tasks.TaskId}\n    outcomes:\n      - name: added",
    );
    let text = swap(
        &text,
        "      - {name: blocker, type: Optional<demo.tasks.TaskId>}\n",
        "      - {name: blocker, type: demo.tasks.TaskId}\n",
    );
    let text = swap(
        &text,
        "        when_related: {via: blocked_by, predicate: state != Done}\n",
        "        when_related: {via: blocked_by, predicate: state == Open}\n",
    );
    let text = swap(&text, "        sets: {blocked_by: {cleared: true}}\n", "");
    swap(
        &text,
        "      - {name: blocked_by, type: Optional<demo.tasks.TaskId>}\n      - {name: state",
        "      - {name: blocked_by, type: demo.tasks.TaskId}\n      - {name: state",
    )
}

fn checked_sequences() -> Vec<(&'static str, Vec<Req>)> {
    use Arg::{Bool, Made, Omit, Text};
    let complete = |step: usize, check: bool| {
        req(COMPLETE, &[("task_id", Made(step)), ("check", Bool(check))])
    };
    vec![
        (
            "checked, completed, checked when done, refused with its row",
            vec![
                req(ADD, &[("blocked_by", Omit)]),
                complete(0, true),
                complete(0, false),
                complete(0, true),
                complete(0, false),
            ],
        ),
        (
            "checked beside a dangling blocker",
            vec![
                req(ADD, &[("blocked_by", Text(DANGLING))]),
                complete(0, true),
                req(CANCEL, &task(0)),
                complete(0, false),
                complete(0, true),
            ],
        ),
        (
            "checked beside an open blocker",
            vec![
                req(ADD, &[("blocked_by", Omit)]),
                req(ADD, &[("blocked_by", Made(0))]),
                complete(1, true),
                complete(1, false),
                req(CANCEL, &task(1)),
                complete(1, false),
            ],
        ),
        (
            "unknown task, checked",
            vec![req(
                COMPLETE,
                &[("task_id", Text(UNKNOWN)), ("check", Bool(true))],
            )],
        ),
    ]
}

fn required_sequences() -> Vec<(&'static str, Vec<Req>)> {
    use Arg::{Made, Text};
    vec![
        (
            "missing, blocked, unblocked by cancelling, rewritten after done",
            vec![
                req(ADD, &[("blocked_by", Text(DANGLING))]),
                req(COMPLETE, &task(0)),
                req(ADD, &[("blocked_by", Made(0))]),
                req(COMPLETE, &task(2)),
                req(CANCEL, &task(0)),
                req(COMPLETE, &task(2)),
                req(COMPLETE, &task(2)),
                req(SET, &[("task_id", Made(2)), ("blocker", Text(DANGLING))]),
                req(COMPLETE, &task(2)),
            ],
        ),
        (
            "blocked by itself, then cancelled",
            vec![
                req(ADD, &[("blocked_by", Text(DANGLING))]),
                req(SET, &[("task_id", Made(0)), ("blocker", Made(0))]),
                req(COMPLETE, &task(0)),
                req(CANCEL, &task(0)),
                req(COMPLETE, &task(0)),
            ],
        ),
    ]
}

/// A labelled model and the request sequences it is driven with.
type Model = (&'static str, String, Vec<(&'static str, Vec<Req>)>);

fn models() -> Vec<Model> {
    vec![
        ("stored", STORED.to_owned(), stored_sequences()),
        ("stored-reordered", stored_reordered(), stored_sequences()),
        ("stored-checked", stored_checked(), checked_sequences()),
        ("stored-required", stored_required(), required_sequences()),
        (
            "stored-checked-guarded",
            swap(
                &stored_checked(),
                "        when_related: {via: blocked_by, predicate: state != Done}\n        error: demo.tasks.Blocked\n",
                "        when_related: {via: blocked_by, predicate: state != Done}\n        when: check == false\n        error: demo.tasks.Blocked\n",
            ),
            checked_sequences(),
        ),
        ("optional", OPTIONAL.to_owned(), optional_sequences()),
        (
            "declared-order",
            DECLARED_ORDER.to_owned(),
            declared_order_sequences(),
        ),
    ]
}

fn differential(target: Target) {
    let mut found = Vec::new();
    for (label, text, sequences) in models() {
        let label = format!("{target:?}-{label}");
        let built = build(&text, target, &label);
        found.extend(disagreements(&label, &built, &sequences));
    }
    assert!(
        found.is_empty(),
        "the generated behaviour disagrees with the interpreter:\n{}",
        found.join("\n")
    );
}

#[test]
fn adv319_generated_rust_answers_every_sequence_as_the_interpreter() {
    differential(Target::Rust);
}

#[test]
fn adv319_generated_go_answers_every_sequence_as_the_interpreter() {
    assert!(related::go_available(), "Go is required");
    differential(Target::Go);
}

/// Many connections at once against the Go entry point built with the race detector: tasks blocked
/// by one another completed concurrently. The serving lock must keep every related read and the
/// write it decides in one critical section, so the detector stays silent and no task completes
/// while the blocker it read was still open.
#[test]
fn adv319_concurrent_requests_against_the_go_entry_point_race_free() {
    assert!(related::go_available(), "Go is required");
    let built = build(STORED, Target::Go, "Go-stored-concurrent");
    let live = Live::start(&built.binary);
    let address = live.address.clone();
    let route = |command: &str| built.routes[command].clone();
    let (add, complete, set) = (route(ADD), route(COMPLETE), route(SET));
    let actor = built.actor.clone();
    // A chain: task i is blocked by task i-1.
    let mut chain: Vec<String> = Vec::new();
    for index in 0..24 {
        let body = match chain.last() {
            Some(previous) if index % 6 != 0 => serde_json::json!({ "blocked_by": previous }),
            _ => serde_json::json!({}),
        };
        let (_, answer) = post(&address, &add, &actor, &body);
        chain.push(
            answer["published"][0]["payload"]
                .as_object()
                .and_then(|payload| payload.values().find_map(serde_json::Value::as_str))
                .expect("an identity")
                .to_owned(),
        );
    }
    let threads: Vec<_> = (0..8)
        .map(|worker| {
            let (address, complete, set, actor, chain) = (
                address.clone(),
                complete.clone(),
                set.clone(),
                actor.clone(),
                chain.clone(),
            );
            std::thread::spawn(move || {
                let mut answers = Vec::new();
                for round in 0..6 {
                    for (index, task) in chain.iter().enumerate().rev() {
                        if (index + worker + round) % 5 == 0 {
                            let _ = post(
                                &address,
                                &set,
                                &actor,
                                &serde_json::json!({ "task_id": task, "blocker": chain[(index + 1) % chain.len()] }),
                            );
                        }
                        let (_, answer) =
                            post(&address, &complete, &actor, &serde_json::json!({ "task_id": task }));
                        answers.push(answer["outcome"].as_str().unwrap_or("?").to_owned());
                    }
                }
                answers
            })
        })
        .collect();
    let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
    for thread in threads {
        for outcome in thread.join().expect("a worker") {
            *outcomes.entry(outcome).or_default() += 1;
        }
    }
    eprintln!("concurrent outcomes: {outcomes:?}");
    assert!(
        !outcomes.contains_key("?"),
        "every request answered: {outcomes:?}"
    );
    assert!(!live.stop(), "the race detector reported a data race");
}

// ---- a related row of another domain, stored by the one component ------------------------------

/// `split.shops`, the domain whose `Shop` row the `split.orders` command reads.
const SPLIT_SHOPS: &str = "format: ess/22
system: split
version: v1
domain: split.shops
types:
  - {name: split.shops.ShopId, kind: newtype, of: Uuid}
entities:
  - name: split.shops.Shop
    identity: {name: shop_id, type: split.shops.ShopId}
    fields:
      - {name: region, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - {name: split.shops.ShopStateConflict, summary: The shop cannot move., fields: []}
events:
  - name: split.shops.ShopOpened
    fields: [{name: shop_id, type: split.shops.ShopId}]
  - name: split.shops.ShopClosed
    fields: [{name: shop_id, type: split.shops.ShopId}]
commands:
  - name: split.shops.OpenShop
    input:
      - {name: region, type: String}
    outcomes:
      - name: opened
        creates: split.shops.Shop
        instance: shop_id
        sets: {region: input.region}
        emits: [split.shops.ShopOpened]
        payload: {split.shops.ShopOpened: {shop_id: {generated: true}}}
  - name: split.shops.CloseShop
    input:
      - {name: shop_id, type: split.shops.ShopId}
    outcomes:
      - name: closed
        moves: split.shops.Shop.close
        instance: shop_id
        emits: [split.shops.ShopClosed]
        payload: {split.shops.ShopClosed: {shop_id: input.shop_id}}
      - {name: close-conflict, wrong_state: true, error: split.shops.ShopStateConflict}
";

/// `split.orders`, one network component owning both domains.
const SPLIT_ORDERS: &str = "domain: split.orders
errors:
  - name: split.orders.NoShop
  - name: split.orders.ShopClosed
  - name: split.orders.WrongRegion
events:
  - name: split.orders.Ordered
    fields: [{name: shop_id, type: split.shops.ShopId}]
actors:
  - name: split.orders.Clerk
    may: [split.shops.OpenShop, split.shops.CloseShop, split.orders.Order]
commands:
  - name: split.orders.Order
    input:
      - {name: shop_id, type: split.shops.ShopId}
      - {name: region, type: String}
    outcomes:
      - name: no-shop
        when_related: {via: input.shop_id, exists: false}
        error: split.orders.NoShop
      - name: shop-closed
        when_related: {via: input.shop_id, predicate: state == Closed}
        error: split.orders.ShopClosed
      - name: wrong-region
        when_related: {via: input.shop_id, predicate: region != input.region}
        error: split.orders.WrongRegion
      - name: ordered
        emits: [split.orders.Ordered]
        payload: {split.orders.Ordered: {shop_id: input.shop_id}}
components:
  - component: orders
    reached_by: network
    owns: {domains: [split.orders, split.shops]}
    accepts: {commands: [split.orders.Order, split.shops.OpenShop, split.shops.CloseShop]}
    publishes: {events: [split.orders.Ordered, split.shops.ShopOpened, split.shops.ShopClosed]}
";

fn split_model() -> EssIr {
    use ess_domain::spec::{RawSpecFile, Specification};
    use ess_domain::system::Source;
    let specification = Specification::assemble(
        [("shops.yaml", SPLIT_SHOPS), ("orders.yaml", SPLIT_ORDERS)]
            .into_iter()
            .map(|(path, text)| {
                (
                    Source::new(path),
                    RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
                )
            }),
    )
    .unwrap_or_else(|errors| panic!("{errors}"));
    ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
}

fn split_sequences() -> Vec<(&'static str, Vec<Req>)> {
    use Arg::{Made, Text};
    const OPEN: &str = "split.shops.OpenShop";
    const CLOSE: &str = "split.shops.CloseShop";
    const ORDER: &str = "split.orders.Order";
    vec![(
        "missing, ordered, wrong region, closed",
        vec![
            req(
                ORDER,
                &[("shop_id", Text(DANGLING)), ("region", Text("eu"))],
            ),
            req(OPEN, &[("region", Text("eu"))]),
            req(ORDER, &[("shop_id", Made(1)), ("region", Text("eu"))]),
            req(ORDER, &[("shop_id", Made(1)), ("region", Text("us"))]),
            req(CLOSE, &[("shop_id", Made(1))]),
            req(ORDER, &[("shop_id", Made(1)), ("region", Text("us"))]),
            req(ORDER, &[("shop_id", Made(1)), ("region", Text("eu"))]),
        ],
    )]
}

fn split(target: Target) {
    let label = format!("{target:?}-split");
    let built = build_ir(split_model(), target, &label);
    let found = disagreements(&label, &built, &split_sequences());
    assert!(
        found.is_empty(),
        "the generated behaviour disagrees with the interpreter:\n{}",
        found.join("\n")
    );
}

/// The plan generates a command whose related row is of another domain where one component owns
/// both (`a_related_row_no_accepting_component_stores_stays_owed` checks only the plan): the
/// emitted tree builds and answers as the interpreter does.
#[test]
fn adv319_a_related_row_of_another_domain_rust() {
    split(Target::Rust);
}

#[test]
fn adv319_a_related_row_of_another_domain_go() {
    assert!(related::go_available(), "Go is required");
    split(Target::Go);
}

// ---- committed generated output ----------------------------------------------------------------

/// Every artifact the Rust, Go and Web targets synthesize for the committed examples is the
/// committed byte for byte, not only the plan and the behaviour: the related read is rendered for
/// no command that does not read a related row.
#[test]
fn adv319_every_committed_artifact_of_the_examples_is_unchanged() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut changed = Vec::new();
    let mut compared = 0_usize;
    for (target, directory, examples) in [
        (Target::Rust, "rust", &["billing", "gatepass"][..]),
        (Target::Go, "go", &["billing", "gatepass"][..]),
        (Target::Web, "web", &["billing"][..]),
    ] {
        for example in examples {
            let ir = compile_directory(&repository.join("examples").join(example));
            let synthesis = ess_synth::synthesize_for(&ir, target).expect("synthesizes");
            for artifact in synthesis.artifacts.values() {
                let path = repository
                    .join("generated")
                    .join(directory)
                    .join(example)
                    .join(&artifact.path);
                let Ok(committed) = std::fs::read_to_string(&path) else {
                    continue;
                };
                compared += 1;
                if committed != artifact.contents {
                    changed.push(format!("generated/{directory}/{example}/{}", artifact.path));
                }
            }
        }
    }
    eprintln!("compared {compared} committed artifacts");
    assert!(
        compared > 20,
        "the comparison read the committed trees: {compared}"
    );
    assert!(
        changed.is_empty(),
        "committed artifacts drift: {changed:#?}"
    );
}

/// Every YAML document under `base`, compiled as `ess synthesize` reads a directory.
fn compile_directory(base: &Path) -> EssIr {
    let mut labels = Vec::new();
    let mut pending = vec![base.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                labels.push(path.strip_prefix(base).unwrap().display().to_string());
            }
        }
    }
    labels.sort();
    let mut sources = ess_compiler::source::SourceMap::new();
    let mut parsed = Vec::new();
    for label in &labels {
        let text = std::fs::read_to_string(base.join(label)).expect("readable");
        let raw = ess_domain::spec::RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((ess_domain::system::Source::new(label.clone()), raw));
    }
    let specification = ess_domain::spec::Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("{errors}"));
    ess_compiler::resolve::compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

// ---- `existing_instance:` before the missing related row ----------------------------------------

/// The suite's precedence model (`declared_behaviour.rs`, `related_precedence_text(false)`): an
/// input refusal, a missing row, a present-row refusal, a creation and `existing_instance:`, in a
/// declaration order that disagrees with selection.
fn precedence_text() -> String {
    let head = include_str!("fixtures/declared-behaviour-kept/head.yaml");
    format!(
        "{head}  - name: kept.shop.PlaceOrder
    input:
      - {{name: order_id, type: kept.shop.OrderId}}
      - {{name: shop_id, type: kept.shop.ShopId}}
      - {{name: quantity, type: Integer}}
    outcomes:
      - {{name: invalid-quantity, when: 'quantity < 0', error: kept.shop.Refused}}
      - name: no-shop
        when_related: {{via: input.shop_id, exists: false}}
        error: kept.shop.NoShop
      - name: wrong-region
        when_related: {{via: input.shop_id, predicate: 'region != \"EU\"'}}
        error: kept.shop.Refused
      - name: placed
        creates: kept.shop.Order
        instance: order_id
        sets: {{shop_id: input.shop_id, quantity: input.quantity}}
        emits: [kept.shop.OrderPlaced]
        payload:
          kept.shop.OrderPlaced: {{order_id: input.order_id}}
      - {{name: already-placed, existing_instance: true, error: kept.shop.Conflict}}
"
    )
}

const ORDER_1: &str = "00000000-0000-4000-8000-0000000000a1";
const ORDER_2: &str = "00000000-0000-4000-8000-0000000000a2";

fn precedence_sequences() -> Vec<(&'static str, Vec<Req>)> {
    use Arg::{Made, Text};
    const OPEN: &str = "kept.shop.OpenShop";
    const STOCK: &str = "kept.shop.StockOrder";
    const PLACE: &str = "kept.shop.PlaceOrder";
    let place = |order: Arg, shop: Arg, quantity: i64| {
        req(
            PLACE,
            &[
                ("order_id", order),
                ("shop_id", shop),
                ("quantity", Arg::Int(quantity)),
            ],
        )
    };
    vec![(
        "an existing order before a missing shop, an input refusal and a present-row refusal",
        vec![
            req(OPEN, &[("region", Text("EU"))]),
            place(Text(ORDER_1), Made(0), 1),
            place(Text(ORDER_1), Text(DANGLING), -1),
            place(Text(ORDER_2), Text(DANGLING), -1),
            place(Text(ORDER_2), Made(0), -1),
            req(OPEN, &[("region", Text("US"))]),
            place(Text(ORDER_2), Made(5), 1),
            place(Text(ORDER_1), Made(5), 1),
            req(STOCK, &[("shop_id", Made(0)), ("quantity", Arg::Int(2))]),
            place(Made(8), Text(DANGLING), 1),
        ],
    )]
}

fn precedence(target: Target, faulty: Option<(&str, &str)>) -> (Vec<String>, usize) {
    let label = format!(
        "{target:?}-precedence{}",
        if faulty.is_some() { "-faulty" } else { "" }
    );
    let ir = related::model(&related::served(&precedence_text(), "guarded"));
    let built = build_ir(ir.clone(), target, &label);
    if let Some((from, to)) = faulty {
        // The faulty variant: the same emitted tree, one line of its behaviour edited, rebuilt.
        related::mutate(
            &related::behaviour_path(&built.root, target, "kept"),
            from,
            to,
        );
        let component = ir.components().keys().next().unwrap().to_string();
        related::build(
            &built.root,
            target,
            "kept",
            &component,
            target == Target::Go,
        );
    }
    let found = disagreements(&label, &built, &precedence_sequences());
    // What the synthesized suite (the unit's own oracle) reports against the same binary.
    let failed = related::run(&built.ir, &built.binary)
        .values()
        .filter(|status| **status != ess_conformance::report::Status::Passed)
        .count();
    (found, failed)
}

#[test]
fn adv319_existing_instance_answers_before_a_missing_related_row_rust() {
    let (found, failed) = precedence(Target::Rust, None);
    assert!(found.is_empty(), "{}", found.join("\n"));
    assert_eq!(failed, 0, "the honest tree passes its synthesized suite");
}

#[test]
fn adv319_existing_instance_answers_before_a_missing_related_row_go() {
    assert!(related::go_available(), "Go is required");
    let (found, failed) = precedence(Target::Go, None);
    assert!(found.is_empty(), "{}", found.join("\n"));
    assert_eq!(failed, 0, "the honest tree passes its synthesized suite");
}

/// Faulty control: the emitted behaviour answers a missing shop before an existing order (the
/// `existing_instance:` lookup taken only where the related row is present). The sequence above
/// fails it; the synthesized suite's count is printed beside it.
#[test]
fn adv319_a_missing_row_answered_before_existing_instance_is_caught_rust() {
    let (found, failed) = precedence(
        Target::Rust,
        Some((
            "if OrderStorage::get(&self.ports, &input.order_id).is_some() {",
            "if OrderStorage::get(&self.ports, &input.order_id).is_some() && ShopStorage::get(&self.ports, &input.shop_id).is_some() {",
        )),
    );
    eprintln!("Rust faulty precedence: the synthesized suite failed {failed} scenarios; the sequence found {found:#?}");
    assert!(!found.is_empty(), "the sequence catches the faulty order");
}

#[test]
fn adv319_a_missing_row_answered_before_existing_instance_is_caught_go() {
    assert!(related::go_available(), "Go is required");
    let (found, failed) = precedence(
        Target::Go,
        Some((
            "if _, found := b.ports.OrderStorage.Get(input.OrderId); found {",
            "if _, found := b.ports.OrderStorage.Get(input.OrderId); found && func() bool { _, shop := b.ports.ShopStorage.Get(input.ShopId); return shop }() {",
        )),
    );
    eprintln!("Go faulty precedence: the synthesized suite failed {failed} scenarios; the sequence found {found:#?}");
    assert!(!found.is_empty(), "the sequence catches the faulty order");
}
