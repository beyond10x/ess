//! A `creates:` whose subject is owned arranges its owner first.
//!
//! The claim under test is not "the suite has an extra step". It is that a generated scenario stops
//! arranging a world the specification says cannot exist: `owns` means the far side does not stand
//! on its own, so a scenario that creates one row without the row it belongs to is asking an
//! implementation a question with no answer — which is what fifteen `unsupported` verdicts on an
//! adopter's model turned out to be.
//!
//! Every negative case here is a *fallback*, never a refusal. The suite that could not arrange an
//! owner is the suite it was before, and the four ways that happens are one test each.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::{ScenarioStep, ScenarioValue, ViewExpectation};
use ess_conformance::{ConformanceSuite, ScenarioId};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/owned-subject.yaml");
const POSTED: &str = "ledger.book.PostEntry/outcome/posted";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("owned-subject.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let result = ess_conformance::synthesize::synthesize(ir);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    result.suite
}

/// The commands a scenario runs, in order, as plain names.
fn commands(suite: &ConformanceSuite, id: &str) -> Vec<String> {
    suite.scenarios[&ScenarioId::parse(id).unwrap()]
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

/// What the scenario's last command was handed for one input field.
fn supplied(suite: &ConformanceSuite, id: &str, field: &str) -> ScenarioValue {
    suite.scenarios[&ScenarioId::parse(id).unwrap()]
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => input.get(field).cloned(),
            _ => None,
        })
        .next_back()
        .unwrap_or_else(|| panic!("`{id}` supplies `{field}`"))
}

#[test]
fn a_created_subject_that_is_owned_is_created_under_an_owner_that_exists() {
    let suite = suite(&ir(MODEL));

    assert_eq!(
        commands(&suite, POSTED),
        ["ledger.book.OpenAccount", "ledger.book.PostEntry"],
        "the entry is posted to an account the scenario opened"
    );

    // Not a literal. A generated identity is the implementation's to assign, so the only way to name
    // the account that was just opened is to refer to it.
    let account = match supplied(&suite, POSTED, "account_id") {
        ScenarioValue::Instance { instance } => instance,
        other => panic!("`account_id` names the arranged account, not {other:?}"),
    };

    // And the reference resolves: the step that bound that name is in the scenario, ahead of the
    // command that uses it. Without this the assertion above would pass against a name nobody bound.
    let bound = suite.scenarios[&ScenarioId::parse(POSTED).unwrap()]
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::CaptureInstance { instance, event, .. }
                if instance == &account && event.to_string() == "ledger.book.AccountOpened")
        })
        .expect("the account is captured from the event that announced it");
    let used = suite.scenarios[&ScenarioId::parse(POSTED).unwrap()]
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "ledger.book.PostEntry")
        })
        .expect("the entry is posted");
    assert!(bound < used, "the account is bound before it is named");
}

#[test]
fn the_row_is_asserted_to_hold_the_owner_the_scenario_arranged() {
    let suite = suite(&ir(MODEL));

    // `sets: account_id: input.account_id` is what makes this assertable, and what it asserts is a
    // value neither the suite nor the specification can spell: the id of the account step one
    // opened. An implementation that files the entry under some other account fails here.
    let account = match supplied(&suite, POSTED, "account_id") {
        ScenarioValue::Instance { instance } => instance,
        other => panic!("{other:?}"),
    };
    let shown = suite.scenarios[&ScenarioId::parse(POSTED).unwrap()]
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => fields.get("account_id").cloned(),
            _ => None,
        })
        .expect("`EntryById` is asserted to carry the entry's account");
    assert_eq!(shown, ScenarioValue::instance(account));
}

#[test]
fn without_the_declared_link_nothing_is_arranged_and_nothing_is_guessed() {
    // The relation stays declared and stays validated; only the `sets:` line naming the input goes.
    // The field is still spelled `account_id` on both sides and still typed as the account's
    // identity — so a synthesizer that matched on a name or on a type would arrange an account here,
    // and this is the test that says it must not.
    let without = MODEL.replace(
        "sets: {account_id: input.account_id, memo: input.memo}",
        "sets: {memo: input.memo}",
    );
    assert_ne!(without, MODEL, "the line this test removes is still there");
    let suite = suite(&ir(&without));

    assert_eq!(commands(&suite, POSTED), ["ledger.book.PostEntry"]);
    assert!(matches!(
        supplied(&suite, POSTED, "account_id"),
        ScenarioValue::Literal { .. }
    ));
}

