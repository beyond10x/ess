//! Adversary pass 2 on identity and link-field view filters (beyond10x/ess#193), after correction 1.
//!
//! Each case synthesizes a suite from a ledger model and runs it against a target written here,
//! with one defect switched in at a time. A correct target must pass; a defective one must fail
//! at least one scenario. Where the synthesizer cannot tell the two apart, the case is red.
//!
//! The TypeScript case runs one JavaScript target through the emitted TypeScript package and
//! through the Rust runner (the harness is copied from `tests/typescript_adversary_rp2.rs`).
#![allow(dead_code)]

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use ess_compiler::refs::CommandRef;
use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::Status,
    scenario::{ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::consistency::{ConsistencyToken, QueryConsistency};
use ess_primitives::{facts::Number, node::Node};
use serde_json::{json, Value};

const MODEL: &str = r"
format: ess/16
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
      - {name: note, type: String}
      - {name: counterparty_id, type: ledger.book.AccountId}
    lifecycle:
      initial: Posted
      states: [Posted, Voided]
      terminal: [Voided]
      transitions:
        - {name: void, from: [Posted], to: Voided}
errors:
  - {name: ledger.book.AlreadyVoided, fields: []}
events:
  - name: ledger.book.AccountOpened
    fields: [{name: account_id, type: ledger.book.AccountId}]
  - name: ledger.book.EntryPosted
    fields: [{name: id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryVoided
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
      - {name: note, type: String}
      - {name: counterparty_id, type: ledger.book.AccountId}
    outcomes:
      - name: posted
        creates: ledger.book.Entry
        instance: id
        emits: [ledger.book.EntryPosted]
        payload:
          ledger.book.EntryPosted:
            id: {generated: true}
        sets:
          account_id: input.account_id
          memo: input.memo
          note: input.note
          counterparty_id: input.counterparty_id
  - name: ledger.book.VoidEntry
    input: [{name: id, type: ledger.book.EntryId}]
    outcomes:
      - {name: already, wrong_state: true, error: ledger.book.AlreadyVoided}
      - name: voided
        moves: ledger.book.Entry.void
        instance: id
        emits: [ledger.book.EntryVoided]
        payload:
          ledger.book.EntryVoided:
            id: input.id
views:
  - name: ledger.book.Entries
    source: ledger.book.Entry
    consistency: read_your_writes
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: state, type: ledger.book.Entry.State}
";

const BY_ID: &str = r"
  - name: ledger.book.EntryById
    source: ledger.book.Entry
    consistency: read_your_writes
    params:
      - {name: id, type: ledger.book.EntryId}
    filter: id == param.id
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

/// A by-id read scoped to its account: `GET /accounts/{account}/entries/{id}`.
const BY_ID_OF_ACCOUNT: &str = r"
  - name: ledger.book.EntryOfAccount
    source: ledger.book.Entry
    consistency: read_your_writes
    params:
      - {name: id, type: ledger.book.EntryId}
      - {name: account, type: ledger.book.AccountId}
    filter:
      all:
        - id == param.id
        - account_id == param.account
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

const OF_ACCOUNT_BY_MEMO: &str = r"
  - name: ledger.book.EntriesOfAccountByMemo
    source: ledger.book.Entry
    consistency: read_your_writes
    params:
      - {name: account, type: ledger.book.AccountId}
    filter: account_id == param.account
    order_by: [memo asc]
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

/// A link-filtered list that does not project the identity.
const ACCOUNT_MEMOS: &str = r"
  - name: ledger.book.AccountMemos
    source: ledger.book.Entry
    consistency: read_your_writes
    params:
      - {name: account, type: ledger.book.AccountId}
    filter: account_id == param.account
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

/// A link-filtered, ordered, paged list: `GET /accounts/{account}/entries?page=&size=`.
const ACCOUNT_PAGES: &str = r"
  - name: ledger.book.AccountPages
    source: ledger.book.Entry
    consistency: read_your_writes
    params:
      - {name: account, type: ledger.book.AccountId}
      - {name: page, type: Integer}
      - {name: size, type: Integer}
    filter: account_id == param.account
    order_by: [memo asc]
    paging: {page: page, size: size, total: true}
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
";

const PER_ACCOUNT_MEMO: &str = r"
  - name: ledger.book.EntriesPerAccountMemo
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [account_id, memo]
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
      - {name: entries, type: Integer, aggregate: {count: {}}}
";

const PER_ACCOUNT_MEMO_NOTE: &str = r"
  - name: ledger.book.PerAccountMemoNote
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [account_id, memo, note]
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
      - {name: note, type: String}
      - {name: entries, type: Integer, aggregate: {count: {}}}
";

const PER_MEMO_ACCOUNT: &str = r"
  - name: ledger.book.PerMemoAccount
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [memo, account_id]
    fields:
      - {name: memo, type: String}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: entries, type: Integer, aggregate: {count: {}}}
";

const PER_ACCOUNT_STATE: &str = r"
  - name: ledger.book.PerAccountState
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [account_id, state]
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: state, type: ledger.book.Entry.State}
      - {name: entries, type: Integer, aggregate: {count: {}}}
