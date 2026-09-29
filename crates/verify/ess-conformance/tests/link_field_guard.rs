//! A `when_subject` guard comparing a link field with an input is synthesized (beyond10x/ess#193,
//! the guard case).
//!
//! `account_id != input.account_id` compares the owner an entry was filed under with the one the
//! caller names. Neither side is a value the specification can spell: the owner is the instance the
//! arrangement created, and the input names one. So the input is sent as an arranged instance — the
//! entry's own owner, or a second owner arranged beside it — and the comparison is decided on the
//! opaque tokens the view-filter case binds (`synthesize/identity.rs`). Each side of the guard is
//! witnessed, so a target that ignores the guard, or compares with the wrong owner, fails.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::{ScenarioId, ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, ConformanceScenario, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};

/// An account that owns entries, a command that creates each, and the views every test reads.
/// The commands under test are appended before `views:`.
const MODEL: &str = r"
format: ess/15
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
    identity: {name: id, type: ledger.book.EntryId}
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    lifecycle: {initial: Posted, states: [Posted], terminal: [Posted]}
errors:
  - {name: ledger.book.OtherAccount, fields: []}
events:
  - name: ledger.book.AccountOpened
    fields: [{name: account_id, type: ledger.book.AccountId}]
  - name: ledger.book.EntryPosted
    fields: [{name: id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryAmended
    fields: [{name: id, type: ledger.book.EntryId}]
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
views:
  - name: ledger.book.Entries
    source: ledger.book.Entry
    consistency: read_your_writes
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: state, type: ledger.book.Entry.State}
";

const BY_ACCOUNT_WHOLE: &str = r"
  - name: ledger.book.EntryAccounts
    source: ledger.book.Entry
    consistency: read_your_writes
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: state, type: ledger.book.Entry.State}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

/// A command on an entry that refuses when the caller names another account than the one the
/// entry is filed under: a guard comparing the link field with an input.
const AMEND: &str = r"
  - name: ledger.book.AmendEntry
    input:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    outcomes:
      - name: other-account
        when_subject: {predicate: account_id != input.account_id}
        error: ledger.book.OtherAccount
      - name: amended
        updates: ledger.book.Entry
        instance: id
        sets: {memo: input.memo}
        emits: [ledger.book.EntryAmended]
        payload:
          ledger.book.EntryAmended:
            id: input.id
views:
";

const OTHER: &str = "ledger.book.AmendEntry/outcome/other-account";
const AMENDED: &str = "ledger.book.AmendEntry/outcome/amended";

/// The model with further commands, which end in `views:`, and then `views`.
fn synthesis_with(commands: &str, views: &str) -> Synthesis {
    let (head, entries) = MODEL
        .split_once("views:\n")
        .expect("the model declares `views:`");
    compiled(&format!(
        "{head}{}{entries}{views}",
        commands.trim_start_matches('\n')
    ))
}

fn compiled(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    synthesize(&ir)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| panic!("no scenario {id}: {:#?}", refusals(synthesis)))
}

/// Every command a scenario runs, in order, with the input it sends.
fn executed<'a>(
    scenario: &'a ConformanceScenario,
    name: &str,
) -> Vec<&'a BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } if command.to_string() == name => {
                Some(input)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_guard_comparing_a_link_field_is_synthesized() {
    let synthesis = synthesis_with(AMEND, BY_ACCOUNT_WHOLE);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
}

