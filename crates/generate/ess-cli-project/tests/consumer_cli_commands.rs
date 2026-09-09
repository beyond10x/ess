//! Application command semantics remain with the independently selected application handler.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProcessOutput, ProtectedSource, Sources,
};
use ess_compiler::EssIr;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};

const MODEL: &str = r"
format: ess/1
system: demo
version: v1
domains: [demo.commands]
domain: demo.commands
errors:
  - name: demo.commands.Rejected
    summary: The request was rejected.
  - name: demo.commands.Unavailable
    summary: The provider was unavailable.
commands:
  - name: demo.commands.Work
    input: [{name: value, type: String}]
    outcomes:
      - name: accepted
        when: value == 'yes'
        emits: [demo.commands.Happened]
        payload:
          demo.commands.Happened: {value: input.value}
      - name: rejected
        error: demo.commands.Rejected
  - name: demo.commands.Send
    input: [{name: value, type: String}]
    outcomes:
      - name: sent
        emits: [demo.commands.Logged]
      - name: failed
        external: the provider is unavailable
        error: demo.commands.Unavailable
events:
  - name: demo.commands.Happened
    fields: [{name: value, type: String}]
  - name: demo.commands.Logged
    fields: [{name: value, type: String}]
";

struct Model {
    raw: RawSpecFile,
    specification: Specification,
    ir: EssIr,
}

fn model(text: &str) -> Model {
    let raw = RawSpecFile::parse(text).unwrap();
    let specification = Specification::assemble(vec![
        (Source::new("commands.yaml"), raw.clone()),
        (Source::new("selection.yaml"), RawSpecFile::parse("types:\n  - name: demo.CliInput\n    kind: struct\n    fields: [{name: value, type: Integer}]\n").unwrap()),
    ]).unwrap();
    let ir =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    Model {
        raw,
        specification,
        ir,
    }
}

fn changed(old: &str, new: &str) -> Model {
    assert_eq!(MODEL.matches(old).count(), 1, "{old}");
    model(&MODEL.replacen(old, new, 1))
}

fn command(model: &Model, name: &str) -> Value {
    serde_json::to_value(
        model
            .ir
            .commands()
            .values()
            .find(|item| item.name.to_string() == name)
            .unwrap(),
    )
    .unwrap()
}

fn bound(model: &Model) -> CompiledBinding {
    let binding = Binding::from_yaml(
        r"
format: ess-cli/1
binary: commands
about: An independent selected typed value
globals: {config: config, state: state-dir, output: output}
callables:
  observe:
    target: {kind: local, owner: demo.cli, action: observe}
    input: demo.CliInput
    result: Integer
commands:
  - path: [observe]
    callable: observe
    about: Observe the selected integer
    arguments:
      - {field: value, source: {kind: option, long: value}}
",
    )
    .unwrap();
    compile(&model.ir, &binding).unwrap()
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("command model witnesses cannot acquire external sources")
    }
}

#[derive(Default)]
struct Recorder(Vec<Value>);
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        assert!(invocation.context.config.is_none());
        assert!(invocation.context.state_dir.is_none());
        self.0.push(json!({
            "callable": invocation.callable,
            "target": invocation.target,
            "input": invocation.input,
        }));
        HandlerReply::Success(invocation.input["value"].clone())
    }
}

fn execute(binding: &CompiledBinding) -> (ProcessOutput, Vec<Value>) {
    let mut handler = Recorder::default();
    let output = runtime::run(
        binding.plan(),
        ["commands", "observe", "--value", "17", "--output=json"]
            .into_iter()
            .map(Into::into)
            .collect(),
        &mut NoSources,
        &mut handler,
        None,
    );
    assert_eq!(output.exit_code, 0);
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&output.stdout).unwrap(),
        json!({"ok": true, "result": 17})
    );
    assert_eq!(handler.0.len(), 1);
    assert_eq!(handler.0[0]["callable"], "observe");
    assert_eq!(handler.0[0]["input"], json!({"value": 17}));
    assert_eq!(handler.0[0]["target"]["owner"], "demo.cli");
    (output, handler.0)
}

fn unchanged(control: &Model, changed: &Model) {
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    let a = bound(control);
    let b = bound(changed);
    assert_eq!(a.to_canonical_json(), b.to_canonical_json());
    assert_eq!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    assert_eq!(execute(&a), execute(&b));
}

