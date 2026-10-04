//! `input.<field>` in a `when_subject` predicate (beyond10x/ess#157, E6) and `equals_ignore_case` /
//! `in_ignore_case` (beyond10x/ess#140, E7) in conformance: synthesis witnesses both sides of each,
//! the suite format pair `ess-conformance/20` and `/21` carries a fold, and the Go runtime reads the
//! pair and refuses the construct below it. `docs/design/value-expressions.md` §§ E6, E7.

use std::collections::BTreeMap;
use std::process::Command;

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::witness::{candidates, Distinction};
use ess_conformance::{
    flatten, synthesize, AdmittedSuite, ConformanceScenario, ConformanceSuite, ScenarioStep,
    ScenarioValue,
};
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use serde_json::{json, Value};

const RECORDING: &str = include_str!("fixtures/subject-guard-input.yaml");
const OTHER: &str = "calls.rec.Stop/outcome/other-recording";
const STOPPED: &str = "calls.rec.Stop/outcome/stopped";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let specification = Specification::assemble([(Source::new("guards.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id} in {:?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The input every `ExecuteCommand` of `command` sent, in step order.
fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

fn text(value: Option<&ScenarioValue>) -> String {
    match value.and_then(ScenarioValue::as_literal) {
        Some(Node::Text(text)) => text.clone(),
        other => panic!("not a literal text: {other:?}"),
    }
}

/// The recording the arranged call holds, and the recording the stop under test names.
fn recordings(scenario: &ConformanceScenario) -> (String, String) {
    let started = inputs(scenario, "calls.rec.Start");
    let stops = inputs(scenario, "calls.rec.Stop");
    let held = text(
        started
            .first()
            .expect("the call is created")
            .get("recording_id"),
    );
    let named = text(stops.last().expect("the stop is sent").get("recording_id"));
    (held, named)
}

#[test]
fn the_157_repro_synthesizes_with_only_the_undeclared_wrong_state_refusal() {
    let synthesis = synthesize(&compiled(RECORDING));
    let codes: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string)
            )
        })
        .collect();
    assert_eq!(
        codes,
        ["ESS-SYNTH-012 calls.rec.Call/state/Stopped/refuses/calls.rec.Stop"],
        "{:?}",
        synthesis.refusals
    );
    scenario(&synthesis.suite, OTHER);
    scenario(&synthesis.suite, STOPPED);
}

#[test]
fn the_guarded_branch_is_witnessed_with_a_recording_other_than_the_one_the_row_holds() {
    let suite = synthesize(&compiled(RECORDING)).suite;
    let other = scenario(&suite, OTHER);
    let (held, named) = recordings(other);
    assert_ne!(held, named, "the stop names another recording");
    let stop = inputs(other, "calls.rec.Stop");
    assert_eq!(
        stop.last().unwrap().get("call_id"),
        Some(&ScenarioValue::instance("call".parse().unwrap())),
        "the stop is sent for the arranged call"
    );
}

#[test]
fn the_default_is_witnessed_with_the_recording_the_row_holds() {
    let suite = synthesize(&compiled(RECORDING)).suite;
    let (held, named) = recordings(scenario(&suite, STOPPED));
    assert_eq!(held, named, "the stop names the call's own recording");
}

/// The same holds under `==`, with the branches swapped, and under a connective.
#[test]
fn either_operator_and_a_negation_is_witnessed_both_ways() {
    for (guard, equal_branch) in [
        ("recording_id == input.recording_id", OTHER),
        ("{not: {recording_id: {eq: input.recording_id}}}", STOPPED),
    ] {
        let text = RECORDING.replace("recording_id != input.recording_id", guard);
        let suite = synthesize(&compiled(&text)).suite;
        for id in [OTHER, STOPPED] {
            let (held, named) = recordings(scenario(&suite, id));
            assert_eq!(held == named, id == equal_branch, "{guard}: {id}");
        }
    }
}

// ---- E7 ------------------------------------------------------------------------------------------

const LOOKUP: &str = r"format: ess/15
system: demo
version: v1
domain: demo.orders
errors:
  - name: demo.orders.UnknownSource
    summary: The source is not a known source.
    fields: []
events:
  - name: demo.orders.Looked
    fields:
      - {name: source, type: Optional<String>}
commands:
  - name: demo.orders.Lookup
    input:
      - {name: source, type: Optional<String>}
      - {name: channel, type: String}
    outcomes:
      - name: unknown-source
        when: {source: {equals_ignore_case: web}}
        error: demo.orders.UnknownSource
      - name: found
        emits: [demo.orders.Looked]
        payload:
          demo.orders.Looked:
            source: input.source
";

