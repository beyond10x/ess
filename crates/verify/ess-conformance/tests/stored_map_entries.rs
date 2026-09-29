//! A stored `Map` (or `List`) field a guard reads by its entries is arranged non-empty where the
//! guard needs it (beyond10x/ess#240).
//!
//! A row's collection is set from a command input, and that input used to be witnessed only as the
//! base value — one entry under the map's own path, never the value a guard compares the bound
//! variable with — so `exists r in redirect_uris: r == input.application` was never true on an
//! arranged row, and both the branch it guards and its sibling were refused ESS-SYNTH-003. The
//! quantifier binds the map's **values** (the compiler types `r` as the value type; Entity Runtime
//! folds `members.values()`), so an arranged row's map holds an entry whose value is the one the
//! guard compares with on one side, and an entry whose value is not on the other.
#![allow(clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::command::OutcomeName;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// An account holds a collection of redirect targets, set whole from `Register`'s input. `Redirect`
/// reads the account named by `input.account_id` (`when_related:`); `Launch` updates the account and
/// reads its own row (`when_subject:`). Both are refused where `{pred}` holds.
const TEMPLATE: &str = "format: ess/18
system: demo
version: v1
domain: demo.apps
types:
  - {name: demo.apps.AccountId, kind: newtype, of: Uuid}
  - {name: demo.apps.VisitId, kind: newtype, of: Uuid}
errors:
  - {name: demo.apps.NoAccount, summary: No account., fields: []}
  - {name: demo.apps.Refused, summary: Refused., fields: []}
entities:
  - name: demo.apps.Account
    identity: {name: account_id, type: demo.apps.AccountId}
    fields:
      - {name: redirect_uris, type: '{field}'}
      - {name: last, type: Optional<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.apps.Visit
    identity: {name: visit_id, type: demo.apps.VisitId}
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    lifecycle: {initial: Issued, states: [Issued], terminal: [Issued]}
events:
  - name: demo.apps.Registered
    fields: [{name: account_id, type: demo.apps.AccountId}]
  - name: demo.apps.Visited
    fields: [{name: visit_id, type: demo.apps.VisitId}]
  - name: demo.apps.Launched
    fields: [{name: account_id, type: demo.apps.AccountId}]
actors:
  - {name: demo.apps.Client, may: [demo.apps.Register, demo.apps.Redirect, demo.apps.Launch]}
commands:
  - name: demo.apps.Register
    input:
      - {name: redirect_uris, type: '{field}'}
    outcomes:
      - name: registered
        creates: demo.apps.Account
        instance: account_id
        emits: [demo.apps.Registered]
        payload:
          demo.apps.Registered: {account_id: {generated: true}}
        sets:
          redirect_uris: input.redirect_uris
  - name: demo.apps.Redirect
    input:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    outcomes:
      - name: no-account
        when_related: {via: input.account_id, exists: false}
        error: demo.apps.NoAccount
      - name: refused
        when_related: {via: input.account_id, predicate: {pred}}
        error: demo.apps.Refused
      - name: redirected
        creates: demo.apps.Visit
        instance: visit_id
        emits: [demo.apps.Visited]
        payload:
          demo.apps.Visited: {visit_id: {generated: true}}
        sets:
          account_id: input.account_id
          application: input.application
  - name: demo.apps.Launch
    input:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    outcomes:
      - name: refused
        when_subject: {predicate: {pred}}
        error: demo.apps.Refused
      - name: launched
        updates: demo.apps.Account
        instance: account_id
        emits: [demo.apps.Launched]
        payload:
          demo.apps.Launched: {account_id: input.account_id}
        sets:
          last: input.application
views:
  - name: demo.apps.Accounts
    source: demo.apps.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: state, type: demo.apps.Account.State}
      - {name: redirect_uris, type: '{field}'}
      - {name: last, type: Optional<String>}
  - name: demo.apps.Visits
    source: demo.apps.Visit
    consistency: read_your_writes
    fields:
      - {name: visit_id, type: demo.apps.VisitId}
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
";

const MAP: &str = "Map<String, String>";
const LIST: &str = "List<String>";

