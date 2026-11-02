//! Adversary pass on wave W1 unit SYN1: the two stories
//! `story:grant-denied-is-synthesized-beside-a-ranked-view` (beyond10x/ess#712) and
//! `story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain`, driven on fixtures the
//! unit's own tests do not build.
//!
//! - #712 names a command that "moves or updates" an existing row; the unit's fixture only moves.
//!   Here an updating command, an update of the ranking field itself, and two ranked views.
//! - The denied read now reads as several actors in one scenario. Here a target that leaks only
//!   the second of them, the same target through the Go runtime, and a system actor and an
//!   attributed actor of the view's own domain.

mod support_go;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ActorRef;
use ess_conformance::synthesize::Note;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioId, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::QualifiedName;

fn assemble(files: &[(&str, String)]) -> EssIr {
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

fn verdicts(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).expect("admits");
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(results: &[(String, Status)]) -> Vec<&str> {
    results
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

// ---- story 1: grant/denied beside a ranked view ------------------------------------------------

const SHELF_SYSTEM: &str = "format: ess/32
system: library
version: v1
domains:
  - library.shelf
";

/// The shelf of #712 with a third command, `RelabelBook` or `RepositionBook`, that updates an
/// existing row without moving it.
const SHELF_DOMAIN: &str = "domain: library.shelf

types:
  - {name: library.shelf.BookId, kind: newtype, of: Uuid}

entities:
  - name: library.shelf.Book
    identity: {name: book_id, type: library.shelf.BookId}
    fields:
      - {name: position, type: Integer}
      - {name: label, type: String}
    lifecycle:
      initial: Shelved
      states: [Shelved, Lent]
      terminal: [Lent]
      transitions:
        - {name: lend, from: [Shelved], to: Lent}

actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook, library.shelf.LendBook, library.shelf.RelabelBook, library.shelf.RepositionBook]
  - name: library.shelf.Reader

commands:
  - name: library.shelf.CatalogueBook
    input:
      - {name: position, type: Integer}
      - {name: label, type: String}
    outcomes:
      - name: catalogued
        creates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookCatalogued]
        payload:
          library.shelf.BookCatalogued: {book_id: {generated: true}}
        sets: {position: input.position, label: input.label}

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

  - name: library.shelf.RelabelBook
    input:
      - {name: book_id, type: library.shelf.BookId}
      - {name: label, type: String}
    outcomes:
      - name: relabelled
        updates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookRelabelled]
        payload:
          library.shelf.BookRelabelled: {book_id: input.book_id}
        sets: {label: input.label}

  - name: library.shelf.RepositionBook
    input:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
    outcomes:
      - name: repositioned
        updates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookRepositioned]
        payload:
          library.shelf.BookRepositioned: {book_id: input.book_id}
        sets: {position: input.position}

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
  - name: library.shelf.BookRelabelled
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookRepositioned
    fields:
      - {name: book_id, type: library.shelf.BookId}

views:
  - name: library.shelf.ShelfOrder
    source: library.shelf.Book
    consistency: read_your_writes
    order_by: [position asc]
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
      - {name: label, type: String}
";

const SECOND_RANKED_VIEW: &str = "  - name: library.shelf.ShelfReversed
    source: library.shelf.Book
    consistency: read_your_writes
    order_by: [position desc]
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
";

/// The shelf, ranked or not, with `second` adding a second ranked view, and `reader_may` the
/// commands a target that skips the grant check lets `Reader` send.
fn shelf(ranked: bool, second: bool, reader_may: &[&str]) -> EssIr {
    let mut domain = SHELF_DOMAIN.to_owned();
    if !ranked {
        domain = domain.replace("    order_by: [position asc]\n", "");
    }
    if second {
        domain.push_str(SECOND_RANKED_VIEW);
    }
    if !reader_may.is_empty() {
        domain = domain.replace(
            "  - name: library.shelf.Reader\n",
            &format!(
                "  - name: library.shelf.Reader\n    may: [{}]\n",
                reader_may.join(", ")
            ),
        );
    }
    let components = "components:
  - component: shelf-service
    owns: {domains: [library.shelf]}
    accepts: {commands: [library.shelf.CatalogueBook, library.shelf.LendBook, library.shelf.RelabelBook, library.shelf.RepositionBook]}
    publishes: {events: [library.shelf.BookCatalogued, library.shelf.BookLent, library.shelf.BookRelabelled, library.shelf.BookRepositioned]}
    reached_by: network
"
    .to_owned();
    assemble(&[
        ("system.yaml", SHELF_SYSTEM.to_owned()),
        ("components.yaml", components),
        ("domains/shelf.yaml", domain),
    ])
}

