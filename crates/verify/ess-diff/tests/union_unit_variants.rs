//! A unit variant compared across revisions (ess/22, beyond10x/ess#418,
//! `docs/design/union-unit-variants.md`).
//!
//! Adding a unit variant widens the union and removing one narrows it, exactly as a payload
//! variant does. A unit variant that starts carrying a payload, or a payload variant that stops,
//! changes the bytes of every value of that variant in both directions — the old spelling no
//! longer decodes and the new one did not — so it is breaking wherever the union is used.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::change::{SemanticChange, TypeChange};
use ess_diff::{classified, diff, Compatibility, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("work.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let mut sources = SourceMap::new();
    sources.insert("work.yaml", text);
    compile(&spec, &sources).unwrap()
}

fn edited(from: &str, to: &str) -> String {
    assert!(MODEL.contains(from), "the model does not contain `{from}`");
    MODEL.replace(from, to)
}

/// The one change between two revisions, and its classification.
fn one(before: &str, after: &str) -> (SemanticChange, [Compatibility; 3]) {
    let delta = classified(&ir(before), &ir(after)).expect("one system");
    assert_eq!(delta.changes().len(), 1, "{:#?}", delta.changes());
    let text = delta.to_canonical_json();
    let raw: RawEssDelta = serde_json::from_str(&text).unwrap();
    let read = EssDelta::try_from(raw).unwrap_or_else(|error| panic!("{error:?}\n{text}"));
    assert_eq!(read.changes(), delta.changes(), "the document reads back");
    let compatibility = &delta.compatibility().expect("classified")[0];
    (
        delta.changes()[0].clone(),
        [
            compatibility.callers(),
            compatibility.readers(),
            compatibility.history(),
        ],
    )
}

#[test]
fn an_unchanged_model_with_a_unit_variant_is_an_empty_delta() {
    let delta = diff(&ir(MODEL), &ir(MODEL)).unwrap();
    assert_eq!(delta.changes().len(), 0, "{:?}", delta.changes());
}

#[test]
fn adding_a_unit_variant_widens_the_union() {
    let after = edited("      Open:\n", "      Open:\n      Paused:\n");
    let (change, _) = one(MODEL, &after);
    let SemanticChange::Type { subject, changed } = &change else {
        panic!("{change:?}");
    };
    assert_eq!(subject.to_string(), "demo.work.Status");
    assert_eq!(
        changed,
        &TypeChange::VariantAdded {
            variant: "Paused".to_owned()
        }
    );
    assert_eq!(change.relation(), SemanticRelation::Expanded);
}

#[test]
fn removing_a_unit_variant_narrows_the_union() {
    let before = edited("      Open:\n", "      Open:\n      Paused:\n");
    let (change, _) = one(&before, MODEL);
    let SemanticChange::Type { changed, .. } = &change else {
        panic!("{change:?}");
    };
    assert_eq!(
        changed,
        &TypeChange::VariantRemoved {
            variant: "Paused".to_owned()
        }
    );
    assert_eq!(change.relation(), SemanticRelation::Narrowed);
}

#[test]
fn a_unit_variant_that_gains_a_payload_is_breaking_wherever_the_union_is_used() {
    let after = edited("      Open:\n", "      Open: demo.work.Completion\n");
    let (change, [callers, readers, history]) = one(MODEL, &after);
    let SemanticChange::Type { changed, .. } = &change else {
        panic!("{change:?}");
    };
    assert_eq!(
        changed,
        &TypeChange::VariantTypeChanged {
            variant: "Open".to_owned(),
            before: "unit".to_owned(),
            after: "demo.work.Completion".to_owned(),
        }
    );
    assert_eq!(
        change.describe(),
        "variant `Open` carries `demo.work.Completion`, carried `unit`"
    );
    // `Status` is a command input and an event field: callers send it, readers and history read it.
    assert_eq!(
        [callers, readers, history],
        [Compatibility::Breaking; 3],
        "{change:?}"
    );
}

#[test]
fn a_payload_variant_that_loses_its_payload_is_breaking_too() {
    let before = edited("      Open:\n", "      Open: demo.work.Completion\n");
    let (change, compatibility) = one(&before, MODEL);
    let SemanticChange::Type { changed, .. } = &change else {
        panic!("{change:?}");
    };
    assert_eq!(
        changed,
        &TypeChange::VariantTypeChanged {
            variant: "Open".to_owned(),
            before: "demo.work.Completion".to_owned(),
            after: "unit".to_owned(),
        }
    );
    assert_eq!(compatibility, [Compatibility::Breaking; 3]);
}

#[test]
fn a_payload_variant_whose_payload_changes_stays_unknown() {
    let before = edited("      Open:\n", "      Open: String\n");
    let after = edited("      Open:\n", "      Open: Integer\n");
    let (_, compatibility) = one(&before, &after);
    assert_eq!(compatibility, [Compatibility::Unknown; 3]);
}

/// A payload moving from a value to an object is refused from every caller still sending the old
/// one (`ess-diff/16`); readers and history keep the answer the union's uses give.
#[test]
fn a_payload_variant_whose_payload_moves_wire_form_is_breaking_for_callers() {
    let before = edited("      Open:\n", "      Open: String\n");
    let after = edited("      Open:\n", "      Open: demo.work.Completion\n");
    let (_, compatibility) = one(&before, &after);
    assert_eq!(
        compatibility,
        [
            Compatibility::Breaking,
            Compatibility::Unknown,
            Compatibility::Unknown
        ]
    );
}
