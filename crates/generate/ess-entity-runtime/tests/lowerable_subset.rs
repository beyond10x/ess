//! The lowerable subset (beyond10x/ess#231).
//!
//! Three things a host needs before it builds anything: every construct a component cannot be
//! lowered for, reported at once rather than one per build; one entry point that lowers a component
//! of a compiled model; and a published table of what lowers and what is refused, generated from
//! the catalogue this crate exports so it cannot drift from the code that refuses.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::subset::{reference_page, Lowering, CONSTRUCTS, REFERENCE_PAGE};
use ess_entity_runtime::{
    lower, lower_component, ComponentLoweringError, LoweringCode, LoweringDiagnostic,
    LoweringOptions,
};
use ess_service_contract::{extract, ServiceDiagnostic};
use ess_synth::SynthesisPlan;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_root() -> PathBuf {
    crate_root().join("../../..")
}

/// The focused contract fixture at ess/18, with `Child` carrying an `Integer` and a struct field,
/// and each `(file, before, after)` edit applied once.
fn contract(changes: &[(&str, &str, &str)]) -> EssIr {
    let base = crate_root().join("tests/fixtures/contract");
    let mut changes = changes.to_vec();
    changes.push(("system.yaml", "format: ess/4", "format: ess/18"));
    changes.push((
        "domains/local.yaml",
        "      - name: memo\n        type: Optional<String>\n    lifecycle:",
        "      - name: memo\n        type: Optional<String>\n      - name: revision\n        type: Integer\n      - name: payload\n        type: contract.foreign.ForeignPayload\n    lifecycle:",
    ));
    let mut paths = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let (mut parsed, mut labels, mut sources) = (Vec::new(), Vec::new(), SourceMap::new());
    for path in paths {
        let label = path
            .strip_prefix(&base)
            .expect("fixture child")
            .display()
            .to_string();
        let mut text = std::fs::read_to_string(&path).expect("fixture source");
        for (target, before, after) in &changes {
            if label == *target {
                assert!(text.contains(before), "{target} holds {before}");
                text = text.replacen(before, after, 1);
            }
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).expect("fixture parses"),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

/// An event publishing the identity a creation takes from its input.
const STORED: (&str, &str, &str) = (
    "domains/local.yaml",
    "  - name: contract.local.First\n",
    "  - name: contract.local.Stored\n    fields:\n      - name: child_id\n        type: contract.local.ChildId\n\n  - name: contract.local.First\n",
);

/// An event reporting how many rows changed.
const COUNTED: (&str, &str, &str) = (
    "domains/local.yaml",
    "  - name: contract.local.First\n",
    "  - name: contract.local.Counted\n    fields:\n      - name: changed\n        type: Integer\n\n  - name: contract.local.First\n",
);

const INPUT: &str = "    input:\n      - name: child_id\n        type: contract.local.ChildId\n      - name: owner_id\n        type: contract.foreign.OwnerId\n      - name: note\n        type: contract.local.Shared\n      - name: memo\n        type: Optional<String>\n";

const CREATE: &str = "      - name: made\n        creates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.Stored]\n        payload:\n          contract.local.Stored: {child_id: input.child_id}\n        sets:\n          owner_id: input.owner_id\n          note: input.note\n";

/// An update of `Child` named by `child_id` that sets exactly `sets`.
fn update(name: &str, sets: &str) -> String {
    format!(
        "      - name: {name}\n        updates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.First]\n        sets:\n{sets}"
    )
}

/// The fixture with one more `local-service` command, `name`, whose outcomes are `outcomes`.
fn with_command(
    name: &str,
    outcomes: &str,
    extra: &[(&'static str, &'static str, &'static str)],
) -> EssIr {
    let declared = format!(
        "  - name: contract.local.{name}\n{INPUT}    outcomes:\n{outcomes}\n  - name: contract.local.Admin\n"
    );
    let accepted = format!("        - contract.local.Run\n        - contract.local.{name}\n");
    let mut changes = extra.to_vec();
    changes.push((
        "domains/local.yaml",
        "  - name: contract.local.Admin\n",
        &declared,
    ));
    changes.push(("wiring.yaml", "        - contract.local.Run\n", &accepted));
    contract(&changes)
}

fn versions() -> LoweringOptions {
    LoweringOptions {
        definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
            .into_iter()
            .map(|name| {
                (
                    QualifiedName::new(name).expect("qualified name"),
                    NonZeroU32::new(1).expect("nonzero"),
                )
            })
            .collect(),
        scales: BTreeMap::new(),
    }
}

fn local() -> ComponentName {
    ComponentName::new("local-service").expect("component name")
}

fn refused_with(ir: &EssIr, options: &LoweringOptions) -> Vec<LoweringDiagnostic> {
    match lower_component(ir, &local(), options) {
        Ok(_) => Vec::new(),
        Err(ComponentLoweringError::Lowering(diagnostics)) => diagnostics.into_vec(),
        Err(other) => panic!("the component extracts: {other:?}"),
    }
}

fn refused(ir: &EssIr) -> Vec<LoweringDiagnostic> {
    refused_with(ir, &versions())
}

fn has(diagnostics: &[LoweringDiagnostic], code: LoweringCode, path: &str) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code && diagnostic.path == path)
}

fn count(diagnostics: &[LoweringDiagnostic], code: LoweringCode) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == code)
        .count()
}

