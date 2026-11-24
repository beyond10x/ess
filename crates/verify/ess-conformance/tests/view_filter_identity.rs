//! A view narrowed on the entity's identity, or on a link field, is synthesized (beyond10x/ess#193).
//!
//! The identity of a row a scenario made is not a value the specification can spell: the
//! implementation assigns it and the scenario refers to it. Such a value is bound as an opaque
//! token, one per instance, so a filter comparing two of them with `==` or `!=` is decidable, and
//! anything else about them — an ordering, a literal, a text test — stays refused.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::{ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation},
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

/// An account that owns entries, and a command that creates each. The views are appended per test.
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

const PER_ACCOUNT: &str = r"
  - name: ledger.book.EntriesPerAccount
    source: ledger.book.Entry
    consistency: read_your_writes
    group_by: [account_id]
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: entries, type: Integer, aggregate: {count: {}}}
";

const POSTED: &str = "ledger.book.PostEntry/outcome/posted";
const VOIDED: &str = "ledger.book.VoidEntry/outcome/voided";

fn synthesis(views: &str) -> Synthesis {
    compiled(&format!("{MODEL}{views}"))
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

/// Every read of `view` in step order: the parameters it sends and what it expects.
fn reads(
    scenario: &ConformanceScenario,
    view: &str,
) -> Vec<(BTreeMap<String, ScenarioValue>, ViewExpectation)> {
    let mut params = BTreeMap::new();
    let mut out = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::QueryView {
                view: read,
                params: bound,
            } if read.to_string() == view => params.clone_from(bound),
            ScenarioStep::ExpectView {
                view: read,
                expectation,
            } if read.to_string() == view => out.push((params.clone(), expectation.clone())),
            ScenarioStep::EventuallyView {
                view: read,
                params: bound,
                expectation,
            } if read.to_string() == view => out.push((bound.clone(), expectation.clone())),
            _ => {}
        }
    }
    out
}

/// The instance name the scenario's first `PostEntry`, which creates its subject, was handed as its
/// account.
fn account_of(scenario: &ConformanceScenario) -> ScenarioValue {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "ledger.book.PostEntry" =>
            {
                input.get("account_id").cloned()
            }
            _ => None,
        })
        .expect("the entry is posted to an account")
}

#[test]
fn a_view_filtered_on_the_identity_is_read_by_the_created_instance() {
    let synthesis = synthesis(BY_ID);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));

    // The created entry is known only as what `EntryPosted` published, and that is what the read
    // sends as `id` and what the row it expects carries.
    let posted = scenario(&synthesis, POSTED);
    let posted_reads = reads(posted, "ledger.book.EntryById");
    assert!(
        !posted_reads.is_empty(),
        "`EntryById` is read after the entry is posted"
    );
    for (params, expectation) in &posted_reads {
        let sent = params.get("id").expect("the read sends `id`");
        assert!(
            matches!(sent, ScenarioValue::Observed { event, field }
                if event.to_string() == "ledger.book.EntryPosted" && field == "id"),
            "{sent:?}"
        );
        if let ViewExpectation::Contains { fields } = expectation {
            assert_eq!(fields.get("id"), Some(sent));
        }
    }
    assert!(
        posted_reads
            .iter()
            .any(|(_, expectation)| matches!(expectation, ViewExpectation::Contains { .. })),
        "{posted_reads:#?}"
    );

    // A moved instance was arranged under a name, and the read names it.
    let voided = scenario(&synthesis, VOIDED);
    let reads = reads(voided, "ledger.book.EntryById");
    assert!(
        reads.iter().any(|(params, expectation)| matches!(
            (params.get("id"), expectation),
            (
                Some(ScenarioValue::Instance { .. }),
                ViewExpectation::Contains { .. }
            )
        )),
        "{reads:#?}"
    );
}

#[test]
fn a_view_filtered_on_a_link_field_is_read_by_the_arranged_owner() {
    let synthesis = synthesis(BY_ACCOUNT);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));

    // The parameter is named apart from the field it is compared with, so it is bound from the
    // comparison and not from its name: the account the scenario opened.
    let posted = scenario(&synthesis, POSTED);
    let account = account_of(posted);
    assert!(
        matches!(account, ScenarioValue::Instance { .. }),
        "{account:?}"
    );
    let reads = reads(posted, "ledger.book.EntriesOfAccount");
    assert!(
        reads.iter().any(
            |(params, expectation)| params.get("account") == Some(&account)
                && matches!(expectation, ViewExpectation::Contains { fields }
                if fields.get("account_id") == Some(&account))
        ),
        "{reads:#?}"
    );
}

