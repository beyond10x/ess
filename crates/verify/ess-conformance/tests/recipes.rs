//! Fixture expansion preserves model ownership and actual response slots.
use ess_compiler::{compile, source::SourceMap, EssIr};
use ess_composition::{CompositionSpec, ServiceImportSpec, ServiceKey};
use ess_conformance::{
    authored::Source,
    models::Models,
    recipes::{Bound, Library},
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source as ModelSource,
};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

fn model() -> EssIr {
    model_with_version("v1")
}

fn model_with_version(version: &str) -> EssIr {
    let files = [
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
    let spec = Specification::assemble(files.map(|(path, text)| {
        let text = text.replace("format: ess/1", "format: ess/4").replace("version: v1", &format!("version: {version}"));
        let text = match path {
            "todo.yaml" => text.replace("    outcomes:", "    response:\n      - { name: details, type: workbench.todo.ListDetails }\n    outcomes:")
                .replace("            details: input.details", "            list_id: {generated: true}\n            details: input.details"),
            "usage.yaml" => text.replace("            details: input.details", "            usage_id: {generated: true}\n            details: input.details"),
            _ => text,
        };
        (ModelSource::new(path), RawSpecFile::parse(&text).unwrap())
    })).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn models(ir: &EssIr) -> Models<'_> {
    let key = ServiceKey::new("todo").unwrap();
    Models::compile(
        &CompositionSpec::new(
            ServiceKey::new("system").unwrap(),
            vec![ServiceImportSpec::of(
                key.clone(),
                "todo-component".parse().unwrap(),
                ir,
            )],
            vec![],
        ),
        &[(key, ir)],
    )
    .unwrap()
}

const LEAF: &str = "type: ess-fixture/1
recipe: list
parameters:
  details: {service: todo, command: workbench.todo.CreateList, field: details}
steps:
  - do: command
    service: todo
    command: workbench.todo.CreateList
    input: {details: {$parameter: details}}
    capture: {created-details: details}
outputs: {details: created-details}
";

const ROOT: &str = "type: ess-fixture/1
recipe: pair
steps:
  - do: use
    recipe: list
    as: first
    with: {details: {title: actual}}
  - do: use
    recipe: list
    as: second
    with: {details: {$capture: first.details}}
outputs: {details: second.details}
";

#[test]
fn nested_fixtures_have_typed_actual_outputs_and_full_expansion_locations() {
    let ir = model();
    let models = models(&ir);
    let library = Library::parse(&[
        Source::new("list.yaml", LEAF),
        Source::new("pair.yaml", ROOT),
    ])
    .unwrap();
    let expanded = library
        .expand(&models, &"pair".parse().unwrap(), BTreeMap::new())
        .unwrap();
    assert_eq!(expanded.commands.len(), 2);
    assert_eq!(expanded.sources.len(), 2);
    let Bound::Capture(captured) = &expanded.commands[1].input["details"] else {
        panic!("the second input must come from the first real response");
    };
    assert_eq!(captured.slot, expanded.commands[0].captures[0].slot);
    assert!(std::ptr::eq(captured.command.model(), &raw const ir));
    assert!(std::ptr::eq(
        captured.field,
        &raw const ir.commands()[&"workbench.todo.CreateList".parse().unwrap()].response[0]
    ));
    assert_ne!(
        expanded.commands[0].captures[0].slot,
        expanded.commands[1].captures[0].slot
    );
    assert_eq!(
        expanded.outputs[&"details".parse().unwrap()].slot,
        expanded.commands[1].captures[0].slot
    );
    assert_eq!(
        expanded.commands[1]
            .locations
            .iter()
            .map(|location| (location.origin.as_str(), location.pointer.as_str()))
            .collect::<Vec<_>>(),
        [("pair.yaml", "/steps/1"), ("list.yaml", "/steps/0"),]
    );
    let replay = library
        .expand(&models, &"pair".parse().unwrap(), BTreeMap::new())
        .unwrap();
    assert_eq!(format!("{expanded:?}"), format!("{replay:?}"));
    let edited = Library::parse(&[
        Source::new("list.yaml", format!("{LEAF}# original bytes matter\n")),
        Source::new("pair.yaml", ROOT),
    ])
    .unwrap()
    .expand(&models, &"pair".parse().unwrap(), BTreeMap::new())
    .unwrap();
    assert_ne!(
        expanded.sources[&"list".parse().unwrap()].digest,
        edited.sources[&"list".parse().unwrap()].digest
    );
    assert_eq!(
        expanded.sources[&"pair".parse().unwrap()],
        edited.sources[&"pair".parse().unwrap()]
    );
}

#[test]
fn setup_only_sources_refuse_cycles_missing_dependencies_duplicates_and_snippets() {
    for text in [
        ROOT.replace("recipe: list", "recipe: pair"),
        ROOT.replace("recipe: list", "recipe: absent"),
        ROOT.replace("do: use", "do: assert"),
        ROOT.replace("ess-fixture/1", "ess-fixture/2"),
        ROOT.replace("{$capture: first.details}", "{$invented: first.details}"),
        ROOT.replace(
            "{$capture: first.details}",
            "{$capture: first.details, title: hidden}",
        ),
        ROOT.replace("outputs:", "javascript: throw new Error()\noutputs:"),
    ] {
        assert!(Library::parse(&[Source::new("pair.yaml", text)]).is_err());
    }
    assert!(
        Library::parse(&[Source::new("one.yaml", LEAF), Source::new("two.yaml", LEAF)]).is_err()
    );
    assert!(Library::parse(&[
        Source::new("same.yaml", LEAF),
        Source::new("same.yaml", ROOT)
    ])
    .is_err());
    let cycle_a = ROOT
        .replace("recipe: pair", "recipe: a")
        .replace("recipe: list", "recipe: b");
    let cycle_b = ROOT
        .replace("recipe: pair", "recipe: b")
        .replace("recipe: list", "recipe: a");
    assert!(Library::parse(&[
        Source::new("a.yaml", cycle_a),
        Source::new("b.yaml", cycle_b)
    ])
    .unwrap_err()
    .contains("cycle"));
}

#[test]
fn unknown_contracts_values_and_unbound_captures_refuse_before_execution() {
    let ir = model();
    let models = models(&ir);
    for text in [
        ROOT.replace("title: actual", "title: false"),
        ROOT.replace("first.details", "future.details"),
        ROOT.replace("as: second", "as: first"),
        ROOT.replace("second.details", "invented"),
        ROOT.replace("with: {details: {title: actual}}", "with: {}"),
    ] {
        let library = Library::parse(&[
            Source::new("list.yaml", LEAF),
            Source::new("pair.yaml", text),
        ])
        .unwrap();
        assert!(library
            .expand(&models, &"pair".parse().unwrap(), BTreeMap::new())
            .is_err());
    }
    for text in [
        LEAF.replace("workbench.todo.CreateList", "workbench.todo.DoesNotExist"),
        LEAF.replace("service: todo", "service: missing"),
        LEAF.replace("field: details", "field: unknown"),
        LEAF.replace("{$parameter: details}", "{$parameter: missing}"),
        LEAF.replace("created-details: details", "created-details: absent"),
        LEAF.replace("input: {details:", "input: {unknown:"),
    ] {
        let library = Library::parse(&[Source::new("list.yaml", text)]).unwrap();
        assert!(library
            .expand(
                &models,
                &"list".parse().unwrap(),
                BTreeMap::from([(
                    "details".parse().unwrap(),
                    Node::Map(BTreeMap::from([("title".into(), Node::from("actual"))]))
                )])
            )
            .is_err());
    }
}

#[test]
fn same_named_type_in_another_component_model_cannot_capture_authority() {
    let first = model();
    let second = model_with_version("v2");
    let todo = ServiceKey::new("todo").unwrap();
    let other = ServiceKey::new("other").unwrap();
    let models = Models::compile(
        &CompositionSpec::new(
            ServiceKey::new("system").unwrap(),
            vec![
                ServiceImportSpec::of(todo.clone(), "todo-component".parse().unwrap(), &first),
                ServiceImportSpec::of(other.clone(), "todo-component".parse().unwrap(), &second),
            ],
            vec![],
        ),
        &[(todo, &first), (other, &second)],
    )
    .unwrap();
    let copy = LEAF
        .replace("recipe: list", "recipe: copy")
        .replace("service: todo", "service: other");
    let root = ROOT.replacen(
        "recipe: list\n    as: second",
        "recipe: copy\n    as: second",
        1,
    );
    let library = Library::parse(&[
        Source::new("list.yaml", LEAF),
        Source::new("copy.yaml", copy),
        Source::new("pair.yaml", root),
    ])
    .unwrap();
    assert!(library
        .expand(&models, &"pair".parse().unwrap(), BTreeMap::new())
        .unwrap_err()
        .contains("expected"));
}

#[test]
fn shared_dependency_graph_is_bounded_without_exponential_walks() {
    let mut sources = vec![Source::new("list.yaml", LEAF)];
    let mut previous = "list".to_owned();
    for index in 0..25 {
        let name = format!("layer-{index}");
        let text = ROOT
            .replace("recipe: pair", &format!("recipe: {name}"))
            .replace("recipe: list", &format!("recipe: {previous}"));
        sources.push(Source::new(format!("{name}.yaml"), text));
        previous = name;
    }
    Library::parse(&sources).unwrap();
    for index in 25..33 {
        let name = format!("layer-{index}");
        let text = ROOT
            .replace("recipe: pair", &format!("recipe: {name}"))
            .replace("recipe: list", &format!("recipe: {previous}"));
        sources.push(Source::new(format!("{name}.yaml"), text));
        previous = name;
    }
    assert!(Library::parse(&sources).unwrap_err().contains("depth"));
}
