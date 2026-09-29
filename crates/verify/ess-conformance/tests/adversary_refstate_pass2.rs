//! Adversary pass 2 for beyond10x/ess#227, after correction 1: one precedence order
//! (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order") answered alike by
//! validation (the model compiles), synthesis (the suite) and the model interpreter (the
//! interpreted target run against that suite).
//!
//! * a table over the combinations the order ranks: every synthesized scenario the interpreter
//!   runs must pass or be `unsupported` — a `failed` scenario is synthesis and the interpreter
//!   answering one request differently;
//! * a refusal or accepting branch every input of which an earlier refusal claims is named by
//!   synthesis (witnessed or refused), not dropped;
//! * the missing-row witness of `exists: false` keeps the accepting branch's input beside an input
//!   refusal (class 3: only the first witnessed).
#![allow(clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");
const ROTATE: &str = include_str!("fixtures/refusal-beside-state.yaml");
const TICKETS: &str = include_str!("fixtures/input-guard-overlap.yaml");

fn edit(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validation refused:\n{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

// ---- models, one per combination the order ranks ------------------------------------------------

const SIGN_IN_ERRORS: &str = "  - {name: demo.signin.NoRedirectEntry, summary: The configuration does not register the client., fields: []}\n";

/// Step 1 against step 2: `when_related` and an input refusal.
fn related_and_refusal() -> String {
    let text = edit(
        SIGN_IN,
        SIGN_IN_ERRORS,
        &format!("{SIGN_IN_ERRORS}  - {{name: demo.signin.Blocked, summary: The client is blocked., fields: []}}\n"),
    );
    edit(
        &text,
        "      - name: initiated\n        creates: demo.signin.SignIn\n",
        "      - name: blocked\n        when: client == \"blocked\"\n        error: demo.signin.Blocked\n      - name: initiated\n        creates: demo.signin.SignIn\n",
    )
}

/// Step 1 against step 2 and step 5: `when_related`, an input refusal and an accepting input
/// branch (`fast-path`), each overlapping `exists: false`.
fn related_refusal_and_accepting() -> String {
    edit(
        &related_and_refusal().replace(
            "predicate: redirect_client != input.client",
            "predicate: redirect_client == \"revoked\"",
        ),
        "      - name: initiated\n        creates: demo.signin.SignIn\n",
        "      - name: fast-path
        when: client == \"console\"
        creates: demo.signin.SignIn
        instance: sign_in_id
        emits: [demo.signin.SignInInitiated]
        payload:
          demo.signin.SignInInitiated: {sign_in_id: {generated: true}}
        sets:
          tenant: input.tenant
          client: input.client
      - name: initiated
        creates: demo.signin.SignIn
",
    )
}

/// Step 1 (`existing_instance` then `exists: false`) against step 2.
const ORDERS: &str = "format: ess/18
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - {name: demo.orders.CustomerId, kind: newtype, of: Uuid}
entities:
  - name: demo.orders.Customer
    identity: {name: customer_id, type: demo.orders.CustomerId}
    fields: []
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: customer, type: demo.orders.CustomerId}
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.orders.CustomerAdded
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
  - name: demo.orders.OrderPlaced
    fields:
      - {name: order_id, type: demo.orders.OrderId}
errors:
  - {name: demo.orders.OrderExists, summary: The order id is taken., fields: []}
  - {name: demo.orders.NoCustomer, summary: The customer does not exist., fields: []}
  - {name: demo.orders.BadNote, summary: The note is refused., fields: []}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.AddCustomer, demo.orders.PlaceOrder]}
commands:
  - name: demo.orders.AddCustomer
    input: []
    outcomes:
      - name: added
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerAdded]
        payload:
          demo.orders.CustomerAdded: {customer_id: {generated: true}}
  - name: demo.orders.PlaceOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: customer, type: demo.orders.CustomerId}
      - {name: note, type: String}
    outcomes:
      - {name: duplicate, existing_instance: true, error: demo.orders.OrderExists}
      - name: no-customer
        when_related: {via: input.customer, exists: false}
        error: demo.orders.NoCustomer
      - name: bad-note
        when: note == \"spam\"
        error: demo.orders.BadNote
      - name: placed
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.OrderPlaced]
        payload:
          demo.orders.OrderPlaced: {order_id: input.order_id}
        sets:
          customer: input.customer
          note: input.note
views:
  - name: demo.orders.Customers
    source: demo.orders.Customer
    consistency: read_your_writes
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: customer, type: demo.orders.CustomerId}
      - {name: note, type: String}
";

