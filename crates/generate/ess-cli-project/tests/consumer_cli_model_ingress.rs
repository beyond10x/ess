//! Authored model admission, entity structure and graph identities through CLI pipelines.
//! The fixture is JSON source read by `RawSpecFile`'s real YAML-compatible reader.
//! No private IR is constructed, no business command executes, and no subprocess runs.

use std::collections::{BTreeMap, BTreeSet};

use ess_cli_contract::{Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
};
use ess_compiler::{
    graph::{DependencyRelation, SemanticDependencyGraph},
    refs::EssSemanticRef,
    source::SourceMap,
    EssIr,
};
use ess_domain::{
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/model-ingress.json");
const CLI: &str = r"
format: ess-cli/1
binary: model-ingress
about: Observe a separately selected CLI value
globals: {config: config, state: state-dir, output: output}
callables:
  observe:
    target: {kind: local, owner: model.cli, action: observe}
    input: model.cli.Input
    result: Integer
commands:
  - path: [observe]
    callable: observe
    about: Observe a typed value
    arguments:
      - {field: value, source: {kind: option, long: value}}
";

fn fixture() -> Vec<Value> {
    serde_json::from_str(FIXTURE).unwrap()
}

fn name(text: &str) -> QualifiedName {
    QualifiedName::new(text).unwrap()
}

fn replace(input: &mut [Value], document: usize, pointer: &str, value: Value) {
    *input[document]
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("fixture has {document}:{pointer}")) = value;
}

fn remove(input: &mut [Value], document: usize, pointer: &str) {
    let (parent, field) = pointer.rsplit_once('/').unwrap();
    assert!(input[document]
        .pointer_mut(parent)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove(field)
        .is_some());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Reader,
    Assembly,
    Compiler,
    Binding,
    Projection,
    Runtime,
}

#[derive(Debug)]
struct Refusal {
    stage: Stage,
    message: String,
    trace: Vec<Stage>,
}

#[derive(Debug)]
struct Model {
    raw: Vec<RawSpecFile>,
    spec: Specification,
    ir: EssIr,
    trace: Vec<Stage>,
}

fn admit(input: &[Value]) -> Result<Model, Refusal> {
    let mut trace = vec![Stage::Reader];
    let mut raw = Vec::new();
    let mut parsed = Vec::new();
    let mut sources = SourceMap::new();
    for (index, value) in input.iter().enumerate() {
        let label = format!("model-ingress-{index}.json");
        let text = serde_json::to_string(value).unwrap();
        let file = RawSpecFile::parse(&text).map_err(|error| Refusal {
            stage: Stage::Reader,
            message: error.to_string(),
            trace: trace.clone(),
        })?;
        raw.push(file.clone());
        parsed.push((Source::new(&label), file));
        sources.insert(label, text);
    }
    trace.push(Stage::Assembly);
    let spec = Specification::assemble(parsed).map_err(|error| Refusal {
        stage: Stage::Assembly,
        message: error.to_string(),
        trace: trace.clone(),
    })?;
    trace.push(Stage::Compiler);
    let ir = ess_compiler::compile(&spec, &sources).map_err(|error| Refusal {
        stage: Stage::Compiler,
        message: error.to_string(),
        trace: trace.clone(),
    })?;
    Ok(Model {
        raw,
        spec,
        ir,
        trace,
    })
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("model witnesses never acquire credentials")
    }
}

#[derive(Default)]
struct Recorder {
    calls: Vec<Value>,
}
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        assert!(invocation.context.config.is_none());
        assert!(invocation.context.state_dir.is_none());
        self.calls.push(json!({
            "callable": invocation.callable,
            "target": invocation.target,
            "input": invocation.input,
        }));
        HandlerReply::Success(invocation.input["value"].clone())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Terminal {
    plan: String,
    files: BTreeMap<String, String>,
    exit: i32,
    stdout: String,
    stderr: String,
    calls: Vec<Value>,
    trace: Vec<Stage>,
}

fn terminal(model: &Model) -> Terminal {
    terminal_binding(model, CLI)
}

fn terminal_binding(model: &Model, cli: &str) -> Terminal {
    let mut trace = model.trace.clone();
    trace.push(Stage::Binding);
    let compiled: CompiledBinding =
        ess_cli_contract::compile(&model.ir, &Binding::from_yaml(cli).unwrap()).unwrap();
    let plan = compiled.to_canonical_json();
    trace.push(Stage::Projection);
    let files = ess_cli_project::project(&compiled);
    assert!(!files.is_empty());
    assert_eq!(files["binding.json"], plan);
    trace.push(Stage::Runtime);
    let mut recorder = Recorder::default();
    let output = runtime::run(
        compiled.plan(),
        ["model-ingress", "observe", "--value", "17", "--output=json"]
            .into_iter()
            .map(Into::into)
            .collect(),
        &mut NoSources,
        &mut recorder,
        None,
    );
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout, "{\"ok\":true,\"result\":17}\n");
    assert!(output.stderr.is_empty());
    assert_eq!(
        recorder.calls,
        [json!({
            "callable":"observe",
            "target":{"kind":"local","owner":"model.cli","action":"observe"},
            "input":{"value":17},
        })]
    );
    assert_eq!(
        trace,
        [
            Stage::Reader,
            Stage::Assembly,
            Stage::Compiler,
            Stage::Binding,
            Stage::Projection,
            Stage::Runtime,
        ]
    );
    Terminal {
        plan,
        files,
        exit: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
        calls: recorder.calls,
        trace,
    }
}

