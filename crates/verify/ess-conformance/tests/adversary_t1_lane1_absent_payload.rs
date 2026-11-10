//! Adversary, lane t1 commit 1 (beyond10x/ess#765): an absent payload read is claimed absent in
//! either spelling, not as `null`.
//!
//! The unit's own test reads the suite bytes only. These cases drive the acceptance through the
//! interpreted target and through targets that publish a present value where the arrangement
//! holds the read absent, on an event and on an expected error, and on a nested struct member.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::occurrence_clock::CommandClock;
use ess_conformance::report::Status;
use ess_conformance::runner::Runner;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceStatus, ConformanceSuite, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn token_model(agent_presence: &str) -> String {
    format!(
        "format: ess/25
system: example
version: v1
domain: example.token
types:
  - {{name: example.token.TokenId, kind: newtype, of: String}}
  - {{name: example.token.AgentName, kind: newtype, of: String}}
entities:
  - name: example.token.PoolToken
    identity: {{name: token_id, type: example.token.TokenId}}
    fields:
      - {{name: owner, type: String}}
      - {{name: agent, type: 'Optional<example.token.AgentName>'}}
    lifecycle:
      initial: Contributed
      states: [Contributed, Withdrawn]
      terminal: [Withdrawn]
      transitions:
        - {{name: withdraw, from: [Contributed], to: Withdrawn}}
events:
  - name: example.token.TokenContributed
    fields:
      - {{name: token_id, type: example.token.TokenId}}
  - name: example.token.TokenWithdrawn
    fields:
      - {{name: token_id, type: example.token.TokenId}}
      - {{name: owner, type: String}}
      - {{name: agent, type: 'Optional<example.token.AgentName>'{agent_presence}}}
commands:
  - name: example.token.ContributeToken
    input:
      - {{name: owner, type: String}}
    outcomes:
      - name: contributed
        creates: example.token.PoolToken
        instance: token_id
        sets: {{owner: input.owner, agent: {{cleared: true}}}}
        emits: [example.token.TokenContributed]
        payload:
          example.token.TokenContributed:
            token_id: {{generated: true}}
  - name: example.token.WithdrawToken
    input:
      - {{name: token_id, type: example.token.TokenId}}
    outcomes:
      - name: withdrawn
        moves: example.token.PoolToken.withdraw
        instance: token_id
        sets: {{agent: {{cleared: true}}}}
        emits: [example.token.TokenWithdrawn]
        payload:
          example.token.TokenWithdrawn:
            token_id: input.token_id
            owner: {{subject: owner}}
            agent: {{subject: agent}}
"
    )
}

/// The token model with a refusal guarded by the stored row whose error reads the cleared `agent`.
fn blocked_model() -> String {
    let text = token_model("");
    let text = text.replacen(
        "events:\n",
        "errors:
  - name: example.token.Blocked
    summary: The token is blocked.
    fields:
      - {name: owner, type: String}
      - {name: agent, type: 'Optional<example.token.AgentName>'}
events:
",
        1,
    );
    let out = text.replace(
        "      - name: withdrawn\n",
        "      - name: blocked
        when_subject: {predicate: owner == \"blocked\"}
        error: example.token.Blocked
        payload:
          example.token.Blocked:
            owner: {subject: owner}
            agent: {subject: agent}
      - name: withdrawn\n",
    );
    assert_ne!(out, text);
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn suite(model: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(model);
    synthesis.suite
}

/// Failed check codes by scenario, and the overall status.
fn run<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> (ConformanceStatus, BTreeMap<String, BTreeSet<String>>) {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report();
    let failed = report
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            (
                result.scenario.to_string(),
                result
                    .checks
                    .iter()
                    .filter(|check| check.status != Status::Passed)
                    .map(|check| format!("{} {check:?}", check.code))
                    .chain(std::iter::once(format!(
                        "{:?}",
                        result.diagnostics().collect::<Vec<_>>()
                    )))
                    .collect(),
            )
        })
        .collect();
    (report.status, failed)
}

/// The interpreted target with one published value rewritten: `event.field` on an event, or
/// `error.field` on a reported error. `None` removes the field.
struct Rewrites {
    inner: Interpreted,
    on: &'static str,
    field: &'static str,
    value: Option<Node>,
}

impl Rewrites {
    fn new(model: EssIr, on: &'static str, field: &'static str, value: Option<Node>) -> Self {
        Self {
            inner: Interpreted::for_model(model),
            on,
            field,
            value,
        }
    }

    fn rewrite(&self, fields: &mut BTreeMap<String, Node>) {
        match &self.value {
            Some(value) => {
                fields.insert(self.field.to_owned(), value.clone());
            }
            None => {
                fields.remove(self.field);
            }
        }
    }

    fn answer(&self, mut answer: SemanticCommandResult) -> SemanticCommandResult {
        if let Some(error) = answer.error.as_mut() {
            if error.error.to_string() == self.on {
                self.rewrite(&mut error.fields);
            }
        }
        for event in &mut answer.direct_events {
            if event.event.to_string() == self.on {
                self.rewrite(&mut event.payload);
            }
        }
        answer
    }
}

