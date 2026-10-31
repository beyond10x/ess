//! Adversary pass 1 over wave D325 (`ess/33`): structured input examples and rows a deployment
//! provides, driven through synthesis and the interpreted target.
//!
//! Each case states the contract it holds the implementation to; a red case is a finding.

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const LINKS: &str = include_str!("fixtures/provided-related-row.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn yaml(text: &str) -> Node {
    ess_yaml::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            let id = refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            format!("{id}: {}", refusal.cause)
        })
        .collect()
}

/// Synthesizes `text`, requires no refusal, runs the suite on the interpreted target and returns
/// every scenario's status by id.
fn synthesized_and_run(text: &str) -> (Synthesis, Vec<(String, Status, String)>) {
    synthesized_and_run_refusing(text, None)
}

/// [`synthesized_and_run`], where the only refusals admitted are those whose text names
/// `refused` (a position the design refuses by name).
fn synthesized_and_run_refusing(
    text: &str,
    refused: Option<&str>,
) -> (Synthesis, Vec<(String, Status, String)>) {
    let result = ess_conformance::synthesize::synthesize(&ir(text));
    let unexpected: Vec<_> = refusals(&result)
        .into_iter()
        .filter(|refusal| refused.is_none_or(|prefix| !refusal.contains(prefix)))
        .collect();
    assert!(unexpected.is_empty(), "no refusal: {unexpected:#?}");
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let run =
        Runner::for_suite(&result.suite).run_admitted(&admitted, &Interpreted::for_model(ir(text)));
    let statuses = run
        .scenarios
        .iter()
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                scenario.status,
                format!("{scenario:?}"),
            )
        })
        .collect();
    (result, statuses)
}

fn all_passed(statuses: &[(String, Status, String)]) {
    assert!(!statuses.is_empty(), "the run executes the suite");
    let failed: Vec<_> = statuses
        .iter()
        .filter(|(_, status, _)| *status != Status::Passed)
        .map(|(id, status, detail)| format!("{id}: {status:?}\n{detail}"))
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes:\n{}",
        failed.join("\n\n")
    );
}

/// Every literal value the scenario's steps executing `command` sent at `field`.
fn sent(result: &Synthesis, scenario: &str, command: &str, field: &str) -> Vec<Node> {
    let (_, scenario) = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {scenario}: {:?}",
                result.suite.scenarios.keys().collect::<Vec<_>>()
            )
        });
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: executed,
                input,
                ..
            } if executed.to_string() == command => match input.get(field) {
                Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                None => None,
                other => panic!("{field} is {other:?}"),
            },
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Rows a deployment provides
// ---------------------------------------------------------------------------------------------

/// A provided entity that holds an invariant over its own field. The model is valid at `ess/33`,
/// synthesis serves the related row as an entity fixture, and the interpreted target is the
/// deployment that provides it: it has to provide a row the entity's invariant admits, as any
/// deployment holding the entity's rows does. The ADR: "The interpreted target provides such a
/// fixture by creating the row."
#[test]
fn a_provided_row_whose_entity_holds_an_invariant_passes_on_the_interpreted_target() {
    let text = LINKS.replace(
        "    fields:\n      - {name: title, type: String}\n    lifecycle: {initial: Active, states: [Active], terminal: [Active]}\n",
        "    fields:\n      - {name: title, type: String}\n      - {name: seats, type: Integer}\n    invariants: [seats > 1000]\n    lifecycle: {initial: Active, states: [Active], terminal: [Active]}\n",
    );
    assert_ne!(text, LINKS, "the invariant is written");
    let (_, statuses) = synthesized_and_run(&text);
    all_passed(&statuses);
}

/// A provided entity whose lifecycle carries `ess/33` summaries on its states and its transition,
/// read by a guard over the row's state: each branch takes the row in the state it needs, and
/// every scenario passes on the interpreted target.
#[test]
fn a_provided_row_in_the_state_a_guard_needs_passes_beside_lifecycle_summaries() {
    let text = LINKS
        .replace(
            "    lifecycle: {initial: Active, states: [Active], terminal: [Active]}\n  - name: linking.links.Link",
            "    lifecycle:
      initial: Active
      states:
        - {name: Active, summary: The app takes links.}
        - {name: Retired, summary: The app takes no new link.}
      terminal: [Retired]
      transitions:
        - {name: retire, from: [Active], to: Retired, summary: The app stops taking links.}
  - name: linking.links.Link",
        )
        .replace(
            "      - name: linked\n",
            "      - name: retired-app
        when_related: {via: input.app_id, predicate: state == Retired}
        error: linking.links.AppNotFound
        payload:
          linking.links.AppNotFound: {app_id: input.app_id}
      - name: linked\n",
        )
        .replace(
            "commands:\n",
            "commands:
  - name: linking.links.RetireApp
    input:
      - {name: app_id, type: linking.links.AppId}
    outcomes:
      - name: retired
        instance: app_id
        moves: linking.links.App.retire
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}, app_id: input.app_id}
",
        );
    assert!(text.contains("retired-app") && text.contains("summary: The app stops"));
    // A provided subject is refused by name (by design); the related-row positions are served.
    let (result, statuses) = synthesized_and_run_refusing(&text, Some("linking.links.RetireApp"));
    let retired = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "linking.links.CreateLink/outcome/retired-app")
        .map(|(_, scenario)| scenario)
        .expect("the retired-app branch is served");
    let states: Vec<String> = retired
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ResolveFixtures { fixtures } => Some(fixtures),
            _ => None,
        })
        .flat_map(|fixtures| fixtures.fields.iter())
        .filter_map(|field| field.state.as_ref().map(ToString::to_string))
        .collect();
    assert_eq!(states, ["Retired"], "{:#?}", retired.steps);
    all_passed(&statuses);
}

