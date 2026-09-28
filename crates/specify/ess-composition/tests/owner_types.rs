//! A consumer names an owner's declared types, and asserts a local type conforms to one (#162).

use std::path::{Path, PathBuf};

use ess_compiler::refs::{CommandRef, ComponentRef, DeclaredTypeRef, EssSemanticRef};
use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionDiagnostic, CompositionDiagnostics,
    CompositionRef, CompositionSpec, EssCompositionIr, ServiceImportSpec, ServiceKey, TypeBinding,
    TypeConformance, COMPOSITION_FORMAT, CONFORMANCE_COMPOSITION_FORMAT, READER_COMPOSITION_FORMAT,
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
fn the_composition_format_admits_every_version() {
    assert_eq!(SUPPORTED_COMPOSITION_FORMATS, &[1, 2, 3]);
    assert_eq!(COMPOSITION_FORMAT, "ess-composition/1");
    assert_eq!(CONFORMANCE_COMPOSITION_FORMAT, "ess-composition/2");
    assert_eq!(READER_COMPOSITION_FORMAT, "ess-composition/3");
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

// Reader-side conformance, `ess-composition/3` (#191): `reader: true` admits the widening a
// consumer that only reads the owner's type off the wire may do, and nothing that could reject a
// value the owner sends.

const CALL_ID_VIEW: &str = "demo.owner.agentstatus.CallIdView";
const CALL_STATE_VIEW: &str = "demo.owner.agentstatus.CallStateView";
const CONTEXT_VIEW: &str = "demo.owner.agentstatus.ContextView";
const OBJECT_VIEW: &str = "demo.owner.agentstatus.ObjectView";

fn binding_pair(local: &str, imported: &str) -> (TypeBinding, TypeBinding) {
    (
        TypeBinding::new(key("consumer"), type_name(local)),
        TypeBinding::new(key("owner"), type_name(imported)),
    )
}

fn reader(local: &str, imported: &str) -> TypeConformance {
    let (local, imported) = binding_pair(local, imported);
    TypeConformance::for_reader(local, imported)
}

fn exact(local: &str, imported: &str) -> TypeConformance {
    let (local, imported) = binding_pair(local, imported);
    TypeConformance::new(local, imported)
}

fn under_v3(asserted: TypeConformance) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let models = models();
    let specification = CompositionSpec::with_reader_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![asserted],
    );
    assert_eq!(specification.format(), READER_COMPOSITION_FORMAT);
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
    assert_eq!(specification.format(), CONFORMANCE_COMPOSITION_FORMAT);
    compose(&models, &specification)
}

/// `reader: true` under `/3` admits the pair, and the compiled composition records it.
fn reader_conforms(local: &str, imported: &str) {
    let asserted = reader(local, imported);
    let composition = under_v3(asserted.clone())
        .unwrap_or_else(|diagnostics| panic!("{local} reads {imported}: {diagnostics}"));
    assert_eq!(composition.format(), READER_COMPOSITION_FORMAT);
    assert!(composition.conformances().contains(&asserted));
    let json = composition.to_canonical_json();
    assert!(json.contains("\"reader\": true"), "{json}");
}

/// The single diagnostic is `type_conformance_drift` naming both types; returns its detail.
fn only_drift(result: Result<EssCompositionIr, CompositionDiagnostics>, local: &str) -> String {
    let diagnostics = result.expect_err("a drifted local type is refused");
    assert_eq!(
        diagnostics
            .as_slice()
            .iter()
            .map(CompositionDiagnostic::code)
            .collect::<Vec<_>>(),
        vec![CompositionCode::TypeConformanceDrift],
        "{diagnostics}"
    );
    let detail = diagnostics.as_slice()[0].detail().to_owned();
    assert!(detail.contains(local), "{detail}");
    detail
}

fn reader_drift(local: &str, imported: &str) -> String {
    let detail = only_drift(under_v3(reader(local, imported)), local);
    assert!(detail.contains(imported), "{detail}");
    detail
}

/// Without `reader: true` a widening is drift, under `/3` exactly as under `/2`.
fn exact_drift(local: &str, imported: &str) {
    only_drift(under_v3(exact(local, imported)), local);
    only_drift(under_v2(exact(local, imported)), local);
}

#[test]
fn reader_admits_a_newtype_chain_read_as_its_primitive() {
    // `CallRef` wraps `CallId`, which wraps `String`.
    reader_conforms("demo.consumer.dashboard.IdAsString", CALL_ID_VIEW);
}

