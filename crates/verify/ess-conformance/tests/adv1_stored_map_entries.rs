//! Adversary pass 1 for beyond10x/ess#240: a stored `Map` a guard reads by its entries.
//!
//! Each case either asserts that a guard over a stored map's values is synthesized on both sides for
//! a value shape the unit's own suite does not cover (a non-text value, a struct value, a nested
//! collection), or runs the synthesized suite against a target that answers the model and against a
//! mutant the suite is meant to catch: one that reads only one of the map's values, one that swaps
//! `exists` for `forall`, and one that reads a sibling map of the same type.
#![allow(clippy::too_many_lines, clippy::type_complexity)]

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

/// An account holds two collections of the same type, both set whole from `Register`'s input.
/// `Redirect` reads the account named by `input.account_id`; `Launch` reads its own subject. Both are
/// refused where `{pred}` holds.
const TEMPLATE: &str = "format: ess/18
system: demo
version: v1
domain: demo.apps
types:
  - {name: demo.apps.AccountId, kind: newtype, of: Uuid}
  - {name: demo.apps.VisitId, kind: newtype, of: Uuid}
  - name: demo.apps.Uri
    kind: struct
    fields:
      - {name: href, type: String}
      - {name: label, type: String}
errors:
  - {name: demo.apps.NoAccount, summary: No account., fields: []}
  - {name: demo.apps.Refused, summary: Refused., fields: []}
entities:
  - name: demo.apps.Account
    identity: {name: account_id, type: demo.apps.AccountId}
    fields:
      - {name: redirect_uris, type: '{field}'}
      - {name: other_uris, type: '{field}'}
      - {name: last, type: 'Optional<{app}>'}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.apps.Visit
    identity: {name: visit_id, type: demo.apps.VisitId}
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: '{app}'}
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
      - {name: other_uris, type: '{field}'}
    outcomes:
      - name: registered
        creates: demo.apps.Account
        instance: account_id
        emits: [demo.apps.Registered]
        payload:
          demo.apps.Registered: {account_id: {generated: true}}
        sets:
          redirect_uris: input.redirect_uris
          other_uris: input.other_uris
  - name: demo.apps.Redirect
    input:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: '{app}'}
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
      - {name: application, type: '{app}'}
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
      - {name: other_uris, type: '{field}'}
      - {name: last, type: 'Optional<{app}>'}
  - name: demo.apps.Visits
    source: demo.apps.Visit
    consistency: read_your_writes
    fields:
      - {name: visit_id, type: demo.apps.VisitId}
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: '{app}'}
";

const MAP: &str = "Map<String, String>";

const NOT_EXISTS: &str =
    "{not: {exists: {in: redirect_uris, as: r, that: r == input.application}}}";
const EXISTS: &str = "{exists: {in: redirect_uris, as: r, that: r == input.application}}";
const FORALL_EQ: &str = "{forall: {in: redirect_uris, as: r, that: r == input.application}}";

fn model(field: &str, app: &str, predicate: &str) -> String {
    TEMPLATE
        .replace("{field}", field)
        .replace("{app}", app)
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

fn scenario<'a>(result: &'a Synthesis, id: &str) -> Option<&'a ConformanceScenario> {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// The `Register` inputs of a scenario, keyed by the instance each is captured as.
fn registered(scenario: &ConformanceScenario) -> Vec<(String, BTreeMap<String, Node>)> {
    let mut out = Vec::new();
    let mut pending = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.apps.Register" =>
            {
                let mut row = BTreeMap::new();
                for (name, value) in input {
                    if let ScenarioValue::Literal { value } = value {
                        row.insert(name.clone(), value.clone());
                    }
                }
                pending = Some(row);
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "demo.apps.Account" => {
                if let Some(row) = pending.take() {
                    out.push((instance.to_string(), row));
                }
            }
            _ => {}
        }
    }
    out
}

/// The row the last `command` of the scenario points at, and the application it sends.
fn pointed_at(
    scenario: &ConformanceScenario,
    command: &str,
) -> Option<(BTreeMap<String, Node>, Node)> {
    let input = scenario.steps.iter().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand {
            command: sent,
            input,
            ..
        } if sent.to_string() == command => Some(input.clone()),
        _ => None,
    })?;
    let Some(ScenarioValue::Instance { instance }) = input.get("account_id") else {
        return None;
    };
    let row = registered(scenario)
        .into_iter()
        .find(|(name, _)| *name == instance.to_string())?
        .1;
    let Some(ScenarioValue::Literal { value }) = input.get("application") else {
        return None;
    };
    Some((row, value.clone()))
}

