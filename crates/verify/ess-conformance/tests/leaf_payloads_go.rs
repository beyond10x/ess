//! The generated Go runtime runs the #179 dotted-leaf suite (`ess-conformance/26`) and gives the
//! Rust reference runner's verdict for every scenario (beyond10x/ess#188).
//!
//! #188 as reported: an unchanged specification with a nested `sets:` struct holding one
//! `{generated: true}` leaf synthesizes suite/26 on 0.38.0, and the Go package the same release
//! generates refused it at admission — `suite admission: unsupported suite version
//! "ess-conformance/26"`, zero verdicts. The target here is written in Go, with native Go values,
//! as an adopter's is.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The #179 shape, as `tests/leaf_payloads.rs` holds it: four leaves read from the input, one
/// generated.
const DIALER: &str = "format: ess/14
system: demo
version: v1
domain: demo.dialer
types:
  - {name: demo.dialer.AgentId, kind: newtype, of: String}
  - name: demo.dialer.Lead
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: uid, type: String}
      - {name: number, type: String}
      - {name: rank, type: Integer}
      - {name: data, type: String}
entities:
  - name: demo.dialer.Membership
    identity: {name: agent_id, type: demo.dialer.AgentId}
    fields:
      - {name: lead, type: Optional<demo.dialer.Lead>}
    lifecycle: {initial: Idle, states: [Idle], terminal: [Idle]}
events:
  - name: demo.dialer.Joined
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
  - name: demo.dialer.LeadSet
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: demo.dialer.Lead}
actors:
  - {name: demo.dialer.Agent, may: [demo.dialer.Join, demo.dialer.SetLead]}
commands:
  - name: demo.dialer.Join
    outcomes:
      - name: joined
        creates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.Joined]
        payload:
          demo.dialer.Joined: {agent_id: {generated: true}}
  - name: demo.dialer.SetLead
    input:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead_id, type: String}
      - {name: lead_uid, type: String}
      - {name: lead_number, type: String}
      - {name: lead_data, type: String}
    outcomes:
      - name: lead-set
        updates: demo.dialer.Membership
        instance: agent_id
        emits: [demo.dialer.LeadSet]
        payload:
          demo.dialer.LeadSet:
            agent_id: input.agent_id
            lead:
              id: input.lead_id
              uid: input.lead_uid
              number: input.lead_number
              rank: {generated: true}
              data: input.lead_data
        sets:
          lead:
            id: input.lead_id
            uid: input.lead_uid
            number: input.lead_number
            rank: {generated: true}
            data: input.lead_data
views:
  - name: demo.dialer.Memberships
    source: demo.dialer.Membership
    consistency: read_your_writes
    fields:
      - {name: agent_id, type: demo.dialer.AgentId}
      - {name: lead, type: Optional<demo.dialer.Lead>}
";

