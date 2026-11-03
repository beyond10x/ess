//! The mutation audit covers actor grants (story:the-mutation-audit-covers-actor-grants,
//! beyond10x/ess#710).
//!
//! `grant-drop` removes one command from one actor's `may:`, `grant-add` gives an actor one command
//! a served component accepts that its `may:` lacks. Where either class is selected the baseline
//! keeps the `grant/denied` scenarios synthesis writes, so a target that admits the dropped actor,
//! or refuses the added one, fails the mutant's suite. A run whose classes exclude both keeps the
//! baseline and mutant ids it had.
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, MutantClass, MutationReport, Verdict, BASELINE_DIR, MANIFEST_FILE, REPORT_FILE,
    SUITE_FILE,
};
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, CountReport, Runner};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

const STORY: &str = "story:the-mutation-audit-covers-actor-grants";

/// The toy specification of beyond10x/ess#710, brand-free: one served command a `Librarian` may
/// invoke and a declared `Reader` may not. Kept here rather than under `fixtures/`, so the
/// repository-wide synthesis byte tables, which walk every `.yaml` file, are not moved by it.
const GRANTS: &str = r"format: ess/32
system: library
version: v1
domain: library.shelf
types:
  - {name: library.shelf.BookId, kind: newtype, of: Uuid}
entities:
  - name: library.shelf.Book
    identity: {name: book_id, type: library.shelf.BookId}
    fields:
      - {name: shelf_code, type: String}
      - {name: position, type: Integer}
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved], transitions: []}
actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Reader
commands:
  - name: library.shelf.CatalogueBook
    input:
      - {name: shelf_code, type: String}
      - {name: position, type: Integer}
    outcomes:
      - name: catalogued
        creates: library.shelf.Book
        instance: book_id
        emits: [library.shelf.BookCatalogued]
        payload:
          library.shelf.BookCatalogued: {book_id: {generated: true}}
        sets: {shelf_code: input.shelf_code, position: input.position}
events:
  - name: library.shelf.BookCatalogued
    fields:
      - {name: book_id, type: library.shelf.BookId}
views:
  - name: library.shelf.ShelfOrder
    source: library.shelf.Book
    consistency: read_your_writes
    order_by: [shelf_code asc, position asc]
    fields:
      - {name: book_id, type: library.shelf.BookId}
      - {name: shelf_code, type: String}
      - {name: position, type: Integer}
components:
  - component: shelf-service
    owns: {domains: [library.shelf]}
    accepts: {commands: [library.shelf.CatalogueBook]}
    publishes: {events: [library.shelf.BookCatalogued]}
    reached_by: network
";

/// Two actors keep lacking the command after `Librarian` loses it, and `Archivist` is named before
/// `Librarian`: the refusal synthesis would send as the lowest-named lacking actor is `Archivist`'s,
/// which a correct target refuses too, so the mutant is killed only by sending it as the actor that
/// lost the grant.
const DROP_NOT_LOWEST: &str = r"format: ess/32
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
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved], transitions: []}
actors:
  - name: library.shelf.Archivist
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Manager
    may: [library.shelf.CatalogueBook]
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
events:
  - name: library.shelf.BookCatalogued
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
    accepts: {commands: [library.shelf.CatalogueBook]}
    publishes: {events: [library.shelf.BookCatalogued]}
    reached_by: network
";

