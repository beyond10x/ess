//! A map keyed by a newtype of `String` projects exactly as a map keyed by `String`
//! (beyond10x/ess#143).
//!
//! The key is resolved to its primitive while the document is read, so the two specifications below
//! differ only in how the key is spelled, and every published schema is byte-identical between
//! them — provenance included, because the resolved models are the same model.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::MapKeyNewtypes;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;

fn model(key: &str) -> String {
    format!(
        "format: ess/14\nsystem: demo\nversion: v1\ndomain: demo.orders\ntypes:\n  - {{name: \
         demo.orders.ItemId, kind: newtype, of: String}}\nevents:\n  - name: demo.orders.Checked\n    \
         fields:\n      - {{name: results, type: \"Map<{key}, Boolean>\"}}\n"
    )
}

fn compiled(text: &str) -> EssIr {
    let raw = MapKeyNewtypes::from_documents([text])
        .scope(|| RawSpecFile::parse(text))
        .unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("orders.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn a_newtype_key_projects_to_the_same_json_schema_as_its_primitive() {
    let newtype = run(&JsonSchema, &compiled(&model("demo.orders.ItemId"))).expect("generates");
    let primitive = run(&JsonSchema, &compiled(&model("String"))).expect("generates");

    let path = "schema/events/demo.orders.Checked.schema.json";
    let event = newtype
        .get(path)
        .unwrap_or_else(|| panic!("no `{path}` among {:?}", newtype.keys().collect::<Vec<_>>()));
    let schema: serde_json::Value = serde_json::from_str(&event.contents).expect("JSON");
    assert_eq!(
        schema["properties"]["results"]["type"],
        serde_json::json!("object"),
        "{schema:#}"
    );

    assert_eq!(
        newtype.keys().collect::<Vec<_>>(),
        primitive.keys().collect::<Vec<_>>()
    );
    for (path, artifact) in &newtype {
        assert_eq!(
            artifact.contents, primitive[path].contents,
            "`{path}` differs between the two spellings of the key"
        );
    }
}
