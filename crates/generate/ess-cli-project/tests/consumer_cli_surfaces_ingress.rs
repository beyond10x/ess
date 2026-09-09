//! Authored component/view ingress, with the actual rejecting stage retained.
//! This local handler does not implement component transport or view evaluation.

use std::collections::BTreeMap;

use ess_cli_contract::{Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
};
use ess_compiler::{source::SourceMap, EssIr};
use ess_domain::{
    component::ComponentName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};

fn document() -> Value {
    json!({
        "format": "ess/1", "system": "surface", "version": "v1",
        "domains": ["surface.records"], "domain": "surface.records",
        "types": [
            {"name":"surface.records.Input", "kind":"struct", "fields":[{"name":"name","type":"String"}]},
            {"name":"surface.records.Row", "kind":"struct", "fields":[{"name":"name","type":"String"}]}
        ],
        "entities": [{
            "name":"surface.records.Record",
            "identity":{"name":"record_id","type":"String"},
            "fields":[{"name":"name","type":"String"}],
            "lifecycle":{"initial":"Active","states":["Active"],"terminal":["Active"]}
        }],
        "commands":[{
            "name":"surface.records.Rename",
            "input":[{"name":"name","type":"String"}],
            "outcomes":[{"name":"renamed","emits":["surface.records.Renamed"],
                "payload":{"surface.records.Renamed":{"name":"input.name"}}}]
        }],
        "events":[{"name":"surface.records.Renamed","fields":[{"name":"name","type":"String"}]}],
        "views":[{
            "name":"surface.records.Named", "source":"surface.records.Record",
            "fields":[{"name":"name","type":"String"}],
            "params":[{"name":"prefix","type":"String"}],
            "filter":{"all":["name == param.prefix"]},
            "consistency":"eventual"
        }],
        "components":[{
            "name":"record-service",
            "owns":{"domains":["surface.records"]},
            "accepts":{"commands":["surface.records.Rename"]},
            "publishes":{"events":["surface.records.Renamed"]}
        }]
    })
}

fn binding() -> Binding {
    Binding::from_yaml(
        r"
format: ess-cli/1
binary: surface
about: Observe one typed local invocation
globals: {config: config, state: state-dir, output: output}
callables:
  observe:
    target: {kind: local, owner: surface.cli, action: observe}
    input: surface.records.Input
    result: String
commands:
  - path: [observe]
    callable: observe
    about: Observe the selected input
    arguments:
      - {field: name, source: {kind: option, long: name}}
",
    )
    .unwrap()
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("surface ingress must not acquire credentials")
    }
}

#[derive(Default)]
struct Recorder(Vec<Value>);
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.0.push(json!({
            "target": invocation.target,
            "input": invocation.input,
            "callable": invocation.callable
        }));
        HandlerReply::Success(invocation.input["name"].clone())
    }
}

struct Observed {
    ir: EssIr,
    bound: CompiledBinding,
    artifacts: BTreeMap<String, String>,
    stdout: String,
    calls: Vec<Value>,
}

fn pipeline(input: &Value, stages: &mut Vec<&'static str>) -> Result<Observed, String> {
    stages.push("reader");
    let text = input.to_string();
    let raw = RawSpecFile::parse(&text).map_err(|error| error.to_string())?;
    stages.push("assembly");
    let spec = Specification::assemble(vec![(Source::new("surface.json"), raw)])
        .map_err(|error| error.to_string())?;
    stages.push("compiler");
    let mut source = SourceMap::new();
    source.insert("surface.json".to_owned(), text);
    let ir = ess_compiler::compile(&spec, &source).map_err(|error| error.to_string())?;
    stages.push("binding");
    let bound = ess_cli_contract::compile(&ir, &binding()).map_err(|error| error.to_string())?;
    stages.push("emission");
    let artifacts = ess_cli_project::project(&bound);
    stages.push("execution");
    let mut recorder = Recorder::default();
    let output = runtime::run(
        bound.plan(),
        ["surface", "observe", "--name", "Ada", "--output=json"]
            .into_iter()
            .map(Into::into)
            .collect(),
        &mut NoSources,
        &mut recorder,
        None,
    );
    assert_eq!(output.exit_code, 0, "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&output.stdout).unwrap(),
        json!({"ok":true,"result":"Ada"})
    );
    assert_eq!(recorder.0.len(), 1);
    assert_eq!(recorder.0[0]["input"], json!({"name":"Ada"}));
    assert_eq!(recorder.0[0]["target"]["owner"], "surface.cli");
    assert!(artifacts.contains_key("Cargo.toml"));
    assert_eq!(artifacts["binding.json"], bound.to_canonical_json());
    Ok(Observed {
        ir,
        bound,
        artifacts,
        stdout: output.stdout,
        calls: recorder.0,
    })
}

