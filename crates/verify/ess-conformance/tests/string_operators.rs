//! `starts_with`, `ends_with` and `contains` (beyond10x/ess#95) in conformance: the witness
//! candidates that decide both branches of a string guard, synthesis over them, the suite format
//! pair that carries one, and the two readers that refuse a suite carrying one.
//! `docs/design/string-predicate-operators.md` is the binding design.

use std::collections::BTreeSet;
use std::process::Command;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::witness::{candidates, Distinction};
use ess_conformance::{flatten, synthesize, AdmittedSuite, ConformanceSuite};
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use serde_json::{json, Value};

const MODEL: &str = r#"format: ess/8
system: routing
version: v1
domain: routing.calls
types:
  - name: routing.calls.PhoneNumber
    kind: newtype
    of: String
    invariants:
      - value: {starts_with: "+"}
  - name: routing.calls.Email
    kind: newtype
    of: String
    invariants:
      - value: {contains: "@"}
events:
  - name: routing.calls.Routed
    fields: []
errors:
  - name: routing.calls.Refused
    fields: []
commands:
  - name: routing.calls.Call
    input:
      - {name: caller, type: routing.calls.PhoneNumber}
    outcomes:
      - name: uk
        when: {caller: {starts_with: "+44"}}
        emits: [routing.calls.Routed]
      - name: other
        error: routing.calls.Refused
  - name: routing.calls.Route
    input:
      - {name: subject, type: String}
      - {name: sku, type: String}
      - {name: dialled, type: String}
      - {name: note, type: Optional<String>}
      - {name: tags, type: List<String>}
    outcomes:
      - name: routed
        when: {subject: {contains: "urgent"}}
        emits: [routing.calls.Routed]
      - name: other
        error: routing.calls.Refused
  - name: routing.calls.Mail
    input:
      - {name: email, type: routing.calls.Email}
    outcomes:
      - name: corp
        when: {email: {ends_with: "@corp.example"}}
        emits: [routing.calls.Routed]
      - name: other
        error: routing.calls.Refused
"#;

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let specification = Specification::assemble([(Source::new("routing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn named<'ir>(ir: &'ir EssIr, name: &str) -> &'ir ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new(format!("routing.calls.{name}")).unwrap())
        .expect("declared")
}

/// The command a guard over `field` is tried against: each newtype-typed field has its own, because
/// a base witness its own type refuses (`caller` is no `+` number) removes every candidate that
/// leaves it at the base.
fn owner(field: &str) -> &'static str {
    match field {
        "caller" => "Call",
        "email" => "Mail",
        _ => "Route",
    }
}

fn route<'ir>(ir: &'ir EssIr, guard: &Predicate) -> &'ir ResolvedCommand {
    let read = guard.fact_paths()[0].namespace().to_owned();
    named(ir, owner(&read))
}

fn guard(yaml: &str) -> Predicate {
    Predicate::from_node(&serde_yaml::from_str(yaml).expect("yaml")).expect("a predicate")
}

/// The admitted values candidates give `field`, in the order they are tried.
fn tried(ir: &EssIr, guard: &Predicate, field: &str) -> Vec<String> {
    let mut values = Vec::new();
    for input in
        candidates(ir, route(ir, guard), &[guard], Distinction::PLAIN).expect("witnessable")
    {
        if let Some(Node::Text(text)) = input.get(field) {
            if !values.contains(text) {
                values.push(text.clone());
            }
        }
    }
    values
}

/// Whether some admitted candidate satisfies `guard`, and whether some refutes it.
fn both_sides(ir: &EssIr, guard: &Predicate) -> (bool, bool) {
    let command = route(ir, guard);
    let decided: Vec<bool> = candidates(ir, command, &[guard], Distinction::PLAIN)
        .expect("witnessable")
        .iter()
        .map(|input| {
            let decision = flatten(ir, command, input).expect("fits").decide(guard);
            assert!(decision.unevaluable().is_none(), "{input:?}: {decision}");
            decision.is_satisfied()
        })
        .collect();
    (decided.contains(&true), decided.contains(&false))
}

