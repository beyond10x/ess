//! Under `ess-composition/5` a type conformance also compares value constraints and enum variant
//! attributes: newtype `alphabet`, `prefix` and invariants, struct invariants, and each variant's
//! attribute values (`docs/design/composition-type-conformance.md`). Earlier formats compare
//! structure only, as before.
//!
//! The fixture is two systems: `wire` (component `wire-server`) owns the types, `demo` (component
//! `relay`) holds local copies, each differing in one constraint, attribute or structural fact.

use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionDiagnostics, CompositionSpec, EssCompositionIr,
    ServiceKey, SourceDigest,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn compiled(system: &str) -> EssIr {
    let base: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/constraint-conformance")
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
        .unwrap_or_else(|errors| panic!("{system} fixture validates:\n{errors}"));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("{system} fixture resolves:\n{diagnostics}"))
}

struct Models {
    demo: EssIr,
    wire: EssIr,
}

impl Models {
    fn new() -> Self {
        Self {
            demo: compiled("demo"),
            wire: compiled("wire"),
        }
    }

    /// Composes `demo.gate.<local>` against `wire.server.<imported>` under `format`.
    fn compose(
        &self,
        format: &str,
        local: &str,
        imported: &str,
        reader: bool,
    ) -> Result<EssCompositionIr, CompositionDiagnostics> {
        let text = format!(
            "format: {format}\n\
             composition: demo-wire\n\
             services:\n  \
               - {{key: demo, system: demo, version: v1, source_digest: {}, component: relay}}\n  \
               - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
             conformances:\n  \
               - local: {{service: demo, type: demo.gate.{local}}}\n    \
                 conforms_to: {{service: wire, type: wire.server.{imported}}}\n{}",
            SourceDigest::of(&self.demo),
            SourceDigest::of(&self.wire),
            if reader { "    reader: true\n" } else { "" },
        );
        let specification = CompositionSpec::from_yaml(&text)
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

    fn conforms(&self, local: &str, imported: &str, reader: bool) {
        if let Err(diagnostics) = self.compose("ess-composition/5", local, imported, reader) {
            panic!("`{local}` conforms to `{imported}` (reader {reader}):\n{diagnostics}");
        }
    }

    /// The one drift detail of `local` against `imported` under `/5`.
    fn drift(&self, local: &str, imported: &str, reader: bool) -> String {
        match self.compose("ess-composition/5", local, imported, reader) {
            Ok(ir) => panic!(
                "`{local}` drifts from `{imported}` (reader {reader}):\n{}",
                ir.to_canonical_json()
            ),
            Err(diagnostics) => {
                let all = diagnostics.as_slice();
                assert_eq!(all.len(), 1, "{diagnostics}");
                assert_eq!(
                    all[0].code().to_string(),
                    "type_conformance_drift",
                    "{diagnostics}"
                );
                let detail = all[0].detail().to_owned();
                assert!(
                    detail.starts_with(&format!(
                        "`demo` type `demo.gate.{local}` does not conform to `wire` type \
                         `wire.server.{imported}`: "
                    )),
                    "{detail}"
                );
                detail
            }
        }
    }
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid fixture service key")
}

#[test]
fn compose_compares_invariants() {
    let detail = Models::new().drift("Code", "Code", false);
    assert!(
        detail.ends_with(
            "the type itself: invariant `value != \"x\"` in `wire.server.Code` and not in \
             `demo.gate.Code`"
        ),
        "{detail}"
    );
}

#[test]
fn compose_compares_invariant_bounds() {
    let detail = Models::new().drift("Level", "Level", false);
    assert!(
        detail
            .contains("invariant `value >= 2` in `wire.server.Level` and not in `demo.gate.Level`"),
        "{detail}"
    );
    assert!(
        detail
            .contains("invariant `value >= 1` in `demo.gate.Level` and not in `wire.server.Level`"),
        "{detail}"
    );
}

#[test]
fn compose_compares_variant_handled_by() {
    let detail = Models::new().drift("KindHandled", "Kind", false);
    assert!(
        detail.ends_with(
            "variant `Closed`: attribute `handled_by` is `desk` where `wire.server.Kind` has \
             `robot`"
        ),
        "{detail}"
    );
}

#[test]
fn compose_compares_variant_terminal() {
    let detail = Models::new().drift("KindTerminal", "Kind", false);
    assert!(
        detail.ends_with(
            "variant `Closed`: attribute `terminal` is `false` where `wire.server.Kind` has \
             `true`"
        ),
        "{detail}"
    );
}

#[test]
fn compose_compares_variant_message_types() {
    let detail = Models::new().drift("KindMessages", "Kind", false);
    assert!(
        detail.contains(
            "variant `Closed`: attribute `request` is `CloseAsk` where `wire.server.Kind` has \
             `CloseRequest`"
        ),
        "{detail}"
    );
    assert!(
        detail.contains(
            "variant `Closed`: attribute `response` is `CloseAnswer` where `wire.server.Kind` \
             has `CloseResponse`"
        ),
        "{detail}"
    );
    // The same attributes and values conform.
    Models::new().conforms("Kind", "Kind", false);
}

#[test]
fn compose_compares_struct_invariants() {
    let detail = Models::new().drift("Window", "Window", false);
    assert!(
        detail.ends_with(
            "the type itself: invariant `low <= high` in `wire.server.Window` and not in \
             `demo.gate.Window`"
        ),
        "{detail}"
    );
}

#[test]
fn compose_compares_newtype_alphabet_and_prefix() {
    let models = Models::new();
    let detail = models.drift("Ref", "Ref", false);
    assert!(
        detail.contains("the type itself: alphabet `abcdr-` where `wire.server.Ref` has `abcr-`"),
        "{detail}"
    );
    assert!(
        detail.contains("the type itself: no prefix where `wire.server.Ref` has `r-`"),
        "{detail}"
    );
    // A reader with a wider alphabet and no prefix rejects nothing the owner sends.
    models.conforms("Ref", "Ref", true);
    // A narrower alphabet or a longer prefix does.
    let tight = models.drift("RefTight", "Ref", true);
    assert!(
        tight.contains("the type itself: alphabet `abr-` where `wire.server.Ref` has `abcr-`"),
        "{tight}"
    );
    assert!(
        tight.contains("the type itself: prefix `r-a` where `wire.server.Ref` has `r-`"),
        "{tight}"
    );
}

#[test]
fn an_invariant_spelled_differently_conforms() {
    Models::new().conforms("Floor", "Floor", false);
}

#[test]
fn a_reader_may_omit_an_owner_invariant_and_attribute() {
    let models = Models::new();
    models.conforms("Code", "Code", true);
    models.conforms("Window", "Window", true);
    models.conforms("KindLess", "Kind", true);
    models.conforms("KindTerminalOnly", "Kind", true);
    // A reader that declares an attribute agrees with the owner on it.
    let detail = models.drift("KindTerminal", "Kind", true);
    assert!(
        detail.contains(
            "variant `Closed`: attribute `terminal` is `false` where `wire.server.Kind` has `true`"
        ),
        "{detail}"
    );
    // Without `reader`, the omitted attributes are drift.
    let exact = models.drift("KindLess", "Kind", false);
    assert!(exact.contains("attributes"), "{exact}");
    assert!(exact.contains("`handled_by`"), "{exact}");
}

#[test]
fn a_reader_invariant_the_owner_lacks_is_drift() {
    let detail = Models::new().drift("Strict", "Code", true);
    assert!(
        detail.ends_with(
            "the type itself: invariant `value != \"y\"` in `demo.gate.Strict` and not in \
             `wire.server.Code`"
        ),
        "{detail}"
    );
}

#[test]
fn compose_still_refuses_structural_drift() {
    let models = Models::new();
    for (local, imported, says) in [
        ("KindWire", "Kind", "wire `shut`"),
        ("KindRenamed", "Kind", "`Shut`"),
        ("KindOpen", "Kind", "variants"),
        ("WindowText", "Window", "field `low`"),
        ("WindowLow", "Window", "field `high`"),
        ("Note", "Note", "presence"),
    ] {
        let detail = models.drift(local, imported, false);
        assert!(detail.contains(says), "{local}: {detail}");
    }
}

#[test]
fn v4_keeps_todays_result() {
    let models = Models::new();
    for (local, imported) in [
        ("Code", "Code"),
        ("Level", "Level"),
        ("KindHandled", "Kind"),
        ("KindTerminal", "Kind"),
        ("KindMessages", "Kind"),
        ("KindLess", "Kind"),
        ("Window", "Window"),
        ("Ref", "Ref"),
    ] {
        for format in [
            "ess-composition/2",
            "ess-composition/3",
            "ess-composition/4",
        ] {
            if let Err(diagnostics) = models.compose(format, local, imported, false) {
                panic!("{format}: `{local}` conforms structurally:\n{diagnostics}");
            }
        }
        models.drift(local, imported, false);
    }
    for format in ["ess-composition/3", "ess-composition/4"] {
        if let Err(diagnostics) = models.compose(format, "Strict", "Code", true) {
            panic!("{format}: a reader's invariant is not compared:\n{diagnostics}");
        }
    }
}
