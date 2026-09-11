//! What an authored scenario compiles to, and every way it is refused.
//!
//! The point of the feature is the refusals. A scenario a person writes against a real model names
//! a few dozen constructs, and the model moves underneath it — so what an authoring surface is
//! *for* is that a name the specification no longer declares is refused when the file is compiled,
//! by name, rather than failing at run time in each consumer that happens to execute it.
//!
//! So there is a case per cause below, and `every_cause_is_reachable_from_a_document` asserts that
//! the set of codes these cases produce is the whole set the module can emit: a cause added without
//! a document that reaches it fails here rather than shipping unexercised.
//!
//! The documents are compiled against `examples/billing/` — the normative example — except where a
//! construct it does not carry is needed, which is once: `gatepass.visit.RegisterVisit` is the
//! command in this repository whose input holds an enum, and an enum is the only place a *variant*
//! can be misspelt.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::diagnostic::Code;
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Cause, Refusal, Source};
use ess_conformance::{
    ConformanceSuite, ScenarioId, ScenarioStep, SuiteProvenance, ViewExpectation,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

// ---- the models under test ---------------------------------------------------------------------

const CALL_HISTORY_SETUP_MODEL: &str = r"
format: ess/1
system: calls
version: v1
domain: calls.history
entities:
  - name: calls.history.CallRecord
    identity: {name: call_id, type: Uuid}
    fields:
      - {name: started_at, type: Timestamp}
      - {name: duration_seconds, type: Integer}
      - {name: note, type: 'Optional<String>'}
    lifecycle:
      initial: Completed
      states: [Completed]
      terminal: [Completed]
    invariants:
      - duration_seconds >= 0
views:
  - name: calls.history.CallHistory
    source: calls.history.CallRecord
    consistency: read_your_writes
    order_by: [started_at desc]
    fields:
      - {name: call_id, type: Uuid}
      - {name: started_at, type: Timestamp}
      - {name: duration_seconds, type: Integer}
";

const CALL_HISTORY_SETUP: &str = r"
type: ess-scenario/2
domain: calls.history
scenario: backend-history
summary: Two backend records retain their values and descending-time order.
arrange:
  - instance: earlier
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {started_at: '2026-01-05T09:00:00Z', duration_seconds: 12}
      state: Completed
  - instance: later
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000002
      fields: {started_at: '2026-01-05T09:01:00Z', duration_seconds: 24, note: null}
      state: Completed
assert:
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: earlier}, duration_seconds: 12}
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: later}, duration_seconds: 24}
  - view: calls.history.CallHistory
    counts: {at_least: 2, at_most: 2}
  - view: calls.history.CallHistory
    at: {row: first, fields: {call_id: {$instance: later}}}
";

#[test]
fn entity_setup_compiles_real_rows_without_a_creator_or_dummy_command() {
    let ir = fixture(CALL_HISTORY_SETUP_MODEL);
    assert!(ir.commands().is_empty());
    let result = authoring(&ir, CALL_HISTORY_SETUP);
    assert!(result.is_complete(), "{:?}", result.refusals);
    let steps = &result.scenarios.values().next().unwrap().steps;
    let json = serde_json::to_value(steps).unwrap();
    assert_eq!(json[0]["step"], "establish_entity");
    assert_eq!(json[1]["step"], "establish_entity");
    assert!(json[0]["fields"].get("note").is_none());
    assert_eq!(json[1]["fields"]["note"], serde_json::Value::Null);
    assert!(!steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::ExecuteCommand { .. })));
}

#[test]
fn entity_setup_requires_valid_complete_nonduplicate_declared_state() {
    let ir = fixture(CALL_HISTORY_SETUP_MODEL);
    assert!(authoring(&ir, CALL_HISTORY_SETUP).is_complete());
    for invalid in [
        CALL_HISTORY_SETUP.replace("duration_seconds: 12", "duration_seconds: -1"),
        CALL_HISTORY_SETUP.replace("duration_seconds: 12", "duration_seconds: wrong"),
        CALL_HISTORY_SETUP.replace(", duration_seconds: 12", ""),
        CALL_HISTORY_SETUP.replace("duration_seconds: 12", "duration_seconds: 12, unknown: 3"),
        CALL_HISTORY_SETUP.replace("state: Completed", "state: Missing"),
        CALL_HISTORY_SETUP.replace("instance: later", "instance: earlier"),
        CALL_HISTORY_SETUP.replace("000000000002", "000000000001"),
        CALL_HISTORY_SETUP.replace("type: ess-scenario/2", "type: ess-scenario/1"),
        CALL_HISTORY_SETUP
            .split("assert:")
            .next()
            .unwrap()
            .to_owned(),
    ] {
        assert!(
            !authoring(&ir, &invalid).is_complete(),
            "accepted invalid setup: {invalid}"
        );
    }
}

#[test]
fn entity_setup_invariant_unknown_refuses_and_identity_namespace_is_qualified() {
    let unknown_model =
        CALL_HISTORY_SETUP_MODEL.replace("duration_seconds >= 0", "note == recorded");
    let unknown = authoring(&fixture(&unknown_model), CALL_HISTORY_SETUP);
    assert!(!unknown.is_complete());
    assert!(unknown
        .refusals
        .iter()
        .any(|refusal| refusal.to_string().contains("Unknown")));
    let two_entities = CALL_HISTORY_SETUP_MODEL.replace(
        "views:",
        r"  - name: calls.history.OtherRecord
    identity: {name: call_id, type: Uuid}
    fields:
      - {name: started_at, type: Timestamp}
      - {name: duration_seconds, type: Integer}
      - {name: note, type: 'Optional<String>'}
    lifecycle: {initial: Completed, states: [Completed], terminal: [Completed]}
views:",
    );
    let source = CALL_HISTORY_SETUP
        .replace(
            "instance: later\n    entity: calls.history.CallRecord",
            "instance: later\n    entity: calls.history.OtherRecord",
        )
        .replace("000000000002", "000000000001");
    let result = authoring(&fixture(&two_entities), &source);
    assert!(result.is_complete(), "{:?}", result.refusals);
}

#[test]
fn entity_setup_checks_collection_members_and_value_object_invariants() {
    for (kind, valid, invalid) in [
        ("List<Integer>", "[1]", "[wrong]"),
        ("Map<String, Integer>", "{key: 1}", "{key: wrong}"),
    ] {
        let model = CALL_HISTORY_SETUP_MODEL.replace(
            "    lifecycle:",
            &format!("      - {{name: extra, type: '{kind}'}}\n    lifecycle:"),
        );
        let ir = fixture(&model);
        let source = CALL_HISTORY_SETUP.replace(
            "fields: {started_at:",
            &format!("fields: {{extra: {valid}, started_at:"),
        );
        assert!(authoring(&ir, &source).is_complete());
        assert!(
            !authoring(&ir, &source.replace(valid, invalid)).is_complete(),
            "accepted invalid member of {kind}"
        );
    }
    let model = CALL_HISTORY_SETUP_MODEL.replace("entities:", "types:\n  - name: calls.history.Duration\n    kind: newtype\n    of: Integer\n    invariants: ['value >= 0']\nentities:")
        .replace("duration_seconds, type: Integer", "duration_seconds, type: calls.history.Duration")
        .replace("duration_seconds >= 0", "duration_seconds >= -100");
    let ir = fixture(&model);
    assert!(authoring(&ir, CALL_HISTORY_SETUP).is_complete());
    assert!(!authoring(
        &ir,
        &CALL_HISTORY_SETUP.replace("duration_seconds: 12", "duration_seconds: -1")
    )
    .is_complete());
}

#[test]
fn entity_setup_checks_union_tags_payloads_and_nested_struct_invariants() {
    for tag in ["kind", "value"] {
        let content = if tag == "value" { "content" } else { "value" };
        let model = CALL_HISTORY_SETUP_MODEL.replace("entities:", &format!("types:\n  - name: calls.history.Extra\n    kind: union\n    tag: {tag}\n    variants: {{number: calls.history.Positive}}\n  - name: calls.history.Positive\n    kind: struct\n    fields: [{{name: amount, type: Integer}}]\n    invariants: ['amount >= 0']\nentities:"))
            .replace("    lifecycle:", "      - {name: extra, type: calls.history.Extra}\n    lifecycle:");
        let ir = fixture(&model);
        let payload = format!("{{{tag}: number, {content}: {{amount: 1}}}}");
        let source = CALL_HISTORY_SETUP.replace(
            "fields: {started_at:",
            &format!("fields: {{extra: {payload}, started_at:"),
        );
        let valid = authoring(&ir, &source);
        assert!(valid.is_complete(), "{:?}", valid.refusals);
        for invalid in [
            source.replace("amount: 1", "amount: -1"),
            source.replace("amount: 1", "other: 1"),
            source.replace(&format!("{tag}: number"), &format!("{tag}: missing")),
            source.replace("amount: 1", "amount: wrong"),
        ] {
            assert!(!authoring(&ir, &invalid).is_complete());
        }
    }
}