#[test]
fn command_identity_and_domain_are_not_the_selected_local_target() {
    let control = model(MODEL);
    let renamed = model(&MODEL.replace("demo.commands.Work", "demo.commands.Perform"));
    assert_ne!(control.raw.commands[0].name, renamed.raw.commands[0].name);
    assert_ne!(
        command(&control, "demo.commands.Work")["name"],
        command(&renamed, "demo.commands.Perform")["name"]
    );
    assert_ne!(
        control.specification.commands(),
        renamed.specification.commands()
    );
    unchanged(&control, &renamed);
    let moved = model(&MODEL.replace("demo.commands", "demo.other"));
    assert_ne!(
        command(&control, "demo.commands.Work")["domain"],
        command(&moved, "demo.other.Work")["domain"]
    );
    unchanged(&control, &moved);
}

#[test]
fn unselected_command_input_fields_have_no_cli_effect() {
    let control = model(MODEL);
    let changed = changed(
        "  - name: demo.commands.Work\n    input: [{name: value, type: String}]",
        "  - name: demo.commands.Work\n    input: [{name: value, type: String}, {name: extra, type: Optional<Integer>}]",
    );
    assert_ne!(control.raw.commands[0].input, changed.raw.commands[0].input);
    assert_ne!(
        control.specification.commands(),
        changed.specification.commands()
    );
    assert_ne!(
        command(&control, "demo.commands.Work")["input"],
        command(&changed, "demo.commands.Work")["input"]
    );
    unchanged(&control, &changed);
}

#[test]
fn command_naming_and_references_are_application_metadata() {
    let control = model(MODEL);
    for (field, addition) in [
        (
            "naming",
            "    naming: {wire: perform, display: Perform work, summary: Application command}\n",
        ),
        ("refs", "    refs: [local:work-contract]\n"),
    ] {
        let changed = changed(
            "  - name: demo.commands.Work\n",
            &format!("  - name: demo.commands.Work\n{addition}"),
        );
        assert_ne!(
            command(&control, "demo.commands.Work")[field],
            command(&changed, "demo.commands.Work")[field]
        );
        assert_ne!(
            control.specification.commands(),
            changed.specification.commands()
        );
        unchanged(&control, &changed);
    }
}

#[test]
fn outcome_identity_summary_and_references_have_no_cli_effect() {
    let control = model(MODEL);
    for (field, old, new) in [
        (
            "name",
            "      - name: accepted\n",
            "      - name: performed\n",
        ),
        (
            "summary",
            "      - name: accepted\n",
            "      - name: accepted\n        summary: Accepted by the application\n",
        ),
        (
            "refs",
            "      - name: accepted\n",
            "      - name: accepted\n        refs: [local:accepted-contract]\n",
        ),
    ] {
        let changed = changed(old, new);
        assert_ne!(
            command(&control, "demo.commands.Work")["outcomes"][0][field],
            command(&changed, "demo.commands.Work")["outcomes"][0][field]
        );
        assert_ne!(
            serde_json::to_value(&control.raw.commands[0].outcomes).unwrap(),
            serde_json::to_value(&changed.raw.commands[0].outcomes).unwrap()
        );
        assert_ne!(
            control.specification.commands(),
            changed.specification.commands()
        );
        unchanged(&control, &changed);
    }
}

#[test]
fn outcome_input_conditions_and_external_causes_do_not_execute_the_application() {
    let control = model(MODEL);
    for (name, old, new) in [
        (
            "demo.commands.Work",
            "when: value == 'yes'",
            "when: value != 'yes'",
        ),
        (
            "demo.commands.Send",
            "external: the provider is unavailable",
            "external: the provider rejects the request",
        ),
    ] {
        let changed = changed(old, new);
        assert_ne!(
            command(&control, name)["outcomes"],
            command(&changed, name)["outcomes"]
        );
        assert_ne!(
            control.specification.commands(),
            changed.specification.commands()
        );
        unchanged(&control, &changed);
    }
    let changed = changed(
        "        when: value == 'yes'\n",
        "        external: the operator accepts the request\n",
    );
    let a = command(&control, "demo.commands.Work");
    let b = command(&changed, "demo.commands.Work");
    assert_eq!(a["outcomes"][0]["condition"]["kind"], "when");
    assert_eq!(b["outcomes"][0]["condition"]["kind"], "external");
    assert_ne!(
        a["outcomes"][0]["test_strategy"],
        b["outcomes"][0]["test_strategy"]
    );
    unchanged(&control, &changed);
    let conditional = model(
        &MODEL
            .replacen(
                "      - name: sent\n",
                "      - name: sent\n        when: value == 'yes'\n",
                1,
            )
            .replace("        external: the provider is unavailable\n", ""),
    );
    let a = command(&control, "demo.commands.Send");
    let b = command(&conditional, "demo.commands.Send");
    assert_eq!(a["outcomes"][0]["condition"]["kind"], "otherwise");
    assert_eq!(b["outcomes"][0]["condition"]["kind"], "when");
    assert_ne!(
        a["outcomes"][0]["test_strategy"],
        b["outcomes"][0]["test_strategy"]
    );
    unchanged(&control, &conditional);
}

