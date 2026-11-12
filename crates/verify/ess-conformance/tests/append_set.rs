//! `sets: {tags: {append: input.tag}}` (`ess/31`): an outcome storing the list its subject held
//! before it, absent read as empty, with one input element last.
//!
//! An adopter's command added one element to a stored list and could not say so; a target that
//! dropped the earlier elements passed. From `ess/31` the interpreted target appends, the
//! synthesized suite asserts the whole list the row holds afterwards, and a model that does not
//! use the source compiles to the bytes it had.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted,
    report::Status,
    target::{
        ConformanceTarget, Deadline, EntitySetupRequest, EventObservationRequest,
        ExternalOutcomeControl, ImplementationIdentity, ObservedEvent, RedeliveryRequest,
        ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
        SemanticViewResult, TargetError, ViewRow,
    },
    AdmittedSuite, Runner, ScenarioStep, ScenarioValue, ViewExpectation,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const ID: &str = "00000000-0000-4000-8000-000000000001";

/// A job opened with a required list and holding an optional one, grown by `AddTag` and
/// `AddNote` from `tag_source` and `note_source`.
fn jobs(format: &str, tag_source: &str, note_source: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
entities:
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: tags, type: List<String>}}
      - {{name: notes, type: Optional<List<String>>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.jobs.Changed
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
commands:
  - name: demo.jobs.OpenJob
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: opened
        creates: demo.jobs.Job
        instance: job_id
        sets: {{tags: input.tags}}
        emits: [demo.jobs.Changed]
        payload: {{demo.jobs.Changed: {{job_id: input.job_id}}}}
  - name: demo.jobs.AddTag
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: tag, type: String}}, {{name: tags, type: List<String>}}]
    outcomes:
      - name: tagged
        updates: demo.jobs.Job
        instance: job_id
        sets: {{tags: {tag_source}}}
        emits: [demo.jobs.Changed]
        payload: {{demo.jobs.Changed: {{job_id: input.job_id}}}}
  - name: demo.jobs.AddNote
    input: [{{name: job_id, type: demo.jobs.JobId}}, {{name: note, type: String}}, {{name: notes, type: List<String>}}]
    outcomes:
      - name: noted
        updates: demo.jobs.Job
        instance: job_id
        sets: {{notes: {note_source}}}
        emits: [demo.jobs.Changed]
        payload: {{demo.jobs.Changed: {{job_id: input.job_id}}}}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {{name: job_id, type: demo.jobs.JobId}}
      - {{name: tags, type: List<String>}}
      - {{name: notes, type: Optional<List<String>>}}
"
    )
}

fn appending() -> String {
    jobs("ess/31", "{append: input.tag}", "{append: input.note}")
}

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn correlation() -> CorrelationId {
    CorrelationId::new("jobs").unwrap()
}

fn texts(items: &[&str]) -> Node {
    Node::Seq(
        items
            .iter()
            .map(|item| Node::Text((*item).into()))
            .collect(),
    )
}

fn run(target: &Interpreted, command: &str, field: &str, value: &str) {
    target
        .execute_command(SemanticCommandRequest {
            command: command.parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([
                ("job_id".into(), Node::Text(ID.into())),
                (field.into(), Node::Text(value.into())),
                // The whole list the ess/30 form stores; the append ignores it.
                (format!("{field}s"), Node::Seq(Vec::new())),
            ]),
            correlation: correlation(),
        })
        .unwrap_or_else(|error| panic!("{command}: {error:?}"));
}