impl ConformanceTarget for Rewrites {
    fn take_command_clock(&mut self, clock: Box<dyn CommandClock + Send>) -> bool {
        self.inner.take_command_clock(clock)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn fixture_values(
        &self,
        scenario: &ScenarioContext,
        contract: &ess_conformance::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        self.inner.fixture_values(scenario, contract)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner
            .execute_command(request)
            .map(|answer| self.answer(answer))
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        let mut completion = self.inner.execute_command_recorded(request);
        completion.answer = completion.answer.map(|answer| self.answer(answer));
        completion
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let mut events = self.inner.observe_events(request)?;
        for event in &mut events {
            if event.event.to_string() == self.on {
                self.rewrite(&mut event.payload);
            }
        }
        Ok(events)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(r)
    }
}

const WITHDRAWN_EVENT: &str = "example.token.TokenWithdrawn";
const WITHDRAWN: &str = "example.token.WithdrawToken/outcome/withdrawn";
const BLOCKED: &str = "example.token.WithdrawToken/outcome/blocked";

/// Acceptance 1: the reproduction passes on the interpreted target, with and without
/// `presence: omitted_when_absent` on `TokenWithdrawn.agent`.
#[test]
fn adversary_t1_lane1_the_reproduction_passes_on_the_interpreted_target() {
    for presence in ["", ", presence: omitted_when_absent"] {
        let model = ir(&token_model(presence));
        let suite = suite(&model);
        let (status, failed) = run(&suite, &Interpreted::for_model(model));
        assert_eq!(
            status,
            ConformanceStatus::Passed,
            "presence `{presence}`: {failed:#?}"
        );
    }
}

/// Acceptance 3: a target publishing `agent` present fails the withdrawn scenario with
/// `ESS-CF-PAYLOAD`; one publishing `null` passes when the field declares no presence.
#[test]
fn adversary_t1_lane1_a_present_agent_fails_and_a_null_agent_passes() {
    let model = ir(&token_model(""));
    let suite = suite(&model);
    let present = Rewrites::new(
        model.clone(),
        WITHDRAWN_EVENT,
        "agent",
        Some(Node::Text("rogue".to_owned())),
    );
    let (_, failed) = run(&suite, &present);
    assert!(
        failed
            .get(WITHDRAWN)
            .is_some_and(|codes| codes.iter().any(|code| code.starts_with("ESS-CF-PAYLOAD"))),
        "a present agent must fail {WITHDRAWN} with ESS-CF-PAYLOAD: {failed:#?}"
    );
    let null = Rewrites::new(model, WITHDRAWN_EVENT, "agent", Some(Node::Null));
    let (status, failed) = run(&suite, &null);
    assert_eq!(status, ConformanceStatus::Passed, "{failed:#?}");
}

/// Beside the acceptance, the story's decision says "Error payloads are untouched". Before this
/// commit an expected error's `{subject: agent}` read of a field the arrangement cleared (`blocked`, guarded by the
/// stored row) was claimed `null` (which `expect_error` compares as absent-or-null). `determined_fields` now
/// drops every evaluated `null`, and an error has no shape leaf to carry the claim instead.
#[test]
fn adversary_t1_lane1_an_error_field_read_absent_is_still_claimed() {
    let model = ir(&blocked_model());
    let suite = suite(&model);
    let claims: Vec<_> = suite
        .scenarios
        .iter()
        .filter(|(id, _)| id.to_string() == BLOCKED)
        .flat_map(|(_, scenario)| &scenario.steps)
        .filter_map(|step| match step {
            ScenarioStep::ExpectError { error, fields }
                if error.to_string() == "example.token.Blocked" =>
            {
                Some(fields.clone())
            }
            _ => None,
        })
        .collect();
    assert!(
        !claims.is_empty(),
        "the suite expects Blocked in {BLOCKED}: {:#?}",
        suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    for fields in &claims {
        assert_eq!(
            fields.get("owner"),
            Some(&Node::Text("blocked".to_owned())),
            "the row's value is claimed: {fields:#?}"
        );
        assert_eq!(
            fields.get("agent"),
            Some(&Node::Null),
            "the cleared agent is claimed absent: {fields:#?}"
        );
    }
}

/// The behaviour behind the case above: a target reporting a present `agent` on the error, where
/// the row holds it absent, must fail the suite.
#[test]
fn adversary_t1_lane1_a_present_error_field_where_the_row_holds_it_absent_fails() {
    let model = ir(&blocked_model());
    let suite = suite(&model);
    let good = run(&suite, &Interpreted::for_model(model.clone()));
    assert_eq!(good.0, ConformanceStatus::Passed, "{:#?}", good.1);
    let rogue = Rewrites::new(
        model,
        "example.token.Blocked",
        "agent",
        Some(Node::Text("rogue".to_owned())),
    );
    let (status, failed) = run(&suite, &rogue);
    assert_ne!(
        status,
        ConformanceStatus::Passed,
        "a target reporting `agent: rogue` on Blocked passes: {failed:#?}"
    );
}
