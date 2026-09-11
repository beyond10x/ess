//! Accessor observations preserve wire presence and refuse unavailable native information.
use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_conformance::accessor::{Expected, Observation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn fixture() -> EssIr {
    fixture_text(include_str!(
        "../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml"
    ))
}
fn fixture_text(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("accessor.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("accessor.yaml", text);
    compile_locating(&spec, &sources, &["accessor.yaml"]).unwrap()
}
fn observation(name: &str) -> Result<Observation, String> {
    let ir = fixture();
    let mapping = ir
        .bindings()
        .values()
        .next()
        .unwrap()
        .mapping
        .iter()
        .find(|m| m.target == name)
        .unwrap();
    let ess_compiler::ir::ResolvedMappingValue::EventAccessor { plan, types, .. } = &mapping.value
    else {
        panic!("accessor")
    };
    Observation::of(&ir, plan, types, &mapping.target_type)
}
fn evaluate(name: &str, value: serde_json::Value) -> Result<Expected, String> {
    observation(name)?.evaluate(&serde_json::from_value(value).unwrap())
}
#[test]
fn absent_traversal_and_missing_terminal_are_different() {
    assert_eq!(
        evaluate("partial", serde_json::json!({})).unwrap(),
        Expected::Absent
    );
    assert!(evaluate("text", serde_json::json!({"data":{}})).is_err());
    assert_eq!(
        evaluate("deeper", serde_json::json!({"data":{}})).unwrap(),
        Expected::Present(Node::Null)
    );
}
#[test]
fn terminal_nested_optional_equal_target_refuses_but_deeper_target_is_observable() {
    assert!(observation("nested")
        .unwrap_err()
        .contains("AmbiguousOptionalObservation"));
    assert_eq!(
        evaluate("deeper", serde_json::json!({"data":{"nested":null}})).unwrap(),
        Expected::Present(Node::Null)
    );
    assert_eq!(
        evaluate("deeper", serde_json::json!({"data":{"nested":"hello"}})).unwrap(),
        Expected::Present(Node::Text("hello".into()))
    );
}
#[test]
fn unavailable_union_branch_still_checks_its_declared_payload_kind() {
    assert_eq!(
        evaluate(
            "choice",
            serde_json::json!({"choice":{"kind":"gone","value":"gone"}})
        )
        .unwrap(),
        Expected::Absent
    );
    for choice in [
        serde_json::json!({"kind":"gone","value":3}),
        serde_json::json!({"kind":"gone"}),
        serde_json::json!({"kind":"invented","value":{}}),
        serde_json::json!({"kind":"ready","value":null}),
    ] {
        assert!(evaluate("choice", serde_json::json!({"choice":choice})).is_err());
    }
}
#[test]
fn observations_use_declared_names_without_wire_alias_fallback() {
    assert_eq!(
        evaluate("text", serde_json::json!({"data":{"status":"yes"}})).unwrap(),
        Expected::Present(Node::Text("yes".into()))
    );
    assert!(evaluate(
        "text",
        serde_json::json!({"data":{"upstream_status":"yes"}})
    )
    .is_err());
}
#[test]
fn hidden_nested_optional_whole_leaf_refuses() {
    assert!(observation("whole")
        .unwrap_err()
        .contains("AmbiguousOptionalObservation"));
}

#[test]
fn executable_accessors_select_new_suite_and_roundtrip_closed_admission() {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml")
        .replace("Optional<Optional<String>>", "Optional<String>");
    let ir = fixture_text(&text);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert_eq!(
        synthesis.suite.provenance.suite_version.major(),
        6,
        "{:?}",
        synthesis.refusals
    );
    let json = synthesis.suite.to_canonical_json().unwrap();
    let admitted = ess_conformance::admission::AdmittedSuite::from_json(&json).unwrap();
    assert_eq!(admitted.suite(), &synthesis.suite);
    for version in [4, 5] {
        let downgraded = json.replace("ess-conformance/6", &format!("ess-conformance/{version}"));
        assert!(ess_conformance::admission::AdmittedSuite::from_json(&downgraded).is_err());
    }
    if let Some(directory) = std::env::var_os("ESS_ACCESSOR_GO_OUT") {
        let base = std::path::PathBuf::from(directory);
        for artifact in ess_conformance::go::emit(&synthesis.suite).unwrap() {
            let path = base.join(artifact.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, artifact.contents).unwrap();
        }
        std::fs::write(
            base.join("go.mod"),
            "module example.invalid/accessorconformance\n\ngo 1.21\n",
        )
        .unwrap();
    }
}

#[test]
fn nominal_observation_facts_are_rechecked_at_the_persisted_boundary() {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml")
        .replace("Optional<Optional<String>>", "Optional<String>");
    let ir = fixture_text(&text);
    let mapping = ir
        .bindings()
        .values()
        .next()
        .unwrap()
        .mapping
        .iter()
        .find(|m| m.target == "whole")
        .unwrap();
    let ess_compiler::ir::ResolvedMappingValue::EventAccessor { plan, types, .. } = &mapping.value
    else {
        panic!("accessor")
    };
    let original =
        serde_json::to_value(Observation::of(&ir, plan, types, &mapping.target_type).unwrap())
            .unwrap();
    for members in [
        serde_json::json!(["Optional<Optional<String>>"]),
        serde_json::json!(["projection.core.Body"]),
    ] {
        let mut value = original.clone();
        value["types"]["nodes"]["projection.core.Body"]["members"] = members;
        let error = serde_json::from_value::<Observation>(value).unwrap_err();
        assert!(
            error.to_string().contains("AmbiguousOptionalObservation"),
            "{error}"
        );
    }
    let mut value = original.clone();
    value["types"]["nodes"] = serde_json::json!({});
    assert!(serde_json::from_value::<Observation>(value)
        .unwrap_err()
        .to_string()
        .contains("missing nominal"));
    let mut value = original;
    value["types"]["nodes"]["projection.core.Unused"] = serde_json::json!({"kind":"enum"});
    assert!(serde_json::from_value::<Observation>(value)
        .unwrap_err()
        .to_string()
        .contains("unreachable"));
}

#[test]
fn coverage_refusals_select_seven_and_exact_filtered_lineage_cannot_downgrade() {
    use ess_conformance::coverage::{AdmittedInput, Origins, Scope};
    let ir = fixture(); // The equal nested-Optional mapping has an explicit capability refusal.
    let input = ess_conformance::coverage_build::build(&ir, &[], Scope::System, Origins::Generated)
        .unwrap();
    assert_eq!(input.selected().suite().provenance.suite_version.major(), 7);
    assert!(input
        .selected()
        .coverage()
        .unwrap()
        .refused
        .iter()
        .any(|r| r.code == "ESS-SYNTH-015"));
    assert!(!ess_conformance::accessor::used_by(
        input.selected().suite()
    ));
    let selected = input.select(&[]).unwrap();
    assert_eq!(
        selected.selected().suite().provenance.suite_version.major(),
        7
    );
    assert_eq!(
        selected.parents()[0].original_json(),
        input.selected().original_json()
    );
    let run = ess_conformance::Runner::for_suite(selected.selected().suite()).run_admitted(
        selected.selected(),
        &ess_conformance::reference::Billing::new(),
    );
    let report = ess_conformance::CountReport::from_run(&run, selected.selected()).unwrap();
    let encoded = report.to_canonical_json().unwrap();
    assert_eq!(
        ess_conformance::CountReport::from_json(&encoded, selected.selected()).unwrap(),
        report
    );
    assert_eq!(report.counts().total, 0);
    assert_eq!(
        report.conformance_status(),
        ess_conformance::CountStatus::Inconclusive
    );
    assert!(encoded.contains("ess-conformance/7"));
    assert!(encoded.contains("ESS-SYNTH-015"));
    let detailed = ess_conformance::CountRun::from_run(&run, selected.selected()).unwrap();
    assert_eq!(
        ess_conformance::CountRun::from_json(
            &detailed.to_canonical_json().unwrap(),
            selected.selected()
        )
        .unwrap(),
        detailed
    );
    let mut document = selected.document();
    document.suite_json = document
        .suite_json
        .replace("ess-conformance/7", "ess-conformance/5");
    assert!(AdmittedInput::from_json(&document.to_canonical_json().unwrap()).is_err());
    let downgraded = input
        .selected()
        .original_json()
        .replace("ess-conformance/7", "ess-conformance/5");
    assert!(ess_conformance::AdmittedSuite::from_json(&downgraded).is_err());
    if let Some(base) = std::env::var_os("ESS_ACCESSOR_GO_OUT") {
        let base = std::path::PathBuf::from(base);
        std::fs::write(
            base.join("coverage-input.json"),
            input.document().to_canonical_json().unwrap(),
        )
        .unwrap();
        std::fs::write(
            base.join("filtered-input.json"),
            selected.document().to_canonical_json().unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn generated_go_executes_accessor_admission_and_presence_faults() {
    use ess_conformance::coverage::{Origins, Scope};
    let directory = std::env::temp_dir().join(format!("ess-accessor-go-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("essconform")).unwrap();
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml")
        .replace("Optional<Optional<String>>", "Optional<String>");
    let suite = ess_conformance::synthesize::synthesize(&fixture_text(&text)).suite;
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        std::fs::write(directory.join(artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/accessorconformance\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform/accessor_test.go"),
        include_str!("fixtures/accessor-runtime.go"),
    )
    .unwrap();
    let coverage =
        ess_conformance::coverage_build::build(&fixture(), &[], Scope::System, Origins::Generated)
            .unwrap();
    std::fs::write(
        directory.join("coverage-input.json"),
        coverage.document().to_canonical_json().unwrap(),
    )
    .unwrap();
    std::fs::write(
        directory.join("filtered-input.json"),
        coverage
            .select(&[])
            .unwrap()
            .document()
            .to_canonical_json()
            .unwrap(),
    )
    .unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestAccessor",
            "-count=1",
            "-v",
        ])
        .current_dir(&directory)
        .output()
        .expect("required Go toolchain");
    let log = format!(
        "exit: {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(directory.join("go.log"), &log).unwrap();
    assert!(output.status.success(), "{log}");
}

fn nullable_union_fixture() -> String {
    include_str!("../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml")
        .replace("Optional<Optional<String>>", "Optional<String>")
        .replace(
            "  - name: projection.core.Choice",
            "  - name: projection.core.NullableBody\n    kind: newtype\n    of: Optional<projection.core.Body>\n  - name: projection.core.Choice",
        )
        .replace("ready: projection.core.Body", "ready: projection.core.NullableBody")
}

fn observation_from_ir(ir: &EssIr, name: &str) -> Observation {
    let mapping = ir
        .bindings()
        .values()
        .next()
        .unwrap()
        .mapping
        .iter()
        .find(|mapping| mapping.target == name)
        .unwrap();
    let ess_compiler::ir::ResolvedMappingValue::EventAccessor { plan, types, .. } = &mapping.value
    else {
        panic!("expected compiled accessor");
    };
    Observation::of(ir, plan, types, &mapping.target_type).unwrap()
}

#[test]
fn adversary_nullable_newtype_union_payload_is_unavailable_not_malformed() {
    let ir = fixture_text(&nullable_union_fixture());
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    assert!(ess_conformance::accessor::used_by(&suite));
    ess_conformance::AdmittedSuite::from_json(&suite.to_canonical_json().unwrap()).unwrap();
    let observation = observation_from_ir(&ir, "choice");
    // Missing content and wrong representation remain malformed even for a nullable newtype.
    for payload in [
        serde_json::json!({"choice":{"kind":"ready"}}),
        serde_json::json!({"choice":{"kind":"ready","value":3}}),
    ] {
        assert!(observation
            .evaluate(&serde_json::from_value(payload).unwrap())
            .is_err());
    }
    let payload = serde_json::from_value(serde_json::json!({
        "choice":{"kind":"ready","value":null}
    }))
    .unwrap();
    assert_eq!(observation.evaluate(&payload), Ok(Expected::Absent));
}

fn run_adversary_go(ir: &EssIr, case: &str, test: &str) {
    let suite = ess_conformance::synthesize::synthesize(ir).suite;
    assert!(ess_conformance::accessor::used_by(&suite));
    ess_conformance::AdmittedSuite::from_json(&suite.to_canonical_json().unwrap()).unwrap();
    let directory = std::env::temp_dir().join(format!(
        "ess-accessor-adversary-{case}-{}",
        std::process::id()
    ));
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/accessoradversary\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(directory.join("essconform/adversary_test.go"), test).unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestAdversary$",
            "-count=1",
            "-v",
        ])
        .current_dir(&directory)
        .output()
        .expect("required Go toolchain");
    let log = format!(
        "exit: {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(directory.join("go.log"), &log).unwrap();
    assert!(output.status.success(), "{log}");
}

#[test]
fn adversary_go_nullable_newtype_union_payload_is_unavailable() {
    run_adversary_go(
        &fixture_text(&nullable_union_fixture()),
        "nullable",
        r#"
package essconform
import "testing"
func TestAdversary(t *testing.T) {
    suite, err := admitRunInput(suiteJSON)
    if err != nil { t.Fatal(err) }
    for _, scenario := range suite.Scenarios {
        for _, step := range scenario.Steps {
            if v, ok := step.Input["choice"]; ok && v.Accessor != nil {
                value, present, err := v.Accessor.evaluate(map[string]Node{
                    "choice": map[string]Node{"kind":"ready", "value":nil},
                })
                if err != nil || present || value != nil { t.Fatalf("got %v %v %v", value, present, err) }
                return
            }
        }
    }
    t.Fatal("no synthesized choice accessor")
}
"#,
    );
}

#[test]
fn adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit() {
    // JSON's U+2028 escape is six bytes, but its canonical UTF-8 encoding is three.
    // Authored field metadata is part of the persisted root, so this is compiler-produced input.
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml")
        .replace("Optional<Optional<String>>", "Optional<String>")
        .replace(
            "      - name: data\n",
            &format!(
                "      - name: data\n        summary: \"{}\"\n",
                "\\u2028".repeat(180_000)
            ),
        );
    let ir = fixture_text(&text);
    let bytes = observation_from_ir(&ir, "text").bytes().unwrap();
    assert!(
        (540_000..600_000).contains(&bytes),
        "unexpected compact byte count: {bytes}"
    );
    run_adversary_go(
        &ir,
        "unicode",
        r#"
package essconform
import "testing"
func TestAdversary(t *testing.T) {
    if _, err := admitRunInput(suiteJSON); err != nil {
        t.Fatalf("Rust-admitted, compiler-produced suite refused by Go: %v", err)
    }
}

"#,
    );
}

#[test]
fn adversary_second_go_exact_canonical_escape_boundaries() {
    use std::fmt::Write as _;

    let ir = fixture_text(&nullable_union_fixture());
    let mut seed = serde_json::to_value(observation_from_ir(&ir, "text")).unwrap();
    seed["plan"]["root"]["summary"] = serde_json::json!("");
    let baseline = serde_json::to_vec(&seed).unwrap().len();
    let mut cases = String::new();
    for pattern in [
        "<>&",
        "\u{2028}\u{2029}",
        "\\u2028\\u2029",
        "\\\u{2028}\\\u{2029}",
        "\"\\\n\t<>&\u{2028}\\u2029",
    ] {
        let repeated = pattern.repeat(20_000);
        seed["plan"]["root"]["summary"] = serde_json::json!(repeated);
        let repeated_bytes = serde_json::to_vec(&seed).unwrap().len() - baseline;
        for over in [false, true] {
            let padding = 1_048_576 - baseline - repeated_bytes + usize::from(over);
            seed["plan"]["root"]["summary"] =
                serde_json::json!(format!("{}{}", repeated, "a".repeat(padding)));
            let want = serde_json::to_vec(&seed).unwrap().len();
            assert_eq!(want, 1_048_576 + usize::from(over));
            assert_eq!(
                serde_json::from_value::<Observation>(seed.clone()).is_ok(),
                !over
            );
            writeln!(
                cases,
                "{{pattern: {}, padding: {padding}, want: {want}, admitted: {}}},",
                serde_json::to_string(pattern).unwrap(),
                !over
            )
            .unwrap();
        }
    }
    seed["plan"]["root"]["summary"] = serde_json::json!("");
    let test = r#"
package essconform
import ("bytes"; "encoding/json"; "strings"; "testing")
func TestAdversary(t *testing.T) {
    cases := []struct{ pattern string; padding, want int; admitted bool }{
        __CASES__
    }
    for i, c := range cases {
        var value map[string]any
        if err := json.Unmarshal([]byte(`__SEED__`), &value); err != nil { t.Fatal(err) }
        value["plan"].(map[string]any)["root"].(map[string]any)["summary"] = strings.Repeat(c.pattern, 20000) + strings.Repeat("a", c.padding)
        var raw bytes.Buffer
        enc := json.NewEncoder(&raw); enc.SetEscapeHTML(false)
        if err := enc.Encode(value); err != nil { t.Fatal(err) }
        if got := accessorCompactBytes(bytes.TrimSuffix(raw.Bytes(), []byte("\n"))); got != c.want {
            t.Fatalf("case %d canonical count %d, Rust %d", i, got, c.want)
        }
        _, err := admitAccessor(value)
        if (err == nil) != c.admitted { t.Fatalf("case %d admitted=%v want=%v: %v", i, err == nil, c.admitted, err) }
        t.Logf("case %d canonical=%d admitted=%v", i, c.want, err == nil)
    }
}
"#
    .replace("__CASES__", &cases)
    .replace("__SEED__", &serde_json::to_string(&seed).unwrap());
    run_adversary_go(&ir, "second-escape-boundaries", &test);
}
