//! Finite model-change witnesses for binding admission, emission and direct process execution.
//! These tests launch no process and claim no generated-package execution.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    AcquireError, Handler, HandlerReply, Invocation, ProcessOutput, ProtectedSource, Sources,
};
use ess_compiler::EssIr;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, ffi::OsString};

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    ess_compiler::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap()
}

fn specification(field_type: &str, extra: &str) -> String {
    format!(
        "format: ess/1\nsystem: demo\nversion: v1\ntypes:\n  - name: demo.Input\n    kind: struct\n    fields:\n      - name: value\n        type: '{field_type}'\n{extra}"
    )
}

fn binding() -> Binding {
    Binding::from_yaml(
        r"
format: ess-cli/1
binary: demo
about: Consumer model witness
globals: {config: config, state: state-dir, output: output}
callables:
  use:
    target: {kind: local, owner: demo.cli, action: use}
    input: demo.Input
    result: String
commands:
  - path: [use]
    callable: use
    about: Use one typed value
    arguments:
      - {field: value, source: {kind: option, long: value}}
",
    )
    .unwrap()
}

fn admitted(text: &str) -> (EssIr, CompiledBinding, BTreeMap<String, String>) {
    let ir = model(text);
    let compiled = compile(&ir, &binding()).unwrap();
    let artifacts = ess_cli_project::project(&compiled);
    assert_eq!(artifacts["binding.json"], compiled.to_canonical_json());
    (ir, compiled, artifacts)
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("ordinary model-value witnesses must not acquire protected sources")
    }
}
#[derive(Default)]
struct Recording {
    inputs: Vec<Value>,
}
impl Handler for Recording {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        assert_eq!(invocation.callable, "use");
        assert!(invocation.context.config.is_none());
        assert!(invocation.context.state_dir.is_none());
        self.inputs.push(invocation.input.clone());
        HandlerReply::Success(json!("handled"))
    }
}
fn execute(compiled: &CompiledBinding, value: Option<&str>) -> (ProcessOutput, Vec<Value>) {
    let mut args: Vec<OsString> = ["demo", "use", "--output=json"]
        .into_iter()
        .map(Into::into)
        .collect();
    if let Some(value) = value {
        args.extend(["--value", value].into_iter().map(Into::into));
    }
    let mut handler = Recording::default();
    let output =
        ess_cli_project::runtime::run(compiled.plan(), args, &mut NoSources, &mut handler, None);
    (output, handler.inputs)
}
fn accepted(compiled: &CompiledBinding, value: Option<&str>, expected: Value) {
    let (output, inputs) = execute(compiled, value);
    assert_eq!(output.exit_code, 0);
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&output.stdout).unwrap(),
        json!({"ok":true,"result":"handled"})
    );
    assert_eq!(inputs, [expected]);
}
fn refused(compiled: &CompiledBinding, value: &str) {
    let (output, inputs) = execute(compiled, Some(value));
    assert_eq!(output.exit_code, 2);
    assert!(output.stdout.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&output.stderr).unwrap(),
        json!({"ok":false,"error":{"code":"cli_input","data":{}}})
    );
    assert!(inputs.is_empty());
}

#[test]
fn scalar_changes_reach_resolution_emission_and_typed_dispatch() {
    let (text_ir, text, text_files) = admitted(&specification("String", ""));
    accepted(&text, Some("7"), json!({"value":"7"}));
    for (kind, native, expected) in [("Integer", "7", json!(7)), ("Boolean", "true", json!(true))] {
        let (ir, compiled, files) = admitted(&specification(kind, ""));
        assert_ne!(ir.to_canonical_json(), text_ir.to_canonical_json());
        assert_ne!(compiled.to_canonical_json(), text.to_canonical_json());
        assert_ne!(files["binding.json"], text_files["binding.json"]);
        accepted(&compiled, Some(native), json!({"value":expected}));
        refused(&compiled, "\"wrong-native-type\"");
    }
}

#[test]
fn selected_wire_field_name_changes_payload_identity() {
    let original = specification("String", "");
    let renamed = original.replace(
        "        type: 'String'",
        "        type: 'String'\n        wire: wire_value",
    );
    let (base_ir, base, base_files) = admitted(&original);
    let (changed_ir, changed, changed_files) = admitted(&renamed);
    assert_ne!(base_ir.to_canonical_json(), changed_ir.to_canonical_json());
    assert_ne!(base.to_canonical_json(), changed.to_canonical_json());
    assert_ne!(base_files["binding.json"], changed_files["binding.json"]);
    accepted(&base, Some("x"), json!({"value":"x"}));
    accepted(&changed, Some("x"), json!({"wire_value":"x"}));
}

