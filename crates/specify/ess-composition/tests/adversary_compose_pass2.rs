//! Adversary pass 2 on `story:consumer-imports-owner-types` (#162): the pass-1 corrections (enum
//! variant wire spellings, `Optional` presence) and the comparison walk's bookkeeping.

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

const OWNER_SYSTEM: &str = "format: ess/15
system: demo.owner
version: v1

domains:
  - demo.owner.shape
components:
  - component: shape-component
    owns:
      domains: [demo.owner.shape]
";

const OWNER_SHAPE: &str = "domain: demo.owner.shape

types:
  - name: demo.owner.shape.Inner
    kind: struct
    fields:
      - { name: value, type: String }
  - name: demo.owner.shape.Pair
    kind: struct
    fields:
      - { name: first, type: demo.owner.shape.Inner }
      - { name: second, type: demo.owner.shape.Inner }
  - name: demo.owner.shape.InnerText
    kind: struct
    fields:
      - { name: value, type: String }
  - name: demo.owner.shape.InnerCount
    kind: struct
    fields:
      - { name: value, type: Integer }
  - name: demo.owner.shape.Split
    kind: struct
    fields:
      - { name: text, type: demo.owner.shape.InnerText }
      - { name: count, type: demo.owner.shape.InnerCount }
  - name: demo.owner.shape.Level
    kind: enum
    variants:
      - name: High
        wire: High
      - Low
  - name: demo.owner.shape.Status
    kind: enum
    variants:
      - name: Live
        wire: live
  - name: demo.owner.shape.Required
    kind: struct
    fields:
      - { name: value, type: String }
  - name: demo.owner.shape.Unstated
    kind: struct
    fields:
      - { name: value, type: Optional<String> }
  - name: demo.owner.shape.Bag
    kind: struct
    fields:
      - { name: items, type: List<String> }
  - name: demo.owner.shape.LooseBag
    kind: struct
    fields:
      - { name: items, type: List<Optional<String>> }
  - name: demo.owner.shape.Narrow
    kind: struct
    fields:
      - { name: a, type: String }
  - name: demo.owner.shape.Ping
    kind: struct
    fields:
      - { name: pong, type: Optional<demo.owner.shape.Pong> }
  - name: demo.owner.shape.Pong
    kind: struct
    fields:
      - { name: ping, type: Optional<demo.owner.shape.Ping> }
      - { name: score, type: Integer }
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
  - name: demo.consumer.view.Inner
    kind: struct
    fields:
      - { name: value, type: Integer }
  - name: demo.consumer.view.Pair
    kind: struct
    fields:
      - { name: first, type: demo.consumer.view.Inner }
      - { name: second, type: demo.consumer.view.Inner }
  - name: demo.consumer.view.InnerText
    kind: struct
    fields:
      - { name: value, type: String }
  - name: demo.consumer.view.Split
    kind: struct
    fields:
      - { name: text, type: demo.consumer.view.InnerText }
      - { name: count, type: demo.consumer.view.InnerText }
  - name: demo.consumer.view.Level
    kind: enum
    variants: [High, Low]
  - name: demo.consumer.view.Status
    kind: enum
    variants:
      - name: Active
        wire: live
  - name: demo.consumer.view.Required
    kind: struct
    fields:
      - name: value
        type: Optional<String>
        presence: omitted_when_absent
  - name: demo.consumer.view.Unstated
    kind: struct
    fields:
      - { name: value, type: Optional<String> }
  - name: demo.consumer.view.Bag
    kind: struct
    fields:
      - { name: items, type: List<Optional<String>> }
  - name: demo.consumer.view.LooseBag
    kind: struct
    fields:
      - { name: items, type: List<String> }
  - name: demo.consumer.view.Narrow
    kind: struct
    fields:
      - { name: a, type: String }
      - { name: b, type: Optional<String> }
  - name: demo.consumer.view.Ping
    kind: struct
    fields:
      - { name: pong, type: Optional<demo.consumer.view.Pong> }
  - name: demo.consumer.view.Pong
    kind: struct
    fields:
      - { name: ping, type: Optional<demo.consumer.view.Ping> }
      - { name: score, type: String }
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

