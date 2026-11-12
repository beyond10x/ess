//! A suffixed instance is created with the inputs the specification gives
//! (`story:a-suffixed-instance-is-created-with-the-inputs-the-specification-gives`,
//! beyond10x/ess#736).
//!
//! A scenario that creates a second instance of an entity (`widget-2`, `room-2`) beside its own
//! subject builds that step's input the way the specification says:
//!
//! - an input's `example:` is sent for every instance, and only the identity, and a field the
//!   search steered, differs between them. Before, the second create sent generated placeholders,
//!   so an implementation needing a member the example carries refused it;
//! - a second create forced through an `external:` branch beside a `when_related: {exists:
//!   false}` sibling references a related row the scenario declared, as the non-external path
//!   does. Before, it sent a literal no step declared, and the specification answered the
//!   sibling.
//!
//! A command without examples synthesizes the suite it did before, byte for byte.
//!
//! A stored field some command's branch decision reads keeps telling the instances apart: a
//! uniqueness check over the created rows, and another command's related-row predicate over a
//! further row, keep the value the search gave it, and the branch keeps its scenario.

use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use sha2::{Digest, Sha256};

/// The issue's first reproduction, as one file: a widget created with a `name` and a `config`
/// example, and a view filtered by the identity, for which a second widget is arranged.
const WIDGET: &str = "format: ess/33
system: demo
version: v1
domain: demo.widget
types:
  - {name: demo.widget.WidgetId, kind: newtype, of: String}
entities:
  - name: demo.widget.Widget
    identity: {name: widget_id, type: demo.widget.WidgetId}
    fields:
      - {name: name, type: String}
      - {name: config, type: \"Optional<Map<String, Json>>\"}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
actors:
  - name: demo.widget.User
    attributes: []
    may:
      - demo.widget.CreateWidget
      - demo.widget.RenameWidget
      - demo.widget.WidgetById
      - demo.widget.Widgets
    naming: {display: User}
errors:
  - name: demo.widget.WidgetExists
    summary: A widget of that id exists already.
    fields: []
  - name: demo.widget.UnknownWidget
    summary: No widget of that id.
    fields: []
commands:
  - name: demo.widget.CreateWidget
    input:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String, example: \"Front door\"}
      - name: config
        type: Optional<Map<String, Json>>
        example: {type: object, properties: {color: {type: string}}}
    outcomes:
      - name: exists
        existing_instance: true
        error: demo.widget.WidgetExists
      - name: created
        creates: demo.widget.Widget
        instance: widget_id
        sets:
          name: input.name
          config: input.config
        emits: [demo.widget.WidgetCreated]
        payload:
          demo.widget.WidgetCreated: {widget_id: input.widget_id}
  - name: demo.widget.RenameWidget
    input:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String, example: \"Back door\"}
    outcomes:
      - name: unknown-widget
        unknown_instance: true
        error: demo.widget.UnknownWidget
      - name: renamed
        updates: demo.widget.Widget
        instance: widget_id
        sets:
          name: input.name
        emits: [demo.widget.WidgetRenamed]
        payload:
          demo.widget.WidgetRenamed: {widget_id: input.widget_id}
events:
  - name: demo.widget.WidgetCreated
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
  - name: demo.widget.WidgetRenamed
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
views:
  - name: demo.widget.WidgetById
    source: demo.widget.Widget
    consistency: read_your_writes
    params:
      - {name: widget_id, type: demo.widget.WidgetId}
    filter: widget_id == param.widget_id
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String}
  - name: demo.widget.Widgets
    source: demo.widget.Widget
    consistency: read_your_writes
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String}
";

/// The issue's second reproduction, as one file: a room opens on a declared model through an
/// `external:` branch, beside a sibling refusing an undeclared model.
const ROOM: &str = "format: ess/33
system: demo
version: v1
domain: demo.room
types:
  - {name: demo.room.ModelId, kind: newtype, of: String}
  - {name: demo.room.RoomId, kind: newtype, of: String}
entities:
  - name: demo.room.Model
    identity: {name: model_id, type: demo.room.ModelId}
    fields: []
    lifecycle:
      initial: Declared
      states: [Declared]
      terminal: [Declared]
  - name: demo.room.Room
    identity: {name: room_id, type: demo.room.RoomId}
    fields:
      - {name: model_id, type: demo.room.ModelId}
      - {name: notes, type: Integer}
    relations:
      - {name: model, kind: references, target: demo.room.Model, cardinality: one, via: model_id}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
actors:
  - name: demo.room.User
    attributes: []
    may:
      - demo.room.DeclareModel
      - demo.room.OpenRoom
      - demo.room.AddNote
      - demo.room.Rooms
      - demo.room.RoomById
    naming: {display: User}
errors:
  - name: demo.room.UnknownModel
    summary: No model of that id is declared.
    fields: []
  - name: demo.room.UpstreamRefused
    summary: The upstream refused the room; none is open.
    fields: []
  - name: demo.room.UnknownRoom
    summary: No room of that id.
    fields: []
