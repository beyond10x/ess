//! The mutation audit's guard operators on a `when_subject:` and a `when_related:` predicate
//! (<https://github.com/beyond10x/ess/issues/515>): `guard-negate`, `guard-connective` and
//! `guard-boundary` edit the predicate over the stored row as they edit a `when:`, each such site
//! naming the key it edits after the outcome, and each mutant is compiled, synthesized and scored
//! against the target implementing the unchanged model. A model without either guard keeps every
//! mutant it had.
#![allow(clippy::missing_panics_doc)]

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const STORE: &str = include_str!("fixtures/mutation-stored-guards.yaml");

const GUARDS: &[MutantClass] = &[
    MutantClass::GuardBoundary,
    MutantClass::GuardNegate,
    MutantClass::GuardConnective,
];

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let label = "store.yaml".to_owned();
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

/// Every guard mutant of a stored predicate, by id, with what it changed.
fn stored_guard_mutants(documents: &[Document]) -> Vec<mutate::Mutant> {
    mutate::mutants(documents, GUARDS)
        .into_iter()
        .filter(|mutant| mutant.id.contains("/when_subject") || mutant.id.contains("/when_related"))
        .collect()
}

#[test]
fn a_when_subject_and_a_when_related_predicate_offer_every_guard_site() {
    let (documents, _) = documents(STORE);
    let found = stored_guard_mutants(&documents);
    let ids: Vec<&str> = found.iter().map(|mutant| mutant.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "guard-boundary/demo.store.Restock/full/when_subject/0",
            "guard-boundary/demo.store.Restock/full/when_subject/1",
            "guard-boundary/demo.store.Restock/full/when_subject/1-outward",
            "guard-boundary/demo.store.Take/too-little/when_related/0",
            "guard-boundary/demo.store.Take/too-little/when_related/1",
            "guard-boundary/demo.store.Take/too-little/when_related/1-outward",
            "guard-connective/demo.store.Restock/full/when_subject/0",
            "guard-connective/demo.store.Take/too-little/when_related/0",
            "guard-negate/demo.store.Restock/full/when_subject",
            "guard-negate/demo.store.Take/too-little/when_related",
        ],
        "{found:#?}"
    );
    let change = |id: &str| {
        found
            .iter()
            .find(|mutant| mutant.id == id)
            .map(|mutant| mutant.change.as_str())
            .unwrap_or_default()
    };
    assert_eq!(
        change("guard-negate/demo.store.Restock/full/when_subject"),
        "`when_subject: (stock >= {fact: capacity} or capacity <= 0)` becomes `when_subject: not \
         ((stock >= {fact: capacity} or capacity <= 0))`"
    );
    assert_eq!(
        change("guard-boundary/demo.store.Take/too-little/when_related/0"),
        "`stock < input.quantity` becomes `stock <= input.quantity`"
    );
    assert_eq!(
        change("guard-boundary/demo.store.Restock/full/when_subject/1-outward"),
        "`capacity <= 0` becomes `capacity <= 1`"
    );
}

/// The negated and re-connected stored guards are what a target ignoring the guard would answer:
/// the suite synthesized for each mutant fails against the target implementing the original, so
/// each is killed, as the same mutant of a `when:` is.
#[test]
fn the_stored_guard_mutants_are_scored_against_the_unchanged_target() {
    let (documents, texts) = documents(STORE);
    let original = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| {
        panic!("{error:?}");
    });
    let found = stored_guard_mutants(&documents);
    assert!(!found.is_empty(), "the fixture offers stored guard sites");
    for mutant in &found {
        let entry = mutate::evaluate(&documents, &texts, mutant, || {
            Interpreted::for_model(original.clone())
        })
        .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
        assert_eq!(
            entry.verdict,
            Verdict::Killed,
            "{} is killed by its own synthesized suite: {entry:#?}",
            mutant.id
        );
        assert!(
            entry
                .killers
                .as_ref()
                .is_some_and(|killers| !killers.is_empty()),
            "{}: {entry:#?}",
            mutant.id
        );
    }
}

/// The audit as an operator reads it: the stored guard mutants are counted and listed beside the
/// rest, killed.
#[test]
fn the_audit_reports_the_stored_guard_mutants_killed() {
    let (documents, texts) = documents(STORE);
    let ir = mutate::compile(documents.clone(), &texts).expect("the specification compiles");
    let report = mutate::audit(&documents, &texts, GUARDS, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let text = report.render_text();
    for id in [
        "guard-negate/demo.store.Restock/full/when_subject",
        "guard-negate/demo.store.Take/too-little/when_related",
        "guard-connective/demo.store.Restock/full/when_subject/0",
        "guard-connective/demo.store.Take/too-little/when_related/0",
    ] {
        assert!(
            text.lines()
                .any(|line| line.starts_with(&format!("killed {id}: "))),
            "{id}: {text}"
        );
    }
}

/// A model with neither guard has no site naming either key.
#[test]
fn a_model_without_stored_guards_offers_no_such_site() {
    let (documents, _) = documents(include_str!("fixtures/mutation-survivor.yaml"));
    assert!(
        mutate::mutants(&documents, MutantClass::ALL)
            .iter()
            .all(|mutant| !mutant.id.contains("/when_subject")
                && !mutant.id.contains("/when_related")),
        "no stored guard site"
    );
}
