//! A `Json` newtype projects to an unconstrained JSON Schema (beyond10x/ess#138).

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use serde_json::json;

#[test]
fn issue_138_json_is_the_empty_schema() {
    let text = "format: ess/15\nsystem: demo\nversion: v1\ndomain: demo.msgs\ntypes:\n  - {name: \
                demo.msgs.Body, kind: newtype, of: Json}\n";
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new("msgs.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("msgs.yaml", text);
    let ir = compile(&spec, &sources).expect("compiles");
    let artifacts = run(&JsonSchema, &ir).expect("generates");
    let schema: serde_json::Value =
        serde_json::from_str(&artifacts["schema/types/demo.msgs.Body.schema.json"].contents)
            .expect("JSON");
    let body = &schema["$defs"]["demo.msgs.Body"];
    for keyword in [
        "type",
        "format",
        "pattern",
        "enum",
        "anyOf",
        "oneOf",
        "properties",
    ] {
        assert!(
            body.get(keyword).is_none(),
            "`{keyword}` constrains a Json value: {schema:#}"
        );
    }
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    for instance in [
        json!({"any": ["thing", 1, null]}),
        json!([1, "two"]),
        json!("text"),
        json!(1.5),
        json!(true),
        json!(null),
    ] {
        assert!(
            validator.is_valid(&instance),
            "{instance} against {schema:#}"
        );
    }
}