commands:
  - name: demo.room.DeclareModel
    input:
      - {name: model_id, type: demo.room.ModelId}
    outcomes:
      - name: declared
        creates: demo.room.Model
        instance: model_id
        emits: [demo.room.ModelDeclared]
        payload:
          demo.room.ModelDeclared: {model_id: input.model_id}
  - name: demo.room.OpenRoom
    input:
      - {name: model_id, type: demo.room.ModelId}
    outcomes:
      - name: unknown-model
        when_related: {via: input.model_id, exists: false}
        error: demo.room.UnknownModel
      - name: opened
        external: the upstream accepts the room
        creates: demo.room.Room
        instance: room_id
        sets:
          model_id: input.model_id
          notes: 0
        emits: [demo.room.RoomOpened]
        payload:
          demo.room.RoomOpened:
            room_id: {generated: true}
            model_id: input.model_id
      - name: upstream-refused
        error: demo.room.UpstreamRefused
  - name: demo.room.AddNote
    input:
      - {name: room_id, type: demo.room.RoomId}
    outcomes:
      - name: unknown-room
        unknown_instance: true
        error: demo.room.UnknownRoom
      - name: added
        updates: demo.room.Room
        instance: room_id
        emits: [demo.room.NoteAdded]
        payload:
          demo.room.NoteAdded: {room_id: input.room_id}
events:
  - name: demo.room.ModelDeclared
    fields:
      - {name: model_id, type: demo.room.ModelId}
  - name: demo.room.RoomOpened
    fields:
      - {name: room_id, type: demo.room.RoomId}
      - {name: model_id, type: demo.room.ModelId}
  - name: demo.room.NoteAdded
    fields:
      - {name: room_id, type: demo.room.RoomId}
views:
  - name: demo.room.RoomById
    source: demo.room.Room
    consistency: read_your_writes
    params:
      - {name: room_id, type: demo.room.RoomId}
    filter: room_id == param.room_id
    fields:
      - {name: room_id, type: demo.room.RoomId}
      - {name: model_id, type: demo.room.ModelId}
  - name: demo.room.Rooms
    source: demo.room.Room
    consistency: read_your_writes
    fields:
      - {name: room_id, type: demo.room.RoomId}
      - {name: model_id, type: demo.room.ModelId}
";

/// The widget, its creating command refusing a `name` another widget carries already: a
/// uniqueness check over the stored rows, reading the input that has an example.
const UNIQUE: &str = "format: ess/33
system: demo
version: v1
domain: demo.widget
types:
  - {name: demo.widget.WidgetId, kind: newtype, of: String}
entities:
  - name: demo.widget.Widget
    identity: {name: widget_id, type: demo.widget.WidgetId}
    fields:
      - {name: name, type: String}
      - {name: config, type: \"Optional<Map<String, Json>>\"}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
actors:
  - name: demo.widget.User
    attributes: []
    may:
      - demo.widget.CreateWidget
      - demo.widget.WidgetById
      - demo.widget.Widgets
    naming: {display: User}
errors:
  - name: demo.widget.WidgetExists
    summary: A widget of that id exists already.
    fields: []
  - name: demo.widget.NameTaken
    summary: A widget of that name exists already.
    fields: []
commands:
  - name: demo.widget.CreateWidget
    input:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String, example: \"Front door\"}
      - name: config
        type: Optional<Map<String, Json>>
        example: {type: object, properties: {color: {type: string}}}
    outcomes:
      - name: exists
        existing_instance: true
        error: demo.widget.WidgetExists
      - name: name-taken
        when_related: {entity: demo.widget.Widget, where: name == input.name, exists: true}
        error: demo.widget.NameTaken
      - name: created
        creates: demo.widget.Widget
        instance: widget_id
        sets:
          name: input.name
          config: input.config
        emits: [demo.widget.WidgetCreated]
        payload:
          demo.widget.WidgetCreated: {widget_id: input.widget_id}
events:
  - name: demo.widget.WidgetCreated
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
views:
  - name: demo.widget.WidgetById
    source: demo.widget.Widget
    consistency: read_your_writes
    params:
      - {name: widget_id, type: demo.widget.WidgetId}
    filter: widget_id == param.widget_id
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String}
  - name: demo.widget.Widgets
    source: demo.widget.Widget
    consistency: read_your_writes
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String}
";

/// The widget, and a tag put on a widget unless the widget's stored `name` is the protected one,
/// which is the `name` example. Every widget the tagged scenario creates is a further instance,
/// and the tag reads its stored name.
const TAGGED: &str = "format: ess/33
system: demo
version: v1
domain: demo.widget
types:
  - {name: demo.widget.WidgetId, kind: newtype, of: String}
  - {name: demo.widget.TagId, kind: newtype, of: String}