/// The issue's guard, verbatim.
const ISSUE: &str = "{not: {exists: {in: redirect_uris, as: r, that: r == input.application}}}";

fn model(field: &str, predicate: &str) -> String {
    TEMPLATE
        .replace("{field}", field)
        .replace("{pred}", predicate)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("apps.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals_about(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
                    result
                        .suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    result
                        .refusals
                        .iter()
                        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The collection each `Register` in the scenario sends, keyed by the instance it is captured as.
fn registered(scenario: &ConformanceScenario) -> Vec<(String, Node)> {
    let mut out = Vec::new();
    let mut pending = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.apps.Register" =>
            {
                pending = match input.get("redirect_uris") {
                    Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                    other => panic!("the collection is a literal: {other:?}"),
                };
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "demo.apps.Account" => {
                if let Some(value) = pending.take() {
                    out.push((instance.to_string(), value));
                }
            }
            _ => {}
        }
    }
    out
}

/// The input of the last `command` the scenario sends.
fn sent(scenario: &ConformanceScenario, command: &str) -> BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the scenario sends {command}"))
}

/// The row the guarded command is sent for, and the application it is sent with.
fn pointed_at(scenario: &ConformanceScenario, command: &str) -> (Node, Node) {
    let input = sent(scenario, command);
    let Some(ScenarioValue::Instance { instance }) = input.get("account_id") else {
        panic!("account_id names an arranged account: {input:?}")
    };
    let rows = registered(scenario);
    let row = rows
        .iter()
        .find(|(name, _)| *name == instance.to_string())
        .unwrap_or_else(|| panic!("{instance} is registered: {rows:?}"))
        .1
        .clone();
    let Some(ScenarioValue::Literal { value: application }) = input.get("application") else {
        panic!("the application is a literal: {input:?}")
    };
    (row, application.clone())
}

/// The values a stored collection holds: a map's values, a list's elements.
fn values(collection: &Node) -> Vec<Node> {
    match collection {
        Node::Map(entries) => entries.values().cloned().collect(),
        Node::Seq(items) => items.clone(),
        other => panic!("a collection, got {other:?}"),
    }
}

fn keys(collection: &Node) -> Vec<String> {
    match collection {
        Node::Map(entries) => entries.keys().cloned().collect(),
        _ => Vec::new(),
    }
}

const REDIRECT_REFUSED: &str = "demo.apps.Redirect/outcome/refused";
const REDIRECTED: &str = "demo.apps.Redirect/outcome/redirected";
const LAUNCH_REFUSED: &str = "demo.apps.Launch/outcome/refused";
const LAUNCHED: &str = "demo.apps.Launch/outcome/launched";

// ---- the issue's shape -----------------------------------------------------------------------

#[test]
fn issue_240_the_refusal_and_the_accepting_default_are_both_synthesized() {
    let result = synthesis(&model(MAP, ISSUE));
    assert_eq!(
        refusals_about(&result, "demo.apps.Redirect"),
        Vec::<String>::new()
    );
    scenario(&result, REDIRECT_REFUSED);
    scenario(&result, REDIRECTED);
}

#[test]
fn issue_240_the_accepting_row_holds_the_application_as_a_value_and_not_as_its_key() {
    let result = synthesis(&model(MAP, ISSUE));
    let (row, application) = pointed_at(scenario(&result, REDIRECTED), "demo.apps.Redirect");
    assert!(
        values(&row).contains(&application),
        "the map holds the application sent: {row:?} / {application:?}"
    );
    let Node::Text(text) = &application else {
        panic!("text: {application:?}")
    };
    assert!(
        !keys(&row).contains(text),
        "the application is a value, not a key, so a target reading the keys fails: {row:?}"
    );
}

#[test]
fn issue_240_the_refused_row_holds_an_entry_that_is_not_the_application() {
    let result = synthesis(&model(MAP, ISSUE));
    let (row, application) = pointed_at(scenario(&result, REDIRECT_REFUSED), "demo.apps.Redirect");
    assert!(
        !values(&row).contains(&application),
        "the map does not hold the application sent: {row:?} / {application:?}"
    );
    assert!(
        !values(&row).is_empty(),
        "the map is not empty, so a target reading only emptiness fails: {row:?}"
    );
}