";

/// Two link-typed fields aggregated at once.
const PARTIES_PER_MEMO: &str = r"
  - name: ledger.book.PartiesPerMemo
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [memo]
    fields:
      - {name: memo, type: String}
      - {name: accounts, type: Integer, aggregate: {count_distinct: account_id}}
      - {name: parties, type: Integer, aggregate: {count_distinct: counterparty_id}}
      - {name: entries, type: Integer, aggregate: {count: {}}}
";

/// Grouped by two link-typed fields.
const PER_ACCOUNT_PARTY: &str = r"
  - name: ledger.book.PerAccountParty
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [account_id, counterparty_id]
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: counterparty_id, type: ledger.book.AccountId}
      - {name: entries, type: Integer, aggregate: {count: {}}}
";

const LINKED: &str = r"
  - name: ledger.book.LinkedEntries
    source: ledger.book.Entry
    consistency: read_your_writes
    filter: 'defined(account_id)'
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
";

fn compiled(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    synthesize(&ir)
}

fn synthesis(views: &str) -> Synthesis {
    compiled(&format!("{MODEL}{views}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// `EntryOfAccount` answers every entry of `param.account`, ignoring `param.id`.
    OfAccountIgnoresId,
    /// `EntriesOfAccountByMemo` answers every entry of every account.
    OrderedIgnoresAccount,
    /// `EntriesOfAccountByMemo` answers in `memo desc`.
    OrderedDescending,
    /// `AccountMemos` answers every entry of every account.
    MemosIgnoresAccount,
    /// `AccountPages` pages over every entry of every account.
    PagedIgnoresAccount,
    /// `AccountPages` answers the whole list whatever the page.
    PagedIgnoresPaging,
    TripleDropsAccount,
    TripleDropsMemo,
    TripleDropsNote,
    MemoAccountDropsAccount,
    MemoAccountDropsMemo,
    AccountStateDropsAccount,
    AccountStateDropsState,
    /// `PartiesPerMemo.accounts` counts rows.
    AccountsCountRows,
    /// `PartiesPerMemo.parties` counts rows.
    PartiesCountRows,
    AccountPartyDropsAccount,
    AccountPartyDropsParty,
    /// `LinkedEntries` answers no row.
    LinkedAnswersNothing,
}

type Row = BTreeMap<String, Node>;

struct Ledger {
    mutant: Mutant,
    accounts: RefCell<Vec<Row>>,
    entries: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn text(node: &Node) -> String {
    match node {
        Node::Text(text) => text.clone(),
        other => format!("{other:?}"),
    }
}

fn count(n: usize) -> Node {
    Node::Number(Number::from(n))
}

fn project(row: &Row, fields: &[&str]) -> Row {
    fields
        .iter()
        .map(|field| ((*field).to_owned(), row[*field].clone()))
        .collect()
}

/// Groups `entries` by `kept`, and answers every key in `declared`: a dropped key is the value
/// of the group's first member, as a wrong grouping still answers the declared shape.
fn grouped<'a>(entries: &'a [Row], declared: &[&str], kept: &[&str]) -> Vec<(Row, Vec<&'a Row>)> {
    let mut groups: Vec<(Row, Vec<&Row>)> = Vec::new();
    for row in entries {
        let key = project(row, kept);
        match groups.iter_mut().find(|(held, _)| *held == key) {
            Some((_, members)) => members.push(row),
            None => groups.push((key, vec![row])),
        }
    }
    groups
        .into_iter()
        .map(|(_, members)| (project(members[0], declared), members))
        .collect()
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

    fn counted(&self, declared: &[&str], kept: &[&str]) -> Vec<Row> {
        let entries = self.entries.borrow();
        grouped(&entries, declared, kept)
            .into_iter()
            .map(|(mut key, members)| {
                key.insert("entries".into(), count(members.len()));
                key
            })
            .collect()
    }

    fn drop_key(&self, declared: &[&str], drops: &[(Mutant, &str)]) -> Vec<Row> {
        let kept: Vec<&str> = declared
            .iter()
            .copied()
            .filter(|key| !drops.iter().any(|(m, k)| *m == self.mutant && k == key))
            .collect();
        self.counted(declared, &kept)
    }

    #[allow(clippy::too_many_lines)]
    fn view(&self, view: &str, params: &BTreeMap<String, Node>) -> (Vec<Row>, Option<u64>) {
        let entries = self.entries.borrow();
        let of_account =
            |row: &&Row, ignore: bool| ignore || Some(&row["account_id"]) == params.get("account");
        let by_memo = |rows: &mut Vec<Row>, descending: bool| {
            rows.sort_by(|a, b| {
                let order = text(&a["memo"]).cmp(&text(&b["memo"]));
                if descending {
                    order.reverse()
                } else {
                    order
                }
            });
        };
        let rows = match view {
            "ledger.book.Entries" => entries
                .iter()
                .map(|row| project(row, &["id", "state"]))
                .collect(),
            "ledger.book.EntryById" => entries
                .iter()
                .filter(|row| Some(&row["id"]) == params.get("id"))
                .map(|row| project(row, &["id", "account_id", "memo"]))
                .collect(),
            "ledger.book.EntryOfAccount" => entries
                .iter()
                .filter(|row| of_account(row, false))
                .filter(|row| {
                    self.mutant == Mutant::OfAccountIgnoresId
                        || Some(&row["id"]) == params.get("id")
                })
                .map(|row| project(row, &["id", "account_id", "memo"]))
                .collect(),
            "ledger.book.EntriesOfAccountByMemo" => {
                let mut rows: Vec<Row> = entries
                    .iter()
                    .filter(|row| of_account(row, self.mutant == Mutant::OrderedIgnoresAccount))
                    .map(|row| project(row, &["id", "account_id", "memo"]))
                    .collect();
                by_memo(&mut rows, self.mutant == Mutant::OrderedDescending);
                rows
            }
            "ledger.book.AccountMemos" => entries
                .iter()
                .filter(|row| of_account(row, self.mutant == Mutant::MemosIgnoresAccount))
                .map(|row| project(row, &["account_id", "memo"]))
                .collect(),
            "ledger.book.AccountPages" => {
                let mut rows: Vec<Row> = entries
                    .iter()
                    .filter(|row| of_account(row, self.mutant == Mutant::PagedIgnoresAccount))
                    .map(|row| project(row, &["id", "account_id", "memo"]))
                    .collect();
                by_memo(&mut rows, false);
                let total = rows.len() as u64;
                let whole = |name: &str| match params.get(name) {
                    Some(Node::Number(n)) => n.as_i64().and_then(|v| usize::try_from(v).ok()),
                    _ => None,
                };
                if let (Some(page), Some(size)) = (whole("page"), whole("size")) {
                    if self.mutant != Mutant::PagedIgnoresPaging {
                        rows = rows.into_iter().skip(page * size).take(size).collect();
                    }
                }
                return (rows, Some(total));
            }
            "ledger.book.EntriesPerAccountMemo" => {
                self.counted(&["account_id", "memo"], &["account_id", "memo"])
            }
            "ledger.book.PerAccountMemoNote" => self.drop_key(
                &["account_id", "memo", "note"],
                &[
                    (Mutant::TripleDropsAccount, "account_id"),
                    (Mutant::TripleDropsMemo, "memo"),
                    (Mutant::TripleDropsNote, "note"),
                ],
            ),
            "ledger.book.PerMemoAccount" => self.drop_key(
                &["memo", "account_id"],
                &[
                    (Mutant::MemoAccountDropsAccount, "account_id"),
                    (Mutant::MemoAccountDropsMemo, "memo"),
                ],
            ),
            "ledger.book.PerAccountState" => self.drop_key(
                &["account_id", "state"],
                &[
                    (Mutant::AccountStateDropsAccount, "account_id"),
                    (Mutant::AccountStateDropsState, "state"),
                ],
            ),
            "ledger.book.PerAccountParty" => self.drop_key(
                &["account_id", "counterparty_id"],
                &[
                    (Mutant::AccountPartyDropsAccount, "account_id"),
                    (Mutant::AccountPartyDropsParty, "counterparty_id"),
                ],
            ),
            "ledger.book.PartiesPerMemo" => grouped(&entries, &["memo"], &["memo"])
                .into_iter()
                .map(|(mut key, members)| {
                    let distinct = |field: &str, rows_instead: bool| {
                        if rows_instead {
                            members.len()
                        } else {
                            members
                                .iter()
                                .map(|row| text(&row[field]))
                                .collect::<BTreeSet<_>>()
                                .len()
                        }
                    };
                    key.insert(
                        "accounts".into(),
                        count(distinct(
                            "account_id",
                            self.mutant == Mutant::AccountsCountRows,
                        )),
                    );
                    key.insert(
                        "parties".into(),
                        count(distinct(
                            "counterparty_id",
                            self.mutant == Mutant::PartiesCountRows,
                        )),
                    );
                    key.insert("entries".into(), count(members.len()));
                    key
                })
                .collect(),
            "ledger.book.LinkedEntries" => entries
                .iter()
                .filter(|_| self.mutant != Mutant::LinkedAnswersNothing)
                .map(|row| project(row, &["id", "account_id"]))
                .collect(),
            other => panic!("no view {other}"),
        };
        (rows, None)
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
            "ledger.book.VoidEntry" => {
                let id = request.input["id"].clone();
                let mut entries = self.entries.borrow_mut();
                let found = entries
                    .iter_mut()
                    .find(|row| row["id"] == id)
                    .filter(|row| row["state"] != Node::Text("Voided".into()));
                if let Some(row) = found {
                    row.insert("state".into(), Node::Text("Voided".into()));
                    took("voided").emitting(
                        ObservedEvent::new("ledger.book.EntryVoided".parse().unwrap())
                            .with("id", id),
                    )
                } else {
                    took("already").with_error(DeclaredErrorValue::new(
                        "ledger.book.AlreadyVoided".parse().unwrap(),
                    ))
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        let token = ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap();
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let (rows, total) = self.view(&request.view.to_string(), &request.params);
        let answer = SemanticViewResult::of(rows);
        Ok(match total {
            Some(total) => answer.with_total(total),
            None => answer,
        })
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

/// The scenarios of `synthesis` that do not pass against `target`, after asserting none was
/// refused.
fn failing_against(synthesis: &Synthesis, target: &Ledger) -> BTreeSet<String> {
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(synthesis));
    let suite = &synthesis.suite;
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} ({:?})", scenario.scenario, scenario.status))
        .collect()
}

/// Every mutant in `mutants` fails at least one scenario of the suite synthesized for `views`,
/// and the correct target passes it. Reports every surviving mutant at once.
fn killed(views: &str, mutants: &[Mutant]) {
    let synthesis = synthesis(views);
    let correct = failing_against(&synthesis, &Ledger::new(Mutant::None));
    assert!(correct.is_empty(), "the correct target fails: {correct:#?}");
    let survivors: Vec<Mutant> = mutants
        .iter()
        .copied()
        .filter(|mutant| failing_against(&synthesis, &Ledger::new(*mutant)).is_empty())
        .collect();
    assert!(
        survivors.is_empty(),
        "{survivors:?} pass every one of {} scenario(s)",
        synthesis.suite.scenarios.len()
    );
}

// ---- F1: a by-id read that also filters on the link ------------------------------------------------

/// `id == param.id && account_id == param.account`. Correction 1 skips the same-owner rule when the
/// filter reads the link, so the only further row is under another account — which the link clause
/// excludes whatever the target does with the id. A target ignoring `param.id` is then untested.
#[test]
fn a_by_id_read_scoped_to_its_account_that_ignores_the_id_fails() {
    killed(BY_ID_OF_ACCOUNT, &[Mutant::OfAccountIgnoresId]);
}

// ---- an owner that owns at most one row ------------------------------------------------------------

/// With `cardinality: one` an account owns at most one entry. Creating a second entry under the
/// subject's account arranges a world the specification says cannot exist.
#[test]
fn an_owner_of_at_most_one_row_is_never_given_two() {
    let text = format!("{MODEL}{BY_ID}{PER_ACCOUNT_MEMO}{OF_ACCOUNT_BY_MEMO}")
        .replace("cardinality: many", "cardinality: one");
    let synthesis = compiled(&text);
    let mut doubled = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut posted: BTreeMap<String, usize> = BTreeMap::new();
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
                if command.to_string() == "ledger.book.PostEntry" {
                    if let Some(ScenarioValue::Instance { instance }) = input.get("account_id") {
                        *posted.entry(instance.to_string()).or_default() += 1;
                    }
                }
            }
        }
        for (account, n) in posted {
            if n > 1 {
                doubled.push(format!("{id}: {n} entries under {account}"));
            }
        }
    }
    assert!(doubled.is_empty(), "{doubled:#?}");
}

// ---- F4: ordered and paged link-filtered lists -----------------------------------------------------

#[test]
fn an_ordered_read_of_an_owners_rows_kills_ignoring_the_owner_or_the_order() {
    killed(
        OF_ACCOUNT_BY_MEMO,
        &[Mutant::OrderedIgnoresAccount, Mutant::OrderedDescending],
    );
}

/// The same filter without the identity projected.
#[test]
fn a_link_filtered_list_without_the_identity_kills_ignoring_the_owner() {
    killed(ACCOUNT_MEMOS, &[Mutant::MemosIgnoresAccount]);
}

#[test]
fn a_paged_read_of_an_owners_rows_kills_ignoring_the_owner_or_the_page() {
    killed(
        ACCOUNT_PAGES,
        &[Mutant::PagedIgnoresAccount, Mutant::PagedIgnoresPaging],
    );
}

// ---- F2/F3: group keys and aggregates over links ---------------------------------------------------

#[test]
fn three_scoped_keys_led_by_the_link_drop_none() {
    killed(
        PER_ACCOUNT_MEMO_NOTE,
        &[
            Mutant::TripleDropsAccount,
            Mutant::TripleDropsMemo,
            Mutant::TripleDropsNote,
        ],
    );
}

#[test]
fn a_link_key_after_a_value_key_is_not_dropped() {
    killed(
        PER_MEMO_ACCOUNT,
        &[
            Mutant::MemoAccountDropsAccount,
            Mutant::MemoAccountDropsMemo,
        ],
    );
}

/// The link beside the lifecycle state: `all_scoped` is false, so B₁ is skipped for the link.
#[test]
fn a_link_key_beside_the_state_is_not_dropped() {
    killed(
        PER_ACCOUNT_STATE,
        &[
            Mutant::AccountStateDropsAccount,
            Mutant::AccountStateDropsState,
        ],
    );
}

#[test]
fn two_distinct_counts_over_link_typed_fields_are_each_told_from_count() {
    killed(
        PARTIES_PER_MEMO,
        &[Mutant::AccountsCountRows, Mutant::PartiesCountRows],
    );
}

#[test]
fn grouping_by_two_link_typed_fields_drops_neither() {
    killed(
        PER_ACCOUNT_PARTY,
        &[
            Mutant::AccountPartyDropsAccount,
            Mutant::AccountPartyDropsParty,
        ],
    );
}

// ---- `defined(<link>)` -----------------------------------------------------------------------------

/// The link now binds as a token, so `defined(account_id)` is `True` for an entry posted to an
/// account where it used to read as absent.
#[test]
fn a_view_of_rows_with_a_link_holds_the_subject() {
    killed(LINKED, &[Mutant::LinkedAnswersNothing]);
}

// ---- determinism -----------------------------------------------------------------------------------

#[test]
fn synthesis_of_the_pass_2_views_is_deterministic() {
    let views = format!(
        "{BY_ID}{BY_ID_OF_ACCOUNT}{OF_ACCOUNT_BY_MEMO}{ACCOUNT_PAGES}{PER_ACCOUNT_MEMO_NOTE}\
         {PER_MEMO_ACCOUNT}{PARTIES_PER_MEMO}{PER_ACCOUNT_PARTY}{LINKED}"
    );
    let once = serde_json::to_string(&synthesis(&views).suite).unwrap();
    let twice = serde_json::to_string(&synthesis(&views).suite).unwrap();
    assert_eq!(once, twice);
}

// ---- Go runtime parity on the new shapes -----------------------------------------------------------

#[test]
fn the_go_runtime_agrees_on_paged_and_two_link_reads() {
    if Command::new("go").arg("version").output().is_err() {
        eprintln!("skipped: no `go` on PATH");
        return;
    }
    let views = format!("{BY_ID_OF_ACCOUNT}{ACCOUNT_PAGES}{PARTIES_PER_MEMO}");
    let suite = synthesis(&views).suite;
    for mutant in [
        Mutant::None,
        Mutant::PagedIgnoresAccount,
        Mutant::PartiesCountRows,
    ] {
        support_go::assert_parity(
            &format!("viewfilter2-{mutant:?}").to_lowercase(),
            &suite,
            Ledger::new(mutant),
        );
    }
}

// ---- TypeScript runtime parity ---------------------------------------------------------------------
//
// The harness below is copied from `tests/typescript_adversary_rp2.rs` (itself a copy of
// `tests/typescript_suite_versions.rs`), trimmed to what these suites use.

fn toolchain() -> bool {
    for name in ["tsc", "node"] {
        let found = Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if !found {
            println!("skipped: no `{name}` on PATH, so the TypeScript runtime was not run");
            return false;
        }
    }
    true
}

type Verdicts = BTreeMap<String, String>;

struct Package {
    dir: PathBuf,
}

impl Package {
    fn new(case: &str, files: Vec<ess_conformance::ts::TsArtifact>, target: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("adversary-view-filter-pass2")
            .join(format!("{case}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for artifact in files {
            let path = root.join(&artifact.path);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
            std::fs::write(path, artifact.contents).expect("an artifact writes");
        }
        let dir = root.join(ess_conformance::ts::PACKAGE);
        for (name, contents) in [
            ("target.mjs", target),
            (
                "driver.mjs",
                include_str!("fixtures/typescript-parity-driver.mjs"),
            ),
            (
                "proxy.mjs",
                include_str!("fixtures/typescript-parity-proxy.mjs"),
            ),
            (
                "runtime-test.tsconfig.json",
                r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
            ),
        ] {
            std::fs::write(dir.join(name), contents).expect("a harness file writes");
        }
        let compiled = Command::new("tsc")
            .args(["--project", "runtime-test.tsconfig.json"])
            .current_dir(&dir)
            .output()
            .expect("tsc runs");
        assert!(
            compiled.status.success(),
            "tsc refused the emitted package:\n{}{}",
            String::from_utf8_lossy(&compiled.stdout),
            String::from_utf8_lossy(&compiled.stderr)
        );
        Self { dir }
    }

    fn typescript(&self, mode: &str) -> Result<Verdicts, String> {
        let report = self.dir.join(format!("report-{mode}.json"));
        let _ = std::fs::remove_file(&report);
        let run = Command::new("node")
            .args(["--test", "driver.mjs"])
            .env("ESS_TARGET_MODE", mode)
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .current_dir(&self.dir)
            .output()
            .expect("node runs");
        let printed = format!(
            "{}{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let Ok(text) = std::fs::read_to_string(&report) else {
            return Err(format!("the TypeScript run wrote no report:\n{printed}"));
        };
        let document: Value = serde_json::from_str(&text).expect("report/2 is JSON");
        let mut verdicts = Verdicts::new();
        for (status, ids) in document["outcomes"]
            .as_object()
            .expect("report/2 carries outcomes")
        {
            for id in ids.as_array().expect("an outcome list") {
                verdicts.insert(id.as_str().expect("an id").to_owned(), status.clone());
            }
        }
        Ok(verdicts)
    }

    fn rust(&self, admitted: &AdmittedSuite, mode: &str) -> Verdicts {
        let target = JsTarget::spawn(&self.dir, mode);
        Runner::for_suite(admitted.suite())
            .run_admitted(admitted, &target)
            .into_report()
            .scenarios
            .into_iter()
            .map(|result| {
                let status = match result.status {
                    Status::Passed => "passed",
                    Status::Failed => "failed",
                    Status::Error => "error",
                    Status::Unsupported => "skipped",
                };
                (result.scenario.to_string(), status.to_owned())
            })
            .collect()
    }
}

struct JsTarget {
    child: RefCell<Child>,
    io: RefCell<(ChildStdin, BufReader<ChildStdout>)>,
}

impl JsTarget {
    fn spawn(dir: &Path, mode: &str) -> Self {
        let mut child = Command::new("node")
            .arg("proxy.mjs")
            .env("ESS_TARGET_MODE", mode)
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("node runs the proxy");
        let stdin = child.stdin.take().expect("a stdin");
        let stdout = BufReader::new(child.stdout.take().expect("a stdout"));
        Self {
            child: RefCell::new(child),
            io: RefCell::new((stdin, stdout)),
        }
    }

    fn call(&self, method: &str, args: Value) -> Result<Option<Value>, TargetError> {
        let mut io = self.io.borrow_mut();
        let mut envelope = serde_json::Map::new();
        envelope.insert("method".to_owned(), Value::from(method));
        envelope.insert("args".to_owned(), args);
        let line = Value::Object(envelope).to_string();
        writeln!(io.0, "{line}").expect("the proxy reads");
        io.0.flush().expect("the proxy reads");
        let mut reply = String::new();
        io.1.read_line(&mut reply).expect("the proxy answers");
        let reply: Value = serde_json::from_str(&reply)
            .unwrap_or_else(|error| panic!("the proxy answered {reply:?}: {error}"));
        if reply["missing"] == Value::Bool(true) {
            return Ok(None);
        }
        if let Some(error) = reply.get("error") {
            let error = error.as_str().unwrap_or_default().to_owned();
            return Err(if reply["unsupported"] == Value::Bool(true) {
                TargetError::unsupported(method, error)
            } else {
                TargetError::unavailable(method, error)
            });
        }
        Ok(Some(reply["ok"].clone()))
    }

    fn required(&self, method: &str, args: Value) -> Result<Value, TargetError> {
        self.call(method, args)?
            .ok_or_else(|| TargetError::unavailable(method, "the target does not define it"))
    }
}

impl Drop for JsTarget {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

fn node(value: &Value) -> Node {
    serde_json::from_value(value.clone()).expect("a JSON value is a node")
}

fn fields(value: &Value) -> BTreeMap<String, Node> {
    value
        .as_object()
        .map(|object| object.iter().map(|(k, v)| (k.clone(), node(v))).collect())
        .unwrap_or_default()
}

fn events(value: &Value) -> Vec<ObservedEvent> {
    value
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|event| {
            let mut observed =
                ObservedEvent::new(event["event"].as_str().expect("an event").parse().unwrap());
            observed.payload = fields(&event["payload"]);
            observed
        })
        .collect()
}

fn command_result(command: &CommandRef, value: &Value) -> SemanticCommandResult {
    let mut result = match value["outcome"].as_str() {
        Some(outcome) if !outcome.is_empty() => SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new(outcome).expect("an outcome name"),
        )),
        _ => SemanticCommandResult::undeclared(),
    };
    if let Some(error) = value["error"].as_str().filter(|error| !error.is_empty()) {
        let declared = DeclaredErrorValue::new(error.parse().expect("an error"));
        result = result.with_error(declared);
    }
    if let Some(token) = value["consistency"]
        .as_str()
        .filter(|token| !token.is_empty())
    {
        result = result.with_consistency(ConsistencyToken::new(token).expect("a token"));
    }
    result.direct_events = events(&value["directEvents"]);
    result
}