entities:
  - name: demo.widget.Widget
    identity: {name: widget_id, type: demo.widget.WidgetId}
    fields:
      - {name: name, type: String}
      - {name: config, type: \"Optional<Map<String, Json>>\"}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
  - name: demo.widget.Tag
    identity: {name: tag_id, type: demo.widget.TagId}
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
    relations:
      - {name: widget, kind: references, target: demo.widget.Widget, cardinality: one, via: widget_id}
    lifecycle:
      initial: Put
      states: [Put]
      terminal: [Put]
actors:
  - name: demo.widget.User
    attributes: []
    may:
      - demo.widget.CreateWidget
      - demo.widget.TagWidget
      - demo.widget.WidgetById
      - demo.widget.Widgets
      - demo.widget.TagById
      - demo.widget.Tags
    naming: {display: User}
errors:
  - name: demo.widget.WidgetExists
    summary: A widget of that id exists already.
    fields: []
  - name: demo.widget.UnknownWidget
    summary: No widget of that id.
    fields: []
  - name: demo.widget.Protected
    summary: The widget is protected from tags.
    fields: []
commands:
  - name: demo.widget.CreateWidget
    input:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String, example: \"Front door\"}
      - name: config
        type: Optional<Map<String, Json>>
        example: {type: object, properties: {color: {type: string}}}
    outcomes:
      - name: exists
        existing_instance: true
        error: demo.widget.WidgetExists
      - name: created
        creates: demo.widget.Widget
        instance: widget_id
        sets:
          name: input.name
          config: input.config
        emits: [demo.widget.WidgetCreated]
        payload:
          demo.widget.WidgetCreated: {widget_id: input.widget_id}
  - name: demo.widget.TagWidget
    input:
      - {name: tag_id, type: demo.widget.TagId}
      - {name: widget_id, type: demo.widget.WidgetId}
    outcomes:
      - name: unknown-widget
        when_related: {via: input.widget_id, exists: false}
        error: demo.widget.UnknownWidget
      - name: protected
        when_related:
          via: input.widget_id
          predicate: 'name == \"Front door\"'
        error: demo.widget.Protected
      - name: tagged
        creates: demo.widget.Tag
        instance: tag_id
        sets:
          widget_id: input.widget_id
        emits: [demo.widget.Tagged]
        payload:
          demo.widget.Tagged: {tag_id: input.tag_id}
events:
  - name: demo.widget.WidgetCreated
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
  - name: demo.widget.Tagged
    fields:
      - {name: tag_id, type: demo.widget.TagId}
views:
  - name: demo.widget.WidgetById
    source: demo.widget.Widget
    consistency: read_your_writes
    params:
      - {name: widget_id, type: demo.widget.WidgetId}
    filter: widget_id == param.widget_id
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String}
  - name: demo.widget.Widgets
    source: demo.widget.Widget
    consistency: read_your_writes
    fields:
      - {name: widget_id, type: demo.widget.WidgetId}
      - {name: name, type: String}
  - name: demo.widget.TagById
    source: demo.widget.Tag
    consistency: read_your_writes
    params:
      - {name: tag_id, type: demo.widget.TagId}
    filter: tag_id == param.tag_id
    fields:
      - {name: tag_id, type: demo.widget.TagId}
      - {name: widget_id, type: demo.widget.WidgetId}
  - name: demo.widget.Tags
    source: demo.widget.Tag
    consistency: read_your_writes
    fields:
      - {name: tag_id, type: demo.widget.TagId}
      - {name: widget_id, type: demo.widget.WidgetId}
";

/// The examples `UNIQUE` and `TAGGED` declare, removed.
fn without_examples(text: &str) -> String {
    let text = with(text, ", example: \"Front door\"", "");
    with(
        &text,
        "\n        example: {type: object, properties: {color: {type: string}}}",
        "",
    )
}

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

/// `text` with every `from` replaced by `to`, where `from` occurs.
fn with(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from:?}");
    text.replace(from, to)
}

/// Every scenario of the suite passes on the interpreted target, and there are `count` of them.
fn passes_on_the_interpreter(text: &str, result: &Synthesis, count: usize) {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let run = Runner::for_suite(&result.suite).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(ir(text)),
    );
    assert_eq!(run.scenarios.len(), count, "{run:#?}");
    for scenario in &run.scenarios {
        assert_eq!(
            scenario.status,
            Status::Passed,
            "{}: {scenario:#?}",
            scenario.scenario
        );
    }
}

/// Each `command` a scenario executes, with the input it sends, by scenario.
fn executed<'s>(
    result: &'s Synthesis,
    command: &str,
) -> Vec<(
    String,
    Vec<&'s std::collections::BTreeMap<String, ScenarioValue>>,
)> {
    result
        .suite
        .scenarios
        .iter()
        .map(|(id, scenario)| {
            let inputs = scenario
                .steps
                .iter()
                .filter_map(|step| match step {
                    ScenarioStep::ExecuteCommand {
                        command: executed,
                        input,
                        ..
                    } if executed.to_string() == command => Some(input),
                    _ => None,
                })
                .collect();
            (id.to_string(), inputs)
        })
        .collect()
}

/// Each input sent to `command` whose next step expects `outcome`, by scenario.
fn sent_expecting<'s>(
    result: &'s Synthesis,
    command: &str,
    outcome: &str,
) -> Vec<(
    String,
    Vec<&'s std::collections::BTreeMap<String, ScenarioValue>>,
)> {
    result
        .suite
        .scenarios
        .iter()
        .map(|(id, scenario)| {
            let inputs = scenario
                .steps
                .iter()
                .zip(scenario.steps.iter().skip(1))
                .filter_map(|(step, next)| match (step, next) {
                    (
                        ScenarioStep::ExecuteCommand {
                            command: executed,
                            input,
                            ..
                        },
                        ScenarioStep::ExpectOutcome { outcome: expected },
                    ) if executed.to_string() == command
                        && expected.outcome.as_str() == outcome =>
                    {
                        Some(input)
                    }
                    _ => None,
                })
                .collect();
            (id.to_string(), inputs)
        })
        .collect()
}