/// The amending branch is sent the entry's own account, and the refusal a second account arranged
/// beside it: both are instances the scenario created, never a literal nobody assigned. The second
/// account holds an entry of its own, so the refusal names an account that holds entries, just not
/// this one.
#[test]
fn each_side_of_the_guard_is_sent_an_arranged_account() {
    let synthesis = synthesis_with(AMEND, BY_ACCOUNT_WHOLE);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let sent = |id: &str, entries: usize| {
        let scenario = scenario(&synthesis, id);
        let posted = executed(scenario, "ledger.book.PostEntry");
        assert_eq!(posted.len(), entries, "{scenario:#?}");
        // The amendment sent to the entry the scenario posted; a refusal naming no subject of its
        // own is first sent for an entry nobody posted, which names no arranged account.
        let amended: Vec<_> = executed(scenario, "ledger.book.AmendEntry")
            .into_iter()
            .filter(|input| matches!(input.get("id"), Some(ScenarioValue::Instance { .. })))
            .collect();
        let [amend] = amended.as_slice() else {
            panic!("one amendment of the posted entry: {scenario:#?}");
        };
        let owner = posted[0].get("account_id").cloned().expect("filed under");
        let named = amend.get("account_id").cloned().expect("names an account");
        assert!(matches!(owner, ScenarioValue::Instance { .. }), "{owner:?}");
        assert!(matches!(named, ScenarioValue::Instance { .. }), "{named:?}");
        (
            owner,
            named,
            executed(scenario, "ledger.book.OpenAccount").len(),
            posted.get(1).and_then(|row| row.get("account_id")).cloned(),
        )
    };
    let (owner, named, opened, _) = sent(AMENDED, 1);
    assert_eq!(owner, named);
    assert_eq!(opened, 1);
    let (owner, named, opened, second) = sent(OTHER, 2);
    assert_ne!(owner, named);
    assert_eq!(opened, 2);
    assert_eq!(second, Some(named), "the second account holds an entry");
}

/// One defect an implementation of `AmendEntry` could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Amends whatever account the caller names.
    IgnoresGuard,
    /// Compares the named account with the one opened last, not the entry's.
    WrongOwner,
    /// Refuses whatever account the caller names.
    RefusesEvery,
}

type Row = BTreeMap<String, Node>;

