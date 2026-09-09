//! Finite source/type/predicate witnesses for the three authored CLI pipelines.
//! A recording local handler observes values; it never evaluates business predicates.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
};
use ess_compiler::{ir::ResolvedBody, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
    types::{RawTypeBody, TypeBody, TypeRef},
    Invariant,
};
use ess_primitives::{
    facts::{FactPath, FactValue},
    predicate::{CompareOp, Operand, Predicate},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const TYPES: &str = r"
format: ess/2
system: sample
version: v1
types:
  - name: sample.CliInput
    kind: struct
    fields:
      - {name: value, type: sample.Value}
  - name: sample.Value
    kind: newtype
    of: String
  - name: sample.Reply
    kind: struct
    fields:
      - {name: answer, type: String}
  - name: sample.Failure
    kind: struct
    fields:
      - {name: detail, type: String}
";
const EXPRESSIONS: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/adversary_expression.yaml");

struct Model {
    raw: RawSpecFile,
    specification: Specification,
    ir: EssIr,
}

fn model(text: &str) -> Model {
    let raw = RawSpecFile::parse(text).unwrap();
    let specification =
        Specification::assemble([(Source::new("types.yaml"), raw.clone())]).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("types.yaml", text);
    let ir = ess_compiler::compile(&specification, &sources).unwrap();
    Model {
        raw,
        specification,
        ir,
    }
}

fn replace_one(text: &str, from: &str, to: &str) -> String {
    assert_eq!(
        text.matches(from).count(),
        1,
        "finite source control: {from}"
    );
    text.replacen(from, to, 1)
}

fn binding_text(namespace: &str) -> String {
    format!(
        "format: ess-cli/1\nbinary: type-observer\nabout: Observe one local value\nglobals: {{config: config, state: state-dir, output: output}}\ncallables:\n  observe:\n    target: {{kind: local, owner: {namespace}.cli, action: observe}}\n    input: {namespace}.CliInput\n    result: Integer\n    errors: {{failed: {namespace}.Failure}}\ncommands:\n  - path: [observe]\n    callable: observe\n    about: Observe without executing model commands\n    arguments:\n      - {{field: value, source: {{kind: option, long: value}}}}\n"
    )
}

fn bound(model: &Model, text: &str) -> CompiledBinding {
    compile(&model.ir, &Binding::from_yaml(text).unwrap()).unwrap()
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("type witnesses must not acquire an external source")
    }
}

struct Recorder {
    calls: Vec<Value>,
    reply: Option<HandlerReply>,
}
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        assert_eq!(invocation.context.output, runtime::OutputMode::Json);
        self.calls.push(json!({
            "callable": invocation.callable, "target": invocation.target, "input": invocation.input,
            "config": invocation.context.config, "state_dir": invocation.context.state_dir
        }));
        self.reply.take().expect("exactly one handler call")
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Observation {
    exit: i32,
    stdout: String,
    stderr: String,
    calls: Vec<Value>,
}

fn execute(binding: &CompiledBinding, value: &str, reply: HandlerReply) -> Observation {
    let mut handler = Recorder {
        calls: Vec::new(),
        reply: Some(reply),
    };
    let output = runtime::run(
        binding.plan(),
        [
            "type-observer",
            "observe",
            "--value",
            value,
            "--config",
            "selected.toml",
            "--state-dir",
            "selected-state",
            "--output=json",
        ]
        .into_iter()
        .map(Into::into)
        .collect(),
        &mut NoSources,
        &mut handler,
        None,
    );
    Observation {
        exit: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
        calls: handler.calls,
    }
}

fn success(
    binding: &CompiledBinding,
    namespace: &str,
    value: &str,
    input: Value,
    result: Value,
) -> Observation {
    let expected_output = json!({"ok":true,"result":result});
    let observed = execute(binding, value, HandlerReply::Success(result));
    assert_eq!(observed.exit, 0, "{observed:?}");
    assert!(observed.stderr.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&observed.stdout).unwrap(),
        expected_output
    );
    let mut expected_call = json!({
        "callable":"observe", "target":{"kind":"local","owner":format!("{namespace}.cli"),"action":"observe"},
        "config":"selected.toml","state_dir":"selected-state"
    });
    expected_call
        .as_object_mut()
        .unwrap()
        .insert("input".to_owned(), input);
    assert_eq!(observed.calls, vec![expected_call]);
    observed
}

fn no_effect(before: &Model, after: &Model, namespace: &str, input: Value) {
    assert_ne!(before.specification, after.specification);
    assert_ne!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    let a = bound(before, &binding_text(namespace));
    let b = bound(after, &binding_text(namespace));
    assert_eq!(a.to_canonical_json(), b.to_canonical_json());
    assert_eq!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    assert_eq!(
        success(&a, namespace, "17", input.clone(), json!(17)),
        success(&b, namespace, "17", input, json!(17))
    );
}

#[derive(Default, Debug)]
struct Trace {
    stages: Vec<&'static str>,
    artifacts: BTreeMap<String, String>,
    observation: Option<Observation>,
    assembly_codes: Vec<String>,
}