#[test]
fn adversary_entity_setup_null_identity_is_refused_at_source_validation() {
    let model = CALL_HISTORY_SETUP_MODEL.replace("type: Uuid", "type: 'Optional<Uuid>'");
    let ir = fixture(&model);
    assert!(authoring(&ir, CALL_HISTORY_SETUP).is_complete());
    let source = CALL_HISTORY_SETUP.replace(
        "identity: 00000000-0000-4000-8000-000000000001",
        "identity: null",
    );
    let result = authoring(&ir, &source);
    if result.is_complete() {
        let mut suite = ess_conformance::synthesize::synthesize(&ir).suite;
        suite.provenance.suite_version =
            ess_conformance::scenario::SuiteFormat::parse("ess-conformance/6").unwrap();
        suite.scenarios = result.scenarios.clone();
        let error = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap_err();
        assert!(
            error.to_string().contains("identity cannot be null"),
            "{error}"
        );
    }
    assert!(
        !result.is_complete(),
        "source compiler accepted null identity even though its produced suite cannot be admitted"
    );
}

mod entity_setup_execution {
    use super::*;
    use ess_conformance::runner::Runner;
    use ess_conformance::target::*;
    use ess_primitives::node::Node;
    use std::cell::RefCell;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Behavior {
        Good,
        Empty,
        Wrong,
        Reversed,
        Unsupported,
        Borrowed,
    }

