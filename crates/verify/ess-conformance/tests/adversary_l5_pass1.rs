//! Adversary pass 1 on <https://github.com/beyond10x/ess/issues/539>: a `precedence-swap` scored
//! `equivalent` because branches answered before both take every input both select.
//!
//! The verdict claims the two branches' declaration order decides nothing a caller observes. Each
//! case here runs both orders through the reference interpreter and holds the audit to what they
//! answer: where the two orders answer one request differently, the swap is not `equivalent`.
#![allow(clippy::missing_panics_doc)]

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::report::Status;
use ess_conformance::target::{ConformanceTarget, ScenarioContext, SemanticCommandRequest};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use serde_json::Value;

/// A row-set command (`refused` reads a row set through the input) whose two accepting branches
/// address different rows: `book-paused` a book through `book`, `shelf-paused` a shelf through
/// `shelf`. `rushed` and `hurried`, input-guarded refusals the plan reads before both, take every
/// input `shelf-paused` selects between them, so they take the pair's whole overlap. Neither
/// takes all of it alone: a branch one earlier refusal wholly takes is refused at compile
/// (`ESS-COMMAND-012`, 0.77.0). `PAIR` is the two, in either order.
const SHELF: &str = r"format: ess/23
system: demo
version: v1
domain: demo.shelf
entities:
  - name: demo.shelf.Library
    identity: {name: library, type: String}
    fields:
      - {name: curator, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.shelf.Book
    identity: {name: book, type: String}
    fields: []
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved]}
  - name: demo.shelf.Shelf
    identity: {name: shelf, type: String}
    fields: []
    lifecycle: {initial: Up, states: [Up], terminal: [Up]}
errors:
  - {name: demo.shelf.Refused, summary: A library of the curator is open., fields: []}
  - {name: demo.shelf.Unknown, summary: No row carries the identity., fields: []}
  - {name: demo.shelf.Rushed, summary: A rush is not taken., fields: []}
  - {name: demo.shelf.Idle, summary: Nothing was asked., fields: []}
events:
  - name: demo.shelf.LibraryOpened
    fields: [{name: library, type: String}]
  - name: demo.shelf.ShelfBuilt
    fields: [{name: shelf, type: String}]
  - name: demo.shelf.BookAdded
    fields: [{name: book, type: String}]
  - name: demo.shelf.BookPaused
    fields: [{name: book, type: String}]
  - name: demo.shelf.ShelfPaused
    fields: [{name: shelf, type: String}]
commands:
  - name: demo.shelf.OpenLibrary
    input:
      - {name: library, type: String}
      - {name: curator, type: String}
    outcomes:
      - name: opened
        creates: demo.shelf.Library
        instance: library
        sets: {curator: input.curator}
        emits: [demo.shelf.LibraryOpened]
        payload:
          demo.shelf.LibraryOpened: {library: input.library}
  - name: demo.shelf.BuildShelf
    input:
      - {name: shelf, type: String}
    outcomes:
      - name: built
        creates: demo.shelf.Shelf
        instance: shelf
        emits: [demo.shelf.ShelfBuilt]
        payload:
          demo.shelf.ShelfBuilt: {shelf: input.shelf}
  - name: demo.shelf.AddBook
    input:
      - {name: book, type: String}
    outcomes:
      - name: added
        creates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookAdded]
        payload:
          demo.shelf.BookAdded: {book: input.book}
  - name: demo.shelf.Act
    input:
      - {name: book, type: String}
      - {name: shelf, type: String}
      - {name: curator, type: String}
      - {name: pause, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - {name: unknown, unknown_instance: true, error: demo.shelf.Unknown}
      - name: refused
        when_related:
          entity: demo.shelf.Library
          where: curator == input.curator
          exists: true
        error: demo.shelf.Refused
      - {name: rushed, when: {all: ['rush == true', 'pause == true']}, error: demo.shelf.Rushed}
      - {name: hurried, when: {all: ['rush == true', 'pause == false']}, error: demo.shelf.Rushed}
PAIR      - {name: idle, error: demo.shelf.Idle}
";

const BOOK_PAUSED: &str = "      - name: book-paused
        when: pause == true
        updates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookPaused]
        payload:
          demo.shelf.BookPaused: {book: input.book}
";

const SHELF_PAUSED: &str = "      - name: shelf-paused
        when: rush == true
        updates: demo.shelf.Shelf
        instance: shelf
        emits: [demo.shelf.ShelfPaused]
        payload:
          demo.shelf.ShelfPaused: {shelf: input.shelf}
";

const SHELF_SWAP: &str = "precedence-swap/demo.shelf.Act/book-paused/shelf-paused";

/// `SHELF` with `book-paused` declared first where `book_first`, else `shelf-paused`: the
/// baseline, and the model the swap writes.
fn shelf(book_first: bool) -> String {
    let pair = if book_first {
        format!("{BOOK_PAUSED}{SHELF_PAUSED}")
    } else {
        format!("{SHELF_PAUSED}{BOOK_PAUSED}")
    };
    SHELF.replace("PAIR", &pair)
}