impl ConformanceTarget for JsTarget {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        let identity = self.required("identity", Value::Null)?;
        Ok(ImplementationIdentity::new(
            identity["name"].as_str().unwrap_or_default(),
            identity["version"].as_str().unwrap_or_default(),
        ))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.required(
            "beginScenario",
            json!({"scenario": scenario.scenario.to_string(), "correlation": scenario.correlation.to_string()}),
        )
        .map(drop)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.required(
            "endScenario",
            json!({"scenario": scenario.scenario.to_string(), "correlation": scenario.correlation.to_string()}),
        )
        .map(drop)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let args = json!({
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string).unwrap_or_default(),
            "input": serde_json::to_value(&request.input).expect("an input serializes"),
            "correlation": request.correlation.to_string(),
        });
        let result = self.required("executeCommand", args)?;
        Ok(command_result(&request.command, &result))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let at_least = match &request.consistency {
            QueryConsistency::AtLeast { token } => token.to_string(),
            QueryConsistency::Current => String::new(),
        };
        let result = self.required(
            "queryView",
            json!({
                "view": request.view.to_string(),
                "params": serde_json::to_value(&request.params).expect("params serialize"),
                "atLeast": at_least,
                "correlation": request.correlation.to_string(),
                "deadline": {"attempts": 1},
            }),
        )?;
        let rows = result["rows"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(fields);
        let mut answer = SemanticViewResult::of(rows);
        if let Some(total) = result["total"].as_u64() {
            answer = answer.with_total(total);
        }
        Ok(answer)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let result = self.required(
            "observeEvents",
            json!({
                "event": request.event.to_string(),
                "correlation": request.correlation.to_string(),
                "deadline": {"attempts": 1},
            }),
        )?;
        Ok(events(&result))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "configureExternalOutcome",
            "the model declares none",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redeliverEvent", "unused"))
    }
}

