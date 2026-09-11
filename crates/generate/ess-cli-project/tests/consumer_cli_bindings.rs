//! Binding and actor controls through three direct CLI consumer boundaries.
//! The recording handler observes only a local value; no interaction policy is executed.

use ess_cli_contract::{compile, Binding, CompiledBinding};
use ess_cli_project::runtime::{
    self, AcquireError, Handler, HandlerReply, Invocation, ProtectedSource, Sources,
};
use ess_compiler::{
    ir::{ResolvedBinding, ResolvedInstance, ResolvedMappingValue},
    source::SourceMap,
    EssIr,
};
use ess_domain::{
    binding::{BindingName, BindingSpec, Delivery, Failure, MappingSource, RawBindingSpec},
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};

const SYSTEM: &str = include_str!("../../../../examples/oracle-fixture/system.yaml");
const ORDER: &str = include_str!("../../../../examples/oracle-fixture/domains/order.yaml");
const DISPATCH: &str = include_str!("../../../../examples/oracle-fixture/domains/dispatch.yaml");
const INTERACTIONS: &str = include_str!("../../../../examples/oracle-fixture/components.yaml");
const FIRST: &str = "handoff-on-placed";

struct Model {
    raw_interactions: RawSpecFile,
    raw_order: RawSpecFile,
    raw_dispatch: RawSpecFile,
    specification: Specification,
    ir: EssIr,
}

impl Model {
    fn raw_binding(&self, name: &str) -> &RawBindingSpec {
        self.raw_interactions
            .bindings
            .iter()
            .find(|b| b.name == name)
            .unwrap()
    }

    fn binding(&self, name: &str) -> &BindingSpec {
        &self.specification.bindings()[&BindingName::new(name).unwrap()]
    }

    fn resolved(&self, name: &str) -> &ResolvedBinding {
        &self.ir.bindings()[&BindingName::new(name).unwrap()]
    }
}

fn replace_one(source: &str, old: &str, new: &str) -> String {
    assert_eq!(source.matches(old).count(), 1, "finite fixture edit: {old}");
    source.replacen(old, new, 1)
}

fn model(interactions: &str, order: &str, dispatch: &str) -> Model {
    let raw_interactions = RawSpecFile::parse(interactions).unwrap();
    let raw_order = RawSpecFile::parse(order).unwrap();
    let raw_dispatch = RawSpecFile::parse(dispatch).unwrap();
    let order_with_value = replace_one(
        order,
        "types:\n",
        "types:\n  - name: oracle.order.CliInput\n    kind: struct\n    fields:\n      - {name: value, type: Integer}\n",
    );
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in [
        ("system.yaml", SYSTEM),
        ("components.yaml", interactions),
        ("domains/order.yaml", &order_with_value),
        ("domains/dispatch.yaml", dispatch),
    ] {
        sources.insert(label.to_owned(), text.to_owned());
        parsed.push((Source::new(label), RawSpecFile::parse(text).unwrap()));
    }
    let specification = Specification::assemble(parsed).unwrap();
    let ir = ess_compiler::compile(&specification, &sources).unwrap();
    Model {
        raw_interactions,
        raw_order,
        raw_dispatch,
        specification,
        ir,
    }
}

fn local_binding(ir: &EssIr) -> CompiledBinding {
    let binding = Binding::from_yaml(
        r"
format: ess-cli/1
binary: oracle-bindings
about: Observe a separate local value
globals: {config: config, state: state-dir, output: output}
callables:
  observe:
    target: {kind: local, owner: oracle.cli, action: observe}
    input: oracle.order.CliInput
    result: Integer
commands:
  - path: [observe]
    callable: observe
    about: Observe a value without executing interaction bindings
    arguments:
      - {field: value, source: {kind: option, long: value}}
",
    )
    .unwrap();
    compile(ir, &binding).unwrap()
}

struct NoSources;
impl Sources for NoSources {
    fn acquire(&mut self, _: ProtectedSource) -> Result<String, AcquireError> {
        panic!("binding witnesses cannot acquire any external source")
    }
}

#[derive(Default)]
struct Recording {
    calls: Vec<Value>,
}

impl Handler for Recording {
    fn call(&mut self, invocation: &Invocation<'_>) -> HandlerReply {
        assert_eq!(invocation.context.output, runtime::OutputMode::Json);
        self.calls.push(json!({
            "callable": invocation.callable,
            "target": invocation.target,
            "input": invocation.input,
            "config": invocation.context.config,
            "state_dir": invocation.context.state_dir,
        }));
        HandlerReply::Success(invocation.input["value"].clone())
    }
}