/// [`shelf`] with `shelf-paused` made reachable: a Boolean input `halt` selects it where no
/// earlier branch takes the request (`pause == false` and `halt == true`), so no refusal covers it
/// and its synthesized scenario is a live one.
fn reachable_shelf(book_first: bool) -> String {
    shelf(book_first)
        .replace(
            "      - {name: rush, type: Boolean}\n",
            "      - {name: rush, type: Boolean}\n      - {name: halt, type: Boolean}\n",
        )
        .replace(
            "when: rush == true\n        updates: demo.shelf.Shelf",
            "when: {any: ['rush == true', {all: ['pause == false', 'halt == true']}]}\n        \
             updates: demo.shelf.Shelf",
        )
}

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let label = "model.yaml".to_owned();
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn target(text: &str) -> Interpreted {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files, &texts).unwrap_or_else(|error| panic!("{error:?}\n{text}"));
    let target = Interpreted::for_model(ir);
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.shelf/authored/adversary-l5".parse().unwrap(),
            CorrelationId::new("adversary-l5").unwrap(),
        ))
        .unwrap();
    target
}

/// The outcome `target` answers, or its error, as text.
fn answer(target: &dyn ConformanceTarget, command: &str, input: &[(&str, Node)]) -> String {
    let request = SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect(),
        correlation: CorrelationId::new("adversary-l5").unwrap(),
    };
    match target.execute_command(request) {
        Ok(result) => result
            .outcome
            .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string()),
        Err(error) => format!("error: {error}"),
    }
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

/// What `Act` answers in the model declaring the pair in `book_first` order, on a store holding
/// shelf `s1` and no book, sent `book: b-missing, shelf: s1, pause: false, rush: false`: a
/// request that selects neither swapped branch.
fn act_on_a_held_shelf(book_first: bool) -> String {
    let target = target(&shelf(book_first));
    assert_eq!(
        answer(&target, "demo.shelf.BuildShelf", &[("shelf", text("s1"))]),
        "built"
    );
    answer(
        &target,
        "demo.shelf.Act",
        &[
            ("book", text("b-missing")),
            ("shelf", text("s1")),
            ("curator", text("nobody")),
            ("pause", Node::Bool(false)),
            ("rush", Node::Bool(false)),
        ],
    )
}

fn mutant<'a>(json: &'a Value, id: &str) -> &'a Value {
    json["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|it| it["id"] == id)
        .unwrap_or_else(|| panic!("no {id} in {json:#}"))
}

/// The two orders of `book-paused` and `shelf-paused` answer a request that selects neither of
/// them differently: on a row-set command the interpreter's `addressed_row` reads the row the
/// first declared accepting branch addresses, so the swap moves which row's absence refuses.
/// `rushed` and `hurried` taking the pair's whole overlap say nothing about that, so the swap is
/// not `equivalent`.
#[test]
fn a_swap_that_moves_the_addressed_row_is_not_scored_equivalent() {
    let baseline = act_on_a_held_shelf(true);
    let swapped = act_on_a_held_shelf(false);
    assert_ne!(
        baseline, swapped,
        "precondition: the two declaration orders answer the same request differently"
    );
    let (json, rendered) = swaps_audited(&shelf(true));
    let swap = mutant(&json, SHELF_SWAP);
    assert_ne!(
        swap["verdict"], "equivalent",
        "the baseline answers `{baseline}` and the swapped model `{swapped}` to one request, yet \
         the swap is scored equivalent: {swap:#}\n{rendered}"
    );
}

/// The two addressed-row cases hold on the reachable model too: its `Act` takes `halt`, and the
/// request that selects neither swapped branch sends `halt: false`.
#[test]
fn a_swap_that_moves_the_addressed_row_is_not_scored_equivalent_on_the_reachable_model() {
    let held_shelf = |book_first: bool| {
        let target = target(&reachable_shelf(book_first));
        assert_eq!(
            answer(&target, "demo.shelf.BuildShelf", &[("shelf", text("s1"))]),
            "built"
        );
        answer(
            &target,
            "demo.shelf.Act",
            &[
                ("book", text("b-missing")),
                ("shelf", text("s1")),
                ("curator", text("nobody")),
                ("pause", Node::Bool(false)),
                ("rush", Node::Bool(false)),
                ("halt", Node::Bool(false)),
            ],
        )
    };
    let (baseline, swapped) = (held_shelf(true), held_shelf(false));
    assert_ne!(baseline, swapped, "the two orders answer the request alike");
    let (json, rendered) = swaps_audited(&reachable_shelf(true));
    let swap = mutant(&json, SHELF_SWAP);
    assert_eq!(swap["verdict"], "killed", "{swap:#}\n{rendered}");
    assert!(swap.get("overlap_taken_by").is_none(), "{swap:#}");
}

/// A row-set command whose accepting branches address different rows (`book-paused` a book,
/// `shelf-paused` a shelf) passes its own synthesized suite against the reference interpreter:
/// the interpreter reads the row of the first declared accepting branch before any branch is
/// selected (`docs/design/filtered-related-reads.md`, "Subject borrowing and precedence"), so the
/// scenario of the later branch arranges that row beside its own. Every scenario passes, and the
/// audit's unmutated suite, which refuses with `ESS-MUTATE-001` when any scenario fails, runs.
#[test]
fn a_row_set_command_whose_branches_address_different_rows_passes_its_own_suite() {
    synthesized_suite_passes_and_audits(&reachable_shelf(true));
}

/// The same with `shelf-paused` declared first: the interpreter reads the shelf first, and the
/// subject-less `refused` scenario, sent a `pause` that selects `book-paused`, arranges the book
/// beside the shelf as well.
#[test]
fn a_row_set_command_whose_branches_address_different_rows_passes_its_own_suite_in_the_other_order()
{
    synthesized_suite_passes_and_audits(&reachable_shelf(false));
}

