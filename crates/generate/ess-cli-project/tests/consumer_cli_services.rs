//! Concrete component/view changes through binding, emission and direct dispatch.
//! View evaluation and transport implementation remain application obligations.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProcessOutput, ProtectedSource, Sources,
};
use ess_compiler::EssIr;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};

const MODEL: &str =
    include_str!("../../../specify/ess-cli-contract/tests/fixtures/service-model.yaml");
const BINDING: &str =
    include_str!("../../../specify/ess-cli-contract/tests/fixtures/service-cli.yaml");

fn model(text: &str) -> EssIr {
    let specification = Specification::assemble(vec![
        (
            Source::new("service.yaml"),
            RawSpecFile::parse(text).unwrap(),
        ),
        (
            Source::new("foreign.yaml"),
            RawSpecFile::parse("domain: demo.foreign\n").unwrap(),
        ),
    ])
    .unwrap();
    ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap()
}

fn bound(ir: &EssIr, text: &str) -> CompiledBinding {
    compile(ir, &Binding::from_yaml(text).unwrap()).unwrap()
}

fn component(ir: &EssIr, name: &str) -> Value {
    serde_json::to_value(
        ir.components()
            .values()
            .find(|item| item.name.to_string() == name)
            .unwrap(),
    )
    .unwrap()
}

fn view(ir: &EssIr) -> Value {
    serde_json::to_value(
        ir.views()
            .values()
            .find(|item| item.name.to_string() == "demo.records.NamedRecords")
            .unwrap(),
    )
    .unwrap()
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("service shape witnesses cannot acquire credentials")
    }
}

struct Recorder {
    calls: Vec<Value>,
    result: Value,
}
impl Handler for Recorder {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.calls.push(json!({"callable":invocation.callable,"target":invocation.target,"input":invocation.input}));
        assert!(invocation.context.config.is_none());
        assert!(invocation.context.state_dir.is_none());
        HandlerReply::Success(self.result.clone())
    }
}