    struct Backend<'a> {
        ir: &'a EssIr,
        behavior: Behavior,
        rows: RefCell<Vec<ViewRow>>,
    }

    impl ConformanceTarget for Backend<'_> {
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            Ok(ImplementationIdentity::new("call-history-fixture", "1"))
        }
        fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
            if self.behavior != Behavior::Borrowed {
                self.rows.borrow_mut().clear();
            }
            Ok(())
        }
        fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
            if self.behavior != Behavior::Borrowed {
                self.rows.borrow_mut().clear();
            }
            Ok(())
        }
        fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
            if self.behavior == Behavior::Unsupported {
                return Err(TargetError::unsupported(
                    "entity setup",
                    "backend cannot establish rows",
                ));
            }
            ess_conformance::input::validate_entity_setup(
                self.ir,
                &request.entity,
                &request.identity,
                &request.fields,
                &request.state,
            )
            .map_err(|detail| TargetError::unavailable("invalid entity setup", detail))?;
            if self.behavior == Behavior::Empty {
                return Ok(());
            }
            let mut row = request.fields;
            row.insert("call_id".into(), request.identity);
            if self.behavior == Behavior::Wrong {
                row.insert("duration_seconds".into(), Node::Number(0_u32.into()));
            }
            self.rows.borrow_mut().push(row);
            Ok(())
        }
        fn query_view(
            &self,
            request: SemanticViewRequest,
        ) -> Result<SemanticViewResult, TargetError> {
            assert_eq!(request.view.to_string(), "calls.history.CallHistory");
            let mut rows = self.rows.borrow().clone();
            rows.sort_by_key(|row| match &row["started_at"] {
                Node::Text(text) => text.clone(),
                _ => panic!("validated timestamp"),
            });
            if self.behavior != Behavior::Reversed {
                rows.reverse();
            }
            Ok(SemanticViewResult::of(rows))
        }
        fn execute_command(
            &self,
            _: SemanticCommandRequest,
        ) -> Result<SemanticCommandResult, TargetError> {
            panic!("CallRecord has no creator command")
        }
        fn observe_events(
            &self,
            _: EventObservationRequest,
        ) -> Result<Vec<ObservedEvent>, TargetError> {
            panic!("setup must not manufacture events")
        }
        fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
            panic!("setup is not an external outcome")
        }
        fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
            panic!("setup has no event history")
        }
    }

    fn suite(ir: &EssIr) -> ConformanceSuite {
        let mut result = ess_conformance::synthesize::synthesize(ir).suite;
        result.provenance.suite_version =
            ess_conformance::scenario::SuiteFormat::parse("ess-conformance/6").unwrap();
        let first = authoring(ir, CALL_HISTORY_SETUP);
        assert!(first.is_complete(), "{:?}", first.refusals);
        result.scenarios = first.scenarios;
        let second = CALL_HISTORY_SETUP
            .replace("backend-history", "second-history")
            .replace("000000000001", "000000000003")
            .replace("000000000002", "000000000004");
        let second = authoring(ir, &second);
        assert!(second.is_complete(), "{:?}", second.refusals);
        result.scenarios.extend(second.scenarios);
        result
    }

    #[test]
    fn entity_setup_target_storage_decides_values_membership_order_and_isolation() {
        let ir = fixture(CALL_HISTORY_SETUP_MODEL);
        let suite = suite(&ir);
        for behavior in [
            Behavior::Good,
            Behavior::Empty,
            Behavior::Wrong,
            Behavior::Reversed,
            Behavior::Unsupported,
            Behavior::Borrowed,
        ] {
            let target = Backend {
                ir: &ir,
                behavior,
                rows: RefCell::default(),
            };
            let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
            let report = Runner::for_suite(&suite)
                .run_admitted(&admitted, &target)
                .into_report();
            assert_eq!(
                report.status == ess_conformance::report::ConformanceStatus::Passed,
                behavior == Behavior::Good,
                "{behavior:?}: {report:?}"
            );
            if behavior != Behavior::Borrowed {
                assert!(target.rows.borrow().is_empty());
            }
        }
        let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
        let run = Runner::for_suite(&suite)
            .run_admitted(&admitted, &ess_conformance::reference::Billing::new());
        let report = ess_conformance::CountReport::from_run(&run, &admitted).unwrap();
        assert_eq!(report.counts().unsupported, 2);
        assert_eq!(report.counts().passed, 0);
    }

    #[test]
    fn entity_setup_manual_suite_refuses_duplicates_and_setup_only_before_target() {
        let ir = fixture(CALL_HISTORY_SETUP_MODEL);
        let suite = suite(&ir);
        let mut duplicate = suite.clone();
        let steps = &mut duplicate.scenarios.values_mut().next().unwrap().steps;
        steps.insert(1, steps[0].clone());
        assert!(ess_conformance::AdmittedSuite::from_suite(&duplicate).is_err());
        let mut duplicate_identity = suite.clone();
        let steps = &mut duplicate_identity
            .scenarios
            .values_mut()
            .next()
            .unwrap()
            .steps;
        let identity = match &steps[0] {
            ScenarioStep::EstablishEntity { identity, .. } => identity.clone(),
            _ => unreachable!(),
        };
        if let ScenarioStep::EstablishEntity {
            identity: second, ..
        } = &mut steps[1]
        {
            *second = identity;
        }
        assert!(ess_conformance::AdmittedSuite::from_suite(&duplicate_identity).is_err());
        let mut no_assertion = suite.clone();
        no_assertion
            .scenarios
            .values_mut()
            .next()
            .unwrap()
            .steps
            .truncate(2);
        assert!(ess_conformance::AdmittedSuite::from_suite(&no_assertion).is_err());
        let mut old = suite;
        old.provenance.suite_version =
            ess_conformance::scenario::SuiteFormat::parse("ess-conformance/4").unwrap();
        assert!(ess_conformance::AdmittedSuite::from_suite(&old).is_err());
        assert!(old.to_canonical_json().is_err());
        assert!(ess_conformance::go::emit(&old).is_err());
    }

    #[test]
    fn entity_setup_direct_writers_refuse_unbounded_literals_and_assertions_before_setup() {
        let ir = fixture(CALL_HISTORY_SETUP_MODEL);
        let mut nested = suite(&ir);
        let mut value = Node::Null;
        for _ in 0..121 {
            value = Node::Seq(vec![value]);
        }
        if let ScenarioStep::EstablishEntity { fields, .. } =
            &mut nested.scenarios.values_mut().next().unwrap().steps[0]
        {
            fields.insert("note".into(), value);
        }
        assert!(nested.to_canonical_json().is_err());
        assert!(ess_conformance::go::emit(&nested).is_err());
        let mut late = suite(&ir);
        late.scenarios
            .values_mut()
            .next()
            .unwrap()
            .steps
            .rotate_left(2);
        assert!(late.to_canonical_json().is_err());
    }

    #[test]
    fn entity_setup_generated_go_reads_established_backend_rows_and_rejects_faults() {
        let ir = fixture(CALL_HISTORY_SETUP_MODEL);
        let suite = suite(&ir);
        let directory =
            std::env::temp_dir().join(format!("ess-arrangement-go-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        for artifact in ess_conformance::go::emit(&suite).unwrap() {
            let filename = Path::new(&artifact.path).file_name().unwrap();
            std::fs::write(directory.join(filename), artifact.contents).unwrap();
        }
        std::fs::write(
            directory.join("go.mod"),
            "module example.test/entitysetup\n\ngo 1.23\n",
        )
        .unwrap();
        std::fs::write(directory.join("arrangement_test.go"), GO_BACKEND).unwrap();
        eprintln!("entity setup Go fixture: {}", directory.display());
        for behavior in [
            "good",
            "empty",
            "wrong",
            "reversed",
            "unsupported",
            "absent-capability",
            "borrowed",
        ] {
            let output = std::process::Command::new("go")
                .args(["test", "-count=1", "-v", "."])
                .current_dir(&directory)
                .env("GOTOOLCHAIN", "local")
                .env("GOPROXY", "off")
                .env_remove("ESS_CONFORMANCE_STRICT")
                .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
                .env("ESS_REPORT_FORMAT", "2")
                .env(
                    "ESS_REPORT_OUT",
                    directory.join(format!("{behavior}-report.json")),
                )
                .env("ESS_SETUP_FAULT", behavior)
                .output()
                .expect("Go is required for native conformance parity");
            eprintln!(
                "Go {behavior}, exit {:?}:\n{}\n{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.status.success(),
                matches!(behavior, "good" | "unsupported" | "absent-capability"),
                "{behavior}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(directory.join(format!("{behavior}-report.json")))
                    .unwrap(),
            )
            .unwrap();
            let expected = match behavior {
                "good" => "passed",
                "unsupported" | "absent-capability" => "inconclusive",
                _ => "failed",
            };
            assert_eq!(report["execution_status"], expected, "{behavior}: {report}");
            let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
            let checked =
                ess_conformance::CountReport::from_json(&report.to_string(), &admitted).unwrap();
            let checked_status = match behavior {
                "good" => ess_conformance::CountStatus::Passed,
                "unsupported" | "absent-capability" => ess_conformance::CountStatus::Inconclusive,
                _ => ess_conformance::CountStatus::Failed,
            };
            assert_eq!(checked.execution_status(), checked_status);
            if matches!(behavior, "unsupported" | "absent-capability") {
                assert_eq!(report["counts"]["skipped"], 2);
                assert_eq!(report["counts"]["passed"], 0);
                assert_ne!(report["conformance_status"], "passed");
            }
            assert!(
                String::from_utf8_lossy(&output.stdout).contains("TestEntitySetup"),
                "Go must execute the deciding test"
            );
        }
    }

    fn stale_setup_observation_suite(ir: &EssIr, fresh: bool) -> ConformanceSuite {
        let mut suite = suite(ir);
        let id = suite.scenarios.keys().next().unwrap().clone();
        suite.scenarios.retain(|key, _| key == &id);
        let scenario = suite.scenarios.values_mut().next().unwrap();
        let setup = scenario.steps[0].clone();
        let query = scenario
            .steps
            .iter()
            .find(|step| matches!(step, ScenarioStep::QueryView { .. }))
            .unwrap()
            .clone();
        let view = match &query {
            ScenarioStep::QueryView { view, .. } => view.clone(),
            _ => unreachable!(),
        };
        let count = usize::from(fresh);
        let assertion = ScenarioStep::ExpectView {
            view,
            expectation: ViewExpectation::Counts {
                at_least: Some(count),
                at_most: Some(count),
            },
        };
        scenario.steps = if fresh {
            vec![setup, query, assertion]
        } else {
            vec![query, setup, assertion]
        };
        suite
    }

    #[test]
    fn adversary_entity_setup_cannot_pass_by_asserting_a_pre_setup_snapshot() {
        let ir = fixture(CALL_HISTORY_SETUP_MODEL);
        let target = Backend {
            ir: &ir,
            behavior: Behavior::Good,
            rows: RefCell::default(),
        };
        // The adapter establishes real queried storage; a fresh query observes exactly one row.
        let fresh = stale_setup_observation_suite(&ir, true);
        let admitted = ess_conformance::AdmittedSuite::from_suite(&fresh).unwrap();
        let report = Runner::for_suite(&fresh).run_admitted(&admitted, &target);
        assert_eq!(
            report.status,
            ess_conformance::report::ConformanceStatus::Passed
        );

        // An assertion after setup must not discharge the setup obligation using an older query.
        let stale = stale_setup_observation_suite(&ir, false);
        if let Ok(admitted) = ess_conformance::AdmittedSuite::from_suite(&stale) {
            let report = Runner::for_suite(&stale).run_admitted(&admitted, &target);
            assert_ne!(report.status, ess_conformance::report::ConformanceStatus::Passed,
                "zero-row assertion passed from a snapshot taken before the acknowledged setup: {report:?}");
        }
    }

    #[test]
    fn adversary_entity_setup_go_cannot_pass_by_asserting_a_pre_setup_snapshot() {
        let ir = fixture(CALL_HISTORY_SETUP_MODEL);
        let suite = stale_setup_observation_suite(&ir, false);
        let Ok(artifacts) = ess_conformance::go::emit(&suite) else {
            return;
        };
        let directory = std::env::temp_dir().join(format!(
            "ess-arrangement-adversary-stale-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        for artifact in artifacts {
            let filename = Path::new(&artifact.path).file_name().unwrap();
            std::fs::write(directory.join(filename), artifact.contents).unwrap();
        }
        std::fs::write(
            directory.join("go.mod"),
            "module example.test/entitysetup\n\ngo 1.23\n",
        )
        .unwrap();
        std::fs::write(directory.join("arrangement_test.go"), GO_BACKEND).unwrap();
        let output = std::process::Command::new("go")
            .args(["test", "-count=1", "-v", "-run", "^TestEntitySetup$", "."])
            .current_dir(&directory)
            .env("GOTOOLCHAIN", "local")
            .env("GOPROXY", "off")
            .env_remove("ESS_CONFORMANCE_STRICT")
            .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", directory.join("report.json"))
            .env("ESS_SETUP_FAULT", "good")
            .output()
            .expect("required Go toolchain");
        let log = format!(
            "exit: {:?}\n{}\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::write(directory.join("go.log"), &log).unwrap();
        assert!(
            !output.status.success(),
            "Go accepted a stale pre-setup snapshot as the final setup assertion:\n{log}"
        );
    }

    const GO_BACKEND: &str = r#"package essconform
import ("encoding/json"; "fmt"; "os"; "sort"; "testing")
type callBackend struct { rows []map[string]Node; mode string }
var borrowed []map[string]Node
func (b *callBackend) Identity() (Identity,error) { return Identity{Name:"call-history-fixture",Version:"1"},nil }
func (b *callBackend) BeginScenario(ScenarioContext) error { b.rows=nil; if b.mode=="borrowed" { b.rows=borrowed }; return nil }
func (b *callBackend) EndScenario(ScenarioContext) error { if b.mode=="borrowed" { borrowed=b.rows }; b.rows=nil; return nil }
func (b *callBackend) EstablishEntity(r EntitySetupRequest) error {
 if b.mode=="unsupported" { return ErrUnsupported }
 if r.Entity!="calls.history.CallRecord" || r.State!="Completed" { return fmt.Errorf("undeclared entity/state") }
 if why:=primitive("uuid",r.Identity); why!="" { return fmt.Errorf("identity: %s",why) }
 if why:=primitive("timestamp",r.Fields["started_at"]); why!="" { return fmt.Errorf("time: %s",why) }
 n,ok:=asNumber(r.Fields["duration_seconds"]); if !ok || n<0 || n!=float64(int64(n)) { return fmt.Errorf("invalid duration/invariant") }
 for key,value:=range r.Fields { switch key { case "started_at","duration_seconds": case "note": if value!=nil { if _,ok:=value.(string); !ok { return fmt.Errorf("invalid Optional String") } }; default: return fmt.Errorf("undeclared field") } }
 if b.mode=="empty" { return nil }
 row:=map[string]Node{}; for k,v:=range r.Fields { row[k]=v }; row["call_id"]=r.Identity
 if b.mode=="wrong" { row["duration_seconds"]=float64(0) }
 b.rows=append(b.rows,row); return nil
}
func (b *callBackend) QueryView(r ViewRequest) (ViewResult,error) {
 rows:=append([]map[string]Node(nil),b.rows...)
 sort.Slice(rows,func(i,j int)bool { if b.mode=="reversed" { return rows[i]["started_at"].(string)<rows[j]["started_at"].(string) }; return rows[i]["started_at"].(string)>rows[j]["started_at"].(string) })
 return ViewResult{Rows:rows},nil
}
func (*callBackend) ExecuteCommand(CommandRequest)(CommandResult,error){ panic("no creator command") }
func (*callBackend) ObserveEvents(EventObservationRequest)([]ObservedEvent,error){ panic("no fabricated events") }
func (*callBackend) ConfigureExternalOutcome(ExternalOutcomeControl)error{ panic("no fake outcome") }
func (*callBackend) RedeliverEvent(RedeliveryRequest)error{ panic("no fabricated history") }
func (*callBackend) ObserveInvocations(InvocationObservationRequest)([]Invocation,error){ panic("no fabricated invocation") }
func TestEntitySetup(t *testing.T){ Run(t,func()Target{ b:=&callBackend{mode:os.Getenv("ESS_SETUP_FAULT")}; if b.mode=="absent-capability" { return struct{Target}{b} }; return b }) }
func TestEntitySetupAdmission(t *testing.T) {
 for _, mutation:=range []func(map[string]any){
  func(s map[string]any){s["identity"]=nil},
  func(s map[string]any){s["state"]="bad_state"},
  func(s map[string]any){s["extra"]="unknown"},
  func(s map[string]any){s["fields"].(map[string]any)["bad-field"]=1},
 } {
  raw,err:=strictJSON(suiteJSON); if err!=nil{t.Fatal(err)}
  root:=raw.(map[string]any); for _,scenario:=range root["scenarios"].(map[string]any){ mutation(scenario.(map[string]any)["steps"].([]any)[0].(map[string]any)); break }
  encoded,err:=json.Marshal(raw); if err!=nil{t.Fatal(err)}
  if _,err:=admitSuite(string(encoded)); err==nil{t.Fatal("malformed setup admitted")}
 }
}
"#;
}

#[test]
fn rejected_authored_candidate_needs_are_proved_independently_of_an_outside_survivor() {
    use ess_conformance::{
        coverage::{Origins, RefusalScope, Scope},
        coverage_build::{build, CoverageSource},
    };
    let ir = example("billing");
    for (valid_candidate, expected_scope, expected_status) in [
        (
            true,
            RefusalScope::OutsideComponent,
            ess_conformance::CountStatus::Passed,
        ),
        (
            false,
            RefusalScope::InScope,
            ess_conformance::CountStatus::Inconclusive,
        ),
    ] {
        let accepted = document(CREATED);
        let rejected = if valid_candidate {
            accepted.clone()
        } else {
            accepted.replace(
                "billing.invoice.CreateInvoice",
                "billing.invoice.UndeclaredCommand",
            )
        };
        let input = build(
            &ir,
            &[
                CoverageSource::new("a.yaml", accepted).unwrap(),
                CoverageSource::new("b.yaml", rejected).unwrap(),
            ],
            Scope::component("email-service").unwrap(),
            Origins::GeneratedAndAuthored,
        )
        .unwrap();
        let inventory = input.selected().coverage().unwrap();
        assert_eq!(inventory.counts.generated, 2);
        assert_eq!(inventory.counts.authored, 0);
        assert_eq!(inventory.counts.outside, 28);
        assert_eq!(inventory.counts.refused, 1);
        let refusal = &inventory.refused[0];
        assert_eq!(refusal.scope, expected_scope);
        assert_eq!(refusal.code, "ESS-AUTHOR-003");
        assert_eq!(refusal.source.as_ref().unwrap().as_str(), "b.yaml");
        assert_eq!(
            refusal
                .retained
                .as_ref()
                .unwrap()
                .source
                .as_ref()
                .unwrap()
                .as_str(),
            "a.yaml"
        );
        assert_eq!(refusal.needs.is_empty(), !valid_candidate);
        let run = ess_conformance::Runner::for_suite(input.selected().suite()).run_admitted(
            input.selected(),
            &ess_conformance::reference::Billing::new(),
        );
        let report = ess_conformance::CountReport::from_run(&run, input.selected()).unwrap();
        assert_eq!(report.counts().passed, 2);
        assert_eq!(report.conformance_status(), expected_status);
    }
}

/// An example directory, compiled from the files it lives in rather than from a copy inlined here.
fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));

    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();

    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((SpecSource::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("`{name}` validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("`{name}` resolves:\n{diagnostics}"))
}

// ---- writing and compiling one document ----------------------------------------------------------

/// A document with the four keys every scenario carries, and whatever body a case needs.
fn document(body: &str) -> String {
    format!(
        "type: ess-scenario/1\n\
         domain: billing.invoice\n\
         scenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n\
         {body}"
    )
}

/// The valid timeline every positive case starts from: one invoice, created and accepted.
const CREATED: &str = "\
timeline:
  - at: 2026-01-05T09:00:00Z
    command: billing.invoice.CreateInvoice
    actor: billing.invoice.Customer
    input:
      account_id: 00000000-0000-4000-8000-000000000001
      customer_email: buyer@example.test
      amount: {amount: 10, currency: EUR}
    outcome: accepted
";

/// The instances the cases that bind one declare.
///
/// Raw strings throughout, because a `\` continuation in a Rust literal swallows the leading
/// whitespace of the next line — and in YAML the leading whitespace is the structure.
const ARRANGED: &str = r"arrange:
  - instance: made
    entity: billing.invoice.Invoice
";

/// The occurrence the creating act publishes, required by name.
const OBSERVED: &str = r"    events:
      - event: billing.invoice.InvoiceCreated
        payload: {customer_email: buyer@example.test}
";

#[test]
fn coverage_builder_retains_authored_duplicate_ownership_and_original_source_bytes() {
    use ess_conformance::{
        coverage::{Disposition, Origin, Origins, Scope},
        coverage_build::{build, CoverageSource},
    };
    let ir = example("billing");
    let source = document(CREATED);
    let files = [
        CoverageSource::new("b.yaml", source.clone()).unwrap(),
        CoverageSource::new("a.yaml", source.clone()).unwrap(),
    ];
    let input = build(&ir, &files, Scope::System, Origins::Authored).expect("build inventory");
    let inventory = input.selected().coverage().unwrap();
    assert_eq!(inventory.authored.len(), 1);
    assert_eq!(
        inventory.outside.len(),
        0,
        "authored acquisition did not request generated inventory"
    );
    assert_eq!(inventory.refused.len(), 1);
    assert_eq!(inventory.refused[0].code, "ESS-AUTHOR-003");
    assert_eq!(
        inventory.refused[0].source.as_ref().unwrap().as_str(),
        "b.yaml"
    );
    let retained = inventory.refused[0].retained.as_ref().unwrap();
    assert_eq!(retained.origin, Origin::Authored);
    assert_eq!(retained.source.as_ref().unwrap().as_str(), "a.yaml");
    assert_eq!(
        inventory.authored_sources[files[1].identity()].disposition,
        Disposition::Accepted
    );
    assert_eq!(
        inventory.authored_sources[files[0].identity()].disposition,
        Disposition::Refused
    );
    assert_eq!(input.selected().suite().len(), 1);
    assert!(!inventory.is_complete());
    let repeated = [files[0].clone(), files[0].clone()];
    assert!(build(&ir, &repeated, Scope::System, Origins::Authored).is_err());
}

#[test]
fn independently_successful_authored_batches_are_refused_only_at_final_merge() {
    use ess_conformance::{
        coverage::{Disposition, Origins, Scope},
        coverage_build::{compile_sources, merge_batches, CoverageSource},
    };
    let ir = example("billing");
    let source = document(CREATED);
    let first = compile_sources(&ir, &[CoverageSource::new("a.yaml", &source).unwrap()]).unwrap();
    let second = compile_sources(&ir, &[CoverageSource::new("b.yaml", &source).unwrap()]).unwrap();
    assert_eq!(first.accepted(), 1);
    assert_eq!(second.accepted(), 1);
    assert_eq!(first.refusals().count(), 0);
    assert_eq!(second.refusals().count(), 0);
    let merged = merge_batches(&ir, &[second, first], Scope::System, Origins::Authored)
        .expect("final merge");
    let coverage = merged.selected().coverage().unwrap();
    assert_eq!(coverage.counts.authored, 1);
    assert_eq!(coverage.counts.refused, 1);
    assert_eq!(coverage.refused[0].code, "ESS-AUTHOR-003");
    assert_eq!(
        coverage.refused[0].source.as_ref().unwrap().as_str(),
        "b.yaml"
    );
    assert_eq!(
        coverage.refused[0]
            .retained
            .as_ref()
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .as_str(),
        "a.yaml"
    );
    assert_eq!(
        coverage.authored_sources
            [&ess_conformance::coverage::SourceIdentity::new("b.yaml").unwrap()]
            .disposition,
        Disposition::Refused
    );
}

#[test]
fn paired_browser_emission_retains_original_input_and_checks_the_actual_model() {
    use ess_conformance::{
        coverage::{Origins, Scope, SuiteReference},
        coverage_build::{build, CoverageSource},
    };
    let ir = example("billing");
    let input = build(
        &ir,
        &[CoverageSource::new("created.yaml", document(CREATED)).unwrap()],
        Scope::System,
        Origins::Authored,
    )
    .unwrap();
    let selected = input
        .select(
            &input
                .selected()
                .suite()
                .scenarios
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
        )
        .unwrap();
    let files = ess_conformance::web::emit_input(&ir, &selected).expect("paired browser emission");
    let replay: serde_json::Value = serde_json::from_str(&files["replay.json"].contents).unwrap();
    assert_eq!(replay["format"], "ess-conformance-replay/1");
    assert_eq!(
        replay["input"]["suite_json"],
        selected.selected().original_json()
    );
    assert_eq!(
        replay["input"]["parent_suites"][0],
        input.selected().original_json()
    );
    assert_eq!(
        replay["suite"],
        serde_json::to_value(SuiteReference::of(selected.selected())).unwrap()
    );
    assert_eq!(
        replay["model"]["spec_digest"],
        selected.selected().suite().provenance.spec_digest.as_str()
    );
    assert!(ess_conformance::web::emit_input(&example("gatepass"), &selected).is_err());
}

/// Binding the identity that act published.
const CAPTURED: &str = r"    capture: {instance: made, event: billing.invoice.InvoiceCreated, field: invoice_id}
";

/// Reading the invoice back out of the projection that holds it.
const READ: &str = r"assert:
  - view: billing.invoice.InvoiceById
    contains: {invoice_id: {$instance: made}}
";

/// A claim about which row a ranked view puts first.
const RANKED: &str = r"assert:
  - view: billing.invoice.OutstandingInvoices
    at:
      row: first
      fields: {total: {amount: 10, currency: EUR}}
";

/// A claim that a reader of the ranked view stopped the producer after two rows.
const HALTS: &str = r"assert:
  - view: billing.invoice.OutstandingInvoices
    halts_after: 2
";

/// The same claim, made after no rows at all.
const HALTS_AT_NOTHING: &str = r"assert:
  - view: billing.invoice.OutstandingInvoices
    halts_after: 0
";

/// The creating act with its instant named, and a second act ten seconds later to hang a claim on.
///
/// Ten seconds because every window case below states a length against it, and the compiler holds
/// the claim to the gap the file writes — so the fixture has to have a gap worth contradicting.
fn windowed(claim: &str) -> String {
    format!(
        "{CREATED}    mark: created\n  - at: 2026-01-05T09:00:10Z\n    \
         command: billing.invoice.CancelInvoice\n    input:\n      \
         invoice_id: 00000000-0000-4000-8000-000000000001\n    elapsed:\n      \
         - since: created\n{claim}"
    )
}

/// Compiles one document against a model.
fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

/// The one refusal a document produced, or a failure naming what it produced instead.
fn refusal(ir: &EssIr, text: &str) -> Refusal {
    let authoring = authoring(ir, text);
    assert!(
        authoring.scenarios.is_empty(),
        "a refused document produced a scenario, which would put an unresolvable check in a suite"
    );
    let mut refusals = authoring.refusals;
    assert_eq!(
        refusals.len(),
        1,
        "one document, one mistake, one refusal: {}",
        refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    refusals.remove(0)
}

/// The cause a document is refused for, when exactly one refusal is expected.
fn cause(ir: &EssIr, text: &str) -> Cause {
    refusal(ir, text).cause
}

/// Every cause a document produced, for a case that legitimately produces more than one.
fn causes(ir: &EssIr, text: &str) -> Vec<Cause> {
    authoring(ir, text)
        .refusals
        .into_iter()
        .map(|refusal| refusal.cause)
        .collect()
}

// ---- the shape of what compiles ------------------------------------------------------------------

#[test]
fn a_scenario_compiles_to_the_id_the_domain_and_the_name_make() {
    let ir = example("billing");
    let authoring = authoring(&ir, &document(CREATED));

    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let ids: Vec<String> = authoring
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert_eq!(ids, vec!["billing.invoice/authored/a-scenario"]);

    // The id is the distinction, and it survives the document: parsed back it is the same id, so a
    // report, a fault matrix and a `go test -run` filter each read it without being told.
    let parsed: ScenarioId = "billing.invoice/authored/a-scenario"
        .parse()
        .expect("an authored id round-trips");
    assert!(authoring.scenarios.contains_key(&parsed));
    assert!(matches!(parsed, ScenarioId::Authored { .. }));
}

#[test]
fn the_steps_are_the_vocabulary_a_generated_scenario_already_uses() {
    // The whole claim of the feature: an authored scenario is not a second kind of check. It comes
    // out as the same closed step vocabulary, so the runners that exist run it with no change to
    // `ConformanceTarget`.
    let ir = example("billing");
    let body = format!("{ARRANGED}{CREATED}{OBSERVED}{CAPTURED}{READ}");
    let authoring = authoring(&ir, &document(&body));
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);

    let scenario = authoring.scenarios.values().next().expect("one scenario");
    let shape: Vec<&str> = scenario
        .steps
        .iter()
        .map(|step| match step {
            ScenarioStep::ExecuteCommand { .. } => "execute",
            ScenarioStep::ExpectOutcome { .. } => "outcome",
            ScenarioStep::ExpectEvent { .. } => "event",
            ScenarioStep::CaptureInstance { .. } => "capture",
            ScenarioStep::EventuallyView { .. } => "eventually-view",
            ScenarioStep::QueryView { .. } => "query",
            ScenarioStep::ExpectView { .. } => "expect-view",
            other => panic!("an authored scenario emitted {other:?}"),
        })
        .collect();
    // `InvoiceById` is `eventual`, so the assertion retries. The author did not say so and cannot:
    // the model decided it, exactly as it decides it for a generated scenario.
    assert_eq!(
        shape,
        vec!["execute", "outcome", "event", "capture", "eventually-view"]
    );
}

#[test]
fn a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author() {
    // The one thing synthesis will not write and an author has to: which row is first. The keys it
    // is relative to are not written here — the view declares `order_by: issued_at desc`, and a
    // second copy in the scenario is the copy that goes stale.
    let ir = example("billing");
    let body = format!("{CREATED}{RANKED}");
    let authoring = authoring(&ir, &document(&body));
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);

    let scenario = authoring.scenarios.values().next().expect("one scenario");
    let ordered = scenario.steps.iter().any(|step| {
        matches!(
            step,
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::At { order_by, .. },
                ..
            } if order_by.iter().map(ToString::to_string).collect::<Vec<_>>()
                == vec!["issued_at desc"]
        )
    });
    assert!(ordered, "the view's own ranking reaches the assertion");
}