fn digest(bytes: &[u8]) -> String {
    let hex = Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    format!("{hex}:{}", bytes.len())
}

/// The canonical JSON of the whole suite, its digest and length.
fn suite_digest(result: &Synthesis) -> String {
    let json = result
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| panic!("{error:?}"));
    digest(json.as_bytes())
}

#[test]
fn a_suffixed_instance_sends_the_input_examples() {
    let result = synthesized(WIDGET);
    let name = Node::Text("Front door".to_owned());
    let config = yaml("{type: object, properties: {color: {type: string}}}");
    let creates = executed(&result, "demo.widget.CreateWidget");
    // The issue names three scenarios creating a further widget: `widget-2` beside the subject in
    // `created` and `renamed`, and the existing row of `exists`.
    let further = creates
        .iter()
        .filter(|(_, inputs)| inputs.len() > 1)
        .count()
        + usize::from(
            creates
                .iter()
                .any(|(id, inputs)| id.ends_with("/outcome/exists") && !inputs.is_empty()),
        );
    assert!(further >= 3, "a further widget is created: {creates:#?}");
    for (id, inputs) in &creates {
        let mut identities = Vec::new();
        for input in inputs {
            assert_eq!(
                input.get("name"),
                Some(&ScenarioValue::literal(name.clone())),
                "{id}: every create of a widget sends the name example: {inputs:#?}"
            );
            assert_eq!(
                input.get("config"),
                Some(&ScenarioValue::literal(config.clone())),
                "{id}: every create of a widget sends the config example: {inputs:#?}"
            );
            identities.push(input.get("widget_id"));
        }
        // The identity alone tells the widgets of one scenario apart, except where the scenario
        // repeats it on purpose: `exists` sends the arranged row's own identity again.
        if !id.ends_with("/outcome/exists") {
            let distinct: std::collections::BTreeSet<String> = identities
                .iter()
                .map(|identity| format!("{identity:?}"))
                .collect();
            assert_eq!(distinct.len(), identities.len(), "{id}: {inputs:#?}");
        }
    }
    passes_on_the_interpreter(WIDGET, &result, 4);
}

#[test]
fn a_forced_external_create_of_a_second_instance_references_a_declared_row() {
    let result = synthesized(ROOM);
    let mut opened_twice = 0;
    for (id, scenario) in &result.suite.scenarios {
        let mut models = Vec::new();
        let mut opens = 0;
        let mut forced = false;
        for step in &scenario.steps {
            match step {
                ScenarioStep::CaptureInstance {
                    instance, entity, ..
                } if entity.to_string() == "demo.room.Model" => models.push(instance.clone()),
                ScenarioStep::ConfigureExternalOutcome { force, .. }
                    if force.command.to_string() == "demo.room.OpenRoom"
                        && force.outcome.as_str() == "opened" =>
                {
                    forced = true;
                }
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "demo.room.OpenRoom" && forced =>
                {
                    forced = false;
                    opens += 1;
                    match input.get("model_id") {
                        Some(ScenarioValue::Instance { instance }) => assert!(
                            models.contains(instance),
                            "{id}: a forced open names a model the scenario captured before it, \
                             not {instance}: {:#?}",
                            scenario.steps
                        ),
                        other => panic!(
                            "{id}: a forced open names a declared model, not {other:?}: {:#?}",
                            scenario.steps
                        ),
                    }
                }
                _ => {}
            }
        }
        if opens > 1 {
            opened_twice += 1;
        }
    }
    assert!(
        opened_twice >= 2,
        "`AddNote/outcome/added` and `OpenRoom/outcome/opened` open a second room"
    );
    passes_on_the_interpreter(ROOM, &result, 6);
}

#[test]
fn a_second_instance_without_examples_is_unchanged() {
    // The widget model without its examples, and the room model without its `external:` line:
    // neither reaches what this story changes, and each synthesizes the suite it did before it.
    let plain = with(
        &with(WIDGET, ", example: \"Front door\"", ""),
        ", example: \"Back door\"",
        "",
    );
    let plain = with(
        &plain,
        "\n        example: {type: object, properties: {color: {type: string}}}",
        "",
    );
    // Without `external:`, `upstream-refused` would be a second unconditional branch, which
    // validation refuses; it goes with it.
    let unforced = with(
        ROOM,
        "\n        external: the upstream accepts the room",
        "",
    );
    let unforced = with(
        &unforced,
        "\n      - name: upstream-refused\n        error: demo.room.UpstreamRefused",
        "",
    );
    for (label, text, pinned) in [
        (
            "widget without examples",
            plain.as_str(),
            "8db043d5be550e648aedba8d1df0eeae64e6e7a889fa10d41cdf84da6b85a677:17116",
        ),
        (
            "room without external",
            unforced.as_str(),
            "6c8db2010f0fb5ffd633c17dc4072eb2c898d067c06826ef095a6f871dfd53ad:22917",
        ),
    ] {
        let result = synthesized(text);
        assert_eq!(suite_digest(&result), pinned, "{label}");
    }
}

