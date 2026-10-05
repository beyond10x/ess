//! A conformance assertion between unions with unit variants (ess/22, beyond10x/ess#418): a unit
//! variant is the tag alone on the wire, so it agrees only with another unit variant. A local
//! variant that carries a payload where the owner's carries none, or the reverse, is drift naming
//! the variant.

use ess_compiler::refs::{ComponentRef, DeclaredTypeRef};
use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionDiagnostics, CompositionSpec,
    EssCompositionIr, ServiceImportSpec, ServiceKey, TypeBinding, TypeConformance,
};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn system(name: &str, domain: &str, component: &str) -> String {
    format!(
        "format: ess/22\nsystem: {name}\nversion: v1\n\ndomains:\n  - {domain}\ncomponents:\n  - \
         component: {component}\n    owns:\n      domains: [{domain}]\n"
    )
}

const OWNER: &str = "domain: demo.owner.shape

types:
  - name: demo.owner.shape.Status
    kind: union
    tag: kind
    variants:
      open:
      done: String
";

const CONSUMER: &str = "domain: demo.consumer.view

types:
  - name: demo.consumer.view.Same
    kind: union
    tag: kind
    variants:
      open:
      done: String
  - name: demo.consumer.view.Carries
    kind: union
    tag: kind
    variants:
      open: String
      done: String
  - name: demo.consumer.view.Drops
    kind: union
    tag: kind
    variants:
      open:
      done:
";

fn model(files: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw =
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label} parses: {error}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        parsed.push((Source::new(*label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid service key")
}

fn component(value: &str) -> ComponentRef {
    ComponentRef::new(ComponentName::new(value).expect("valid component name"))
}

fn type_name(value: &str) -> DeclaredTypeRef {
    DeclaredTypeRef::new(QualifiedName::new(value).expect("valid type name"))
}

fn conform(local: &str) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let owner_system = system("demo.owner", "demo.owner.shape", "shape-component");
    let consumer_system = system("demo.consumer", "demo.consumer.view", "view-component");
    let owner = model(&[
        ("system.yaml", owner_system.as_str()),
        ("domains/shape.yaml", OWNER),
    ]);
    let consumer = model(&[
        ("system.yaml", consumer_system.as_str()),
        ("domains/view.yaml", CONSUMER),
    ]);
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        vec![
            ServiceImportSpec::of(key("owner"), component("shape-component"), &owner),
            ServiceImportSpec::of(key("consumer"), component("view-component"), &consumer),
        ],
        Vec::new(),
        vec![TypeConformance::new(
            TypeBinding::new(key("consumer"), type_name(local)),
            TypeBinding::new(key("owner"), type_name("demo.owner.shape.Status")),
        )],
    );
    let (owner_key, consumer_key) = (key("owner"), key("consumer"));
    compile(
        &specification,
        [
            CompiledService::new(&owner_key, &owner),
            CompiledService::new(&consumer_key, &consumer),
        ],
    )
}

#[test]
fn a_union_whose_unit_variants_match_conforms() {
    conform("demo.consumer.view.Same").unwrap_or_else(|diagnostics| panic!("{diagnostics}"));
}

#[test]
fn a_payload_where_the_owner_carries_none_is_drift_naming_the_variant() {
    let diagnostics = conform("demo.consumer.view.Carries").expect_err("drift");
    assert!(
        diagnostics.contains(CompositionCode::TypeConformanceDrift),
        "{diagnostics}"
    );
    let text = diagnostics.to_string();
    assert!(
        text.contains("open")
            && text.contains("a payload where `demo.owner.shape.Status` carries none"),
        "{text}"
    );
}

#[test]
fn no_payload_where_the_owner_carries_one_is_drift_naming_the_variant() {
    let diagnostics = conform("demo.consumer.view.Drops").expect_err("drift");
    assert!(
        diagnostics.contains(CompositionCode::TypeConformanceDrift),
        "{diagnostics}"
    );
    let text = diagnostics.to_string();
    assert!(
        text.contains("done")
            && text.contains("no payload where `demo.owner.shape.Status` carries one"),
        "{text}"
    );
}
