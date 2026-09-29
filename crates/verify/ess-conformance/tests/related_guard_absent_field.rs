//! A stored-row predicate over an `Optional` field the creating command leaves absent
//! (beyond10x/ess#239).
//!
//! `OnboardTenant` creates the configuration row with `site` unwritten; `ConfigureSite` sets it
//! later. The row as the creator leaves it holds `site` absent, so `{site: {exists: false}}` is
//! witnessed there, and the sibling on a row the later command wrote. The same holds for every
//! reader of a stored row: `when_related`, `when_subject`, `defined()` and a nested `Optional`
//! member of a struct field the creator writes without it.
#![allow(clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
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
const INITIATED: &str = "demo.sso.InitiateSignIn/outcome/initiated";
const NO_CONFIGURATION: &str = "demo.sso.InitiateSignIn/outcome/no-configuration";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("sso.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.code()))
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("no scenario {id}\n refusals: {:#?}", refusals(result)),
            |(_, scenario)| scenario,
        )
}

/// The commands a scenario sends, in order.
fn commands(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

/// The instance the last command of `name` the scenario sends names under `field`.
fn named(scenario: &ConformanceScenario, name: &str, field: &str) -> String {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } if command.to_string() == name => {
                match input.get(field) {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    other => panic!("{field} names no arranged row: {other:?}"),
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("the scenario sends no {name}"))
}

/// The instances every `ConfigureSite` of the scenario writes a site on.
fn configured(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.sso.ConfigureSite" =>
            {
                match input.get("tenant") {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

#[test]
fn issue_239_the_absent_site_branch_is_witnessed_on_the_row_the_creator_leaves() {
    let result = synthesis(MODEL);
    let refused: Vec<String> = refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("InitiateSignIn"))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");

    let absent = scenario(&result, NOT_CONFIGURED);
    let row = named(absent, "demo.sso.InitiateSignIn", "tenant");
    assert!(
        !configured(absent).contains(&row),
        "the row the branch reads is the one the creator left, no site written on it: {:#?}",
        absent.steps
    );

    let present = scenario(&result, INITIATED);
    let row = named(present, "demo.sso.InitiateSignIn", "tenant");
    assert!(
        configured(present).contains(&row),
        "the sibling reads a row a later command wrote the site on: {:?}",
        commands(present)
    );
}

// ---- the suite, run ------------------------------------------------------------------------

/// How the sign-in service reads the configuration's site.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reading {
    /// As declared.
    Correct,
    /// As present on every row: never answers `not-configured`.
    AlwaysPresent,
    /// As absent on every row: answers `not-configured` for every existing row.
    AlwaysAbsent,
}

#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    configurations: RefCell<BTreeMap<String, Option<String>>>,
    sign_ins: RefCell<BTreeMap<String, String>>,
    published: RefCell<Vec<ObservedEvent>>,
}

struct SignInService {
    reading: Reading,
    store: Store,
}

impl SignInService {
    fn new(reading: Reading) -> Self {
        Self {
            reading,
            store: Store::default(),
        }
    }

    fn mint(&self) -> String {
        self.store.minted.set(self.store.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.store.minted.get())
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

impl ConformanceTarget for SignInService {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("sign-in-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.configurations.replace(BTreeMap::new());
        self.store.sign_ins.replace(BTreeMap::new());
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
                    self.store
                        .configurations
                        .borrow_mut()
                        .insert(id.clone(), None);
                    SemanticCommandResult::took(branch(&command, "onboarded")).emitting(event(
                        "demo.sso.TenantOnboarded",
                        "tenant",
                        &id,
                    ))
                }
                "demo.sso.ConfigureSite" => {
                    let id = text(request.input.get("tenant"));
                    let site = text(request.input.get("site"));
                    let mut rows = self.store.configurations.borrow_mut();
                    match rows.get_mut(&id) {
                        Some(held) => {
                            *held = Some(site);
                            SemanticCommandResult::took(branch(&command, "configured"))
                                .emitting(event("demo.sso.SiteConfigured", "tenant", &id))
                        }
                        None => SemanticCommandResult::undeclared(),
                    }
                }
                "demo.sso.InitiateSignIn" => {
                    let tenant = text(request.input.get("tenant"));
                    let held = self.store.configurations.borrow().get(&tenant).cloned();
                    match held {
                        None => SemanticCommandResult::took(branch(&command, "no-configuration"))
                            .with_error(error("demo.sso.NoConfiguration")),
                        Some(site) => {
                            let absent = match self.reading {
                                Reading::Correct => site.is_none(),
                                Reading::AlwaysPresent => false,
                                Reading::AlwaysAbsent => true,
                            };
                            if absent {
                                SemanticCommandResult::took(branch(&command, "not-configured"))
                                    .with_error(error("demo.sso.NotConfigured"))
                            } else {
                                let id = self.mint();
                                self.store.sign_ins.borrow_mut().insert(id.clone(), tenant);
                                SemanticCommandResult::took(branch(&command, "initiated"))
                                    .emitting(event("demo.sso.SignInInitiated", "sign_in_id", &id))
                            }
                        }
                    }
                }
                _ => SemanticCommandResult::undeclared(),
            };
        for published in &result.direct_events {
            self.store.published.borrow_mut().push(published.clone());
        }
        self.store.minted.set(self.store.minted.get() + 1);
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!(
                "seq:{}",
                self.store.minted.get()
            ))
            .unwrap(),
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "demo.sso.Configurations" => self
                .store
                .configurations
                .borrow()
                .iter()
                .map(|(id, site)| {
                    let mut row = BTreeMap::from([
                        ("tenant".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Active".to_owned())),
                    ]);
                    // An absent `Optional` is left out of the row, not projected as a null.
                    if let Some(site) = site {
                        row.insert("site".to_owned(), Node::Text(site.clone()));
                    }
                    row
                })
                .collect(),
            "demo.sso.SignIns" => self
                .store
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

fn statuses(result: &Synthesis, target: &SignInService) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
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

#[test]
fn issue_239_a_target_reading_the_site_as_present_fails_and_the_declared_one_passes() {
    let result = synthesis(MODEL);
    let correct = statuses(&result, &SignInService::new(Reading::Correct));
    for id in [NO_CONFIGURATION, NOT_CONFIGURED, INITIATED] {
        assert_eq!(
            correct.get(id).map(String::as_str),
            Some("passed"),
            "{id}: {correct:#?}"
        );
    }
    assert!(
        correct.values().all(|status| status == "passed"),
        "{correct:#?}"
    );
    let present = statuses(&result, &SignInService::new(Reading::AlwaysPresent));
    assert_ne!(
        present.get(NOT_CONFIGURED).map(String::as_str),
        Some("passed"),
        "{present:#?}"
    );
    let absent = statuses(&result, &SignInService::new(Reading::AlwaysAbsent));
    assert_ne!(
        absent.get(INITIATED).map(String::as_str),
        Some("passed"),
        "{absent:#?}"
    );
}

// ---- the class: every stored-row reader, every presence form ----------------------------------

/// The absent-site branch written as `not defined(site)`, and its sibling as `defined(site)`.
#[test]
fn issue_239_class_not_defined_over_the_related_row_is_witnessed_on_the_created_row() {
    let text = MODEL.replace(
        "predicate: {site: {exists: false}}}",
        "predicate: {not: \"defined(site)\"}}",
    );
    assert_ne!(text, MODEL, "the fixture was not rewritten");
    let result = synthesis(&text);
    let absent = scenario(&result, NOT_CONFIGURED);
    let row = named(absent, "demo.sso.InitiateSignIn", "tenant");
    assert!(!configured(absent).contains(&row), "{:#?}", absent.steps);
    let present = scenario(&result, INITIATED);
    let row = named(present, "demo.sso.InitiateSignIn", "tenant");
    assert!(
        configured(present).contains(&row),
        "{:?}",
        commands(present)
    );
}

/// `defined(site)` guarding the refusal: its default is the row the creator left.
#[test]
fn issue_239_class_the_default_beside_defined_is_witnessed_on_the_created_row() {
    let text = MODEL.replace(
        "predicate: {site: {exists: false}}}",
        "predicate: \"defined(site)\"}",
    );
    assert_ne!(text, MODEL, "the fixture was not rewritten");
    let result = synthesis(&text);
    // `not-configured` now reads "a site is set"; `initiated` the row without one.
    let present = scenario(&result, NOT_CONFIGURED);
    let row = named(present, "demo.sso.InitiateSignIn", "tenant");
    assert!(configured(present).contains(&row), "{:#?}", present.steps);
    let absent = scenario(&result, INITIATED);
    let row = named(absent, "demo.sso.InitiateSignIn", "tenant");
    assert!(!configured(absent).contains(&row), "{:?}", commands(absent));
}

/// The same row read through `when_subject:` on a command acting on the configuration itself.
#[test]
fn issue_239_class_when_subject_absent_site_is_witnessed_on_the_created_row() {
    let text = MODEL
        .replace(
            "  - name: demo.sso.InitiateSignIn\n",
            "  - name: demo.sso.Publish
    input:
      - {name: tenant, type: demo.sso.TenantId}
    outcomes:
      - name: unpublishable
        when_subject: {predicate: {site: {exists: false}}}
        error: demo.sso.NotConfigured
      - name: published
        updates: demo.sso.Configuration
        instance: tenant
        emits: [demo.sso.SiteConfigured]
        payload: {demo.sso.SiteConfigured: {tenant: input.tenant}}
  - name: demo.sso.InitiateSignIn\n",
        )
        .replace(
            "may: [demo.sso.OnboardTenant,",
            "may: [demo.sso.Publish, demo.sso.OnboardTenant,",
        );
    assert_ne!(text, MODEL, "the fixture was not rewritten");
    let result = synthesis(&text);
    let refused: Vec<String> = refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("Publish"))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    let absent = scenario(&result, "demo.sso.Publish/outcome/unpublishable");
    let row = named(absent, "demo.sso.Publish", "tenant");
    assert!(!configured(absent).contains(&row), "{:#?}", absent.steps);
    let present = scenario(&result, "demo.sso.Publish/outcome/published");
    let row = named(present, "demo.sso.Publish", "tenant");
    assert!(
        configured(present).contains(&row),
        "{:?}",
        commands(present)
    );
}

/// The creator writes `site` from an `Optional` input of its own, which it may leave out: the row
/// it leaves then holds `site` absent, reached by choosing that input rather than by not writing.
#[test]
fn issue_239_class_a_site_the_creator_maps_from_an_omitted_optional_input_is_witnessed() {
    let text = MODEL
        .replace(
            "  - name: demo.sso.OnboardTenant\n    input: []\n",
            "  - name: demo.sso.OnboardTenant\n    input:\n      - {name: site, type: Optional<String>}\n",
        )
        .replace(
            "        payload: {demo.sso.TenantOnboarded: {tenant: {generated: true}}}\n",
            "        payload: {demo.sso.TenantOnboarded: {tenant: {generated: true}}}
        sets: {site: input.site}
",
        );
    assert_ne!(text, MODEL, "the fixture was not rewritten");
    let result = synthesis(&text);
    let refused: Vec<String> = refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("InitiateSignIn"))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    let absent = scenario(&result, NOT_CONFIGURED);
    let row = named(absent, "demo.sso.InitiateSignIn", "tenant");
    // The creation whose captured instance is the row the branch reads, not a decoy beside it.
    let sent_site = absent.steps.windows(3).find_map(|steps| match steps {
        [ScenarioStep::ExecuteCommand { command, input, .. }, _, ScenarioStep::CaptureInstance { instance, .. }]
            if command.to_string() == "demo.sso.OnboardTenant" && instance.to_string() == row =>
        {
            Some(input.get("site").cloned())
        }
        _ => None,
    });
    assert!(
        matches!(
            sent_site,
            Some(None | Some(ScenarioValue::Literal { value: Node::Null }))
        ),
        "the row the branch reads was created with its site left out: {:#?}",
        absent.steps
    );
    scenario(&result, INITIATED);
}

/// The boundary of the class, recorded: an `Optional` member of a struct field is left out by the
/// creator only through an omitted `Optional` input, and a struct with an undetermined member is not
/// determined as a whole (`leaf_value` in `synthesize.rs`), so a predicate over the member reads
/// `Unknown` on every row. The branch is refused by name — never silently dropped — until a struct
/// value can carry a member's absence without changing what its payload and view assertions demand.
#[test]
fn issue_239_class_a_nested_optional_member_is_refused_by_name_not_dropped() {
    let text = MODEL
        .replace(
            "types:\n",
            "types:
  - name: demo.sso.Settings
    kind: struct
    fields:
      - {name: issuer, type: String}
      - {name: site, type: Optional<String>}
",
        )
        .replace(
            "      - {name: site, type: Optional<String>}\n    lifecycle: {initial: Active",
            "      - {name: settings, type: demo.sso.Settings}\n    lifecycle: {initial: Active",
        )
        .replace(
            "  - name: demo.sso.OnboardTenant\n    input: []\n",
            "  - name: demo.sso.OnboardTenant\n    input:\n      - {name: issuer, type: String}\n      - {name: site, type: Optional<String>}\n",
        )
        .replace(
            "        payload: {demo.sso.TenantOnboarded: {tenant: {generated: true}}}\n",
            "        payload: {demo.sso.TenantOnboarded: {tenant: {generated: true}}}
        sets: {settings: {issuer: input.issuer, site: input.site}}
",
        )
        .replace(
            "      - {name: site, type: String}\n",
            "      - {name: issuer, type: String}\n      - {name: site, type: String}\n",
        )
        .replace(
            "sets: {site: input.site}",
            "sets: {settings: {issuer: input.issuer, site: input.site}}",
        )
        .replace(
            "predicate: {site: {exists: false}}}",
            "predicate: {settings.site: {exists: false}}}",
        )
        .replace(
            "      - {name: site, type: Optional<String>}\n  - name: demo.sso.SignIns",
            "      - {name: settings, type: demo.sso.Settings}\n  - name: demo.sso.SignIns",
        );
    let result = synthesis(&text);
    let witnessed = result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == NOT_CONFIGURED);
    let refused = refusals(&result)
        .into_iter()
        .any(|refusal| refusal.contains("ESS-SYNTH-003") && refusal.contains(NOT_CONFIGURED));
    assert!(
        witnessed || refused,
        "the branch is witnessed or refused by name: {:#?}",
        refusals(&result)
    );
}

