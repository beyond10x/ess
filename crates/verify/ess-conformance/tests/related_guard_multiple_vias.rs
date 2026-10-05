//! A command guarded by two related rows (beyond10x/ess#283, `ess/22`): the declared interpreter
//! answers every missing/predicate/precedence case by one order — missing rows first, in the
//! declaration order of their `exists: false` branches, then the present-related predicate refusals
//! in declaration order, then acceptance — and synthesis witnesses each branch with the other row
//! arranged to pass, plus the overlaps that order decides, so a target checking the rows in another
//! order, or ignoring either row, fails.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner};
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/related-guard-multiple.yaml");
const START: &str = "demo.run.StartRun";
const STOP: &str = "demo.run.StopRun";

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

const NOBODY: &str = "00000000-0000-4000-8000-ffffffffffff";

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn start_outcomes(body: &str) -> String {
    replaced(
        MODEL,
        &format!(
            "{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{STARTED_BRANCH}"
        ),
        body,
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("multiple-related.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

// ---- interpreter -------------------------------------------------------------------------------

/// Runs `command` once on `store`: the outcome's name and the first identity its events carry.
fn step(
    model: &EssIr,
    store: &Store,
    command: &str,
    input: &BTreeMap<String, Node>,
) -> (String, String, Store) {
    let mut steps = execute(
        model,
        store,
        &command.parse::<QualifiedName>().unwrap(),
        input,
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{command} is interpreted: {error}"));
    assert_eq!(steps.len(), 1, "{command}: one answer");
    let step = steps.remove(0);
    let identity = step
        .events
        .iter()
        .flat_map(|event| event.payload.values())
        .find_map(Node::as_text)
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    let outcome = step.outcome.as_ref().map_or_else(
        || "none".to_owned(),
        |outcome| {
            outcome
                .to_string()
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .to_owned()
        },
    );
    (outcome, identity, step.next)
}

/// The rows one request reads: a switch in `switch` (`None` for an identity no row carries) and a
/// capability in `capability`, and the outcome `StartRun` answers.
fn start(model: &EssIr, switch: Option<&str>, capability: Option<&str>) -> String {
    let none = BTreeMap::new();
    let mut store = Store::default();
    let mut switch_id = NOBODY.to_owned();
    if let Some(state) = switch {
        let (_, id, next) = step(model, &store, "demo.run.InstallSwitch", &none);
        store = next;
        if state == "Paused" {
            let input = BTreeMap::from([("switch_id".to_owned(), Node::Text(id.clone()))]);
            store = step(model, &store, "demo.run.PauseSwitch", &input).2;
        }
        switch_id = id;
    }
    let mut capability_id = NOBODY.to_owned();
    if let Some(state) = capability {
        let (_, id, next) = step(model, &store, "demo.run.GrantCapability", &none);
        store = next;
        if state == "Revoked" {
            let input = BTreeMap::from([("capability_id".to_owned(), Node::Text(id.clone()))]);
            store = step(model, &store, "demo.run.RevokeCapability", &input).2;
        }
        capability_id = id;
    }
    let input = BTreeMap::from([
        ("switch".to_owned(), Node::Text(switch_id)),
        ("capability".to_owned(), Node::Text(capability_id)),
    ]);
    step(model, &store, START, &input).0
}

#[test]
fn issue_283_the_interpreter_answers_every_missing_predicate_and_precedence_case() {
    let model = ir(MODEL);
    let cases: [(Option<&str>, Option<&str>, &str); 9] = [
        (Some("Active"), Some("Granted"), "started"),
        (None, Some("Granted"), "no-such-switch"),
        (Some("Active"), None, "no-such-capability"),
        // Both rows missing: the first declared `exists: false` answers.
        (None, None, "no-such-switch"),
        // Both rows refusing: the first declared present-related refusal answers.
        (Some("Paused"), Some("Revoked"), "switch-paused"),
        // A missing row answers before a present row's predicate refusal, whatever the order.
        (Some("Paused"), None, "no-such-capability"),
        (None, Some("Revoked"), "no-such-switch"),
        (Some("Active"), Some("Revoked"), "capability-revoked"),
        (Some("Paused"), Some("Granted"), "switch-paused"),
    ];
    for (switch, capability, expected) in cases {
        assert_eq!(
            start(&model, switch, capability),
            expected,
            "switch {switch:?}, capability {capability:?}"
        );
    }
}

#[test]
fn issue_283_declaration_order_decides_across_rows_not_between_kinds() {
    // The capability's branches declared first: its answers come first among the missing rows and
    // among the refusals, and a missing row still answers before any predicate refusal.
    let model = ir(&start_outcomes(&format!(
        "{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{STARTED_BRANCH}"
    )));
    assert_eq!(start(&model, None, None), "no-such-capability");
    assert_eq!(
        start(&model, Some("Paused"), Some("Revoked")),
        "capability-revoked"
    );
    assert_eq!(start(&model, None, Some("Revoked")), "no-such-switch");
}

// ---- synthesis ---------------------------------------------------------------------------------

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

fn scenario<'s>(result: &'s Synthesis, id: &str) -> &'s ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("{id} exists; refusals: {:#?}", result.refusals),
            |(_, scenario)| scenario,
        )
}

fn statuses<T: ConformanceTarget>(result: &Synthesis, target: &T) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| {
            let id = run.scenario.to_string();
            id.contains("StartRun") || id.contains("StopRun")
        })
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn honest(text: &str) -> (EssIr, Synthesis) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    for command in [START, STOP] {
        let refused = refusals_for(&result, command);
        assert_eq!(refused.len(), 0, "{command}: {refused:#?}");
    }
    let passed = statuses(&result, &Interpreted::for_model(model.clone()));
    for id in [
        NO_SWITCH,
        SWITCH_PAUSED,
        NO_CAPABILITY,
        CAPABILITY_REVOKED,
        STARTED,
    ] {
        assert_eq!(
            passed.get(id),
            Some(&Status::Passed),
            "{id} executes and passes: {passed:#?}"
        );
    }
    assert!(
        passed
            .iter()
            .any(|(id, status)| id.contains("StopRun") && *status == Status::Passed),
        "a run started through both rows is stopped: {passed:#?}"
    );
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "the declared interpreter passes every scenario: {passed:#?}"
    );
    (model, result)
}

