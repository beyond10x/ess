//! A consumer names an owner's declared types, and asserts a local type conforms to one (#162).

use std::path::{Path, PathBuf};

use ess_compiler::refs::{CommandRef, ComponentRef, DeclaredTypeRef, EssSemanticRef};
use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionDiagnostic, CompositionDiagnostics,
    CompositionRef, CompositionSpec, EssCompositionIr, ServiceImportSpec, ServiceKey, TypeBinding,
    TypeConformance, COMPOSITION_FORMAT, CONFORMANCE_COMPOSITION_FORMAT,
    SUPPORTED_COMPOSITION_FORMATS,
};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const STATUS_VIEW: &str = "demo.owner.agentstatus.StatusView";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn compile_model(base: &Path) -> EssIr {
    let mut found = Vec::new();
    let mut pending = vec![base.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry is readable").path();
            if path.is_dir() {
                pending.push(path);
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
            .strip_prefix(base)
            .expect("fixture source is beneath fixture root")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("fixture source is readable");
        let raw = RawSpecFile::parse(&text).expect("fixture source parses");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("{} validates:\n{errors}", base.display()));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("{} resolves:\n{diagnostics}", base.display()))
}

struct Models {
    owner: EssIr,
    consumer: EssIr,
}

fn models() -> Models {
    let root = fixtures().join("owner-consumer");
    Models {
        owner: compile_model(&root.join("owner")),
        consumer: compile_model(&root.join("consumer")),
    }
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid fixture service key")
}

fn component(value: &str) -> ComponentRef {
    ComponentRef::new(ComponentName::new(value).expect("valid fixture component name"))
}

fn type_name(value: &str) -> DeclaredTypeRef {
    DeclaredTypeRef::new(QualifiedName::new(value).expect("valid type name"))
}

fn declared_type(value: &str) -> EssSemanticRef {
    type_name(value).into()
}

fn command(value: &str) -> EssSemanticRef {
    CommandRef::new(QualifiedName::new(value).expect("valid command name")).into()
}

fn imports(models: &Models) -> Vec<ServiceImportSpec> {
    vec![
        ServiceImportSpec::of(key("owner"), component("status-component"), &models.owner),
        ServiceImportSpec::of(
            key("consumer"),
            component("dashboard-component"),
            &models.consumer,
        ),
    ]
}

fn owner_references() -> Vec<CompositionRef> {
    vec![
        CompositionRef::new(
            key("owner"),
            command("demo.owner.agentstatus.UpdateOwnStatus"),
        ),
        CompositionRef::new(key("owner"), declared_type(STATUS_VIEW)),
    ]
}

fn conformance(local: &str) -> TypeConformance {
    TypeConformance::new(
        TypeBinding::new(key("consumer"), type_name(local)),
        TypeBinding::new(key("owner"), type_name(STATUS_VIEW)),
    )
}

fn compose(
    models: &Models,
    specification: &CompositionSpec,
) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let owner = key("owner");
    let consumer = key("consumer");
    compile(
        specification,
        [
            CompiledService::new(&owner, &models.owner),
            CompiledService::new(&consumer, &models.consumer),
        ],
    )
}

fn drift(local: &str) -> String {
    let models = models();
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![conformance(local)],
    );
    let diagnostics =
        compose(&models, &specification).expect_err("a drifted local type is refused");
    assert_eq!(
        diagnostics
            .as_slice()
            .iter()
            .map(CompositionDiagnostic::code)
            .collect::<Vec<_>>(),
        vec![CompositionCode::TypeConformanceDrift],
        "{diagnostics}"
    );
    let diagnostic = &diagnostics.as_slice()[0];
    assert_eq!(diagnostic.service(), Some(&key("consumer")));
    assert_eq!(
        diagnostic.code().to_string(),
        "type_conformance_drift",
        "the code has one stable spelling"
    );
    for named in [local, STATUS_VIEW] {
        assert!(diagnostic.detail().contains(named), "{diagnostic:?}");
    }
    println!("{diagnostic:?}");
    diagnostic.detail().to_owned()
}

#[test]
fn the_composition_format_admits_both_versions() {
    assert_eq!(SUPPORTED_COMPOSITION_FORMATS, &[1, 2]);
    assert_eq!(COMPOSITION_FORMAT, "ess-composition/1");
    assert_eq!(CONFORMANCE_COMPOSITION_FORMAT, "ess-composition/2");
}