/// Two related guards over two inputs naming rows of one provided entity, and a refusal when the
/// two inputs name the same app: the accepted branch needs two different provided rows, each an
/// entity fixture of its own, and the interpreted target provides both.
#[test]
fn two_provided_rows_through_two_inputs_are_two_rows() {
    let text = LINKS
        .replace(
            "      - {name: app_id, type: linking.links.AppId}\n    outcomes:",
            "      - {name: app_id, type: linking.links.AppId}\n      - {name: peer_id, type: linking.links.AppId}\n    outcomes:",
        )
        .replace(
            "      - name: unknown-app\n",
            "      - name: same-app
        when: input.app_id == input.peer_id
        error: linking.links.AppNotFound
        payload:
          linking.links.AppNotFound: {app_id: input.app_id}
      - name: unknown-app\n",
        )
        .replace(
            "      - name: linked\n",
            "      - name: unknown-peer
        when_related: {via: input.peer_id, exists: false}
        error: linking.links.AppNotFound
        payload:
          linking.links.AppNotFound: {app_id: input.peer_id}
      - name: linked\n",
        );
    assert!(text.contains("unknown-peer") && text.contains("peer_id, type"));
    // Control: the same guards over an app a command creates synthesize with no refusal, so what
    // follows measures the provided-row path alone.
    let created = text.replace("    provided: true\n", "").replace(
        "commands:\n",
        "commands:
  - name: linking.links.CreateApp
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: title, type: String}
    outcomes:
      - name: created
        creates: linking.links.App
        instance: app_id
        sets: {title: input.title}
        emits: [linking.links.Linked]
        payload:
          linking.links.Linked: {link_id: {generated: true}, app_id: input.app_id}
",
    );
    let control = ess_conformance::synthesize::synthesize(&ir(&created));
    assert!(
        control.refusals.is_empty(),
        "control, an app a command creates: {:#?}",
        refusals(&control)
    );
    let (result, statuses) = synthesized_and_run(&text);
    let linked = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "linking.links.CreateLink/outcome/linked")
        .map(|(_, scenario)| scenario)
        .expect("the linked branch is served");
    let mut sent_fixtures = Vec::new();
    for step in &linked.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            if command.to_string() == "linking.links.CreateLink" {
                for field in ["app_id", "peer_id"] {
                    sent_fixtures.push(format!("{:?}", input.get(field)));
                }
            }
        }
    }
    assert_eq!(sent_fixtures.len(), 2, "{:#?}", linked.steps);
    assert_ne!(
        sent_fixtures[0], sent_fixtures[1],
        "two different rows: {:#?}",
        linked.steps
    );
    all_passed(&statuses);
}

// ---------------------------------------------------------------------------------------------
// Structured input examples
// ---------------------------------------------------------------------------------------------

