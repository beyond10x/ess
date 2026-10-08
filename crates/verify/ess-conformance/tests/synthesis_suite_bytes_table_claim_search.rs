//! Adversary case for `synthesis_suite_bytes_table.rs`: the table must pin a model on which the
//! #464 claim search (`held_state_claims`, `synthesize.rs`) decides a witness.
//!
//! The table walks the YAML files on disk. The model the #464 tests hold the claim search with —
//! `DOOR` of `external_beside_held_guard.rs` with `PUSH` sent as `Rough`, so `shut` claims only
//! `Rough` and `stalled` is witnessed with `Gentle` — is built inside that test, so no table line
//! covers it. Mutating `held_state_claims` to ignore a sibling's input guard (scratch copy,
//! `synthesize.rs:3935`) moves that model's `stalled` scenarios and fails
//! `unclaimed_external_witnesses_keep_their_bytes` and
//! `adversary_464_unclaimed_external_witnesses_keep_their_base_bytes`, and leaves the table green.
//! The story names the #464 search as unit 2's surface and the table as the check that it moved no
//! byte, so this case asks the table to hold that model's whole-suite digest, under any label.
use std::fmt::Write as _;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use sha2::{Digest, Sha256};

const TABLE: &str = include_str!("fixtures/synthesis-suite-bytes-table.tsv");
const HOLDER: &str = include_str!("external_beside_held_guard.rs");

fn digest(bytes: &[u8]) -> String {
    let hex = Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    format!("{hex}:{}", bytes.len())
}

/// `suite=<count> <digest>` of `text` synthesized as one file labelled `label`, as the table
/// writes it.
fn suite_column(label: &str, text: &str) -> String {
    let raw = RawSpecFile::parse(text).unwrap();
    let mut sources = SourceMap::new();
    sources.insert(label.to_owned(), text.to_owned());
    let spec = Specification::assemble(vec![(Source::new(label), raw)]).unwrap();
    let ir = compile(&spec, &sources).unwrap();
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let suite = synthesis.suite.to_canonical_json().unwrap();
    format!(
        "suite={} {}",
        synthesis.suite.scenarios.len(),
        digest(suite.as_bytes())
    )
}

/// The `DOOR` model of `external_beside_held_guard.rs`, as its own text.
fn door() -> String {
    let start = HOLDER.find("const DOOR: &str = r\"").unwrap() + "const DOOR: &str = r\"".len();
    let end = start + HOLDER[start..].find('"').unwrap();
    HOLDER[start..end].to_owned()
}

/// Control: the suite column does not depend on the source label, so a search by column finds a
/// model wherever the table files it.
#[test]
fn the_suite_column_does_not_depend_on_the_label() {
    let path = "crates/edge/ess-cli/tests/fixtures/bounded-accessor.yaml";
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(path),
    )
    .unwrap();
    let pinned = TABLE
        .lines()
        .find(|line| line.starts_with(&format!("{path}\t")))
        .unwrap();
    assert!(pinned.contains(&format!("\t{}\t", suite_column("model.yaml", &text))));
}

#[test]
fn the_table_pins_the_model_the_claim_search_decides_a_witness_on() {
    let door = door().replace("PUSH", "Rough");
    let column = suite_column("model.yaml", &door);
    assert!(
        TABLE
            .lines()
            .any(|line| line.contains(&format!("\t{column}\t"))),
        "no table line pins the #464 door model (`stalled` beside `shut` claiming only `Rough`), \
         {column}: a change to `held_state_claims` moves its bytes and the table stays green"
    );
}
