//! Adversary pass 1 for beyond10x/ess#283 (several related rows read through the input, ess/22):
//! admitted models the synthesized suite must witness, and faulty targets it must catch.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/related-guard-multiple.yaml");
const START: &str = "demo.run.StartRun";

const NO_SWITCH: &str = "demo.run.StartRun/outcome/no-such-switch";
const SWITCH_PAUSED: &str = "demo.run.StartRun/outcome/switch-paused";
const NO_CAPABILITY: &str = "demo.run.StartRun/outcome/no-such-capability";
const CAPABILITY_REVOKED: &str = "demo.run.StartRun/outcome/capability-revoked";
const STARTED: &str = "demo.run.StartRun/outcome/started";

const NO_SWITCH_BRANCH: &str = "      - name: no-such-switch\n        when_related: {via: input.switch, exists: false}\n        error: demo.run.NoSuchSwitch\n";
const SWITCH_PAUSED_BRANCH: &str = "      - name: switch-paused\n        when_related: {via: input.switch, predicate: state == Paused}\n        error: demo.run.SwitchIsPaused\n";
const NO_CAPABILITY_BRANCH: &str = "      - name: no-such-capability\n        when_related: {via: input.capability, exists: false}\n        error: demo.run.NoSuchCapability\n";
const CAPABILITY_REVOKED_BRANCH: &str = "      - name: capability-revoked\n        when_related: {via: input.capability, predicate: state == Revoked}\n        error: demo.run.CapabilityIsRevoked\n";
const STARTED_BRANCH: &str = "      - name: started\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n";
const START_INPUT: &str = "  - name: demo.run.StartRun\n    input:\n      - {name: switch, type: demo.run.SwitchId}\n      - {name: capability, type: demo.run.CapabilityId}\n";

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn start_outcomes(text: &str, body: &str) -> String {
    replaced(
        text,
        &format!(
            "{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{STARTED_BRANCH}"
        ),
        body,
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("multiple-related.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals_for(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| {
            format!(
                "{} [{}]: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(|| format!("{:?}", refusal.subject), ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn statuses<T: ConformanceTarget>(result: &Synthesis, target: &T) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains("StartRun"))
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

/// Every invocation of `StartRun` in a scenario, paired with the outcome asserted for it.
fn invocations(
    scenario: &ConformanceScenario,
) -> Vec<(BTreeMap<String, ess_conformance::ScenarioValue>, String)> {
    let mut pending = None;
    let mut found = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } if command.to_string() == START => {
                pending = Some(input.clone());
            }
            ScenarioStep::ExpectOutcome { outcome } => {
                if let Some(input) = pending.take() {
                    let id = ess_conformance::scenario::ScenarioId::Outcome {
                        outcome: outcome.clone(),
                    };
                    found.push((input, id.to_string()));
                }
            }
            ScenarioStep::ExecuteCommand { .. } => pending = None,
            _ => {}
        }
    }
    found
}

fn instance(input: &BTreeMap<String, ess_conformance::ScenarioValue>, field: &str) -> bool {
    matches!(
        input.get(field),
        Some(ess_conformance::ScenarioValue::Instance { .. })
    )
}

/// The suite synthesized from `text`, with every `StartRun` branch named in `ids` witnessed and
/// passing against the declared interpreter, and no `StartRun` refusal.
fn witnessed(text: &str, ids: &[&str]) -> Synthesis {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refusals_for(&result, START);
    let passed = statuses(&result, &Interpreted::for_model(model));
    let missing: Vec<&&str> = ids
        .iter()
        .filter(|id| passed.get(**id) != Some(&Status::Passed))
        .collect();
    assert!(
        refused.is_empty() && missing.is_empty(),
        "an admitted model: every branch is witnessed and passes.\nnot witnessed or not \
         passing: {missing:?}\nrefusals: {refused:#?}\nstatuses: {passed:#?}"
    );
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "the declared interpreter passes every scenario: {passed:#?}"
    );
    result
}

// ---- A1: an absent Optional row beside a missing one ------------------------------------------

/// The capability is an Optional reference whose branches are declared first: on an absent
/// capability and a missing switch the interpreter carries on past the absent reference and
/// answers `no-such-switch` (`related_absent`, `continue`).
fn optional_capability_first() -> String {
    let text = replaced(
        MODEL,
        "      - {name: capability, type: demo.run.CapabilityId}\n",
        "      - {name: capability, type: Optional<demo.run.CapabilityId>}\n",
    );
    start_outcomes(
        &text,
        &format!(
            "{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{STARTED_BRANCH}"
        ),
    )
}

