//! Models written before `{related: …}` read through an Optional reference or across two
//! references (ess/22, beyond10x/ess#285) keep their canonical IR and their synthesized suite byte
//! for byte: the IR's `through` is omitted where it is empty, and a related value whose reference
//! is always there gains no absent witness.
//!
//! The digests are SHA-256 and byte length of `EssIr::to_canonical_json()` and of
//! `ConformanceSuite::to_canonical_json()`, read from a build of 26ad39057 (the base of #285); suite digests re-pinned at integration head 637c8be3a after #273.

use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use sha2::{Digest, Sha256};

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn digest(bytes: &str) -> (String, usize) {
    let hex = Sha256::digest(bytes.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a String");
            hex
        });
    (hex, bytes.len())
}

/// Name, model, IR digest and length, suite digest and length.
type Pinned = (
    &'static str,
    &'static str,
    &'static str,
    usize,
    &'static str,
    usize,
);

const LEGACY: [Pinned; 3] = [
    (
        "related-guard-copied-value.yaml",
        include_str!("fixtures/related-guard-copied-value.yaml"),
        "2f88b3a80c3a08f99c5e0dbc84bfd0627f48afdafa9192dbb2967ede5e5441e3",
        12_758,
        "d6367e0a45ecee54932a5f41797276bd7853ee5b9d61d9d1110207c11365b589",
        16_399,
    ),
    (
        "subject-guard-copied-field.yaml",
        include_str!("fixtures/subject-guard-copied-field.yaml"),
        "bbf12fce16b887c3b60dc126b8c0f85136149deb57afd802ca150078934b2f1e",
        20_824,
        "2705e065bf055f8f707c43580aad1e04b456135688feabdf5ce9f80e1b08979d",
        124_591,
    ),
    (
        "related-copied-view-parameter.yaml",
        include_str!("fixtures/related-copied-view-parameter.yaml"),
        "bc5cc79a54ed39b0428d7935fa6803f440e6a5add8eed3abe925697dba5ebf7c",
        15_775,
        "d5464390121dab5ffa0074d72d453060e377390e860153bbb07e1972c25a8059",
        43_592,
    ),
];

#[test]
fn issue_285_legacy_related_value_models_keep_their_ir_and_suite_bytes() {
    let mut moved = Vec::new();
    for (name, text, ir_digest, ir_length, suite_digest, suite_length) in LEGACY {
        let model = ir(text);
        let canonical = model.to_canonical_json();
        assert!(
            !canonical.contains("\"through\""),
            "{name}: a one-hop read carries no hops"
        );
        let suite = ess_conformance::synthesize::synthesize(&model)
            .suite
            .to_canonical_json()
            .expect("the suite serializes");
        let actual = (digest(&canonical), digest(&suite));
        let pinned = (
            (ir_digest.to_owned(), ir_length),
            (suite_digest.to_owned(), suite_length),
        );
        if actual != pinned {
            moved.push(format!("{name}: {actual:?}"));
        }
    }
    assert_eq!(moved, Vec::<String>::new(), "canonical bytes moved");
}
