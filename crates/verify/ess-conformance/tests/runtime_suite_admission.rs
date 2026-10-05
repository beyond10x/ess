//! Every suite major the synthesizer can write into a generated package is admitted by the Go
//! runtime and by the TypeScript runtime (beyond10x/ess#188).
//!
//! 0.38.0 shipped a synthesizer that wrote `ess-conformance/26` for an unchanged specification and
//! two generated runtimes that stopped at `/21`, so an adopter's run refused the whole suite. The
//! list here is read off the registration — [`SUPPORTED_SUITE_FORMATS`] — never written out, so a
//! new major the synthesizer learns to write turns this red until both runtimes read it.
//!
//! No supported major is exempt. Admission is a necessary condition; the runtime parity tests
//! additionally exercise each feature against healthy and faulty targets.

mod support_go;
mod support_versions;

use std::collections::BTreeSet;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::{ScenarioInitialState, SUPPORTED_SUITE_FORMATS};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// A small specification, for a real suite and a real coverage inventory.
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

/// One row of the model to seed with: no obligation of this model needs it.
const NOTE_SEED: &str = "type: ess-scenario/2
domain: demo.notes
scenario: kept-note
summary: A note established without a command.
arrange:
  - instance: kept
    entity: demo.notes.Note
    setup:
      identity: kept-note
      fields: {text: kept}
      state: Open
assert:
  - view: demo.notes.Notes
    contains: {note_id: {$instance: kept}, text: kept}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    synthesis.suite
}

/// Every suite major a generated Go or TypeScript package can be asked to run.
fn emittable_majors() -> BTreeSet<u32> {
    SUPPORTED_SUITE_FORMATS.iter().copied().collect()
}

/// Whether a major carries a coverage inventory: the odd majors from 5, as
/// `ess_conformance::coverage` admits them.
fn coverage(major: u32) -> bool {
    major >= 5 && major % 2 == 1
}

/// One admission document per major, keyed by the file name the Go driver prints.
fn documents() -> Vec<(String, String)> {
    let ordinary = suite();
    assert_eq!(ordinary.provenance.suite_version.major(), 34);
    assert_eq!(
        ordinary.provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    let ordinary = ordinary.to_canonical_json().unwrap();
    let input = ess_conformance::coverage_build::build(
        &ir(),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let selected = input.selected();
    assert_eq!(selected.suite().provenance.suite_version.major(), 35);
    assert_eq!(
        selected.suite().provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    let covered = selected.original_json();
    // The seed-bearing pair is written only with its record (beyond10x/ess#413): a real seeded
    // synthesis of the same model, with one admitted row no obligation needs.
    let seeds = ess_conformance::synthesize::AdmittedSeeds::compile(
        &ir(),
        &[ess_conformance::synthesize::SeedSelection {
            source: ess_conformance::authored::Source::new("note.yaml", NOTE_SEED),
            instance: ess_conformance::InstanceName::new("kept").unwrap(),
        }],
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let seeded = ess_conformance::synthesize::synthesize_with_seeds(&ir(), &seeds)
        .unwrap()
        .suite
        .to_canonical_json()
        .unwrap();
    let seeded_input = ess_conformance::coverage_build::build_with_seeds(
        &ir(),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
        &seeds,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let seeded_covered = seeded_input.selected().original_json();
    let newest = SUPPORTED_SUITE_FORMATS.iter().copied().max().unwrap();
    emittable_majors()
        .into_iter()
        .chain([newest + 1])
        .map(|major| {
            let current = match major {
                42 => &seeded,
                43 => seeded_covered,
                _ if coverage(major) => covered,
                _ => &ordinary,
            };
            let json = if major < 34 {
                support_versions::legacy_json(current, major)
            } else {
                let mut document: serde_json::Value = serde_json::from_str(current).unwrap();
                document["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
                serde_json::to_string(&document).unwrap()
            };
            let mut document: serde_json::Value = serde_json::from_str(&json).unwrap();
            if !coverage(major) {
                document["scenarios"] = serde_json::json!({});
            }
            (
                format!("suite-{major:02}.json"),
                serde_json::to_string(&document).unwrap(),
            )
        })
        .collect()
}

#[test]
fn go_admits_every_suite_major_the_synthesizer_writes() {
    let directory = support_go::package(
        "admission",
        &suite(),
        &[(
            "admission_test.go",
            include_str!("fixtures/suite-admission-go.go"),
        )],
    );
    let folder = directory.join("documents");
    std::fs::create_dir_all(&folder).unwrap();
    for (name, contents) in documents() {
        std::fs::write(folder.join(name), contents).unwrap();
    }
    let run = support_go::go_test(
        &directory,
        "TestSuiteAdmission",
        &[("ESS_ADMISSION_DIR", folder.to_str().unwrap())],
    );
    assert!(run.success, "{}", run.log);
    let newest = SUPPORTED_SUITE_FORMATS.iter().copied().max().unwrap();
    let mut refused = Vec::new();
    for major in emittable_majors() {
        let name = format!("suite-{major:02}.json");
        if !run.log.contains(&format!("ADMITTED {name}\n")) {
            refused.push(
                run.log
                    .lines()
                    .find(|line| line.contains(&name))
                    .unwrap_or(&name)
                    .to_owned(),
            );
        }
    }
    assert!(
        refused.is_empty(),
        "the Go runtime refuses suite majors the synthesizer writes:\n{}",
        refused.join("\n")
    );
    let future = format!(
        "REFUSED suite-{:02}.json: unsupported suite version",
        newest + 1
    );
    assert!(
        run.log.contains(&future),
        "a major this build does not know is refused by version: {}",
        run.log
    );
    std::fs::remove_dir_all(directory).unwrap();
}

/// The majors the TypeScript runtime admits.
///
/// Read from its admission table, `SUITE_MAJORS` in `src/ts/runtime.ts`. The TypeScript unit of
/// story `generated-runtimes-run-every-emitted-suite-version` owns making this complete; it may
/// replace this reading with an executed admission, and nothing else in this file changes.
fn typescript_admitted_majors() -> BTreeSet<u32> {
    let source = include_str!("../src/ts/runtime.ts");
    let table = source
        .split("const SUITE_MAJORS")
        .nth(1)
        .and_then(|rest| rest.split("};").next())
        .expect("src/ts/runtime.ts declares its SUITE_MAJORS admission table");
    table
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("'ess-conformance/")?
                .split('\'')
                .next()?
                .parse()
                .ok()
        })
        .collect()
}

#[test]
fn typescript_admits_every_suite_major_the_synthesizer_writes() {
    let admitted = typescript_admitted_majors();
    let missing: Vec<u32> = emittable_majors()
        .into_iter()
        .filter(|major| !admitted.contains(major))
        .collect();
    assert!(
        missing.is_empty(),
        "the TypeScript runtime refuses suite majors the synthesizer writes: {missing:?}. \
         Owned by the ts unit of story:generated-runtimes-run-every-emitted-suite-version \
         (beyond10x/ess#188); red until that unit lands."
    );
}
