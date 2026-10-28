//! Adversary pass 1 over the 0.75.0 wave's integrated branch.
//!
//! - U1 (`story:a-seed-says-it-does-not-serve-a-set-move-filter`): a set move's filter refusal
//!   names each selected seed row of the filtered entity. `seeds_unserved` lists four row reasons
//!   besides `UNARRANGED`; after the merge the wave's own fixtures reach only the set-move-only one.
//!   Here each other reason is reached with a seed of the filtered entity: `unwritten/` (a filter
//!   over a field no command writes), `arrives-only/` (an entry writing nothing whose rows rest only
//!   where its move arrives) and `undriven/` (an entry whose creator no arrangement takes), and
//!   `applied-elsewhere/` holds a seed row applied to a single-row branch while a set move over the
//!   same entity is refused.
//! - U4 (`story:a-seeded-scenario-notes-its-skipped-reversed-caller-run`): the note a seeded
//!   scenario on a caller model carries, read against what the scenario sends.

use std::collections::BTreeSet;

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

macro_rules! model_files {
    ($dir:literal) => {
        [
            (
                "system.yaml",
                include_str!(concat!("fixtures/adversary-0750/", $dir, "/system.yaml")),
            ),
            (
                "components.yaml",
                include_str!(concat!(
                    "fixtures/adversary-0750/",
                    $dir,
                    "/components.yaml"
                )),
            ),
            (
                "domains/s.yaml",
                include_str!(concat!("fixtures/adversary-0750/", $dir, "/domains/s.yaml")),
            ),
        ]
    };
}

const UNWRITTEN: [(&str, &str); 3] = model_files!("unwritten");
const ARRIVES_ONLY: [(&str, &str); 3] = model_files!("arrives-only");
const UNDRIVEN: [(&str, &str); 3] = model_files!("undriven");
const APPLIED_ELSEWHERE: [(&str, &str); 3] = model_files!("applied-elsewhere");

const UNWRITTEN_SEED: (&str, &str) = (
    "unwritten-seed.yaml",
    include_str!("fixtures/adversary-0750/unwritten-seed.yaml"),
);
const OWNER_SEED: (&str, &str) = (
    "owner-seed.yaml",
    include_str!("fixtures/adversary-0750/owner-seed.yaml"),
);
const APPLIED_SEED: (&str, &str) = (
    "applied-elsewhere-seed.yaml",
    include_str!("fixtures/adversary-0750/applied-elsewhere-seed.yaml"),
);

/// The U4 fixture: a caller model whose `Use/used` only a seed row reaches.
const CALLER_SYSTEM: &str = include_str!("fixtures/caller-model-seed/attr/system.yaml");
const CALLER_COMPONENTS: &str = include_str!("fixtures/caller-model-seed/attr/components.yaml");
const CALLER_DOMAIN: &str = include_str!("fixtures/caller-model-seed/attr/d.yaml");
const CALLER_SEED: (&str, &str) = (
    "seed.yaml",
    include_str!("fixtures/caller-model-seed/seed.yaml"),
);

const RESUMED: &str = "demo.s.Resume/outcome/resumed";
const PARKED: &str = "demo.s.Park/outcome/parked";
const CLOSED: &str = "demo.s.Close/outcome/closed";
const INSPECTED: &str = "demo.s.Inspect/outcome/inspected";
const USED: &str = "demo.s.Use/outcome/used";

/// What the refusal of a set move's filter says of a selected seed row.
const UNSERVED: &str = "seeds do not serve a set move's filter";
/// The cause of a filter over a field no declared command writes (`set_effects::UNWRITTEN`).
const UNWRITTEN_CAUSE: &str = "which a filter reads and no declared command writes";
const ARRIVES_ONLY_CAUSE: &str = "is moved by its entry only from the state the move arrives in";
const UNDRIVEN_CAUSE: &str = "which no arrangement takes for a row the filter selects";
const NO_MATCHING_CAUSE: &str =
    "which has no three rows the filter selects that the arranging commands can produce";

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

