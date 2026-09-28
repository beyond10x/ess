//! Adversary pass 2 on `story:a-reader-side-conformance-admits-reader-widening` (#191).
//!
//! The rule: under `reader: true` a local type conforms only if it accepts every value the imported
//! type can send, given a consumer reader that ignores keys it does not declare. Cases named
//! `control_*` hold the checker to what it must keep doing; the others probe where it decides
//! otherwise than the rule.

use ess_compiler::refs::{ComponentRef, DeclaredTypeRef};
use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionDiagnostic, CompositionDiagnostics,
    CompositionSpec, EssCompositionIr, ServiceImportSpec, ServiceKey, TypeBinding, TypeConformance,
    CONFORMANCE_COMPOSITION_FORMAT,
};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const OWNER_SYSTEM: &str = "format: ess/15
system: adv2.owner
version: v1

domains:
  - adv2.owner.wire
components:
  - component: wire-component
    owns:
      domains: [adv2.owner.wire]
";

const OWNER_WIRE: &str = "domain: adv2.owner.wire

types:
  - name: adv2.owner.wire.Point
    kind: struct
    fields:
      - { name: x, type: Integer }
  - name: adv2.owner.wire.PointRef
    kind: newtype
    of: adv2.owner.wire.Point
  - name: adv2.owner.wire.PointRefRef
    kind: newtype
    of: adv2.owner.wire.PointRef
  - name: adv2.owner.wire.ChainHolder
    kind: struct
    fields:
      - { name: p, type: adv2.owner.wire.PointRefRef }
  - name: adv2.owner.wire.NullablePointHolder
    kind: struct
    fields:
      - name: p
        type: Optional<adv2.owner.wire.Point>
        presence: null_when_absent
  - name: adv2.owner.wire.IntKeyed
    kind: struct
    fields:
      - { name: m, type: \"Map<Integer, String>\" }
  - name: adv2.owner.wire.Phase
    kind: enum
    variants: [in_call, ringing]
  - name: adv2.owner.wire.EnumHolder
    kind: struct
    fields:
      - { name: s, type: adv2.owner.wire.Phase }
  - name: adv2.owner.wire.Cased
    kind: struct
    fields:
      - { name: label, type: String, wire: Label }
  - name: adv2.owner.wire.Inner
    kind: struct
    fields:
      - { name: code, type: Integer }
  - name: adv2.owner.wire.Outer
    kind: struct
    fields:
      - { name: id, type: String }
      - { name: inner, type: adv2.owner.wire.Inner }
  - name: adv2.owner.wire.Swapped
    kind: struct
    fields:
      - { name: a, type: Integer, wire: k }
      - { name: b, type: String, wire: a }
";

const CONSUMER_SYSTEM: &str = "format: ess/15
system: adv2.consumer
version: v1

domains:
  - adv2.consumer.read
components:
  - component: read-component
    owns:
      domains: [adv2.consumer.read]
";

const CONSUMER_READ: &str = "domain: adv2.consumer.read

types:
  - name: adv2.consumer.read.PAsMap
    kind: struct
    fields:
      - { name: p, type: \"Map<String, Json>\" }
  - name: adv2.consumer.read.PAsNullableMap
    kind: struct
    fields:
      - name: p
        type: \"Optional<Map<String, Json>>\"
        presence: null_when_absent
  - name: adv2.consumer.read.MAsMap
    kind: struct
    fields:
      - { name: m, type: \"Map<String, Json>\" }
  - name: adv2.consumer.read.SAsMap
    kind: struct
    fields:
      - { name: s, type: \"Map<String, Json>\" }
  - name: adv2.consumer.read.LowerLabel
    kind: struct
    fields:
      - { name: label, type: String }
  - name: adv2.consumer.read.LabelAsInteger
    kind: struct
    fields:
      - name: count
        type: Optional<Integer>
        wire: Label
        presence: omitted_when_absent
  - name: adv2.consumer.read.UpperLabel
    kind: struct
    fields:
      - name: shout
        type: Optional<Integer>
        wire: LABEL
        presence: omitted_when_absent
  - name: adv2.consumer.read.FlatOuter
    kind: struct
    fields:
      - { name: id, type: String }
      - name: code
        type: Optional<String>
        presence: omitted_when_absent
  - name: adv2.consumer.read.ReadsA
    kind: struct
    fields:
      - { name: a, type: String }
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
        TypeBinding::new(
            key("consumer"),
            type_name(&format!("adv2.consumer.read.{local}")),
        ),
        TypeBinding::new(
            key("owner"),
            type_name(&format!("adv2.owner.wire.{imported}")),
        ),
    )
}

fn v3(models: &Models, asserted: TypeConformance) -> CompositionSpec {
    CompositionSpec::with_reader_conformances(
        key("devcenter"),
        imports(models),
        Vec::new(),
        vec![asserted],
    )
}

fn under_v3(asserted: TypeConformance) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let models = models();
    compose(&models, &v3(&models, asserted))
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

fn reader(local: &str, imported: &str) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let (l, i) = bindings(local, imported);
    under_v3(TypeConformance::for_reader(l, i))
}

fn reader_admits(local: &str, imported: &str, why: &str) {
    if let Err(diagnostics) = reader(local, imported) {
        panic!("reader `{local}` refused against `{imported}`, but {why}:\n{diagnostics}");
    }
}

fn reader_drifts(local: &str, imported: &str, why: &str) {
    match reader(local, imported) {
        Ok(_) => panic!("reader `{local}` admitted against `{imported}`, but {why}"),
        Err(diagnostics) => assert!(
            diagnostics.contains(CompositionCode::TypeConformanceDrift),
            "{why}: {diagnostics}"
        ),
    }
}

// --- Object-shaped detection ------------------------------------------------------------------

#[test]
fn control_a_struct_through_a_newtype_chain_is_read_as_a_map_of_json() {
    reader_admits(
        "PAsMap",
        "ChainHolder",
        "`PointRefRef` -> `PointRef` -> `Point` is always a JSON object",
    );
}

#[test]
fn control_an_optional_struct_sent_as_null_is_not_read_by_a_required_map() {
    reader_drifts(
        "PAsMap",
        "NullablePointHolder",
        "the producer sends `\"p\": null` (`null_when_absent`), which a `Map<String, Json>` rejects",
    );
}

#[test]
fn control_an_optional_struct_is_read_by_an_optional_map_with_the_same_presence() {
    reader_admits(
        "PAsNullableMap",
        "NullablePointHolder",
        "`Optional<Map<String, Json>>` with `null_when_absent` reads `null` and every object",
    );
}

#[test]
fn control_an_enum_is_not_read_as_a_map() {
    reader_drifts(
        "SAsMap",
        "EnumHolder",
        "an enum travels as a string, which a `Map<String, Json>` rejects",
    );
}

#[test]
fn reader_reads_an_integer_keyed_map_as_a_map_of_json() {
    // `Map<Integer, String>` travels as a JSON object whose keys are integer text
    // (`ess-gen` types.rs: `propertyNames` K-as-text), exactly as `Map<String, _>` does, and every
    // JSON object key is a `String`. `Map<String, Json>` accepts every value it can send; the
    // checker treats only `String`-keyed maps as object-shaped.
    //
    // Pins an over-refusal (fails safe): today this is `type_conformance_drift`. Deferred to
    // `story:reader-conformance-over-refusals`; flip it to `reader_admits` when that story lands.
    reader_drifts(
        "MAsMap",
        "IntKeyed",
        "pins an over-refusal deferred to story:reader-conformance-over-refusals (flip to \
         reader_admits when it lands): `Map<String, Json>` reads every `Map<Integer, String>` \
         value, yet today only `String`-keyed maps are object-shaped",
    );
}

// --- Wire-name matching of fields --------------------------------------------------------------

#[test]
fn control_a_case_variant_wire_name_is_a_different_key() {
    // `Label` is sent; the local `label` travels as `label`, a key the producer never sends.
    reader_drifts(
        "LowerLabel",
        "Cased",
        "the local required `label` (wire `label`) is never sent: the producer's key is `Label`",
    );
}

#[test]
fn control_an_extra_field_with_the_same_wire_name_reads_that_key() {
    reader_drifts(
        "LabelAsInteger",
        "Cased",
        "the local `count` travels as `Label` and reads the producer's `String` as `Integer`",
    );
}

#[test]
fn control_an_extra_field_whose_wire_name_differs_only_in_case_is_never_sent() {
    reader_admits(
        "UpperLabel",
        "Cased",
        "`LABEL` is not `Label`; the optional local key is never sent and never rejects",
    );
}

#[test]
fn control_a_wire_name_one_level_deeper_is_not_a_top_level_key() {
    reader_admits(
        "FlatOuter",
        "Outer",
        "`code` travels inside `inner`, never at the top level, so the optional local `code` is \
         never sent",
    );
}

#[test]
fn reader_matches_a_shared_field_name_by_wire_name_as_it_does_an_extra_field() {
    // The producer sends `{"k": 1, "a": "text"}`: field `a` travels as `k`, field `b` as `a`.
    // The local `a: String` travels as `a` and so reads the producer's `b`, a `String`; the
    // producer's `a` (key `k`) is a field the reader omits. That reader rejects no value. An extra
    // local field is matched by wire name (`structs`, conformance.rs:177), but a local field that
    // shares a producer field's *name* is matched by name first and refused for its wire name.
    //
    // Pins an over-refusal (fails safe): today this is `type_conformance_drift`. Deferred to
    // `story:reader-conformance-over-refusals`; flip it to `reader_admits` when that story lands.
    reader_drifts(
        "ReadsA",
        "Swapped",
        "pins an over-refusal deferred to story:reader-conformance-over-refusals (flip to \
         reader_admits when it lands): the local `a` reads key `a`, which the producer sends as \
         its `String` field `b`, yet today a shared field name is matched by name first",
    );
}

// --- `reader:` as a value ----------------------------------------------------------------------

fn canonical_reader(models: &Models) -> String {
    let (l, i) = bindings("PAsMap", "ChainHolder");
    let canonical = v3(models, TypeConformance::for_reader(l, i)).to_canonical_json();
    assert!(canonical.contains("\"reader\": true"), "{canonical}");
    canonical
}

fn refused_json(text: &str, models: &Models) -> bool {
    match CompositionSpec::from_json(text) {
        Err(_) => true,
        Ok(specification) => compose(models, &specification).is_err(),
    }
}

#[test]
fn control_a_reader_key_that_is_not_a_boolean_is_refused_in_json() {
    let models = models();
    let canonical = canonical_reader(&models);
    for written in ["\"true\"", "1", "\"yes\"", "{}"] {
        let text = canonical.replace("\"reader\": true", &format!("\"reader\": {written}"));
        assert!(
            refused_json(&text, &models),
            "`reader: {written}` was read and compiled:\n{text}"
        );
    }
}

fn yaml_with_reader(models: &Models, format: &str, reader: &str) -> String {
    format!(
        "format: {format}\n\
         composition: devcenter\n\
         services:\n\
         \x20 - key: owner\n\
         \x20   system: adv2.owner\n\
         \x20   version: v1\n\
         \x20   source_digest: {owner}\n\
         \x20   component: wire-component\n\
         \x20 - key: consumer\n\
         \x20   system: adv2.consumer\n\
         \x20   version: v1\n\
         \x20   source_digest: {consumer}\n\
         \x20   component: read-component\n\
         conformances:\n\
         \x20 - local: {{ service: consumer, type: adv2.consumer.read.FlatOuter }}\n\
         \x20   conforms_to: {{ service: owner, type: adv2.owner.wire.Outer }}\n\
         {reader}",
        owner = models.owner.source_digest(),
        consumer = models.consumer.source_digest(),
    )
}

#[test]
fn control_a_reader_key_that_is_not_a_boolean_is_refused_in_yaml() {
    let models = models();
    // The harness is live: an unquoted `true` reads and compiles.
    let good = yaml_with_reader(&models, "ess-composition/3", "\x20   reader: true\n");
    let specification = CompositionSpec::from_yaml(&good).expect("`reader: true` reads");
    compose(&models, &specification).expect("the reader subset compiles");
    for written in ["\"true\"", "'true'", "yes", "1", "on"] {
        let text = yaml_with_reader(
            &models,
            "ess-composition/3",
            &format!("\x20   reader: {written}\n"),
        );
        let refused = match CompositionSpec::from_yaml(&text) {
            Err(_) => true,
            Ok(specification) => compose(&models, &specification).is_err(),
        };
        assert!(
            refused,
            "`reader: {written}` was read and compiled:\n{text}"
        );
    }
}

#[test]
fn control_reader_false_and_null_under_v3_compare_exactly() {
    let models = models();
    for written in [
        "\x20   reader: false\n",
        "\x20   reader: null\n",
        "\x20   reader:\n",
    ] {
        let text = yaml_with_reader(&models, "ess-composition/3", written);
        let specification = CompositionSpec::from_yaml(&text).expect("the /3 document reads");
        assert!(!specification.conformances()[0].reader(), "{written}");
        let diagnostics = compose(&models, &specification)
            .expect_err("a field subset is drift without `reader: true`");
        assert!(
            diagnostics.contains(CompositionCode::TypeConformanceDrift),
            "{written}: {diagnostics}"
        );
    }
}

#[test]
fn control_v2_refuses_every_written_reader_value() {
    let models = models();
    for written in [
        "\x20   reader: false\n",
        "\x20   reader: null\n",
        "\x20   reader:\n",
        "\x20   reader: true\n",
    ] {
        let text = yaml_with_reader(&models, CONFORMANCE_COMPOSITION_FORMAT, written);
        let refused = match CompositionSpec::from_yaml(&text) {
            Err(_) => true,
            Ok(specification) => compose(&models, &specification)
                .is_err_and(|diagnostics| diagnostics.contains(CompositionCode::UnsupportedFormat)),
        };
        assert!(refused, "`/2` admitted {written:?}");
    }
}

// --- `/3` without `reader` is `/2` -------------------------------------------------------------

const LOCALS: &[&str] = &[
    "PAsMap",
    "PAsNullableMap",
    "MAsMap",
    "SAsMap",
    "LowerLabel",
    "LabelAsInteger",
    "UpperLabel",
    "FlatOuter",
    "ReadsA",
];

const IMPORTED: &[&str] = &[
    "Point",
    "PointRef",
    "PointRefRef",
    "ChainHolder",
    "NullablePointHolder",
    "IntKeyed",
    "Phase",
    "EnumHolder",
    "Cased",
    "Inner",
    "Outer",
    "Swapped",
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
fn control_v3_without_reader_decides_every_pair_exactly_as_v2() {
    for local in LOCALS {
        for imported in IMPORTED {
            let (l, i) = bindings(local, imported);
            let two = under_v2(TypeConformance::new(l.clone(), i.clone()));
            let three = under_v3(TypeConformance::new(l, i));
            assert_eq!(
                outcome(&two),
                outcome(&three),
                "{local} against {imported}: /2 and /3-exact diagnose differently"
            );
        }
    }
}