// This finite source probe calls each real stage in order. `?` exits at the actual rejection;
// the trace and terminal observations are retained for both rejected and admitted controls.
fn pipeline(text: &str, trace: &mut Trace) -> Result<(), String> {
    trace.stages.push("reader");
    let raw = RawSpecFile::parse(text).map_err(|e| e.to_string())?;
    trace.stages.push("assembly");
    let specification =
        Specification::assemble([(Source::new("types.yaml"), raw)]).map_err(|e| {
            trace.assembly_codes = e
                .as_slice()
                .iter()
                .map(|item| format!("{:?}", item.code))
                .collect();
            e.to_string()
        })?;
    trace.stages.push("compiler");
    let mut sources = SourceMap::new();
    sources.insert("types.yaml", text);
    let ir = ess_compiler::compile(&specification, &sources).map_err(|e| e.to_string())?;
    trace.stages.push("binding");
    let binding = compile(&ir, &Binding::from_yaml(&binding_text("sample")).unwrap())
        .map_err(|e| e.to_string())?;
    trace.stages.push("project");
    trace.artifacts = ess_cli_project::project(&binding);
    assert_eq!(trace.artifacts["binding.json"], binding.to_canonical_json());
    trace.stages.push("runtime");
    trace.observation = Some(success(
        &binding,
        "sample",
        "17",
        json!({"value":"17"}),
        json!(17),
    ));
    Ok(())
}

fn source_refused(control: &str, rejected: &str, stages: &[&str], needle: &str) -> Trace {
    assert_ne!(control, rejected);
    let mut accepted = Trace::default();
    pipeline(control, &mut accepted).unwrap();
    assert_eq!(
        accepted.stages,
        ["reader", "assembly", "compiler", "binding", "project", "runtime"]
    );
    assert!(!accepted.artifacts.is_empty());
    assert_eq!(accepted.observation.as_ref().unwrap().calls.len(), 1);
    let mut refused = Trace::default();
    let message = pipeline(rejected, &mut refused).unwrap_err();
    assert!(
        message.contains(needle),
        "expected {needle:?}, got {message}"
    );
    assert_eq!(refused.stages, stages);
    assert!(
        refused.artifacts.is_empty(),
        "refused source emitted artifacts"
    );
    assert!(
        refused.observation.is_none(),
        "refused source entered runtime"
    );
    refused
}

#[test]
fn declared_type_identity_and_reference_chains_resolve_without_local_shape_change() {
    let before = model(TYPES);
    let renamed = TYPES.replace("sample.Value", "sample.TextValue");
    let after = model(&renamed);
    assert_eq!(before.raw.types[1].name.to_string(), "sample.Value");
    assert_eq!(after.raw.types[1].name.to_string(), "sample.TextValue");
    let before_name = "sample.Value".parse().unwrap();
    let after_name = "sample.TextValue".parse().unwrap();
    assert!(before
        .specification
        .system()
        .types
        .get(&before_name)
        .is_some());
    assert!(before
        .specification
        .system()
        .types
        .get(&after_name)
        .is_none());
    assert!(after
        .specification
        .system()
        .types
        .get(&before_name)
        .is_none());
    assert_eq!(
        after
            .specification
            .system()
            .types
            .get(&after_name)
            .unwrap()
            .name,
        after_name
    );
    assert!(!after.ir.types().contains_key(&before_name));
    assert_eq!(after.ir.types()[&after_name].name, after_name);
    let RawTypeBody::Struct { fields, .. } = &after.raw.types[0].body else {
        panic!("struct")
    };
    assert_eq!(fields[0].type_ref, TypeRef::Named(after_name.clone()));
    let ResolvedBody::Struct { fields, .. } =
        &after.ir.types()[&"sample.CliInput".parse().unwrap()].body
    else {
        panic!("struct")
    };
    assert_eq!(fields[0].type_ref.declared().unwrap().name(), &after_name);
    no_effect(&before, &after, "sample", json!({"value":"17"}));

    let chain = replace_one(
        TYPES,
        "    of: String",
        "    of: sample.Inner\n  - {name: sample.Inner, kind: newtype, of: String}",
    );
    let after = model(&chain);
    assert_eq!(after.raw.types.len(), before.raw.types.len() + 1);
    assert_eq!(
        after.specification.system().types.len(),
        before.specification.system().types.len() + 1
    );
    assert_eq!(after.ir.types().len(), before.ir.types().len() + 1);
    let TypeBody::Newtype { of, .. } = &after
        .specification
        .system()
        .types
        .get(&before_name)
        .unwrap()
        .body
    else {
        panic!("newtype")
    };
    assert_eq!(of.to_string(), "sample.Inner");
    let ResolvedBody::Newtype { of, .. } = &after.ir.types()[&before_name].body else {
        panic!("newtype")
    };
    assert_eq!(of.declared().unwrap().to_string(), "sample.Inner");
    no_effect(&before, &after, "sample", json!({"value":"17"}));
}

#[test]
fn semantic_field_rename_refuses_stale_binding_then_preserves_explicit_wire_payload() {
    let before_text = replace_one(
        TYPES,
        "{name: value, type: sample.Value}",
        "{name: value, type: sample.Value, wire: value}",
    );
    let after_text = replace_one(&before_text, "name: value,", "name: quantity,");
    let before = model(&before_text);
    let after = model(&after_text);
    for (witness, expected) in [(&before, "value"), (&after, "quantity")] {
        let RawTypeBody::Struct { fields, .. } = &witness.raw.types[0].body else {
            panic!("raw struct")
        };
        assert_eq!(fields[0].name, expected);
        assert_eq!(fields[0].naming.wire.as_deref(), Some("value"));
        let TypeBody::Struct { fields, .. } = &witness
            .specification
            .system()
            .types
            .get(&"sample.CliInput".parse().unwrap())
            .unwrap()
            .body
        else {
            panic!("assembled struct")
        };
        assert_eq!(fields[0].name, expected);
        let ResolvedBody::Struct { fields, .. } =
            &witness.ir.types()[&"sample.CliInput".parse().unwrap()].body
        else {
            panic!("resolved struct")
        };
        assert_eq!(fields[0].name, expected);
        assert_eq!(fields[0].naming.wire.as_deref(), Some("value"));
    }
    source_refused(
        &before_text,
        &after_text,
        &["reader", "assembly", "compiler", "binding"],
        "unresolved argument field `value`",
    );
    let a = bound(&before, &binding_text("sample"));
    let b = bound(
        &after,
        &binding_text("sample").replace("field: value", "field: quantity"),
    );
    assert_ne!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    assert_eq!(a.to_canonical_json(), b.to_canonical_json());
    assert_eq!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    assert_eq!(
        success(&a, "sample", "17", json!({"value":"17"}), json!(17)),
        success(&b, "sample", "17", json!({"value":"17"}), json!(17))
    );
}