fn admitted(input: &Value) -> Observed {
    let mut stages = Vec::new();
    let result = pipeline(input, &mut stages).unwrap_or_else(|error| panic!("{stages:?}: {error}"));
    assert_eq!(
        stages,
        [
            "reader",
            "assembly",
            "compiler",
            "binding",
            "emission",
            "execution"
        ]
    );
    result
}

fn equal_terminals(before: &Observed, after: &Observed) {
    assert_eq!(
        before.bound.to_canonical_json(),
        after.bound.to_canonical_json()
    );
    assert_eq!(before.artifacts, after.artifacts);
    assert_eq!(before.stdout, after.stdout);
    assert_eq!(before.calls, after.calls);
}

fn reader_refused(input: &Value, diagnostic: &str, control: &Value) {
    let mut stages = Vec::new();
    let Err(error) = pipeline(input, &mut stages) else {
        panic!("expected reader refusal for {input}");
    };
    assert_eq!(stages, ["reader"], "{error}");
    assert!(
        error.contains(diagnostic),
        "expected {diagnostic:?}: {error}"
    );
    admitted(control);
}

fn object<'a>(value: &'a mut Value, path: &str) -> &'a mut serde_json::Map<String, Value> {
    value.pointer_mut(path).unwrap().as_object_mut().unwrap()
}

fn set(value: &Value, path: &str, replacement: Value) -> Value {
    let mut changed = value.clone();
    *changed.pointer_mut(path).unwrap() = replacement;
    assert_ne!(changed, *value);
    changed
}

fn legacy(grouped: bool) -> Value {
    let mut input = document();
    let surface = if grouped {
        json!({"binary":"old-records","groups":[{
            "name":"admin","summary":"Legacy administration",
            "commands":["surface.records.Rename"],"views":["surface.records.Named"]
        }]})
    } else {
        json!({"binary":"old-records","commands":["surface.records.Rename"],
            "views":["surface.records.Named"]})
    };
    object(&mut input, "/components/0").insert("reached_by".into(), json!("command_line"));
    object(&mut input, "/components/0").insert("cli".into(), surface);
    input
}

#[test]
fn legacy_members_are_resolved_and_do_not_override_the_selected_cli() {
    let base = admitted(&document());
    for grouped in [false, true] {
        let mut input = legacy(grouped);
        for (binary, group, summary) in [
            ("old-records", "admin", "Legacy administration"),
            ("other-records", "inspect", "Read legacy records"),
        ] {
            input["components"][0]["cli"]["binary"] = json!(binary);
            if grouped {
                input["components"][0]["cli"]["groups"][0]["name"] = json!(group);
                input["components"][0]["cli"]["groups"][0]["summary"] = json!(summary);
            }
            let raw = RawSpecFile::parse(&input.to_string()).unwrap();
            let raw_cli = raw.components[0].cli.as_ref().unwrap();
            assert_eq!(raw_cli.binary, binary);
            let observed = admitted(&input);
            let owner = &observed.ir.components()[&ComponentName::new("record-service").unwrap()];
            let resolved = owner.cli.as_ref().unwrap();
            assert_eq!(resolved.binary.as_str(), binary);
            let (commands, views) = if grouped {
                assert_eq!(raw_cli.groups[0].name, group);
                assert_eq!(raw_cli.groups[0].summary.as_deref(), Some(summary));
                assert_eq!(
                    raw_cli.groups[0].commands[0].to_string(),
                    "surface.records.Rename"
                );
                assert_eq!(
                    raw_cli.groups[0].views[0].to_string(),
                    "surface.records.Named"
                );
                assert!(raw_cli.commands.is_empty() && raw_cli.views.is_empty());
                assert_eq!(resolved.groups[0].name.as_str(), group);
                assert_eq!(resolved.groups[0].summary.as_deref(), Some(summary));
                assert!(resolved.commands.is_empty() && resolved.views.is_empty());
                (&resolved.groups[0].commands, &resolved.groups[0].views)
            } else {
                assert_eq!(raw_cli.commands[0].to_string(), "surface.records.Rename");
                assert_eq!(raw_cli.views[0].to_string(), "surface.records.Named");
                assert!(raw_cli.groups.is_empty() && resolved.groups.is_empty());
                (&resolved.commands, &resolved.views)
            };
            assert_eq!(commands.len(), 1);
            assert_eq!(views.len(), 1);
            assert_eq!(
                observed
                    .ir
                    .command(commands.first().unwrap())
                    .name
                    .to_string(),
                "surface.records.Rename"
            );
            assert_eq!(
                observed.ir.view(views.first().unwrap()).name.to_string(),
                "surface.records.Named"
            );
            equal_terminals(&base, &observed);
        }
    }
}

