//! Adversary pass 1 against beyond10x/ess#239 (`Arrangement::unwritten`).
//!
//! `unwritten` is computed from the creating branch's own `sets:` and shrunk only by the `sets:` of
//! later acts on the same row. An `affects:` of another command writes a stored field on rows other
//! than its subject, and nothing folds that write into `unwritten`: a row whose `site` a later
//! `ConfigureSite` on another row wrote through `affects:` is still read as absent, and a scenario expecting the absent-site
//! branch is sent against a correct target that holds a site there.
#![allow(clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner, ScenarioStep};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

/// The #239 model, with `ConfigureSite` also filling in a fallback site on every other
/// configuration that has none (`affects:`, ess/16).
const MODEL: &str = "format: ess/18
system: demo
version: v1
domain: demo.sso
types:
  - {name: demo.sso.TenantId, kind: newtype, of: Uuid}
  - {name: demo.sso.SignInId, kind: newtype, of: Uuid}
entities:
  - name: demo.sso.Configuration
    identity: {name: tenant, type: demo.sso.TenantId}
    fields:
      - {name: site, type: Optional<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.sso.SignIn
    identity: {name: sign_in_id, type: demo.sso.SignInId}
    fields:
      - {name: tenant, type: demo.sso.TenantId}
    lifecycle: {initial: Initiated, states: [Initiated], terminal: [Initiated]}
errors:
  - {name: demo.sso.NoConfiguration, summary: The tenant has no configuration., fields: []}
  - {name: demo.sso.NotConfigured, summary: The configuration has no site yet., fields: []}
events:
  - name: demo.sso.TenantOnboarded
    fields: [{name: tenant, type: demo.sso.TenantId}]
  - name: demo.sso.SiteConfigured
    fields: [{name: tenant, type: demo.sso.TenantId}]
  - name: demo.sso.SignInInitiated
    fields: [{name: sign_in_id, type: demo.sso.SignInId}]
actors:
  - name: demo.sso.Operator
    may: [demo.sso.OnboardTenant, demo.sso.ConfigureSite, demo.sso.InitiateSignIn]
commands:
  - name: demo.sso.OnboardTenant
    input: []
    outcomes:
      - name: onboarded
        creates: demo.sso.Configuration
        instance: tenant
        emits: [demo.sso.TenantOnboarded]
        payload: {demo.sso.TenantOnboarded: {tenant: {generated: true}}}
  - name: demo.sso.ConfigureSite
    input:
      - {name: tenant, type: demo.sso.TenantId}
      - {name: site, type: String}
    outcomes:
      - name: configured
        updates: demo.sso.Configuration
        instance: tenant
        sets: {site: input.site}
        emits: [demo.sso.SiteConfigured]
        payload: {demo.sso.SiteConfigured: {tenant: input.tenant}}
        affects:
          - entity: demo.sso.Configuration
            where: not defined(site)
            sets:
              site: fallback
  - name: demo.sso.InitiateSignIn
    input:
      - {name: tenant, type: demo.sso.TenantId}
    outcomes:
      - name: no-configuration
        when_related: {via: input.tenant, exists: false}
        error: demo.sso.NoConfiguration
      - name: not-configured
        when_related: {via: input.tenant, predicate: {site: {exists: false}}}
        error: demo.sso.NotConfigured
      - name: initiated
        creates: demo.sso.SignIn
        instance: sign_in_id
        emits: [demo.sso.SignInInitiated]
        payload: {demo.sso.SignInInitiated: {sign_in_id: {generated: true}}}
        sets: {tenant: input.tenant}
views:
  - name: demo.sso.Configurations
    source: demo.sso.Configuration
    consistency: read_your_writes
    fields:
      - {name: tenant, type: demo.sso.TenantId}
      - {name: state, type: demo.sso.Configuration.State}
      - {name: site, type: Optional<String>}
  - name: demo.sso.SignIns
    source: demo.sso.SignIn
    consistency: read_your_writes
    fields:
      - {name: sign_in_id, type: demo.sso.SignInId}
      - {name: tenant, type: demo.sso.TenantId}
";

const NOT_CONFIGURED: &str = "demo.sso.InitiateSignIn/outcome/not-configured";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("sso.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

/// A target implementing the model above as declared, `affects:` included.
#[derive(Default)]
struct Declared {
    minted: Cell<u64>,
    configurations: RefCell<BTreeMap<String, Option<String>>>,
    sign_ins: RefCell<BTreeMap<String, String>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Declared {
    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Declared {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("declared-sso", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.configurations.replace(BTreeMap::new());
        self.sign_ins.replace(BTreeMap::new());
        self.published.replace(Vec::new());
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
        let event = |name: &str, field: &str, id: &str| {
            ObservedEvent::new(name.parse::<EventRef>().unwrap())
                .with(field, Node::Text(id.to_owned()))
        };
        let error = |name: &str| DeclaredErrorValue::new(name.parse::<ErrorRef>().unwrap());
        let result =
            match command.to_string().as_str() {
                "demo.sso.OnboardTenant" => {
                    let id = self.mint();
                    self.configurations.borrow_mut().insert(id.clone(), None);
                    SemanticCommandResult::took(branch(&command, "onboarded")).emitting(event(
                        "demo.sso.TenantOnboarded",
                        "tenant",
                        &id,
                    ))
                }
                "demo.sso.ConfigureSite" => {
                    let id = text(request.input.get("tenant"));
                    let site = text(request.input.get("site"));
                    let mut rows = self.configurations.borrow_mut();
                    match rows.get(&id).map(|_| ()) {
                        Some(()) => {
                            // `affects:` — every other configuration with no site gets the fallback;
                            // the subject is not among the rows.
                            for (other, held) in rows.iter_mut() {
                                if *other != id && held.is_none() {
                                    *held = Some("fallback".to_owned());
                                }
                            }
                            rows.insert(id.clone(), Some(site));
                            SemanticCommandResult::took(branch(&command, "configured"))
                                .emitting(event("demo.sso.SiteConfigured", "tenant", &id))
                        }
                        None => SemanticCommandResult::undeclared(),
                    }
                }
                "demo.sso.InitiateSignIn" => {
                    let tenant = text(request.input.get("tenant"));
                    let held = self.configurations.borrow().get(&tenant).cloned();
                    match held {
                        None => SemanticCommandResult::took(branch(&command, "no-configuration"))
                            .with_error(error("demo.sso.NoConfiguration")),
                        Some(None) => {
                            SemanticCommandResult::took(branch(&command, "not-configured"))
                                .with_error(error("demo.sso.NotConfigured"))
                        }
                        Some(Some(_)) => {
                            let id = self.mint();
                            self.sign_ins.borrow_mut().insert(id.clone(), tenant);
                            SemanticCommandResult::took(branch(&command, "initiated"))
                                .emitting(event("demo.sso.SignInInitiated", "sign_in_id", &id))
                        }
                    }
                }
                _ => SemanticCommandResult::undeclared(),
            };
        for published in &result.direct_events {
            self.published.borrow_mut().push(published.clone());
        }
        self.minted.set(self.minted.get() + 1);
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!(
                "seq:{}",
                self.minted.get()
            ))
            .unwrap(),
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "demo.sso.Configurations" => self
                .configurations
                .borrow()
                .iter()
                .map(|(id, site)| {
                    let mut row = BTreeMap::from([
                        ("tenant".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Active".to_owned())),
                    ]);
                    if let Some(site) = site {
                        row.insert("site".to_owned(), Node::Text(site.clone()));
                    }
                    row
                })
                .collect(),
            "demo.sso.SignIns" => self
                .sign_ins
                .borrow()
                .iter()
                .map(|(id, tenant)| {
                    BTreeMap::from([
                        ("sign_in_id".to_owned(), Node::Text(id.clone())),
                        ("tenant".to_owned(), Node::Text(tenant.clone())),
                        ("state".to_owned(), Node::Text("Initiated".to_owned())),
                    ])
                })
                .collect(),
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
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

/// The commands the scenario sends, in order.
fn commands(steps: &[ScenarioStep]) -> Vec<String> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

/// The absent-site branch is either refused by name, or its scenario passes a target that does
/// exactly what the model declares. It must never be filed as a scenario that fails that target.
#[test]
fn a_site_written_by_a_later_creations_affects_is_not_read_as_absent() {
    let result = synthesis(MODEL);
    let Some(scenario) = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == NOT_CONFIGURED)
        .map(|(_, scenario)| scenario)
    else {
        // Refused: acceptable, the arrangement cannot know the site is absent.
        return;
    };
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Declared::default())
        .into_report();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == NOT_CONFIGURED)
        .expect("the filed scenario ran");
    assert_eq!(
        run.status,
        Status::Passed,
        "a target implementing the declared `affects:` fails the filed absent-site scenario.\n\
         commands sent: {:?}\nchecks: {:#?}",
        commands(&scenario.steps),
        run.checks
    );
}

/// Every scenario filed for the model passes a target doing what it declares.
#[test]
fn every_filed_scenario_passes_the_declared_affects_target() {
    let result = synthesis(MODEL);
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Declared::default())
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|run| run.status != Status::Passed)
        .map(|run| run.scenario.to_string())
        .collect();
    assert!(
        failed.is_empty(),
        "filed scenarios failing a target that does what the model declares: {failed:#?}"
    );
}