#[test]
fn the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones() {
    // The acceptance, read off the artifact rather than off a run: one document, two populations,
    // and a reader can tell which is which from the key alone.
    let suite = ConformanceSuite::from_json(include_str!(
        "../../../../suites/generated/billing/suite.json"
    ))
    .expect("the committed billing suite parses");
    let authored: Vec<String> = suite
        .scenarios
        .keys()
        .filter(|id| matches!(id, ScenarioId::Authored { .. }))
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        authored,
        vec!["billing.invoice/authored/outstanding-invoices-rank-latest-first"]
    );
    assert_eq!(suite.len(), 30, "twenty-nine obligations and one assertion");
}

#[test]
fn two_compilations_of_one_file_produce_identical_bytes() {
    // §37's determinism, for the authoring half. Compiled twice and written twice, and read back
    // and written again: a suite that moved between two compilations of one unchanged file could
    // not be drift-checked, which is what a committed suite is for.
    let ir = example("billing");
    let text =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml",
        ))
        .expect("the committed authored scenario is readable");

    let render = || {
        let authoring = compile_authored(&ir, &[Source::new("scenario.yaml", text.clone())]);
        assert!(authoring.is_complete(), "{:?}", authoring.refusals);
        let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
        for (id, scenario) in authoring.scenarios {
            suite.insert(id, scenario).expect("one scenario, one id");
        }
        suite
    };
    let first = render().to_canonical_json().unwrap();
    assert_eq!(first, render().to_canonical_json().unwrap());
    assert_eq!(
        first,
        ConformanceSuite::from_json(&first)
            .expect("the suite reads back")
            .to_canonical_json()
            .unwrap(),
        "compile, write, read, write: the same bytes"
    );
}