/// Every scenario of the suite synthesized for `text` passes on the interpreted target, and the
/// precedence-swap audit runs.
fn synthesized_suite_passes_and_audits(text: &str) {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let written = synthesis
                .suite
                .scenarios
                .iter()
                .find(|(id, _)| **id == result.scenario)
                .map(|(_, scenario)| serde_json::to_string_pretty(scenario).unwrap());
            format!(
                "{}: {:?} {:#?}\n{}",
                result.scenario,
                result.status,
                result.diagnostics().collect::<Vec<_>>(),
                written.unwrap_or_default()
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes on the interpreted target:\n{}",
        failed.join("\n")
    );
    let report = mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}\n{text}"));
    let rendered = report.render_text();
    assert!(!rendered.contains("ESS-MUTATE-001"), "{rendered}");
}

/// The scenario of `Act`'s branch `branch` in the suite synthesized for `text`, run with the whole
/// suite on the interpreted target: panics where synthesis wrote no such scenario, and returns
/// `Err` naming every scenario that did not pass where this one did not.
fn act_scenario_passes(text: &str, branch: &str) -> Result<(), String> {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files, &texts).unwrap_or_else(|error| panic!("{error:?}\n{text}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let wanted = format!("Act/outcome/{branch}");
    let written: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        written.iter().any(|id| id.ends_with(&wanted)),
        "synthesis wrote no `{wanted}` scenario: {written:#?}\nrefused:\n{}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    let failed: Vec<(String, String)> = report
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let scenario = synthesis
                .suite
                .scenarios
                .iter()
                .find(|(id, _)| **id == result.scenario)
                .map(|(_, scenario)| serde_json::to_string_pretty(scenario).unwrap());
            (
                result.scenario.to_string(),
                format!(
                    "{}: {:?} {:#?}\n{}",
                    result.scenario,
                    result.status,
                    result.diagnostics().collect::<Vec<_>>(),
                    scenario.unwrap_or_default()
                ),
            )
        })
        .collect();
    if failed.iter().any(|(id, _)| id.ends_with(&wanted)) {
        return Err(failed
            .into_iter()
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n"));
    }
    Ok(())
}

/// BLOCKER (adversary pass 1 on the different-rows unit). Two accepting branches of a row-set
/// command address the same entity through different input fields: `book-paused` a book through
/// `book`, `shelf-paused` a book through `shelf`. Each row-set scenario arranges its own book and,
/// through `beside_addressed_row`, a second book for the other field, both at the plain
/// distinction: one identity is created twice, and synthesis refuses `book-paused`, `refused`,
/// `shelf-paused` and `idle` (`ESS-SYNTH-001`, "creates a second row under a value a row it created
/// still holds"). One row sent through both fields is what the interpreter reads.
#[test]
fn branches_addressing_one_entity_through_two_fields_pass_their_own_suite() {
    let text = reachable_shelf(true).replace(
        "updates: demo.shelf.Shelf\n        instance: shelf",
        "updates: demo.shelf.Book\n        instance: shelf",
    );
    assert!(text.contains("updates: demo.shelf.Book\n        instance: shelf"));
    if let Err(failed) = act_scenario_passes(&text, "shelf-paused") {
        panic!("the scenario of `shelf-paused` fails its own interpreter:\n{failed}");
    }
}

