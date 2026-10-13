//! Set effects over filtered instances (ess/16, beyond10x/ess#167, #175) land in the IR: a set
//! subject with its entity, verb and filter; `{count: changed}` as its own value; `affects:` with
//! each entry's entity, filter and resolved `sets:`. An outcome declaring neither keeps its bytes.

use ess_compiler::ir::{EssIr, ResolvedEffect, ResolvedOutcome, ResolvedPayloadValue};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/set-effects.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("set-effects.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn outcome<'i>(ir: &'i EssIr, command: &str, name: &str) -> &'i ResolvedOutcome {
    ir.commands()
        .get(&command.parse().unwrap())
        .unwrap_or_else(|| panic!("{command} is declared"))
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("{command}/{name} is declared"))
}

#[test]
fn a_set_move_carries_its_entity_transition_filter_sets_and_count() {
    let ir = ir();
    let ended = outcome(&ir, "demo.desk.EndTeam", "ended");
    assert!(ended.subject.is_none());
    let set = ended
        .instances
        .as_ref()
        .expect("the set subject is in the IR");
    assert_eq!(set.entity.name().to_string(), "demo.desk.Session");
    let ResolvedEffect::Moves { transition } = &set.effect else {
        panic!("a move: {:?}", set.effect)
    };
    assert_eq!(transition.name, "end");
    assert_eq!(set.filter.to_string(), "team == input.team");
    assert_eq!(
        ended
            .sets
            .iter()
            .map(|set| set.target.as_str())
            .collect::<Vec<_>>(),
        ["note"]
    );
    let count = ended
        .payload
        .iter()
        .flat_map(|payload| &payload.fields)
        .find(|field| field.target == "ended")
        .expect("the count is determined");
    assert_eq!(count.value, ResolvedPayloadValue::ChangedCount);
    let json = serde_json::to_value(ended).unwrap();
    assert_eq!(json["instances"]["effect"], "moves");
    assert_eq!(json["instances"]["filter"], "team == input.team");
}

#[test]
fn a_set_update_carries_no_transition() {
    let ir = ir();
    let noted = outcome(&ir, "demo.desk.NoteTeam", "noted");
    let set = noted
        .instances
        .as_ref()
        .expect("the set subject is in the IR");
    assert_eq!(set.effect, ResolvedEffect::Updates);
}

#[test]
fn affects_carries_each_entry_resolved() {
    let ir = ir();
    let invited = outcome(&ir, "demo.desk.Invite", "invited");
    assert!(invited.subject.is_some());
    let [affect] = invited.affects.as_slice() else {
        panic!("one entry: {:#?}", invited.affects)
    };
    assert_eq!(affect.entity.name().to_string(), "demo.desk.Session");
    assert_eq!(
        affect
            .filter
            .as_ref()
            .expect("a `where:` entry")
            .to_string(),
        "team == subject.team"
    );
    assert_eq!(affect.sets.len(), 1);
    assert_eq!(affect.sets[0].target, "on_hold");
}

#[test]
fn an_outcome_declaring_neither_keeps_its_bytes() {
    let ir = ir();
    let opened = outcome(&ir, "demo.desk.Open", "opened");
    let json = serde_json::to_value(opened).unwrap();
    assert!(json.get("instances").is_none(), "{json}");
    assert!(json.get("affects").is_none(), "{json}");
}

/// A call and a queue entry storing the call's identity; expiring the entry ends the call it names
/// (ess/25, beyond10x/ess#547, `docs/design/set-effect-identity-filters.md`).
const IDENTITY: &str = "format: ess/25
system: demo
version: v1
domain: demo.q
types:
  - {name: demo.q.CallId, kind: newtype, of: String}
  - {name: demo.q.EntryId, kind: newtype, of: String}
entities:
  - name: demo.q.Call
    identity: {name: call_id, type: demo.q.CallId}
    fields: [{name: label, type: String}]
    lifecycle:
      initial: Waiting
      states: [Waiting, Ended]
      terminal: [Ended]
      transitions: [{name: end, from: [Waiting], to: Ended}]
  - name: demo.q.Entry
    identity: {name: entry_id, type: demo.q.EntryId}
    fields: [{name: call_id, type: demo.q.CallId}]
    lifecycle:
      initial: Queued
      states: [Queued, Expired]
      terminal: [Expired]
      transitions: [{name: expire, from: [Queued], to: Expired}]