#[test]
fn the_order_the_files_are_handed_over_in_does_not_reach_the_result() {
    // `read_dir` is not ordered, so the compiler orders its input. Without it, which of two files
    // declaring one scenario is the duplicate would depend on the file system.
    let ir = example("billing");
    let first = Source::new("a.yaml", document(CREATED));
    let second = Source::new(
        "b.yaml",
        document(CREATED).replace("scenario: a-scenario", "scenario: b-scenario"),
    );
    let forwards = compile_authored(&ir, &[first.clone(), second.clone()]);
    let backwards = compile_authored(&ir, &[second, first]);
    assert_eq!(forwards, backwards);
    assert_eq!(forwards.scenarios.len(), 2);
}

// ---- one case per refusal ------------------------------------------------------------------------

#[test]
fn a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario() {
    let ir = example("billing");
    let text = document("timelime:\n  - at: 2026-01-05T09:00:00Z\n");
    assert!(matches!(cause(&ir, &text), Cause::Unreadable { .. }));
}

#[test]
fn a_format_this_build_does_not_implement_is_refused_before_anything_is_read() {
    let ir = example("billing");
    let text = document(CREATED).replace("type: ess-scenario/1", "type: ess-scenario/9");
    assert!(matches!(
        cause(&ir, &text),
        Cause::UnsupportedFormat { found } if found == "ess-scenario/9"
    ));
}

#[test]
fn two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other() {
    let ir = example("billing");
    let authoring = compile_authored(
        &ir,
        &[
            Source::new("a.yaml", document(CREATED)),
            Source::new("b.yaml", document(CREATED)),
        ],
    );
    assert_eq!(authoring.scenarios.len(), 1, "the first one stands");
    assert!(matches!(
        authoring.refusals.as_slice(),
        [Refusal { origin, cause: Cause::Duplicate { first }, .. }]
            if origin == "b.yaml" && first == "a.yaml"
    ));
}

#[test]
fn a_domain_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace("domain: billing.invoice", "domain: billing.invoicing");
    assert!(matches!(
        cause(&ir, &text),
        Cause::UndeclaredDomain { domain, .. } if domain == "billing.invoicing"
    ));
}

#[test]
fn an_entity_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body =
        format!("arrange:\n  - instance: made\n    entity: billing.invoice.Facture\n{CREATED}");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredEntity { entity } if entity == "billing.invoice.Facture"
    ));
}

#[test]
fn a_command_the_model_does_not_declare_is_refused_by_name() {
    // The case the whole feature is measured by: a scenario naming a command the specification does
    // not declare is refused when it is compiled, naming the command and the file it is in.
    let ir = example("billing");
    let text = document(CREATED).replace(
        "command: billing.invoice.CreateInvoice",
        "command: billing.invoice.CreateInvoce",
    );
    let refused = refusal(&ir, &text);
    assert!(matches!(
        &refused.cause,
        Cause::UndeclaredCommand { command } if command == "billing.invoice.CreateInvoce"
    ));
    let printed = refused.to_string();
    assert!(
        printed.contains("billing.invoice.CreateInvoce"),
        "{printed}"
    );
    assert!(printed.contains("scenario.yaml"), "{printed}");
    assert!(printed.contains("ESS-AUTHOR-006"), "{printed}");
}

#[test]
fn an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does() {
    let ir = example("billing");
    let text = document(CREATED).replace("outcome: accepted", "outcome: approved");
    assert!(matches!(
        cause(&ir, &text),
        Cause::UndeclaredOutcome { outcome, declared, .. }
            if outcome == "approved" && declared == vec!["accepted", "rejected"]
    ));
}