const DROP: &str = "grant-drop/library.shelf.Librarian/library.shelf.CatalogueBook";
const ADD: &str = "grant-add/library.shelf.Reader/library.shelf.CatalogueBook";
const DENIED: &str = "library.shelf.CatalogueBook/grant/denied";

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let label = "shelf.yaml".to_owned();
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn audit(text: &str, classes: &[MutantClass]) -> MutationReport {
    let (documents, texts) = documents(text);
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    mutate::audit(&documents, &texts, classes, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn verdict(report: &MutationReport, id: &str) -> Verdict {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("{STORY}: no `{id}`: {}", report.render_text()))
        .verdict
}

/// The two classes exist, are written as their names, and follow every older class.
#[test]
fn the_grant_classes_exist() {
    assert_eq!(MutantClass::GrantDrop.as_str(), "grant-drop");
    assert_eq!(MutantClass::GrantAdd.as_str(), "grant-add");
    assert_eq!(
        &MutantClass::ALL[MutantClass::ALL.len() - 2..],
        &[MutantClass::GrantDrop, MutantClass::GrantAdd],
        "{STORY}"
    );
}

/// Over the toy, the interpreter kills both grant mutants, and the baseline holds every scenario
/// `synthesize` writes, the `grant/denied` one among them.
#[test]
fn the_interpreter_kills_the_grant_mutants_of_the_toy() {
    let report = audit(GRANTS, MutantClass::ALL);
    assert_eq!(
        verdict(&report, DROP),
        Verdict::Killed,
        "{}",
        report.render_text()
    );
    assert_eq!(
        verdict(&report, ADD),
        Verdict::Killed,
        "{}",
        report.render_text()
    );
    let (documents, texts) = documents(GRANTS);
    let ir = mutate::compile(documents, &texts).expect("the toy compiles");
    let synthesized = synthesize(&ir).suite.scenarios.len();
    assert_eq!(
        synthesized, 2,
        "the toy synthesizes the outcome and the refusal"
    );
    assert_eq!(
        report.baseline.scenarios,
        synthesized,
        "{STORY}: {}",
        report.render_text()
    );
}

/// A drop where another actor lacks the grant too is still killed: the refusal is sent as the actor
/// that lost it.
#[test]
fn a_grant_drop_is_sent_as_the_actor_that_lost_it() {
    let report = audit(DROP_NOT_LOWEST, &[MutantClass::GrantDrop]);
    let id = "grant-drop/library.shelf.Librarian/library.shelf.CatalogueBook";
    assert_eq!(
        verdict(&report, id),
        Verdict::Killed,
        "{STORY}: {}",
        report.render_text()
    );
}

/// A selection without either grant class keeps the baseline it had, without the refusal, and the
/// ids every other class had.
#[test]
fn a_selection_without_grant_classes_keeps_its_baseline_and_ids() {
    let older: Vec<MutantClass> = MutantClass::ALL
        .iter()
        .copied()
        .filter(|class| !matches!(class, MutantClass::GrantDrop | MutantClass::GrantAdd))
        .collect();
    let without = audit(GRANTS, &older);
    assert_eq!(without.baseline.scenarios, 1, "{}", without.render_text());
    let with = audit(GRANTS, MutantClass::ALL);
    let ids = |report: &MutationReport, grant: bool| -> Vec<String> {
        report
            .mutants
            .iter()
            .filter(|entry| entry.id.starts_with("grant-") == grant)
            .map(|entry| entry.id.clone())
            .collect()
    };
    assert_eq!(ids(&without, false), ids(&with, false), "{STORY}");
    assert_eq!(ids(&without, true), Vec::<String>::new());
    assert_eq!(ids(&with, true), [ADD.to_owned(), DROP.to_owned()]);
}

/// `--emit` names the grant mutants in an `ess-mutation-manifest/7` whose baseline suite carries
/// the refusal, and `--collect` scores them; an emission without a grant class keeps its version.
#[test]
fn the_manifest_names_the_grant_classes_and_collect_scores_them() {
    let (documents, texts) = documents(GRANTS);
    let ir = mutate::compile(documents.clone(), &texts).expect("the model compiles");
    let emission = mutate::emit_for(
        &documents,
        &texts,
        &[MutantClass::GrantDrop, MutantClass::GrantAdd],
        None,
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let manifest: Value = serde_json::from_str(&emission.files[MANIFEST_FILE]).unwrap();
    assert_eq!(
        manifest["format"], "ess-mutation-manifest/7",
        "{STORY}: {manifest:#}"
    );
    let baseline: Value =
        serde_json::from_str(&emission.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")]).unwrap();
    assert!(baseline["scenarios"].get(DENIED).is_some(), "{baseline:#}");
    let mut written: BTreeMap<String, String> = emission.files.clone();
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|mutant| mutant.dir.clone()),
    );
    for dir in dirs {
        let admitted = AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")])
            .expect("an emitted suite is admitted");
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir.clone()));
        let text = CountReport::from_run(&run, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    let collected =
        mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
    assert_eq!(
        verdict(&collected, DROP),
        Verdict::Killed,
        "{}",
        collected.render_text()
    );
    assert_eq!(
        verdict(&collected, ADD),
        Verdict::Killed,
        "{}",
        collected.render_text()
    );

    let older = mutate::emit_for(
        &documents,
        &texts,
        &[MutantClass::ErrorSwap, MutantClass::OrderFlip],
        None,
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let manifest: Value = serde_json::from_str(&older.files[MANIFEST_FILE]).unwrap();
    assert_ne!(
        manifest["format"], "ess-mutation-manifest/7",
        "{manifest:#}"
    );
    let baseline: Value =
        serde_json::from_str(&older.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")]).unwrap();
    assert!(baseline["scenarios"].get(DENIED).is_none(), "{baseline:#}");
}

/// A manifest of an earlier version naming a grant mutant is refused naming the version that has
/// the class.
#[test]
fn a_version_6_manifest_naming_a_grant_mutant_is_refused() {
    let (documents, texts) = documents(GRANTS);
    let emission = mutate::emit_for(&documents, &texts, &[MutantClass::GrantDrop], None)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let older =
        emission.files[MANIFEST_FILE].replace("ess-mutation-manifest/7", "ess-mutation-manifest/6");
    let refusal =
        mutate::Manifest::from_json(&older).expect_err("a /6 manifest has no grant class");
    assert!(
        refusal.contains("grant-drop") && refusal.contains("ess-mutation-manifest/7"),
        "{STORY}: {refusal}"
    );
}
