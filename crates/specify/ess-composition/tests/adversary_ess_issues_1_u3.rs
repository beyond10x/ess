//! Adversarial cases against `ess-composition/5` payload ends, value-constraint comparison and the
//! `reference_outside_component` remedy.
//!
//! Each case reuses a fixture set of this crate, applying in-memory edits to a system's sources
//! where it needs a shape the fixture lacks, so no fixture file changes.

use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionDiagnostics, CompositionSpec, EssCompositionIr,
    ServiceKey, SourceDigest,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// One in-memory edit: in the source labelled `label`, replace the first `from` with `to`.
type Edit<'a> = (&'a str, &'a str, &'a str);

fn compiled(set: &str, system: &str, edits: &[Edit<'_>]) -> EssIr {
    let base: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(set)
        .join(system);
    let mut found = Vec::new();
    let mut pending = vec![base.clone()];
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

    let mut applied = vec![false; edits.len()];
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("fixture source is beneath fixture root")
            .display()
            .to_string();
        let mut text = std::fs::read_to_string(&path).expect("fixture source is readable");
        for (index, (target, from, to)) in edits.iter().enumerate() {
            if *target == label {
                assert!(
                    text.contains(from),
                    "edit {index}: `{label}` holds {from:?}"
                );
                text = text.replacen(from, to, 1);
                applied[index] = true;
            }
        }
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} parses: {error}\n{text}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    assert!(
        applied.iter().all(|done| *done),
        "every edit names a source"
    );
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("{set}/{system} fixture validates:\n{errors}"));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("{set}/{system} fixture resolves:\n{diagnostics}"))
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid fixture service key")
}

struct Models {
    demo: EssIr,
    wire: EssIr,
}

impl Models {
    fn of(set: &str, demo: &[Edit<'_>], wire: &[Edit<'_>]) -> Self {
        Self {
            demo: compiled(set, "demo", demo),
            wire: compiled(set, "wire", wire),
        }
    }

    /// The document header, `demo` selecting `component`; `rest` follows at line 6.
    fn document(&self, format: &str, component: &str, rest: &str) -> String {
        format!(
            "format: {format}\n\
             composition: demo-wire\n\
             services:\n  \
               - {{key: demo, system: demo, version: v1, source_digest: {}, component: {component}}}\n  \
               - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
             {rest}",
            SourceDigest::of(&self.demo),
            SourceDigest::of(&self.wire),
        )
    }

    fn compose(&self, text: &str) -> Result<EssCompositionIr, CompositionDiagnostics> {
        let specification = CompositionSpec::from_yaml(text)
            .unwrap_or_else(|error| panic!("the document reads: {error}\n{text}"));
        let (demo, wire) = (key("demo"), key("wire"));
        compile(
            &specification,
            [
                CompiledService::new(&demo, &self.demo),
                CompiledService::new(&wire, &self.wire),
            ],
        )
    }

    /// A `/5` type conformance of `demo.gate.<local>` against `wire.server.<imported>`.
    fn conformance(&self, local: &str, imported: &str, reader: bool) -> String {
        self.document(
            "ess-composition/5",
            "relay",
            &format!(
                "conformances:\n  \
                   - local: {{service: demo, type: demo.gate.{local}}}\n    \
                     conforms_to: {{service: wire, type: wire.server.{imported}}}\n{}",
                if reader { "    reader: true\n" } else { "" }
            ),
        )
    }

    /// A one-reference document under `format`, `demo` selecting `component`.
    fn reference(&self, format: &str, component: &str, kind: &str, name: &str) -> String {
        self.document(
            format,
            component,
            &format!(
                "references:\n  \
                   - service: demo\n    \
                     semantic: {{kind: {kind}, name: {name}}}\n"
            ),
        )
    }
}

// ---------------------------------------------------------------------------------------------
// `/1` to `/4`: a document reads, and is refused, exactly as before `/5`.
// ---------------------------------------------------------------------------------------------

/// The contract-role models, whose `/4` documents existed before `/5`.
fn role_models() -> Models {
    Models::of("contract-role", &[], &[])
}