const SET_LEAD: &str = "demo.dialer.SetLead/outcome/lead-set";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("dialer.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// The mutations the Go fixture and the Rust target below both implement, by the name the Go
/// fixture reads from `ESS_TEST_DIALER_MODE`.
const MODES: [&str; 3] = [
    "correct",
    "drops-number-from-the-row",
    "wrong-number-on-the-event",
];

/// The Rust twin of `fixtures/dialer-runtime-go.go`, for the reference runner's verdicts.
struct Dialer {
    mode: &'static str,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Dialer {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("dialer-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.dialer.Join" => {
                let id = format!("agent-{}", self.minted.get());
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("agent_id".to_owned(), Node::Text(id.clone())),
                        ("lead".to_owned(), Node::Null),
                    ]),
                );
                SemanticCommandResult::took(outcome(&command, "joined")).emitting(
                    ObservedEvent::new("demo.dialer.Joined".parse().unwrap())
                        .with("agent_id", Node::Text(id)),
                )
            }
            "demo.dialer.SetLead" => {
                let Some(Node::Text(id)) = request.input.get("agent_id") else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let Some(row) = rows.get_mut(id) else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let lead = BTreeMap::from([
                    ("id".to_owned(), request.input["lead_id"].clone()),
                    ("uid".to_owned(), request.input["lead_uid"].clone()),
                    ("number".to_owned(), request.input["lead_number"].clone()),
                    ("rank".to_owned(), Node::Number(0_i64.into())),
                    ("data".to_owned(), request.input["lead_data"].clone()),
                ]);
                let mut stored = lead.clone();
                if self.mode == "drops-number-from-the-row" {
                    stored.remove("number");
                }
                let mut published = lead;
                if self.mode == "wrong-number-on-the-event" {
                    published.insert("number".to_owned(), request.input["lead_data"].clone());
                }
                row.insert("lead".to_owned(), Node::Map(stored));
                SemanticCommandResult::took(outcome(&command, "lead-set")).emitting(
                    ObservedEvent::new("demo.dialer.LeadSet".parse().unwrap())
                        .with("agent_id", Node::Text(id.clone()))
                        .with("lead", Node::Map(published)),
                )
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

#[test]
fn issue_188_go_runs_the_dotted_leaf_suite_with_the_reference_verdicts() {
    let suite = suite(DIALER);
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/26",
        "the #188 specification synthesizes suite/26"
    );
    let directory = support_go::package(
        "dialer",
        &suite,
        &[(
            "dialer_test.go",
            include_str!("fixtures/dialer-runtime-go.go"),
        )],
    );
    for mode in MODES {
        let go = support_go::go_test(
            &directory,
            "TestDialerRuntime",
            &[("ESS_TEST_DIALER_MODE", mode)],
        );
        let rust = support_go::rust_outcomes(
            &suite,
            &Dialer {
                mode,
                rows: RefCell::default(),
                minted: Cell::new(0),
            },
        );
        assert!(
            !go.outcomes.is_empty(),
            "{mode}: the Go run published no verdicts:\n{}",
            go.log
        );
        assert_eq!(
            go.outcomes, rust,
            "{mode}: per-scenario verdicts\n{}",
            go.log
        );
        let failed: Vec<_> = rust
            .iter()
            .filter(|(_, status)| *status != "passed")
            .map(|(id, _)| id.as_str())
            .collect();
        if mode == "correct" {
            assert!(failed.is_empty(), "{failed:?}");
            assert!(go.success, "{}", go.log);
        } else {
            assert_eq!(failed, [SET_LEAD], "{mode}");
            assert!(!go.success, "{mode}: {}", go.log);
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn issue_188_go_runs_the_dotted_leaf_coverage_input_27() {
    let input = ess_conformance::coverage_build::build(
        &ir(DIALER),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let selected = input.selected();
    assert_eq!(
        selected.suite().provenance.suite_version.to_string(),
        "ess-conformance/27"
    );
    let directory = support_go::package_input(
        "dialer-coverage",
        &input,
        &[(
            "dialer_test.go",
            include_str!("fixtures/dialer-runtime-go.go"),
        )],
    );
    for mode in MODES {
        let go = support_go::go_test(
            &directory,
            "TestDialerRuntime",
            &[("ESS_TEST_DIALER_MODE", mode)],
        );
        let rust = support_go::rust_outcomes_admitted(
            selected,
            &Dialer {
                mode,
                rows: RefCell::default(),
                minted: Cell::new(0),
            },
        );
        assert!(!go.outcomes.is_empty(), "{mode}: {}", go.log);
        assert_eq!(
            go.outcomes, rust,
            "{mode}: per-scenario verdicts\n{}",
            go.log
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

/// The same comparison through the recorded transcript of the Rust target, which also requires the
/// Go runtime to send every request the reference runner sent.
#[test]
fn issue_188_go_matches_the_reference_runner_over_the_recorded_dialer() {
    let suite = suite(DIALER);
    for mode in MODES {
        let verdicts = support_go::assert_parity(
            &format!("dialer-{mode}"),
            &suite,
            Dialer {
                mode,
                rows: RefCell::default(),
                minted: Cell::new(0),
            },
        );
        let failed = verdicts
            .values()
            .filter(|status| *status != "passed")
            .count();
        assert_eq!(
            failed,
            usize::from(mode != "correct"),
            "{mode}: {verdicts:?}"
        );
    }
}

/// The two refusals `ess_conformance::leaf_payloads::admit_format` makes, made by the Go runtime on
/// the document itself: a dotted key under a major older than /26, and a dotted key that names no
/// leaf of the step's own shape. Each refuses the suite before any target is reached.
#[test]
fn go_refuses_a_leaf_path_below_suite_26_and_one_naming_no_leaf_of_its_shape() {
    let suite = suite(DIALER);
    let directory = support_go::package(
        "dialer-refusals",
        &suite,
        &[(
            "dialer_test.go",
            include_str!("fixtures/dialer-runtime-go.go"),
        )],
    );
    support_go::rewrite_suite(&directory, |document| {
        document["provenance"]["suite_version"] = "ess-conformance/24".into();
    });
    let go = support_go::go_test(&directory, "TestDialerRuntime", &[]);
    assert!(
        go.log
            .contains("per-leaf struct values require suite/26 or /27"),
        "{}",
        go.log
    );
    assert!(go.outcomes.is_empty() && !go.success);

    support_go::rewrite_suite(&directory, |document| {
        document["provenance"]["suite_version"] = "ess-conformance/26".into();
        let steps = document["scenarios"][SET_LEAD]["steps"]
            .as_array_mut()
            .unwrap();
        let payload = steps
            .iter_mut()
            .find(|step| step["step"] == "expect_event" && step["event"] == "demo.dialer.LeadSet")
            .unwrap()["payload"]
            .as_object_mut()
            .unwrap();
        let value = payload.remove("lead.number").unwrap();
        payload.insert("lead.numbr".to_owned(), value);
    });
    let go = support_go::go_test(&directory, "TestDialerRuntime", &[]);
    assert!(
        go.log
            .contains("`lead.numbr` names no leaf of the step's own shape"),
        "{}",
        go.log
    );
    assert!(go.outcomes.is_empty() && !go.success);
    std::fs::remove_dir_all(directory).unwrap();
}