fn conform(local: &str, imported: &str) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let models = models();
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        vec![
            ServiceImportSpec::of(key("owner"), component("shape-component"), &models.owner),
            ServiceImportSpec::of(
                key("consumer"),
                component("view-component"),
                &models.consumer,
            ),
        ],
        Vec::new(),
        vec![TypeConformance::new(
            TypeBinding::new(key("consumer"), type_name(local)),
            TypeBinding::new(key("owner"), type_name(imported)),
        )],
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

/// The one drift diagnostic's detail.
fn drift(local: &str, imported: &str, why: &str) -> String {
    match conform(local, imported) {
        Ok(_) => panic!("`{local}` asserted to conform to `{imported}` was admitted: {why}"),
        Err(diagnostics) => {
            assert!(
                diagnostics.contains(CompositionCode::TypeConformanceDrift),
                "{why}: {diagnostics}"
            );
            diagnostics.as_slice()[0].detail().to_owned()
        }
    }
}

fn conforms(local: &str, imported: &str, why: &str) {
    if let Err(diagnostics) = conform(local, imported) {
        panic!("`{local}` should conform to `{imported}` ({why}): {diagnostics}");
    }
}

#[test]
fn a_drift_in_a_shared_nested_type_names_every_field_path_that_reaches_it() {
    // The design note promises "every differing field path". `first.value` and `second.value`
    // both differ (Integer where the owner has String); the walk's visited set is a memo, not a
    // stack, so the second path to the same pair is never reported.
    let detail = drift(
        "demo.consumer.view.Pair",
        "demo.owner.shape.Pair",
        "`value` is Integer where the owner has String, under both fields",
    );
    assert!(detail.contains("field `first.value`"), "{detail}");
    assert!(detail.contains("field `second.value`"), "{detail}");
}

#[test]
fn one_local_type_against_two_owner_types_is_compared_against_each() {
    // Mutant guard: a visited set keyed on the local name alone would take `count` as conforming
    // because `InnerText` was already compared under `text`.
    let detail = drift(
        "demo.consumer.view.Split",
        "demo.owner.shape.Split",
        "`count.value` is String where the owner's `InnerCount.value` is Integer",
    );
    assert!(detail.contains("field `count.value`"), "{detail}");
    assert!(!detail.contains("field `text"), "{detail}");
}

#[test]
fn an_explicit_wire_equal_to_the_variant_name_conforms_to_a_bare_variant() {
    conforms(
        "demo.consumer.view.Level",
        "demo.owner.shape.Level",
        "`High` travels as `High` on both sides",
    );
}

#[test]
fn an_enum_variant_renamed_with_the_same_wire_spelling_is_drift() {
    // Mutant guard: comparing wire spellings alone would admit this.
    drift(
        "demo.consumer.view.Status",
        "demo.owner.shape.Status",
        "variant `Active` where the owner has `Live`, both travelling as `live`",
    );
}

#[test]
fn a_local_optional_with_a_declared_presence_conforms_to_a_required_field() {
    // The tolerance: the owner always sends the key with a value, so how the local copy spells
    // absence never matters. Presence is compared only where both ends are Optional.
    conforms(
        "demo.consumer.view.Required",
        "demo.owner.shape.Required",
        "Optional<String> (omitted_when_absent) where the owner has String",
    );
}

#[test]
fn two_optional_fields_with_no_declared_presence_conform() {
    conforms(
        "demo.consumer.view.Unstated",
        "demo.owner.shape.Unstated",
        "the same Optional<String>, neither declaring presence",
    );
}

#[test]
fn the_tolerance_holds_inside_a_list_and_its_reverse_is_drift() {
    conforms(
        "demo.consumer.view.Bag",
        "demo.owner.shape.Bag",
        "List<Optional<String>> where the owner has List<String>",
    );
    drift(
        "demo.consumer.view.LooseBag",
        "demo.owner.shape.LooseBag",
        "List<String> where the owner has List<Optional<String>>",
    );
}

#[test]
fn an_extra_optional_local_field_is_drift() {
    let detail = drift(
        "demo.consumer.view.Narrow",
        "demo.owner.shape.Narrow",
        "field `b` exists only in the local copy",
    );
    assert!(detail.contains("field `b`"), "{detail}");
}

#[test]
fn a_drift_inside_a_mutual_recursion_is_found() {
    let detail = drift(
        "demo.consumer.view.Ping",
        "demo.owner.shape.Ping",
        "`Pong.score` is String where the owner has Integer, inside a Ping/Pong cycle",
    );
    assert!(detail.contains("field `pong.score`"), "{detail}");
}