/// A misspelt key on a `/4` conformance end was refused, before `/5`, by the derived closed reader
/// of `{service, type}`: ``unknown field `typo`, expected `service` or `type` `` and nothing more. `/5`
/// did not exist, so no `/4` refusal could name it.
#[test]
fn a_v4_misspelt_end_key_reads_as_before() {
    let models = role_models();
    let text = models.document(
        "ess-composition/4",
        "relay",
        "conformances:\n  \
           - local: {service: demo, type: demo.gate.Channel, typo: x}\n    \
             conforms_to: {service: wire, type: wire.server.Channel}\n",
    );
    let error = CompositionSpec::from_yaml(&text)
        .expect_err("an unknown key does not read")
        .to_string();
    assert!(
        error.contains("unknown field `typo`, expected `service` or `type` at line 7"),
        "{error}"
    );
    assert!(!error.contains("ess-composition/5"), "{error}");
}

/// An empty `/4` end was refused by the derived reader for its first missing field, `service`.
#[test]
fn a_v4_empty_end_names_its_first_missing_field() {
    let models = role_models();
    let text = models.document(
        "ess-composition/4",
        "relay",
        "conformances:\n  \
           - local: {}\n    \
             conforms_to: {service: wire, type: wire.server.Channel}\n",
    );
    let error = CompositionSpec::from_yaml(&text)
        .expect_err("an empty end does not read")
        .to_string();
    assert!(error.contains("missing field `service`"), "{error}");
}

/// A `/4` end without `type` was refused where the end is, at its path and its line (7), as every
/// other closed-reader refusal in the document is.
#[test]
fn a_v4_end_without_type_is_refused_where_it_is() {
    let models = role_models();
    let text = models.document(
        "ess-composition/4",
        "relay",
        "conformances:\n  \
           - local: {service: demo}\n    \
             conforms_to: {service: wire, type: wire.server.Channel}\n",
    );
    let error = CompositionSpec::from_yaml(&text)
        .expect_err("an end without `type` does not read")
        .to_string();
    assert!(error.contains("missing field `type`"), "{error}");
    assert!(error.contains("conformances[0].local"), "{error}");
    assert!(error.contains("at line 7"), "{error}");
}

// ---------------------------------------------------------------------------------------------
// `/5` value constraints: the same condition, written another way.
// ---------------------------------------------------------------------------------------------

const DEMO_TYPES: &str = "domains/gate.yaml";
const WIRE_TYPES: &str = "domains/server.yaml";

fn constraint_models(demo: &str, wire: &str) -> Models {
    let demo = format!("types:\n{demo}");
    let wire = format!("types:\n{wire}");
    Models::of(
        "constraint-conformance",
        &[(DEMO_TYPES, "types:\n", &demo)],
        &[(WIRE_TYPES, "types:\n", &wire)],
    )
}

fn drift_detail(models: &Models, text: &str) -> Option<String> {
    match models.compose(text) {
        Ok(_) => None,
        Err(diagnostics) => Some(diagnostics.to_string()),
    }
}

/// The documented promise: invariants are compared "by parsed predicate, so spelling does not
/// matter", and exact ends must "state the same conditions". `1 <= value <= 9` written as a
/// conjunction in the other order is the same condition.
#[test]
fn a_conjunction_in_another_order_states_the_same_condition() {
    let models = constraint_models(
        "  - name: demo.gate.Band\n    kind: newtype\n    of: Integer\n    invariants:\n      \
         - all: [\"value >= 1\", \"value <= 9\"]\n",
        "  - name: wire.server.Band\n    kind: newtype\n    of: Integer\n    invariants:\n      \
         - all: [\"value <= 9\", \"value >= 1\"]\n",
    );
    for reader in [false, true] {
        let text = models.conformance("Band", "Band", reader);
        assert_eq!(drift_detail(&models, &text), None, "reader {reader}");
    }
}