fn execute(binding: &CompiledBinding) -> (i32, String, String, Vec<Value>) {
    let mut handler = Recording::default();
    let output = runtime::run(
        binding.plan(),
        [
            "oracle-bindings",
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
        .collect(),
        &mut NoSources,
        &mut handler,
        None,
    );
    assert_eq!(output.exit_code, 0, "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_str::<Value>(&output.stdout).unwrap(),
        json!({"ok":true,"result":17})
    );
    assert_eq!(
        handler.calls,
        vec![json!({
            "callable":"observe", "target":{"kind":"local","owner":"oracle.cli","action":"observe"},
            "input":{"value":17}, "config":"selected.toml", "state_dir":"selected-state"
        })]
    );
    (
        output.exit_code,
        output.stdout,
        output.stderr,
        handler.calls,
    )
}

fn unchanged_cli(before: &Model, after: &Model) {
    let a = local_binding(&before.ir);
    let b = local_binding(&after.ir);
    assert_eq!(a.to_canonical_json(), b.to_canonical_json());
    assert_eq!(ess_cli_project::project(&a), ess_cli_project::project(&b));
    assert_eq!(execute(&a), execute(&b));
}

fn changed_model_same_cli(before: &Model, after: &Model) {
    assert_ne!(before.specification, after.specification);
    assert_ne!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    unchanged_cli(before, after);
}

#[test]
fn binding_trigger_and_invocation_change_resolved_handles_only_outside_local_cli() {
    let before = model(INTERACTIONS, ORDER, DISPATCH);
    let trigger = replace_one(
        INTERACTIONS,
        "event: oracle.order.OrderPlaced",
        "event: oracle.order.OrderHeld",
    );
    let after = model(&trigger, ORDER, DISPATCH);
    assert_eq!(
        before
            .raw_binding(FIRST)
            .when
            .event
            .as_ref()
            .expect("event fixture")
            .to_string(),
        "oracle.order.OrderPlaced"
    );
    assert_eq!(
        after
            .raw_binding(FIRST)
            .when
            .event
            .as_ref()
            .expect("event fixture")
            .to_string(),
        "oracle.order.OrderHeld"
    );
    assert_eq!(
        after.binding(FIRST).cause.event(),
        after.raw_binding(FIRST).when.event.as_ref()
    );
    assert_eq!(
        before.binding(FIRST).cause.event(),
        before.raw_binding(FIRST).when.event.as_ref()
    );
    assert_eq!(
        before
            .resolved(FIRST)
            .cause
            .event()
            .expect("event fixture")
            .to_string(),
        "oracle.order.OrderPlaced"
    );
    assert_eq!(
        after
            .resolved(FIRST)
            .cause
            .event()
            .expect("event fixture")
            .to_string(),
        "oracle.order.OrderHeld"
    );
    changed_model_same_cli(&before, &after);

    let invocation = INTERACTIONS.replace("oracle.dispatch.Handoff", "oracle.dispatch.Redirect");
    let dispatch = DISPATCH.replace("oracle.dispatch.Handoff", "oracle.dispatch.Redirect");
    let after = model(&invocation, ORDER, &dispatch);
    assert_eq!(
        before.raw_binding(FIRST).invoke.command.to_string(),
        "oracle.dispatch.Handoff"
    );
    assert_eq!(
        after.raw_binding(FIRST).invoke.command.to_string(),
        "oracle.dispatch.Redirect"
    );
    assert_eq!(
        before.binding(FIRST).command,
        before.raw_binding(FIRST).invoke.command
    );
    assert_eq!(
        after.binding(FIRST).command,
        after.raw_binding(FIRST).invoke.command
    );
    assert_eq!(
        before.resolved(FIRST).command.to_string(),
        "oracle.dispatch.Handoff"
    );
    assert_eq!(
        after.resolved(FIRST).command.to_string(),
        "oracle.dispatch.Redirect"
    );
    changed_model_same_cli(&before, &after);
}

#[test]
fn binding_event_mapping_selects_another_declared_field_without_cli_effect() {
    let before = model(INTERACTIONS, ORDER, DISPATCH);
    let text = INTERACTIONS.replacen(
        "recipient: event.contact",
        "recipient: event.alternate_contact",
        1,
    );
    let after = model(&text, ORDER, DISPATCH);
    for (witness, expected) in [(&before, "contact"), (&after, "alternate_contact")] {
        assert_eq!(witness.raw_binding(FIRST).mapping.0[0].target, "recipient");
        let source = MappingSource::EventField {
            field: expected.to_owned(),
        };
        assert_eq!(witness.raw_binding(FIRST).mapping.0[0].source, source);
        assert_eq!(witness.binding(FIRST).mapping["recipient"], source);
        let ResolvedMappingValue::EventField { field, type_ref } =
            &witness.resolved(FIRST).mapping[0].value
        else {
            panic!("an event-field mapping");
        };
        assert_eq!(field, expected);
        assert_eq!(
            type_ref.declared().unwrap().to_string(),
            "oracle.order.Email"
        );
    }
    assert_ne!(
        before.raw_binding(FIRST).mapping,
        after.raw_binding(FIRST).mapping
    );
    assert_ne!(
        before.resolved(FIRST).mapping,
        after.resolved(FIRST).mapping
    );
    changed_model_same_cli(&before, &after);
}

#[test]
fn binding_literal_replaces_event_mapping_and_its_conversion_without_cli_effect() {
    let before = model(INTERACTIONS, ORDER, DISPATCH);
    let text = INTERACTIONS.replacen(
        "recipient: event.contact",
        "recipient: fixed@example.invalid",
        1,
    );
    let after = model(&text, ORDER, DISPATCH);
    let literal = MappingSource::Literal {
        value: "fixed@example.invalid".to_owned(),
    };
    assert!(matches!(
        before.raw_binding(FIRST).mapping.0[0].source,
        MappingSource::EventField { .. }
    ));
    assert_eq!(after.raw_binding(FIRST).mapping.0[0].source, literal);
    assert_eq!(after.binding(FIRST).mapping["recipient"], literal);
    assert!(matches!(
        before.resolved(FIRST).mapping[0].value,
        ResolvedMappingValue::EventField { .. }
    ));
    assert_eq!(
        after.resolved(FIRST).mapping[0].value,
        ResolvedMappingValue::Literal {
            value: "fixed@example.invalid".to_owned()
        }
    );
    assert!(before.resolved(FIRST).mapping[0].conversion.is_some());
    assert!(after.resolved(FIRST).mapping[0].conversion.is_none());
    changed_model_same_cli(&before, &after);
}

#[test]
fn binding_mapping_target_literal_and_resolved_types_change_without_cli_effect() {
    let before = model(INTERACTIONS, ORDER, DISPATCH);
    let text = INTERACTIONS
        .replace("label:", "template:")
        .replace("template: placed", "template: revised");
    let dispatch = DISPATCH.replace("name: label", "name: template");
    let after = model(&text, ORDER, &dispatch);
    assert_eq!(before.raw_binding(FIRST).mapping.0[1].target, "label");
    assert_eq!(after.raw_binding(FIRST).mapping.0[1].target, "template");
    assert_eq!(
        before.binding(FIRST).mapping["label"],
        MappingSource::Literal {
            value: "placed".to_owned()
        }
    );
    assert_eq!(
        after.binding(FIRST).mapping["template"],
        MappingSource::Literal {
            value: "revised".to_owned()
        }
    );
    assert_eq!(before.resolved(FIRST).mapping[1].target, "label");
    assert_eq!(after.resolved(FIRST).mapping[1].target, "template");
    assert_eq!(
        after.resolved(FIRST).mapping[1].value,
        ResolvedMappingValue::Literal {
            value: "revised".to_owned()
        }
    );
    changed_model_same_cli(&before, &after);

    let text = INTERACTIONS
        .replace("oracle.order.Email", "oracle.order.ContactEmail")
        .replace("oracle.dispatch.Recipient", "oracle.dispatch.Address")
        .replace(
            "An order's contact address is where the carrier's notice goes;",
            "The revised address conversion supplies the recipient;",
        );
    let order = ORDER.replace("oracle.order.Email", "oracle.order.ContactEmail");
    let dispatch = DISPATCH.replace("oracle.dispatch.Recipient", "oracle.dispatch.Address");
    let after = model(&text, &order, &dispatch);
    let a = &before.resolved(FIRST).mapping[0];
    let b = &after.resolved(FIRST).mapping[0];
    let ResolvedMappingValue::EventField {
        type_ref: source_a, ..
    } = &a.value
    else {
        panic!("event field")
    };
    let ResolvedMappingValue::EventField {
        type_ref: source_b, ..
    } = &b.value
    else {
        panic!("event field")
    };
    assert_ne!(source_a, source_b);
    assert_ne!(a.target_type, b.target_type);
    assert_eq!(
        source_a.declared().unwrap().to_string(),
        "oracle.order.Email"
    );
    assert_eq!(
        source_b.declared().unwrap().to_string(),
        "oracle.order.ContactEmail"
    );
    assert_eq!(
        a.target_type.declared().unwrap().to_string(),
        "oracle.dispatch.Recipient"
    );
    assert_eq!(
        b.target_type.declared().unwrap().to_string(),
        "oracle.dispatch.Address"
    );
    assert!(a
        .conversion
        .as_ref()
        .unwrap()
        .starts_with("An order's contact address"));
    assert!(b
        .conversion
        .as_ref()
        .unwrap()
        .starts_with("The revised address conversion"));
    assert_ne!(
        before.raw_interactions.conversions,
        after.raw_interactions.conversions
    );
    changed_model_same_cli(&before, &after);
}

#[test]
fn binding_delivery_obligation_arrives_with_a_real_binding_without_cli_effect() {
    let (prefix, first_and_rest) = INTERACTIONS
        .split_once("  - id: handoff-on-placed\n")
        .unwrap();
    let (_, rest) = first_and_rest
        .split_once("  - id: handoff-on-held\n")
        .unwrap();
    let without = format!("{prefix}  - id: handoff-on-held\n{rest}");
    let before = model(&without, ORDER, DISPATCH);
    let after = model(INTERACTIONS, ORDER, DISPATCH);
    assert!(before
        .raw_interactions
        .bindings
        .iter()
        .all(|b| b.name != FIRST));
    assert!(!before
        .specification
        .bindings()
        .contains_key(&BindingName::new(FIRST).unwrap()));
    assert!(!before
        .ir
        .bindings()
        .contains_key(&BindingName::new(FIRST).unwrap()));
    assert_eq!(after.raw_binding(FIRST).delivery, Delivery::AtLeastOnce);
    assert_eq!(after.binding(FIRST).delivery, Delivery::AtLeastOnce);
    assert_eq!(after.resolved(FIRST).delivery, Delivery::AtLeastOnce);
    assert_eq!(
        before.raw_interactions.bindings.len() + 1,
        after.raw_interactions.bindings.len()
    );
    assert_eq!(
        before.specification.bindings().len() + 1,
        after.specification.bindings().len()
    );
    assert_eq!(before.ir.bindings().len() + 1, after.ir.bindings().len());
    changed_model_same_cli(&before, &after);
}

#[test]
fn binding_failure_and_escalation_policies_change_without_cli_execution() {
    let before = model(INTERACTIONS, ORDER, DISPATCH);
    let drop_text = replace_one(INTERACTIONS, "on_failure: retry", "on_failure: drop");
    let dropped = model(&drop_text, ORDER, DISPATCH);
    let escalate_text = replace_one(
        INTERACTIONS,
        "on_failure: retry",
        "on_failure:\n      escalate:\n        emits: oracle.dispatch.HandoffEscalated",
    );
    let escalated = model(&escalate_text, ORDER, DISPATCH);
    for (witness, expected, event) in [
        (&before, Failure::Retry, None),
        (&dropped, Failure::Drop, None),
        (
            &escalated,
            Failure::Escalate,
            Some("oracle.dispatch.HandoffEscalated"),
        ),
    ] {
        assert_eq!(witness.raw_binding(FIRST).on_failure.failure, expected);
        assert_eq!(witness.binding(FIRST).failure, expected);
        assert_eq!(witness.resolved(FIRST).failure, expected);
        assert_eq!(
            witness
                .raw_binding(FIRST)
                .on_failure
                .emits
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            event
        );
        assert_eq!(
            witness
                .binding(FIRST)
                .escalation
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            event
        );
        assert_eq!(
            witness
                .resolved(FIRST)
                .escalation
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            event
        );
    }
    changed_model_same_cli(&before, &dropped);
    changed_model_same_cli(&before, &escalated);
    let other_event = escalate_text.replacen(
        "emits: oracle.dispatch.HandoffEscalated",
        "emits: oracle.dispatch.HandedOff",
        1,
    );
    let after = model(&other_event, ORDER, DISPATCH);
    assert_eq!(
        after
            .raw_binding(FIRST)
            .on_failure
            .emits
            .as_ref()
            .unwrap()
            .to_string(),
        "oracle.dispatch.HandedOff"
    );
    assert_eq!(
        after
            .binding(FIRST)
            .escalation
            .as_ref()
            .unwrap()
            .to_string(),
        "oracle.dispatch.HandedOff"
    );
    assert_eq!(
        after
            .resolved(FIRST)
            .escalation
            .as_ref()
            .unwrap()
            .to_string(),
        "oracle.dispatch.HandedOff"
    );
    changed_model_same_cli(&escalated, &after);
}

#[test]
fn binding_identity_naming_summary_and_refs_change_without_cli_effect() {
    let before = model(INTERACTIONS, ORDER, DISPATCH);
    let text = replace_one(
        INTERACTIONS,
        "  - id: handoff-on-placed\n    summary: Ask the carrier to collect a new order.\n",
        "  - name: revised-handoff\n    naming: {wire: revised-notice, display: Revised notice}\n    summary: A revised interaction description.\n    refs: ['aep:binding-review']\n",
    );
    let after = model(&text, ORDER, DISPATCH);
    let a = before.raw_binding(FIRST);
    let b = after.raw_binding("revised-handoff");
    assert_eq!(a.name, FIRST);
    assert_eq!(b.name, "revised-handoff");
    assert_ne!(a.naming, b.naming);
    assert_eq!(b.naming.wire.as_deref(), Some("revised-notice"));
    assert_eq!(b.naming.display.as_deref(), Some("Revised notice"));
    assert_eq!(
        a.summary.as_deref(),
        Some("Ask the carrier to collect a new order.")
    );
    assert_eq!(
        b.summary.as_deref(),
        Some("A revised interaction description.")
    );
    assert!(a.refs.is_empty());
    assert_eq!(
        b.refs.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["aep:binding-review"]
    );
    for (assembled, resolved, raw) in [
        (before.binding(FIRST), before.resolved(FIRST), a),
        (
            after.binding("revised-handoff"),
            after.resolved("revised-handoff"),
            b,
        ),
    ] {
        assert_eq!(assembled.name.to_string(), raw.name);
        assert_eq!(resolved.name.to_string(), raw.name);
        assert_eq!(assembled.naming.wire, raw.naming.wire);
        assert_eq!(assembled.naming.display, raw.naming.display);
        assert_eq!(assembled.naming.summary, raw.summary);
        assert_eq!(resolved.naming, assembled.naming);
        assert_eq!(assembled.refs, raw.refs);
        assert_eq!(resolved.refs, raw.refs);
    }
    changed_model_same_cli(&before, &after);
}

#[test]
fn binding_empty_metadata_forms_normalize_without_cli_effect() {
    let omitted = replace_one(
        INTERACTIONS,
        "    summary: Ask the carrier to collect a new order.\n",
        "",
    );
    let explicit = replace_one(
        &omitted,
        "  - id: handoff-on-placed\n",
        "  - id: handoff-on-placed\n    summary: null\n    naming: {}\n    refs: []\n",
    );
    assert_ne!(omitted, explicit);
    let before = model(&omitted, ORDER, DISPATCH);
    let after = model(&explicit, ORDER, DISPATCH);
    assert_eq!(before.raw_binding(FIRST).summary, None);
    assert_eq!(after.raw_binding(FIRST).summary, None);
    assert_eq!(
        before.raw_binding(FIRST).naming,
        after.raw_binding(FIRST).naming
    );
    assert!(after.raw_binding(FIRST).naming.is_empty());
    assert_eq!(
        before.raw_binding(FIRST).refs,
        after.raw_binding(FIRST).refs
    );
    assert_eq!(before.specification, after.specification);
    assert_eq!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    unchanged_cli(&before, &after);
}

#[test]
fn actor_grants_change_raw_assembled_and_resolved_authority_without_cli_effect() {
    let before_text = format!(
        "{ORDER}\nactors:\n  - name: oracle.order.Clerk\n    may: [oracle.order.PlaceOrder]\n"
    );
    let after_text = replace_one(
        &before_text,
        "may: [oracle.order.PlaceOrder]",
        "may: [oracle.order.PlaceOrder, oracle.order.HoldOrder]",
    );
    let before = model(INTERACTIONS, &before_text, DISPATCH);
    let after = model(INTERACTIONS, &after_text, DISPATCH);
    let command = "oracle.order.HoldOrder".parse().unwrap();
    assert!(!before.raw_order.actors[0].may.contains(&command));
    assert!(after.raw_order.actors[0].may.contains(&command));
    assert_eq!(before.raw_order.actors[0].may.len(), 1);
    assert_eq!(after.raw_order.actors[0].may.len(), 2);
    let a = before.specification.actors().values().next().unwrap();
    let b = after.specification.actors().values().next().unwrap();
    assert!(!a.may_invoke(&command));
    assert!(b.may_invoke(&command));
    let a = before.ir.actors().values().next().unwrap();
    let b = after.ir.actors().values().next().unwrap();
    assert_eq!(
        a.may.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["oracle.order.PlaceOrder"]
    );
    assert_eq!(
        b.may.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["oracle.order.HoldOrder", "oracle.order.PlaceOrder"]
    );
    changed_model_same_cli(&before, &after);
}

#[test]
fn actor_owner_identity_and_naming_change_without_cli_effect() {
    let order = format!(
        "{ORDER}\nactors:\n  - name: oracle.order.Clerk\n    naming: {{wire: clerk, display: Clerk}}\n    may: [oracle.dispatch.Handoff]\n"
    );
    let dispatch = format!(
        "{DISPATCH}\nactors:\n  - name: oracle.dispatch.Carrier\n    naming: {{wire: carrier, display: Carrier}}\n    may: [oracle.dispatch.Handoff]\n"
    );
    let before = model(INTERACTIONS, &order, DISPATCH);
    let after = model(INTERACTIONS, ORDER, &dispatch);
    assert!(before.raw_dispatch.actors.is_empty());
    assert!(after.raw_order.actors.is_empty());
    let raw_a = &before.raw_order.actors[0];
    let raw_b = &after.raw_dispatch.actors[0];
    assert_eq!(raw_a.name.to_string(), "oracle.order.Clerk");
    assert_eq!(raw_b.name.to_string(), "oracle.dispatch.Carrier");
    assert_ne!(raw_a.naming, raw_b.naming);
    assert_eq!(raw_a.naming.wire.as_deref(), Some("clerk"));
    assert_eq!(raw_b.naming.wire.as_deref(), Some("carrier"));
    for (witness, raw, domain) in [
        (&before, raw_a, "oracle.order"),
        (&after, raw_b, "oracle.dispatch"),
    ] {
        let assembled = witness.specification.actors().values().next().unwrap();
        let resolved = witness.ir.actors().values().next().unwrap();
        assert_eq!(assembled.name, raw.name);
        assert_eq!(assembled.naming, raw.naming);
        assert_eq!(resolved.name, raw.name);
        assert_eq!(resolved.naming, raw.naming);
        assert_eq!(resolved.domain.to_string(), domain);
    }
    changed_model_same_cli(&before, &after);
}

#[test]
fn actor_empty_and_duplicate_grants_normalize_without_cli_effect() {
    let omitted = format!("{ORDER}\nactors:\n  - name: oracle.order.Clerk\n");
    let explicit = format!("{omitted}    may: []\n    naming: {{}}\n");
    assert_ne!(omitted, explicit);
    let before = model(INTERACTIONS, &omitted, DISPATCH);
    let after = model(INTERACTIONS, &explicit, DISPATCH);
    assert!(before.raw_order.actors[0].may.is_empty());
    assert!(after.raw_order.actors[0].may.is_empty());
    assert_eq!(
        before.raw_order.actors[0].naming,
        after.raw_order.actors[0].naming
    );
    assert_eq!(before.specification, after.specification);
    assert_eq!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    unchanged_cli(&before, &after);

    let once = format!("{omitted}    may: [oracle.order.PlaceOrder]\n");
    let twice = format!("{omitted}    may: [oracle.order.PlaceOrder, oracle.order.PlaceOrder]\n");
    assert_ne!(once, twice);
    let before = model(INTERACTIONS, &once, DISPATCH);
    let after = model(INTERACTIONS, &twice, DISPATCH);
    assert_eq!(before.raw_order.actors[0].may.len(), 1);
    assert_eq!(
        before.raw_order.actors[0].may,
        after.raw_order.actors[0].may
    );
    assert_eq!(before.specification, after.specification);
    assert_eq!(before.ir.to_canonical_json(), after.ir.to_canonical_json());
    unchanged_cli(&before, &after);
}

fn instance<'a>(model: &'a Model, command: &str) -> &'a ResolvedInstance {
    &model
        .ir
        .commands()
        .values()
        .find(|c| c.name.to_string() == command)
        .unwrap()
        .outcomes[0]
        .subject
        .as_ref()
        .unwrap()
        .instance
}