fn command(ir: &EssIr) -> &ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new("demo.orders.Lookup").unwrap())
        .expect("declared")
}

fn guard(yaml: &str) -> Predicate {
    Predicate::from_node(&serde_yaml::from_str(yaml).expect("yaml")).expect("a predicate")
}

/// The texts candidates give `field`, in the order they are tried.
fn tried(ir: &EssIr, guard: &Predicate, field: &str) -> Vec<String> {
    let mut values = Vec::new();
    for input in candidates(ir, command(ir), &[guard], Distinction::PLAIN).expect("witnessable") {
        if let Some(Node::Text(text)) = input.get(field) {
            if !values.contains(text) {
                values.push(text.clone());
            }
        }
    }
    values
}

#[test]
fn a_case_changed_literal_matches_and_a_one_character_change_does_not() {
    let ir = compiled(LOOKUP);
    for (written, field, matching, refuting) in [
        (
            "{source: {equals_ignore_case: web}}",
            "source",
            "WEB",
            "xeb",
        ),
        (
            "{channel: {equals_ignore_case: Web}}",
            "channel",
            "wEB",
            "xeb",
        ),
        (
            "{channel: {in_ignore_case: [web, x1]}}",
            "channel",
            "X1",
            "y1",
        ),
        (
            "{not: {channel: {equals_ignore_case: web}}}",
            "channel",
            "WEB",
            "xeb",
        ),
    ] {
        let guard = guard(written);
        let values = tried(&ir, &guard, field);
        for expected in [matching, refuting] {
            assert!(
                values.iter().any(|value| value == expected),
                "{written}: {expected} in {values:?}"
            );
        }
        let decided: Vec<bool> = candidates(&ir, command(&ir), &[&guard], Distinction::PLAIN)
            .unwrap()
            .iter()
            .map(|input| {
                let decision = flatten(&ir, command(&ir), input).unwrap().decide(&guard);
                assert!(decision.unevaluable().is_none(), "{input:?}: {decision}");
                decision.is_satisfied()
            })
            .collect();
        assert!(
            decided.contains(&true) && decided.contains(&false),
            "{written}"
        );
    }
}

/// A literal with no non-ASCII letter is still tried in a form only Unicode folding equates with it:
/// its first `k` as the Kelvin sign, or its first `s` as the long s. ASCII folding refutes both.
#[test]
fn a_literal_with_k_or_s_is_tried_with_the_unicode_letter_that_folds_onto_it() {
    let ir = compiled(LOOKUP);
    for (written, expected) in [
        ("{channel: {equals_ignore_case: sku}}", "\u{17F}ku"),
        ("{channel: {equals_ignore_case: Kelp}}", "\u{212A}elp"),
        ("{channel: {equals_ignore_case: \"café\"}}", "CAFÉ"),
    ] {
        let guard = guard(written);
        let values = tried(&ir, &guard, "channel");
        assert!(
            values.iter().any(|value| value == expected),
            "{written}: {expected} in {values:?}"
        );
        let decision = flatten(
            &ir,
            command(&ir),
            &candidates(&ir, command(&ir), &[&guard], Distinction::PLAIN)
                .unwrap()
                .into_iter()
                .find(|input| input.get("channel") == Some(&Node::Text(expected.to_owned())))
                .unwrap(),
        )
        .unwrap()
        .decide(&guard);
        assert!(
            !decision.is_satisfied(),
            "{written}: ASCII folding refutes {expected}"
        );
    }
}

/// The refuting side of a fold is sent the one-character change first, avoiding every member, and
/// then the form only Unicode folding equates with the literal — the texts the design page names.
#[test]
fn the_default_is_sent_the_one_character_change_and_then_the_unicode_case() {
    for (guard, expected) in [
        ("{channel: {in_ignore_case: [web, xeb]}}", vec!["yeb"]),
        (
            "{channel: {equals_ignore_case: \"sku-1\"}}",
            vec!["xku-1", "\u{17F}ku-1"],
        ),
        (
            "{channel: {equals_ignore_case: \"café\"}}",
            vec!["xafé", "CAFÉ"],
        ),
    ] {
        let model = LOOKUP.replace("{source: {equals_ignore_case: web}}", guard);
        let suite = synthesize(&compiled(&model)).suite;
        let found: Vec<String> = inputs(
            scenario(&suite, "demo.orders.Lookup/outcome/found"),
            "demo.orders.Lookup",
        )
        .iter()
        .map(|input| text(input.get("channel")))
        .collect();
        assert_eq!(found, expected, "{guard}");
    }
}