/// An `example:` sits on a scalar input only, so a map is never written one. The compared input's
/// example is its base: the refusal is sent it, beside a row that does not hold it, and the
/// accepting branch is sent the value the row holds.
#[test]
fn issue_240_an_example_on_the_compared_input_is_sent_where_the_row_does_not_hold_it() {
    let sent = "      - {name: application, type: String}\n    outcomes:\n      - name: no-account";
    let written = "      - {name: application, type: String, example: 'https://app.test/cb'}\n    \
                   outcomes:\n      - name: no-account";
    let plain = model(MAP, ISSUE);
    let text = plain.replacen(sent, written, 1);
    assert_ne!(text, plain, "the example is written");
    let result = synthesis(&text);
    assert_eq!(
        refusals_about(&result, "demo.apps.Redirect"),
        Vec::<String>::new()
    );
    let example = Node::Text("https://app.test/cb".to_owned());
    let (row, application) = pointed_at(scenario(&result, REDIRECT_REFUSED), "demo.apps.Redirect");
    assert_eq!(application, example, "the refusal is sent the example");
    assert!(
        !values(&row).is_empty() && !values(&row).contains(&application),
        "{row:?} / {application:?}"
    );
    let (row, application) = pointed_at(scenario(&result, REDIRECTED), "demo.apps.Redirect");
    assert!(
        values(&row).contains(&application),
        "{row:?} / {application:?}"
    );
    assert_ne!(application, example, "{row:?}");
}

/// The same reading on a command's own input: a map input publishes its values, so a quantifier
/// over it in `when:` is decided, and the input is sent holding the literal on one side and not on
/// the other.
#[test]
fn issue_240_a_quantifier_over_a_map_input_is_decided_on_both_sides() {
    let plain = model(MAP, ISSUE);
    let text = plain.replacen(
        "      - name: registered\n        creates: demo.apps.Account\n",
        "      - name: preferred\n        creates: demo.apps.Account\n        instance: account_id\n        \
         when: {exists: {in: redirect_uris, as: r, that: r == demo}}\n        \
         emits: [demo.apps.Registered]\n        payload:\n          \
         demo.apps.Registered: {account_id: {generated: true}}\n        sets:\n          \
         redirect_uris: input.redirect_uris\n      \
         - name: registered\n        creates: demo.apps.Account\n",
        1,
    );
    assert_ne!(text, plain, "the guard is written");
    let result = synthesis(&text);
    assert_eq!(
        refusals_about(&result, "demo.apps.Register"),
        Vec::<String>::new()
    );
    let demo = Node::Text("demo".to_owned());
    for (id, holds) in [
        ("demo.apps.Register/outcome/preferred", true),
        ("demo.apps.Register/outcome/registered", false),
    ] {
        let input = sent(scenario(&result, id), "demo.apps.Register");
        let Some(ScenarioValue::Literal { value }) = input.get("redirect_uris") else {
            panic!("a literal map: {input:?}")
        };
        assert_eq!(values(value).contains(&demo), holds, "{id}: {value:?}");
        assert!(!values(value).is_empty(), "{id}: {value:?}");
    }
}

/// A map's value is read at its declared type where a quantifier walks it: a `Timestamp` orders by
/// the instant it names, not its spelling. `00:30+01:00` is before `00:00Z` although its bytes sort
/// after.
#[test]
fn issue_240_a_timestamp_map_value_orders_by_its_instant() {
    let text = "format: ess/18
system: demo
version: v1
domain: demo.due
actors:
  - {name: demo.due.Clerk, may: [demo.due.Check]}
commands:
  - name: demo.due.Check
    input:
      - {name: due, type: 'Map<String, Timestamp>'}
    outcomes:
      - name: checked
        error: demo.due.Late
errors:
  - {name: demo.due.Late, summary: Late., fields: []}
";
    let ir = ir(text);
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.due.Check")
        .expect("declared");
    let input = BTreeMap::from([(
        "due".to_owned(),
        Node::Map(BTreeMap::from([(
            "a".to_owned(),
            Node::Text("2020-01-01T00:30:00+01:00".to_owned()),
        )])),
    )]);
    let facts = ess_conformance::flatten(&ir, command, &input).unwrap_or_else(|e| panic!("{e}"));
    let guard: ess_primitives::predicate::Predicate =
        serde_yaml::from_str("{exists: {in: due, as: r, that: 'r < 2020-01-01T00:00:00Z'}}")
            .expect("a predicate");
    assert_eq!(
        guard.evaluate(&facts),
        ess_primitives::predicate::Truth::True,
        "{guard}"
    );
}

