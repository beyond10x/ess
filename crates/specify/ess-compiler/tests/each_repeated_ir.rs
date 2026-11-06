//! An `each:` entry admitting repeated identities (`repeated: last_wins`, ess/35,
//! story:an-each-write-may-take-repeated-identities-in-list-order) in the IR: carried on the
//! entry's `each` as `repeated`, left out where absent, so every model without the key keeps its IR
//! bytes and digest.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use sha2::{Digest, Sha256};

const EACH: &str = include_str!("fixtures/set-each.yaml");
const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n        error: demo.feed.DuplicateDocument\n";

/// The SHA-256 of `set-each.yaml`'s canonical IR before the key existed: what `ess 0.87.0 specify
/// compile --out` writes for it.
const PINNED: &str = "faab006ce2b8ad0be93c117edefd5dab2f4a24e3f61ad1867a5db72577ee53bd";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("feed.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn digest(ir: &EssIr) -> String {
    use std::fmt::Write as _;
    Sha256::digest(ir.to_canonical_json().as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[test]
fn the_key_is_carried_on_the_each() {
    let text = EACH
        .replacen("format: ess/23", "format: ess/35", 1)
        .replacen(DISTINCT, "", 1)
        .replacen(
            "each: {in: input.applied, as: doc}",
            "each: {in: input.applied, as: doc, repeated: last_wins}",
            1,
        );
    let compact = ir(&text).to_compact_json();
    assert_eq!(
        compact.matches(r#""repeated":"last_wins""#).count(),
        1,
        "written once, on the entry's each: {compact}"
    );
}

#[test]
fn an_entry_without_the_key_keeps_its_ir_bytes_and_digest() {
    let without = ir(EACH);
    let canonical = without.to_canonical_json();
    assert!(!canonical.contains("repeated"), "{canonical}");
    assert_eq!(digest(&without), PINNED, "the IR bytes moved");
}
