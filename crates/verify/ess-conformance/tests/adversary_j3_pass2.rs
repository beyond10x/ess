//! Adversary pass 2 on `mutate --scenarios` (<https://github.com/beyond10x/ess/issues/515>): the
//! correction's identical-answer rule, attacked where the outcome's name is not only a label.
//!
//! `Ruler::judge` (`mutate.rs`) now drops every failure of an authored scenario on a
//! `precedence-swap` carrying `identical_answer`, on the claim that the mutant's interpreter
//! "answers every input as the specification's does but for the outcome's name, which only names
//! the branch". A binding's refusal-selected failure policy (ess/22,
//! `docs/design/conditional-binding-failure-policies.md`) is selected by that name: the interpreter
//! picks drop, retry or escalate from the outcome the attempt answered (`interpret/bindings.rs`,
//! `policy.select`). So two refusals with the same error and no payload, one dropped and one
//! escalated, answer the command's caller alike and still publish different events.
//!
//! The model is inline rather than a fixture file, so the repository-wide fixture tables do not
//! move.
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn documents(label: &str, text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label.to_owned()), raw)], texts)
}

/// The audit as `ess verify conform mutate --target interpreted --scenarios` runs it.
fn audit(text: &str, scenarios: &[(&str, &str)]) -> MutationReport {
    let (documents, texts) = documents("ledger.yaml", text);
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let mut set = BTreeMap::new();
    for (name, scenario) in scenarios {
        let authoring = ess_conformance::authored::compile(
            &ir,
            &[ess_conformance::authored::Source::new(*name, *scenario)],
        );
        assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
        set.extend(authoring.scenarios);
    }
    mutate::audit_with_authored(
        &documents,
        &texts,
        &[MutantClass::PrecedenceSwap],
        || Interpreted::for_model(ir.clone()),
        None,
        &set,
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn entry<'a>(report: &'a MutationReport, id: &str) -> &'a mutate::MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("{id}: {}", report.render_text()))
}

/// `Record` refuses a prompt starting with `hold ` and one ending with ` hold` with the same
/// error and no payload, the shape `identical_answer` calls identical. The binding that records
/// each placed order drops the first refusal and escalates the second.
const LEDGER: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.ledger
errors:
  - name: demo.ledger.Invalid
    fields: []
events:
  - name: demo.ledger.OrderPlaced
    fields:
      - {name: prompt, type: String}
  - name: demo.ledger.RecordEscalated
    fields:
      - {name: prompt, type: String}
commands:
  - name: demo.ledger.Ping
    input: []
    outcomes:
      - name: ponged
        accepts: nothing
  - name: demo.ledger.Place
    input:
      - {name: prompt, type: String}
    outcomes:
      - name: placed
        emits: [demo.ledger.OrderPlaced]
        payload:
          demo.ledger.OrderPlaced: {prompt: input.prompt}
  - name: demo.ledger.Record
    input:
      - {name: prompt, type: String}
    outcomes:
      - name: leading-hold
        when: {prompt: {starts_with: "hold "}}
        error: demo.ledger.Invalid
      - name: trailing-hold
        when: {prompt: {ends_with: " hold"}}
        error: demo.ledger.Invalid
      - name: recorded
        accepts: nothing
bindings:
  - id: notify-ledger
    when:
      event: demo.ledger.OrderPlaced
    invoke:
      command: demo.ledger.Record
    mapping:
      prompt: event.prompt
    delivery: at_least_once
    on_failure:
      drop: [leading-hold]
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [leading-hold]
"#;

const RECORD_SWAP: &str = "precedence-swap/demo.ledger.Record/leading-hold/trailing-hold";