/// Every construct name a diagnostic carries is a row of the published table, so a reader can look
/// it up.
fn assert_constructs_are_catalogued(diagnostics: &[LoweringDiagnostic]) {
    for diagnostic in diagnostics {
        assert!(
            CONSTRUCTS
                .iter()
                .any(|construct| construct.name == diagnostic.construct)
                || LoweringCode::ALL
                    .iter()
                    .any(|code| !code.is_source_construct()
                        && code.construct() == diagnostic.construct),
            "{diagnostic:?} names a construct the table does not list"
        );
    }
}

// --- One entry point for one component -----------------------------------------------------------

#[test]
fn the_extended_fixture_lowers_without_a_refusal() {
    let ir = with_command(
        "Plain",
        &update("touched", "          note: input.note\n"),
        &[],
    );
    let diagnostics = refused(&ir);
    assert_eq!(diagnostics.len(), 0, "{diagnostics:#?}");
}

#[test]
fn lower_component_is_extract_then_lower_over_the_model_plan() {
    let ir = with_command(
        "Plain",
        &update("touched", "          note: input.note\n"),
        &[],
    );
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &local()).expect("service extracts");
    let expected = lower(&service, &versions()).expect("the component lowers");
    let lowered = lower_component(&ir, &local(), &versions()).expect("the component lowers");
    assert_eq!(lowered, expected);
}

#[test]
fn lower_component_names_a_component_the_model_does_not_declare() {
    let ir = with_command(
        "Plain",
        &update("touched", "          note: input.note\n"),
        &[],
    );
    let missing = ComponentName::new("absent-service").expect("component name");
    match lower_component(&ir, &missing, &versions()) {
        Err(ComponentLoweringError::Component(diagnostics)) => assert!(
            diagnostics.iter().any(|diagnostic| matches!(
                diagnostic,
                ServiceDiagnostic::UnknownComponent { component } if component == &missing
            )),
            "{diagnostics:?}"
        ),
        other => panic!("an unknown component is refused by name: {other:?}"),
    }
}

// --- Every refusal at once ----------------------------------------------------------------------

#[test]
fn a_related_guard_does_not_hide_a_refused_value_in_the_same_command() {
    let outcomes = format!(
        "      - name: no-owner\n        when_related: {{via: input.owner_id, exists: false}}\n        error: contract.local.Rejected\n{}",
        update("bumped", "          memo: {cleared: true}\n")
    );
    let diagnostics = refused(&with_command("Bump", &outcomes, &[]));
    assert!(
        has(
            &diagnostics,
            LoweringCode::RelatedGuardUnsupported,
            "contract.local.Bump.no-owner.when_related"
        ),
        "{diagnostics:#?}"
    );
    assert!(
        has(
            &diagnostics,
            LoweringCode::ClearedValueUnsupported,
            "contract.local.Bump.bumped.sets.memo"
        ),
        "{diagnostics:#?}"
    );
    assert_eq!(
        count(&diagnostics, LoweringCode::RelatedGuardUnsupported),
        1,
        "{diagnostics:#?}"
    );
    assert_constructs_are_catalogued(&diagnostics);
}

