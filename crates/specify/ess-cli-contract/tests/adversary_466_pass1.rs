//! Adversary pass 1 for beyond10x/ess#466 (`source: {kind: trailing}`): admission boundaries the
//! unit's own cases do not state — the closed reader on every extra key, more refused shapes,
//! the at-most-one rule's scope, stdin counting, the positional rule beside the list, dynamic
//! targets and wire-renamed fields.

use ess_cli_contract::wire::{ArgumentSource, Plan};
use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = "format: ess/1
system: demo
version: v1
types:
  - name: demo.Word
    kind: newtype
    of: String
  - name: demo.Mode
    kind: enum
    variants: [Safe, Fast]
  - name: demo.Pair
    kind: struct
    fields:
      - {name: left, type: String}
  - name: demo.LaunchInput
    kind: struct
    fields:
      - {name: connection, type: String}
      - {name: target, type: 'Optional<String>'}
      - {name: body, type: String}
      - {name: args, type: 'List<String>'}
  - name: demo.Dynamic
    kind: struct
    fields:
      - {name: operation, type: String}
      - {name: schema, type: String}
      - {name: payload, type: String}
      - {name: args, type: 'List<String>'}
  - name: demo.Launched
    kind: struct
    fields:
      - {name: status, type: String}
";

const BINDING: &str = "format: ess-cli/1
binary: demo
about: Trailing admission adversary
globals: {config: config, state: state-dir, output: output}
callables:
  launch:
    target: {kind: local, owner: demo.cli, action: launch}
    input: demo.LaunchInput
    result: demo.Launched
  dynamic:
    target: {kind: dynamic, owner: demo.cli, operation_field: operation, schema_field: schema, payload_field: payload}
    input: demo.Dynamic
    result: demo.Launched
commands:
  - path: [launch]
    callable: launch
    about: Launch
    arguments:
      - {field: args, source: {kind: trailing}}
      - {field: connection, source: {kind: option, long: connection}}
      - {field: target, source: {kind: positional, index: 1}}
      - {field: body, source: {kind: document, inline: body, file: body-file, stdin: body-stdin}}
  - path: [dynamic]
    callable: dynamic
    about: Dynamic
    arguments:
      - {field: operation, source: {kind: option, long: operation}}
      - {field: schema, source: {kind: option, long: schema}}
      - {field: payload, source: {kind: document, inline: payload, file: payload-file, stdin: payload-stdin}}
      - {field: args, source: {kind: trailing}}
";

fn model(text: &str) -> EssIr {
    let specification = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap()
}