#[test]
fn component_and_legacy_closed_objects_refuse_unknown_members_at_the_reader() {
    for path in [
        "/components/0",
        "/components/0/owns",
        "/components/0/accepts",
        "/components/0/publishes",
        "/components/0/cli",
        "/components/0/cli/groups/0",
    ] {
        let control = legacy(true);
        let mut malformed = control.clone();
        object(&mut malformed, path).insert("undeclared".into(), json!(true));
        reader_refused(&malformed, "unknown field `undeclared`", &control);
    }
}

#[test]
fn component_and_legacy_required_names_refuse_before_assembly() {
    for (path, field) in [
        ("/components/0", "name"),
        ("/components/0/cli", "binary"),
        ("/components/0/cli/groups/0", "name"),
    ] {
        let control = legacy(true);
        let mut malformed = control.clone();
        assert!(object(&mut malformed, path).remove(field).is_some());
        reader_refused(&malformed, &format!("missing field `{field}`"), &control);
    }
}

#[test]
fn component_surface_wire_container_and_scalar_shapes_are_reader_obligations() {
    let control = legacy(true);
    for path in [
        "/components/0",
        "/components/0/owns",
        "/components/0/accepts",
        "/components/0/publishes",
        "/components/0/cli",
        "/components/0/cli/groups/0",
    ] {
        reader_refused(&set(&control, path, json!(false)), "invalid type", &control);
    }
    for path in [
        "/components/0/owns/domains",
        "/components/0/accepts/commands",
        "/components/0/publishes/events",
        "/components/0/cli/groups",
        "/components/0/cli/groups/0/commands",
        "/components/0/cli/groups/0/views",
    ] {
        reader_refused(
            &set(&control, path, json!({})),
            "expected a sequence",
            &control,
        );
    }
    for path in [
        "/components/0/name",
        "/components/0/cli/binary",
        "/components/0/cli/groups/0/name",
        "/components/0/cli/groups/0/summary",
        "/components/0/owns/domains/0",
        "/components/0/accepts/commands/0",
        "/components/0/publishes/events/0",
        "/components/0/cli/groups/0/commands/0",
        "/components/0/cli/groups/0/views/0",
    ] {
        reader_refused(
            &set(&control, path, json!({})),
            "expected a string",
            &control,
        );
    }
    let root = legacy(false);
    for path in ["/components/0/cli/commands", "/components/0/cli/views"] {
        reader_refused(&set(&root, path, json!({})), "expected a sequence", &root);
        reader_refused(
            &set(&root, &format!("{path}/0"), json!({})),
            "expected a string",
            &root,
        );
    }
}

