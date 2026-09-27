//! A `String` newtype's `prefix:` projects to a JSON Schema `pattern` anchored at the start, with
//! every regular-expression metacharacter of the prefix escaped (beyond10x/ess#146).

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;

fn compiled(prefix: &str) -> EssIr {
    let text = format!(
        "format: ess/15\nsystem: demo\nversion: v1\ndomain: demo.msgs\ntypes:\n  - name: \
         demo.msgs.Channel\n    kind: newtype\n    of: String\n    prefix: {prefix:?}\n"
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("msgs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("msgs.yaml", &text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn schema(prefix: &str) -> serde_json::Value {
    let artifacts = run(&JsonSchema, &compiled(prefix)).expect("generates");
    let path = "schema/types/demo.msgs.Channel.schema.json";
    let artifact = artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    });
    serde_json::from_str(&artifact.contents).expect("JSON")
}

#[test]
fn issue_146_a_prefix_is_an_anchored_pattern() {
    let schema = schema("/");
    assert_eq!(
        schema["$defs"]["demo.msgs.Channel"]["pattern"],
        serde_json::json!("^/"),
        "{schema:#}"
    );
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    assert!(validator.is_valid(&serde_json::json!("/general")));
    assert!(validator.is_valid(&serde_json::json!("/")));
    assert!(!validator.is_valid(&serde_json::json!("general")));
    assert!(!validator.is_valid(&serde_json::json!("x/")));
}

#[test]
fn every_metacharacter_of_the_prefix_is_escaped() {
    let prefix = "a.b*c+(d)[e]{f}|g?h^i$j\\k/";
    let schema = schema(prefix);
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    assert!(
        validator.is_valid(&serde_json::json!(format!("{prefix}tail"))),
        "{schema:#}"
    );
    assert!(
        !validator.is_valid(&serde_json::json!("aXb*c+(d)[e]{f}|g?h^i$j\\k/")),
        "`.` is literal: {schema:#}"
    );
    assert!(!validator.is_valid(&serde_json::json!("tail")));
}
