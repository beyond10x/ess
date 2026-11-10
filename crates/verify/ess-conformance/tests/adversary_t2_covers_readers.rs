//! Adversary (turn 2): the Rust, Go, TypeScript and browser coverage readers agree on suites that
//! carry `covered_by` (beyond10x/ess#735).
//!
//! The design page says a reader admits `covered_by` from `ess-conformance/65` only, and that every
//! reader applies the same completeness rule. Each reader is asked for one line per vector:
//! `refused`, `admitted-complete` or `admitted-incomplete`. The Rust reader is the oracle.

mod support_go;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    coverage::{AdmittedInput, Origins, Scope},
    coverage_build::{self, CoverageSource},
    AdmittedSuite, ScenarioId,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use serde_json::{json, Value};

const SCENARIO: &str = "example.domain/authored/created";
const REFUSED: &str = "example.domain.Make/outcome/already-exists";
const COVERING: &str = "example.review/authored/confirm-decision-already-exists";
const EFFECTS: [&str; 2] = ["candidate_not_emitted", "check_not_emitted"];

fn document(major: u32, refusal: &Value) -> Value {
    let digest = |c: char| format!("sha256:{}", c.to_string().repeat(64));
    json!({
        "provenance": {
            "suite_version": format!("ess-conformance/{major}"), "system": "example",
            "specification_version": "v1", "spec_digest": "a".repeat(64),
            "contract_digest": "a".repeat(64), "scenario_initial_state": "empty"
        },
        "scenarios": {SCENARIO: {"purpose": "Known candidate", "steps": [], "source": []}},
        "coverage": {
            "selection": {"scope": {"kind": "system"}, "origins": "generated_and_authored",
                "filter": {"kind": "all"}},
            "knowledge": "complete_inventory", "generated": [], "authored": [SCENARIO],
            "outside": [], "refused": [refusal],
            "authored_sources": {
                "created.yaml": {"digest": digest('b'), "scenario": SCENARIO, "disposition": "accepted"}
            },
            "counts": {"generated": 0, "authored": 1, "outside": 0, "refused": 1}
        }
    })
}

fn refusal(scenario: Option<&str>, effect: &str, covered_by: Option<&str>) -> Value {
    let mut value = json!({
        "origin": "generated", "scenario": scenario,
        "subject": {"kind": "command", "name": "example.domain.Make"}, "source": null,
        "code": "ESS-SYNTH-004", "message": "refused", "effect": effect,
        "retained": null, "scope": "in_scope", "needs": []
    });
    if let Some(by) = covered_by {
        value["covered_by"] = json!(by);
    }
    value
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/authored-covers")
}

fn model() -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for name in ["system.yaml", "review.yaml"] {
        let original = std::fs::read_to_string(fixture().join(format!("{name}.in"))).unwrap();
        let raw = RawSpecFile::parse(&original).unwrap();
        sources.insert(name, original);
        parsed.push((Source::new(name), raw));
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}

/// The suite the unit's own fixture synthesizes: one refusal covered by the authored scenario.
fn synthesized() -> (String, String) {
    let ir = model();
    let text = std::fs::read_to_string(fixture().join("scenarios/confirm-twice.yaml.in")).unwrap();
    let source = CoverageSource::new("scenarios/confirm-twice.yaml", &text).unwrap();
    let input = coverage_build::build(&ir, &[source], Scope::System, Origins::GeneratedAndAuthored)
        .unwrap();
    let kept: Vec<ScenarioId> = input
        .selected()
        .suite()
        .scenarios
        .keys()
        .filter(|id| id.to_string() != COVERING)
        .cloned()
        .collect();
    let child = input.select(&kept).unwrap();
    (
        input.selected().original_json().to_owned(),
        child.document().to_canonical_json().unwrap(),
    )
}