#[test]
fn adversary_283_an_absent_optional_row_beside_a_missing_row_is_sent() {
    let text = optional_capability_first();
    let result = witnessed(
        &text,
        &[
            NO_SWITCH,
            SWITCH_PAUSED,
            NO_CAPABILITY,
            CAPABILITY_REVOKED,
            STARTED,
        ],
    );
    let sent: Vec<_> = result
        .suite
        .scenarios
        .values()
        .flat_map(invocations)
        .collect();
    assert!(
        sent.iter().any(|(input, outcome)| outcome == NO_SWITCH
            && !input.contains_key("capability")
            && !instance(input, "switch")),
        "a missing switch beside an absent capability is answered `no-such-switch`; a target \
         that stops reading rows at an absent reference answers otherwise, and no scenario sends \
         it: {sent:#?}"
    );
}

/// A target that stops reading related rows at the first absent Optional reference — the one-row
/// reader's `return` where several rows need a `continue`. Its answer on an absent capability and
/// a missing switch stands in as `undeclared`: whatever it answers, it is not `no-such-switch`.
struct AbsentEndsTheRead {
    inner: Interpreted,
}

impl ConformanceTarget for AbsentEndsTheRead {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        let skips = command == START && !request.input.contains_key("capability");
        let result = self.inner.execute_command(request)?;
        if skips
            && result
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.to_string().ends_with("no-such-switch"))
        {
            return Ok(SemanticCommandResult::undeclared());
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

#[test]
fn adversary_283_a_target_ending_the_read_at_an_absent_reference_fails() {
    let text = optional_capability_first();
    let model = ir(&text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let faulty = statuses(
        &result,
        &AbsentEndsTheRead {
            inner: Interpreted::for_model(model),
        },
    );
    assert!(
        faulty.values().any(|status| *status != Status::Passed),
        "a target answering a missing switch beside an absent capability with anything but \
         `no-such-switch` fails some scenario: {faulty:#?}"
    );
}

// ---- A2: a row beside the scenario's own on which every row selects a branch ------------------

/// The domain test `issue_283_an_optional_second_row_still_needs_its_absent_case_answered` admits
/// this model as its required twin: the run starts only on a granted capability, with no default.
/// Every capability that exists selects one of its branches — granted `started`, revoked
/// `capability-revoked` — so no capability row "selects none of its branches".
fn started_on_granted() -> String {
    let granted = "      - name: started\n        when_related: {via: input.capability, predicate: state == Granted}\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n";
    replaced(MODEL, STARTED_BRANCH, granted)
}

#[test]
fn adversary_283_switch_branches_are_witnessed_beside_a_capability_that_always_selects() {
    witnessed(
        &started_on_granted(),
        &[
            NO_SWITCH,
            SWITCH_PAUSED,
            NO_CAPABILITY,
            CAPABILITY_REVOKED,
            STARTED,
        ],
    );
}

// ---- A3: two rows of one entity type -----------------------------------------------------------

fn two_switches() -> String {
    let text = replaced(
        MODEL,
        START_INPUT,
        "  - name: demo.run.StartRun\n    input:\n      - {name: primary, type: demo.run.SwitchId}\n      - {name: backup, type: demo.run.SwitchId}\n",
    );
    start_outcomes(
        &text,
        "      - name: no-such-switch\n        when_related: {via: input.primary, exists: false}\n        error: demo.run.NoSuchSwitch\n      - name: switch-paused\n        when_related: {via: input.primary, predicate: state == Paused}\n        error: demo.run.SwitchIsPaused\n      - name: no-such-capability\n        when_related: {via: input.backup, exists: false}\n        error: demo.run.NoSuchCapability\n      - name: capability-revoked\n        when_related: {via: input.backup, predicate: state == Paused}\n        error: demo.run.CapabilityIsRevoked\n      - name: started\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n",
    )
}

#[test]
fn adversary_283_two_rows_of_one_entity_are_witnessed_and_order_is_caught() {
    let text = two_switches();
    let result = witnessed(
        &text,
        &[
            NO_SWITCH,
            SWITCH_PAUSED,
            NO_CAPABILITY,
            CAPABILITY_REVOKED,
            STARTED,
        ],
    );
    // The backup's branches declared first: both missing answers the backup's, both paused too.
    let reversed = text.replace(
        "      - name: no-such-switch\n        when_related: {via: input.primary, exists: false}\n        error: demo.run.NoSuchSwitch\n      - name: switch-paused\n        when_related: {via: input.primary, predicate: state == Paused}\n        error: demo.run.SwitchIsPaused\n",
        "",
    );
    let reversed = replaced(
        &reversed,
        "      - name: started\n",
        "      - name: no-such-switch\n        when_related: {via: input.primary, exists: false}\n        error: demo.run.NoSuchSwitch\n      - name: switch-paused\n        when_related: {via: input.primary, predicate: state == Paused}\n        error: demo.run.SwitchIsPaused\n      - name: started\n",
    );
    let faulty = statuses(&result, &Interpreted::for_model(ir(&reversed)));
    assert!(
        [NO_SWITCH, SWITCH_PAUSED].iter().all(|id| faulty
            .get(*id)
            .is_some_and(|status| *status != Status::Passed)),
        "a target reading the backup before the primary fails both: {faulty:#?}"
    );
}

// ---- A4: a predicate reading the input and the row together ------------------------------------

/// `switch-paused` also refuses every switch in strict mode: the capability's branches are only
/// reachable with `strict: false`, which the capability's own projection never decides.
fn strict_switch() -> String {
    let text = replaced(
        MODEL,
        START_INPUT,
        &format!("{START_INPUT}      - {{name: strict, type: Boolean}}\n"),
    );
    replaced(
        &text,
        "        when_related: {via: input.switch, predicate: state == Paused}\n",
        "        when_related: {via: input.switch, predicate: {any: [state == Paused, input.strict == true]}}\n",
    )
}

#[test]
fn adversary_283_a_predicate_over_the_row_and_the_input_beside_a_second_row() {
    witnessed(
        &strict_switch(),
        &[
            NO_SWITCH,
            SWITCH_PAUSED,
            NO_CAPABILITY,
            CAPABILITY_REVOKED,
            STARTED,
        ],
    );
}

// ---- A5: `existing_instance:` beside two rows ---------------------------------------------------

const DUPLICATE: &str = "demo.run.StartRun/outcome/duplicate";

fn identified_run() -> String {
    let text = replaced(
        MODEL,
        START_INPUT,
        &format!("{START_INPUT}      - {{name: run_id, type: demo.run.RunId}}\n"),
    );
    let text = replaced(
        &text,
        "  - {name: demo.run.RunStateConflict,",
        "  - {name: demo.run.RunExists, summary: The run id is taken., fields: []}\n  - {name: demo.run.RunStateConflict,",
    );
    start_outcomes(
        &text,
        &format!(
            "      - {{name: duplicate, existing_instance: true, error: demo.run.RunExists}}\n{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}      - name: started\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {{demo.run.RunStarted: {{run_id: input.run_id}}}}\n"
        ),
    )
}

#[test]
fn adversary_283_existing_instance_beside_two_rows_is_witnessed() {
    witnessed(
        &identified_run(),
        &[
            DUPLICATE,
            NO_SWITCH,
            SWITCH_PAUSED,
            NO_CAPABILITY,
            CAPABILITY_REVOKED,
            STARTED,
        ],
    );
}

// ---- A6: three rows ------------------------------------------------------------------------------

#[test]
fn adversary_283_three_rows_are_witnessed() {
    let text = replaced(
        MODEL,
        START_INPUT,
        &format!("{START_INPUT}      - {{name: spare, type: demo.run.SwitchId}}\n"),
    );
    let text = replaced(
        &text,
        STARTED_BRANCH,
        &format!(
            "      - name: no-such-spare\n        when_related: {{via: input.spare, exists: false}}\n        error: demo.run.NoSuchSwitch\n      - name: spare-paused\n        when_related: {{via: input.spare, predicate: state == Paused}}\n        error: demo.run.SwitchIsPaused\n{STARTED_BRANCH}"
        ),
    );
    witnessed(
        &text,
        &[
            NO_SWITCH,
            SWITCH_PAUSED,
            NO_CAPABILITY,
            CAPABILITY_REVOKED,
            "demo.run.StartRun/outcome/no-such-spare",
            "demo.run.StartRun/outcome/spare-paused",
            STARTED,
        ],
    );
}
