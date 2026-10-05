//! The mutation audit's operator for a row set (`docs/design/filtered-related-reads.md`;
//! beyond10x/ess#228, #299): a `guard-boundary` mutant moves a `count:` test's bound one step —
//! `gt 1` becomes `gte 1`, `eq 0` becomes `eq 1` — and is compiled and scored against the target
//! implementing the unchanged model. The selector and a `forall` predicate are not mutated, and a
//! model without a row set offers no such site.

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, Mutation, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const READS: &str = include_str!("fixtures/filtered-related-reads.yaml");

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let mut texts = SourceMap::new();
    texts.insert("jobs.yaml".to_owned(), text.to_owned());
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    (vec![(Source::new("jobs.yaml"), raw)], texts)
}

#[test]
fn a_row_set_count_bound_is_mutated_and_killed() {
    let (documents, texts) = documents(READS);
    let original = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| {
        panic!("{error:?}");
    });
    let mutants: Vec<_> = mutate::mutants(&documents, &[MutantClass::GuardBoundary])
        .into_iter()
        .filter(|mutant| matches!(mutant.mutation, Mutation::RowSetCount { .. }))
        .collect();
    let changes: Vec<&str> = mutants
        .iter()
        .map(|mutant| mutant.change.as_str())
        .collect();
    for expected in [
        "`count: {gt: 1}` becomes `count: {gte: 1}`",
        "`count: {eq: 0}` becomes `count: {eq: 1}`",
    ] {
        assert!(changes.contains(&expected), "{expected}: {changes:#?}");
    }
    let ids: Vec<&str> = mutants.iter().map(|mutant| mutant.id.as_str()).collect();
    for expected in [
        "guard-boundary/demo.jobs.Retry/ambiguous/row-set-count",
        "guard-boundary/demo.jobs.Retry/started/row-set-count",
        "guard-boundary/demo.jobs.Close/crowded/row-set-count",
    ] {
        assert!(ids.contains(&expected), "{expected}: {ids:#?}");
    }
    for mutant in &mutants {
        let entry = mutate::evaluate(&documents, &texts, mutant, || {
            Interpreted::for_model(original.clone())
        })
        .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
        // `crowded` counting the subject itself at `gte 1` answers every open row, so the mutant's
        // synthesis has no witness left for `closed` and says so; every other bound is killed.
        let expected = if mutant.id.contains("/crowded/") {
            Verdict::Unwitnessed
        } else {
            Verdict::Killed
        };
        assert_eq!(
            entry.verdict, expected,
            "{} is scored on its own synthesized suite: {entry:#?}",
            mutant.id
        );
    }
}

#[test]
fn a_model_without_a_row_set_offers_no_row_set_site() {
    let leases = include_str!("fixtures/now-stored-rows.yaml");
    let (documents, _) = documents(leases);
    assert!(
        mutate::mutants(&documents, MutantClass::ALL)
            .iter()
            .all(|mutant| !matches!(mutant.mutation, Mutation::RowSetCount { .. })),
        "no row-set site"
    );
}
