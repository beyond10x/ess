//! Adversary, pass 2, for synthesis of a `when_subject` guard comparing a link field with an input
//! (beyond10x/ess#193, the guard case), after correction 1.
//!
//! Correction 1 gives the second owner a row of its own (`row_under`) and routes every guard of a
//! link-comparing command through the input-bound search. These cases ask whether that row, or that
//! routing, breaks a shape the unit's own tests and pass 1 do not build: an accepting branch that
//! moves the row to the named owner under `cardinality: one`, and the link guard beside a
//! state-scoped refusal (#201) or an input comparison (#157) on the same command.
#![allow(clippy::too_many_lines, clippy::similar_names, clippy::match_same_arms)]
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::{ScenarioId, ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};

/// `{cardinality}` is the owner relation's cardinality, `{commands}` the commands under test.
const MODEL: &str = r"
format: ess/18
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
      - {name: entries, kind: owns, target: ledger.book.Entry, cardinality: {cardinality}, via: account_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: ledger.book.Entry
    identity: {name: id, type: ledger.book.EntryId}
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    lifecycle: {initial: Posted, states: [Posted], terminal: [Posted]}
errors:
  - {name: ledger.book.OtherAccount, fields: []}
  - {name: ledger.book.SameAccount, fields: []}
  - {name: ledger.book.Voided, fields: []}
events:
  - name: ledger.book.AccountOpened
    fields: [{name: account_id, type: ledger.book.AccountId}]
  - name: ledger.book.EntryPosted
    fields: [{name: id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryVoided
    fields: [{name: id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryAmended
    fields: [{name: id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryMoved
    fields: [{name: id, type: ledger.book.EntryId}, {name: account_id, type: ledger.book.AccountId}]
commands:
  - name: ledger.book.OpenAccount
    input: [{name: holder, type: String}]
    outcomes:
      - name: opened
        creates: ledger.book.Account
        instance: account_id
        emits: [ledger.book.AccountOpened]
        payload:
          ledger.book.AccountOpened:
            account_id: {generated: true}
        sets: {holder: input.holder}
  - name: ledger.book.PostEntry
    input:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    outcomes:
      - name: posted
        creates: ledger.book.Entry
        instance: id
        emits: [ledger.book.EntryPosted]
        payload:
          ledger.book.EntryPosted:
            id: {generated: true}
        sets: {account_id: input.account_id, memo: input.memo}
{commands}
views:
  - name: ledger.book.EntryAccounts
    source: ledger.book.Entry
    consistency: read_your_writes
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: state, type: ledger.book.Entry.State}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

/// Moves an entry to the account the caller names, refusing the account it is already filed under.
const TRANSFER: &str = r"
  - name: ledger.book.TransferEntry
    input:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
    outcomes:
      - name: same-account
        when_subject: {predicate: account_id == input.account_id}
        error: ledger.book.SameAccount
      - name: moved
        updates: ledger.book.Entry
        instance: id
        sets: {account_id: input.account_id}
        emits: [ledger.book.EntryMoved]
        payload:
          ledger.book.EntryMoved:
            id: input.id
            account_id: input.account_id
";

/// `{before}` holds branches declared before the link guard, `{guard}` its predicate.
const AMEND: &str = r"
  - name: ledger.book.AmendEntry
    input:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    outcomes:
{before}      - name: other-account
        when_subject:
          predicate: {guard}
        error: ledger.book.OtherAccount
      - name: amended
        updates: ledger.book.Entry
        instance: id
        sets: {memo: input.memo}
        emits: [ledger.book.EntryAmended]
        payload:
          ledger.book.EntryAmended:
            id: input.id
";

/// An entry that can be voided: a second state a guard reading `state` (#204) can be refuted in.
const VOID: &str = r"
  - name: ledger.book.VoidEntry
    input: [{name: id, type: ledger.book.EntryId}]
    outcomes:
      - name: voided
        moves: ledger.book.Entry.void
        instance: id
        emits: [ledger.book.EntryVoided]
        payload:
          ledger.book.EntryVoided:
            id: input.id
      - {name: wrong-state, wrong_state: true, error: ledger.book.Voided}
";

fn model(cardinality: &str, commands: &str) -> String {
    MODEL
        .replace("{cardinality}", cardinality)
        .replace("{commands}\n", commands.trim_start_matches('\n'))
}

/// `model`, with `Entry` voidable.
fn voidable(cardinality: &str, commands: &str) -> String {
    let text = model(cardinality, &format!("{VOID}{commands}"));
    let out = text.replace(
        "    lifecycle: {initial: Posted, states: [Posted], terminal: [Posted]}\n",
        "    lifecycle:\n      initial: Posted\n      states: [Posted, Voided]\n      terminal: [Voided]\n      transitions:\n        - {name: void, from: [Posted], to: Voided}\n",
    );
    assert_ne!(out, text, "the entry's lifecycle is in the model");
    out
}

const POSTED_AND_OTHER: &str = "{all: [state == Posted, account_id != input.account_id]}";

fn amend(before: &str, guard: &str) -> String {
    AMEND.replace("{before}", before).replace("{guard}", guard)
}

fn compiled(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    synthesize(&ir)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.cause.code()))
        .collect()
}

/// One defect an implementation of the guarded command could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Answers as if the link guard were not declared.
    IgnoresGuard,
    /// Answers the link guard's refusal whatever account is named.
    RefusesEvery,
    /// Under `all: [link, memo != input.memo]`, reads the link half only.
    IgnoresMemoComparison,
    /// Under `all: [state == Posted, link]`, reads the link half only.
    IgnoresState,
}

type Row = BTreeMap<String, Node>;

/// The ledger, implemented here and not by the synthesizer, with one defect switched in at a time.
struct Ledger {
    mutant: Mutant,
    /// `cardinality: one`: an account holds at most one entry, so a post or a move into an account
    /// already holding one is not something the specification lets happen.
    one: bool,
    /// The link guard also asks `state == Posted`.
    state_scoped: bool,
    /// The link guard also asks `memo != input.memo`.
    memo_comparison: bool,
    accounts: RefCell<Vec<Row>>,
    entries: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Ledger {
    fn new(mutant: Mutant, one: bool, state_scoped: bool, memo_comparison: bool) -> Self {
        Self {
            mutant,
            one,
            state_scoped,
            memo_comparison,
            accounts: RefCell::default(),
            entries: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn mint(&self) -> Node {
        self.minted.set(self.minted.get() + 1);
        Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
    }

    fn token(&self) -> ConsistencyToken {
        ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap()
    }

    /// Whether `account` holds an entry other than `except`.
    fn holds_other(&self, account: &Node, except: Option<&Node>) -> bool {
        self.entries
            .borrow()
            .iter()
            .any(|row| &row["account_id"] == account && Some(&row["id"]) != except)
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("ledger-adversary-2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.accounts.replace(Vec::new());
        self.entries.replace(Vec::new());
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
        let took = |name: &str| {
            SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                OutcomeName::new(name).unwrap(),
            ))
        };
        let refused = |name: &str, error: &str| {
            took(name).with_error(DeclaredErrorValue::new(error.parse().unwrap()))
        };
        let find = |id: &Node| {
            self.entries
                .borrow()
                .iter()
                .find(|row| &row["id"] == id)
                .cloned()
        };
        let result = match command.to_string().as_str() {
            "ledger.book.OpenAccount" => {
                let id = self.mint();
                let mut row = request.input.clone();
                row.insert("account_id".into(), id.clone());
                self.accounts.borrow_mut().push(row);
                took("opened").emitting(
                    ObservedEvent::new("ledger.book.AccountOpened".parse().unwrap())
                        .with("account_id", id),
                )
            }
            "ledger.book.PostEntry" => {
                let account = request.input["account_id"].clone();
                let exists = self
                    .accounts
                    .borrow()
                    .iter()
                    .any(|row| row["account_id"] == account);
                if !exists || (self.one && self.holds_other(&account, None)) {
                    return Ok(SemanticCommandResult::undeclared());
                }
                let id = self.mint();
                let mut row = request.input.clone();
                row.insert("id".into(), id.clone());
                row.insert("state".into(), Node::Text("Posted".into()));
                self.entries.borrow_mut().push(row);
                took("posted").emitting(
                    ObservedEvent::new("ledger.book.EntryPosted".parse().unwrap()).with("id", id),
                )
            }
            "ledger.book.VoidEntry" => {
                let id = request.input["id"].clone();
                let mut entries = self.entries.borrow_mut();
                let row = entries.iter_mut().find(|row| row["id"] == id);
                // An entry nobody posted is answered as one that cannot be voided.
                let Some(row) = row.filter(|row| row["state"] == Node::Text("Posted".into()))
                else {
                    let token = self.token();
                    return Ok(refused("wrong-state", "ledger.book.Voided").with_consistency(token));
                };
                row.insert("state".into(), Node::Text("Voided".into()));
                took("voided").emitting(
                    ObservedEvent::new("ledger.book.EntryVoided".parse().unwrap()).with("id", id),
                )
            }
            "ledger.book.AmendEntry" => {
                let id = request.input["id"].clone();
                let named = request.input["account_id"].clone();
                let memo = request.input["memo"].clone();
                let refusal = refused("other-account", "ledger.book.OtherAccount");
                let Some(row) = find(&id) else {
                    return Ok(refusal.with_consistency(self.token()));
                };
                let posted = row["state"] == Node::Text("Posted".into());
                let differs = row["account_id"] != named;
                let memo_differs = row["memo"] != memo;
                let refuses = match self.mutant {
                    Mutant::IgnoresGuard => false,
                    Mutant::RefusesEvery => true,
                    Mutant::IgnoresMemoComparison | Mutant::IgnoresState => differs,
                    Mutant::None if self.memo_comparison => differs && memo_differs,
                    Mutant::None if self.state_scoped => differs && posted,
                    Mutant::None => differs,
                };
                if refuses {
                    refusal
                } else {
                    for entry in self.entries.borrow_mut().iter_mut() {
                        if entry["id"] == id {
                            entry.insert("memo".into(), memo.clone());
                        }
                    }
                    took("amended").emitting(
                        ObservedEvent::new("ledger.book.EntryAmended".parse().unwrap())
                            .with("id", id),
                    )
                }
            }
            "ledger.book.TransferEntry" => {
                let id = request.input["id"].clone();
                let named = request.input["account_id"].clone();
                let refusal = refused("same-account", "ledger.book.SameAccount");
                let Some(row) = find(&id) else {
                    return Ok(refusal.with_consistency(self.token()));
                };
                let same = match self.mutant {
                    Mutant::IgnoresGuard => false,
                    Mutant::RefusesEvery => true,
                    _ => row["account_id"] == named,
                };
                if same {
                    refusal
                } else {
                    // `cardinality: one`: the named account may not end up holding two entries.
                    if self.one && self.holds_other(&named, Some(&id)) {
                        return Ok(SemanticCommandResult::undeclared());
                    }
                    for entry in self.entries.borrow_mut().iter_mut() {
                        if entry["id"] == id {
                            entry.insert("account_id".into(), named.clone());
                        }
                    }
                    took("moved").emitting(
                        ObservedEvent::new("ledger.book.EntryMoved".parse().unwrap())
                            .with("id", id)
                            .with("account_id", named),
                    )
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(self.token()))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let fields: &[&str] = match request.view.to_string().as_str() {
            "ledger.book.EntryAccounts" => &["id", "state", "account_id", "memo"],
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(
            self.entries
                .borrow()
                .iter()
                .map(|row| {
                    fields
                        .iter()
                        .map(|field| ((*field).to_owned(), row[*field].clone()))
                        .collect::<Row>()
                })
                .collect::<Vec<Row>>(),
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// The scenarios of `text`'s suite that do not pass against `ledger`, with what they reported.
fn failing(text: &str, ledger: &Ledger) -> BTreeMap<String, String> {
    let synthesis = compiled(text);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, ledger)
        .into_report();
    let failed: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                format!("{:?}", scenario.diagnostics().collect::<Vec<_>>()),
            )
        })
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

// ---- (2) the second owner's row under `cardinality: one` -------------------------------------

const MOVED: &str = "ledger.book.TransferEntry/outcome/moved";

/// Every account a `PostEntry` step of `scenario` filed an entry under before the first
/// `TransferEntry` step sent for an arranged entry, and the account that transfer names.
fn filed_before_transfer(
    synthesis: &Synthesis,
    id: &str,
) -> (Vec<ScenarioValue>, Option<ScenarioValue>) {
    let scenario = synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| panic!("no scenario {id}: {:#?}", refusals(synthesis)));
    let mut filed = Vec::new();
    for step in &scenario.steps {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        match command.to_string().as_str() {
            "ledger.book.PostEntry" => filed.extend(input.get("account_id").cloned()),
            "ledger.book.TransferEntry"
                if matches!(input.get("id"), Some(ScenarioValue::Instance { .. })) =>
            {
                return (filed, input.get("account_id").cloned());
            }
            _ => {}
        }
    }
    (filed, None)
}

/// Under `cardinality: one` an account holds one entry. The branch that moves an entry to the
/// account the caller names is sent the second owner — and correction 1 gives that owner an entry
/// of its own first, so the move files a second entry under it: a state the specification says no
/// account reaches.
#[test]
fn under_cardinality_one_the_moved_branch_names_an_account_holding_no_entry() {
    let synthesis = compiled(&model("one", TRANSFER));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let (filed, named) = filed_before_transfer(&synthesis, MOVED);
    let named = named.expect("the moved scenario transfers an arranged entry");
    assert!(
        !filed.contains(&named),
        "the entry is moved into an account that already holds one, under `cardinality: one`: \
         filed under {filed:?}, moved to {named:?}"
    );
}

/// The same, run: a ledger that keeps `cardinality: one` — no account ever holds two entries —
/// fails the moved scenario the suite synthesizes for it.
#[test]
fn a_ledger_keeping_cardinality_one_passes_the_transfer_suite() {
    let failed = failing(
        &model("one", TRANSFER),
        &Ledger::new(Mutant::None, true, false, false),
    );
    assert!(
        failed.is_empty(),
        "a ledger keeping `cardinality: one` fails the suite synthesized for it: {failed:#?}"
    );
}

/// Control: under `cardinality: many` the same transfer passes and the guard mutants are killed.
#[test]
fn control_the_transfer_under_cardinality_many_passes_and_kills_the_guard_mutants() {
    let text = model("many", TRANSFER);
    assert_eq!(
        failing(&text, &Ledger::new(Mutant::None, false, false, false)),
        BTreeMap::new()
    );
    let ignored = failing(
        &text,
        &Ledger::new(Mutant::IgnoresGuard, false, false, false),
    );
    assert!(
        ignored.contains_key("ledger.book.TransferEntry/outcome/same-account"),
        "{ignored:?}"
    );
    let refused = failing(
        &text,
        &Ledger::new(Mutant::RefusesEvery, false, false, false),
    );
    assert!(refused.contains_key(MOVED), "{refused:?}");
}

// ---- (1) beside a state-scoped refusal (#201) and an input comparison (#157) -----------------

/// `all: [state == Posted, account_id != input.account_id]` (#204) where the entry can be voided:
/// the row refuting the state half — voided, another account named — is reachable, so a target
/// reading the link half only fails. (`when_subject_state:` beside `when_subject` is refused by the
/// compiler, `conflicting_declaration`, so #201 has no shape to combine with.)
#[test]
fn a_link_guard_beside_a_reachable_state_half_kills_a_target_reading_the_link_only() {
    let text = voidable("many", &amend("", POSTED_AND_OTHER));
    let synthesis = compiled(&text);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let ledger = |mutant| Ledger::new(mutant, false, true, false);
    assert_eq!(failing(&text, &ledger(Mutant::None)), BTreeMap::new());
    assert!(!failing(&text, &ledger(Mutant::IgnoresGuard)).is_empty());
    assert!(!failing(&text, &ledger(Mutant::RefusesEvery)).is_empty());
    assert!(
        !failing(&text, &ledger(Mutant::IgnoresState)).is_empty(),
        "a target refusing every other account, voided or not, passed every scenario"
    );
}

/// `all: [account_id != input.account_id, memo != input.memo]`: the second conjunct compares a
/// stored field with an input (#157) in the same guard. A target reading the link half only fails.
#[test]
fn a_link_guard_beside_an_input_comparison_kills_a_target_reading_the_link_only() {
    let text = model(
        "many",
        &amend(
            "",
            "{all: [account_id != input.account_id, memo != input.memo]}",
        ),
    );
    let synthesis = compiled(&text);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let ledger = |mutant| Ledger::new(mutant, false, false, true);
    assert_eq!(failing(&text, &ledger(Mutant::None)), BTreeMap::new());
    assert!(!failing(&text, &ledger(Mutant::IgnoresGuard)).is_empty());
    assert!(
        !failing(&text, &ledger(Mutant::IgnoresMemoComparison)).is_empty(),
        "a target refusing every other account, whatever memo is sent, passed every scenario"
    );
}

// ---- (3), (5) instance names in one scenario, and determinism --------------------------------

/// Every instance name a scenario of `text`'s suite captures more than once, by scenario; and the
/// suite is the same bytes on a second synthesis.
fn captured_twice(text: &str) -> Vec<String> {
    let synthesis = compiled(text);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(
        synthesis.suite.to_canonical_json().unwrap(),
        compiled(text).suite.to_canonical_json().unwrap(),
        "two syntheses differ"
    );
    synthesis
        .suite
        .scenarios
        .iter()
        .filter_map(|(id, scenario)| {
            let mut seen = BTreeSet::new();
            let twice: Vec<String> = scenario
                .steps
                .iter()
                .filter_map(|step| match step {
                    ScenarioStep::CaptureInstance { instance, .. } => Some(instance.to_string()),
                    _ => None,
                })
                .filter(|named| !seen.insert(named.clone()))
                .collect();
            (!twice.is_empty()).then(|| format!("{id}: {twice:?}"))
        })
        .collect()
}

/// No scenario of the link shapes captures one instance name twice: the second owner and its row
/// are arranged once (`present`), and no further row reuses a name the branch's own row holds.
#[test]
fn no_scenario_of_a_link_guard_captures_one_name_twice() {
    let mut doubled = Vec::new();
    for text in [
        model("many", TRANSFER),
        model(
            "many",
            &amend(
                "",
                "{all: [account_id != input.account_id, memo == \"locked\"]}",
            ),
        ),
        model(
            "many",
            &amend(
                "",
                "{any: [account_id != input.account_id, memo == \"locked\"]}",
            ),
        ),
        voidable("many", &amend("", POSTED_AND_OTHER)),
    ] {
        doubled.extend(captured_twice(&text));
    }
    assert!(doubled.is_empty(), "{doubled:#?}");
}

/// Control for origin: the same guard with `memo == "locked"` in place of the link comparison — a
/// shape the base synthesizes — asked the same question.
#[test]
fn control_no_scenario_of_the_same_guard_without_a_link_captures_one_name_twice() {
    let mut doubled = Vec::new();
    for guard in [
        "{all: [state == Posted, memo == \"locked\"]}",
        "{all: [state == Posted, memo != input.memo]}",
        "{all: [state == Posted, memo != \"locked\"]}",
    ] {
        doubled.extend(captured_twice(&voidable("many", &amend("", guard))));
    }
    assert!(doubled.is_empty(), "{doubled:#?}");
}
