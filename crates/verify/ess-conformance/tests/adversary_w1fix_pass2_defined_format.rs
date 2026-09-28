//! Adversary, w1fix pass 2: `defined_aggregates::used_by` has two arms, `ExpectView` and
//! `EventuallyView`. Every model the unit and pass 1 drive declares its view `read_your_writes`, so
//! only the first arm is exercised: a mutant dropping the `EventuallyView` arm keeps every suite
//! green while an eventual view's #176 invariant is written below suite/26 again.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, synthesize::synthesize, ConformanceSuite, ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn eventually_carries(suite: &ConformanceSuite, needle: &str) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            } if predicate.to_string().contains(needle))
        })
    })
}

fn only_eventually(suite: &ConformanceSuite) -> bool {
    !suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::ExpectView {
                    expectation: ViewExpectation::Satisfies { .. },
                    ..
                }
            )
        })
    })
}

#[test]
fn adversary_w1fix_pass2_an_eventual_view_invariant_over_an_optional_struct_selects_suite_26() {
    let text = QUEUE.replace(
        "    consistency: read_your_writes\n",
        "    consistency: eventual\n",
    );
    assert_ne!(text, QUEUE, "the view is made eventual");
    let ir = ir(&text);
    let synthesis = synthesize(&ir);
    let suite = synthesis.suite;
    assert!(
        eventually_carries(&suite, "defined(metrics)"),
        "the invariant reaches the suite as an eventual view step; refusals: {:#?}",
        synthesis.refusals
    );
    assert!(
        only_eventually(&suite),
        "no read-your-writes step carries it"
    );
    assert!(ess_conformance::defined_aggregates::used_by(&ir, &suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/26"
    );
}
