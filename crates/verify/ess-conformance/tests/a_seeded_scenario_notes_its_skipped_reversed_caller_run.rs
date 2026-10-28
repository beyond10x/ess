//! A seeded scenario on a caller model notes the swapped-caller run it does not get
//! (`story:a-seeded-scenario-notes-its-skipped-reversed-caller-run`).
//!
//! On a model whose actors carry caller attributes, every scenario sending a command that reads
//! the caller is run a second time in the same scenario with the two callers' roles swapped. The
//! unseeded twin below reaches `Use`'s `used` branch by arrangement (`DefineGrant` takes its flag
//! from input), so its `used` scenario carries that second run. The seeded model reaches the
//! branch only through the seed row `seed.yaml#flagged`, which is established by setup and whose
//! `account` fixes the caller the seeded run is sent as: a row established by setup cannot be
//! established again for the swapped callers, so the second run is left out. A note names the
//! scenario and that seed row. The unseeded suites keep their bytes.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::authored::Source as Authored;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{InstanceName, ScenarioStep};
use ess_conformance::synthesize::{
    synthesize, synthesize_with_seeds, AdmittedSeeds, Note, SeedSelection,
};
use ess_conformance::{AdmittedSuite, Runner, Synthesis};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use sha2::{Digest, Sha256};

const SYSTEM: &str = include_str!("fixtures/caller-model-seed/attr/system.yaml");
const COMPONENTS: &str = include_str!("fixtures/caller-model-seed/attr/components.yaml");
const DOMAIN: &str = include_str!("fixtures/caller-model-seed/attr/d.yaml");
const SEED: &str = include_str!("fixtures/caller-model-seed/seed.yaml");

const USED: &str = "demo.s.Use/outcome/used";
const USE: &str = "demo.s.Use";

/// The canonical JSON digests of the two unseeded suites, taken at the base before the note.
const SEEDED_MODEL_UNSEEDED_SHA256: &str =
    "6c8d4dcf4534d83a39515b20495457848539bf1a1084b19e9b8636c570a984d9";
const TWIN_UNSEEDED_SHA256: &str =
    "c75b27a131a029d76f6125269cb71e6d3a84602dcd7f378fd57860e06b940212";
/// The seeded suite's digest, taken at the same base: the note is not part of the suite.
const SEEDED_SHA256: &str = "e1ef5e4e4626392f0e3c1ca5466255bfd1d4937a26caa1389a040aa7e84e25aa";

/// The fixture's caller model, whose `used` branch only a seed row reaches.
fn seeded_model() -> EssIr {
    model(DOMAIN)
}

/// The same model with `DefineGrant` taking its flag from input, so arrangement reaches `used`.
fn twin() -> EssIr {
    let domain = DOMAIN
        .replacen(
            "    input: []\n",
            "    input:\n      - {name: flag, type: Boolean}\n",
            1,
        )
        .replacen("flag: 'false'", "flag: input.flag", 1);
    assert_ne!(domain, DOMAIN, "the twin differs from the fixture");
    model(&domain)
}

fn model(domain: &str) -> EssIr {
    let files = [
        ("system.yaml", SYSTEM),
        ("components.yaml", COMPONENTS),
        ("d.yaml", domain),
    ];
    let parsed = files.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn seeded(ir: &EssIr) -> Synthesis {
    let selection = SeedSelection {
        source: Authored::new("seed.yaml", SEED),
        instance: InstanceName::new("flagged").unwrap(),
    };
    let seeds = AdmittedSeeds::compile(ir, &[selection]).unwrap_or_else(|error| panic!("{error}"));
    synthesize_with_seeds(ir, &seeds)
        .unwrap_or_else(|error| panic!("the seed is admitted: {error}"))
}

/// The distinct callers (`account_id`) a scenario sends `Use` as.
fn use_callers(synthesis: &Synthesis, scenario: &str) -> BTreeSet<String> {
    let steps = &synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| panic!("{scenario} is written"))
        .1
        .steps;
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command, caller, ..
            } if command.to_string() == USE => Some(format!("{:?}", caller.get("account_id"))),
            _ => None,
        })
        .collect()
}

/// Every note about a skipped second run, by scenario: its rendered text.
fn unswapped(synthesis: &Synthesis) -> Vec<(String, String)> {
    synthesis
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::CrossCallerUnswapped { scenario, .. }
            | Note::UnswappedCallers { scenario, .. } => {
                Some((scenario.to_string(), note.to_string()))
            }
            _ => None,
        })
        .collect()
}

fn digest(synthesis: &Synthesis) -> String {
    let json = synthesis.suite.to_canonical_json().expect("serialises");
    Sha256::digest(json.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a String");
            hex
        })
}

fn not_passed(ir: &EssIr, synthesis: &Synthesis) -> Vec<String> {
    let json = synthesis.suite.to_canonical_json().expect("serialises");
    let admitted = AdmittedSuite::from_json(&json).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect()
}

/// The unseeded twin's `used` scenario is run for both callers; the seeded one for one only.
#[test]
fn the_seeded_scenario_skips_the_swapped_run_its_unseeded_twin_gets() {
    let twin = synthesize(&twin());
    assert_eq!(
        use_callers(&twin, USED).len(),
        2,
        "the twin runs `used` twice"
    );
    let seeded = seeded(&seeded_model());
    assert_eq!(
        use_callers(&seeded, USED).len(),
        1,
        "the seeded run is sent as one caller"
    );
    assert_eq!(digest(&seeded), SEEDED_SHA256);
}

/// The skipped run is named in a note carrying the scenario and the seed row that fixed its
/// caller; every scenario a seed row is applied to that sends a caller-reading command has one.
#[test]
fn a_seeded_scenario_notes_its_skipped_reversed_caller_run() {
    let ir = seeded_model();
    let synthesis = seeded(&ir);
    let notes = unswapped(&synthesis);
    let record = synthesis
        .suite
        .provenance
        .synthesis_seeds
        .as_ref()
        .expect("the seeds are recorded");
    assert_eq!(record.applications.len(), 1, "the one seed row is applied");
    for application in &record.applications {
        let scenario = application.scenario.to_string();
        let row = format!("{}#{}", application.source.as_str(), application.instance);
        let about: Vec<&String> = notes
            .iter()
            .filter(|(id, _)| *id == scenario)
            .map(|(_, text)| text)
            .collect();
        assert_eq!(about.len(), 1, "{scenario}: {notes:#?}");
        assert!(about[0].contains(&format!("`{row}`")), "{}", about[0]);
    }
    assert!(
        notes.iter().any(|(id, _)| id == USED),
        "{USED} is noted: {notes:#?}"
    );
    // The seeded suite still passes the interpreted target.
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());
}

/// Without seeds there is no setup row to skip a run for: the unseeded suites keep their bytes and
/// carry no note naming a seed row.
#[test]
fn an_unseeded_caller_model_keeps_its_suite_bytes_and_notes() {
    for (ir, pinned) in [
        (seeded_model(), SEEDED_MODEL_UNSEEDED_SHA256),
        (twin(), TWIN_UNSEEDED_SHA256),
    ] {
        let synthesis = synthesize(&ir);
        assert_eq!(digest(&synthesis), pinned);
        let notes = unswapped(&synthesis);
        assert!(
            notes.iter().all(|(_, text)| !text.contains("seed")),
            "{notes:#?}"
        );
    }
}