#[test]
fn selected_newtype_and_struct_shapes_change_native_payloads() {
    let before = model(TYPES);
    let after = model(&replace_one(TYPES, "    of: String", "    of: Boolean"));
    let name = "sample.Value".parse().unwrap();
    assert_ne!(
        before.specification.system().types.get(&name).unwrap().body,
        after.specification.system().types.get(&name).unwrap().body
    );
    assert_ne!(before.ir.types()[&name].body, after.ir.types()[&name].body);
    let a = bound(&before, &binding_text("sample"));
    let b = bound(&after, &binding_text("sample"));
    assert_ne!(a.to_canonical_json(), b.to_canonical_json());
    assert_ne!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    success(&a, "sample", "true", json!({"value":"true"}), json!(17));
    success(&b, "sample", "true", json!({"value":true}), json!(17));
    let wrong = execute(&b, "17", HandlerReply::Success(json!(17)));
    assert_eq!(wrong.exit, 2);
    assert!(wrong.calls.is_empty());
    assert!(wrong.stdout.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&wrong.stderr).unwrap()["error"]["code"],
        "cli_input"
    );

    let nested = replace_one(
        TYPES,
        "    kind: newtype\n    of: String",
        "    kind: struct\n    fields: [{name: inner, type: Integer}]",
    );
    let after = model(&nested);
    assert!(matches!(
        after.raw.types[1].body,
        RawTypeBody::Struct { .. }
    ));
    assert!(matches!(
        after.specification.system().types.get(&name).unwrap().body,
        TypeBody::Struct { .. }
    ));
    assert!(matches!(
        after.ir.types()[&name].body,
        ResolvedBody::Struct { .. }
    ));
    let b = bound(&after, &binding_text("sample"));
    assert_ne!(a.to_canonical_json(), b.to_canonical_json());
    assert_ne!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    success(
        &b,
        "sample",
        r#"{"inner":17}"#,
        json!({"value":{"inner":17}}),
        json!(17),
    );
}

#[test]
fn selected_enum_wire_variants_change_admission_and_runtime_values() {
    let enum_text = replace_one(
        TYPES,
        "    kind: newtype\n    of: String",
        "    kind: enum\n    variants: [Safe]",
    );
    let revised = replace_one(&enum_text, "variants: [Safe]", "variants: [Fast]");
    let before = model(&enum_text);
    let after = model(&revised);
    assert!(
        matches!(&before.raw.types[1].body, RawTypeBody::Enum { variants } if variants == &["Safe"])
    );
    assert!(
        matches!(&after.raw.types[1].body, RawTypeBody::Enum { variants } if variants == &["Fast"])
    );
    let a = bound(&before, &binding_text("sample"));
    let b = bound(&after, &binding_text("sample"));
    assert_ne!(a.to_canonical_json(), b.to_canonical_json());
    assert_ne!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    success(&a, "sample", "Safe", json!({"value":"Safe"}), json!(17));
    success(&b, "sample", "Fast", json!({"value":"Fast"}), json!(17));
    for (binding, wrong) in [(&a, "Fast"), (&b, "Safe")] {
        let rejected = execute(binding, wrong, HandlerReply::Success(json!(17)));
        assert_eq!(rejected.exit, 2);
        assert!(rejected.stdout.is_empty());
        assert!(rejected.calls.is_empty());
        assert_eq!(
            serde_json::from_str::<Value>(&rejected.stderr).unwrap()["error"]["code"],
            "cli_input"
        );
    }
}

#[test]
fn unused_union_tag_and_variants_change_real_types_without_cli_effect() {
    let before_text = format!(
        "{TYPES}  - name: sample.Choice\n    kind: union\n    tag: kind\n    variants: {{text: String}}\n"
    );
    let after_text = replace_one(
        &before_text,
        "tag: kind\n    variants: {text: String}",
        "tag: category\n    variants: {count: Integer, text: String}",
    );
    let before = model(&before_text);
    let after = model(&after_text);
    let name = "sample.Choice".parse().unwrap();
    for (witness, tag, count) in [(&before, "kind", 1), (&after, "category", 2)] {
        let RawTypeBody::Union {
            tag: actual,
            variants,
        } = &witness.raw.types[4].body
        else {
            panic!("raw union")
        };
        assert_eq!(actual, tag);
        assert_eq!(variants.len(), count);
        assert_eq!(variants["text"].to_string(), "String");
        let TypeBody::Union {
            tag: actual,
            variants,
        } = &witness
            .specification
            .system()
            .types
            .get(&name)
            .unwrap()
            .body
        else {
            panic!("assembled union")
        };
        assert_eq!(actual, tag);
        assert_eq!(variants.len(), count);
        let ResolvedBody::Union {
            tag: actual,
            variants,
        } = &witness.ir.types()[&name].body
        else {
            panic!("resolved union")
        };
        assert_eq!(actual, tag);
        assert_eq!(variants.len(), count);
    }
    no_effect(&before, &after, "sample", json!({"value":"17"}));
}

