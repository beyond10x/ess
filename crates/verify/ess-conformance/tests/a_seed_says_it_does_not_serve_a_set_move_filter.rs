//! A selected seed row of a set move's filtered entity is named in the refusal
//! (`story:a-seed-says-it-does-not-serve-a-set-move-filter`).
//!
//! A seed serves two obligations only (`docs/design/synthesis-seeds.md` § Arrangement and
//! obligation): the primary witness of a branch reading its stored subject row, and one side of a
//! stored counter's limit. A set move's filter needs several rows it selects, so a seed row of the
//! filtered entity is selected and recorded, and never applied. The refusal of that filter, an
//! `instances:` subject or an `affects:` entry, keeps its cause, names each selected seed row of
//! the filtered entity, says that seeds do not serve a set move's filter, and points to an
//! authored scenario. The same model without a seed keeps its refusal byte for byte, and a seed
//! row of another entity is not named.
//!
//! `instances/` is `demo.s.Cap`, whose `Parked` only the set move `Park/parked` reaches, so
//! `Resume/resumed` is refused. `affects/` moves the same rows back through an `affects:` entry of
//! `Close/closed`.

use std::fmt::Write as _;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::authored::Source as Authored;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::InstanceName;
use ess_conformance::synthesize::{
    synthesize, synthesize_with_seeds, AdmittedSeeds, SeedSelection,
};
use ess_conformance::{AdmittedSuite, Runner, Synthesis};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use sha2::{Digest, Sha256};

const INSTANCES: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("fixtures/set-move-filter-seed/instances/system.yaml"),
    ),
    (
        "components.yaml",
        include_str!("fixtures/set-move-filter-seed/instances/components.yaml"),
    ),
    (
        "domains/s.yaml",
        include_str!("fixtures/set-move-filter-seed/instances/domains/s.yaml"),
    ),
];
const AFFECTS: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("fixtures/set-move-filter-seed/affects/system.yaml"),
    ),
    (
        "components.yaml",
        include_str!("fixtures/set-move-filter-seed/affects/components.yaml"),
    ),
    (
        "domains/s.yaml",
        include_str!("fixtures/set-move-filter-seed/affects/domains/s.yaml"),
    ),
];
const INSTANCES_SEED: (&str, &str) = (
    "instances-seed.yaml",
    include_str!("fixtures/set-move-filter-seed/instances-seed.yaml"),
);
const AFFECTS_SEED: (&str, &str) = (
    "affects-seed.yaml",
    include_str!("fixtures/set-move-filter-seed/affects-seed.yaml"),
);

const RESUMED: &str = "demo.s.Resume/outcome/resumed";
const CLOSED: &str = "demo.s.Close/outcome/closed";

/// What the refusal of a set move's filter says of a selected seed row.
const UNSERVED: &str = "seeds do not serve a set move's filter";

/// The `instances:` refusal without a seed, as 0.74.0 printed it.
const RESUMED_UNSEEDED: &str = "refusal[ESS-SYNTH-001]: outcome demo.s.Resume/resumed has no \
     scenario `demo.s.Resume/outcome/resumed`\n  no witness: `resumed.instances` is `demo.s.Cap`, \
     whose move starts only in `Parked`, a state reached only by the set move \
     `demo.s.Park/parked`, which synthesis does not take to arrange a row for another scenario: a \
     state only a set move reaches is not arranged, so there are no three rows the filter selects \
     that the arranging commands can produce\n  help: a set move arranges no row for another \
     scenario; declare a single-row transition into the state (a branch moving the one instance \
     its `instance:` names there), or cover the branch with an authored scenario (ess-scenario/1)";

/// The `affects:` refusal without a seed, as 0.74.0 printed it.
const CLOSED_UNSEEDED: &str = "refusal[ESS-SYNTH-001]: outcome demo.s.Close/closed has no \
     scenario `demo.s.Close/outcome/closed`\n  no witness: `closed.affects[0]` is `demo.s.Cap`, \
     which is moved by its entry only from the state the move arrives in, as far as the arranging \
     commands reach, and the entry writes no field, so a target that skips the move reads the \
     same\n  help: let a command leave the rows in a `from` state of the move other than the one \
     it arrives in, or let the entry write a field (`sets:`) a view publishes, so a target that \
     skips the move reads differently";

/// The canonical JSON of the reproduction's seeded suite, taken at the base before the refusal
/// named its seed: the refusal text is not part of the ordinary suite.
const INSTANCES_SEEDED_SHA256: &str =
    "8018981a0e3ec0db298dc18851abed7f12a68007e006d81234a154ac0ab4eaaf";