/// The guarded branch is witnessed on the case-changed literal, so an implementation that compares
/// bytes fails it; the default on the literal with one character changed, `xeb`.
#[test]
fn the_140_repro_synthesizes_both_branches() {
    let synthesis = synthesize(&compiled(LOOKUP));
    assert!(
        synthesis.refusals.is_empty(),
        "{:?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let suite = synthesis.suite;
    let refused = inputs(
        scenario(&suite, "demo.orders.Lookup/outcome/unknown-source"),
        "demo.orders.Lookup",
    );
    let sent = text(refused.last().unwrap().get("source"));
    assert!(
        sent.eq_ignore_ascii_case("web") && sent != "web",
        "the refusal is witnessed on a case-changed literal, not {sent:?}"
    );
    let found = inputs(
        scenario(&suite, "demo.orders.Lookup/outcome/found"),
        "demo.orders.Lookup",
    );
    // The default is witnessed on the literal with one character changed, not on a base text: a
    // target comparing only lengths, or only the folded first byte, would pass a base text.
    assert_eq!(
        found
            .iter()
            .map(|input| text(input.get("source")))
            .collect::<Vec<_>>(),
        ["xeb"],
        "the default is sent exactly the one-character change of `web`"
    );
    // A guard is decided at synthesis and never reaches the suite, which keeps its format.
    assert!(!ess_conformance::text_match_format::case_fold_used_by(
        &suite
    ));
    assert_eq!(suite.provenance.suite_version.major(), 34);
}

fn suite(version: &str, predicate: &Value) -> Value {
    json!({
        "provenance": {"suite_version": version, "system": "demo", "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"demo.orders/authored/rows": {"purpose": "Check a folded filter over rows",
            "steps": [{"step": "expect_view", "view": "demo.orders.Rows",
                "expectation": {"expect": "satisfies", "predicate": predicate}}],
            "source": []}}
    })
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

const REQUIRES: &str = "case-insensitive text operators require suite/20 or /21";

#[test]
fn a_suite_carrying_a_fold_takes_the_new_ordinary_major() {
    for predicate in [
        json!({"not": {"source": {"equals_ignore_case": "web"}}}),
        json!({"source": {"in_ignore_case": ["web", "phone"]}}),
    ] {
        let raw = suite("ess-conformance/4", &predicate);
        let mut typed: ConformanceSuite = serde_json::from_value(raw).unwrap();
        assert!(ess_conformance::text_match_format::case_fold_used_by(
            &typed
        ));
        assert!(
            typed.to_canonical_json().is_err(),
            "a pinned older suite refuses to serialise the construct"
        );
        typed.select_fresh_format();
        assert_eq!(typed.provenance.suite_version.major(), 34);
        AdmittedSuite::from_json(&typed.to_canonical_json().unwrap()).unwrap();
        AdmittedSuite::from_json(&coverage("ess-conformance/21", &predicate).to_string())
            .expect("coverage/21 carries a fold");

        for below in 1..=19 {
            let version = format!("ess-conformance/{below}");
            let older = if below % 2 == 1 && below >= 5 {
                coverage(&version, &predicate)
            } else {
                suite(&version, &predicate)
            };
            let error = AdmittedSuite::from_json(&older.to_string())
                .err()
                .unwrap_or_else(|| panic!("suite/{below} carrying a fold is refused"));
            assert!(
                error.to_string().contains(REQUIRES),
                "suite/{below}: {error}"
            );
        }
    }
    // A string operator alongside a fold is carried by the fold's major, which implies /14.
    let both = json!({"all": [{"a": {"starts_with": "x"}}, {"b": {"equals_ignore_case": "y"}}]});
    let mut typed: ConformanceSuite =
        serde_json::from_value(suite("ess-conformance/4", &both)).unwrap();
    typed.select_fresh_format();
    assert_eq!(typed.provenance.suite_version.major(), 34);
}

#[test]
fn a_fold_operand_that_is_not_a_json_string_is_refused_because_suites_are_not_checked() {
    for predicate in [
        json!({"source": {"equals_ignore_case": 44}}),
        json!({"source": {"equals_ignore_case": ["web"]}}),
        json!({"source": {"in_ignore_case": "web"}}),
        json!({"source": {"in_ignore_case": ["web", true]}}),
    ] {
        let raw = suite("ess-conformance/20", &predicate);
        assert!(
            AdmittedSuite::from_json(&raw.to_string()).is_err(),
            "{predicate}"
        );
    }
}

#[test]
fn the_new_majors_are_supported_and_the_next_is_not() {
    use ess_conformance::scenario::SuiteFormat;
    for version in ["ess-conformance/20", "ess-conformance/21"] {
        assert!(
            SuiteFormat::parse(version).unwrap().is_supported(),
            "{version}"
        );
    }
    assert!(!SuiteFormat::parse("ess-conformance/38")
        .unwrap()
        .is_supported());
}

fn scratch(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ess-subject-guard-input-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a working directory");
    path
}

/// The Go runtime's whole-document admission at both new majors and below them, and its evaluator
/// over every `case_folds` vector of the shared corpus.
const GO_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"strings"
	"testing"
)

