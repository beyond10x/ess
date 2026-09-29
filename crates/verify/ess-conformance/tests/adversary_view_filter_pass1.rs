//! Adversary pass 1 on identity and link-field view filters (beyond10x/ess#193).
//!
//! Each case synthesizes a suite from a ledger model and runs it against a target written here,
//! with one defect switched in at a time. A correct target must pass; a defective one must fail
//! at least one scenario. Where the synthesizer cannot tell the two apart, the case is red.
mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::Status,
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, facts::Number, node::Node};

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
    outcomes:
      - name: posted
        creates: ledger.book.Entry
        instance: id
        emits: [ledger.book.EntryPosted]
        payload:
          ledger.book.EntryPosted:
            id: {generated: true}
        sets: {account_id: input.account_id, memo: input.memo}
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

const OTHER_ENTRIES: &str = r"
  - name: ledger.book.OtherEntries
    source: ledger.book.Entry
    consistency: read_your_writes
    params:
      - {name: id, type: ledger.book.EntryId}
    filter: id != param.id
    fields:
      - {name: id, type: ledger.book.EntryId}
      - {name: memo, type: String}
";

const DISTINCT_ACCOUNTS: &str = r"
  - name: ledger.book.AccountsPosted
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [memo]
    fields:
      - {name: memo, type: String}
      - {name: accounts, type: Integer, aggregate: {count_distinct: account_id}}
      - {name: entries, type: Integer, aggregate: {count: {}}}
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
    /// `EntryById` answers every entry of the account the named entry belongs to.
    ByIdReturnsOwnersRows,
    /// `OtherEntries` answers no row at all.
    OtherAnswersNothing,
    /// `AccountsPosted.accounts` counts rows instead of distinct accounts.
    DistinctCountsRows,
    /// `EntriesPerAccountMemo` groups by account only.
    PerAccountMemoIgnoresMemo,
    /// `EntriesPerAccountMemo` groups by memo only.
    PerAccountMemoIgnoresAccount,
}

type Row = BTreeMap<String, Node>;

