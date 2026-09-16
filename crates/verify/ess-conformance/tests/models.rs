//! Cross-service lookup retains original type ownership and exact component pins.
use ess_compiler::{compile, source::SourceMap, EssIr};
use ess_composition::{CompositionSpec, ServiceImportSpec, ServiceKey};
use ess_conformance::models::Models;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

fn model(system: &str) -> EssIr {
    model_with_title(system, "String")
}

fn model_with_title(system: &str, primitive: &str) -> EssIr {
    let inputs = [
        (
            "system.yaml",
            include_str!(
                "../../../specify/ess-composition/tests/fixtures/two-components/system.yaml"
            ),
        ),
        (
            "components.yaml",
            include_str!(
                "../../../specify/ess-composition/tests/fixtures/two-components/components.yaml"
            ),
        ),
        (
            "todo.yaml",
            include_str!(
                "../../../specify/ess-composition/tests/fixtures/two-components/domains/todo.yaml"
            ),
        ),
        (
            "usage.yaml",
            include_str!(
                "../../../specify/ess-composition/tests/fixtures/two-components/domains/usage.yaml"
            ),
        ),
    ];
    let specification = Specification::assemble(inputs.map(|(path, text)| {
        (
            Source::new(path),
            RawSpecFile::parse(
                &text
                    .replace("workbench", system)
                    .replace("of: String", &format!("of: {primitive}")),
            )
            .unwrap(),
        )
    }))
    .unwrap();
    compile(&specification, &SourceMap::new()).unwrap()
}

#[test]
fn declarations_and_nested_types_keep_the_original_owner() {
    let first = model("first");
    let second = model("second");
    let first_key = ServiceKey::new("first").unwrap();
    let second_key = ServiceKey::new("second").unwrap();
    let spec = CompositionSpec::new(
        ServiceKey::new("system").unwrap(),
        vec![
            ServiceImportSpec::of(first_key.clone(), "todo-component".parse().unwrap(), &first),
            ServiceImportSpec::of(
                second_key.clone(),
                "todo-component".parse().unwrap(),
                &second,
            ),
        ],
        vec![],
    );
    let models = Models::compile(
        &spec,
        &[(first_key.clone(), &first), (second_key.clone(), &second)],
    )
    .unwrap();
    let operation = models
        .command(&first_key, &"first.todo.CreateList".parse().unwrap())
        .unwrap();
    assert!(std::ptr::eq(operation.model(), &raw const first));
    assert!(std::ptr::eq(
        operation.declaration(),
        &raw const first.commands()[&"first.todo.CreateList".parse().unwrap()]
    ));
    let types = operation.types();
    for name in ["first.todo.ListDetails", "first.todo.Title"] {
        let reference = name.parse().unwrap();
        assert!(std::ptr::eq(
            types[&reference],
            &raw const first.types()[&name.parse().unwrap()]
        ));
    }
    assert!(models
        .command(&first_key, &"second.todo.CreateList".parse().unwrap())
        .is_none());
    assert!(models
        .command(&first_key, &"first.usage.RecordUsage".parse().unwrap())
        .is_none());
    assert!(models
        .entity(&first_key, &"first.usage.Usage".parse().unwrap())
        .is_none());
    let entity = models
        .entity(&first_key, &"first.todo.TodoList".parse().unwrap())
        .unwrap();
    assert!(entity
        .types()
        .contains_key(&"first.todo.ListId".parse().unwrap()));
    assert!(models
        .view(&second_key, &"second.todo.ListById".parse().unwrap())
        .unwrap()
        .types()
        .contains_key(&"second.todo.Title".parse().unwrap()));
    assert!(models
        .event(&second_key, &"second.todo.ListCreated".parse().unwrap())
        .unwrap()
        .types()
        .contains_key(&"second.todo.ListDetails".parse().unwrap()));
}

#[test]
fn missing_duplicate_and_wrong_model_bindings_never_create_authority() {
    let original = model("first");
    let wrong = model("wrong");
    let drifted = model_with_title("first", "Boolean");
    let key = ServiceKey::new("service").unwrap();
    let spec = CompositionSpec::new(
        ServiceKey::new("system").unwrap(),
        vec![ServiceImportSpec::of(
            key.clone(),
            "todo-component".parse().unwrap(),
            &original,
        )],
        vec![],
    );
    assert!(Models::compile(&spec, &[]).is_err());
    assert!(Models::compile(&spec, &[(key.clone(), &wrong)]).is_err());
    assert!(Models::compile(&spec, &[(key.clone(), &drifted)]).is_err());
    assert!(Models::compile(&spec, &[(key.clone(), &original), (key, &original)]).is_err());
}
