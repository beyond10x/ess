//! `error-swap` mutates refusals that carry a payload
//! (story:error-swap-mutates-refusals-that-carry-a-payload, beyond10x/ess#709).
//!
//! An outcome's `payload:` entry keyed by the error it reports says where that error's fields come
//! from. Swapping the error and keeping the entry left it keyed by an error the outcome no longer
//! reports, which the model refuses (`ESS-COMMAND-001`, or `ESS-COMMAND-004` on an
//! `unknown_instance` refusal), so every such refusal had only a stillborn mutant. The entry is now
//! re-keyed to the error swapped in where that error has exactly the same fields, and dropped
//! otherwise; a site where no candidate compiles is listed with a named reason.
#![allow(clippy::missing_panics_doc)]

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, Mutation, MutationReport, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const STORY: &str = "story:error-swap-mutates-refusals-that-carry-a-payload";

/// The toy specification of beyond10x/ess#709, brand-free: two refusals that say what fills their
/// error's fields (`too-far`, and `no-such-book`, an `unknown_instance` refusal) and one that does
/// not (`wrong-state`).
const SHELF: &str = r"format: ess/32
system: library
version: v1
domain: library.shelf
types:
  - {name: library.shelf.BookId, kind: newtype, of: Uuid}
entities:
  - name: library.shelf.Book
    identity: {name: book_id, type: library.shelf.BookId}
    fields:
      - {name: position, type: Integer}
      - {name: note, type: Optional<String>}
    lifecycle:
      initial: Shelved
      states: [Shelved, Lent]
      terminal: [Lent]
      transitions:
        - {name: lend, from: [Shelved], to: Lent}
actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook, library.shelf.LendBook, library.shelf.AnnotateBook]
errors:
  - name: library.shelf.ShelfFull
    fields:
      - {name: requested, type: Integer}
  - name: library.shelf.BookNotFound
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookStateConflict
    fields:
      - {name: state, type: library.shelf.Book.State}
commands:
  - name: library.shelf.CatalogueBook
    input:
      - {name: position, type: Integer}
    outcomes:
      - name: too-far
        when: position > 100
        error: library.shelf.ShelfFull
        payload:
          library.shelf.ShelfFull: {requested: input.position}
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
      - name: no-such-book
        unknown_instance: true
        error: library.shelf.BookNotFound
        payload:
          library.shelf.BookNotFound: {book_id: input.book_id}
      - name: wrong-state
        wrong_state: true
        error: library.shelf.BookStateConflict
  - name: library.shelf.AnnotateBook
    input:
      - {name: book_id, type: library.shelf.BookId}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: annotated
        updates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookAnnotated]
        payload:
          library.shelf.BookAnnotated: {book_id: input.book_id}
        sets: {note: input.note}
events:
  - name: library.shelf.BookCatalogued
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookLent
    fields:
      - {name: book_id, type: library.shelf.BookId}
  - name: library.shelf.BookAnnotated
    fields:
      - {name: book_id, type: library.shelf.BookId}
views:
  - name: library.shelf.BookById
    source: library.shelf.Book
    consistency: read_your_writes
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: position, type: Integer}
components:
  - component: shelf-service
    owns: {domains: [library.shelf]}
    accepts:
      commands: [library.shelf.CatalogueBook, library.shelf.LendBook, library.shelf.AnnotateBook]
    publishes:
      events: [library.shelf.BookCatalogued, library.shelf.BookLent, library.shelf.BookAnnotated]
    reached_by: network
";

