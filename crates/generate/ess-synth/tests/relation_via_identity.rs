//! A relation carried by the entity's own identity field, in the synthesised Rust
//! (beyond10x/ess#230).
//!
//! The data struct documents the field that carries a relation. When that field is the identity,
//! the identity's line says so, as a declared field's line would.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The conformance fixture for the same issue, one document per domain.
const MODEL: [(&str, &str); 3] = [
    (
        "system.yaml",
        include_str!(
            "../../../verify/ess-conformance/tests/fixtures/relation-via-identity/system.yaml"
        ),
    ),
    (
        "provisioning.yaml",
        include_str!(
            "../../../verify/ess-conformance/tests/fixtures/relation-via-identity/provisioning.yaml"
        ),
    ),
    (
        "binding.yaml",
        include_str!(
            "../../../verify/ess-conformance/tests/fixtures/relation-via-identity/binding.yaml"
        ),
    ),
];

#[test]
fn the_identity_line_says_what_relation_it_carries() {
    let parsed = MODEL.map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let synthesis = ess_synth::synthesize(&ir).expect("the model has a realizable target");
    let module = synthesis
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .find(|contents| contents.contains("pub struct IdentityData"))
        .unwrap_or_else(|| {
            panic!(
                "no module declares `IdentityData`; the synthesis wrote {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        });
    assert!(
        module.contains(
            "    /// The identity: `user_id` — `demo.provisioning.UserId`.\n    ///\n    /// \
             Carries `user`: `demo.binding.Identity` references one `demo.provisioning.User`.\n"
        ),
        "the identity line names both ends of the relation it carries:\n{module}"
    );
}
