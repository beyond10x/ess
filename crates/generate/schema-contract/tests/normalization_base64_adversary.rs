//! Generation must qualify every pattern in a newly admitted model closure.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::json;

#[test]
fn base64_values_do_not_qualify_unrelated_model_property_name_patterns() {
    let source = r"format: ess/1
system: sample
version: v1
domains: [sample.maps]
domain: sample.maps
types:
  - name: sample.maps.Encoded
    kind: struct
    fields:
      - name: values
        type: Map<Integer, Bytes>
";
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, source.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(source).unwrap())])
            .unwrap();
    let ir = compile(&specification, &sources).unwrap();
    let model =
        ModelTypes::select(&ir, &BTreeSet::from(["sample.maps.Encoded".to_owned()])).unwrap();
    let map = &model.definitions()["sample.maps.Encoded"]["properties"]["values"];
    assert_eq!(map["propertyNames"]["pattern"], r"^-?(0|[1-9][0-9]*)$");
    assert_eq!(
        map["additionalProperties"]["pattern"],
        "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$"
    );
    let root = Root::pin_model(&model, "sample.maps.Encoded").unwrap();
    let recipe = json!({"format":"ess-normalization/3","branches":{"copy":[{
        "input":root,"output":root,"requires":[],
        "value":{"op":"read","scope":"input","path":[]}
    }]}});
    let plan =
        Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap();
    assert_eq!(
        plan.run_json("copy", r#"{"values":{"1":"AB=="}}"#).unwrap(),
        json!({"values":{"1":"AB=="}})
    );
    let Err(refused) = plan.go("adapter", "example.invalid/adapter") else {
        panic!("Go generation admitted an unqualified pattern under model propertyNames");
    };
    assert!(refused.0.iter().any(|finding| {
        finding.rule == "go_schema_pattern"
            && finding.pointer.starts_with("/models/")
            && finding
                .pointer
                .ends_with("/sample.maps.Encoded/properties/values/propertyNames/pattern")
    }));
}
