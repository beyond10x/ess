//! Adversary pass 1 against `story:synthesis-reads-selection-plan`, wave 5 unit U2 (the one
//! "before" query in `synthesize/precedence.rs`).
//!
//! - A1 (`authored::not_taken`'s early returns): the unit removed the early return for every
//!   `exists: false` on a command guarded by a related row. A stored `exists: false` (through a
//!   field of the subject, ess/22 #304) sits in `PresentRelated`, after the input refusals, so an
//!   authored act claiming it with an input an input refusal decides is now refused
//!   (`ESS-AUTHOR-041`) where the base accepted it. The interpreter answers the input refusal for
//!   that request (A1a, green), so the new answer is the plan's; the base answer is what the
//!   unit's contract ("no behaviour change for any model that validates") pins (A1b, red).
//! - A2 (the kind checks that stayed, `earlier_accepting_branches` and `admits_plain`): with the
//!   input-refusal and accepting phases exchanged through the plan's one test seam, an accepting
//!   `when:` answers before an input refusal, but `earlier_accepting_branches` returns nothing for
//!   a refusal by its kind, so the refusal's witness is one the accepting branch claims. The
//!   interpreter, reading the same exchanged order, fails that scenario.
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::{AdmittedSuite, Runner, SuiteProvenance};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(SpecSource::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every scenario of `suite` run on the model interpreter of `ir`, by id, where it did not pass.
fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    statuses
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

// ---- A1: a stored `exists: false` beside an input refusal -----------------------------------

const STORED: &str = include_str!("fixtures/related-guard-stored-reference.yaml");

/// `related-guard-stored-reference.yaml` with a `note` input and an input refusal on an empty one,
/// declared first.
fn stored_with_an_input_refusal() -> String {
    let text = STORED
        .replace(
            "errors:\n",
            "errors:\n  - {name: demo.tasks.EmptyNote, summary: The note is empty., fields: []}\n",
        )
        .replace(
            "  - name: demo.tasks.CompleteTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n    outcomes:\n",
            "  - name: demo.tasks.CompleteTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n      - {name: note, type: String}\n    outcomes:\n      - {name: empty-note, when: note == \"\", error: demo.tasks.EmptyNote}\n",
        );
    assert!(text.contains("empty-note"), "the fixture moved: {text}");
    text
}

/// A task stored as blocked by a task nobody added, then completed with an empty note, claiming
/// `claim` (an `outcome:` line and whatever follows it).
fn blocked_by_nobody(claim: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: demo.tasks\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n\
         arrange:\n  - instance: t\n    entity: demo.tasks.Task\n\
         timeline:\n\
         \x20 - at: 2026-01-05T09:00:00Z\n    command: demo.tasks.AddTask\n    \
         input: {{blocked_by: 00000000-0000-4000-8000-000000000009}}\n    outcome: added\n    \
         events:\n      - event: demo.tasks.TaskAdded\n    \
         capture: {{instance: t, event: demo.tasks.TaskAdded, field: task_id}}\n\
         \x20 - at: 2026-01-05T09:00:01Z\n    command: demo.tasks.CompleteTask\n    \
         input: {{task_id: {{$instance: t}}, note: ''}}\n{claim}"
    )
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

fn refusals(authoring: &Authoring) -> Vec<String> {
    authoring
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.code()))
        .collect()
}

/// A1a (runtime observation): for a stored blocker nobody added and an empty note, the model
/// interpreter answers the input refusal, not the stored `exists: false`.
#[test]
fn a1a_the_interpreter_answers_the_input_refusal_before_a_stored_missing_row() {
    let model = ir(&stored_with_an_input_refusal());
    let refused = authoring(
        &model,
        &blocked_by_nobody("    outcome: empty-note\n    error: {name: demo.tasks.EmptyNote}\n"),
    );
    assert!(refused.is_complete(), "{:?}", refusals(&refused));
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&model));
    suite.scenarios = refused.scenarios;
    assert_eq!(not_passed(model, &suite), Vec::<String>::new());
}

/// A1b: at base 3bfa27aed1 `not_taken` returned early for every `exists: false` on a command
/// reading a related row, so this act claiming the stored `blocker-missing` was accepted, although
/// the interpreter answers `empty-note` for it (A1a). Reading the precedence plan, the stored
/// `exists: false` answers after the input refusals, so the act is refused with `ESS-AUTHOR-041`
/// naming `empty-note`. Kept as a correction the plan brings (coordinator decision, wave 5 U2
/// adversary pass 1, A1); the changelog records it.
#[test]
fn a1b_a_claim_of_a_stored_missing_row_an_input_refusal_answers_is_refused() {
    let model = ir(&stored_with_an_input_refusal());
    let claimed = authoring(&model, &blocked_by_nobody("    outcome: blocker-missing\n"));
    let refused = refusals(&claimed);
    assert!(
        refused.len() == 1
            && refused[0].starts_with("ESS-AUTHOR-041")
            && refused[0].contains("`empty-note`")
            && refused[0].contains("`blocker-missing` is not taken"),
        "the act is refused because `empty-note` answers first: {refused:#?}"
    );
}

// ---- A2: input refusal and accepting phases exchanged -----------------------------------------

const GATE: &str = r"format: ess/22
system: demo
version: v1
domain: demo.gate
summary: A gate admits small groups and refuses large ones.
events:
  - {name: demo.gate.Admitted, fields: []}
  - {name: demo.gate.Waved, fields: []}
errors:
  - {name: demo.gate.TooMany, fields: []}
commands:
  - name: demo.gate.Admit
    input:
      - {name: count, type: Integer}
    outcomes:
      - {name: too-many, when: count > 5, error: demo.gate.TooMany}
      - {name: small, when: {all: [count > 0, count < 10]}, emits: [demo.gate.Admitted]}
      - {name: waved, emits: [demo.gate.Waved]}
";

/// The precedence order with the input-refusal and accepting phases exchanged.
fn refusals_after_accepting() -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| match phase {
        Phase::InputRefusal => Phase::Accepting,
        Phase::Accepting => Phase::InputRefusal,
        other => other,
    })
}

/// Sanity: under the precedence order the synthesized suite passes on the interpreter.
#[test]
fn a2_declared_order_suite_passes() {
    let model = ir(GATE);
    let result = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

/// With accepting read before the input refusals, `too-many`'s witness must step out of `small`
/// (`count >= 10`). The interpreter, reading the same order, passes every synthesized scenario.
#[test]
fn a2_exchanged_refusal_and_accepting_suite_passes_on_the_interpreter() {
    let model = ir(GATE);
    let result = with_phase_order(refusals_after_accepting(), || {
        ess_conformance::synthesize::synthesize(&model)
    });
    let failed = with_phase_order(refusals_after_accepting(), || {
        not_passed(model.clone(), &result.suite)
    });
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "every witness synthesis wrote under the exchanged order is answered as claimed by the \
         interpreter reading that order"
    );
}
