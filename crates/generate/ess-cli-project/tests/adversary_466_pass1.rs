//! Adversary pass 1 for beyond10x/ess#466 (`source: {kind: trailing}`): the cases the unit's own
//! suite does not state — a required positional before the list, group paths and aliases, globals
//! and help words after `--`, `invalid_input`, the plan as the generated package reads it, and the
//! projected artifacts.

use ess_cli_contract::wire::Plan;
use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Context, Handler, HandlerReply, Invocation, OutputMode, ProcessOutput,
    ProtectedSource, Sources,
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
  - name: demo.RunInput
    kind: struct
    fields:
      - {name: target, type: String}
      - {name: argv, type: 'List<String>'}
  - name: demo.CountInput
    kind: struct
    fields:
      - {name: count, type: Integer}
      - {name: args, type: 'List<String>'}
  - name: demo.Launched
    kind: struct
    fields:
      - {name: status, type: String}
  - name: demo.Invalid
    kind: struct
    fields:
      - {name: detail, type: 'Optional<String>'}
";

const BINDING: &str = "format: ess-cli/1
binary: demo
about: Trailing adversary
globals: {config: config, state: state-dir, output: output}
callables:
  launch:
    target: {kind: local, owner: demo.cli, action: launch}
    input: demo.LaunchInput
    result: demo.Launched
  run:
    target: {kind: local, owner: demo.cli, action: run}
    input: demo.RunInput
    result: demo.Launched
  count:
    target: {kind: local, owner: demo.cli, action: count}
    input: demo.CountInput
    result: demo.Launched
    errors: {bad_input: demo.Invalid}
    invalid_input: bad_input
commands:
  - path: [tools, launch]
    aliases: [[tools, start], [go]]
    callable: launch
    about: Launch from a group
    arguments:
      - {field: args, source: {kind: trailing}}
      - {field: connection, source: {kind: option, long: connection}}
  - path: [run]
    callable: run
    about: A required positional before the list
    arguments:
      - {field: argv, source: {kind: trailing}}
      - {field: target, source: {kind: positional, index: 1}}
  - path: [count]
    callable: count
    about: A declared invalid-input answer beside the list
    arguments:
      - {field: count, source: {kind: option, long: count}}
      - {field: args, source: {kind: trailing}}
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

/// Records every input and process context it is handed.
#[derive(Default)]
struct Recorder(Vec<(Value, Context)>);
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.0
            .push((invocation.input.clone(), invocation.context.clone()));
        HandlerReply::Success(json!({"status":"launched"}))
    }
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        unreachable!("no command here reads a protected or document source")
    }
}

fn argv(words: &[&str]) -> Vec<OsString> {
    std::iter::once("demo")
        .chain(words.iter().copied())
        .map(OsString::from)
        .collect()
}

fn run_on(plan: &Plan, words: &[&str]) -> (ProcessOutput, Vec<(Value, Context)>) {
    let mut handler = Recorder::default();
    let output = runtime::run(plan, argv(words), &mut NoSources, &mut handler, None);
    (output, handler.0)
}

fn handled(words: &[&str]) -> (ProcessOutput, Value, Context) {
    let (output, mut calls) = run_on(compiled().plan(), words);
    assert_eq!(output.exit_code, 0, "{words:?}: {output:?}");
    assert_eq!(calls.len(), 1, "{words:?}");
    let (input, context) = calls.remove(0);
    (output, input, context)
}

fn refused(words: &[&str], stderr: &str) {
    let (output, calls) = run_on(compiled().plan(), words);
    assert_eq!(output.exit_code, 2, "{words:?}: {output:?}");
    assert_eq!(output.stderr, stderr, "{words:?}: {output:?}");
    assert!(output.stdout.is_empty(), "{words:?}: {output:?}");
    assert!(calls.is_empty(), "{words:?}");
}

fn refused_json(words: &[&str], code: &str) {
    let (output, calls) = run_on(compiled().plan(), words);
    assert_eq!(output.exit_code, 2, "{words:?}: {output:?}");
    assert_eq!(
        serde_json::from_str::<Value>(&output.stderr).unwrap(),
        json!({"ok":false,"error":{"code":code,"data":{}}}),
        "{words:?}: {output:?}"
    );
    assert!(!output.stderr.contains("canary"), "{words:?}: {output:?}");
    assert!(output.stdout.is_empty(), "{words:?}: {output:?}");
    assert!(calls.is_empty(), "{words:?}");
}