fn execute(binding: &CompiledBinding, query: bool, result: Value) -> (ProcessOutput, Vec<Value>) {
    let mut handler = Recorder {
        calls: Vec::new(),
        result,
    };
    let args = if query {
        vec![
            "records",
            "records",
            "find",
            "--prefix",
            "7",
            "--output=json",
        ]
    } else {
        vec!["records", "rename", "--name", "7", "--output=json"]
    };
    let output = runtime::run(
        binding.plan(),
        args.into_iter().map(Into::into).collect(),
        &mut NoSources,
        &mut handler,
        None,
    );
    assert_eq!(output.exit_code, 0, "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(handler.calls.len(), 1);
    assert_eq!(
        serde_json::from_str::<Value>(&output.stdout).unwrap(),
        json!({"ok":true,"result":handler.result})
    );
    (output, handler.calls)
}

fn unchanged(before: &EssIr, after: &EssIr) {
    assert_ne!(before.to_canonical_json(), after.to_canonical_json());
    let a = bound(before, BINDING);
    let b = bound(after, BINDING);
    assert_eq!(a.to_canonical_json(), b.to_canonical_json());
    assert_eq!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    for query in [false, true] {
        let result = if query {
            json!([{"name":"7"}])
        } else {
            json!({"name":"7"})
        };
        let first = execute(&a, query, result.clone());
        let second = execute(&b, query, result);
        assert_eq!(first, second);
        assert_eq!(first.1[0]["target"]["owner"], "owner-service");
        assert_eq!(
            first.1[0]["input"],
            if query {
                json!({"prefix":"7"})
            } else {
                json!({"name":"7"})
            }
        );
    }
}

#[test]
fn component_reach_publication_and_naming_do_not_replace_explicit_presentation() {
    let before = model(MODEL);
    for (field, addition) in [
        ("reached_by", "    reached_by: network\n"),
        ("reached_by", "    reached_by: command_line\n    cli: {binary: old-records, commands: [demo.records.Rename], views: [demo.records.NamedRecords]}\n"),
        (
            "publishes",
            "    publishes: {events: [demo.records.Renamed]}\n",
        ),
        (
            "naming",
            "    naming: {wire: external-owner, display: Public Owner}\n",
        ),
        ("refs", "    refs: [local:fixture-owner]\n"),
    ] {
        let changed = MODEL.replace(
            "  - component: owner-service\n",
            &format!("  - component: owner-service\n{addition}"),
        );
        let after = model(&changed);
        assert_ne!(
            component(&before, "owner-service")[field],
            component(&after, "owner-service")[field],
            "{field}"
        );
        unchanged(&before, &after);
    }
}

#[test]
fn legacy_binary_root_and_group_placement_do_not_override_additive_binding() {
    let before = model(MODEL);
    for surface in [
        "    cli: {binary: old-records, commands: [demo.records.Rename], views: [demo.records.NamedRecords]}\n",
        "    cli: {binary: other-records, groups: [{name: admin, summary: Legacy commands, commands: [demo.records.Rename], views: [demo.records.NamedRecords]}]}\n",
    ] {
        let changed = MODEL.replace("  - component: owner-service\n", &format!("  - component: owner-service\n    reached_by: command_line\n{surface}"));
        let after = model(&changed);
        assert!(component(&before, "owner-service")["cli"].is_null());
        assert!(component(&after, "owner-service")["cli"].is_object());
        unchanged(&before, &after);
    }
}

#[test]
fn owner_identity_change_requires_explicit_binding_and_reaches_dispatch() {
    let before = model(MODEL);
    let after = model(&MODEL.replace("owner-service", "replacement-service"));
    assert_ne!(
        component(&before, "owner-service")["name"],
        component(&after, "replacement-service")["name"]
    );
    assert_eq!(
        compile(&after, &Binding::from_yaml(BINDING).unwrap())
            .unwrap_err()
            .to_string(),
        "unresolved service owner `owner-service`"
    );
    let original = bound(&before, BINDING);
    let replaced = bound(
        &after,
        &BINDING.replace("owner-service", "replacement-service"),
    );
    assert_ne!(original.to_canonical_json(), replaced.to_canonical_json());
    assert_ne!(
        ess_cli_project::project(&original)["binding.json"],
        ess_cli_project::project(&replaced)["binding.json"]
    );
    for query in [false, true] {
        let result = if query {
            json!([{"name":"7"}])
        } else {
            json!({"name":"7"})
        };
        let (_, old) = execute(&original, query, result.clone());
        let (_, new) = execute(&replaced, query, result);
        assert_eq!(old[0]["target"]["owner"], "owner-service");
        assert_eq!(new[0]["target"]["owner"], "replacement-service");
        assert_eq!(old[0]["input"], new[0]["input"]);
    }
}

#[test]
fn command_acceptance_and_view_domain_transfer_have_named_admission_refusals() {
    let before = model(MODEL);
    let unaccepted = model(&MODEL.replace("    accepts: {commands: [demo.records.Rename]}\n", ""));
    assert_ne!(
        component(&before, "owner-service")["accepts"],
        component(&unaccepted, "owner-service")["accepts"]
    );
    assert_eq!(
        compile(&unaccepted, &Binding::from_yaml(BINDING).unwrap())
            .unwrap_err()
            .to_string(),
        "service `owner-service` does not accept `demo.records.Rename`"
    );
    let transferred = MODEL.replace("owns: {domains: [demo.records]}", "owns: {domains: [demo.foreign]}")
        .replace("    accepts: {commands: [demo.records.Rename]}\n", "")
        .replace("  - component: foreign-service\n    owns: {domains: [demo.foreign]}", "  - component: foreign-service\n    owns: {domains: [demo.records]}\n    accepts: {commands: [demo.records.Rename]}");
    let after = model(&transferred);
    assert_ne!(
        component(&before, "owner-service")["owns"],
        component(&after, "owner-service")["owns"]
    );
    assert_eq!(
        compile(&after, &Binding::from_yaml(BINDING).unwrap())
            .unwrap_err()
            .to_string(),
        "service does not own view domain"
    );
    // Neither refusal is execution of a business command or view query.
    let valid = bound(&before, BINDING);
    execute(&valid, false, json!({"name":"7"}));
    execute(&valid, true, json!([{"name":"7"}]));
}

#[test]
fn selected_service_parameter_and_row_types_reach_emission_and_typed_execution() {
    let before = model(MODEL);
    let after = model(&MODEL.replace("type: String", "type: Integer"));
    for field in ["fields", "params"] {
        assert_ne!(view(&before)[field], view(&after)[field]);
    }
    let original = bound(&before, BINDING);
    let changed = bound(&after, BINDING);
    assert_ne!(original.to_canonical_json(), changed.to_canonical_json());
    assert_ne!(
        ess_cli_project::project(&original)["binding.json"],
        ess_cli_project::project(&changed)["binding.json"]
    );
    for query in [false, true] {
        let result = if query {
            json!([{"name":7}])
        } else {
            json!({"name":7})
        };
        let (_, calls) = execute(&changed, query, result);
        assert_eq!(
            calls[0]["input"],
            if query {
                json!({"prefix":7})
            } else {
                json!({"name":7})
            }
        );
        let old = if query {
            json!([{"name":"7"}])
        } else {
            json!({"name":"7"})
        };
        execute(&original, query, old);
    }
}

#[test]
fn view_filter_ranking_consistency_and_naming_remain_application_semantics() {
    let before = model(MODEL);
    for (field, changed) in [
        ("filter", MODEL.replace("name == param.prefix", "name != param.prefix")),
        ("order_by", MODEL.replace("    consistency: eventual", "    order_by: [name asc]\n    consistency: eventual")),
        ("order_by", MODEL.replace("    consistency: eventual", "    order_by: [name desc]\n    consistency: eventual")),
        ("consistency", MODEL.replace("    consistency: eventual", "    consistency: read_your_writes")),
        ("naming", MODEL.replace("  - name: demo.records.NamedRecords\n", "  - name: demo.records.NamedRecords\n    naming: {wire: named-records-v2, display: Named Records}\n")),
    ] {
        let after = model(&changed);
        assert_ne!(view(&before)[field], view(&after)[field], "{field}");
        if field == "consistency" {
            assert_ne!(view(&before)["assertion_style"], view(&after)["assertion_style"]);
        }
        unchanged(&before, &after);
    }
}

#[test]
fn named_view_shape_preserves_the_expanded_row_contract() {
    let before = model(MODEL);
    let after = model(&MODEL.replace(
        "    source: demo.records.Record\n    fields: [{name: name, type: String}]",
        "    source: demo.records.Record\n    shape: demo.records.Row",
    ));
    assert_ne!(view(&before)["shape"], view(&after)["shape"]);
    assert_eq!(view(&before)["fields"], view(&after)["fields"]);
    unchanged(&before, &after);
}