#[test]
fn component_defaults_nulls_and_metadata_preserve_local_invocation() {
    let control = document();
    let before = admitted(&control);
    for field in ["summary", "cli"] {
        let mut changed = control.clone();
        object(&mut changed, "/components/0").insert(field.into(), Value::Null);
        equal_terminals(&before, &admitted(&changed));
    }
    for (field, value) in [
        ("summary", json!("Records ownership")),
        ("reached_by", json!("in_process")),
        ("reached_by", json!("network")),
        ("naming", json!({})),
        ("refs", json!([])),
        ("refs", json!(["local:records"])),
    ] {
        let mut changed = control.clone();
        object(&mut changed, "/components/0").insert(field.into(), value.clone());
        let raw = RawSpecFile::parse(&changed.to_string()).unwrap();
        if field == "summary" {
            assert_eq!(
                raw.components[0].summary.as_deref(),
                Some("Records ownership")
            );
        }
        assert_eq!(changed["components"][0][field], value);
        equal_terminals(&before, &admitted(&changed));
    }
    for field in ["owns", "accepts", "publishes"] {
        let mut omitted = control.clone();
        object(&mut omitted, "/components/0").remove(field);
        let mut empty = omitted.clone();
        object(&mut empty, "/components/0").insert(field.into(), json!({}));
        equal_terminals(&admitted(&omitted), &admitted(&empty));
        equal_terminals(&before, &admitted(&empty));
        let members: &[&str] = if field == "owns" {
            &["domains"]
        } else {
            &["commands", "events"]
        };
        for member in members {
            let mut explicit = empty.clone();
            object(&mut explicit, &format!("/components/0/{field}"))
                .insert((*member).into(), json!([]));
            let a = admitted(&empty);
            let b = admitted(&explicit);
            assert_eq!(a.ir.to_canonical_json(), b.ir.to_canonical_json());
            equal_terminals(&a, &b);
        }
    }
    for (field, value, expected) in [
        ("summary", json!({}), "expected a string"),
        ("naming", json!(false), "invalid type"),
        ("refs", json!({}), "expected a sequence"),
        ("refs", json!([{}]), "expected a string"),
        ("reached_by", json!("elsewhere"), "unknown variant"),
        ("reached_by", json!({}), "invalid type"),
    ] {
        let mut malformed = control.clone();
        object(&mut malformed, "/components/0").insert(field.into(), value);
        reader_refused(&malformed, expected, &control);
    }
}

#[test]
fn legacy_empty_defaults_and_optional_summary_are_observed_before_cli_execution() {
    let control = legacy(true);
    for (path, field, value) in [
        ("/components/0/cli", "commands", json!([])),
        ("/components/0/cli", "views", json!([])),
        ("/components/0/cli/groups/0", "summary", Value::Null),
    ] {
        let mut omitted = control.clone();
        object(&mut omitted, path).remove(field);
        let mut explicit = omitted.clone();
        object(&mut explicit, path).insert(field.into(), value);
        let a = admitted(&omitted);
        let b = admitted(&explicit);
        assert_eq!(a.ir.to_canonical_json(), b.ir.to_canonical_json());
        equal_terminals(&a, &b);
    }
    for field in ["commands", "views"] {
        let mut omitted = control.clone();
        let value = object(&mut omitted, "/components/0/cli/groups/0")
            .remove(field)
            .unwrap();
        object(&mut omitted, "/components/0/cli").insert(field.into(), value);
        let mut explicit = omitted.clone();
        object(&mut explicit, "/components/0/cli/groups/0").insert(field.into(), json!([]));
        let a = admitted(&omitted);
        let b = admitted(&explicit);
        assert_eq!(a.ir.to_canonical_json(), b.ir.to_canonical_json());
        equal_terminals(&a, &b);
    }
    let mut root = legacy(false);
    let a = admitted(&root);
    object(&mut root, "/components/0/cli").insert("groups".into(), json!([]));
    let b = admitted(&root);
    assert_eq!(a.ir.to_canonical_json(), b.ir.to_canonical_json());
    equal_terminals(&a, &b);
}

