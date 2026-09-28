//! The rest of the class adversary pass 2 finding 4 is an instance of, on `when_related:`
//! (beyond10x/ess#211): every connective child of a related predicate is witnessed on a row that
//! isolates it. Pass 2 found a conjunct a target could drop; these are the other members:
//!
//! * a disjunct a target drops is caught — a row holding that disjunct alone selects the guarded
//!   branch, and a target reading the other disjunct only answers otherwise;
//! * a boundary no arrangement can reach is refused under the scenario's id, never dropped.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
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

/// The sign-in fixture with the configuration on a plan and in a region, and the refusal reading
/// both stored fields through `predicate`.
fn two_fields(plans: &str, predicate: &str) -> String {
    let text = replaced(
        SIGN_IN,
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
        &format!(
            "  - {{name: demo.signin.SignInId, kind: newtype, of: Uuid}}\n  - name: demo.signin.Plan\n    kind: enum\n    variants: [{plans}]\n  - name: demo.signin.Region\n    kind: enum\n    variants: [North, South]\n"
        ),
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    lifecycle",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n      - {name: region, type: demo.signin.Region}\n    lifecycle",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    outcomes:",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n      - {name: region, type: demo.signin.Region}\n    outcomes:",
    );
    let text = replaced(
        &text,
        "          redirect_client: input.redirect_client\n",
        "          redirect_client: input.redirect_client\n          plan: input.plan\n          region: input.region\n",
    );
    replaced(
        &text,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n",
        &format!("        when_related: {{via: input.tenant, predicate: {predicate}}}\n"),
    )
}

// ---- an in-memory target ---------------------------------------------------------------------

#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, Vec<BTreeMap<String, Node>>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Store {
    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }

    fn insert(&self, entity: &str, state: &str, mut row: BTreeMap<String, Node>) {
        row.insert("state".to_owned(), Node::Text(state.to_owned()));
        self.rows
            .borrow_mut()
            .entry(entity.to_owned())
            .or_default()
            .push(row);
    }

    fn find(&self, entity: &str, key: &str, id: &str) -> Option<BTreeMap<String, Node>> {
        self.rows.borrow().get(entity).and_then(|rows| {
            rows.iter()
                .find(|row| row.get(key) == Some(&Node::Text(id.to_owned())))
                .cloned()
        })
    }
}

/// A sign-in service refusing where the named configuration holds any of `disjuncts`.
struct Service {
    disjuncts: &'static [(&'static str, &'static str)],
    store: Store,
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn refusal(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name))
        .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
}

fn took(
    command: &CommandRef,
    name: &str,
    event: &str,
    field: &str,
    id: &str,
) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name)).emitting(
        ObservedEvent::new(event.parse::<EventRef>().unwrap()).with(field, Node::Text(id.into())),
    )
}

impl Service {
    fn answer(
        &self,
        command: &CommandRef,
        input: &BTreeMap<String, Node>,
    ) -> SemanticCommandResult {
        let store = &self.store;
        match command.to_string().as_str() {
            "demo.signin.ConfigureTenant" => {
                let tenant = store.mint();
                let mut row = BTreeMap::from([("tenant".to_owned(), Node::Text(tenant.clone()))]);
                for field in ["redirect_client", "plan", "region"] {
                    row.insert(field.to_owned(), input[field].clone());
                }
                store.insert("Configuration", "Active", row);
                took(
                    command,
                    "configured",
                    "demo.signin.TenantConfigured",
                    "tenant",
                    &tenant,
                )
            }
            "demo.signin.InitiateSignIn" => {
                let Some(Node::Text(tenant)) = input.get("tenant") else {
                    panic!("a tenant is sent");
                };
                let Some(row) = store.find("Configuration", "tenant", tenant) else {
                    return refusal(command, "no-configuration", "demo.signin.NoConfiguration");
                };
                if self
                    .disjuncts
                    .iter()
                    .any(|(field, value)| row.get(*field) == Some(&Node::Text((*value).into())))
                {
                    return refusal(command, "no-redirect-entry", "demo.signin.NoRedirectEntry");
                }
                let id = store.mint();
                store.insert(
                    "SignIn",
                    "Initiated",
                    BTreeMap::from([
                        ("sign_in_id".to_owned(), Node::Text(id.clone())),
                        ("tenant".to_owned(), Node::Text(tenant.clone())),
                        ("client".to_owned(), input["client"].clone()),
                    ]),
                );
                took(
                    command,
                    "initiated",
                    "demo.signin.SignInInitiated",
                    "sign_in_id",
                    &id,
                )
            }
            _ => SemanticCommandResult::undeclared(),
        }
    }
}

impl ConformanceTarget for Service {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "related-guard-pass2-fixes",
            "1",
        ))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.rows.replace(BTreeMap::new());
        self.store.published.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let result = self.answer(&request.command, &request.input);
        for event in &result.direct_events {
            self.store.published.borrow_mut().push(event.clone());
        }
        let token = self.store.mint();
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{token}")).unwrap(),
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let entity = match request.view.to_string().as_str() {
            "demo.signin.Configurations" => "Configuration",
            "demo.signin.SignIns" => "SignIn",
            other => panic!("no view {other}"),
        };
        let rows = self
            .store
            .rows
            .borrow()
            .get(entity)
            .cloned()
            .unwrap_or_default();
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

/// Every sign-in scenario that does not pass against a service refusing on `disjuncts`.
fn failing(result: &Synthesis, disjuncts: &'static [(&'static str, &'static str)]) -> Vec<String> {
    let target = Service {
        disjuncts,
        store: Store::default(),
    };
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains("InitiateSignIn"))
        .filter(|run| run.status != Status::Passed)
        .map(|run| format!("{}: {:?}", run.scenario, run.checks))
        .collect()
}

#[test]
fn a_target_reading_one_disjunct_of_the_related_predicate_fails_a_scenario() {
    let result = synthesis(&two_fields(
        "Basic, Premium",
        "{any: [plan == Basic, region == North]}",
    ));
    assert_eq!(
        refusals_about(&result, "InitiateSignIn"),
        Vec::<String>::new()
    );
    let correct = failing(&result, &[("plan", "Basic"), ("region", "North")]);
    assert!(correct.is_empty(), "a correct target fails: {correct:#?}");
    for kept in [&[("plan", "Basic")][..], &[("region", "North")][..]] {
        assert!(
            !failing(&result, kept).is_empty(),
            "a target reading only {kept:?} passes every scenario"
        );
    }
}

/// `plan == Basic` cannot be false while `plan != Premium` holds when `Basic` and `Premium` are
/// the only plans: that boundary has no row, and it is refused rather than left out.
#[test]
fn a_boundary_no_row_reaches_is_refused_under_the_scenario_id() {
    let result = synthesis(&two_fields(
        "Basic, Premium",
        "{all: [plan == Basic, plan != Premium]}",
    ));
    let refused: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal.scenario.as_ref().is_some_and(|id| {
                id.to_string() == "demo.signin.InitiateSignIn/outcome/no-redirect-entry"
            })
        })
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.starts_with("ESS-SYNTH-003")
                && refusal.contains("`plan == Basic` is false")),
        "the unreachable boundary is refused: {refused:#?}\nall: {:#?}",
        refusals_about(&result, "InitiateSignIn")
    );
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == "demo.signin.InitiateSignIn/outcome/no-redirect-entry"),
        "the branch's own scenario still stands"
    );
}
