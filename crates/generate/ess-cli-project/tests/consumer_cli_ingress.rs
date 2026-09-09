//! Explicit authored-source admission traces with real CLI terminal controls.
//! The source reader is `serde_yaml`; these cases do not run a JSON Schema evaluator.

use ess_cli_contract::Binding;
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
};
use ess_domain::{spec::RawSpecFile, spec::Specification, system::Source};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const HEADER: &str = r"
format: ess/1
system: pilot
version: v1
domains: []
types:
  - name: pilot.Input
    kind: struct
    fields: [{name: value, type: Integer}]
";

const APP: &str = r"
domain: pilot.app
commands:
  - name: pilot.app.Work
    input: [{name: value, type: String}]
    outcomes:
      - name: done
        emits: [pilot.app.Done]
events:
  - name: pilot.app.Done
    fields: [{name: value, type: String}]
";

const BINDING: &str = r"
format: ess-cli/1
binary: pilot
about: An independently selected typed CLI value
globals: {config: config, state: state-dir, output: output}
callables:
  observe:
    target: {kind: local, owner: pilot.cli, action: observe}
    input: pilot.Input
    result: Integer
commands:
  - path: [observe]
    callable: observe
    about: Observe a typed value
    arguments:
      - {field: value, source: {kind: option, long: value}}
";

#[derive(Debug, PartialEq, Eq)]
struct Failure {
    stage: &'static str,
    diagnostic: String,
    details: Value,
}

#[derive(Debug, PartialEq, Eq)]
struct Terminal {
    ir: String,
    commands: BTreeMap<String, Value>,
    events: BTreeMap<String, Value>,
    errors: BTreeMap<String, Value>,
    binding: String,
    artifacts: BTreeMap<String, String>,
    stdout: String,
    invocation: Value,
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("source-admission witnesses cannot acquire external process inputs")
    }
}

struct Recorder<'a> {
    trace: &'a mut Vec<&'static str>,
    calls: Vec<Value>,
}
impl Handler for Recorder<'_> {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.trace.push("handler");
        assert!(invocation.context.config.is_none());
        assert!(invocation.context.state_dir.is_none());
        self.calls.push(json!({
            "callable":invocation.callable,
            "target":invocation.target,
            "input":invocation.input,
        }));
        HandlerReply::Success(invocation.input["value"].clone())
    }
}

fn assert_selected_binding(text: &str) {
    let plan: Value = serde_json::from_str(text).unwrap();
    assert_eq!(plan["format"], "ess-cli-plan/1");
    assert_eq!(plan["binary"], "pilot");
    assert_eq!(
        plan["callables"]["observe"]["input"]["type_ref"],
        "pilot.Input"
    );
}

fn pipeline(app: &str, extra: &str, trace: &mut Vec<&'static str>) -> Result<Terminal, Failure> {
    let mut parsed = Vec::new();
    let mut sources = ess_compiler::source::SourceMap::new();
    for (stage, name, text) in [
        ("read:header", "header.yaml", HEADER),
        ("read:app", "app.yaml", app),
        ("read:extra", "extra.yaml", extra),
    ] {
        trace.push(stage);
        let raw = RawSpecFile::parse(text).map_err(|error| Failure {
            stage,
            diagnostic: error.to_string(),
            details: Value::Null,
        })?;
        parsed.push((Source::new(name), raw));
        sources.insert(name.to_owned(), text.to_owned());
    }
    trace.push("assemble");
    let specification = Specification::assemble(parsed).map_err(|error| Failure {
        stage: "assemble",
        diagnostic: error.to_string(),
        details: serde_json::to_value(error.as_slice()).unwrap(),
    })?;
    trace.push("ess-compile");
    let ir = ess_compiler::compile(&specification, &sources).map_err(|error| Failure {
        stage: "ess-compile",
        diagnostic: error.to_string(),
        details: Value::Null,
    })?;
    trace.push("binding-reader");
    let binding = Binding::from_yaml(BINDING).map_err(|error| Failure {
        stage: "binding-reader",
        diagnostic: error.to_string(),
        details: Value::Null,
    })?;
    trace.push("binding-compile");
    let binding = ess_cli_contract::compile(&ir, &binding).map_err(|error| Failure {
        stage: "binding-compile",
        diagnostic: error.to_string(),
        details: Value::Null,
    })?;
    let binding_json = binding.to_canonical_json();
    assert_selected_binding(&binding_json);
    trace.push("project");
    let artifacts = ess_cli_project::project(&binding);
    assert_eq!(artifacts.len(), 10);
    assert_eq!(artifacts["binding.json"], binding_json);
    assert_eq!(artifacts, ess_cli_project::project(&binding));
    trace.push("runtime");
    let mut handler = Recorder {
        trace,
        calls: Vec::new(),
    };
    let output = runtime::run(
        binding.plan(),
        ["pilot", "observe", "--value", "17", "--output=json"]
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
        json!({"ok":true,"result":17})
    );
    assert_eq!(
        handler.calls,
        vec![
            json!({"callable":"observe","target":{"kind":"local","owner":"pilot.cli","action":"observe"},"input":{"value":17}})
        ]
    );
    Ok(Terminal {
        ir: ir.to_canonical_json(),
        commands: ir
            .commands()
            .values()
            .map(|item| (item.name.to_string(), serde_json::to_value(item).unwrap()))
            .collect(),
        events: ir
            .events()
            .values()
            .map(|item| (item.name.to_string(), serde_json::to_value(item).unwrap()))
            .collect(),
        errors: ir
            .errors()
            .values()
            .map(|item| (item.name.to_string(), serde_json::to_value(item).unwrap()))
            .collect(),
        binding: binding_json,
        artifacts,
        stdout: output.stdout,
        invocation: handler.calls.remove(0),
    })
}

