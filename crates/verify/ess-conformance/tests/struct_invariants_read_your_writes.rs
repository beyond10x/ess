//! A struct with type invariants published by a `read_your_writes` view is observed as a complete
//! subject by its declared fields (beyond10x/ess#730): the invariants stay the obligation of the
//! value-invariant scenarios, and the retained-result profile is unchanged.

use std::{cell::Cell, collections::BTreeMap};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{synthesize::synthesize, AdmittedSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
const MODEL: &str = r"format: ess/36
system: example
version: v1
domain: example.span
summary: A tracked span with a start and an end.
types:
  - name: example.span.Range
    kind: struct
    fields:
      - {name: start, type: Integer}
      - {name: end, type: Integer}
    invariants:
      - start >= 0
      - end >= start
entities:
  - name: example.span.Item
    identity: {name: item_id, type: String}
    fields:
      - {name: range, type: example.span.Range}
    lifecycle:
      initial: Open
      states: [Open, Stale]
      terminal: []
      transitions:
        - {name: mark_stale, from: [Open], to: Stale}
        - {name: confirm, from: [Stale], to: Open}
events:
  - name: example.span.ItemRead
    fields: [{name: item_id, type: String}]
  - name: example.span.ItemStale
    fields: [{name: item_id, type: String}]
  - name: example.span.ItemConfirmed
    fields: [{name: item_id, type: String}]
errors:
  - name: example.span.Exists
  - name: example.span.NoItem
  - name: example.span.NotOpen
  - name: example.span.NotStale
  - name: example.span.OtherRange
commands:
  - name: example.span.ReadItem
    input:
      - {name: item_id, type: String}
      - {name: range, type: example.span.Range}
    outcomes:
      - name: read
        creates: example.span.Item
        instance: item_id
        sets: {range: input.range}
        emits: [example.span.ItemRead]
        payload: {example.span.ItemRead: {item_id: input.item_id}}
      - {name: exists, existing_instance: true, error: example.span.Exists}
  - name: example.span.MarkStale
    input:
      - {name: item_id, type: String}
    outcomes:
      - name: marked
        moves: example.span.Item.mark_stale
        instance: item_id
        emits: [example.span.ItemStale]
        payload: {example.span.ItemStale: {item_id: input.item_id}}
      - {name: unknown, unknown_instance: true, error: example.span.NoItem}
      - {name: wrong-state, wrong_state: true, error: example.span.NotOpen}
  - name: example.span.Confirm
    input:
      - {name: item_id, type: String}
      - {name: end, type: Integer}
    outcomes:
      - name: other-range
        when_subject: {predicate: range.end != input.end}
        error: example.span.OtherRange
      - name: confirmed
        moves: example.span.Item.confirm
        instance: item_id
        emits: [example.span.ItemConfirmed]
        payload: {example.span.ItemConfirmed: {item_id: input.item_id}}
      - {name: unknown, unknown_instance: true, error: example.span.NoItem}
      - {name: wrong-state, wrong_state: true, error: example.span.NotStale}
views:
  - name: example.span.Items
    source: example.span.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: String}
      - {name: state, type: example.span.Item.State}
      - {name: range, type: example.span.Range}
";
const OTHER_RANGE: &str = "example.span.Confirm/outcome/other-range";
const RANGE: &str = "example.span.Range";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("span.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn the_constrained_struct_is_a_complete_subject_declaration() {
    let result = synthesize(&ir_of(MODEL));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let ids: Vec<String> = result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for wanted in [
        OTHER_RANGE,
        "example.span.Item/state/Open/refuses/example.span.Confirm",
        "example.span.Item/state/Stale/refuses/example.span.MarkStale",
        "example.span.ReadItem/outcome/exists",
    ] {
        assert!(ids.iter().any(|id| id == wanted), "{wanted} in {ids:#?}");
    }
    let control = synthesize(&ir_of(&MODEL.replace(
        "    invariants:\n      - start >= 0\n      - end >= start\n",
        "",
    )));
    assert_eq!(
        result.suite.provenance.suite_version,
        control.suite.provenance.suite_version
    );
    let mut seen = 0;
    for step in result.suite.scenarios.values().flat_map(|s| &s.steps) {
        if let ScenarioStep::SnapshotCompleteSubject { shape, .. } = step {
            let value = serde_json::to_value(shape).unwrap();
            let declared = &value["declarations"][RANGE];
            assert_eq!(declared["kind"], "struct", "{value}");
            assert_eq!(declared.as_object().unwrap().len(), 2, "{declared}");
            seen += 1;
        }
    }
    assert!(seen > 0);
}

#[test]
fn the_interpreted_target_passes_every_scenario() {
    let model = ir_of(MODEL);
    let suite = synthesize(&model).suite;
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    for run in &report.scenarios {
        assert_eq!(
            run.status,
            Status::Passed,
            "{}: {:?}",
            run.scenario,
            run.checks
        );
    }
}

struct ChangesRangeEnd {
    inner: ess_conformance::interpret::Interpreted,
    changed: Cell<bool>,
}

impl ConformanceTarget for ChangesRangeEnd {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.changed.set(false);
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() == "example.span.Confirm" {
            self.changed.set(true);
        }
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        if self.changed.get() {
            for row in &mut result.rows {
                if let Some(Node::Map(range)) = row.get_mut("range") {
                    range.insert("end".into(), Node::Number(9_999_i64.into()));
                }
            }
        }
        Ok(result)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

#[test]
fn a_target_that_changes_the_stored_range_fails_the_refusal_scenario() {
    let model = ir_of(MODEL);
    let suite = synthesize(&model).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = ChangesRangeEnd {
        inner: ess_conformance::interpret::Interpreted::for_model(model),
        changed: Cell::new(false),
    };
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let statuses: BTreeMap<String, Status> = report
        .scenarios
        .iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect();
    assert_eq!(statuses[OTHER_RANGE], Status::Failed, "{statuses:#?}");
}

#[test]
fn an_ignored_undeclared_fields_struct_still_refuses_and_names_the_second_view() {
    let ignored = MODEL.replace(
        "    kind: struct\n    fields:\n      - {name: start",
        "    kind: struct\n    undeclared_fields: ignored\n    fields:\n      - {name: start",
    );
    assert_ne!(ignored, MODEL);
    let result = synthesize(&ir_of(&ignored));
    let first = result.refusals.first().expect("the ignored struct refuses");
    let text = format!("{:?}", result.refusals);
    assert!(text.contains("undeclared_fields: ignored"), "{text}");
    let help = first.hint();
    assert!(help.contains("not `read_your_writes`"), "{help}");

    let moved = ignored.replace(
        "      - {name: state, type: example.span.Item.State}\n      - {name: range, type: example.span.Range}\n",
        "      - {name: state, type: example.span.Item.State}\n",
    );
    assert_ne!(moved, ignored);
    let second = format!(
        "{moved}  - name: example.span.ItemsLater\n    source: example.span.Item\n    consistency: eventual\n    fields:\n      - {{name: item_id, type: String}}\n      - {{name: range, type: example.span.Range}}\n"
    );
    let repaired = synthesize(&ir_of(&second));
    assert!(repaired.refusals.is_empty(), "{:#?}", repaired.refusals);
}