/// `hold me hold` is placed; the binding's `Record` refuses it as `leading-hold`, which is
/// dropped, so nothing is escalated in the twenty seconds after.
const NOT_ESCALATED: &str = r"type: ess-scenario/1
domain: demo.ledger
scenario: both-holds-are-dropped
summary: An order whose prompt both starts and ends with hold is dropped, never escalated.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.ledger.Ping
    input: {}
    outcome: ponged
    mark: opened
  - at: 2026-01-05T09:00:01Z
    command: demo.ledger.Place
    input: {prompt: hold me hold}
    outcome: placed
  - at: 2026-01-05T09:00:30Z
    command: demo.ledger.Ping
    input: {}
    outcome: ponged
    elapsed:
      - since: opened
        quiet: {for: PT20S, events: [demo.ledger.RecordEscalated]}
";

/// On the swapped interpreter `Record` answers `trailing-hold` where both guards hold, the binding
/// escalates it and `RecordEscalated` is published: a real difference an authored scenario observes,
/// not the outcome's label. The F3 rule drops that failure because the swap carries
/// `identical_answer`, and the entry comes out `equivalent`, the claim that nothing a caller observes
/// tells the mutant apart.
#[test]
fn a_swap_a_binding_policy_tells_apart_is_killed_by_the_authored_scenario_that_sees_it() {
    let report = audit(LEDGER, &[("both-holds-are-dropped.yaml", NOT_ESCALATED)]);
    let entry = entry(&report, RECORD_SWAP);
    assert_eq!(
        entry.verdict,
        Verdict::Killed,
        "an escalation the authored scenario saw: {entry:#?}\n{}",
        report.render_text()
    );
    assert_eq!(
        entry.killers.as_deref(),
        Some(["demo.ledger/authored/both-holds-are-dropped".to_owned()].as_slice()),
        "{entry:#?}"
    );
}

/// Whatever the authored scenarios say, `identical_answer` must not claim of this swap that
/// "neither changes state, sets or emits anything": one of the two is escalated by the binding.
#[test]
fn a_swap_a_binding_policy_tells_apart_carries_no_identical_answer() {
    let report = audit(LEDGER, &[]);
    let entry = entry(&report, RECORD_SWAP);
    assert!(
        entry.identical_answer.is_none() && entry.verdict != Verdict::Equivalent,
        "the binding drops one branch and escalates the other: {entry:#?}\n{}",
        report.render_text()
    );
}

/// The difference the two red cases above rest on, measured as `AuthoredSuite::on` measures it:
/// the authored scenario on the scenario-clock runner, against the interpreter of the
/// specification (passed) and of the swap (failed, on the escalation the binding published).
#[test]
fn the_authored_scenario_passes_on_the_specification_and_fails_on_the_swap() {
    let (documents, texts) = documents("ledger.yaml", LEDGER);
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let authoring = ess_conformance::authored::compile(
        &ir,
        &[ess_conformance::authored::Source::new(
            "both-holds-are-dropped.yaml",
            NOT_ESCALATED,
        )],
    );
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    let mut suite =
        ess_conformance::ConformanceSuite::new(ess_conformance::SuiteProvenance::of(&ir));
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    suite.select_fresh_format_for(&ir);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).expect("admitted");
    let status = |model: &ess_compiler::EssIr| {
        let mut runner = ess_conformance::now_offset::scenario_clock_runner(&suite);
        let interpreter =
            Interpreted::for_model(model.clone()).with_command_clock(runner.command_clock());
        let executed = runner.run_admitted(&admitted, &interpreter);
        executed
            .report()
            .scenarios
            .iter()
            .map(|result| (result.scenario.to_string(), result.status))
            .collect::<Vec<_>>()
    };
    let swap = mutate::selection(&documents, &[MutantClass::PrecedenceSwap])
        .mutants
        .into_iter()
        .find(|mutant| mutant.id == RECORD_SWAP)
        .expect("the swap is selected");
    let swapped = mutate::compile(
        mutate::apply(&documents, &swap.mutation).expect("applies"),
        &texts,
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    let id = "demo.ledger/authored/both-holds-are-dropped".to_owned();
    assert_eq!(
        status(&ir),
        vec![(id.clone(), ess_conformance::report::Status::Passed)]
    );
    assert_eq!(
        status(&swapped),
        vec![(id, ess_conformance::report::Status::Failed)]
    );
}
