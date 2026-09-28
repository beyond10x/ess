//! Selection by existence (ess/16, beyond10x/ess#164) lands in the IR: the creating half of
//! create-or-update as an `unknown_instance` condition carrying its creation, the refusal half of
//! create-or-refuse as its own `existing_instance` condition and test strategy.

use ess_compiler::ir::{EssIr, ResolvedCondition, ResolvedEffect};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::TestStrategy;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/upsert-by-existence.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("upsert-by-existence.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn outcome<'i>(ir: &'i EssIr, command: &str, name: &str) -> &'i ess_compiler::ir::ResolvedOutcome {
    ir.commands()
        .get(&command.parse().unwrap())
        .unwrap_or_else(|| panic!("{command} is declared"))
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("{command}/{name} is declared"))
}

#[test]
fn the_creating_half_of_create_or_update_carries_its_creation() {
    let ir = ir();
    let created = outcome(&ir, "demo.items.PutItem", "created");
    assert_eq!(created.condition, ResolvedCondition::UnknownInstance);
    assert_eq!(created.test_strategy, TestStrategy::SendUnknownIdentity);
    assert_eq!(
        created.subject.as_ref().map(|subject| &subject.effect),
        Some(&ResolvedEffect::Creates)
    );
    let json = serde_json::to_value(created).unwrap();
    assert_eq!(json["condition"]["kind"], "unknown_instance");
    let updated = outcome(&ir, "demo.items.PutItem", "updated");
    assert_eq!(updated.condition, ResolvedCondition::Otherwise);
}

#[test]
fn the_refusal_half_of_create_or_refuse_is_its_own_condition_and_strategy() {
    let ir = ir();
    let branch = outcome(&ir, "demo.items.BookSlot", "already-booked");
    assert_eq!(branch.condition, ResolvedCondition::ExistingInstance);
    assert_eq!(branch.test_strategy, TestStrategy::SendExistingIdentity);
    assert!(branch.refuses);
    assert!(branch.subject.is_none());
    let json = serde_json::to_value(branch).unwrap();
    assert_eq!(json["condition"]["kind"], "existing_instance");
    assert_eq!(json["test_strategy"], "send_existing_identity");
}