/// A row-set command whose two accepting branches each `moves:` a row of its own entity, the
/// first declared (`book-lost`) from a state that is not the lifecycle's initial one (`Out`).
/// `HALT` stands for the guard of `shelf-dropped`.
const MOVING: &str = r"format: ess/23
system: demo
version: v1
domain: demo.shelf
entities:
  - name: demo.shelf.Library
    identity: {name: library, type: String}
    fields:
      - {name: curator, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.shelf.Book
    identity: {name: book, type: String}
    fields: []
    lifecycle:
      initial: Shelved
      states: [Shelved, Out, Lost]
      terminal: [Lost]
      transitions:
        - {name: lend, from: [Shelved], to: Out}
        - {name: lose, from: [Out], to: Lost}
  - name: demo.shelf.Shelf
    identity: {name: shelf, type: String}
    fields: []
    lifecycle:
      initial: Up
      states: [Up, Down]
      terminal: [Down]
      transitions:
        - {name: drop, from: [Up], to: Down}
errors:
  - {name: demo.shelf.Refused, summary: A library of the curator is open., fields: []}
  - {name: demo.shelf.Unknown, summary: No row carries the identity., fields: []}
  - {name: demo.shelf.Idle, summary: Nothing was asked., fields: []}
events:
  - name: demo.shelf.LibraryOpened
    fields: [{name: library, type: String}]
  - name: demo.shelf.ShelfBuilt
    fields: [{name: shelf, type: String}]
  - name: demo.shelf.BookAdded
    fields: [{name: book, type: String}]
  - name: demo.shelf.BookLent
    fields: [{name: book, type: String}]
  - name: demo.shelf.BookLost
    fields: [{name: book, type: String}]
  - name: demo.shelf.ShelfDropped
    fields: [{name: shelf, type: String}]
commands:
  - name: demo.shelf.OpenLibrary
    input:
      - {name: library, type: String}
      - {name: curator, type: String}
    outcomes:
      - name: opened
        creates: demo.shelf.Library
        instance: library
        sets: {curator: input.curator}
        emits: [demo.shelf.LibraryOpened]
        payload:
          demo.shelf.LibraryOpened: {library: input.library}
  - name: demo.shelf.BuildShelf
    input:
      - {name: shelf, type: String}
    outcomes:
      - name: built
        creates: demo.shelf.Shelf
        instance: shelf
        emits: [demo.shelf.ShelfBuilt]
        payload:
          demo.shelf.ShelfBuilt: {shelf: input.shelf}
  - name: demo.shelf.AddBook
    input:
      - {name: book, type: String}
    outcomes:
      - name: added
        creates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookAdded]
        payload:
          demo.shelf.BookAdded: {book: input.book}
  - name: demo.shelf.LendBook
    input:
      - {name: book, type: String}
    outcomes:
      - name: lent
        moves: demo.shelf.Book.lend
        instance: book
        emits: [demo.shelf.BookLent]
        payload:
          demo.shelf.BookLent: {book: input.book}
  - name: demo.shelf.Act
    input:
      - {name: book, type: String}
      - {name: shelf, type: String}
      - {name: curator, type: String}
      - {name: pause, type: Boolean}
      - {name: halt, type: Boolean}
    outcomes:
      - {name: unknown, unknown_instance: true, error: demo.shelf.Unknown}
      - name: refused
        when_related:
          entity: demo.shelf.Library
          where: curator == input.curator
          exists: true
        error: demo.shelf.Refused
      - name: book-lost
        when: pause == true
        moves: demo.shelf.Book.lose
        instance: book
        emits: [demo.shelf.BookLost]
        payload:
          demo.shelf.BookLost: {book: input.book}
      - name: shelf-dropped
        when: halt == true
        moves: demo.shelf.Shelf.drop
        instance: shelf
        emits: [demo.shelf.ShelfDropped]
        payload:
          demo.shelf.ShelfDropped: {shelf: input.shelf}
      - {name: idle, error: demo.shelf.Idle}
";

/// BLOCKER (adversary pass 1 on the different-rows unit). The interpreter reads the first declared
/// accepting branch's row (the book) before selecting a branch, and where every acting branch
/// `moves:` it answers the held-state refusal unless some transition moves from the state that row
/// holds. `beside_addressed_row` arranges the book in the lifecycle's initial state (`Shelved`),
/// which `book-lost` (`from: [Out]`) does not move from, so the scenario of `shelf-dropped` is
/// still answered before its branch: the row arranged beside must rest in a state the first
/// declared branch acts from.
#[test]
fn a_moving_branch_beside_a_first_branch_moving_from_a_later_state_passes_its_own_suite() {
    if let Err(failed) = act_scenario_passes(MOVING, "shelf-dropped") {
        panic!("the scenario of `shelf-dropped` fails its own interpreter:\n{failed}");
    }
}

/// BLOCKER (adversary pass 1 on the different-rows unit). The row set of `refused` reads shelves
/// (`exists: false` over `label == input.shelf`: no shelf carries the label the input names), and
/// `shelf-paused` updates a shelf through `shelf`. The witness of `refused` sends a shelf no row
/// carries; `beside_addressed_row` arranges a shelf for `shelf-paused`'s field and binds `shelf`
/// to it, so the row the refusal says is absent now exists, and synthesis refuses `refused` ("its
/// own row set is False"); `book-paused`, `shelf-paused` and `idle` are refused for a second shelf
/// under one identity. The interpreter reads only the book before `refused`, so no shelf is owed.
#[test]
fn a_row_set_refusal_over_the_other_branchs_entity_keeps_its_witness() {
    let text = reachable_shelf(true)
        .replace(
            "    identity: {name: shelf, type: String}\n    fields: []\n",
            "    identity: {name: shelf, type: String}\n    fields:\n      - {name: label, type: \
             String}\n",
        )
        .replace(
            "        creates: demo.shelf.Shelf\n        instance: shelf\n",
            "        creates: demo.shelf.Shelf\n        instance: shelf\n        sets: {label: \
             input.shelf}\n",
        )
        .replace(
            "          entity: demo.shelf.Library\n          where: curator == input.curator\n          \
             exists: true\n",
            "          entity: demo.shelf.Shelf\n          where: label == input.shelf\n          \
             exists: false\n",
        );
    assert!(text.contains("where: label == input.shelf"), "{text}");
    assert!(text.contains("sets: {label: input.shelf}"), "{text}");
    if let Err(failed) = act_scenario_passes(&text, "refused") {
        panic!("the scenario of `refused` fails its own interpreter:\n{failed}");
    }
}

/// The audit of `text`'s precedence swaps against its own interpreter, as JSON, and its text.
fn swaps_audited(text: &str) -> (Value, String) {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let report = mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}\n{text}"));
    let rendered = report.render_text();
    (
        serde_json::from_str(&report.to_canonical_json()).unwrap(),
        rendered,
    )
}

