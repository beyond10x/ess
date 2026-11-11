//! Adversary pass 1 on beyond10x/ess#730: a constrained struct in a `read_your_writes` view, attacked
//! through nesting, `Optional` and `List` positions, an `Optional` member its invariant reads, a
//! `reading:` member, the retained-result profile, and targets that change one member of the row.

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
const RANGE_FIELD: &str = "      - {name: range, type: example.span.Range}\n";

fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the model carries {from:?}");
    model.replace(from, to)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("span.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

/// A struct with invariants whose own member is another struct with invariants.
fn nested(outer_invariants: bool) -> String {
    let outer = format!(
        "  - name: example.span.Outer\n    kind: struct\n    fields:\n      - {{name: inner, type: example.span.Range}}\n      - {{name: tag, type: Integer}}\n{}entities:\n",
        if outer_invariants {
            "    invariants:\n      - tag >= 0\n"
        } else {
            ""
        }
    );
    let model = with(MODEL, "entities:\n", &outer);
    let model = with(
        &model,
        "{name: range, type: example.span.Range}",
        "{name: range, type: example.span.Outer}",
    );
    with(
        &model,
        "range.end != input.end",
        "range.inner.end != input.end",
    )
}

/// The constrained struct also published as an `Optional` and as a `List` member of the row.
fn optional_and_list() -> String {
    let model = with(
        MODEL,
        RANGE_FIELD,
        &format!("{RANGE_FIELD}      - {{name: maybe, type: Optional<example.span.Range>}}\n      - {{name: many, type: List<example.span.Range>}}\n"),
    );
    with(
        &model,
        "sets: {range: input.range}",
        "sets: {range: input.range, maybe: input.maybe, many: input.many}",
    )
}

/// The struct's invariant reads an `Optional` member of it.
fn optional_member() -> String {
    let model = with(
        MODEL,
        "      - {name: end, type: Integer}\n    outcomes:",
        "      - {name: start, type: Integer}\n    outcomes:",
    );
    let model = with(
        &model,
        "range.end != input.end",
        "range.start != input.start",
    );
    with(
        &model,
        "      - {name: end, type: Integer}\n    invariants:\n      - start >= 0\n      - end >= start\n",
        "      - {name: end, type: Optional<Integer>}\n    invariants:\n      - start >= 0\n      - {any: [\"missing(end)\", \"end >= start\"]}\n",
    )
}

/// The constrained struct carries a member whose type declares a `reading:`.
fn reading_member() -> String {
    let model = with(
        MODEL,
        "entities:\n",
        "  - name: example.span.Stamp\n    kind: newtype\n    of: String\n    reading:\n      encoding: offset_date_time_text\n      origins: [{role: producer_process, offset: encoded_offset}]\nentities:\n",
    );
    with(
        &model,
        "      - {name: end, type: Integer}\n    invariants:",
        "      - {name: end, type: Integer}\n      - {name: at, type: example.span.Stamp}\n    invariants:",
    )
}

fn snapshot_steps(model: &str) -> usize {
    let result = synthesize(&ir_of(model));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let mut seen = 0;
    for step in result.suite.scenarios.values().flat_map(|s| &s.steps) {
        if let ScenarioStep::SnapshotCompleteSubject { shape, .. } = step {
            let value = serde_json::to_value(shape).unwrap();
            let declared = &value["declarations"]["example.span.Range"];
            assert_eq!(declared["kind"], "struct", "{value}");
            assert!(declared.get("invariants").is_none(), "{declared}");
            seen += 1;
        }
    }
    seen
}

fn statuses<T: ConformanceTarget>(model: &str, target: &T) -> BTreeMap<String, Status> {
    let suite = synthesize(&ir_of(model)).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report();
    report
        .scenarios
        .iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn interpreted_passes_all(model: &str) {
    let target = ess_conformance::interpret::Interpreted::for_model(ir_of(model));
    let statuses = statuses(model, &target);
    assert!(statuses.contains_key(OTHER_RANGE), "{statuses:#?}");
    let not_passed: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(not_passed.is_empty(), "{not_passed:#?}");
}

#[test]
fn a_nested_constrained_struct_is_observed_and_passes() {
    for outer_invariants in [true, false] {
        let model = nested(outer_invariants);
        assert!(
            snapshot_steps(&model) > 0,
            "outer invariants {outer_invariants}"
        );
        interpreted_passes_all(&model);
    }
}

#[test]
fn optional_and_list_constrained_struct_fields_are_observed_and_pass() {
    let model = optional_and_list();
    assert!(snapshot_steps(&model) > 0);
    interpreted_passes_all(&model);
}

#[test]
fn an_invariant_over_an_optional_member_is_observed_and_passes() {
    let model = optional_member();
    assert!(snapshot_steps(&model) > 0);
    interpreted_passes_all(&model);
}

#[test]
fn a_reading_member_still_refuses_and_the_second_view_repair_clears_it() {
    let model = reading_member();
    let result = synthesize(&ir_of(&model));
    let text = format!("{:?}", result.refusals);
    assert!(
        text.contains("invariant/reading observer is unsupported"),
        "{text}"
    );
    assert!(
        result
            .refusals
            .iter()
            .any(|refusal| refusal.hint().contains("not `read_your_writes`")),
        "{text}"
    );
    // website/docs/guides/verify/observations.md: "a type with a `reading:` ... refuses, and moving
    // that field to a second view that is not `read_your_writes` clears the refusal."
    let moved = with(
        &model,
        "      - {name: state, type: example.span.Item.State}\n      - {name: range, type: example.span.Range}\n",
        "      - {name: state, type: example.span.Item.State}\n",
    );
    let second = format!(
        "{moved}  - name: example.span.ItemsLater\n    source: example.span.Item\n    consistency: eventual\n    fields:\n      - {{name: item_id, type: String}}\n      - {{name: range, type: example.span.Range}}\n"
    );
    let repaired = synthesize(&ir_of(&second));
    assert!(repaired.refusals.is_empty(), "{:#?}", repaired.refusals);
}

const RETAINED_TYPES: &str = "domain: retained.core\ntypes:\n  - name: retained.core.Range\n    kind: struct\n    fields:\n      - {name: start, type: Integer}\n      - {name: end, type: Integer}\n    invariants:\n      - start >= 0\n      - end >= 0\n";

#[test]
fn a_constrained_struct_in_a_retained_response_still_refuses() {
    let model = with(
        &with(
            include_str!("fixtures/retained-replay.yaml"),
            "domain: retained.core\n",
            RETAINED_TYPES,
        ),
        "{name: number, type: Integer}",
        "{name: number, type: retained.core.Range}",
    );
    let observe = |text: &str| {
        let ir = ir_of(text);
        let command = &ir.commands()[&"retained.core.Seed".parse().unwrap()];
        let replayed = command
            .outcomes
            .iter()
            .find(|outcome| outcome.name.as_str() == "replayed")
            .unwrap();
        ess_conformance::replay::Observation::of(
            &ir,
            command,
            replayed,
            ess_conformance::InstanceName::new("subject").unwrap(),
        )
        .map(|_| ())
    };
    let refusal = observe(&model).expect_err("the retained-result profile stays strict");
    assert!(
        refusal.contains("replay response invariant/reading observer is unsupported"),
        "{refusal}"
    );
    // Control: the same struct without invariants is admitted, so the refusal is the invariants'.
    let control = with(
        &model,
        "    invariants:\n      - start >= 0\n      - end >= 0\n",
        "",
    );
    observe(&control).expect("an unconstrained struct response is admitted");
    let result = synthesize(&ir_of(&model));
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().ends_with("/outcome/replayed")),
        "{:#?}",
        result.suite.scenarios.keys().collect::<Vec<_>>()
    );
}