#[test]
fn an_actor_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace(
        "actor: billing.invoice.Customer",
        "actor: billing.invoice.Cusomer",
    );
    assert!(matches!(
        cause(&ir, &text),
        Cause::UndeclaredActor { actor } if actor == "billing.invoice.Cusomer"
    ));
}

#[test]
fn an_actor_the_specification_does_not_grant_the_command_is_refused() {
    // Declared, spelt right, and not permitted: `may:` is a claim about who can do what, and a
    // scenario acting as somebody the model does not permit is checking a system it does not
    // describe.
    let ir = example("billing");
    let text = document(CREATED).replace(
        "actor: billing.invoice.Customer",
        "actor: billing.invoice.Auditor",
    );
    assert!(matches!(
        cause(&ir, &text),
        Cause::ActorMayNot { actor, command }
            if actor.to_string() == "billing.invoice.Auditor"
                && command.to_string() == "billing.invoice.CreateInvoice"
    ));
}

#[test]
fn an_event_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!("{CREATED}    events:\n      - event: billing.invoice.InvoiceMade\n");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredEvent { event } if event == "billing.invoice.InvoiceMade"
    ));
}

#[test]
fn an_error_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!("{CREATED}    error:\n      name: billing.invoice.BadAmount\n");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredError { error } if error == "billing.invoice.BadAmount"
    ));
}

#[test]
fn a_view_the_model_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}assert:\n  - view: billing.invoice.AllInvoices\n    counts: {{at_least: 1}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredView { view } if view == "billing.invoice.AllInvoices"
    ));
}

#[test]
fn a_field_the_surface_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace("      account_id:", "      acount_id:");
    let found = causes(&ir, &text);
    assert!(
        found.iter().any(|cause| matches!(
            cause,
            Cause::UndeclaredField { field, .. } if field == "acount_id"
        )),
        "{found:?}"
    );
}

#[test]
fn a_declared_field_nothing_supplies_is_refused_by_name() {
    let ir = example("billing");
    let text = document(CREATED).replace("      amount: {amount: 10, currency: EUR}\n", "");
    assert!(matches!(
        cause(&ir, &text),
        Cause::MissingField { field, .. } if field == "amount"
    ));
}

#[test]
fn a_value_the_declared_type_does_not_admit_is_refused_where_it_sits() {
    let ir = example("billing");
    let text = document(CREATED).replace(
        "{amount: 10, currency: EUR}",
        "{amount: ten, currency: EUR}",
    );
    assert!(matches!(
        cause(&ir, &text),
        Cause::ValueRejected { detail, .. } if detail.contains("amount.amount")
    ));
}

#[test]
fn a_name_a_closed_set_does_not_have_is_refused_with_the_set() {
    // The one case `examples/billing` cannot make: it declares an enum and no command takes one.
    // `gatepass.visit.RegisterVisit` does, which is why this document is written against it.
    let ir = example("gatepass");
    let text = "\
type: ess-scenario/1
domain: gatepass.visit
scenario: a-scenario
summary: What this scenario proves, in one line.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: gatepass.visit.RegisterVisit
    input:
      building: Basement
";
    let found = causes(&ir, text);
    assert!(
        found.iter().any(|cause| matches!(
            cause,
            Cause::UndeclaredVariant { value, variants, .. }
                if value == "Basement" && variants == &["North", "South", "Annex"]
        )),
        "{found:?}"
    );
}

#[test]
fn a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant() {
    // A lifecycle's states reach the model as an enum like any other, and a reader who wrote
    // `Payed` needs to be told which states the *invoice* has.
    let ir = example("billing");
    let body = format!(
        "{CREATED}    error:\n      name: billing.invoice.InvoiceStateConflict\n      \
         fields: {{state: Payed}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UndeclaredState { entity, state, declared }
            if entity.to_string() == "billing.invoice.Invoice"
                && state == "Payed"
                && declared == vec!["Cancelled", "Draft", "Issued", "Paid"]
    ));
}

#[test]
fn an_instance_the_arrangement_does_not_declare_is_refused_by_name() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}\
         assert:\n  - view: billing.invoice.InvoiceById\n    \
         contains: {{invoice_id: {{$instance: nobody}}}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnarrangedInstance { instance, .. } if instance.as_str() == "nobody"
    ));
}

#[test]
fn an_instance_named_before_anything_binds_it_is_refused() {
    // A suite carries no identity of its own — the run mints it — so a reference before the capture
    // that binds it is a step the runner could not have executed.
    let ir = example("billing");
    let body = format!(
        "arrange:\n  - instance: made\n    entity: billing.invoice.Invoice\n\
         {CREATED}\
         assert:\n  - view: billing.invoice.InvoiceById\n    \
         contains: {{invoice_id: {{$instance: made}}}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnboundInstance { instance } if instance.as_str() == "made"
    ));
}

#[test]
fn a_value_read_off_an_event_nothing_required_is_refused() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}  - at: 2026-01-05T09:00:01Z\n    \
         command: billing.invoice.PayInvoice\n    input:\n      \
         invoice_id: {{$observed: {{event: billing.invoice.InvoicePaid, field: invoice_id}}}}\n      \
         amount: {{amount: 10, currency: EUR}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::Unobserved { event } if event.to_string() == "billing.invoice.InvoicePaid"
    ));
}

#[test]
fn a_reference_where_the_suite_compares_a_value_it_carries_is_refused() {
    // `ExpectEvent`'s payload is compared against values the suite holds, so a reference there is a
    // claim the format cannot make — refused rather than silently dropped.
    let ir = example("billing");
    let body = format!(
        "arrange:\n  - instance: made\n    entity: billing.invoice.Invoice\n\
         {CREATED}    events:\n      - event: billing.invoice.InvoiceCreated\n        \
         payload: {{invoice_id: {{$instance: made}}}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::NotComparable { field, .. } if field == "invoice_id"
    ));
}

#[test]
fn an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused() {
    // `PayInvoice` takes an invoice and an amount, and the model says which is which. Binding the
    // invoice to the amount is a scenario nothing would ever have executed.
    let ir = example("billing");
    let body = format!(
        "arrange:\n  - instance: made\n    entity: billing.invoice.Invoice\n\
         {CREATED}    capture: {{instance: made, event: billing.invoice.InvoiceCreated, \
         field: invoice_id}}\n  - at: 2026-01-05T09:00:01Z\n    \
         command: billing.invoice.PayInvoice\n    input:\n      \
         invoice_id: {{$instance: made}}\n      amount: {{$instance: made}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::InstanceMistyped { field, declared, identity, .. }
            if field == "amount"
                && declared == "billing.invoice.Money"
                && identity == "billing.invoice.InvoiceId"
    ));
}

#[test]
fn a_timeline_whose_instants_do_not_ascend_is_refused() {
    // The file's order is the scenario's order, and `at:` is what states it. Without this a moved
    // block changes what the scenario means and nothing says so.
    let ir = example("billing");
    let body = format!(
        "{CREATED}  - at: 2026-01-05T08:00:00Z\n    \
         command: billing.invoice.CancelInvoice\n    input:\n      \
         invoice_id: 00000000-0000-4000-8000-000000000001\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnorderedTimeline { .. }
    ));
}

#[test]
fn a_position_in_a_view_that_declares_no_order_is_refused() {
    // `InvoiceById` declares no `order_by:`, so "the first row" is a different row on every read
    // and an assertion about it is a coin toss reported as a check.
    let ir = example("billing");
    let body = format!(
        "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    at:\n      row: first\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::Unordered { view } if view.to_string() == "billing.invoice.InvoiceById"
    ));
}

#[test]
fn an_assertion_that_states_other_than_one_claim_is_refused() {
    let ir = example("billing");
    let two = format!(
        "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    \
         counts: {{at_least: 1}}\n    contains: {{}}\n"
    );
    assert!(matches!(
        cause(&ir, &document(&two)),
        Cause::AmbiguousClaim { stated, .. } if stated == vec!["contains", "counts"]
    ));

    let none = format!("{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n");
    assert!(matches!(
        cause(&ir, &document(&none)),
        Cause::AmbiguousClaim { stated, .. } if stated.is_empty()
    ));
}

#[test]
fn a_predicate_reading_something_the_view_does_not_publish_is_refused() {
    let ir = example("billing");
    let body = format!(
        "{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    \
         satisfies: customer_email == buyer@example.test\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnreadablePredicate { path, .. } if path == "customer_email"
    ));
}

#[test]
fn authored_predicate_operand_errors_have_their_own_refusal() {
    let ir = example("billing");
    for predicate in [
        "total.amount == text",
        "{total.amount: {any_of: [1, text]}}",
        "{forall: {in: total.amount, as: n, that: true}}",
    ] {
        let body = format!("{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    satisfies: {predicate}\n");
        let refused = refusal(&ir, &document(&body));
        assert_eq!(refused.code().to_string(), "ESS-AUTHOR-035");
    }
}

#[test]
fn a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check() {
    let ir = example("billing");
    assert!(matches!(cause(&ir, &document("")), Cause::NothingHappens));
}

