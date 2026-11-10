//! Adversary cases for story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs: the
//! pair `ess-conformance/64` (ordinary) and `/65` (coverage) is read wherever `/62` and `/63` are,
//! in the two places the unit's suite does not reach: an explicitly selected coverage suite whose
//! retained parent is `/65` (`coverage::is_coverage_version`), and the generated TypeScript
//! runtime's coverage table (`COVERAGE_MAJORS` in `src/ts/runtime.ts`), which no other test reads
//! at `/65`.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::coverage::AdmittedInput;
use ess_conformance::{AdmittedSuite, ConformanceSuite};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const MODEL: &str = "format: ess/14
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.notes.Written
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
actors:
  - {name: demo.notes.Writer, may: [demo.notes.Write]}
commands:
  - name: demo.notes.Write
    input:
      - {name: text, type: String}
    outcomes:
      - name: written
        creates: demo.notes.Note
        instance: note_id
        emits: [demo.notes.Written]
        payload:
          demo.notes.Written: {note_id: {generated: true}, text: input.text}
        sets:
          text: input.text
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ordinary() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    synthesis.suite
}

/// The system's unfiltered coverage suite, original bytes.
fn coverage() -> String {
    ess_conformance::coverage_build::build(
        &ir(),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"))
    .selected()
    .original_json()
    .to_owned()
}

fn relabelled(document: &str, major: u32) -> String {
    let mut value: Value = serde_json::from_str(document).unwrap();
    value["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
    serde_json::to_string(&value).unwrap()
}

/// An input carrier: one scenario explicitly selected from the coverage suite at `major`, its
/// retained parent the unfiltered suite at the same major.
fn selected_carrier(major: u32) -> Result<String, String> {
    let parent = AdmittedSuite::from_json(&relabelled(&coverage(), major))
        .map_err(|error| format!("parent: {error}"))?;
    let input = AdmittedInput::from_suite(parent).map_err(|error| format!("input: {error}"))?;
    let first = input
        .selected()
        .suite()
        .scenarios
        .keys()
        .next()
        .cloned()
        .expect("a scenario");
    let selected = input
        .select(&[first])
        .map_err(|error| format!("select: {error}"))?;
    assert_eq!(
        selected
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        format!("ess-conformance/{major}")
    );
    Ok(selected.document().to_canonical_json().unwrap())
}

#[test]
fn an_explicit_selection_of_a_coverage_suite_65_is_admitted_as_one_of_63_is() {
    for major in [63, 65] {
        let carrier = selected_carrier(major)
            .unwrap_or_else(|error| panic!("suite/{major} explicit selection refused: {error}"));
        let read = AdmittedInput::from_json(&carrier)
            .unwrap_or_else(|error| panic!("suite/{major} carrier refused on read: {error}"));
        assert_eq!(read.parents().len(), 1);
    }
}

/// The emitted TypeScript package, its `runtime.ts` passed through `edit`, compiled.
fn typescript_package(label: &str, edit: impl Fn(&str) -> String) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-t1-pair-64-65-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit(&ordinary()).unwrap() {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let contents = if path.ends_with("src/runtime.ts") {
            edit(&artifact.contents)
        } else {
            artifact.contents.clone()
        };
        std::fs::write(path, contents).unwrap();
    }
    let package = root.join(ess_conformance::ts::PACKAGE);
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(Path::new(&modules).join("@types"));
    }
    let output = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    package
}

const DRIVER: &str = r"
import {readFileSync, readdirSync} from 'node:fs';
import {admitSuite, admitRunInput} from './dist/runtime.js';
for (const name of readdirSync('documents').sort()) {
  const raw = readFileSync(`documents/${name}`, 'utf8');
  try {
    if (name.startsWith('input-')) admitRunInput(raw); else admitSuite(raw);
    process.stdout.write(`ADMITTED ${name}\n`);
  } catch (error) {
    process.stdout.write(`REFUSED ${name}: ${String(error?.message ?? error)}\n`);
  }
}
";

/// Each document's verdict from the TypeScript runtime in `package`.
fn typescript_admission(package: &Path) -> String {
    let folder = package.join("documents");
    std::fs::create_dir_all(&folder).unwrap();
    let ordinary = ordinary().to_canonical_json().unwrap();
    let coverage = coverage();
    let documents = [
        ("suite-62.json", relabelled(&ordinary, 62)),
        ("suite-63.json", relabelled(&coverage, 63)),
        ("suite-64.json", relabelled(&ordinary, 64)),
        ("suite-65.json", relabelled(&coverage, 65)),
        ("wrong-64-with-coverage.json", relabelled(&coverage, 64)),
        ("wrong-65-without-coverage.json", relabelled(&ordinary, 65)),
        ("wrong-66.json", relabelled(&ordinary, 66)),
        ("input-63.json", selected_carrier(63).unwrap()),
        ("input-65.json", selected_carrier(65).unwrap()),
    ];
    for (name, contents) in &documents {
        std::fs::write(folder.join(name), contents).unwrap();
    }
    std::fs::write(package.join("admission.mjs"), DRIVER).unwrap();
    let output = Command::new("node")
        .arg("admission.mjs")
        .current_dir(package)
        .output()
        .expect("node runs");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{log}");
    log
}

fn verdicts(log: &str) -> Vec<(String, bool)> {
    log.lines()
        .filter_map(|line| {
            if let Some(name) = line.strip_prefix("ADMITTED ") {
                Some((name.to_owned(), true))
            } else {
                line.strip_prefix("REFUSED ")
                    .map(|rest| (rest.split(':').next().unwrap().to_owned(), false))
            }
        })
        .collect()
}

const EXPECTED: [(&str, bool); 9] = [
    ("input-63.json", true),
    ("input-65.json", true),
    ("suite-62.json", true),
    ("suite-63.json", true),
    ("suite-64.json", true),
    ("suite-65.json", true),
    ("wrong-64-with-coverage.json", false),
    ("wrong-65-without-coverage.json", false),
    ("wrong-66.json", false),
];

fn expected() -> Vec<(String, bool)> {
    EXPECTED
        .iter()
        .map(|(name, admitted)| ((*name).to_owned(), *admitted))
        .collect()
}

#[test]
fn the_typescript_runtime_reads_64_and_65_where_it_reads_62_and_63() {
    let package = typescript_package("as-emitted", str::to_owned);
    let log = typescript_admission(&package);
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    assert_eq!(verdicts(&log), expected(), "{log}");
}

/// The case above fails on the mutant the unit's suite misses: `/65` dropped from the TypeScript
/// runtime's `COVERAGE_MAJORS`. A probe of that claim, run against an edited copy of the emitted
/// package, never the tree.
#[test]
fn probe_the_case_above_fails_when_65_leaves_the_typescript_coverage_table() {
    let package = typescript_package("mutant-coverage-65", |runtime| {
        let mutated = runtime.replacen("61, 63, 65,\n]);", "61, 63,\n]);", 1);
        assert_ne!(mutated, runtime, "the coverage table line moved");
        mutated
    });
    let log = typescript_admission(&package);
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    assert_ne!(verdicts(&log), expected(), "{log}");
}
