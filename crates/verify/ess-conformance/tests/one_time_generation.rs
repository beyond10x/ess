//! The source policy must install observation obligations without manual DTO injection.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    authored,
    coverage::{Origins, Scope},
    coverage_build, ScenarioStep,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("../../../../docs/design/one-time-response-values.example.yaml");
const AUTHORED: &str = "type: ess-scenario/4\ndomain: credentials.api\nscenario: issued\nsummary: Issuance carries a protected value.\ntimeline:\n  - at: 2026-10-02T00:00:00Z\n    command: credentials.api.Issue\n    outcome: issued\n";

fn model(text: &str) -> ess_compiler::EssIr {
    let spec = Specification::assemble([(
        Source::new("one-time.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn generated_source_policy_installs_trace_and_rotation_obligations() {
    let synthesis = ess_conformance::synthesize(&model(MODEL));
    assert_eq!(synthesis.suite.provenance.suite_version.major(), 34);
    assert!(synthesis.suite.scenarios.values().any(|scenario| scenario
        .one_time_response
        .is_some()
        && scenario
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .count()
            >= 2));
    ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
}

#[test]
fn generated_constrained_string_policy_has_an_executable_witness() {
    let source = MODEL.replace("type: String", "type: credentials.api.Secret") + "\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    prefix: tok_\n    invariants: [value.count >= 8]\n";
    let synthesis = ess_conformance::synthesize(&model(&source));
    assert!(synthesis.suite.scenarios.values().any(|scenario| scenario
        .one_time_response
        .as_ref()
        .is_some_and(|policy| !policy.origins[0].response.constraints.is_empty())));
}

#[test]
fn equally_authorized_actors_both_get_disclosure_cells() {
    let source = format!("{MODEL}\nactors:\n  - {{name: credentials.api.Alice, may: [credentials.api.Issue]}}\n  - {{name: credentials.api.Bob, may: [credentials.api.Issue]}}\n");
    let synthesis = ess_conformance::synthesize(&model(&source));
    for name in ["credentials.api.Alice", "credentials.api.Bob"] {
        assert!(synthesis.suite.scenarios.values().any(|scenario| scenario.one_time_response.is_some() && scenario.steps.iter().any(|step| matches!(step, ScenarioStep::ExecuteCommand { actor: Some(actor), .. } if actor.to_string() == name))), "missing actor {name}");
    }
}

#[test]
fn authored_and_coverage_producers_preserve_the_policy() {
    let ir = model(MODEL);
    let authoring = authored::compile(&ir, &[authored::Source::new("issued.yaml", AUTHORED)]);
    assert!(authoring.refusals.is_empty(), "{:?}", authoring.refusals);
    assert!(authoring
        .scenarios
        .values()
        .all(|scenario| scenario.one_time_response.is_some()));
    let source = coverage_build::CoverageSource::new("issued.yaml", AUTHORED).unwrap();
    let coverage =
        coverage_build::build(&ir, &[source], Scope::System, Origins::GeneratedAndAuthored)
            .unwrap();
    assert_eq!(
        coverage.selected().suite().provenance.suite_version.major(),
        35
    );
}

#[test]
fn every_declared_followup_and_actor_has_a_cell_or_a_named_refusal() {
    let source = format!("{MODEL}\n  - name: credentials.api.Read\n    response: [{{name: summary, type: String}}]\n    outcomes: [{{name: read, returns: true}}]\nactors:\n  - {{name: credentials.api.Alice, may: [credentials.api.Issue, credentials.api.Read]}}\n  - {{name: credentials.api.Bob, may: [credentials.api.Issue]}}\nentities:\n  - name: credentials.api.Record\n    identity: {{name: id, type: Uuid}}\n    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}\nviews:\n  - name: credentials.api.Records\n    source: credentials.api.Record\n    consistency: read_your_writes\n    fields: [{{name: id, type: Uuid}}]\n");
    let synthesis = ess_conformance::synthesize(&model(&source));
    let ids: std::collections::BTreeSet<_> = synthesis
        .suite
        .scenarios
        .keys()
        .chain(
            synthesis
                .refusals
                .iter()
                .filter_map(|refusal| refusal.scenario.as_ref()),
        )
        .map(ToString::to_string)
        .collect();
    for suffix in [
        "command/credentials.api.Read/read/as/actor/credentials.api.Alice",
        "denied/credentials.api.Read/as/actor/credentials.api.Bob",
        "read/credentials.api.Records/as/actor/credentials.api.Alice",
        "read/credentials.api.Records/as/actor/credentials.api.Bob",
    ] {
        let expected = format!("credentials.api.Issue/disclosure/issued/secret/{suffix}");
        assert!(ids.contains(&expected), "silently omitted {expected}");
    }
}

#[test]
fn actorless_read_cells_observe_after_origin_and_after_rotation() {
    let source = format!("{MODEL}\nentities:\n  - name: credentials.api.Record\n    identity: {{name: id, type: Uuid}}\n    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}\nviews:\n  - name: credentials.api.Records\n    source: credentials.api.Record\n    consistency: read_your_writes\n    fields: [{{name: id, type: Uuid}}]\n");
    let synthesis = ess_conformance::synthesize(&model(&source));
    let id =
        "credentials.api.Issue/disclosure/issued/secret/read/credentials.api.Records/as/anonymous"
            .parse()
            .unwrap();
    let scenario = synthesis
        .suite
        .scenario(&id)
        .expect("actorless reads have a real query seam");
    assert_eq!(
        scenario
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::QueryView { .. }))
            .count(),
        2
    );
    assert_eq!(
        scenario
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::ExpectOutcome { .. }))
            .count(),
        2
    );
}