#[test]
fn an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act() {
    // The order is the semantics rather than a filing decision. A window is a claim about the time
    // *before* an act, so a runner that checked it after the act had run would be checking a system
    // the act had already changed — which is how "twenty seconds passed" becomes true for the wrong
    // reason.
    let ir = example("billing");
    let body = windowed(
        "        not_before: PT10S\n      - since: created\n        \
         quiet:\n          for: PT10S\n          \
         events: [billing.invoice.InvoicePaid]\n",
    );
    let authoring = authoring(&ir, &document(&body));
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let scenario = authoring.scenarios.values().next().expect("one scenario");
    let shape: Vec<&str> = scenario
        .steps
        .iter()
        .map(|step| match step {
            ScenarioStep::ExecuteCommand { .. } => "execute",
            ScenarioStep::ExpectOutcome { .. } => "outcome",
            ScenarioStep::MarkInstant { .. } => "mark",
            ScenarioStep::ExpectNotBefore { .. } => "not-before",
            ScenarioStep::ExpectQuiet { .. } => "quiet",
            other => panic!("an elapsed claim compiled to {other:?}"),
        })
        .collect();
    assert_eq!(
        shape,
        vec![
            "execute",
            "outcome",
            "mark",
            "not-before",
            "quiet",
            "execute"
        ],
        "the instant is named after the act that reaches it, and the windows close before the act \
         that claims them"
    );
    assert!(scenario.steps.iter().any(|step| matches!(
        step,
        ScenarioStep::ExpectNotBefore { instant, elapsed }
            if instant.as_str() == "created" && elapsed.get() == 10
    )));
}

#[test]
fn a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are() {
    // The refusal the whole feature turns on. The suite this format was extended for had a bounded
    // negative whose window opened wherever the preceding block happened to end, so inserting one
    // arrangement step moved five assertions and nothing said so. An anchor a reader reconstructs
    // is an anchor that has already moved.
    let ir = example("billing");
    let body =
        windowed("        not_before: PT10S\n").replace("- since: created", "- since: dialled");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnmarkedInstant { instant, marked }
            if instant.as_str() == "dialled" && marked == vec!["created".to_owned()]
    ));
}

#[test]
fn an_act_cannot_open_a_window_at_its_own_instant() {
    // The same refusal, reached the other way, and the reason `mark:` is recorded after the act
    // rather than before it: a window from an act to itself has no width, and a claim about no time
    // is a claim that cannot fail.
    let ir = example("billing");
    let body = format!(
        "{CREATED}    mark: created\n    elapsed:\n      - since: created\n        \
         not_before: PT10S\n"
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::UnmarkedInstant { instant, .. } if instant.as_str() == "created"
    ));
}

#[test]
fn one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one() {
    let ir = example("billing");
    let body = format!(
        "{}    mark: created\n",
        windowed("        not_before: PT10S\n")
    );
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::DuplicateInstant { instant, .. } if instant.as_str() == "created"
    ));
}

#[test]
fn a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail() {
    let ir = example("billing");
    assert!(matches!(
        cause(&ir, &document(&windowed("        not_before: PT0S\n"))),
        Cause::VacuousWindow { instant } if instant.as_str() == "created"
    ));
}

#[test]
fn a_bounded_negative_that_forbids_no_event_is_refused() {
    let ir = example("billing");
    let body = windowed("        quiet:\n          for: PT10S\n          events: []\n");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::QuietAboutNothing { instant, duration }
            if instant.as_str() == "created" && duration.get() == 10
    ));
}

#[test]
fn a_claim_the_timelines_own_instants_contradict_is_refused() {
    // What makes `at:` load-bearing. It still reaches no runner, and it is now the thing a duration
    // is held against: a file that writes ten seconds between two acts and claims thirty is a
    // document saying two things, and the reader believes the timeline.
    let ir = example("billing");
    assert!(matches!(
        cause(&ir, &document(&windowed("        not_before: PT30S\n"))),
        Cause::WindowContradictsTimeline { stated, elapsed, written, .. }
            if stated == "not_before" && elapsed.get() == 30 && written.get() == 10
    ));

    // And the other way for `within`, whose contradiction is the opposite inequality: the file puts
    // the act ten seconds later and the claim says it happened inside five.
    assert!(matches!(
        cause(&ir, &document(&windowed("        within: PT5S\n"))),
        Cause::WindowContradictsTimeline { stated, elapsed, written, .. }
            if stated == "within" && elapsed.get() == 5 && written.get() == 10
    ));
}

#[test]
fn a_window_that_states_other_than_one_bound_is_refused() {
    let ir = example("billing");
    let two = windowed("        not_before: PT10S\n        within: PT10S\n");
    assert!(matches!(
        cause(&ir, &document(&two)),
        Cause::AmbiguousWindow { stated, .. } if stated == vec!["not_before", "within"]
    ));

    let none = windowed("        {}\n").replace(
        "      - since: created\n        {}\n",
        "      - since: created\n",
    );
    assert!(matches!(
        cause(&ir, &document(&none)),
        Cause::AmbiguousWindow { stated, .. } if stated.is_empty()
    ));
}

// ---- the set is closed ----------------------------------------------------------------------------

#[test]
fn every_cause_is_reachable_from_a_document() {
    // The guard on the list above. A cause added to the module without a case that reaches it would
    // ship as a refusal nobody has seen the wording of — and the wording is the whole product here.
    let billing = example("billing");
    let gatepass = example("gatepass");
    let reached: BTreeSet<String> = refusable()
        .into_iter()
        .flat_map(|(model, documents)| {
            let ir = if model == "gatepass" {
                &gatepass
            } else {
                &billing
            };
            let sources: Vec<Source> = documents
                .into_iter()
                .enumerate()
                .map(|(index, text)| Source::new(format!("scenario-{index}.yaml"), text))
                .collect();
            compile_authored(ir, &sources).refusals
        })
        .map(|refusal| refusal.code().to_string())
        .collect();
    let declared: BTreeSet<String> = (1..=CAUSES)
        .map(|number| Code::new("AUTHOR", number).to_string())
        .collect();
    assert_eq!(
        reached, declared,
        "every numbered cause has a document that reaches it, and no number is unused"
    );
}

/// How many causes `ess_conformance::authored::Cause` numbers.
///
/// Written down rather than counted, because the point of the case above is that the numbering and
/// the documents agree: a count taken from the enum would agree with itself whatever happened.
const CAUSES: u16 = 35;

/// One entry per refusal, each the documents that reach it compiled together.
///
/// Restated here rather than shared with the cases above, because a case reads better with its
/// document beside it and this reads better as a list. All but one entry is a single document; the
/// duplicate is the one refusal that is about two.
fn refusable() -> Vec<(&'static str, Vec<String>)> {
    let billing = |body: String| ("billing", vec![document(&body)]);
    let edited = |from: &str, to: &str| ("billing", vec![document(CREATED).replace(from, to)]);
    vec![
        // 1 unreadable, 2 unsupported format, 3 duplicate, 4 undeclared domain
        billing("timelime: []\n".to_owned()),
        edited("ess-scenario/1", "ess-scenario/9"),
        (
            "billing",
            vec![document(CREATED), document(CREATED)],
        ),
        edited("domain: billing.invoice", "domain: billing.ledger"),
        // 5 entity, 6 command, 7 outcome, 8 actor, 9 grant
        billing(format!(
            "arrange:\n  - instance: made\n    entity: billing.invoice.Facture\n{CREATED}"
        )),
        edited("billing.invoice.CreateInvoice", "billing.invoice.CreateInvoce"),
        edited("outcome: accepted", "outcome: approved"),
        edited("actor: billing.invoice.Customer", "actor: billing.invoice.Cusomer"),
        edited("actor: billing.invoice.Customer", "actor: billing.invoice.Auditor"),
        // 10 event, 11 error, 12 view
        billing(format!(
            "{CREATED}    events:\n      - event: billing.invoice.InvoiceMade\n"
        )),
        billing(format!(
            "{CREATED}    error:\n      name: billing.invoice.BadAmount\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.AllInvoices\n    counts: {{at_least: 1}}\n"
        )),
        // 13 undeclared field, 14 missing field, 15 value rejected
        edited("      account_id:", "      acount_id:"),
        edited("      amount: {amount: 10, currency: EUR}\n", ""),
        edited("{amount: 10, currency: EUR}", "{amount: ten, currency: EUR}"),
        // 16 variant — the one case `examples/billing` cannot make
        (
            "gatepass",
            vec!["type: ess-scenario/1\ndomain: gatepass.visit\nscenario: a-scenario\n\
                  summary: What this scenario proves.\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    \
                  command: gatepass.visit.RegisterVisit\n    input:\n      building: Basement\n"
                .to_owned()],
        ),
        // 17 state
        billing(format!(
            "{CREATED}    error:\n      name: billing.invoice.InvoiceStateConflict\n      \
             fields: {{state: Payed}}\n"
        )),
        // 18 unarranged, 19 unbound, 20 unobserved, 21 not comparable, 22 mistyped
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    \
             contains: {{invoice_id: {{$instance: nobody}}}}\n"
        )),
        billing(format!("{ARRANGED}{CREATED}{READ}")),
        billing(format!(
            "{CREATED}  - at: 2026-01-05T09:00:01Z\n    command: billing.invoice.PayInvoice\n    \
             input:\n      invoice_id: {{$observed: {{event: billing.invoice.InvoicePaid, \
             field: invoice_id}}}}\n      amount: {{amount: 10, currency: EUR}}\n"
        )),
        billing(format!(
            "{ARRANGED}{CREATED}    events:\n      - event: billing.invoice.InvoiceCreated\n        \
             payload: {{invoice_id: {{$instance: made}}}}\n"
        )),
        billing(format!(
            "{ARRANGED}{CREATED}{CAPTURED}  - at: 2026-01-05T09:00:01Z\n    \
             command: billing.invoice.PayInvoice\n    input:\n      \
             invoice_id: {{$instance: made}}\n      amount: {{$instance: made}}\n"
        )),
        // 23 unordered timeline, 24 unordered view, 25 ambiguous, 26 unreadable predicate
        billing(format!(
            "{CREATED}  - at: 2026-01-05T08:00:00Z\n    command: billing.invoice.CancelInvoice\n    \
             input:\n      invoice_id: 00000000-0000-4000-8000-000000000001\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    at:\n      row: first\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    counts: {{at_least: 1}}\n    \
             contains: {{}}\n"
        )),
        billing(format!(
            "{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    \
             satisfies: customer_email == buyer@example.test\n"
        )),
        // 27 nothing happens
        billing(String::new()),
        // 28 unmarked instant, 29 duplicate instant, 30 vacuous window
        (
            "billing",
            vec![document(
                &windowed("        not_before: PT10S\n").replace("- since: created", "- since: dialled"),
            )],
        ),
        billing(format!(
            "{}    mark: created\n",
            windowed("        not_before: PT10S\n")
        )),
        billing(windowed("        not_before: PT0S\n")),
        // 31 quiet about nothing, 32 contradicted by the timeline, 33 ambiguous window
        billing(windowed(
            "        quiet:\n          for: PT10S\n          events: []\n",
        )),
        billing(windowed("        not_before: PT30S\n")),
        billing(windowed("        not_before: PT10S\n        within: PT10S\n")),
        // 34 a halt after no rows at all
        billing(format!("{CREATED}{HALTS_AT_NOTHING}")),
        // 35 semantically incompatible scalar operands
        billing(format!("{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    satisfies: total.amount == text\n")),
    ]
}

