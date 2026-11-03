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

// ---- every grant is a mutant or listed (story:the-mutation-audit-reports-the-sites-it-could-not-mutate)

const LISTED: &str = "story:the-mutation-audit-reports-the-sites-it-could-not-mutate";

/// [`GRANTS`] with its `actors:` block replaced by `actors`.
fn with_actors(actors: &str) -> String {
    let declared = "actors:
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Reader
";
    assert!(GRANTS.contains(declared));
    GRANTS.replace(declared, &format!("actors:\n{actors}"))
}

/// Every `.yaml` file under `examples/<name>`.
fn example(name: &str) -> Vec<Document> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name);
    let mut pending = vec![root];
    let mut found = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("readable") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("readable");
            let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
            (Source::new(path.display().to_string()), raw)
        })
        .collect()
}

/// The reason each `grant-drop/<actor>/<command>` of a declared `may:` entry is listed with, or
/// `"mutant"` where it is a mutant; a grant that is neither fails.
fn every_grant(documents: &[Document]) -> BTreeMap<String, String> {
    let selection = mutate::selection(documents, &[MutantClass::GrantDrop, MutantClass::GrantAdd]);
    let mut accounted = BTreeMap::new();
    for (_, file) in documents {
        for actor in &file.actors {
            for command in &actor.may {
                let id = format!("grant-drop/{}/{command}", actor.name);
                let how = if selection.mutants.iter().any(|mutant| mutant.id == id) {
                    "mutant".to_owned()
                } else {
                    let site = selection
                        .unmutated
                        .iter()
                        .find(|site| site.id == id)
                        .unwrap_or_else(|| {
                            panic!(
                                "{LISTED}: `{id}` is neither a mutant nor listed: {selection:#?}"
                            )
                        });
                    site.reason.as_str().to_owned()
                };
                accounted.insert(id, how);
            }
        }
    }
    accounted
}

/// The class the adversary found two members of: every `may:` entry a specification declares is a
/// `grant-drop` mutant or an unmutated site with its reason, the system actor's and those of
/// commands nothing serves included, over billing and the grant toys.
#[test]
fn every_declared_grant_is_a_grant_drop_mutant_or_listed() {
    assert_eq!(
        every_grant(&example("billing")),
        BTreeMap::from([(
            "grant-drop/billing.invoice.Customer/billing.invoice.CreateInvoice".to_owned(),
            "command_not_served".to_owned()
        )])
    );
    assert_eq!(
        every_grant(&documents(GRANTS).0),
        BTreeMap::from([(DROP.to_owned(), "mutant".to_owned())])
    );
    let unserved = GRANTS.replace("    reached_by: network\n", "");
    assert_eq!(
        every_grant(&documents(&unserved).0),
        BTreeMap::from([(DROP.to_owned(), "command_not_served".to_owned())])
    );
    let system = with_actors(
        "  - name: library.shelf.Keeper
    system: true
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Librarian
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Reader
",
    );
    assert_eq!(
        every_grant(&documents(&system).0),
        BTreeMap::from([
            (
                "grant-drop/library.shelf.Keeper/library.shelf.CatalogueBook".to_owned(),
                "system_actor".to_owned()
            ),
            (DROP.to_owned(), "mutant".to_owned()),
        ])
    );
}

/// Each listed grant says what was not audited, and its reason belongs to its class.
#[test]
fn a_listed_grant_says_why() {
    let selection = mutate::selection(&example("billing"), &[MutantClass::GrantDrop]);
    let site = selection
        .unmutated
        .iter()
        .find(|site| site.class == MutantClass::GrantDrop)
        .unwrap_or_else(|| panic!("{LISTED}: {selection:#?}"));
    assert!(
        site.unaudited.contains("`billing.invoice.Customer`")
            && site.unaudited.contains("`billing.invoice.CreateInvoice`")
            && site.unaudited.contains("network"),
        "{LISTED}: {}",
        site.unaudited
    );
    let without = mutate::selection(&example("billing"), &[MutantClass::GrantAdd]);
    assert!(
        without.unmutated.is_empty(),
        "a held grant is listed under grant-drop only: {without:#?}"
    );
}

/// Where the gaining actor carries attributes beside an attribute-carrying holder, no scenario
/// can be sent as it asking what it asked the holder: the site is listed as `attributed_holder`,
/// not left a mutant that can only survive.
#[test]
fn a_grant_add_no_scenario_can_be_sent_for_is_listed() {
    let text = with_actors(
        "  - name: library.shelf.Librarian
    attributes: [{name: branch, type: String}]
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Reader
    attributes: [{name: branch, type: String}]
",
    );
    let (documents, texts) = documents(&text);
    mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let selection = mutate::selection(&documents, &[MutantClass::GrantAdd]);
    let site = selection
        .unmutated
        .iter()
        .find(|site| site.id == ADD)
        .unwrap_or_else(|| panic!("{STORY}: `{ADD}` is not listed: {selection:#?}"));
    assert_eq!(site.reason.as_str(), "attributed_holder", "{STORY}");
    assert!(
        !selection.mutants.iter().any(|mutant| mutant.id == ADD),
        "{STORY}: {selection:#?}"
    );
}

/// The `grant/admitted/<actor>` copy a grant-add mutant of an attribute-carrying holder gets is
/// written only under the mutation audit: the toy's own synthesis keeps its scenarios.
#[test]
fn the_added_grant_scenario_is_the_mutants_alone() {
    let text = with_actors(
        "  - name: library.shelf.Librarian
    attributes: [{name: branch, type: String}]
    may: [library.shelf.CatalogueBook]
  - name: library.shelf.Reader
    may: [library.shelf.CatalogueBook]
",
    );
    let (documents, texts) = documents(&text);
    let ir = mutate::compile(documents, &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let suite = synthesize(&ir).suite;
    assert!(
        !suite
            .scenarios
            .keys()
            .any(|id| id.to_string().contains("/grant/admitted/")),
        "{STORY}: {:?}",
        suite.scenarios.keys().collect::<Vec<_>>()
    );
}
