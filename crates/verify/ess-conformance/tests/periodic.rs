//! Controlled target time exercises actual timer dispatch, host reads and command calls.
use ess_compiler::{resolve::compile_locating, source::SourceMap};
use ess_conformance::{periodic::*, scenario::ScenarioStep, target::*};
use ess_domain::{system::Source, RawSpecFile, Specification};
use ess_primitives::{ids::CorrelationId, node::Node};
use std::{cell::RefCell, collections::BTreeMap};

fn suite() -> ess_conformance::ConformanceSuite {
    let text = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml");
    let spec = Specification::assemble([(
        Source::new("periodic.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("periodic.yaml", text);
    let ir = compile_locating(&spec, &sources, &["periodic.yaml"]).unwrap();
    let mut suite = ess_conformance::synthesize::synthesize(&ir).suite;
    suite.scenarios.retain(|_, scenario| {
        scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::CheckPeriodic { .. }))
    });
    suite
}
fn checks() -> Vec<Check> {
    suite()
        .scenarios
        .values()
        .flat_map(|scenario| scenario.steps.iter())
        .filter_map(|step| match step {
            ScenarioStep::CheckPeriodic { check } => Some(check.clone()),
            _ => None,
        })
        .collect()
}
#[derive(Clone, Copy, Default, Debug)]
enum Fault {
    #[default]
    None,
    Early,
    Late,
    Missing,
    OtherCause,
    WrongScope,
    WrongInput,
    Unsupported,
    PostStop,
    Overlap,
    OvershootMissingTick,
    OvershootComplete,
    ClosureMissingTick,
}
#[derive(Default)]
struct Host {
    check: Option<Check>,
    scope: Option<Scope>,
    now: u64,
    next: u64,
    busy: Option<(u64, u64)>,
    pending: Option<u64>,
    active: bool,
    logs: Vec<Record>,
    reads: usize,
    calls: usize,
    fault: Fault,
    closes: usize,
}
impl Host {
    fn emit(&mut self, fact: Fact) {
        let mut scope = self.scope.clone().unwrap();
        if matches!(self.fault, Fault::WrongScope) {
            scope.lifetime = "old-lifetime".into();
        }
        self.logs.push(Record { scope, fact });
    }
    fn receive(&mut self, ordinal: u64) {
        let check = self.check.as_ref().unwrap();
        let due = check.periodic.every.milliseconds() * ordinal;
        let eligible = !(check.fixture == Fixture::InitiallyInactive && ordinal == 1);
        let slow = check.fixture == Fixture::SlowFirstRead && ordinal == 1;
        let at = match self.fault {
            Fault::Early => due - 1,
            Fault::Late => due + 1,
            _ => self.now,
        };
        self.emit(Fact::Received {
            ordinal,
            due_ms: due,
            at_ms: at,
            eligible,
        });
        if !eligible {
            self.emit(Fact::Completed {
                ordinal,
                at_ms: self.now,
            });
            return;
        }
        self.reads += 1;
        self.busy = Some((ordinal, if slow { due + 5000 } else { self.now }));
    }
    fn finish(&mut self, ordinal: u64) {
        let check = self.check.as_ref().unwrap().clone();
        if check.fixture == Fixture::FirstReadFails && ordinal == 1 {
            self.emit(Fact::ReadFailed {
                ordinal,
                at_ms: self.now,
            });
        } else {
            let read = BTreeMap::from([("status".into(), Node::Text(format!("fresh-{ordinal}")))]);
            let mut input = read.clone();
            input.insert("agent_id".into(), Node::Text("agent-1".into()));
            if matches!(self.fault, Fault::WrongInput) {
                input.insert("status".into(), Node::Text("stale".into()));
            }
            self.calls += 1;
            if matches!(self.fault, Fault::OtherCause) {
                self.emit(Fact::Independent {
                    at_ms: self.now,
                    invocation: format!("push-{ordinal}"),
                    command: check.command,
                });
            } else {
                self.emit(Fact::Invoked {
                    ordinal,
                    at_ms: self.now,
                    invocation: format!("poll-{ordinal}"),
                    command: check.command,
                    read,
                    input,
                });
            }
        }
        self.emit(Fact::Completed {
            ordinal,
            at_ms: self.now,
        });
        self.busy = None;
        if let Some(pending) = self.pending.take() {
            self.receive(pending);
        }
    }
    fn advance(&mut self, through: u64) {
        loop {
            let finish = self.busy.map_or(u64::MAX, |(_, at)| at);
            let tick = if self.active { self.next } else { u64::MAX };
            let next = finish.min(tick);
            if next > through {
                break;
            }
            self.now = next;
            if finish <= tick {
                self.finish(self.busy.unwrap().0);
            } else {
                let p = self.check.as_ref().unwrap().periodic.every.milliseconds();
                let ordinal = tick / p;
                self.next += p;
                if matches!(self.fault, Fault::Missing) {
                    continue;
                }
                if self.busy.is_some() {
                    if matches!(self.fault, Fault::Overlap) {
                        self.receive(ordinal);
                    } else if self.pending.is_none() {
                        self.pending = Some(ordinal);
                    } else {
                        self.emit(Fact::Dropped {
                            first: ordinal,
                            last: ordinal,
                            at_ms: self.now,
                        });
                    }
                } else {
                    self.receive(ordinal);
                }
            }
        }
        self.now = through;
    }
}
struct Controlled(RefCell<Host>);
impl Controlled {
    fn new(fault: Fault) -> Self {
        Self(RefCell::new(Host {
            fault,
            ..Host::default()
        }))
    }
}
fn unsupported() -> TargetError {
    TargetError::unsupported("unused target method", "periodic test")
}
impl ConformanceTarget for Controlled {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("controlled-periodic", "test"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        let fault = self.0.borrow().fault;
        *self.0.borrow_mut() = Host {
            fault,
            ..Host::default()
        };
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        _: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(unsupported())
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(unsupported())
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(unsupported())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(unsupported())
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(unsupported())
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(unsupported())
    }
    fn mark_instant(&self, _: InstantMark) -> Result<(), TargetError> {
        Ok(())
    }
    fn open_periodic(&self, request: Open) -> Result<Opened, TargetError> {
        let mut host = self.0.borrow_mut();
        if matches!(host.fault, Fault::Unsupported) {
            return Err(unsupported());
        }
        // This fixture is bound to this exact authority, not an arbitrary source-named contract.
        if request.check.periodic.host.authority.as_str() != "authenticated-session-status" {
            return Err(unsupported());
        }
        let scope = Scope {
            correlation: request.mark.correlation,
            binding: request.check.binding.clone(),
            lifetime: "session-1".into(),
        };
        host.next = request.check.periodic.every.milliseconds();
        host.check = Some(request.check);
        host.scope = Some(scope.clone());
        host.active = true;
        Ok(Opened {
            scope,
            anchor_ms: 0,
            context: BTreeMap::from([("agent_id".into(), Node::Text("agent-1".into()))]),
        })
    }
    fn observe_periodic(&self, request: Observe) -> Result<Observation, TargetError> {
        let mut host = self.0.borrow_mut();
        let mut through = u64::from(request.elapsed.hold.get()) * 1000;
        host.advance(through);
        let period = host.check.as_ref().unwrap().periodic.every.milliseconds();
        if matches!(host.fault, Fault::OvershootComplete) && through == 4 * period {
            through += period;
            host.advance(through);
        }
        if matches!(host.fault, Fault::OvershootMissingTick) && through == 4 * period {
            // The clock actually advances; the broken timer misses the next due occurrence.
            through += period;
            host.now = through;
            host.next += period;
        }
        if matches!(host.fault, Fault::Late) && !host.logs.is_empty() {
            through += 1;
        }
        Ok(Observation {
            elapsed_ms: through,
            complete_through_ms: through,
            cursor: host.logs.len() as u64,
            records: host.logs[usize::try_from(request.after).expect("bounded test cursor")..]
                .to_vec(),
        })
    }
    fn close_periodic(&self, scope: Scope) -> Result<Closed, TargetError> {
        let mut host = self.0.borrow_mut();
        host.closes += 1;
        if matches!(host.fault, Fault::ClosureMissingTick) {
            host.now += 2 * host.check.as_ref().unwrap().periodic.every.milliseconds();
        }
        if !matches!(host.fault, Fault::PostStop) {
            host.active = false;
        }
        Ok(Closed {
            scope,
            at_ms: host.now,
        })
    }
}

#[test]
fn controlled_periodic_ready_inactive_failure_and_slow_read_are_executable() {
    let checks = checks();
    assert_eq!(checks.len(), 4);
    for check in checks {
        let target = Controlled::new(Fault::None);
        execute(
            &check,
            CorrelationId::new("periodic-test").unwrap(),
            &target,
        )
        .unwrap();
        let host = target.0.borrow();
        assert_eq!(host.closes, 1);
        assert!(!host.active);
        assert!(host.calls >= 2);
        assert!(host.reads >= host.calls);
    }
}

#[test]
fn controlled_periodic_faults_are_nonpassing_and_close_the_host() {
    let ready = checks()
        .into_iter()
        .find(|check| check.fixture == Fixture::Ready)
        .unwrap();
    for fault in [
        Fault::Early,
        Fault::Late,
        Fault::Missing,
        Fault::OtherCause,
        Fault::WrongScope,
        Fault::WrongInput,
        Fault::PostStop,
    ] {
        let target = Controlled::new(fault);
        assert!(
            matches!(
                execute(
                    &ready,
                    CorrelationId::new("periodic-fault").unwrap(),
                    &target
                ),
                Err(Error::Violation(_))
            ),
            "fault passed: {fault:?}"
        );
        assert_eq!(target.0.borrow().closes, 1);
    }
    let slow = checks()
        .into_iter()
        .find(|check| check.fixture == Fixture::SlowFirstRead)
        .unwrap();
    assert!(execute(
        &slow,
        CorrelationId::new("periodic-overlap").unwrap(),
        &Controlled::new(Fault::Overlap)
    )
    .is_err());
}

#[test]
fn elapsed_overshoot_cannot_hide_a_missing_live_tick() {
    let ready = checks()
        .into_iter()
        .find(|check| check.fixture == Fixture::Ready)
        .unwrap();
    execute(
        &ready,
        CorrelationId::new("complete-overshoot").unwrap(),
        &Controlled::new(Fault::OvershootComplete),
    )
    .unwrap();
    for fault in [Fault::OvershootMissingTick, Fault::ClosureMissingTick] {
        let target = Controlled::new(fault);
        assert!(
            matches!(
                execute(
                    &ready,
                    CorrelationId::new("periodic-overshoot").unwrap(),
                    &target
                ),
                Err(Error::Violation(_))
            ),
            "a missing tick in the completely observed live interval must not pass: {fault:?}"
        );
        assert_eq!(target.0.borrow().closes, 1);
    }
}

#[test]
fn periodic_suites_require_new_vocabulary_and_unsupported_is_not_passing() {
    let suite = suite();
    assert_eq!(suite.provenance.suite_version.major(), 6);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let run = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &Controlled::new(Fault::Unsupported));
    assert!(!run.report().is_conformant());
    let report = ess_conformance::CountReport::from_run(&run, &admitted).unwrap();
    assert_eq!(report.counts().unsupported, 4);
    assert_eq!(report.counts().passed, 0);
    assert_ne!(
        report.conformance_status(),
        ess_conformance::CountStatus::Passed
    );
    let encoded = report.to_canonical_json().unwrap();
    assert_eq!(
        ess_conformance::CountReport::from_json(&encoded, &admitted).unwrap(),
        report
    );
    let mut legacy = suite.clone();
    legacy.provenance.suite_version =
        ess_conformance::scenario::SuiteFormat::parse("ess-conformance/4").unwrap();
    assert!(legacy.to_canonical_json().is_err());
}

fn run_go(case: &str, code: &str) {
    let directory =
        std::env::temp_dir().join(format!("ess-periodic-{case}-{}", std::process::id()));
    for artifact in ess_conformance::go::emit(&suite()).unwrap() {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/periodic\n\ngo 1.25\n",
    )
    .unwrap();
    std::fs::write(directory.join("essconform/periodic_test.go"), code).unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestPeriodic",
            "-count=1",
            "-v",
        ])
        .current_dir(&directory)
        .output()
        .unwrap();
    let log = format!(
        "exit: {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(directory.join("go.log"), &log).unwrap();
    assert!(output.status.success(), "{}\n{log}", directory.display());
    if case == "controlled" {
        let strict = std::process::Command::new("go")
            .args([
                "test",
                "./essconform",
                "-run",
                "^TestPeriodicCountReports/report-unsupported$",
                "-count=1",
                "-v",
            ])
            .env("ESS_CONFORMANCE_STRICT", "1")
            .current_dir(&directory)
            .output()
            .unwrap();
        let strict_log = format!(
            "exit: {:?}\n{}\n{}",
            strict.status.code(),
            String::from_utf8_lossy(&strict.stdout),
            String::from_utf8_lossy(&strict.stderr)
        );
        std::fs::write(directory.join("strict.log"), &strict_log).unwrap();
        assert!(
            !strict.status.success() && strict_log.contains("strict conformance"),
            "{strict_log}"
        );
        let admitted = ess_conformance::AdmittedSuite::from_json(
            &std::fs::read_to_string(directory.join("essconform/suite.json")).unwrap(),
        )
        .unwrap();
        for (file, passed, skipped) in [
            ("report-ready.json", 4, 0),
            ("report-unsupported.json", 0, 4),
        ] {
            let report = ess_conformance::CountReport::from_json(
                &std::fs::read_to_string(directory.join("essconform").join(file)).unwrap(),
                &admitted,
            )
            .unwrap();
            assert_eq!(report.counts().passed, passed);
            assert_eq!(report.counts().skipped, skipped);
            assert_ne!(
                report.conformance_status(),
                ess_conformance::CountStatus::Passed,
                "ordinary suites retain unknown coverage"
            );
        }
    }
}
#[test]
fn generated_go_admits_periodic_vocabulary() {
    run_go(
        "admission",
        r#"package essconform
import ("testing"; "strings")
func TestPeriodicAdmission(t *testing.T) {
    suite, err := admitRunInput(suiteJSON)
    if err != nil { t.Fatal(err) }
    if len(suite.Scenarios) != 4 { t.Fatalf("want four real periodic fixtures; got %d",len(suite.Scenarios)) }
    for _,bad:=range []string{"PT0S","PT02S","PT+2S","PT4294967296S"} { if _,err:=admitRunInput(strings.ReplaceAll(suiteJSON,"PT2S",bad));err==nil{t.Fatalf("admitted %s",bad)} }
    if _,err:=admitRunInput(strings.ReplaceAll(suiteJSON,"ess-conformance/6","ess-conformance/4"));err==nil{t.Fatal("legacy suite admitted periodic vocabulary")}
}
"#,
    );
}

#[test]
fn generated_go_runs_controlled_periodic_hosts_and_rejects_faults() {
    run_go(
        "controlled",
        include_str!("fixtures/periodic/controlled.go"),
    );
}

#[test]
fn unrepresentable_period_window_is_a_refusal_with_a_serializable_remaining_suite() {
    let text = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml")
        .replace("PT2S", "PT4294967295S");
    let spec = Specification::assemble([(
        Source::new("periodic.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("periodic.yaml", &text);
    let ir = compile_locating(&spec, &sources, &["periodic.yaml"]).unwrap();
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        synthesis
            .refusals
            .iter()
            .any(|refusal| format!("{:?}", refusal.cause).contains("PeriodicResource")),
        "bounded window must be refused at synthesis"
    );
    assert!(!used_by(&synthesis.suite));
    synthesis
        .suite
        .to_canonical_json()
        .expect("remaining obligations remain serializable");
}
