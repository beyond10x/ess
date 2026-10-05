//! A union variant declared without a payload: a unit variant (source format `ess/22`,
//! beyond10x/ess#418, `docs/design/union-unit-variants.md`).
//!
//! `Open:` and `Open: ~` both declare a variant that carries nothing. From ess/22 it is admitted
//! beside payload variants and on its own; below ess/22 it is refused by name, where an older
//! reader failed it as `invalid type: unit value, expected a string`.

use ess_domain::types::{TypeBody, TypeRef};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/union-unit-variants.yaml");

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("work.yaml"), raw)])
        .map_err(|e| e.to_string())
}

/// The variants of `demo.work.Status` as the model reads them.
fn status_variants(body: &str) -> Vec<(String, Option<TypeRef>)> {
    let spec = spec(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let declared = spec
        .system()
        .types
        .get(&"demo.work.Status".parse().unwrap())
        .expect("the union is declared");
    let TypeBody::Union { variants, .. } = &declared.body else {
        panic!("a union: {:?}", declared.body);
    };
    variants
        .iter()
        .map(|(label, payload)| (label.clone(), payload.clone()))
        .collect()
}

#[test]
fn a_variant_written_with_no_type_is_a_unit_variant_at_ess_22() {
    let variants = status_variants(MODEL);
    assert_eq!(
        variants,
        [
            (
                "Complete".to_owned(),
                Some(TypeRef::Named("demo.work.Completion".parse().unwrap()))
            ),
            ("Open".to_owned(), None),
        ]
    );
}

#[test]
fn a_variant_written_as_an_explicit_null_is_the_same_unit_variant() {
    let tilde = MODEL.replace("      Open:\n", "      Open: ~\n");
    assert_ne!(tilde, MODEL, "the edit took");
    assert_eq!(status_variants(&tilde), status_variants(MODEL));
    let null = MODEL.replace("      Open:\n", "      Open: null\n");
    assert_ne!(null, MODEL, "the edit took");
    assert_eq!(status_variants(&null), status_variants(MODEL));
}

#[test]
fn a_union_of_only_unit_variants_is_admitted() {
    let only = MODEL.replace(
        "      Complete: demo.work.Completion\n",
        "      Complete:\n",
    );
    assert_ne!(only, MODEL, "the edit took");
    assert_eq!(
        status_variants(&only),
        [("Complete".to_owned(), None), ("Open".to_owned(), None)]
    );
}

#[test]
fn below_ess_22_a_unit_variant_is_refused_naming_ess_22() {
    for format in ["ess/21", "ess/4", "ess/1"] {
        let older = MODEL.replace("format: ess/22", &format!("format: {format}"));
        let error = spec(&older)
            .err()
            .unwrap_or_else(|| panic!("{format} must refuse a unit variant"));
        for needle in [
            "types.demo.work.Status.variants.Open",
            "a union variant with no payload requires specification format ess/22",
        ] {
            assert!(
                error.contains(needle),
                "{format}: expected {needle:?} in:\n{error}"
            );
        }
    }
}

#[test]
fn below_ess_22_payload_variants_keep_their_meaning() {
    let older = MODEL
        .replace("format: ess/22", "format: ess/21")
        .replace("      Open:\n", "      Open: String\n");
    assert_eq!(
        status_variants(&older)[1],
        (
            "Open".to_owned(),
            Some(TypeRef::Primitive(ess_domain::types::Primitive::String))
        )
    );
}

#[test]
fn a_union_with_no_variants_is_still_refused() {
    let none = MODEL.replace(
        "    variants:\n      Open:\n      Complete: demo.work.Completion\n",
        "    variants: {}\n",
    );
    assert_ne!(none, MODEL, "the edit took");
    let error = spec(&none).expect_err("no variants");
    assert!(error.contains("declares no variants"), "{error}");
}
