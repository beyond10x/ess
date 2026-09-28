//! Adversary pass 1 on `story:a-reader-side-conformance-admits-reader-widening` (#191).
//!
//! The rule every case holds the checker to: under `reader: true` a local type conforms only if it
//! accepts every value the imported type can send on the wire. Anything that could reject a
//! producer value stays `type_conformance_drift`.

use ess_compiler::refs::{ComponentRef, DeclaredTypeRef};
use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionDiagnostic, CompositionDiagnostics,
    CompositionSpec, EssCompositionIr, ServiceImportSpec, ServiceKey, TypeBinding, TypeConformance,
    CONFORMANCE_COMPOSITION_FORMAT, READER_COMPOSITION_FORMAT,
};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const OWNER_SYSTEM: &str = "format: ess/15
system: adv.owner
version: v1

domains:
  - adv.owner.wire
components:
  - component: wire-component
    owns:
      domains: [adv.owner.wire]
";

const OWNER_WIRE: &str = "domain: adv.owner.wire

types:
  - name: adv.owner.wire.Phase
    kind: enum
    variants: [in_call, ringing]
  - name: adv.owner.wire.Context
    kind: struct
    fields:
      - { name: body, type: Json }
  - name: adv.owner.wire.NestedContext
    kind: struct
    fields:
      - { name: bodies, type: \"Map<String, Json>\" }
  - name: adv.owner.wire.ListContext
    kind: struct
    fields:
      - { name: items, type: List<Json> }
  - name: adv.owner.wire.OptionalContext
    kind: struct
    fields:
      - { name: body, type: Optional<Json> }
  - name: adv.owner.wire.Blob
    kind: newtype
    of: Json
  - name: adv.owner.wire.BlobView
    kind: struct
    fields:
      - { name: blob, type: adv.owner.wire.Blob }
  - name: adv.owner.wire.Tagged
    kind: struct
    fields:
      - { name: status, type: adv.owner.wire.Phase }
      - { name: label, type: String }
  - name: adv.owner.wire.Coded
    kind: struct
    fields:
      - { name: internal, type: Integer, wire: code }
      - { name: label, type: String }
  - name: adv.owner.wire.Choice
    kind: union
    tag: kind
    variants:
      first: adv.owner.wire.Tagged
      second: String
";

const CONSUMER_SYSTEM: &str = "format: ess/15
system: adv.consumer
version: v1

domains:
  - adv.consumer.read
components:
  - component: read-component
    owns:
      domains: [adv.consumer.read]
";

const CONSUMER_READ: &str = "domain: adv.consumer.read

types:
  - name: adv.consumer.read.JsonAsMap
    kind: struct
    fields:
      - { name: body, type: \"Map<String, Json>\" }
  - name: adv.consumer.read.NestedJsonAsMap
    kind: struct
    fields:
      - { name: bodies, type: \"Map<String, Map<String, Json>>\" }
  - name: adv.consumer.read.ListJsonAsMap
    kind: struct
    fields:
      - { name: items, type: \"List<Map<String, Json>>\" }
  - name: adv.consumer.read.OptionalJsonAsMap
    kind: struct
    fields:
      - { name: body, type: \"Optional<Map<String, Json>>\" }
  - name: adv.consumer.read.BlobAsMap
    kind: struct
    fields:
      - { name: blob, type: \"Map<String, Json>\" }
  - name: adv.consumer.read.TaggedSubset
    kind: struct
    fields:
      - { name: label, type: String }
  - name: adv.consumer.read.CollidingStatus
    kind: struct
    fields:
      - { name: label, type: String }
      - name: state
        type: Optional<Integer>
        wire: status
        presence: omitted_when_absent
  - name: adv.consumer.read.CollidingCode
    kind: struct
    fields:
      - { name: label, type: String }
      - name: code
        type: Optional<String>
        presence: omitted_when_absent
  - name: adv.consumer.read.ChoiceReader
    kind: union
    tag: kind
    variants:
      first: adv.consumer.read.CollidingStatus
      second: String
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
            ("domains/wire.yaml", OWNER_WIRE),
        ]),
        consumer: model(&[
            ("system.yaml", CONSUMER_SYSTEM),
            ("domains/read.yaml", CONSUMER_READ),
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
        ServiceImportSpec::of(key("owner"), component("wire-component"), &models.owner),
        ServiceImportSpec::of(
            key("consumer"),
            component("read-component"),
            &models.consumer,
        ),
    ]
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

fn bindings(local: &str, imported: &str) -> (TypeBinding, TypeBinding) {
    (
        TypeBinding::new(key("consumer"), type_name(local)),
        TypeBinding::new(key("owner"), type_name(imported)),
    )
}

fn under_v3(asserted: TypeConformance) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let models = models();
    let specification = CompositionSpec::with_reader_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![asserted],
    );
    compose(&models, &specification)
}

fn under_v2(asserted: TypeConformance) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let models = models();
    let specification = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![asserted],
    );
    compose(&models, &specification)
}

fn reader_drift(local: &str, imported: &str, why: &str) {
    let (local_binding, imported_binding) = bindings(local, imported);
    match under_v3(TypeConformance::for_reader(local_binding, imported_binding)) {
        Ok(_) => panic!(
            "reader `{local}` was admitted against `{imported}`, but it rejects a value the \
             producer sends: {why}"
        ),
        Err(diagnostics) => assert!(
            diagnostics.contains(CompositionCode::TypeConformanceDrift),
            "{why}: {diagnostics}"
        ),
    }
}

// --- Decision 5: `Json` read as `Map<String, Json>` -------------------------------------------

#[test]
fn reader_refuses_json_read_as_a_map_because_json_may_be_an_array_or_a_scalar() {
    reader_drift(
        "adv.consumer.read.JsonAsMap",
        "adv.owner.wire.Context",
        "the producer's `Json` body may be `[1]`, `\"text\"`, `3` or `null`; a `Map<String, Json>` \
         reader rejects each",
    );
}

#[test]
fn reader_refuses_json_read_as_a_map_inside_a_map_value() {
    reader_drift(
        "adv.consumer.read.NestedJsonAsMap",
        "adv.owner.wire.NestedContext",
        "`{\"a\": [1]}` is a `Map<String, Json>` the producer sends; the reader's value type \
         `Map<String, Json>` rejects `[1]`",
    );
}

#[test]
fn reader_refuses_json_read_as_a_map_inside_a_list() {
    reader_drift(
        "adv.consumer.read.ListJsonAsMap",
        "adv.owner.wire.ListContext",
        "`[\"text\"]` is a `List<Json>`; a `List<Map<String, Json>>` reader rejects it",
    );
}

#[test]
fn reader_refuses_optional_json_read_as_an_optional_map() {
    reader_drift(
        "adv.consumer.read.OptionalJsonAsMap",
        "adv.owner.wire.OptionalContext",
        "a present `Optional<Json>` may hold `42`; `Optional<Map<String, Json>>` rejects it",
    );
}

#[test]
fn reader_refuses_a_json_newtype_read_as_a_map() {
    reader_drift(
        "adv.consumer.read.BlobAsMap",
        "adv.owner.wire.BlobView",
        "`Blob` wraps `Json`; through the newtype widening it is read as `Map<String, Json>`, \
         which rejects an array",
    );
}

// --- Decision 6: field subset, where an extra local field reads a producer's wire key ---------

#[test]
fn control_reader_admits_a_plain_field_subset() {
    // Green control: the harness admits what decision 6 admits, so the reds below are the
    // checker's and not the fixture's.
    let (local, imported) = bindings("adv.consumer.read.TaggedSubset", "adv.owner.wire.Tagged");
    under_v3(TypeConformance::for_reader(local, imported))
        .unwrap_or_else(|diagnostics| panic!("a plain subset reads: {diagnostics}"));
}

#[test]
fn reader_refuses_an_extra_local_field_whose_wire_name_is_an_omitted_producer_field() {
    // The producer always sends `{"status": "in_call", "label": …}`. The local type omits the
    // producer field `status` (a subset) and declares its own optional `state` travelling as
    // `status`, typed `Integer`. Matched by field *name*, `state` looks like a field the producer
    // never sends; on the wire it reads the producer's `status` key and rejects every value.
    reader_drift(
        "adv.consumer.read.CollidingStatus",
        "adv.owner.wire.Tagged",
        "the local `state` (wire `status`, Integer) reads the producer's `status` (Phase)",
    );
}

#[test]
fn reader_refuses_an_extra_local_field_named_as_a_producer_wire_rename() {
    // The producer's field `internal` travels as `code` (Integer). The local type omits
    // `internal` and declares `code: Optional<String>`, which travels as `code` too.
    reader_drift(
        "adv.consumer.read.CollidingCode",
        "adv.owner.wire.Coded",
        "the local `code` (String) reads the producer's `internal`, sent as `code` (Integer)",
    );
}

#[test]
fn reader_refuses_a_union_whose_variant_payload_reads_a_producer_key_as_another_type() {
    reader_drift(
        "adv.consumer.read.ChoiceReader",
        "adv.owner.wire.Choice",
        "variant `first`'s payload reads the producer's `status` as Integer",
    );
}

// --- `/3` without `reader` is `/2`, and `/2` keeps its closed key set ------------------------

const LOCALS: &[&str] = &[
    "adv.consumer.read.JsonAsMap",
    "adv.consumer.read.NestedJsonAsMap",
    "adv.consumer.read.ListJsonAsMap",
    "adv.consumer.read.OptionalJsonAsMap",
    "adv.consumer.read.BlobAsMap",
    "adv.consumer.read.TaggedSubset",
    "adv.consumer.read.CollidingStatus",
    "adv.consumer.read.CollidingCode",
    "adv.consumer.read.ChoiceReader",
];

const IMPORTED: &[&str] = &[
    "adv.owner.wire.Phase",
    "adv.owner.wire.Context",
    "adv.owner.wire.NestedContext",
    "adv.owner.wire.ListContext",
    "adv.owner.wire.OptionalContext",
    "adv.owner.wire.Blob",
    "adv.owner.wire.BlobView",
    "adv.owner.wire.Tagged",
    "adv.owner.wire.Coded",
    "adv.owner.wire.Choice",
];

fn outcome(result: &Result<EssCompositionIr, CompositionDiagnostics>) -> Vec<(String, String)> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .as_slice()
            .iter()
            .map(|diagnostic: &CompositionDiagnostic| {
                (
                    diagnostic.code().to_string(),
                    diagnostic.detail().to_owned(),
                )
            })
            .collect(),
    }
}

