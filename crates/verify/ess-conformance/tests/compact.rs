//! Native compact compilation retains real captures and refuses unsupported authoring.
use ess_compiler::{compile, source::SourceMap, EssIr};
use ess_composition::{CompositionSpec, ServiceImportSpec, ServiceKey};
use ess_conformance::{
    authored::Source, compact, models::Models, recipes::Library, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source as ModelSource,
};

const DOMAIN: &str = "domain: fixture.routing
types:
  - name: fixture.routing.State
    kind: enum
    variants: [queued, bridged]
  - name: fixture.routing.Item
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: agent, type: String}
      - {name: state, type: fixture.routing.State}
events:
  - name: fixture.routing.Created
    fields: [{name: id, type: String}]
  - name: fixture.routing.Changed
    fields: [{name: item, type: fixture.routing.Item}]
commands:
  - name: fixture.routing.Create
    input: [{name: name, type: String}]
    response: [{name: id, type: String}]
    outcomes:
      - name: ready
        emits: [fixture.routing.Created]
        payload:
          fixture.routing.Created: {id: {response: id}}
";
const RECIPE: &str = "type: ess-fixture/1
recipe: participant
parameters:
  name: {service: fixture, command: fixture.routing.Create, field: name}
steps:
  - do: command
    service: fixture
    command: fixture.routing.Create
    input: {name: {$parameter: name}}
    capture: {id: id}
outputs: {id: id}
";
const SCENARIO: &str = "type: ess-scenario/3
domain: fixture.routing
scenario: actual-identities
summary: The actual request is routed to its participant
given:
  - {recipe: participant, as: first, with: {name: first}}
  - {recipe: participant, as: second, with: {name: {$capture: first.id}}}
when:
  - do: command
    service: fixture
    command: fixture.routing.Create
    input: {name: request}
    capture: {request: id}
then:
  - service: fixture
    event: fixture.routing.Changed
    matches:
      item.id: {$capture: request}
      item.agent: {$capture: second.id}
      item.state: bridged
";

fn model() -> EssIr {
    model_with_domain(DOMAIN)
}

fn model_with_domain(domain: &str) -> EssIr {
    let files = [
        ("system.yaml", "format: ess/4\nsystem: fixture\nversion: v1\ndomains: [fixture.routing]\n"),
        ("routing.yaml", domain),
        ("components.yaml", "components:\n  - component: fixture\n    owns: {domains: [fixture.routing]}\n    accepts: {commands: [fixture.routing.Create]}\n    publishes: {events: [fixture.routing.Created, fixture.routing.Changed]}\n"),
    ];
    let spec = Specification::assemble(
        files.map(|(name, text)| (ModelSource::new(name), RawSpecFile::parse(text).unwrap())),
    )
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn original_enum_variants_control_compact_payload_admission() {
    let compile_source = |ir: &EssIr, source: &str| {
        let key = ServiceKey::new("fixture").unwrap();
        let composition = CompositionSpec::new(
            "live".parse().unwrap(),
            vec![ServiceImportSpec::of(
                key.clone(),
                "fixture".parse().unwrap(),
                ir,
            )],
            vec![],
        );
        let models = Models::compile(&composition, &[(key, ir)]).unwrap();
        let library = Library::parse(&[Source::new("participant.yaml", RECIPE)]).unwrap();
        compact::compile(&models, &library, &[Source::new("routing.yaml", source)])
    };
    let original = model();
    let changed = model_with_domain(&DOMAIN.replace("[queued, bridged]", "[queued, connected]"));
    assert_ne!(original.to_canonical_json(), changed.to_canonical_json());
    let accepted = compile_source(&original, SCENARIO).unwrap();
    let refusal = compile_source(&changed, SCENARIO).unwrap_err();
    assert!(
        refusal.contains("fixture.routing.Changed: literal violates item.state"),
        "{refusal}"
    );
    let repaired = compile_source(
        &changed,
        &SCENARIO.replace("item.state: bridged", "item.state: connected"),
    )
    .unwrap();
    assert_ne!(accepted.manifest, repaired.manifest);
    assert_ne!(accepted.input.document(), repaired.input.document());
}

#[test]
fn fixtures_and_stimulus_lower_to_actual_response_slots_deterministically() {
    let ir = model();
    let key = ServiceKey::new("fixture").unwrap();
    let composition = CompositionSpec::new(
        "live".parse().unwrap(),
        vec![ServiceImportSpec::of(
            key.clone(),
            "fixture".parse().unwrap(),
            &ir,
        )],
        vec![],
    );
    let models = Models::compile(&composition, &[(key, &ir)]).unwrap();
    let library = Library::parse(&[Source::new("participant.yaml", RECIPE)]).unwrap();
    let sources = [Source::new("routing.yaml", SCENARIO)];
    let result = compact::compile(&models, &library, &sources).unwrap();
    let again = compact::compile(&models, &library, &sources).unwrap();
    assert_eq!(result.input.document(), again.input.document());
    assert_eq!(result.manifest, again.manifest);
    let changed_recipe = Library::parse(&[Source::new(
        "participant.yaml",
        format!("# reviewed fixture source\n{RECIPE}"),
    )])
    .unwrap();
    let changed = compact::compile(&models, &changed_recipe, &sources).unwrap();
    assert_ne!(result.manifest, changed.manifest);
    assert_ne!(
        result
            .input
            .selected()
            .suite()
            .provenance
            .live_inputs_digest,
        changed
            .input
            .selected()
            .suite()
            .provenance
            .live_inputs_digest,
        "the input must bind the exact source, including comments outside semantic IR"
    );
    let second = Source::new(
        "second.yaml",
        SCENARIO.replace("scenario: actual-identities", "scenario: another-routing"),
    );
    let ordered =
        compact::compile(&models, &library, &[sources[0].clone(), second.clone()]).unwrap();
    let reversed = compact::compile(&models, &library, &[second, sources[0].clone()]).unwrap();
    assert_eq!(ordered.input.document(), reversed.input.document());
    assert_eq!(ordered.manifest, reversed.manifest);
    let suite = result.input.selected();
    assert_eq!(suite.suite().provenance.suite_version.major(), 11);
    assert!(suite.coverage().unwrap().is_complete());
    assert_eq!(suite.coverage().unwrap().authored.len(), 1);
    let scenario = suite.suite().scenarios.values().next().unwrap();
    assert_eq!(scenario.steps.len(), 7);
    let ScenarioStep::ExecuteCommand { input, .. } = &scenario.steps[2] else {
        panic!()
    };
    assert!(matches!(input["name"], ScenarioValue::Instance { .. }));
    assert_eq!(result.locations.values().next().unwrap()[0].len(), 2);
    assert_eq!(result.recipes.len(), 1);
    for bad in [
        SCENARIO.replace("type: ess-scenario/3", "type: ess-scenario/2"),
        SCENARIO.replace("as: second", "as: first"),
        SCENARIO.replace("item.agent:", "item.unknown:"),
        SCENARIO.replace("item.state: bridged", "item.state: invented"),
        SCENARIO.replace("{$capture: request}", "{$capture: later}"),
        SCENARIO.replace("input: {name: request}", "input: {unknown: request}"),
        SCENARIO.replace("then:", "snippet: forbidden\nthen:"),
        SCENARIO.replace(
            "item.id: {$capture: request}",
            "item.id: {$parameter: request}",
        ),
    ] {
        assert!(
            compact::compile(&models, &library, &[Source::new("bad.yaml", bad.clone())]).is_err(),
            "{bad}"
        );
    }
}