const TOO_FAR: &str = "error-swap/library.shelf.CatalogueBook/too-far";
const NO_SUCH_BOOK: &str = "error-swap/library.shelf.LendBook/no-such-book";
const WRONG_STATE: &str = "error-swap/library.shelf.LendBook/wrong-state";

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let label = "shelf.yaml".to_owned();
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn audit(text: &str) -> MutationReport {
    let (documents, texts) = documents(text);
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    mutate::audit(&documents, &texts, &[MutantClass::ErrorSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn entry<'a>(report: &'a MutationReport, id: &str) -> &'a mutate::MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("{STORY}: no `{id}`: {}", report.render_text()))
}

/// Over the toy, both payload-carrying refusals get a live mutant, and the synthesized suite kills
/// each; the audit has no stillborn error-swap mutant left.
#[test]
fn both_payload_carrying_refusals_get_a_killed_mutant() {
    let report = audit(SHELF);
    for id in [TOO_FAR, NO_SUCH_BOOK] {
        assert_eq!(
            entry(&report, id).verdict,
            Verdict::Killed,
            "{STORY}: {}",
            report.render_text()
        );
    }
    assert_eq!(report.counts.stillborn, 0, "{}", report.render_text());
}

/// The payload-less refusal keeps its id, its edit and its verdict.
#[test]
fn a_payload_less_refusal_keeps_its_mutant() {
    let report = audit(SHELF);
    let wrong_state = entry(&report, WRONG_STATE);
    assert_eq!(wrong_state.verdict, Verdict::Killed);
    assert_eq!(
        wrong_state.change,
        "`error: library.shelf.BookStateConflict` becomes `error: library.shelf.BookNotFound`"
    );
}

/// Where the swapped-in error has fields the entry cannot fill the same way, the entry is dropped:
/// the `unknown_instance` refusal's mutant reports another error and declares no effect, and the
/// model admits it (`ESS-COMMAND-004` refuses an effect there).
#[test]
fn an_unknown_instance_refusal_stays_free_of_effects() {
    let (documents, texts) = documents(SHELF);
    let mutant = mutate::mutants(&documents, &[MutantClass::ErrorSwap])
        .into_iter()
        .find(|mutant| mutant.id == NO_SUCH_BOOK)
        .expect("the site has a mutant");
    let mutated = mutate::apply(&documents, &mutant.mutation).expect("the edit applies");
    let outcome = mutated
        .iter()
        .flat_map(|(_, file)| &file.commands)
        .find(|command| command.name.to_string() == "library.shelf.LendBook")
        .and_then(|command| {
            command
                .outcomes
                .iter()
                .find(|outcome| outcome.name.to_string() == "no-such-book")
        })
        .expect("the outcome is there");
    assert!(outcome.unknown_instance);
    assert!(
        outcome.emits.is_empty() && outcome.sets.0.is_empty(),
        "{outcome:#?}"
    );
    assert!(outcome.payload.0.is_empty(), "{STORY}: {outcome:#?}");
    mutate::compile(mutated, &texts).unwrap_or_else(|error| panic!("{STORY}: {error:?}"));
    assert!(
        mutant.change.contains("payload"),
        "the change says what became of the payload: {}",
        mutant.change
    );
}

/// Two errors with exactly the same fields: the entry is re-keyed to the error swapped in, so the
/// mutant still says where that error's fields come from.
#[test]
fn an_error_with_the_same_fields_takes_the_payload_over() {
    let text = SHELF.replace(
        "  - name: library.shelf.ShelfFull\n    fields:\n      - {name: requested, type: Integer}",
        "  - name: library.shelf.ShelfFull\n    fields:\n      - {name: requested, type: Integer}\n  - name: library.shelf.AisleFull\n    fields:\n      - {name: requested, type: Integer}",
    );
    let (documents, texts) = documents(&text);
    let mutant = mutate::mutants(&documents, &[MutantClass::ErrorSwap])
        .into_iter()
        .find(|mutant| mutant.id == TOO_FAR)
        .expect("the site has a mutant");
    assert_eq!(
        mutant.mutation,
        Mutation::ErrorSwap {
            command: "library.shelf.CatalogueBook".to_owned(),
            outcome: "too-far".to_owned(),
            error: "library.shelf.AisleFull".to_owned(),
        }
    );
    let mutated = mutate::apply(&documents, &mutant.mutation).expect("the edit applies");
    let outcome = mutated
        .iter()
        .flat_map(|(_, file)| &file.commands)
        .flat_map(|command| &command.outcomes)
        .find(|outcome| outcome.name.to_string() == "too-far")
        .expect("the outcome is there");
    let keys: Vec<String> = outcome
        .payload
        .0
        .iter()
        .map(|(key, _)| key.to_string())
        .collect();
    assert_eq!(keys, ["library.shelf.AisleFull"], "{STORY}");
    mutate::compile(mutated, &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let report = audit(&text);
    assert_eq!(
        entry(&report, TOO_FAR).verdict,
        Verdict::Killed,
        "{}",
        report.render_text()
    );
}
