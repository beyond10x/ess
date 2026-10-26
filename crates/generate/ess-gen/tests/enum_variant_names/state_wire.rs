//! A lifecycle state's wire name (`ess/32`, beyond10x/ess#639) is the value of `<Entity>.State`
//! in every projection that publishes it, and `x-ess-variants` names the declared state.

use std::collections::BTreeSet;

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::schema::ModelTypes;
use serde_json::{json, Value};

const SOURCE: &str = include_str!("../fixtures/state-wire.yaml");
const STATE: &str = "jobs.queue.Job.State";

fn ir() -> ess_compiler::EssIr {
    let raw = RawSpecFile::parse(SOURCE).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("jobs.yaml", SOURCE);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn a_state_wire_name_is_the_state_enums_wire_value() {
    let ir = ir();
    let spelled = json!(["indexing", "Ready"]);

    let selected = ModelTypes::select(&ir, &BTreeSet::from([STATE.to_owned()]))
        .unwrap_or_else(|errors| panic!("selects: {errors:?}"));
    let definition = &selected.definitions()[STATE];
    assert_eq!(definition["enum"], spelled, "{definition}");
    assert_eq!(
        definition["x-ess-variants"],
        json!({"indexing": "Indexing"}),
        "{definition}"
    );

    let outputs = ess_gen::generate_all(&ir).unwrap_or_else(|error| panic!("{error}"));
    let schema: Value =
        serde_json::from_str(&outputs[&format!("schema/types/{STATE}.schema.json")].contents)
            .expect("a schema is JSON");
    assert_eq!(schema["$defs"][STATE]["enum"], spelled, "{schema}");

    let openapi: Value =
        ess_yaml::from_str(&outputs["openapi/jobs-service.yaml"].contents).expect("YAML");
    assert_eq!(
        openapi["components"]["schemas"][STATE]["enum"], spelled,
        "{}",
        openapi["components"]["schemas"][STATE]
    );

    let asyncapi: Value =
        ess_yaml::from_str(&outputs["asyncapi/jobs-service.yaml"].contents).expect("YAML");
    let key = format!("type.{STATE}");
    assert_eq!(
        asyncapi["components"]["schemas"][&key]["enum"], spelled,
        "{}",
        asyncapi["components"]["schemas"][&key]
    );
}