fn admitted(app: &str, extra: &str) -> Terminal {
    let mut trace = Vec::new();
    let terminal = pipeline(app, extra, &mut trace).unwrap();
    assert_eq!(
        trace,
        [
            "read:header",
            "read:app",
            "read:extra",
            "assemble",
            "ess-compile",
            "binding-reader",
            "binding-compile",
            "project",
            "runtime",
            "handler"
        ]
    );
    terminal
}

fn refused(app: &str, extra: &str, stage: &'static str, diagnostic: &str) -> Failure {
    let mut trace = Vec::new();
    let error = pipeline(app, extra, &mut trace).unwrap_err();
    assert_eq!(error.stage, stage);
    assert!(
        error.diagnostic.contains(diagnostic),
        "expected {diagnostic:?}, got {error:?}"
    );
    let stages = [
        "read:header",
        "read:app",
        "read:extra",
        "assemble",
        "ess-compile",
        "binding-reader",
        "binding-compile",
        "project",
        "runtime",
        "handler",
    ];
    let last = stages
        .iter()
        .position(|candidate| *candidate == stage)
        .unwrap();
    assert_eq!(trace, stages[..=last]);
    assert!(!trace.contains(&"project"));
    assert!(!trace.contains(&"runtime"));
    assert!(!trace.contains(&"handler"));
    error
}

fn same_selected_cli(a: &Terminal, b: &Terminal) {
    assert_eq!(a.binding, b.binding);
    assert_eq!(a.artifacts, b.artifacts);
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(a.invocation, b.invocation);
}

fn topology(replicas: &str) -> String {
    format!("components:\n  - component: pilot-service\n    owns: {{domains: [pilot.app]}}\ntopology:\n  workloads:\n    pilot-service:\n      stateless: true\n      replicas: {replicas}\n")
}

#[test]
fn topology_closed_readers_stop_before_assembly_and_terminals() {
    let control = topology("{min: 1, max: 3}");
    let accepted = admitted(APP, &control);
    for malformed in [
        control.replace("{min: 1, max: 3}", "{min: 1, max: 3, extra: 1}"),
        control.replace("topology:\n", "topology:\n  extra: 1\n"),
        control.replace(
            "      stateless: true",
            "      extra: 1\n      stateless: true",
        ),
    ] {
        assert_ne!(control, malformed);
        refused(APP, &malformed, "read:extra", "unknown field `extra`");
    }
    same_selected_cli(&accepted, &admitted(APP, &control));
}

#[test]
fn topology_required_floor_reader_refusal_has_a_complete_terminal_control() {
    let control = topology("{min: 1, max: 3}");
    let accepted = admitted(APP, &control);
    let missing = control.replace("min: 1, ", "");
    assert!(missing.contains("replicas: {max: 3}"));
    refused(APP, &missing, "read:extra", "missing field `min`");
    same_selected_cli(&accepted, &admitted(APP, &control));
}

#[test]
fn topology_unsigned_bounds_distinguish_reader_and_assembly_refusals() {
    let valid = topology("{min: 1, max: 3}");
    let accepted = admitted(APP, &valid);
    for (field, from, zero_message, code) in [
        ("min", "min: 1", "a floor of zero", "type_mismatch"),
        (
            "max",
            "max: 3",
            "a ceiling of 0 below a floor of 1",
            "conflicting_declaration",
        ),
    ] {
        let negative = valid.replace(from, &format!("{field}: -1"));
        refused(APP, &negative, "read:extra", "expected u32");
        let zero = valid.replace(from, &format!("{field}: 0"));
        let raw = RawSpecFile::parse(&zero).unwrap();
        let replicas = raw.topology.unwrap().workloads["pilot-service"]
            .replicas
            .unwrap();
        assert_eq!(
            if field == "min" {
                replicas.min
            } else {
                replicas.max.unwrap()
            },
            0
        );
        let error = refused(APP, &zero, "assemble", zero_message);
        assert_eq!(error.details[0]["code"], code);
    }
    same_selected_cli(&accepted, &admitted(APP, &valid));
}

#[test]
fn topology_uint32_representation_refuses_overflow_and_admits_maximum() {
    let ordinary = topology("{min: 1, max: 3}");
    let accepted = admitted(APP, &ordinary);
    for (maximum, overflow) in [
        (topology("{min: 4294967295}"), topology("{min: 4294967296}")),
        (
            topology("{min: 1, max: 4294967295}"),
            topology("{min: 1, max: 4294967296}"),
        ),
    ] {
        let large = admitted(APP, &maximum);
        assert_ne!(large.ir, accepted.ir);
        same_selected_cli(&accepted, &large);
        refused(APP, &overflow, "read:extra", "expected u32");
    }
}