fn instance(input: &BTreeMap<String, ess_conformance::ScenarioValue>, field: &str) -> bool {
    matches!(
        input.get(field),
        Some(ess_conformance::ScenarioValue::Instance { .. })
    )
}

#[test]
fn issue_283_each_branch_is_witnessed_with_the_other_row_arranged() {
    let (_, result) = honest(MODEL);
    for id in [
        NO_SWITCH,
        SWITCH_PAUSED,
        NO_CAPABILITY,
        CAPABILITY_REVOKED,
        STARTED,
    ] {
        let own = invocations(scenario(&result, id));
        assert!(
            own.iter().any(|(_, outcome)| outcome == id),
            "{id} is sent: {own:#?}"
        );
    }
    // The missing switch is sent beside a capability row, and the missing capability beside a
    // switch row: the other row is there, so it is not what answers.
    let missing_switch = invocations(scenario(&result, NO_SWITCH));
    assert!(
        missing_switch
            .iter()
            .any(|(input, outcome)| outcome == NO_SWITCH
                && !instance(input, "switch")
                && instance(input, "capability")),
        "{missing_switch:#?}"
    );
    let missing_capability = invocations(scenario(&result, NO_CAPABILITY));
    assert!(
        missing_capability
            .iter()
            .any(|(input, outcome)| outcome == NO_CAPABILITY
                && instance(input, "switch")
                && !instance(input, "capability")),
        "{missing_capability:#?}"
    );
    for id in [SWITCH_PAUSED, CAPABILITY_REVOKED, STARTED] {
        let sent = invocations(scenario(&result, id));
        assert!(
            sent.iter().any(|(input, outcome)| outcome == id
                && instance(input, "switch")
                && instance(input, "capability")),
            "{id} names both arranged rows: {sent:#?}"
        );
    }
}