fn admitted(input: &[Value]) -> Model {
    admit(input).unwrap_or_else(|error| panic!("{:?}: {}", error.stage, error.message))
}

fn refused(input: &[Value], stage: Stage, diagnostic: &[&str]) {
    let error = admit(input).expect_err("the malformed source must stop the pipeline");
    assert_eq!(error.stage, stage, "{}", error.message);
    for fragment in diagnostic {
        assert!(
            error.message.contains(fragment),
            "expected {fragment:?} at {stage:?}: {}",
            error.message
        );
    }
    assert_eq!(
        error.trace,
        match stage {
            Stage::Reader => vec![Stage::Reader],
            Stage::Assembly => vec![Stage::Reader, Stage::Assembly],
            Stage::Compiler => vec![Stage::Reader, Stage::Assembly, Stage::Compiler],
            _ => panic!("this witness's refusal is model ingress"),
        }
    );
    // The same fixture without the exact malformed edit reaches all terminal stages.
    terminal(&admitted(&fixture()));
}

fn unchanged(control: &Model, changed: &Model, equal_ir: bool) {
    assert_eq!(
        control.ir.to_canonical_json() == changed.ir.to_canonical_json(),
        equal_ir,
        "assert whether this source change is preserved or deliberately normalized"
    );
    assert_eq!(terminal(control), terminal(changed));
}

fn entity(model: &Model, index: usize) -> Value {
    let identity = &model.raw[1].entities[index].name;
    serde_json::to_value(&model.ir.entities()[identity]).unwrap()
}

#[test]
fn entity_reader_requires_identity_lifecycle_and_name_and_rejects_extra_members() {
    for field in ["name", "identity", "lifecycle"] {
        let mut input = fixture();
        remove(&mut input, 1, &format!("/entities/0/{field}"));
        refused(&input, Stage::Reader, &["missing field", field]);
    }
    let mut input = fixture();
    input[1]["entities"][0]["extra"] = json!(1);
    refused(&input, Stage::Reader, &["unknown field", "extra"]);
    replace(&mut input, 1, "/entities/0", json!(17));
    refused(&input, Stage::Reader, &["RawEntitySpec"]);
}

#[test]
fn entity_optional_fields_relations_and_naming_have_explicit_default_controls() {
    let control = admitted(&fixture());
    for (field, value) in [
        ("fields", json!([])),
        ("relations", json!([])),
        ("naming", json!({})),
    ] {
        let mut absent = fixture();
        // Other is unrelated to the selected CLI and is not the relation target.
        absent[1]["entities"][2]
            .as_object_mut()
            .unwrap()
            .remove(field);
        let mut explicit = absent.clone();
        explicit[1]["entities"][2][field] = value;
        assert_ne!(absent, explicit);
        let a = admitted(&absent);
        let b = admitted(&explicit);
        assert_eq!(entity(&a, 2), entity(&b, 2));
        unchanged(&a, &b, true);
        if field == "fields" {
            assert_ne!(entity(&control, 2)["fields"], entity(&a, 2)["fields"]);
            unchanged(&control, &a, false);
        }
    }
}

#[test]
fn entity_identity_fields_naming_and_invariants_change_the_model_only() {
    let control = admitted(&fixture());
    for (pointer, value, member, index) in [
        (
            "/entities/2/identity/name",
            json!("other_key"),
            "identity",
            2,
        ),
        ("/entities/2/identity/type", json!("Integer"), "identity", 2),
        ("/entities/2/fields/0/type", json!("Integer"), "fields", 2),
        (
            "/entities/0/invariants/0",
            json!("amount >= 1"),
            "invariants",
            0,
        ),
    ] {
        let mut input = fixture();
        replace(&mut input, 1, pointer, value);
        let changed = admitted(&input);
        assert_ne!(
            entity(&control, index)[member],
            entity(&changed, index)[member]
        );
        unchanged(&control, &changed, false);
    }
    let mut input = fixture();
    input[1]["entities"][0]["naming"] = json!({"wire":"parents","display":"Parent records"});
    let changed = admitted(&input);
    assert_eq!(entity(&changed, 0)["naming"]["wire"], "parents");
    assert_ne!(entity(&control, 0)["naming"], entity(&changed, 0)["naming"]);
    unchanged(&control, &changed, false);
    let old = &control.spec.entities()[&name("model.data.Parent")].invariants[0];
    let new = &changed.spec.entities()[&name("model.data.Parent")].invariants[0];
    assert_eq!(old.statement, "amount >= 0");
    assert_eq!(old.predicate, new.predicate);
    let mut invariant_input = fixture();
    replace(
        &mut invariant_input,
        1,
        "/entities/0/invariants/0",
        json!("amount >= 1"),
    );
    let other = admitted(&invariant_input);
    let new = &other.spec.entities()[&name("model.data.Parent")].invariants[0];
    assert_eq!(new.statement, "amount >= 1");
    assert_ne!(old.predicate, new.predicate);
}

