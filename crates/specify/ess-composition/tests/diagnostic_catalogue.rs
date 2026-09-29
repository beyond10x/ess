//! Every composition diagnostic category is catalogued once, with a meaning and a repair.
//!
//! `website/docs/reference/diagnostics.md` is rendered from [`CompositionCode::CATALOGUE`], so a
//! category missing from it is one the reference page does not explain. The variants are read from
//! the enum's own declaration, so a category added to the enum and not to the catalogue fails here.

use ess_composition::CompositionCode;
use std::collections::BTreeSet;

/// The variant names of `pub enum <name>` as written in `source`: one unit variant per line.
fn declared_variants(source: &str, name: &str) -> BTreeSet<String> {
    let start = source
        .find(&format!("pub enum {name} {{"))
        .unwrap_or_else(|| panic!("`pub enum {name}` is declared"));
    let body = &source[start..];
    let end = body.find("\n}").expect("the enum closes");
    body[..end]
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.starts_with("//") && !line.starts_with('#') && !line.is_empty())
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect()
}

#[test]
fn every_composition_code_is_catalogued_once_with_a_meaning_and_a_repair() {
    let declared = declared_variants(include_str!("../src/lib.rs"), "CompositionCode");
    assert_eq!(declared.len(), 14, "{declared:?}");
    let catalogued: BTreeSet<String> = CompositionCode::CATALOGUE
        .iter()
        .map(|entry| format!("{:?}", entry.key))
        .collect();
    assert_eq!(
        catalogued.len(),
        CompositionCode::CATALOGUE.len(),
        "a category twice"
    );
    assert_eq!(catalogued, declared, "the catalogue and the enum disagree");
    for entry in CompositionCode::CATALOGUE {
        assert_eq!(entry.key.catalogue_entry().key, entry.key);
        assert!(!entry.meaning.trim().is_empty(), "{}", entry.key);
        assert!(!entry.repair.trim().is_empty(), "{}", entry.key);
    }
}
