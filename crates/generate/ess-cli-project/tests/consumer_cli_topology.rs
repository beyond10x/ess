//! Valid billing-topology controls for three direct CLI consumer boundaries.
//! These witnesses run the source runtime in-process; they launch no generated binary.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
};
use ess_compiler::{ir::ResolvedWorkload, source::SourceMap, EssIr};
use ess_domain::{
    component::ComponentName,
    spec::{RawSpecFile, Specification},
    system::Source,
    topology::{RawTopology, RawWorkload, Workload},
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, ffi::OsString};

const BILLING_TOPOLOGY: &str = include_str!("../../../../examples/billing/topology.yaml");

struct Model {
    raw: RawTopology,
    specification: Specification,
    ir: EssIr,
}

impl Model {
    fn raw_workload(&self, component: &str) -> &RawWorkload {
        &self.raw.workloads[component]
    }

    fn workload(&self, component: &str) -> &Workload {
        &self.specification.topology().workloads[&ComponentName::new(component).unwrap()]
    }

    fn resolved(&self, component: &str) -> &ResolvedWorkload {
        &self.ir.workloads()[&ComponentName::new(component).unwrap()]
    }
}

fn replace_one(source: &str, old: &str, new: &str) -> String {
    assert_eq!(source.matches(old).count(), 1);
    source.replacen(old, new, 1)
}

fn model(topology: &str) -> Model {
    let files = [
        (
            "system.yaml",
            include_str!("../../../../examples/billing/system.yaml"),
        ),
        (
            "components.yaml",
            include_str!("../../../../examples/billing/components.yaml"),
        ),
        (
            "domains/invoice.yaml",
            include_str!("../../../../examples/billing/domains/invoice.yaml"),
        ),
        (
            "domains/email.yaml",
            include_str!("../../../../examples/billing/domains/email.yaml"),
        ),
        ("topology.yaml", topology),
    ];
    let mut parsed = Vec::new();
    let mut sources = SourceMap::new();
    for (label, original) in files {
        // Add one separately selected value within the existing invoice domain.
        // No CLI target invokes a billing command or deploys a workload.
        let text = if label == "domains/invoice.yaml" {
            replace_one(
                original,
                "types:\n",
                "types:\n  - name: billing.invoice.CliInput\n    kind: struct\n    fields:\n      - {name: value, type: Integer}\n",
            )
        } else {
            original.to_owned()
        };
        let raw = RawSpecFile::parse(&text).unwrap();
        sources.insert(label.to_owned(), text);
        parsed.push((Source::new(label), raw));
    }
    let raw = RawSpecFile::parse(topology).unwrap().topology.unwrap();
    let specification = Specification::assemble(parsed).unwrap();
    let ir = ess_compiler::compile(&specification, &sources).unwrap();
    Model {
        raw,
        specification,
        ir,
    }
}

fn invoice_only() -> &'static str {
    let (invoice, email) = BILLING_TOPOLOGY.split_once("\n    email-service:").unwrap();
    assert!(email.contains("email-events"));
    invoice
}

fn binding() -> Binding {
    Binding::from_yaml(
        r"
format: ess-cli/1
binary: billing-topology
about: Observe one separately selected typed value
globals: {config: config, state: state-dir, output: output}
callables:
  observe:
    target: {kind: local, owner: billing.cli, action: observe}
    input: billing.invoice.CliInput
    result: Integer
commands:
  - path: [observe]
    callable: observe
    about: Observe a typed value without running billing workloads
    arguments:
      - {field: value, source: {kind: option, long: value}}
",
    )
    .unwrap()
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("topology witnesses must not acquire any external source")
    }
}

