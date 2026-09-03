//! Component selection, transitive closure, determinism, compatibility, and generated-client corpus.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ess_compiler::refs::{
    CommandRef, ComponentRef, DeclaredTypeRef, EssSemanticRef, EventRef, ViewRef,
};
use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionRef, CompositionSpec, ServiceImportSpec,
    ServiceKey, SourceDigest, CLIENT_PLAN_FORMAT, COMPOSITION_FORMAT,
};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/two-components")
}

fn compiled() -> EssIr {
    let base = fixture();
    let mut found = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry is readable").path();
            if path.is_dir() {
                if path.file_name().is_none_or(|name| name != "expected") {
                    pending.push(path);
                }
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                found.push(path);
            }
        }
    }
    found.sort();

    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("fixture source is beneath fixture root")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("fixture source is readable");
        let raw = RawSpecFile::parse(&text).expect("fixture source parses");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("two-component fixture validates:\n{errors}"));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("two-component fixture resolves:\n{diagnostics}"))
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid fixture service key")
}

fn component(value: &str) -> ComponentRef {
    ComponentRef::new(ComponentName::new(value).expect("valid fixture component name"))
}

fn command(value: &str) -> EssSemanticRef {
    CommandRef::new(QualifiedName::new(value).expect("valid command name")).into()
}

fn view(value: &str) -> EssSemanticRef {
    ViewRef::new(QualifiedName::new(value).expect("valid view name")).into()
}

fn event(value: &str) -> EssSemanticRef {
    EventRef::new(QualifiedName::new(value).expect("valid event name")).into()
}

fn declared_type(value: &str) -> EssSemanticRef {
    DeclaredTypeRef::new(QualifiedName::new(value).expect("valid type name")).into()
}

fn spellings<'a, T: ToString + 'a>(values: impl IntoIterator<Item = &'a T>) -> BTreeSet<String> {
    values.into_iter().map(ToString::to_string).collect()
}

#[test]
fn selecting_todo_excludes_usage_and_keeps_the_recursive_contract_closure() {
    let model = compiled();
    let service_key = key("todo");
    let specification = CompositionSpec::new(
        key("devcenter"),
        vec![ServiceImportSpec::of(
            service_key.clone(),
            component("todo-component"),
            &model,
        )],
        vec![
            CompositionRef::new(service_key.clone(), command("workbench.todo.CreateList")),
            CompositionRef::new(service_key.clone(), view("workbench.todo.ListById")),
            CompositionRef::new(service_key.clone(), event("workbench.todo.ListCreated")),
            CompositionRef::new(service_key.clone(), declared_type("workbench.todo.Title")),
        ],
    );
    let composition = compile(&specification, [CompiledService::new(&service_key, &model)])
        .expect("selected Todo component compiles");
    let surface = &composition.services()[&service_key];

    assert_eq!(surface.component().to_string(), "todo-component");
    assert_eq!(
        spellings(surface.commands()),
        BTreeSet::from(["workbench.todo.CreateList".to_owned()])
    );
    assert_eq!(
        spellings(surface.queries()),
        BTreeSet::from(["workbench.todo.ListById".to_owned()])
    );
    assert_eq!(
        spellings(surface.events()),
        BTreeSet::from(["workbench.todo.ListCreated".to_owned()])
    );
    assert_eq!(
        spellings(surface.types()),
        BTreeSet::from([
            "workbench.todo.ListDetails".to_owned(),
            "workbench.todo.ListId".to_owned(),
            "workbench.todo.ListRow".to_owned(),
            "workbench.todo.Title".to_owned(),
        ])
    );
    assert!(surface.errors().is_empty());

    let composition_json = composition.to_canonical_json();
    let client_json = composition.client_plan().to_canonical_json();
    for excluded in [
        "workbench.usage.RecordUsage",
        "workbench.usage.UsageById",
        "workbench.usage.UsageRecorded",
        "workbench.usage.UsageCount",
    ] {
        assert!(!composition_json.contains(excluded), "IR leaked {excluded}");
        assert!(
            !client_json.contains(excluded),
            "client plan leaked {excluded}"
        );
    }
}

