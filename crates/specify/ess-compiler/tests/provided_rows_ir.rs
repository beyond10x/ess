//! An entity whose rows the deployment provides (`provided: true`, ess/33,
//! story:a-fixture-may-be-a-row-the-deployment-already-holds) in the IR: carried on the entity as
//! `provided`, left out where `false`, so every model without the key keeps its IR bytes.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn model(format: u32, marker: &str) -> String {
    format!(
        "format: ess/{format}
system: linking
version: v1
domain: linking.links
types:
  - {{name: linking.links.AppId, kind: newtype, of: Uuid}}
entities:
  - name: linking.links.App
{marker}    identity: {{name: app_id, type: linking.links.AppId}}
    fields: [{{name: title, type: String}}]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn the_key_is_carried_on_the_entity() {
    let ir = ir(&model(33, "    provided: true\n"));
    let app = &ir.entities()[&"linking.links.App".parse().unwrap()];
    assert!(app.provided);
    assert_eq!(
        ir.to_compact_json().matches("\"provided\":true").count(),
        1,
        "written once, on the marked entity"
    );
}

/// A model without the key, or with `provided: false`, writes no `provided` in its IR; the
/// committed IR fixtures of every other test hold the bytes of models that predate the key.
#[test]
fn a_model_without_the_key_keeps_its_ir_bytes() {
    let without = ir(&model(33, "")).to_canonical_json();
    assert!(!without.contains("provided"), "{without}");
    assert_eq!(
        ir(&model(33, "    provided: false\n")).to_canonical_json(),
        without
    );
}
