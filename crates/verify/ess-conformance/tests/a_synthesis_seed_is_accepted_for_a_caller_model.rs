//! A synthesis seed for a model whose actors carry caller attributes
//! (`story:a-synthesis-seed-is-accepted-for-a-caller-model`).
//!
//! `DefineGrant` writes the caller's `account_id` into the grant it creates and never flags one,
//! so no arrangement reaches `Use`'s `used` branch: it needs a flagged grant, held by the account
//! of the caller who sends `Use`. A seed row supplies the flagged grant. Its `account` is a field
//! the caller attribute sets, so the seed implies the caller: the seeded run sends every command
//! as a caller whose `account_id` is the row's `account`, and the scenario passes the interpreted
//! target. Two rows implying two accounts for that one caller contradict the assignment and are
//! refused by the field. The same model without the attribute keeps its seeded suite's bytes.

use std::fmt::Write as _;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::authored::Source as Authored;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{InstanceName, ScenarioStep};
use ess_conformance::synthesize::{
    synthesize, synthesize_with_seeds, AdmittedSeeds, SeedSelection,
};
use ess_conformance::{AdmittedSuite, Runner, Synthesis};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use sha2::{Digest, Sha256};

const ATTR: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("fixtures/caller-model-seed/attr/system.yaml"),
    ),
    (
        "components.yaml",
        include_str!("fixtures/caller-model-seed/attr/components.yaml"),
    ),
    (
        "d.yaml",
        include_str!("fixtures/caller-model-seed/attr/d.yaml"),
    ),
];
const PLAIN: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!("fixtures/caller-model-seed/plain/system.yaml"),
    ),
    (
        "components.yaml",
        include_str!("fixtures/caller-model-seed/plain/components.yaml"),
    ),
    (
        "d.yaml",
        include_str!("fixtures/caller-model-seed/plain/d.yaml"),
    ),
];
const SEED: &str = include_str!("fixtures/caller-model-seed/seed.yaml");

const USED: &str = "demo.s.Use/outcome/used";

/// The canonical JSON of the plain twin's seeded suite, taken at the base before seeds were
/// admitted for a caller model.
const PLAIN_SEEDED_SHA256: &str =
    "9b8c0e51ca27b980ec6b7cd32df513fe12797058d1b4462c6dbb4823c7d4ea48";

fn model(files: [(&str, &str); 3]) -> EssIr {
    let parsed = files.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn seeds(ir: &EssIr, instances: &[&str]) -> AdmittedSeeds {
    let selections: Vec<SeedSelection> = instances
        .iter()
        .map(|instance| SeedSelection {
            source: Authored::new("seed.yaml", SEED),
            instance: InstanceName::new(*instance).unwrap(),
        })
        .collect();
    AdmittedSeeds::compile(ir, &selections).unwrap_or_else(|error| panic!("{error}"))
}

fn rendered(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
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

#[test]
fn a_synthesis_seed_is_accepted_for_a_caller_model() {
    let ir = model(ATTR);
    // Without the seed no arrangement reaches the branch.
    let unseeded = synthesize(&ir);
    assert!(
        !unseeded
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == USED),
        "the fixture needs its seed"
    );

    let synthesis = synthesize_with_seeds(&ir, &seeds(&ir, &["flagged"]))
        .unwrap_or_else(|error| panic!("the seed is admitted: {error}"));
    let refused: Vec<String> = rendered(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(USED))
        .collect();
    assert_eq!(refused, Vec::<String>::new());

    // The seeded row arranges the scenario it is offered for.
    let record = synthesis
        .suite
        .provenance
        .synthesis_seeds
        .as_ref()
        .expect("the seeds are recorded");
    let used: Vec<String> = record
        .applications
        .iter()
        .map(|application| format!("{}#{}", application.scenario, application.instance))
        .collect();
    assert_eq!(used, [format!("{USED}#flagged")]);
    let application = &record.applications[0];
    let steps = &synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == USED)
        .expect("the seeded scenario is written")
        .1
        .steps;
    let ScenarioStep::EstablishEntity { fields, .. } = &steps[application.establish_step] else {
        panic!("{steps:#?}")
    };
    assert_eq!(fields.get("account"), Some(&Node::Text("a-1".to_owned())));
    // Sent as the caller the row's caller-sourced field implies.
    let ScenarioStep::ExecuteCommand { caller, .. } = &steps[application.command_step] else {
        panic!("{steps:#?}")
    };
    assert_eq!(
        caller.get("account_id"),
        Some(&Node::Text("a-1".to_owned()))
    );

    // Every scenario passes the interpreted target.
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());
}

#[test]
fn a_seed_contradicting_the_caller_assignment_is_refused_by_the_field() {
    let ir = model(ATTR);
    let error = synthesize_with_seeds(&ir, &seeds(&ir, &["flagged", "other-account"]))
        .expect_err("two accounts for one caller contradict the assignment");
    let shown = error.to_string();
    assert_eq!(error.code(), "caller-contradiction", "{shown}");
    assert!(shown.contains("`account`"), "{shown}");
    assert!(shown.contains("`account_id`"), "{shown}");
}

#[test]
fn a_seed_row_whose_caller_set_field_is_null_is_refused_by_the_field() {
    let optional = ATTR[2].1.replace(
        "{name: account, type: demo.s.AccountRef}",
        "{name: account, type: Optional<demo.s.AccountRef>}",
    );
    let ir = model([ATTR[0], ATTR[1], ("d.yaml", optional.as_str())]);
    let selection = SeedSelection {
        source: Authored::new(
            "seed.yaml",
            SEED.replacen("account: a-1", "account: null", 1),
        ),
        instance: InstanceName::new("flagged").unwrap(),
    };
    let seeds = AdmittedSeeds::compile(&ir, &[selection]).unwrap_or_else(|error| panic!("{error}"));
    let error = synthesize_with_seeds(&ir, &seeds).expect_err("no caller carries null");
    let shown = error.to_string();
    assert_eq!(error.code(), "caller-contradiction", "{shown}");
    assert!(shown.contains("`account`"), "{shown}");
}

#[test]
fn seeded_synthesis_of_the_model_without_caller_attributes_keeps_its_suite_bytes() {
    let ir = model(PLAIN);
    let synthesis = synthesize_with_seeds(&ir, &seeds(&ir, &["flagged"])).unwrap();
    let json = synthesis.suite.to_canonical_json().expect("serialises");
    let digest = Sha256::digest(json.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a String");
            hex
        });
    assert_eq!(digest, PLAIN_SEEDED_SHA256);
    assert_eq!(rendered(&synthesis), Vec::<String>::new());
    assert_eq!(not_passed(&ir, &synthesis), Vec::<String>::new());
}