#[test]
fn view_names_sources_and_ranking_keys_are_real_model_changes() {
    let control = document();
    let before = admitted(&control);
    let renamed = set(
        &control,
        "/views/0/name",
        json!("surface.records.RenamedView"),
    );
    let renamed = admitted(&renamed);
    assert_ne!(
        before.ir.views().values().next().unwrap().name,
        renamed.ir.views().values().next().unwrap().name
    );
    assert_ne!(
        before
            .ir
            .views()
            .values()
            .next()
            .unwrap()
            .domain
            .to_string(),
        ""
    );
    equal_terminals(&before, &renamed);
    let mut sourced = control.clone();
    let mut entity = sourced["entities"][0].clone();
    entity["name"] = json!("surface.records.OtherRecord");
    sourced["entities"].as_array_mut().unwrap().push(entity);
    sourced["views"][0]["source"] = json!("surface.records.OtherRecord");
    let after = admitted(&sourced);
    assert_ne!(
        before.ir.views().values().next().unwrap().source,
        after.ir.views().values().next().unwrap().source
    );
    equal_terminals(&before, &after);
    // Ranking requires a projected key: both name and identity are String values.
    let mut ranked = control.clone();
    ranked["views"][0]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"record_id","type":"String"}));
    for key in ["name", "record_id"] {
        for direction in ["asc", "ascending", "desc", "descending"] {
            ranked["views"][0]["order_by"] = json!([format!("{key} {direction}")]);
            let raw = RawSpecFile::parse(&ranked.to_string()).unwrap();
            assert_eq!(raw.views[0].order_by[0].field, key);
            assert_eq!(
                raw.views[0].order_by[0].direction.as_str(),
                if direction.starts_with("asc") {
                    "asc"
                } else {
                    "desc"
                }
            );
            equal_terminals(&before, &admitted(&ranked));
        }
    }
}

#[test]
fn view_reader_refusals_have_a_successful_full_pipeline_control() {
    let control = document();
    for field in ["name", "source"] {
        let mut malformed = control.clone();
        object(&mut malformed, "/views/0").remove(field);
        reader_refused(&malformed, &format!("missing field `{field}`"), &control);
    }
    let mut malformed = control.clone();
    object(&mut malformed, "/views/0").insert("undeclared".into(), json!(true));
    reader_refused(&malformed, "unknown field `undeclared`", &control);
    for path in ["/views/0", "/views/0/fields/0", "/views/0/params/0"] {
        reader_refused(&set(&control, path, json!(false)), "invalid type", &control);
    }
    for path in ["/views/0/fields", "/views/0/params"] {
        reader_refused(
            &set(&control, path, json!({})),
            "expected a sequence",
            &control,
        );
    }
    for path in ["/views/0/name", "/views/0/source"] {
        reader_refused(
            &set(&control, path, json!({})),
            "expected a string",
            &control,
        );
    }
    for (field, value, expected) in [
        ("shape", json!({}), "expected a string"),
        ("filter", json!(42), "predicate"),
        ("naming", json!(false), "invalid type"),
        ("consistency", json!("instant"), "unknown variant"),
        ("consistency", json!({}), "invalid type"),
        ("order_by", json!({}), "expected a sequence"),
        (
            "order_by",
            json!([{"field":"name","direction":"ascending"}]),
            "expected a string",
        ),
        ("order_by", json!(["name sideways"]), "order_by direction"),
    ] {
        let mut malformed = control.clone();
        object(&mut malformed, "/views/0").insert(field.into(), value);
        reader_refused(&malformed, expected, &control);
    }
}

#[test]
fn view_option_defaults_and_named_shape_normalization_preserve_local_contract() {
    let mut control = document();
    object(&mut control, "/views/0").remove("filter");
    object(&mut control, "/views/0").remove("params");
    let before = admitted(&control);
    for (field, value) in [
        ("filter", Value::Null),
        ("shape", Value::Null),
        ("params", json!([])),
        ("order_by", json!([])),
        ("naming", json!({})),
    ] {
        let mut changed = control.clone();
        object(&mut changed, "/views/0").insert(field.into(), value);
        let after = admitted(&changed);
        assert_eq!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
        equal_terminals(&before, &after);
    }
    let mut omitted = control.clone();
    object(&mut omitted, "/views/0").remove("consistency");
    let after = admitted(&omitted);
    assert_eq!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    equal_terminals(&before, &after);
    let mut shaped = control.clone();
    object(&mut shaped, "/views/0").remove("fields");
    object(&mut shaped, "/views/0").insert("shape".into(), json!("surface.records.Row"));
    let a = admitted(&shaped);
    object(&mut shaped, "/views/0").insert("fields".into(), json!([]));
    let b = admitted(&shaped);
    assert_eq!(a.ir.to_canonical_json(), b.ir.to_canonical_json());
    equal_terminals(&a, &b);
    equal_terminals(&before, &a);
}