/// Step 2 against step 3: an input refusal beside `unknown_instance`.
const NOTES: &str = "format: ess/18
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: body, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.notes.Written
    fields:
      - {name: note_id, type: demo.notes.NoteId}
  - name: demo.notes.Edited
    fields:
      - {name: note_id, type: demo.notes.NoteId}
errors:
  - {name: demo.notes.Blank, summary: The body is blank., fields: []}
  - {name: demo.notes.NoSuchNote, summary: No note carries the id., fields: []}
actors:
  - {name: demo.notes.Writer, may: [demo.notes.Write, demo.notes.Edit]}
commands:
  - name: demo.notes.Write
    input: []
    outcomes:
      - name: written
        creates: demo.notes.Note
        instance: note_id
        sets: {body: \"first body\"}
        emits: [demo.notes.Written]
        payload:
          demo.notes.Written: {note_id: {generated: true}}
  - name: demo.notes.Edit
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: body, type: String}
    outcomes:
      - name: blank
        when: body == \"\"
        error: demo.notes.Blank
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {body: input.body}
        emits: [demo.notes.Edited]
        payload:
          demo.notes.Edited: {note_id: input.note_id}
      - {name: missing, unknown_instance: true, error: demo.notes.NoSuchNote}
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: body, type: String}
";

/// `refusal-beside-state.yaml` with a closed `mode` input; `refusals` is inserted after
/// `too-short`, and `accepting` before `rotated`.
fn rotate_with(refusals: &str, accepting: &str) -> String {
    let text = edit(
        ROTATE,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze, Pause]}\n",
    );
    let text = edit(
        &text,
        "      - {name: secret, type: String}\n    outcomes:\n",
        "      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n    outcomes:\n",
    );
    edit(
        &text,
        "        error: demo.secrets.SecretTooShort\n      - name: rotated\n",
        &format!("        error: demo.secrets.SecretTooShort\n{refusals}{accepting}      - name: rotated\n"),
    )
}

/// Step 2 against step 4: an input refusal beside a held state.
fn held_and_refusal() -> String {
    rotate_with(
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.NotPending\n",
        "",
    )
}

/// Two refusals on a held-state command: `frozen` declared first, `halted` claiming `Freeze` too.
fn held_two_refusals() -> String {
    rotate_with(
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.NotPending\n      - name: halted\n        when: mode != Rotate\n        error: demo.secrets.NotActive\n",
        "",
    )
}

/// Two refusals on a plain command, the narrower declared first.
fn plain_two_refusals() -> String {
    TICKETS
        .replace(
            "      - {name: open, type: Boolean}\n",
            "      - {name: open, type: Boolean}\n      - {name: count, type: Integer}\n",
        )
        .replace(
            "        error: demo.tickets.TicketIdRequired\n",
            "        error: demo.tickets.TicketIdRequired\n      - name: very-few\n        when: count < 2\n        error: demo.tickets.VeryFew\n      - name: too-few\n        when: count < 5\n        error: demo.tickets.TooFew\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - {name: demo.tickets.TooFew, summary: Below five., fields: []}\n  - {name: demo.tickets.VeryFew, summary: Below two., fields: []}\n",
        )
}

/// Step 2 against step 5: an input refusal, an accepting default and an external branch whose
/// guards it overlaps.
const MAIL: &str = r#"format: ess/18
system: delivery
version: v1
domain: delivery.mail
events:
  - name: delivery.mail.Sent
    fields: []
errors:
  - name: delivery.mail.Rejected
    fields: []
  - name: delivery.mail.Bounced
    fields: []
commands:
  - name: delivery.mail.Send
    input:
      - {name: retry, type: Boolean}
      - {name: recipient, type: String}
    outcomes:
      - name: bounced
        when: recipient == "bounce"
        error: delivery.mail.Bounced
      - name: rejected
        when:
          all: [retry == false, recipient != ""]
        external: the provider rejects the initial delivery
        error: delivery.mail.Rejected
      - name: sent
        emits: [delivery.mail.Sent]
"#;

fn combinations() -> Vec<(&'static str, &'static str, String)> {
    vec![
        (
            "when_related + input refusal",
            "demo.signin.InitiateSignIn",
            related_and_refusal(),
        ),
        (
            "when_related + input refusal + accepting input branch",
            "demo.signin.InitiateSignIn",
            related_refusal_and_accepting(),
        ),
        (
            "when_related + existing_instance + input refusal",
            "demo.orders.PlaceOrder",
            ORDERS.to_owned(),
        ),
        (
            "input refusal + unknown_instance",
            "demo.notes.Edit",
            NOTES.to_owned(),
        ),
        (
            "input refusal + held state",
            "demo.secrets.RotateSecret",
            held_and_refusal(),
        ),
        (
            "two refusals, held state",
            "demo.secrets.RotateSecret",
            held_two_refusals(),
        ),
        (
            "two refusals, plain command",
            "demo.tickets.SetTicketOpen",
            plain_two_refusals(),
        ),
        (
            "input refusal + accepting + external",
            "delivery.mail.Send",
            MAIL.to_owned(),
        ),
    ]
}