#[test]
fn relation_kind_cardinality_target_carrier_and_name_have_concrete_controls() {
    let control = admitted(&fixture());
    for (member, value) in [
        ("kind", json!("references")),
        ("cardinality", json!("one")),
        ("target", json!("model.data.Other")),
        ("via", json!("alternate_link")),
        ("name", json!("descendants")),
    ] {
        let mut input = fixture();
        replace(
            &mut input,
            1,
            &format!("/entities/0/relations/0/{member}"),
            value.clone(),
        );
        let changed = admitted(&input);
        assert_eq!(entity(&changed, 0)["relations"][0][member], value);
        assert_ne!(
            entity(&control, 0)["relations"][0][member],
            entity(&changed, 0)["relations"][0][member]
        );
        unchanged(&control, &changed, false);
    }
}

#[test]
fn relation_reader_refuses_missing_fields_unknown_members_and_unknown_alternatives() {
    for field in ["name", "kind", "target", "cardinality", "via"] {
        let mut input = fixture();
        remove(&mut input, 1, &format!("/entities/0/relations/0/{field}"));
        refused(&input, Stage::Reader, &["missing field", field]);
    }
    for field in ["kind", "cardinality"] {
        let mut input = fixture();
        replace(
            &mut input,
            1,
            &format!("/entities/0/relations/0/{field}"),
            json!("unrecognized"),
        );
        refused(&input, Stage::Reader, &["unknown variant", "unrecognized"]);
    }
    for field in ["name", "via"] {
        let mut input = fixture();
        replace(
            &mut input,
            1,
            &format!("/entities/0/relations/0/{field}"),
            json!("bad.field"),
        );
        refused(&input, Stage::Reader, &["bad.field"]);
    }
    let mut input = fixture();
    input[1]["entities"][0]["relations"][0]["extra"] = json!(1);
    refused(&input, Stage::Reader, &["unknown field", "extra"]);
}

#[test]
fn lifecycle_and_transition_readers_are_closed_and_require_the_declared_members() {
    for field in ["initial", "states"] {
        let mut input = fixture();
        remove(&mut input, 1, &format!("/entities/0/lifecycle/{field}"));
        refused(&input, Stage::Reader, &["missing field", field]);
    }
    for field in ["name", "from", "to"] {
        let mut input = fixture();
        remove(
            &mut input,
            1,
            &format!("/entities/0/lifecycle/transitions/0/{field}"),
        );
        refused(&input, Stage::Reader, &["missing field", field]);
    }
    for pointer in [
        "/entities/0/lifecycle",
        "/entities/0/lifecycle/transitions/0",
    ] {
        let mut input = fixture();
        input[1].pointer_mut(pointer).unwrap()["extra"] = json!(1);
        refused(&input, Stage::Reader, &["unknown field", "extra"]);
    }
    for pointer in [
        "/entities/0/lifecycle/initial",
        "/entities/0/lifecycle/states/0",
        "/entities/0/lifecycle/terminal/0",
        "/entities/0/lifecycle/transitions/0/from/0",
        "/entities/0/lifecycle/transitions/0/to",
    ] {
        let mut input = fixture();
        replace(&mut input, 1, pointer, json!("bad_state"));
        refused(&input, Stage::Reader, &["state name", "bad_state"]);
    }
    let mut input = fixture();
    replace(
        &mut input,
        1,
        "/entities/0/lifecycle/transitions/0/name",
        json!("bad.transition"),
    );
    refused(&input, Stage::Reader, &["single segment"]);
}

#[test]
fn lifecycle_set_duplicates_normalize_at_the_reader_without_claiming_schema_rejection() {
    let control = admitted(&fixture());
    for (pointer, value) in [
        (
            "/entities/0/lifecycle/states",
            json!(["Draft", "Draft", "Done"]),
        ),
        ("/entities/0/lifecycle/terminal", json!(["Done", "Done"])),
        (
            "/entities/0/lifecycle/transitions/0/from",
            json!(["Draft", "Draft"]),
        ),
    ] {
        let mut input = fixture();
        replace(&mut input, 1, pointer, value);
        assert_ne!(fixture()[1].pointer(pointer), input[1].pointer(pointer));
        let changed = admitted(&input);
        assert_eq!(
            control.raw[1].entities[0].states.states,
            changed.raw[1].entities[0].states.states
        );
        assert_eq!(
            control.raw[1].entities[0].states.terminal,
            changed.raw[1].entities[0].states.terminal
        );
        assert_eq!(
            control.raw[1].entities[0].states.transitions,
            changed.raw[1].entities[0].states.transitions
        );
        unchanged(&control, &changed, true);
    }
}