#[test]
fn adv466_required_positional_is_filled_only_before_double_dash() {
    let (_, input, _) = handled(&["run", "p", "--", "a", "b"]);
    assert_eq!(input, json!({"target":"p","argv":["a","b"]}));
    for words in [&["run", "p"][..], &["run", "p", "--"][..]] {
        let (_, input, _) = handled(words);
        assert_eq!(input, json!({"target":"p","argv":[]}), "{words:?}");
    }
    // After `--` a word is never the positional, so the required one is missing.
    refused(&["run", "--", "secret-canary"], "cli_parse\n");
    // A second word before `--` is neither the positional nor the list.
    refused(&["run", "p", "secret-canary"], "cli_parse\n");
    refused(&["run"], "cli_parse\n");
    runtime::command(compiled().plan()).debug_assert();
}

#[test]
fn adv466_group_path_and_every_alias_reach_the_list() {
    for path in [
        &["tools", "launch"][..],
        &["tools", "start"][..],
        &["go"][..],
    ] {
        let words = [path, &["--connection", "c", "--", "x", "--", "y"][..]].concat();
        let (output, input, _) = handled(&words);
        assert_eq!(
            input,
            json!({"connection":"c","args":["x","--","y"]}),
            "{path:?}"
        );
        assert_eq!(output.stdout, "{\"status\":\"launched\"}\n", "{path:?}");
        let words = [path, &["--connection", "c", "stray-canary"][..]].concat();
        refused(&words, "cli_parse\n");
    }
}

#[test]
fn adv466_help_shows_the_list_after_a_required_positional_and_in_a_group() {
    for (words, usage) in [
        (
            &["run", "--help"][..],
            "<field:target> [-- <field:argv>...]",
        ),
        (&["help", "run"][..], "<field:target> [-- <field:argv>...]"),
        (&["tools", "launch", "--help"][..], "[-- <field:args>...]"),
        (&["go", "--help"][..], "[-- <field:args>...]"),
    ] {
        let (output, calls) = run_on(compiled().plan(), words);
        assert_eq!(output.exit_code, 0, "{words:?}: {output:?}");
        assert_eq!(calls.len(), 0, "the handler must not run");
        assert!(
            output.stdout.contains(usage),
            "{words:?}: {}",
            output.stdout
        );
    }
    // After `--`, help and version words are values handed to the launched program.
    let (_, input, _) = handled(&["run", "p", "--", "--help", "-h", "--version", "-V", "help"]);
    assert_eq!(
        input,
        json!({"target":"p","argv":["--help","-h","--version","-V","help"]})
    );
}

#[test]
fn adv466_globals_after_double_dash_are_values_not_process_context() {
    let (output, input, context) = handled(&[
        "--config",
        "cfg",
        "tools",
        "launch",
        "--connection",
        "c",
        "--",
        "--config",
        "x",
        "--state-dir",
        "y",
        "--output",
        "json",
        "--output=json",
    ]);
    assert_eq!(
        input["args"],
        json!([
            "--config",
            "x",
            "--state-dir",
            "y",
            "--output",
            "json",
            "--output=json"
        ])
    );
    assert_eq!(
        context,
        Context {
            config: Some("cfg".into()),
            state_dir: None,
            output: OutputMode::Human,
        }
    );
    assert_eq!(output.stdout, "{\"status\":\"launched\"}\n");
}

#[test]
fn adv466_parse_failure_in_json_mode_names_no_trailing_word() {
    // The required option is missing; the selector before `--` governs the refusal.
    refused_json(
        &["--output", "json", "tools", "launch", "--", "secret-canary"],
        "cli_parse",
    );
    // A selector only after `--` is a value, so the refusal stays human.
    refused(
        &["tools", "launch", "--", "--output", "json", "secret-canary"],
        "cli_parse\n",
    );
}

#[test]
fn adv466_invalid_input_answer_is_unchanged_by_a_trailing_list() {
    let (_, input, _) = handled(&["count", "--count", "3", "--", "a", "-1"]);
    assert_eq!(input, json!({"count":3,"args":["a","-1"]}));
    refused(
        &["count", "--count", "three", "--", "secret-canary"],
        "bad_input\n",
    );
    refused_json(
        &[
            "--output=json",
            "count",
            "--count",
            "three",
            "--",
            "secret-canary",
        ],
        "bad_input",
    );
}