struct ServiceRecorder(Vec<Value>);
impl Handler for ServiceRecorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.0
            .push(json!({"target":invocation.target,"input":invocation.input}));
        HandlerReply::Success(if invocation.callable == "rename" {
            json!({"name":"Ada"})
        } else {
            json!([{"name":"Ada"}])
        })
    }
}

fn service_pipeline(input: &Value, stages: &mut Vec<&'static str>) -> Result<(), String> {
    stages.push("reader");
    let raw = RawSpecFile::parse(&input.to_string()).map_err(|error| error.to_string())?;
    stages.push("assembly");
    let spec = Specification::assemble(vec![(Source::new("service.json"), raw)])
        .map_err(|error| error.to_string())?;
    stages.push("compiler");
    let ir = ess_compiler::compile(&spec, &SourceMap::new()).map_err(|error| error.to_string())?;
    let declaration =
        include_str!("../../../specify/ess-cli-contract/tests/fixtures/service-cli.yaml")
            .replace("demo.records", "surface.records")
            .replace("NamedRecords", "Named")
            .replace("RenameInput", "Input")
            .replace("owner-service", "record-service");
    stages.push("binding");
    let compiled = ess_cli_contract::compile(&ir, &Binding::from_yaml(&declaration).unwrap())
        .map_err(|error| error.to_string())?;
    stages.push("emission");
    let artifacts = ess_cli_project::project(&compiled);
    assert_eq!(artifacts["binding.json"], compiled.to_canonical_json());
    assert!(artifacts["README.md"].contains("records find"));
    stages.push("execution");
    for (args, expected_input, expected_result) in [
        (
            vec!["records", "rename", "--name", "Ada", "--output=json"],
            json!({"name":"Ada"}),
            json!({"name":"Ada"}),
        ),
        (
            vec![
                "records",
                "records",
                "find",
                "--prefix",
                "Ada",
                "--output=json",
            ],
            json!({"prefix":"Ada"}),
            json!([{"name":"Ada"}]),
        ),
    ] {
        let mut handler = ServiceRecorder(Vec::new());
        let output = runtime::run(
            compiled.plan(),
            args.into_iter().map(Into::into).collect(),
            &mut NoSources,
            &mut handler,
            None,
        );
        assert_eq!(output.exit_code, 0, "{output:?}");
        assert!(output.stderr.is_empty());
        assert_eq!(
            serde_json::from_str::<Value>(&output.stdout).unwrap(),
            json!({"ok":true,"result":expected_result})
        );
        assert_eq!(handler.0.len(), 1);
        assert_eq!(handler.0[0]["input"], expected_input);
        assert_eq!(handler.0[0]["target"]["owner"], "record-service");
    }
    Ok(())
}

#[test]
fn service_owner_acceptance_and_view_ownership_refuse_at_binding_before_emission_or_dispatch() {
    let mut control = document();
    control["types"].as_array_mut().unwrap().push(json!({
        "name":"surface.records.QueryInput","kind":"struct",
        "fields":[{"name":"prefix","type":"String"}]
    }));
    let mut unaccepted = control.clone();
    assert!(object(&mut unaccepted, "/components/0")
        .remove("accepts")
        .is_some());
    let mut unowned = control.clone();
    assert!(object(&mut unowned, "/components/0")
        .remove("owns")
        .is_some());
    for (malformed, expected) in [
        (
            set(&control, "/components/0/name", json!("other-service")),
            "unresolved service owner `record-service`",
        ),
        (
            unaccepted,
            "service `record-service` does not accept `surface.records.Rename`",
        ),
        (unowned, "service does not own view domain"),
    ] {
        let mut stages = Vec::new();
        let error = service_pipeline(&malformed, &mut stages).unwrap_err();
        assert_eq!(error, expected);
        assert_eq!(stages, ["reader", "assembly", "compiler", "binding"]);
        let mut control_stages = Vec::new();
        service_pipeline(&control, &mut control_stages).unwrap();
        assert_eq!(
            control_stages,
            [
                "reader",
                "assembly",
                "compiler",
                "binding",
                "emission",
                "execution"
            ]
        );
    }
}
