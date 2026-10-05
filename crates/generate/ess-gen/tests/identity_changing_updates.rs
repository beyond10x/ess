//! An `updates:` whose `sets:` writes the identity re-keys the record (ess/23, beyond10x/ess#429,
//! `docs/design/identity-changing-updates.md`): the documentation projection says so, rather than
//! describing the branch as a change that leaves the record where it was.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::docs::Docs;

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/identity-changing-updates.yaml");

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("vault.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn text(ir: &EssIr) -> String {
    run(&Docs, ir)
        .expect("no two pages claim one path")
        .into_values()
        .map(|artifact| artifact.contents)
        .collect()
}

#[test]
fn the_rename_is_documented_as_a_re_key() {
    let text = text(&ir(MODEL));
    assert!(text.contains("It re-keys a `demo.vault.Secret`"), "{text}");
    assert!(
        text.contains("comes to rest under the identity written to `name`"),
        "{text}"
    );
}

#[test]
fn an_update_of_another_field_is_documented_as_before() {
    let plain = MODEL.replace(
        "        sets: {name: input.new_name}\n",
        "        sets: {value: \"renamed\"}\n",
    );
    let text = text(&ir(&plain));
    assert!(!text.contains("re-keys"), "{text}");
    assert!(
        text.contains("It changes a `demo.vault.Secret` without moving it along its lifecycle."),
        "{text}"
    );
}
