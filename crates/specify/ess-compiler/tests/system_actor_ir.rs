//! A system actor and a system-issued command in the IR (`ess/27`, beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *IR shape*): `ResolvedActor.system`,
//! `ResolvedCommand.system_issued` and `ResolvedCommand.await_within`, each omitted when unset, and
//! where each refusal of the validation table is located.

use ess_compiler::resolve::{compile, diagnose_locating};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn model(actors: &str, assign: &str) -> String {
    format!(
        "format: ess/27
system: queueing
version: v1
domain: queueing.entry
types:
  - {{name: queueing.entry.EntryId, kind: newtype, of: Uuid}}
  - {{name: queueing.entry.WorkerId, kind: newtype, of: String}}
entities:
  - name: queueing.entry.Entry
    identity: {{name: entry_id, type: queueing.entry.EntryId}}
    fields:
      - {{name: worker, type: Optional<queueing.entry.WorkerId>}}
    lifecycle:
      initial: Waiting
      states: [Waiting, Assigned]
      terminal: [Assigned]
      transitions:
        - {{name: assign, from: [Waiting], to: Assigned}}
errors:
  - {{name: queueing.entry.EntryStateConflict, summary: The entry is not waiting., fields: []}}
events:
  - name: queueing.entry.Enqueued
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
  - name: queueing.entry.Assigned
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
actors:
  - name: queueing.entry.Caller
    may: [queueing.entry.Enqueue]
{actors}commands:
  - name: queueing.entry.Enqueue
    input: []
    outcomes:
      - name: enqueued
        creates: queueing.entry.Entry
        instance: entry_id
        emits: [queueing.entry.Enqueued]
        payload:
          queueing.entry.Enqueued: {{entry_id: {{generated: true}}}}
  - name: queueing.entry.Assign
{assign}    input:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
    outcomes:
      - name: assigned
        moves: queueing.entry.Entry.assign
        instance: entry_id
        emits: [queueing.entry.Assigned]
        sets: {{worker: input.worker}}
        payload:
          queueing.entry.Assigned: {{entry_id: input.entry_id, worker: input.worker}}
      - {{name: wrong-state, wrong_state: true, error: queueing.entry.EntryStateConflict}}
"
    )
}

const SCHEDULER: &str = "  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Assign]
";

const DRIVER: &str = "  - name: queueing.entry.Scheduler
    may: [queueing.entry.Assign]
";

fn ir_json(text: &str) -> serde_json::Value {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("queueing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).expect("compiles");
    serde_json::to_value(&ir).unwrap()
}

/// Every diagnostic `text` earns, as `(code, path)`.
fn diagnosed(text: &str) -> Vec<(String, String)> {
    let raw = RawSpecFile::parse(text).unwrap();
    let errors = Specification::assemble([(Source::new("queueing.yaml"), raw)])
        .expect_err("the document must be refused");
    let mut sources = SourceMap::new();
    sources.insert("queueing.yaml".to_owned(), text.to_owned());
    diagnose_locating(&errors, &sources, &["queueing.yaml"])
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic.span.as_ref().expect("a span");
            assert!(span.located.is_some(), "{span:?} is found in the text");
            (diagnostic.code.to_string(), span.path.clone())
        })
        .collect()
}

/// The object under `kind` (`actors`, `commands`) named `name`, wherever the IR holds it.
fn named<'v>(ir: &'v serde_json::Value, kind: &str, name: &str) -> &'v serde_json::Value {
    let holder = &ir[kind];
    holder
        .get(name)
        .or_else(|| {
            holder
                .as_array()
                .and_then(|items| items.iter().find(|item| item["name"] == name))
        })
        .unwrap_or_else(|| panic!("no {kind} `{name}` in {holder}"))
}

#[test]
fn a_system_actor_and_its_command_carry_the_flags_in_the_ir() {
    let ir = ir_json(&model(SCHEDULER, "    await_within: PT5S\n"));
    assert_eq!(
        named(&ir, "actors", "queueing.entry.Scheduler")["system"],
        true
    );
    let assign = named(&ir, "commands", "queueing.entry.Assign");
    assert_eq!(assign["system_issued"], true);
    assert_eq!(assign["await_within"], "PT5S");

    let caller = named(&ir, "actors", "queueing.entry.Caller");
    assert!(caller.get("system").is_none(), "{caller}");
    let enqueue = named(&ir, "commands", "queueing.entry.Enqueue");
    assert!(enqueue.get("system_issued").is_none(), "{enqueue}");
    assert!(enqueue.get("await_within").is_none(), "{enqueue}");
}

#[test]
fn a_system_issued_command_without_a_bound_omits_it() {
    let ir = ir_json(&model(SCHEDULER, ""));
    let assign = named(&ir, "commands", "queueing.entry.Assign");
    assert_eq!(assign["system_issued"], true);
    assert!(assign.get("await_within").is_none(), "{assign}");
}

#[test]
fn a_model_without_either_key_keeps_its_ir_bytes() {
    let with_false = model(
        "  - name: queueing.entry.Scheduler\n    system: false\n    may: [queueing.entry.Assign]\n",
        "",
    );
    let without = model(DRIVER, "");
    let compile_text = |text: &str| {
        let raw = RawSpecFile::parse(text).unwrap();
        let spec = Specification::assemble([(Source::new("queueing.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("{errors}"));
        compile(&spec, &SourceMap::new())
            .expect("compiles")
            .to_canonical_json()
    };
    let bytes = compile_text(&without);
    assert_eq!(compile_text(&with_false), bytes);
    for key in ["\"system\": true", "\"system_issued\"", "\"await_within\""] {
        assert!(!bytes.contains(key), "{key} in {bytes}");
    }
}

#[test]
fn each_refusal_is_located_at_its_key() {
    let at = |text: String| -> Vec<String> {
        diagnosed(&text).into_iter().map(|(_, path)| path).collect()
    };
    assert_eq!(
        at(model(DRIVER, "    await_within: PT5S\n")),
        vec!["command.queueing.entry.Assign.await_within".to_owned()]
    );
    assert_eq!(
        at(model(SCHEDULER, "    await_within: PT5S\n").replace("ess/27", "ess/26")),
        vec![
            "actor.queueing.entry.Scheduler.system".to_owned(),
            "command.queueing.entry.Assign.await_within".to_owned()
        ]
    );
    assert_eq!(
        at(model(
            &format!("{SCHEDULER}    attributes:\n      - {{name: tenant, type: String}}\n"),
            ""
        )),
        vec!["actor.queueing.entry.Scheduler.attributes".to_owned()]
    );
}
