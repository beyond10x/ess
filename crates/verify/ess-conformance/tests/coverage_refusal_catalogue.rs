//! Every refusal code a catalogue can write into a coverage inventory is one every coverage reader
//! admits, with the effect the writer gives it (beyond10x/ess#542).
//!
//! The Go and TypeScript readers named `ESS-SYNTH-001` to `-017` and the Rust one did not name
//! `-018` to `-022`, so a coverage document carrying a refusal any newer synthesis writes was
//! refused at admission. The codes here are read off the catalogues themselves
//! ([`RefusalCause::CATALOGUE`], [`Cause::CATALOGUE`]), so a code added to either is a vector here
//! before any reader has heard of it.
//!
//! The Rust reader is the oracle: for every generated code it admits exactly one effect, for
//! every file refusal code but `ESS-AUTHOR-036` it admits the code, and it refuses one past the
//! end of each catalogue. The generated Go and TypeScript runtimes and the browser replay give
//! the same answer for every vector they can read (the browser reads only `/5`, `/9` and `/35`).

mod support_go;

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use ess_conformance::authored::Cause;
use ess_conformance::synthesize::RefusalCause;
use ess_conformance::AdmittedSuite;
use serde_json::{json, Value};

const SCENARIO: &str = "example.domain/authored/created";
const EFFECTS: [&str; 2] = ["candidate_not_emitted", "check_not_emitted"];

/// A coverage document of suite `major` carrying exactly `refusal`.
fn document(major: u32, refusal: &Value) -> Value {
    let digest = |c: char| format!("sha256:{}", c.to_string().repeat(64));
    let mut sources = json!({
        "created.yaml": {"digest": digest('b'), "scenario": SCENARIO, "disposition": "accepted"}
    });
    if refusal["origin"] == "authored" {
        sources["refused.yaml"] =
            json!({"digest": digest('c'), "scenario": null, "disposition": "refused"});
    }
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
            "outside": [], "refused": [refusal], "authored_sources": sources,
            "counts": {"generated": 0, "authored": 1, "outside": 0, "refused": 1}
        }
    })
}

fn generated(number: u16, effect: &str) -> Value {
    json!({
        "origin": "generated", "scenario": null,
        "subject": {"kind": "command", "name": "example.domain.Make"}, "source": null,
        "code": format!("ESS-SYNTH-{number:03}"), "message": "refused", "effect": effect,
        "retained": null, "scope": "in_scope", "needs": []
    })
}

fn authored(number: u16) -> Value {
    json!({
        "origin": "authored", "scenario": null, "subject": null, "source": "refused.yaml",
        "code": format!("ESS-AUTHOR-{number:03}"), "message": "refused",
        "effect": "candidate_not_emitted", "retained": null, "scope": "in_scope", "needs": []
    })
}

/// Every vector, by file name: each generated code of the catalogue and one past it under both
/// effects, and each file refusal code, `036` and one past the catalogue, at suites /35 and /53.
fn vectors() -> BTreeMap<String, String> {
    let synth: Vec<u16> = RefusalCause::CATALOGUE.iter().map(|e| e.key).collect();
    let author: Vec<u16> = Cause::CATALOGUE.iter().map(|e| e.key).collect();
    let past = |keys: &[u16]| keys.iter().max().unwrap() + 1;
    let mut vectors = BTreeMap::new();
    for major in [35, 53] {
        for number in synth.iter().copied().chain([past(&synth)]) {
            for effect in EFFECTS {
                vectors.insert(
                    format!("{major}-synth-{number:03}-{effect}.json"),
                    document(major, &generated(number, effect)).to_string(),
                );
            }
        }
        for number in author.iter().copied().chain([past(&author)]) {
            vectors.insert(
                format!("{major}-author-{number:03}.json"),
                document(major, &authored(number)).to_string(),
            );
        }
    }
    vectors
}

fn verdict(admitted: bool) -> String {
    if admitted { "admitted" } else { "refused" }.to_owned()
}