#[test]
fn selected_union_invariants_recursion_and_map_keys_refuse_at_binding_only() {
    for (rejected, message) in [
        (
            replace_one(
                TYPES,
                "    kind: newtype\n    of: String",
                "    kind: union\n    tag: kind\n    variants: {text: String}",
            ),
            "CLI type `sample.Value` has unsupported invariants or union semantics",
        ),
        (
            replace_one(
                TYPES,
                "    of: String",
                "    of: String\n    invariants: [defined(value)]",
            ),
            "CLI type `sample.Value` has unsupported invariants or union semantics",
        ),
        (
            replace_one(TYPES, "    of: String", "    of: Optional<sample.Value>"),
            "recursive or overly deep CLI type `sample.Value` is unsupported",
        ),
        (
            replace_one(TYPES, "    of: String", "    of: 'Map<Integer, String>'"),
            "CLI maps require String keys",
        ),
    ] {
        source_refused(
            TYPES,
            &rejected,
            &["reader", "assembly", "compiler", "binding"],
            message,
        );
    }
    let constrained_input = replace_one(
        TYPES,
        "      - {name: value, type: sample.Value}",
        "      - {name: value, type: sample.Value}\n    invariants: [defined(value)]",
    );
    source_refused(
        TYPES,
        &constrained_input,
        &["reader", "assembly", "compiler", "binding"],
        "CLI type `sample.CliInput` has unsupported invariants or union semantics",
    );
}

#[test]
fn unsupported_primitive_contracts_refuse_by_exact_type_at_binding() {
    for primitive in [
        "Decimal",
        "Binary64",
        "Timestamp",
        "Duration",
        "Uuid",
        "Bytes",
    ] {
        let rejected = replace_one(TYPES, "    of: String", &format!("    of: {primitive}"));
        let resolved = model(&rejected);
        let RawTypeBody::Newtype { of, .. } = &resolved.raw.types[1].body else {
            panic!("newtype")
        };
        assert_eq!(of.to_string(), primitive);
        source_refused(
            TYPES,
            &rejected,
            &["reader", "assembly", "compiler", "binding"],
            &format!("unsupported CLI primitive `{primitive}`"),
        );
    }
}

#[test]
fn declared_result_and_error_fields_check_actual_handler_values() {
    let before = model(TYPES);
    let result_text = replace_one(
        TYPES,
        "{name: answer, type: String}",
        "{name: answer, type: Integer}",
    );
    let after = model(&result_text);
    let binding = binding_text("sample").replace("result: Integer", "result: sample.Reply");
    let a = bound(&before, &binding);
    let b = bound(&after, &binding);
    assert_ne!(a.to_canonical_json(), b.to_canonical_json());
    assert_ne!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    success(
        &a,
        "sample",
        "17",
        json!({"value":"17"}),
        json!({"answer":"17"}),
    );
    success(
        &b,
        "sample",
        "17",
        json!({"value":"17"}),
        json!({"answer":17}),
    );
    let wrong = execute(&b, "17", HandlerReply::Success(json!({"answer":"17"})));
    assert_eq!(wrong.exit, 1);
    assert_eq!(wrong.calls.len(), 1);
    assert!(wrong.stdout.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&wrong.stderr).unwrap()["error"]["code"],
        "cli_result"
    );

    let error_text = replace_one(
        TYPES,
        "{name: detail, type: String}",
        "{name: detail, type: Boolean}",
    );
    let after = model(&error_text);
    let b = bound(&after, &binding);
    assert_ne!(a.to_canonical_json(), b.to_canonical_json());
    assert_ne!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    for (compiled, data) in [
        (&a, json!({"detail":"failed"})),
        (&b, json!({"detail":true})),
    ] {
        let observed = execute(
            compiled,
            "17",
            HandlerReply::Error {
                code: "failed".into(),
                data: data.clone(),
            },
        );
        assert_eq!(observed.exit, 1);
        assert_eq!(observed.calls.len(), 1);
        assert!(observed.stdout.is_empty());
        assert_eq!(
            serde_json::from_str::<Value>(&observed.stderr).unwrap(),
            json!({"ok":false,"error":{"code":"failed","data":data}})
        );
    }
    let wrong = execute(
        &b,
        "17",
        HandlerReply::Error {
            code: "failed".into(),
            data: json!({"detail":"failed"}),
        },
    );
    assert_eq!(wrong.exit, 1);
    assert_eq!(wrong.calls.len(), 1);
    assert!(wrong.stdout.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&wrong.stderr).unwrap()["error"]["code"],
        "cli_error"
    );
}

#[test]
fn conversion_endpoints_and_reason_are_resolved_without_local_cli_effect() {
    let before_text = format!(
        "{TYPES}\nconversions:\n  - {{from: sample.Value, to: String, because: Initial conversion}}\n"
    );
    let after_text = replace_one(
        &before_text,
        "{from: sample.Value, to: String, because: Initial conversion}",
        "{from: String, to: sample.Value, because: Reversed conversion}",
    );
    let before = model(&before_text);
    let after = model(&after_text);
    for (witness, from, to, reason) in [
        (&before, "sample.Value", "String", "Initial conversion"),
        (&after, "String", "sample.Value", "Reversed conversion"),
    ] {
        let raw = &witness.raw.conversions[0];
        assert_eq!(
            (
                raw.from.to_string(),
                raw.to.to_string(),
                raw.because.as_str()
            ),
            (from.to_owned(), to.to_owned(), reason)
        );
        assert_eq!(witness.specification.conversions().len(), 1);
        assert_eq!(
            witness.specification.conversions().iter().next().unwrap(),
            raw
        );
        assert!(witness
            .specification
            .conversions()
            .permits(&raw.from, &raw.to));
        let resolved = &witness.ir.conversions()[0];
        assert_eq!(resolved.from.to_string(), from);
        assert_eq!(resolved.to.to_string(), to);
        assert_eq!(resolved.because, reason);
    }
    no_effect(&before, &after, "sample", json!({"value":"17"}));
}

