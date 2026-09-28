//! Adversary pass 1 against the arrangement unit (beyond10x/ess#198, #209, #199).
//!
//! The unit's own tests read the synthesized steps. These cases run the suites: against a
//! hand-written target that answers the model as `docs/design/outcome-shapes.md` "Precedence"
//! says, and against mutants of it the arranged half exists to catch. The rest hold the claims on
//! shapes the unit's fixtures do not cover: three creations, a guarded creation, an owned subject,
//! a model with no view, and byte-identical output.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner,
    ScenarioStep,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const TWO_CREATORS: &str = include_str!("fixtures/arrangement-two-creators.yaml");
const INPUT_REFUSAL: &str = include_str!("fixtures/arrangement-input-refusal.yaml");

fn synthesis(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}"));
    ess_conformance::synthesize::synthesize(&ir)
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

fn ids(suite: &ConformanceSuite) -> Vec<String> {
    suite.scenarios.keys().map(ToString::to_string).collect()
}

fn find<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    find(&result.suite, id).unwrap_or_else(|| {
        panic!(
            "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
            ids(&result.suite),
            refusals(result)
        )
    })
}

fn sent(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => {
                Some(command.to_string().rsplit('.').next().unwrap().to_owned())
            }
            _ => None,
        })
        .collect()
}

// ---- #209 run against a target -------------------------------------------------------------

/// How the hand-written account service answers `Configure`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Input refusals first, in declaration order, then existence, then the move (the documented
    /// precedence).
    Correct,
    /// [`Mode::Correct`] with the two overlapping refusals checked the other way round. The model
    /// orders neither before the other beside a default (`input-guard-overlap-precedence.md`,
    /// "two input-guarded refusals: unchanged"), so this is as faithful as `Correct`.
    CorrectOtherOrder,
    /// Looks the account up before it checks the input: an unknown id is "not found".
    ExistenceFirst,
    /// Checks the input only for an account it does not hold.
    LaxOnceStored,
    /// Refuses, and writes the sent issuer onto the stored account anyway.
    WritesOnRefusal,
    /// Refuses, and publishes `Configured` for the stored account anyway.
    EmitsOnRefusal,
}

struct Accounts {
    mode: Mode,
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Accounts {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            minted: Cell::new(0),
            rows: RefCell::default(),
            published: RefCell::default(),
        }
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

impl ConformanceTarget for Accounts {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("accounts-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
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
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let refuse = |name: &str, error: &str| {
            SemanticCommandResult::took(branch(&command, name))
                .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
        };
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "vault.acct.Onboard" => {
                let id = format!("acct-{n}");
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("account_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("New".to_owned())),
                        ("issuer".to_owned(), Node::Text(String::new())),
                    ]),
                );
                let event = ObservedEvent::new("vault.acct.Onboarded".parse::<EventRef>().unwrap())
                    .with("account_id", Node::Text(id));
                SemanticCommandResult::took(branch(&command, "onboarded")).emitting(event)
            }
            "vault.acct.Configure" => {
                let id = text(request.input.get("id"));
                let secret = text(request.input.get("secret"));
                let issuer = text(request.input.get("issuer"));
                let stored = rows.contains_key(&id);
                if self.mode == Mode::ExistenceFirst && !stored {
                    return Ok(refuse("already-configured", "vault.acct.AlreadyConfigured")
                        .with_consistency(token));
                }
                let lax = self.mode == Mode::LaxOnceStored && stored;
                let short = !lax && secret.chars().count() < 12;
                let missing = !lax && issuer.is_empty();
                let refused = if id.is_empty() {
                    Some(("id-required", "vault.acct.IdRequired"))
                } else if self.mode == Mode::CorrectOtherOrder && missing {
                    Some(("missing-configuration", "vault.acct.MissingConfiguration"))
                } else if short {
                    Some(("secret-too-short", "vault.acct.SecretTooShort"))
                } else if missing {
                    Some(("missing-configuration", "vault.acct.MissingConfiguration"))
                } else {
                    None
                };
                if let Some((name, error)) = refused {
                    let mut answer = refuse(name, error);
                    if stored {
                        match self.mode {
                            Mode::WritesOnRefusal => {
                                rows.get_mut(&id)
                                    .unwrap()
                                    .insert("issuer".to_owned(), Node::Text(issuer));
                            }
                            Mode::EmitsOnRefusal => {
                                answer = answer.emitting(
                                    ObservedEvent::new(
                                        "vault.acct.Configured".parse::<EventRef>().unwrap(),
                                    )
                                    .with("account_id", Node::Text(id)),
                                );
                            }
                            _ => {}
                        }
                    }
                    answer
                } else {
                    match rows.get_mut(&id) {
                        // The wrong-state branch answers an identity nothing stored, as the
                        // synthesized `already-configured` scenario requires.
                        None => refuse("already-configured", "vault.acct.AlreadyConfigured"),
                        Some(row) if row["state"] != Node::Text("New".to_owned()) => {
                            refuse("already-configured", "vault.acct.AlreadyConfigured")
                        }
                        Some(row) => {
                            row.insert("state".to_owned(), Node::Text("Configured".to_owned()));
                            row.insert("issuer".to_owned(), Node::Text(issuer));
                            SemanticCommandResult::took(branch(&command, "configured")).emitting(
                                ObservedEvent::new(
                                    "vault.acct.Configured".parse::<EventRef>().unwrap(),
                                )
                                .with("account_id", Node::Text(id)),
                            )
                        }
                    }
                }
            }
            other => panic!("unexpected command {other}"),
        };
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.rows.borrow().values().cloned().collect::<Vec<_>>(),
        ))
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