#[test]
fn lifecycle_state_and_transition_renaming_changes_declared_and_resolved_values() {
    let control = admitted(&fixture());
    for (old, new) in [
        ("Draft", "Open"),
        ("Done", "Closed"),
        ("finish", "complete"),
    ] {
        let input: Vec<Value> =
            serde_json::from_str(&serde_json::to_string(&fixture()).unwrap().replace(old, new))
                .unwrap();
        let changed = admitted(&input);
        assert_ne!(
            entity(&control, 0)["lifecycle"],
            entity(&changed, 0)["lifecycle"]
        );
        let a = &control.raw[1].entities[0].states;
        let b = &changed.raw[1].entities[0].states;
        assert!(
            a.initial != b.initial
                || a.states != b.states
                || a.terminal != b.terminal
                || a.transitions != b.transitions
        );
        unchanged(&control, &changed, false);
    }
}

#[test]
fn lifecycle_terminal_and_transition_defaults_remain_semantic_refusals_when_needed() {
    for field in ["terminal", "transitions"] {
        let mut absent = fixture();
        remove(&mut absent, 1, &format!("/entities/0/lifecycle/{field}"));
        let mut empty = absent.clone();
        empty[1]["entities"][0]["lifecycle"][field] = json!([]);
        assert_ne!(absent, empty);
        let first = admit(&absent).unwrap_err();
        let second = admit(&empty).unwrap_err();
        assert_eq!(first.stage, Stage::Assembly);
        assert_eq!(second.stage, Stage::Assembly);
        assert_eq!(first.message, second.message);
        assert!(
            first.message.contains("terminal") || first.message.contains("dead_end_state"),
            "{}",
            first.message
        );
        let state = if field == "terminal" { "Done" } else { "Draft" };
        refused(&absent, Stage::Assembly, &["dead_end_state", state]);
        refused(&empty, Stage::Assembly, &["dead_end_state", state]);
    }
}

#[test]
fn root_reader_refuses_unknown_fields_and_wrong_collection_shapes_before_assembly() {
    let mut input = fixture();
    input[0]["unrecognized"] = json!(true);
    refused(&input, Stage::Reader, &["unknown field", "unrecognized"]);
    for field in [
        "types",
        "entities",
        "commands",
        "events",
        "errors",
        "views",
        "actors",
        "components",
        "bindings",
        "conversions",
        "domains",
    ] {
        let mut input = fixture();
        input[0][field] = json!({"unexpected":"object"});
        refused(&input, Stage::Reader, &["sequence"]);
    }
    let mut input = fixture();
    input[0] = json!(17);
    refused(&input, Stage::Reader, &["RawSpecFile"]);
}

#[test]
fn root_collection_empty_defaults_normalize_to_the_same_model_and_terminals() {
    let control = admitted(&fixture());
    for field in [
        "types",
        "entities",
        "commands",
        "events",
        "errors",
        "views",
        "actors",
        "components",
        "bindings",
        "conversions",
        "domains",
    ] {
        let mut omitted = fixture();
        // This extra source declares no domain or header and holds no members.
        omitted.push(json!({}));
        let mut explicit = omitted.clone();
        explicit[3][field] = json!([]);
        assert_ne!(omitted, explicit);
        let a = admitted(&omitted);
        let b = admitted(&explicit);
        unchanged(&a, &b, true);
        unchanged(&control, &a, true);
    }
}

#[test]
fn root_format_and_version_omission_and_null_use_declared_defaults() {
    for field in ["format", "version"] {
        let mut omitted = fixture();
        remove(&mut omitted, 0, &format!("/{field}"));
        let mut null = omitted.clone();
        null[0][field] = Value::Null;
        let explicit = fixture();
        assert_ne!(omitted, null);
        let a = admitted(&omitted);
        let b = admitted(&null);
        let c = admitted(&explicit);
        assert!(a.raw[0].format.is_none() || field == "version");
        assert!(a.raw[0].version.is_none() || field == "format");
        unchanged(&a, &b, true);
        unchanged(&a, &c, true);
    }
}

#[test]
fn malformed_format_spelling_and_unsupported_major_are_different_admission_stages() {
    for spelling in ["ess/0", "ess/01", "other/1"] {
        let mut input = fixture();
        replace(&mut input, 0, "/format", json!(spelling));
        refused(&input, Stage::Reader, &["format"]);
    }
    let mut input = fixture();
    replace(&mut input, 0, "/format", json!("ess/2"));
    let raw = RawSpecFile::parse(&serde_json::to_string(&input[0]).unwrap()).unwrap();
    assert_eq!(raw.format.unwrap().to_string(), "ess/2");
    let supported = admitted(&input);
    assert_eq!(supported.spec.system().format.major(), 2);
    unchanged(&admitted(&fixture()), &supported, true);
    replace(&mut input, 0, "/format", json!("ess/5"));
    let raw = RawSpecFile::parse(&serde_json::to_string(&input[0]).unwrap()).unwrap();
    assert_eq!(raw.format.unwrap().to_string(), "ess/5");
    refused(
        &input,
        Stage::Assembly,
        &["unsupported_format_version", "ess/5"],
    );
}