fn row(target: &Interpreted) -> ViewRow {
    let rows = target
        .query_view(SemanticViewRequest {
            view: "demo.jobs.Jobs".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows;
    assert_eq!(rows.len(), 1, "{rows:?}");
    rows[0].clone()
}

#[test]
fn the_interpreter_appends_the_input_last() {
    let target = Interpreted::for_model(model(&appending()));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.jobs/authored/append".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    target
        .establish_entity(EntitySetupRequest {
            entity: "demo.jobs.Job".parse().unwrap(),
            identity: Node::Text(ID.into()),
            fields: BTreeMap::from([("tags".into(), texts(&["first"]))]),
            state: "Open".parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();

    run(&target, "demo.jobs.AddTag", "tag", "second");
    run(&target, "demo.jobs.AddTag", "tag", "third");
    assert_eq!(row(&target)["tags"], texts(&["first", "second", "third"]));

    // An absent optional list reads as empty: the first append stores a list of one.
    run(&target, "demo.jobs.AddNote", "note", "seen");
    assert_eq!(row(&target)["notes"], texts(&["seen"]));
    run(&target, "demo.jobs.AddNote", "note", "again");
    let after = row(&target);
    assert_eq!(after["notes"], texts(&["seen", "again"]));
    assert_eq!(after["tags"], texts(&["first", "second", "third"]));
}

#[test]
fn the_synthesized_suite_passes_on_the_interpreted_target() {
    let ir = model(&appending());
    let synthesis = ess_conformance::synthesize(&ir);
    for command in ["demo.jobs.AddTag", "demo.jobs.AddNote"] {
        assert!(
            synthesis
                .suite
                .scenarios
                .keys()
                .any(|id| id.to_string().contains(command)),
            "{command} is witnessed: {:?}",
            synthesis.refusals
        );
    }
    // The appending branch's scenario asserts the whole list the row holds afterwards.
    let suite = format!("{:?}", synthesis.suite);
    assert!(
        suite.contains("tags"),
        "the suite names the appended list:\n{suite}"
    );
    let input = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report =
        Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(ir.clone()));
    let unpassed: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert!(unpassed.is_empty(), "{unpassed:#?}");

    // A target that overwrites the list with the element alone fails the suite.
    let overwriting = model(&jobs("ess/31", "[]", "{cleared: true}"));
    let report =
        Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(overwriting));
    assert!(
        report
            .scenarios
            .iter()
            .any(|scenario| scenario.status == Status::Failed),
        "a target that drops the earlier elements fails"
    );
}

/// A job holding a `List<Optional<String>>`, grown by `AddLabel` from an `Optional<String>`
/// input: the element type the domain admits is the list's own, `Optional`.
const LABELLED: &str = "format: ess/31
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: labels, type: List<Optional<String>>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
events:
  - name: demo.jobs.Changed
    fields: [{name: job_id, type: demo.jobs.JobId}]
commands:
  - name: demo.jobs.OpenJob
    input: [{name: job_id, type: demo.jobs.JobId}, {name: labels, type: List<Optional<String>>}]
    outcomes:
      - name: opened
        creates: demo.jobs.Job
        instance: job_id
        sets: {labels: input.labels}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: input.job_id}}
  - name: demo.jobs.AddLabel
    input: [{name: job_id, type: demo.jobs.JobId}, {name: label, type: Optional<String>}]
    outcomes:
      - name: labelled
        updates: demo.jobs.Job
        instance: job_id
        sets: {labels: {append: input.label}}
        emits: [demo.jobs.Changed]
        payload: {demo.jobs.Changed: {job_id: input.job_id}}
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: labels, type: List<Optional<String>>}
";

fn add_label(target: &Interpreted, label: Option<Node>) {
    let mut input = BTreeMap::from([("job_id".into(), Node::Text(ID.into()))]);
    if let Some(label) = label {
        input.insert("label".into(), label);
    }
    target
        .execute_command(SemanticCommandRequest {
            command: "demo.jobs.AddLabel".parse().unwrap(),
            actor: None,
            caller: None,
            input,
            correlation: correlation(),
        })
        .unwrap_or_else(|error| {
            panic!(
                "story:an-optional-element-is-appended-or-refused-consistently: the domain \
                 admits the Optional element, so the interpreter appends it: {error:?}"
            )
        });
}