#[test]
fn create_or_update_does_not_hide_a_refused_update_value() {
    let outcomes = format!(
        "{}{}",
        update("stored", "          memo: {cleared: true}\n"),
        CREATE.replace(
            "        creates:",
            "        unknown_instance: true\n        creates:"
        )
    );
    let diagnostics = refused(&with_command("Put", &outcomes, &[STORED]));
    assert!(
        has(
            &diagnostics,
            LoweringCode::ExistenceSelectionUnsupported,
            "contract.local.Put"
        ),
        "{diagnostics:#?}"
    );
    assert!(
        has(
            &diagnostics,
            LoweringCode::ClearedValueUnsupported,
            "contract.local.Put.stored.sets.memo"
        ),
        "{diagnostics:#?}"
    );
    assert_eq!(
        count(&diagnostics, LoweringCode::ExistenceSelectionUnsupported),
        1,
        "one construct is reported once: {diagnostics:#?}"
    );
    assert_constructs_are_catalogued(&diagnostics);
}

#[test]
fn a_mixed_entrypoint_does_not_hide_a_refused_update_value() {
    let outcomes = format!(
        "      - name: stored\n        when: note == kept\n        updates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.First]\n        sets:\n          memo: {{cleared: true}}\n{CREATE}"
    );
    let diagnostics = refused(&with_command("Either", &outcomes, &[STORED]));
    assert!(
        has(
            &diagnostics,
            LoweringCode::MixedEntrypointUnsupported,
            "contract.local.Either"
        ),
        "{diagnostics:#?}"
    );
    assert!(
        has(
            &diagnostics,
            LoweringCode::ClearedValueUnsupported,
            "contract.local.Either.stored.sets.memo"
        ),
        "{diagnostics:#?}"
    );
    assert_constructs_are_catalogued(&diagnostics);
}

/// The alphabet still refused is one on a declared response field, which entity-core admits and does
/// not enforce: a command whose entity has no definition version is walked as refused, and its
/// response is still checked.
#[test]
fn a_missing_definition_version_does_not_hide_an_alphabet() {
    let ir = with_command(
        "Plain",
        &update("touched", "          note: input.note\n"),
        &[(
            "domains/local.yaml",
            "  - name: contract.local.Receipt\n    kind: newtype\n    of: String\n",
            "  - name: contract.local.Receipt\n    kind: newtype\n    of: String\n    alphabet: \"0123456789-ceiprt\"\n",
        )],
    );
    let diagnostics = refused_with(&ir, &LoweringOptions::default());
    assert_eq!(
        count(&diagnostics, LoweringCode::MissingDefinitionVersion),
        2,
        "{diagnostics:#?}"
    );
    assert!(
        count(&diagnostics, LoweringCode::AlphabetUnsupported) > 0,
        "{diagnostics:#?}"
    );
    assert_constructs_are_catalogued(&diagnostics);
}

/// `contract.local.Shared` over the alphabet `kep xactly`.
const SHARED_ALPHABET: (&str, &str, &str) = (
    "domains/local.yaml",
    "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n",
    "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n    alphabet: \"kep xactly\"\n",
);

/// `contract.local.ChildId` as a text over lowercase hexadecimal digits and `-`.
const CHILD_ID_ALPHABET: (&str, &str, &str) = (
    "domains/local.yaml",
    "  - name: contract.local.ChildId\n    kind: newtype\n    of: Uuid\n",
    "  - name: contract.local.ChildId\n    kind: newtype\n    of: String\n    alphabet: \"0123456789abcdef-\"\n",
);