#[test]
fn version_numeric_and_written_forms_normalize_and_out_of_range_values_stop_at_reader() {
    let control = admitted(&fixture());
    for major in [1_u32, 2, u32::MAX] {
        let mut written = fixture();
        written[0]["version"] = json!(format!("v{major}"));
        let mut numeric = written.clone();
        numeric[0]["version"] = json!(major);
        let a = admitted(&written);
        let b = admitted(&numeric);
        assert_eq!(a.raw[0].version.unwrap().get(), major);
        assert_eq!(b.raw[0].version.unwrap().get(), major);
        assert_eq!(a.ir.version().get(), major);
        unchanged(&a, &b, true);
        unchanged(&control, &a, major == 1);
    }
    for value in [
        json!(0),
        json!(-1),
        json!(4_294_967_296_u64),
        json!(1.5),
        json!("v01"),
    ] {
        let mut input = fixture();
        input[0]["version"] = value;
        refused(&input, Stage::Reader, &["version"]);
    }
}

#[test]
fn root_and_domain_names_and_roster_have_named_admission_refusals() {
    for (document, field) in [(0, "system"), (1, "domain")] {
        let mut input = fixture();
        input[document][field] = json!("bad..name");
        refused(&input, Stage::Reader, &["empty segment"]);
    }
    for field in ["system", "domain"] {
        let document = usize::from(field == "domain");
        let mut input = fixture();
        input[document][field] = Value::Null;
        let error = admit(&input).unwrap_err();
        assert_eq!(error.stage, Stage::Assembly);
        assert!(!error.message.is_empty());
        assert!(!error.trace.contains(&Stage::Compiler));
        terminal(&admitted(&fixture()));
    }
    let mut input = fixture();
    input[0]["domains"] = json!(["model.data", "model.cli", "model.missing"]);
    refused(&input, Stage::Assembly, &["model.missing"]);
}

#[test]
fn root_summary_null_and_omission_normalize_while_authored_summary_changes_ir() {
    let mut absent = fixture();
    remove(&mut absent, 0, "/summary");
    let mut null = absent.clone();
    null[0]["summary"] = Value::Null;
    let a = admitted(&absent);
    let b = admitted(&null);
    assert_eq!(a.raw[0].summary, None);
    assert_eq!(b.raw[0].summary, None);
    unchanged(&a, &b, true);
    let original = admitted(&fixture());
    assert_ne!(a.ir.summary(), original.ir.summary());
    unchanged(&a, &original, false);
}

#[test]
fn domain_naming_and_header_naming_are_separate_owned_inputs() {
    let control = admitted(&fixture());
    let mut input = fixture();
    input[1]["naming"] = json!({"wire":"renamed-data","display":"Renamed data"});
    let changed = admitted(&input);
    assert_ne!(
        control.ir.domains()[&name("model.data")].naming,
        changed.ir.domains()[&name("model.data")].naming
    );
    unchanged(&control, &changed, false);
    let mut header = fixture();
    header[0]["naming"] = json!({"wire":"unused-header-name","display":"Header"});
    let changed = admitted(&header);
    assert_ne!(control.raw[0].naming, changed.raw[0].naming);
    assert_eq!(control.spec.system().naming, changed.spec.system().naming);
    assert_eq!(control.ir.naming(), changed.ir.naming());
    unchanged(&control, &changed, true);
}

#[test]
fn empty_optional_topology_and_domain_source_shapes_normalize_without_invented_members() {
    for (field, explicit) in [("domain", Value::Null), ("system", Value::Null)] {
        let mut absent = fixture();
        absent.push(json!({}));
        let mut changed = absent.clone();
        changed[3][field] = explicit;
        let a = admitted(&absent);
        let b = admitted(&changed);
        assert_ne!(absent, changed);
        unchanged(&a, &b, true);
    }
    let mut null_topology = fixture();
    null_topology.push(json!({"topology":null}));
    unchanged(&admitted(&fixture()), &admitted(&null_topology), true);
    let mut absent = fixture();
    remove(&mut absent, 0, "/topology");
    let mut empty = absent.clone();
    empty[0]["topology"] = json!({"workloads":{}});
    let mut null = absent.clone();
    null[0]["topology"] = Value::Null;
    let a = admitted(&absent);
    let b = admitted(&empty);
    let c = admitted(&null);
    assert!(a.raw[0].topology.is_none());
    assert!(b.raw[0].topology.as_ref().unwrap().workloads.is_empty());
    assert!(c.raw[0].topology.is_none());
    assert!(a.ir.workloads().is_empty());
    unchanged(&a, &b, true);
    unchanged(&a, &c, true);
    unchanged(&admitted(&fixture()), &a, false);
}