/// The ledger in JavaScript. Modes: `correct`, `by-id-owners` (a by-id read answers every entry of
/// the named entry's account), `ignores-memo` (`EntriesPerAccountMemo` groups by account only),
/// `paged-ignores-account` (`AccountPages` pages every account's entries).
const JS_LEDGER: &str = r"
import { unsupported } from './dist/runtime.js';

const mode = process.env.ESS_TARGET_MODE ?? 'correct';
let minted = 0;
const mint = () => {
  minted += 1;
  return `00000000-0000-4000-8000-${String(minted).padStart(12, '0')}`;
};
const pick = (row, fields) => Object.fromEntries(fields.map((f) => [f, row[f]]));
const byMemo = (a, b) => (a.memo < b.memo ? -1 : a.memo > b.memo ? 1 : 0);

class Ledger {
  accounts = [];
  entries = [];
  identity() {
    return { name: 'ledger-adversary-2', version: '1' };
  }
  beginScenario() {
    this.accounts = [];
    this.entries = [];
  }
  endScenario() {}
  executeCommand({ command, input }) {
    const token = () => `seq:${minted}`;
    if (command === 'ledger.book.OpenAccount') {
      const account_id = mint();
      this.accounts.push({ ...input, account_id });
      return {
        outcome: 'opened',
        consistency: token(),
        directEvents: [{ event: 'ledger.book.AccountOpened', payload: { account_id } }],
      };
    }
    if (command === 'ledger.book.PostEntry') {
      if (!this.accounts.some((a) => a.account_id === input.account_id)) {
        return { outcome: '' };
      }
      const id = mint();
      this.entries.push({ ...input, id, state: 'Posted' });
      return {
        outcome: 'posted',
        consistency: token(),
        directEvents: [{ event: 'ledger.book.EntryPosted', payload: { id } }],
      };
    }
    if (command === 'ledger.book.VoidEntry') {
      const row = this.entries.find((e) => e.id === input.id && e.state !== 'Voided');
      if (row === undefined) {
        return { outcome: 'already', consistency: token(), error: 'ledger.book.AlreadyVoided' };
      }
      row.state = 'Voided';
      return {
        outcome: 'voided',
        consistency: token(),
        directEvents: [{ event: 'ledger.book.EntryVoided', payload: { id: input.id } }],
      };
    }
    throw unsupported(`command ${command}`);
  }
  queryView({ view, params }) {
    const e = this.entries;
    switch (view) {
      case 'ledger.book.Entries':
        return { rows: e.map((r) => pick(r, ['id', 'state'])) };
      case 'ledger.book.EntryById': {
        const named = e.find((r) => r.id === params.id);
        const rows = e.filter((r) =>
          mode === 'by-id-owners' ? named !== undefined && r.account_id === named.account_id : r.id === params.id,
        );
        return { rows: rows.map((r) => pick(r, ['id', 'account_id', 'memo'])) };
      }
      case 'ledger.book.EntriesOfAccountByMemo': {
        const rows = e.filter((r) => r.account_id === params.account).sort(byMemo);
        return { rows: rows.map((r) => pick(r, ['id', 'account_id', 'memo'])) };
      }
      case 'ledger.book.AccountPages': {
        const rows = e
          .filter((r) => mode === 'paged-ignores-account' || r.account_id === params.account)
          .sort(byMemo);
        const whole = (v) => (Number.isInteger(v) && v >= 0 ? v : undefined);
        const [page, size] = [whole(params.page), whole(params.size)];
        const slice = page !== undefined && size !== undefined ? rows.slice(page * size, page * size + size) : rows;
        return { rows: slice.map((r) => pick(r, ['id', 'account_id', 'memo'])), total: rows.length };
      }
      case 'ledger.book.EntriesPerAccountMemo': {
        const keys = mode === 'ignores-memo' ? ['account_id'] : ['account_id', 'memo'];
        const groups = [];
        for (const r of e) {
          const g = groups.find(([k]) => keys.every((f) => k[f] === r[f]));
          if (g) g[1] += 1;
          else groups.push([r, 1]);
        }
        return { rows: groups.map(([k, n]) => ({ account_id: k.account_id, memo: k.memo, entries: n })) };
      }
      default:
        throw new Error(`no view ${view}`);
    }
  }
  observeEvents() {
    return [];
  }
  configureExternalOutcome() {
    throw unsupported('the model declares none');
  }
  redeliverEvent() {
    throw unsupported('unused');
  }
}