/// The refusal's cause line.
fn cause(refusal: &str) -> &str {
    refusal.lines().nth(1).expect("a cause line")
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

/// The seeded refusal of `scenario` keeps its unseeded cause whole, then names `rows` in that
/// order and says seeds do not serve the filter.
fn assert_names(seeded: &Synthesis, unseeded: &Synthesis, scenario: &str, rows: &[&str]) {
    let seeded = refusal_of(seeded, scenario);
    let unseeded = refusal_of(unseeded, scenario);
    let (cause, unseeded_cause) = (cause(&seeded), cause(&unseeded));
    assert!(
        cause.starts_with(unseeded_cause),
        "the refusal keeps its cause:\n{cause}\n{unseeded_cause}"
    );
    let named = rows
        .iter()
        .map(|row| format!("`{row}`"))
        .collect::<Vec<_>>()
        .join(", ");
    assert!(
        cause.contains(&named),
        "the refusal names the seed rows {named} of the filtered entity: {cause}"
    );
    assert!(cause.contains(UNSERVED), "{cause}");
}

fn applications(synthesis: &Synthesis) -> Vec<(String, String)> {
    synthesis
        .suite
        .provenance
        .synthesis_seeds
        .as_ref()
        .expect("the seeds are recorded")
        .applications
        .iter()
        .map(|application| {
            (
                application.scenario.to_string(),
                format!("{}#{}", application.source.as_str(), application.instance),
            )
        })
        .collect()
}

// ---- U1 -----------------------------------------------------------------------------------------

/// `unwritten/`: `Open` writes no `owner`, so the `instances:` filters of `Park` and `Resume` read
/// a field no command writes. The refusal is a set move's filter refusal, and a seed row of
/// `demo.s.Cap` is selected: the story's acceptance asks that the refusal name it and say seeds
/// do not serve a set move's filter, and `seeds_unserved` lists `UNWRITTEN` among the reasons it
/// handles. The `UNWRITTEN` gap's path is the field (`demo.s.Cap.owner`), not
/// `<outcome>.instances`, so `seeds_unserved` finds no filtered entity and returns the cause
/// as it was.
#[test]
fn an_unwritten_filter_refusal_names_the_seed_row_of_its_entity() {
    let ir = model(UNWRITTEN);
    let unseeded = synthesize(&ir);
    for scenario in [PARKED, RESUMED] {
        let refusal = refusal_of(&unseeded, scenario);
        assert!(cause(&refusal).contains(UNWRITTEN_CAUSE), "{refusal}");
    }
    let synthesis = seeded(&ir, UNWRITTEN_SEED, &["parked-cap"]);
    assert_eq!(applications(&synthesis), Vec::<(String, String)>::new());
    for scenario in [PARKED, RESUMED] {
        assert_names(
            &synthesis,
            &unseeded,
            scenario,
            &["unwritten-seed.yaml#parked-cap"],
        );
    }
}

/// `arrives-only/`: `Close/closed`'s entry moves `Cap` along `reset` (`Held`, `Parked` to
/// `Held`) and writes nothing; `Parked` is reached only by the set move `Park`, and `Held` is
/// arranged, so the refusal keeps `ARRIVES_ONLY`, not the set-move-only text. Both seed rows of
/// `Cap` are named in order; the `Owner` seed row is not.
#[test]
fn an_arrives_only_entry_refusal_names_its_seed_rows_in_order() {
    let ir = model(ARRIVES_ONLY);
    let unseeded = synthesize(&ir);
    assert_eq!(unseeded.refusals.len(), 1, "{:#?}", rendered(&unseeded));
    assert!(
        cause(&refusal_of(&unseeded, CLOSED)).contains(ARRIVES_ONLY_CAUSE),
        "{:#?}",
        rendered(&unseeded)
    );
    let synthesis = seeded(
        &ir,
        OWNER_SEED,
        &["parked-cap", "active-owner", "other-parked-cap"],
    );
    assert_eq!(synthesis.refusals.len(), 1, "{:#?}", rendered(&synthesis));
    assert_names(
        &synthesis,
        &unseeded,
        CLOSED,
        &[
            "owner-seed.yaml#other-parked-cap",
            "owner-seed.yaml#parked-cap",
        ],
    );
    assert!(!cause(&refusal_of(&synthesis, CLOSED)).contains("active-owner"));
    assert_eq!(applications(&synthesis), Vec::<(String, String)>::new());
    assert_eq!(ids(&synthesis), ids(&unseeded));
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());
}

/// `undriven/`: `Open` creates a `Cap` only for an owner that exists and is not `Active`, so no
/// row `Close/closed`'s entry selects is arranged (`UNDRIVEN`), and `Park/parked`'s `instances:`
/// has too few rows. One seed row is named by both refusals, an `affects:` entry's and an
/// `instances:` subject's, in one model.
#[test]
fn an_undriven_entry_and_an_instances_refusal_both_name_the_seed_row() {
    let ir = model(UNDRIVEN);
    let unseeded = synthesize(&ir);
    assert_eq!(unseeded.refusals.len(), 2, "{:#?}", rendered(&unseeded));
    assert!(cause(&refusal_of(&unseeded, CLOSED)).contains(UNDRIVEN_CAUSE));
    assert!(cause(&refusal_of(&unseeded, PARKED)).contains(NO_MATCHING_CAUSE));
    let synthesis = seeded(&ir, OWNER_SEED, &["parked-cap"]);
    assert_eq!(synthesis.refusals.len(), 2, "{:#?}", rendered(&synthesis));
    for scenario in [CLOSED, PARKED] {
        assert_names(
            &synthesis,
            &unseeded,
            scenario,
            &["owner-seed.yaml#parked-cap"],
        );
    }
    assert_eq!(applications(&synthesis), Vec::<(String, String)>::new());
    assert_eq!(ids(&synthesis), ids(&unseeded));
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());
}

