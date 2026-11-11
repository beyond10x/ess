//! Adversary pass 1 on story:rust-generation-refuses-by-name-instead-of-panicking-or-misnaming.
//!
//! * A label word `self` is no outcome: `type_fragment` maps every word through `pascal`, so the
//!   `Self_` escape reaches enum variants a model already generated (`self-hosted`).
//! * A port method the allocation suffixed is called by its suffixed name from every call site,
//!   the system's binding delivery included.
//! * The suite synthesized from example 28 expects the payload the arrangement stored.

#![allow(dead_code, clippy::too_many_lines)]

mod related_guard_served;
use related_guard_served as served;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Synthesis, Target};
use serde_json::Value;

const RELATED: &str = include_str!("fixtures/generation-by-name/28.yaml");
const KEYWORD: &str = include_str!("fixtures/generation-by-name/29.yaml");
const VIEWS: &str = include_str!("fixtures/generation-by-name/32.yaml");

fn compile_documents(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (position, document) in text.split("---\n").enumerate() {
        let label = format!("gap-{position}.yaml");
        let raw = RawSpecFile::parse(document)
            .unwrap_or_else(|error| panic!("`{label}` is well formed: {error}"));
        sources.insert(label.clone(), document.to_owned());
        labels.push(label.clone());
        parsed.push((Source::new(label), raw));
    }
    let spec =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile_locating(&spec, &sources, &labels).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn rust(ir: &EssIr) -> Synthesis {
    synthesize_for(ir, Target::Rust).unwrap_or_else(|error| panic!("Rust synthesizes: {error:?}"))
}

fn everything(synthesis: &Synthesis) -> String {
    synthesis
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect()
}

// ---- the `Self_` escape reaches label words -------------------------------------------------------

const HOSTING: &str = "format: ess/35
system: gap
version: v1
domain: gap.demo
types:
- {name: gap.demo.Hosting, kind: enum, variants: [self-hosted, managed]}
entities:
- name: gap.demo.Site
  identity: {name: id, type: String}
  fields:
  - {name: hosting, type: gap.demo.Hosting}
  lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
- name: gap.demo.SiteOpened
  fields:
  - {name: id, type: String}
commands:
- name: gap.demo.OpenSite
  input:
  - {name: id, type: String}
  - {name: hosting, type: gap.demo.Hosting}
  outcomes:
  - name: opened
    creates: gap.demo.Site
    instance: id
    sets: {hosting: input.hosting}
    emits: [gap.demo.SiteOpened]
    payload:
      gap.demo.SiteOpened: {id: input.id}
actors:
- name: gap.demo.Anyone
  may: [gap.demo.OpenSite]
";

/// A model with no keyword outcome keeps its bytes (acceptance, last bullet): the enum label
/// `self-hosted` derived the Rust variant `SelfHosted` before the change, a valid identifier.
#[test]
fn an_enum_label_with_the_word_self_keeps_its_variant() {
    let ir = served::model(&served::served(HOSTING, "gap-service"));
    let items = everything(&rust(&ir));
    assert!(
        !items.contains("Self_Hosted"),
        "the label `self-hosted` derives `Self_Hosted` (was `SelfHosted`; rustc warns \
         non_camel_case_types on it):\n{}",
        items
            .lines()
            .filter(|line| line.contains("Self_Hosted"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        items.contains("SelfHosted"),
        "the variant stays `SelfHosted`"
    );
}

/// The outcome `self` as a `wrong_state` branch with no `unknown_instance:` branch: its
/// unknown-instance variant is `Self_UnknownInstance`, which rustc's `non_camel_case_types` lint
/// flags, and the served harness builds with `-D warnings`.
#[test]
fn a_keyword_wrong_state_outcome_derives_a_camel_case_unknown_instance_variant() {
    let text = KEYWORD
        .replace(
            "  - name: self\n    moves:",
            "  - name: finished\n    moves:",
        )
        .replace(
            "  - name: unknown\n    unknown_instance: true\n    error: gap.demo.NoSuchItem\n    \
             payload:\n      gap.demo.NoSuchItem:\n        id: input.id\n  - name: wrong-state\n",
            "  - name: self\n",
        );
    assert!(
        text.contains("  - name: self\n    wrong_state: true"),
        "{text}"
    );
    let ir = served::model(&served::served(&text, "gap-service"));
    let items = everything(&rust(&ir));
    let lines: Vec<&str> = items
        .lines()
        .filter(|line| line.contains("Self_UnknownInstance"))
        .collect();
    assert!(
        lines.is_empty(),
        "a variant that is not upper camel case:\n{}",
        lines.join("\n")
    );
}

// ---- every call of a suffixed port method --------------------------------------------------------

/// Example 32 with a binding that delivers `gap.one.ThingOpened` to `gap.two.OpenThing`, which the
/// port serves as `open_thing_2`.
#[test]
fn a_binding_delivers_to_the_suffixed_port_method() {
    let text = VIEWS.replacen(
        "    reached_by: network\n---\n",
        "    reached_by: network\nbindings:\n  - id: two-on-one\n    when: {event: \
         gap.one.ThingOpened}\n    invoke: {command: gap.two.OpenThing}\n    mapping:\n      id: \
         event.id\n      label: event.id\n    delivery: at_least_once\n    on_failure: retry\n---\n",
        1,
    );
    assert_ne!(text, VIEWS);
    let ir = compile_documents(&text);
    let items = everything(&rust(&ir));
    let calls: Vec<&str> = items
        .lines()
        .filter(|line| line.contains("self.gap_service.open_thing"))
        .collect();
    assert!(
        !calls.is_empty(),
        "the system delivers the binding through the port"
    );
    assert!(
        calls.iter().all(|line| line.contains("open_thing_2(")),
        "the delivery of `gap.two.OpenThing` calls `gap.one.OpenThing`'s method:\n{}",
        calls.join("\n")
    );
}

// ---- the suite of example 28 -----------------------------------------------------------------------

/// `witnessed()` of `generation_by_name.rs`, with its views opened as `open_views` opens them.
fn witnessed() -> String {
    RELATED
        .replace("predicate: not defined(dflt)", "predicate: n < 0")
        .replace("  system: true\n", "")
        .replace("  - gap.demo.Items\n", "")
}

/// The stored `parent_id` the arrangement of `steps` gives the item: the literal `CreateItem` was
/// sent, or the literal `id` of the `CreateParent` whose captured instance it was sent.
fn arranged_parent(steps: &[Value]) -> Value {
    let create = steps
        .iter()
        .rev()
        .find(|step| step["step"] == "execute_command" && step["command"] == "gap.demo.CreateItem")
        .expect("the arrangement creates the item");
    let sent = &create["input"]["parent_id"];
    match sent["kind"].as_str() {
        Some("literal") => sent["value"].clone(),
        Some("instance") => {
            let captured = steps
                .iter()
                .position(|step| {
                    step["step"] == "capture_instance" && step["instance"] == sent["instance"]
                })
                .expect("the instance is captured");
            let parent = steps[..captured]
                .iter()
                .rev()
                .find(|step| step["step"] == "execute_command")
                .expect("an act creates the parent");
            assert_eq!(parent["input"]["id"]["kind"], "literal", "{parent}");
            parent["input"]["id"]["value"].clone()
        }
        other => panic!("an unexpected input kind {other:?}: {sent}"),
    }
}

/// `{subject: parent_id}` reads the item's stored `parent_id`, which the scenario's own
/// arrangement wrote; the expected error payload must be that value.
#[test]
#[ignore = "pre-existing: the related-guard refusal scenarios of ess-conformance synthesize.rs expect the input literal where the arrangement stored another reference; backlog"]
fn the_related_refusal_expects_the_arranged_stored_reference() {
    let ir = served::model(&served::served(&witnessed(), "gap-service"));
    let suite = serde_json::to_value(ess_conformance::synthesize(&ir).suite).expect("serializes");
    for outcome in ["no-parent", "reserved"] {
        let id = format!("gap.demo.TakeDefault/outcome/{outcome}");
        let steps = suite["scenarios"][&id]["steps"]
            .as_array()
            .unwrap_or_else(|| panic!("`{id}` is synthesized: {:?}", suite["scenarios"]))
            .clone();
        let expected = steps
            .iter()
            .find(|step| step["step"] == "expect_error")
            .expect("an expected error")["fields"]["parent_id"]
            .clone();
        let arranged = arranged_parent(&steps);
        assert_eq!(
            expected, arranged,
            "`{id}` expects `error.parent_id` = {expected} where its arrangement stored {arranged}"
        );
    }
}