#[test]
fn named_type_and_field_naming_strings_and_null_forms_are_observed() {
    let named = replace_one(
        TYPES,
        "  - name: sample.Value\n",
        "  - name: sample.Value\n    naming: {wire: semantic-value, display: Semantic value, summary: A semantic value}\n",
    );
    let fields = replace_one(
        TYPES,
        "{name: value, type: sample.Value}",
        "{name: value, type: sample.Value, display: Supplied value, summary: A supplied value}",
    );
    let before = model(TYPES);
    let after = model(&named);
    assert_eq!(
        after.raw.types[1].naming.wire.as_deref(),
        Some("semantic-value")
    );
    assert_eq!(
        after.raw.types[1].naming.display.as_deref(),
        Some("Semantic value")
    );
    assert_eq!(
        after.raw.types[1].naming.summary.as_deref(),
        Some("A semantic value")
    );
    assert_eq!(
        after.ir.types()[&"sample.Value".parse().unwrap()].naming,
        after.raw.types[1].naming
    );
    no_effect(&before, &after, "sample", json!({"value":"17"}));
    let after = model(&fields);
    let RawTypeBody::Struct { fields, .. } = &after.raw.types[0].body else {
        panic!("struct")
    };
    assert_eq!(fields[0].naming.display.as_deref(), Some("Supplied value"));
    assert_eq!(
        fields[0].naming.summary.as_deref(),
        Some("A supplied value")
    );
    no_effect(&before, &after, "sample", json!({"value":"17"}));

    let explicit = replace_one(
        TYPES,
        "  - name: sample.Value\n",
        "  - name: sample.Value\n    naming: {wire: null, display: null, summary: null}\n",
    );
    let explicit = replace_one(
        &explicit,
        "{name: value, type: sample.Value}",
        "{name: value, type: sample.Value, wire: null, display: null, summary: null}",
    );
    assert_ne!(TYPES, explicit);
    let after = model(&explicit);
    assert!(after.raw.types[1].naming.is_empty());
    let RawTypeBody::Struct { fields, .. } = &after.raw.types[0].body else {
        panic!("struct")
    };
    assert!(fields[0].naming.is_empty());
    assert_eq!(before.specification, after.specification);
    assert_eq!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    let a = bound(&before, &binding_text("sample"));
    let b = bound(&after, &binding_text("sample"));
    assert_eq!(a.to_canonical_json(), b.to_canonical_json());
    assert_eq!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    assert_eq!(
        success(&a, "sample", "17", json!({"value":"17"}), json!(17)),
        success(&b, "sample", "17", json!({"value":"17"}), json!(17))
    );
}

#[test]
fn named_type_branches_refuse_unknown_and_missing_owned_members_at_reader() {
    for body in [
        "kind: newtype\n    of: String",
        "kind: struct\n    fields: [{name: item, type: String}]",
        "kind: enum\n    variants: [Ready]",
        "kind: union\n    tag: kind\n    variants: {text: String}",
    ] {
        let control = format!("{TYPES}  - name: sample.Probe\n    {body}\n");
        let unknown = format!("{control}    unexpected: true\n");
        source_refused(
            &control,
            &unknown,
            &["reader"],
            "unknown field `unexpected`",
        );
        let missing_name = replace_one(&control, "  - name: sample.Probe\n    ", "  - ");
        source_refused(&control, &missing_name, &["reader"], "missing field `name`");
        let kind_line = body.lines().next().unwrap();
        let missing_kind = format!(
            "{TYPES}  - name: sample.Probe\n    {}\n",
            body.strip_prefix(&format!("{kind_line}\n    ")).unwrap()
        );
        source_refused(&control, &missing_kind, &["reader"], "missing field `kind`");
    }
    for (body, member) in [
        ("kind: newtype\n    of: String", "of"),
        (
            "kind: struct\n    fields: [{name: item, type: String}]",
            "fields",
        ),
        ("kind: enum\n    variants: [Ready]", "variants"),
        (
            "kind: union\n    tag: kind\n    variants: {text: String}",
            "tag",
        ),
        (
            "kind: union\n    tag: kind\n    variants: {text: String}",
            "variants",
        ),
    ] {
        let control = format!("{TYPES}  - name: sample.Probe\n    {body}\n");
        let line = body
            .lines()
            .find(|line| line.trim_start().starts_with(&format!("{member}:")))
            .unwrap();
        let missing_body = body
            .lines()
            .filter(|candidate| *candidate != line)
            .collect::<Vec<_>>()
            .join("\n");
        let missing = format!("{TYPES}  - name: sample.Probe\n    {missing_body}\n");
        source_refused(
            &control,
            &missing,
            &["reader"],
            &format!("missing field `{member}`"),
        );
    }
}

#[test]
fn type_and_field_wire_shapes_refuse_at_the_reader_with_full_controls() {
    for (old, new, needle) in [
        (
            "{name: value, type: sample.Value}",
            "{name: value, type: sample.Value, unknown: true}",
            "unknown field `unknown`",
        ),
        (
            "{name: value, type: sample.Value}",
            "{type: sample.Value}",
            "missing field `name`",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: value}",
            "missing field `type`",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: '1bad', type: sample.Value}",
            "must start with a letter",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: 'bad-name', type: sample.Value}",
            "a field name has to survive",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: value, type: []}",
            "expected a string",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: [], type: sample.Value}",
            "expected a string",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: value, type: sample.Value, display: []}",
            "expected a string",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: value, type: sample.Value, summary: []}",
            "expected a string",
        ),
        (
            "{name: value, type: sample.Value}",
            "{name: value, type: sample.Value, wire: []}",
            "expected a string",
        ),
        ("    of: String", "    of: []", "expected a string"),
        (
            "    of: String",
            "    of: String\n    naming: {unknown: true}",
            "unknown field `unknown`",
        ),
        (
            "    of: String",
            "    of: String\n    naming: []",
            "expected struct Naming",
        ),
    ] {
        source_refused(TYPES, &replace_one(TYPES, old, new), &["reader"], needle);
    }
}