#[test]
fn issue_283_the_overlaps_the_precedence_decides_are_sent() {
    let (_, result) = honest(MODEL);
    // Both rows missing: the first declared `exists: false` answers.
    let missing_switch = invocations(scenario(&result, NO_SWITCH));
    assert!(
        missing_switch
            .iter()
            .any(|(input, outcome)| outcome == NO_SWITCH
                && !instance(input, "switch")
                && !instance(input, "capability")),
        "no-such-switch is sent with the capability missing too: {missing_switch:#?}"
    );
    // A missing capability beside a paused switch: missing rows before predicate refusals.
    let missing_capability = invocations(scenario(&result, NO_CAPABILITY));
    assert!(
        missing_capability
            .iter()
            .filter(|(input, outcome)| outcome == NO_CAPABILITY && instance(input, "switch"))
            .count()
            >= 2,
        "no-such-capability is sent beside a passing and a refusing switch: {missing_capability:#?}"
    );
    // Both rows refusing: the first declared refusal answers.
    let paused = invocations(scenario(&result, SWITCH_PAUSED));
    assert!(
        paused
            .iter()
            .filter(|(input, outcome)| outcome == SWITCH_PAUSED
                && instance(input, "switch")
                && instance(input, "capability"))
            .count()
            >= 2,
        "switch-paused is sent beside a passing and a revoked capability: {paused:#?}"
    );
}

/// A target interpreting another model than the one the suite was synthesized from.
fn mutant(result: &Synthesis, text: &str) -> BTreeMap<String, Status> {
    statuses(result, &Interpreted::for_model(ir(text)))
}

fn fails(statuses: &BTreeMap<String, Status>, id: &str) -> bool {
    statuses
        .get(id)
        .is_some_and(|status| *status != Status::Passed)
}

#[test]
fn issue_283_a_target_checking_the_rows_in_another_order_fails() {
    let (_, result) = honest(MODEL);
    let reversed = mutant(
        &result,
        &start_outcomes(&format!(
            "{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{STARTED_BRANCH}"
        )),
    );
    assert!(fails(&reversed, NO_SWITCH), "{reversed:#?}");
    assert!(fails(&reversed, SWITCH_PAUSED), "{reversed:#?}");
}

#[test]
fn issue_283_a_target_ignoring_either_row_fails() {
    let (_, result) = honest(MODEL);
    let no_switch = mutant(
        &result,
        &start_outcomes(&format!(
            "{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{STARTED_BRANCH}"
        )),
    );
    assert!(fails(&no_switch, NO_SWITCH), "{no_switch:#?}");
    assert!(fails(&no_switch, SWITCH_PAUSED), "{no_switch:#?}");
    let no_capability = mutant(
        &result,
        &start_outcomes(&format!(
            "{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{STARTED_BRANCH}"
        )),
    );
    assert!(fails(&no_capability, NO_CAPABILITY), "{no_capability:#?}");
    assert!(
        fails(&no_capability, CAPABILITY_REVOKED),
        "{no_capability:#?}"
    );
}

#[test]
fn issue_283_a_target_reading_only_whether_a_switch_exists_fails() {
    let (_, result) = honest(MODEL);
    let unguarded = mutant(
        &result,
        &start_outcomes(&format!(
            "{NO_SWITCH_BRANCH}{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{STARTED_BRANCH}"
        )),
    );
    assert!(fails(&unguarded, SWITCH_PAUSED), "{unguarded:#?}");
}

// ---- an Optional second row --------------------------------------------------------------------

fn optional_capability() -> String {
    replaced(
        MODEL,
        "      - {name: capability, type: demo.run.CapabilityId}\n",
        "      - {name: capability, type: Optional<demo.run.CapabilityId>}\n",
    )
}