#[test]
fn an_optional_element_the_domain_admits_is_appended_by_the_interpreter() {
    let target = Interpreted::for_model(model(LABELLED));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.jobs/authored/append-optional".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    target
        .establish_entity(EntitySetupRequest {
            entity: "demo.jobs.Job".parse().unwrap(),
            identity: Node::Text(ID.into()),
            fields: BTreeMap::from([("labels".into(), texts(&["a"]))]),
            state: "Open".parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();

    // One element left out, one sent as `null`: both are the absent `Optional`, appended as null.
    add_label(&target, None);
    add_label(&target, Some(Node::Null));
    assert_eq!(
        row(&target)["labels"],
        Node::Seq(vec![Node::Text("a".into()), Node::Null, Node::Null]),
        "story:an-optional-element-is-appended-or-refused-consistently"
    );

    // A present element is appended as it is.
    add_label(&target, Some(Node::Text("b".into())));
    assert_eq!(
        row(&target)["labels"],
        Node::Seq(vec![
            Node::Text("a".into()),
            Node::Null,
            Node::Null,
            Node::Text("b".into()),
        ])
    );
}

/// The suite synthesized for an `Optional` element, run on the interpreted target.
#[test]
fn the_synthesized_suite_appending_an_optional_element_passes_on_the_interpreted_target() {
    let ir = model(LABELLED);
    let synthesis = ess_conformance::synthesize(&ir);
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().contains("demo.jobs.AddLabel")),
        "AddLabel is witnessed: {:?}",
        synthesis.refusals
    );
    let labelled = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.jobs.AddLabel/outcome/labelled")
        .map_or_else(|| panic!("no labelled scenario"), |(_, scenario)| scenario);
    // The scenario sends the `Optional` element present, `label: "label"`, and asserts it appended
    // after the element the arrangement stored; beside it, a send that leaves the element out
    // asserts the null appended after the stored element
    // (story:a-present-optional-element-is-appended).
    let sends: Vec<_> = labelled
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.jobs.AddLabel" =>
            {
                Some(input.get("label"))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        sends,
        [
            Some(&ScenarioValue::Literal {
                value: Node::Text("label".into())
            }),
            None
        ],
        "story:a-present-optional-element-is-appended: {:#?}",
        labelled.steps
    );
    let expected: Vec<_> = labelled
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => fields.get("labels"),
            _ => None,
        })
        .collect();
    assert_eq!(
        expected[0],
        &ScenarioValue::Literal {
            value: Node::Seq(vec![
                Node::Text("labels.0".into()),
                Node::Text("label".into())
            ]),
        },
        "story:a-present-optional-element-is-appended: {:#?}",
        labelled.steps
    );
    // The send that leaves the element out runs on a further row, as every omission run does, so
    // the element its arrangement stored carries that row's own distinction.
    assert!(
        matches!(
            expected.as_slice(),
            [_, ScenarioValue::Literal { value: Node::Seq(items) }]
                if matches!(items.as_slice(), [Node::Text(stored), Node::Null]
                    if stored.starts_with("labels.0"))
        ),
        "story:a-present-optional-element-is-appended: {:#?}",
        labelled.steps
    );
    let input = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report =
        Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(ir.clone()));
    let unpassed: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert!(
        unpassed.is_empty(),
        "story:an-optional-element-is-appended-or-refused-consistently: {unpassed:#?}"
    );
}

/// The interpreter with `label` dropped from every `AddLabel` input: whatever is sent, it appends
/// the absent `Optional`, `null`.
struct DropsLabel(Interpreted);

impl ConformanceTarget for DropsLabel {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.0.establish_entity(request)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() == "demo.jobs.AddLabel" {
            request.input.remove("label");
        }
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
}

/// A target that appends `null` whatever `AddLabel` sends passed the suite, which never sent a
/// present element.
#[test]
fn a_target_that_appends_null_whatever_is_sent_fails_the_present_element_scenario() {
    let ir = model(LABELLED);
    let synthesis = ess_conformance::synthesize(&ir);
    let input = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(input.suite())
        .run_admitted(&input, &DropsLabel(Interpreted::for_model(ir)));
    let labelled: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.scenario.to_string() == "demo.jobs.AddLabel/outcome/labelled")
        .collect();
    assert_eq!(labelled.len(), 1, "{:#?}", report.scenarios);
    assert_eq!(
        labelled[0].status,
        Status::Failed,
        "story:a-present-optional-element-is-appended: {labelled:#?}"
    );
}

/// The IR of a model that sets both lists from its input at `ess/30`, as compiled before
/// `{append: …}` existed: its SHA-256 and length.
const BEFORE_APPEND: (&str, usize) = (
    "4c1fe8d0e2d72e14fa5ea3f31fa2359f7b0721c78ba1d84230c81724879c36c2",
    11462,
);

