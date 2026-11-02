//! Security pass on W1 unit SYN1: the grant refusal beside a ranked or filtered view, and the
//! denied read of every ungranted actor of a view's own domain.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::Note;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioId, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn build(files: &[(&str, String)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert((*label).to_owned(), text.clone());
        parsed.push((Source::new(*label), raw));
    }
    let specification =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("assemble: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("compile: {errors}"))
}

fn verdicts(suite: &ConformanceSuite, model: EssIr) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).expect("admits");
    Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn failing(verdicts: &[(String, Status)]) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.clone())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Story 2: every ungranted actor of the view's own domain.

const TWO_SYSTEM: &str = "format: ess/32
system: demo
version: v1
domains:
  - demo.public
  - demo.tasks
";
const TWO_COMPONENTS: &str = "components:
  - component: demo-service
    owns: {domains: [demo.public, demo.tasks]}
    accepts: {commands: [demo.tasks.OpenTask]}
    publishes: {events: [demo.tasks.TaskOpened]}
    reached_by: network
";
const TWO_PUBLIC: &str = "domain: demo.public

actors:
  - name: demo.public.Client
    may: [demo.tasks.OpenTask]
";
const TWO_TASKS: &str = "domain: demo.tasks

entities:
  - name: demo.tasks.Task
    identity: {name: id, type: String}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}

events:
  - name: demo.tasks.TaskOpened
    fields:
      - {name: id, type: String}

commands:
  - name: demo.tasks.OpenTask
    input:
      - {name: id, type: String}
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.tasks.Task
        instance: id
        sets: {title: input.title}
        emits: [demo.tasks.TaskOpened]
        payload:
          demo.tasks.TaskOpened: {id: input.id}

views:
  - name: demo.tasks.AllTasks
    source: demo.tasks.Task
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: title, type: String}

actors:
  - name: demo.tasks.Agent
    may: [demo.tasks.OpenTask, demo.tasks.AllTasks]
  - name: demo.tasks.Client
    may: [demo.tasks.OpenTask]
  - name: demo.tasks.Guest
    may: [demo.tasks.OpenTask]
";

const ALL_TASKS_DENIED: &str = "demo.tasks.AllTasks/grant/read/denied";

/// The three-client model; `leaking` names the ungranted actor of `demo.tasks` the target
/// wrongly admits to `AllTasks`, if any.
fn three_clients(leaking: Option<&str>) -> EssIr {
    let mut tasks = TWO_TASKS.to_owned();
    if let Some(actor) = leaking {
        let from = format!("  - name: {actor}\n    may: [demo.tasks.OpenTask]\n");
        let to = format!("  - name: {actor}\n    may: [demo.tasks.OpenTask, demo.tasks.AllTasks]\n");
        assert!(tasks.contains(&from), "{actor} is declared");
        tasks = tasks.replace(&from, &to);
    }
    build(&[
        ("system.yaml", TWO_SYSTEM.to_owned()),
        ("components.yaml", TWO_COMPONENTS.to_owned()),
        ("domains/public.yaml", TWO_PUBLIC.to_owned()),
        ("domains/tasks.yaml", tasks),
    ])
}

/// A target admitting any one ungranted actor of the view's domain, not only the lowest-named,
/// fails the denied read and nothing else; the honest target passes every scenario.
#[test]
fn w1_syn1_a_target_admitting_any_ungranted_actor_of_the_views_domain_fails() {
    let suite = ess_conformance::synthesize(&three_clients(None)).suite;
    let honest = verdicts(&suite, three_clients(None));
    assert!(failing(&honest).is_empty(), "{honest:#?}");
    for leaked in ["demo.tasks.Client", "demo.tasks.Guest"] {
        let leaking = verdicts(&suite, three_clients(Some(leaked)));
        assert_eq!(
            failing(&leaking),
            [ALL_TASKS_DENIED],
            "a target admitting {leaked}: {leaking:#?}"
        );
    }
}

/// The denied read names no actor of another domain while the view's own domain has an
/// ungranted one, and names each ungranted actor of the domain once.
#[test]
fn w1_syn1_the_denied_read_names_no_actor_of_another_domain() {
    let suite = ess_conformance::synthesize(&three_clients(None)).suite;
    let id: ScenarioId = ALL_TASKS_DENIED.parse().unwrap();
    let scenario = suite.scenario(&id).expect("filed");
    let read_as: Vec<Option<String>> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ReadAs { actor } => Some(actor.as_ref().map(ToString::to_string)),
            _ => None,
        })
        .collect();
    assert_eq!(
        read_as,
        [
            Some("demo.tasks.Client".to_owned()),
            Some("demo.tasks.Guest".to_owned()),
            None
        ]
    );
    let refused: Vec<Option<String>> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectNotGranted { actor, .. } => {
                Some(actor.as_ref().map(ToString::to_string))
            }
            _ => None,
        })
        .collect();
    assert_eq!(refused, read_as, "each read is followed by its own refusal");
}

// ---------------------------------------------------------------------------------------------
// Story 1: grant/denied beside a ranked or filtered view.

const SHELF_SYSTEM: &str = "format: ess/32
system: library
version: v1
domains:
  - library.shelf
";
const SHELF_COMPONENTS: &str = "components:
  - component: shelf-service
    owns: {domains: [library.shelf]}
    accepts: {commands: [library.shelf.CatalogueBook, library.shelf.LendBook]}
    publishes: {events: [library.shelf.BookCatalogued, library.shelf.BookLent]}
    reached_by: network
";
const SHELF_DOMAIN: &str = "domain: library.shelf

types:
  - {name: library.shelf.BookId, kind: newtype, of: Uuid}

