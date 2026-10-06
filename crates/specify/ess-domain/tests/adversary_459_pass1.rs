//! Adversary pass 1 on W3-1 (beyond10x/ess#459): the held-distinct rule of an `each:` entry.
//!
//! The design note admits an `each:` entry "only where a declared `distinct:` holds that member
//! distinct across the list", either conjoined in the branch's own `when:` or negated in the
//! `when:` of a refusal of the command. A refusal answers only where its whole condition holds, so a
//! negated `distinct:` beside a guard over the stored subject refuses a repeated member only where
//! that subject guard holds too; elsewhere the repeated list reaches the `each:` branch.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const EACH: &str = include_str!("../../ess-compiler/tests/fixtures/set-each.yaml");

const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n";

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("the model parses: {error}"));
    Specification::assemble([(Source::new("feed.yaml"), raw)])
}

fn edited(from: &str, to: &str) -> String {
    assert!(EACH.contains(from), "the fixture holds:\n{from}");
    EACH.replacen(from, to, 1)
}

/// The premise: the fixture validates as the unit shipped it.
#[test]
fn premise_the_fixture_validates() {
    assemble(EACH).unwrap_or_else(|errors| panic!("{errors}"));
}

/// A refusal that refuses a repeated `document_id` only where the source's label is `locked`
/// leaves every other source's repeated list to the `each:` branch, whose result then depends on
/// the order of the elements. The rule the note states is not met, so validate must refuse with
/// `missing_declaration` at the entry's `each`, as it does with no `distinct:` at all.
#[test]
fn distinct_behind_a_subject_guard_does_not_hold_the_member_distinct() {
    let model = edited(
        DISTINCT,
        "      - name: duplicated\n        when_subject: {predicate: label == \"locked\"}\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n",
    );
    match assemble(&model) {
        Ok(_) => panic!(
            "admitted: a repeated `document_id` reaches `ran` wherever the source's label is not \
             `locked`, and no `distinct:` holds it distinct there"
        ),
        Err(errors) => {
            assert!(
                errors.as_slice().iter().any(|error| error.code
                    == ValidationCode::MissingDeclaration
                    && error
                        .location
                        .ends_with("RunSource.outcomes.ran.affects[0].each")),
                "expected missing_declaration at `ran.affects[0].each`, got:\n{errors}"
            );
        }
    }
}
