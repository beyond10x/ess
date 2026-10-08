//! beyond10x/ess#481: a generated CLI defines only the globals its `ess-cli/2` binding declares,
//! and one without `output` writes JSON only.
use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Context, Handler, HandlerReply, Invocation, OutputMode, ProcessOutput,
    ProtectedSource, Sources,
};
use ess_cli_project::wire::Plan;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

const MODEL: &str = include_str!("../../../specify/ess-cli-contract/tests/fixtures/model.yaml");

/// An inputless `ess-cli/2` CLI whose `globals` is the flow mapping `globals`.
fn compiled(globals: &str) -> CompiledBinding {
    let model = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    let model = ess_compiler::compile(&model, &ess_compiler::source::SourceMap::new()).unwrap();
    let text = format!(
        "format: ess-cli/2\nbinary: demo\nabout: Globals\nglobals: {globals}\ncallables:\n  show:\n    target: {{kind: local, owner: demo.cli, action: show}}\n    input: null\n    result: demo.Stored\n    errors: {{failed: demo.Failure}}\ncommands:\n  - path: [show]\n    callable: show\n    about: Show\n    arguments: []\n"
    );
    compile(&model, &Binding::from_yaml(&text).unwrap()).unwrap()
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("this command has no external input source")
    }
}

#[derive(Default)]
struct Recorder {
    contexts: Vec<Context>,
    fail: bool,
}
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.contexts.push(invocation.context.clone());
        if self.fail {
            HandlerReply::Error {
                code: "failed".to_owned(),
                data: json!({"reason": "r"}),
            }
        } else {
            HandlerReply::Success(json!({"profile": "p"}))
        }
    }
}

fn run(plan: &Plan, args: &[&str], handler: &mut Recorder) -> ProcessOutput {
    runtime::run(
        plan,
        args.iter().map(|&arg| arg.into()).collect(),
        &mut NoSources,
        handler,
        None,
    )
}

fn parsed(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|error| panic!("not JSON ({error}): {text}"))
}

#[test]
fn a_cli_without_config_and_output_defines_neither_flag() {
    let compiled = compiled("{state: state-dir}");
    let longs: Vec<String> = runtime::command(compiled.plan())
        .get_arguments()
        .filter_map(|argument| argument.get_long().map(str::to_owned))
        .filter(|long| long != "help" && long != "version")
        .collect();
    assert_eq!(longs, ["state-dir"]);
    let artifacts = ess_cli_project::project(&compiled);
    for file in ["help.txt", "completions/demo.bash"] {
        assert!(artifacts[file].contains("--state-dir"), "{file}");
        for absent in ["--config", "--output"] {
            assert!(!artifacts[file].contains(absent), "{file} lists {absent}");
        }
    }
}

#[test]
fn a_cli_without_output_writes_json_only() {
    let compiled = compiled("{state: state-dir}");
    let plan = compiled.plan();
    let mut handler = Recorder::default();
    let success = run(plan, &["demo", "show"], &mut handler);
    assert_eq!(success.exit_code, 0, "{success:?}");
    assert_eq!(
        parsed(&success.stdout),
        json!({"ok": true, "result": {"profile": "p"}})
    );
    handler.fail = true;
    let failure = run(plan, &["demo", "show"], &mut handler);
    assert_eq!(failure.exit_code, 1, "{failure:?}");
    assert_eq!(
        parsed(&failure.stderr),
        json!({"ok": false, "error": {"code": "failed", "data": {"reason": "r"}}})
    );
    assert_eq!(handler.contexts.len(), 2);
    for context in &handler.contexts {
        assert_eq!(context.output, OutputMode::Json);
        assert_eq!(context.config, None);
    }
    for args in [
        &["demo", "unknown"][..],
        &["demo", "show", "--output", "human"],
        &["demo", "show", "--output=human"],
        &["demo", "show", "--config", "settings.yaml"],
    ] {
        let refused = run(plan, args, &mut handler);
        assert_eq!(refused.exit_code, 2, "{args:?}: {refused:?}");
        assert_eq!(
            parsed(&refused.stderr),
            json!({"ok": false, "error": {"code": "cli_parse", "data": {}}}),
            "{args:?}"
        );
    }
    assert_eq!(
        handler.contexts.len(),
        2,
        "a refused parse reached the handler"
    );
}

#[test]
fn a_cli_without_config_still_selects_its_output_and_state() {
    let compiled = compiled("{state: state-dir, output: output}");
    let plan = compiled.plan();
    let mut handler = Recorder::default();
    let human = run(
        plan,
        &["demo", "show", "--state-dir", "records"],
        &mut handler,
    );
    assert_eq!(human.stdout, "{\"profile\":\"p\"}\n", "{human:?}");
    let selected = run(plan, &["demo", "show", "--output=json"], &mut handler);
    assert_eq!(
        parsed(&selected.stdout),
        json!({"ok": true, "result": {"profile": "p"}})
    );
    assert_eq!(
        handler.contexts,
        [
            Context {
                config: None,
                state_dir: Some("records".into()),
                output: OutputMode::Human,
            },
            Context {
                config: None,
                state_dir: None,
                output: OutputMode::Json,
            },
        ]
    );
    let refused = run(
        plan,
        &["demo", "show", "--config", "settings.yaml"],
        &mut handler,
    );
    assert_eq!(
        (refused.exit_code, refused.stderr.as_str()),
        (2, "cli_parse\n")
    );
    assert_eq!(handler.contexts.len(), 2);
}

#[test]
fn a_cli_without_output_says_so_in_its_generated_reference() {
    let without = ess_cli_project::project(&compiled("{state: state-dir}"));
    assert!(
        without["README.md"].contains("JSON only"),
        "{}",
        without["README.md"]
    );
    let with = ess_cli_project::project(&compiled("{state: state-dir, output: output}"));
    assert!(
        !with["README.md"].contains("JSON only"),
        "{}",
        with["README.md"]
    );
}

/// The generated package reads `binding.json` with its own copy of `wire.rs`: an omitted global
/// stays omitted through that round trip, and the read-back plan runs like the compiled one.
#[test]
fn a_plan_without_config_and_output_reads_back_from_binding_json() {
    let compiled = compiled("{state: state-dir}");
    let artifacts = ess_cli_project::project(&compiled);
    let written = parsed(&artifacts["binding.json"]);
    assert_eq!(written["format"], "ess-cli-plan/2");
    assert_eq!(written["globals"], json!({"state": "state-dir"}));
    let read: Plan = serde_json::from_str(&artifacts["binding.json"]).unwrap();
    assert_eq!(serde_json::to_value(&read).unwrap(), written);
    let mut handler = Recorder::default();
    assert_eq!(
        run(&read, &["demo", "show"], &mut handler),
        run(compiled.plan(), &["demo", "show"], &mut handler)
    );
}

/// `fixtures/binding.json` is the `ess-cli-plan/1` ESS 0.53.0 projected: today's reader reads it
/// and writes it back byte for byte, so declaring a global optional moved no released plan.
#[test]
fn a_released_plan_reads_and_writes_back_byte_for_byte() {
    let released = include_str!("fixtures/binding.json");
    let plan: Plan = serde_json::from_str(released).unwrap();
    assert_eq!(
        format!("{}\n", serde_json::to_string_pretty(&plan).unwrap()),
        released
    );
}
