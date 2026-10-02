//! beyond10x/ess#274: a callable's `invalid_input` names one of its declared errors as the answer
//! for every invalid input — unparsable, empty or duplicate-keyed dynamic payloads, validator
//! rejection, and typed shape failures — and the answer never carries the input text.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, DynamicError, DynamicPhase, DynamicValidator, Handler, HandlerReply,
    Invocation, ProcessOutput, ProtectedSource, Sources,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};
use std::ffi::OsString;

const MODEL: &str = "format: ess/1
system: tool
version: v1
types:
  - name: tool.Dynamic
    kind: struct
    fields:
      - {name: operation, type: String}
      - {name: schema, type: String}
      - {name: input, type: String}
  - name: tool.Typed
    kind: struct
    fields:
      - {name: count, type: Integer}
  - name: tool.Result
    kind: struct
    fields:
      - {name: status, type: String}
  - name: tool.InvalidInput
    kind: struct
    fields:
      - {name: detail, type: 'Optional<String>'}
";

/// `{invalid}` is replaced by the callable-level key, or by nothing for the unchanged contract.
const BINDING: &str = "format: ess-cli/1
binary: tool
about: Invalid input witness
globals: {config: config, state: state-dir, output: output}
callables:
  invoke:
    target: {kind: dynamic, owner: tool.cli, operation_field: operation, schema_field: schema, payload_field: input}
    input: tool.Dynamic
    result: tool.Result
    errors: {invalid_input: tool.InvalidInput}
{invalid}  typed:
    target: {kind: local, owner: tool.cli, action: typed}
    input: tool.Typed
    result: tool.Result
    errors: {invalid_input: tool.InvalidInput}
{invalid}commands:
  - path: [invoke]
    callable: invoke
    about: Invoke an operation
    arguments:
      - {field: operation, source: {kind: option, long: operation}}
      - {field: schema, source: {kind: option, long: schema}}
      - {field: input, source: {kind: document, inline: input-json, file: input-file, stdin: input-stdin}}
  - path: [typed]
    callable: typed
    about: Typed input
    arguments:
      - {field: count, source: {kind: option, long: count}}
";

const DECLARED: &str = "    invalid_input: invalid_input\n";

