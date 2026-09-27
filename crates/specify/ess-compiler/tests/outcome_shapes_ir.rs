//! The five outcome shapes of `ess/15` land in the IR (`docs/design/outcome-shapes.md`).
//!
//! What each consumer needs is in the compiled document, not re-derived: the unknown-instance
//! branch as its own condition, a removal as its own effect, the state a creation lands in beside
//! the creation, an accepted no-op as a flag, and the system's preconditions with their commands
//! resolved. A model using none of them keeps its bytes, which the committed corpus pins.

use ess_compiler::ir::{EssIr, ResolvedCondition, ResolvedEffect};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::TestStrategy;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/outcome-shapes.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("outcome-shapes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn outcome<'a>(ir: &'a EssIr, command: &str, name: &str) -> &'a ess_compiler::ir::ResolvedOutcome {
    ir.commands()
        .get(&command.parse().unwrap())
        .unwrap_or_else(|| panic!("{command}"))
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("{command}/{name}"))
}

#[test]
fn the_unknown_instance_branch_is_its_own_condition_and_strategy() {
    let ir = ir();
    let answer = outcome(&ir, "example.call.AnswerCall", "no-such-call");
    assert_eq!(answer.condition, ResolvedCondition::UnknownInstance);
    assert_eq!(answer.test_strategy, TestStrategy::SendUnknownIdentity);
    assert!(answer.refuses);
    let json = serde_json::to_value(answer).unwrap();
    assert_eq!(json["condition"]["kind"], "unknown_instance");
    assert_eq!(json["test_strategy"], "send_unknown_identity");
}

#[test]
fn a_removal_is_its_own_effect() {
    let ir = ir();
    let ended = outcome(&ir, "example.call.EndCall", "ended");
    let subject = ended.subject.as_ref().expect("a subject");
    assert_eq!(subject.effect, ResolvedEffect::Deletes);
    assert_eq!(subject.instance.field().name, "call_id");
    let json = serde_json::to_value(ended).unwrap();
    assert_eq!(json["subject"]["effect"], "deletes");
    assert_eq!(json["subject"]["instance"]["from"], "supplied");
}

#[test]
fn a_creation_carries_the_state_it_lands_in_and_only_when_written() {
    let ir = ir();
    let offered = outcome(&ir, "example.call.OfferCall", "offered");
    let subject = offered.subject.as_ref().expect("a subject");
    assert_eq!(subject.effect, ResolvedEffect::Creates);
    assert_eq!(
        subject.into.as_ref().map(ToString::to_string).as_deref(),
        Some("Ringing")
    );
    assert_eq!(
        serde_json::to_value(offered).unwrap()["subject"]["into"],
        "Ringing"
    );
    let placed = outcome(&ir, "example.call.PlaceCall", "placed");
    assert!(
        serde_json::to_value(placed).unwrap()["subject"]
            .get("into")
            .is_none(),
        "a creation into `initial` writes no key"
    );
}

#[test]
fn an_accepted_no_op_is_flagged_and_other_outcomes_write_nothing() {
    let ir = ir();
    let accepted = outcome(&ir, "example.call.Touch", "accepted");
    assert!(accepted.accepts_nothing);
    assert_eq!(
        serde_json::to_value(accepted).unwrap()["accepts_nothing"],
        true
    );
    let placed = outcome(&ir, "example.call.PlaceCall", "placed");
    assert!(serde_json::to_value(placed)
        .unwrap()
        .get("accepts_nothing")
        .is_none());
}

#[test]
fn the_system_preconditions_are_resolved_in_order() {
    let ir = ir();
    let preconditions = ir.preconditions();
    assert_eq!(preconditions.len(), 1);
    assert_eq!(
        ir.command(&preconditions[0].command).name.to_string(),
        "example.call.OpenSession"
    );
    assert_eq!(
        preconditions[0]
            .actor
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("example.call.Agent")
    );
    let json: serde_json::Value = serde_json::from_str(&ir.to_canonical_json()).unwrap();
    assert_eq!(
        json["preconditions"][0]["command"],
        "example.call.OpenSession"
    );
    assert_eq!(json["preconditions"][0]["actor"], "example.call.Agent");
    assert_eq!(
        json["preconditions"][0]["input"]["user_id"],
        "00000000-0000-4000-8000-000000000152"
    );
}

#[test]
fn a_model_without_preconditions_writes_no_key() {
    let without = MODEL.replace(
        "preconditions:
  - command: example.call.OpenSession
    as: example.call.Agent
    input: {user_id: 00000000-0000-4000-8000-000000000152}
",
        "",
    );
    let raw = RawSpecFile::parse(&without).unwrap();
    let spec = Specification::assemble([(Source::new("outcome-shapes.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&ir.to_canonical_json()).unwrap();
    assert!(json.get("preconditions").is_none());
}
