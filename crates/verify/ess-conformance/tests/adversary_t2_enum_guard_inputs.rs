//! Adversary, turn 2: the `exists: false` branch beside input-guarded refusals that take the plain
//! input, over connectives, several enum inputs, an integer bound, a boolean and a refusal declared
//! after the branch (beyond10x/ess#731).
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const NO_DECISION: &str = "example.marks.RecordMark/outcome/no-decision";

fn spec(variants: &str, before: &str, after: &str) -> String {
    format!(
        r"format: ess/29
system: example
version: v1
domain: example.marks
summary: A mark is recorded against a decision that must already exist.
types:
  - {{name: example.marks.MarkId, kind: newtype, of: Uuid}}
  - {{name: example.marks.DecisionId, kind: newtype, of: Uuid}}
  - name: example.marks.Kind
    kind: enum
    variants:
{variants}
  - name: example.marks.Level
    kind: enum
    variants:
      - {{name: Low, wire: low}}
      - {{name: High, wire: high}}
entities:
  - name: example.marks.Decision
    identity: {{name: decision_id, type: example.marks.DecisionId}}
    fields: []
    lifecycle: {{initial: Live, states: [Live], terminal: [Live], transitions: []}}
  - name: example.marks.Mark
    identity: {{name: mark_id, type: example.marks.MarkId}}
    fields:
      - {{name: kind, type: example.marks.Kind}}
      - {{name: level, type: example.marks.Level}}
      - {{name: amount, type: Integer}}
      - {{name: urgent, type: Boolean}}
      - {{name: limit, type: Integer}}
      - {{name: note, type: Optional<String>}}
      - {{name: due_at, type: Timestamp}}
      - {{name: decision, type: example.marks.DecisionId}}
    lifecycle: {{initial: Live, states: [Live], terminal: [Live], transitions: []}}
    relations:
      - {{name: decided_by, kind: references, target: example.marks.Decision, cardinality: one, via: decision}}
errors:
  - {{name: example.marks.NotPermitted, summary: Not permitted., fields: []}}
  - {{name: example.marks.Exists, summary: The row already exists., fields: []}}
  - {{name: example.marks.NoDecision, summary: No such decision., fields: []}}
events:
  - name: example.marks.DecisionRecorded
    fields: [{{name: decision_id, type: example.marks.DecisionId}}]
  - name: example.marks.MarkRecorded
    fields: [{{name: mark_id, type: example.marks.MarkId}}]
actors:
  - name: example.marks.Clerk
    may: [example.marks.RecordDecision, example.marks.RecordMark]
commands:
  - name: example.marks.RecordDecision
    input:
      - {{name: decision_id, type: example.marks.DecisionId}}
    outcomes:
      - {{name: exists, existing_instance: true, error: example.marks.Exists}}
      - name: recorded
        creates: example.marks.Decision
        instance: decision_id
        emits: [example.marks.DecisionRecorded]
        payload: {{example.marks.DecisionRecorded: {{decision_id: input.decision_id}}}}
  - name: example.marks.RecordMark
    input:
      - {{name: mark_id, type: example.marks.MarkId}}
      - {{name: kind, type: example.marks.Kind}}
      - {{name: level, type: example.marks.Level}}
      - {{name: amount, type: Integer}}
      - {{name: urgent, type: Boolean}}
      - {{name: limit, type: Integer}}
      - {{name: note, type: Optional<String>}}
      - {{name: due_at, type: Timestamp}}
      - {{name: decision, type: example.marks.DecisionId}}
    outcomes:
{before}      - {{name: exists, existing_instance: true, error: example.marks.Exists}}
      - name: no-decision
        when_related: {{via: input.decision, exists: false}}
        error: example.marks.NoDecision
{after}      - name: recorded
        creates: example.marks.Mark
        instance: mark_id
        emits: [example.marks.MarkRecorded]
        payload: {{example.marks.MarkRecorded: {{mark_id: input.mark_id}}}}
        sets: {{kind: input.kind, level: input.level, amount: input.amount, urgent: input.urgent, limit: input.limit, note: input.note, due_at: input.due_at, decision: input.decision}}
views:
  - name: example.marks.Marks
    source: example.marks.Mark
    consistency: read_your_writes
    fields:
      - {{name: mark_id, type: example.marks.MarkId}}
      - {{name: kind, type: example.marks.Kind}}
  - name: example.marks.Decisions
    source: example.marks.Decision
    consistency: read_your_writes
    fields:
      - {{name: decision_id, type: example.marks.DecisionId}}
"
    )
}

const TWO: &str =
    "      - {name: Satisfied, wire: satisfied}\n      - {name: Unsupported, wire: unsupported}\n";
const THREE: &str = "      - {name: Satisfied, wire: satisfied}\n      - {name: Unsupported, wire: unsupported}\n      - {name: Partial, wire: partial}\n";

fn refusal(name: &str, guard: &str) -> String {
    format!(
        "      - name: {name}\n        when: {guard}\n        error: example.marks.NotPermitted\n"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble(vec![(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Synthesizes `text`, asserts no refusal and a no-decision scenario, runs the suite against the
/// interpreted target and asserts every scenario passes; returns the no-decision send rendered.
fn arranged(text: &str) -> String {
    let result = ess_conformance::synthesize::synthesize(&ir(text));
    let refusals: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, Vec::<String>::new(), "{text}");
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == NO_DECISION)
        .map(|(_, scenario)| scenario)
        .unwrap_or_else(|| panic!("no scenario {NO_DECISION}"));
    let sent: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "example.marks.RecordMark" =>
            {
                Some(input)
            }
            _ => None,
        })
        .collect();
    let last = format!(
        "{:?}",
        sent.last().unwrap_or_else(|| panic!("{scenario:#?}"))
    );
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(text)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    let failed: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}\n{last}");
    last
}

