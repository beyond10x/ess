//! Feasibility refusals precede files, and admitted refinements retain source semantics.
use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};

fn plan(schema: &Value) -> (Plan, String) {
    let source = json!({"components":{"schemas":{"Input/~":schema}}});
    let bundle = import(
        &source.to_string(),
        &["Input/~".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let root = Root::pin(&bundle, "Input/~").unwrap();
    let prefix = format!(
        "/bundles/{}{}",
        serde_json::to_value(&root).unwrap()["bundle_digest"]
            .as_str()
            .unwrap(),
        bundle.source_pointer("Input/~")
    );
    let recipe = json!({"format":"ess-normalization/6","branches":{"copy":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
    (Plan::read(&recipe.to_string(), &[bundle]).unwrap(), prefix)
}

#[test]
fn unqualified_constraints_refuse_at_every_nested_source_pointer() {
    for (schema, suffixes) in [
        (
            json!({"type":"array","items":{"type":"array","uniqueItems":true}}),
            vec!["/items/uniqueItems"],
        ),
        (
            json!({"type":"object","properties":{"a/~":{"type":"number","multipleOf":2}},"additionalProperties":{"type":"string","pattern":"^a$"}}),
            vec![
                "/additionalProperties/pattern",
                "/properties/a~1~0/multipleOf",
            ],
        ),
        (
            json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2,"uniqueItems":true}),
            vec!["/uniqueItems"],
        ),
        (
            json!({"anyOf":[{"type":"array","uniqueItems":true},{"type":"null"}]}),
            vec!["/anyOf/0/uniqueItems"],
        ),
        (
            json!({"type":"object","minProperties":1,"maxProperties":2}),
            vec!["/maxProperties", "/minProperties"],
        ),
    ] {
        let (plan, prefix) = plan(&schema);
        let errors = plan.typescript("adapter").unwrap_err().0;
        assert_eq!(
            errors.iter().map(|f| f.pointer.clone()).collect::<Vec<_>>(),
            suffixes
                .iter()
                .map(|s| format!("{prefix}{s}"))
                .collect::<Vec<_>>()
        );
        assert!(errors.iter().all(|f| f.rule == "typescript_schema_profile"));
    }
}

#[test]
fn ref_closures_and_annotation_decoys_preserve_qualified_source_locations() {
    let source = json!({"components":{"schemas":{
        "Input":{"type":"object","properties":{"value":{"$ref":"#/components/schemas/Bad"}}},
        "Bad":{"type":"string","pattern":"^(?=a)a$"},
        "Plain":{"type":"array","uniqueItems":false,"items":{"type":"string"},"default":{"uniqueItems":true},"examples":[{"pattern":"bad"}]}
    }}});
    let bundle = import(
        &source.to_string(),
        &["Input".to_owned(), "Plain".to_owned()]
            .into_iter()
            .collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let make = |name| {
        let root = Root::pin(&bundle, name).unwrap();
        Plan::read(&json!({"format":"ess-normalization/1","branches":{"copy":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}}).to_string(), std::slice::from_ref(&bundle)).unwrap()
    };
    let errors = make("Input").typescript("adapter").unwrap_err().0;
    assert_eq!(errors.len(), 1);
    assert!(errors[0].pointer.ends_with("/Bad/pattern"));
    assert!(make("Plain").typescript("adapter").is_ok());
}

#[test]
fn numeric_and_positional_constraints_are_in_the_fixed_profile() {
    for schema in [
        json!({"type":"number","minimum":9_007_199_254_740_993_u64,"maximum":18_446_744_073_709_551_615_u64}),
        json!({"enum":[-0.0,0,1]}),
        json!({"type":"array","prefixItems":[{"type":"string","minLength":1},{"type":"integer"}],"items":false,"minItems":2,"maxItems":2}),
        json!({"anyOf":[{"const":"yes"},{"type":"null"}]}),
    ] {
        let (plan, _) = plan(&schema);
        assert!(plan.typescript("adapter").is_ok());
    }
}

#[test]
fn source_tuple_alternatives_cannot_mint_a_position_binding() {
    let tuple = json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2});
    let source = json!({"components":{"schemas":{"Input":{"anyOf":[tuple,tuple]},"Output":true}}});
    let bundle = import(
        &source.to_string(),
        &["Input".to_owned(), "Output".to_owned()]
            .into_iter()
            .collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let recipe = json!({"format":"ess-normalization/6","branches":{"copy":[{"input":Root::pin(&bundle,"Input").unwrap(),"output":Root::pin(&bundle,"Output").unwrap(),"requires":[],"value":{"op":"position","value":{"op":"read","scope":"input","path":[]},"index":0}}]}});
    assert!(Plan::read(&recipe.to_string(), &[bundle]).is_err());
}