/// A swap of branches addressing different rows is killed by its suite and claims nothing about
/// its overlap: `rushed` and `hurried` take every input both select, but the order still decides
/// which row a request neither of them takes reads, so the mutant carries no `overlap_taken_by`
/// and its text line names no branch taking it. The killed line that does name one
/// (`render_text`'s branch for a verdict other than `equivalent`) is held in
/// `mutation_identical_answer`, on two branches addressing the same row.
#[test]
fn a_killed_swap_of_branches_addressing_different_rows_names_no_overlap_taken() {
    let (json, rendered) = swaps_audited(&shelf(true));
    let swap = mutant(&json, SHELF_SWAP);
    assert_eq!(swap["verdict"], "killed", "{swap:#}\n{rendered}");
    assert!(swap.get("overlap_taken_by").is_none(), "{swap:#}");
    assert_ne!(json["format"], "ess-mutation-report/6", "{rendered}");
    let line = rendered
        .lines()
        .find(|line| line.starts_with(&format!("killed {SHELF_SWAP}")))
        .unwrap_or_else(|| panic!("no killed line for the swap:\n{rendered}"));
    assert!(
        !line.contains("taken first by") && !line.contains("ESS-MUTATE-005"),
        "{line}"
    );
}

/// A command whose branches each create a row, so no row has to be arranged: `paused`
/// (`pause == true`) and `defaulted` (`code` absent or `""`) are the swapped pair, `EARLIER` the
/// branches declared before both, and `plain` the default.
const DESK: &str = r"format: ess/23
system: demo
version: v1
domain: demo.desk
entities:
  - name: demo.desk.Ticket
    identity: {name: ticket, type: Uuid}
    fields: []
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.desk.Early, summary: Refused early., fields: []}
events:
  - name: demo.desk.Early1
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Early2
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Paused
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Defaulted
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Plain
    fields: [{name: ticket, type: Uuid}]
commands:
  - name: demo.desk.File
    input:
      - {name: pause, type: Boolean}
      - {name: code, type: Optional<String>}
    outcomes:
EARLIER      - name: PAIR_ONE
PAIR_ONE_BODY      - name: PAIR_TWO
PAIR_TWO_BODY      - name: plain
        creates: demo.desk.Ticket
        instance: ticket
        emits: [demo.desk.Plain]
        payload:
          demo.desk.Plain: {ticket: {generated: true}}
";

const PAUSED_BODY: &str = "        when: pause == true
        creates: demo.desk.Ticket
        instance: ticket
        emits: [demo.desk.Paused]
        payload:
          demo.desk.Paused: {ticket: {generated: true}}
";

const DEFAULTED_BODY: &str = "        when: {any: [{code: {defined: false}}, 'code == \"\"']}
        creates: demo.desk.Ticket
        instance: ticket
        emits: [demo.desk.Defaulted]
        payload:
          demo.desk.Defaulted: {ticket: {generated: true}}
";

/// An accepting branch `name` declared before the pair, guarded by `when`, emitting `event`.
fn accepting(name: &str, when: &str, event: &str) -> String {
    format!(
        "      - name: {name}\n        when: {when}\n        creates: demo.desk.Ticket\n        \
         instance: ticket\n        emits: [demo.desk.{event}]\n        payload:\n          \
         demo.desk.{event}: {{ticket: {{generated: true}}}}\n"
    )
}

/// An input-guarded refusal `name` declared before the pair, guarded by `when`.
fn refusing(name: &str, when: &str) -> String {
    format!("      - {{name: {name}, when: {when}, error: demo.desk.Early}}\n")
}

/// `DESK` with `earlier` before the pair, `paused` first where `paused_first`.
fn desk(earlier: &str, paused_first: bool) -> String {
    let (one, one_body, two, two_body) = if paused_first {
        ("paused", PAUSED_BODY, "defaulted", DEFAULTED_BODY)
    } else {
        ("defaulted", DEFAULTED_BODY, "paused", PAUSED_BODY)
    };
    DESK.replace("EARLIER", earlier)
        .replace("PAIR_ONE_BODY", one_body)
        .replace("PAIR_TWO_BODY", two_body)
        .replace("PAIR_ONE", one)
        .replace("PAIR_TWO", two)
}

/// What `File` answers in `text` for `pause` and `code` (absent where `None`).
fn filed(text: &str, pause: bool, code: Option<&str>) -> String {
    let target = target(text);
    let mut input = vec![("pause", Node::Bool(pause))];
    if let Some(code) = code {
        input.push(("code", self::text(code)));
    }
    answer(&target, "demo.desk.File", &input)
}