#[test]
fn an_owner_nothing_creates_leaves_the_arrangement_as_it_was() {
    // `examples/billing/` in miniature: the ownership is declared, and no command says how an owner
    // comes to be. Refusing the scenario was the alternative and would delete every scenario that
    // creates a billing invoice — scenarios that pass.
    let head = MODEL
        .split("  - name: ledger.book.OpenAccount")
        .next()
        .unwrap();
    let tail = MODEL
        .split("  - name: ledger.book.PostEntry")
        .nth(1)
        .unwrap();
    let without = format!("{head}  - name: ledger.book.PostEntry{tail}");
    let suite = suite(&ir(&without));

    assert_eq!(commands(&suite, POSTED), ["ledger.book.PostEntry"]);
    assert!(matches!(
        supplied(&suite, POSTED, "account_id"),
        ScenarioValue::Literal { .. }
    ));
}

#[test]
fn an_entity_nobody_owns_arranges_what_it_always_did() {
    // The root case, and the one that has to stay free: an unowned entity is not an error, so
    // `OpenAccount` arranges nothing and the scenario is one command long.
    let suite = suite(&ir(MODEL));
    assert_eq!(
        commands(&suite, "ledger.book.OpenAccount/outcome/opened"),
        ["ledger.book.OpenAccount"]
    );
}

/// Two entities that each own the other. Neither declaration is wrong on its own — `owns` is
/// refused only where *two* entities claim one target — so `validate_relations` has nothing to say
/// and this is a model an author can write by accident.
const CYCLE: &str = r"
format: ess/4
system: ring
version: v1
domain: ring.core
entities:
  - name: ring.core.Left
    identity: {name: left_id, type: Uuid}
    fields:
      - {name: right_id, type: Uuid}
    relations:
      - {name: lefts, kind: owns, target: ring.core.Right, cardinality: one, via: left_id}
    lifecycle: {initial: Up, states: [Up], terminal: [Up]}
  - name: ring.core.Right
    identity: {name: right_id, type: Uuid}
    fields:
      - {name: left_id, type: Uuid}
    relations:
      - {name: rights, kind: owns, target: ring.core.Left, cardinality: one, via: right_id}
    lifecycle: {initial: Up, states: [Up], terminal: [Up]}
events:
  - name: ring.core.LeftMade
    fields:
      - {name: left_id, type: Uuid}
  - name: ring.core.RightMade
    fields:
      - {name: right_id, type: Uuid}
commands:
  - name: ring.core.MakeLeft
    input:
      - {name: right_id, type: Uuid}
    outcomes:
      - name: made
        creates: ring.core.Left
        instance: left_id
        emits: [ring.core.LeftMade]
        payload:
          ring.core.LeftMade:
            left_id: {generated: true}
        sets: {right_id: input.right_id}
  - name: ring.core.MakeRight
    input:
      - {name: left_id, type: Uuid}
    outcomes:
      - name: made
        creates: ring.core.Right
        instance: right_id
        emits: [ring.core.RightMade]
        payload:
          ring.core.RightMade:
            right_id: {generated: true}
        sets: {left_id: input.left_id}
";

#[test]
fn two_entities_that_own_each_other_arrange_once_and_stop() {
    // The chain, not a depth count. Arranging a `Left` needs a `Right`, which needs a `Left` — and
    // the second one is refused because arranging a `Left` is already in progress, not because some
    // number ran out. Two commands, and the suite is finite.
    let raw = RawSpecFile::parse(CYCLE).unwrap();
    let spec = Specification::assemble([(Source::new("ring.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let suite = suite(&ir);

    assert_eq!(
        commands(&suite, "ring.core.MakeLeft/outcome/made"),
        ["ring.core.MakeRight", "ring.core.MakeLeft"]
    );
    assert_eq!(
        commands(&suite, "ring.core.MakeRight/outcome/made"),
        ["ring.core.MakeLeft", "ring.core.MakeRight"]
    );
}
