//! Adversary pass 1 over the integrated 0.74.0 wave (U1 a binding tries every publisher of its
//! trigger, U2 synthesis seeds for a caller model, U3 a state only a set move reaches is named in
//! the refusal).
//!
//! Each model is synthesized and every scenario run against the interpreter of the same model.
//! The fixtures under `fixtures/adversary-0740/` hold no `format:` line and no `system.yaml`, so the
//! repository-wide suite-bytes tables do not walk them; the system document is written here.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::authored::Source as Authored;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::InstanceName;
use ess_conformance::synthesize::{
    synthesize, synthesize_with_seeds, AdmittedSeeds, SeedSelection, Synthesis,
};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SEED: &str = include_str!("fixtures/caller-model-seed/seed.yaml");
const SET_MOVE_ONLY: &str = include_str!("fixtures/set-move-only-state.yaml");

fn system(domain: &str, format: &str) -> String {
    format!("format: {format}\nsystem: demo\nversion: v1\ndomains:\n  - {domain}\n")
}

fn components(domain: &str, commands: &[&str]) -> String {
    format!(
        "components:\n  - component: svc\n    owns:\n      domains: [{domain}]\n    accepts:\n      \
         commands: [{}]\n",
        commands.join(", ")
    )
}

fn model(files: &[(&str, &str)]) -> EssIr {
    let mut sources = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.push((Source::new(*label), raw));
    }
    let spec = Specification::assemble(sources)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

/// Every scenario the interpreter of the same model does not pass, with its diagnostics.
fn failing(ir: &EssIr, synthesis: &Synthesis) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    assert_eq!(report.scenarios.len(), synthesis.suite.scenarios.len());
    report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics: Vec<String> = result.diagnostics().map(|d| format!("{d:?}")).collect();
            (
                result.scenario.to_string(),
                format!("{:?}: {diagnostics:#?}", result.status),
            )
        })
        .collect()
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