#[test]
fn selecting_usage_excludes_todo_even_though_both_share_one_compiled_model() {
    let model = compiled();
    let service_key = key("usage");
    let composition = compile(
        &CompositionSpec::new(
            key("devcenter"),
            vec![ServiceImportSpec::of(
                service_key.clone(),
                component("usage-component"),
                &model,
            )],
            Vec::new(),
        ),
        [CompiledService::new(&service_key, &model)],
    )
    .expect("selected Usage component compiles");
    let surface = &composition.services()[&service_key];

    assert_eq!(
        spellings(surface.commands()),
        BTreeSet::from(["workbench.usage.RecordUsage".to_owned()])
    );
    assert_eq!(
        spellings(surface.queries()),
        BTreeSet::from(["workbench.usage.UsageById".to_owned()])
    );
    assert!(!composition.to_canonical_json().contains("workbench.todo"));
}

#[test]
fn a_reference_that_exists_but_is_outside_the_selected_component_is_refused() {
    let model = compiled();
    let service_key = key("todo");
    let diagnostics = compile(
        &CompositionSpec::new(
            key("devcenter"),
            vec![ServiceImportSpec::of(
                service_key.clone(),
                component("todo-component"),
                &model,
            )],
            vec![CompositionRef::new(
                service_key.clone(),
                command("workbench.usage.RecordUsage"),
            )],
        ),
        [CompiledService::new(&service_key, &model)],
    )
    .expect_err("whole-model resolution must not bypass component selection");

    assert!(diagnostics.contains(CompositionCode::ReferenceOutsideComponent));
    assert_eq!(diagnostics.as_slice()[0].service(), Some(&service_key));
}

fn both_components(model: &EssIr, reverse: bool) -> ess_composition::EssCompositionIr {
    let todo_key = key("todo");
    let usage_key = key("usage");
    let todo_import = ServiceImportSpec::of(todo_key.clone(), component("todo-component"), model);
    let usage_import =
        ServiceImportSpec::of(usage_key.clone(), component("usage-component"), model);
    let todo_reference =
        CompositionRef::new(todo_key.clone(), command("workbench.todo.CreateList"));
    let usage_reference = CompositionRef::new(usage_key.clone(), view("workbench.usage.UsageById"));
    let (imports, references) = if reverse {
        (
            vec![usage_import, todo_import],
            vec![usage_reference, todo_reference],
        )
    } else {
        (
            vec![todo_import, usage_import],
            vec![todo_reference, usage_reference],
        )
    };
    let specification = CompositionSpec::new(key("devcenter"), imports, references);
    let services = if reverse {
        vec![
            CompiledService::new(&usage_key, model),
            CompiledService::new(&todo_key, model),
        ]
    } else {
        vec![
            CompiledService::new(&todo_key, model),
            CompiledService::new(&usage_key, model),
        ]
    };
    compile(&specification, services).expect("both distinct components compile")
}

#[test]
fn canonical_ir_plan_and_generated_clients_ignore_all_input_order() {
    let model = compiled();
    let first = both_components(&model, false);
    let second = both_components(&model, true);

    assert_eq!(first.to_canonical_json(), second.to_canonical_json());
    assert_eq!(
        first.client_plan().to_canonical_json(),
        second.client_plan().to_canonical_json()
    );
    assert_eq!(
        first.client_plan().rust_artifacts(),
        second.client_plan().rust_artifacts()
    );
}

#[test]
fn generated_rust_client_matches_the_committed_corpus_and_compiles() {
    let plan = both_components(&compiled(), false).client_plan();
    assert_eq!(plan.format(), CLIENT_PLAN_FORMAT);
    let artifacts = plan.rust_artifacts();
    let expected_root = fixture().join("expected/rust-client");
    for (path, artifact) in &artifacts {
        let expected = std::fs::read_to_string(expected_root.join(path)).unwrap_or_else(|error| {
            panic!("committed client artifact {path} is readable: {error}")
        });
        assert_eq!(artifact.path(), path);
        assert_eq!(artifact.contents(), expected, "generated drift in {path}");
    }
    assert_eq!(
        artifacts.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["Cargo.toml", "ess-client-plan.json", "src/lib.rs"]
    );
    let generated = artifacts["src/lib.rs"].contents();
    for forbidden in ["realm_id", "tenant_id", "/realms/", "realm_url"] {
        assert!(
            !generated.contains(forbidden),
            "generated operation surface leaked authentication coordinate {forbidden:?}"
        );
    }

    let temp = std::env::temp_dir().join(format!(
        "ess-composition-client-compile-{}",
        std::process::id()
    ));
    if temp.exists() {
        std::fs::remove_dir_all(&temp).expect("owned temporary client directory is removable");
    }
    for artifact in artifacts.values() {
        let path = temp.join(artifact.path());
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("temporary client directory is creatable");
        }
        std::fs::write(path, artifact.contents()).expect("temporary client artifact is writable");
    }
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = std::process::Command::new(rustc)
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg(temp.join("src/lib.rs"))
        .arg("--out-dir")
        .arg(&temp)
        .output()
        .expect("rustc starts");
    assert!(
        output.status.success(),
        "generated client did not compile:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(temp).expect("owned temporary client directory is removable");
}

