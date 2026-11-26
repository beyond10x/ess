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
use ess_conformance::interpret::Interpreted;
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

/// [`AMEND`] beside a refusal of a missing account named by the same input. The link guard answers
/// before the related row is read (held state first, then missing and present related rows), so
/// every branch below it is sent the account the entry is filed under, or is not synthesized.
const REVISE: &str = r"
  - name: ledger.book.ReviseEntry
    input:
      - {name: id, type: ledger.book.EntryId}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    outcomes:
      - name: other-account
        when_subject: {predicate: account_id != input.account_id}
        error: ledger.book.OtherAccount
      - name: no-account
        when_related: {via: input.account_id, exists: false}
        error: ledger.book.NoAccount
      - name: revised
        updates: ledger.book.Entry
        instance: id
        sets: {memo: input.memo}
        emits: [ledger.book.EntryAmended]
        payload:
          ledger.book.EntryAmended:
            id: input.id
views:
";

/// A tag no entry is filed under, read beside the account: [`REVISE`] with a second related row,
/// so the scenario of `no-tag` arranges the account as a row beside the missing tag.
const TAGS: [(&str, &str); 4] = [
    (
        "  - {name: ledger.book.EntryId, kind: newtype, of: Uuid}\n",
        "  - {name: ledger.book.EntryId, kind: newtype, of: Uuid}\n  - {name: ledger.book.TagId, kind: newtype, of: Uuid}\n",
    ),
    (
        "errors:\n",
        "  - name: ledger.book.Tag\n    identity: {name: tag_id, type: ledger.book.TagId}\n    fields: []\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nerrors:\n  - {name: ledger.book.NoTag, fields: []}\n",
    ),
    (
        "commands:\n",
        "  - name: ledger.book.TagOpened\n    fields: [{name: tag_id, type: ledger.book.TagId}]\ncommands:\n  - name: ledger.book.OpenTag\n    input: []\n    outcomes:\n      - name: opened\n        creates: ledger.book.Tag\n        instance: tag_id\n        emits: [ledger.book.TagOpened]\n        payload:\n          ledger.book.TagOpened:\n            tag_id: {generated: true}\n",
    ),
    (
        "      - {name: memo, type: String}\n    outcomes:\n      - name: other-account\n",
        "      - {name: memo, type: String}\n      - {name: tag_id, type: ledger.book.TagId}\n    outcomes:\n      - name: other-account\n",
    ),
];

/// [`REVISE`] reading a tag beside the account, refused where no tag carries the identity.
const NO_TAG: (&str, &str) = (
    "      - name: revised\n",
    "      - name: no-tag\n        when_related: {via: input.tag_id, exists: false}\n        error: ledger.book.NoTag\n      - name: revised\n",
);

/// Every scenario synthesized from the ledger with [`REVISE`] that the model's own interpreter
/// does not pass, with what it reported.
fn revise_failing() -> BTreeMap<String, String> {
    revise_failing_with(&[])
}

/// [`revise_failing`] with each `(from, to)` of `edits` applied, in order, to the model and the
/// command.
fn revise_failing_with(edits: &[(&str, &str)]) -> BTreeMap<String, String> {
    let (head, entries) = MODEL
        .split_once("views:\n")
        .expect("the model declares `views:`");
    let mut text = format!(
        "{}{}{entries}{BY_ACCOUNT_WHOLE}",
        head.replace("format: ess/15", "format: ess/29").replace(
            "  - {name: ledger.book.OtherAccount, fields: []}\n",
            "  - {name: ledger.book.OtherAccount, fields: []}\n  - {name: ledger.book.NoAccount, fields: []}\n",
        ),
        REVISE.trim_start_matches('\n')
    );
    for (from, to) in edits {
        assert!(text.contains(from), "the model holds {from:?}");
        text = text.replacen(from, to, 1);
    }
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let suite = synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    Runner::for_suite(&suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                format!("{:#?}", scenario.diagnostics().collect::<Vec<_>>()),
            )
        })
        .collect()
}