const INTERACTION: &str = r"
bindings:
  - name: work-on-done
    when: {event: pilot.app.Done}
    invoke: {command: pilot.app.Work}
    mapping: {value: event.value}
    delivery: at_least_once
    on_failure: retry
";

#[test]
fn binding_closed_objects_refuse_at_the_reader_before_any_terminal() {
    let control = admitted(APP, INTERACTION);
    for malformed in [
        INTERACTION.replace(
            "  - name: work-on-done",
            "  - name: work-on-done\n    extra: 1",
        ),
        INTERACTION.replace(
            "{event: pilot.app.Done}",
            "{event: pilot.app.Done, extra: 1}",
        ),
        INTERACTION.replace(
            "{command: pilot.app.Work}",
            "{command: pilot.app.Work, extra: 1}",
        ),
        INTERACTION.replace(
            "on_failure: retry",
            "on_failure: {escalate: {emits: pilot.app.Done, extra: 1}}",
        ),
    ] {
        refused(APP, &malformed, "read:extra", "unknown field `extra`");
    }
    let two_policies = INTERACTION.replace(
        "on_failure: retry",
        "on_failure: {escalate: {emits: pilot.app.Done}, extra: 1}",
    );
    refused(
        APP,
        &two_policies,
        "read:extra",
        "`on_failure` says `escalate` and `extra`",
    );
    same_selected_cli(&control, &admitted(APP, INTERACTION));
}

#[test]
fn binding_and_reference_required_keys_have_named_reader_refusals() {
    let control = admitted(APP, INTERACTION);
    for (old, replacement, name) in [
        ("  - name: work-on-done\n    when:", "  - when:", "name"),
        ("    when: {event: pilot.app.Done}\n", "", "when"),
        ("    invoke: {command: pilot.app.Work}\n", "", "invoke"),
        ("    delivery: at_least_once\n", "", "delivery"),
        ("    on_failure: retry\n", "", "on_failure"),
        ("{event: pilot.app.Done}", "{}", "event"),
        ("{command: pilot.app.Work}", "{}", "command"),
    ] {
        let malformed = INTERACTION.replace(old, replacement);
        refused(
            APP,
            &malformed,
            "read:extra",
            &format!("missing field `{name}`"),
        );
    }
    refused(
        APP,
        &INTERACTION.replace("on_failure: retry", "on_failure: {}"),
        "read:extra",
        "`on_failure` says nothing",
    );
    same_selected_cli(&control, &admitted(APP, INTERACTION));
}

#[test]
fn binding_object_and_scalar_representations_are_read_before_cli_resolution() {
    let control = admitted(APP, INTERACTION);
    for (old, new, expected) in [
        ("when: {event: pilot.app.Done}", "when: 9", "RawTrigger"),
        (
            "invoke: {command: pilot.app.Work}",
            "invoke: 9",
            "RawInvocation",
        ),
        ("name: work-on-done", "name: 9", "string"),
        (
            "delivery: at_least_once",
            "delivery: 9",
            "expected a Value::Tagged enum",
        ),
        (
            "on_failure: retry",
            "on_failure: 9",
            "`retry`, `drop`, or `escalate:`",
        ),
    ] {
        refused(APP, &INTERACTION.replace(old, new), "read:extra", expected);
    }
    refused(APP, "bindings: [9]", "read:extra", "RawBindingSpec");
    same_selected_cli(&control, &admitted(APP, INTERACTION));
}

#[test]
fn binding_delivery_and_failure_words_have_finite_admission_and_terminal_controls() {
    let control = admitted(APP, INTERACTION);
    for policy in ["drop", "{escalate: {emits: pilot.app.Done}}"] {
        let changed = admitted(
            APP,
            &INTERACTION.replace("on_failure: retry", &format!("on_failure: {policy}")),
        );
        assert_ne!(control.ir, changed.ir);
        same_selected_cli(&control, &changed);
    }
    refused(
        APP,
        &INTERACTION.replace("at_least_once", "exactly_once"),
        "read:extra",
        "unknown variant `exactly_once`",
    );
    refused(
        APP,
        &INTERACTION.replace("on_failure: retry", "on_failure: forget"),
        "read:extra",
        "unknown variant `forget`",
    );
}

#[test]
fn binding_escalation_null_and_omission_reach_assembly_then_stop() {
    let valid = INTERACTION.replace(
        "on_failure: retry",
        "on_failure: {escalate: {emits: pilot.app.Done}}",
    );
    let control = admitted(APP, &valid);
    for policy in [
        "escalate",
        "{escalate: null}",
        "{escalate: {}}",
        "{escalate: {emits: null}}",
    ] {
        let malformed = INTERACTION.replace("on_failure: retry", &format!("on_failure: {policy}"));
        let raw = RawSpecFile::parse(&malformed).unwrap();
        assert_eq!(raw.bindings[0].on_failure.failure.as_str(), "escalate");
        assert!(raw.bindings[0].on_failure.emits.is_none());
        let error = refused(APP, &malformed, "assemble", "escalat");
        assert!(error
            .details
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["code"] == "missing_declaration"));
    }
    same_selected_cli(&control, &admitted(APP, &valid));
}