// ---- correction 1: every writer of the field other than the row's own later acts ----------------

/// A binding invoking `ConfigureSite` on every onboarded tenant writes the site after the creator
/// runs, in the target, with no step of the scenario asking for it: the row is not known to hold
/// it absent, so the absent-site branch is not witnessed on it.
#[test]
fn issue_239_correction_a_site_a_binding_writes_is_not_read_as_absent() {
    let text = MODEL
        .replace(
            "    fields: [{name: tenant, type: demo.sso.TenantId}]\n  - name: demo.sso.SiteConfigured",
            "    fields: [{name: tenant, type: demo.sso.TenantId}, {name: site, type: String}]\n  - name: demo.sso.SiteConfigured",
        )
        .replace(
            "payload: {demo.sso.TenantOnboarded: {tenant: {generated: true}}}",
            "payload: {demo.sso.TenantOnboarded: {tenant: {generated: true}, site: {generated: true}}}",
        )
        + "bindings:
  - id: configure-on-onboarded
    when:
      event: demo.sso.TenantOnboarded
    invoke:
      command: demo.sso.ConfigureSite
    mapping:
      tenant: event.tenant
      site: event.site
    delivery: at_least_once
    on_failure: retry
";
    let result = synthesis(&text);
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == NOT_CONFIGURED),
        "the absent-site branch was filed on a row a binding writes the site on: {:#?}",
        result
            .suite
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == NOT_CONFIGURED)
            .map(|(_, scenario)| commands(scenario))
    );
}