const SECRET: &str = "vault.acct.Configure/outcome/secret-too-short";
const MISSING: &str = "vault.acct.Configure/outcome/missing-configuration";
const ID_REQUIRED: &str = "vault.acct.Configure/outcome/id-required";

/// Every `Configure` scenario that did not pass against `mode`, with its checks.
fn configure_failures(mode: Mode) -> BTreeMap<String, String> {
    let result = synthesis(INPUT_REFUSAL);
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Accounts::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| {
            run.scenario
                .to_string()
                .starts_with("vault.acct.Configure/")
        })
        .filter(|run| run.status != Status::Passed)
        .map(|run| {
            (
                run.scenario.to_string(),
                format!("{:?}: {:?}", run.status, run.checks),
            )
        })
        .collect()
}

#[test]
fn issue_209_the_refusal_scenarios_pass_a_target_answering_input_before_existence() {
    let failed = configure_failures(Mode::Correct);
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn issue_209_the_refusal_scenarios_pass_the_same_target_with_the_open_order_taken_the_other_way() {
    let failed = configure_failures(Mode::CorrectOtherOrder);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The model's own interpreter, asked about every literal `Configure` input a refusal scenario
/// sends: the input must select exactly one declared outcome, or the scenario requires a choice
/// the model does not make.
#[test]
fn issue_209_every_literal_refused_input_selects_exactly_one_outcome_under_the_interpreter() {
    use ess_conformance::interpret::execute::{execute, Externals, Store};
    use ess_conformance::ScenarioValue;
    let raw = RawSpecFile::parse(INPUT_REFUSAL).unwrap();
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)]).unwrap();
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    let command = "vault.acct.Configure".parse().unwrap();
    let mut open = Vec::new();
    for id in [SECRET, MISSING] {
        for step in &scenario(&result, id).steps {
            let ScenarioStep::ExecuteCommand { input, .. } = step else {
                continue;
            };
            let literal: Option<BTreeMap<String, Node>> = input
                .iter()
                .map(|(field, value)| match value {
                    ScenarioValue::Literal { value } => Some((field.clone(), value.clone())),
                    _ => None,
                })
                .collect();
            // Only complete literal requests: an arranged send carries an instance, and a send
            // leaving a field out is another family's witness.
            let Some(literal) = literal.filter(|literal| literal.len() == 3) else {
                continue;
            };
            let steps = execute(
                &ir,
                &Store::default(),
                &command,
                &literal,
                &Externals::Withheld,
            )
            .unwrap_or_else(|why| panic!("{why}"));
            if steps.len() != 1 {
                let outcomes: Vec<String> = steps
                    .iter()
                    .map(|step| format!("{:?}", step.outcome))
                    .collect();
                open.push(format!("{id}: {literal:?} -> {outcomes:?}"));
            }
        }
    }
    assert!(open.is_empty(), "{open:#?}");
}