#[test]
fn binding_reference_and_metadata_wire_values_enter_the_named_reader() {
    let described = INTERACTION.replace("  - name: work-on-done", "  - name: work-on-done\n    naming: {wire: work-notice, display: Work Notice}\n    summary: Application interaction\n    refs: [local:interaction]");
    let base = admitted(APP, INTERACTION);
    let control = admitted(APP, &described);
    assert_ne!(base.ir, control.ir);
    same_selected_cli(&base, &control);
    for (old, new, expected) in [
        ("{event: pilot.app.Done}", "{event: 'pilot..Done'}", "empty"),
        (
            "{command: pilot.app.Work}",
            "{command: 'pilot..Work'}",
            "empty",
        ),
        ("{wire: work-notice, display: Work Notice}", "9", "Naming"),
        ("refs: [local:interaction]", "refs: 9", "sequence"),
        ("refs: [local:interaction]", "refs: [9]", "string"),
    ] {
        refused(APP, &described.replace(old, new), "read:extra", expected);
    }
    let null_summary = described.replace("summary: Application interaction", "summary: null");
    let omitted_summary = described.replace("    summary: Application interaction\n", "");
    let a = admitted(APP, &null_summary);
    let b = admitted(APP, &omitted_summary);
    assert_eq!(a.ir, b.ir);
    same_selected_cli(&a, &b);
}

#[test]
fn binding_mapping_object_values_and_empty_default_have_terminal_controls() {
    let control = admitted(APP, INTERACTION);
    refused(
        APP,
        &INTERACTION.replace("mapping: {value: event.value}", "mapping: 9"),
        "read:extra",
        "mapping",
    );
    refused(
        APP,
        &INTERACTION.replace("mapping: {value: event.value}", "mapping: {value: 9}"),
        "read:extra",
        "string",
    );
    let empty_input = APP.replace("input: [{name: value, type: String}]", "input: []");
    let omitted = INTERACTION.replace("    mapping: {value: event.value}\n", "");
    let explicit = INTERACTION.replace("mapping: {value: event.value}", "mapping: {}");
    let raw_omitted = RawSpecFile::parse(&omitted).unwrap();
    let raw_explicit = RawSpecFile::parse(&explicit).unwrap();
    assert!(raw_omitted.bindings[0].mapping.0.is_empty());
    assert_eq!(
        raw_omitted.bindings[0].mapping,
        raw_explicit.bindings[0].mapping
    );
    let a = admitted(&empty_input, &omitted);
    let b = admitted(&empty_input, &explicit);
    assert_eq!(a.ir, b.ir);
    same_selected_cli(&a, &b);
    same_selected_cli(&control, &a);
}

#[test]
fn actor_closed_required_and_typed_members_refuse_in_the_reader() {
    let control_text = format!("{APP}\nactors:\n  - name: pilot.app.Worker\n    may: [pilot.app.Work]\n    naming: {{wire: worker, display: Worker}}\n");
    let control = admitted(&control_text, "{}");
    for (old, new, expected) in [
        (
            "  - name: pilot.app.Worker\n",
            "  - name: pilot.app.Worker\n    extra: 1\n",
            "unknown field `extra`",
        ),
        (
            "  - name: pilot.app.Worker\n    may:",
            "  - may:",
            "missing field `name`",
        ),
        ("name: pilot.app.Worker", "name: 'pilot..Worker'", "empty"),
        ("may: [pilot.app.Work]", "may: 9", "sequence"),
        ("may: [pilot.app.Work]", "may: ['pilot..Work']", "empty"),
        ("{wire: worker, display: Worker}", "9", "Naming"),
    ] {
        refused(&control_text.replace(old, new), "{}", "read:app", expected);
    }
    refused(
        &format!("{APP}\nactors: [9]\n"),
        "{}",
        "read:app",
        "RawActorSpec",
    );
    same_selected_cli(&control, &admitted(&control_text, "{}"));
}

#[test]
fn actor_duplicate_grants_are_normalized_by_the_reader_not_schema_validated() {
    let once = format!("{APP}\nactors:\n  - name: pilot.app.Worker\n    may: [pilot.app.Work]\n");
    let twice = once.replace(
        "may: [pilot.app.Work]",
        "may: [pilot.app.Work, pilot.app.Work]",
    );
    assert_ne!(once, twice);
    let first = RawSpecFile::parse(&once).unwrap();
    let second = RawSpecFile::parse(&twice).unwrap();
    assert_eq!(first.actors[0].may.len(), 1);
    assert_eq!(second.actors[0].may.len(), 1);
    assert_eq!(first.actors[0].may, second.actors[0].may);
    let a = admitted(&once, "{}");
    let b = admitted(&twice, "{}");
    assert_eq!(a.ir, b.ir);
    same_selected_cli(&a, &b);
}