/// Each command on an existing row owes its `grant/denied` with or without ranking: filed, no
/// unwitnessed note, passing the interpreter, failing a target that lets `Reader` send it.
fn assert_refusal_owed(command: &str, second: bool) {
    let denied = format!("{command}/grant/denied");
    for ranked in [false, true] {
        let synthesis = ess_conformance::synthesize(&shelf(ranked, second, &[]));
        let filed = synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| matches!(id, ScenarioId::Grant { .. }) && id.to_string() == denied);
        let unwitnessed = synthesis.notes.iter().any(|note| {
            matches!(note, Note::GrantDeniedUnwitnessed { command: named } if named.to_string() == command)
        });
        assert!(
            filed && !unwitnessed,
            "story:grant-denied-is-synthesized-beside-a-ranked-view: ranked {ranked}, second \
             ranked view {second}: `{denied}` filed {filed}, unwitnessed note {unwitnessed}; \
             notes {:#?}",
            synthesis.notes
        );
        let honest = verdicts(
            &synthesis.suite,
            &Interpreted::for_model(shelf(ranked, second, &[])),
        );
        assert!(
            not_passed(&honest).is_empty(),
            "ranked {ranked}: {honest:#?}"
        );
        let skipping = verdicts(
            &synthesis.suite,
            &Interpreted::for_model(shelf(ranked, second, &[command])),
        );
        assert_eq!(
            skipping
                .iter()
                .find(|(id, _)| *id == denied)
                .map(|(_, status)| *status),
            Some(Status::Failed),
            "ranked {ranked}: {skipping:#?}"
        );
    }
}

/// #712: "the command acts on an existing row (moves or updates)". An update of a field the
/// ranking does not read.
#[test]
fn s1_an_updating_command_beside_a_ranked_view_owes_its_grant_refusal() {
    assert_refusal_owed("library.shelf.RelabelBook", false);
}

/// An update of the very field the view is ranked by.
#[test]
fn s1_an_update_of_the_ranking_field_beside_a_ranked_view_owes_its_grant_refusal() {
    assert_refusal_owed("library.shelf.RepositionBook", false);
}

/// Two ranked views of the entity: each may append its own creating send after the command's.
#[test]
fn s1_a_moving_command_beside_two_ranked_views_owes_its_grant_refusal() {
    assert_refusal_owed("library.shelf.LendBook", true);
}

/// The template's send is the last send of the command under audit, and nothing sent after it in
/// the template survives, for each command on an existing row of the extended shelf.
#[test]
fn s1_every_ranked_denied_scenario_ends_at_its_own_refused_send() {
    let synthesis = ess_conformance::synthesize(&shelf(true, true, &[]));
    for command in [
        "library.shelf.LendBook",
        "library.shelf.RelabelBook",
        "library.shelf.RepositionBook",
    ] {
        let id: ScenarioId = format!("{command}/grant/denied").parse().unwrap();
        let Some(scenario) = synthesis.suite.scenario(&id) else {
            panic!(
                "story:grant-denied-is-synthesized-beside-a-ranked-view: `{id}` not filed: {:#?}",
                synthesis.notes
            );
        };
        let refused = scenario
            .steps
            .iter()
            .position(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. }))
            .expect("the refusal is required");
        let ScenarioStep::ExecuteCommand {
            command: sent,
            actor,
            ..
        } = &scenario.steps[refused - 1]
        else {
            panic!(
                "{id}: the refused send comes just before: {:?}",
                scenario.steps
            );
        };
        assert_eq!(sent.to_string(), command, "{id}");
        assert_eq!(
            actor.as_ref().map(ToString::to_string).as_deref(),
            Some("library.shelf.Reader"),
            "{id}"
        );
        let later_sends = scenario.steps[refused + 1..]
            .iter()
            .filter(|step| {
                matches!(
                    step,
                    ScenarioStep::ExecuteCommand { .. }
                        | ScenarioStep::ExecuteCommandWithoutInput { .. }
                )
            })
            .count();
        assert_eq!(later_sends, 0, "{id}: {:?}", scenario.steps);
    }
}

// ---- story 2: every ungranted actor of the view's domain ---------------------------------------

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