#[test]
fn exact_component_identity_digest_and_closed_registry_are_enforced() {
    let model = compiled();
    let todo_key = key("todo");
    let usage_key = key("usage");
    let forged = ServiceImportSpec::new(
        todo_key.clone(),
        model.system().clone(),
        *model.version(),
        SourceDigest::new("0".repeat(64)).expect("syntactically valid digest"),
        component("missing-component"),
    );
    let diagnostics = compile(
        &CompositionSpec::new(key("devcenter"), vec![forged], Vec::new()),
        [
            CompiledService::new(&todo_key, &model),
            CompiledService::new(&usage_key, &model),
        ],
    )
    .expect_err("digest, component, and closed registry are checked");

    assert!(diagnostics.contains(CompositionCode::DigestMismatch));
    assert!(diagnostics.contains(CompositionCode::UnknownComponent));
    assert!(diagnostics.contains(CompositionCode::UndeclaredServiceInput));
}

#[test]
fn identical_component_imports_are_duplicates_but_two_components_of_one_model_are_not() {
    let model = compiled();
    let first_key = key("todo-primary");
    let second_key = key("todo-shadow");
    let specification = CompositionSpec::new(
        key("devcenter"),
        vec![
            ServiceImportSpec::of(first_key.clone(), component("todo-component"), &model),
            ServiceImportSpec::of(second_key.clone(), component("todo-component"), &model),
        ],
        Vec::new(),
    );
    let diagnostics = compile(
        &specification,
        [
            CompiledService::new(&first_key, &model),
            CompiledService::new(&second_key, &model),
        ],
    )
    .expect_err("one exact component surface cannot masquerade under two keys");
    assert!(diagnostics.contains(CompositionCode::DuplicateServiceIdentity));

    assert_eq!(both_components(&model, false).services().len(), 2);
}

#[test]
fn v1_is_strict_and_requires_an_explicit_component_selection() {
    let model = compiled();
    let service_key = key("todo");
    let specification = CompositionSpec::new(
        key("devcenter"),
        vec![ServiceImportSpec::of(
            service_key,
            component("todo-component"),
            &model,
        )],
        Vec::new(),
    );
    let canonical = specification.to_canonical_json();
    assert_eq!(
        CompositionSpec::from_json(&canonical)
            .expect("current format reads")
            .to_canonical_json(),
        canonical
    );

    let future = canonical.replacen("{\n", "{\n  \"future_meaning\": true,\n", 1);
    assert!(CompositionSpec::from_json(&future)
        .expect_err("unknown meaning must be refused")
        .to_string()
        .contains("unknown field"));
    let missing_component = canonical.replace(",\n      \"component\": \"todo-component\"", "");
    assert!(CompositionSpec::from_json(&missing_component)
        .expect_err("whole-model imports are not a valid v1 surface")
        .to_string()
        .contains("missing field `component`"));
    assert_eq!(specification.format(), COMPOSITION_FORMAT);
}

#[test]
fn service_keys_and_digests_have_one_canonical_spelling() {
    for invalid in ["", "Todo", "todo/usage", "todo..usage", "todo-"] {
        assert!(ServiceKey::new(invalid).is_err(), "{invalid:?}");
    }
    assert_eq!(key("todo.usage-v1").as_str(), "todo.usage-v1");
    for invalid in ["abc", &"A".repeat(64), &"0".repeat(63)] {
        assert!(SourceDigest::new(invalid).is_err(), "{invalid:?}");
    }
    assert_eq!(SourceDigest::of(&compiled()).as_str().len(), 64);
}