#[test]
fn command_event_error_and_outcome_closed_required_readers_are_separate() {
    let source = format!("{APP}\nerrors:\n  - name: pilot.app.Rejected\n    fields: [{{name: reason, type: String}}]\n");
    let control = admitted(&source, "{}");
    for (old, new) in [
        (
            "  - name: pilot.app.Work\n",
            "  - name: pilot.app.Work\n    extra: 1\n",
        ),
        (
            "  - name: pilot.app.Done\n",
            "  - name: pilot.app.Done\n    extra: 1\n",
        ),
        (
            "  - name: pilot.app.Rejected\n",
            "  - name: pilot.app.Rejected\n    extra: 1\n",
        ),
        (
            "      - name: done\n",
            "      - name: done\n        extra: 1\n",
        ),
    ] {
        refused(
            &source.replace(old, new),
            "{}",
            "read:app",
            "unknown field `extra`",
        );
    }
    for (old, new) in [
        ("  - name: pilot.app.Work\n    input:", "  - input:"),
        ("  - name: pilot.app.Done\n    fields:", "  - fields:"),
        ("  - name: pilot.app.Rejected\n    fields:", "  - fields:"),
        ("      - name: done\n        emits:", "      - emits:"),
    ] {
        refused(
            &source.replace(old, new),
            "{}",
            "read:app",
            "missing field `name`",
        );
    }
    same_selected_cli(&control, &admitted(&source, "{}"));
}

#[test]
fn command_event_and_error_object_array_and_reference_shapes_are_admitted_explicitly() {
    let source = format!("{APP}\nerrors:\n  - name: pilot.app.Rejected\n    fields: [{{name: reason, type: String}}]\n");
    let control = admitted(&source, "{}");
    for (text, expected) in [
        (
            "domain: pilot.app\ncommands: [9]\n".to_owned(),
            "RawCommandSpec",
        ),
        (
            "domain: pilot.app\nevents: [9]\n".to_owned(),
            "RawEventSpec",
        ),
        (
            "domain: pilot.app\nerrors: [9]\n".to_owned(),
            "RawErrorSpec",
        ),
        (
            source.replace(
                "    outcomes:\n      - name: done\n        emits: [pilot.app.Done]",
                "    outcomes: [9]",
            ),
            "RawOutcome",
        ),
        (
            source.replace("input: [{name: value, type: String}]", "input: 9"),
            "sequence",
        ),
        (
            source.replace("input: [{name: value, type: String}]", "input: [9]"),
            "Field",
        ),
        (
            source.replace("fields: [{name: value, type: String}]", "fields: 9"),
            "sequence",
        ),
        (
            source.replace("fields: [{name: value, type: String}]", "fields: [9]"),
            "Field",
        ),
        (
            source.replace("fields: [{name: reason, type: String}]", "fields: 9"),
            "sequence",
        ),
        (
            source.replace("fields: [{name: reason, type: String}]", "fields: [9]"),
            "Field",
        ),
        (
            source.replace("emits: [pilot.app.Done]", "emits: 9"),
            "sequence",
        ),
        (
            source.replace("emits: [pilot.app.Done]", "emits: ['pilot..Done']"),
            "empty",
        ),
        (
            source.replace("name: pilot.app.Work", "name: 'pilot..Work'"),
            "empty",
        ),
        (
            source.replace("name: pilot.app.Done", "name: 'pilot..Done'"),
            "empty",
        ),
        (
            source.replace("name: pilot.app.Rejected", "name: 'pilot..Rejected'"),
            "empty",
        ),
    ] {
        refused(&text, "{}", "read:app", expected);
    }
    same_selected_cli(&control, &admitted(&source, "{}"));
}

#[test]
fn outcome_name_pattern_and_scalar_type_stop_at_the_source_reader() {
    let control = admitted(APP, "{}");
    for malformed in ["Done", "not valid", "two--parts"] {
        refused(
            &APP.replace("name: done", &format!("name: {malformed:?}")),
            "{}",
            "read:app",
            "outcome",
        );
    }
    refused(
        &APP.replace("name: done", "name: 9"),
        "{}",
        "read:app",
        "string",
    );
    let renamed = admitted(&APP.replace("name: done", "name: work-done"), "{}");
    assert_ne!(control.ir, renamed.ir);
    same_selected_cli(&control, &renamed);
}

#[test]
fn outcome_nullable_members_and_false_flag_normalize_through_all_terminals() {
    let control = admitted(APP, "{}");
    for field in [
        "creates", "moves", "updates", "error", "external", "instance", "when", "summary",
        "refuses",
    ] {
        let source = APP.replace(
            "      - name: done\n",
            &format!("      - name: done\n        {field}: null\n"),
        );
        assert_ne!(source, APP);
        let raw = RawSpecFile::parse(&source).unwrap();
        let outcome = &raw.commands[0].outcomes[0];
        assert!(
            outcome.creates.is_none()
                && outcome.moves.is_none()
                && outcome.updates.is_none()
                && outcome.error.is_none()
                && outcome.external.is_none()
                && outcome.instance.is_none()
                && outcome.when.is_none()
                && outcome.summary.is_none()
                && outcome.refuses.is_none()
        );
        let normalized = admitted(&source, "{}");
        assert_eq!(control.ir, normalized.ir);
        same_selected_cli(&control, &normalized);
    }
    let source = APP.replace(
        "      - name: done\n",
        "      - name: done\n        wrong_state: false\n",
    );
    assert!(!RawSpecFile::parse(&source).unwrap().commands[0].outcomes[0].wrong_state);
    let normalized = admitted(&source, "{}");
    assert_eq!(control.ir, normalized.ir);
    same_selected_cli(&control, &normalized);
}