#[test]
fn a_type_the_owner_only_declares_is_referenceable_under_v2() {
    let models = models();
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(&models),
        owner_references(),
        Vec::new(),
    );
    assert_eq!(specification.format(), CONFORMANCE_COMPOSITION_FORMAT);
    let composition =
        compose(&models, &specification).expect("an owner-declared type is referenceable");
    assert_eq!(composition.format(), CONFORMANCE_COMPOSITION_FORMAT);
    assert!(composition.references().contains(&CompositionRef::new(
        key("owner"),
        declared_type(STATUS_VIEW)
    )));

    // The closure a client carries is unchanged: the owner-declared type is referenceable, not
    // part of the generated surface.
    let plan = composition.client_plan();
    let owner_plan = &plan.services()[&key("owner")];
    assert!(!owner_plan.types().contains(&type_name(STATUS_VIEW)));
}

#[test]
fn v1_still_refuses_a_type_the_owner_only_declares() {
    let models = models();
    let specification =
        CompositionSpec::new(key("devcenter"), imports(&models), owner_references());
    let diagnostics = compose(&models, &specification)
        .expect_err("ess-composition/1 keeps its reachable-surface rule");
    assert!(diagnostics.contains(CompositionCode::ReferenceOutsideComponent));
    assert!(
        diagnostics.as_slice()[0]
            .detail()
            .contains(CONFORMANCE_COMPOSITION_FORMAT),
        "the refusal names the format that admits it: {diagnostics}"
    );
}

#[test]
fn v2_leaves_the_client_plan_bytes_of_an_existing_composition_alone() {
    let models = models();
    let references = vec![CompositionRef::new(
        key("owner"),
        command("demo.owner.agentstatus.UpdateOwnStatus"),
    )];
    let first = compose(
        &models,
        &CompositionSpec::new(key("devcenter"), imports(&models), references.clone()),
    )
    .expect("v1 composition compiles");
    let second = compose(
        &models,
        &CompositionSpec::with_conformances(
            key("devcenter"),
            imports(&models),
            references,
            vec![conformance("demo.consumer.dashboard.AgentStatus")],
        ),
    )
    .expect("v2 composition compiles");
    assert_eq!(first.format(), COMPOSITION_FORMAT);
    assert_eq!(
        first.client_plan().to_canonical_json(),
        second.client_plan().to_canonical_json()
    );
    assert_eq!(
        first.client_plan().rust_artifacts(),
        second.client_plan().rust_artifacts()
    );
    assert!(!first.to_canonical_json().contains("conformances"));
}

#[test]
fn a_local_type_that_treats_a_required_field_as_optional_conforms() {
    let models = models();
    let asserted = conformance("demo.consumer.dashboard.AgentStatus");
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![asserted.clone()],
    );
    let composition = compose(&models, &specification).expect("required-to-optional is tolerated");
    assert!(composition.conformances().contains(&asserted));
    let json = composition.to_canonical_json();
    assert!(json.contains("\"conformances\""), "{json}");
    assert!(
        json.contains("demo.consumer.dashboard.AgentStatus"),
        "{json}"
    );
}

#[test]
fn a_field_name_drift_is_refused_naming_both_types_and_the_field() {
    let detail = drift("demo.consumer.dashboard.RenamedStatus");
    assert!(detail.contains("`since`"), "{detail}");
    assert!(detail.contains("`updated_at`"), "{detail}");
}

#[test]
fn a_field_type_drift_is_refused_naming_both_types_and_the_field() {
    let detail = drift("demo.consumer.dashboard.MistypedStatus");
    assert!(detail.contains("`since`"), "{detail}");
    assert!(!detail.contains("`agent_id`"), "{detail}");
}

#[test]
fn a_local_field_required_where_the_owner_leaves_it_optional_is_refused() {
    let detail = drift("demo.consumer.dashboard.RequiredNoteStatus");
    assert!(detail.contains("`note`"), "{detail}");
}

