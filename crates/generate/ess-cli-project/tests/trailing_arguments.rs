//! beyond10x/ess#466: `source: {kind: trailing}` binds one required `List<String>` field to every
//! argv word after the first `--`, verbatim, with no option parsing and no JSON decoding.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProcessOutput, ProtectedSource, Sources,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};
use std::ffi::OsString;

const MODEL: &str = "format: ess/1
system: demo
version: v1
types:
  - name: demo.LaunchInput
    kind: struct
    fields:
      - {name: connection, type: String}
      - {name: args, type: 'List<String>'}
  - name: demo.ExecInput
    kind: struct
    fields:
      - {name: target, type: 'Optional<String>'}
      - {name: args, type: 'List<String>'}
  - name: demo.Launched
    kind: struct
    fields:
      - {name: status, type: String}
";

const BINDING: &str = "format: ess-cli/1
binary: demo
about: Launch a pinned program
globals: {config: config, state: state-dir, output: output}
callables:
  launch:
    target: {kind: local, owner: demo.cli, action: launch}
    input: demo.LaunchInput
    result: demo.Launched
  exec:
    target: {kind: local, owner: demo.cli, action: exec}
    input: demo.ExecInput
    result: demo.Launched
commands:
  - path: [launch]
    callable: launch
    about: Launch with the arguments after --
    arguments:
      - {field: connection, source: {kind: option, long: connection}}
      - {field: args, source: {kind: trailing}}
  - path: [exec]
    callable: exec
    about: Execute in an optional target
    arguments:
      - {field: args, source: {kind: trailing}}
      - {field: target, source: {kind: positional, index: 1}}
";

fn compiled() -> CompiledBinding {
    let specification = Specification::assemble(vec![(
        Source::new("model.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap();
    compile(&ir, &Binding::from_yaml(BINDING).unwrap()).unwrap()
}

/// Records every input it is handed.
#[derive(Default)]
struct Recorder(Vec<Value>);
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.0.push(invocation.input.clone());
        HandlerReply::Success(json!({"status":"launched"}))
    }
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        unreachable!("no command here reads a protected or document source")
    }
}

fn run(args: Vec<OsString>, handler: &mut Recorder) -> ProcessOutput {
    runtime::run(compiled().plan(), args, &mut NoSources, handler, None)
}

fn argv(words: &[&str]) -> Vec<OsString> {
    std::iter::once("demo")
        .chain(words.iter().copied())
        .map(OsString::from)
        .collect()
}

fn handled(words: &[&str]) -> (ProcessOutput, Value) {
    let mut handler = Recorder::default();
    let output = run(argv(words), &mut handler);
    assert_eq!(output.exit_code, 0, "{words:?}: {output:?}");
    assert_eq!(handler.0.len(), 1, "{words:?}");
    (output, handler.0.remove(0))
}

fn parse_failure(args: Vec<OsString>) -> ProcessOutput {
    let mut handler = Recorder::default();
    let output = run(args, &mut handler);
    assert_eq!(output.exit_code, 2, "{output:?}");
    assert_eq!(output.stderr, "cli_parse\n", "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(handler.0.len(), 0);
    output
}

#[test]
fn trailing_collects_everything_after_double_dash_verbatim() {
    let (output, input) = handled(&[
        "launch",
        "--connection",
        "c",
        "--",
        "--flag",
        "-x",
        "v",
        "--",
        "--output",
        "json",
    ]);
    assert_eq!(
        input,
        json!({"connection":"c","args":["--flag","-x","v","--","--output","json"]})
    );
    // `--output json` after `--` is a value of the launched program, not the adapter's selector.
    assert_eq!(output.stdout, "{\"status\":\"launched\"}\n");
    // Words that look like JSON stay text.
    let (_, input) = handled(&[
        "launch",
        "--connection",
        "c",
        "--",
        "[1]",
        "{}",
        "7",
        "",
        " ",
    ]);
    assert_eq!(input["args"], json!(["[1]", "{}", "7", "", " "]));
    // The adapter's own selector before `--` still applies.
    let (output, input) = handled(&["launch", "--output", "json", "--connection", "c", "--", "x"]);
    assert_eq!(input["args"], json!(["x"]));
    assert_eq!(
        output.stdout,
        "{\"ok\":true,\"result\":{\"status\":\"launched\"}}\n"
    );
}

#[test]
fn trailing_absent_or_bare_double_dash_is_empty_list() {
    for words in [
        &["launch", "--connection", "c"][..],
        &["launch", "--connection", "c", "--"][..],
    ] {
        let (_, input) = handled(words);
        assert_eq!(input, json!({"connection":"c","args":[]}), "{words:?}");
    }
}

#[test]
fn trailing_words_before_double_dash_are_parse_failures() {
    for words in [
        &["launch", "--connection", "c", "extra-canary"][..],
        &["launch", "extra-canary", "--connection", "c", "--", "x"][..],
        &["exec", "p", "extra-canary"][..],
    ] {
        let output = parse_failure(argv(words));
        assert!(!output.stderr.contains("extra"), "{words:?}: {output:?}");
    }
    // A required option still applies when the list is present.
    parse_failure(argv(&["launch", "--", "--connection", "c"]));
}

#[test]
fn trailing_after_optional_positional() {
    let (_, input) = handled(&["exec", "p", "--", "a", "b"]);
    assert_eq!(input, json!({"target":"p","args":["a","b"]}));
    let (_, input) = handled(&["exec", "--", "p"]);
    assert_eq!(input, json!({"args":["p"]}));
    let (_, input) = handled(&["exec", "p"]);
    assert_eq!(input, json!({"target":"p","args":[]}));
    let (_, input) = handled(&["exec"]);
    assert_eq!(input, json!({"args":[]}));
}

#[test]
fn trailing_help_shows_double_dash_list() {
    for (command, usage) in [
        ("launch", "[-- <field:args>...]"),
        ("exec", "[field:target] [-- <field:args>...]"),
    ] {
        let mut handler = Recorder::default();
        let output = run(argv(&[command, "--help"]), &mut handler);
        assert_eq!(output.exit_code, 0, "{output:?}");
        assert!(
            output.stdout.contains(usage),
            "{command}: {}",
            output.stdout
        );
    }
    runtime::command(compiled().plan()).debug_assert();
}

#[cfg(unix)]
#[test]
fn trailing_non_utf8_value_is_parse_failure() {
    use std::os::unix::ffi::OsStringExt;
    let mut args = argv(&["launch", "--connection", "c", "--", "ok"]);
    args.push(OsString::from_vec(b"canary-\xff".to_vec()));
    let output = parse_failure(args);
    assert!(!output.stderr.contains("canary"), "{output:?}");
    // The JSON selector before `--` governs the parse failure, as for any other one.
    let mut args = argv(&["--output=json", "launch", "--connection", "c", "--"]);
    args.push(OsString::from_vec(b"canary-\xff".to_vec()));
    let mut handler = Recorder::default();
    let output = run(args, &mut handler);
    assert_eq!(output.exit_code, 2, "{output:?}");
    assert_eq!(
        serde_json::from_str::<Value>(&output.stderr).unwrap(),
        json!({"ok":false,"error":{"code":"cli_parse","data":{}}})
    );
}