#[test]
fn reader_refuses_a_newtype_read_as_a_narrower_primitive() {
    let detail = reader_drift("demo.consumer.dashboard.IdAsUuid", CALL_ID_VIEW);
    assert!(detail.contains("field `call_id`"), "{detail}");
    assert!(detail.contains("Uuid"), "{detail}");
}

#[test]
fn reader_admits_an_enum_read_as_string() {
    reader_conforms("demo.consumer.dashboard.StateAsString", CALL_STATE_VIEW);
    reader_conforms(
        "demo.consumer.dashboard.StateAsOptionalString",
        CALL_STATE_VIEW,
    );
}

#[test]
fn reader_refuses_an_optional_enum_read_as_a_required_string() {
    let detail = reader_drift("demo.consumer.dashboard.PreviousAsString", CALL_STATE_VIEW);
    assert!(detail.contains("field `previous`"), "{detail}");
    assert!(!detail.contains("field `state`"), "{detail}");
}

#[test]
fn reader_admits_variants_compared_by_wire_name() {
    // The owner's `in_call`/`ringing` against the consumer's `InCall`/`Ringing`/`Held`, whose
    // wire names include every owner wire name.
    reader_conforms("demo.consumer.dashboard.PhaseView", CALL_STATE_VIEW);
}

#[test]
fn reader_refuses_an_enum_lacking_an_owner_wire_name() {
    let detail = reader_drift("demo.consumer.dashboard.NarrowPhaseView", CALL_STATE_VIEW);
    assert!(detail.contains("`ringing`"), "{detail}");
    assert!(detail.contains("field `state`"), "{detail}");
    assert!(detail.contains("field `previous`"), "{detail}");
}

#[test]
fn reader_admits_an_object_read_as_a_map_of_json() {
    // A struct and a `Map<String, String>` are always JSON objects, so `Map<String, Json>` reads
    // every value either can send.
    reader_conforms("demo.consumer.dashboard.ObjectAsMap", OBJECT_VIEW);
}

#[test]
fn reader_refuses_json_read_as_a_map_that_rejects_some_json() {
    let detail = reader_drift("demo.consumer.dashboard.ContextAsStrings", CONTEXT_VIEW);
    assert!(detail.contains("field `body`"), "{detail}");
    assert!(detail.contains("Map<String, String>"), "{detail}");
    // Coordinator decision F1 (correction round 1), revising story decision 5: a producer `Json`
    // may be an array or a scalar, so even `Map<String, Json>` rejects some of its values.
    let detail = reader_drift("demo.consumer.dashboard.ContextAsMap", CONTEXT_VIEW);
    assert!(detail.contains("field `body`"), "{detail}");
    assert!(detail.contains("Map<String, Json>"), "{detail}");
}

#[test]
fn reader_refuses_an_extra_field_that_reads_an_omitted_owner_wire_key() {
    // The consumer omits the owner's `note` and declares `remark`, travelling as `note`, typed
    // `Optional<Integer>`: on the wire it reads the owner's `Optional<String>`.
    let detail = reader_drift("demo.consumer.dashboard.RemarkAsInteger", STATUS_VIEW);
    assert!(detail.contains("field `remark`"), "{detail}");
    // The same key read with a conforming type is a reader like any other.
    reader_conforms("demo.consumer.dashboard.RemarkAsString", STATUS_VIEW);
}

#[test]
fn reader_admits_a_subset_of_the_owner_fields() {
    // `note` and `availability` are omitted; `team` is optional and the owner never sends it.
    reader_conforms("demo.consumer.dashboard.StatusSummary", STATUS_VIEW);
}

#[test]
fn reader_refuses_a_field_the_owner_may_omit_or_lacks() {
    let detail = reader_drift("demo.consumer.dashboard.SummaryNoteRequired", STATUS_VIEW);
    assert!(detail.contains("field `note`"), "{detail}");
    let detail = reader_drift("demo.consumer.dashboard.SummaryTeamRequired", STATUS_VIEW);
    assert!(detail.contains("field `team`"), "{detail}");
    // A key the consumer reads as always sent, from an owner that never sends it.
    let detail = reader_drift("demo.consumer.dashboard.SummaryTeamAlwaysSent", STATUS_VIEW);
    assert!(detail.contains("field `team`"), "{detail}");
    assert!(detail.contains("null_when_absent"), "{detail}");
}

