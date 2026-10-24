//! A generated enum schema carries the declared name of each variant whose wire spelling differs
//! from it, as `x-ess-variants` (beyond10x/ess#658), so the types-only outputs can name the
//! variant; an enum whose names are its spellings keeps its bytes.

use std::collections::BTreeSet;

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::schema::ModelTypes;
use serde_json::json;

const SOURCE: &str = "format: ess/5
system: probe
version: v1
domain: probe.flow
types:
  - name: probe.flow.Status
    kind: enum
    variants:
      - pending
      - in_review
      - {name: done, wire: DONE}
  - name: probe.flow.Plain
    kind: enum
    variants: [open, shut]
";

fn definition(name: &str) -> serde_json::Value {
    let raw = RawSpecFile::parse(SOURCE).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("flow.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("flow.yaml", SOURCE);
    let ir = compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let selected = ModelTypes::select(&ir, &BTreeSet::from([name.to_owned()]))
        .unwrap_or_else(|errors| panic!("selects: {errors:?}"));
    selected.definitions()[name].clone()
}

#[test]
fn a_variant_named_apart_from_its_spelling_is_annotated() {
    let status = definition("probe.flow.Status");
    assert_eq!(status["enum"], json!(["pending", "in_review", "DONE"]));
    assert_eq!(status["x-ess-variants"], json!({"DONE": "done"}));
}

#[test]
fn an_enum_named_by_its_spellings_has_no_annotation() {
    let plain = definition("probe.flow.Plain");
    assert!(plain.get("x-ess-variants").is_none(), "{plain}");
}
