//! Adversary pass 1 on `story:consumer-imports-owner-types` (#162): shapes the unit's own suite
//! does not drive through the structural comparison.

use ess_compiler::refs::{ComponentRef, DeclaredTypeRef, EssSemanticRef};
use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionDiagnostics, CompositionRef,
    CompositionSpec, EssCompositionIr, ServiceImportSpec, ServiceKey, TypeBinding, TypeConformance,
};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const OWNER_SYSTEM: &str = "format: ess/15
system: demo.owner
version: v1

domains:
  - demo.owner.shape
  - demo.owner.hidden
components:
  - component: shape-component
    owns:
      domains: [demo.owner.shape]
  - component: hidden-component
    owns:
      domains: [demo.owner.hidden]
";

const OWNER_SHAPE: &str = "domain: demo.owner.shape

types:
  - name: demo.owner.shape.Mode
    kind: enum
    variants:
      - name: Live
        wire: live
      - Paused
  - name: demo.owner.shape.Note
    kind: struct
    fields:
      - name: text
        type: Optional<String>
        presence: omitted_when_absent
  - name: demo.owner.shape.Wired
    kind: struct
    fields:
      - { name: order_id, type: String, wire: orderId }
  - name: demo.owner.shape.Code
    kind: newtype
    of: String
  - name: demo.owner.shape.Choice
    kind: union
    tag: kind
    variants:
      first: String
      second: Integer
  - name: demo.owner.shape.Tree
    kind: struct
    fields:
      - { name: label, type: String }
      - { name: children, type: List<demo.owner.shape.Tree> }
  - name: demo.owner.shape.Counts
    kind: struct
    fields:
      - { name: tally, type: \"Map<String, Integer>\" }
";

const OWNER_HIDDEN: &str = "domain: demo.owner.hidden

types:
  - name: demo.owner.hidden.Secret
    kind: struct
    fields:
      - { name: value, type: String }
";

const CONSUMER_SYSTEM: &str = "format: ess/15
system: demo.consumer
version: v1

domains:
  - demo.consumer.view
components:
  - component: view-component
    owns:
      domains: [demo.consumer.view]
";

const CONSUMER_VIEW: &str = "domain: demo.consumer.view

types:
  - name: demo.consumer.view.Mode
    kind: enum
    variants: [Live, Paused]
  - name: demo.consumer.view.Note
    kind: struct
    fields:
      - name: text
        type: Optional<String>
        presence: null_when_absent
  - name: demo.consumer.view.Wired
    kind: struct
    fields:
      - { name: order_id, type: String }
  - name: demo.consumer.view.Code
    kind: newtype
    of: Integer
  - name: demo.consumer.view.Choice
    kind: union
    tag: type
    variants:
      first: String
      second: Integer
  - name: demo.consumer.view.ChoiceShape
    kind: union
    tag: kind
    variants:
      first: String
      second: String
  - name: demo.consumer.view.Tree
    kind: struct
    fields:
      - { name: label, type: String }
      - { name: children, type: List<demo.consumer.view.Tree> }
  - name: demo.consumer.view.Counts
    kind: struct
    fields:
      - { name: tally, type: \"Map<Uuid, Integer>\" }
  - name: demo.consumer.view.NotAStruct
    kind: enum
    variants: [label, children]
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
        .unwrap_or_else(|errors| panic!("adversary model validates:\n{errors}"));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("adversary model resolves:\n{diagnostics}"))
}

struct Models {
    owner: EssIr,
    consumer: EssIr,
}

fn models() -> Models {
    Models {
        owner: model(&[
            ("system.yaml", OWNER_SYSTEM),
            ("domains/shape.yaml", OWNER_SHAPE),
            ("domains/hidden.yaml", OWNER_HIDDEN),
        ]),
        consumer: model(&[
            ("system.yaml", CONSUMER_SYSTEM),
            ("domains/view.yaml", CONSUMER_VIEW),
        ]),
    }
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

fn imports(models: &Models) -> Vec<ServiceImportSpec> {
    vec![
        ServiceImportSpec::of(key("owner"), component("shape-component"), &models.owner),
        ServiceImportSpec::of(
            key("consumer"),
            component("view-component"),
            &models.consumer,
        ),
    ]
}

fn compose(
    models: &Models,
    references: Vec<CompositionRef>,
    conformances: Vec<TypeConformance>,
) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(models),
        references,
        conformances,
    );
    let owner = key("owner");
    let consumer = key("consumer");
    compile(
        &specification,
        [
            CompiledService::new(&owner, &models.owner),
            CompiledService::new(&consumer, &models.consumer),
        ],
    )
}

