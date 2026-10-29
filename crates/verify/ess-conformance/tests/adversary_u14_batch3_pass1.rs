//! Adversary cases for the last try of a cascade witness: rows of an `affects:` entry whose creator
//! reads a second related row the filter does not bind, or copies the filtered key from a related
//! row. Each witnessed case runs the synthesized suite on the interpreter and against a model that
//! skips the move.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("demo.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds:\n{}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            |(_, scenario)| scenario,
        )
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn causes(synthesis: &ess_conformance::synthesize::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal}: {}", refusal.cause))
        .collect()
}

fn edit(model: &str, edits: &[(&str, &str)]) -> String {
    edits.iter().fold(model.to_owned(), |model, (from, to)| {
        assert!(
            model.contains(from),
            "missing edit anchor:\n{from}\nin\n{model}"
        );
        model.replacen(from, to, 1)
    })
}

fn sends_before(scenario: &ConformanceScenario, command: &str, last: &str) -> usize {
    let is = |step: &ScenarioStep, name: &str| {
        matches!(step, ScenarioStep::ExecuteCommand { command: sent, .. }
            if sent.to_string() == name)
    };
    let end = scenario
        .steps
        .iter()
        .rposition(|step| is(step, last))
        .unwrap_or_else(|| panic!("{last} is never sent: {:#?}", scenario.steps));
    scenario.steps[..end]
        .iter()
        .filter(|step| is(step, command))
        .count()
}

/// `model` keeps no refusal of `id`; the scenario sends each of `before` before `last`; every
/// scenario of the suite passes the interpreter; and `id` fails on every model of `skips`.
fn witnessed(model: &str, (id, last): (&str, &str), before: &[&str], skips: &[String]) {
    let ir = ir_of(model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refused: Vec<String> = causes(&synthesis)
        .into_iter()
        .filter(|cause| cause.contains(id))
        .collect();
    assert_eq!(
        refused,
        Vec::<String>::new(),
        "all: {:#?}",
        causes(&synthesis)
    );
    let witness = scenario(&synthesis.suite, id);
    for command in before {
        assert!(
            sends_before(witness, command, last) >= 1,
            "{command} before {last}: {:#?}",
            witness.steps
        );
    }
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir),
    );
    let failing: Vec<(&String, &Status)> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert_eq!(
        failing,
        Vec::<(&String, &Status)>::new(),
        "interpreted target fails; scenario {id}:\n{:#?}",
        witness.steps
    );
    for skipping in skips {
        let skipped = run(
            &synthesis.suite,
            &ess_conformance::interpret::Interpreted::for_model(ir_of(skipping)),
        );
        assert_eq!(skipped.get(id), Some(&Status::Failed), "{skipped:#?}");
    }
}

// ---- demo.graph: a run's cancellation over rows whose creator reads a node too ---------------

const GRAPH: &str = r"format: ess/25
system: demo
version: v1
domain: demo.graph
types:
  - {name: demo.graph.RunId, kind: newtype, of: Uuid}
  - {name: demo.graph.NodeId, kind: newtype, of: Uuid}
  - {name: demo.graph.GenerationId, kind: newtype, of: Uuid}