#[test]
fn outcome_predicate_and_external_string_branches_reach_admitted_terminals() {
    let control = admitted(APP, "{}");
    for (declaration, kind) in [
        ("when: value == 'yes'", "when"),
        ("external: an application decision", "external"),
    ] {
        let source = APP.replace("      - name: done\n", &format!("      - name: done\n        {declaration}\n"))
            .replace("        emits: [pilot.app.Done]\n", "        emits: [pilot.app.Done]\n      - name: otherwise\n        emits: [pilot.app.Done]\n");
        let admitted = admitted(&source, "{}");
        assert_eq!(
            admitted.commands["pilot.app.Work"]["outcomes"][0]["condition"]["kind"],
            kind
        );
        assert_eq!(
            admitted.commands["pilot.app.Work"]["outcomes"][1]["condition"]["kind"],
            "otherwise"
        );
        assert_ne!(control.ir, admitted.ir);
        same_selected_cli(&control, &admitted);
    }
}

#[test]
fn outcome_reference_payload_set_and_metadata_types_have_exact_reader_boundaries() {
    let control = admitted(APP, "{}");
    for (line, expected) in [
        ("creates: 'pilot..Record'", "empty"),
        ("moves: 'pilot..Record.close'", "empty"),
        ("updates: 'pilot..Record'", "empty"),
        ("error: 'pilot..Rejected'", "empty"),
        (
            "when: 9",
            "predicate: expected an expression, list or mapping, found the number 9",
        ),
        ("external: []", "string"),
        ("instance: []", "string"),
        ("summary: []", "string"),
        ("wrong_state: 9", "boolean"),
        ("refuses: 9", "boolean"),
        ("refs: 9", "sequence"),
        ("refs: [9]", "string"),
        ("payload: 9", "mapping"),
        ("payload: {pilot.app.Done: 9}", "mapping"),
        ("payload: {pilot.app.Done: {value: 9}}", "string"),
        ("sets: 9", "mapping"),
        ("sets: {value: 9}", "string"),
    ] {
        let source = APP.replace(
            "      - name: done\n",
            &format!("      - name: done\n        {line}\n"),
        );
        refused(&source, "{}", "read:app", expected);
    }
    same_selected_cli(&control, &admitted(APP, "{}"));
}

#[test]
fn command_event_naming_refs_and_error_summary_have_admitted_shapes() {
    let error = format!("{APP}\nerrors:\n  - name: pilot.app.Rejected\n    summary: Rejected\n");
    let base = admitted(&error, "{}");
    for (needle, addition) in [
        (
            "  - name: pilot.app.Work\n",
            "    naming: {wire: application-work, display: Work}\n    refs: [local:work]\n",
        ),
        (
            "  - name: pilot.app.Done\n",
            "    naming: {wire: application-done, display: Done}\n",
        ),
    ] {
        let named = error.replace(needle, &format!("{needle}{addition}"));
        let control = admitted(&named, "{}");
        assert_ne!(base.ir, control.ir);
        same_selected_cli(&base, &control);
        let bad_naming = named
            .replace("{wire: application-work, display: Work}", "9")
            .replace("{wire: application-done, display: Done}", "9");
        refused(&bad_naming, "{}", "read:app", "Naming");
    }
    let command_refs = error.replace(
        "  - name: pilot.app.Work\n",
        "  - name: pilot.app.Work\n    refs: [local:work]\n",
    );
    refused(
        &command_refs.replace("refs: [local:work]", "refs: 9"),
        "{}",
        "read:app",
        "sequence",
    );
    refused(
        &command_refs.replace("refs: [local:work]", "refs: [9]"),
        "{}",
        "read:app",
        "string",
    );
    let omitted = error.replace("    summary: Rejected\n", "");
    let null = error.replace("summary: Rejected", "summary: null");
    let a = admitted(&omitted, "{}");
    let b = admitted(&null, "{}");
    assert_eq!(a.ir, b.ir);
    same_selected_cli(&a, &b);
    refused(
        &error.replace("summary: Rejected", "summary: []"),
        "{}",
        "read:app",
        "string",
    );
}

const EFFECTS: &str = r"
domain: pilot.app
entities:
  - name: pilot.app.Record
    identity: {name: record_id, type: String}
    fields: [{name: value, type: String}, {name: note, type: String}]
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions: [{name: close, from: [Open], to: Closed}]
errors:
  - name: pilot.app.WrongState
    fields: [{name: message, type: String}]