// ---- the class: every quantifier and count, map and list, related and subject ----------------

/// Each guard over a stored collection, and the field type it is written over.
fn class() -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    for field in [MAP, LIST] {
        for predicate in [
            ISSUE,
            "{exists: {in: redirect_uris, as: r, that: r == input.application}}",
            "{forall: {in: redirect_uris, as: r, that: r != input.application}}",
            "{not: {forall: {in: redirect_uris, as: r, that: r != input.application}}}",
            "{exists: {in: redirect_uris, as: r, that: r == demo}}",
            "{not: {exists: {in: redirect_uris, as: r, that: r == demo}}}",
            "redirect_uris.count == 0",
            "redirect_uris.count > 1",
        ] {
            out.push((field, predicate));
        }
    }
    out
}

#[test]
fn issue_240_every_guard_over_a_stored_collection_is_synthesized_on_both_sides() {
    let mut failures = Vec::new();
    for (field, predicate) in class() {
        let result = synthesis(&model(field, predicate));
        for command in ["demo.apps.Redirect", "demo.apps.Launch"] {
            let refused = refusals_about(&result, command);
            if !refused.is_empty() {
                failures.push(format!("{field} / {predicate} / {command}: {refused:#?}"));
            }
        }
        for id in [REDIRECT_REFUSED, REDIRECTED, LAUNCH_REFUSED, LAUNCHED] {
            if !result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id)
            {
                failures.push(format!("{field} / {predicate}: no {id}"));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

// ---- the suite, run against a target and its mutants -----------------------------------------

/// Which of the class's guards the target evaluates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Guard {
    NotExistsSent,
    ExistsSent,
    ForallNotSent,
    NotForallNotSent,
    ExistsLiteral,
    NotExistsLiteral,
    CountZero,
    CountAboveOne,
    ForallEqSent,
}

impl Guard {
    fn predicate(self) -> &'static str {
        match self {
            Self::NotExistsSent => ISSUE,
            Self::ExistsSent => {
                "{exists: {in: redirect_uris, as: r, that: r == input.application}}"
            }
            Self::ForallNotSent => {
                "{forall: {in: redirect_uris, as: r, that: r != input.application}}"
            }
            Self::NotForallNotSent => {
                "{not: {forall: {in: redirect_uris, as: r, that: r != input.application}}}"
            }
            Self::ExistsLiteral => "{exists: {in: redirect_uris, as: r, that: r == demo}}",
            Self::NotExistsLiteral => {
                "{not: {exists: {in: redirect_uris, as: r, that: r == demo}}}"
            }
            Self::CountZero => "redirect_uris.count == 0",
            Self::CountAboveOne => "redirect_uris.count > 1",
            Self::ForallEqSent => {
                "{forall: {in: redirect_uris, as: r, that: r == input.application}}"
            }
        }
    }

    /// Whether the guard reads what the entries hold, rather than how many there are.
    fn reads_entries(self) -> bool {
        !matches!(self, Self::CountZero | Self::CountAboveOne)
    }

    fn holds(self, elements: &[Node], application: &str) -> bool {
        let sent = Node::Text(application.to_owned());
        let literal = Node::Text("demo".to_owned());
        match self {
            Self::NotExistsSent => !elements.contains(&sent),
            Self::ExistsSent => elements.contains(&sent),
            Self::ForallNotSent => elements.iter().all(|element| *element != sent),
            Self::NotForallNotSent => !elements.iter().all(|element| *element != sent),
            Self::ExistsLiteral => elements.contains(&literal),
            Self::NotExistsLiteral => !elements.contains(&literal),
            Self::CountZero => elements.is_empty(),
            Self::CountAboveOne => elements.len() > 1,
            Self::ForallEqSent => elements.iter().all(|element| *element == sent),
        }
    }

    /// The guard with `exists` read as `forall` and `forall` as `exists`; a count guard as written.
    fn swapped(self, elements: &[Node], application: &str) -> bool {
        let sent = Node::Text(application.to_owned());
        let literal = Node::Text("demo".to_owned());
        match self {
            Self::NotExistsSent => !elements.iter().all(|element| *element == sent),
            Self::ExistsSent => elements.iter().all(|element| *element == sent),
            Self::ForallNotSent => elements.iter().any(|element| *element != sent),
            Self::NotForallNotSent => !elements.iter().any(|element| *element != sent),
            Self::ExistsLiteral => elements.iter().all(|element| *element == literal),
            Self::NotExistsLiteral => !elements.iter().all(|element| *element == literal),
            Self::ForallEqSent => elements.contains(&sent),
            Self::CountZero | Self::CountAboveOne => self.holds(elements, application),
        }
    }
}

/// The class's guards that compare an element with the command's input (beyond10x/ess#240
/// correction 1), and `forall ==`, which only a row of several equal values holds non-vacuously.
const ELEMENTWISE: [Guard; 5] = [
    Guard::NotExistsSent,
    Guard::ExistsSent,
    Guard::ForallNotSent,
    Guard::NotForallNotSent,
    Guard::ForallEqSent,
];

const GUARDS: [Guard; 8] = [
    Guard::NotExistsSent,
    Guard::ExistsSent,
    Guard::ForallNotSent,
    Guard::NotForallNotSent,
    Guard::ExistsLiteral,
    Guard::NotExistsLiteral,
    Guard::CountZero,
    Guard::CountAboveOne,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Answers the model: a map's values, a list's elements, are what the quantifier binds.
    Correct,
    /// Never reads the entries: the guard never holds on an existing account.
    IgnoresEntries,
    /// The guard always holds on an existing account.
    AlwaysHolds,
    /// Reads a map's keys where the model reads its values.
    ReadsKeys,
    /// Holds exactly where the collection is empty, whatever it holds.
    ReadsEmptiness,
    /// Reads the first account stored rather than the one named.
    ReadsFirstRow,
    /// Reads only the first value of the collection, in the order the quantifier walks it.
    FirstValue,
    /// Reads only the last value.
    LastValue,
    /// Answers `forall` where the guard says `exists`, and `exists` where it says `forall`.
    Swapped,
}

#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    accounts: RefCell<Vec<(String, Node, Option<String>)>>,
    visits: RefCell<Vec<BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Store {
    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }

    fn token(&self) -> ess_primitives::consistency::ConsistencyToken {
        self.minted.set(self.minted.get() + 1);
        ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.minted.get()))
            .unwrap()
    }
}

