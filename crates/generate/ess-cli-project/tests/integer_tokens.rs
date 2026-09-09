//! Preserve integer token semantics without rewriting JSON strings or fractions.

use ess_cli_contract::{Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};

fn binding(shape: &str) -> CompiledBinding {
    let source = format!(
        "format: ess/1\nsystem: integers\nversion: v1\ntypes:\n  - name: integers.Input\n    kind: struct\n    fields: [{{name: value, type: '{shape}'}}]\n"
    );
    let specification = Specification::assemble([(
        Source::new("integers.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    let binding = Binding::from_yaml("format: ess-cli/1\nbinary: integers\nabout: Observe exact values\nglobals: {config: config, state: state-dir, output: output}\ncallables:\n  inspect:\n    target: {kind: local, owner: integers.cli, action: inspect}\n    input: integers.Input\n    result: String\ncommands:\n  - path: [inspect]\n    callable: inspect\n    about: Inspect a typed value\n    arguments:\n      - {field: value, source: {kind: option, long: value}}\n").unwrap();
    ess_cli_contract::compile(&ir, &binding).unwrap()
}

#[derive(Default)]
struct Recorder(Vec<Value>);
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.0.push(invocation.input["value"].clone());
        HandlerReply::Success(json!("accepted"))
    }
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("integer tokens need no credential input")
    }
}

fn run(binding: &CompiledBinding, text: &str, recorder: &mut Recorder) -> runtime::ProcessOutput {
    runtime::run(
        binding.plan(),
        // Keep malformed number spellings such as `--0` in the value decoder,
        // rather than letting Clap interpret them as additional option names.
        vec![
            "integers".into(),
            "inspect".into(),
            format!("--value={text}").into(),
            "--output=json".into(),
        ],
        &mut NoSources,
        recorder,
        None,
    )
}

#[test]
fn integer_zero_normalization_preserves_nested_values_quoted_keys_and_text() {
    for (shape, text, expected) in [
        ("Integer", " \t-0\n", json!(0)),
        (
            "List<Map<String, Integer>>",
            r#"[{"-0":-0,"quote\"-0":-0}, {"negative":-1,"zero":0}]"#,
            json!([{"-0":0,"quote\"-0":0},{"negative":-1,"zero":0}]),
        ),
        (
            "Map<String, String>",
            r#"{"-0":"-0","quote\"-0":"escaped\\\" -0","unicode":"é -0"}"#,
            json!({"-0":"-0","quote\"-0":"escaped\\\" -0","unicode":"é -0"}),
        ),
        ("String", "-0", json!("-0")),
    ] {
        let mut recorder = Recorder::default();
        let result = run(&binding(shape), text, &mut recorder);
        assert_eq!(result.exit_code, 0, "{shape}: {result:?}");
        assert!(result.stderr.is_empty());
        assert_eq!(recorder.0, [expected]);
        assert_eq!(
            serde_json::from_str::<Value>(&result.stdout).unwrap(),
            json!({"ok":true,"result":"accepted"})
        );
    }
}

#[test]
fn integer_arguments_still_refuse_fraction_exponent_and_malformed_number_spellings() {
    for shape in ["Integer", "List<Integer>", "Map<String, Integer>"] {
        let binding = binding(shape);
        for token in [
            "-0.0", "-0e0", "-0E+0", "0.0", "-00", "00", "+0", "--0", "- 0", "-01",
        ] {
            let text = match shape {
                "List<Integer>" => format!("[{token}]"),
                "Map<String, Integer>" => format!("{{\"value\":{token}}}"),
                _ => token.to_owned(),
            };
            let mut recorder = Recorder::default();
            let output = run(&binding, &text, &mut recorder);
            assert_eq!(output.exit_code, 2, "{shape} {token}: {output:?}");
            assert!(output.stdout.is_empty());
            assert!(recorder.0.is_empty());
            assert_eq!(
                serde_json::from_str::<Value>(&output.stderr).unwrap(),
                json!({"ok":false,"error":{"code":"cli_input","data":{}}}),
                "{shape} {token}"
            );
        }
    }
}