/// ESS holds a literal to a declared prefix and to an enum's variants, not to an alphabet. Every
/// place the lowering writes a text literal into a field entity-core validates — an update's
/// `sets:`, a creation's `sets:`, a creation's identity — refuses one with a character outside the
/// field's alphabet by name; the same literal inside the alphabet lowers.
#[test]
fn a_literal_outside_its_fields_alphabet_is_refused_by_name_wherever_it_is_written() {
    let refused_literals =
        |outcomes: &str, extra: &[(&'static str, &'static str, &'static str)]| {
            refused(&with_command("Tune", outcomes, extra))
                .into_iter()
                .filter(|diagnostic| diagnostic.code == LoweringCode::AlphabetUnsupported)
                .map(|diagnostic| (diagnostic.path, diagnostic.construct))
                .collect::<Vec<_>>()
        };
    let construct = "a text literal written outside its field's `alphabet:`";
    assert_eq!(
        refused_literals(
            &update("tuned", "          note: \"kept!\"\n"),
            &[SHARED_ALPHABET]
        ),
        [("contract.local.Tune.tuned.note".to_owned(), construct)],
        "an update's `sets:`"
    );
    assert_eq!(
        refused_literals(
            &CREATE.replace("note: input.note", "note: \"kept!\""),
            &[SHARED_ALPHABET, STORED]
        ),
        [("contract.local.Tune.made.note".to_owned(), construct)],
        "a creation's `sets:`"
    );
    assert_eq!(
        refused_literals(
            &CREATE.replace("{child_id: input.child_id}", "{child_id: \"kept!\"}"),
            &[CHILD_ID_ALPHABET, STORED]
        ),
        [("contract.local.Tune.made.identity".to_owned(), construct)],
        "a creation's identity"
    );
    assert_eq!(
        refused_literals(
            &update("tuned", "          note: \"kept\"\n"),
            &[SHARED_ALPHABET]
        ),
        Vec::<(String, &str)>::new(),
        "a literal inside the alphabet"
    );
}

/// Four nested alphabets that overlap pairwise, `abc` over `ac` over `bc` over `ab`, share no
/// character: `ac` empties the intersection. Every string the type reaches is refused by name once,
/// by the layer that empties it, naming the layers, and nothing reaches entity-core's own refusal
/// of an empty alphabet.
#[test]
fn alphabets_that_share_no_character_are_refused_by_name_once_per_string() {
    let ir = with_command(
        "Plain",
        &update("touched", "          note: input.note\n"),
        &[(
            "domains/local.yaml",
            "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n",
            "  - name: contract.local.Base\n    kind: newtype\n    of: String\n    alphabet: \"ab\"\n\n  - name: contract.local.Middle\n    kind: newtype\n    of: contract.local.Base\n    alphabet: \"bc\"\n\n  - name: contract.local.Inner\n    kind: newtype\n    of: contract.local.Middle\n    alphabet: \"ac\"\n\n  - name: contract.local.Shared\n    kind: newtype\n    of: contract.local.Inner\n    alphabet: \"abc\"\n",
        )],
    );
    let diagnostics = refused(&ir);
    assert_eq!(
        count(&diagnostics, LoweringCode::TargetDefinitionRefused),
        0,
        "{diagnostics:#?}"
    );
    let empty = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LoweringCode::AlphabetUnsupported)
        .collect::<Vec<_>>();
    assert_eq!(
        empty
            .iter()
            .map(|diagnostic| diagnostic.path.as_str())
            .collect::<Vec<_>>(),
        [
            "contract.local.Child.fields.note",
            "contract.local.Plain.input.note",
            "contract.local.Run.input.note"
        ],
        "{diagnostics:#?}"
    );
    for diagnostic in empty {
        assert_eq!(
            diagnostic.construct,
            "nested `alphabet:`s that share no character"
        );
        for layer in [
            "contract.local.Inner",
            "contract.local.Middle",
            "contract.local.Base",
        ] {
            assert!(diagnostic.message.contains(layer), "{diagnostic:?}");
        }
        assert!(
            diagnostic.message.contains("share no character"),
            "{diagnostic:?}"
        );
    }
    assert_constructs_are_catalogued(&diagnostics);
}

/// A set outcome's own `sets:` and `{count: changed}` are part of the set effect, which is refused
/// once; they are not refused again on their own.
#[test]
fn a_set_effect_is_reported_once_however_far_the_command_is_walked() {
    let outcomes = "      - name: retagged\n        updates: contract.local.Child\n        instances: {where: note == input.note}\n        emits: [contract.local.Counted]\n        payload:\n          contract.local.Counted: {changed: {count: changed}}\n        sets:\n          memo: {cleared: true}\n";
    let diagnostics = refused(&with_command("Retag", outcomes, &[COUNTED]));
    assert_eq!(
        count(&diagnostics, LoweringCode::SetEffectUnsupported),
        1,
        "{diagnostics:#?}"
    );
    assert_constructs_are_catalogued(&diagnostics);
}

