//! Adversary pass 1 against #268 slice 2 (ess/22, beyond10x/ess#268 and beyond10x/ess#194): the
//! mutants `docs/design/conditional-binding-failure-policies.md` names as required to fail,
//! expressed as interpreted targets over a rewritten model and run against the suite synthesized
//! for the model as written.
//!
//! Each mutant is the interpreter (the reference reading of the condition) over a model whose
//! condition or sibling was rewritten so that it behaves exactly as the defect does on every
//! payload. The suite is the one synthesized for the original model, so a mutant that no scenario
//! fails is a defect the suite cannot see in any target, generated Rust and Go included.
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, ConformanceTarget, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("fixtures/binding-condition.yaml");
const SELECTED: &str = include_str!("fixtures/binding-condition-selected.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";

fn rewrite(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
    text.replacen(from, to, 1)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("messages.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite_of(text: &str) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir_of(text)).suite
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, (Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let checks = format!("{:?}", result.checks);
            (result.scenario.to_string(), (result.status, checks))
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, (Status, String)>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, (status, _))| *status == Status::Failed)
        .map(|(id, _)| id.clone())
        .collect()
}

/// The design, "Payload condition": Unknown "invokes nothing and is reported as an unmet
/// obligation; it cannot become a successful skip", and "Mutants ignoring the predicate, treating
/// Unknown as false, unwrapping before testing, and sharing another binding's condition fail."
///
/// `[event.kind == ship, event.order.id == o-1]` is Unknown for a shipping message with no order.
/// Prepending `defined(event.order)` makes exactly that case False and changes nothing else: it is
/// the condition read with Unknown as False, the successful skip the design forbids. The honest
/// interpreter reports the obligation (`unsupported`); the mutant must fail some scenario.
#[test]
fn a_target_reading_unknown_as_false_fails_some_synthesized_scenario() {
    let unknown = rewrite(
        MODEL,
        WHERE,
        "      where: [event.kind == ship, event.order.id == o-1]\n",
    );
    let as_false = rewrite(
        MODEL,
        WHERE,
        "      where: [defined(event.order), event.kind == ship, event.order.id == o-1]\n",
    );
    let suite = suite_of(&unknown);
    let honest = statuses(&suite, &Interpreted::for_model(ir_of(&unknown)));
    assert_eq!(
        honest["received/binding/condition-absent"].0,
        Status::Unsupported,
        "precondition: the honest target reports Unknown as its obligation: {honest:#?}"
    );
    let mutant = statuses(&suite, &Interpreted::for_model(ir_of(&as_false)));
    assert_ne!(
        failed(&mutant),
        Vec::<String>::new(),
        "a target that skips on Unknown (successful skip) fails no scenario; condition-absent is \
         {:?} for it and {:?} for the honest target",
        mutant["received/binding/condition-absent"],
        honest["received/binding/condition-absent"].0,
    );
}

/// `other_binding_still_invokes` and the design's "sharing another binding's condition" mutant,
/// with the sibling a selection binding: `siblings` in `synthesize.rs` leaves every selecting
/// binding out of the sibling expectation of both negative witnesses.
///
/// `logged` is conditioned on `event.tag != z`; `received` is unconditioned. The mutant gives
/// `received` `logged`'s condition, so it stops wherever `logged` skips. With `received` a plain
/// binding (the control) some scenario fails the mutant; with `received` selecting its first leg,
/// the same mutant must fail some scenario too.
#[test]
fn a_sibling_sharing_the_conditioned_bindings_condition_fails_some_scenario_when_it_selects() {
    let base = rewrite(
        SELECTED,
        "      where: [defined(event.tag), event.kind == ship]\n",
        "",
    );
    let selecting = rewrite(
        &base,
        "      order_id: event.tag\n",
        "      order_id: event.message_id\n",
    );
    let plain = rewrite(
        &selecting,
        "    selection_inputs:\n      - name: legs\n        from: event.legs\n        as: List<demo.messages.Ref>\n    selections:\n      - name: first_leg\n        first:\n          in: legs\n          where: 'item.id != \"\"'\n",
        "",
    );
    let plain = rewrite(
        &plain,
        "      leg: {selection: first_leg, path: [id]}\n",
        "",
    );
    let condition = |text: &str| {
        rewrite(
            text,
            "  - id: logged\n    when: {event: demo.messages.MessageReceived}\n",
            "  - id: logged\n    when:\n      event: demo.messages.MessageReceived\n      where: event.tag != z\n",
        )
    };
    let shared = |text: &str| {
        rewrite(
            text,
            "      event: demo.messages.MessageReceived\n    invoke: {command: demo.messages.MessageEvent}\n",
            "      event: demo.messages.MessageReceived\n      where: event.tag != z\n    invoke: {command: demo.messages.MessageEvent}\n",
        )
    };
    let mut caught = BTreeMap::new();
    for (shape, text) in [("plain", plain), ("selecting", selecting)] {
        let honest_text = condition(&text);
        let suite = suite_of(&honest_text);
        let honest = statuses(&suite, &Interpreted::for_model(ir_of(&honest_text)));
        assert_eq!(
            honest["logged/binding/condition-false"].0,
            Status::Passed,
            "precondition ({shape}): the honest target passes the false witness: {honest:#?}"
        );
        assert_eq!(
            failed(&honest),
            Vec::<String>::new(),
            "precondition ({shape}): the honest target fails nothing"
        );
        let mutant = statuses(
            &suite,
            &Interpreted::for_model(ir_of(&shared(&honest_text))),
        );
        let document: serde_json::Value =
            serde_json::from_str(&suite.to_canonical_json().expect("the suite serialises"))
                .expect("JSON");
        println!(
            "{shape}: logged/binding/condition-false steps: {:#}",
            document["scenarios"]["logged/binding/condition-false"]["steps"]
        );
        caught.insert(shape, failed(&mutant));
    }
    assert_ne!(
        caught["plain"],
        Vec::<String>::new(),
        "control: the mutant is caught beside a plain sibling"
    );
    assert_ne!(
        caught["selecting"],
        Vec::<String>::new(),
        "the same mutant beside a selecting sibling fails no scenario (control caught it in {:?})",
        caught["plain"]
    );
}
