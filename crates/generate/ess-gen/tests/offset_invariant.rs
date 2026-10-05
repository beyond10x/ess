//! A struct invariant comparing a field with one constant offset of another (`ess/22`,
//! `docs/design/expression-family-source22.md` A2) is published as an annotation and never lowered
//! to a JSON Schema keyword: `high <= low + 10` bounds `high` by a value no keyword can name, so
//! reading it as `maximum: 10` — or as a bound against the text `low + 10` — would publish a rule the
//! model does not state.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use serde_json::Value;

const MODEL: &str = "format: ess/22
system: demo
version: v1
domain: demo.metering
types:
  - name: demo.metering.Window
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
      - {name: opened_at, type: Timestamp}
      - {name: closed_at, type: Timestamp}
    invariants:
      - high <= low + 10
      - closed_at <= opened_at + 24h
      - low >= 0
";

#[test]
fn a2_an_offset_invariant_stays_an_annotation() {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("metering.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("metering.yaml", MODEL);
    let ir = compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let artifacts = run(&JsonSchema, &ir).expect("generates");
    let artifact = &artifacts["schema/types/demo.metering.Window.schema.json"];
    let schema: Value = serde_json::from_str(&artifact.contents).expect("JSON");
    let window = &schema["$defs"]["demo.metering.Window"];
    let high = &window["properties"]["high"];
    for keyword in ["minimum", "maximum", "const"] {
        assert!(
            high.get(keyword).is_none(),
            "`high` gained `{keyword}`: {high:#}"
        );
    }
    assert_eq!(
        window["properties"]["low"]["minimum"],
        serde_json::json!(0),
        "a literal bound beside it is still lowered: {window:#}"
    );
    let invariants = window["x-ess-invariants"].to_string();
    assert!(
        invariants.contains("high <= low + 10")
            && invariants.contains("closed_at <= opened_at + 24h"),
        "{invariants}"
    );
}