#[test]
fn an_aggregate_grouped_on_a_link_field_is_synthesized() {
    let synthesis = synthesis(PER_ACCOUNT);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let aggregate = synthesis
        .suite
        .scenarios
        .keys()
        .find(|id| id.to_string().contains("EntriesPerAccount"))
        .unwrap_or_else(|| {
            panic!(
                "{:#?}",
                synthesis.suite.scenarios.keys().collect::<Vec<_>>()
            )
        });
    let scenario = &synthesis.suite.scenarios[aggregate];
    // Each group is keyed by an account the scenario opened, never by a literal nobody assigned.
    let groups: Vec<(ScenarioValue, Option<ScenarioValue>)> =
        reads(scenario, "ledger.book.EntriesPerAccount")
            .into_iter()
            .filter_map(|(_, expectation)| match expectation {
                ViewExpectation::Contains { fields } => fields
                    .get("account_id")
                    .cloned()
                    .map(|group| (group, fields.get("entries").cloned())),
                _ => None,
            })
            .collect();
    assert!(groups.len() >= 2, "{scenario:#?}");
    assert!(
        groups
            .iter()
            .all(|(group, _)| matches!(group, ScenarioValue::Instance { .. })),
        "{groups:#?}"
    );
    let distinct: std::collections::BTreeSet<String> = groups
        .iter()
        .map(|(group, _)| format!("{group:?}"))
        .collect();
    assert_eq!(
        distinct.len(),
        groups.len(),
        "one read per group: {groups:#?}"
    );

    // Rows of one group share one owner: one account is opened per group, and a group of several
    // rows counts them all, so an implementation grouping by anything finer fails.
    let opened = scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "ledger.book.OpenAccount")
        })
        .count();
    let posted = scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "ledger.book.PostEntry")
        })
        .count();
    assert_eq!(opened, groups.len(), "{scenario:#?}");
    assert!(posted > opened, "{scenario:#?}");
    let count = |value: &Option<ScenarioValue>| match value {
        Some(ScenarioValue::Literal {
            value: Node::Number(number),
        }) => number.get(),
        other => panic!("`entries` is a number: {other:?}"),
    };
    assert!(
        groups.iter().any(|(_, entries)| count(entries) >= 2.0),
        "{groups:#?}"
    );
    #[allow(clippy::cast_precision_loss)]
    let total = posted as f64;
    assert!(
        (groups
            .iter()
            .map(|(_, entries)| count(entries))
            .sum::<f64>()
            - total)
            .abs()
            < 0.5,
        "every posted entry is counted in its account's group: {groups:#?}"
    );
}

/// An owner under `cardinality: one` holds one row, so an ordered list read by the owner has no
/// second row to rank: the order is refused naming the cardinality, and no account is given two
/// entries.
#[test]
fn an_ordered_list_of_an_owner_holding_one_row_is_refused_by_its_cardinality() {
    let view = format!("{BY_ACCOUNT}    order_by: [memo asc]\n");
    let text = format!("{MODEL}{view}").replace("cardinality: many", "cardinality: one");
    let refused = refusals(&compiled(&text));
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-014")
                && refusal.contains("`cardinality: one`")),
        "{refused:#?}"
    );
}

/// An order over identities is a question about the values the implementation chose, which no
/// token answers: the view stays refused by name, as it was.
#[test]
fn an_ordering_over_an_identity_stays_refused() {
    let view = BY_ID.replace("filter: id == param.id", "filter: id > param.id");
    assert_ne!(view, BY_ID);
    let synthesis = synthesis(&view);
    let refused = refusals(&synthesis);
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-005")
                && refusal.contains("`ledger.book.EntryById` filters on `id > param.id`")),
        "{refused:#?}"
    );
}