#[test]
fn observed_and_supplied_instance_sources_change_without_cli_effect() {
    let observed_fields = replace_one(
        ORDER,
        "  - name: oracle.order.OrderPlaced\n    fields:\n",
        "  - name: oracle.order.OrderPlaced\n    fields:\n      - {name: alternate_order_id, type: oracle.order.OrderId}\n",
    );
    let observed_other = replace_one(
        &observed_fields,
        "creates: oracle.order.Order\n        instance: order_id",
        "creates: oracle.order.Order\n        instance: alternate_order_id",
    );
    let before = model(INTERACTIONS, &observed_fields, DISPATCH);
    let after = model(INTERACTIONS, &observed_other, DISPATCH);
    assert_eq!(
        before.raw_order.commands[0].outcomes[0].instance.as_deref(),
        Some("order_id")
    );
    assert_eq!(
        after.raw_order.commands[0].outcomes[0].instance.as_deref(),
        Some("alternate_order_id")
    );
    for (witness, expected) in [(&before, "order_id"), (&after, "alternate_order_id")] {
        let assembled = &witness.specification.commands()
            [&"oracle.order.PlaceOrder".parse().unwrap()]
            .outcomes[0];
        assert_eq!(assembled.subject.as_ref().unwrap().instance, expected);
        let ResolvedInstance::Observed { event, field } =
            instance(witness, "oracle.order.PlaceOrder")
        else {
            panic!("observed instance")
        };
        assert_eq!(event.to_string(), "oracle.order.OrderPlaced");
        assert_eq!(field.name, expected);
    }
    changed_model_same_cli(&before, &after);

    let event_order = observed_fields.replace(
        "oracle.order.OrderPlaced",
        "oracle.order.InitialOrderPlaced",
    );
    let event_interactions = INTERACTIONS.replace(
        "oracle.order.OrderPlaced",
        "oracle.order.InitialOrderPlaced",
    );
    let after = model(&event_interactions, &event_order, DISPATCH);
    let ResolvedInstance::Observed { event, field } = instance(&after, "oracle.order.PlaceOrder")
    else {
        panic!("observed instance")
    };
    assert_eq!(event.to_string(), "oracle.order.InitialOrderPlaced");
    assert_eq!(field.name, "order_id");
    assert_ne!(
        instance(&before, "oracle.order.PlaceOrder").event(),
        instance(&after, "oracle.order.PlaceOrder").event()
    );
    changed_model_same_cli(&before, &after);

    let supplied_fields = replace_one(
        ORDER,
        "    input:\n      - name: order_id\n        type: oracle.order.OrderId\n      - name: weight_grams",
        "    input:\n      - {name: alternate_order_id, type: oracle.order.OrderId}\n      - name: order_id\n        type: oracle.order.OrderId\n      - name: weight_grams",
    );
    let supplied_other = replace_one(
        &supplied_fields,
        "updates: oracle.order.Order\n        instance: order_id",
        "updates: oracle.order.Order\n        instance: alternate_order_id",
    );
    let before = model(INTERACTIONS, &supplied_fields, DISPATCH);
    let after = model(INTERACTIONS, &supplied_other, DISPATCH);
    assert_eq!(
        before.raw_order.commands[1].outcomes[0].instance.as_deref(),
        Some("order_id")
    );
    assert_eq!(
        after.raw_order.commands[1].outcomes[0].instance.as_deref(),
        Some("alternate_order_id")
    );
    for (witness, expected) in [(&before, "order_id"), (&after, "alternate_order_id")] {
        let assembled = &witness.specification.commands()
            [&"oracle.order.AmendOrder".parse().unwrap()]
            .outcomes[0];
        assert_eq!(assembled.subject.as_ref().unwrap().instance, expected);
        let ResolvedInstance::Supplied { field } = instance(witness, "oracle.order.AmendOrder")
        else {
            panic!("supplied instance")
        };
        assert_eq!(field.name, expected);
        assert!(instance(witness, "oracle.order.AmendOrder")
            .event()
            .is_none());
    }
    changed_model_same_cli(&before, &after);
}
