//! A binding condition and a binding selection compare a Boolean leaf with `true` or `false` from
//! `ess/27` (downstream report, 2026-10-18), and nothing else about the bounded fragment moves: a
//! text literal against a Boolean stays a type mismatch, a bare Boolean path stays outside it, and
//! below `ess/27` the comparison is refused naming `ess/27`.

use std::collections::BTreeMap;

use ess_domain::binding::condition::{ConditionPlan, Leaf};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::{FormatVersion, Source};
use ess_domain::Specification;
use ess_primitives::node::Node;
use ess_primitives::predicate::Truth;

fn document(format: &str, condition: &str, selection: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.calls
types:
  - name: demo.calls.Leg
    kind: struct
    fields:
      - {{name: id, type: String}}
      - {{name: held, type: Boolean}}
events:
  - name: demo.calls.CallFinished
    fields:
      - {{name: call_id, type: String}}
      - {{name: bridged, type: Optional<Boolean>}}
      - {{name: legs, type: List<demo.calls.Leg>}}
  - name: demo.calls.Recorded
    fields: [{{name: leg_id, type: Optional<String>}}]
commands:
  - name: demo.calls.Record
    input: [{{name: leg_id, type: Optional<String>}}]
    outcomes:
      - name: recorded
        emits: [demo.calls.Recorded]
        payload: {{demo.calls.Recorded: {{leg_id: input.leg_id}}}}
bindings:
  - id: record
    when:
      event: demo.calls.CallFinished
      where: {condition}
    selection_inputs:
      - {{name: legs, from: event.legs, as: List<demo.calls.Leg>}}
    selections:
      - name: free
        first: {{in: legs, where: {selection}}}
    invoke: {{command: demo.calls.Record}}
    mapping:
      leg_id: {{selection: free, path: [id]}}
    delivery: at_least_once
    on_failure: drop
"
    )
}

fn validate(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("calls.yaml"), raw)]).map_err(|e| e.to_string())
}

fn plan(spec: &Specification, format: FormatVersion) -> ConditionPlan {
    let binding = spec.bindings().values().next().expect("a binding");
    let event = spec
        .events()
        .get(binding.cause.event().expect("an event cause"))
        .expect("the event");
    ConditionPlan::resolve(
        binding.condition.as_ref().expect("a condition"),
        event,
        &spec.system().types,
        "",
        format,
    )
    .expect("the condition resolves")
}

#[test]
fn a_boolean_comparison_validates_from_ess_27_and_is_evaluated() {
    let spec = validate(&document(
        "ess/27",
        "[event.bridged == false]",
        "item.held != true",
    ))
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let plan = plan(&spec, FormatVersion::V27);
    assert_eq!(plan.reads["event.bridged"].leaf, Leaf::Boolean);
    let payload = |bridged: Option<bool>| {
        let mut payload = BTreeMap::new();
        if let Some(flag) = bridged {
            payload.insert("bridged".to_owned(), Node::Bool(flag));
        }
        payload
    };
    assert_eq!(plan.evaluate(&payload(Some(false))), Ok(Truth::True));
    assert_eq!(plan.evaluate(&payload(Some(true))), Ok(Truth::False));
    assert_eq!(plan.evaluate(&payload(None)), Ok(Truth::Unknown));
    // A definite comparison proves its Optional path present, as any comparison does.
    assert!(plan.proves_present().contains(&vec!["bridged".to_owned()]));
}

#[test]
fn below_ess_27_both_comparisons_are_refused_naming_ess_27() {
    let errors = validate(&document(
        "ess/26",
        "[event.bridged == false]",
        "item.held == false",
    ))
    .expect_err("refused below ess/27");
    assert_eq!(
        errors.matches("unsupported_format_version").count(),
        2,
        "the condition and the selection: {errors}"
    );
    assert_eq!(errors.matches("ess/27").count(), 4, "{errors}");
}

#[test]
fn below_ess_27_a_boolean_read_by_defined_keeps_its_other_leaf() {
    let spec = validate(&document(
        "ess/26",
        "defined(event.bridged)",
        "item.id == a",
    ))
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    assert_eq!(
        plan(&spec, FormatVersion::V26).reads["event.bridged"].leaf,
        Leaf::Other
    );
}

#[test]
fn a_text_literal_against_a_boolean_leaf_and_a_bare_path_stay_refused() {
    let quoted = validate(&document(
        "ess/27",
        "[event.bridged == \"false\"]",
        "item.id == a",
    ))
    .expect_err("text against a Boolean");
    assert!(
        quoted.contains("type_mismatch") && quoted.contains("compare it with `true` or `false`"),
        "{quoted}"
    );
    let bare = validate(&document("ess/27", "event.bridged", "item.id == a"))
        .expect_err("a bare path is no comparison");
    assert!(bare.contains("unsupported_construct"), "{bare}");
    let selected = validate(&document(
        "ess/27",
        "defined(event.bridged)",
        "item.held == \"false\"",
    ))
    .expect_err("text against a Boolean item leaf");
    assert!(selected.contains("binding.record.selections"), "{selected}");
}