/// Two invariants and their conjunction state the same conditions: every value one admits the
/// other admits.
#[test]
fn two_invariants_state_what_their_conjunction_states() {
    let models = constraint_models(
        "  - name: demo.gate.Band\n    kind: newtype\n    of: Integer\n    invariants:\n      \
         - value >= 1\n      - value <= 9\n",
        "  - name: wire.server.Band\n    kind: newtype\n    of: Integer\n    invariants:\n      \
         - all: [\"value >= 1\", \"value <= 9\"]\n",
    );
    for reader in [false, true] {
        let text = models.conformance("Band", "Band", reader);
        assert_eq!(drift_detail(&models, &text), None, "reader {reader}");
    }
}

/// An alphabet is a set of characters (`types.rs`, `TypeBody::Newtype::alphabet`): the same set
/// written in another order admits exactly the same values. The reader rule already compares it
/// as a set; an exact end compares the text.
#[test]
fn an_alphabet_in_another_order_admits_the_same_values() {
    let models = constraint_models(
        "  - name: demo.gate.Hex\n    kind: newtype\n    of: String\n    \
         alphabet: \"fedcba9876543210\"\n",
        "  - name: wire.server.Hex\n    kind: newtype\n    of: String\n    \
         alphabet: \"0123456789abcdef\"\n",
    );
    // The reader rule agrees that nothing is narrowed.
    assert_eq!(
        drift_detail(&models, &models.conformance("Hex", "Hex", true)),
        None
    );
    assert_eq!(
        drift_detail(&models, &models.conformance("Hex", "Hex", false)),
        None
    );
}

/// What held: a reader whose wrapped newtype narrows the alphabet is drift, though its own layer
/// states none.
#[test]
fn a_reader_narrowing_an_inner_layer_is_drift() {
    let models = constraint_models(
        "  - name: demo.gate.Inner\n    kind: newtype\n    of: String\n    alphabet: \"ab\"\n  \
         - name: demo.gate.Outer\n    kind: newtype\n    of: demo.gate.Inner\n",
        "  - name: wire.server.Inner\n    kind: newtype\n    of: String\n    alphabet: \"abc\"\n  \
         - name: wire.server.Outer\n    kind: newtype\n    of: wire.server.Inner\n",
    );
    let detail = drift_detail(&models, &models.conformance("Outer", "Outer", true))
        .expect("a narrower inner alphabet is drift");
    assert!(detail.contains("type_conformance_drift"), "{detail}");
    assert!(detail.contains("alphabet `ab`"), "{detail}");
}

/// What held: a named attribute type is walked, so an attribute of an enum type whose variants
/// differ is drift.
#[test]
fn an_attribute_of_a_drifting_enum_type_is_drift() {
    let models = constraint_models(
        "  - name: demo.gate.Team\n    kind: enum\n    variants: [Desk, Robot]\n  \
         - name: demo.gate.Queue\n    kind: enum\n    attributes:\n      \
         - {name: handled_by, type: demo.gate.Team}\n    variants:\n      \
         - {name: Open, attributes: {handled_by: Desk}}\n",
        "  - name: wire.server.Team\n    kind: enum\n    variants: [Desk, Robot, Bot]\n  \
         - name: wire.server.Queue\n    kind: enum\n    attributes:\n      \
         - {name: handled_by, type: wire.server.Team}\n    variants:\n      \
         - {name: Open, attributes: {handled_by: Desk}}\n",
    );
    for reader in [false, true] {
        let detail = drift_detail(&models, &models.conformance("Queue", "Queue", reader))
            .unwrap_or_else(|| panic!("reader {reader}: the attribute's type drifts"));
        assert!(detail.contains("attribute `handled_by`"), "{detail}");
        assert!(detail.contains("Bot"), "{detail}");
    }
}

// ---------------------------------------------------------------------------------------------
// `/5` payload ends: what held.
// ---------------------------------------------------------------------------------------------