#[test]
fn issue_209_a_target_looking_the_record_up_first_fails_every_input_refusal() {
    let failed = configure_failures(Mode::ExistenceFirst);
    for id in [SECRET, MISSING, ID_REQUIRED] {
        assert!(failed.contains_key(id), "{id} passed: {failed:#?}");
    }
}

#[test]
fn issue_209_a_target_skipping_the_input_check_for_a_stored_record_fails() {
    let failed = configure_failures(Mode::LaxOnceStored);
    for id in [SECRET, MISSING] {
        assert!(failed.contains_key(id), "{id} passed: {failed:#?}");
    }
}

#[test]
fn issue_209_a_target_writing_the_stored_row_on_refusal_fails() {
    let failed = configure_failures(Mode::WritesOnRefusal);
    // `missing-configuration` is refused for `issuer == ""`, and nothing but `Configure` writes
    // `issuer`, so the arranged row already holds whatever the target starts it with. Writing
    // `""` over it is invisible to any suite. Only `secret-too-short` carries an issuer that differs.
    for id in [SECRET] {
        assert!(failed.contains_key(id), "{id} passed: {failed:#?}");
    }
}

#[test]
fn issue_209_a_target_publishing_on_refusal_for_a_stored_record_fails() {
    let failed = configure_failures(Mode::EmitsOnRefusal);
    for id in [SECRET, MISSING] {
        assert!(failed.contains_key(id), "{id} passed: {failed:#?}");
    }
}

// ---- #209 on shapes the unit's fixture does not carry ---------------------------------------

/// The fixture with its only view removed: nothing reads an account back.
fn without_view() -> String {
    let cut = INPUT_REFUSAL
        .find("views:")
        .expect("the fixture declares a view");
    INPUT_REFUSAL[..cut].to_owned()
}

/// The plain send needs no record and no view: the refused input for an identity nothing stored.
/// The base files it. The unit's doc withdraws a scenario only "if no arrangement reaches that
/// record"; here the arrangement reaches it and only the row observation is missing, so the plain
/// half the base witnessed must not be lost with it.
#[test]
fn issue_209_a_model_with_no_view_keeps_the_plain_send_of_an_input_refusal() {
    let result = synthesis(&without_view());
    for id in [SECRET, MISSING] {
        let found = find(&result.suite, id);
        assert!(
            found.is_some(),
            "{id} withdrawn although the arrangement reaches the record: {:#?}",
            refusals(&result)
        );
    }
}

/// An owned subject: the arranged half has to arrange the owner, then the record, then send.
const OWNED: &str = r#"format: ess/14
system: ledger
version: v1
domain: ledger.book
types:
  - {name: ledger.book.AccountId, kind: newtype, of: Uuid}
  - {name: ledger.book.EntryId, kind: newtype, of: Uuid}