fn refused(text: &str) -> String {
    let result = ess_conformance::synthesize::synthesize(&ir(text));
    let missing: Vec<_> = result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == NO_DECISION)
        })
        .collect();
    let all: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(missing.len(), 1, "{all:#?}");
    assert_eq!(
        missing[0].code().to_string(),
        "ESS-SYNTH-003",
        "{}",
        missing[0]
    );
    missing[0].to_string()
}

#[test]
fn adversary_t2_enum_guard_base_control() {
    arranged(&spec(TWO, "", ""));
}

#[test]
fn adversary_t2_enum_guard_conjunction_over_two_enum_inputs() {
    let sent = arranged(&spec(
        TWO,
        &refusal(
            "not-permitted",
            r#"{all: ["kind == Satisfied", "level == Low"]}"#,
        ),
        "",
    ));
    assert!(
        sent.contains("Unsupported") || sent.contains("High"),
        "{sent}"
    );
}

#[test]
fn adversary_t2_enum_guard_disjunction_leaves_third_variant() {
    let sent = arranged(&spec(
        THREE,
        &refusal(
            "not-permitted",
            r#"{any: ["kind == Satisfied", "kind == Unsupported"]}"#,
        ),
        "",
    ));
    assert!(sent.contains("Partial"), "{sent}");
}

#[test]
fn adversary_t2_enum_guard_two_refusals_on_two_enum_inputs() {
    let both = format!(
        "{}{}",
        refusal("not-permitted", "kind == Satisfied"),
        refusal(
            "too-low",
            r#"{all: ["kind == Unsupported", "level == Low"]}"#
        )
    );
    let sent = arranged(&spec(TWO, &both, ""));
    assert!(
        sent.contains("Unsupported") && sent.contains("High"),
        "{sent}"
    );
}

#[test]
fn adversary_t2_enum_guard_integer_and_boolean_bounds() {
    let all = format!(
        "{}{}{}",
        refusal("not-permitted", "kind == Satisfied"),
        refusal("too-small", "amount < 1000"),
        refusal("not-urgent", "urgent == false"),
    );
    arranged(&spec(TWO, &all, ""));
}

#[test]
fn adversary_t2_enum_guard_tautology_names_the_refusal() {
    let text = refused(&spec(
        TWO,
        &refusal(
            "not-permitted",
            r#"{any: ["kind == Satisfied", "kind == Unsupported"]}"#,
        ),
        "",
    ));
    assert!(text.contains("taken first by `not-permitted`"), "{text}");
}

#[test]
fn adversary_t2_enum_guard_every_value_refused_names_the_first_taker() {
    let both = format!(
        "{}{}",
        refusal("not-permitted", "kind == Satisfied"),
        refusal("not-supported", "kind == Unsupported")
    );
    let text = refused(&spec(TWO, &both, ""));
    assert!(text.contains("taken first by `not-permitted`"), "{text}");
    assert!(!text.contains("arranges none"), "{text}");
}

#[test]
fn adversary_t2_enum_guard_two_input_comparison() {
    let all = format!(
        "{}{}",
        refusal("not-permitted", "kind == Satisfied"),
        refusal("over-limit", "amount >= limit"),
    );
    arranged(&spec(TWO, &all, ""));
}

#[test]
fn adversary_t2_enum_guard_optional_defined() {
    let all = format!(
        "{}{}",
        refusal("not-permitted", "kind == Satisfied"),
        refusal("noted", "defined(note)"),
    );
    arranged(&spec(TWO, &all, ""));
}

#[test]
fn adversary_t2_enum_guard_timestamp_against_now() {
    let all = format!(
        "{}{}",
        refusal("not-permitted", "kind == Satisfied"),
        refusal("overdue", "due_at < now"),
    );
    // The interpreted target answers a now-relative send Unsupported (beside this unit), so the
    // send itself is asserted: not `overdue`, not `not-permitted`.
    let text = spec(TWO, &all, "");
    let result = ess_conformance::synthesize::synthesize(&ir(&text));
    let refusals: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, Vec::<String>::new());
    let sent = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == NO_DECISION)
        .and_then(|(_, scenario)| {
            scenario.steps.iter().rev().find_map(|step| match step {
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "example.marks.RecordMark" =>
                {
                    Some(format!("{input:?}"))
                }
                _ => None,
            })
        })
        .unwrap_or_else(|| panic!("no send for {NO_DECISION}"));
    assert!(sent.contains("Unsupported"), "{sent}");
    assert!(
        sent.contains("NowOffset { seconds: ") && !sent.contains("NowOffset { seconds: -"),
        "{sent}"
    );
}

/// The acceptance quotes the wording: every such input taken first by `<branch>` (<guard>).
#[test]
fn adversary_t2_enum_guard_refusal_text_names_branch_and_guard() {
    let both = format!(
        "{}{}",
        refusal("not-permitted", "kind == Satisfied"),
        refusal("not-supported", "kind == Unsupported")
    );
    let text = refused(&spec(TWO, &both, ""));
    assert!(
        text.contains("every such input taken first by `not-permitted` (kind == Satisfied)"),
        "{text}"
    );
    assert!(
        text.contains("none of: kind == Satisfied, kind == Unsupported"),
        "{text}"
    );
}