#[test]
fn every_operator_has_a_satisfying_and_a_refuting_candidate() {
    let ir = compiled(MODEL);
    for yaml in [
        r#"{subject: {starts_with: "RE"}}"#,
        r#"{subject: {ends_with: "!"}}"#,
        r#"{subject: {contains: "urgent"}}"#,
        // `b` contains `L`: only `L′` refutes (the middle character replaced).
        r#"{subject: {contains: "sub"}}"#,
        r#"{subject: {starts_with: "subject"}}"#,
        r#"{not: {dialled: {ends_with: "0"}}}"#,
        r#"{note: {contains: "x"}}"#,
        "{sku: {starts_with: \"e\u{301}\"}}",
        "{sku: {ends_with: \"\u{1F600}\"}}",
    ] {
        assert_eq!(both_sides(&ir, &guard(yaml)), (true, true), "{yaml}");
    }
}

/// The design's first worked case: the trial's motivating guard under the type's own invariant.
#[test]
fn a_phone_prefix_under_a_plus_invariant_keeps_refuting_candidates() {
    let ir = compiled(MODEL);
    let guard = guard(r#"{caller: {starts_with: "+44"}}"#);
    let values: BTreeSet<String> = tried(&ir, &guard, "caller").into_iter().collect();
    let expected: BTreeSet<String> = ["+44", "+44caller", "+4x", "+caller", "+"]
        .map(str::to_owned)
        .into();
    assert_eq!(values, expected);
    assert_eq!(both_sides(&ir, &guard), (true, true));
}

/// The design's second: `L′` loses the `@`, and only the invariant's own compositions refute.
#[test]
fn an_email_suffix_under_an_at_invariant_is_refuted_by_the_invariant_compositions() {
    let ir = compiled(MODEL);
    let guard = guard(r#"{email: {ends_with: "@corp.example"}}"#);
    let values: BTreeSet<String> = tried(&ir, &guard, "email").into_iter().collect();
    let expected: BTreeSet<String> = [
        "@corp.example",
        "email@corp.example",
        "@email",
        "email@",
        "@",
    ]
    .map(str::to_owned)
    .into();
    assert_eq!(values, expected);
    assert_eq!(both_sides(&ir, &guard), (true, true));

    let middle = guard_middle(&ir);
    assert!(middle.contains(&"@cxrp".to_owned()), "{middle:?}");
}

fn guard_middle(ir: &EssIr) -> Vec<String> {
    tried(ir, &guard(r#"{email: {contains: "@corp"}}"#), "email")
}

/// The reference page's example: no single literal satisfies it, the composition does, and the
/// negated literal is never composed in.
#[test]
fn a_prefix_a_suffix_and_a_negated_substring_compose_into_one_witness() {
    let ir = compiled(MODEL);
    let guard = guard(
        "all:\n  - sku: {starts_with: \"SKU-\"}\n  - not: {sku: {contains: \"test\"}}\n  - sku: {ends_with: \"0\"}\n",
    );
    let values = tried(&ir, &guard, "sku");
    assert!(values.contains(&"SKU-sku0".to_owned()), "{values:?}");
    for value in &values {
        assert!(
            value == "test" || !value.contains("test"),
            "a negated literal was composed into {value:?}: {values:?}"
        );
    }
    assert_eq!(both_sides(&ir, &guard), (true, true));
}

/// The layout that puts the path's own text first is what witnesses this guard.
#[test]
fn contains_without_starting_with_is_witnessed_by_the_text_after_the_path() {
    let ir = compiled(MODEL);
    let guard =
        guard("all:\n  - subject: {contains: \"RE\"}\n  - not: {subject: {starts_with: \"RE\"}}\n");
    let values = tried(&ir, &guard, "subject");
    assert!(values.contains(&"subjectRE".to_owned()), "{values:?}");
    assert_eq!(both_sides(&ir, &guard), (true, true));
}

/// A1's one-element list is built from the guard's own literal, so a string operator over an
/// element is witnessed with no code specific to it.
#[test]
fn a_string_operator_over_a_list_element_is_witnessed_by_a_one_element_list() {
    let ir = compiled(MODEL);
    let guard = guard("exists: {in: tags, as: t, that: {t: {starts_with: \"vip\"}}}");
    let command = route(&ir, &guard);
    let options = candidates(&ir, command, &[&guard], Distinction::PLAIN).unwrap();
    let satisfying: Vec<_> = options
        .iter()
        .filter(|input| {
            flatten(&ir, command, input)
                .unwrap()
                .decide(&guard)
                .is_satisfied()
        })
        .collect();
    assert!(
        satisfying
            .iter()
            .any(|input| input.get("tags") == Some(&Node::Seq(vec![Node::Text("vip".into())]))),
        "{options:?}"
    );
    assert!(options
        .iter()
        .any(|input| input.get("tags") == Some(&Node::Seq(Vec::new()))));
}

/// Both branches of each operator's guard become scenarios, with nothing about the command refused.
///
/// The two newtypes' own invariants are refused as `ESS-SYNTH-013` — no view publishes them — which
/// is about the types and not about any guard, so only refusals naming a command are counted.
#[test]
fn synthesis_witnesses_both_branches_of_every_string_guard() {
    const ROUTE: &str = r#"{subject: {contains: "urgent"}}"#;
    for when in [
        ROUTE,
        r#"{dialled: {ends_with: "0"}}"#,
        r#"{not: {subject: {contains: "spam"}}}"#,
        "{all: [{sku: {starts_with: \"SKU-\"}}, {not: {sku: {contains: \"test\"}}}, {sku: {ends_with: \"0\"}}]}",
        "{all: [{subject: {contains: \"RE\"}}, {not: {subject: {starts_with: \"RE\"}}}]}",
        "{exists: {in: tags, as: t, that: {t: {starts_with: \"vip\"}}}}",
    ] {
        let ir = compiled(&MODEL.replace(ROUTE, when));
        let synthesis = synthesize(&ir);
        let refused: Vec<String> = synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .filter(|refusal| !refusal.contains("ESS-SYNTH-013"))
            .collect();
        assert!(refused.is_empty(), "{when}: {refused:?}");
        let ids: Vec<String> = synthesis
            .suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect();
        for scenario in [
            "routing.calls.Call/outcome/uk",
            "routing.calls.Call/outcome/other",
            "routing.calls.Mail/outcome/corp",
            "routing.calls.Mail/outcome/other",
            "routing.calls.Route/outcome/routed",
            "routing.calls.Route/outcome/other",
        ] {
            assert!(ids.iter().any(|id| id == scenario), "{when}: {scenario} in {ids:?}");
        }
        // A string guard is decided at synthesis and never reaches the suite, which keeps its format.
        assert!(!ess_conformance::text_match_format::used_by(&synthesis.suite));
    }
}

fn suite(version: &str, predicate: &Value) -> Value {
    json!({
        "provenance": {"suite_version": version, "system": "routing", "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"routing.calls/authored/rows": {"purpose": "Check a string guard over rows",
            "steps": [{"step": "expect_view", "view": "routing.calls.Rows",
                "expectation": {"expect": "satisfies", "predicate": predicate}}],
            "source": []}}
    })
}

#[test]
fn a_suite_carrying_a_string_operator_takes_the_new_ordinary_major() {
    let predicate = json!({"not": {"caller": {"starts_with": "+44"}}});
    let raw = suite("ess-conformance/4", &predicate);
    let mut typed: ConformanceSuite = serde_json::from_value(raw.clone()).unwrap();
    assert!(ess_conformance::text_match_format::used_by(&typed));
    assert!(
        typed.to_canonical_json().is_err(),
        "a pinned older suite refuses to serialise the construct"
    );
    typed.select_fresh_format();
    assert_eq!(typed.provenance.suite_version.major(), 14);
    AdmittedSuite::from_json(&typed.to_canonical_json().unwrap()).unwrap();

    for below in 1..=13 {
        let version = format!("ess-conformance/{below}");
        // A coverage major is refused without its inventory first, so it carries one here.
        let older = if below % 2 == 1 && below >= 5 {
            coverage(&version, &predicate)
        } else {
            suite(&version, &predicate)
        };
        let error = AdmittedSuite::from_json(&older.to_string())
            .err()
            .unwrap_or_else(|| panic!("suite/{below} carrying a string operator is refused"));
        assert!(
            error
                .to_string()
                .contains("string predicate operators require suite/14 or /15"),
            "suite/{below}: {error}"
        );
    }

    let plain = suite("ess-conformance/4", &json!({"caller": {"eq": "+44"}}));
    let mut typed: ConformanceSuite = serde_json::from_value(plain).unwrap();
    assert!(!ess_conformance::text_match_format::used_by(&typed));
    typed.select_fresh_format();
    assert_eq!(
        typed.provenance.suite_version.major(),
        4,
        "a suite without the construct keeps its format and bytes"
    );
}

#[test]
fn a_suite_string_operand_that_is_not_a_json_string_is_refused_because_suites_are_not_checked() {
    for operand in [json!(44), json!(true)] {
        let raw = suite(
            "ess-conformance/14",
            &json!({"caller": {"ends_with": operand}}),
        );
        assert!(
            AdmittedSuite::from_json(&raw.to_string()).is_err(),
            "{operand}"
        );
    }
    AdmittedSuite::from_json(
        &suite("ess-conformance/14", &json!({"caller": {"contains": "44"}})).to_string(),
    )
    .expect("a string operand is admitted at suite/14");
}

const ADMISSION_JS: &str = include_str!("../assets/coverage-admission.js");

fn scratch(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ess-string-operators-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a working directory");
    path
}

/// A coverage suite at `version` whose one scenario carries `predicate` under `satisfies`.
fn coverage(version: &str, predicate: &Value) -> Value {
    json!({
        "provenance": {"suite_version": version, "system": "example", "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "a".repeat(64)},
        "scenarios": {"example.domain/authored/created": {"purpose": "Known candidate",
            "steps": [{"step": "expect_view", "view": "example.All",
                "expectation": {"expect": "satisfies", "predicate": predicate}}],
            "source": []}},
        "coverage": {
            "selection": {"scope": {"kind": "system"}, "origins": "authored", "filter": {"kind": "all"}},
            "knowledge": "complete_inventory", "generated": [],
            "authored": ["example.domain/authored/created"], "outside": [], "refused": [],
            "authored_sources": {"created.yaml": {
                "digest": format!("sha256:{}", "b".repeat(64)),
                "scenario": "example.domain/authored/created", "disposition": "accepted"}},
            "counts": {"generated": 0, "authored": 1, "outside": 0, "refused": 0}
        }
    })
}

/// The browser replay adapter does not learn the operators: the new coverage major is refused by
/// its version, and a forged `/9` carrying one is refused by the operator.
#[test]
fn the_browser_adapter_refuses_the_new_major_and_a_forged_older_suite_carrying_one() {
    let string = json!({"x": {"starts_with": "a"}});
    let current = coverage("ess-conformance/15", &string);
    AdmittedSuite::from_json(&current.to_string()).expect("Rust admits coverage/15");
    let root = scratch("browser");
    std::fs::write(root.join("admission.js"), ADMISSION_JS).unwrap();
    std::fs::write(root.join("current.json"), current.to_string()).unwrap();
    std::fs::write(
        root.join("forged.json"),
        coverage("ess-conformance/9", &string).to_string(),
    )
    .unwrap();
    std::fs::write(
        root.join("plain.json"),
        coverage("ess-conformance/9", &json!({"x": {"eq": "a"}})).to_string(),
    )
    .unwrap();
    std::fs::write(
        root.join("harness.mjs"),
        r"import {readFileSync} from 'node:fs'
import {admitSuite} from './admission.js'
const answer = async name => { try { await admitSuite(readFileSync(name, 'utf8')); return true } catch (error) { return String(error) } }
console.log(JSON.stringify({current: await answer('current.json'), forged: await answer('forged.json'), plain: await answer('plain.json')}))
",
    )
    .unwrap();
    let node = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let output = Command::new(&node)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("the required Node toolchain executes; set ESS_NODE to name it");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{record}");
    let answer: Value = serde_json::from_slice(&output.stdout).expect("JSON");
    assert_eq!(
        answer["plain"],
        json!(true),
        "the forged pair's control is admitted: {record}"
    );
    assert!(
        answer["current"]
            .as_str()
            .is_some_and(|error| error.contains("replay requires suite/5 or /9")),
        "{record}"
    );
    assert!(
        answer["forged"]
            .as_str()
            .is_some_and(|error| error.contains("unsupported predicate operator")),
        "{record}"
    );
}

/// An undecidable string guard names the path nothing bound, because that is the only way one is
/// `Unknown`: the literal always resolves, and a resolved value that is not text is `False`.
#[test]
fn an_undecidable_string_guard_names_the_absent_path() {
    let ir = compiled(MODEL);
    let guard = guard(r#"{note: {contains: "x"}}"#);
    let command = route(&ir, &guard);
    let omitted = candidates(&ir, command, &[&guard], Distinction::PLAIN)
        .unwrap()
        .into_iter()
        .next()
        .map(|mut input| {
            input.remove("note");
            input
        })
        .unwrap();
    let decision = flatten(&ir, command, &omitted).unwrap().decide(&guard);
    let refusal = decision
        .unevaluable()
        .expect("an absent optional is undecidable");
    assert_eq!(refusal.causes.len(), 1, "{refusal}");
    assert_eq!(
        refusal.causes[0].reason,
        ess_conformance::Reason::ValueAbsent {
            path: ess_primitives::facts::FactPath::new("note").unwrap()
        }
    );
}

/// The Go runtime's own whole-document admission, at both new majors and below them.
const GO_ADMISSION_TEST: &str = r#"package essconform

import (
	"os"
	"strings"
	"testing"
)

func TestStringOperatorSuiteAdmission(t *testing.T) {
	for name, want := range map[string]string{
		"ordinary14.json": "",
		"coverage15.json": "",
		"ordinary12.json": "string predicate operators require suite/14 or /15",
		"coverage13.json": "string predicate operators require suite/14 or /15",
		"number14.json":   "takes a string",
	} {
		raw, err := os.ReadFile(name)
		if err != nil {
			t.Fatal(err)
		}
		_, err = admitSuite(string(raw))
		switch {
		case want == "" && err != nil:
			t.Errorf("%s: refused: %v", name, err)
		case want != "" && (err == nil || !strings.Contains(err.Error(), want)):
			t.Errorf("%s: want %q, got %v", name, want, err)
		}
	}
}
"#;

#[test]
fn the_go_runtime_admits_the_new_majors_and_refuses_the_construct_below_them() {
    let string = json!({"not": {"x": {"starts_with": "a"}}});
    let root = scratch("go-admission");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    let emitted = AdmittedSuite::from_json(&coverage("ess-conformance/15", &string).to_string())
        .expect("Rust admits coverage/15");
    for artifact in ess_conformance::go::emit(emitted.suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module stringadmission\n\ngo 1.24\n").unwrap();
    for (name, document) in [
        ("ordinary14.json", suite("ess-conformance/14", &string)),
        ("coverage15.json", coverage("ess-conformance/15", &string)),
        ("ordinary12.json", suite("ess-conformance/12", &string)),
        ("coverage13.json", coverage("ess-conformance/13", &string)),
        (
            "number14.json",
            suite("ess-conformance/14", &json!({"x": {"contains": 44}})),
        ),
    ] {
        std::fs::write(package.join(name), document.to_string()).unwrap();
    }
    std::fs::write(package.join("admission_test.go"), GO_ADMISSION_TEST).unwrap();
    let output = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-v",
            "./...",
            "-run",
            "^TestStringOperatorSuiteAdmission$",
        ])
        .current_dir(&root)
        .env("GOWORK", "off")
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{record}");
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("--- PASS: TestStringOperatorSuiteAdmission"),
        "the Go case ran rather than selecting nothing: {record}"
    );
}

/// Every operator of one constraint mapping, in every spelling Rust reads, is decided by Go as
/// Rust decides it (correction round 1, F2 and its class): Go conjoins them rather than reading the
/// first it finds, and reads a scalar or a null under a membership key as Rust does.
#[test]
fn go_decides_every_constraint_mapping_as_rust_does() {
    use ess_primitives::facts::{FactPath, FactStore, FactValue, Number};
    let cases: Vec<(Value, Value)> = vec![
        (json!({"x": {"gte": 1, "lte": 5}}), json!(7)),
        (json!({"x": {"gte": 1, "lte": 5}}), json!(3)),
        (
            json!({"x": {"starts_with": "+", "ends_with": "0"}}),
            json!("+441"),
        ),
        (
            json!({"x": {"starts_with": "+", "ends_with": "0"}}),
            json!("+440"),
        ),
        (json!({"x": {"in": "a"}}), json!("a")),
        (json!({"x": {"one_of": null}}), json!("a")),
        (json!({"x": {"not_in": ["a", "b"], "ne": "c"}}), json!("c")),
        (json!({"x": {"exists": false}}), json!("a")),
        (
            json!({"x": {"defined": true, "contains": "b"}}),
            json!("abc"),
        ),
        (json!({"x": {"truthy": true, "eq": ""}}), json!("")),
    ];
    let mut expected = Vec::new();
    for (predicate, fact) in &cases {
        let rust =
            Predicate::from_node(&serde_json::from_value::<Node>(predicate.clone()).unwrap())
                .unwrap();
        let mut facts = FactStore::new();
        let value = match fact {
            Value::String(text) => FactValue::text(text),
            Value::Number(number) => {
                FactValue::Number(Number::new(number.as_f64().unwrap()).unwrap())
            }
            other => panic!("{other}"),
        };
        facts.set(FactPath::new("x").unwrap(), value);
        expected.push(
            json!({"predicate": predicate, "fact": fact, "truth": rust.evaluate(&facts).as_str()}),
        );
    }
    let root = scratch("go-mappings");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    let emitted =
        AdmittedSuite::from_json(&coverage("ess-conformance/15", &json!(true)).to_string())
            .expect("Rust admits coverage/15");
    for artifact in ess_conformance::go::emit(emitted.suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module mappings\n\ngo 1.24\n").unwrap();
    std::fs::write(
        package.join("cases.json"),
        Value::Array(expected).to_string(),
    )
    .unwrap();
    std::fs::write(package.join("mapping_test.go"), GO_MAPPING_TEST).unwrap();
    let output = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-v",
            "./...",
            "-run",
            "^TestConstraintMappings$",
        ])
        .current_dir(&root)
        .env("GOWORK", "off")
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{record}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("--- PASS: TestConstraintMappings"),
        "the Go case ran rather than selecting nothing: {record}"
    );
}

const GO_MAPPING_TEST: &str = r#"package essconform

import (
	"bytes"
	"encoding/json"
	"os"
	"testing"
)

func TestConstraintMappings(t *testing.T) {
	raw, err := os.ReadFile("cases.json")
	if err != nil {
		t.Fatal(err)
	}
	var cases []struct {
		Predicate any    `json:"predicate"`
		Fact      any    `json:"fact"`
		Truth     string `json:"truth"`
	}
	// Numbers as the suite reader decodes them, so admission reads them as it reads a suite.
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	if err := decoder.Decode(&cases); err != nil {
		t.Fatal(err)
	}
	if len(cases) < 10 {
		t.Fatalf("%d cases", len(cases))
	}
	names := map[truth]string{truthTrue: "true", truthFalse: "false", truthUnknown: "unknown"}
	for _, c := range cases {
		if err := admitPredicateEnvelope(c.Predicate, 0); err != nil {
			t.Errorf("%v: admission refuses: %v", c.Predicate, err)
			continue
		}
		leaf, err := fromNode(c.Predicate)
		if err != nil {
			t.Errorf("%v: %v", c.Predicate, err)
			continue
		}
		if got := names[leaf.evaluate(factSource{"x": c.Fact})]; got != c.Truth {
			t.Errorf("Go %v over %v: %s, Rust says %s", c.Predicate, c.Fact, got, c.Truth)
		}
	}
}
"#;