#[test]
fn external_reference_spelling_preserves_provider_key_and_order_with_real_reader_refusals() {
    let control = admitted(&fixture());
    for refs in [
        json!(["other:initial"]),
        json!(["fixture:replacement"]),
        json!(["fixture:initial", "other:second"]),
        json!(["other:second", "fixture:initial"]),
    ] {
        let mut input = fixture();
        input[0]["components"][0]["refs"] = refs;
        let changed = admitted(&input);
        let old = &control.raw[0].components[0].refs;
        let new = &changed.raw[0].components[0].refs;
        assert_ne!(old, new);
        assert_eq!(
            serde_json::to_value(new).unwrap(),
            input[0]["components"][0]["refs"]
        );
        assert_eq!(
            serde_json::to_value(&changed.ir.components().values().next().unwrap().refs).unwrap(),
            input[0]["components"][0]["refs"]
        );
        unchanged(&control, &changed, false);
    }
    for (reference, diagnostic) in [
        (json!("missing-colon"), "provider:key"),
        (json!("UPPER:key"), "provider containing"),
        (json!("fixture:"), "empty provider or key"),
        (
            json!({"provider":"fixture","reference":"key"}),
            "expected a string",
        ),
    ] {
        let mut input = fixture();
        input[0]["components"][0]["refs"] = json!([reference]);
        let error = admit(&input).unwrap_err();
        assert_eq!(error.stage, Stage::Reader);
        assert!(error.message.contains(diagnostic), "{}", error.message);
        assert_eq!(error.trace, [Stage::Reader]);
        terminal(&control);
    }
}

#[test]
fn a_view_and_entity_can_move_between_two_admitted_domains_without_changing_the_cli() {
    let mut input = fixture();
    input[0]["domains"]
        .as_array_mut()
        .unwrap()
        .push(json!("model.extra"));
    input.push(json!({"domain":"model.extra"}));
    let control = admitted(&input);
    assert!(control.ir.domains().contains_key(&name("model.data")));
    assert!(control.ir.domains().contains_key(&name("model.extra")));

    let mut view_input = input.clone();
    let mut moved = view_input[1]["views"].as_array_mut().unwrap().remove(0);
    moved["name"] = json!("model.extra.Rows");
    view_input[3]["views"] = json!([moved]);
    let changed = admitted(&view_input);
    let before = &control.ir.views()[&name("model.data.Rows")];
    let after = &changed.ir.views()[&name("model.extra.Rows")];
    assert_eq!(before.domain.name().to_string(), "model.data");
    assert_eq!(after.domain.name().to_string(), "model.extra");
    assert_eq!(before.source.name(), after.source.name());
    assert_eq!(before.fields, after.fields);
    assert_eq!(before.params, after.params);
    assert!(!changed.ir.domains()[&name("model.data")]
        .views
        .iter()
        .any(|v| v.name() == &after.name));
    assert!(changed.ir.domains()[&name("model.extra")]
        .views
        .iter()
        .any(|v| v.name() == &after.name));
    unchanged(&control, &changed, false);

    let mut entity_input = input;
    let mut moved = entity_input[1]["entities"]
        .as_array_mut()
        .unwrap()
        .remove(1);
    moved["name"] = json!("model.extra.Child");
    entity_input[3]["entities"] = json!([moved]);
    entity_input[1]["entities"][0]["relations"][0]["target"] = json!("model.extra.Child");
    let changed = admitted(&entity_input);
    let before = &control.ir.entities()[&name("model.data.Child")];
    let after = &changed.ir.entities()[&name("model.extra.Child")];
    assert_eq!(before.domain.name().to_string(), "model.data");
    assert_eq!(after.domain.name().to_string(), "model.extra");
    assert_eq!(before.identity, after.identity);
    assert_eq!(before.fields, after.fields);
    assert_eq!(before.lifecycle, after.lifecycle);
    assert_ne!(before.state_type.name(), after.state_type.name());
    assert_eq!(
        changed.ir.named_type(&after.state_type).name.to_string(),
        "model.extra.Child.State"
    );
    unchanged(&control, &changed, false);
}

fn renamed_fixture() -> Vec<Value> {
    serde_json::from_str(
        &serde_json::to_string(&fixture())
            .unwrap()
            .replace("model.data", "model.renamed")
            .replace("data-service", "renamed-service")
            .replace("data-react", "renamed-react")
            .replace("finished", "completed")
            .replace("finish", "complete"),
    )
    .unwrap()
}

#[test]
fn a_system_namespace_change_requires_explicit_rebinding_and_preserves_typed_dispatch() {
    let control = admitted(&fixture());
    let encoded = serde_json::to_string(&fixture())
        .unwrap()
        .replace("model.", "other.")
        .replace("\"system\":\"model\"", "\"system\":\"other\"");
    let input: Vec<Value> = serde_json::from_str(&encoded).unwrap();
    let changed = admitted(&input);
    assert_eq!(control.raw[0].system.as_ref().unwrap().to_string(), "model");
    assert_eq!(changed.raw[0].system.as_ref().unwrap().to_string(), "other");
    assert_eq!(control.spec.system().name.to_string(), "model");
    assert_eq!(changed.spec.system().name.to_string(), "other");
    assert_eq!(control.ir.system().to_string(), "model");
    assert_eq!(changed.ir.system().to_string(), "other");
    let refusal =
        ess_cli_contract::compile(&changed.ir, &Binding::from_yaml(CLI).unwrap()).unwrap_err();
    assert_eq!(
        refusal.to_string(),
        "unresolved input type `model.cli.Input`"
    );
    let before = terminal(&control);
    let after = terminal_binding(&changed, &CLI.replace("model.cli.Input", "other.cli.Input"));
    assert_eq!(
        before.plan.replace("model.cli.Input", "other.cli.Input"),
        after.plan
    );
    assert_ne!(before.files, after.files);
    assert_eq!(before.exit, after.exit);
    assert_eq!(before.stdout, after.stdout);
    assert_eq!(before.stderr, after.stderr);
    assert_eq!(before.calls, after.calls);
    assert_eq!(before.trace, after.trace);
}