/// The ledger, implemented here and not by the synthesizer, with one defect switched in at a time.
struct Ledger {
    mutant: Mutant,
    accounts: RefCell<Vec<Row>>,
    entries: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Ledger {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            accounts: RefCell::default(),
            entries: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn mint(&self) -> Node {
        self.minted.set(self.minted.get() + 1);
        Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
    }

    fn view(&self, view: &str, params: &BTreeMap<String, Node>) -> Vec<Row> {
        let project = |row: &Row, fields: &[&str]| -> Row {
            fields
                .iter()
                .map(|field| ((*field).to_owned(), row[*field].clone()))
                .collect()
        };
        let (fields, of): (&[&str], Option<&Node>) = match view {
            "ledger.book.Entries" => (&["id", "state"], None),
            "ledger.book.EntryAccounts" => (&["id", "state", "account_id", "memo"], None),
            "ledger.book.EntriesOfAccount" => (
                &["id", "account_id", "memo"],
                Some(params.get("account").expect("the read names an account")),
            ),
            other => panic!("no view {other}"),
        };
        self.entries
            .borrow()
            .iter()
            .filter(|row| of.is_none_or(|account| &row["account_id"] == account))
            .map(|row| project(row, fields))
            .collect()
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("ledger-fixture", "1"))
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
                if !self
                    .accounts
                    .borrow()
                    .iter()
                    .any(|row| row["account_id"] == account)
                {
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
            "ledger.book.AmendEntry" => {
                let id = request.input["id"].clone();
                let named = request.input["account_id"].clone();
                let last = self
                    .accounts
                    .borrow()
                    .last()
                    .map(|row| row["account_id"].clone());
                let refused = took("other-account").with_error(DeclaredErrorValue::new(
                    "ledger.book.OtherAccount".parse().unwrap(),
                ));
                let mut entries = self.entries.borrow_mut();
                // An entry nobody posted is filed under no account the caller could name.
                let row = entries.iter_mut().find(|row| row["id"] == id);
                let Some(row) = row else {
                    let token =
                        ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap();
                    return Ok(refused.with_consistency(token));
                };
                let filed = match self.mutant {
                    Mutant::WrongOwner => last.unwrap_or(Node::Null),
                    Mutant::RefusesEvery => Node::Null,
                    Mutant::None | Mutant::IgnoresGuard => row["account_id"].clone(),
                };
                if self.mutant != Mutant::IgnoresGuard && filed != named {
                    refused
                } else {
                    row.insert("memo".into(), request.input["memo"].clone());
                    took("amended").emitting(
                        ObservedEvent::new("ledger.book.EntryAmended".parse().unwrap())
                            .with("id", id),
                    )
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        let token = ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap();
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.view(&request.view.to_string(), &request.params),
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

/// A view reading the entries of the account a parameter names: a filter over the link, whose
/// companion rows arrange owners of their own beside the second owner the guard is sent.
const BY_ACCOUNT: &str = r"
  - name: ledger.book.EntriesOfAccount
    source: ledger.book.Entry
    consistency: read_your_writes
    params:
      - {name: account, type: ledger.book.AccountId}
    filter: account_id == param.account
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

/// Every scenario of the suite synthesized with `views` that does not pass against `mutant`, with
/// what it reported.
fn failing_with(views: &str, mutant: Mutant) -> BTreeMap<String, String> {
    let synthesis = synthesis_with(AMEND, views);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Ledger::new(mutant))
        .into_report();
    let failed: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                format!("{:#?}", scenario.diagnostics().collect::<Vec<_>>()),
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

/// The scenarios that do not pass against `mutant`.
fn failing(mutant: Mutant) -> BTreeSet<String> {
    failing_with(BY_ACCOUNT_WHOLE, mutant).into_keys().collect()
}

#[test]
fn the_ledger_as_specified_passes_its_own_suite() {
    let failed = failing_with(BY_ACCOUNT_WHOLE, Mutant::None);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// An owner under `cardinality: one` holds one entry: neither side of the guard gives an owner a
/// second one — the refusal's owner is given its own, and only that — so the guard is synthesized
/// there too.
#[test]
fn a_guard_over_an_owner_holding_one_row_is_synthesized() {
    let (head, entries) = MODEL
        .split_once("views:\n")
        .expect("the model declares `views:`");
    let text = format!(
        "{head}{}{entries}{BY_ACCOUNT_WHOLE}",
        AMEND.trim_start_matches('\n')
    )
    .replace("cardinality: many", "cardinality: one");
    assert!(text.contains("cardinality: one"));
    let synthesis = compiled(&text);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let other = scenario(&synthesis, OTHER);
    let posted = executed(other, "ledger.book.PostEntry");
    assert_eq!(posted.len(), 2, "{other:#?}");
    assert_ne!(
        posted[0].get("account_id"),
        posted[1].get("account_id"),
        "each account holds one entry: {other:#?}"
    );
    assert_eq!(
        executed(other, "ledger.book.OpenAccount").len(),
        2,
        "{other:#?}"
    );
}

/// An ordering between the link and the input asks for the identities' values, which no token
/// answers: the branch stays refused, naming the guard.
#[test]
fn an_ordering_between_the_link_and_an_input_stays_refused() {
    let ordered = AMEND.replace(
        "account_id != input.account_id",
        "account_id > input.account_id",
    );
    assert_ne!(ordered, AMEND);
    let refused = refusals(&synthesis_with(&ordered, BY_ACCOUNT_WHOLE));
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-003")
                && refusal.contains("ledger.book.AmendEntry/other-account")),
        "{refused:#?}"
    );
}

/// A view filtered on the link beside the guard: its companion rows' owners and the guard's second
/// owner are distinct instances, and the suite is admitted and passes.
#[test]
fn a_view_filtered_on_the_link_beside_the_guard_passes() {
    let failed = failing_with(&format!("{BY_ACCOUNT_WHOLE}{BY_ACCOUNT}"), Mutant::None);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A target that ignores the guard fails the refusal's scenario, and one comparing the named
/// account with the wrong owner fails it too; one refusing every account fails the amendment's.
/// Each side of the guard is witnessed by its own scenario.
#[test]
fn a_target_ignoring_the_guard_or_comparing_the_wrong_owner_fails_the_suite() {
    let mut wrong = Vec::new();
    for (mutant, expected) in [
        (Mutant::IgnoresGuard, OTHER),
        (Mutant::WrongOwner, OTHER),
        (Mutant::RefusesEvery, AMENDED),
    ] {
        let failed = failing(mutant);
        if failed != BTreeSet::from([expected.to_owned()]) {
            wrong.push(format!(
                "{mutant:?}: failed {failed:?}, expected {expected}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