#[test]
fn display_and_summary_controls_have_no_cli_effect() {
    let original = specification("String", "");
    let (base_ir, base, base_files) = admitted(&original);
    for changed in [
        original.replace("    kind: struct", "    kind: struct\n    naming: {display: Input display, summary: Input summary}"),
        original.replace("        type: 'String'", "        type: 'String'\n        display: Value display\n        summary: Value summary"),
    ] {
        let (ir, compiled, files) = admitted(&changed);
        assert_ne!(ir.to_canonical_json(), base_ir.to_canonical_json());
        assert_eq!(compiled.to_canonical_json(), base.to_canonical_json());
        assert_eq!(files, base_files);
        let (before, before_inputs) = execute(&base, Some("x"));
        let (after, after_inputs) = execute(&compiled, Some("x"));
        assert_eq!((before.exit_code, before.stdout, before.stderr, before_inputs), (after.exit_code, after.stdout, after.stderr, after_inputs));
        accepted(&compiled, Some("x"), json!({"value":"x"}));
    }
}

#[test]
fn enum_membership_changes_admitted_values_in_all_three_boundaries() {
    let original = specification(
        "demo.Mode",
        "  - name: demo.Mode\n    kind: enum\n    variants: [Safe]\n",
    );
    let changed = original.replace("[Safe]", "[Fast]");
    let (base_ir, base, base_files) = admitted(&original);
    let (changed_ir, next, changed_files) = admitted(&changed);
    assert_ne!(base_ir.to_canonical_json(), changed_ir.to_canonical_json());
    assert_ne!(base.to_canonical_json(), next.to_canonical_json());
    assert_ne!(base_files["binding.json"], changed_files["binding.json"]);
    accepted(&base, Some("Safe"), json!({"value":"Safe"}));
    refused(&base, "Fast");
    accepted(&next, Some("Fast"), json!({"value":"Fast"}));
    refused(&next, "Safe");
}

#[test]
fn optional_list_and_string_map_shapes_change_validation() {
    let (_, scalar, scalar_files) = admitted(&specification("Integer", ""));
    accepted(&scalar, Some("1"), json!({"value":1}));
    for (kind, native, expected) in [
        ("Optional<Integer>", "null", Value::Null),
        ("List<Integer>", "[1,2]", json!([1, 2])),
        ("Map<String, Integer>", "{\"a\":1}", json!({"a":1})),
    ] {
        let (_, compiled, files) = admitted(&specification(kind, ""));
        assert_ne!(compiled.to_canonical_json(), scalar.to_canonical_json());
        assert_ne!(files["binding.json"], scalar_files["binding.json"]);
        accepted(&compiled, Some(native), json!({"value":expected}));
        refused(&compiled, "\"wrong-native-type\"");
    }
    let (_, optional, _) = admitted(&specification("Optional<Integer>", ""));
    accepted(&optional, None, json!({}));
    let (required, inputs) = execute(&scalar, None);
    assert_eq!(required.exit_code, 2);
    assert!(inputs.is_empty());
}

#[test]
fn newtype_underlying_shape_changes_admission_and_dispatch() {
    let original = specification(
        "demo.Value",
        "  - name: demo.Value\n    kind: newtype\n    of: String\n",
    );
    let changed = original.replace("of: String", "of: Integer");
    let (base_ir, base, base_files) = admitted(&original);
    let (changed_ir, next, changed_files) = admitted(&changed);
    assert_ne!(base_ir.to_canonical_json(), changed_ir.to_canonical_json());
    assert_ne!(base.to_canonical_json(), next.to_canonical_json());
    assert_ne!(base_files["binding.json"], changed_files["binding.json"]);
    accepted(&base, Some("42"), json!({"value":"42"}));
    accepted(&next, Some("42"), json!({"value":42}));
    refused(&next, "\"text\"");
}

#[test]
fn unsupported_primitives_are_named_admission_refusals_before_emission_or_execution() {
    let (_, admitted_control, files) = admitted(&specification("String", ""));
    assert!(!files.is_empty());
    accepted(
        &admitted_control,
        Some("control"),
        json!({"value":"control"}),
    );
    for primitive in [
        "Uuid",
        "Timestamp",
        "Duration",
        "Bytes",
        "Decimal",
        "Binary64",
    ] {
        // Binary64 is a v2 model primitive; binding admission is a separate boundary.
        let source = specification(primitive, "").replace("format: ess/1", "format: ess/2");
        let ir = model(&source);
        let error = compile(&ir, &binding()).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("unsupported CLI primitive `{primitive}`")
        );
        // No CompiledBinding exists, so neither downstream pipeline is entered.
    }
}

#[test]
fn constrained_newtype_and_struct_are_named_admission_refusals() {
    let original = specification(
        "demo.Value",
        "  - name: demo.Value\n    kind: newtype\n    of: String\n",
    );
    let (_, compiled, files) = admitted(&original);
    assert!(!files.is_empty());
    accepted(&compiled, Some("control"), json!({"value":"control"}));
    for text in [
        original.replace(
            "    of: String",
            "    of: String\n    invariants: [value != \"\"]",
        ),
        specification("String", "").replace(
            "    kind: struct",
            "    kind: struct\n    invariants: [value != \"\"]",
        ),
    ] {
        let ir = model(&text);
        let error = compile(&ir, &binding()).unwrap_err().to_string();
        assert!(
            error == "CLI type `demo.Value` has unsupported invariants or union semantics"
                || error == "CLI type `demo.Input` has unsupported invariants or union semantics"
        );
    }
}