#[test]
fn issue_283_an_absent_optional_row_reads_nothing_beside_the_other_row() {
    let model = ir(&optional_capability());
    let none = BTreeMap::new();
    let (_, switch, store) = step(&model, &Store::default(), "demo.run.InstallSwitch", &none);
    let input = BTreeMap::from([("switch".to_owned(), Node::Text(switch.clone()))]);
    assert_eq!(step(&model, &store, START, &input).0, "started");
    let missing = BTreeMap::from([("switch".to_owned(), Node::Text(NOBODY.to_owned()))]);
    assert_eq!(step(&model, &store, START, &missing).0, "no-such-switch");
}

#[test]
fn issue_283_an_optional_second_row_is_witnessed_absent_present_and_missing() {
    let text = optional_capability();
    let (_, result) = honest(&text);
    let sent: Vec<_> = result
        .suite
        .scenarios
        .values()
        .flat_map(invocations)
        .collect();
    assert!(
        sent.iter().any(|(input, outcome)| outcome == STARTED
            && instance(input, "switch")
            && !input.contains_key("capability")),
        "an absent capability beside a passing switch starts the run: {sent:#?}"
    );
    // A target reading an absent capability as a missing one answers `no-such-capability`.
    let absent_as_missing = statuses(&result, &AbsentAsMissing(Interpreted::for_model(ir(&text))));
    assert!(fails(&absent_as_missing, STARTED), "{absent_as_missing:#?}");
}

/// A target that reads an omitted capability as an identity no row carries.
struct AbsentAsMissing(Interpreted);

impl ConformanceTarget for AbsentAsMissing {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(context)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() == START && !request.input.contains_key("capability") {
            request
                .input
                .insert("capability".to_owned(), Node::Text(NOBODY.to_owned()));
        }
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
}

// ---- stored fields, a disjunction and a copied value ----------------------------------------------

/// The fixture with a stored flag on the switch read by a disjunction, and a scope on the
/// capability the run copies: each row's predicate boundaries and the `{related: …}` copy are
/// arranged around their own row, with the other row beside it.
fn with_fields() -> String {
    let text = replaced(
        MODEL,
        "    identity: {name: switch_id, type: demo.run.SwitchId}\n    fields: []\n",
        "    identity: {name: switch_id, type: demo.run.SwitchId}\n    fields:\n      - {name: locked, type: Boolean}\n",
    );
    let text = replaced(
        &text,
        "    identity: {name: capability_id, type: demo.run.CapabilityId}\n    fields: []\n",
        "    identity: {name: capability_id, type: demo.run.CapabilityId}\n    fields:\n      - {name: scope, type: String}\n",
    );
    let text = replaced(
        &text,
        "    identity: {name: run_id, type: demo.run.RunId}\n    fields: []\n",
        "    identity: {name: run_id, type: demo.run.RunId}\n    fields:\n      - {name: scope, type: String}\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.run.InstallSwitch\n    input: []\n",
        "  - name: demo.run.InstallSwitch\n    input:\n      - {name: locked, type: Boolean}\n",
    );
    let text = replaced(
        &text,
        "        payload: {demo.run.SwitchInstalled: {switch_id: {generated: true}}}\n",
        "        payload: {demo.run.SwitchInstalled: {switch_id: {generated: true}}}\n        sets: {locked: input.locked}\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.run.GrantCapability\n    input: []\n",
        "  - name: demo.run.GrantCapability\n    input:\n      - {name: scope, type: String}\n",
    );
    let text = replaced(
        &text,
        "        payload: {demo.run.CapabilityGranted: {capability_id: {generated: true}}}\n",
        "        payload: {demo.run.CapabilityGranted: {capability_id: {generated: true}}}\n        sets: {scope: input.scope}\n",
    );
    let text = replaced(
        &text,
        "        when_related: {via: input.switch, predicate: state == Paused}\n",
        "        when_related: {via: input.switch, predicate: {any: [state == Paused, locked == true]}}\n",
    );
    replaced(
        &text,
        "        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n",
        "        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n        sets: {scope: {related: {via: input.capability, field: scope}}}\n",
    )
}

