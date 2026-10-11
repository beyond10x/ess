//! Every `sets:` entry has a `sets-drop` site or a stated reason it has none
//! (<https://github.com/beyond10x/ess/issues/515>).
//!
//! A model whose three `sets:` blocks were all on creating branches got no `sets-drop` mutant and
//! no word about why: the enumeration skipped each entry silently. The audit now decides every
//! entry once, as a site or as an entry listed with its reason, and this suite holds that decision
//! total over every fixture and example in the repository.
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, SetsDropReason, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const KEYS: &str = include_str!("fixtures/mutation-sets-entries.yaml");

fn documents(label: &str, text: &str) -> Option<(Vec<Document>, SourceMap)> {
    let raw = RawSpecFile::parse(text).ok()?;
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    Some((vec![(Source::new(label.to_owned()), raw)], texts))
}

fn keys() -> (Vec<Document>, SourceMap) {
    documents("keys.yaml", KEYS).expect("the fixture is well formed")
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// Every `.yaml` under `directory`, in byte order of path.
fn yaml_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// An example's parsed documents, every file that parses as a specification document.
fn example(directory: &Path) -> Vec<Document> {
    yaml_under(directory)
        .into_iter()
        .filter_map(|path| {
            let text = std::fs::read_to_string(&path).ok()?;
            let raw = RawSpecFile::parse(&text).ok()?;
            Some((Source::new(path.display().to_string()), raw))
        })
        .collect()
}

/// The id a `sets-drop` site of every `sets:` entry the documents declare would have.
fn every_entry(documents: &[Document]) -> BTreeSet<String> {
    documents
        .iter()
        .flat_map(|(_, file)| &file.commands)
        .flat_map(|command| {
            command.outcomes.iter().flat_map(move |outcome| {
                outcome.sets.0.iter().map(move |set| {
                    format!("sets-drop/{}/{}/{}", command.name, outcome.name, set.target)
                })
            })
        })
        .collect()
}

#[test]
fn every_sets_entry_has_a_site_or_is_listed_with_its_reason() {
    let (documents, _) = keys();
    let sites: Vec<String> = mutate::mutants(&documents, &[MutantClass::SetsDrop])
        .into_iter()
        .map(|mutant| mutant.id)
        .collect();
    assert_eq!(sites, ["sets-drop/demo.keys.Rename/renamed/label"]);
    let listed: Vec<(String, SetsDropReason)> =
        mutate::unmutated_sets(&documents, &[MutantClass::SetsDrop])
            .into_iter()
            .map(|entry| (entry.id, entry.reason))
            .collect();
    assert_eq!(
        listed,
        [
            (
                "sets-drop/demo.keys.Issue/issued/label".to_owned(),
                SetsDropReason::CreatesRow
            ),
            (
                "sets-drop/demo.keys.Issue/issued/owner".to_owned(),
                SetsDropReason::CreatesRow
            ),
            (
                "sets-drop/demo.keys.Issue/issued/tier".to_owned(),
                SetsDropReason::CreatesRow
            ),
            (
                "sets-drop/demo.keys.Rename/renamed/note".to_owned(),
                SetsDropReason::OptionalField
            ),
            (
                "sets-drop/demo.keys.Rename/renamed/tier".to_owned(),
                SetsDropReason::SourceNotInput
            ),
        ]
    );
}

#[test]
fn each_listed_entry_says_why_it_has_no_mutant() {
    let (documents, _) = keys();
    let lines: Vec<String> = mutate::unmutated_sets(&documents, &[MutantClass::SetsDrop])
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        lines[1],
        "sets-drop/demo.keys.Issue/issued/owner has no mutant (creates_row): `owner: \
         input.owner` is written by a branch that creates its row, so without it the value is \
         the implementation's to choose, a weaker rule no correct target fails, and an outcome \
         whose guard is arranged through the field loses its scenario"
    );
    assert_eq!(
        lines[3],
        "sets-drop/demo.keys.Rename/renamed/note has no mutant (optional_field): `note` is \
         `Optional`, and a row nothing wrote holds it absent, which no synthesized row expectation \
         states"
    );
    assert_eq!(
        lines[4],
        "sets-drop/demo.keys.Rename/renamed/tier has no mutant (source_not_input): `tier: Gold` \
         is not read from an input, and only an input is sent apart from what the row held, so no \
         scenario tells the dropped write from the kept value"
    );
    for (reason, word) in [
        (SetsDropReason::CreatesRow, "creates_row"),
        (SetsDropReason::OptionalField, "optional_field"),
        (SetsDropReason::SourceNotInput, "source_not_input"),
    ] {
        assert_eq!(reason.as_str(), word);
    }
}