export function makeTarget() {
  return new Ledger();
}
";

/// The emitted TypeScript runtime gives the Rust runner's verdict, scenario by scenario, on
/// identity parameters, link-keyed groups and a link-filtered paged read, for the correct target and
/// for each defect.
#[test]
fn the_typescript_runtime_agrees_on_identity_and_link_reads() {
    let views = format!("{BY_ID}{OF_ACCOUNT_BY_MEMO}{ACCOUNT_PAGES}{PER_ACCOUNT_MEMO}");
    let synthesis = synthesis(&views);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    if !toolchain() {
        return;
    }
    let package = Package::new(
        "ledger",
        ess_conformance::ts::emit(admitted.suite()).expect("the package emits"),
        JS_LEDGER,
    );
    for (index, mode) in [
        "correct",
        "by-id-owners",
        "ignores-memo",
        "paged-ignores-account",
    ]
    .into_iter()
    .enumerate()
    {
        let rust = package.rust(&admitted, mode);
        assert!(!rust.is_empty(), "the suite holds no scenario");
        if index == 0 {
            let wrong: Vec<_> = rust.iter().filter(|(_, s)| *s != "passed").collect();
            assert!(
                wrong.is_empty(),
                "the correct JS target fails the Rust runner: {wrong:?}"
            );
        } else {
            assert!(
                rust.values().any(|status| status != "passed"),
                "mode {mode} is caught by nothing"
            );
        }
        let typescript = package
            .typescript(mode)
            .unwrap_or_else(|log| panic!("mode {mode}: {log}"));
        assert_eq!(
            typescript, rust,
            "mode {mode}: the TypeScript runtime (left) disagrees with the Rust runner (right)"
        );
    }
}