#[test]
fn issue_283_stored_fields_a_disjunction_and_a_copied_value_beside_a_second_row() {
    let text = with_fields();
    let (_, result) = honest(&text);
    // The disjunction is witnessed once more per disjunct on a switch isolating it, each beside a
    // capability arranged to pass.
    let paused = invocations(scenario(&result, SWITCH_PAUSED));
    assert!(
        paused
            .iter()
            .filter(|(input, outcome)| outcome == SWITCH_PAUSED
                && instance(input, "switch")
                && instance(input, "capability"))
            .count()
            >= 3,
        "the refusal, and each disjunct alone, beside a capability: {paused:#?}"
    );
    // A target reading the capability before the switch answers a request missing both otherwise.
    let capability_first = replaced(
        &replaced(&text, NO_CAPABILITY_BRANCH, ""),
        NO_SWITCH_BRANCH,
        &format!("{NO_CAPABILITY_BRANCH}{NO_SWITCH_BRANCH}"),
    );
    let reversed = mutant(&result, &capability_first);
    assert!(fails(&reversed, NO_SWITCH), "{reversed:#?}");
}

// ---- a moving command beside `wrong_state:` --------------------------------------------------------

/// `StopRun` guarded by both rows too, beside its `wrong_state:` refusal: the addressed run's held
/// state answers before either row's predicate, and a missing row before the held state.
fn guarded_stop() -> String {
    replaced(
        MODEL,
        "  - name: demo.run.StopRun\n    input:\n      - {name: run_id, type: demo.run.RunId}\n    outcomes:\n",
        &format!(
            "  - name: demo.run.StopRun\n    input:\n      - {{name: run_id, type: demo.run.RunId}}\n      - {{name: switch, type: demo.run.SwitchId}}\n      - {{name: capability, type: demo.run.CapabilityId}}\n    outcomes:\n{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}"
        ),
    )
}

#[test]
fn issue_283_a_moving_command_beside_wrong_state_reads_both_rows() {
    let text = guarded_stop();
    let model = ir(&text);
    let result = ess_conformance::synthesize::synthesize(&model);
    // Nothing is refused that the one-row twin — the switch's guards alone — does not refuse too.
    let twin = replaced(
        &text,
        &format!("{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}      - {{name: wrong-state"),
        "      - {name: wrong-state",
    );
    let twin_refused = refusals_for(&ess_conformance::synthesize::synthesize(&ir(&twin)), STOP);
    let refused: Vec<String> = refusals_for(&result, STOP)
        .into_iter()
        .filter(|refusal| !twin_refused.contains(refusal))
        .collect();
    assert_eq!(refused.len(), 0, "{refused:#?}\ntwin: {twin_refused:#?}");
    let passed = statuses(&result, &Interpreted::for_model(model));
    assert!(
        passed
            .iter()
            .filter(|(id, _)| id.contains("StopRun"))
            .all(|(_, status)| *status == Status::Passed),
        "{passed:#?}"
    );
    for branch in [
        "no-such-switch",
        "switch-paused",
        "no-such-capability",
        "capability-revoked",
        "stopped",
    ] {
        let id = format!("{STOP}/outcome/{branch}");
        assert_eq!(passed.get(&id), Some(&Status::Passed), "{id}: {passed:#?}");
    }
    // The held state is witnessed against both rows present: a missing one answers before it.
    let id = format!("demo.run.Run/state/Stopped/refuses/{STOP}");
    assert_eq!(passed.get(&id), Some(&Status::Passed), "{id}: {passed:#?}");
    let capability_first = replaced(
        &replaced(&text, &format!("{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}      - {{name: wrong-state"), "      - {name: wrong-state"),
        "      - {name: wrong-state",
        &format!("{NO_CAPABILITY_BRANCH}{CAPABILITY_REVOKED_BRANCH}{NO_SWITCH_BRANCH}{SWITCH_PAUSED_BRANCH}      - {{name: wrong-state"),
    );
    let reversed = mutant(&result, &capability_first);
    assert!(
        fails(&reversed, &format!("{STOP}/outcome/no-such-switch"))
            || fails(&reversed, &format!("{STOP}/outcome/switch-paused")),
        "{reversed:#?}"
    );
}