#[derive(Default)]
struct Recording {
    calls: Vec<Value>,
}
impl Handler for Recording {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        self.calls.push(json!({
            "callable": invocation.callable,
            "target": serde_json::to_value(invocation.target).unwrap(),
            "input": invocation.input,
        }));
        assert_eq!(
            invocation.context.config.as_deref(),
            Some(std::path::Path::new("selected.toml"))
        );
        assert_eq!(
            invocation.context.state_dir.as_deref(),
            Some(std::path::Path::new("selected-state"))
        );
        HandlerReply::Success(invocation.input["value"].clone())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Observation {
    exit_code: i32,
    stdout: String,
    stderr: String,
    calls: Vec<Value>,
}

fn execute(compiled: &CompiledBinding) -> Observation {
    let args: Vec<OsString> = [
        "billing-topology",
        "observe",
        "--value",
        "17",
        "--config",
        "selected.toml",
        "--state-dir",
        "selected-state",
        "--output=json",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    let mut handler = Recording::default();
    let output =
        ess_cli_project::runtime::run(compiled.plan(), args, &mut NoSources, &mut handler, None);
    Observation {
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
        calls: handler.calls,
    }
}

fn assert_same_cli(control: &Model, changed: &Model) {
    // Resolution is entered for both real, independently compiled ESS inputs.
    let before = compile(&control.ir, &binding()).unwrap();
    let after = compile(&changed.ir, &binding()).unwrap();
    assert_eq!(before.to_canonical_json(), after.to_canonical_json());
    let before_files: BTreeMap<String, String> = ess_cli_project::project(&before);
    let after_files = ess_cli_project::project(&after);
    assert!(!before_files.is_empty());
    assert_eq!(before_files["binding.json"], before.to_canonical_json());
    assert_eq!(after_files["binding.json"], after.to_canonical_json());
    assert_eq!(before_files, after_files);

    // Compare actual runtime observations, including recorded typed input.
    let before_output = execute(&before);
    let after_output = execute(&after);
    assert_eq!(before_output, after_output);
    assert_eq!(before_output.exit_code, 0);
    assert!(before_output.stderr.is_empty());
    assert_eq!(before_output.stdout, "{\"ok\":true,\"result\":17}\n");
    assert_eq!(
        before_output.calls,
        [json!({
            "callable": "observe",
            "target": {"kind": "local", "owner": "billing.cli", "action": "observe"},
            "input": {"value": 17},
        })]
    );
}

#[test]
fn topology_workload_membership_has_no_cli_effect() {
    let control = model(BILLING_TOPOLOGY);
    let changed = model(invoice_only());
    assert_eq!(control.raw.workloads.len(), 2);
    assert_eq!(changed.raw.workloads.len(), 1);
    assert!(control.raw.workloads.contains_key("email-service"));
    assert!(!changed.raw.workloads.contains_key("email-service"));
    assert_eq!(control.specification.topology().workloads.len(), 2);
    assert_eq!(changed.specification.topology().workloads.len(), 1);
    assert_eq!(control.ir.workloads().len(), 2);
    assert_eq!(changed.ir.workloads().len(), 1);
    assert_eq!(
        control.resolved("invoice-service"),
        changed.resolved("invoice-service")
    );
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}

#[test]
fn topology_empty_workload_map_has_no_cli_effect() {
    let control = model(invoice_only());
    let changed = model("topology: {workloads: {}}\n");
    assert_eq!(control.raw.workloads.len(), 1);
    assert!(changed.raw.workloads.is_empty());
    assert!(changed.specification.topology().workloads.is_empty());
    assert!(changed.ir.workloads().is_empty());
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}

#[test]
fn topology_workload_component_identity_has_no_cli_effect() {
    let control = model(invoice_only());
    let changed = model(&replace_one(
        invoice_only(),
        "    invoice-service:",
        "    email-service:",
    ));
    assert!(control.raw.workloads.contains_key("invoice-service"));
    assert!(changed.raw.workloads.contains_key("email-service"));
    assert_eq!(
        control.workload("invoice-service").component.as_str(),
        "invoice-service"
    );
    assert_eq!(
        changed.workload("email-service").component.as_str(),
        "email-service"
    );
    let before = control.resolved("invoice-service");
    let after = changed.resolved("email-service");
    assert_eq!(before.component.name().as_str(), "invoice-service");
    assert_eq!(after.component.name().as_str(), "email-service");
    assert_ne!(before.component, after.component);
    assert_eq!(before.replicas, after.replicas);
    assert_eq!(before.requires, after.requires);
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}

#[test]
fn topology_replica_floor_has_no_cli_effect() {
    let control = model(invoice_only());
    let changed = model(&replace_one(
        invoice_only(),
        "        min: 2",
        "        min: 3",
    ));
    assert_eq!(
        control
            .raw_workload("invoice-service")
            .replicas
            .unwrap()
            .min,
        2
    );
    assert_eq!(
        changed
            .raw_workload("invoice-service")
            .replicas
            .unwrap()
            .min,
        3
    );
    assert_eq!(control.workload("invoice-service").replicas.min, 2);
    assert_eq!(changed.workload("invoice-service").replicas.min, 3);
    assert_eq!(control.resolved("invoice-service").replicas.min, 2);
    assert_eq!(changed.resolved("invoice-service").replicas.min, 3);
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}

#[test]
fn topology_replica_ceiling_and_null_shapes_have_no_cli_effect() {
    let omitted = model(invoice_only());
    let null_source = replace_one(
        invoice_only(),
        "        min: 2",
        "        min: 2\n        max: null",
    );
    let null = model(&null_source);
    assert_ne!(invoice_only(), null_source);
    assert_eq!(
        omitted
            .raw_workload("invoice-service")
            .replicas
            .unwrap()
            .max,
        None
    );
    assert_eq!(
        null.raw_workload("invoice-service").replicas.unwrap().max,
        None
    );
    assert_eq!(omitted.workload("invoice-service").replicas.max, None);
    assert_eq!(null.resolved("invoice-service").replicas.max, None);
    assert_eq!(omitted.ir.to_canonical_json(), null.ir.to_canonical_json());
    assert_same_cli(&omitted, &null);

    for ceiling in [3, 4] {
        let changed = model(&replace_one(
            invoice_only(),
            "        min: 2",
            &format!("        min: 2\n        max: {ceiling}"),
        ));
        assert_eq!(
            changed
                .raw_workload("invoice-service")
                .replicas
                .unwrap()
                .max,
            Some(ceiling)
        );
        assert_eq!(
            changed.workload("invoice-service").replicas.max,
            Some(ceiling)
        );
        assert_eq!(
            changed.resolved("invoice-service").replicas.max,
            Some(ceiling)
        );
        assert_ne!(
            omitted.ir.to_canonical_json(),
            changed.ir.to_canonical_json()
        );
        assert_same_cli(&omitted, &changed);
    }
}

#[test]
fn topology_optional_replica_shapes_normalize_without_cli_effect() {
    let declaration = "      replicas:\n        min: 2\n";
    let omitted_source = replace_one(invoice_only(), declaration, "");
    let omitted = model(&omitted_source);
    assert!(omitted.raw_workload("invoice-service").replicas.is_none());
    for (replacement, explicit) in [
        ("      replicas: null\n", false),
        ("      replicas: {min: 1}\n", true),
    ] {
        let source = replace_one(invoice_only(), declaration, replacement);
        assert_ne!(omitted_source, source);
        let changed = model(&source);
        assert_eq!(
            changed.raw_workload("invoice-service").replicas.is_some(),
            explicit
        );
        for candidate in [&omitted, &changed] {
            assert_eq!(candidate.workload("invoice-service").replicas.min, 1);
            assert_eq!(candidate.workload("invoice-service").replicas.max, None);
            assert_eq!(candidate.resolved("invoice-service").replicas.min, 1);
            assert_eq!(candidate.resolved("invoice-service").replicas.max, None);
        }
        assert_eq!(
            omitted.ir.to_canonical_json(),
            changed.ir.to_canonical_json()
        );
        assert_same_cli(&omitted, &changed);
    }
}

#[test]
fn topology_stateless_boolean_has_no_cli_effect() {
    let source = replace_one(invoice_only(), "        min: 2", "        min: 1");
    let control = model(&source);
    let changed = model(&replace_one(&source, "stateless: true", "stateless: false"));
    assert_eq!(
        control.raw_workload("invoice-service").stateless,
        Some(true)
    );
    assert_eq!(
        changed.raw_workload("invoice-service").stateless,
        Some(false)
    );
    assert!(control.workload("invoice-service").stateless);
    assert!(!changed.workload("invoice-service").stateless);
    assert!(control.resolved("invoice-service").stateless);
    assert!(!changed.resolved("invoice-service").stateless);
    assert_eq!(changed.resolved("invoice-service").replicas.min, 1);
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}

#[test]
fn topology_optional_stateless_shapes_normalize_without_cli_effect() {
    let control = model(invoice_only());
    assert_eq!(
        control.raw_workload("invoice-service").stateless,
        Some(true)
    );
    for replacement in ["", "      stateless: null\n"] {
        let source = replace_one(invoice_only(), "      stateless: true\n", replacement);
        assert_ne!(invoice_only(), source);
        let changed = model(&source);
        assert_eq!(changed.raw_workload("invoice-service").stateless, None);
        assert!(changed.workload("invoice-service").stateless);
        assert!(changed.resolved("invoice-service").stateless);
        assert_eq!(
            control.ir.to_canonical_json(),
            changed.ir.to_canonical_json()
        );
        assert_same_cli(&control, &changed);
    }
}

#[test]
fn topology_resource_kind_has_no_cli_effect() {
    let control = model(invoice_only());
    let changed = model(&replace_one(
        invoice_only(),
        "postgres: invoice-store",
        "cache: invoice-store",
    ));
    assert_eq!(
        control.raw_workload("invoice-service").requires[0].0["postgres"],
        "invoice-store"
    );
    assert_eq!(
        changed.raw_workload("invoice-service").requires[0].0["cache"],
        "invoice-store"
    );
    assert_eq!(
        control.workload("invoice-service").requires[0].kind,
        "postgres"
    );
    assert_eq!(
        changed.workload("invoice-service").requires[0].kind,
        "cache"
    );
    assert_eq!(
        control.resolved("invoice-service").requires[0].kind,
        "postgres"
    );
    assert_eq!(
        changed.resolved("invoice-service").requires[0].kind,
        "cache"
    );
    assert_eq!(
        control.resolved("invoice-service").requires[0].name,
        changed.resolved("invoice-service").requires[0].name
    );
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}

#[test]
fn topology_resource_name_has_no_cli_effect() {
    let control = model(invoice_only());
    let changed = model(&replace_one(
        invoice_only(),
        "postgres: invoice-store",
        "postgres: invoice-archive",
    ));
    assert_eq!(
        control.raw_workload("invoice-service").requires[0].0["postgres"],
        "invoice-store"
    );
    assert_eq!(
        changed.raw_workload("invoice-service").requires[0].0["postgres"],
        "invoice-archive"
    );
    assert_eq!(
        control.workload("invoice-service").requires[0].name,
        "invoice-store"
    );
    assert_eq!(
        changed.workload("invoice-service").requires[0].name,
        "invoice-archive"
    );
    assert_eq!(
        control.resolved("invoice-service").requires[0].name,
        "invoice-store"
    );
    assert_eq!(
        changed.resolved("invoice-service").requires[0].name,
        "invoice-archive"
    );
    assert_eq!(
        control.resolved("invoice-service").requires[0].kind,
        changed.resolved("invoice-service").requires[0].kind
    );
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}

#[test]
fn topology_resource_list_membership_has_no_cli_effect() {
    let control = model(invoice_only());
    let changed = model(&replace_one(
        invoice_only(),
        "        - publish: invoice-events\n",
        "",
    ));
    assert_eq!(control.raw_workload("invoice-service").requires.len(), 2);
    assert_eq!(changed.raw_workload("invoice-service").requires.len(), 1);
    assert_eq!(control.workload("invoice-service").requires.len(), 2);
    assert_eq!(changed.workload("invoice-service").requires.len(), 1);
    assert_eq!(control.resolved("invoice-service").requires.len(), 2);
    assert_eq!(changed.resolved("invoice-service").requires.len(), 1);
    assert_eq!(
        control.resolved("invoice-service").requires[1].kind,
        "publish"
    );
    assert_eq!(
        control.resolved("invoice-service").requires[1].name,
        "invoice-events"
    );
    assert_eq!(
        control.resolved("invoice-service").requires[0],
        changed.resolved("invoice-service").requires[0]
    );
    assert_ne!(
        control.ir.to_canonical_json(),
        changed.ir.to_canonical_json()
    );
    assert_same_cli(&control, &changed);
}