fn compiled(model_text: &str, binding_text: &str) -> Result<CompiledBinding, String> {
    compile(
        &model(model_text),
        &Binding::from_yaml(binding_text).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[test]
fn adv466_control_binding_compiles_with_a_trailing_list_beside_every_other_kind() {
    let compiled = compiled(MODEL, BINDING).unwrap();
    for command in &compiled.plan().commands {
        assert_eq!(
            command
                .arguments
                .iter()
                .filter(|a| matches!(a.source, ArgumentSource::Trailing {}))
                .count(),
            1,
            "{:?}",
            command.path
        );
    }
}

#[test]
fn adv466_closed_reader_refuses_every_extra_key_on_trailing() {
    for extra in [
        "long: args",
        "index: 1",
        "index: ~",
        "inline: args",
        "file: args-file",
        "stdin: args-stdin",
        "hidden_tty: args-prompt",
        "field: args",
        "required: true",
        "unknown: false",
    ] {
        let binding = BINDING.replacen(
            "{field: args, source: {kind: trailing}}",
            &format!("{{field: args, source: {{kind: trailing, {extra}}}}}"),
            1,
        );
        assert_ne!(binding, BINDING);
        assert!(Binding::from_yaml(&binding).is_err(), "accepted {extra}");
    }
    // The kind is a tagged object, never a bare word.
    let bare = BINDING.replacen("source: {kind: trailing}", "source: trailing", 1);
    assert!(Binding::from_yaml(&bare).is_err());
}

/// The plan the generated package reads back is closed on the new variant as well.
#[test]
fn adv466_plan_reader_round_trips_and_stays_closed() {
    let text = compiled(MODEL, BINDING).unwrap().to_canonical_json();
    let read: Plan = serde_json::from_str(&text).unwrap();
    assert_eq!(
        format!("{}\n", serde_json::to_string_pretty(&read).unwrap()),
        text
    );
    let widened = text.replacen(
        "\"kind\": \"trailing\"",
        "\"kind\": \"trailing\", \"long\": \"x\"",
        1,
    );
    assert_ne!(widened, text);
    assert!(serde_json::from_str::<Plan>(&widened).is_err());
}

#[test]
fn adv466_trailing_refuses_every_shape_but_a_required_list_of_strings() {
    for shape in [
        "Boolean",
        "Integer",
        "demo.Word",
        "demo.Mode",
        "'List<Boolean>'",
        "'List<Integer>'",
        "'List<List<String>>'",
        "'List<demo.Mode>'",
        "'List<demo.Pair>'",
        "'Map<String, String>'",
        "'Optional<List<demo.Word>>'",
        "'Optional<String>'",
    ] {
        let text = MODEL.replacen(
            "{name: args, type: 'List<String>'}",
            &format!("{{name: args, type: {shape}}}"),
            1,
        );
        assert_ne!(text, MODEL);
        let error = compiled(&text, BINDING).unwrap_err();
        assert!(
            error.contains("a trailing source binds a required List<String> field"),
            "{shape}: {error}"
        );
    }
}

#[test]
fn adv466_at_most_one_trailing_is_per_command_not_per_binding() {
    // Two commands each holding one trailing list compile (the control binding has two).
    assert!(compiled(MODEL, BINDING).is_ok());
    // One field cannot be both the list and an option.
    let twice = BINDING.replacen(
        "      - {field: connection, source: {kind: option, long: connection}}\n",
        "      - {field: connection, source: {kind: option, long: connection}}\n      - {field: args, source: {kind: option, long: args}}\n",
        1,
    );
    assert_ne!(twice, BINDING);
    let error = compiled(MODEL, &twice).unwrap_err();
    assert!(error.contains("mapped more than once"), "{error}");
    // The same field twice as a trailing list is a mapping refusal before the count.
    let doubled = BINDING.replacen(
        "      - {field: args, source: {kind: trailing}}\n",
        "      - {field: args, source: {kind: trailing}}\n      - {field: args, source: {kind: trailing}}\n",
        1,
    );
    assert!(compiled(MODEL, &doubled).is_err());
}

#[test]
fn adv466_trailing_reads_no_stdin_and_does_not_fill_a_positional_gap() {
    // The control holds a stdin-capable document beside the list: the list is no stdin source.
    assert!(compiled(MODEL, BINDING).is_ok());
    // A positional index 2 with no index 1 stays refused; the list does not take index 1.
    let gap = BINDING.replacen(
        "kind: positional, index: 1",
        "kind: positional, index: 2",
        1,
    );
    let error = compiled(MODEL, &gap).unwrap_err();
    assert!(error.contains("positionals must be consecutive"), "{error}");
}

#[test]
fn adv466_trailing_cannot_bind_a_dynamic_string_field() {
    let swapped = BINDING
        .replacen(
            "      - {field: payload, source: {kind: document, inline: payload, file: payload-file, stdin: payload-stdin}}\n      - {field: args, source: {kind: trailing}}\n",
            "      - {field: payload, source: {kind: trailing}}\n      - {field: args, source: {kind: option, long: args}}\n",
            1,
        );
    assert_ne!(swapped, BINDING);
    let error = compiled(MODEL, &swapped).unwrap_err();
    assert!(
        error.contains("a trailing source binds a required List<String> field"),
        "{error}"
    );
}

#[test]
fn adv466_wire_renamed_list_is_bound_under_its_wire_name() {
    let renamed = MODEL.replacen(
        "      - {name: args, type: 'List<String>'}\n  - name: demo.Dynamic",
        "      - name: args\n        type: 'List<String>'\n        wire: argv\n  - name: demo.Dynamic",
        1,
    );
    assert_ne!(renamed, MODEL);
    let compiled = compiled(&renamed, BINDING).unwrap();
    let launch = &compiled.plan().commands[1];
    assert_eq!(launch.path, ["launch"]);
    let trailing = launch
        .arguments
        .iter()
        .find(|a| matches!(a.source, ArgumentSource::Trailing {}))
        .unwrap();
    assert_eq!(trailing.field, "argv");
}