/// Two rows the scenario made are two identities: a filter excluding the one the parameter names
/// is decided, and the created row is asserted absent from it.
#[test]
fn an_identity_excluded_by_the_parameter_is_asserted_absent() {
    let view = BY_ID.replace("filter: id == param.id", "filter: id != param.id");
    assert_ne!(view, BY_ID);
    let synthesis = synthesis(&view);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let reads = reads(scenario(&synthesis, POSTED), "ledger.book.EntryById");
    assert!(
        reads
            .iter()
            .any(|(params, expectation)| matches!(expectation,
            ViewExpectation::Excludes { fields } if fields.get("id").is_some()
                && fields.get("id") == params.get("id"))),
        "{reads:#?}"
    );
}

/// One defect an implementation of the three views could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    ByIdIgnoresFilter,
    OfAccountIgnoresFilter,
    PerAccountIgnoresKey,
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
        let entries = self.entries.borrow();
        let project = |row: &Row, fields: &[&str]| -> Row {
            fields
                .iter()
                .map(|field| ((*field).to_owned(), row[*field].clone()))
                .collect()
        };
        match view {
            "ledger.book.Entries" => entries
                .iter()
                .map(|row| project(row, &["id", "state"]))
                .collect(),
            "ledger.book.EntryById" => entries
                .iter()
                .filter(|row| {
                    self.mutant == Mutant::ByIdIgnoresFilter || Some(&row["id"]) == params.get("id")
                })
                .map(|row| project(row, &["id", "account_id", "memo"]))
                .collect(),
            "ledger.book.EntriesOfAccount" => entries
                .iter()
                .filter(|row| {
                    self.mutant == Mutant::OfAccountIgnoresFilter
                        || Some(&row["account_id"]) == params.get("account")
                })
                .map(|row| project(row, &["id", "account_id", "memo"]))
                .collect(),
            "ledger.book.EntriesPerAccount" => {
                let mut groups: Vec<(Node, usize)> = Vec::new();
                for row in entries.iter() {
                    let key = if self.mutant == Mutant::PerAccountIgnoresKey {
                        Node::Null
                    } else {
                        row["account_id"].clone()
                    };
                    match groups.iter_mut().find(|(held, _)| *held == key) {
                        Some((_, count)) => *count += 1,
                        None => groups.push((key, 1)),
                    }
                }
                groups
                    .into_iter()
                    .map(|(key, count)| {
                        Row::from([
                            ("account_id".to_owned(), key),
                            (
                                "entries".to_owned(),
                                Node::Number(ess_primitives::facts::Number::from(count)),
                            ),
                        ])
                    })
                    .collect()
            }
            other => panic!("no view {other}"),
        }
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
            "ledger.book.VoidEntry" => {
                let id = request.input["id"].clone();
                let mut entries = self.entries.borrow_mut();
                // The model declares no `unknown_instance:` answer, so an identity no entry carries
                // takes the wrong-state one, as the suite says.
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

/// The scenarios that do not pass against `mutant`, synthesized from all three views.
fn failing(mutant: Mutant) -> BTreeSet<String> {
    let synthesis = synthesis(&format!("{BY_ID}{BY_ACCOUNT}{PER_ACCOUNT}"));
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Ledger::new(mutant))
        .into_report();
    let failed: BTreeSet<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

#[test]
fn the_ledger_as_specified_passes_its_own_suite() {
    assert_eq!(failing(Mutant::None), BTreeSet::new());
}

/// A target that ignores the identity filter, the link filter or the link group key fails the
/// synthesized suite: each view's assertions read the rows the filter or the key tells apart.
#[test]
fn a_target_ignoring_an_identity_filter_or_link_key_fails_the_suite() {
    // Every scenario that makes an entry reads both row views, the created row and one beside it.
    let rows = [
        POSTED,
        VOIDED,
        "ledger.book.Entry/transition/void/by/ledger.book.VoidEntry/voided",
    ];
    let set = |ids: &[&str]| {
        ids.iter()
            .map(|id| (*id).to_owned())
            .collect::<BTreeSet<_>>()
    };
    let mut wrong = Vec::new();
    for (mutant, expected) in [
        (Mutant::ByIdIgnoresFilter, set(&rows)),
        (Mutant::OfAccountIgnoresFilter, set(&rows)),
        (
            Mutant::PerAccountIgnoresKey,
            set(&["ledger.book.EntriesPerAccount/aggregate"]),
        ),
    ] {
        let failed = failing(mutant);
        if failed != expected {
            wrong.push(format!(
                "{mutant:?}: failed {failed:?}, expected {expected:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