commands:
  - name: pilot.app.Make
    input: [{name: id, type: String}, {name: value, type: String}]
    outcomes:
      - name: made
        creates: pilot.app.Record
        instance: record_id
        emits: [pilot.app.Changed]
        payload:
          pilot.app.Changed: {value: input.value}
        sets: {value: input.value}
  - name: pilot.app.Close
    input: [{name: id, type: String}]
    outcomes:
      - name: closed
        moves: pilot.app.Record.close
        instance: id
        emits: [pilot.app.Changed]
      - name: wrong-state
        wrong_state: true
        error: pilot.app.WrongState
events:
  - name: pilot.app.Changed
    fields: [{name: record_id, type: String}, {name: value, type: String}, {name: note, type: String}]
";

#[test]
fn creating_and_updating_the_same_real_entity_have_separate_resolved_effects() {
    let updated_source = EFFECTS.replace(
        "creates: pilot.app.Record\n        instance: record_id",
        "updates: pilot.app.Record\n        instance: id",
    );
    let raw_create = RawSpecFile::parse(EFFECTS).unwrap();
    let raw_update = RawSpecFile::parse(&updated_source).unwrap();
    assert!(raw_create.commands[0].outcomes[0].creates.is_some());
    assert!(raw_create.commands[0].outcomes[0].updates.is_none());
    assert!(raw_update.commands[0].outcomes[0].creates.is_none());
    assert!(raw_update.commands[0].outcomes[0].updates.is_some());
    let a = admitted(EFFECTS, "{}");
    let b = admitted(&updated_source, "{}");
    assert_eq!(
        a.commands["pilot.app.Make"]["outcomes"][0]["subject"]["effect"],
        "creates"
    );
    assert_eq!(
        b.commands["pilot.app.Make"]["outcomes"][0]["subject"]["effect"],
        "updates"
    );
    assert_eq!(
        a.commands["pilot.app.Make"]["outcomes"][0]["subject"]["entity"],
        b.commands["pilot.app.Make"]["outcomes"][0]["subject"]["entity"]
    );
    assert_ne!(a.ir, b.ir);
    same_selected_cli(&a, &b);
}

#[test]
fn lifecycle_move_identity_and_subject_entity_are_resolved_before_cli_terminals() {
    let a = admitted(EFFECTS, "{}");
    let moved = EFFECTS
        .replace("name: close, from:", "name: finish, from:")
        .replace(
            "moves: pilot.app.Record.close",
            "moves: pilot.app.Record.finish",
        );
    let b = admitted(&moved, "{}");
    assert_eq!(
        a.commands["pilot.app.Close"]["outcomes"][0]["subject"]["effect"],
        "moves"
    );
    assert_eq!(
        a.commands["pilot.app.Close"]["outcomes"][0]["subject"]["transition"]["name"],
        "close"
    );
    assert_eq!(
        b.commands["pilot.app.Close"]["outcomes"][0]["subject"]["transition"]["name"],
        "finish"
    );
    assert_ne!(
        RawSpecFile::parse(EFFECTS).unwrap().commands[1].outcomes[0].moves,
        RawSpecFile::parse(&moved).unwrap().commands[1].outcomes[0].moves
    );
    same_selected_cli(&a, &b);
    let renamed = admitted(
        &EFFECTS.replace("pilot.app.Record", "pilot.app.Document"),
        "{}",
    );
    assert_eq!(
        a.commands["pilot.app.Make"]["outcomes"][0]["subject"]["entity"],
        "pilot.app.Record"
    );
    assert_eq!(
        renamed.commands["pilot.app.Make"]["outcomes"][0]["subject"]["entity"],
        "pilot.app.Document"
    );
    same_selected_cli(&a, &renamed);
}