func TestCaseFoldSuiteAdmission(t *testing.T) {
	for name, want := range map[string]string{
		"ordinary20.json": "",
		"coverage21.json": "",
		"ordinary18.json": "case-insensitive text operators require suite/20 or /21",
		"coverage19.json": "case-insensitive text operators require suite/20 or /21",
		"number20.json":   "takes a string",
		"scalar20.json":   "takes a list of strings",
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

func TestCaseFoldCorpus(t *testing.T) {
	raw, err := os.ReadFile("vectors.json")
	if err != nil {
		t.Fatal(err)
	}
	var corpus struct {
		CaseFolds []map[string]json.RawMessage `json:"case_folds"`
	}
	if err := json.Unmarshal(raw, &corpus); err != nil {
		t.Fatal(err)
	}
	if len(corpus.CaseFolds) < 25 {
		t.Fatalf("the corpus selected %d case folds", len(corpus.CaseFolds))
	}
	for _, vector := range corpus.CaseFolds {
		var name, op, expected string
		var literal any
		for key, into := range map[string]any{"name": &name, "op": &op, "literal": &literal, "truth": &expected} {
			if err := json.Unmarshal(vector[key], into); err != nil {
				t.Fatalf("%s: %v", key, err)
			}
		}
		node := map[string]any{"source": map[string]any{op: literal}}
		if err := admitPredicateVersion(node, 20); err != nil {
			t.Fatalf("%s: suite/20 refuses %v: %v", name, node, err)
		}
		if err := admitPredicateVersion(node, 19); err == nil {
			t.Errorf("%s: suite/19 admits %v", name, node)
		}
		leaf, err := fromNode(node)
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		source := factSource{}
		var decoded any
		if err := json.Unmarshal(vector["value"], &decoded); err != nil {
			t.Fatal(err)
		}
		source["source"] = decoded
		want := map[string]truth{"true": truthTrue, "false": truthFalse, "unknown": truthUnknown}[expected]
		if got := leaf.evaluate(source); got != want {
			t.Errorf("Go %s: %v over %v: got %v, corpus says %s", name, node, source, got, expected)
		}
		if got := (predicate{kind: "not", body: &leaf}).evaluate(source); got != want.not() {
			t.Errorf("Go %s: not %v: got %v", name, node, got)
		}
		if meaning, ok := predicateMeaning(node).([]any); !ok || meaning[0] != op {
			t.Errorf("%s: meaning of %v is %v, not its own operator", name, node, predicateMeaning(node))
		}
	}
}
"#;

#[test]
fn the_go_runtime_reads_the_new_majors_and_answers_every_case_fold_the_corpus_states() {
    let fold = json!({"not": {"x": {"equals_ignore_case": "a"}}});
    let root = scratch("go");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    let emitted = AdmittedSuite::from_json(&coverage("ess-conformance/21", &fold).to_string())
        .expect("Rust admits coverage/21");
    for artifact in ess_conformance::go::emit(emitted.suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module casefold\n\ngo 1.24\n").unwrap();
    for (name, document) in [
        ("ordinary20.json", suite("ess-conformance/20", &fold)),
        ("coverage21.json", coverage("ess-conformance/21", &fold)),
        ("ordinary18.json", suite("ess-conformance/18", &fold)),
        ("coverage19.json", coverage("ess-conformance/19", &fold)),
        (
            "number20.json",
            suite(
                "ess-conformance/20",
                &json!({"x": {"equals_ignore_case": 44}}),
            ),
        ),
        (
            "scalar20.json",
            suite("ess-conformance/20", &json!({"x": {"in_ignore_case": "a"}})),
        ),
    ] {
        std::fs::write(package.join(name), document.to_string()).unwrap();
    }
    std::fs::write(
        package.join("vectors.json"),
        include_str!("../../../specify/ess-primitives/tests/vectors/primitive-semantics.json"),
    )
    .unwrap();
    std::fs::write(package.join("case_fold_test.go"), GO_TEST).unwrap();
    let output = Command::new("go")
        .args(["test", "-count=1", "-v", "./...", "-run", "^TestCaseFold"])
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
    for case in ["TestCaseFoldSuiteAdmission", "TestCaseFoldCorpus"] {
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(&format!("--- PASS: {case}")),
            "the Go case {case} ran rather than selecting nothing: {record}"
        );
    }
}