/// A uniqueness check on `name` (`when_related: {entity, where: name == input.name}`) is what
/// tells two widgets apart, so `name` differs between them; `config` is read by nothing and is
/// sent as its example. Before, the further widget took the `name` example too, collided with the
/// plain one, and synthesis refused `CreateWidget/created`.
#[test]
fn a_uniqueness_check_tells_further_widgets_apart_in_the_synthesized_inputs() {
    let result = synthesized(UNIQUE);
    let config =
        ScenarioValue::literal(yaml("{type: object, properties: {color: {type: string}}}"));
    let mut further = 0;
    for (id, inputs) in sent_expecting(&result, "demo.widget.CreateWidget", "created") {
        if inputs.len() > 1 {
            further += 1;
        }
        let names: std::collections::BTreeSet<String> = inputs
            .iter()
            .map(|input| format!("{:?}", input.get("name")))
            .collect();
        assert_eq!(
            names.len(),
            inputs.len(),
            "{id}: each widget created in one scenario carries its own name, or `name-taken` \
             answers the further one: {inputs:#?}"
        );
        for input in &inputs {
            assert_eq!(
                input.get("config"),
                Some(&config),
                "{id}: nothing reads `config`, so every widget sends its example: {inputs:#?}"
            );
        }
    }
    assert!(further >= 1, "a scenario creates a further widget");
}

/// `TagWidget` refuses `protected` when the stored `name` of the widget it reads is the `name`
/// example. Every widget the tagged scenario creates is a further one, and the search gave it a
/// name the predicate passes over; that value stays, and the tagged branch keeps its scenario.
/// `config` is read by no branch decision and is sent as its example. Before, the further widget
/// took the `name` example after the search, and synthesis refused `TagWidget/tagged`.
#[test]
fn a_related_row_predicate_on_a_further_widget_keeps_its_branch() {
    let control = without_examples(TAGGED);
    let result = synthesized(&control);
    passes_on_the_interpreter(&control, &result, result.suite.scenarios.len());

    let result = synthesized(TAGGED);
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().ends_with("TagWidget/outcome/tagged")),
        "the tagged branch has a scenario: {:?}",
        result.suite.scenarios.keys().collect::<Vec<_>>()
    );
    let config =
        ScenarioValue::literal(yaml("{type: object, properties: {color: {type: string}}}"));
    for (id, inputs) in sent_expecting(&result, "demo.widget.CreateWidget", "created") {
        for input in &inputs {
            assert_eq!(
                input.get("config"),
                Some(&config),
                "{id}: nothing decides by `config`, so every widget sends its example: \
                 {inputs:#?}"
            );
        }
    }
    passes_on_the_interpreter(TAGGED, &result, result.suite.scenarios.len());
}

/// A create under a uniqueness check, read by identity, with no input examples: the scenario
/// creates a further widget before its subject, and the `WidgetById` read names the subject by the
/// instance it captured. Before, the read sent `{kind: observed}` for `WidgetCreated`, which the
/// runners resolve to the event's first occurrence, the further widget's, so the view was
/// expected to hold the subject's values and held the further widget's.
#[test]
fn a_uniqueness_create_read_by_identity_passes_on_the_interpreter() {
    let control = without_examples(UNIQUE);
    let result = synthesized(&control);
    let mut read = 0;
    for (id, scenario) in &result.suite.scenarios {
        let mut published = false;
        for step in &scenario.steps {
            match step {
                ScenarioStep::CaptureInstance { .. } => published = true,
                ScenarioStep::QueryView { view, params }
                    if view.to_string() == "demo.widget.WidgetById" =>
                {
                    read += 1;
                    let sent = params.get("widget_id");
                    if id.to_string().ends_with("CreateWidget/outcome/created") {
                        assert!(
                            matches!(sent, Some(ScenarioValue::Instance { instance })
                                if instance.to_string() == "widget"),
                            "{id}: the read sends the subject as the captured instance: {sent:?}"
                        );
                    }
                    assert!(
                        !(published && matches!(sent, Some(ScenarioValue::Observed { .. }))),
                        "{id}: an observed reference after an instance was published: {sent:?}"
                    );
                }
                _ => {}
            }
        }
    }
    assert!(read >= 1, "a scenario reads WidgetById");
    passes_on_the_interpreter(&control, &result, result.suite.scenarios.len());
}