fn assert_conformance(
    local: &str,
    imported: &str,
) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let models = models();
    compose(
        &models,
        Vec::new(),
        vec![TypeConformance::new(
            TypeBinding::new(key("consumer"), type_name(local)),
            TypeBinding::new(key("owner"), type_name(imported)),
        )],
    )
}

fn assert_drift(local: &str, imported: &str, why: &str) {
    match assert_conformance(local, imported) {
        Ok(_) => panic!("`{local}` asserted to conform to `{imported}` was admitted: {why}"),
        Err(diagnostics) => assert!(
            diagnostics.contains(CompositionCode::TypeConformanceDrift),
            "{why}: {diagnostics}"
        ),
    }
}

#[test]
fn an_enum_whose_variant_wire_spelling_differs_is_drift() {
    // Owner `Live` travels as `live`; the consumer's bare `Live` travels as `Live`. Same variant
    // names, different bytes on the wire: a reader built from the local type rejects every `live`.
    assert_drift(
        "demo.consumer.view.Mode",
        "demo.owner.shape.Mode",
        "enum variant wire spellings differ (`live` vs `Live`)",
    );
}

#[test]
fn an_optional_field_whose_absence_travels_differently_is_drift() {
    // Owner omits the key when absent; the local copy says the key is always sent as `null`.
    assert_drift(
        "demo.consumer.view.Note",
        "demo.owner.shape.Note",
        "field presence differs (omitted_when_absent vs null_when_absent)",
    );
}

#[test]
fn a_struct_field_whose_wire_name_differs_is_drift() {
    assert_drift(
        "demo.consumer.view.Wired",
        "demo.owner.shape.Wired",
        "field wire name differs (`orderId` vs `order_id`)",
    );
}

#[test]
fn a_newtype_over_a_different_representation_is_drift() {
    assert_drift(
        "demo.consumer.view.Code",
        "demo.owner.shape.Code",
        "newtype of Integer vs newtype of String",
    );
}

#[test]
fn a_union_with_another_tag_is_drift() {
    assert_drift(
        "demo.consumer.view.Choice",
        "demo.owner.shape.Choice",
        "union tag `type` vs `kind`",
    );
}

#[test]
fn a_union_variant_with_another_shape_is_drift() {
    assert_drift(
        "demo.consumer.view.ChoiceShape",
        "demo.owner.shape.Choice",
        "union variant `second` is String vs Integer",
    );
}

#[test]
fn a_map_with_another_key_is_drift() {
    assert_drift(
        "demo.consumer.view.Counts",
        "demo.owner.shape.Counts",
        "Map<Uuid, _> vs Map<String, _>",
    );
}

#[test]
fn a_different_kind_is_drift() {
    assert_drift(
        "demo.consumer.view.NotAStruct",
        "demo.owner.shape.Tree",
        "an enum is not a struct",
    );
}

#[test]
fn a_recursive_shape_conforms_and_terminates() {
    let composition = assert_conformance("demo.consumer.view.Tree", "demo.owner.shape.Tree")
        .expect("a recursive copy with the same shape conforms");
    assert_eq!(composition.conformances().len(), 1);
}

#[test]
fn v2_does_not_open_a_type_owned_by_another_component_of_the_same_service() {
    let models = models();
    let diagnostics = compose(
        &models,
        vec![CompositionRef::new(
            key("owner"),
            EssSemanticRef::from(type_name("demo.owner.hidden.Secret")),
        )],
        Vec::new(),
    )
    .expect_err("a type the selected component does not own stays outside it under /2");
    assert!(
        diagnostics.contains(CompositionCode::ReferenceOutsideComponent),
        "{diagnostics}"
    );
}
