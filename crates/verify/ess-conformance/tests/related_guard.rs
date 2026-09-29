//! Synthesis witnesses a branch guarded by a row of another entity (`when_related:`, beyond10x/ess#211,
//! `ess/18`): `exists: false` on a scenario where no row carries the identity sent — beside rows of
//! the entity that carry others — and a predicate over the row on both of its sides, each on a row
//! the scenario arranged and pointed the input at.
//!
//! The suite is run against a hand-written target that answers the model, which must pass every
//! scenario of the guarded command, and against mutants of it, each of which must fail at least one.
//! The interpreted target does not evaluate a guard over another entity's row, and says so by name.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCondition, ResolvedRelatedTest};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");

const NO_CONFIGURATION: &str = "demo.signin.InitiateSignIn/outcome/no-configuration";
const NO_REDIRECT: &str = "demo.signin.InitiateSignIn/outcome/no-redirect-entry";
const INITIATED: &str = "demo.signin.InitiateSignIn/outcome/initiated";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("sign-in.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals_about(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
                    result
                        .suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    result
                        .refusals
                        .iter()
                        .map(|refusal| format!("{refusal:?}"))
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Every configuration the scenario creates: the instance it captures and the client it registers.
fn configurations(scenario: &ConformanceScenario) -> Vec<(String, Node)> {
    let mut out = Vec::new();
    let mut client = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.signin.ConfigureTenant" =>
            {
                client = match input.get("redirect_client") {
                    Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                    other => panic!("the client is a literal: {other:?}"),
                };
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "demo.signin.Configuration" => {
                out.push((
                    instance.to_string(),
                    client
                        .take()
                        .expect("a capture follows its ConfigureTenant"),
                ));
            }
            _ => {}
        }
    }
    out
}

/// The input of the last `InitiateSignIn` the scenario sends.
fn initiation(scenario: &ConformanceScenario) -> BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.signin.InitiateSignIn" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the scenario initiates a sign-in")
}

fn requires_outcome(scenario: &ConformanceScenario, outcome: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome: expected }
            if expected.to_string() == outcome)
    })
}

fn requires_error(scenario: &ConformanceScenario, error: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectError { error: expected, .. }
            if expected.to_string() == error)
    })
}

/// The configuration an arranged-row scenario points `tenant` at, and the client it registers,
/// after checking that it sits between two decoys registering other clients.
fn pointed_at(scenario: &ConformanceScenario) -> (String, Node) {
    let rows = configurations(scenario);
    let input = initiation(scenario);
    let Some(ScenarioValue::Instance { instance }) = input.get("tenant") else {
        panic!("tenant names an arranged configuration: {input:?}")
    };
    assert_eq!(
        rows.len(),
        3,
        "a configuration between two decoys: {rows:?}"
    );
    assert_eq!(rows[1].0, instance.to_string(), "the middle one: {rows:?}");
    assert_ne!(
        rows[0].1, rows[1].1,
        "the first decoy registers another client"
    );
    assert_ne!(
        rows[2].1, rows[1].1,
        "the last decoy registers another client"
    );
    rows[1].clone()
}

fn client(input: &BTreeMap<String, ScenarioValue>) -> Node {
    match input.get("client") {
        Some(ScenarioValue::Literal { value }) => value.clone(),
        other => panic!("the client is a literal: {other:?}"),
    }
}

#[test]
fn issue_211_the_compiled_guard_names_the_related_entity() {
    let model = ir(SIGN_IN);
    let command = model
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.signin.InitiateSignIn")
        .unwrap();
    let ResolvedCondition::Related {
        via, entity, test, ..
    } = &command.outcomes[0].condition
    else {
        panic!("a related guard: {:?}", command.outcomes[0].condition)
    };
    assert_eq!(via.field(), "tenant");
    assert_eq!(entity.name().to_string(), "demo.signin.Configuration");
    assert_eq!(*test, ResolvedRelatedTest::Absent);
    let ResolvedCondition::Related { test, .. } = &command.outcomes[1].condition else {
        panic!("a related guard")
    };
    assert!(
        matches!(test, ResolvedRelatedTest::Holds { predicate } if predicate.to_string() == "redirect_client != input.client"),
        "{test:?}"
    );
}