fn compiled(invalid: &str) -> CompiledBinding {
    let specification = Specification::assemble(vec![(
        Source::new("model.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    compile(
        &ir,
        &Binding::from_yaml(&BINDING.replace("{invalid}", invalid)).unwrap(),
    )
    .unwrap()
}

#[derive(Default)]
struct Calls(usize);
impl Handler for Calls {
    fn call(&mut self, _: &Invocation<'_>) -> HandlerReply {
        self.0 += 1;
        HandlerReply::Success(json!({"status":"handled"}))
    }
}

struct Stdin(&'static str);
impl Sources for Stdin {
    fn acquire(&mut self, source: ProtectedSource) -> Result<String, AcquireError> {
        assert_eq!(source, ProtectedSource::DocumentStdin);
        Ok(self.0.to_owned())
    }
}

/// Records every value it is asked to validate; accepts only `{"id":7}`.
#[derive(Default)]
struct Native(Vec<Value>);
impl DynamicValidator for Native {
    fn validate(
        &mut self,
        _: &Invocation<'_>,
        phase: DynamicPhase,
        value: &Value,
    ) -> Result<(), DynamicError> {
        if matches!(phase, DynamicPhase::Input) {
            self.0.push(value.clone());
        }
        if !matches!(phase, DynamicPhase::Input) || value == &json!({"id":7}) {
            Ok(())
        } else {
            Err(DynamicError::InvalidValue)
        }
    }
}

const CANARY: &str = "input-canary";

fn invoke(
    plan: &CompiledBinding,
    payload: Option<&str>,
    stdin: &'static str,
    validator: &mut Native,
    handler: &mut Calls,
) -> ProcessOutput {
    let mut args = vec![
        "tool",
        "invoke",
        "--operation",
        "lookup",
        "--schema",
        "lookup/v1",
        "--output=json",
    ];
    match payload {
        Some(text) => args.extend(["--input-json", text]),
        None => args.push("--input-stdin"),
    }
    runtime::run(
        plan.plan(),
        args.into_iter().map(OsString::from).collect(),
        &mut Stdin(stdin),
        handler,
        Some(validator),
    )
}

fn answered(output: &ProcessOutput, code: &str) {
    assert_eq!(output.exit_code, 2, "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        serde_json::from_str::<Value>(&output.stderr).unwrap(),
        json!({"ok":false,"error":{"code":code,"data":{}}}),
    );
    assert!(!output.stderr.contains(CANARY), "{output:?}");
}

#[test]
fn unparsable_empty_duplicate_and_rejected_dynamic_input_answer_the_declared_error() {
    let plan = compiled(DECLARED);
    for (payload, stdin, validated) in [
        (Some("not json input-canary"), "", false),
        (Some(""), "", false),
        (None, "", false),
        (None, "  \n", false),
        (Some(r#"{"id":"input-canary","id":7}"#), "", false),
        (Some(r#"{"id":"input-canary"}"#), "", true),
    ] {
        let (mut validator, mut handler) = (Native::default(), Calls::default());
        let output = invoke(&plan, payload, stdin, &mut validator, &mut handler);
        answered(&output, "invalid_input");
        assert_eq!(handler.0, 0, "{payload:?}");
        assert_eq!(validator.0.len(), usize::from(validated), "{payload:?}");
    }
    // A valid payload is unchanged: validated once, then handled.
    let (mut validator, mut handler) = (Native::default(), Calls::default());
    let output = invoke(&plan, Some(r#"{"id":7}"#), "", &mut validator, &mut handler);
    assert_eq!(output.exit_code, 0, "{output:?}");
    assert_eq!((validator.0.len(), handler.0), (1, 1));
}

#[test]
fn human_output_names_only_the_declared_code() {
    let plan = compiled(DECLARED);
    let output = runtime::run(
        plan.plan(),
        [
            "tool",
            "invoke",
            "--operation",
            "lookup",
            "--schema",
            "lookup/v1",
            "--input-json",
            "not json input-canary",
        ]
        .into_iter()
        .map(OsString::from)
        .collect(),
        &mut Stdin(""),
        &mut Calls::default(),
        Some(&mut Native::default()),
    );
    assert_eq!(output.exit_code, 2);
    assert_eq!(output.stderr, "invalid_input\n");
}

#[test]
fn typed_shape_failures_answer_the_declared_error() {
    let plan = compiled(DECLARED);
    for value in ["input-canary", "1.5", "9223372036854775808"] {
        let mut handler = Calls::default();
        let output = runtime::run(
            plan.plan(),
            ["tool", "typed", "--count", value, "--output=json"]
                .into_iter()
                .map(OsString::from)
                .collect(),
            &mut Stdin(""),
            &mut handler,
            None,
        );
        answered(&output, "invalid_input");
        assert_eq!(handler.0, 0);
    }
}

#[test]
fn without_the_key_invalid_input_keeps_the_closed_adapter_codes() {
    let plan = compiled("");
    for (payload, validated) in [
        ("not json input-canary", false),
        ("", false),
        (r#"{"id":"input-canary"}"#, true),
    ] {
        let (mut validator, mut handler) = (Native::default(), Calls::default());
        let output = invoke(&plan, Some(payload), "", &mut validator, &mut handler);
        answered(&output, "cli_dynamic_input");
        assert_eq!(validator.0.len(), usize::from(validated));
    }
    let output = runtime::run(
        plan.plan(),
        ["tool", "typed", "--count", "input-canary", "--output=json"]
            .into_iter()
            .map(OsString::from)
            .collect(),
        &mut Stdin(""),
        &mut Calls::default(),
        None,
    );
    answered(&output, "cli_input");
}

/// `serde_json` reads a duplicate key last-wins, so the validator would check `{"id":7}` while the
/// handler received text whose first `id` is something else. The payload is refused at parse.
#[test]
fn duplicate_keys_are_refused_before_the_validator_at_every_depth() {
    let plan = compiled("");
    for payload in [
        r#"{"id":"input-canary","id":7}"#,
        r#"{"id":7,"nested":{"a":1,"a":2}}"#,
        r#"[{"id":7},{"b":1,"b":1}]"#,
    ] {
        let (mut validator, mut handler) = (Native::default(), Calls::default());
        let output = invoke(&plan, Some(payload), "", &mut validator, &mut handler);
        answered(&output, "cli_dynamic_input");
        assert!(validator.0.is_empty(), "{payload}: {:?}", validator.0);
        assert_eq!(handler.0, 0);
    }
    // Equal keys in different objects are not duplicates: the payload reaches the validator.
    let (mut validator, mut handler) = (Native::default(), Calls::default());
    let payload = r#"[{"a":1},{"a":2,"b":{"a":3}}]"#;
    let output = invoke(&plan, Some(payload), "", &mut validator, &mut handler);
    answered(&output, "cli_dynamic_input");
    assert_eq!(
        validator.0,
        [serde_json::from_str::<Value>(payload).unwrap()]
    );
}

#[test]
fn the_reference_names_the_invalid_input_answer() {
    let readme = &ess_cli_project::project(&compiled(DECLARED))["README.md"];
    assert!(
        readme.contains("Invalid input answers `invalid_input`."),
        "{readme}"
    );
    let unchanged = &ess_cli_project::project(&compiled(""))["README.md"];
    assert!(!unchanged.contains("Invalid input answers"));
}
