//! The compiler's class and family catalogues cover the whole code space and say something.
//!
//! `website/docs/reference/diagnostics.md` is rendered from these catalogues, so a class or a family
//! missing here is a code the reference page does not explain.

use ess_compiler::resolve::codes::{class, family};

#[test]
fn every_class_is_catalogued_once_in_code_order_with_a_meaning_and_a_repair() {
    let numbers: Vec<u16> = class::CATALOGUE.iter().map(|entry| entry.number).collect();
    assert_eq!(
        numbers,
        class::ALL,
        "the catalogue and `class::ALL` disagree"
    );
    let expected: Vec<u16> = (1..=u16::try_from(numbers.len()).unwrap()).collect();
    assert_eq!(numbers, expected, "class numbers run 1, 2, … with no gap");
    for entry in class::CATALOGUE {
        assert!(
            !entry.name.is_empty(),
            "class {:03} has no name",
            entry.number
        );
        assert!(
            !entry.meaning.trim().is_empty(),
            "class {:03}",
            entry.number
        );
        assert!(!entry.repair.trim().is_empty(), "class {:03}", entry.number);
    }
    let names: std::collections::BTreeSet<&str> =
        class::CATALOGUE.iter().map(|entry| entry.name).collect();
    assert_eq!(
        names.len(),
        class::CATALOGUE.len(),
        "two classes share a name"
    );
    assert_eq!(class::CATALOGUE[0].name, "UNDECLARED");
    assert_eq!(class::CATALOGUE[17].name, "UNSET_AT_CREATION");
}

/// Every `pub const NAME: u16 = N;` in `resolve.rs`, with the first line of the documentation
/// block above it.
fn class_constants() -> Vec<(String, u16, String)> {
    let lines: Vec<&str> = include_str!("../src/resolve.rs").lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(rest) = line.trim().strip_prefix("pub const ") else {
            continue;
        };
        let Some((name, value)) = rest.split_once(": u16 = ") else {
            continue;
        };
        let number: u16 = value.trim_end_matches(';').parse().expect("a class number");
        let mut first = index;
        while first > 0 && lines[first - 1].trim().starts_with("///") {
            first -= 1;
        }
        let doc = lines[first]
            .trim()
            .trim_start_matches("///")
            .trim()
            .to_owned();
        found.push((name.to_owned(), number, doc));
    }
    found
}

#[test]
fn each_class_row_names_its_constant_and_restates_its_documentation() {
    let constants = class_constants();
    assert_eq!(constants.len(), class::CATALOGUE.len(), "{constants:?}");
    for (name, number, doc) in constants {
        let row = class::CATALOGUE
            .iter()
            .find(|entry| entry.number == number)
            .unwrap_or_else(|| panic!("class {name} = {number} has no catalogue row"));
        assert_eq!(row.name, name, "class {number:03}");
        assert_eq!(
            row.meaning, doc,
            "class {name}: the row and the documentation differ"
        );
    }
}

#[test]
fn every_family_is_catalogued_once_with_where_it_applies() {
    let names: Vec<&str> = family::CATALOGUE.iter().map(|entry| entry.name).collect();
    assert_eq!(
        names,
        family::ALL,
        "the catalogue and `family::ALL` disagree"
    );
    assert_eq!(names.len(), 12);
    for entry in family::CATALOGUE {
        assert!(!entry.applies_to.trim().is_empty(), "family {}", entry.name);
    }
}