/// An `instances:` update writing the site on every configuration it selects writes it on rows no
/// step of the arrangement names: the row is not known to hold it absent either.
#[test]
fn issue_239_correction_a_site_an_instances_update_writes_is_not_read_as_absent() {
    let text = MODEL
        .replace(
            "  - name: demo.sso.InitiateSignIn\n    input:",
            "  - name: demo.sso.DefaultSites
    input:
      - {name: site, type: String}
    outcomes:
      - name: defaulted
        updates: demo.sso.Configuration
        instances: {where: not defined(site)}
        sets: {site: input.site}
        emits: [demo.sso.SitesDefaulted]
        payload: {demo.sso.SitesDefaulted: {site: input.site}}
  - name: demo.sso.InitiateSignIn\n    input:",
        )
        .replace(
            "  - name: demo.sso.SignInInitiated\n",
            "  - name: demo.sso.SitesDefaulted\n    fields: [{name: site, type: String}]\n  - name: demo.sso.SignInInitiated\n",
        )
        .replace(
            "may: [demo.sso.OnboardTenant,",
            "may: [demo.sso.DefaultSites, demo.sso.OnboardTenant,",
        );
    assert_ne!(text, MODEL, "the fixture was not rewritten");
    let result = synthesis(&text);
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == NOT_CONFIGURED),
        "the absent-site branch was filed on a row an `instances:` update may write: {:#?}",
        result
            .suite
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == NOT_CONFIGURED)
            .map(|(_, scenario)| commands(scenario))
    );
}