#[test]
fn models_without_it_keep_their_ir_bytes() {
    let at_30 = model(&jobs("ess/30", "input.tags", "input.notes")).to_canonical_json();
    let digest: String =
        Sha256::digest(at_30.as_bytes())
            .iter()
            .fold(String::new(), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            });
    assert_eq!(
        (digest.as_str(), at_30.len()),
        BEFORE_APPEND,
        "the ess/30 IR keeps its bytes:\n{at_30}"
    );
    let at_31 = model(&jobs("ess/31", "input.tags", "input.notes")).to_canonical_json();
    assert_eq!(at_31, at_30);
}

// Adversary pass 1 of story:a-present-optional-element-is-appended: the present element and the
// left-out one, over other element types and other shapes of the appending branch.

const LABELLED_ID: &str = "demo.jobs.AddLabel/outcome/labelled";

/// `LABELLED` with `edits` applied, each `(from, to)` replacing every occurrence.
fn labelled_with(edits: &[(&str, &str)]) -> String {
    edits.iter().fold(LABELLED.to_owned(), |text, (from, to)| {
        assert!(text.contains(from), "`{from}` is in the model:\n{text}");
        text.replace(from, to)
    })
}

/// `LABELLED` over `List<Optional<{element}>>`, `types` declared beside the identity.
fn labelled_of(element: &str, types: &str) -> String {
    labelled_with(&[
        ("Optional<String>", &format!("Optional<{element}>")),
        (
            "  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}\n",
            &format!("  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}\n{types}"),
        ),
    ])
}

/// `LABELLED` whose appending branch `moves:` the job, the other way the append is admitted.
fn labelled_on_a_move() -> String {
    labelled_with(&[
        (
            "    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n",
            "    lifecycle: {initial: Open, states: [Open, Labelled], terminal: [Labelled], \
             transitions: [{name: label, from: [Open], to: Labelled}]}\n",
        ),
        (
            "        updates: demo.jobs.Job\n",
            "        moves: demo.jobs.Job.label\n",
        ),
        (
            "entities:\n",
            "errors:\n  - {name: demo.jobs.AlreadyLabelled, summary: The job is already labelled., \
             fields: []}\nentities:\n",
        ),
        (
            "views:\n",
            "      - {name: already-labelled, wrong_state: true, error: \
             demo.jobs.AlreadyLabelled}\nviews:\n",
        ),
    ])
}

/// Each `AddLabel` send of `input` in the labelled scenario, with the `list` the view is expected to
/// hold after it and the list the arranging `OpenJob` stored before it: `(stored, sent, expected)`,
/// an element left out read as `null`, an `OpenJob` leaving `list` out as an empty list.
fn appends_in(
    suite: &ess_conformance::ConformanceSuite,
    (list, input): (&str, &str),
) -> Vec<(Vec<Node>, Node, Node)> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == LABELLED_ID)
        .map_or_else(
            || {
                panic!(
                    "no labelled scenario: {:#?}",
                    suite.scenarios.keys().collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        );
    let literal = |value: Option<&ScenarioValue>| match value {
        None => Some(Node::Null),
        Some(ScenarioValue::Literal { value }) => Some(value.clone()),
        Some(_) => None,
    };
    let mut stored: Option<Vec<Node>> = None;
    let mut sent: Option<(Vec<Node>, Node)> = None;
    let mut found = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand {
                command,
                input: sends,
                ..
            } if command.to_string() == "demo.jobs.OpenJob" => {
                stored = match literal(sends.get(list)) {
                    Some(Node::Seq(items)) => Some(items),
                    Some(Node::Null) => Some(Vec::new()),
                    _ => None,
                };
            }
            ScenarioStep::ExecuteCommand {
                command,
                input: sends,
                ..
            } if command.to_string() == "demo.jobs.AddLabel" => {
                sent = stored.clone().zip(literal(sends.get(input)));
            }
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => {
                if let Some(ScenarioValue::Literal { value }) = fields.get(list) {
                    if let Some((before, element)) = sent.take() {
                        found.push((before, element, value.clone()));
                    }
                }
            }
            _ => {}
        }
    }
    found
}

