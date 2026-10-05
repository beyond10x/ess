//! The mutation audit and `distinct` (`docs/design/expression-family-source22.md`, `distinct`, final
//! review decision 16): an explicit exclusion, not an operator. A `distinct` leaf has no boundary to
//! move and no operator to flip, and its one altering edit — another key — names a member no source
//! chose, so `guard-boundary` offers no mutant of it. The classes that edit the guard around it
//! still do: `guard-negate` of a guard reading `distinct` edits the resolved predicate, keeps its
//! key kind, compiles, and is killed by the decisive witnesses synthesis sends.

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const MODEL: &str = r"format: ess/22
system: pool
version: v1
domain: pool.files
errors:
  - {name: pool.files.DuplicateTag, summary: A tag repeats.}
  - {name: pool.files.TooMany, summary: Too many tags.}
events:
  - {name: pool.files.Accepted, fields: []}
commands:
  - name: pool.files.Tag
    input:
      - {name: tags, type: List<String>}
      - {name: at, type: List<Timestamp>}
      - {name: limit, type: Integer}
    outcomes:
      - name: duplicate
        when: {any: [{not: {distinct: {in: tags, as: tag}}}, {not: {distinct: {in: at, as: t}}}]}
        error: pool.files.DuplicateTag
      - name: too-many
        when: limit > 5
        error: pool.files.TooMany
      - name: accepted
        emits: [pool.files.Accepted]
";

fn documents() -> (Vec<Document>, SourceMap) {
    let mut texts = SourceMap::new();
    texts.insert("pool.yaml".to_owned(), MODEL.to_owned());
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    (vec![(Source::new("pool.yaml"), raw)], texts)
}

#[test]
fn distinct_offers_no_boundary_mutant_and_its_guard_is_still_negated_and_killed() {
    let (documents, texts) = documents();
    let original = mutate::compile(documents.clone(), &texts).unwrap_or_else(|error| {
        panic!("{error:?}");
    });
    let mutants = mutate::mutants(&documents, MutantClass::ALL);
    let boundary: Vec<&str> = mutants
        .iter()
        .filter(|mutant| mutant.class == MutantClass::GuardBoundary)
        .map(|mutant| mutant.change.as_str())
        .collect();
    assert!(
        boundary.iter().all(|change| !change.contains("distinct")),
        "no boundary mutant edits a distinct leaf: {boundary:#?}"
    );
    assert!(
        boundary.iter().any(|change| change.contains("limit")),
        "the ordering beside it keeps its mutants: {boundary:#?}"
    );
    let reading: Vec<_> = mutants
        .iter()
        .filter(|mutant| mutant.change.contains("distinct"))
        .collect();
    assert!(
        reading
            .iter()
            .any(|mutant| mutant.class == MutantClass::GuardNegate),
        "{:#?}",
        mutants
            .iter()
            .map(|mutant| &mutant.change)
            .collect::<Vec<_>>()
    );
    for mutant in reading {
        let mutated = mutate::apply(&documents, &mutant.mutation).expect("applies");
        let ir = mutate::compile(mutated, &texts)
            .unwrap_or_else(|stillborn| panic!("{}: stillborn {stillborn:?}", mutant.id));
        let compiled = serde_json::to_string(&ir.commands()).unwrap();
        assert!(
            compiled.contains(r#""kind":"string""#) && compiled.contains(r#""kind":"timestamp""#),
            "{}: the edit keeps the resolved key kinds: {compiled}",
            mutant.id
        );
        let entry = mutate::evaluate(&documents, &texts, mutant, || {
            Interpreted::for_model(original.clone())
        })
        .unwrap_or_else(|refusal| panic!("{}: {refusal:?}", mutant.id));
        if mutant.class == MutantClass::GuardNegate {
            assert_eq!(entry.verdict, Verdict::Killed, "{}: {entry:#?}", mutant.id);
        }
    }
}