#[test]
fn a_conformance_end_is_resolved_like_any_other_reference() {
    let models = models();
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![
            TypeConformance::new(
                TypeBinding::new(
                    key("consumer"),
                    type_name("demo.consumer.dashboard.Missing"),
                ),
                TypeBinding::new(key("owner"), type_name(STATUS_VIEW)),
            ),
            TypeConformance::new(
                TypeBinding::new(
                    key("consumer"),
                    type_name("demo.consumer.dashboard.AgentStatus"),
                ),
                TypeBinding::new(key("elsewhere"), type_name(STATUS_VIEW)),
            ),
        ],
    );
    let diagnostics = compose(&models, &specification).expect_err("unresolved ends are refused");
    assert!(diagnostics.contains(CompositionCode::UnresolvedSemanticReference));
    assert!(diagnostics.contains(CompositionCode::UnknownReferenceService));
    assert!(!diagnostics.contains(CompositionCode::TypeConformanceDrift));
}

#[test]
fn the_authored_key_is_v2_only_and_closed() {
    let models = models();
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![conformance("demo.consumer.dashboard.AgentStatus")],
    );
    let canonical = specification.to_canonical_json();
    let read = CompositionSpec::from_json(&canonical).expect("v2 reads");
    assert_eq!(read.to_canonical_json(), canonical);
    assert_eq!(read.conformances().len(), 1);
    assert!(canonical.contains("\"conforms_to\""), "{canonical}");

    let open = canonical.replacen("\"conforms_to\": {", "\"conforms_to\": {\n\"extra\": 1,", 1);
    assert!(CompositionSpec::from_json(&open)
        .expect_err("a conformance entry is closed")
        .to_string()
        .contains("unknown field"));

    let as_v1 = canonical.replace(CONFORMANCE_COMPOSITION_FORMAT, COMPOSITION_FORMAT);
    let diagnostics = compose(
        &models,
        &CompositionSpec::from_json(&as_v1).expect("the DTO reads; the compiler checks the marker"),
    )
    .expect_err("ess-composition/1 does not admit conformances");
    assert!(
        diagnostics.contains(CompositionCode::UnsupportedFormat),
        "{diagnostics}"
    );

    let empty_v1 = CompositionSpec::new(key("devcenter"), imports(&models), Vec::new())
        .to_canonical_json()
        .replacen("{\n", "{\n  \"conformances\": [],\n", 1);
    let diagnostics = compose(
        &models,
        &CompositionSpec::from_json(&empty_v1).expect("the DTO reads"),
    )
    .expect_err("even an empty conformances key is a v2 construct");
    assert!(
        diagnostics.contains(CompositionCode::UnsupportedFormat),
        "{diagnostics}"
    );
}

#[test]
fn the_issue_162_composition_reads_from_yaml() {
    let models = models();
    let yaml = format!(
        "format: ess-composition/2\n\
         composition: devcenter\n\
         services:\n\
         \x20 - key: owner\n\
         \x20   system: demo.owner\n\
         \x20   version: v1\n\
         \x20   source_digest: {owner}\n\
         \x20   component: status-component\n\
         \x20 - key: consumer\n\
         \x20   system: demo.consumer\n\
         \x20   version: v1\n\
         \x20   source_digest: {consumer}\n\
         \x20   component: dashboard-component\n\
         references:\n\
         \x20 - service: owner\n\
         \x20   semantic: {{ kind: type, name: {STATUS_VIEW} }}\n\
         conformances:\n\
         \x20 - local: {{ service: consumer, type: demo.consumer.dashboard.AgentStatus }}\n\
         \x20   conforms_to: {{ service: owner, type: {STATUS_VIEW} }}\n",
        owner = models.owner.source_digest(),
        consumer = models.consumer.source_digest(),
    );
    let specification = CompositionSpec::from_yaml(&yaml).expect("the #162 composition reads");
    let composition = compose(&models, &specification).expect("the #162 composition compiles");
    assert_eq!(composition.conformances().len(), 1);
}

#[test]
fn a_drift_inside_a_nested_named_type_names_the_field_that_reaches_it() {
    let detail = drift("demo.consumer.dashboard.WiderStatus");
    assert!(detail.contains("field `availability`"), "{detail}");
    assert!(detail.contains("`Busy`"), "{detail}");
    assert!(
        detail.contains("demo.owner.agentstatus.Availability"),
        "{detail}"
    );
}