#[test]
fn an_affects_effect_does_not_hide_a_refused_value_in_its_branch() {
    let outcomes = "      - name: touched\n        updates: contract.local.Child\n        instance: child_id\n        emits: [contract.local.First]\n        sets:\n          memo: {cleared: true}\n        affects:\n          - entity: contract.local.Child\n            where: note == subject.note\n            sets:\n              note: input.note\n";
    let diagnostics = refused(&with_command("Touch", outcomes, &[]));
    assert!(
        has(
            &diagnostics,
            LoweringCode::SetEffectUnsupported,
            "contract.local.Touch.touched.affects"
        ),
        "{diagnostics:#?}"
    );
    assert!(
        has(
            &diagnostics,
            LoweringCode::ClearedValueUnsupported,
            "contract.local.Touch.touched.sets.memo"
        ),
        "{diagnostics:#?}"
    );
    assert_eq!(
        count(&diagnostics, LoweringCode::SetEffectUnsupported),
        1,
        "{diagnostics:#?}"
    );
    assert_constructs_are_catalogued(&diagnostics);
}

// --- Each refusal names its construct -----------------------------------------------------------

/// The value expressions of ess/14 entity-core has no form for, each refused under its own name.
#[test]
fn each_value_expression_is_refused_under_its_own_construct() {
    for (sets, construct) in [
        ("          revision: {increment: 1}\n", "`{increment: …}`"),
        ("          note: {subject: note}\n", "`{subject: …}`"),
        (
            "          memo: {input: memo, else: {generated: true}}\n",
            "`{input: …, else: …}`",
        ),
        ("          payload: {value: fixed}\n", "a struct of sources"),
    ] {
        let diagnostics = refused(&with_command("Tune", &update("tuned", sets), &[]));
        let value: Vec<_> = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == LoweringCode::ValueExpressionUnsupported)
            .collect();
        assert_eq!(value.len(), 1, "{sets}: {diagnostics:#?}");
        assert_eq!(value[0].path, "contract.local.Tune.tuned", "{sets}");
        assert_eq!(value[0].construct, construct, "{sets}");
        assert_constructs_are_catalogued(&diagnostics);
    }
}

#[test]
fn a_related_value_is_refused_under_its_own_construct() {
    let diagnostics = refused(&with_command(
        "Tune",
        &update(
            "tuned",
            "          owner_id: {related: {via: input.owner_id, field: owner_id}}\n",
        ),
        &[],
    ));
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code
            == LoweringCode::ValueExpressionUnsupported
            && diagnostic.construct == "`{related: …}`"),
        "{diagnostics:#?}"
    );
    assert_constructs_are_catalogued(&diagnostics);
}

#[test]
fn a_diagnostic_renders_its_path_code_and_construct() {
    let diagnostics = refused(&with_command(
        "Tune",
        &update("tuned", "          revision: {increment: 1}\n"),
        &[],
    ));
    let rendered = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == LoweringCode::ValueExpressionUnsupported)
        .expect("the increment is refused")
        .to_string();
    assert!(
        rendered.starts_with(
            "contract.local.Tune.tuned: ValueExpressionUnsupported (`{increment: …}`): "
        ),
        "{rendered}"
    );
}

// --- The catalogue and the page ------------------------------------------------------------------

/// The variant names of `pub enum LoweringCode` as declared in `src/lib.rs`, in order.
fn declared_codes() -> Vec<String> {
    let source = std::fs::read_to_string(crate_root().join("src/lib.rs")).expect("lib.rs");
    let start = source
        .find("pub enum LoweringCode {")
        .expect("the enum is declared");
    let body = &source[start..];
    let body = &body[body.find('{').expect("body") + 1..body.find("\n}").expect("end")];
    body.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("///") && !line.starts_with("#[") && !line.is_empty())
        .map(|line| line.trim_end_matches(',').to_owned())
        .collect()
}