// ---- the early stop ------------------------------------------------------------------------------

/// A model with an ordered listing the specification declares `eventual`.
///
/// Neither example carries one: `billing.invoice.OutstandingInvoices` is ordered and
/// `read_your_writes`, and `billing.invoice.InvoiceById` is `eventual` and unordered. The eventual
/// half of a halt claim is a real branch of the compiler and this is the smallest model that reaches
/// it — which is the shape a real one has, because a ranked listing a projection maintains is
/// exactly the case the feature was built for.
const RANKED_AND_EVENTUAL: &str = r"
format: ess/1
system: shelf
version: v1
domain: shelf.orders

types:
  - name: shelf.orders.OrderId
    kind: newtype
    of: Uuid

entities:
  - name: shelf.orders.Order
    identity:
      name: order_id
      type: shelf.orders.OrderId
    fields:
      - name: weight_grams
        type: Integer
    lifecycle:
      initial: Placed
      states: [Placed, Shipped]
      terminal: [Shipped]
      transitions:
        - name: ship
          from: [Placed]
          to: Shipped

events:
  - name: shelf.orders.OrderPlaced
    fields:
      - name: order_id
        type: shelf.orders.OrderId

  - name: shelf.orders.OrderShipped
    fields:
      - name: order_id
        type: shelf.orders.OrderId

commands:
  - name: shelf.orders.PlaceOrder
    input:
      - name: weight_grams
        type: Integer
    outcomes:
      - name: accepted
        creates: shelf.orders.Order
        instance: order_id
        emits:
          - shelf.orders.OrderPlaced

  - name: shelf.orders.ShipOrder
    input:
      - name: order_id
        type: shelf.orders.OrderId
    outcomes:
      - name: shipped
        moves: shelf.orders.Order.ship
        instance: order_id
        emits:
          - shelf.orders.OrderShipped

views:
  - name: shelf.orders.HeaviestFirst
    source: shelf.orders.Order
    consistency: eventual
    order_by:
      - weight_grams desc
    fields:
      - name: order_id
        type: shelf.orders.OrderId
      - name: weight_grams
        type: Integer
";

/// A specification written inline, for a corner neither example carries.
fn fixture(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let specification = Specification::assemble([(SpecSource::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

#[test]
fn a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows() {
    // The whole design decision, asserted rather than argued. `halts_after` does not become a
    // seventh `ViewExpectation`, because an expectation is decided against the rows a read returned
    // and two targets returning identical rows differ on whether the producer stopped. It becomes a
    // step that carries the view and its parameters, so the *read* is the bounded one.
    let ir = example("billing");
    let authoring = authoring(&ir, &document(&format!("{CREATED}{HALTS}")));
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);

    let scenario = authoring.scenarios.values().next().expect("one scenario");
    let halts: Vec<&ScenarioStep> = scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(
                step,
                ScenarioStep::ExpectHalt { .. } | ScenarioStep::EventuallyHalt { .. }
            )
        })
        .collect();
    assert!(
        matches!(
            halts.as_slice(),
            [ScenarioStep::ExpectHalt { view, after, .. }]
                if view.to_string() == "billing.invoice.OutstandingInvoices" && *after == 2
        ),
        "one halt step, naming the view it reads and how many rows the reader takes: {halts:?}"
    );
    assert!(
        !scenario.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::QueryView { .. } | ScenarioStep::ExpectView { .. }
        )),
        "and no query beside it: the halt is the read, so a second one would be a second read the \
         claim is not about"
    );
}

#[test]
fn a_halt_of_a_listing_the_model_calls_eventual_retries_because_the_model_said_so() {
    // The style is never written by an author, exactly as it is never written for `at:` or
    // `contains:`. A listing a projection maintains may not hold the rows yet, and a scan of one
    // that has not caught up runs out before the reader stops it — which is lag, not a wrong
    // implementation.
    let ir = fixture(RANKED_AND_EVENTUAL);
    let text = "type: ess-scenario/1\n\
                domain: shelf.orders\n\
                scenario: a-reader-stops-the-listing\n\
                summary: A reader of the ranked listing takes two rows and stops it.\n\
                timeline:\n  \
                - at: 2026-01-05T09:00:00Z\n    \
                command: shelf.orders.PlaceOrder\n    \
                input:\n      weight_grams: 900\n    \
                outcome: accepted\n\
                assert:\n  \
                - view: shelf.orders.HeaviestFirst\n    \
                halts_after: 2\n";
    let authoring = authoring(&ir, text);
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);

    let scenario = authoring.scenarios.values().next().expect("one scenario");
    assert!(
        scenario.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::EventuallyHalt { after, .. } if *after == 2
        )),
        "the eventual listing gets the retrying step: {:?}",
        scenario.steps
    );
}

#[test]
fn a_halt_after_no_rows_at_all_is_refused_rather_than_compiled() {
    // `ESS-AUTHOR-034`. A reader that takes no row never sees one and so never says stop, and what
    // an honest source produced before being refused its first row is a different number in two
    // implementations that are both right. There is nothing here for a target to be wrong about.
    let ir = example("billing");
    let refused = refusal(&ir, &document(&format!("{CREATED}{HALTS_AT_NOTHING}")));
    assert_eq!(refused.code().to_string(), "ESS-AUTHOR-034");
    assert!(matches!(
        refused.cause,
        Cause::HaltsAtNothing { ref view } if view.to_string() == "billing.invoice.OutstandingInvoices"
    ));
}

#[test]
fn a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so()
{
    // `ESS-AUTHOR-024`, in a new place rather than a new code: it is the same mistake with the same
    // repair. A halt after two rows of an unordered listing says the reader stopped and says nothing
    // about what it read, because the two rows are different rows on every read — and what these
    // scenarios claim is that a consumer of an *ordered* listing did not have to see the rest of it.
    let ir = example("billing");
    let body =
        format!("{CREATED}assert:\n  - view: billing.invoice.InvoiceById\n    halts_after: 2\n");
    assert!(matches!(
        cause(&ir, &document(&body)),
        Cause::Unordered { view } if view.to_string() == "billing.invoice.InvoiceById"
    ));
}

#[test]
fn a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one() {
    // The seventh key is exclusive with the other six, and the refusal names it — a message that
    // listed six of the seven would send an author looking for a key the format has.
    let ir = example("billing");
    let body = format!(
        "{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    \
         halts_after: 2\n    counts: {{at_least: 1}}\n"
    );
    match cause(&ir, &document(&body)) {
        Cause::AmbiguousClaim { stated, .. } => {
            assert_eq!(stated, vec!["counts", "halts_after"]);
        }
        other => panic!("two claims in one assertion is refused as two: {other}"),
    }
}

#[test]
fn authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate() {
    let ir = example("billing");
    let body = |predicate: &str| {
        document(&format!("{CREATED}assert:\n  - view: billing.invoice.OutstandingInvoices\n    satisfies: {predicate}\n"))
    };
    for predicate in [
        "{total: {exists: true}}",
        "total",
        "param.limit > 0",
        "total.amount.nonexistent > 0",
    ] {
        assert_eq!(
            refusal(&ir, &body(predicate)).code().to_string(),
            "ESS-AUTHOR-026"
        );
    }
    let authored = authoring(&ir, &body("total.amount > 0"));
    assert!(authored.refusals.is_empty());
    assert_eq!(authored.scenarios.len(), 1);
}