#[test]
fn an_undeclared_event_or_command_end_is_unresolved() {
    let models = Models::of("payload-conformance", &[], &[]);
    for end in [
        "event: demo.gate.Nowhere",
        "error: demo.gate.Nowhere",
        "command: demo.gate.Nowhere, part: input",
        "command: demo.gate.Nowhere, part: response",
    ] {
        let text = models.document(
            "ess-composition/5",
            "relay",
            &format!(
                "conformances:\n  \
                   - local: {{service: demo, {end}}}\n    \
                     conforms_to: {{service: wire, event: wire.server.Subscribed}}\n"
            ),
        );
        let detail = drift_detail(&models, &text).unwrap_or_else(|| panic!("{end} is refused"));
        assert!(
            detail.contains("unresolved_semantic_reference"),
            "{end}: {detail}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// `reference_outside_component`: the remedy it names admits the name.
// ---------------------------------------------------------------------------------------------

fn outside_models(demo: &[Edit<'_>]) -> Models {
    Models::of("outside-component", demo, &[])
}

/// The single refusal of `text`.
fn refusal(models: &Models, text: &str) -> String {
    let diagnostics = models.compose(text).expect_err("the reference is refused");
    let all = diagnostics.as_slice();
    assert_eq!(all.len(), 1, "{diagnostics}");
    assert_eq!(
        all[0].code().to_string(),
        "reference_outside_component",
        "{diagnostics}"
    );
    all[0].detail().to_owned()
}

/// Under `/1` a type is exported only when a command, event, error or view of the component
/// reaches it; owning its domain is not enough (`the_v1_hint_is_kept`: `relay` owns `demo.gate`
/// and is refused `demo.gate.Unused`). So under `/1` the remedy must not send the author to a
/// component that only owns the domain.
#[test]
fn a_v1_remedy_names_a_component_that_admits_the_type() {
    let models = outside_models(&[]);
    let detail = refusal(
        &models,
        &models.reference("ess-composition/1", "auditor", "type", "demo.gate.Unused"),
    );
    if detail.contains("component `relay` owns it: select it") {
        let followed = models.compose(&models.reference(
            "ess-composition/1",
            "relay",
            "type",
            "demo.gate.Unused",
        ));
        assert!(
            followed.is_ok(),
            "the remedy `{detail}` was followed and the reference is still refused:\n{}",
            followed
                .err()
                .map(|found| found.to_string())
                .unwrap_or_default()
        );
    }
}

/// Under `/1` adding the domain to `owns` does not export a type nothing reaches, so a remedy that
/// says so sends the author into a second refusal.
#[test]
fn a_v1_owns_remedy_admits_the_type() {
    let models = outside_models(&[]);
    let detail = refusal(
        &models,
        &models.reference("ess-composition/1", "relay", "type", "demo.loose.Note"),
    );
    if detail.contains("add it to `relay`'s `owns`") {
        let owning = outside_models(&[(
            "system.yaml",
            "      domains:\n        - demo.gate\n",
            "      domains:\n        - demo.gate\n        - demo.loose\n",
        )]);
        let followed = owning.compose(&owning.reference(
            "ess-composition/1",
            "relay",
            "type",
            "demo.loose.Note",
        ));
        assert!(
            followed.is_ok(),
            "the remedy `{detail}` was followed and the reference is still refused:\n{}",
            followed
                .err()
                .map(|found| found.to_string())
                .unwrap_or_default()
        );
    }
}

/// A type in a domain no component owns can still be exported, by a component that accepts a
/// command whose input carries it. The refusal says "no component owns" and names no component;
/// the remedy contract is "the other components that export it".
#[test]
fn a_type_exported_through_accepts_names_its_exporter() {
    let models = outside_models(&[(
        "domains/audit.yaml",
        "    input:\n      - name: entry\n        type: demo.audit.Entry\n",
        "    input:\n      - name: entry\n        type: demo.audit.Entry\n      \
         - name: note\n        type: demo.loose.Note\n",
    )]);
    // `auditor` exports it: selecting it admits the reference.
    models
        .compose(&models.reference("ess-composition/4", "auditor", "type", "demo.loose.Note"))
        .expect("`auditor` exports `demo.loose.Note` through `demo.audit.Record`");
    let detail = refusal(
        &models,
        &models.reference("ess-composition/4", "relay", "type", "demo.loose.Note"),
    );
    assert!(detail.contains("`auditor`"), "{detail}");
}