#[test]
fn conversion_wire_members_refuse_missing_unknown_and_wrong_shapes() {
    let control = format!(
        "{TYPES}\nconversions:\n  - {{from: sample.Value, to: String, because: Explicit reason}}\n"
    );
    for (declaration, needle) in [
        (
            "{from: sample.Value, to: String, because: Explicit reason, unknown: true}",
            "unknown field `unknown`",
        ),
        (
            "{to: String, because: Explicit reason}",
            "missing field `from`",
        ),
        (
            "{from: sample.Value, because: Explicit reason}",
            "missing field `to`",
        ),
        (
            "{from: sample.Value, to: String}",
            "missing field `because`",
        ),
        (
            "{from: [], to: String, because: Explicit reason}",
            "expected a string",
        ),
        (
            "{from: sample.Value, to: [], because: Explicit reason}",
            "expected a string",
        ),
        (
            "{from: sample.Value, to: String, because: []}",
            "expected a string",
        ),
    ] {
        let rejected = replace_one(
            &control,
            "{from: sample.Value, to: String, because: Explicit reason}",
            declaration,
        );
        source_refused(&control, &rejected, &["reader"], needle);
    }
}

#[test]
fn unresolved_and_duplicate_type_declarations_refuse_during_assembly() {
    let unresolved = replace_one(
        TYPES,
        "{name: value, type: sample.Value}",
        "{name: value, type: sample.Absent}",
    );
    let trace = source_refused(TYPES, &unresolved, &["reader", "assembly"], "sample.Absent");
    assert!(trace
        .assembly_codes
        .iter()
        .any(|code| code == "UndeclaredReference"));
    let duplicate = format!("{TYPES}  - {{name: sample.Value, kind: newtype, of: String}}\n");
    let trace = source_refused(TYPES, &duplicate, &["reader", "assembly"], "sample.Value");
    assert!(trace
        .assembly_codes
        .iter()
        .any(|code| code == "DuplicateDeclaration"));
}

fn expression_source(expression: &str) -> String {
    let local = replace_one(
        EXPRESSIONS,
        "types:\n",
        "types:\n  - name: review.data.CliInput\n    kind: struct\n    fields: [{name: value, type: Integer}]\n  - name: review.data.Failure\n    kind: struct\n    fields: [{name: detail, type: String}]\n",
    );
    replace_one(
        &local,
        "        when: amount > 0",
        &format!("        when: {expression}"),
    )
}

fn guard(model: &Model) -> &Predicate {
    let outcome = &model.ir.commands()[&"review.data.Update".parse().unwrap()].outcomes[0];
    let ess_compiler::ir::ResolvedCondition::When { predicate } = &outcome.condition else {
        panic!("predicate guard")
    };
    predicate
}

fn raw_guard(model: &Model) -> &Predicate {
    model
        .raw
        .commands
        .iter()
        .find(|command| command.name.to_string() == "review.data.Update")
        .unwrap()
        .outcomes[0]
        .when
        .as_ref()
        .unwrap()
}

fn predicate_control(expression: &str, expected: &Predicate) -> Model {
    let witness = model(&expression_source(expression));
    assert_eq!(raw_guard(&witness), expected);
    let command = &witness.specification.commands()[&"review.data.Update".parse().unwrap()];
    let ess_domain::command::OutcomeCondition::When(actual) = &command.outcomes[0].condition else {
        panic!("assembled predicate")
    };
    assert_eq!(actual, expected);
    assert_eq!(guard(&witness), expected);
    witness
}

#[test]
fn predicate_comparisons_change_operators_operands_and_fact_paths_without_cli_effect() {
    let before = model(&expression_source("amount > 0"));
    for (operator, spelling) in [
        (CompareOp::Eq, "=="),
        (CompareOp::Ne, "!="),
        (CompareOp::Lt, "<"),
        (CompareOp::Le, "<="),
        (CompareOp::Ge, ">="),
    ] {
        let expected = Predicate::Compare {
            left: Operand::Fact(FactPath::new("amount").unwrap()),
            op: operator,
            right: Operand::Literal(FactValue::Number(0_i64.into())),
        };
        let after = predicate_control(&format!("amount {spelling} 0"), &expected);
        no_effect(&before, &after, "review.data", json!({"value":17}));
    }
    let expected = Predicate::Compare {
        left: Operand::Fact(FactPath::new("amount").unwrap()),
        op: CompareOp::Gt,
        right: Operand::Fact(FactPath::new("entry.amount").unwrap()),
    };
    let after = predicate_control("amount > entry.amount", &expected);
    let Predicate::Compare {
        left: Operand::Fact(left),
        right: Operand::Fact(right),
        ..
    } = guard(&after)
    else {
        panic!("two fact operands")
    };
    assert_eq!(left.segments(), &["amount"]);
    assert_eq!(right.segments(), &["entry", "amount"]);
    no_effect(&before, &after, "review.data", json!({"value":17}));
}

