//! The mutation audit's operators for one constant offset (`docs/design/expression-family-source22.md`,
//! A2, final review decision 16): a guard-boundary mutant of `upper > lower + 5` edits the typed
//! offset the source format resolved — `upper >= lower + 5`, `upper <= lower - 2` — and is compiled
//! and scored, never left as the text `lower + 5` that a mutant of the written spelling would compare
//! with and the model refuse.

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, Mutation, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const MODEL: &str = r"format: ess/22
system: pool
version: v1
domain: pool.lease
errors:
  - {name: pool.lease.TooWide, summary: The range is too wide.}
  - {name: pool.lease.TooLow, summary: The range is too low.}
  - {name: pool.lease.TooLate, summary: The lease ends too late.}
events:
  - {name: pool.lease.Accepted, fields: []}
commands:
  - name: pool.lease.Open
    input:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: issued_at, type: Timestamp}
      - {name: expires_at, type: Timestamp}
    outcomes:
      - name: too-wide
        when: upper > lower + 5
        error: pool.lease.TooWide
      - name: too-low
        when: upper <= lower - 3
        error: pool.lease.TooLow
      - name: too-late
        when: expires_at >= issued_at + 1h
        error: pool.lease.TooLate
      - name: accepted
        emits: [pool.lease.Accepted]
";

fn documents() -> (Vec<Document>, SourceMap) {
    let mut texts = SourceMap::new();
    texts.insert("pool.yaml".to_owned(), MODEL.to_owned());
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    (vec![(Source::new("pool.yaml"), raw)], texts)
}

#[test]
fn a2_guard_boundary_mutants_edit_the_offset_and_are_scored() {
    let (documents, texts) = documents();
    let original = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| {
        panic!("{error:?}");
    });
    let mutants = mutate::mutants(&documents, &[MutantClass::GuardBoundary]);
    let changes: Vec<&str> = mutants
        .iter()
        .map(|mutant| mutant.change.as_str())
        .collect();
    for expected in [
        "becomes `upper >= {offset: {fact: lower, add: 5}}`",
        "becomes `upper < {offset: {fact: lower, subtract: 3}}`",
        "becomes `upper <= {offset: {fact: lower, subtract: 2}}`",
        "becomes `expires_at > {offset: {fact: issued_at, add: 1h}}`",
        "becomes `expires_at >= {offset: {fact: issued_at, add: 3599s}}`",
    ] {
        assert!(
            changes.iter().any(|change| change.contains(expected)),
            "a mutant {expected}: {changes:#?}"
        );
    }
    let mut killed = Vec::new();
    for mutant in &mutants {
        let mutated = mutate::apply(&documents, &mutant.mutation).expect("applies");
        let ir = mutate::compile(mutated, &texts)
            .unwrap_or_else(|stillborn| panic!("{}: stillborn {stillborn:?}", mutant.id));
        assert_ne!(
            serde_json::to_string(&ir.commands()).unwrap(),
            serde_json::to_string(&original.commands()).unwrap(),
            "{} changes the compiled guard",
            mutant.id
        );
        let entry = mutate::evaluate(&documents, &texts, mutant, || {
            Interpreted::for_model(original.clone())
        })
        .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
        assert!(
            matches!(entry.verdict, Verdict::Killed | Verdict::Survived),
            "{} is scored on its suite, never stillborn: {entry:#?}",
            mutant.id
        );
        if entry.verdict == Verdict::Killed {
            killed.push(mutant.change.clone());
        }
    }
    // A boundary witnessed exactly kills the mutant that moves it: each outward step, and the
    // strict `>` made inclusive. A strictness swap whose refusal is witnessed off the boundary
    // survives, as it does for a literal bound.
    for expected in [
        "`upper > {offset: {fact: lower, add: 5}}` becomes `upper >= {offset: {fact: lower, add: 5}}`",
        "becomes `upper <= {offset: {fact: lower, subtract: 2}}`",
        "becomes `expires_at >= {offset: {fact: issued_at, add: 3599s}}`",
    ] {
        assert!(
            killed.iter().any(|change| change.contains(expected)),
            "the mutant that {expected} is killed: {killed:#?}"
        );
    }
    assert!(
        mutants
            .iter()
            .any(|mutant| matches!(mutant.mutation, Mutation::GuardOutward { .. })),
        "{changes:#?}"
    );
}

#[test]
fn a2_an_older_source_keeps_its_mutants() {
    let older = MODEL
        .replace("format: ess/22", "format: ess/21")
        .replace("upper > lower + 5", "upper > 5")
        .replace("upper <= lower - 3", "upper <= -3")
        .replace(
            "expires_at >= issued_at + 1h",
            "expires_at >= \"2030-01-01T00:00:00Z\"",
        );
    let raw = RawSpecFile::parse(&older).unwrap_or_else(|error| panic!("{error}"));
    let documents = vec![(Source::new("pool.yaml"), raw)];
    let changes: Vec<String> = mutate::mutants(&documents, &[MutantClass::GuardBoundary])
        .into_iter()
        .map(|mutant| mutant.change)
        .collect();
    assert!(
        changes
            .iter()
            .any(|change| change.contains("`upper <= -2`")),
        "{changes:#?}"
    );
    assert!(
        changes.iter().all(|change| !change.contains("offset")),
        "{changes:#?}"
    );
}
