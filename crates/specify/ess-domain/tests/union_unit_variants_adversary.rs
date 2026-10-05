//! Adversary cases for the source form of a unit variant (ess/22, beyond10x/ess#418).
//!
//! The claim under test: below ess/22 a unit variant is "one refusal naming ess/22". The
//! implementor's case checks that the refusal is present; these check that it is the only one, so
//! an author is not handed a second, unrelated diagnostic for the same line.

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/union-unit-variants.yaml");

fn errors(body: &str) -> Vec<String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).unwrap_or_else(|e| panic!("{e}"));
    match ess_domain::Specification::assemble([(ess_domain::system::Source::new("work.yaml"), raw)])
    {
        Ok(_) => Vec::new(),
        Err(errors) => errors.into_iter().map(|error| error.to_string()).collect(),
    }
}

#[test]
fn below_ess_22_a_unit_variant_is_exactly_one_refusal() {
    for format in ["ess/21", "ess/20"] {
        let older = MODEL.replace("format: ess/22", &format!("format: {format}"));
        let found = errors(&older);
        assert_eq!(found.len(), 1, "{format}: {found:#?}");
        assert!(found[0].contains("ess/22"), "{format}: {found:#?}");
    }
}

#[test]
fn below_ess_22_two_unit_variants_are_two_refusals_each_at_its_label() {
    let older = MODEL
        .replace("format: ess/22", "format: ess/21")
        .replace("      Open:\n", "      Open:\n      Paused: ~\n");
    let found = errors(&older);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(
        found.iter().any(|e| e.contains("variants.Open")),
        "{found:#?}"
    );
    assert!(
        found.iter().any(|e| e.contains("variants.Paused")),
        "{found:#?}"
    );
}