/// The unit's two-domain fixture with `extra_actors` appended to `demo.tasks`'s actors.
fn tasks_model(extra_actors: &str) -> EssIr {
    let tasks = format!(
        "domain: demo.tasks

entities:
  - name: demo.tasks.Task
    identity: {{name: id, type: String}}
    fields:
      - {{name: title, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}

events:
  - name: demo.tasks.TaskOpened
    fields:
      - {{name: id, type: String}}

commands:
  - name: demo.tasks.OpenTask
    input:
      - {{name: id, type: String}}
      - {{name: title, type: String}}
    outcomes:
      - name: opened
        creates: demo.tasks.Task
        instance: id
        sets: {{title: input.title}}
        emits: [demo.tasks.TaskOpened]
        payload:
          demo.tasks.TaskOpened: {{id: input.id}}

views:
  - name: demo.tasks.AllTasks
    source: demo.tasks.Task
    consistency: read_your_writes
    fields:
      - {{name: id, type: String}}
      - {{name: title, type: String}}

actors:
  - name: demo.tasks.Agent
    may: [demo.tasks.OpenTask, demo.tasks.AllTasks]
  - name: demo.tasks.Client
    may: [demo.tasks.OpenTask]
{extra_actors}"
    );
    assemble(&[
        ("system.yaml", TWO_SYSTEM.to_owned()),
        ("components.yaml", TWO_COMPONENTS.to_owned()),
        ("domains/public.yaml", TWO_PUBLIC.to_owned()),
        ("domains/tasks.yaml", tasks),
    ])
}

const GUEST: &str = "  - name: demo.tasks.Guest\n    may: [demo.tasks.OpenTask]\n";
const ALL_TASKS: &str = "demo.tasks.AllTasks";
const ALL_TASKS_DENIED: &str = "demo.tasks.AllTasks/grant/read/denied";

fn readers(suite: &ConformanceSuite) -> Vec<Option<String>> {
    let id: ScenarioId = ALL_TASKS_DENIED.parse().unwrap();
    let scenario = suite.scenario(&id).expect("the denied read is filed");
    let mut reader = None;
    let mut out = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ReadAs { actor } => reader = actor.as_ref().map(ToString::to_string),
            ScenarioStep::QueryView { view, .. } if view.to_string() == ALL_TASKS => {
                out.push(reader.clone());
            }
            _ => {}
        }
    }
    out
}

/// The interpreter, serving `AllTasks` to `leaked` as if it were the granted `Agent`: a surface
/// whose read grant admits one ungranted actor of the view's domain by mistake.
struct Leaks {
    inner: Interpreted,
    leaked: &'static str,
}

impl ConformanceTarget for Leaks {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("read-grant-leak-probe", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        if request.view.to_string() == ALL_TASKS && reader.to_string() == self.leaked {
            let agent = ActorRef::new(QualifiedName::new("demo.tasks.Agent").unwrap());
            return self.inner.query_view_as(request, &agent);
        }
        self.inner.query_view_as(request, reader)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// A target leaking only `demo.tasks.Guest`, the second ungranted actor of the view's domain in
/// name order, fails exactly the denied read: the runner reads on after the first refusal.
#[test]
fn s2_a_target_leaking_only_the_second_ungranted_actor_fails_the_denied_read() {
    let suite = ess_conformance::synthesize(&tasks_model(GUEST)).suite;
    assert_eq!(
        readers(&suite),
        [
            Some("demo.tasks.Client".to_owned()),
            Some("demo.tasks.Guest".to_owned()),
            None
        ]
    );
    let leaking = verdicts(
        &suite,
        &Leaks {
            inner: Interpreted::for_model(tasks_model(GUEST)),
            leaked: "demo.tasks.Guest",
        },
    );
    assert_eq!(
        not_passed(&leaking),
        [ALL_TASKS_DENIED],
        "story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain: {leaking:#?}"
    );
}

/// The Go runtime gives the Rust reference's verdict on the multi-actor denied read, against the
/// same target leaking the second actor.
#[test]
fn s2_go_gives_the_reference_verdict_on_a_multi_actor_denied_read() {
    let suite = ess_conformance::synthesize(&tasks_model(GUEST)).suite;
    let verdicts = support_go::assert_parity(
        "w1-syn1-adversary-leak",
        &suite,
        Leaks {
            inner: Interpreted::for_model(tasks_model(GUEST)),
            leaked: "demo.tasks.Guest",
        },
    );
    assert_eq!(
        verdicts.get(ALL_TASKS_DENIED).map(String::as_str),
        Some("failed"),
        "{verdicts:?}"
    );
}

/// A system actor of the view's domain that the grant does not name is read as too, and the
/// honest interpreter still passes every scenario.
#[test]
fn s2_a_system_actor_of_the_views_domain_keeps_the_suite_passing() {
    let extra = "  - name: demo.tasks.Keeper\n    system: true\n";
    let synthesis = ess_conformance::synthesize(&tasks_model(extra));
    assert_eq!(
        readers(&synthesis.suite),
        [
            Some("demo.tasks.Client".to_owned()),
            Some("demo.tasks.Keeper".to_owned()),
            None
        ]
    );
    let honest = verdicts(
        &synthesis.suite,
        &Interpreted::for_model(tasks_model(extra)),
    );
    assert!(not_passed(&honest).is_empty(), "{honest:#?}");
}

/// An actor of the view's domain carrying attributes, which the grant does not name, is read as
/// too, and the honest interpreter still passes every scenario.
#[test]
fn s2_an_attributed_actor_of_the_views_domain_keeps_the_suite_passing() {
    let extra =
        "  - name: demo.tasks.Member\n    attributes:\n      - {name: member_id, type: String}\n";
    let synthesis = ess_conformance::synthesize(&tasks_model(extra));
    assert_eq!(
        readers(&synthesis.suite),
        [
            Some("demo.tasks.Client".to_owned()),
            Some("demo.tasks.Member".to_owned()),
            None
        ]
    );
    let honest = verdicts(
        &synthesis.suite,
        &Interpreted::for_model(tasks_model(extra)),
    );
    assert!(not_passed(&honest).is_empty(), "{honest:#?}");
}