/// Adversary: every read of `view` made once the scenario captured its `subject` and, before it,
/// another instance of `event` (so the event's first occurrence is not the subject's), and the
/// ones among them sending `param` as an `observed` reference, which the runners resolve to that
/// first occurrence rather than to the row the scenario is about.
fn adv_observed_after_two_captures(
    result: &Synthesis,
    view: &str,
    param: &str,
    event: &str,
    subject: &str,
) -> (Vec<String>, Vec<String>) {
    let mut reached = Vec::new();
    let mut wrong = Vec::new();
    for (id, scenario) in &result.suite.scenarios {
        let mut captured = Vec::new();
        for step in &scenario.steps {
            match step {
                ScenarioStep::CaptureInstance {
                    instance,
                    event: published,
                    ..
                } if published.to_string() == event => {
                    captured.push(instance.to_string());
                }
                ScenarioStep::QueryView { view: read, params }
                    if read.to_string() == view
                        && captured.iter().any(|name| name == subject)
                        && captured.first().is_some_and(|first| first != subject) =>
                {
                    reached.push(id.to_string());
                    if let Some(sent @ ScenarioValue::Observed { .. }) = params.get(param) {
                        wrong.push(format!(
                            "{id}: {sent:?} after capturing, in order, {captured:?}"
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    (reached, wrong)
}

/// Adversary: `without_examples(UNIQUE)` with an entity invariant every row satisfies (`rev` is
/// set to 0), so `invariant/after` reads it off `WidgetById` after the same run, a further widget
/// created before the subject. The invariant witness is a read by identity like the created
/// scenario's, and it is bound by `identity_of` (synthesize.rs `holds_after`), not by the arm this
/// unit added: it sends `{kind: observed}` and reads the further widget's row.
#[test]
fn adv_an_invariant_read_after_a_further_widget_names_the_subject() {
    let text = without_examples(UNIQUE);
    let text = with(
        &text,
        "      - {name: config, type: \"Optional<Map<String, Json>>\"}\n    lifecycle:",
        "      - {name: config, type: \"Optional<Map<String, Json>>\"}\n      \
         - {name: rev, type: Integer}\n    invariants:\n      - rev >= 0\n    lifecycle:",
    );
    let text = with(
        &text,
        "          config: input.config\n",
        "          config: input.config\n          rev: 0\n",
    );
    let text = with(
        &text,
        "    filter: widget_id == param.widget_id\n    fields:\n",
        "    filter: widget_id == param.widget_id\n    fields:\n      - {name: rev, type: Integer}\n",
    );
    let text = with(
        &text,
        "    consistency: read_your_writes\n    fields:\n",
        "    consistency: read_your_writes\n    fields:\n      - {name: rev, type: Integer}\n",
    );
    let result = synthesize(&ir(&text));
    let ids: Vec<String> = result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id.contains("invariant")),
        "an invariant scenario: {ids:?}, refusals {:?}",
        result.refusals
    );
    // Every row satisfies the invariant, so the honest interpreter passes whichever row is read.
    passes_on_the_interpreter(&text, &result, result.suite.scenarios.len());
    let (reached, wrong) = adv_observed_after_two_captures(
        &result,
        "demo.widget.WidgetById",
        "widget_id",
        "demo.widget.WidgetCreated",
        "widget",
    );
    assert!(
        reached.iter().any(|id| id.contains("invariant")),
        "the invariant scenario reads WidgetById after a further widget preceded the subject: \
         {reached:?}"
    );
    assert!(
        wrong.is_empty(),
        "an observed widget_id after a further widget was published reads that widget: {wrong:#?}"
    );
}

/// Adversary: two uniqueness checks (`name`, `code`), so the created scenario may arrange more
/// than one further widget of the same event before its subject. The read still names the subject
/// by its capture, and the suite passes on the interpreter.
#[test]
fn adv_two_uniqueness_checks_read_the_subject_by_its_capture() {
    let text = without_examples(UNIQUE);
    let text = with(
        &text,
        "      - {name: widget_id, type: demo.widget.WidgetId}\n      - {name: name, type: String}\n      - name: config",
        "      - {name: widget_id, type: demo.widget.WidgetId}\n      - {name: name, type: String}\n      \
         - {name: code, type: String}\n      - name: config",
    );
    let text = with(
        &text,
        "      - {name: config, type: \"Optional<Map<String, Json>>\"}\n    lifecycle:",
        "      - {name: config, type: \"Optional<Map<String, Json>>\"}\n      \
         - {name: code, type: String}\n    lifecycle:",
    );
    let text = with(
        &text,
        "    summary: A widget of that name exists already.\n    fields: []\n",
        "    summary: A widget of that name exists already.\n    fields: []\n  \
         - name: demo.widget.CodeTaken\n    summary: A widget of that code exists already.\n    \
         fields: []\n",
    );
    let text = with(
        &text,
        "        error: demo.widget.NameTaken\n",
        "        error: demo.widget.NameTaken\n      - name: code-taken\n        \
         when_related: {entity: demo.widget.Widget, where: code == input.code, exists: true}\n        \
         error: demo.widget.CodeTaken\n",
    );
    let text = with(
        &text,
        "          config: input.config\n",
        "          config: input.config\n          code: input.code\n",
    );
    let text = with(
        &text,
        "    filter: widget_id == param.widget_id\n    fields:\n",
        "    filter: widget_id == param.widget_id\n    fields:\n      - {name: code, type: String}\n",
    );
    let result = synthesize(&ir(&text));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().ends_with("CreateWidget/outcome/created")),
        "the created scenario: {:?}",
        result.refusals
    );
    let (reached, wrong) = adv_observed_after_two_captures(
        &result,
        "demo.widget.WidgetById",
        "widget_id",
        "demo.widget.WidgetCreated",
        "widget",
    );
    assert!(!reached.is_empty(), "a read after two widgets");
    assert!(wrong.is_empty(), "{wrong:#?}");
    passes_on_the_interpreter(&text, &result, result.suite.scenarios.len());
}

/// Adversary: a further instance of the same event that is not a uniqueness competitor but the
/// row the created one requires: a child folder is created only where the folder `parent_id`
/// names exists (a related-row guard through the input, ess/18), and roots and children publish
/// the same `FolderCreated`, a generated `Uuid` identity.
#[test]
fn adv_a_child_folder_is_read_by_its_capture_not_the_parent() {
    const FOLDER: &str = "format: ess/33
system: demo
version: v1
domain: demo.f
types:
  - {name: demo.f.FolderId, kind: newtype, of: Uuid}
entities:
  - name: demo.f.Folder
    identity: {name: folder_id, type: demo.f.FolderId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
actors:
  - name: demo.f.User
    attributes: []
    may: [demo.f.CreateRoot, demo.f.CreateChild, demo.f.FolderById]
    naming: {display: User}
errors:
  - name: demo.f.UnknownParent
    summary: No folder of that id.
    fields: []
commands:
  - name: demo.f.CreateRoot
    input:
      - {name: label, type: String}
    outcomes:
      - name: created
        creates: demo.f.Folder
        instance: folder_id
        sets: {label: input.label}
        emits: [demo.f.FolderCreated]
        payload:
          demo.f.FolderCreated: {folder_id: {generated: true}}
  - name: demo.f.CreateChild
    input:
      - {name: parent_id, type: demo.f.FolderId}
      - {name: label, type: String}
    outcomes:
      - name: unknown-parent
        when_related: {via: input.parent_id, exists: false}
        error: demo.f.UnknownParent
      - name: created
        creates: demo.f.Folder
        instance: folder_id
        sets: {label: input.label}
        emits: [demo.f.FolderCreated]
        payload:
          demo.f.FolderCreated: {folder_id: {generated: true}}
events:
  - name: demo.f.FolderCreated
    fields:
      - {name: folder_id, type: demo.f.FolderId}
views:
  - name: demo.f.FolderById
    source: demo.f.Folder
    consistency: read_your_writes
    params:
      - {name: folder_id, type: demo.f.FolderId}
    filter: folder_id == param.folder_id
    fields:
      - {name: folder_id, type: demo.f.FolderId}
      - {name: label, type: String}
";
    let result = synthesize(&ir(FOLDER));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().ends_with("CreateChild/outcome/created")),
        "the child's created scenario: {:?}",
        result.refusals
    );
    passes_on_the_interpreter(FOLDER, &result, result.suite.scenarios.len());
    let (_, wrong) = adv_observed_after_two_captures(
        &result,
        "demo.f.FolderById",
        "folder_id",
        "demo.f.FolderCreated",
        "folder",
    );
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Adversary: the repository's key-unique-across-callers model with a read by identity added: a
/// generated `Uuid` identity under a uniqueness check, and a caller-swapped second half that
/// renames the captured instances. Every scenario passes on the interpreter.
#[test]
fn adv_a_key_unique_across_callers_read_by_identity_passes_the_swap() {
    const CLAIMS: &str = "format: ess/25
system: repro
version: v1
domain: repro.names
summary: A name unique across callers.
types:
  - {name: repro.names.CallerId, kind: newtype, of: String}
entities:
  - name: repro.names.Claim
    identity: {name: claim_id, type: Uuid}
    fields:
      - {name: name, type: String}
      - {name: holder, type: repro.names.CallerId}
    lifecycle:
      initial: Held
      states: [Held]
      terminal: [Held]
actors:
  - name: repro.names.Alice
    attributes:
      - {name: id, type: repro.names.CallerId}
    may: [repro.names.TakeName]
  - name: repro.names.Bob
    attributes:
      - {name: id, type: repro.names.CallerId}
    may: [repro.names.TakeName]
errors:
  - name: repro.names.Taken
    fields:
      - {name: name, type: String}
commands:
  - name: repro.names.TakeName
    input:
      - {name: name, type: String}
    outcomes:
      - name: taken
        when_related: {entity: repro.names.Claim, where: name == input.name, exists: true}
        error: repro.names.Taken
        payload:
          repro.names.Taken: {name: input.name}
      - name: took
        creates: repro.names.Claim
        instance: claim_id
        sets: {name: input.name, holder: {caller: id}}
        emits: [repro.names.NameTaken]
        payload:
          repro.names.NameTaken: {claim_id: {generated: true}, name: input.name, holder: {caller: id}}
events:
  - name: repro.names.NameTaken
    fields:
      - {name: claim_id, type: Uuid}
      - {name: name, type: String}
      - {name: holder, type: repro.names.CallerId}
views:
  - name: repro.names.ClaimById
    source: repro.names.Claim
    consistency: read_your_writes
    params:
      - {name: claim_id, type: Uuid}
    filter: claim_id == param.claim_id
    fields:
      - {name: claim_id, type: Uuid}
      - {name: name, type: String}
      - {name: holder, type: repro.names.CallerId}
";
    let result = synthesize(&ir(CLAIMS));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().ends_with("TakeName/outcome/took")),
        "the took scenario: {:?}",
        result.refusals
    );
    let (_, wrong) = adv_observed_after_two_captures(
        &result,
        "repro.names.ClaimById",
        "claim_id",
        "repro.names.NameTaken",
        "claim",
    );
    assert!(wrong.is_empty(), "{wrong:#?}");
    passes_on_the_interpreter(CLAIMS, &result, result.suite.scenarios.len());
}

/// Adversary: the widget's create also requires a gadget, whose identity field has the same name
/// (`id`) and is published by another event (`GadgetCreated`). The setup captures that gadget and
/// a further widget; only the further widget is an earlier occurrence of `WidgetCreated`.
#[test]
#[allow(clippy::too_many_lines)]
fn adv_a_capture_of_another_event_with_the_same_field_name_keeps_the_read() {
    const GADGET: &str = "format: ess/33
system: demo
version: v1
domain: demo.g
types:
  - {name: demo.g.GadgetId, kind: newtype, of: String}
  - {name: demo.g.WidgetId, kind: newtype, of: String}
entities:
  - name: demo.g.Gadget
    identity: {name: id, type: demo.g.GadgetId}
    fields: []
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: demo.g.Widget
    identity: {name: id, type: demo.g.WidgetId}
    fields:
      - {name: gadget_id, type: demo.g.GadgetId}
      - {name: name, type: String}
    relations:
      - {name: gadget, kind: references, target: demo.g.Gadget, cardinality: one, via: gadget_id}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
actors:
  - name: demo.g.User
    attributes: []
    may: [demo.g.CreateGadget, demo.g.CreateWidget, demo.g.WidgetById]
    naming: {display: User}
errors:
  - name: demo.g.WidgetExists
    summary: A widget of that id exists already.
    fields: []
  - name: demo.g.UnknownGadget
    summary: No gadget of that id.
    fields: []
  - name: demo.g.NameTaken
    summary: A widget of that name exists already.
    fields: []
commands:
  - name: demo.g.CreateGadget
    input:
      - {name: id, type: demo.g.GadgetId}
    outcomes:
      - name: created
        creates: demo.g.Gadget
        instance: id
        emits: [demo.g.GadgetCreated]
        payload:
          demo.g.GadgetCreated: {id: input.id}
  - name: demo.g.CreateWidget
    input:
      - {name: id, type: demo.g.WidgetId}
      - {name: gadget_id, type: demo.g.GadgetId}
      - {name: name, type: String}
    outcomes:
      - name: exists
        existing_instance: true
        error: demo.g.WidgetExists
      - name: unknown-gadget
        when_related: {via: input.gadget_id, exists: false}
        error: demo.g.UnknownGadget
      - name: name-taken
        when_related: {entity: demo.g.Widget, where: name == input.name, exists: true}
        error: demo.g.NameTaken
      - name: created
        creates: demo.g.Widget
        instance: id
        sets: {gadget_id: input.gadget_id, name: input.name}
        emits: [demo.g.WidgetCreated]
        payload:
          demo.g.WidgetCreated: {id: input.id}
events:
  - name: demo.g.GadgetCreated
    fields:
      - {name: id, type: demo.g.GadgetId}
  - name: demo.g.WidgetCreated
    fields:
      - {name: id, type: demo.g.WidgetId}
views:
  - name: demo.g.WidgetById
    source: demo.g.Widget
    consistency: read_your_writes
    params:
      - {name: id, type: demo.g.WidgetId}
    filter: id == param.id
    fields:
      - {name: id, type: demo.g.WidgetId}
      - {name: name, type: String}
      - {name: gadget_id, type: demo.g.GadgetId}
";
    let result = synthesize(&ir(GADGET));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string().ends_with("CreateWidget/outcome/created")),
        "the created scenario: {:?}",
        result.refusals
    );
    let (_, wrong) = adv_observed_after_two_captures(
        &result,
        "demo.g.WidgetById",
        "id",
        "demo.g.WidgetCreated",
        "widget",
    );
    assert!(wrong.is_empty(), "{wrong:#?}");
    passes_on_the_interpreter(GADGET, &result, result.suite.scenarios.len());
}

