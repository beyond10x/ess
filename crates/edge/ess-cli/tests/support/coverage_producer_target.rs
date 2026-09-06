use super::{check_report, text, write_json};
use ess_conformance::{
    coverage::AdmittedInput,
    reference::Billing,
    target::{
        ConformanceTarget, ElapsedObservation, ElapsedObservationRequest, EventObservationRequest,
        ExternalOutcomeControl, ImplementationIdentity, InstantMark, InvocationObservationRequest,
        ObservedEvent, ObservedInvocation, OrderedScan, OrderedScanRequest, RedeliveryRequest,
        ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
        SemanticViewResult, TargetError,
    },
    Clock, CountReport, CountRun, CountStatus, Ids, Runner, RunnerConfig,
};
use ess_primitives::time::Timestamp;
use serde_json::{json, Value};
use std::{cell::RefCell, fs, path::Path, rc::Rc};

struct LoggedClock {
    fixed: Option<u64>,
    next: u64,
    log: Rc<RefCell<Vec<u64>>>,
}
impl Clock for LoggedClock {
    fn now(&mut self) -> Timestamp {
        let value = self.fixed.unwrap_or(self.next);
        self.log.borrow_mut().push(value);
        if self.fixed.is_none() {
            self.next = self.next.checked_add(1).unwrap();
        }
        Timestamp::from_epoch_millis(value)
    }
}
struct LoggedTarget {
    billing: Billing,
    control: bool,
    mode: String,
    callbacks: RefCell<Vec<Value>>,
}
impl LoggedTarget {
    fn record(&self, method: &str, request: &Value) {
        self.callbacks
            .borrow_mut()
            .push(json!({"method":method,"request":request}));
    }
}
macro_rules! forward {
    ($method:ident, $request:ty, $result:ty) => {
        fn $method(&self, request: $request) -> Result<$result, TargetError> {
            self.record(stringify!($method), &json!(request));
            self.billing.$method(request)
        }
    };
}
impl ConformanceTarget for LoggedTarget {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.record("identity", &Value::Null);
        if self.control {
            Ok(ImplementationIdentity::new(
                "coverage-producer-control",
                "1",
            ))
        } else {
            self.billing.identity()
        }
    }
    fn begin_scenario(&self, request: &ScenarioContext) -> Result<(), TargetError> {
        self.record("begin", &json!(request));
        match self.mode.as_str() {
            "error" => Err(TargetError::unavailable(
                "begin",
                "independent fixture failure",
            )),
            "unsupported" => Err(TargetError::unsupported(
                "begin",
                "independent fixture capability",
            )),
            _ => self.billing.begin_scenario(request),
        }
    }
    fn end_scenario(&self, request: &ScenarioContext) -> Result<(), TargetError> {
        self.record("end", &json!(request));
        self.billing.end_scenario(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.record("execute_command", &json!(request));
        if self.control {
            if self.mode == "failed" {
                Ok(SemanticCommandResult::undeclared())
            } else {
                Ok(SemanticCommandResult::took(
                    serde_json::from_value(json!({
                        "command":"billing.invoice.CreateInvoice","outcome":"accepted"
                    }))
                    .unwrap(),
                ))
            }
        } else {
            self.billing.execute_command(request)
        }
    }
    forward!(query_view, SemanticViewRequest, SemanticViewResult);
    forward!(observe_events, EventObservationRequest, Vec<ObservedEvent>);
    forward!(configure_external_outcome, ExternalOutcomeControl, ());
    forward!(redeliver_event, RedeliveryRequest, ());
    forward!(
        observe_invocations,
        InvocationObservationRequest,
        Vec<ObservedInvocation>
    );
    forward!(mark_instant, InstantMark, ());
    forward!(
        observe_elapsed,
        ElapsedObservationRequest,
        ElapsedObservation
    );
    forward!(scan_view, OrderedScanRequest, OrderedScan);
}
pub(super) fn run(instance: &Value, input: &AdmittedInput, out: &Path) {
    let transcript = Rc::new(RefCell::new(Vec::new()));
    let fixed = instance["clock"].as_u64();
    if fixed.is_none() {
        assert_eq!(
            instance["clock"],
            json!({"kind":"logged_advancing","start":0,"step":1})
        );
    }
    let clock = LoggedClock {
        fixed,
        next: 0,
        log: transcript.clone(),
    };
    let target = LoggedTarget {
        billing: Billing::new(),
        control: instance["target"] == "single_scenario_controls",
        mode: text(&instance["target_mode"]).into(),
        callbacks: RefCell::new(Vec::new()),
    };
    let run = Runner::new(
        RunnerConfig::default(),
        clock,
        Ids::for_suite(input.selected().suite()),
    )
    .run_admitted(input.selected(), &target);
    // Freeze the independent clock/callback transcript before producing or reading either report.
    write_json(
        &out.join("clock.json"),
        &json!({"configuration":instance["clock"],"reads":*transcript.borrow()}),
    );
    write_json(
        &out.join("callbacks.json"),
        &json!(*target.callbacks.borrow()),
    );
    let completed = *transcript.borrow().last().unwrap();
    for method in ["identity", "begin", "end", "execute_command"] {
        let actual = target
            .callbacks
            .borrow()
            .iter()
            .filter(|call| call["method"] == method)
            .count();
        assert_eq!(
            json!(actual),
            instance["callbacks"][method],
            "{} / {method}",
            instance["id"]
        );
    }
    let producer_exit = match run.status {
        ess_conformance::ConformanceStatus::Passed => 0,
        ess_conformance::ConformanceStatus::Failed => 1,
        ess_conformance::ConformanceStatus::Error => 3,
    };
    assert_eq!(
        json!(producer_exit),
        instance["expected_diagnostic_producer_exit"]
    );
    let report_json = CountReport::from_run(&run, input.selected())
        .unwrap()
        .to_canonical_json()
        .unwrap();
    fs::write(out.join("report.json"), &report_json).unwrap();
    let detailed = CountRun::from_run(&run, input.selected())
        .unwrap()
        .to_canonical_json()
        .unwrap();
    fs::write(out.join("run.json"), &detailed).unwrap();
    let summary = check_report(instance, &report_json, input, completed);
    CountRun::from_json(&detailed, input.selected()).unwrap();
    let strict_exit = match summary.conformance_status() {
        CountStatus::Passed => 0,
        CountStatus::Failed => 1,
        CountStatus::Inconclusive => 3,
    };
    write_json(
        &out.join("fixture.json"),
        &json!({
            "id":instance["id"],"structure":instance["structure"],"producer_profile":instance["producer_profile"],
            "command":"Runner::new(...).run_admitted + CountReport::from_run + CountRun::from_run",
            "runtime":concat!("ess-conformance/",env!("CARGO_PKG_VERSION")),
            "report":"report.json","suite":"suite.json","input":"input.json","detailed":"run.json",
            "transport":"transport.json","callbacks":"callbacks.json","clock_transcript":"clock.json",
            "independent_expected":instance,"independent_completed_at":completed,
            "diagnostic_producer_exit":producer_exit,"strict_producer_exit":strict_exit,
            "source_manifest":"../source-manifest.json","producer_receipt":"../producer.json"
        }),
    );
    println!(
        "{}: report2, diagnostic {producer_exit}, strict {strict_exit}",
        instance["id"]
    );
}