#[test]
fn v3_without_reader_decides_every_pair_exactly_as_v2() {
    for local in LOCALS {
        for imported in IMPORTED {
            let (l, i) = bindings(local, imported);
            let v2 = under_v2(TypeConformance::new(l.clone(), i.clone()));
            let v3 = under_v3(TypeConformance::new(l, i));
            assert_eq!(
                v2.is_ok(),
                v3.is_ok(),
                "{local} against {imported}: /2 and /3-exact disagree"
            );
            assert_eq!(
                outcome(&v2),
                outcome(&v3),
                "{local} against {imported}: /2 and /3-exact diagnose differently"
            );
        }
    }
}

#[test]
fn v2_refuses_an_authored_null_reader_key() {
    // Released `/2` (0.38.0) denies unknown keys, so it refuses any `reader` key. This build reads
    // `"reader": null` into `None` and the `/2` check (`reader.is_some()`) lets it through, so a
    // `/2` document that 0.38.0 refused now compiles.
    let models = models();
    let (local, imported) = bindings("adv.consumer.read.TaggedSubset", "adv.owner.wire.Tagged");
    let canonical = CompositionSpec::with_reader_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![TypeConformance::for_reader(local, imported)],
    )
    .to_canonical_json();
    assert!(canonical.contains("\"reader\": true"), "{canonical}");
    let as_v2_null = canonical
        .replace(READER_COMPOSITION_FORMAT, CONFORMANCE_COMPOSITION_FORMAT)
        .replace("\"reader\": true", "\"reader\": null");
    let refused = match CompositionSpec::from_json(&as_v2_null) {
        Err(_) => true,
        Ok(specification) => compose(&models, &specification)
            .is_err_and(|diagnostics| diagnostics.contains(CompositionCode::UnsupportedFormat)),
    };
    assert!(
        refused,
        "an `ess-composition/2` document carrying `reader: null` is admitted:\n{as_v2_null}"
    );
}

#[test]
fn v1_refuses_a_reader_conformance() {
    let models = models();
    let (local, imported) = bindings("adv.consumer.read.TaggedSubset", "adv.owner.wire.Tagged");
    let canonical = CompositionSpec::with_reader_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![TypeConformance::for_reader(local, imported)],
    )
    .to_canonical_json()
    .replace(READER_COMPOSITION_FORMAT, "ess-composition/1");
    let refused = match CompositionSpec::from_json(&canonical) {
        Err(_) => true,
        Ok(specification) => compose(&models, &specification)
            .is_err_and(|diagnostics| diagnostics.contains(CompositionCode::UnsupportedFormat)),
    };
    assert!(refused, "ess-composition/1 admitted a reader conformance");
}