#[test]
fn issue_211_exists_false_is_witnessed_with_no_row_for_the_identity_sent_beside_decoys() {
    let result = synthesis(SIGN_IN);
    assert_eq!(
        refusals_about(&result, "demo.signin.InitiateSignIn"),
        Vec::<String>::new()
    );
    let witness = scenario(&result, NO_CONFIGURATION);
    assert!(requires_outcome(
        witness,
        "demo.signin.InitiateSignIn/no-configuration"
    ));
    assert!(requires_error(witness, "demo.signin.NoConfiguration"));
    let input = initiation(witness);
    assert!(
        matches!(input.get("tenant"), Some(ScenarioValue::Literal { .. })),
        "an identity no arranged row carries is sent as a literal, not a captured one: {input:?}"
    );
    let rows = configurations(witness);
    assert!(
        rows.len() >= 2,
        "rows of the entity exist, carrying other identities: {rows:?}"
    );
}

#[test]
fn issue_211_the_predicate_is_witnessed_true_and_false_on_an_arranged_row() {
    let result = synthesis(SIGN_IN);
    let refused = scenario(&result, NO_REDIRECT);
    assert!(requires_outcome(
        refused,
        "demo.signin.InitiateSignIn/no-redirect-entry"
    ));
    assert!(requires_error(refused, "demo.signin.NoRedirectEntry"));
    let (_, registered) = pointed_at(refused);
    assert_ne!(
        client(&initiation(refused)),
        registered,
        "the predicate holds: the client sent is not the one registered"
    );

    let initiated = scenario(&result, INITIATED);
    assert!(requires_outcome(
        initiated,
        "demo.signin.InitiateSignIn/initiated"
    ));
    let (_, registered) = pointed_at(initiated);
    assert_eq!(
        client(&initiation(initiated)),
        registered,
        "the predicate is false: the client sent is the one registered"
    );
}

// ---- the suite, run ------------------------------------------------------------------------

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

/// How the hand-written sign-in service answers `InitiateSignIn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Refuses a tenant with no configuration, then a client it does not register; else initiates.
    Correct,
    /// Never reads the configuration: initiates every sign-in.
    IgnoresRelated,
    /// Reads existence the wrong way round: refuses a configured tenant as unconfigured, and treats
    /// an unconfigured one as configured for every client.
    InvertsExists,
    /// Reads the first configuration stored rather than the tenant's.
    ReadsFirstRow,
    /// Checks existence and never the registered client.
    IgnoresPredicate,
}

#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    configurations: RefCell<Vec<(String, String)>>,
    sign_ins: RefCell<Vec<BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Store {
    fn begin(&self) {
        self.configurations.replace(Vec::new());
        self.sign_ins.replace(Vec::new());
        self.published.replace(Vec::new());
    }

    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }

    fn token(&self) -> ess_primitives::consistency::ConsistencyToken {
        self.minted.set(self.minted.get() + 1);
        ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.minted.get()))
            .unwrap()
    }
}

struct SignInService {
    mode: Mode,
    store: Store,
}

impl SignInService {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            store: Store::default(),
        }
    }

    /// The client the configuration the service reads registers, if it reads one.
    fn configuration(&self, tenant: &str) -> Option<String> {
        let rows = self.store.configurations.borrow();
        let row = match self.mode {
            Mode::ReadsFirstRow => rows.first(),
            _ => rows.iter().find(|(id, _)| id == tenant),
        };
        row.map(|(_, client)| client.clone())
    }
}

fn refusal(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name))
        .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
}