/// RED (a downstream report, 2026-11-25): the `revised` scenario names an account arranged for
/// the missing-account refusal instead of the entry's own, and the `no-account` scenario sends a
/// missing account that is not the entry's, so the link guard answers `other-account` in both.
/// Synthesis arranges the row the input names without the held-state guard that compares the
/// entry's link with that same input.
#[test]
fn a_link_guard_beside_a_missing_row_refusal_on_the_same_input_passes_its_own_model() {
    let failed = revise_failing();
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The same link guard where the row its input names is arranged beside another row the command
/// reads: the `no-tag` scenario arranges an account beside the missing tag, apart from the entry's
/// own, so the link guard answers `other-account` first; it is refused by synthesis instead.
#[test]
fn a_link_guard_on_the_input_naming_a_row_beside_passes_its_own_model() {
    let mut edits = TAGS.to_vec();
    edits.push(NO_TAG);
    let failed = revise_failing_with(&edits);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A store and the orders placed at it. `ShipOrder` names the store as input and refuses first
/// where the order is of another store (`store_id != input.store_id`); its later branches read the
/// store that input names, which is then always the order's own. A hand-written scenario places
/// the order at a store, freezes that store and ships the order naming it.
const SHOP: &str = "format: ess/33
system: demo
version: v1
domain: demo.shop
types:
  - {name: demo.shop.StoreId, kind: newtype, of: String}
  - {name: demo.shop.OrderId, kind: newtype, of: String}
entities:
  - name: demo.shop.Store
    identity: {name: store_id, type: demo.shop.StoreId}
    fields: [{name: title, type: String}]
    lifecycle: {initial: Open, states: [Open, Frozen], terminal: [Frozen], transitions: [{name: freeze, from: [Open], to: Frozen}]}
  - name: demo.shop.Order
    identity: {name: order_id, type: demo.shop.OrderId}
    fields: [{name: store_id, type: demo.shop.StoreId}]
    relations: [{name: store, kind: references, target: demo.shop.Store, cardinality: one, via: store_id}]
    lifecycle: {initial: Placed, states: [Placed, Shipped], terminal: [Shipped], transitions: [{name: ship, from: [Placed], to: Shipped}]}
actors:
  - {name: demo.shop.Clerk, may: [demo.shop.OpenStore, demo.shop.FreezeStore, demo.shop.PlaceOrder, demo.shop.ShipOrder]}
errors:
  - {name: demo.shop.Missing, summary: No such row.}
  - {name: demo.shop.Elsewhere, summary: The order is of another store.}
  - {name: demo.shop.Frozen, summary: The store is frozen.}
commands:
  - name: demo.shop.OpenStore
    input: [{name: store_id, type: demo.shop.StoreId}]
    outcomes:
      - {name: opened, creates: demo.shop.Store, instance: store_id, emits: [demo.shop.Opened], payload: {demo.shop.Opened: {store_id: input.store_id}}}
  - name: demo.shop.FreezeStore
    input: [{name: store_id, type: demo.shop.StoreId}]
    outcomes:
      - {name: missing, unknown_instance: true, error: demo.shop.Missing}
      - {name: frozen, moves: demo.shop.Store.freeze, instance: store_id, emits: [demo.shop.Froze], payload: {demo.shop.Froze: {store_id: input.store_id}}}
  - name: demo.shop.PlaceOrder
    input: [{name: order_id, type: demo.shop.OrderId}, {name: store_id, type: demo.shop.StoreId}]
    outcomes:
      - {name: no-store, when_related: {via: input.store_id, exists: false}, error: demo.shop.Missing}
      - {name: placed, creates: demo.shop.Order, instance: order_id, sets: {store_id: input.store_id}, emits: [demo.shop.Placed], payload: {demo.shop.Placed: {order_id: input.order_id}}}
  - name: demo.shop.ShipOrder
    input: [{name: order_id, type: demo.shop.OrderId}, {name: store_id, type: demo.shop.StoreId}]
    outcomes:
      - {name: missing, unknown_instance: true, error: demo.shop.Missing}
      - {name: elsewhere, when_subject: {predicate: store_id != input.store_id}, error: demo.shop.Elsewhere}
      - {name: no-store, when_related: {via: input.store_id, exists: false}, error: demo.shop.Missing}
      - {name: frozen, when_related: {via: input.store_id, predicate: state == Frozen}, error: demo.shop.Frozen}
      - {name: shipped, moves: demo.shop.Order.ship, instance: order_id, emits: [demo.shop.Shipped], payload: {demo.shop.Shipped: {order_id: input.order_id}}}
events:
  - {name: demo.shop.Opened, fields: [{name: store_id, type: demo.shop.StoreId}]}
  - {name: demo.shop.Froze, fields: [{name: store_id, type: demo.shop.StoreId}]}
  - {name: demo.shop.Placed, fields: [{name: order_id, type: demo.shop.OrderId}]}
  - {name: demo.shop.Shipped, fields: [{name: order_id, type: demo.shop.OrderId}]}
";

/// The scenarios of `ShipOrder` synthesized from `text`, and every scenario of its suite that the
/// model's own interpreter does not pass, with what it reported.
fn ship_order(text: &str) -> (BTreeSet<String>, BTreeMap<String, String>) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("shop.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let synthesis = synthesize(&ir);
    let written = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.contains("ShipOrder"))
        .collect();
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let failed = Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
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
    (written, failed)
}

/// `written` holds the scenarios of the branches `ShipOrder` takes on the row its order is placed
/// at, and `failed` is empty.
fn assert_ships_at_its_own_store(written: &BTreeSet<String>, failed: &BTreeMap<String, String>) {
    for id in [
        "demo.shop.ShipOrder/outcome/frozen",
        "demo.shop.ShipOrder/outcome/shipped",
        "demo.shop.Order/transition/ship/by/demo.shop.ShipOrder/shipped",
    ] {
        assert!(written.contains(id), "{id} is written: {written:#?}");
    }
    assert!(failed.is_empty(), "{failed:#?}");
}

/// RED (a downstream report on 0.99.4, N93): the branches below the link refusal read the row the
/// input names, and that input must name the order's own store. Synthesis arranged that row apart
/// from the order and, since 0.99.4, refuses each branch with "no candidate of the 0 tried"; the
/// arrangement that names the order's own store is the one a hand-written scenario uses.
#[test]
fn a_branch_reading_the_row_the_input_names_beside_a_link_refusal_is_sent_the_subjects_own_row() {
    let (written, failed) = ship_order(SHOP);
    assert_ships_at_its_own_store(&written, &failed);
}

/// [`SHOP`] with `ShipOrder` reading the store through the order's stored link instead of its
/// input: the row is the order's own, so no `exists: false` branch is declared (`ess/30`, every
/// write of the link checks the row).
fn shop_through_the_stored_link() -> String {
    let text = SHOP.replace(
        "      - {name: no-store, when_related: {via: input.store_id, exists: false}, error: demo.shop.Missing}\n      - {name: frozen, when_related: {via: input.store_id,",
        "      - {name: frozen, when_related: {via: store_id,",
    );
    assert_ne!(text, SHOP, "the edit applies");
    text
}

/// RED (N93, the stored-link form): the order is placed at the frozen store and the branch reads
/// it through the stored link, but the input the link refusal compares with it is sent as a
/// placeholder, so `elsewhere` answers on the model's interpreter.
#[test]
fn a_branch_reading_the_subjects_stored_link_beside_a_link_refusal_is_sent_that_row() {
    let (written, failed) = ship_order(&shop_through_the_stored_link());
    assert_ships_at_its_own_store(&written, &failed);
}

/// [`SHOP`] with the order's store copied from a cart the order is placed from: `OpenCart` stores
/// the store its input names, and `PlaceOrder` copies the cart's store into the order.
fn shop_with_the_link_copied_from_a_cart() -> String {
    let edits = [
        (
            "  - {name: demo.shop.OrderId, kind: newtype, of: String}\n",
            "  - {name: demo.shop.OrderId, kind: newtype, of: String}\n  - {name: demo.shop.CartId, kind: newtype, of: String}\n",
        ),
        (
            "  - name: demo.shop.Order\n",
            "  - name: demo.shop.Cart\n    identity: {name: cart_id, type: demo.shop.CartId}\n    fields: [{name: store_id, type: demo.shop.StoreId}]\n    lifecycle: {initial: Live, states: [Live], terminal: [Live]}\n  - name: demo.shop.Order\n",
        ),
        (
            "may: [demo.shop.OpenStore,",
            "may: [demo.shop.OpenCart, demo.shop.OpenStore,",
        ),
        (
            "  - name: demo.shop.PlaceOrder\n    input: [{name: order_id, type: demo.shop.OrderId}, {name: store_id, type: demo.shop.StoreId}]\n    outcomes:\n      - {name: no-store, when_related: {via: input.store_id, exists: false}, error: demo.shop.Missing}\n      - {name: placed, creates: demo.shop.Order, instance: order_id, sets: {store_id: input.store_id},",
            "  - name: demo.shop.OpenCart\n    input: [{name: cart_id, type: demo.shop.CartId}, {name: store_id, type: demo.shop.StoreId}]\n    outcomes:\n      - {name: no-store, when_related: {via: input.store_id, exists: false}, error: demo.shop.Missing}\n      - {name: opened, creates: demo.shop.Cart, instance: cart_id, sets: {store_id: input.store_id}, emits: [demo.shop.CartOpened], payload: {demo.shop.CartOpened: {cart_id: input.cart_id}}}\n  - name: demo.shop.PlaceOrder\n    input: [{name: order_id, type: demo.shop.OrderId}, {name: cart_id, type: demo.shop.CartId}]\n    outcomes:\n      - {name: no-cart, when_related: {via: input.cart_id, exists: false}, error: demo.shop.Missing}\n      - {name: placed, creates: demo.shop.Order, instance: order_id, sets: {store_id: {related: {via: input.cart_id, field: store_id}}},",
        ),
        (
            "  - {name: demo.shop.Placed,",
            "  - {name: demo.shop.CartOpened, fields: [{name: cart_id, type: demo.shop.CartId}]}\n  - {name: demo.shop.Placed,",
        ),
    ];
    let mut text = SHOP.to_owned();
    for (from, to) in edits {
        assert!(text.contains(from), "the model holds {from:?}");
        text = text.replacen(from, to, 1);
    }
    text
}

/// RED (N93, the reported shape): the order's store is copied from the cart it is placed from, so
/// pointing the order at a store means opening its cart at that store first. Synthesis rewrites
/// only an act that stores the link from its own input, and refuses the order's later branches.
#[test]
fn a_branch_beside_a_link_refusal_is_sent_the_subjects_own_row_where_the_link_is_copied_from_another_row(
) {
    let (written, failed) = ship_order(&shop_with_the_link_copied_from_a_cart());
    assert_ships_at_its_own_store(&written, &failed);
}
