//! Every suite major the synthesizer can write into a generated package is admitted by the Go
//! runtime and by the TypeScript runtime (beyond10x/ess#188).
//!
//! 0.38.0 shipped a synthesizer that wrote `ess-conformance/26` for an unchanged specification and
//! two generated runtimes that stopped at `/21`, so an adopter's run refused the whole suite. The
//! list here is read off the registration — [`SUPPORTED_SUITE_FORMATS`] — never written out, so a
//! new major the synthesizer learns to write turns this red until both runtimes read it.
//!
//! The only majors left out are the ones a generated package cannot hold: the direct-response pair
//! ([`direct_response::ORDINARY`], [`direct_response::COVERAGE`]), which the Go and TypeScript
//! emitters refuse at generation. `only_the_direct_response_pair_is_left_out` keeps that
//! exclusion honest.

mod support_go;

use std::collections::BTreeSet;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::direct_response;
use ess_conformance::scenario::SUPPORTED_SUITE_FORMATS;
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
    SUPPORTED_SUITE_FORMATS
        .iter()
        .copied()
        .filter(|major| *major != direct_response::ORDINARY && *major != direct_response::COVERAGE)
        .collect()
}

/// Whether a major carries a coverage inventory: the odd majors from 5, as
/// `ess_conformance::coverage` admits them.
fn coverage(major: u32) -> bool {
    major >= 5 && major % 2 == 1
}

/// One admission document per major, keyed by the file name the Go driver prints.
fn documents() -> Vec<(String, String)> {
    let ordinary = serde_json::to_value(suite()).unwrap();
    let input = ess_conformance::coverage_build::build(
        &ir(),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let covered: serde_json::Value =
        serde_json::from_str(input.selected().original_json()).unwrap();
    let newest = SUPPORTED_SUITE_FORMATS.iter().copied().max().unwrap();
    emittable_majors()
        .into_iter()
        .chain([newest + 1])
        .map(|major| {
            let mut document = if coverage(major) {
                covered.clone()
            } else {
                ordinary.clone()
            };
            if !coverage(major) {
                document["scenarios"] = serde_json::json!({});
            }
            document["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
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

/// The generated packages cannot hold the direct-response pair: both emitters refuse a suite that
/// carries a direct-response observation, which is the only thing that selects /28 or /29.
#[test]
fn only_the_direct_response_pair_is_left_out() {
    let excluded: Vec<u32> = SUPPORTED_SUITE_FORMATS
        .iter()
        .copied()
        .filter(|major| !emittable_majors().contains(major))
        .collect();
    assert_eq!(
        excluded,
        [direct_response::ORDINARY, direct_response::COVERAGE],
        "only the direct-response pair is left out; `tests/direct_returns.rs` \
         (`pure_return_generators_refuse_unsupported_execution`) holds both emitters to refusing it"
    );
}