/// The suite synthesized from `text` sends each appended `input` present and expects it last in its
/// `list`, and leaves it out and expects `null` last, each after exactly the list the arranging step
/// stored; and every scenario passes on the interpreted target.
fn appends_present_and_absent(text: &str, appended: &[(&str, &str)]) {
    let ir = model(text);
    let synthesis = ess_conformance::synthesize(&ir);
    for &(list, input) in appended {
        let found = appends_in(&synthesis.suite, (list, input));
        let exact = |(before, element, after): &(Vec<Node>, Node, Node)| {
            let mut appended = before.clone();
            appended.push(element.clone());
            Node::Seq(appended) == *after
        };
        let steps = synthesis
            .suite
            .scenarios
            .iter()
            .find(|(id, _)| id.to_string() == LABELLED_ID)
            .map(|(_, scenario)| &scenario.steps);
        assert!(
            found.iter().any(|each| each.1 != Node::Null && exact(each)),
            "story:a-present-optional-element-is-appended: `{input}` is sent present and \
             expected last in `{list}`: {found:#?}\nrefusals: {:#?}\nsteps: {steps:#?}",
            synthesis.refusals
        );
        assert!(
            found.iter().any(|each| each.1 == Node::Null && exact(each)),
            "story:a-present-optional-element-is-appended: `{input}` is left out and `null` is \
             expected last in `{list}`: {found:#?}\nrefusals: {:#?}",
            synthesis.refusals
        );
        assert!(
            found.iter().all(exact),
            "each expectation is the stored list with the sent element last: {found:#?}"
        );
    }
    let input = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(ir));
    let unpassed: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert!(unpassed.is_empty(), "{unpassed:#?}");
}

/// Deviation 1 of the unit: the left-out run's stored element is `labels.0-<n>`, which the unit's
/// test checks with `starts_with`; here it is the list that run's own `OpenJob` stored, exactly.
#[test]
fn adv_the_left_out_run_expects_exactly_its_own_stored_list() {
    appends_present_and_absent(LABELLED, &[("labels", "label")]);
}

#[test]
fn adv_an_integer_element_is_appended_present_and_absent() {
    appends_present_and_absent(&labelled_of("Integer", ""), &[("labels", "label")]);
}

#[test]
fn adv_a_decimal_element_is_appended_present_and_absent() {
    appends_present_and_absent(&labelled_of("Decimal", ""), &[("labels", "label")]);
}

#[test]
fn adv_a_boolean_element_is_appended_present_and_absent() {
    appends_present_and_absent(&labelled_of("Boolean", ""), &[("labels", "label")]);
}

#[test]
fn adv_a_timestamp_element_is_appended_present_and_absent() {
    appends_present_and_absent(&labelled_of("Timestamp", ""), &[("labels", "label")]);
}

#[test]
fn adv_an_enum_element_is_appended_present_and_absent() {
    appends_present_and_absent(
        &labelled_of(
            "demo.jobs.Colour",
            "  - {name: demo.jobs.Colour, kind: enum, variants: [Red, Green]}\n",
        ),
        &[("labels", "label")],
    );
}

#[test]
fn adv_a_struct_element_is_appended_present_and_absent() {
    appends_present_and_absent(
        &labelled_of(
            "demo.jobs.Tag",
            "  - name: demo.jobs.Tag\n    kind: struct\n    fields:\n      - {name: name, type: \
             String}\n      - {name: note, type: Optional<String>}\n",
        ),
        &[("labels", "label")],
    );
}

#[test]
fn adv_a_newtype_element_is_appended_present_and_absent() {
    appends_present_and_absent(
        &labelled_of(
            "demo.jobs.Code",
            "  - {name: demo.jobs.Code, kind: newtype, of: String}\n",
        ),
        &[("labels", "label")],
    );
}

#[test]
fn adv_a_reference_element_is_appended_present_and_absent() {
    appends_present_and_absent(&labelled_of("demo.jobs.JobId", ""), &[("labels", "label")]);
}

/// An `Optional<List<Optional<String>>>` the arranging step leaves absent.
#[test]
#[ignore = "backlog: absent-optional-list-append (story:an-append-onto-an-absent-optional-list-is-asserted)"]
fn adv_an_element_appended_to_an_absent_optional_list_is_present_and_absent() {
    appends_present_and_absent(
        &labelled_with(&[
            (
                "input: [{name: job_id, type: demo.jobs.JobId}, {name: labels, type: \
                 List<Optional<String>>}]",
                "input: [{name: job_id, type: demo.jobs.JobId}]",
            ),
            ("        sets: {labels: input.labels}\n", ""),
            (
                "type: List<Optional<String>>}",
                "type: Optional<List<Optional<String>>>}",
            ),
        ]),
        &[("labels", "label")],
    );
}

