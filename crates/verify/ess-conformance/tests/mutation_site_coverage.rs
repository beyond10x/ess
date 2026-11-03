//! The mutation audit names the sites it could not mutate
//! (story:the-mutation-audit-reports-the-sites-it-could-not-mutate, beyond10x/ess#711).
//!
//! A selected class that excludes a site, or finds no edit there the model admits, lists it in the
//! report and the manifest with its reason, beside the stillborn mutants, and the text summary
//! says how many of the sites considered have a scored mutant. A creating or `Optional` write with
//! no sibling input of its type is retargeted to a literal, and a transition on a linear lifecycle
//! that no other state can be the target of is listed rather than left stillborn.
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, MutantClass, MutationReport, UnmutatedReason, Verdict, BASELINE_DIR,
    MANIFEST_FILE, REPORT_FILE, SUITE_FILE,
};
use ess_conformance::{AdmittedSuite, CountReport, Runner};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

const STORY: &str = "story:the-mutation-audit-reports-the-sites-it-could-not-mutate";

/// The toy specification of beyond10x/ess#711, brand-free: a two-state linear lifecycle, three
/// single-event accepting branches, a creating write with no sibling input of its type and an
/// `Optional` update. Kept here rather than under `fixtures/`, so the repository-wide synthesis
/// byte tables, which walk every `.yaml` file, are not moved by it.
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

const TRANSITION: &str = "transition-to/library.shelf.Book.lend";
const CREATING_DROP: &str = "sets-drop/library.shelf.CatalogueBook/catalogued/position";
const OPTIONAL_DROP: &str = "sets-drop/library.shelf.AnnotateBook/annotated/note";
const CREATING_LITERAL: &str = "sets-retarget/library.shelf.CatalogueBook/catalogued/position";
const OPTIONAL_LITERAL: &str = "sets-retarget/library.shelf.AnnotateBook/annotated/note";