fn values(collection: Option<&Node>) -> Vec<Node> {
    match collection {
        Some(Node::Map(entries)) => entries.values().cloned().collect(),
        Some(Node::Seq(items)) => items.clone(),
        _ => Vec::new(),
    }
}

// ---- the target ------------------------------------------------------------------------------

type Row = BTreeMap<String, Node>;
type Refuses = Box<dyn Fn(&Row, &Node) -> bool>;

#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    accounts: RefCell<Vec<(String, Row, Option<Node>)>>,
    visits: RefCell<Vec<Row>>,
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
    refuses: Refuses,
    store: Store,
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn refusal(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name))
        .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
}

fn account_text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

impl Apps {
    fn new(refuses: impl Fn(&Row, &Node) -> bool + 'static) -> Self {
        Self {
            refuses: Box::new(refuses),
            store: Store::default(),
        }
    }
}

impl ConformanceTarget for Apps {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adv1-apps", "1"))
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
                let mut row = Row::new();
                for name in ["redirect_uris", "other_uris"] {
                    let value = request.input.get(name).cloned().expect("sent");
                    row.insert(name.to_owned(), value);
                }
                store.accounts.borrow_mut().push((id.clone(), row, None));
                SemanticCommandResult::took(branch(&command, "registered")).emitting(
                    ObservedEvent::new("demo.apps.Registered".parse::<EventRef>().unwrap())
                        .with("account_id", Node::Text(id)),
                )
            }
            "demo.apps.Redirect" | "demo.apps.Launch" => {
                let account = account_text(request.input.get("account_id"));
                let application = request.input.get("application").cloned().expect("sent");
                let redirect = command.to_string() == "demo.apps.Redirect";
                let found = store
                    .accounts
                    .borrow()
                    .iter()
                    .find(|(id, _, _)| *id == account)
                    .map(|(_, row, _)| row.clone());
                match found {
                    None if redirect => refusal(&command, "no-account", "demo.apps.NoAccount"),
                    None => SemanticCommandResult::undeclared(),
                    Some(row) if (self.refuses)(&row, &application) => {
                        refusal(&command, "refused", "demo.apps.Refused")
                    }
                    Some(_) if redirect => {
                        let id = store.mint();
                        store.visits.borrow_mut().push(BTreeMap::from([
                            ("visit_id".to_owned(), Node::Text(id.clone())),
                            ("account_id".to_owned(), Node::Text(account)),
                            ("application".to_owned(), application),
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
        let rows: Vec<Row> = match request.view.to_string().as_str() {
            "demo.apps.Accounts" => self
                .store
                .accounts
                .borrow()
                .iter()
                .map(|(id, stored, last)| {
                    let mut row = stored.clone();
                    row.insert("account_id".to_owned(), Node::Text(id.clone()));
                    row.insert("state".to_owned(), Node::Text("Active".to_owned()));
                    if let Some(last) = last {
                        row.insert("last".to_owned(), last.clone());
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

/// Every scenario of `command` and its status against `target`.
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
                    format!("{:?}", run.status)
                },
            )
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, String>) -> usize {
    statuses
        .values()
        .filter(|status| *status != "passed")
        .count()
}

const COMMANDS: [&str; 2] = ["demo.apps.Redirect/", "demo.apps.Launch/"];

/// The map a guard reads, for the model's target and for a mutant.
fn map_values(row: &Row, field: &str) -> Vec<Node> {
    values(row.get(field))
}

/// The collections every scenario of a suite registers, for the failure message.
fn arranged(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .iter()
        .flat_map(|(id, scenario)| {
            registered(scenario)
                .into_iter()
                .map(move |(instance, row)| format!("{id} {instance}: {row:?}"))
        })
        .collect()
}

// ---- class 3: only the first value witnessed -------------------------------------------------

/// A map with two entries, only the second of which matches, is never arranged: every arranged map
/// holds one entry, so a target that reads only the first value in key order (or only the last)
/// answers every scenario exactly as the model does. The guard quantifies over *every* value, and a
/// suite that cannot tell `exists` from "the first value is" has witnessed only the first element.
#[test]
fn adv1_a_target_reading_only_the_first_map_value_fails_a_scenario() {
    let mut survivors = Vec::new();
    for (name, predicate, holds) in [
        (
            "not exists ==",
            NOT_EXISTS,
            (|values: &[Node], app: &Node| !values.contains(app)) as fn(&[Node], &Node) -> bool,
        ),
        ("exists ==", EXISTS, |values: &[Node], app: &Node| {
            values.contains(app)
        }),
    ] {
        let result = synthesis(&model(MAP, "String", predicate));
        for command in COMMANDS {
            let correct = statuses(
                &result,
                &Apps::new(move |row, app| holds(&map_values(row, "redirect_uris"), app)),
                command,
            );
            assert!(
                !correct.is_empty() && failed(&correct) == 0,
                "{name} {command}: the model's target passes: {correct:#?}"
            );
            for (mutant, pick) in [
                (
                    "first value",
                    (|v: Vec<Node>| v.into_iter().take(1).collect()) as fn(Vec<Node>) -> Vec<Node>,
                ),
                ("last value", |v: Vec<Node>| {
                    v.into_iter().rev().take(1).collect()
                }),
            ] {
                let ran = statuses(
                    &result,
                    &Apps::new(move |row, app| holds(&pick(map_values(row, "redirect_uris")), app)),
                    command,
                );
                if failed(&ran) == 0 {
                    survivors.push(format!("{name} / {command}: a target reading only the {mutant} passes every scenario {ran:?}"));
                }
            }
        }
        if !survivors.is_empty() {
            survivors.push(format!("arranged: {:#?}", arranged(&result)));
        }
    }
    assert!(survivors.is_empty(), "{survivors:#?}");
}

/// The model says `exists`; a target that answers `forall` instead agrees with it on every map of at
/// most one entry, so only a map holding one matching and one other value (or an empty map, where
/// `forall` is vacuously true and `exists` false) tells them apart.
#[test]
fn adv1_a_target_that_reads_forall_where_the_model_says_exists_fails_a_scenario() {
    let result = synthesis(&model(MAP, "String", EXISTS));
    let mut survivors = Vec::new();
    for command in COMMANDS {
        let ran = statuses(
            &result,
            &Apps::new(|row, app| map_values(row, "redirect_uris").iter().all(|v| v == app)),
            command,
        );
        if failed(&ran) == 0 {
            survivors.push(format!("{command}: {ran:?}"));
        }
    }
    assert!(
        survivors.is_empty(),
        "forall-for-exists passes: {survivors:#?}\narranged: {:#?}",
        arranged(&result)
    );
}

/// `forall r: r == input.application` holds on an empty map and on a map every value of which is
/// the application. Both sides are synthesized, and a target that answers "the map is empty" — which
/// agrees with the model wherever the holding side is witnessed only vacuously — fails.
#[test]
fn adv1_forall_equal_is_witnessed_on_a_non_empty_map() {
    let result = synthesis(&model(MAP, "String", FORALL_EQ));
    let mut failures = Vec::new();
    for command in ["demo.apps.Redirect", "demo.apps.Launch"] {
        let refused = refusals_about(&result, command);
        if !refused.is_empty() {
            failures.push(format!("{command}: {refused:#?}"));
        }
    }
    for command in COMMANDS {
        let correct = statuses(
            &result,
            &Apps::new(|row, app| map_values(row, "redirect_uris").iter().all(|v| v == app)),
            command,
        );
        if correct.is_empty() || failed(&correct) != 0 {
            failures.push(format!("{command}: the model's target: {correct:#?}"));
        }
        let ran = statuses(
            &result,
            &Apps::new(|row, _| map_values(row, "redirect_uris").is_empty()),
            command,
        );
        if failed(&ran) == 0 {
            failures.push(format!(
                "{command}: a target reading only emptiness passes {ran:?}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{failures:#?}\narranged: {:#?}",
        arranged(&result)
    );
}

// ---- class 1: two maps of one type -----------------------------------------------------------

/// The guard reads `other_uris`; a target that reads `redirect_uris`, a field of the same type set
/// from a same-typed input, fails at least one scenario of each command.
#[test]
fn adv1_a_target_reading_the_sibling_map_fails_a_scenario() {
    let predicate = "{not: {exists: {in: other_uris, as: r, that: r == input.application}}}";
    let result = synthesis(&model(MAP, "String", predicate));
    let mut failures = Vec::new();
    for command in ["demo.apps.Redirect", "demo.apps.Launch"] {
        let refused = refusals_about(&result, command);
        if !refused.is_empty() {
            failures.push(format!("{command}: {refused:#?}"));
        }
    }
    for command in COMMANDS {
        let correct = statuses(
            &result,
            &Apps::new(|row, app| !map_values(row, "other_uris").contains(app)),
            command,
        );
        if correct.is_empty() || failed(&correct) != 0 {
            failures.push(format!("{command}: the model's target: {correct:#?}"));
        }
        let ran = statuses(
            &result,
            &Apps::new(|row, app| !map_values(row, "redirect_uris").contains(app)),
            command,
        );
        if failed(&ran) == 0 {
            failures.push(format!(
                "{command}: a target reading redirect_uris passes {ran:?}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{failures:#?}\narranged: {:#?}",
        arranged(&result)
    );
}

// ---- value shapes the unit's suite does not cover --------------------------------------------

/// Synthesizes `{pred}` over a stored `field` compared with an `app`-typed input and checks that the
/// refusal and the accepting default are both synthesized for each command, and that the accepting
/// related row holds the application sent where `contains` says it does and the refused one not.
fn both_sides(field: &str, app: &str, predicate: &str, contains: fn(&Row, &Node) -> bool) {
    let result = synthesis(&model(field, app, predicate));
    let mut failures = Vec::new();
    for command in ["demo.apps.Redirect", "demo.apps.Launch"] {
        let refused = refusals_about(&result, command);
        if !refused.is_empty() {
            failures.push(format!("{command}: {refused:#?}"));
        }
    }
    for (id, holds) in [
        ("demo.apps.Redirect/outcome/refused", false),
        ("demo.apps.Redirect/outcome/redirected", true),
        ("demo.apps.Launch/outcome/refused", false),
        ("demo.apps.Launch/outcome/launched", true),
    ] {
        let Some(found) = scenario(&result, id) else {
            failures.push(format!("no {id}"));
            continue;
        };
        let command = id.split('/').next().expect("an id");
        match pointed_at(found, command) {
            Some((row, app)) => {
                if contains(&row, &app) != holds {
                    failures.push(format!("{id}: the row {row:?} / {app:?} holds={}", !holds));
                }
            }
            None if command == "demo.apps.Redirect" => {
                failures.push(format!("{id}: no arranged row pointed at"));
            }
            None => {}
        }
    }
    let mut ran = Vec::new();
    for command in COMMANDS {
        let correct = statuses(
            &result,
            &Apps::new(move |row, app| !contains(row, app)),
            command,
        );
        if correct.is_empty() || failed(&correct) != 0 {
            ran.push(format!("{command}: {correct:#?}"));
        }
    }
    if !ran.is_empty() {
        failures.push(format!("the model's target: {ran:#?}"));
    }
    assert!(
        failures.is_empty(),
        "{field} / {app} / {predicate}\n{failures:#?}"
    );
}

#[test]
fn adv1_an_integer_map_value_is_grounded_on_both_sides() {
    both_sides("Map<String, Integer>", "Integer", NOT_EXISTS, |row, app| {
        map_values(row, "redirect_uris").contains(app)
    });
}

#[test]
fn adv1_a_timestamp_map_value_is_grounded_on_both_sides() {
    both_sides(
        "Map<String, Timestamp>",
        "Timestamp",
        NOT_EXISTS,
        |row, app| map_values(row, "redirect_uris").contains(app),
    );
}

#[test]
fn adv1_a_struct_map_value_is_grounded_on_both_sides() {
    both_sides(
        "Map<String, demo.apps.Uri>",
        "String",
        "{not: {exists: {in: redirect_uris, as: r, that: r.href == input.application}}}",
        |row, app| {
            map_values(row, "redirect_uris")
                .iter()
                .any(|value| match value {
                    Node::Map(members) => members.get("href") == Some(app),
                    _ => false,
                })
        },
    );
}

#[test]
fn adv1_a_map_of_lists_is_grounded_on_both_sides() {
    both_sides(
        "Map<String, List<String>>",
        "String",
        "{not: {exists: {in: redirect_uris, as: l, that: {exists: {in: l, as: r, that: r == input.application}}}}}",
        |row, app| {
            map_values(row, "redirect_uris")
                .iter()
                .any(|list| values(Some(list)).contains(app))
        },
    );
}
