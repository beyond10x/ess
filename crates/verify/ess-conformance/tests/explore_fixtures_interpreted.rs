//! The interpreter passes every scenario synthesized from the explorer fixtures of wave W2's
//! explorer stories, as the fixture sweeps (`adversary_273_pass1.rs`) require of every model under
//! `tests/fixtures`.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const FIXTURES: [(&str, &str); 4] = [
    (
        "explore-held-state-external.yaml",
        include_str!("fixtures/explore-held-state-external.yaml"),
    ),
    (
        "explore-direct-response.yaml",
        include_str!("fixtures/explore-direct-response.yaml"),
    ),
    (
        "explore-input-beside-unknown.yaml",
        include_str!("fixtures/explore-input-beside-unknown.yaml"),
    ),
    (
        "explore-component.yaml",
        include_str!("fixtures/explore-component.yaml"),
    ),
];

#[test]
fn the_interpreter_passes_every_synthesized_scenario_of_each_fixture() {
    for (name, text) in FIXTURES {
        let mut sources = SourceMap::new();
        sources.insert(name, text);
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name}: {error}"));
        let specification = Specification::assemble(vec![(Source::new(name), raw)])
            .unwrap_or_else(|errors| panic!("{name} validates:\n{errors}"));
        let ir = compile(&specification, &sources)
            .unwrap_or_else(|diagnostics| panic!("{name}: {diagnostics}"));
        let synthesis = ess_conformance::synthesize(&ir);
        assert!(
            !synthesis.suite.scenarios.is_empty(),
            "{name}: synthesis writes scenarios"
        );
        let admitted = AdmittedSuite::from_suite(&synthesis.suite)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let report = Runner::new(
            RunnerConfig::default(),
            AdvancingClock::default(),
            Ids::for_suite(&synthesis.suite),
        )
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
        let failed: Vec<String> = report
            .scenarios
            .iter()
            .filter(|scenario| scenario.status != Status::Passed)
            .map(|scenario| format!("{:?} {:?}", scenario.scenario, scenario.status))
            .collect();
        assert!(failed.is_empty(), "{name}: the interpreter: {failed:#?}");
    }
}