fn vectors() -> BTreeMap<String, String> {
    let mut vectors = BTreeMap::new();
    for major in [35, 53] {
        for effect in EFFECTS {
            let cases = [
                ("uncovered", refusal(Some(REFUSED), effect, None)),
                ("covered", refusal(Some(REFUSED), effect, Some(SCENARIO))),
                (
                    "covered-null-scenario",
                    refusal(None, effect, Some(SCENARIO)),
                ),
                (
                    "covered-by-missing",
                    refusal(
                        Some(REFUSED),
                        effect,
                        Some("example.domain/authored/missing"),
                    ),
                ),
            ];
            for (name, refusal) in cases {
                vectors.insert(
                    format!("{major}-{name}-{effect}.json"),
                    document(major, &refusal).to_string(),
                );
            }
        }
    }
    let (real, child) = synthesized();
    assert!(real.contains("\"ess-conformance/65\""), "{real}");
    vectors.insert("real-65.json".into(), real);
    vectors.insert("input-child-65.json".into(), child);
    vectors
}

fn rust_verdicts(vectors: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    vectors
        .iter()
        .map(|(name, raw)| {
            let admitted = if name.starts_with("input-") {
                AdmittedInput::from_json(raw).map(|input| input.selected().clone())
            } else {
                AdmittedSuite::from_json(raw)
            };
            let verdict = match admitted {
                Err(_) => "refused",
                Ok(suite) if suite.coverage().is_some_and(|c| c.is_complete()) => {
                    "admitted-complete"
                }
                Ok(_) => "admitted-incomplete",
            };
            (name.clone(), verdict.to_owned())
        })
        .collect()
}

fn read_verdicts(path: &Path, log: &str) -> BTreeMap<String, String> {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{error}: {log}"))
        .lines()
        .map(|line| {
            let (name, verdict) = line.split_once(' ').unwrap();
            (name.to_owned(), verdict.to_owned())
        })
        .collect()
}

fn write_vectors(directory: &Path, vectors: &BTreeMap<String, String>) {
    std::fs::create_dir_all(directory).unwrap();
    for (name, raw) in vectors {
        std::fs::write(directory.join(name), raw).unwrap();
    }
}

fn carrier() -> AdmittedSuite {
    let raw = json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "example",
            "specification_version": "v1", "spec_digest": "a".repeat(64),
            "contract_digest": "a".repeat(64)},
        "scenarios": {SCENARIO: {"purpose": "Known candidate", "steps": [], "source": []}}
    })
    .to_string();
    AdmittedSuite::from_json(&raw).expect("the carrier is admitted")
}

/// The oracle itself: the Rust reader admits `covered_by` at /65 only, and counts it complete.
#[test]
fn the_rust_reader_admits_covered_by_only_from_65() {
    let rust = rust_verdicts(&vectors());
    let admitted = |prefix: &str| {
        EFFECTS
            .iter()
            .filter(|e| rust[&format!("{prefix}-{e}.json")] != "refused")
            .count()
    };
    for major in [35, 53] {
        assert_eq!(admitted(&format!("{major}-covered")), 0, "/{major}");
        assert_eq!(admitted(&format!("{major}-uncovered")), 1, "/{major}");
    }
    assert_eq!(rust["real-65.json"], "admitted-complete");
    assert_eq!(rust["input-child-65.json"], "admitted-incomplete");
}

const GO_DRIVER: &str = r#"package essconform