fn documents(text: &str, label: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn shelf() -> (Vec<Document>, SourceMap) {
    documents(SHELF, "shelf.yaml")
}

fn audit_of(
    (documents, texts): (Vec<Document>, SourceMap),
    classes: &[MutantClass],
) -> MutationReport {
    let ir = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    mutate::audit(&documents, &texts, classes, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn json(report: &MutationReport) -> Value {
    serde_json::from_str(&report.to_canonical_json()).expect("the report is JSON")
}

fn listed<'a>(json: &'a Value, id: &str) -> &'a Value {
    json["unmutated_sites"]
        .as_array()
        .and_then(|sites| sites.iter().find(|site| site["id"] == id))
        .unwrap_or_else(|| panic!("{STORY}: `{id}` is not listed: {json:#}"))
}

/// The transition with no other admissible target and the creating write `sets-drop` excludes are
/// listed with their reasons, in the report's JSON and its text; neither is a stillborn mutant.
#[test]
fn the_report_names_each_site_without_a_live_mutant() {
    let report = audit_of(shelf(), MutantClass::ALL);
    let json = json(&report);
    assert_eq!(json["format"], "ess-mutation-report/8", "{json:#}");
    let transition = listed(&json, TRANSITION);
    assert_eq!(transition["class"], "transition-to");
    assert_eq!(
        transition["reason"],
        UnmutatedReason::NoAdmissibleState.as_str()
    );
    let creating = listed(&json, CREATING_DROP);
    assert_eq!(creating["class"], "sets-drop");
    assert_eq!(creating["reason"], "creates_row");
    assert!(
        !report.mutants.iter().any(|entry| entry.id == TRANSITION),
        "{STORY}: a listed site is not also a stillborn mutant: {}",
        report.render_text()
    );
    let text = report.render_text();
    for id in [TRANSITION, CREATING_DROP] {
        assert!(
            text.lines()
                .any(|line| line.starts_with(&format!("unmutated {id} ("))),
            "{STORY}: {text}"
        );
    }
}

/// The creating write, which has no sibling input of its type and whose field a view reads, gets
/// a live `sets-retarget` mutant: a literal of the field's type in place of the input, and it is
/// killed. The `Optional` write has no sibling either, but no view, guard or other write reads
/// `note`, so no target's answer could tell a literal there from the input: it is listed as
/// `unobserved_field` rather than left a mutant that can only survive, which kept the default exit
/// of the toy at 0 (story:the-mutation-audit-reports-the-sites-it-could-not-mutate, re-pinned from
/// a live mutant). `sets-drop` still lists the `Optional` write, whose absence no synthesized row
/// states.
#[test]
fn a_creating_or_optional_write_gets_a_live_mutant() {
    let report = audit_of(shelf(), &[MutantClass::SetsDrop, MutantClass::SetsRetarget]);
    let json = json(&report);
    let optional = listed(&json, OPTIONAL_DROP).clone();
    assert_eq!(optional["reason"], "optional_field", "{optional:#}");
    let unread = listed(&json, OPTIONAL_LITERAL).clone();
    assert_eq!(
        (&unread["class"], &unread["reason"]),
        (
            &Value::from("sets-retarget"),
            &Value::from(UnmutatedReason::UnobservedField.as_str())
        ),
        "{STORY}: {unread:#}"
    );
    assert!(
        !report
            .mutants
            .iter()
            .any(|entry| entry.id == OPTIONAL_LITERAL),
        "{STORY}: a listed site is not also a mutant: {}",
        report.render_text()
    );
    let literal = report
        .mutants
        .iter()
        .find(|entry| entry.id == CREATING_LITERAL)
        .unwrap_or_else(|| panic!("{STORY}: no `{CREATING_LITERAL}`: {}", report.render_text()));
    assert_eq!(literal.verdict, Verdict::Killed, "{}", report.render_text());
}

/// Whether a field is read is the compiler's to say: once a view reads `note`, the `Optional`
/// write gets its literal mutant again, and it is killed.
#[test]
fn a_field_a_view_reads_keeps_its_literal_mutant() {
    let read = SHELF.replace(
        "      - {name: position, type: Integer}\ncomponents:",
        "      - {name: position, type: Integer}\n      - {name: note, type: Optional<String>}\ncomponents:",
    );
    assert_ne!(read, SHELF, "the view gains `note`");
    let report = audit_of(documents(&read, "shelf.yaml"), &[MutantClass::SetsRetarget]);
    assert!(
        !report
            .unmutated_sites
            .iter()
            .flatten()
            .any(|site| site.id == OPTIONAL_LITERAL),
        "{STORY}: {}",
        report.render_text()
    );
    let entry = report
        .mutants
        .iter()
        .find(|entry| entry.id == OPTIONAL_LITERAL)
        .unwrap_or_else(|| panic!("{STORY}: no `{OPTIONAL_LITERAL}`: {}", report.render_text()));
    assert_eq!(entry.verdict, Verdict::Killed, "{}", report.render_text());
}

/// A selection that finds only unmutated sites is refused (`ESS-MUTATE-003`), and the refusal
/// names each site with its reason instead of saying it found none.
#[test]
fn a_refusal_over_unmutated_sites_names_them() {
    let (documents, texts) = shelf();
    let ir = mutate::compile(documents.clone(), &texts).expect("the toy compiles");
    let refusal = mutate::audit(&documents, &texts, &[MutantClass::TransitionTo], || {
        Interpreted::for_model(ir.clone())
    })
    .expect_err("transition-to alone finds no mutant over the toy");
    let said = refusal.to_string();
    assert!(
        said.starts_with("refusal[ESS-MUTATE-003]: `transition-to` find(s) no site it can mutate"),
        "{STORY}: {said}"
    );
    assert!(
        said.lines().any(
            |line| line.starts_with(&format!("unmutated {TRANSITION} (no_admissible_state): "))
        ),
        "{STORY}: {said}"
    );
    assert!(!said.contains("find(s) no site in"), "{STORY}: {said}");
}

/// The summary line ends with the coverage count: sites with a scored mutant out of the sites
/// considered, the listed ones and the stillborn ones among them.
#[test]
fn the_summary_gives_a_coverage_count() {
    let report = audit_of(shelf(), MutantClass::ALL);
    let (scored, considered) = report.coverage();
    let listed = report.unmutated_sites.as_ref().map_or(0, Vec::len);
    assert_eq!(
        considered,
        report.mutants.len() + listed + report.unavailable_sites.as_ref().map_or(0, Vec::len),
        "{}",
        report.render_text()
    );
    assert_eq!(
        scored,
        report
            .mutants
            .iter()
            .filter(|entry| matches!(entry.verdict, Verdict::Killed | Verdict::Survived))
            .count()
    );
    assert!(scored < considered, "{}", report.render_text());
    let text = report.render_text();
    let summary = text.lines().next().expect("a summary line");
    assert!(
        summary.ends_with(&format!("; {scored} of {considered} site(s) scored")),
        "{STORY}: {summary}"
    );
    assert!(report.sites_without_a_live_mutant() > 0, "{text}");
}

/// `--emit` writes the list into an `ess-mutation-manifest/7`, and `--collect` reads it back into
/// the report it scores.
#[test]
fn the_manifest_lists_the_sites_and_collect_reports_them() {
    let (documents, texts) = shelf();
    let ir = mutate::compile(documents.clone(), &texts).expect("the model compiles");
    let emission = mutate::emit_for(&documents, &texts, MutantClass::ALL, None)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let manifest: Value = serde_json::from_str(&emission.files[MANIFEST_FILE]).unwrap();
    assert_eq!(
        manifest["format"], "ess-mutation-manifest/7",
        "{manifest:#}"
    );
    let _ = listed(&manifest, TRANSITION);
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
    let json = json(&collected);
    assert_eq!(json["format"], "ess-mutation-report/8", "{json:#}");
    assert_eq!(json["unmutated_sites"], manifest["unmutated_sites"]);
}

/// A manifest of an earlier version carrying the list is refused naming the version that has it.
#[test]
fn a_version_6_manifest_carrying_unmutated_sites_is_refused() {
    let (documents, texts) = shelf();
    let emission = mutate::emit_for(&documents, &texts, MutantClass::ALL, None)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let older =
        emission.files[MANIFEST_FILE].replace("ess-mutation-manifest/7", "ess-mutation-manifest/6");
    let mut written = emission.files.clone();
    written.insert(MANIFEST_FILE.to_owned(), older);
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("a /6 manifest has no unmutated_sites");
    assert!(
        refusal.to_string().contains("ess-mutation-manifest/7"),
        "{STORY}: {refusal}"
    );
}

/// A model and class selection with nothing to list keeps its report version and bytes: no
/// `unmutated_sites` member, and the format it had.
#[test]
fn nothing_to_list_keeps_the_report_version() {
    let report = audit_of(
        documents(
            include_str!("fixtures/mutation-authored.yaml"),
            "mutation-authored.yaml",
        ),
        &[MutantClass::ErrorSwap, MutantClass::GuardNegate],
    );
    let json = json(&report);
    assert!(json.get("unmutated_sites").is_none(), "{json:#}");
    assert_ne!(json["format"], "ess-mutation-report/8", "{json:#}");
}

/// Listed sites do not change the default exit decision: the report's own count of what keeps
/// the audit from succeeding (`unaudited_sites`) does not include them.
#[test]
fn listed_sites_are_not_unaudited_sites() {
    let report = audit_of(
        shelf(),
        &[
            MutantClass::TransitionTo,
            MutantClass::SetsDrop,
            MutantClass::SetsRetarget,
        ],
    );
    assert!(report.unmutated_sites.is_some(), "{}", report.render_text());
    assert_eq!(report.unaudited_sites(), 0, "{STORY}: D135");
}