struct Apps {
    guard: Guard,
    mode: Mode,
    store: Store,
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

fn refusal(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name))
        .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
}

impl Apps {
    fn new(guard: Guard, mode: Mode) -> Self {
        Self {
            guard,
            mode,
            store: Store::default(),
        }
    }

    /// Whether the guard refuses `application` on an account holding `collection`.
    fn refuses(&self, collection: &Node, application: &str) -> bool {
        match self.mode {
            Mode::Correct | Mode::ReadsFirstRow => {
                self.guard.holds(&values(collection), application)
            }
            Mode::IgnoresEntries => false,
            Mode::AlwaysHolds => true,
            Mode::ReadsKeys => self.guard.holds(
                &keys(collection)
                    .into_iter()
                    .map(Node::Text)
                    .collect::<Vec<_>>(),
                application,
            ),
            Mode::ReadsEmptiness => values(collection).is_empty(),
            Mode::FirstValue => self.guard.holds(
                &values(collection).into_iter().take(1).collect::<Vec<_>>(),
                application,
            ),
            Mode::LastValue => self.guard.holds(
                &values(collection)
                    .into_iter()
                    .rev()
                    .take(1)
                    .collect::<Vec<_>>(),
                application,
            ),
            Mode::Swapped => self.guard.swapped(&values(collection), application),
        }
    }
}