import (
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

func TestCoveredVectors(t *testing.T) {
	directory := os.Getenv("ESS_COVERAGE_VECTORS")
	entries, err := os.ReadDir(directory)
	if err != nil {
		t.Fatal(err)
	}
	var lines []string
	for _, entry := range entries {
		raw, err := os.ReadFile(filepath.Join(directory, entry.Name()))
		if err != nil {
			t.Fatal(err)
		}
		verdict := "refused"
		if suite, err := admitRunInput(string(raw)); err == nil {
			verdict = "admitted-incomplete"
			if completeCoverage(suite.coverage) {
				verdict = "admitted-complete"
			}
		}
		lines = append(lines, entry.Name()+" "+verdict)
	}
	sort.Strings(lines)
	if err := os.WriteFile(os.Getenv("ESS_COVERAGE_VERDICTS"), []byte(strings.Join(lines, "\n")), 0o644); err != nil {
		t.Fatal(err)
	}
}
"#;

#[test]
fn the_go_reader_agrees_with_the_rust_reader_on_covered_by() {
    let vectors = vectors();
    let directory = support_go::package(
        "adversary-covers",
        carrier().suite(),
        &[("covered_vectors_test.go", GO_DRIVER)],
    );
    let input = directory.join("vectors");
    write_vectors(&input, &vectors);
    let output = directory.join("verdicts.txt");
    let run = support_go::go_test(
        &directory,
        "TestCoveredVectors",
        &[
            ("ESS_COVERAGE_VECTORS", input.to_str().unwrap()),
            ("ESS_COVERAGE_VERDICTS", output.to_str().unwrap()),
        ],
    );
    assert!(run.success, "{}", run.log);
    assert_eq!(
        read_verdicts(&output, &run.log),
        rust_verdicts(&vectors),
        "Go (left) and Rust (right)"
    );
}

const NODE_DRIVER: &str = r"import {readdirSync, readFileSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
const [reader, directory, output, only] = process.argv.slice(2);
const module = await import(reader);
const lines = [];
for (const name of readdirSync(directory).sort()) {
  if (only !== '' && !name.startsWith(only)) continue;
  const raw = readFileSync(join(directory, name), 'utf8');
  let verdict = 'refused';
  try {
    const suite = await (module.admitRunInput ?? module.admitSuite)(raw);
    verdict = 'admitted';
    if (module.completeCoverage) verdict = module.completeCoverage(suite.coverage) ? 'admitted-complete' : 'admitted-incomplete';
  } catch {}
  lines.push(`${name} ${verdict}`);
}
writeFileSync(output, lines.join('\n'));
";

fn node_verdicts(reader: &Path, directory: &Path, only: &str) -> BTreeMap<String, String> {
    let driver = directory.join("driver.mjs");
    std::fs::write(&driver, NODE_DRIVER).unwrap();
    let output = directory.join(format!("verdicts-{only}.txt"));
    let run = Command::new("node")
        .arg(&driver)
        .arg(format!("file://{}", reader.display()))
        .arg(directory.join("vectors"))
        .arg(&output)
        .arg(only)
        .output()
        .expect("node runs");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(run.status.success(), "{log}");
    read_verdicts(&output, &log)
}

#[test]
fn the_typescript_reader_agrees_with_the_rust_reader_on_covered_by() {
    let vectors = vectors();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-covers-ts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit(carrier().suite()).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = root.join("essconform");
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(Path::new(&modules).join("@types"));
    }
    let compiled = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(compiled.status.success(), "{compiled:?}");
    write_vectors(&package.join("vectors"), &vectors);
    let typescript = node_verdicts(&package.join("dist/runtime.js"), &package, "");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        typescript,
        rust_verdicts(&vectors),
        "TypeScript (left) and Rust (right)"
    );
}

/// The browser reads only /5, /9 and /35; at /35 `covered_by` is a field Rust refuses.
#[test]
fn the_browser_reader_agrees_with_the_rust_reader_on_covered_by_at_35() {
    let vectors: BTreeMap<String, String> = vectors()
        .into_iter()
        .filter(|(name, _)| name.starts_with("35-"))
        .collect();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-covers-browser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_vectors(&root.join("vectors"), &vectors);
    let reader = root.join("coverage-admission.js");
    std::fs::write(&reader, include_str!("../assets/coverage-admission.js")).unwrap();
    let browser = node_verdicts(&reader, &root, "35-");
    let _ = std::fs::remove_dir_all(&root);
    let rust: BTreeMap<String, String> = rust_verdicts(&vectors)
        .into_iter()
        .map(|(name, verdict)| {
            let verdict = if verdict == "refused" {
                "refused"
            } else {
                "admitted"
            };
            (name, verdict.to_owned())
        })
        .collect();
    assert_eq!(browser, rust, "the browser replay (left) and Rust (right)");
}