/// The generated package reads `binding.json` with its own copy of `wire.rs`; that plan must
/// behave exactly like the in-memory one the unit's tests drive.
#[test]
fn adv466_plan_read_back_from_binding_json_runs_like_the_compiled_plan() {
    let compiled = compiled();
    let text = compiled.to_canonical_json();
    let read: Plan = serde_json::from_str(&text).unwrap();
    assert_eq!(
        format!("{}\n", serde_json::to_string_pretty(&read).unwrap()),
        text
    );
    for words in [
        &[
            "tools",
            "launch",
            "--connection",
            "c",
            "--",
            "--flag",
            "--",
            "x",
        ][..],
        &["go", "--connection", "c"][..],
        &["run", "p", "--", "a"][..],
        &["run", "--", "a"][..],
        &["count", "--count", "x", "--", "a"][..],
    ] {
        let (left, left_calls) = run_on(compiled.plan(), words);
        let (right, right_calls) = run_on(&read, words);
        assert_eq!(left, right, "{words:?}");
        assert_eq!(left_calls, right_calls, "{words:?}");
    }
    // The generated reader is closed on the new variant too.
    let widened = text.replacen(
        "\"kind\": \"trailing\"",
        "\"kind\": \"trailing\",\n            \"index\": 3",
        1,
    );
    assert_ne!(widened, text);
    assert!(serde_json::from_str::<Plan>(&widened).is_err());
}

#[test]
fn adv466_projection_and_completions_admit_a_trailing_list() {
    let compiled = compiled();
    let artifacts = ess_cli_project::project(&compiled);
    assert!(artifacts["binding.json"].contains("\"kind\": \"trailing\""));
    assert_ne!(
        artifacts["completions/demo.bash"].len(),
        0,
        "bash completion is generated"
    );
    assert!(artifacts["help.txt"].contains("Usage: demo"));
    let (output, calls) = run_on(compiled.plan(), &["completions", "bash"]);
    assert_eq!(output.exit_code, 0, "{output:?}");
    assert_eq!(calls.len(), 0, "the handler must not run");
    assert_eq!(output.stdout, artifacts["completions/demo.bash"]);
}

/// Property: with a fixed seed, every sequence of awkward words after the first `--` reaches the
/// handler verbatim, in order, and never changes the process context or output mode.
#[test]
fn adv466_words_after_double_dash_reach_the_handler_verbatim() {
    const WORDS: &[&str] = &[
        "",
        " ",
        "-",
        "--",
        "---",
        "-x",
        "-h",
        "--help",
        "-V",
        "--version",
        "--output",
        "--output=json",
        "json",
        "--config",
        "--state-dir=s",
        "--connection",
        "c",
        "launch",
        "tools",
        "help",
        "completions",
        "[1]",
        "{}",
        "null",
        "true",
        "-1",
        "-0",
        "1e3",
        "\"",
        "\\",
        "a b",
        "\t",
        "\n",
        "\u{fc}",
        "\u{1F600}",
        "=",
        "--=x",
        "-=",
        "@file",
        "'",
        "a,b",
    ];
    let mut state: u64 = 0x0466_0466_0466_0466;
    let mut next = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(state >> 33).unwrap()
    };
    for _ in 0..400 {
        let length = next() % 9;
        let words = (0..length)
            .map(|_| WORDS[next() % WORDS.len()])
            .collect::<Vec<_>>();
        let argv = [&["go", "--connection", "c", "--"][..], &words[..]].concat();
        let (output, input, context) = handled(&argv);
        assert_eq!(input["args"], json!(words), "{argv:?}");
        assert_eq!(input["connection"], json!("c"), "{argv:?}");
        assert_eq!(context.output, OutputMode::Human, "{argv:?}");
        assert_eq!(context.config, None, "{argv:?}");
        assert_eq!(context.state_dir, None, "{argv:?}");
        assert_eq!(output.stdout, "{\"status\":\"launched\"}\n", "{argv:?}");
        assert!(output.stderr.is_empty(), "{argv:?}");
    }
}