#[test]
fn predicate_boolean_text_and_membership_literals_change_without_cli_effect() {
    let before = model(&expression_source("amount > 0"));
    for (expression, expected) in [
        (
            "flag == true",
            Predicate::Compare {
                left: Operand::Fact(FactPath::new("flag").unwrap()),
                op: CompareOp::Eq,
                right: Operand::Literal(FactValue::Bool(true)),
            },
        ),
        (
            "phase == Ready",
            Predicate::Compare {
                left: Operand::Fact(FactPath::new("phase").unwrap()),
                op: CompareOp::Eq,
                right: Operand::Literal(FactValue::Text("Ready".into())),
            },
        ),
        (
            "{phase: {any_of: [Ready, Done]}}",
            Predicate::AnyOf {
                path: FactPath::new("phase").unwrap(),
                values: vec![
                    FactValue::Text("Ready".into()),
                    FactValue::Text("Done".into()),
                ],
            },
        ),
        (
            "{entry.phase: {none_of: [Done]}}",
            Predicate::NoneOf {
                path: FactPath::new("entry.phase").unwrap(),
                values: vec![FactValue::Text("Done".into())],
            },
        ),
    ] {
        let after = predicate_control(expression, &expected);
        no_effect(&before, &after, "review.data", json!({"value":17}));
    }
}

#[test]
fn predicate_boolean_composition_and_wire_forms_change_without_cli_effect() {
    let before = model(&expression_source("amount > 0"));
    let flag = Predicate::Truthy(FactPath::new("flag").unwrap());
    let defined = Predicate::Defined(FactPath::new("phase").unwrap());
    for (expression, expected) in [
        ("flag", flag.clone()),
        ("defined(phase)", defined.clone()),
        ("{not: flag}", Predicate::Not(Box::new(flag.clone()))),
        (
            "[flag, defined(phase)]",
            Predicate::All(vec![flag.clone(), defined.clone()]),
        ),
        (
            "{any: [flag, defined(phase)]}",
            Predicate::Any(vec![flag, defined]),
        ),
    ] {
        let after = predicate_control(expression, &expected);
        assert_eq!(guard(&after).to_node(), expected.to_node());
        no_effect(&before, &after, "review.data", json!({"value":17}));
    }
}

#[test]
fn quantified_predicates_change_kind_collection_binder_and_body_without_cli_effect() {
    let before_text = "{forall: {in: groups, as: group, that: {exists: {in: group, as: entry, that: entry.amount > 0}}}}";
    let after_text = "{exists: {in: entry.children, as: batch, that: {forall: {in: batch, as: item, that: item.amount >= 1}}}}";
    let before = model(&expression_source(before_text));
    let after = model(&expression_source(after_text));
    let Predicate::Forall(a) = guard(&before) else {
        panic!("forall")
    };
    let Predicate::Exists(b) = guard(&after) else {
        panic!("exists")
    };
    assert_eq!(a.over.segments(), &["groups"]);
    assert_eq!(b.over.segments(), &["entry", "children"]);
    assert_eq!(a.bind, "group");
    assert_eq!(b.bind, "batch");
    assert!(matches!(a.body, Predicate::Exists(_)));
    assert!(matches!(b.body, Predicate::Forall(_)));
    assert_ne!(a.body, b.body);
    assert_eq!(raw_guard(&before), guard(&before));
    assert_eq!(raw_guard(&after), guard(&after));
    no_effect(&before, &after, "review.data", json!({"value":17}));
}

#[test]
fn exact_and_binary_numeric_predicate_facts_remain_distinct_without_cli_effect() {
    let before = model(&expression_source("amount == 9007199254740992"));
    let after = model(&expression_source("amount == 9007199254740993"));
    let Predicate::Compare {
        right: Operand::Literal(FactValue::Number(a)),
        ..
    } = guard(&before)
    else {
        panic!("numeric literal")
    };
    let Predicate::Compare {
        right: Operand::Literal(FactValue::Number(b)),
        ..
    } = guard(&after)
    else {
        panic!("numeric literal")
    };
    assert_eq!(a.as_i64(), Some(9_007_199_254_740_992));
    assert_eq!(b.as_i64(), Some(9_007_199_254_740_993));
    assert_eq!(
        a.get().to_bits(),
        b.get().to_bits(),
        "the binary carrier rounds these together"
    );
    assert_ne!(a, b);
    assert_eq!(
        serde_json::to_value(b).unwrap(),
        json!(9_007_199_254_740_993_i64)
    );
    assert!(format!("{b:?}").contains("Exact { units: 9007199254740993, scale: 0, binary:"));
    no_effect(&before, &after, "review.data", json!({"value":17}));

    let fraction = model(&expression_source("amount == 1.5"));
    let large = model(&expression_source("amount == 1e40"));
    let Predicate::Compare {
        right: Operand::Literal(FactValue::Number(f)),
        ..
    } = guard(&fraction)
    else {
        panic!("fraction")
    };
    let Predicate::Compare {
        right: Operand::Literal(FactValue::Number(n)),
        ..
    } = guard(&large)
    else {
        panic!("binary magnitude")
    };
    assert_eq!(f.exact_text(), "1.5");
    assert_eq!(f.get().to_bits(), 1.5_f64.to_bits());
    assert!(!f.is_integral());
    assert_eq!(
        format!("{f:?}"),
        "Number(Exact { units: 15, scale: 1, binary: 1.5 })"
    );
    assert_eq!(n.get().to_bits(), 1e40_f64.to_bits());
    assert!(!n.is_integral());
    assert!(format!("{n:?}").starts_with("Number(Binary64("));
    assert_eq!(serde_json::to_value(n).unwrap(), json!(1e40));
    no_effect(&fraction, &large, "review.data", json!({"value":17}));
}