impl ConformanceTarget for Apps {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("apps-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.accounts.replace(Vec::new());
        self.store.visits.replace(Vec::new());
        self.store.published.replace(Vec::new());
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
        let store = &self.store;
        let result = match command.to_string().as_str() {
            "demo.apps.Register" => {
                let id = store.mint();
                let collection = request
                    .input
                    .get("redirect_uris")
                    .cloned()
                    .expect("the collection is sent");
                store
                    .accounts
                    .borrow_mut()
                    .push((id.clone(), collection, None));
                SemanticCommandResult::took(branch(&command, "registered")).emitting(
                    ObservedEvent::new("demo.apps.Registered".parse::<EventRef>().unwrap())
                        .with("account_id", Node::Text(id)),
                )
            }
            "demo.apps.Redirect" | "demo.apps.Launch" => {
                let account = text(request.input.get("account_id"));
                let application = text(request.input.get("application"));
                let redirect = command.to_string() == "demo.apps.Redirect";
                let found = {
                    let rows = store.accounts.borrow();
                    let named = rows.iter().find(|(id, _, _)| *id == account);
                    let read = if self.mode == Mode::ReadsFirstRow && redirect {
                        named.and(rows.first())
                    } else {
                        named
                    };
                    read.map(|(_, collection, _)| collection.clone())
                };
                match found {
                    None if redirect => refusal(&command, "no-account", "demo.apps.NoAccount"),
                    None => SemanticCommandResult::undeclared(),
                    Some(collection) if self.refuses(&collection, &application) => {
                        refusal(&command, "refused", "demo.apps.Refused")
                    }
                    Some(_) if redirect => {
                        let id = store.mint();
                        store.visits.borrow_mut().push(BTreeMap::from([
                            ("visit_id".to_owned(), Node::Text(id.clone())),
                            ("account_id".to_owned(), Node::Text(account)),
                            ("application".to_owned(), Node::Text(application)),
                            ("state".to_owned(), Node::Text("Issued".to_owned())),
                        ]));
                        SemanticCommandResult::took(branch(&command, "redirected")).emitting(
                            ObservedEvent::new("demo.apps.Visited".parse::<EventRef>().unwrap())
                                .with("visit_id", Node::Text(id)),
                        )
                    }
                    Some(_) => {
                        for row in store.accounts.borrow_mut().iter_mut() {
                            if row.0 == account {
                                row.2 = Some(application.clone());
                            }
                        }
                        SemanticCommandResult::took(branch(&command, "launched")).emitting(
                            ObservedEvent::new("demo.apps.Launched".parse::<EventRef>().unwrap())
                                .with("account_id", Node::Text(account)),
                        )
                    }
                }
            }
            _ => SemanticCommandResult::undeclared(),
        };
        for event in &result.direct_events {
            store.published.borrow_mut().push(event.clone());
        }
        Ok(result.with_consistency(store.token()))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "demo.apps.Accounts" => self
                .store
                .accounts
                .borrow()
                .iter()
                .map(|(id, collection, last)| {
                    let mut row = BTreeMap::from([
                        ("account_id".to_owned(), Node::Text(id.clone())),
                        ("redirect_uris".to_owned(), collection.clone()),
                        ("state".to_owned(), Node::Text("Active".to_owned())),
                    ]);
                    if let Some(last) = last {
                        row.insert("last".to_owned(), Node::Text(last.clone()));
                    }
                    row
                })
                .collect(),
            "demo.apps.Visits" => self.store.visits.borrow().clone(),
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .store
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

/// Every scenario about `command` and its status against `target`.
fn statuses(result: &Synthesis, target: &Apps, command: &str) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().starts_with(command))
        .map(|run| {
            (
                run.scenario.to_string(),
                if run.status == Status::Passed {
                    "passed".to_owned()
                } else {
                    format!("{:?}: {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, String>) -> Vec<&String> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn issue_240_a_target_answering_the_model_passes_and_every_mutant_fails() {
    let result = synthesis(&model(MAP, ISSUE));
    for command in ["demo.apps.Redirect/", "demo.apps.Launch/"] {
        let statuses = statuses(
            &result,
            &Apps::new(Guard::NotExistsSent, Mode::Correct),
            command,
        );
        assert!(!statuses.is_empty(), "{command} is run");
        assert!(failed(&statuses).is_empty(), "{statuses:#?}");
    }
    for (mode, fails) in [
        (Mode::IgnoresEntries, REDIRECT_REFUSED),
        (Mode::IgnoresEntries, LAUNCH_REFUSED),
        (Mode::ReadsEmptiness, REDIRECT_REFUSED),
        (Mode::ReadsEmptiness, LAUNCH_REFUSED),
        (Mode::ReadsKeys, REDIRECTED),
        (Mode::ReadsKeys, LAUNCHED),
        (Mode::AlwaysHolds, REDIRECTED),
        (Mode::AlwaysHolds, LAUNCHED),
        (Mode::ReadsFirstRow, REDIRECTED),
    ] {
        let command = if fails.starts_with("demo.apps.Redirect") {
            "demo.apps.Redirect/"
        } else {
            "demo.apps.Launch/"
        };
        let statuses = statuses(&result, &Apps::new(Guard::NotExistsSent, mode), command);
        assert!(
            failed(&statuses).contains(&&fails.to_owned()),
            "{mode:?} passes {fails}: {statuses:#?}"
        );
    }
}

/// Every guard of the class, over a map and over a list, read through a related row and through the
/// command's own subject: a target answering the model passes every scenario, and a target that
/// ignores the entries, reads them as always holding, reads only whether the collection is empty,
/// or — for a map — reads its keys, fails at least one scenario of each command.
#[test]
fn issue_240_every_guard_of_the_class_passes_the_model_and_fails_each_mutant() {
    let mut failures = Vec::new();
    for field in [MAP, LIST] {
        for guard in GUARDS {
            let result = synthesis(&model(field, guard.predicate()));
            for command in ["demo.apps.Redirect/", "demo.apps.Launch/"] {
                let correct = statuses(&result, &Apps::new(guard, Mode::Correct), command);
                if correct.is_empty() || !failed(&correct).is_empty() {
                    failures.push(format!(
                        "{field} / {guard:?} / {command}: the model's target: {correct:#?}"
                    ));
                }
                let mut mutants = vec![Mode::IgnoresEntries, Mode::AlwaysHolds];
                if guard.reads_entries() {
                    mutants.push(Mode::ReadsEmptiness);
                    if field == MAP {
                        mutants.push(Mode::ReadsKeys);
                    }
                }
                for mode in mutants {
                    let ran = statuses(&result, &Apps::new(guard, mode), command);
                    if failed(&ran).is_empty() {
                        failures.push(format!("{field} / {guard:?} / {command}: {mode:?} passes"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Every guard comparing an element of a stored map or list with the input, read through a related
/// row and through the command's own subject: a target reading only the first value, only the last,
/// or `forall` for `exists` (and the reverse) fails at least one scenario of each command, because
/// the row a quantifier one element decides is arranged on holds several values with that element
/// neither first nor last, and the row the whole collection decides holds several values
/// (beyond10x/ess#240, correction 1).
#[test]
fn issue_240_a_target_reading_one_value_or_the_other_quantifier_fails_a_scenario() {
    let mut failures = Vec::new();
    for field in [MAP, LIST] {
        for guard in ELEMENTWISE {
            let result = synthesis(&model(field, guard.predicate()));
            for command in ["demo.apps.Redirect", "demo.apps.Launch"] {
                let refused = refusals_about(&result, command);
                if !refused.is_empty() {
                    failures.push(format!("{field} / {guard:?} / {command}: {refused:#?}"));
                }
            }
            for command in ["demo.apps.Redirect/", "demo.apps.Launch/"] {
                let correct = statuses(&result, &Apps::new(guard, Mode::Correct), command);
                if correct.is_empty() || !failed(&correct).is_empty() {
                    failures.push(format!(
                        "{field} / {guard:?} / {command}: the model's target: {correct:#?}"
                    ));
                }
                for mode in [Mode::FirstValue, Mode::LastValue, Mode::Swapped] {
                    let ran = statuses(&result, &Apps::new(guard, mode), command);
                    if failed(&ran).is_empty() {
                        failures.push(format!("{field} / {guard:?} / {command}: {mode:?} passes"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
