//! The generated Rust and Go lanes and `distinct: {in, as, by}` (`docs/design/expression-family-source22.md`,
//! `distinct`): neither lane compares keys across a list's elements, so each refuses or owes the
//! construct by name rather than deciding a different rule. A command guard reading it leaves the
//! command's behaviour owed, a view filter leaves the view query owed, and an entity invariant
//! refuses the generated target, each naming `distinct`. The same model without it is generated.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str = "format: ess/22
system: pool
version: v1
domain: pool.files
entities:
  - name: pool.files.Bundle
    identity: {name: bundle_id, type: Uuid}
    fields:
      - {name: tags, type: List<String>}
    invariants:
      - INVARIANT
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
views:
  - name: pool.files.Bundles
    source: pool.files.Bundle
    consistency: read_your_writes
    filter: FILTER
    fields:
      - {name: bundle_id, type: Uuid}
events:
  - {name: pool.files.Repeated, fields: []}
  - {name: pool.files.Opened, fields: [{name: bundle_id, type: Uuid}]}
commands:
  - name: pool.files.Open
    input:
      - {name: tags, type: List<String>}
    outcomes:
      - name: repeated
        when: GUARD
        emits: [pool.files.Repeated]
      - name: opened
        creates: pool.files.Bundle
        instance: bundle_id
        sets: {tags: input.tags}
        emits: [pool.files.Opened]
        payload: {pool.files.Opened: {bundle_id: {generated: true}}}
components:
  - component: files-service
    owns: {domains: [pool.files]}
    accepts: {commands: [pool.files.Open]}
    publishes: {events: [pool.files.Repeated, pool.files.Opened]}
";

const DISTINCT: &str = "{distinct: {in: tags, as: tag}}";
const PLAIN: &str = "defined(bundle_id)";

fn model(guard: &str, invariant: &str, filter: &str) -> String {
    MODEL
        .replace("GUARD", guard)
        .replace("INVARIANT", invariant)
        .replace("FILTER", filter)
}

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("pool.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn owed(text: &str, target: Target, kind: CapabilityKind, source: &str) -> String {
    let synthesis = synthesize_for(&ir_of(text), target)
        .unwrap_or_else(|error| panic!("{target:?} synthesizes: {error:?}"));
    match synthesis.plan.disposition_of(kind, source) {
        Some(SynthesisDisposition::Obligation(obligation)) => {
            serde_json::to_string(&obligation.reason).unwrap()
        }
        other => format!("not owed: {other:?}"),
    }
}

#[test]
fn distinct_in_a_guard_leaves_the_behaviour_owed_by_name() {
    let text = model("{not: {distinct: {in: tags, as: tag}}}", PLAIN, PLAIN);
    for target in [Target::Rust, Target::Go] {
        let reason = owed(
            &text,
            target,
            CapabilityKind::CommandBehavior,
            "pool.files.Open",
        );
        assert!(
            reason.contains("distinct list members no generated behaviour compares"),
            "{target:?}: {reason}"
        );
    }
}

#[test]
fn distinct_in_a_view_filter_leaves_the_query_owed_by_name() {
    let text = model("tags.count > 9", PLAIN, DISTINCT);
    for target in [Target::Rust, Target::Go] {
        let reason = owed(
            &text,
            target,
            CapabilityKind::ViewQuery,
            "pool.files.Bundles",
        );
        assert!(reason.contains("distinct"), "{target:?}: {reason}");
    }
}

#[test]
fn distinct_in_an_entity_invariant_refuses_the_generated_target_by_name() {
    let text = model("tags.count > 9", DISTINCT, PLAIN);
    for target in [Target::Rust, Target::Go] {
        let refused = synthesize_for(&ir_of(&text), target)
            .err()
            .unwrap_or_else(|| panic!("{target:?}: a distinct invariant is refused"));
        let rendered = format!("{refused:?}");
        assert!(
            rendered.contains("requires distinct list members"),
            "{target:?}: {rendered}"
        );
    }
}

#[test]
fn distinct_the_same_model_without_it_is_generated() {
    let text = model("tags.count > 9", PLAIN, PLAIN);
    for target in [Target::Rust, Target::Go] {
        synthesize_for(&ir_of(&text), target)
            .unwrap_or_else(|error| panic!("{target:?}: {error:?}"));
    }
}