/// Only an audit that selects `sets-drop` lists anything.
#[test]
fn nothing_is_listed_when_sets_drop_is_not_selected() {
    let (documents, _) = keys();
    let others: Vec<MutantClass> = MutantClass::ALL
        .iter()
        .copied()
        .filter(|class| *class != MutantClass::SetsDrop)
        .collect();
    assert_eq!(mutate::unmutated_sets(&documents, &others), []);
    assert_eq!(
        mutate::unmutated_sets(&documents, MutantClass::ALL).len(),
        5,
        "every class selects sets-drop"
    );
}

/// The site is scored as every other class's is: its synthesized suite fails against the target
/// implementing the unchanged model.
#[test]
fn the_sets_drop_site_is_scored_against_the_unchanged_target() {
    let (documents, texts) = keys();
    let ir = mutate::compile(documents.clone(), &texts).expect("the fixture compiles");
    let report = mutate::audit(&documents, &texts, &[MutantClass::SetsDrop], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let verdicts: Vec<(&str, Verdict)> = report
        .mutants
        .iter()
        .map(|entry| (entry.id.as_str(), entry.verdict))
        .collect();
    assert_eq!(
        verdicts,
        [("sets-drop/demo.keys.Rename/renamed/label", Verdict::Killed)],
        "{}",
        report.render_text()
    );
}

/// The issue's shape: every write is on a creating branch or of an `Optional` field, so there is
/// no site, and every entry is listed rather than left out in silence.
#[test]
fn billing_has_no_site_and_lists_every_entry() {
    let documents = example(&root().join("examples/billing"));
    assert_eq!(mutate::mutants(&documents, &[MutantClass::SetsDrop]), []);
    let listed: Vec<(String, SetsDropReason)> =
        mutate::unmutated_sets(&documents, &[MutantClass::SetsDrop])
            .into_iter()
            .map(|entry| (entry.id, entry.reason))
            .collect();
    assert_eq!(
        listed,
        [
            (
                "sets-drop/billing.invoice.CreateInvoice/accepted/account_id".to_owned(),
                SetsDropReason::CreatesRow
            ),
            (
                "sets-drop/billing.invoice.CreateInvoice/accepted/reminder_count".to_owned(),
                SetsDropReason::CreatesRow
            ),
            (
                "sets-drop/billing.invoice.CreateInvoice/accepted/total".to_owned(),
                SetsDropReason::CreatesRow
            ),
            (
                "sets-drop/billing.invoice.IssueInvoice/issued/issued_at".to_owned(),
                SetsDropReason::OptionalField
            ),
        ]
    );
}

/// The class, not the instance: in every single-file fixture and every example that parses, each
/// `sets:` entry is exactly one of a site and a listed entry, and nothing else is either.
#[test]
fn no_sets_entry_in_any_fixture_or_example_goes_unaccounted() {
    let mut models: Vec<(String, Vec<Document>)> = Vec::new();
    for path in yaml_under(&root().join("crates/verify/ess-conformance/tests/fixtures")) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Some((documents, _)) = documents(&path.display().to_string(), &text) {
            models.push((path.display().to_string(), documents));
        }
    }
    let examples = root().join("examples");
    let mut directories: Vec<PathBuf> = std::fs::read_dir(&examples)
        .expect("the examples are readable")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.is_dir())
        .collect();
    directories.sort();
    for directory in directories {
        models.push((directory.display().to_string(), example(&directory)));
    }
    let mut entries = 0;
    for (name, documents) in &models {
        let declared = every_entry(documents);
        let sites: BTreeSet<String> = mutate::mutants(documents, &[MutantClass::SetsDrop])
            .into_iter()
            .map(|mutant| mutant.id)
            .collect();
        let listed: Vec<String> = mutate::unmutated_sets(documents, &[MutantClass::SetsDrop])
            .into_iter()
            .map(|entry| entry.id)
            .collect();
        let listed_set: BTreeSet<String> = listed.iter().cloned().collect();
        assert_eq!(listed.len(), listed_set.len(), "{name}: listed once each");
        assert!(
            sites.is_disjoint(&listed_set),
            "{name}: a site is also listed: {:?}",
            sites.intersection(&listed_set).collect::<Vec<_>>()
        );
        let accounted: BTreeSet<String> = sites.union(&listed_set).cloned().collect();
        assert_eq!(accounted, declared, "{name}");
        entries += declared.len();
    }
    assert!(
        entries > 100,
        "the sweep reads the repository's models: {entries} entries in {} models",
        models.len()
    );
}