/// Validation admits each combination, and synthesis and the interpreter answer every request of
/// the suite alike: no scenario of the command fails or errors against the interpreted target.
#[test]
fn validation_synthesis_and_the_interpreter_answer_every_combination_alike() {
    let mut disagreements = Vec::new();
    for (name, command, text) in combinations() {
        let model = ir(&text);
        let result = ess_conformance::synthesize::synthesize(&model);
        let admitted = AdmittedSuite::from_suite(&result.suite)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let target = ess_conformance::interpret::Interpreted::for_model(model);
        let report = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &target)
            .into_report();
        let mut ran = 0;
        // Outcome checks of `command` the interpreter answered as synthesis required. A scenario
        // may still end `unsupported` where the interpreted target reads no view.
        let mut agreed = 0;
        for run in &report.scenarios {
            if !run.scenario.to_string().starts_with(command) {
                continue;
            }
            ran += 1;
            agreed += run
                .checks
                .iter()
                .filter(|check| {
                    check.status == Status::Passed
                        && check.about.starts_with(&format!("outcome {command}/"))
                })
                .count();
            if matches!(run.status, Status::Failed | Status::Error) {
                disagreements.push(format!(
                    "{name}: {} {:?}: {:?}",
                    run.scenario, run.status, run.checks
                ));
            }
        }
        assert!(
            ran > 0,
            "{name}: no scenario of {command} ran; refusals: {:#?}",
            refusals(&result)
        );
        // The interpreter declines a `when_related` command declaring `existing_instance:`
        // (documented), so that row checks only that nothing disagrees.
        if !name.contains("existing_instance") && agreed == 0 {
            disagreements.push(format!(
                "{name}: the interpreter answers no request of {command}"
            ));
        }
    }
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

// ---- a branch every input of which an earlier refusal claims ------------------------------------

/// Every outcome of `command` is either a scenario of the suite or a named synthesis refusal.
fn named_or_refused(result: &Synthesis, command: &str, outcome: &str) -> bool {
    let id = format!("{command}/outcome/{outcome}");
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
        || refusals(result).iter().any(|refusal| refusal.contains(&id))
}

/// `frozen-again` claims exactly what `frozen`, declared first, claims: it never answers. It is
/// admitted (first declared), so synthesis must say so by name — as #217 does for a shadowed
/// external branch — rather than file nothing for it.
#[test]
fn a_refusal_every_input_of_which_an_earlier_refusal_claims_is_named() {
    let held = rotate_with(
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.NotPending\n      - name: frozen-again\n        when: mode == Freeze\n        error: demo.secrets.NotActive\n",
        "",
    );
    let result = ess_conformance::synthesize::synthesize(&ir(&held));
    assert!(
        named_or_refused(&result, "demo.secrets.RotateSecret", "frozen-again"),
        "held state: frozen-again is neither filed nor refused: {:#?}",
        refusals(&result)
    );
    let plain = plain_two_refusals().replace("when: count < 5", "when: count < 1");
    let result = ess_conformance::synthesize::synthesize(&ir(&plain));
    assert!(
        named_or_refused(&result, "demo.tickets.SetTicketOpen", "too-few"),
        "plain: too-few (count < 1, inside very-few count < 2) is neither filed nor refused: {:#?}",
        refusals(&result)
    );
}

/// An accepting held-state branch every input of which a refusal claims (`paused-rotation` reads
/// `mode == Freeze` only, which `frozen` takes first): named by synthesis, not dropped.
#[test]
fn an_accepting_branch_every_input_of_which_a_refusal_claims_is_named() {
    let text = rotate_with(
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.NotPending\n",
        "      - name: frozen-rotation
        when_subject_state: Configured
        when: mode == Freeze
        updates: demo.secrets.Configuration
        instance: tenant_id
        sets: {secret: input.secret}
        emits: [demo.secrets.SecretRotated]
        payload: {demo.secrets.SecretRotated: {tenant_id: input.tenant_id}}
",
    );
    let text = text.replace(
        "      - name: rotated\n        when_subject_state: Configured\n",
        "      - name: rotated\n        when_subject_state: Configured\n        when: mode != Freeze\n",
    );
    let Ok(raw) = RawSpecFile::parse(&text) else {
        panic!("parses")
    };
    if Specification::assemble([(Source::new("model.yaml"), raw)]).is_err() {
        return; // refused by validation: named, which is the requirement
    }
    let result = ess_conformance::synthesize::synthesize(&ir(&text));
    assert!(
        named_or_refused(&result, "demo.secrets.RotateSecret", "frozen-rotation"),
        "frozen-rotation is neither filed nor refused: {:#?}",
        refusals(&result)
    );
}