fn rust_verdicts(vectors: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    vectors
        .iter()
        .map(|(name, raw)| (name.clone(), verdict(AdmittedSuite::from_json(raw).is_ok())))
        .collect()
}

/// `name verdict` lines, as the Go and node drivers write them.
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

#[test]
fn the_rust_reader_admits_every_catalogued_refusal_code_and_nothing_past_it() {
    let rust = rust_verdicts(&vectors());
    let admitted = |name: &str| rust[name] == "admitted";
    let effects = |major: u32, number: u16| {
        EFFECTS
            .into_iter()
            .filter(|effect| admitted(&format!("{major}-synth-{number:03}-{effect}.json")))
            .count()
    };
    let gated = |number: u16| ess_conformance::system_issued::REFUSALS.contains(&number);
    for entry in RefusalCause::CATALOGUE {
        assert_eq!(
            effects(53, entry.key),
            1,
            "ESS-SYNTH-{:03} at /53",
            entry.key
        );
        let at_35 = usize::from(!gated(entry.key));
        assert_eq!(
            effects(35, entry.key),
            at_35,
            "ESS-SYNTH-{:03} at /35",
            entry.key
        );
    }
    let past = u16::try_from(RefusalCause::CATALOGUE.len()).unwrap() + 1;
    assert_eq!(effects(53, past), 0, "one past the generated catalogue");
    for entry in Cause::CATALOGUE {
        for major in [35, 53] {
            assert_eq!(
                admitted(&format!("{major}-author-{:03}.json", entry.key)),
                entry.key != 36,
                "ESS-AUTHOR-{:03} at /{major}",
                entry.key
            );
        }
    }
    let past = u16::try_from(Cause::CATALOGUE.len()).unwrap() + 1;
    assert!(!admitted(&format!("53-author-{past:03}.json")));
}

const GO_DRIVER: &str = r#"package essconform

import (
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

func TestCoverageVectors(t *testing.T) {
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
		verdict := "admitted"
		if _, err := admitRunInput(string(raw)); err != nil {
			verdict = "refused"
		}
		lines = append(lines, entry.Name()+" "+verdict)
	}
	sort.Strings(lines)
	if err := os.WriteFile(os.Getenv("ESS_COVERAGE_VERDICTS"), []byte(strings.Join(lines, "\n")), 0o644); err != nil {
		t.Fatal(err)
	}
}
"#;

/// A suite the generated runtimes run, to carry the reader into a package.
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

#[test]
fn the_go_reader_admits_what_the_rust_reader_admits() {
    let vectors = vectors();
    let directory = support_go::package(
        "coverage-codes",
        carrier().suite(),
        &[("coverage_vectors_test.go", GO_DRIVER)],
    );
    let input = directory.join("vectors");
    write_vectors(&input, &vectors);
    let output = directory.join("verdicts.txt");
    let run = support_go::go_test(
        &directory,
        "TestCoverageVectors",
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
  let verdict = 'admitted';
  try { await (module.admitRunInput ?? module.admitSuite)(raw); } catch { verdict = 'refused'; }
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
fn the_typescript_reader_admits_what_the_rust_reader_admits() {
    let vectors = vectors();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("coverage-codes-ts-{}", std::process::id()));
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

#[test]
fn the_browser_reader_admits_what_the_rust_reader_admits_at_35() {
    let vectors: BTreeMap<String, String> = vectors()
        .into_iter()
        .filter(|(name, _)| name.starts_with("35-"))
        .collect();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("coverage-codes-browser-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_vectors(&root.join("vectors"), &vectors);
    let reader = root.join("coverage-admission.js");
    std::fs::write(&reader, include_str!("../assets/coverage-admission.js")).unwrap();
    let browser = node_verdicts(&reader, &root, "35-");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        browser,
        rust_verdicts(&vectors),
        "the browser replay (left) and Rust (right)"
    );
}