#[test]
fn every_lowering_code_is_catalogued_in_declaration_order() {
    let catalogued = LoweringCode::ALL
        .iter()
        .map(|code| code.name().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(catalogued, declared_codes());
    for code in LoweringCode::ALL {
        assert_eq!(code.name(), format!("{code:?}"));
        assert!(!code.construct().is_empty(), "{code:?}");
        assert!(!code.meaning().is_empty(), "{code:?}");
    }
}

#[test]
fn every_source_construct_code_has_a_row_and_every_row_a_source_construct_code() {
    for code in LoweringCode::ALL {
        let rows = CONSTRUCTS
            .iter()
            .filter(|construct| {
                matches!(construct.lowering, Lowering::Refused { code: refused, .. } if refused == *code)
            })
            .count();
        if code.is_source_construct() {
            assert!(
                rows > 0,
                "{code:?} refuses a construct the table does not list"
            );
        } else {
            assert_eq!(rows, 0, "{code:?} is not a source construct");
        }
    }
    let mut names = CONSTRUCTS
        .iter()
        .map(|construct| construct.name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before, "two rows share a construct name");
}

/// entity-core 0.27.0 carries a String alphabet and a text's length
/// (<https://github.com/beyond10x/entity-runtime/issues/54>), so both rows are lowered. The shapes
/// it does not take keep refused rows of their own, and the two codes name those rows rather than
/// the lowered ones.
#[test]
fn an_alphabet_and_a_text_count_are_lowered_rows() {
    let lowered = ["`alphabet:`", "`.count` of a text"];
    for name in lowered {
        let row = CONSTRUCTS
            .iter()
            .find(|construct| construct.name == name)
            .unwrap_or_else(|| panic!("{name} has a row"));
        assert!(matches!(row.lowering, Lowering::Lowered { .. }), "{row:?}");
    }
    for code in [
        LoweringCode::AlphabetUnsupported,
        LoweringCode::TextLengthUnsupported,
    ] {
        assert!(
            !lowered.contains(&code.construct()),
            "{code:?} names a lowered row"
        );
    }
}

/// Every row cites at least one test, and every cited test exists, so a row cannot claim a
/// lowering or a refusal nothing exercises.
#[test]
fn every_row_cites_tests_that_exist() {
    let codes = LoweringCode::ALL
        .iter()
        .filter(|code| !code.is_source_construct())
        .map(|code| code.evidence());
    let rows = CONSTRUCTS.iter().map(|construct| construct.evidence);
    for evidence in codes.chain(rows) {
        assert!(!evidence.is_empty(), "a row cites no test");
        for cited in evidence {
            let (file, test) = cited
                .split_once("::")
                .unwrap_or_else(|| panic!("`{cited}` is `<file>::<test>`"));
            let source = std::fs::read_to_string(crate_root().join(file))
                .unwrap_or_else(|error| panic!("`{cited}`: {error}"));
            let declared = format!("fn {test}()");
            let at = source
                .find(&declared)
                .unwrap_or_else(|| panic!("`{cited}` names no test"));
            assert!(
                source[..at].trim_end().ends_with("#[test]")
                    || source[..at]
                        .trim_end()
                        .rsplit_once('\n')
                        .is_some_and(|(above, _)| above.trim_end().ends_with("#[test]")),
                "`{cited}` is not a #[test]"
            );
        }
    }
}

/// The published page is the catalogue's rendering, byte for byte. Regenerate it with
/// `ESS_LOWERING_REFERENCE=write cargo test -p ess-entity-runtime --test lowerable_subset`.
#[test]
fn the_reference_page_is_the_catalogue_rendering() {
    let path = repository_root().join(REFERENCE_PAGE);
    let rendered = reference_page();
    if std::env::var_os("ESS_LOWERING_REFERENCE").is_some_and(|value| value == "write") {
        std::fs::write(&path, &rendered).expect("the page writes");
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == rendered,
        "{} is not the rendering of `ess_entity_runtime::subset`; regenerate it with \
         `ESS_LOWERING_REFERENCE=write cargo test -p ess-entity-runtime --test lowerable_subset`",
        Path::new(REFERENCE_PAGE).display()
    );
    for code in LoweringCode::ALL {
        assert!(rendered.contains(&format!("`{}`", code.name())), "{code:?}");
    }
}