fn model(files: [(&str, &str); 3]) -> EssIr {
    let parsed = files.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn seeded(ir: &EssIr, (origin, text): (&str, &str), instances: &[&str]) -> Synthesis {
    let selections: Vec<SeedSelection> = instances
        .iter()
        .map(|instance| SeedSelection {
            source: Authored::new(origin, text),
            instance: InstanceName::new(*instance).unwrap(),
        })
        .collect();
    let seeds = AdmittedSeeds::compile(ir, &selections).unwrap_or_else(|error| panic!("{error}"));
    synthesize_with_seeds(ir, &seeds).unwrap_or_else(|error| panic!("{error}"))
}

/// Every refusal, rendered as the CLI prints it.
fn rendered(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// The one refusal of `wanted`, rendered.
fn refusal_of(synthesis: &Synthesis, wanted: &str) -> String {
    let found: Vec<String> = rendered(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{wanted}`")))
        .collect();
    let [refusal] = found.as_slice() else {
        panic!("{wanted} is refused once: {found:#?}");
    };
    refusal.clone()
}

/// The refusal's cause and help lines.
fn lines(refusal: &str) -> (&str, &str) {
    let mut lines = refusal.lines().skip(1);
    let cause = lines.next().expect("a cause line");
    let help = lines.next().expect("a help line");
    assert_eq!(lines.next(), None, "{refusal}");
    (cause, help)
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn not_passed(ir: &EssIr, synthesis: &Synthesis) -> Vec<String> {
    let json = synthesis.suite.to_canonical_json().expect("serialises");
    let admitted = AdmittedSuite::from_json(&json).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    assert_eq!(report.scenarios.len(), synthesis.suite.scenarios.len());
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect()
}

fn sha256(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a String");
            hex
        })
}

/// The seeded refusal keeps the unseeded cause whole, then names `rows` and says seeds do not
/// serve the filter; its help points to an authored scenario.
fn assert_names_its_seeds(seeded: &str, unseeded: &str, rows: &[&str], absent: &[&str]) {
    let (cause, help) = lines(seeded);
    let (unseeded_cause, _) = lines(unseeded);
    assert_eq!(
        seeded.lines().next(),
        unseeded.lines().next(),
        "the refusal is about the same scenario"
    );
    assert!(
        cause.starts_with(unseeded_cause),
        "the refusal keeps its cause:\n{cause}\n{unseeded_cause}"
    );
    for row in rows {
        assert!(
            cause.contains(&format!("`{row}`")),
            "the refusal names the seed row {row}: {cause}"
        );
    }
    for row in absent {
        assert!(
            !cause.contains(row),
            "a seed row of another entity is not named, {row}: {cause}"
        );
    }
    assert!(cause.contains(UNSERVED), "{cause}");
    assert!(
        help.contains("authored scenario (ess-scenario/1)"),
        "the help points to an authored scenario: {help}"
    );
    assert!(
        help.contains("stored subject row") && help.contains("stored counter's limit"),
        "the help names the two obligations a seed serves: {help}"
    );
}

#[test]
fn a_seed_says_it_does_not_serve_a_set_move_filter() {
    let ir = model(INSTANCES);
    let unseeded = synthesize(&ir);
    let synthesis = seeded(&ir, INSTANCES_SEED, &["parked-cap"]);

    // The selection is recorded and applied nowhere; the scenarios are the unseeded ones, and the
    // suite's bytes are those it had before the refusal named its seed.
    let record = synthesis
        .suite
        .provenance
        .synthesis_seeds
        .as_ref()
        .expect("the seeds are recorded");
    assert_eq!(record.selections.len(), 1);
    assert!(record.applications.is_empty(), "{:#?}", record.applications);
    assert_eq!(ids(&synthesis), ids(&unseeded));
    let json = synthesis.suite.to_canonical_json().expect("serialises");
    assert_eq!(
        sha256(&json),
        INSTANCES_SEEDED_SHA256,
        "the suite bytes stay"
    );
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());

    assert_eq!(synthesis.refusals.len(), 1, "{:#?}", rendered(&synthesis));
    assert_names_its_seeds(
        &refusal_of(&synthesis, RESUMED),
        &refusal_of(&unseeded, RESUMED),
        &["instances-seed.yaml#parked-cap"],
        &[],
    );
}

#[test]
fn an_affects_entry_names_each_seed_row_of_its_entity() {
    let ir = model(AFFECTS);
    let unseeded = synthesize(&ir);
    let synthesis = seeded(
        &ir,
        AFFECTS_SEED,
        &["parked-cap", "other-parked-cap", "active-owner"],
    );
    assert_eq!(synthesis.refusals.len(), 1, "{:#?}", rendered(&synthesis));
    assert_names_its_seeds(
        &refusal_of(&synthesis, CLOSED),
        &refusal_of(&unseeded, CLOSED),
        &[
            "affects-seed.yaml#other-parked-cap",
            "affects-seed.yaml#parked-cap",
        ],
        &["active-owner"],
    );
    let record = synthesis
        .suite
        .provenance
        .synthesis_seeds
        .as_ref()
        .expect("the seeds are recorded");
    assert_eq!(record.selections.len(), 3);
    assert!(record.applications.is_empty(), "{:#?}", record.applications);
    assert_eq!(ids(&synthesis), ids(&unseeded));
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());
}

/// Control: without a seed, or with a seed row of another entity only, each refusal keeps its
/// text byte for byte.
#[test]
fn without_a_seed_of_the_filtered_entity_the_refusal_keeps_its_text() {
    let instances = model(INSTANCES);
    assert_eq!(rendered(&synthesize(&instances)), [RESUMED_UNSEEDED]);
    let affects = model(AFFECTS);
    assert_eq!(rendered(&synthesize(&affects)), [CLOSED_UNSEEDED]);
    assert_eq!(
        rendered(&seeded(&affects, AFFECTS_SEED, &["active-owner"])),
        [CLOSED_UNSEEDED]
    );
}