events:
  - name: demo.q.CallStarted
    fields: [{name: call_id, type: demo.q.CallId}]
  - name: demo.q.EntryQueued
    fields: [{name: entry_id, type: demo.q.EntryId}]
  - name: demo.q.EntryExpired
    fields: [{name: entry_id, type: demo.q.EntryId}]
errors:
  - name: demo.q.NotQueued
commands:
  - name: demo.q.StartCall
    input: [{name: label, type: String}]
    outcomes:
      - name: started
        creates: demo.q.Call
        instance: call_id
        sets: {label: input.label}
        emits: [demo.q.CallStarted]
        payload: {demo.q.CallStarted: {call_id: {generated: true}}}
  - name: demo.q.QueueEntry
    input: [{name: call_id, type: demo.q.CallId}]
    outcomes:
      - name: queued
        creates: demo.q.Entry
        instance: entry_id
        sets: {call_id: input.call_id}
        emits: [demo.q.EntryQueued]
        payload: {demo.q.EntryQueued: {entry_id: {generated: true}}}
  - name: demo.q.ExpireEntry
    input: [{name: entry_id, type: demo.q.EntryId}]
    outcomes:
      - name: expired
        moves: demo.q.Entry.expire
        instance: entry_id
        emits: [demo.q.EntryExpired]
        payload: {demo.q.EntryExpired: {entry_id: input.entry_id}}
        affects:
          - entity: demo.q.Call
            where: subject.call_id == call_id
            moves: demo.q.Call.end
      - {name: not-queued, wrong_state: true, error: demo.q.NotQueued}
  - name: demo.q.ForgetEntry
    input: [{name: entry_id, type: demo.q.EntryId}]
    outcomes:
      - name: forgotten
        moves: demo.q.Entry.expire
        instances: {where: entry_id == input.entry_id}
        emits: [demo.q.EntryExpired]
        payload: {demo.q.EntryExpired: {entry_id: input.entry_id}}
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn an_identity_filter_compiles_into_the_filter() {
    let ir = compiled(IDENTITY);
    let expired = outcome(&ir, "demo.q.ExpireEntry", "expired");
    let [affect] = expired.affects.as_slice() else {
        panic!("one entry: {:#?}", expired.affects)
    };
    let filter = affect.filter.as_ref().expect("a `where:` entry");
    // The bare identity on the right is the candidate row's, not the text `call_id`.
    assert!(
        filter
            .fact_paths()
            .iter()
            .any(|path| path.to_string() == "call_id"),
        "{filter:?}"
    );
    // A bare fact on the right is written canonically so it does not read as text.
    assert_eq!(filter.to_string(), "subject.call_id == {fact: call_id}");
    let json = serde_json::to_value(expired).unwrap();
    assert_eq!(
        json["affects"][0]["filter"],
        serde_json::json!({"subject.call_id": {"eq": {"fact": "call_id"}}})
    );

    let forgotten = outcome(&ir, "demo.q.ForgetEntry", "forgotten");
    let set = forgotten.instances.as_ref().expect("the set subject");
    assert_eq!(set.filter.to_string(), "entry_id == input.entry_id");
    let json = serde_json::to_value(forgotten).unwrap();
    assert_eq!(json["instances"]["filter"], "entry_id == input.entry_id");
}

/// At `ess/25` a document reading no identity in a set effect's filter compiles to the canonical
/// IR it compiled to at `ess/24`, byte for byte.
#[test]
fn a_model_without_an_identity_filter_keeps_its_ir_bytes() {
    for fixture in [
        MODEL,
        include_str!("fixtures/set-deletes.yaml"),
        include_str!("fixtures/set-each.yaml"),
    ] {
        let at = |format: &str| {
            let written = fixture.lines().next().expect("a header line").to_owned();
            assert!(written.starts_with("format: ess/"), "{written}");
            compiled(&fixture.replacen(&written, &format!("format: {format}"), 1))
                .to_canonical_json()
        };
        assert_eq!(at("ess/24"), at("ess/25"));
    }
}
