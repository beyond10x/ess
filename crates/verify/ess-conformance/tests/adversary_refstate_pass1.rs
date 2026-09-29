//! Adversary pass 1 for beyond10x/ess#227: `refused_by_input` (the model interpreter) now answers
//! every input-guarded refusal naming no subject before anything else, on every command — including
//! a command with a `when_related:` guard, which it refused as `NotInterpreted` before.
//!
//! The documented precedence for that command is the other way round
//! (`docs/design/cross-record-and-stored-field-guards.md`, "Precedence", beyond10x/ess#211 adversary
//! pass 1, decisions.md #211 row): "A missing related row is answered by the `exists: false` branch
//! before any other branch: a predicate branch, an input-guarded branch, the default … on a missing
//! row `exists: false` answers, whatever the input."
//!
//! The model is the committed `related-guard-sign-in.yaml` with one input-guarded refusal added,
//! `blank-client: when: client == ""`.
#![allow(clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");

const BLANK: &str = "demo.signin.InitiateSignIn/outcome/blank-client";

fn model() -> String {
    let edit = |text: &str, from: &str, to: &str| {
        let out = text.replace(from, to);
        assert_ne!(out, text, "`{from}` is in the fixture");
        out
    };
    let text = edit(
        SIGN_IN,
        "  - {name: demo.signin.NoRedirectEntry, summary: The configuration does not register the client., fields: []}\n",
        "  - {name: demo.signin.NoRedirectEntry, summary: The configuration does not register the client., fields: []}\n  - {name: demo.signin.BlankClient, summary: The client is blank., fields: []}\n",
    );
    edit(
        &text,
        "      - name: initiated\n        creates: demo.signin.SignIn\n",
        "      - name: blank-client\n        when: client == \"\"\n        error: demo.signin.BlankClient\n      - name: initiated\n        creates: demo.signin.SignIn\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("sign-in.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// A blank client for a tenant nothing configured: the related row is missing, so the documented
/// answer is `no-configuration`. The interpreter must give that answer or decline
/// (`NotInterpreted`, as it did before this unit) — it must not answer `blank-client`.
#[test]
fn the_interpreter_does_not_answer_an_input_refusal_before_a_missing_related_row() {
    let model = ir(&model());
    let input = BTreeMap::from([
        (
            "tenant".to_owned(),
            Node::Text("00000000-0000-4000-8000-000000000077".to_owned()),
        ),
        ("client".to_owned(), Node::Text(String::new())),
    ]);
    let answer = execute(
        &model,
        &Store::default(),
        &"demo.signin.InitiateSignIn".parse().unwrap(),
        &input,
        &Externals::Withheld,
    );
    if let Ok(steps) = &answer {
        let taken: Vec<String> = steps
            .iter()
            .map(|step| {
                step.outcome
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default()
            })
            .collect();
        assert!(
            !taken.iter().any(|name| name.ends_with("/blank-client")),
            "the interpreter answered {taken:?} for a tenant with no configuration; \
             #211 precedence says `no-configuration` answers a missing row whatever the input"
        );
    }
}

// ---- two input refusals claiming one request, beside held-state branches -----------------------

const ROTATE: &str = include_str!("fixtures/refusal-beside-state.yaml");

/// `refusal-beside-state.yaml` with a closed `mode` input and two refusals that both claim
/// `mode == Freeze`, beside the undecidable `too-short`. Validation admits it (the partition drops
/// every input refusal once `too-short` is undecidable; see the ess-domain adversary file).
fn frozen_twice() -> String {
    let edit = |text: &str, from: &str, to: &str| {
        let out = text.replace(from, to);
        assert_ne!(out, text, "`{from}` is in the fixture");
        out
    };
    let text = edit(
        ROTATE,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze]}\n",
    );
    let text = edit(
        &text,
        "      - {name: secret, type: String}\n    outcomes:\n",
        "      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n    outcomes:\n",
    );
    edit(
        &text,
        "        error: demo.secrets.SecretTooShort\n      - name: rotated\n",
        "        error: demo.secrets.SecretTooShort\n      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n      - name: frozen-again\n        when: mode == Freeze\n        error: demo.secrets.NotConfigured\n      - name: rotated\n",
    )
}