entities:
  - name: library.shelf.Book
    identity: {name: book_id, type: library.shelf.BookId}
    fields:
      - {name: position, type: Integer}
    lifecycle:
      initial: Shelved
      states: [Shelved, Lent]
      terminal: [Lent]
      transitions:
        - {name: lend, from: [Shelved], to: Lent}

actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook, library.shelf.LendBook]
  - name: library.shelf.Reader

commands:
  - name: library.shelf.CatalogueBook
    input:
      - {name: position, type: Integer}
    outcomes:
      - name: catalogued
        creates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookCatalogued]
        payload:
          library.shelf.BookCatalogued: {book_id: {generated: true}}
        sets: {position: input.position}

  - name: library.shelf.LendBook
    input:
      - {name: book_id, type: library.shelf.BookId}
    outcomes:
      - name: lent
        moves: library.shelf.Book.lend
        instance: book_id
        emits: [library.shelf.BookLent]
        payload:
          library.shelf.BookLent: {book_id: input.book_id}
      - name: wrong-state
        wrong_state: true
        error: library.shelf.BookStateConflict

errors:
  - name: library.shelf.BookStateConflict
    fields:
      - {name: state, type: library.shelf.Book.State}

events:
  - name: library.shelf.BookCatalogued
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookLent
    fields:
      - {name: book_id, type: library.shelf.BookId}

views:
  - name: library.shelf.ShelfOrder
    source: library.shelf.Book
    consistency: read_your_writes
    VIEW_SHAPE
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
";

const LEND_DENIED: &str = "library.shelf.LendBook/grant/denied";

/// The shelf model with `shape` lines in its view (indented four spaces, joined by newlines);
/// `reader_lends` is the target that skips the grant check.
fn shelf(shape: &[&str], reader_lends: bool) -> EssIr {
    let lines = if shape.is_empty() {
        String::new()
    } else {
        shape.join("\n    ")
    };
    let mut domain = if lines.is_empty() {
        SHELF_DOMAIN.replace("    VIEW_SHAPE\n", "")
    } else {
        SHELF_DOMAIN.replace("VIEW_SHAPE", &lines)
    };
    if reader_lends {
        domain = domain.replace(
            "  - name: library.shelf.Reader\n",
            "  - name: library.shelf.Reader\n    may: [library.shelf.LendBook]\n",
        );
    }
    build(&[
        ("system.yaml", SHELF_SYSTEM.to_owned()),
        ("components.yaml", SHELF_COMPONENTS.to_owned()),
        ("domains/shelf.yaml", domain),
    ])
}

fn assert_denied_filed_and_discriminating(shape: &[&str]) {
    let synthesis = ess_conformance::synthesize(&shelf(shape, false));
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == LEND_DENIED),
        "shape {shape:?}: grant/denied withheld: {:#?}",
        synthesis.notes
    );
    assert!(
        !synthesis.notes.iter().any(|note| matches!(
            note,
            Note::GrantDeniedUnwitnessed { command } if command.to_string() == "library.shelf.LendBook"
        )),
        "shape {shape:?}: {:#?}",
        synthesis.notes
    );
    let honest = verdicts(&synthesis.suite, shelf(shape, false));
    assert!(failing(&honest).is_empty(), "shape {shape:?}: {honest:#?}");
    let skipping = verdicts(&synthesis.suite, shelf(shape, true));
    assert_eq!(
        skipping
            .iter()
            .find(|(id, _)| id == LEND_DENIED)
            .map(|(_, status)| *status),
        Some(Status::Failed),
        "shape {shape:?}: {skipping:#?}"
    );
}

/// A view of the entity filtered on its state beside the moving command.
#[test]
fn w1_syn1_grant_denied_is_synthesized_beside_a_filtered_view() {
    assert_denied_filed_and_discriminating(&["filter: state == Shelved"]);
}

/// A view both filtered and ranked beside the moving command.
#[test]
fn w1_syn1_grant_denied_is_synthesized_beside_a_ranked_and_filtered_view() {
    assert_denied_filed_and_discriminating(&["filter: state == Shelved", "order_by: [position asc]"]);
}

/// A view ranked descending beside the moving command.
#[test]
fn w1_syn1_grant_denied_is_synthesized_beside_a_descending_ranked_view() {
    assert_denied_filed_and_discriminating(&["order_by: [position desc]"]);
}

/// Where a scenario ends in the command's accepting send, the denied scenario is built from such
/// a scenario, as before the change: its arrangement is that scenario's prefix, step for step.
#[test]
fn w1_syn1_an_unranked_denied_template_still_ends_in_the_command_send() {
    let synthesis = ess_conformance::synthesize(&shelf(&[], false));
    let id: ScenarioId = LEND_DENIED.parse().unwrap();
    let denied = synthesis.suite.scenario(&id).expect("filed");
    let refused = denied
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. }))
        .expect("refusal");
    let at = refused - 1;
    let is_send = |step: &ScenarioStep| {
        matches!(
            step,
            ScenarioStep::ExecuteCommand { .. } | ScenarioStep::ExecuteCommandWithoutInput { .. }
        )
    };
    let lends = |step: &ScenarioStep| match step {
        ScenarioStep::ExecuteCommand { command, .. }
        | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => {
            command.to_string() == "library.shelf.LendBook"
        }
        _ => false,
    };
    let from_a_last_send = synthesis.suite.scenarios.iter().any(|(other, scenario)| {
        !other.to_string().contains("/grant/")
            && scenario.steps.len() > at
            && lends(&scenario.steps[at])
            && scenario.steps[at + 1..].iter().all(|step| !is_send(step))
            && scenario.steps[..at] == denied.steps[..at]
    });
    assert!(from_a_last_send, "{:#?}", denied.steps);
}
