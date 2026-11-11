//! A branch that copies an input into an entity, with no guard keeping the input inside the
//! entity's invariant, leaves the model without an outcome for an input outside it
//! (beyond10x/ess#738). Synthesis names that hole as `ESS-SYNTH-025` beside the `invariant/after`
//! scenario; it never writes a scenario expecting a refusal the model does not declare.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::coverage::{Origins, Scope};
use ess_conformance::coverage_build;
use ess_conformance::synthesize::{synthesize, RefusalCause, Synthesis};
use ess_conformance::{AdmittedSuite, ScenarioId};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const SET: &str = "bounded.counter.Counter/invariant/after/bounded.counter.Set/set";
const RESET: &str = "bounded.counter.Counter/invariant/after/bounded.counter.Set/reset";

fn model(extra_outcome: &str, guard: &str) -> String {
    format!(
        "format: ess/35
system: bounded
version: v1
summary: A counter whose value stays at or below eight.
domain: bounded.counter
types:
  - {{name: bounded.counter.CounterId, kind: newtype, of: String}}
entities:
  - name: bounded.counter.Counter
    identity: {{name: counter_id, type: bounded.counter.CounterId}}
    fields:
      - {{name: value, type: Integer}}
    invariants:
      - value <= 8
    lifecycle: {{initial: Live, states: [Live], terminal: [Live]}}
actors:
  - name: bounded.counter.Operator
    attributes: []
    may: [bounded.counter.Set, bounded.counter.Counters]
errors:
  - {{name: bounded.counter.ValueTooLarge, summary: The value is above eight., fields: []}}
events:
  - name: bounded.counter.CounterSet
    fields:
      - {{name: counter_id, type: bounded.counter.CounterId}}
commands:
  - name: bounded.counter.Set
    input:
      - {{name: counter_id, type: bounded.counter.CounterId}}
      - {{name: value, type: Integer}}
    outcomes:
{extra_outcome}      - name: set
        unknown_instance: true
{guard}        creates: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value}}
        emits: [bounded.counter.CounterSet]
        payload:
          bounded.counter.CounterSet: {{counter_id: input.counter_id}}
      - name: reset
{guard}        updates: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value}}
        emits: [bounded.counter.CounterSet]
        payload:
          bounded.counter.CounterSet: {{counter_id: input.counter_id}}
views:
  - name: bounded.counter.Counters
    source: bounded.counter.Counter
    consistency: read_your_writes
    fields:
      - {{name: counter_id, type: bounded.counter.CounterId}}
      - {{name: value, type: Integer}}
"
    )
}

const TOO_LARGE: &str = "      - name: too-large
        when: value > 8
        error: bounded.counter.ValueTooLarge
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("counter.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn broken(synthesis: &Synthesis) -> Vec<(String, String)> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| matches!(refusal.cause, RefusalCause::InvariantInputUnanswered { .. }))
        .map(|refusal| {
            (
                refusal.scenario.as_ref().unwrap().to_string(),
                refusal.to_string(),
            )
        })
        .collect()
}

#[test]
fn an_unguarded_input_is_refused_once_per_branch_beside_the_invariant_scenarios() {
    let synthesis = synthesize(&ir(&model("", "")));
    let found = broken(&synthesis);
    let ids: Vec<&str> = found.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, [SET, RESET], "{found:#?}");
    for (_, text) in &found {
        assert!(text.contains("ESS-SYNTH-025"), "{text}");
        assert!(text.contains("value <= 8"), "{text}");
        assert!(text.contains("value: 9"), "{text}");
        assert!(
            text.contains(
                "guard the branch on the input, or declare a refusal branch whose `when:` answers it"
            ),
            "{text}"
        );
    }
    for id in [SET, RESET] {
        assert!(
            synthesis
                .suite
                .scenarios
                .contains_key(&ScenarioId::parse(id).unwrap()),
            "the invariant scenario {id} still stands"
        );
    }
}

#[test]
fn a_declared_refusal_answers_the_input_and_is_witnessed_with_nothing_changed() {
    let synthesis = synthesize(&ir(&model(TOO_LARGE, "")));
    assert_eq!(broken(&synthesis), [], "{:#?}", synthesis.refusals);
    let id = ScenarioId::parse("bounded.counter.Set/outcome/too-large").unwrap();
    let scenario = synthesis
        .suite
        .scenarios
        .get(&id)
        .expect("the refusal scenario");
    let text = serde_json::to_string(scenario).unwrap();
    assert!(text.contains("\"value\":9"), "{text}");
    assert!(text.contains("too-large"), "{text}");
    assert!(text.contains("expect_complete_subject_unchanged"), "{text}");
}

#[test]
fn a_branch_whose_own_guard_keeps_the_input_inside_gets_no_refusal() {
    let guard = "        when: value <= 8\n";
    let guarded = model("", guard)
        .replace("        unknown_instance: true\n", "")
        .replace(
            "views:\n",
            "      - {name: otherwise, error: bounded.counter.ValueTooLarge}\nviews:\n",
        );
    let synthesis = synthesize(&ir(&guarded));
    assert_eq!(broken(&synthesis), [], "{:#?}", synthesis.refusals);
}

#[test]
fn a_coverage_suite_carrying_the_code_is_written_at_the_major_that_admits_it() {
    let ir = ir(&model("", ""));
    let input = coverage_build::build(&ir, &[], Scope::System, Origins::GeneratedAndAuthored)
        .expect("the coverage suite builds");
    let suite = input.selected();
    assert_eq!(
        suite.suite().provenance.suite_version.to_string(),
        "ess-conformance/65"
    );
    assert!(suite
        .coverage()
        .unwrap()
        .refused
        .iter()
        .any(|refusal| refusal.code == "ESS-SYNTH-025"));
    let older = suite
        .original_json()
        .replace("ess-conformance/65", "ess-conformance/63");
    let error = AdmittedSuite::from_json(&older).expect_err("a reader below /65 refuses it");
    assert!(error.to_string().contains("suite/65"), "{error}");
}