#[test]
fn conversion_membership_direction_and_reason_survive_model_assembly_without_cli_execution() {
    let control = admitted(&fixture());
    for conversion in [
        json!({"from":"model.data.Tag","to":"model.data.Alias","because":"Explicit model conversion."}),
        json!({"from":"model.data.Alias","to":"model.data.Tag","because":"Explicit reverse conversion."}),
    ] {
        let mut input = fixture();
        input[0]["conversions"] = json!([conversion]);
        let changed = admitted(&input);
        assert!(control.raw[0].conversions.is_empty());
        assert_eq!(changed.raw[0].conversions.len(), 1);
        assert_eq!(
            serde_json::to_value(&changed.raw[0].conversions).unwrap(),
            input[0]["conversions"]
        );
        assert!(control.ir.conversions().is_empty());
        assert_eq!(changed.ir.conversions().len(), 1);
        assert_eq!(
            changed.ir.conversions()[0].from.to_string(),
            input[0]["conversions"][0]["from"].as_str().unwrap()
        );
        assert_eq!(
            changed.ir.conversions()[0].to.to_string(),
            input[0]["conversions"][0]["to"].as_str().unwrap()
        );
        assert_eq!(
            changed.ir.conversions()[0].because,
            input[0]["conversions"][0]["because"]
        );
        assert_ne!(
            serde_json::to_value(&control.spec).unwrap()["conversions"],
            serde_json::to_value(&changed.spec).unwrap()["conversions"]
        );
        unchanged(&control, &changed, false);
    }
}

fn renamed_reference(reference: &EssSemanticRef) -> EssSemanticRef {
    serde_json::from_str(
        &serde_json::to_string(reference)
            .unwrap()
            .replace("model.data", "model.renamed")
            .replace("data-service", "renamed-service")
            .replace("data-react", "renamed-react")
            .replace("finished", "completed")
            .replace("finish", "complete"),
    )
    .unwrap()
}