/// Property: wherever the audit names branches taking a swap's overlap or scores it
/// `equivalent`, the two declaration orders answer every input alike. `pause` takes both values
/// and `code` is absent, `""` or another text — every class of input any guard here tells apart.
#[test]
fn a_swap_scored_equivalent_answers_every_input_alike_in_both_orders() {
    let earlier = [
        accepting(
            "early",
            "{all: ['pause == true', {any: [{code: {defined: false}}, 'code == \"\"']}]}",
            "Early1",
        ),
        accepting("early", "pause == true", "Early1"),
        accepting("early", "pause != false", "Early1"),
        format!(
            "{}{}",
            accepting(
                "early-absent",
                "{all: ['pause == true', {code: {defined: false}}]}",
                "Early1"
            ),
            accepting(
                "early-empty",
                "{all: ['pause == true', 'code == \"\"']}",
                "Early2"
            )
        ),
        accepting(
            "early",
            "{all: ['pause == true', 'code != \"x\"']}",
            "Early1",
        ),
        accepting(
            "early",
            "{all: ['pause == true', {code: {defined: false}}]}",
            "Early1",
        ),
        accepting("early", "{not: {all: ['pause == false']}}", "Early1"),
        refusing(
            "early",
            "{all: ['pause == true', {any: [{code: {defined: false}}, 'code == \"\"']}]}",
        ),
        format!(
            "{}{}",
            refusing("early-empty", "{all: ['pause == true', 'code == \"\"']}"),
            accepting(
                "early-absent",
                "{all: ['pause == true', {code: {defined: false}}]}",
                "Early1"
            )
        ),
    ];
    let swap = "precedence-swap/demo.desk.File/paused/defaulted";
    let mut claimed = 0;
    for earlier in &earlier {
        let baseline = desk(earlier, true);
        let swapped = desk(earlier, false);
        let (json, rendered) = swaps_audited(&baseline);
        let entry = mutant(&json, swap);
        if entry.get("overlap_taken_by").is_none() && entry["verdict"] != "equivalent" {
            continue;
        }
        claimed += 1;
        for pause in [false, true] {
            for code in [None, Some(""), Some("x")] {
                assert_eq!(
                    filed(&baseline, pause, code),
                    filed(&swapped, pause, code),
                    "pause {pause}, code {code:?}: the orders answer differently, yet {entry:#}\n\
                     {rendered}\n{baseline}"
                );
            }
        }
    }
    assert!(
        claimed >= 3,
        "the property was asked of {claimed} models only"
    );
}

/// A `precedence-swap` the manifest lists as stillborn — no suite, a `stillborn` cause — while it
/// carries `overlap_taken_by` is refused as incoherent: a stillborn mutant was never decided, and
/// `to_entry` would copy the member into a `stillborn` report entry. The same entry without it is
/// read, so the refusal is the member's.
#[test]
fn a_stillborn_swap_carrying_overlap_taken_by_is_refused() {
    let (files, texts) = documents(&desk(&accepting("early", "pause == true", "Early1"), true));
    let emission = mutate::emit_for(&files, &texts, &[MutantClass::PrecedenceSwap], None)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/6");
    let mut manifest: Value =
        serde_json::from_str(&emission.files[mutate::MANIFEST_FILE]).expect("the manifest");
    let swap = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it["id"] == "precedence-swap/demo.desk.File/paused/defaulted")
        .expect("the swap");
    assert_eq!(
        swap["overlap_taken_by"],
        serde_json::json!(["early"]),
        "{swap:#}"
    );
    let object = swap.as_object_mut().unwrap();
    for key in [
        "dir",
        "refusals",
        "refused",
        "scenarios",
        "spec_digest",
        "suite_digest",
    ] {
        object.remove(key);
    }
    object.insert(
        "stillborn".to_owned(),
        serde_json::json!({"cause": "ESS-COMMAND-005", "code": "ESS-MUTATE-002", "message": "refused"}),
    );
    let refusal = mutate::Manifest::from_json(&manifest.to_string())
        .expect_err("a stillborn mutant carries no `overlap_taken_by`");
    assert!(refusal.contains("neither a stillborn mutant"), "{refusal}");
    let swap = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it["id"] == "precedence-swap/demo.desk.File/paused/defaulted")
        .unwrap();
    swap.as_object_mut().unwrap().remove("overlap_taken_by");
    mutate::Manifest::from_json(&manifest.to_string())
        .unwrap_or_else(|refusal| panic!("the same stillborn entry without it is read: {refusal}"));
}

/// A row-set command `Act` over three entities a branch may act on (a book, a shelf carrying a
/// `label`, a cart). `LABEL` is what `BuildShelf` sets the label to, `OUTCOMES` every branch of
/// `Act` after `unknown`.
const ROWS: &str = r"format: ess/23
system: demo
version: v1
domain: demo.shelf
entities:
  - name: demo.shelf.Library
    identity: {name: library, type: String}
    fields:
      - {name: curator, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.shelf.Book
    identity: {name: book, type: String}
    fields: []
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved]}
  - name: demo.shelf.Shelf
    identity: {name: shelf, type: String}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Up, states: [Up], terminal: [Up]}
  - name: demo.shelf.Cart
    identity: {name: cart, type: String}
    fields: []
    lifecycle: {initial: Parked, states: [Parked], terminal: [Parked]}
errors:
  - {name: demo.shelf.Refused, summary: The row set decides., fields: []}
  - {name: demo.shelf.Unknown, summary: No row carries the identity., fields: []}
  - {name: demo.shelf.Rushed, summary: A rush is not taken., fields: []}
  - {name: demo.shelf.Idle, summary: Nothing was asked., fields: []}
events:
  - name: demo.shelf.LibraryOpened
    fields: [{name: library, type: String}]
  - name: demo.shelf.ShelfBuilt
    fields: [{name: shelf, type: String}]
  - name: demo.shelf.BookAdded
    fields: [{name: book, type: String}]
  - name: demo.shelf.CartParked
    fields: [{name: cart, type: String}]
  - name: demo.shelf.BookPaused
    fields: [{name: book, type: String}]
  - name: demo.shelf.ShelfPaused
    fields: [{name: shelf, type: String}]
  - name: demo.shelf.CartPaused
    fields: [{name: cart, type: String}]
