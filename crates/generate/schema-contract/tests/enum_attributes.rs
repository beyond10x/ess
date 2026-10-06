//! Typed enum attributes (`ess/23`, beyond10x/ess#450) in the types-only Rust, Go and TypeScript
//! outputs: one accessor per attribute, answering each variant's value.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use schema_contract::realize::Plan;

const SOURCE: &str = r"format: ess/23
system: probe
version: v1
domains: [probe.rules]
domain: probe.rules
types:
  - name: probe.rules.Operator
    kind: enum
    attributes:
      - {name: takes_number, type: Boolean}
      - {name: arity, type: Optional<Integer>}
      - {name: label, type: String}
    variants:
      - {name: Contains, attributes: {takes_number: false, label: contains}}
      - {name: GreaterThan, wire: gt, attributes: {takes_number: true, arity: 2, label: '>'}}
";

fn plan() -> Plan {
    Plan::from_model(&model::selection(SOURCE, &["probe.rules.Operator"])).expect("plan")
}

#[test]
fn enum_attributes_generated_accessors() {
    // The wire values sort `Contains` before `gt`, so `V0` is `Contains` and `V1` is `gt`.
    let rust = plan().rust("probe-rules").expect("rust").declarations;
    for expected in [
        "impl ProbeRulesOperator {",
        "pub fn takes_number(&self) -> bool {",
        "Self::V0 => false,",
        "Self::V1 => true,",
        "pub fn arity(&self) -> Option<i64> {",
        "Self::V0 => None,",
        "Self::V1 => Some(2),",
        "pub fn label(&self) -> &'static str {",
        "Self::V1 => \">\",",
    ] {
        assert!(
            rust.contains(expected),
            "Rust is missing `{expected}`:\n{rust}"
        );
    }

    let go = plan()
        .go("rules", "example.com/rules")
        .expect("go")
        .declarations;
    for expected in [
        "func (v ProbeRulesOperator) TakesNumber() bool {",
        "case ProbeRulesOperatorV1:\n\t\treturn true",
        "func (v ProbeRulesOperator) Arity() (int64, bool) {",
        "case ProbeRulesOperatorV1:\n\t\treturn 2, true",
        "func (v ProbeRulesOperator) Label() string {",
    ] {
        assert!(go.contains(expected), "Go is missing `{expected}`:\n{go}");
    }

    let typescript = plan().typescript().declarations;
    for expected in [
        "export function probeRulesOperatorTakesNumber(value: ProbeRulesOperator): boolean {",
        "case \"gt\":\n      return true;",
        "export function probeRulesOperatorArity(value: ProbeRulesOperator): number | undefined {",
        "export function probeRulesOperatorLabel(value: ProbeRulesOperator): string {",
    ] {
        assert!(
            typescript.contains(expected),
            "TypeScript is missing `{expected}`:\n{typescript}"
        );
    }
}

#[test]
fn enum_without_attributes_keeps_its_bytes() {
    let bare = SOURCE
        .replace(
            "    attributes:\n      - {name: takes_number, type: Boolean}\n      - {name: arity, type: Optional<Integer>}\n      - {name: label, type: String}\n",
            "",
        )
        .replace(", attributes: {takes_number: false, label: contains}", "")
        .replace(", attributes: {takes_number: true, arity: 2, label: '>'}", "");
    let plan = Plan::from_model(&model::selection(&bare, &["probe.rules.Operator"])).expect("plan");
    let rust = plan.rust("probe-rules").expect("rust").declarations;
    assert!(!rust.contains("impl ProbeRulesOperator {"), "{rust}");
}

fn scratch(name: &str) -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("scratch");
    root
}

#[test]
fn enum_attribute_accessors_compile_and_answer() {
    let realization = plan().rust("probe-rules").expect("rust");
    let root = scratch("enum-attributes-rust");
    std::fs::create_dir_all(root.join("tests")).expect("tests dir");
    std::fs::write(
        root.join("Cargo.toml"),
        &realization.supporting["Cargo.toml"],
    )
    .expect("manifest");
    std::fs::write(root.join("types.rs"), &realization.declarations).expect("types");
    std::fs::write(
        root.join("tests/accessors.rs"),
        r#"
use probe_rules::*;

#[test]
fn each_variant_answers_its_attributes() {
    let gt: ProbeRulesOperator = serde_json::from_str("\"gt\"").unwrap();
    assert!(gt.takes_number());
    assert_eq!(gt.arity(), Some(2));
    assert_eq!(gt.label(), ">");
    let contains: ProbeRulesOperator = serde_json::from_str("\"Contains\"").unwrap();
    assert!(!contains.takes_number());
    assert_eq!(contains.arity(), None);
}
"#,
    )
    .expect("test");
    let output = std::process::Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("enum-attributes-target"),
        )
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if std::process::Command::new("go")
        .arg("version")
        .output()
        .is_err()
    {
        println!("skipped compiling Go: no `go` on PATH");
        return;
    }
    let realization = plan().go("rules", "example.invalid/rules").expect("go");
    let root = scratch("enum-attributes-go");
    std::fs::write(root.join("go.mod"), &realization.supporting["go.mod"]).expect("go.mod");
    std::fs::write(root.join("types.go"), &realization.declarations).expect("types");
    std::fs::write(
        root.join("types_test.go"),
        r#"package rules

import "testing"

func TestEachVariantAnswersItsAttributes(t *testing.T) {
	if !ProbeRulesOperatorV1.TakesNumber() || ProbeRulesOperatorV0.TakesNumber() {
		t.Fatal("takes_number")
	}
	if arity, ok := ProbeRulesOperatorV1.Arity(); !ok || arity != 2 {
		t.Fatal("arity")
	}
	if _, ok := ProbeRulesOperatorV0.Arity(); ok {
		t.Fatal("an unfilled arity")
	}
	if ProbeRulesOperatorV1.Label() != ">" {
		t.Fatal("label")
	}
}
"#,
    )
    .expect("test");
    let output = std::process::Command::new("go")
        .args(["test", "."])
        .current_dir(&root)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .expect("go runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
