//! Combined finite-primitive and original-byte count admission boundaries.

use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    AdmittedSuite, Clock, ConformanceSuite, Holds, Ids, LeafShape, Runner, RunnerConfig,
    ScenarioStep,
};
use ess_domain::Primitive;
use ess_primitives::time::Timestamp;
use serde_json::{json, Value};
use std::cell::Cell;

const ID: &str = "probe.data/authored/guard";

fn document() -> Value {
    json!({"provenance":{"suite_version":"ess-conformance/4","system":"probe","specification_version":"v1","spec_digest":"a".repeat(64),"contract_digest":"a".repeat(64)},
        "scenarios":{ID:{"purpose":"Refuse unsupported primitive before effects","steps":[
            {"step":"expect_event","event":"probe.data.Created","shape":{}},
            {"step":"eventually_event","event":"probe.data.Created","shape":{}}
        ],"source":[]}}})
}

struct CountingClock<'a>(&'a Cell<usize>);
impl Clock for CountingClock<'_> {
    fn now(&mut self) -> Timestamp {
        self.0.set(self.0.get() + 1);
        Timestamp::from_epoch_millis(0)
    }
}
struct UnreachableTarget(Cell<usize>);
impl ConformanceTarget for UnreachableTarget {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.set(self.0.get() + 1);
        Ok(ImplementationIdentity::new("unreachable", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        panic!("refused suite reached scenario callback")
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        panic!("refused suite reached scenario teardown")
    }
    fn execute_command(
        &self,
        _: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        panic!("refused suite reached command")
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        panic!("refused suite reached view")
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        panic!("refused suite reached event observation")
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("refused suite reached outcome control")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("refused suite reached redelivery")
    }
}

#[test]
fn both_fallible_runners_preserve_all_binary64_issues_before_effects() {
    let mut suite = ConformanceSuite::from_json(&document().to_string()).unwrap();
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            let (ScenarioStep::ExpectEvent { shape, .. }
            | ScenarioStep::EventuallyEvent { shape, .. }) = step
            else {
                panic!("fixture step")
            };
            shape.insert(
                "ratio/a~b",
                LeafShape::required(Holds::Primitive {
                    kind: Primitive::Binary64,
                })
                .optional(),
            );
            shape.insert(
                "z",
                LeafShape::required(Holds::Primitive {
                    kind: Primitive::Binary64,
                }),
            );
        }
    }
    let expected = suite.to_canonical_json().unwrap_err();
    assert_eq!(expected.issues.len(), 4);
    let pointers = expected
        .issues
        .iter()
        .map(|issue| issue.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        pointers,
        [
            "/scenarios/probe.data~1authored~1guard/steps/0/shape/ratio~1a~0b",
            "/scenarios/probe.data~1authored~1guard/steps/0/shape/z",
            "/scenarios/probe.data~1authored~1guard/steps/1/shape/ratio~1a~0b",
            "/scenarios/probe.data~1authored~1guard/steps/1/shape/z"
        ]
    );
    assert!(expected
        .issues
        .iter()
        .all(|issue| issue.reason == "UnsupportedPrimitive" && issue.detail.contains("Binary64")));
    assert_eq!(AdmittedSuite::from_suite(&suite).unwrap_err(), expected);
    assert_eq!(ess_conformance::go::emit(&suite).unwrap_err(), expected);
    assert!(serde_json::to_string(&suite).is_err());
    let clock = Cell::new(0);
    let target = UnreachableTarget(Cell::new(0));
    let runner = || {
        Runner::new(
            RunnerConfig::default(),
            CountingClock(&clock),
            Ids::for_suite(&suite),
        )
    };
    assert_eq!(runner().run(&suite, &target).unwrap_err(), expected);
    assert_eq!(runner().try_run(&suite, &target).unwrap_err(), expected);
    assert_eq!(clock.get(), 0);
    assert_eq!(target.0.get(), 0);
    eprintln!("four located issues retained; run and try_run refused with zero clock/identity/callback effects");
}

#[test]
fn original_byte_and_serde_admission_refuse_binary64_across_all_legacy_suite_majors() {
    for major in 1..=4 {
        for step in ["expect_event", "eventually_event"] {
            for optional in [false, true] {
                let mut value = document();
                value["provenance"]["suite_version"] = json!(format!("ess-conformance/{major}"));
                value["scenarios"][ID]["steps"] = json!([{"step":step,"event":"probe.data.Created","shape":{"ratio/a~b":{"holds":"primitive","kind":"decimal","optional":optional}}}]);
                AdmittedSuite::from_json(&value.to_string()).expect("legacy control admits");
                value["scenarios"][ID]["steps"][0]["shape"]["ratio/a~b"]["kind"] =
                    json!("binary64");
                let raw = value.to_string();
                assert!(AdmittedSuite::from_json(&raw)
                    .unwrap_err()
                    .to_string()
                    .contains("Binary64"));
                assert!(ConformanceSuite::from_json(&raw).is_err());
                assert!(serde_json::from_str::<ConformanceSuite>(&raw).is_err());
            }
        }
    }
    let mut value = document();
    value["provenance"]["suite_version"] = json!("ess-conformance/5");
    let dto = ConformanceSuite::from_json(&value.to_string())
        .expect("historical unadmitted DTO still parses");
    let clock = Cell::new(0);
    let target = UnreachableTarget(Cell::new(0));
    let runner = || {
        Runner::new(
            RunnerConfig::default(),
            CountingClock(&clock),
            Ids::for_suite(&dto),
        )
    };
    assert!(runner().run(&dto, &target).is_err());
    assert!(runner().try_run(&dto, &target).is_err());
    assert_eq!((clock.get(), target.0.get()), (0, 0));
    eprintln!("16 admitted decimal controls and 48 Binary64 reader refusals; both typed runners reject suite/5 before effects");
}

#[test]
fn unified_model_issues_survive_authored_synthesis_and_web_boundaries() {
    use ess_compiler::{resolve::compile, source::SourceMap};
    use ess_domain::{
        spec::{RawSpecFile, Specification},
        system::Source,
    };
    let source = "format: ess/2\nsystem: probe\nversion: v1\ndomains: [probe.data]\ndomain: probe.data\ntypes:\n  - name: probe.data.Values\n    kind: struct\n    fields:\n      - {name: alpha, type: Binary64}\n      - {name: beta, type: 'List<Binary64>'}\n      - {name: gamma, type: 'Optional<Binary64>'}\n";
    let spec = Specification::assemble([(Source::document(), RawSpecFile::parse(source).unwrap())])
        .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let expected = ess_conformance::admission::model(&ir).unwrap_err();
    assert_eq!(expected.issues.len(), 3);
    let paths = expected
        .issues
        .iter()
        .map(|issue| issue.path.clone())
        .collect::<Vec<_>>();
    let authoring = ess_conformance::authored::compile(&ir, &[]);
    assert!(authoring.scenarios.is_empty());
    assert_eq!(authoring.refusals.len(), 1);
    let ess_conformance::authored::Cause::UnsupportedBinary64 { locations } =
        &authoring.refusals[0].cause
    else {
        panic!("wrong refusal")
    };
    assert_eq!(locations, &paths);
    let synthesis = ess_conformance::synthesize(&ir);
    assert!(synthesis.suite.is_empty());
    assert_eq!(synthesis.refusals.len(), 3);
    assert_eq!(
        ess_conformance::web::emit(&ir, &synthesis.suite).unwrap_err(),
        expected
    );
    eprintln!("three model positions retained through unified issues, authored cause, synthesis and prepublication web refusal");
}