#[test]
fn wrong_state_and_idempotent_acceptance_have_explicit_application_semantics() {
    let a = admitted(EFFECTS, "{}");
    let absent = EFFECTS.replace("      - name: wrong-state\n        wrong_state: true\n        error: pilot.app.WrongState\n", "");
    let omitted = admitted(&absent, "{}");
    assert_eq!(
        a.commands["pilot.app.Close"]["outcomes"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        omitted.commands["pilot.app.Close"]["outcomes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        a.commands["pilot.app.Close"]["outcomes"][1]["condition"]["kind"],
        "wrong_state"
    );
    assert_eq!(
        a.commands["pilot.app.Close"]["outcomes"][1]["test_strategy"],
        "arrange_state"
    );
    same_selected_cli(&a, &omitted);
    let accepted_source = EFFECTS.replace(
        "        error: pilot.app.WrongState",
        "        refuses: false",
    );
    let explicit_true = EFFECTS.replace(
        "        error: pilot.app.WrongState",
        "        refuses: true\n        error: pilot.app.WrongState",
    );
    let accepting_raw = RawSpecFile::parse(&accepted_source).unwrap();
    assert_eq!(accepting_raw.commands[1].outcomes[1].refuses, Some(false));
    assert_eq!(
        RawSpecFile::parse(&explicit_true).unwrap().commands[1].outcomes[1].refuses,
        Some(true)
    );
    let accepted = admitted(&accepted_source, "{}");
    let explicit = admitted(&explicit_true, "{}");
    assert_eq!(
        accepted.commands["pilot.app.Close"]["outcomes"][1]["refuses"],
        false
    );
    assert!(accepted.commands["pilot.app.Close"]["outcomes"][1]["error"].is_null());
    assert_eq!(explicit.ir, a.ir);
    assert_ne!(accepted.ir, a.ir);
    same_selected_cli(&a, &accepted);
    same_selected_cli(&a, &explicit);
}

#[test]
fn payload_targets_events_and_entity_sets_are_application_facts() {
    let a = admitted(EFFECTS, "{}");
    for (source, expected) in [
        (
            EFFECTS.replace("sets: {value: input.value}", "sets: {note: input.value}"),
            "note",
        ),
        (
            EFFECTS.replace("sets: {value: input.value}", "sets: {value: '\"fixed\"'}"),
            "value",
        ),
    ] {
        let b = admitted(&source, "{}");
        assert_ne!(
            a.commands["pilot.app.Make"]["outcomes"][0]["sets"],
            b.commands["pilot.app.Make"]["outcomes"][0]["sets"]
        );
        assert_eq!(
            b.commands["pilot.app.Make"]["outcomes"][0]["sets"][0]["target"],
            expected
        );
        assert_ne!(
            RawSpecFile::parse(EFFECTS).unwrap().commands[0].outcomes[0].sets,
            RawSpecFile::parse(&source).unwrap().commands[0].outcomes[0].sets
        );
        same_selected_cli(&a, &b);
    }
    let target_changed = EFFECTS.replace(
        "pilot.app.Changed: {value: input.value}",
        "pilot.app.Changed: {note: input.value}",
    );
    let b = admitted(&target_changed, "{}");
    assert_eq!(
        a.commands["pilot.app.Make"]["outcomes"][0]["payload"][0]["fields"][0]["target"],
        "value"
    );
    assert_eq!(
        b.commands["pilot.app.Make"]["outcomes"][0]["payload"][0]["fields"][0]["target"],
        "note"
    );
    same_selected_cli(&a, &b);
    let event_renamed = admitted(
        &EFFECTS.replace("pilot.app.Changed", "pilot.app.Revised"),
        "{}",
    );
    assert_eq!(
        a.commands["pilot.app.Make"]["outcomes"][0]["payload"][0]["event"],
        "pilot.app.Changed"
    );
    assert_eq!(
        event_renamed.commands["pilot.app.Make"]["outcomes"][0]["payload"][0]["event"],
        "pilot.app.Revised"
    );
    same_selected_cli(&a, &event_renamed);
}

#[test]
fn payload_source_target_types_and_declared_conversion_reach_resolved_facts() {
    let plain = EFFECTS.replace("        sets: {value: input.value}\n", "");
    let a = admitted(&plain, "{}");
    let typed = plain.replace(
        "domain: pilot.app\n",
        "domain: pilot.app\ntypes:\n  - {name: pilot.app.Source, kind: newtype, of: String}\n  - {name: pilot.app.Target, kind: newtype, of: String}\nconversions:\n  - {from: pilot.app.Source, to: pilot.app.Target, because: Explicit application payload conversion}\n",
    ).replace(
        "input: [{name: id, type: String}, {name: value, type: String}]",
        "input: [{name: id, type: String}, {name: value, type: pilot.app.Source}]",
    ).replace(
        "fields: [{name: record_id, type: String}, {name: value, type: String}, {name: note, type: String}]",
        "fields: [{name: record_id, type: String}, {name: value, type: pilot.app.Target}, {name: note, type: String}]",
    );
    let b = admitted(&typed, "{}");
    let first = &a.commands["pilot.app.Make"]["outcomes"][0]["payload"][0]["fields"][0];
    let second = &b.commands["pilot.app.Make"]["outcomes"][0]["payload"][0]["fields"][0];
    assert!(first["conversion"].is_null());
    assert_eq!(
        second["conversion"],
        "Explicit application payload conversion"
    );
    assert_eq!(
        second["target_type"],
        json!({"kind":"declared", "name":"pilot.app.Target"})
    );
    assert_eq!(
        second["value"]["type_ref"],
        json!({"kind":"declared", "name":"pilot.app.Source"})
    );
    assert_ne!(first["target_type"], second["target_type"]);
    assert_ne!(first["value"]["type_ref"], second["value"]["type_ref"]);
    same_selected_cli(&a, &b);
}

#[test]
fn event_and_error_domain_ownership_changes_before_independent_cli_execution() {
    let a = admitted(EFFECTS, "{}");
    let b = admitted(&EFFECTS.replace("pilot.app", "pilot.other"), "{}");
    assert_eq!(a.events["pilot.app.Changed"]["domain"], "pilot.app");
    assert_eq!(b.events["pilot.other.Changed"]["domain"], "pilot.other");
    assert_eq!(a.errors["pilot.app.WrongState"]["domain"], "pilot.app");
    assert_eq!(b.errors["pilot.other.WrongState"]["domain"], "pilot.other");
    same_selected_cli(&a, &b);
}