// ---- the missing-row witness beside an input refusal and an accepting branch --------------------

const NO_CONFIGURATION: &str = "demo.signin.InitiateSignIn/outcome/no-configuration";

fn client_sent(result: &Synthesis) -> Vec<Option<ScenarioValue>> {
    let (_, scenario) = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == NO_CONFIGURATION)
        .unwrap_or_else(|| panic!("{NO_CONFIGURATION} is filed: {:#?}", refusals(result)));
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.signin.InitiateSignIn" =>
            {
                Some(input.get("client").cloned())
            }
            _ => None,
        })
        .collect()
}

/// #211 F3/F4 required the missing-row witness to carry the input `fast-path` takes, so a target
/// taking `fast-path` before reading the configuration fails. Correction 1 prefers a refusal's
/// input: beside `blocked`, the `fast-path` overlap must still be sent.
#[test]
fn the_missing_row_witness_still_carries_the_accepting_branch_input_beside_a_refusal() {
    let result = ess_conformance::synthesize::synthesize(&ir(&related_refusal_and_accepting()));
    let sent = client_sent(&result);
    let console = Some(ScenarioValue::literal(Node::Text("console".into())));
    let blocked = Some(ScenarioValue::literal(Node::Text("blocked".into())));
    assert!(
        sent.contains(&blocked),
        "the refusal's input is sent for a missing row: {sent:?}"
    );
    assert!(
        sent.contains(&console),
        "the accepting branch's input is not sent for a missing row: {sent:?}"
    );
}

/// The same finding as behaviour: a target answering `fast-path` before the missing row (and
/// otherwise the order) must fail some scenario; the correct target must pass them all.
#[test]
fn a_target_taking_the_accepting_branch_before_the_missing_row_fails() {
    let result = ess_conformance::synthesize::synthesize(&ir(&related_refusal_and_accepting()));
    assert!(
        refusals(&result)
            .iter()
            .all(|refusal| !refusal.contains("InitiateSignIn")),
        "every InitiateSignIn branch is filed: {:#?}",
        refusals(&result)
    );
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let run = |accepting_first: bool| {
        Runner::for_suite(admitted.suite())
            .run_admitted(
                &admitted,
                &SignIn {
                    accepting_first,
                    ..SignIn::default()
                },
            )
            .into_report()
            .scenarios
            .into_iter()
            .filter(|run| run.scenario.to_string().contains("InitiateSignIn"))
            .filter(|run| run.status != Status::Passed)
            .map(|run| format!("{} {:?}: {:?}", run.scenario, run.status, run.checks))
            .collect::<Vec<_>>()
    };
    let correct = run(false);
    assert!(correct.is_empty(), "the correct target fails: {correct:#?}");
    let mutant = run(true);
    assert!(
        !mutant.is_empty(),
        "a target answering `fast-path` for client \"console\" before reading the configuration \
         passes every InitiateSignIn scenario"
    );
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

#[derive(Default)]
struct SignIn {
    accepting_first: bool,
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
        Ok(ImplementationIdentity::new("sign-in-pass2", "1"))
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
                let create = |name: &str| {
                    let id = self.mint();
                    self.sign_ins.borrow_mut().push(BTreeMap::from([
                        ("sign_in_id".to_owned(), Node::Text(id.clone())),
                        ("tenant".to_owned(), Node::Text(tenant.clone())),
                        ("client".to_owned(), Node::Text(client.clone())),
                        ("state".to_owned(), Node::Text("Initiated".to_owned())),
                    ]));
                    SemanticCommandResult::took(branch(&command, name)).emitting(
                        ObservedEvent::new(
                            "demo.signin.SignInInitiated".parse::<EventRef>().unwrap(),
                        )
                        .with("sign_in_id", Node::Text(id)),
                    )
                };
                if self.accepting_first && client == "console" {
                    create("fast-path")
                } else {
                    match registered {
                        None => refuse("no-configuration", "demo.signin.NoConfiguration"),
                        Some(_) if client == "blocked" => refuse("blocked", "demo.signin.Blocked"),
                        Some(registered) if registered == "revoked" => {
                            refuse("no-redirect-entry", "demo.signin.NoRedirectEntry")
                        }
                        Some(_) if client == "console" => create("fast-path"),
                        Some(_) => create("initiated"),
                    }
                }
            }
            _ => SemanticCommandResult::undeclared(),
        };
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        let token =
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.mint()))
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