struct Ledger {
    mutant: Mutant,
    numeric: bool,
    accounts: RefCell<Vec<Row>>,
    entries: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Ledger {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            numeric: false,
            accounts: RefCell::default(),
            entries: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn mint(&self, table_len: usize) -> Node {
        self.minted.set(self.minted.get() + 1);
        if self.numeric {
            Node::Number(Number::from(table_len + 1))
        } else {
            Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
        }
    }

    fn count(n: usize) -> Node {
        Node::Number(Number::from(n))
    }

    #[allow(clippy::too_many_lines)]
    fn view(&self, view: &str, params: &BTreeMap<String, Node>) -> Vec<Row> {
        let entries = self.entries.borrow();
        let project = |row: &Row, fields: &[&str]| -> Row {
            fields
                .iter()
                .map(|field| ((*field).to_owned(), row[*field].clone()))
                .collect()
        };
        let group = |keys: &[&str]| -> Vec<Row> {
            let mut groups: Vec<(Row, usize)> = Vec::new();
            for row in entries.iter() {
                let key = project(row, keys);
                match groups.iter_mut().find(|(held, _)| *held == key) {
                    Some((_, n)) => *n += 1,
                    None => groups.push((key, 1)),
                }
            }
            groups
                .into_iter()
                .map(|(mut key, n)| {
                    key.insert("entries".into(), Self::count(n));
                    key
                })
                .collect()
        };
        match view {
            "ledger.book.Entries" => entries
                .iter()
                .map(|row| project(row, &["id", "state"]))
                .collect(),
            "ledger.book.EntryById" => {
                let named = params.get("id");
                let owner = entries
                    .iter()
                    .find(|row| Some(&row["id"]) == named)
                    .map(|row| row["account_id"].clone());
                entries
                    .iter()
                    .filter(|row| match self.mutant {
                        Mutant::ByIdReturnsOwnersRows => Some(&row["account_id"]) == owner.as_ref(),
                        _ => Some(&row["id"]) == named,
                    })
                    .map(|row| project(row, &["id", "account_id", "memo"]))
                    .collect()
            }
            "ledger.book.OtherEntries" => entries
                .iter()
                .filter(|row| {
                    self.mutant != Mutant::OtherAnswersNothing
                        && Some(&row["id"]) != params.get("id")
                })
                .map(|row| project(row, &["id", "memo"]))
                .collect(),
            "ledger.book.AccountsPosted" => {
                let memos: BTreeSet<String> = entries
                    .iter()
                    .map(|row| format!("{:?}", row["memo"]))
                    .collect();
                memos
                    .into_iter()
                    .map(|memo| {
                        let members: Vec<&Row> = entries
                            .iter()
                            .filter(|row| format!("{:?}", row["memo"]) == memo)
                            .collect();
                        let distinct: BTreeSet<String> = members
                            .iter()
                            .map(|row| format!("{:?}", row["account_id"]))
                            .collect();
                        let accounts = if self.mutant == Mutant::DistinctCountsRows {
                            members.len()
                        } else {
                            distinct.len()
                        };
                        Row::from([
                            ("memo".to_owned(), members[0]["memo"].clone()),
                            ("accounts".to_owned(), Self::count(accounts)),
                            ("entries".to_owned(), Self::count(members.len())),
                        ])
                    })
                    .collect()
            }
            "ledger.book.EntriesPerAccountMemo" => {
                let mut rows = match self.mutant {
                    Mutant::PerAccountMemoIgnoresMemo => group(&["account_id"]),
                    Mutant::PerAccountMemoIgnoresAccount => group(&["memo"]),
                    _ => group(&["account_id", "memo"]),
                };
                // A wrong grouping still answers the declared shape: the key it dropped is the
                // first row's value of that key.
                for row in &mut rows {
                    for key in ["account_id", "memo"] {
                        if !row.contains_key(key) {
                            let first = entries
                                .iter()
                                .find(|entry| {
                                    row.iter()
                                        .filter(|(name, _)| *name != "entries")
                                        .all(|(name, value)| &entry[name] == value)
                                })
                                .map_or(Node::Null, |entry| entry[key].clone());
                            row.insert(key.into(), first);
                        }
                    }
                }
                rows
            }
            "ledger.book.EntriesOfAccountByMemo" => {
                let mut rows: Vec<Row> = entries
                    .iter()
                    .filter(|row| Some(&row["account_id"]) == params.get("account"))
                    .map(|row| project(row, &["id", "account_id", "memo"]))
                    .collect();
                rows.sort_by(|a, b| format!("{:?}", a["memo"]).cmp(&format!("{:?}", b["memo"])));
                rows
            }
            other => panic!("no view {other}"),
        }
    }
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("ledger-adversary", "1"))
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
                let id = self.mint(self.accounts.borrow().len());
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
                let id = self.mint(self.entries.borrow().len());
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

fn killed(views: &str, mutant: Mutant) {
    let synthesis = synthesis(views);
    let correct = failing_against(&synthesis, &Ledger::new(Mutant::None));
    assert!(correct.is_empty(), "the correct target fails: {correct:#?}");
    let failed = failing_against(&synthesis, &Ledger::new(mutant));
    assert!(
        !failed.is_empty(),
        "{mutant:?} passes every one of {} scenario(s)",
        synthesis.suite.scenarios.len()
    );
}

/// `GET /entries/{id}` answering every entry of that entry's account. The companion row the
/// synthesizer arranges is created under a *different* account, so it is excluded either way.
#[test]
fn a_by_id_read_answering_the_owners_rows_fails() {
    killed(BY_ID, Mutant::ByIdReturnsOwnersRows);
}

/// `id != param.id` answering nothing: only a `Contains` for a row other than the named one
/// catches it.
#[test]
fn an_excluding_identity_filter_answering_nothing_fails() {
    killed(OTHER_ENTRIES, Mutant::OtherAnswersNothing);
}

/// `count_distinct` over the link field, grouped by `memo`: each row is created under its own
/// owner, so distinct and plain counts coincide in every group unless two rows of a group share
/// one.
#[test]
fn count_distinct_over_a_link_field_is_told_from_count() {
    killed(DISTINCT_ACCOUNTS, Mutant::DistinctCountsRows);
}

/// Several groups on one owner: `group_by: [account_id, memo]`.
#[test]
fn grouping_by_link_and_value_drops_neither_key() {
    killed(PER_ACCOUNT_MEMO, Mutant::PerAccountMemoIgnoresMemo);
    killed(PER_ACCOUNT_MEMO, Mutant::PerAccountMemoIgnoresAccount);
}

/// A list of an account's entries in an order: the link-field comparison with an `order_by`,
/// the most common list read an HTTP service has, is synthesized and passes a correct target.
#[test]
fn an_ordered_read_of_an_owners_rows_is_synthesized() {
    let synthesis = synthesis(OF_ACCOUNT_BY_MEMO);
    let failed = failing_against(&synthesis, &Ledger::new(Mutant::None));
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The synthesized suite is the same bytes on two runs.
#[test]
fn synthesis_of_identity_filters_is_deterministic() {
    let views = format!("{BY_ID}{OTHER_ENTRIES}{DISTINCT_ACCOUNTS}{PER_ACCOUNT_MEMO}");
    let once = serde_json::to_string(&synthesis(&views).suite).unwrap();
    let twice = serde_json::to_string(&synthesis(&views).suite).unwrap();
    assert_eq!(once, twice);
}

/// The generated Go runtime gives the reference verdict on identity parameters and link-keyed
/// groups, for the correct target and for mutants.
#[test]
fn the_go_runtime_agrees_on_identity_and_link_key_reads() {
    if std::process::Command::new("go")
        .arg("version")
        .output()
        .is_err()
    {
        eprintln!("skipped: no `go` on PATH");
        return;
    }
    let views = format!("{BY_ID}{OTHER_ENTRIES}{PER_ACCOUNT_MEMO}");
    let suite = synthesis(&views).suite;
    for mutant in [
        Mutant::None,
        Mutant::ByIdReturnsOwnersRows,
        Mutant::PerAccountMemoIgnoresMemo,
    ] {
        support_go::assert_parity(
            &format!("viewfilter-{mutant:?}").to_lowercase(),
            &suite,
            Ledger::new(mutant),
        );
    }
}