#[test]
fn outcome_emission_and_payload_sources_have_no_cli_effect() {
    let control = model(MODEL);
    let emitted = changed(
        "emits: [demo.commands.Happened]",
        "emits: [demo.commands.Happened, demo.commands.Logged]",
    );
    assert_ne!(
        command(&control, "demo.commands.Work")["outcomes"][0]["emits"],
        command(&emitted, "demo.commands.Work")["outcomes"][0]["emits"]
    );
    unchanged(&control, &emitted);
    let literal = changed("{value: input.value}", "{value: '\"fixed\"'}");
    assert_ne!(
        command(&control, "demo.commands.Work")["outcomes"][0]["payload"],
        command(&literal, "demo.commands.Work")["outcomes"][0]["payload"]
    );
    let a = command(&control, "demo.commands.Work");
    let b = command(&literal, "demo.commands.Work");
    assert_eq!(
        a["outcomes"][0]["payload"][0]["fields"][0]["value"]["kind"],
        "input_field"
    );
    assert_eq!(
        a["outcomes"][0]["payload"][0]["fields"][0]["value"]["field"],
        "value"
    );
    assert_eq!(
        b["outcomes"][0]["payload"][0]["fields"][0]["value"],
        json!({"kind":"literal", "value":"\"fixed\""})
    );
    assert_ne!(
        control.specification.commands(),
        literal.specification.commands()
    );
    unchanged(&control, &literal);
}

#[test]
fn outcome_named_error_is_application_semantics() {
    let control = model(MODEL);
    let changed = changed(
        "        error: demo.commands.Rejected",
        "        error: demo.commands.Unavailable",
    );
    assert_ne!(
        command(&control, "demo.commands.Work")["outcomes"][1]["error"],
        command(&changed, "demo.commands.Work")["outcomes"][1]["error"]
    );
    assert_ne!(
        control.specification.commands(),
        changed.specification.commands()
    );
    unchanged(&control, &changed);
}

#[test]
fn unselected_event_fields_naming_and_identity_have_no_cli_effect() {
    let control = model(MODEL);
    for changed in [
        changed("  - name: demo.commands.Happened\n    fields: [{name: value, type: String}]", "  - name: demo.commands.Happened\n    fields: [{name: value, type: String}, {name: count, type: Integer}]"),
        changed("  - name: demo.commands.Happened\n", "  - name: demo.commands.Happened\n    naming: {wire: occurred, display: Occurred, summary: Application fact}\n"),
        model(&MODEL.replace("demo.commands.Happened", "demo.commands.Occurred")),
    ] {
        assert_ne!(serde_json::to_value(&control.raw.events).unwrap(), serde_json::to_value(&changed.raw.events).unwrap());
        assert_ne!(control.specification.events(), changed.specification.events());
        assert_ne!(control.ir.events(), changed.ir.events());
        unchanged(&control, &changed);
    }
}

#[test]
fn unselected_error_fields_summary_and_identity_have_no_cli_effect() {
    let control = model(MODEL);
    for changed in [
        changed(
            "    summary: The request was rejected.\n",
            "    summary: The request was rejected.\n    fields: [{name: reason, type: String}]\n",
        ),
        changed(
            "summary: The request was rejected.",
            "summary: The application rejected the request.",
        ),
        model(&MODEL.replace("demo.commands.Rejected", "demo.commands.Denied")),
    ] {
        assert_ne!(
            serde_json::to_value(&control.raw.errors).unwrap(),
            serde_json::to_value(&changed.raw.errors).unwrap()
        );
        assert_ne!(
            control.specification.errors(),
            changed.specification.errors()
        );
        assert_ne!(control.ir.errors(), changed.ir.errors());
        unchanged(&control, &changed);
    }
}