/// One command `demo.docs.Submit(name, body: <body>)` with `body`'s example and an optional
/// refusal guard.
fn submit(types: &str, body: &str, example: &str, guard: Option<&str>) -> String {
    let refusal = guard.map_or(String::new(), |when| {
        format!(
            "      - name: unsupported
        when: '{when}'
        error: demo.docs.Unsupported
        payload:
          demo.docs.Unsupported: {{name: input.name}}
"
        )
    });
    format!(
        "format: ess/33
system: demo
version: v1
domain: demo.docs
types:
{types}entities:
  - name: demo.docs.Doc
    identity: {{name: doc_id, type: Uuid}}
    fields:
      - {{name: name, type: String}}
    lifecycle: {{initial: Draft, states: [Draft], terminal: [Draft]}}
errors:
  - name: demo.docs.Unsupported
    fields:
      - {{name: name, type: String}}
events:
  - name: demo.docs.Submitted
    fields:
      - {{name: doc_id, type: Uuid}}
commands:
  - name: demo.docs.Submit
    input:
      - {{name: name, type: String}}
      - {{name: body, type: \"{body}\", example: {example}}}
    outcomes:
{refusal}      - name: accepted
        creates: demo.docs.Doc
        instance: doc_id
        sets: {{name: input.name}}
        emits: [demo.docs.Submitted]
        payload:
          demo.docs.Submitted: {{doc_id: {{generated: true}}}}
"
    )
}

const ACCEPTED: &str = "demo.docs.Submit/outcome/accepted";
const UNSUPPORTED: &str = "demo.docs.Submit/outcome/unsupported";

/// The story: "an `Optional` member the example leaves out stays out", and "a struct example's
/// declared members as examples at `<path>.<member>` (so guards reading them keep their ladders)".
/// A guard reading the `Optional` member the example leaves out keeps the scenario that takes it:
/// the refusing step sends `note: urgent`, the accepted one the example, and both pass.
#[test]
fn a_guard_over_an_optional_member_the_example_leaves_out_keeps_its_ladder() {
    let types = "  - name: demo.docs.Body
    kind: struct
    fields:
      - {name: version, type: String}
      - {name: note, type: Optional<String>}
";
    let text = submit(
        types,
        "demo.docs.Body",
        "{version: \"1\"}",
        Some("body.note == \"urgent\""),
    );
    // Control: the example naming `note` too synthesizes with no refusal, so what follows measures
    // the member the example leaves out.
    let named = text.replace(
        "example: {version: \"1\"}",
        "example: {version: \"1\", note: calm}",
    );
    assert_ne!(named, text);
    let control = ess_conformance::synthesize::synthesize(&ir(&named));
    assert!(
        control.refusals.is_empty(),
        "control, the example names `note`: {:#?}",
        refusals(&control)
    );
    let (result, statuses) = synthesized_and_run(&text);
    // A guard over an absent `Optional` is undecided, so the accepted step cannot send the example
    // as written: it keeps `version` and takes a `note` from the guard's ladder that is not
    // `urgent` (wave D325 adversary pass 1, F4).
    for value in sent(&result, ACCEPTED, "demo.docs.Submit", "body") {
        let members = value.as_map().expect("the accepted step sends a struct");
        assert_eq!(
            members.get("version"),
            Some(&Node::Text("1".to_owned())),
            "story:a-structured-input-may-carry-a-fixed-example: the accepted step keeps the example's `version`: {value}"
        );
        assert!(
            matches!(members.get("note"), Some(Node::Text(note)) if note != "urgent"),
            "story:a-structured-input-may-carry-a-fixed-example: the accepted step takes a `note` that is not `urgent`: {value}"
        );
    }
    let refuting = sent(&result, UNSUPPORTED, "demo.docs.Submit", "body");
    assert!(!refuting.is_empty(), "the refusing scenario sends body");
    for value in refuting {
        assert_eq!(
            value.as_map().and_then(|members| members.get("note")),
            Some(&Node::Text("urgent".to_owned())),
            "{value}"
        );
    }
    all_passed(&statuses);
}

/// A struct example's members of every structured kind are sent as written: a list member, a map
/// member, a `Json` member and a newtype member, and the run passes.
#[test]
fn a_struct_examples_structured_members_are_sent_as_written() {
    let types = "  - {name: demo.docs.Version, kind: newtype, of: String, prefix: v}
  - name: demo.docs.Body
    kind: struct
    fields:
      - {name: version, type: demo.docs.Version}
      - {name: tags, type: List<String>}
      - {name: limits, type: \"Map<String, Integer>\"}
      - {name: meta, type: Json}
";
    let example = "{version: v7, tags: [alpha, beta], limits: {low: 1, high: 5}, meta: {steps: [a, b], depth: 2}}";
    let text = submit(types, "demo.docs.Body", example, None);
    let (result, statuses) = synthesized_and_run(&text);
    let values = sent(&result, ACCEPTED, "demo.docs.Submit", "body");
    assert!(!values.is_empty(), "the accepted step sends body");
    for value in values {
        assert_eq!(value, yaml(example), "the accepted step sends the example");
    }
    all_passed(&statuses);
}

/// An example on the identity input a provided-row guard reads: the linked branch still sends the
/// row's fixture, the absent branch a literal, and both pass.
#[test]
fn an_example_on_the_input_a_provided_row_guard_reads_yields_to_the_fixture() {
    let text = LINKS.replace(
        "    input:\n      - {name: app_id, type: linking.links.AppId}\n",
        "    input:\n      - {name: app_id, type: linking.links.AppId, example: 00000000-0000-4000-8000-000000000001}\n",
    );
    assert_ne!(text, LINKS);
    let (result, statuses) = synthesized_and_run(&text);
    let linked = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "linking.links.CreateLink/outcome/linked")
        .map(|(_, scenario)| scenario)
        .expect("linked");
    assert!(
        linked.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::ExecuteCommand { input, .. }
                if matches!(input.get("app_id"), Some(ScenarioValue::Fixture { .. }))
        )),
        "{:#?}",
        linked.steps
    );
    all_passed(&statuses);
}
