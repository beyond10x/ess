//! Adversary, pass 1, for synthesis of a `when_subject` guard comparing a link field with an input
//! (beyond10x/ess#193, the guard case).
//!
//! The story's acceptance: "a target whose guard ignores the link, and one comparing the wrong
//! owner, each fail the synthesized suite". Each case below asks that of a shape the unit's own
//! test does not cover, against a ledger implemented here with one defect switched in at a time.
#![allow(clippy::struct_excessive_bools, clippy::too_many_lines)]
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
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

/// `{types}` is replaced by the account identity's base type, `{amended}` by the extra payload
/// fields of `EntryAmended`, `{commands}` by the command under test.
const MODEL: &str = r"
format: ess/18
system: ledger
version: v1
domain: ledger.book
types:
  - {name: ledger.book.AccountId, kind: newtype, of: {types}}
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
  - {name: ledger.book.SameAccount, fields: []}
  - {name: ledger.book.UnknownAccount, fields: []}
events:
  - name: ledger.book.AccountOpened
    fields: [{name: account_id, type: ledger.book.AccountId}]
  - name: ledger.book.EntryPosted
    fields: [{name: id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryAmended
    fields: [{name: id, type: ledger.book.EntryId}{amended}]
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

/// `{guard}` is the refusal's `when_subject` predicate, `{payload}` the amendment's extra payload.
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
            id: input.id{payload}
";

/// Moves an entry to the account the caller names, refusing the account it is already filed under:
/// the `==` form of the guard, on the refusal, with the accepting branch writing the link.
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

struct Shape {
    id_type: &'static str,
    guard: &'static str,
    payload_account: bool,
    transfer: bool,
    /// Branches inserted before the refusal.
    before: &'static str,
    /// Replacements applied to the model text, each of which must match.
    edits: &'static [(&'static str, &'static str)],
}

const PLAIN: Shape = Shape {
    id_type: "Uuid",
    guard: "account_id != input.account_id",
    payload_account: false,
    transfer: false,
    before: "",
    edits: &[],
};

fn text(shape: &Shape) -> String {
    let amend = AMEND
        .replace("{before}", shape.before)
        .replace("{guard}", shape.guard)
        .replace(
            "{payload}",
            if shape.payload_account {
                "\n            account_id: input.account_id"
            } else {
                ""
            },
        );
    let commands = if shape.transfer {
        format!("{amend}{TRANSFER}")
    } else {
        amend
    };
    let mut out = MODEL
        .replace("{types}", shape.id_type)
        .replace(
            "{amended}",
            if shape.payload_account {
                ", {name: account_id, type: ledger.book.AccountId}"
            } else {
                ""
            },
        )
        .replace("{commands}\n", commands.trim_start_matches('\n'));
    for (from, to) in shape.edits {
        let next = out.replace(from, to);
        assert_ne!(next, out, "`{from}` is in the model");
        out = next;
    }
    out
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

/// FNV-1a as `witness::uuid_of` spreads it, so the Uuid token of an instance is recognisable.
fn uuid_of(seed: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in seed.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("00000000-0000-4000-8000-{:012x}", hash & 0xffff_ffff_ffff)
}

/// Every search token that reached the emitted suite as a literal value.
fn leaked(synthesis: &Synthesis) -> Vec<String> {
    let json = synthesis
        .suite
        .to_canonical_json()
        .expect("the suite renders");
    let mut out = Vec::new();
    if json.contains("instance:") {
        out.push("`instance:` text".to_owned());
    }
    for name in [
        "account",
        "account-2",
        "account-3",
        "account-4",
        "account-66",
    ] {
        let token = uuid_of(&format!("instance:{name}"));
        if json.contains(&token) {
            out.push(format!("the Uuid token of {name}: {token}"));
        }
    }
    out
}

/// One defect an implementation of `AmendEntry` or `TransferEntry` could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Amends, or moves, whatever account the caller names.
    IgnoresGuard,
    /// Refuses whatever account the caller names.
    RefusesEvery,
    /// Refuses unless the named account holds some entry — any entry, not this one.
    NamedHoldsAny,
    /// Under an `all:` guard over the link and the memo, reads the memo half only.
    IgnoresLink,
    /// Under an `all:` guard over the link and the memo, reads the link half only.
    IgnoresMemo,
}

type Row = BTreeMap<String, Node>;

/// The ledger, implemented here and not by the synthesizer.
struct Ledger {
    mutant: Mutant,
    /// The refusal also asks `memo == "locked"` of the stored entry.
    locked_conjunct: bool,
    /// The two halves are joined by `any:`, not `all:`.
    disjunct: bool,
    /// `EntryAmended` carries the named account.
    payload_account: bool,
    accounts: RefCell<Vec<Row>>,
    entries: RefCell<Vec<Row>>,
    minted: Cell<u64>,
    text_ids: bool,
    /// A missing named account is refused first (`when_related: {exists: false}`).
    related: bool,
}

impl Ledger {
    fn new(shape: &Shape, mutant: Mutant) -> Self {
        Self {
            mutant,
            locked_conjunct: shape.guard.contains("locked"),
            disjunct: shape.guard.starts_with("{any:"),
            payload_account: shape.payload_account,
            accounts: RefCell::default(),
            entries: RefCell::default(),
            minted: Cell::new(0),
            text_ids: shape.id_type == "String",
            related: shape.before.contains("when_related"),
        }
    }

    fn mint(&self, text: bool) -> Node {
        self.minted.set(self.minted.get() + 1);
        if text {
            Node::Text(format!("acct-{}", self.minted.get()))
        } else {
            Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
        }
    }

    fn token(&self) -> ConsistencyToken {
        ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap()
    }

    /// Whether the refusal answers for this stored entry and named account.
    fn refuses(&self, row: &Row, named: &Node) -> bool {
        let differs = &row["account_id"] != named;
        let locked = row["memo"] == Node::Text("locked".into());
        let holds_any = self
            .entries
            .borrow()
            .iter()
            .any(|entry| &entry["account_id"] == named);
        match self.mutant {
            Mutant::IgnoresGuard => false,
            Mutant::RefusesEvery => true,
            Mutant::NamedHoldsAny => !holds_any,
            Mutant::IgnoresLink if self.locked_conjunct => locked,
            Mutant::IgnoresMemo if self.locked_conjunct => differs,
            _ if self.locked_conjunct && self.disjunct => differs || locked,
            _ if self.locked_conjunct => differs && locked,
            _ => differs,
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
        let refused = |name: &str, error: &str| {
            took(name).with_error(DeclaredErrorValue::new(error.parse().unwrap()))
        };
        let result = match command.to_string().as_str() {
            "ledger.book.OpenAccount" => {
                let id = self.mint(self.text_ids);
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
                let id = self.mint(false);
                let mut row = request.input.clone();
                row.insert("id".into(), id.clone());
                row.insert("state".into(), Node::Text("Posted".into()));
                self.entries.borrow_mut().push(row);
                took("posted").emitting(
                    ObservedEvent::new("ledger.book.EntryPosted".parse().unwrap()).with("id", id),
                )
            }
            "ledger.book.OpenOrg" => {
                let id = self.mint(false);
                took("opened").emitting(
                    ObservedEvent::new("ledger.book.OrgOpened".parse().unwrap()).with("org_id", id),
                )
            }
            "ledger.book.AmendEntry" => {
                let id = request.input["id"].clone();
                let named = request.input["account_id"].clone();
                if self.related
                    && !self
                        .accounts
                        .borrow()
                        .iter()
                        .any(|row| row["account_id"] == named)
                {
                    let refusal = refused("unknown-account", "ledger.book.UnknownAccount");
                    return Ok(refusal.with_consistency(self.token()));
                }
                let row = self
                    .entries
                    .borrow()
                    .iter()
                    .find(|row| row["id"] == id)
                    .cloned();
                let refusal = refused("other-account", "ledger.book.OtherAccount");
                let Some(row) = row else {
                    return Ok(refusal.with_consistency(self.token()));
                };
                if self.refuses(&row, &named) {
                    refusal
                } else {
                    for entry in self.entries.borrow_mut().iter_mut() {
                        if entry["id"] == id {
                            entry.insert("memo".into(), request.input["memo"].clone());
                        }
                    }
                    let mut event = ObservedEvent::new("ledger.book.EntryAmended".parse().unwrap())
                        .with("id", id);
                    if self.payload_account {
                        event = event.with("account_id", row["account_id"].clone());
                    }
                    took("amended").emitting(event)
                }
            }
            "ledger.book.TransferEntry" => {
                let id = request.input["id"].clone();
                let named = request.input["account_id"].clone();
                let row = self
                    .entries
                    .borrow()
                    .iter()
                    .find(|row| row["id"] == id)
                    .cloned();
                let refusal = refused("same-account", "ledger.book.SameAccount");
                let Some(row) = row else {
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

/// The scenarios of `shape`'s suite that do not pass against `mutant`, with what they reported.
fn failing(shape: &Shape, mutant: Mutant) -> BTreeMap<String, String> {
    let synthesis = compiled(&text(shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Ledger::new(shape, mutant))
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

fn names(failed: &BTreeMap<String, String>) -> BTreeSet<String> {
    failed.keys().cloned().collect()
}

// ---- control: the unit's own shape against this fixture -------------------------------------

#[test]
fn control_the_plain_guard_passes_this_ledger_and_kills_the_guard_mutants() {
    assert_eq!(failing(&PLAIN, Mutant::None), BTreeMap::new());
    assert!(!failing(&PLAIN, Mutant::IgnoresGuard).is_empty());
    assert!(!failing(&PLAIN, Mutant::RefusesEvery).is_empty());
}

// ---- (1) a target checking the named account holds *some* entry -----------------------------

/// The second owner the refusal is sent holds no entry of its own, so a target that refuses only
/// an account holding no entries — "does the caller own anything here", not "does the caller own
/// this entry" — passes both scenarios.
#[test]
fn a_target_checking_the_named_account_holds_any_entry_fails_the_suite() {
    let failed = failing(&PLAIN, Mutant::NamedHoldsAny);
    assert!(
        !failed.is_empty(),
        "a target refusing only accounts that hold no entry passed every scenario"
    );
}

// ---- (2) conjunction: each half of `all:` witnessed ------------------------------------------

const LOCKED: Shape = Shape {
    id_type: "Uuid",
    guard: "{all: [account_id != input.account_id, memo == \"locked\"]}",
    payload_account: false,
    transfer: false,
    before: "",
    edits: &[],
};

#[test]
fn a_link_guard_in_all_is_synthesized_and_passes_the_ledger() {
    let synthesis = compiled(&text(&LOCKED));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(failing(&LOCKED, Mutant::None), BTreeMap::new());
}

#[test]
fn a_target_reading_only_the_memo_half_of_the_guard_fails_the_suite() {
    let failed = failing(&LOCKED, Mutant::IgnoresLink);
    assert!(
        !failed.is_empty(),
        "a target refusing every locked entry, whichever account is named, passed every scenario"
    );
}

#[test]
fn a_target_reading_only_the_link_half_of_the_guard_fails_the_suite() {
    let failed = failing(&LOCKED, Mutant::IgnoresMemo);
    assert!(
        !failed.is_empty(),
        "a target refusing every other account, whatever the memo, passed every scenario"
    );
}

// ---- (3) the token never reaches a suite -----------------------------------------------------

#[test]
fn a_payload_carrying_the_named_account_carries_the_account() {
    let shape = Shape {
        payload_account: true,
        ..PLAIN
    };
    let synthesis = compiled(&text(&shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    assert_eq!(failing(&shape, Mutant::None), BTreeMap::new());
}

#[test]
fn a_text_identity_link_is_synthesized_without_leaking_its_token() {
    let shape = Shape {
        id_type: "String",
        ..PLAIN
    };
    let synthesis = compiled(&text(&shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    assert_eq!(failing(&shape, Mutant::None), BTreeMap::new());
    assert!(!failing(&shape, Mutant::IgnoresGuard).is_empty());
}

// ---- (1) the `==` form, and an accepting branch writing the link -----------------------------

const MOVE: Shape = Shape {
    id_type: "Uuid",
    guard: "account_id != input.account_id",
    payload_account: false,
    transfer: true,
    before: "",
    edits: &[],
};

#[test]
fn a_transfer_refusing_the_same_account_is_synthesized_and_passes_the_ledger() {
    let synthesis = compiled(&text(&MOVE));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    assert_eq!(failing(&MOVE, Mutant::None), BTreeMap::new());
}

#[test]
fn a_transfer_ignoring_or_refusing_every_account_fails_the_suite() {
    let ignored = names(&failing(&MOVE, Mutant::IgnoresGuard));
    assert!(
        ignored.contains("ledger.book.TransferEntry/outcome/same-account"),
        "{ignored:?}"
    );
    let refused = names(&failing(&MOVE, Mutant::RefusesEvery));
    assert!(
        refused.contains("ledger.book.TransferEntry/outcome/moved"),
        "{refused:?}"
    );
}

// ---- negation --------------------------------------------------------------------------------

#[test]
fn a_negated_equality_is_synthesized_like_its_inequality() {
    let shape = Shape {
        guard: "not account_id == input.account_id",
        ..PLAIN
    };
    let synthesis = compiled(&text(&shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(failing(&shape, Mutant::None), BTreeMap::new());
    assert!(!failing(&shape, Mutant::IgnoresGuard).is_empty());
}

// ---- (4) owner of an owner, an optional link -------------------------------------------------

/// An organisation owns the account that owns the entry: the second account is arranged with an
/// owner of its own, and nothing it arranges is named twice.
const ORG_EDITS: &[(&str, &str)] = &[
    (
        "  - {name: ledger.book.EntryId, kind: newtype, of: Uuid}\n",
        "  - {name: ledger.book.EntryId, kind: newtype, of: Uuid}\n  - {name: ledger.book.OrgId, kind: newtype, of: Uuid}\n",
    ),
    (
        "entities:\n",
        "entities:\n  - name: ledger.book.Org\n    identity: {name: org_id, type: ledger.book.OrgId}\n    fields: []\n    relations:\n      - {name: accounts, kind: owns, target: ledger.book.Account, cardinality: many, via: org_id}\n    lifecycle: {initial: Active, states: [Active], terminal: [Active]}\n",
    ),
    (
        "      - {name: holder, type: String}\n    relations:",
        "      - {name: holder, type: String}\n      - {name: org_id, type: ledger.book.OrgId}\n    relations:",
    ),
    (
        "events:\n",
        "events:\n  - name: ledger.book.OrgOpened\n    fields: [{name: org_id, type: ledger.book.OrgId}]\n",
    ),
    (
        "commands:\n",
        "commands:\n  - name: ledger.book.OpenOrg\n    input: []\n    outcomes:\n      - name: opened\n        creates: ledger.book.Org\n        instance: org_id\n        emits: [ledger.book.OrgOpened]\n        payload:\n          ledger.book.OrgOpened:\n            org_id: {generated: true}\n",
    ),
    (
        "    input: [{name: holder, type: String}]\n",
        "    input: [{name: holder, type: String}, {name: org_id, type: ledger.book.OrgId}]\n",
    ),
    ("        sets: {holder: input.holder}\n", "        sets: {holder: input.holder, org_id: input.org_id}\n"),
];

#[test]
fn a_link_guard_under_an_owned_owner_is_synthesized_and_kills_the_guard_mutants() {
    let shape = Shape {
        edits: ORG_EDITS,
        ..PLAIN
    };
    let synthesis = compiled(&text(&shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(leaked(&synthesis), Vec::<String>::new());
    assert_eq!(failing(&shape, Mutant::None), BTreeMap::new());
    assert!(!failing(&shape, Mutant::IgnoresGuard).is_empty());
    assert!(!failing(&shape, Mutant::RefusesEvery).is_empty());
}

/// The compiler refuses an `Optional` owner link, so `defined()` is asked of a required one.
#[test]
fn a_link_read_through_defined_beside_the_comparison_kills_the_guard_mutants() {
    let shape = Shape {
        guard: "{all: [defined(account_id), account_id != input.account_id]}",
        ..PLAIN
    };
    let synthesis = compiled(&text(&shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(failing(&shape, Mutant::None), BTreeMap::new());
    assert!(!failing(&shape, Mutant::IgnoresGuard).is_empty());
    assert!(!failing(&shape, Mutant::RefusesEvery).is_empty());
}

// ---- (5) beside the state pseudo-field -------------------------------------------------------

#[test]
fn a_link_guard_with_the_state_pseudo_field_is_synthesized_and_kills_the_guard_mutants() {
    let shape = Shape {
        guard: "{all: [state == Posted, account_id != input.account_id]}",
        ..PLAIN
    };
    let synthesis = compiled(&text(&shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(failing(&shape, Mutant::None), BTreeMap::new());
    assert!(!failing(&shape, Mutant::IgnoresGuard).is_empty());
    assert!(!failing(&shape, Mutant::RefusesEvery).is_empty());
}

// ---- (2) disjunction: each half of `any:` witnessed alone ------------------------------------

const EITHER: Shape = Shape {
    guard: "{any: [account_id != input.account_id, memo == \"locked\"]}",
    ..PLAIN
};

#[test]
fn a_link_guard_in_any_is_synthesized_and_passes_the_ledger() {
    let synthesis = compiled(&text(&EITHER));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(failing(&EITHER, Mutant::None), BTreeMap::new());
}

#[test]
fn under_any_a_target_reading_only_the_memo_half_fails_the_suite() {
    let failed = failing(&EITHER, Mutant::IgnoresLink);
    assert!(
        !failed.is_empty(),
        "a target refusing only locked entries, whichever account is named, passed every scenario"
    );
}

#[test]
fn under_any_a_target_reading_only_the_link_half_fails_the_suite() {
    let failed = failing(&EITHER, Mutant::IgnoresMemo);
    assert!(
        !failed.is_empty(),
        "a target refusing only other accounts, whatever the memo, passed every scenario"
    );
}

// ---- (4) the guard on the accepting branch, the refusal its default --------------------------

#[test]
fn an_equality_guarding_the_accepting_branch_kills_the_guard_mutants() {
    let shape = Shape {
        edits: &[
            (
                "      - name: other-account\n        when_subject:\n          predicate: account_id != input.account_id\n        error: ledger.book.OtherAccount\n      - name: amended\n        updates: ledger.book.Entry\n",
                "      - name: amended\n        when_subject:\n          predicate: account_id == input.account_id\n        updates: ledger.book.Entry\n",
            ),
            (
                "            id: input.id\nviews:",
                "            id: input.id\n      - name: other-account\n        error: ledger.book.OtherAccount\nviews:",
            ),
        ],
        ..PLAIN
    };
    let synthesis = compiled(&text(&shape));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    assert_eq!(failing(&shape, Mutant::None), BTreeMap::new());
    assert!(!failing(&shape, Mutant::IgnoresGuard).is_empty());
    assert!(!failing(&shape, Mutant::RefusesEvery).is_empty());
}

// ---- (6) one model, one suite -----------------------------------------------------------------

#[test]
fn each_shape_synthesizes_the_same_bytes_twice() {
    for shape in [&PLAIN, &LOCKED, &EITHER, &MOVE] {
        let first = compiled(&text(shape)).suite.to_canonical_json().unwrap();
        let second = compiled(&text(shape)).suite.to_canonical_json().unwrap();
        assert!(
            first == second,
            "`{}` differs between two runs",
            shape.guard
        );
    }
}