/// Two appends in one branch, the second onto an absent `Optional` list of another element type.
#[test]
#[ignore = "backlog: absent-optional-list-append (story:an-append-onto-an-absent-optional-list-is-asserted)"]
fn adv_two_appends_in_one_branch_are_each_present_and_absent() {
    appends_present_and_absent(
        &labelled_with(&[
            (
                "      - {name: labels, type: List<Optional<String>>}\n",
                "      - {name: labels, type: List<Optional<String>>}\n      - {name: marks, type: \
                 Optional<List<Optional<Integer>>>}\n",
            ),
            (
                "{name: label, type: Optional<String>}]",
                "{name: label, type: Optional<String>}, {name: mark, type: Optional<Integer>}]",
            ),
            (
                "sets: {labels: {append: input.label}}",
                "sets: {labels: {append: input.label}, marks: {append: input.mark}}",
            ),
        ]),
        &[("labels", "label"), ("marks", "mark")],
    );
}

/// An append beside a `{cleared: true}` of another field.
#[test]
fn adv_an_append_beside_a_cleared_field_is_present_and_absent() {
    appends_present_and_absent(
        &labelled_with(&[
            (
                "      - {name: labels, type: List<Optional<String>>}\n",
                "      - {name: labels, type: List<Optional<String>>}\n      - {name: note, type: \
                 Optional<String>}\n",
            ),
            (
                "{name: labels, type: List<Optional<String>>}]",
                "{name: labels, type: List<Optional<String>>}, {name: note, type: String}]",
            ),
            (
                "sets: {labels: input.labels}",
                "sets: {labels: input.labels, note: input.note}",
            ),
            (
                "sets: {labels: {append: input.label}}",
                "sets: {labels: {append: input.label}, note: {cleared: true}}",
            ),
        ]),
        &[("labels", "label")],
    );
}

/// The appended input also copied into an `Optional` field of the row.
#[test]
fn adv_an_append_whose_input_is_also_copied_is_present_and_absent() {
    appends_present_and_absent(
        &labelled_with(&[
            (
                "      - {name: labels, type: List<Optional<String>>}\n",
                "      - {name: labels, type: List<Optional<String>>}\n      - {name: last, type: \
                 Optional<String>}\n",
            ),
            (
                "sets: {labels: {append: input.label}}",
                "sets: {labels: {append: input.label}, last: input.label}",
            ),
        ]),
        &[("labels", "label")],
    );
}

/// The append in a branch that `moves:` its subject, where no omission run is built for a command
/// with a state guard: before this unit its scenario left the element out and asserted the `null`.
#[test]
fn adv_an_append_on_a_move_is_present_and_absent() {
    appends_present_and_absent(&labelled_on_a_move(), &[("labels", "label")]);
}

/// The interpreter storing `self.1` where `AddLabel` leaves `label` out or sends `null`: a target
/// appending a default for an absent element instead of `null`.
struct DefaultsLabel(Interpreted, Node);

impl ConformanceTarget for DefaultsLabel {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.0.establish_entity(request)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() == "demo.jobs.AddLabel"
            && matches!(request.input.get("label"), None | Some(Node::Null))
        {
            request.input.insert("label".into(), self.1.clone());
        }
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
}

/// The suite synthesized from `text` fails a target appending `"absent"` for a left-out element.
fn fails_a_target_appending_a_default(text: &str) {
    let ir = model(text);
    let synthesis = ess_conformance::synthesize(&ir);
    let input = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(input.suite()).run_admitted(
        &input,
        &DefaultsLabel(Interpreted::for_model(ir), Node::Text("absent".into())),
    );
    let statuses: Vec<_> = report
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), &scenario.status))
        .collect();
    assert!(
        report
            .scenarios
            .iter()
            .any(|scenario| scenario.status == Status::Failed),
        "a target appending a default for an absent element fails: {statuses:#?}"
    );
}

#[test]
fn adv_a_target_appending_a_default_for_an_absent_element_fails() {
    fails_a_target_appending_a_default(LABELLED);
}

#[test]
fn adv_a_target_appending_a_default_for_an_absent_element_on_a_move_fails() {
    fails_a_target_appending_a_default(&labelled_on_a_move());
}