impl ConformanceTarget for SignInService {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("sign-in-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.begin();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let store = &self.store;
        let result = match command.to_string().as_str() {
            "demo.signin.ConfigureTenant" => {
                let tenant = store.mint();
                store
                    .configurations
                    .borrow_mut()
                    .push((tenant.clone(), text(request.input.get("redirect_client"))));
                SemanticCommandResult::took(branch(&command, "configured")).emitting(
                    ObservedEvent::new("demo.signin.TenantConfigured".parse::<EventRef>().unwrap())
                        .with("tenant", Node::Text(tenant)),
                )
            }
            "demo.signin.InitiateSignIn" => {
                let tenant = text(request.input.get("tenant"));
                let client = text(request.input.get("client"));
                let read = self.configuration(&tenant);
                let decided = match self.mode {
                    Mode::IgnoresRelated => Ok(()),
                    Mode::InvertsExists => match read {
                        Some(_) => Err(("no-configuration", "demo.signin.NoConfiguration")),
                        None => Ok(()),
                    },
                    Mode::IgnoresPredicate => match read {
                        None => Err(("no-configuration", "demo.signin.NoConfiguration")),
                        Some(_) => Ok(()),
                    },
                    Mode::Correct | Mode::ReadsFirstRow => match read {
                        None => Err(("no-configuration", "demo.signin.NoConfiguration")),
                        Some(registered) if registered != client => {
                            Err(("no-redirect-entry", "demo.signin.NoRedirectEntry"))
                        }
                        Some(_) => Ok(()),
                    },
                };
                if let Err((name, error)) = decided {
                    refusal(&command, name, error)
                } else {
                    let id = store.mint();
                    store.sign_ins.borrow_mut().push(BTreeMap::from([
                        ("sign_in_id".to_owned(), Node::Text(id.clone())),
                        ("tenant".to_owned(), Node::Text(tenant)),
                        ("client".to_owned(), Node::Text(client)),
                        ("state".to_owned(), Node::Text("Initiated".to_owned())),
                    ]));
                    SemanticCommandResult::took(branch(&command, "initiated")).emitting(
                        ObservedEvent::new(
                            "demo.signin.SignInInitiated".parse::<EventRef>().unwrap(),
                        )
                        .with("sign_in_id", Node::Text(id)),
                    )
                }
            }
            _ => SemanticCommandResult::undeclared(),
        };
        for event in &result.direct_events {
            store.published.borrow_mut().push(event.clone());
        }
        Ok(result.with_consistency(store.token()))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "demo.signin.Configurations" => self
                .store
                .configurations
                .borrow()
                .iter()
                .map(|(tenant, client)| {
                    BTreeMap::from([
                        ("tenant".to_owned(), Node::Text(tenant.clone())),
                        ("redirect_client".to_owned(), Node::Text(client.clone())),
                        ("state".to_owned(), Node::Text("Active".to_owned())),
                    ])
                })
                .collect(),
            "demo.signin.SignIns" => self.store.sign_ins.borrow().clone(),
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .store
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// Every scenario about `command` and its status against `target`.
fn statuses<T: ConformanceTarget>(
    result: &Synthesis,
    target: &T,
    command: &str,
) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains(command))
        .map(|run| {
            (
                run.scenario.to_string(),
                if run.status == Status::Passed {
                    "passed".to_owned()
                } else {
                    format!("{:?}: {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, String>) -> Vec<&String> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn issue_211_a_target_answering_the_model_passes_every_initiate_scenario() {
    let result = synthesis(SIGN_IN);
    let statuses = statuses(
        &result,
        &SignInService::new(Mode::Correct),
        "InitiateSignIn",
    );
    for id in [NO_CONFIGURATION, NO_REDIRECT, INITIATED] {
        assert!(statuses.contains_key(id), "{id} is run: {statuses:#?}");
    }
    assert!(failed(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn issue_211_each_mutant_fails_an_initiate_scenario() {
    let result = synthesis(SIGN_IN);
    for (mode, fails) in [
        (Mode::IgnoresRelated, NO_CONFIGURATION),
        (Mode::InvertsExists, NO_CONFIGURATION),
        (Mode::InvertsExists, INITIATED),
        (Mode::ReadsFirstRow, INITIATED),
        (Mode::IgnoresPredicate, NO_REDIRECT),
    ] {
        let statuses = statuses(&result, &SignInService::new(mode), "InitiateSignIn");
        assert!(
            failed(&statuses).contains(&&fails.to_owned()),
            "{mode:?} passes {fails}: {statuses:#?}"
        );
    }
}

#[test]
fn issue_211_the_interpreted_target_names_the_related_guard_it_does_not_evaluate() {
    let model = ir(SIGN_IN);
    let result = ess_conformance::synthesize::synthesize(&model);
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    // Superseded by the precedence order (#227 correction 1): the interpreter answers a missing
    // related row by its `exists: false` branch, so `no-configuration` passes rather than being
    // unsupported. The branches that read a stored related row are still not evaluated.
    let absent = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == NO_CONFIGURATION)
        .unwrap_or_else(|| panic!("{NO_CONFIGURATION} is run"));
    assert_eq!(absent.status, Status::Passed, "{:?}", absent.checks);
    for id in [NO_REDIRECT, INITIATED] {
        let run = report
            .scenarios
            .iter()
            .find(|run| run.scenario.to_string() == id)
            .unwrap_or_else(|| panic!("{id} is run"));
        assert_eq!(run.status, Status::Unsupported, "{id}: {:?}", run.checks);
        assert!(
            format!("{:?}", run.checks).contains("related row"),
            "{id} names the construct: {:?}",
            run.checks
        );
    }
}
