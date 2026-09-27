//! `input_absent: true` (ess/16, beyond10x/ess#170) lands in the IR as its own condition and test
//! strategy, so every consumer reads the branch rather than re-deriving it.

use ess_compiler::ir::{EssIr, ResolvedCondition};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::TestStrategy;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/absent-input.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("absent-input.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn the_absent_input_branch_is_its_own_condition_and_strategy() {
    let ir = ir();
    let command = ir
        .commands()
        .get(&"demo.notes.SubmitNote".parse().unwrap())
        .expect("SubmitNote");
    let branch = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "body-missing")
        .expect("body-missing");
    assert_eq!(branch.condition, ResolvedCondition::InputAbsent);
    assert_eq!(branch.test_strategy, TestStrategy::SendNoInput);
    assert!(branch.refuses);
    assert!(branch.subject.is_none());
    let json = serde_json::to_value(branch).unwrap();
    assert_eq!(json["condition"]["kind"], "input_absent");
    assert_eq!(json["test_strategy"], "send_no_input");
}