#[test]
fn actual_interpreted_target_executes_issuance_rotation_reads_and_event_logs() {
    let source = MODEL.replace("        one_time_response: [secret]", "        one_time_response: [secret]\n        emits: [credentials.api.Issued]\n        payload: {credentials.api.Issued: {audit: public}}")
        + "\nevents:\n  - name: credentials.api.Issued\n    fields: [{name: audit, type: String}]\nentities:\n  - name: credentials.api.Record\n    identity: {name: id, type: Uuid}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nviews:\n  - name: credentials.api.Records\n    source: credentials.api.Record\n    consistency: read_your_writes\n    fields: [{name: id, type: Uuid}]\n";
    let ir = model(&source);
    let synthesis = ess_conformance::synthesize(&ir);
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(ir);
    let report =
        ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert!(report.scenarios.len() >= 5);
    for scenario in &report.scenarios {
        assert_eq!(
            scenario.status,
            ess_conformance::report::Status::Passed,
            "{}: {:?}",
            scenario.scenario,
            scenario.checks
        );
    }
}

#[test]
fn actual_interpreted_target_generates_fresh_constrained_strings() {
    let source = MODEL.replace("type: String", "type: credentials.api.Secret") + "\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    prefix: tok_\n    invariants: [value.count >= 8]\n";
    let ir = model(&source);
    let admitted =
        ess_conformance::AdmittedSuite::from_suite(&ess_conformance::synthesize(&ir).suite)
            .unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(ir);
    let report =
        ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert!(!report.scenarios.is_empty());
    assert!(report
        .scenarios
        .iter()
        .all(|scenario| scenario.status == ess_conformance::report::Status::Passed));
}

fn interpreted_all_pass(source: &str) {
    let ir = model(source);
    let synthesis = ess_conformance::synthesize(&ir);
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(ir);
    let report =
        ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert!(!report.scenarios.is_empty());
    for scenario in &report.scenarios {
        assert_eq!(
            scenario.status,
            ess_conformance::report::Status::Passed,
            "{}: {:?}",
            scenario.scenario,
            scenario.checks
        );
    }
}

#[test]
fn actual_interpreted_two_marked_strings_issue_and_rotate() {
    let source = MODEL
        .replace(
            "      - {name: secret, type: String}",
            "      - {name: secret, type: String}\n      - {name: recovery, type: String}",
        )
        .replace(
            "one_time_response: [secret]",
            "one_time_response: [secret, recovery]",
        );
    interpreted_all_pass(&source);
}

#[test]
fn actual_interpreted_small_finite_string_space_issues_and_rotates() {
    let source = MODEL.replace("      - {name: secret, type: String}", "      - {name: secret, type: credentials.api.Secret}\n      - {name: recovery, type: credentials.api.Secret}")
        .replace("one_time_response: [secret]", "one_time_response: [secret, recovery]")
        + "\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    alphabet: ab\n    invariants: [value.count == 2]\n";
    let ir = model(&source);
    // Isolate actual response execution from the separate named-type invariant inventory.
    let mut suite = ess_conformance::synthesize(&ir).suite;
    let authored = authored::compile(&ir, &[authored::Source::new("finite.yaml", AUTHORED)]);
    assert!(authored.refusals.is_empty(), "{:?}", authored.refusals);
    suite.scenarios = authored.scenarios;
    for scenario in suite.scenarios.values_mut() {
        scenario.steps.extend(scenario.steps.clone());
    }
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(ir);
    let report =
        ess_conformance::Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    assert_eq!(report.scenarios.len(), 1);
    assert_eq!(
        report.scenarios[0].status,
        ess_conformance::report::Status::Passed,
        "{:?}",
        report.scenarios[0].checks
    );
}

#[test]
fn finite_source_generated_disclosure_cells_execute_with_actual_interpreter() {
    let source = MODEL.replace("      - {name: secret, type: String}", "      - {name: secret, type: credentials.api.Secret}\n      - {name: recovery, type: credentials.api.Secret}")
        .replace("one_time_response: [secret]", "one_time_response: [secret, recovery]")
        + "\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    alphabet: ab\n    invariants: [value.count == 2]\n";
    let ir = model(&source);
    let synthesis = ess_conformance::synthesize(&ir);
    // Existing invariant census requires a view position; this model intentionally has none.
    // It must not suppress any source-owned disclosure cell or become an execution Unsupported.
    assert_eq!(synthesis.refusals.len(), 1);
    assert!(synthesis.refusals[0].scenario.is_none());
    assert!(synthesis.refusals[0].cause.to_string().contains("view"));
    let cells = synthesis
        .suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ess_conformance::ScenarioId::Disclosure { .. }))
        .count();
    assert_eq!(cells, 8);
    let input = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(ir);
    let report = ess_conformance::Runner::for_suite(input.suite()).run_admitted(&input, &target);
    assert_eq!(report.scenarios.len(), 9);
    for scenario in &report.scenarios {
        assert_eq!(
            scenario.status,
            ess_conformance::report::Status::Passed,
            "{}: {:?}",
            scenario.scenario,
            scenario.checks
        );
    }
}