entities:
  - name: demo.graph.Node
    identity: {name: node_id, type: demo.graph.NodeId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Ready
      states: [Ready]
      terminal: [Ready]
      transitions: []
  - name: demo.graph.Run
    identity: {name: run_id, type: demo.graph.RunId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Active
      states: [Active, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: cancel, from: [Active], to: Cancelled}
  - name: demo.graph.Generation
    identity: {name: generation_id, type: demo.graph.GenerationId}
    fields:
      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
    relations:
      - {name: run, kind: references, target: demo.graph.Run, cardinality: one, via: run_id}
      - {name: node, kind: references, target: demo.graph.Node, cardinality: one, via: node_id}
    lifecycle:
      initial: Running
      states: [Running, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: cancel, from: [Running], to: Cancelled}
errors:
  - {name: demo.graph.RunNotFound, fields: []}
  - {name: demo.graph.RunNotActive, fields: []}
  - {name: demo.graph.NodeNotFound, fields: []}
  - {name: demo.graph.WrongState, fields: []}
events:
  - name: demo.graph.NodeAdded
    fields: [{name: node_id, type: demo.graph.NodeId}]
  - name: demo.graph.RunStarted
    fields: [{name: run_id, type: demo.graph.RunId}]
  - name: demo.graph.GenerationStarted
    fields: [{name: generation_id, type: demo.graph.GenerationId}]
  - name: demo.graph.RunCancelled
    fields: [{name: run_id, type: demo.graph.RunId}]
actors:
  - name: demo.graph.Operator
    may: [demo.graph.AddNode, demo.graph.StartRun, demo.graph.StartGeneration, demo.graph.CancelRun]
commands:
  - name: demo.graph.AddNode
    input:
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: demo.graph.Node
        instance: node_id
        emits: [demo.graph.NodeAdded]
        payload: {demo.graph.NodeAdded: {node_id: {generated: true}}}
        sets: {label: input.label}
  - name: demo.graph.StartRun
    input:
      - {name: label, type: String}
    outcomes:
      - name: started
        creates: demo.graph.Run
        instance: run_id
        emits: [demo.graph.RunStarted]
        payload: {demo.graph.RunStarted: {run_id: {generated: true}}}
        sets: {label: input.label}
  - name: demo.graph.StartGeneration
    input:
      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
    outcomes:
      - name: unknown-run
        when_related: {via: input.run_id, exists: false}
        error: demo.graph.RunNotFound
      - name: run-not-active
        when_related: {via: input.run_id, predicate: state != Active}
        error: demo.graph.RunNotActive
      - name: unknown-node
        when_related: {via: input.node_id, exists: false}
        error: demo.graph.NodeNotFound
      - name: started
        creates: demo.graph.Generation
        instance: generation_id
        emits: [demo.graph.GenerationStarted]
        payload: {demo.graph.GenerationStarted: {generation_id: {generated: true}}}
        sets: {run_id: input.run_id, node_id: input.node_id}
  - name: demo.graph.CancelRun
    input:
      - {name: run_id, type: demo.graph.RunId}
    outcomes:
      - name: cancelled
        moves: demo.graph.Run.cancel
        instance: run_id
        emits: [demo.graph.RunCancelled]
        payload: {demo.graph.RunCancelled: {run_id: input.run_id}}
        affects:
          - entity: demo.graph.Generation
            where: run_id == input.run_id
            moves: demo.graph.Generation.cancel
      - {name: wrong-state, wrong_state: true, error: demo.graph.WrongState}
views:
  - name: demo.graph.Generations
    source: demo.graph.Generation
    consistency: read_your_writes
    fields:
      - {name: generation_id, type: demo.graph.GenerationId}
      - {name: run_id, type: demo.graph.RunId}
      - {name: state, type: demo.graph.Generation.State}
  - name: demo.graph.Runs
    source: demo.graph.Run
    consistency: read_your_writes
    fields:
      - {name: run_id, type: demo.graph.RunId}
      - {name: state, type: demo.graph.Run.State}
";

const RUN_CANCELLED: &str = "demo.graph.CancelRun/outcome/cancelled";
const CANCEL_RUN: &str = "demo.graph.CancelRun";

const GENERATION_ENTRY: &str = "          - entity: demo.graph.Generation
            where: run_id == input.run_id
            moves: demo.graph.Generation.cancel
";

/// `model` without `entry` in the cancellation, and a command `command` moving `entity` by
/// `cancel` on the row its input names: the target that skips that move.
fn skipping(
    model: &str,
    entry: &str,
    (entity, id, command, event): (&str, &str, &str, &str),
) -> String {
    let may = "demo.graph.CancelRun";
    let others = model.replacen(entry, "", 1);
    let model = if others.contains("        affects:\n          -") {
        others
    } else {
        model.replacen(&format!("        affects:\n{entry}"), "", 1)
    };
    edit(
        &model,
        &[
            (may, &format!("{may}, demo.graph.{command}")),
            (
                "views:\n",
                &format!(
                    "  - name: demo.graph.{command}
    input: [{{name: {id}, type: demo.graph.{entity}Id}}]
    outcomes:
      - name: cancelled
        moves: demo.graph.{entity}.cancel
        instance: {id}
        emits: [demo.graph.{event}]
        payload: {{demo.graph.{event}: {{{id}: input.{id}}}}}
      - {{name: not-running, wrong_state: true, error: demo.graph.WrongState}}
views:
"
                ),
            ),
        ],
    )
}

const SKIP_GENERATION: (&str, &str, &str, &str) = (
    "Generation",
    "generation_id",
    "CancelGeneration",
    "GenerationStarted",
);

/// A second cascade entry over leases, whose creator also needs a node of its own.
fn with_leases(model: &str) -> String {
    edit(
        model,
        &[
            (
                "  - {name: demo.graph.GenerationId, kind: newtype, of: Uuid}\n",
                "  - {name: demo.graph.GenerationId, kind: newtype, of: Uuid}
  - {name: demo.graph.LeaseId, kind: newtype, of: Uuid}
",
            ),
            (
                "errors:\n",
                "  - name: demo.graph.Lease
    identity: {name: lease_id, type: demo.graph.LeaseId}
    fields:
      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
    relations:
      - {name: run, kind: references, target: demo.graph.Run, cardinality: one, via: run_id}
      - {name: node, kind: references, target: demo.graph.Node, cardinality: one, via: node_id}
    lifecycle:
      initial: Held
      states: [Held, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: cancel, from: [Held], to: Cancelled}
errors:
",
            ),
            (
                "events:\n",
                "events:
  - name: demo.graph.LeaseTaken
    fields: [{name: lease_id, type: demo.graph.LeaseId}]
",
            ),
            (
                "demo.graph.StartGeneration, demo.graph.CancelRun]",
                "demo.graph.StartGeneration, demo.graph.TakeLease, demo.graph.CancelRun]",
            ),
            (
                "  - name: demo.graph.CancelRun\n",
                "  - name: demo.graph.TakeLease
    input:
      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
    outcomes:
      - name: unknown-node
        when_related: {via: input.node_id, exists: false}
        error: demo.graph.NodeNotFound
      - name: taken
        creates: demo.graph.Lease
        instance: lease_id
        emits: [demo.graph.LeaseTaken]
        payload: {demo.graph.LeaseTaken: {lease_id: {generated: true}}}
        sets: {run_id: input.run_id, node_id: input.node_id}
  - name: demo.graph.CancelRun
",
            ),
            (
                GENERATION_ENTRY,
                &format!("{GENERATION_ENTRY}{LEASE_ENTRY}"),
            ),
            (
                "views:\n",
                "views:
  - name: demo.graph.Leases
    source: demo.graph.Lease
    consistency: read_your_writes
    fields:
      - {name: lease_id, type: demo.graph.LeaseId}
      - {name: run_id, type: demo.graph.RunId}
      - {name: state, type: demo.graph.Lease.State}
",
            ),
        ],
    )
}

const LEASE_ENTRY: &str = "          - entity: demo.graph.Lease
            where: run_id == input.run_id
            moves: demo.graph.Lease.cancel
";

const SKIP_LEASE: (&str, &str, &str, &str) = ("Lease", "lease_id", "CancelLease", "LeaseTaken");

/// Rows of two entries, each created beside a node of its own: the nodes arranged for one entry's
/// rows must not be the nodes arranged for the other's.
#[test]
fn u14_two_entries_whose_creators_each_read_a_node_pass_the_interpreter() {
    let model = with_leases(GRAPH);
    witnessed(
        &model,
        (RUN_CANCELLED, CANCEL_RUN),
        &[
            "demo.graph.AddNode",
            "demo.graph.StartGeneration",
            "demo.graph.TakeLease",
        ],
        &[
            skipping(&model, GENERATION_ENTRY, SKIP_GENERATION),
            skipping(&model, LEASE_ENTRY, SKIP_LEASE),
        ],
    );
}

/// The segments' creator copies the run from a generation and needs a node too; the segment
/// entry comes first, so the generation it reads is claimed before the generation entry's rows.
#[allow(clippy::too_many_lines)] // One model edit: the segment entity, its creator and its view.
fn with_segments(model: &str, segment_first: bool, segment_reads_node: bool) -> String {
    let entry = SEGMENT_ENTRY;
    let entries = if segment_first {
        format!("{entry}{GENERATION_ENTRY}")
    } else {
        format!("{GENERATION_ENTRY}{entry}")
    };
    let (node_input, node_guard, node_set) = if segment_reads_node {
        (
            "      - {name: node_id, type: demo.graph.NodeId}\n",
            "      - name: unknown-node
        when_related: {via: input.node_id, exists: false}
        error: demo.graph.NodeNotFound
",
            "          node_id: input.node_id\n",
        )
    } else {
        ("", "", "")
    };
    let segment_fields = if segment_reads_node {
        "      - {name: node_id, type: demo.graph.NodeId}\n"
    } else {
        ""
    };
    edit(
        model,
        &[
            (
                "  - {name: demo.graph.GenerationId, kind: newtype, of: Uuid}\n",
                "  - {name: demo.graph.GenerationId, kind: newtype, of: Uuid}
  - {name: demo.graph.SegmentId, kind: newtype, of: Uuid}
",
            ),
            (
                "errors:\n",
                &format!(
                    "  - name: demo.graph.Segment
    identity: {{name: segment_id, type: demo.graph.SegmentId}}
    fields:
      - {{name: generation_id, type: demo.graph.GenerationId}}
      - {{name: run_id, type: demo.graph.RunId}}
{segment_fields}    relations:
      - {{name: generation, kind: references, target: demo.graph.Generation, cardinality: one, via: generation_id}}
    lifecycle:
      initial: Open
      states: [Open, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {{name: cancel, from: [Open], to: Cancelled}}
errors:
  - {{name: demo.graph.GenerationNotFound, fields: []}}
"
                ),
            ),
            (
                "events:\n",
                "events:
  - name: demo.graph.SegmentAppended
    fields: [{name: segment_id, type: demo.graph.SegmentId}]
",
            ),
            (
                "demo.graph.StartGeneration, demo.graph.CancelRun]",
                "demo.graph.StartGeneration, demo.graph.AppendSegment, demo.graph.CancelRun]",
            ),
            (
                "  - name: demo.graph.CancelRun\n",
                &format!(
                    "  - name: demo.graph.AppendSegment
    input:
      - {{name: generation_id, type: demo.graph.GenerationId}}
{node_input}    outcomes:
      - name: unknown-generation
        when_related: {{via: input.generation_id, exists: false}}
        error: demo.graph.GenerationNotFound
{node_guard}      - name: appended
        creates: demo.graph.Segment
        instance: segment_id
        emits: [demo.graph.SegmentAppended]
        payload: {{demo.graph.SegmentAppended: {{segment_id: {{generated: true}}}}}}
        sets:
          generation_id: input.generation_id
          run_id: {{related: {{via: input.generation_id, field: run_id}}}}
{node_set}  - name: demo.graph.CancelRun
"
                ),
            ),
            (GENERATION_ENTRY, &entries),
            (
                "views:\n",
                "views:
  - name: demo.graph.Segments
    source: demo.graph.Segment
    consistency: read_your_writes
    fields:
      - {name: segment_id, type: demo.graph.SegmentId}
      - {name: run_id, type: demo.graph.RunId}
      - {name: state, type: demo.graph.Segment.State}
",
            ),
        ],
    )
}

const SEGMENT_ENTRY: &str = "          - entity: demo.graph.Segment
            where: run_id == input.run_id
            moves: demo.graph.Segment.cancel
";

const SKIP_SEGMENT: (&str, &str, &str, &str) =
    ("Segment", "segment_id", "CancelSegment", "SegmentAppended");

fn segments_witnessed(model: &str) {
    witnessed(
        model,
        (RUN_CANCELLED, CANCEL_RUN),
        &[
            "demo.graph.AddNode",
            "demo.graph.StartGeneration",
            "demo.graph.AppendSegment",
        ],
        &[
            skipping(model, GENERATION_ENTRY, SKIP_GENERATION),
            skipping(model, SEGMENT_ENTRY, SKIP_SEGMENT),
        ],
    );
}

#[test]
fn u14_copied_key_entry_first_passes_the_interpreter() {
    segments_witnessed(&with_segments(GRAPH, true, false));
}

#[test]
fn u14_copied_key_entry_first_with_its_creator_reading_a_node_passes_the_interpreter() {
    segments_witnessed(&with_segments(GRAPH, true, true));
}

#[test]
fn u14_copied_key_entry_second_with_its_creator_reading_a_node_passes_the_interpreter() {
    segments_witnessed(&with_segments(GRAPH, false, true));
}

/// The node must be published before a generation starts on it: the creator is guarded by the
/// second related row's state.
#[test]
fn u14_creator_guarded_by_the_second_rows_state_passes_the_interpreter() {
    let model = edit(
        GRAPH,
        &[
            (
                "    lifecycle:
      initial: Ready
      states: [Ready]
      terminal: [Ready]
      transitions: []
",
                "    lifecycle:
      initial: Draft
      states: [Draft, Ready]
      terminal: [Ready]
      transitions:
        - {name: publish, from: [Draft], to: Ready}
",
            ),
            (
                "  - {name: demo.graph.WrongState, fields: []}\n",
                "  - {name: demo.graph.WrongState, fields: []}
  - {name: demo.graph.NodeNotReady, fields: []}
",
            ),
            (
                "events:\n",
                "events:
  - name: demo.graph.NodePublished
    fields: [{name: node_id, type: demo.graph.NodeId}]
",
            ),
            (
                "    may: [demo.graph.AddNode,",
                "    may: [demo.graph.PublishNode, demo.graph.AddNode,",
            ),
            (
                "  - name: demo.graph.StartRun\n",
                "  - name: demo.graph.PublishNode
    input: [{name: node_id, type: demo.graph.NodeId}]
    outcomes:
      - name: published
        moves: demo.graph.Node.publish
        instance: node_id
        emits: [demo.graph.NodePublished]
        payload: {demo.graph.NodePublished: {node_id: input.node_id}}
      - {name: not-draft, wrong_state: true, error: demo.graph.WrongState}
  - name: demo.graph.StartRun
",
            ),
            (
                "        error: demo.graph.NodeNotFound
",
                "        error: demo.graph.NodeNotFound
      - name: node-not-ready
        when_related: {via: input.node_id, predicate: state != Ready}
        error: demo.graph.NodeNotReady
",
            ),
        ],
    );
    witnessed(
        &model,
        (RUN_CANCELLED, CANCEL_RUN),
        &["demo.graph.AddNode", "demo.graph.PublishNode"],
        &[skipping(&model, GENERATION_ENTRY, SKIP_GENERATION)],
    );
}

/// A node must be enabled: the creator is guarded by a stored field of the second related row.
#[test]
fn u14_creator_guarded_by_the_second_rows_field_passes_the_interpreter() {
    let model = edit(
        GRAPH,
        &[
            (
                "      - {name: label, type: String}
    lifecycle:
      initial: Ready
",
                "      - {name: label, type: String}
      - {name: enabled, type: Boolean}
    lifecycle:
      initial: Ready
",
            ),
            (
                "  - {name: demo.graph.WrongState, fields: []}\n",
                "  - {name: demo.graph.WrongState, fields: []}
  - {name: demo.graph.NodeDisabled, fields: []}
",
            ),
            (
                "  - name: demo.graph.AddNode
    input:
      - {name: label, type: String}
",
                "  - name: demo.graph.AddNode
    input:
      - {name: label, type: String}
      - {name: enabled, type: Boolean}
",
            ),
            (
                "        sets: {label: input.label}
  - name: demo.graph.StartRun
",
                "        sets: {label: input.label, enabled: input.enabled}
  - name: demo.graph.StartRun
",
            ),
            (
                "        error: demo.graph.NodeNotFound
",
                "        error: demo.graph.NodeNotFound
      - name: node-disabled
        when_related: {via: input.node_id, predicate: enabled == false}
        error: demo.graph.NodeDisabled
",
            ),
        ],
    );
    witnessed(
        &model,
        (RUN_CANCELLED, CANCEL_RUN),
        &["demo.graph.AddNode"],
        &[skipping(&model, GENERATION_ENTRY, SKIP_GENERATION)],
    );
}

/// The node input is optional: a generation may start without a node, and with one only where it
/// exists.
#[test]
fn u14_optional_second_row_input_passes_the_interpreter() {
    let model = edit(
        GRAPH,
        &[
            (
                "      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
    relations:",
                "      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: Optional<demo.graph.NodeId>}
    relations:",
            ),
            (
                "      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
    outcomes:",
                "      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: Optional<demo.graph.NodeId>}
    outcomes:",
            ),
        ],
    );
    witnessed(
        &model,
        (RUN_CANCELLED, CANCEL_RUN),
        &["demo.graph.StartGeneration"],
        &[skipping(&model, GENERATION_ENTRY, SKIP_GENERATION)],
    );
}

/// The segments' run is optional on the row and copied from the generation.
#[test]
fn u14_optional_copied_key_passes_the_interpreter() {
    let model = edit(
        &with_segments(GRAPH, false, false),
        &[
            (
                "      - {name: generation_id, type: demo.graph.GenerationId}
      - {name: run_id, type: demo.graph.RunId}
    relations:",
                "      - {name: generation_id, type: demo.graph.GenerationId}
      - {name: run_id, type: Optional<demo.graph.RunId>}
    relations:",
            ),
            (
                "      - {name: segment_id, type: demo.graph.SegmentId}
      - {name: run_id, type: demo.graph.RunId}
",
                "      - {name: segment_id, type: demo.graph.SegmentId}
      - {name: run_id, type: Optional<demo.graph.RunId>}
",
            ),
        ],
    );
    segments_witnessed(&model);
}

/// Two synthesis runs of the reported two-entry shape, on two threads with another model
/// synthesized in between, give identical bytes.
#[test]
fn u14_synthesis_is_deterministic() {
    for model in [
        with_segments(GRAPH, false, false),
        with_segments(GRAPH, true, true),
        with_leases(GRAPH),
    ] {
        let first = ess_conformance::synthesize::synthesize(&ir_of(&model));
        let _between = ess_conformance::synthesize::synthesize(&ir_of(&with_leases(GRAPH)));
        let again = model.clone();
        let second =
            std::thread::spawn(move || ess_conformance::synthesize::synthesize(&ir_of(&again)))
                .join()
                .expect("synthesis on another thread");
        assert_eq!(
            first.suite.to_canonical_json().expect("suite json"),
            second.suite.to_canonical_json().expect("suite json"),
        );
        assert_eq!(causes(&first), causes(&second));
    }
}

// ---- demo.seats: a key copied from a related row ---------------------------------------------

const SEATS: &str = r"format: ess/25
system: demo
version: v1
domain: demo.seats
types:
  - {name: demo.seats.TierId, kind: newtype, of: Uuid}
  - {name: demo.seats.SeatId, kind: newtype, of: Uuid}
  - {name: demo.seats.PassId, kind: newtype, of: Uuid}
  - {name: demo.seats.GroupId, kind: newtype, of: Uuid}
entities:
  - name: demo.seats.Group
    identity: {name: group_id, type: demo.seats.GroupId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.seats.Tier
    identity: {name: tier_id, type: demo.seats.TierId}
    fields:
      - {name: group_id, type: demo.seats.GroupId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.seats.Seat
    identity: {name: seat_id, type: demo.seats.SeatId}
    fields:
      - {name: tier_id, type: demo.seats.TierId}
      - {name: group_id, type: demo.seats.GroupId}
    relations:
      - {name: tier, kind: references, target: demo.seats.Tier, cardinality: one, via: tier_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.seats.Pass
    identity: {name: pass_id, type: demo.seats.PassId}
    fields:
      - {name: seat_id, type: demo.seats.SeatId}
      - {name: group_id, type: demo.seats.GroupId}
    relations:
      - {name: seat, kind: references, target: demo.seats.Seat, cardinality: one, via: seat_id}
    lifecycle:
      initial: Valid
      states: [Valid, Void]
      terminal: [Void]
      transitions:
        - {name: void, from: [Valid], to: Void}
errors:
  - {name: demo.seats.TierNotFound, fields: []}
  - {name: demo.seats.SeatNotFound, fields: []}
  - {name: demo.seats.WrongState, fields: []}
events:
  - name: demo.seats.GroupOpened
    fields: [{name: group_id, type: demo.seats.GroupId}]
  - name: demo.seats.TierDefined
    fields: [{name: tier_id, type: demo.seats.TierId}]
  - name: demo.seats.Issued
    fields: [{name: seat_id, type: demo.seats.SeatId}]
  - name: demo.seats.PassPrinted
    fields: [{name: pass_id, type: demo.seats.PassId}]
  - name: demo.seats.GroupClosed
    fields: [{name: group_id, type: demo.seats.GroupId}]
actors:
  - name: demo.seats.Operator
    may: [demo.seats.OpenGroup, demo.seats.DefineTier, demo.seats.Issue, demo.seats.PrintPass, demo.seats.CloseGroup]
commands:
  - name: demo.seats.OpenGroup
    input:
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: demo.seats.Group
        instance: group_id
        emits: [demo.seats.GroupOpened]
        payload: {demo.seats.GroupOpened: {group_id: {generated: true}}}
        sets: {label: input.label}
  - name: demo.seats.DefineTier
    input:
      - {name: group_id, type: demo.seats.GroupId}
    outcomes:
      - name: defined
        creates: demo.seats.Tier
        instance: tier_id
        emits: [demo.seats.TierDefined]
        payload: {demo.seats.TierDefined: {tier_id: {generated: true}}}
        sets: {group_id: input.group_id}
  - name: demo.seats.Issue
    input:
      - {name: tier_id, type: demo.seats.TierId}
    outcomes:
      - name: unknown-tier
        when_related: {via: input.tier_id, exists: false}
        error: demo.seats.TierNotFound
      - name: issued
        creates: demo.seats.Seat
        instance: seat_id
        emits: [demo.seats.Issued]
        payload: {demo.seats.Issued: {seat_id: {generated: true}}}
        sets:
          tier_id: input.tier_id
          group_id: {related: {via: input.tier_id, field: group_id}}
  - name: demo.seats.PrintPass
    input:
      - {name: seat_id, type: demo.seats.SeatId}
    outcomes:
      - name: unknown-seat
        when_related: {via: input.seat_id, exists: false}
        error: demo.seats.SeatNotFound
      - name: printed
        creates: demo.seats.Pass
        instance: pass_id
        emits: [demo.seats.PassPrinted]
        payload: {demo.seats.PassPrinted: {pass_id: {generated: true}}}
        sets:
          seat_id: input.seat_id
          group_id: {related: {via: input.seat_id, field: group_id}}
  - name: demo.seats.CloseGroup
    input:
      - {name: group_id, type: demo.seats.GroupId}
    outcomes:
      - name: closed
        moves: demo.seats.Group.close
        instance: group_id
        emits: [demo.seats.GroupClosed]
        payload: {demo.seats.GroupClosed: {group_id: input.group_id}}
        affects:
          - entity: demo.seats.Pass
            where: group_id == input.group_id
            moves: demo.seats.Pass.void
      - {name: wrong-state, wrong_state: true, error: demo.seats.WrongState}
views:
  - name: demo.seats.Passes
    source: demo.seats.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.seats.PassId}
      - {name: group_id, type: demo.seats.GroupId}
      - {name: state, type: demo.seats.Pass.State}
  - name: demo.seats.Groups
    source: demo.seats.Group
    consistency: read_your_writes
    fields:
      - {name: group_id, type: demo.seats.GroupId}
      - {name: state, type: demo.seats.Group.State}
";

// ---- more shapes -------------------------------------------------------------------------------

/// A generation reads a primary and a backup node, both of which must exist.
#[test]
fn u14_creator_reading_two_nodes_passes_the_interpreter() {
    let model = edit(
        GRAPH,
        &[
            (
                "      - {name: node_id, type: demo.graph.NodeId}
    relations:
      - {name: run, kind: references, target: demo.graph.Run, cardinality: one, via: run_id}
      - {name: node, kind: references, target: demo.graph.Node, cardinality: one, via: node_id}
",
                "      - {name: node_id, type: demo.graph.NodeId}
      - {name: backup_id, type: demo.graph.NodeId}
    relations:
      - {name: run, kind: references, target: demo.graph.Run, cardinality: one, via: run_id}
      - {name: node, kind: references, target: demo.graph.Node, cardinality: one, via: node_id}
      - {name: backup, kind: references, target: demo.graph.Node, cardinality: one, via: backup_id}
",
            ),
            (
                "      - {name: node_id, type: demo.graph.NodeId}
    outcomes:",
                "      - {name: node_id, type: demo.graph.NodeId}
      - {name: backup_id, type: demo.graph.NodeId}
    outcomes:",
            ),
            (
                "        error: demo.graph.NodeNotFound
",
                "        error: demo.graph.NodeNotFound
      - name: unknown-backup
        when_related: {via: input.backup_id, exists: false}
        error: demo.graph.NodeNotFound
",
            ),
            (
                "        sets: {run_id: input.run_id, node_id: input.node_id}",
                "        sets: {run_id: input.run_id, node_id: input.node_id, backup_id: input.backup_id}",
            ),
        ],
    );
    witnessed(
        &model,
        (RUN_CANCELLED, CANCEL_RUN),
        &["demo.graph.AddNode"],
        &[skipping(&model, GENERATION_ENTRY, SKIP_GENERATION)],
    );
}

/// The cascade moves generations only once they are running; a generation starts queued.
#[test]
fn u14_cascade_rows_moved_from_a_later_state_pass_the_interpreter() {
    let model = edit(
        GRAPH,
        &[
            (
                "    lifecycle:
      initial: Running
      states: [Running, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: cancel, from: [Running], to: Cancelled}
",
                "    lifecycle:
      initial: Queued
      states: [Queued, Running, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: begin, from: [Queued], to: Running}
        - {name: cancel, from: [Running], to: Cancelled}
",
            ),
            (
                "events:\n",
                "events:
  - name: demo.graph.GenerationBegun
    fields: [{name: generation_id, type: demo.graph.GenerationId}]
",
            ),
            (
                "    may: [demo.graph.AddNode,",
                "    may: [demo.graph.BeginGeneration, demo.graph.AddNode,",
            ),
            (
                "  - name: demo.graph.CancelRun\n",
                "  - name: demo.graph.BeginGeneration
    input: [{name: generation_id, type: demo.graph.GenerationId}]
    outcomes:
      - name: begun
        moves: demo.graph.Generation.begin
        instance: generation_id
        emits: [demo.graph.GenerationBegun]
        payload: {demo.graph.GenerationBegun: {generation_id: input.generation_id}}
      - {name: not-queued, wrong_state: true, error: demo.graph.WrongState}
  - name: demo.graph.CancelRun
",
            ),
        ],
    );
    witnessed(
        &model,
        (RUN_CANCELLED, CANCEL_RUN),
        &["demo.graph.AddNode", "demo.graph.BeginGeneration"],
        &[skipping(&model, GENERATION_ENTRY, SKIP_GENERATION)],
    );
}

/// [`SEATS`] with the cascade over seats, whose group is copied from the tier.
fn seats_single(edits: &[(&str, &str)]) -> String {
    let model = edit(
        SEATS,
        &[
            (
                "      - {name: tier_id, type: demo.seats.TierId}
      - {name: group_id, type: demo.seats.GroupId}
    relations:
      - {name: tier, kind: references, target: demo.seats.Tier, cardinality: one, via: tier_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
",
                "      - {name: tier_id, type: demo.seats.TierId}
      - {name: group_id, type: demo.seats.GroupId}
    relations:
      - {name: tier, kind: references, target: demo.seats.Tier, cardinality: one, via: tier_id}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
",
            ),
            (
                "          - entity: demo.seats.Pass
            where: group_id == input.group_id
            moves: demo.seats.Pass.void
",
                "          - entity: demo.seats.Seat
            where: group_id == input.group_id
            moves: demo.seats.Seat.close
",
            ),
            (
                "    lifecycle:
      initial: Valid
      states: [Valid, Void]
      terminal: [Void]
      transitions:
        - {name: void, from: [Valid], to: Void}
",
                "    lifecycle: {initial: Valid, states: [Valid], terminal: [Valid]}
",
            ),
            ("      - {name: state, type: demo.seats.Pass.State}\n", ""),
            (
                "views:\n",
                "views:
  - name: demo.seats.Seats
    source: demo.seats.Seat
    consistency: read_your_writes
    fields:
      - {name: seat_id, type: demo.seats.SeatId}
      - {name: group_id, type: demo.seats.GroupId}
      - {name: state, type: demo.seats.Seat.State}
",
            ),
        ],
    );
    edit(&model, edits)
}

fn seats_skipping(model: &str) -> String {
    edit(
        model,
        &[
            (
                "        affects:
          - entity: demo.seats.Seat
            where: group_id == input.group_id
            moves: demo.seats.Seat.close
",
                "",
            ),
            (
                "demo.seats.CloseGroup]",
                "demo.seats.CloseGroup, demo.seats.CloseSeat]",
            ),
            (
                "views:\n",
                "  - name: demo.seats.CloseSeat
    input: [{name: seat_id, type: demo.seats.SeatId}]
    outcomes:
      - name: closed
        moves: demo.seats.Seat.close
        instance: seat_id
        emits: [demo.seats.Issued]
        payload: {demo.seats.Issued: {seat_id: input.seat_id}}
      - {name: not-open, wrong_state: true, error: demo.seats.WrongState}
views:
",
            ),
        ],
    )
}

const GROUP_CLOSED: &str = "demo.seats.CloseGroup/outcome/closed";

/// Tiers are defined only in an open group: the copy's source row is created under a guard over
/// the moving subject's state.
#[test]
fn u14_copied_key_source_created_under_the_subject_state_is_witnessed() {
    let model = seats_single(&[
        (
            "      - {name: group_id, type: demo.seats.GroupId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.seats.Seat
",
            "      - {name: group_id, type: demo.seats.GroupId}
    relations:
      - {name: group, kind: references, target: demo.seats.Group, cardinality: one, via: group_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.seats.Seat
",
        ),
        (
            "  - {name: demo.seats.WrongState, fields: []}\n",
            "  - {name: demo.seats.WrongState, fields: []}
  - {name: demo.seats.GroupNotFound, fields: []}
  - {name: demo.seats.GroupClosedAlready, fields: []}
",
        ),
        (
            "      - name: defined
        creates: demo.seats.Tier
",
            "      - name: unknown-group
        when_related: {via: input.group_id, exists: false}
        error: demo.seats.GroupNotFound
      - name: group-closed
        when_related: {via: input.group_id, predicate: state != Open}
        error: demo.seats.GroupClosedAlready
      - name: defined
        creates: demo.seats.Tier
",
        ),
    ]);
    witnessed(
        &model,
        (GROUP_CLOSED, "demo.seats.CloseGroup"),
        &["demo.seats.DefineTier", "demo.seats.Issue"],
        &[seats_skipping(&model)],
    );
}

/// The segment entry also filters on a field the creator writes from its own input.
#[test]
fn u14_copied_key_beside_a_literal_conjunct_passes_the_interpreter() {
    let model = edit(
        &with_segments(GRAPH, false, true),
        &[
            (
                "      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
    relations:
      - {name: generation,",
                "      - {name: run_id, type: demo.graph.RunId}
      - {name: node_id, type: demo.graph.NodeId}
      - {name: draft, type: Boolean}
    relations:
      - {name: generation,",
            ),
            (
                "      - {name: generation_id, type: demo.graph.GenerationId}
      - {name: node_id, type: demo.graph.NodeId}
    outcomes:",
                "      - {name: generation_id, type: demo.graph.GenerationId}
      - {name: node_id, type: demo.graph.NodeId}
      - {name: draft, type: Boolean}
    outcomes:",
            ),
            (
                "          node_id: input.node_id\n",
                "          node_id: input.node_id\n          draft: input.draft\n",
            ),
            (
                "          - entity: demo.graph.Segment
            where: run_id == input.run_id
",
                "          - entity: demo.graph.Segment
            where: {all: [run_id == input.run_id, draft == true]}
",
            ),
        ],
    );
    let entry = "          - entity: demo.graph.Segment
            where: {all: [run_id == input.run_id, draft == true]}
            moves: demo.graph.Segment.cancel
";
    witnessed(
        &model,
        (RUN_CANCELLED, CANCEL_RUN),
        &["demo.graph.AppendSegment"],
        &[
            skipping(&model, GENERATION_ENTRY, SKIP_GENERATION),
            skipping(&model, entry, SKIP_SEGMENT),
        ],
    );
}

/// The two-entry shape with the segment creator reading a node, at `format: ess/32`.
#[test]
fn u14_two_entries_with_node_reading_segments_at_ess_32_pass_the_interpreter() {
    let model = edit(
        &with_segments(GRAPH, false, true),
        &[("format: ess/25\n", "format: ess/32\n")],
    );
    segments_witnessed(&model);
}