/// Entity Runtime orders input-guarded refusals first and then by declaration, and takes the
/// first whose guard holds (`ess-entity-runtime/src/lib.rs:1476-1491`): `frozen`. The coordinator's
/// #217 precedence is "refusals first, then declaration order". The interpreter must agree: one
/// answer, `frozen`.
#[test]
fn the_interpreter_answers_the_first_declared_of_two_overlapping_refusals_as_entity_runtime_does() {
    let model = ir(&frozen_twice());
    let input = BTreeMap::from([
        (
            "tenant_id".to_owned(),
            Node::Text("00000000-0000-4000-8000-000000000055".to_owned()),
        ),
        (
            "secret".to_owned(),
            Node::Text("a-long-enough-secret".to_owned()),
        ),
        ("mode".to_owned(), Node::Text("Freeze".to_owned())),
    ]);
    let steps = execute(
        &model,
        &Store::default(),
        &"demo.secrets.RotateSecret".parse().unwrap(),
        &input,
        &Externals::Withheld,
    )
    .unwrap_or_else(|why| panic!("{why}"));
    let taken: Vec<String> = steps
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(taken, ["demo.secrets.RotateSecret/frozen"], "{taken:?}");
}

// ---- the synthesized blank-client scenario against a target answering the #211 precedence ----

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

/// Missing configuration first (`exists: false`), then the blank client (input refusal, before
/// the stored row's fields are read), then the registered-client predicate, else initiated.
#[derive(Default)]
struct SignIn {
    minted: Cell<u64>,
    configurations: RefCell<Vec<(String, String)>>,
    sign_ins: RefCell<Vec<BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl SignIn {
    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }
}

impl ConformanceTarget for SignIn {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("sign-in-precedence", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.configurations.replace(Vec::new());
        self.sign_ins.replace(Vec::new());
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
        let refuse = |name: &str, error: &str| {
            SemanticCommandResult::took(branch(&command, name))
                .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
        };
        let result = match command.to_string().as_str() {
            "demo.signin.ConfigureTenant" => {
                let tenant = self.mint();
                self.configurations
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
                let registered = self
                    .configurations
                    .borrow()
                    .iter()
                    .find(|(id, _)| *id == tenant)
                    .map(|(_, client)| client.clone());
                match registered {
                    None => refuse("no-configuration", "demo.signin.NoConfiguration"),
                    Some(_) if client.is_empty() => {
                        refuse("blank-client", "demo.signin.BlankClient")
                    }
                    Some(registered) if registered != client => {
                        refuse("no-redirect-entry", "demo.signin.NoRedirectEntry")
                    }
                    Some(_) => {
                        let id = self.mint();
                        self.sign_ins.borrow_mut().push(BTreeMap::from([
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
            }
            _ => SemanticCommandResult::undeclared(),
        };
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "demo.signin.Configurations" => self
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
            "demo.signin.SignIns" => self.sign_ins.borrow().clone(),
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

/// The `blank-client` scenario the suite files must pass a target that answers a missing
/// configuration first, as #211 documents; one that sends the blank client for an unconfigured
/// tenant and requires `blank-client` asserts the opposite precedence.
#[test]
fn the_blank_client_scenario_passes_a_target_answering_a_missing_row_first() {
    let model = ir(&model());
    let result = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &SignIn::default())
        .into_report();
    let refused: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{:?}", refusal.scenario))
        .collect();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == BLANK)
        .unwrap_or_else(|| panic!("{BLANK} is filed; refusals: {refused:#?}"));
    assert_eq!(run.status, Status::Passed, "{:#?}", run.checks);
}