#[test]
fn without_reader_every_widening_stays_drift() {
    for (local, imported) in [
        ("demo.consumer.dashboard.IdAsString", CALL_ID_VIEW),
        ("demo.consumer.dashboard.StateAsString", CALL_STATE_VIEW),
        (
            "demo.consumer.dashboard.StateAsOptionalString",
            CALL_STATE_VIEW,
        ),
        ("demo.consumer.dashboard.PhaseView", CALL_STATE_VIEW),
        ("demo.consumer.dashboard.ObjectAsMap", OBJECT_VIEW),
        ("demo.consumer.dashboard.RemarkAsString", STATUS_VIEW),
        ("demo.consumer.dashboard.StatusSummary", STATUS_VIEW),
    ] {
        exact_drift(local, imported);
    }
    // And what conformed before still conforms under `/3`, reader or not.
    for asserted in [
        exact("demo.consumer.dashboard.AgentStatus", STATUS_VIEW),
        reader("demo.consumer.dashboard.AgentStatus", STATUS_VIEW),
    ] {
        under_v3(asserted).expect("an exact mirror conforms under /3");
    }
}

#[test]
fn v2_refuses_the_reader_key() {
    let models = models();
    let canonical = CompositionSpec::with_reader_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![reader("demo.consumer.dashboard.IdAsString", CALL_ID_VIEW)],
    )
    .to_canonical_json();
    assert!(canonical.contains("\"reader\": true"), "{canonical}");
    let as_v2 = canonical.replace(READER_COMPOSITION_FORMAT, CONFORMANCE_COMPOSITION_FORMAT);
    let diagnostics = compose(
        &models,
        &CompositionSpec::from_json(&as_v2).expect("the DTO reads; the compiler checks the marker"),
    )
    .expect_err("ess-composition/2 does not admit reader");
    assert_eq!(
        diagnostics
            .as_slice()
            .iter()
            .map(CompositionDiagnostic::code)
            .collect::<Vec<_>>(),
        vec![CompositionCode::UnsupportedFormat],
        "the refusal is the marker's, not a drift: {diagnostics}"
    );
    assert!(
        diagnostics.as_slice()[0]
            .detail()
            .contains(READER_COMPOSITION_FORMAT),
        "the refusal names the format that admits it: {diagnostics}"
    );

    // The key is `/3`'s: `/2` refuses it even when it says `false`; `/3` keeps it as written.
    let written_false = canonical.replace("\"reader\": true", "\"reader\": false");
    let read = CompositionSpec::from_json(&written_false).expect("the DTO reads");
    assert!(!read.conformances()[0].reader());
    assert_eq!(read.to_canonical_json(), written_false);
    let diagnostics = compose(
        &models,
        &CompositionSpec::from_json(
            &written_false.replace(READER_COMPOSITION_FORMAT, CONFORMANCE_COMPOSITION_FORMAT),
        )
        .expect("the DTO reads"),
    )
    .expect_err("ess-composition/2 does not admit reader, even false");
    assert!(
        diagnostics.contains(CompositionCode::UnsupportedFormat),
        "{diagnostics}"
    );

    // Built through the /2 constructor, a reader entry is refused the same way.
    let diagnostics = under_v2(reader("demo.consumer.dashboard.IdAsString", CALL_ID_VIEW))
        .expect_err("ess-composition/2 does not admit reader");
    assert!(
        diagnostics.contains(CompositionCode::UnsupportedFormat),
        "{diagnostics}"
    );
}

#[test]
fn v2_bytes_carry_no_reader_key() {
    let asserted = exact("demo.consumer.dashboard.AgentStatus", STATUS_VIEW);
    let composition = under_v2(asserted.clone()).expect("v2 composition compiles");
    let json = composition.to_canonical_json();
    assert!(!json.contains("reader"), "{json}");
    let models = models();
    let authored = CompositionSpec::with_conformances(
        key("devcenter"),
        imports(&models),
        Vec::new(),
        vec![asserted],
    )
    .to_canonical_json();
    assert!(!authored.contains("reader"), "{authored}");
}

#[test]
fn the_reader_key_reads_from_yaml() {
    let models = models();
    let yaml = format!(
        "format: ess-composition/3\n\
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
         conformances:\n\
         \x20 - local: {{ service: consumer, type: demo.consumer.dashboard.StatusSummary }}\n\
         \x20   conforms_to: {{ service: owner, type: {STATUS_VIEW} }}\n\
         \x20   reader: true\n",
        owner = models.owner.source_digest(),
        consumer = models.consumer.source_digest(),
    );
    let specification = CompositionSpec::from_yaml(&yaml).expect("the /3 composition reads");
    assert!(specification.conformances()[0].reader());
    let composition = compose(&models, &specification).expect("the /3 composition compiles");
    assert_eq!(composition.conformances().len(), 1);
}
