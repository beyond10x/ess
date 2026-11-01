//! A struct, `Map` or `Json` input's `example:` is what synthesis sends (ess/33, a downstream
//! report, 2026-10-30).
//!
//! Without one, an open struct with no declared members is sent `{}` and a `Json` input its own
//! path, and a service that requires a version member refuses every accepted step. With one, the
//! plain instance sends the example: a `Map` or `Json` one as written, and a struct one through its
//! members, so a guard over a declared member keeps the ladder that refutes it and an undeclared
//! member is carried as written.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The brand-free reproduction: a document input too large to type, declared an open struct
/// with no members, beside an alternative `preset`.
const MODEL: &str =
    include_str!("../../../specify/ess-domain/tests/fixtures/structured-input-example.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn yaml(text: &str) -> Node {
    ess_yaml::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

fn synthesized(text: &str) -> Synthesis {
    let result = synthesize(&ir(text));
    assert!(
        result.refusals.is_empty(),
        "no refusal: {:?}",
        result.refusals
    );
    result
}

/// Every value `scenario`'s steps executing `command` sent at `field`, where they sent one.
fn sent(result: &Synthesis, scenario: &str, command: &str, field: &str) -> Vec<Node> {
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {scenario}: {:?}",
                result.suite.scenarios.keys().collect::<Vec<_>>()
            )
        })
        .1;
    let mut values = Vec::new();
    for step in &scenario.steps {
        if let ScenarioStep::ExecuteCommand {
            command: executed,
            input,
            ..
        } = step
        {
            if executed.to_string() != command {
                continue;
            }
            match input.get(field) {
                Some(ScenarioValue::Literal { value }) => values.push(value.clone()),
                None => {}
                other => panic!("{field} is {other:?}"),
            }
        }
    }
    values
}

/// One command `demo.docs.Submit(name, body: <body>)` taking `body` with `example`, and a refusal
/// `unsupported` where `guard` reads a declared member.
fn submit(types: &str, body: &str, example: Option<&str>, guard: Option<&str>) -> String {
    let example = example.map_or(String::new(), |value| format!(", example: {value}"));
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
      - {{name: body, type: \"{body}\"{example}}}
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

const OPEN: &str = "  - name: demo.docs.Body
    kind: struct
    undeclared_fields: ignored
    fields: []
";

const VERSIONED: &str = "  - name: demo.docs.Body
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: version, type: String}
";

const ACCEPTED: &str = "demo.docs.Submit/outcome/accepted";

#[test]
fn a_struct_or_json_input_with_an_example_is_sent_as_the_example() {
    // Without an example, the invented witnesses the report named.
    for (types, body, invented) in [
        (OPEN, "demo.docs.Body", "{}"),
        ("  []\n", "Json", "{body: body}"),
    ] {
        let result = synthesized(&submit(types, body, None, None));
        let values = sent(&result, ACCEPTED, "demo.docs.Submit", "body");
        assert!(!values.is_empty(), "{body}: the scenario sends body");
        for value in values {
            assert_eq!(value, yaml(invented), "{body} without an example");
        }
    }
    // With one, the example, at every accepted step that carries the input.
    for (types, body, example) in [
        (OPEN, "demo.docs.Body", "{version: \"1\", nodes: [{id: a}]}"),
        (OPEN, "Optional<demo.docs.Body>", "{version: \"1\"}"),
        ("  []\n", "Json", "{version: 2, steps: [a, b]}"),
        ("  []\n", "Map<String, Integer>", "{low: 1, high: 2}"),
    ] {
        let result = synthesized(&submit(types, body, Some(example), None));
        let values = sent(&result, ACCEPTED, "demo.docs.Submit", "body");
        assert!(!values.is_empty(), "{body}: the scenario sends body");
        for value in values {
            assert_eq!(value, yaml(example), "{body} with an example");
        }
    }
    // A guard over a declared member keeps the scenario that refutes it, which carries the
    // undeclared member still. A `!=` against the example's own value is held in
    // `example_inequality.rs`.
    let example = "{version: \"1\", origin: upload}";
    let result = synthesized(&submit(
        VERSIONED,
        "demo.docs.Body",
        Some(example),
        Some("body.version == \"0\""),
    ));
    for value in sent(&result, ACCEPTED, "demo.docs.Submit", "body") {
        assert_eq!(value, yaml(example), "the accepted step sends the example");
    }
    let refuting = sent(
        &result,
        "demo.docs.Submit/outcome/unsupported",
        "demo.docs.Submit",
        "body",
    );
    assert!(!refuting.is_empty(), "the refusing scenario sends body");
    for value in refuting {
        let members = value.as_map().expect("a struct");
        assert_eq!(
            members.get("version"),
            Some(&Node::Text("0".to_owned())),
            "{value}"
        );
        assert_eq!(
            members.get("origin"),
            Some(&Node::Text("upload".to_owned())),
            "{value}"
        );
    }
}

#[test]
fn the_minimal_reproduction_synthesizes_and_passes_on_the_interpreted_target() {
    let result = synthesized(MODEL);
    let body = yaml("{version: \"1\", nodes: [{id: start, kind: entry}]}");
    let carried = sent(
        &result,
        "demo.docs.CreateDoc/outcome/created",
        "demo.docs.CreateDoc",
        "body",
    );
    assert!(!carried.is_empty(), "an accepted step carries body");
    for value in carried {
        assert_eq!(value, body, "the accepted step sends the example");
    }
    for (field, example) in [
        ("header", "{version: \"2\", note: draft, origin: upload}"),
        ("limits", "{low: 1, high: 5}"),
        ("labels", "{team: core}"),
        ("settings", "{retries: 3, mode: fast}"),
        ("parts", "[{kind: intro}]"),
    ] {
        let values = sent(
            &result,
            "demo.docs.ImportDoc/outcome/imported",
            "demo.docs.ImportDoc",
            field,
        );
        assert!(!values.is_empty(), "{field} is sent");
        for value in values {
            assert_eq!(value, yaml(example), "{field}");
        }
    }
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let run = Runner::for_suite(&result.suite).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(ir(MODEL)),
    );
    assert!(!run.scenarios.is_empty(), "the run executes the suite");
    for scenario in &run.scenarios {
        assert_eq!(
            scenario.status,
            Status::Passed,
            "{}: {scenario:#?}",
            scenario.scenario
        );
    }
}