#[test]
fn invariant_wire_forms_change_real_unused_newtype_and_struct_predicates() {
    for (body, expressions) in [
        (
            "kind: newtype\n    of: Integer",
            [
                "'value > 0'",
                "true",
                "[value >= 0, value < 10]",
                "{any: [value == 0, value > 2]}",
            ],
        ),
        (
            "kind: struct\n    fields: [{name: value, type: Integer}]",
            [
                "'value > 0'",
                "false",
                "[value >= 0, value < 10]",
                "{all: [value >= 0, value < 10]}",
            ],
        ),
    ] {
        let initial = format!("{TYPES}  - name: sample.Constrained\n    {body}\n");
        let before = model(&initial);
        for expression in expressions {
            let text = format!("{initial}    invariants:\n      - {expression}\n");
            let after = model(&text);
            let raw = match &after.raw.types[4].body {
                RawTypeBody::Newtype { invariants, .. }
                | RawTypeBody::Struct { invariants, .. } => Invariant::from(invariants[0].clone()),
                _ => panic!("invariant owner"),
            };
            let assembled = match &after
                .specification
                .system()
                .types
                .get(&"sample.Constrained".parse().unwrap())
                .unwrap()
                .body
            {
                TypeBody::Newtype { invariants, .. } | TypeBody::Struct { invariants, .. } => {
                    &invariants[0]
                }
                _ => panic!("invariant owner"),
            };
            let resolved = match &after.ir.types()[&"sample.Constrained".parse().unwrap()].body {
                ResolvedBody::Newtype { invariants, .. }
                | ResolvedBody::Struct { invariants, .. } => &invariants[0],
                _ => panic!("invariant owner"),
            };
            assert_eq!(raw.predicate, assembled.predicate);
            assert_eq!(raw.predicate, resolved.predicate);
            if expression == "true" {
                assert_eq!(raw.predicate, Predicate::Always);
            } else if expression == "false" {
                assert_eq!(raw.predicate, Predicate::Never);
            }
            assert!(!matches!(raw.predicate, Predicate::Never) || expression == "false");
            no_effect(&before, &after, "sample", json!({"value":"17"}));
        }
    }
}

#[test]
fn every_named_type_branch_checks_its_owned_object_tag_name_and_naming_representation() {
    for body in [
        "kind: newtype\n    of: String",
        "kind: struct\n    fields: [{name: item, type: String}]",
        "kind: enum\n    variants: [Ready]",
        "kind: union\n    tag: kind\n    variants: {text: String}",
    ] {
        let control = format!("{TYPES}  - name: sample.Probe\n    {body}\n");
        let kind = body.lines().next().unwrap();
        for (from, to, message) in [
            (kind, "kind: missing-kind", "unknown variant"),
            (kind, "kind: []", "invalid type"),
            ("name: sample.Probe", "name: []", "expected a string"),
        ] {
            let rejected = if from == kind {
                format!(
                    "{TYPES}  - name: sample.Probe\n    {}\n",
                    replace_one(body, from, to)
                )
            } else {
                replace_one(&control, from, to)
            };
            source_refused(&control, &rejected, &["reader"], message);
        }
        source_refused(
            &control,
            &format!("{control}    naming: []\n"),
            &["reader"],
            "expected struct Naming",
        );
        source_refused(
            &control,
            &format!("{TYPES}  - false\n"),
            &["reader"],
            "invalid type",
        );
    }
}

#[test]
fn type_body_collections_and_type_reference_shapes_have_matched_reader_refusals() {
    for (body, from, to, message) in [
        (
            "kind: newtype\n    of: String",
            "of: String",
            "of: []",
            "expected a string",
        ),
        (
            "kind: newtype\n    of: String\n    invariants: []",
            "invariants: []",
            "invariants: false",
            "expected a sequence",
        ),
        (
            "kind: struct\n    fields: [{name: item, type: String}]",
            "fields: [{name: item, type: String}]",
            "fields: false",
            "expected a sequence",
        ),
        (
            "kind: struct\n    fields: [{name: item, type: String}]\n    invariants: []",
            "invariants: []",
            "invariants: false",
            "expected a sequence",
        ),
        (
            "kind: enum\n    variants: [Ready]",
            "variants: [Ready]",
            "variants: false",
            "expected a sequence",
        ),
        (
            "kind: union\n    tag: kind\n    variants: {text: String}",
            "tag: kind",
            "tag: []",
            "expected a string",
        ),
        (
            "kind: union\n    tag: kind\n    variants: {text: String}",
            "variants: {text: String}",
            "variants: false",
            "expected a map",
        ),
    ] {
        let control = format!("{TYPES}  - name: sample.Probe\n    {body}\n");
        let rejected = format!(
            "{TYPES}  - name: sample.Probe\n    {}\n",
            replace_one(body, from, to)
        );
        source_refused(&control, &rejected, &["reader"], message);
    }
    source_refused(
        TYPES,
        &replace_one(TYPES, "{name: value, type: sample.Value}", "false"),
        &["reader"],
        "invalid type",
    );
    let conversion = "{from: sample.Value, to: String, because: Explicit reason}";
    let control = format!("{TYPES}\nconversions:\n  - {conversion}\n");
    source_refused(
        &control,
        &replace_one(&control, conversion, "false"),
        &["reader"],
        "invalid type",
    );
}

#[test]
fn raw_boolean_predicate_alternatives_reach_compiled_views_and_preserve_local_cli() {
    let control = expression_source("amount > 0");
    let before = model(&control);
    for (source, expected) in [("true", Predicate::Always), ("false", Predicate::Never)] {
        let changed = replace_one(
            &control,
            "filter: amount >= 0",
            &format!("filter: {source}"),
        );
        let after = model(&changed);
        assert_eq!(after.raw.views[0].filter.as_ref(), Some(&expected));
        assert_eq!(
            after.specification.views()[&"review.data.Items".parse().unwrap()]
                .filter
                .as_ref(),
            Some(&expected)
        );
        assert_eq!(
            after.ir.views()[&"review.data.Items".parse().unwrap()]
                .filter
                .as_ref(),
            Some(&expected)
        );
        no_effect(&before, &after, "review.data", json!({"value":17}));
    }
}
