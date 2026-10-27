//! A binding's per-refusal failure policy on a predicate refusal over a related row.
//!
//! `Give` refuses `not-flagged` where the grant its input names reads `flag == false`
//! (`when_related: {via: input.grant_id, predicate: …}`), and the binding drops that refusal. The
//! refusal is not `external:`, so it cannot be forced; but `DefineGrant` takes `flag` as input, so
//! the scenario arranges a grant the predicate refuses and sends the trigger naming it, as it sends
//! the trigger naming no grant for the `exists: false` refusal `missing`. Where no command creating
//! a grant can leave `flag` false, the refusal stays, and its help names the authored scenario.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const SYSTEM: &str = include_str!("fixtures/predicate-refusal-policy-arranged-row/system.yaml");
const COMPONENTS: &str =
    include_str!("fixtures/predicate-refusal-policy-arranged-row/components.yaml");
const DOMAIN: &str = include_str!("fixtures/predicate-refusal-policy-arranged-row/domains/d.yaml");

const PREDICATE: &str = "give-on-touched/binding/refusal/not-flagged";
const MISSING: &str = "give-on-touched/binding/refusal/missing";

fn model(system: &str, domain: &str) -> EssIr {
    let parsed = [
        ("system.yaml", system),
        ("components.yaml", COMPONENTS),
        ("domains/d.yaml", domain),
    ]
    .map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn steps(synthesis: &Synthesis, id: &str) -> Vec<serde_json::Value> {
    let document: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` has steps: {document:#}"))
        .clone()
}

fn not_passed(ir: EssIr, synthesis: &Synthesis) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
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
fn a_predicate_refusal_policy_is_forced_through_an_arranged_row() {
    let ir = model(SYSTEM, DOMAIN);
    let synthesis = synthesize(&ir);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, Vec::<String>::new());
    assert_eq!(
        synthesis.suite.scenarios.len(),
        10,
        "{:#?}",
        synthesis.suite.scenarios.keys()
    );
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == MISSING),
        "`{MISSING}` is synthesized"
    );

    // Answered by the grant the scenario arranges, not forced: no injection, a grant defined with
    // `flag: false` before the trigger, and the trigger sent naming it.
    let steps = steps(&synthesis, PREDICATE);
    assert!(
        steps
            .iter()
            .all(|step| step["step"] != "configure_external_outcome"),
        "nothing is injected: {steps:#?}"
    );
    let defined = steps
        .iter()
        .position(|step| {
            step["step"] == "execute_command"
                && step["command"] == "demo.d.DefineGrant"
                && step["input"]["flag"]["value"] == false
        })
        .unwrap_or_else(|| panic!("a grant is defined with `flag: false`: {steps:#?}"));
    let grant = steps[defined..]
        .iter()
        .find(|step| step["step"] == "capture_instance" && step["entity"] == "demo.d.Grant")
        .map_or_else(
            || panic!("the grant is captured: {steps:#?}"),
            |step| step["instance"].clone(),
        );
    let touched = steps
        .iter()
        .position(|step| step["step"] == "execute_command" && step["command"] == "demo.d.Touch")
        .unwrap_or_else(|| panic!("the trigger is sent: {steps:#?}"));
    assert!(
        defined < touched,
        "the grant precedes the trigger: {steps:#?}"
    );
    assert_eq!(
        steps[touched]["input"]["grant_id"]["instance"], grant,
        "the trigger names the grant defined with `flag: false`: {steps:#?}"
    );
    let counts: Vec<u64> = steps
        .iter()
        .filter(|step| step["step"] == "expect_invocation")
        .filter_map(|step| step["count"].as_u64())
        .collect();
    assert_eq!(counts, [1]);

    assert_eq!(not_passed(ir, &synthesis), Vec::<String>::new());
}

#[test]
fn a_predicate_refusal_no_row_can_answer_names_the_authored_scenario() {
    // No command creating a grant leaves `flag` false, so no grant the predicate refuses can be
    // arranged: the refusal stays, and its help does not send the author to `external:`.
    let domain = DOMAIN.replace("sets: {flag: input.flag}", "sets: {flag: true}");
    assert_ne!(domain, DOMAIN);
    let ir = model(SYSTEM, &domain);
    let synthesis = synthesize(&ir);
    let binding: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("give-on-touched/binding/"))
        .collect();
    let [refusal] = binding.as_slice() else {
        panic!("one binding refusal: {binding:#?}");
    };
    assert!(
        refusal.contains(&format!("no scenario `{PREDICATE}`")),
        "{refusal}"
    );
    assert!(refusal.contains("ESS-SYNTH-010"), "{refusal}");
    let help = refusal
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("help: "))
        .unwrap_or_else(|| panic!("a help line: {refusal}"));
    assert!(
        help.starts_with("cover its policy with an authored scenario (ess-scenario/1)"),
        "{help}"
    );
    assert!(!help.contains("external:"), "{help}");
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == MISSING),
        "`{MISSING}` is still synthesized"
    );
    assert_eq!(not_passed(ir, &synthesis), Vec::<String>::new());
}

#[test]
fn a_predicate_refusal_policy_arranged_for_two_callers_passes() {
    // The grant records its caller, so the arranged scenario runs a second, swapped pass: its
    // invocation count is cumulative over both passes, and both pass.
    let domain = DOMAIN
        .replace(
            "      - {name: flag, type: Boolean}\n    lifecycle",
            "      - {name: flag, type: Boolean}\n      - {name: account, type: demo.d.AccountRef}\n    lifecycle",
        )
        .replace(
            "sets: {flag: input.flag}",
            "sets: {flag: input.flag, account: {caller: account_id}}",
        )
        .replace(
            "  - {name: demo.d.GrantId, kind: newtype, of: String}\n",
            "  - {name: demo.d.GrantId, kind: newtype, of: String}\n  - {name: demo.d.AccountRef, kind: newtype, of: Uuid}\n",
        );
    let domain = format!(
        "{domain}actors:\n  - {{name: demo.d.Clerk, attributes: [{{name: account_id, type: demo.d.AccountRef}}], may: [demo.d.DefineGrant, demo.d.Touch, demo.d.Give]}}\n"
    );
    let system = SYSTEM.replace("format: ess/25", "format: ess/32");
    assert_ne!(system, SYSTEM);
    let ir = model(&system, &domain);
    let synthesis = synthesize(&ir);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, Vec::<String>::new());
    let counts: Vec<u64> = steps(&synthesis, PREDICATE)
        .iter()
        .filter(|step| step["step"] == "expect_invocation")
        .filter_map(|step| step["count"].as_u64())
        .collect();
    assert!(
        counts == [1, 2],
        "one count per pass, cumulative: {counts:?}"
    );
    assert_eq!(not_passed(ir, &synthesis), Vec::<String>::new());
}