/// A value type's invariant (`Rev`, `value >= 0`) read off `WidgetById` by the scenario that arranges
/// a widget into the view (`holds_at`), a further widget created before the subject. The read names
/// the subject by its capture, as the created scenario's does.
#[test]
fn a_value_invariant_read_after_a_further_widget_names_the_subject() {
    let text = without_examples(UNIQUE);
    let text = with(
        &text,
        "types:\n",
        "types:\n  - {name: demo.widget.Rev, kind: newtype, of: Integer, invariants: ['value >= 0']}\n",
    );
    let text = with(
        &text,
        "      - {name: config, type: \"Optional<Map<String, Json>>\"}\n    lifecycle:",
        "      - {name: config, type: \"Optional<Map<String, Json>>\"}\n      \
         - {name: rev, type: demo.widget.Rev}\n    lifecycle:",
    );
    let text = with(
        &text,
        "          config: input.config\n",
        "          config: input.config\n          rev: 0\n",
    );
    let text = with(
        &text,
        "    filter: widget_id == param.widget_id\n    fields:\n",
        "    filter: widget_id == param.widget_id\n    fields:\n      \
         - {name: rev, type: demo.widget.Rev}\n",
    );
    let result = synthesize(&ir(&text));
    let ids: Vec<String> = result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id.contains("Rev")),
        "a value invariant scenario: {ids:?}, refusals {:?}",
        result.refusals
    );
    passes_on_the_interpreter(&text, &result, result.suite.scenarios.len());
    let (reached, wrong) = adv_observed_after_two_captures(
        &result,
        "demo.widget.WidgetById",
        "widget_id",
        "demo.widget.WidgetCreated",
        "widget",
    );
    assert!(
        reached.iter().any(|id| id.contains("Rev")),
        "the value invariant scenario reads WidgetById after a further widget preceded the \
         subject: {reached:?}"
    );
    assert!(
        wrong.is_empty(),
        "an observed widget_id after a further widget was published reads that widget: {wrong:#?}"
    );
}
