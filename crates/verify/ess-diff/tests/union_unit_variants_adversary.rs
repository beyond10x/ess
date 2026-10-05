//! Adversary cases for the diff of a unit variant that gains or loses an `Optional<…>` payload
//! (ess/22, beyond10x/ess#418).
//!
//! The design page (`docs/design/union-unit-variants.md`, "Wire form") keeps the rule that a
//! payload variant whose payload is `Optional<…>` is read without its content member. So
//! `{"kind": "Open"}` is a valid value of `Open: Optional<demo.work.Completion>` as much as of the
//! unit variant `Open:`. Turning `Open:` into `Open: Optional<…>` therefore leaves every old value
//! readable by the new revision — callers and history keep working — and only a reader of the old
//! revision meets a content member it refuses. That is the `expanded` shape, `[C, B, C]`, not
//! "neither revision reads what the other writes".

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, Compatibility};
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

fn optional_open() -> String {
    let edited = MODEL.replace(
        "      Open:\n",
        "      Open: Optional<demo.work.Completion>\n",
    );
    assert_ne!(edited, MODEL, "the edit took");
    edited
}

fn compatibility(before: &str, after: &str) -> [Compatibility; 3] {
    let delta = classified(&ir(before), &ir(after)).expect("one system");
    assert_eq!(delta.changes().len(), 1, "{:#?}", delta.changes());
    let compatibility = &delta.compatibility().expect("classified")[0];
    [
        compatibility.callers(),
        compatibility.readers(),
        compatibility.history(),
    ]
}

#[test]
fn a_unit_variant_gaining_an_optional_payload_keeps_old_values_readable() {
    // Old callers still send `{"kind":"Open"}`, which the new revision reads; stored history holds
    // only that spelling. Only an old reader meets `{"kind":"Open","value":…}`.
    assert_eq!(
        compatibility(MODEL, &optional_open()),
        [
            Compatibility::Compatible,
            Compatibility::Breaking,
            Compatibility::Compatible
        ]
    );
}

#[test]
fn an_optional_payload_variant_becoming_a_unit_variant_keeps_new_values_readable_by_old_readers() {
    // The new revision writes `{"kind":"Open"}`, which an old reader reads as an absent optional
    // payload; old callers and history may hold a content member the new revision refuses.
    assert_eq!(
        compatibility(&optional_open(), MODEL),
        [
            Compatibility::Breaking,
            Compatibility::Compatible,
            Compatibility::Breaking
        ]
    );
}
