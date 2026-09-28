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
    assert_eq!(affect.filter.to_string(), "team == subject.team");
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