/// `applied-elsewhere/`: the seed row `flagged-cap` (`Parked`, `flag: true`) is the only witness
/// of `Inspect/inspected`, whose subject read `flag == false` refuses every arranged row, while
/// `Resume/resumed`'s set move over the same entity is refused as set-move-only. The seed is
/// still applied to `inspected`, which passes, and the set move's refusal names it.
#[test]
fn a_seed_applied_to_a_subject_read_is_still_applied_beside_a_named_set_move_refusal() {
    let ir = model(APPLIED_ELSEWHERE);
    let unseeded = synthesize(&ir);
    assert!(rendered(&unseeded)
        .iter()
        .any(|refusal| refusal.contains(&format!("`{INSPECTED}`"))));
    let synthesis = seeded(&ir, APPLIED_SEED, &["flagged-cap"]);
    assert_eq!(
        applications(&synthesis),
        [(
            INSPECTED.to_owned(),
            "applied-elsewhere-seed.yaml#flagged-cap".to_owned()
        )]
    );
    assert!(ids(&synthesis).contains(&INSPECTED.to_owned()));
    assert_eq!(synthesis.refusals.len(), 1, "{:#?}", rendered(&synthesis));
    assert_names(
        &synthesis,
        &unseeded,
        RESUMED,
        &["applied-elsewhere-seed.yaml#flagged-cap"],
    );
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());
}

/// `applied-elsewhere/`, read for what the set move's refusal says of `flagged-cap`: the same run
/// applies the row to `inspected` and records it applied, so the refusal says the row is not
/// applied to this filter, not that synthesis does not apply it at all.
#[test]
fn a_seed_applied_elsewhere_is_refused_only_for_this_filter() {
    let ir = model(APPLIED_ELSEWHERE);
    let synthesis = seeded(&ir, APPLIED_SEED, &["flagged-cap"]);
    assert_eq!(applications(&synthesis).len(), 1);
    let refusal = refusal_of(&synthesis, RESUMED);
    let cause = cause(&refusal);
    assert!(
        cause.contains(
            "`applied-elsewhere-seed.yaml#flagged-cap`, which synthesis does not apply to this \
             filter: "
        ),
        "{cause}"
    );
    assert!(
        !cause.contains("which synthesis does not apply: "),
        "the refusal says the row is not applied anywhere: {cause}"
    );
}

// ---- U4 -----------------------------------------------------------------------------------------

fn caller_model() -> EssIr {
    let files = [
        ("system.yaml", CALLER_SYSTEM),
        ("components.yaml", CALLER_COMPONENTS),
        ("d.yaml", CALLER_DOMAIN),
    ];
    let parsed = files.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("{d}"))
}

/// The distinct non-empty callers a scenario sends any command as.
fn callers(synthesis: &Synthesis, scenario: &str) -> BTreeSet<String> {
    let (_, found) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| panic!("{scenario} is written"));
    found
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { caller, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { caller, .. }
                if !caller.is_empty() =>
            {
                Some(format!("{caller:?}"))
            }
            _ => None,
        })
        .collect()
}

/// `Note::CrossCallerUnswapped` is "the mixed-caller scenario is retained, but its reversed-role
/// counterpart cannot compose", rendered "`<scenario>` retains its mixed-caller witness without
/// an appended reversed-role run". U4 files its seeded-scenario note as that variant. On the U4
/// fixture the seeded `Use/used` sends every command as one caller (the U4 test pins
/// `use_callers == 1`), and the same suite carries a `CrossCallerUnwitnessed` note for it, "retains
/// its ordinary witness without a mixed-caller same-row witness": the two notes contradict, and
/// the first asserts a mixed-caller witness the scenario is not.
#[test]
fn a_seeded_single_caller_scenario_is_not_noted_as_a_mixed_caller_witness() {
    let ir = caller_model();
    let synthesis = seeded(&ir, CALLER_SEED, &["flagged"]);
    let sent_as = callers(&synthesis, USED);
    assert_eq!(
        sent_as.len(),
        1,
        "{USED} is sent as one caller: {sent_as:?}"
    );
    let about: Vec<String> = synthesis
        .notes
        .iter()
        .filter(|note| {
            matches!(note, Note::CrossCallerUnswapped { scenario, .. }
                if scenario.to_string() == USED)
        })
        .map(ToString::to_string)
        .collect();
    let unwitnessed: Vec<String> = synthesis
        .notes
        .iter()
        .filter(|note| {
            matches!(note, Note::CrossCallerUnwitnessed { scenario, .. }
                if scenario.to_string() == USED)
        })
        .map(ToString::to_string)
        .collect();
    assert!(
        about
            .iter()
            .all(|note| !note.contains("mixed-caller witness")),
        "a single-caller scenario is noted as retaining a mixed-caller witness:\n{about:#?}\n\
         beside:\n{unwitnessed:#?}"
    );
}

/// Pinned: the U4 note names the seed row and is the same on every run.
#[test]
fn the_seeded_scenario_note_is_deterministic_and_names_its_row() {
    let ir = caller_model();
    let first: Vec<String> = seeded(&ir, CALLER_SEED, &["flagged"])
        .notes
        .iter()
        .map(ToString::to_string)
        .collect();
    let second: Vec<String> = seeded(&ir, CALLER_SEED, &["flagged"])
        .notes
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(first, second);
    assert!(
        first
            .iter()
            .any(|note| note.starts_with(&format!("`{USED}`"))
                && note.contains("`seed.yaml#flagged`")),
        "{first:#?}"
    );
}