#[test]
fn renamed_source_graph_has_every_concrete_relation_and_changed_semantic_reference_kind() {
    let control = admitted(&fixture());
    let changed = admitted(&renamed_fixture());
    let before = SemanticDependencyGraph::of(&control.ir);
    let after = SemanticDependencyGraph::of(&changed.ir);
    assert_ne!(before, after);
    let mut relations: BTreeSet<_> = before.edges().map(|edge| edge.relation).collect();
    // The separate periodic source exercises its additional host relation; the legacy rename
    // assertions below retain their original complete event-model witness.
    let text = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml");
    let specification = Specification::assemble([(
        Source::new("periodic.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = ess_compiler::source::SourceMap::new();
    sources.insert("periodic.yaml", text);
    let ir = ess_compiler::resolve::compile_locating(&specification, &sources, &["periodic.yaml"])
        .unwrap();
    relations.extend(
        SemanticDependencyGraph::of(&ir)
            .edges()
            .map(|edge| edge.relation),
    );
    assert_eq!(relations, DependencyRelation::ALL.into_iter().collect());
    assert_eq!(
        before
            .nodes()
            .iter()
            .map(renamed_reference)
            .collect::<BTreeSet<_>>(),
        *after.nodes()
    );
    let mut kinds = BTreeSet::new();
    for node in before.nodes() {
        let renamed = renamed_reference(node);
        assert!(control.ir.resolves(node));
        assert!(changed.ir.resolves(&renamed));
        if renamed != *node {
            assert!(!changed.ir.resolves(node));
            assert!(!control.ir.resolves(&renamed));
            kinds.insert(
                serde_json::to_value(node).unwrap()["kind"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            );
        }
    }
    assert_eq!(
        kinds,
        [
            "domain",
            "type",
            "entity",
            "command",
            "outcome",
            "event",
            "error",
            "view",
            "actor",
            "transition",
            "binding",
            "component"
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    );
    for edge in before.edges() {
        let mut expected = edge.clone();
        expected.dependent = renamed_reference(&edge.dependent);
        expected.dependency = renamed_reference(&edge.dependency);
        assert!(after.edges().any(|actual| *actual == expected), "{edge:?}");
        assert!(after
            .dependents_of(&expected.dependency)
            .any(|actual| *actual == expected));
        assert!(after.nodes().contains(&expected.dependency));
        assert!(after.nodes().contains(&expected.dependent));
    }
    assert_eq!(before.edges().count(), after.edges().count());
    unchanged(&control, &changed, false);
}

fn assert_raw_assembly_member_renaming(control: &Model, changed: &Model) {
    assert_ne!(control.raw[0].domains, changed.raw[0].domains);
    assert_ne!(control.raw[1].domain, changed.raw[1].domain);
    assert_ne!(control.raw[1].types[0].name, changed.raw[1].types[0].name);
    assert_ne!(
        control.raw[1].entities[0].name,
        changed.raw[1].entities[0].name
    );
    assert_ne!(
        control.raw[1].commands[0].name,
        changed.raw[1].commands[0].name
    );
    assert_ne!(control.raw[1].events[0].name, changed.raw[1].events[0].name);
    assert_ne!(control.raw[1].errors[0].name, changed.raw[1].errors[0].name);
    assert_ne!(control.raw[1].views[0].name, changed.raw[1].views[0].name);
    assert_ne!(control.raw[1].actors[0].name, changed.raw[1].actors[0].name);
    assert_ne!(
        control.raw[0].components[0].name,
        changed.raw[0].components[0].name
    );
    assert_ne!(
        control.raw[0].bindings[0].name,
        changed.raw[0].bindings[0].name
    );
    assert_ne!(
        control.raw[0]
            .topology
            .as_ref()
            .unwrap()
            .workloads
            .keys()
            .collect::<Vec<_>>(),
        changed.raw[0]
            .topology
            .as_ref()
            .unwrap()
            .workloads
            .keys()
            .collect::<Vec<_>>()
    );
    let old_domain = &control.spec.system().domains[0];
    let new_domain = &changed.spec.system().domains[0];
    assert_eq!(old_domain.name.to_string(), "model.data");
    assert_eq!(new_domain.name.to_string(), "model.renamed");
    assert_ne!(old_domain.name.segments(), new_domain.name.segments());
    for (member, old, new) in [
        (
            "entities",
            json!(old_domain.entities),
            json!(new_domain.entities),
        ),
        (
            "commands",
            json!(old_domain.commands),
            json!(new_domain.commands),
        ),
        ("events", json!(old_domain.events), json!(new_domain.events)),
        ("errors", json!(old_domain.errors), json!(new_domain.errors)),
        ("views", json!(old_domain.views), json!(new_domain.views)),
        ("actors", json!(old_domain.actors), json!(new_domain.actors)),
    ] {
        assert!(!old.as_array().unwrap().is_empty(), "{member}");
        assert_ne!(old, new, "{member}");
    }
    // Assembly moves domain type bodies into the system registry rather than
    // retaining duplicate definitions in DomainSpec.
    assert!(old_domain.types.is_empty());
    assert!(new_domain.types.is_empty());
    assert_ne!(control.spec.system().types, changed.spec.system().types);
}

#[test]
fn domain_member_and_compiler_handle_names_follow_real_compilation_after_rename() {
    let control = admitted(&fixture());
    let changed = admitted(&renamed_fixture());
    assert_raw_assembly_member_renaming(&control, &changed);
    let old = &control.ir.domains()[&name("model.data")];
    let new = &changed.ir.domains()[&name("model.renamed")];
    assert_ne!(old.name, new.name);
    for (member, old, new) in [
        ("types", json!(old.types), json!(new.types)),
        ("entities", json!(old.entities), json!(new.entities)),
        ("commands", json!(old.commands), json!(new.commands)),
        ("events", json!(old.events), json!(new.events)),
        ("errors", json!(old.errors), json!(new.errors)),
        ("views", json!(old.views), json!(new.views)),
        ("actors", json!(old.actors), json!(new.actors)),
    ] {
        assert!(!old.as_array().unwrap().is_empty(), "{member}");
        assert_ne!(old, new, "{member}");
        assert_eq!(
            serde_json::from_str::<Value>(&old.to_string().replace("model.data", "model.renamed"))
                .unwrap(),
            new,
            "{member}"
        );
    }
    for handle in &new.types {
        assert_eq!(changed.ir.named_type(handle).name, *handle.name());
    }
    for handle in &new.entities {
        assert_eq!(changed.ir.entity(handle).name, *handle.name());
    }
    for handle in &new.commands {
        assert_eq!(changed.ir.command(handle).name, *handle.name());
    }
    for handle in &new.events {
        assert_eq!(changed.ir.event(handle).name, *handle.name());
    }
    for handle in &new.errors {
        assert_eq!(changed.ir.error(handle).name, *handle.name());
    }
    for handle in &new.views {
        assert_eq!(changed.ir.view(handle).name, *handle.name());
    }
    for handle in &new.actors {
        assert_eq!(changed.ir.actor(handle).name, *handle.name());
    }
    for handle in &changed.ir.components().values().next().unwrap().owns {
        assert_eq!(changed.ir.domain(handle).name, *handle.name());
    }
    let workload = changed.ir.workloads().values().next().unwrap();
    assert_eq!(
        changed.ir.component(&workload.component).name,
        *workload.component.name()
    );
    assert_eq!(workload.component.name().to_string(), "renamed-service");
    // Observe the private-parts assembly through its public final maps, never from_parts.
    let before = serde_json::to_value(&control.ir).unwrap();
    let after = serde_json::to_value(&changed.ir).unwrap();
    for member in [
        "domains",
        "types",
        "entities",
        "commands",
        "events",
        "errors",
        "views",
        "actors",
        "components",
        "bindings",
        "workloads",
    ] {
        assert_ne!(before[member], after[member], "{member}");
    }
    unchanged(&control, &changed, false);
}