entities:
  - name: ledger.book.Account
    identity: {name: account_id, type: ledger.book.AccountId}
    fields:
      - {name: holder, type: String}
    relations:
      - {name: entries, kind: owns, target: ledger.book.Entry, cardinality: many, via: account_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: ledger.book.Entry
    identity: {name: entry_id, type: ledger.book.EntryId}
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    lifecycle: {initial: Posted, states: [Posted], terminal: [Posted]}
actors:
  - name: ledger.book.Clerk
    may: [ledger.book.OpenAccount, ledger.book.PostEntry, ledger.book.AmendEntry]
errors:
  - {name: ledger.book.EmptyMemo, fields: []}
events:
  - name: ledger.book.AccountOpened
    fields: [{name: account_id, type: ledger.book.AccountId}]
  - name: ledger.book.EntryPosted
    fields: [{name: entry_id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryAmended
    fields: [{name: entry_id, type: ledger.book.EntryId}]
commands:
  - name: ledger.book.OpenAccount
    input: [{name: holder, type: String}]
    outcomes:
      - name: opened
        creates: ledger.book.Account
        instance: account_id
        sets: {holder: input.holder}
        emits: [ledger.book.AccountOpened]
        payload:
          ledger.book.AccountOpened: {account_id: {generated: true}}
  - name: ledger.book.PostEntry
    input:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    outcomes:
      - name: posted
        creates: ledger.book.Entry
        instance: entry_id
        sets: {account_id: input.account_id, memo: input.memo}
        emits: [ledger.book.EntryPosted]
        payload:
          ledger.book.EntryPosted: {entry_id: {generated: true}}
  - name: ledger.book.AmendEntry
    input:
      - {name: entry_id, type: ledger.book.EntryId}
      - {name: memo, type: String}
    outcomes:
      - {name: empty-memo, when: memo == "", error: ledger.book.EmptyMemo}
      - name: amended
        updates: ledger.book.Entry
        instance: entry_id
        sets: {memo: input.memo}
        emits: [ledger.book.EntryAmended]
        payload:
          ledger.book.EntryAmended: {entry_id: input.entry_id}
views:
  - name: ledger.book.Entries
    source: ledger.book.Entry
    consistency: read_your_writes
    fields:
      - {name: entry_id, type: ledger.book.EntryId}
      - {name: state, type: ledger.book.Entry.State}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
"#;

#[test]
fn issue_209_an_owned_record_is_arranged_with_its_owner_before_the_refused_send() {
    let result = synthesis(OWNED);
    let refused = scenario(&result, "ledger.book.AmendEntry/outcome/empty-memo");
    let commands = sent(refused);
    assert_eq!(
        commands,
        ["AmendEntry", "OpenAccount", "PostEntry", "AmendEntry"],
        "{:#?}",
        refusals(&result)
    );
}

// ---- #198 on three creations, a guarded one, and declaration order ---------------------------

const SHIPPED: &str = "shop.order.ShipOrder/outcome/shipped";
const NOT_PAID: &str = "shop.order.ShipOrder/outcome/not-paid";
const CLOSE: &str = "shop.order.Order/transition/close/by/shop.order.CloseOrder/closed";

/// A third creation `PlaceOrderC`, inserted after `PlaceOrderB`, carrying `paid`. `PlaceOrderB` is
/// made unpaid, so only the third-declared creation's row selects `shipped`.
fn three_creators(guarded: bool) -> String {
    let third = if guarded {
        "  - name: shop.order.PlaceOrderC
    input:
      - {name: note, type: String}
    outcomes:
      - {name: blank, when: note == \"\", error: shop.order.Conflict}
      - name: placed
        creates: shop.order.Order
        instance: order_id
        sets: {paid: \"true\"}
        emits: [shop.order.OrderPlaced]
        payload:
          shop.order.OrderPlaced: {order_id: {generated: true}}
"
    } else {
        "  - name: shop.order.PlaceOrderC
    input:
      - {name: note, type: String}
    outcomes:
      - name: placed
        creates: shop.order.Order
        instance: order_id
        sets: {paid: \"true\"}
        emits: [shop.order.OrderPlaced]
        payload:
          shop.order.OrderPlaced: {order_id: {generated: true}}
"
    };
    let text = TWO_CREATORS
        .replace("sets: {paid: \"true\"}", "sets: {paid: \"false\"}")
        .replace(
            "shop.order.PlaceOrderB, ",
            "shop.order.PlaceOrderB, shop.order.PlaceOrderC, ",
        )
        .replace(
            "  - name: shop.order.ShipOrder\n",
            &format!("{third}  - name: shop.order.ShipOrder\n"),
        );
    assert!(
        text.contains("PlaceOrderC, shop.order.ShipOrder"),
        "actor grant rewritten"
    );
    assert!(
        text.contains("name: shop.order.PlaceOrderC\n"),
        "creation inserted"
    );
    text
}

#[test]
fn issue_198_only_the_third_creation_selects_the_branch_and_it_is_witnessed() {
    let result = synthesis(&three_creators(false));
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    assert_eq!(
        sent(scenario(&result, SHIPPED)),
        ["PlaceOrderC", "ShipOrder"]
    );
    // Two creations leave an unpaid row; command-name order picks the first by name.
    assert_eq!(
        sent(scenario(&result, NOT_PAID)),
        ["ShipOrder", "PlaceOrderA", "ShipOrder"]
    );
    assert!(
        sent(scenario(&result, CLOSE)).contains(&"CloseOrder".to_owned()),
        "{:#?}",
        refusals(&result)
    );
}

#[test]
fn issue_198_a_guarded_third_creation_is_sent_with_the_input_its_guard_admits() {
    let result = synthesis(&three_creators(true));
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    let shipped = scenario(&result, SHIPPED);
    assert_eq!(sent(shipped), ["PlaceOrderC", "ShipOrder"]);
    // The creation is required to take `placed`, not `blank`.
    let placed = shipped.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome }
            if outcome.to_string() == "shop.order.PlaceOrderC/placed")
    });
    assert!(placed, "{:#?}", shipped.steps);
}

/// Command-name order (the #198 decision as revised after adversary pass 1, `decisions.md`
/// "Revisions after implementation": the IR keeps commands by name): the names are exchanged and
/// the positions kept, so the creation declared first is now called `PlaceOrderB`. The choice
/// stays with `PlaceOrderA`, first by name and now second declared.
#[test]
fn issue_198_exchanging_two_unpaid_creations_keeps_the_choice_on_the_first_by_name() {
    let text = three_creators(false);
    // Rename A <-> B: the one declared first is now called PlaceOrderB.
    let swapped = text
        .replace("PlaceOrderA", "PLACEHOLDER")
        .replace("PlaceOrderB", "PlaceOrderA")
        .replace("PLACEHOLDER", "PlaceOrderB");
    let result = synthesis(&swapped);
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    assert_eq!(
        sent(scenario(&result, NOT_PAID)),
        ["ShipOrder", "PlaceOrderA", "ShipOrder"],
        "the revised #198 decision tries creations in command-name order, not declaration order"
    );
}

/// The same question on a shape the base also synthesizes: no stored guard, so every arrangement
/// is `arrange`'s route search. Both creations leave the same row and `PlaceOrderB` is declared
/// first; under the revised #198 decision (command-name order) the tie goes to `PlaceOrderA`.
#[test]
fn a_route_tie_goes_to_the_creation_first_by_name_not_the_first_declared() {
    let a_block_start = TWO_CREATORS
        .find("  - name: shop.order.PlaceOrderA\n")
        .unwrap();
    let b_block_start = TWO_CREATORS
        .find("  - name: shop.order.PlaceOrderB\n")
        .unwrap();
    let ship_start = TWO_CREATORS
        .find("  - name: shop.order.ShipOrder\n")
        .unwrap();
    let a_block = &TWO_CREATORS[a_block_start..b_block_start];
    let b_block = &TWO_CREATORS[b_block_start..ship_start];
    // B declared before A, and ShipOrder's stored guard dropped so no search is involved.
    let text = format!(
        "{}{}{}{}",
        &TWO_CREATORS[..a_block_start],
        b_block,
        a_block,
        &TWO_CREATORS[ship_start..]
    )
    .replace(
        "      - name: not-paid
        when_subject:
          predicate: paid == false
        error: shop.order.NotPaid
",
        "",
    );
    assert!(!text.contains("not-paid"), "guard dropped");
    let result = synthesis(&text);
    let close = sent(scenario(&result, CLOSE));
    assert_eq!(
        close.first().map(String::as_str),
        Some("PlaceOrderA"),
        "the revised #198 decision breaks a route tie in command-name order: {close:?}"
    );
}

// ---- determinism ------------------------------------------------------------------------------

#[test]
fn every_new_shape_synthesizes_byte_identically_twice() {
    for text in [
        TWO_CREATORS.to_owned(),
        three_creators(false),
        three_creators(true),
        INPUT_REFUSAL.to_owned(),
        OWNED.to_owned(),
    ] {
        let one = serde_json::to_string(&synthesis(&text).suite).unwrap();
        let two = serde_json::to_string(&synthesis(&text).suite).unwrap();
        assert_eq!(one, two);
    }
}