/// U1. `CloseAll` closes every seat of a team and publishes `Left` once per changed seat that was
/// `Busy`; `Busy` is reached by the single-row move `Take`. The binding `log-on-left` is set off
/// only by `CloseAll`, so its trigger needs a `Busy` seat of the team arranged first. The binding
/// is witnessed, not refused, and every scenario passes the interpreted target.
#[test]
fn p1_a_per_row_trigger_whose_row_rests_where_a_single_row_move_takes_it() {
    let ir = model(&[
        ("system.yaml", &system("demo.desk", "ess/25")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0740/desk/components.yaml"),
        ),
        (
            "d.yaml",
            include_str!("fixtures/adversary-0740/desk/d.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    // `Log` has no branch that can be forced to fail, so the failure policy alone is refused.
    let binding: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains("log-on-left"))
        .filter(|refusal| !refusal.contains("log-on-left/binding/on-failure"))
        .collect();
    assert_eq!(binding, Vec::<String>::new(), "{:#?}", ids(&synthesis));
    assert!(
        ids(&synthesis)
            .iter()
            .any(|id| id == "log-on-left/binding/mapping"),
        "{:#?}",
        ids(&synthesis)
    );
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// U1. `SyncAll` (first in name order) publishes `Changed` once per ticket it updates; the only
/// creator of tickets, `ZOpenTicket`, publishes `Changed` itself. The row arranged before the
/// `SyncAll` trigger is opened through `ZOpenTicket`, which sets the binding off once more in the
/// arrangement. Every scenario passes the interpreted target.
#[test]
fn p2_a_creator_publishing_the_trigger_event_in_the_arranged_row() {
    let ir = model(&[
        ("system.yaml", &system("demo.e", "ess/25")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0740/creator-publishes/components.yaml"),
        ),
        (
            "e.yaml",
            include_str!("fixtures/adversary-0740/creator-publishes/e.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    assert_eq!(
        failing(&ir, &synthesis),
        BTreeMap::new(),
        "{:#?}",
        refusals(&synthesis)
    );
}

/// U2. `DefineGrant` writes `account` from its input, and only `Claim` writes it from the caller's
/// `account_id`. Rows holding two accounts are rows any caller can create, so no caller
/// contradiction: the two seeds are admitted and the seeded suite passes the interpreted target.
#[test]
fn p3_seed_rows_on_a_field_a_creator_writes_from_input_contradict_no_caller() {
    let ir = model(&[
        ("system.yaml", &system("demo.s", "ess/32")),
        (
            "components.yaml",
            &components(
                "demo.s",
                &[
                    "demo.s.DefineGrant",
                    "demo.s.Claim",
                    "demo.s.Touch",
                    "demo.s.Use",
                ],
            ),
        ),
        (
            "d.yaml",
            include_str!("fixtures/adversary-0740/input-and-caller/d.yaml"),
        ),
    ]);
    let synthesis = synthesize_with_seeds(&ir, &seeds(&ir, &["flagged", "other-account"]))
        .unwrap_or_else(|error| panic!("rows any caller can create are admitted: {error}"));
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// U2. `Clerk` creates the grant, writing its `account_id` into `account`; `Manager`, which
/// declares the same attribute, sends `Use`, whose `used` branch needs a flagged grant of the
/// sender's account. The seed supplies one for `a-1`: the seeded row arranges the scenario it is
/// offered for, `Use/outcome/used`, and it passes the interpreted target.
#[test]
fn p4_a_seed_arranges_its_scenario_when_another_actor_sends_the_command() {
    let ir = model(&[
        ("system.yaml", &system("demo.s", "ess/32")),
        (
            "components.yaml",
            &components(
                "demo.s",
                &["demo.s.DefineGrant", "demo.s.Touch", "demo.s.Use"],
            ),
        ),
        (
            "d.yaml",
            include_str!("fixtures/adversary-0740/two-actors/d.yaml"),
        ),
    ]);
    let synthesis = synthesize_with_seeds(&ir, &seeds(&ir, &["flagged"]))
        .unwrap_or_else(|error| panic!("the seed is admitted: {error}"));
    let used: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains("demo.s.Use/outcome/used"))
        .collect();
    assert_eq!(used, Vec::<String>::new(), "{:#?}", ids(&synthesis));
    assert!(ids(&synthesis)
        .iter()
        .any(|id| id == "demo.s.Use/outcome/used"));
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// U2, the contrast to `p4`: the same model with `Manager` also allowed to send `DefineGrant`, so
/// the seed implies its `account_id` too, and the seeded row arranges `Use/outcome/used`.
#[test]
fn p4_contrast_the_seed_arranges_its_scenario_when_the_sender_may_create() {
    let domain = include_str!("fixtures/adversary-0740/two-actors/d.yaml");
    let widened = domain.replacen(
        "may: [demo.s.Touch, demo.s.Use]",
        "may: [demo.s.DefineGrant, demo.s.Touch, demo.s.Use]",
        1,
    );
    assert_ne!(widened, domain);
    let ir = model(&[
        ("system.yaml", &system("demo.s", "ess/32")),
        (
            "components.yaml",
            &components(
                "demo.s",
                &["demo.s.DefineGrant", "demo.s.Touch", "demo.s.Use"],
            ),
        ),
        ("d.yaml", &widened),
    ]);
    let synthesis = synthesize_with_seeds(&ir, &seeds(&ir, &["flagged"]))
        .unwrap_or_else(|error| panic!("the seed is admitted: {error}"));
    assert!(
        ids(&synthesis)
            .iter()
            .any(|id| id == "demo.s.Use/outcome/used"),
        "{:#?}",
        refusals(&synthesis)
    );
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// U2. `Use` refuses the sender's own grant (`account == caller.account_id`) and uses another's.
/// The seed's row implies the caller `a-1`, the very account its `used` branch must not be sent
/// as: whatever synthesis writes for the branch passes the interpreted target.
#[test]
fn p7_a_seed_whose_implied_caller_contradicts_the_seeded_branch_guard() {
    let attr = include_str!("fixtures/caller-model-seed/attr/d.yaml");
    let own = attr.replacen(
        "- name: foreign\n        when_subject: {predicate: account != caller.account_id}",
        "- name: own\n        when_subject: {predicate: account == caller.account_id}",
        1,
    );
    assert_ne!(own, attr);
    let ir = model(&[
        (
            "system.yaml",
            include_str!("fixtures/caller-model-seed/attr/system.yaml"),
        ),
        (
            "components.yaml",
            include_str!("fixtures/caller-model-seed/attr/components.yaml"),
        ),
        ("d.yaml", &own),
    ]);
    let synthesis = synthesize_with_seeds(&ir, &seeds(&ir, &["flagged"]))
        .unwrap_or_else(|error| panic!("the seed is admitted: {error}"));
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// U1 under a caller model. `SitDown` writes the seat's `team` from the caller's `team_id`, and
/// `CloseAll` publishes `Left` once per closed `Busy` seat of the team it is sent. Every scenario,
/// with its swapped and reversed caller runs, passes the interpreted target.
#[test]
fn p8_a_per_row_trigger_whose_row_a_caller_attribute_keys() {
    let ir = model(&[
        ("system.yaml", &system("demo.desk", "ess/25")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0740/desk/components.yaml"),
        ),
        (
            "d.yaml",
            include_str!("fixtures/adversary-0740/desk-caller/d.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    assert_eq!(
        failing(&ir, &synthesis),
        BTreeMap::new(),
        "{:#?}",
        refusals(&synthesis)
    );
}

/// U3. With the creator writing `key` from a generated value, `Resume/resumed` misses its rows for
/// two reasons: no three children share a key, and `Parked` is reached only by a set move. The
/// refusal's help still names the filter's cause, which a single-row transition into `Parked`
/// alone does not cure.
#[test]
fn p5_a_refusal_whose_filter_is_also_unmet_keeps_the_filter_cause() {
    let text = SET_MOVE_ONLY.replacen(
        "sets: {key: input.key}",
        "sets: {key: {generated: true}}",
        1,
    );
    assert_ne!(text, SET_MOVE_ONLY);
    let ir = model(&[("spec.yaml", &text)]);
    let synthesis = synthesize(&ir);
    let resumed: Vec<(String, String)> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some("demo.z.Resume/outcome/resumed")
        })
        .map(|refusal| (refusal.cause.to_string(), refusal.hint().to_string()))
        .collect();
    let [(cause, help)] = resumed.as_slice() else {
        panic!("{:#?}", refusals(&synthesis));
    };
    assert!(
        help.contains("rather than from a literal or a generated value"),
        "the help names the generated key: {cause}\n{help}"
    );
}

/// U1 meets U3. `Resume` publishes `Resumed` once per child it moves out of `Parked`, a state only
/// the set move `Park` reaches, so no row can be arranged before the trigger and the binding is
/// refused rather than filed unpassable. The refusal names the state, not a key to write from an
/// input, which the creator already does.
#[test]
fn p6_a_per_row_trigger_from_a_state_only_a_set_move_reaches() {
    let ir = model(&[
        ("system.yaml", &system("demo.z", "ess/25")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0740/set-move-trigger/components.yaml"),
        ),
        (
            "z.yaml",
            include_str!("fixtures/adversary-0740/set-move-trigger/z.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
    let binding: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.to_string().contains("note-on-resumed"))
        .map(|refusal| format!("{refusal}\n{}", refusal.hint()))
        .collect();
    assert!(
        !binding.is_empty() && binding.iter().all(|refusal| refusal.contains("Parked")),
        "{binding:#?}"
    );
}