/// The interpreter, with one change to every row it returns once `Confirm` has run `from` times.
///
/// `other-range` runs `Confirm` twice: first as an unknown-instance probe before the row exists,
/// then as the refusal itself after the complete-subject snapshot. `from: 2` changes only what the
/// row reads while answering the refusal, so only the complete-subject comparison can see it.
struct Mutating {
    inner: ess_conformance::interpret::Interpreted,
    confirms: Cell<usize>,
    from: usize,
    mutate: fn(&mut ViewRow),
}

impl ConformanceTarget for Mutating {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.confirms.set(0);
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
            self.confirms.set(self.confirms.get() + 1);
        }
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        if self.confirms.get() >= self.from {
            for row in &mut result.rows {
                (self.mutate)(row);
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

const COMPLETE_CHECK: &str = "subject snapshot example.span.Items";
/// The complete subject's shape admission: a row that drops or adds a struct member fails here.
const COMPLETE_SHAPE_CHECK: &str = "complete subject snapshot example.span.Items";
const PLAIN_VIEW_CHECK: &str = "view example.span.Items";

/// The `other-range` status and the `about` of every check that did not pass.
fn other_range_under(model: &str, from: usize, mutate: fn(&mut ViewRow)) -> (Status, Vec<String>) {
    let target = Mutating {
        inner: ess_conformance::interpret::Interpreted::for_model(ir_of(model)),
        confirms: Cell::new(0),
        from,
        mutate,
    };
    let suite = synthesize(&ir_of(model)).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == OTHER_RANGE)
        .expect("other-range ran");
    let failed = run
        .checks
        .iter()
        .filter(|check| check.status != Status::Passed)
        .map(|check| check.about.clone())
        .collect();
    (run.status, failed)
}

/// The change reaches only the complete-subject comparison, and that comparison fails.
fn fails_on_the_complete_subject(model: &str, mutate: fn(&mut ViewRow), how: &str) {
    let (status, failed) = other_range_under(model, 2, mutate);
    assert_eq!(status, Status::Failed, "{how}: {failed:?}");
    assert_eq!(failed.len(), 1, "{how}: {failed:?}");
    assert!(
        failed[0] == COMPLETE_CHECK || failed[0] == COMPLETE_SHAPE_CHECK,
        "{how}: {failed:?}"
    );
}

/// One change to a returned row.
type MemberChange = fn(&mut ViewRow);

fn range_map(row: &mut ViewRow) -> &mut BTreeMap<String, Node> {
    match row.get_mut("range") {
        Some(Node::Map(range)) => range,
        other => panic!("range is a map: {other:?}"),
    }
}

fn change_end(row: &mut ViewRow) {
    range_map(row).insert("end".into(), Node::Number(9_999_i64.into()));
}

#[test]
fn a_target_that_changes_one_member_while_refusing_fails_the_complete_subject() {
    let cases: [(&str, MemberChange); 4] = [
        ("end changed", change_end),
        ("start changed", |row| {
            range_map(row).insert("start".into(), Node::Number(4_242_i64.into()));
        }),
        ("extra member", |row| {
            range_map(row).insert("extra".into(), Node::Number(1_i64.into()));
        }),
        ("end dropped", |row| {
            range_map(row).remove("end");
        }),
    ];
    for (how, mutate) in cases {
        fails_on_the_complete_subject(MODEL, mutate, how);
    }
}

/// The unit's own `ChangesRangeEnd` changes `range.end` from the first `Confirm`, the probe that
/// runs before the row exists. Its `Failed` comes from the plain `ExpectView` contains-check on
/// `range`, and the complete-subject comparison passes: snapshot and comparison both read the
/// changed row. Kept as evidence for the report; it pins no behaviour.
#[test]
fn the_unit_change_target_fails_only_at_the_plain_view_check() {
    let (status, failed) = other_range_under(MODEL, 1, change_end);
    assert_eq!(status, Status::Failed, "{failed:?}");
    assert!(
        failed.iter().any(|about| about == PLAIN_VIEW_CHECK),
        "{failed:?}"
    );
    assert!(
        !failed.iter().any(|about| about == COMPLETE_CHECK),
        "{failed:?}"
    );
}

#[test]
fn a_target_that_changes_a_nested_member_while_refusing_fails_the_complete_subject() {
    fails_on_the_complete_subject(
        &nested(true),
        |row| {
            let Some(Node::Map(inner)) = range_map(row).get_mut("inner") else {
                panic!("inner is a map")
            };
            inner.insert("end".into(), Node::Number(9_999_i64.into()));
        },
        "nested end",
    );
}

fn flip_maybe(row: &mut ViewRow) {
    let replacement = match row.get("maybe") {
        Some(Node::Map(_)) => Node::Null,
        _ => Node::Map(BTreeMap::from([
            ("start".into(), Node::Number(0_i64.into())),
            ("end".into(), Node::Number(7_i64.into())),
        ])),
    };
    row.insert("maybe".into(), replacement);
}

fn grow_many(row: &mut ViewRow) {
    let Some(Node::Seq(many)) = row.get_mut("many") else {
        panic!("many is a list: {row:?}")
    };
    many.push(Node::Map(BTreeMap::from([
        ("start".into(), Node::Number(0_i64.into())),
        ("end".into(), Node::Number(7_i64.into())),
    ])));
}

#[test]
fn a_target_that_changes_the_optional_or_list_position_fails_the_complete_subject() {
    fails_on_the_complete_subject(&optional_and_list(), flip_maybe, "maybe flipped");
    fails_on_the_complete_subject(&optional_and_list(), grow_many, "many grown");
}
