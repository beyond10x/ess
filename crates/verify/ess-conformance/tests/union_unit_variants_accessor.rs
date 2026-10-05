//! An accessor reading through a union with a unit variant (ess/22, beyond10x/ess#418): the unit
//! variant carries nothing, so a member read through it is unavailable — and a content member
//! written beside its tag is malformed, never absence. The reference runner, the Go runtime and
//! the TypeScript runtime (`src/ts/runtime.test.ts`, over `fixtures/unit-variant-accessor.json`)
//! answer alike.

mod support_go;

use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_conformance::accessor::{Expected, Observation};
use ess_conformance::interpret::Interpreted;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The bounded-accessor fixture at ess/22, its `gone` variant carrying nothing. Synthesis
/// witnesses `gone`, the first variant, so the suite reads through the unit variant.
fn unit_fixture() -> String {
    include_str!("../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml")
        .replace("format: ess/3", "format: ess/22")
        .replace("Optional<Optional<String>>", "Optional<String>")
        .replace("      gone: String\n", "      gone:\n")
        .replace(
            "        emits: [projection.core.Arrived]\n",
            "        emits: [projection.core.Arrived]\n        payload:\n          \
             projection.core.Arrived:\n            data: {generated: true}\n            \
             partial: {generated: true}\n            choice: {generated: true}\n            \
             wrapped: {generated: true}\n",
        )
        .replace(
            "        emits: [projection.core.Done]\n",
            "        emits: [projection.core.Done]\n        payload:\n          \
             projection.core.Done:\n            result: {generated: true}\n",
        )
}

fn ir() -> EssIr {
    let text = unit_fixture();
    let spec = Specification::assemble([(
        Source::new("accessor.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("accessor.yaml", text.as_str());
    compile_locating(&spec, &sources, &["accessor.yaml"]).unwrap()
}

fn choice(ir: &EssIr) -> Observation {
    let mapping = ir
        .bindings()
        .values()
        .next()
        .unwrap()
        .mapping
        .iter()
        .find(|mapping| mapping.target == "choice")
        .unwrap();
    let ess_compiler::ir::ResolvedMappingValue::EventAccessor { plan, types, .. } = &mapping.value
    else {
        panic!("expected compiled accessor");
    };
    Observation::of(ir, plan, types, &mapping.target_type).unwrap()
}

fn evaluate(observation: &Observation, value: &serde_json::Value) -> Result<Expected, String> {
    observation.evaluate(&serde_json::from_value(serde_json::json!({ "choice": value })).unwrap())
}

const FIXTURE: &str = "tests/fixtures/unit-variant-accessor.json";

#[test]
fn the_reference_runner_reads_a_unit_variant_as_unavailable() {
    let observation = choice(&ir());
    assert_eq!(
        evaluate(&observation, &serde_json::json!({"kind": "gone"})),
        Ok(Expected::Absent)
    );
    assert_eq!(
        evaluate(
            &observation,
            &serde_json::json!({"kind": "ready", "value": {"status": "yes"}})
        ),
        Ok(Expected::Present(Node::Text("yes".to_owned())))
    );
    for malformed in [
        serde_json::json!({"kind": "gone", "value": "gone"}),
        serde_json::json!({"kind": "gone", "value": null}),
        serde_json::json!({"kind": "ready"}),
    ] {
        assert!(
            evaluate(&observation, &malformed).is_err(),
            "{malformed} is malformed, not absence"
        );
    }
    // The document the TypeScript runtime's case reads: the observation as the suite carries it.
    let written = serde_json::to_string_pretty(&observation).unwrap() + "\n";
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    if std::env::var_os("ESS_UNIT_ACCESSOR_FIXTURE").is_some_and(|value| value == "write") {
        std::fs::write(&path, &written).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap_or_default(),
        written,
        "{FIXTURE} is not the observation; regenerate with ESS_UNIT_ACCESSOR_FIXTURE=write"
    );
}

#[test]
fn the_suite_reads_through_the_unit_variant_and_passes_the_interpreter_in_rust_and_go() {
    let ir = ir();
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    // The fixture's at-most-once, dropping binding leaves its delivery and failure aspects
    // unobservable, as it does at ess/3; the invocation the accessor fills is witnessed.
    assert!(ess_conformance::accessor::used_by(&synthesis.suite));
    let verdicts = support_go::assert_parity(
        "unit-accessor",
        &synthesis.suite,
        Interpreted::for_model(ir.clone()),
    );
    assert_ne!(verdicts.len(), 0);
    assert_eq!(support_go::not_passed(&verdicts).len(), 0, "{verdicts:?}");
}

const GO_CASES: &str = r#"package essconform

import "testing"

func TestUnitVariantAccessor(t *testing.T) {
	suite, err := admitRunInput(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	var accessor *accessorObservation
	for _, scenario := range suite.Scenarios {
		for _, step := range scenario.Steps {
			if v, ok := step.Input["choice"]; ok && v.Accessor != nil {
				accessor = v.Accessor
			}
		}
	}
	if accessor == nil {
		t.Fatal("no synthesized choice accessor")
	}
	value, present, err := accessor.evaluate(map[string]Node{"choice": map[string]Node{"kind": "gone"}})
	if err != nil || present || value != nil {
		t.Fatalf("unit variant: got %v %v %v", value, present, err)
	}
	value, present, err = accessor.evaluate(map[string]Node{"choice": map[string]Node{"kind": "ready", "value": map[string]Node{"status": "yes"}}})
	if err != nil || !present || value != "yes" {
		t.Fatalf("payload variant: got %v %v %v", value, present, err)
	}
	for _, malformed := range []map[string]Node{
		{"kind": "gone", "value": "gone"},
		{"kind": "gone", "value": nil},
		{"kind": "ready"},
	} {
		if _, _, err := accessor.evaluate(map[string]Node{"choice": malformed}); err == nil {
			t.Fatalf("%v is malformed, not absence", malformed)
		}
	}
	t.Log("go accessor: unit variant unavailable, malformed shapes refused")
}
"#;

#[test]
fn the_go_runtime_reads_a_unit_variant_as_unavailable() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    let directory = support_go::package(
        "unit-accessor-cases",
        &synthesis.suite,
        &[("unit_accessor_test.go", GO_CASES)],
    );
    let run = support_go::go_test(&directory, "TestUnitVariantAccessor", &[]);
    let _ = std::fs::remove_dir_all(&directory);
    assert!(run.success, "{}", run.log);
    assert!(
        run.log
            .contains("go accessor: unit variant unavailable, malformed shapes refused"),
        "{}",
        run.log
    );
}