commands:
  - name: demo.shelf.OpenLibrary
    input:
      - {name: library, type: String}
      - {name: curator, type: String}
    outcomes:
      - name: opened
        creates: demo.shelf.Library
        instance: library
        sets: {curator: input.curator}
        emits: [demo.shelf.LibraryOpened]
        payload:
          demo.shelf.LibraryOpened: {library: input.library}
  - name: demo.shelf.BuildShelf
    input:
      - {name: shelf, type: String}
      - {name: label, type: String}
    outcomes:
      - name: built
        creates: demo.shelf.Shelf
        instance: shelf
        sets: {label: LABEL}
        emits: [demo.shelf.ShelfBuilt]
        payload:
          demo.shelf.ShelfBuilt: {shelf: input.shelf}
  - name: demo.shelf.AddBook
    input:
      - {name: book, type: String}
    outcomes:
      - name: added
        creates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookAdded]
        payload:
          demo.shelf.BookAdded: {book: input.book}
  - name: demo.shelf.ParkCart
    input:
      - {name: cart, type: String}
    outcomes:
      - name: parked
        creates: demo.shelf.Cart
        instance: cart
        emits: [demo.shelf.CartParked]
        payload:
          demo.shelf.CartParked: {cart: input.cart}
  - name: demo.shelf.Act
    input:
      - {name: book, type: String}
      - {name: shelf, type: String}
      - {name: cart, type: String}
      - {name: curator, type: String}
      - {name: pause, type: Boolean}
      - {name: halt, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - {name: unknown, unknown_instance: true, error: demo.shelf.Unknown}
OUTCOMES";

/// The row set over libraries `SHELF` reads: `refused` where a library of the curator exists.
const LIBRARY_SET: &str = "      - name: refused
        when_related:
          entity: demo.shelf.Library
          where: curator == input.curator
          exists: true
        error: demo.shelf.Refused
";

/// A row set over shelves: `refused` where a shelf labelled with the curator exists.
const SHELF_SET_PRESENT: &str = "      - name: refused
        when_related:
          entity: demo.shelf.Shelf
          where: label == input.curator
          exists: true
        error: demo.shelf.Refused
";

/// A row set over shelves: `refused` where no shelf carries the label `shelf` names (pass 1's
/// finding 3).
const SHELF_SET_ABSENT: &str = "      - name: refused
        when_related:
          entity: demo.shelf.Shelf
          where: label == input.shelf
          exists: false
        error: demo.shelf.Refused
";

const IDLE: &str = "      - {name: idle, error: demo.shelf.Idle}\n";
const RUSHED: &str = "      - {name: rushed, when: rush == true, error: demo.shelf.Rushed}\n";

/// What an acting branch updates: entity, input field, event.
type Target = (&'static str, &'static str, &'static str);
const BOOK: Target = ("Book", "book", "BookPaused");
const SHELF_ROW: Target = ("Shelf", "shelf", "ShelfPaused");
const CART: Target = ("Cart", "cart", "CartPaused");

/// An accepting branch `name` updating `target` through its field, guarded by `when` (the default
/// where `None`).
fn acting(name: &str, when: Option<&str>, (entity, field, event): Target) -> String {
    let guard = when.map_or_else(String::new, |when| format!("        when: {when}\n"));
    format!(
        "      - name: {name}\n{guard}        updates: demo.shelf.{entity}\n        instance: \
         {field}\n        emits: [demo.shelf.{event}]\n        payload:\n          \
         demo.shelf.{event}: {{{field}: input.{field}}}\n"
    )
}

/// `ROWS` with `BuildShelf` setting the label to `label` and `outcomes` after `unknown`.
fn rows(label: &str, outcomes: &str) -> String {
    ROWS.replace("LABEL", label).replace("OUTCOMES", outcomes)
}

/// `None` where synthesis writes a scenario for each of `branches` of `Act` and every scenario of
/// the suite passes on the interpreted target; else what went wrong.
fn act_suite_fails(text: &str, branches: &[&str]) -> Option<String> {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files, &texts).unwrap_or_else(|error| panic!("{error:?}\n{text}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let written: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let mut problems: Vec<String> = branches
        .iter()
        .map(|branch| format!("Act/outcome/{branch}"))
        .filter(|wanted| !written.iter().any(|id| id.ends_with(wanted)))
        .map(|wanted| format!("synthesis wrote no `{wanted}` scenario"))
        .collect();
    if !problems.is_empty() {
        problems.push(format!(
            "refused:\n{}",
            synthesis
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    for result in report
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
    {
        let scenario = synthesis
            .suite
            .scenarios
            .iter()
            .find(|(id, _)| **id == result.scenario)
            .map(|(_, scenario)| serde_json::to_string_pretty(scenario).unwrap());
        problems.push(format!(
            "{}: {:?} {:#?}\n{}",
            result.scenario,
            result.status,
            result.diagnostics().collect::<Vec<_>>(),
            scenario.unwrap_or_default()
        ));
    }
    (!problems.is_empty()).then(|| problems.join("\n"))
}

/// BLOCKER (adversary pass 2 on the different-rows unit). `book-paused` updates a book through
/// `book`; the default branch `shelf-touched` (no `when:`) updates a shelf through `shelf`. The
/// subject-less `refused` scenario sends a `pause` that either selects `book-paused` or leaves
/// the request to the default, in one of the two polarities. The interpreter answers the
/// selected branch's row before the row set (`subject_refusals_before_present_related`), the
/// default included; `beside_addressed_row` picks the selected branch only among branches whose
/// `when:` holds ("else the default" in its comment is not in the code), so no shelf is arranged
/// and `refused` is answered `unknown`.
#[test]
fn a_row_set_refusal_whose_send_selects_a_default_acting_on_another_row_passes_its_own_suite() {
    let mut failed = Vec::new();
    for pause in ["true", "false"] {
        let guard = format!("pause == {pause}");
        let text = rows(
            "input.label",
            &[
                LIBRARY_SET.to_owned(),
                acting("book-paused", Some(&guard), BOOK),
                acting("shelf-touched", None, SHELF_ROW),
            ]
            .concat(),
        );
        if let Some(why) = act_suite_fails(&text, &["refused", "book-paused", "shelf-touched"]) {
            failed.push(format!("book-paused when {guard}:\n{why}"));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
}

/// BLOCKER (adversary pass 2 on the different-rows unit). The row set of `refused` reads shelves
/// (`label == input.curator`), and `shelf-paused` updates a shelf through `shelf`. With
/// `shelf-paused` declared first, the scenario of `book-paused` holds two shelves: the shelf the
/// row-set arrangement arranges beside its row set, and the first declared branch's row
/// `beside_addressed_row` arranges under `Distinction::further(1)`, a number the row-set
/// arrangement numbers its own rows from too, so one identity is created twice and synthesis
/// refuses `book-paused` (`ESS-SYNTH-001`). The last polarity is the one where the `refused` send
/// (every Boolean `true`) selects `shelf-paused`, the same two shelves through the selected
/// branch. Every order and polarity passes its own suite.
#[test]
fn a_row_set_over_the_selected_branchs_entity_keeps_its_rows_apart() {
    let mut failed = Vec::new();
    for (book, shelf) in [
        (
            "pause == true",
            "{all: ['pause == false', 'halt == false']}",
        ),
        (
            "pause == false",
            "{all: ['pause == true', 'halt == false']}",
        ),
        ("pause == false", "{all: ['pause == true', 'halt == true']}"),
    ] {
        for book_first in [true, false] {
            let pair = if book_first {
                [
                    acting("book-paused", Some(book), BOOK),
                    acting("shelf-paused", Some(shelf), SHELF_ROW),
                ]
            } else {
                [
                    acting("shelf-paused", Some(shelf), SHELF_ROW),
                    acting("book-paused", Some(book), BOOK),
                ]
            };
            let text = rows(
                "input.label",
                &[SHELF_SET_PRESENT.to_owned(), pair.concat(), IDLE.to_owned()].concat(),
            );
            if let Some(why) = act_suite_fails(&text, &["refused", "book-paused", "shelf-paused"]) {
                failed.push(format!(
                    "book `{book}`, shelf `{shelf}`, book first {book_first}:\n{why}"
                ));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
}

/// BLOCKER (adversary pass 2 on the different-rows unit). Pass 1's finding 3 through the selected
/// branch: `refused` holds where no shelf carries the label `shelf` names, and a shelf's label is
/// its identity, so `refused` is reachable only by a send selecting `book-paused` (a shelf the
/// send names exists and carries that label). Synthesis has to send such a `pause`; where its
/// witness selects `shelf-paused` instead (the last polarity, for a send whose Booleans are all
/// `true`), the shelf `beside_addressed_row` arranges fills the row set `refused` needs empty.
#[test]
fn an_exists_false_refusal_whose_send_selects_the_shelf_branch_keeps_its_witness() {
    let mut failed = Vec::new();
    for (book, shelf) in [
        (
            "pause == true",
            "{all: ['pause == false', 'halt == false']}",
        ),
        (
            "pause == false",
            "{all: ['pause == true', 'halt == false']}",
        ),
        ("pause == false", "{all: ['pause == true', 'halt == true']}"),
    ] {
        let text = rows(
            "input.shelf",
            &[
                SHELF_SET_ABSENT.to_owned(),
                acting("book-paused", Some(book), BOOK),
                acting("shelf-paused", Some(shelf), SHELF_ROW),
                IDLE.to_owned(),
            ]
            .concat(),
        );
        if let Some(why) = act_suite_fails(&text, &["refused"]) {
            failed.push(format!("book `{book}`, shelf `{shelf}`:\n{why}"));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
}

/// Three accepting branches addressing three rows, under mutually exclusive `when:` guards, in
/// every declaration order, with an input-guarded refusal declared between the first two: each
/// passes its own suite.
#[test]
fn three_branches_addressing_three_rows_pass_their_own_suite_in_every_order() {
    let branches = [
        acting(
            "book-paused",
            Some("{all: ['pause == true', 'halt == false']}"),
            BOOK,
        ),
        acting(
            "shelf-paused",
            Some("{all: ['pause == false', 'halt == true']}"),
            SHELF_ROW,
        ),
        acting(
            "cart-paused",
            Some("{all: ['pause == true', 'halt == true']}"),
            CART,
        ),
    ];
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut failed = Vec::new();
    for order in orders {
        let text = rows(
            "input.label",
            &[
                LIBRARY_SET,
                branches[order[0]].as_str(),
                RUSHED,
                branches[order[1]].as_str(),
                branches[order[2]].as_str(),
                IDLE,
            ]
            .concat(),
        );
        if let Some(why) = act_suite_fails(
            &text,
            &["refused", "book-paused", "shelf-paused", "cart-paused"],
        ) {
            failed.push(format!("order {order:?}:\n{why}"));
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
}
