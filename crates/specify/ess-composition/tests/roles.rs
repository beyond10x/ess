//! `ess-composition/4` contract roles: one imported service's component plays another's
//! (beyond10x/ess#525, `docs/design/composition-contract-roles.md`).
//!
//! The fixture is the issue's two systems, `demo` (component `relay`) and `wire` (component
//! `wire-server`), each with a second component whose commands are outside the selected surface.

use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionDiagnostics, CompositionSpec, EssCompositionIr,
    ServiceKey, SourceDigest,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn fixture(system: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/contract-role")
        .join(system)
}

fn compiled(system: &str) -> EssIr {
    let base = fixture(system);
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

/// Both fixture models, compiled once per test.
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

    /// A composition document of `format` importing `demo`/`relay` and `wire`/`wire-server` at
    /// their real digests, followed by `rest`.
    fn document(&self, format: &str, rest: &str) -> String {
        format!(
            "format: {format}\n\
             composition: demo-wire\n\
             services:\n  \
               - {{key: demo, system: demo, version: v1, source_digest: {}, component: relay}}\n  \
               - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
             {rest}",
            SourceDigest::of(&self.demo),
            SourceDigest::of(&self.wire),
        )
    }

    fn compose(&self, text: &str) -> Result<EssCompositionIr, CompositionDiagnostics> {
        let specification = CompositionSpec::from_yaml(text)
            .unwrap_or_else(|error| panic!("the document reads: {error}\n{text}"));
        self.compose_spec(&specification)
    }

    fn compose_spec(
        &self,
        specification: &CompositionSpec,
    ) -> Result<EssCompositionIr, CompositionDiagnostics> {
        let demo = key("demo");
        let wire = key("wire");
        compile(
            specification,
            [
                CompiledService::new(&demo, &self.demo),
                CompiledService::new(&wire, &self.wire),
            ],
        )
    }

    fn refused(&self, text: &str) -> CompositionDiagnostics {
        match self.compose(text) {
            Ok(ir) => panic!(
                "the composition is refused, not compiled:\n{}",
                ir.to_canonical_json()
            ),
            Err(diagnostics) => diagnostics,
        }
    }
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid fixture service key")
}

fn ir_json(ir: &EssCompositionIr) -> serde_json::Value {
    serde_json::from_str(&ir.to_canonical_json()).expect("composition IR is JSON")
}

/// Every refusal as `(code, detail)`, in emission order.
fn findings(diagnostics: &CompositionDiagnostics) -> Vec<(String, String)> {
    diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code().to_string(),
                diagnostic.detail().to_owned(),
            )
        })
        .collect()
}

/// The one finding whose detail starts with `at`, which must carry `code`.
fn finding_at(diagnostics: &CompositionDiagnostics, at: &str, code: &str) -> String {
    let all = findings(diagnostics);
    let matching: Vec<_> = all
        .iter()
        .filter(|(_, detail)| detail.starts_with(at))
        .collect();
    assert_eq!(matching.len(), 1, "one finding at `{at}`: {all:#?}");
    assert_eq!(matching[0].0, code, "{all:#?}");
    matching[0].1.clone()
}

#[test]
fn a_component_plays_a_contract_component_and_the_role_is_in_the_ir() {
    let models = Models::new();
    let ir = models
        .compose(&models.document(
            "ess-composition/4",
            "roles:\n  \
               - service: demo\n    \
                 plays: wire\n    \
                 commands:\n      \
                   - {local: demo.gate.Answer, stands: implements, contract: wire.server.Subscribe}\n      \
                   - {local: demo.gate.Admit, stands: gates, contract: wire.server.Subscribe}\n      \
                   - {local: demo.gate.Admit, stands: implements, contract: wire.server.Subscribe}\n",
        ))
        .unwrap_or_else(|diagnostics| panic!("the issue's role composes:\n{diagnostics}"));
    let json = ir_json(&ir);
    assert_eq!(json["format"], "ess-composition/4");
    let demo = SourceDigest::of(&models.demo).to_string();
    let wire = SourceDigest::of(&models.wire).to_string();
    assert_eq!(
        json["roles"],
        serde_json::json!([{
            "service": {
                "key": "demo",
                "system": "demo",
                "version": 1,
                "source_digest": demo,
                "component": "relay",
            },
            "plays": {
                "key": "wire",
                "system": "wire",
                "version": 1,
                "source_digest": wire,
                "component": "wire-server",
            },
            // Document order, not sorted: `Answer` stays ahead of `Admit`.
            "commands": [
                {"local": "demo.gate.Answer", "stands": "implements", "contract": "wire.server.Subscribe"},
                {"local": "demo.gate.Admit", "stands": "gates", "contract": "wire.server.Subscribe"},
                {"local": "demo.gate.Admit", "stands": "implements", "contract": "wire.server.Subscribe"},
            ],
        }]),
        "{json:#}"
    );

    // A role adds no operation: the client plan is the one the same services give without it.
    let without = models
        .compose(&models.document("ess-composition/4", ""))
        .expect("the bare composition compiles");
    assert_eq!(
        ir.client_plan().to_canonical_json(),
        without.client_plan().to_canonical_json()
    );
    assert!(
        ir_json(&without).get("roles").is_none(),
        "no roles, no key: {}",
        without.to_canonical_json()
    );

    // A role with no command entries says only that the component plays the other one.
    let bare = models
        .compose(&models.document(
            "ess-composition/4",
            "roles:\n  - {service: demo, plays: wire}\n",
        ))
        .unwrap_or_else(|diagnostics| panic!("a role without commands composes:\n{diagnostics}"));
    assert_eq!(
        ir_json(&bare)["roles"][0]["commands"],
        serde_json::json!([])
    );
}

#[test]
fn a_role_naming_an_unimported_service_is_refused() {
    let models = Models::new();
    let diagnostics = models.refused(&models.document(
        "ess-composition/4",
        "roles:\n  \
           - {service: demo, plays: nowhere}\n  \
           - {service: ghost, plays: wire}\n",
    ));
    let plays = finding_at(&diagnostics, "roles[0].plays", "unknown_role_service");
    assert!(plays.contains("`nowhere`"), "{plays}");
    let service = finding_at(&diagnostics, "roles[1].service", "unknown_role_service");
    assert!(service.contains("`ghost`"), "{service}");
    assert_eq!(findings(&diagnostics).len(), 2, "{diagnostics}");
}

#[test]
fn a_role_command_outside_its_selected_component_is_refused() {
    let models = Models::new();
    let diagnostics = models.refused(&models.document(
        "ess-composition/4",
        "roles:\n  \
           - service: demo\n    \
             plays: wire\n    \
             commands:\n      \
               - {local: demo.audit.Record, stands: gates, contract: wire.server.Subscribe}\n      \
               - {local: demo.gate.Admit, stands: gates, contract: wire.admin.Drain}\n      \
               - {local: demo.gate.Missing, stands: implements, contract: wire.server.Absent}\n",
    ));
    let local = finding_at(
        &diagnostics,
        "roles[0].commands[0].local",
        "reference_outside_component",
    );
    assert!(local.contains("demo.audit.Record"), "{local}");
    assert!(local.contains("`relay`"), "{local}");
    let contract = finding_at(
        &diagnostics,
        "roles[0].commands[1].contract",
        "reference_outside_component",
    );
    assert!(contract.contains("wire.admin.Drain"), "{contract}");
    assert!(contract.contains("`wire-server`"), "{contract}");
    let undeclared = finding_at(
        &diagnostics,
        "roles[0].commands[2].local",
        "unresolved_semantic_reference",
    );
    assert!(undeclared.contains("demo.gate.Missing"), "{undeclared}");
    finding_at(
        &diagnostics,
        "roles[0].commands[2].contract",
        "unresolved_semantic_reference",
    );
    assert_eq!(findings(&diagnostics).len(), 4, "{diagnostics}");
}

#[test]
fn an_implements_entry_whose_inputs_drift_is_refused() {
    let models = Models::new();
    let diagnostics = models.refused(&models.document(
        "ess-composition/4",
        "roles:\n  \
           - service: demo\n    \
             plays: wire\n    \
             commands:\n      \
               - {local: demo.gate.Admit, stands: implements, contract: wire.server.Subscribe}\n      \
               - {local: demo.gate.Count, stands: implements, contract: wire.server.Subscribe}\n      \
               - {local: demo.gate.Bare, stands: implements, contract: wire.server.Subscribe}\n      \
               - {local: demo.gate.Insist, stands: implements, contract: wire.server.Subscribe}\n",
    ));
    assert_eq!(findings(&diagnostics).len(), 3, "{diagnostics}");

    // Another type for the contract's field.
    let retyped = finding_at(&diagnostics, "roles[0].commands[1]", "role_input_drift");
    assert!(retyped.contains("demo.gate.Count"), "{retyped}");
    assert!(retyped.contains("wire.server.Subscribe"), "{retyped}");
    assert!(retyped.contains("field `channel`"), "{retyped}");
    assert!(retyped.contains("Integer"), "{retyped}");

    // The contract's field missing, and a required one the contract's caller never sends.
    let bare = finding_at(&diagnostics, "roles[0].commands[2]", "role_input_drift");
    assert!(bare.contains("field `channel`"), "{bare}");
    assert!(bare.contains("field `topic`"), "{bare}");

    // An extra input read as `null` when absent: the caller never sends the key.
    let insist = finding_at(&diagnostics, "roles[0].commands[3]", "role_input_drift");
    assert!(insist.contains("field `note`"), "{insist}");
    assert!(insist.contains("null_when_absent"), "{insist}");
}

#[test]
fn a_gates_entry_is_recorded_without_an_input_comparison() {
    let models = Models::new();
    let ir = models
        .compose(&models.document(
            "ess-composition/4",
            "roles:\n  \
               - service: demo\n    \
                 plays: wire\n    \
                 commands:\n      \
                   - {local: demo.gate.Count, stands: gates, contract: wire.server.Subscribe}\n      \
                   - {local: demo.gate.Bare, stands: gates, contract: wire.server.Subscribe}\n",
        ))
        .unwrap_or_else(|diagnostics| panic!("a gate compares no inputs:\n{diagnostics}"));
    assert_eq!(
        ir_json(&ir)["roles"][0]["commands"],
        serde_json::json!([
            {"local": "demo.gate.Count", "stands": "gates", "contract": "wire.server.Subscribe"},
            {"local": "demo.gate.Bare", "stands": "gates", "contract": "wire.server.Subscribe"},
        ])
    );
}

#[test]
fn a_stands_value_outside_the_set_is_refused() {
    let models = Models::new();
    for stands in ["precedes", "Implements", "\"\""] {
        let text = models.document(
            "ess-composition/4",
            &format!(
                "roles:\n  \
                   - service: demo\n    \
                     plays: wire\n    \
                     commands:\n      \
                       - {{local: demo.gate.Admit, stands: {stands}, contract: wire.server.Subscribe}}\n"
            ),
        );
        let error = CompositionSpec::from_yaml(&text)
            .expect_err("a `stands` outside the set does not read")
            .to_string();
        assert!(error.contains("unknown variant"), "{stands}: {error}");
        assert!(
            error.contains("`implements`") && error.contains("`gates`"),
            "{stands}: {error}"
        );
    }
}

#[test]
fn a_role_that_plays_itself_or_repeats_is_refused() {
    let models = Models::new();
    let diagnostics = models.refused(&models.document(
        "ess-composition/4",
        "roles:\n  \
           - {service: demo, plays: demo}\n  \
           - service: demo\n    \
             plays: wire\n    \
             commands:\n      \
               - {local: demo.gate.Admit, stands: gates, contract: wire.server.Subscribe}\n      \
               - {local: demo.gate.Admit, stands: gates, contract: wire.server.Subscribe}\n  \
           - {service: demo, plays: wire}\n",
    ));
    let itself = finding_at(&diagnostics, "roles[0]", "conflicting_role");
    assert!(itself.contains("`demo`"), "{itself}");
    finding_at(&diagnostics, "roles[1].commands[1]", "conflicting_role");
    let repeated = finding_at(&diagnostics, "roles[2]", "conflicting_role");
    assert!(repeated.contains("roles[1]"), "{repeated}");
    assert_eq!(findings(&diagnostics).len(), 3, "{diagnostics}");
}

/// A `/3` document: a reference and a reader conformance, the constructs `/4` keeps.
const COMPOSITION_3_REST: &str = "references:
  - service: wire
    semantic: {kind: command, name: wire.server.Subscribe}
conformances:
  - local: {service: demo, type: demo.gate.Channel}
    conforms_to: {service: wire, type: wire.server.Channel}
  - local: {service: demo, type: demo.gate.Channel}
    conforms_to: {service: wire, type: wire.server.Channel}
    reader: true
";

#[test]
fn composition_3_documents_keep_their_ir_bytes_under_the_4_reader() {
    let models = Models::new();
    let ir = models
        .compose(&models.document("ess-composition/3", COMPOSITION_3_REST))
        .unwrap_or_else(|diagnostics| panic!("the /3 document composes:\n{diagnostics}"));
    // Written by the `/3` reader of the base this format was added on, with the two digests
    // replaced by placeholders so a change to compiled-model bytes elsewhere does not move it.
    let expected = std::fs::read_to_string(fixture("expected/composition-3.ir.json"))
        .expect("the /3 IR fixture is readable")
        .replace("<demo>", SourceDigest::of(&models.demo).as_str())
        .replace("<wire>", SourceDigest::of(&models.wire).as_str());
    assert_eq!(ir.to_canonical_json(), expected);

    // `/4` admits the same document and changes only the marker it echoes.
    let four = models
        .compose(&models.document("ess-composition/4", COMPOSITION_3_REST))
        .unwrap_or_else(|diagnostics| panic!("the /4 document composes:\n{diagnostics}"));
    assert_eq!(
        four.to_canonical_json(),
        expected.replace("ess-composition/3", "ess-composition/4")
    );

    // An earlier document carrying `roles`, even empty, is refused when read, as before `/4`.
    for format in [
        "ess-composition/1",
        "ess-composition/2",
        "ess-composition/3",
    ] {
        for roles in ["roles: []\n", "roles:\n  - {service: demo, plays: wire}\n"] {
            let text = models.document(format, roles);
            let error = CompositionSpec::from_yaml(&text)
                .expect_err("`roles` is unknown before /4")
                .to_string();
            assert!(error.contains("unknown field `roles`"), "{format}: {error}");
        }
    }
}

/// Under `/4` an authored `null` (`roles:`, `roles: ~`, `roles: null`, JSON `null`) is the absent
/// key: it reads, writes no `roles`, and compiles to the bytes of the document without the key.
#[test]
fn a_composition_4_document_with_null_roles_reads_as_without_the_key() {
    let models = Models::new();
    let without = models.document("ess-composition/4", "");
    let read_without = CompositionSpec::from_yaml(&without).expect("the /4 document reads");
    let compiled_without = models
        .compose(&without)
        .unwrap_or_else(|diagnostics| panic!("the /4 document composes:\n{diagnostics}"))
        .to_canonical_json();
    for roles in ["roles:\n", "roles: ~\n", "roles: null\n"] {
        let text = models.document("ess-composition/4", roles);
        let read = CompositionSpec::from_yaml(&text)
            .unwrap_or_else(|error| panic!("/4 with {roles:?} reads: {error}"));
        assert!(read.roles().is_empty(), "{roles:?}");
        assert_eq!(read, read_without, "{roles:?}");
        assert_eq!(
            read.to_canonical_json(),
            read_without.to_canonical_json(),
            "{roles:?}"
        );
        let compiled = models
            .compose(&text)
            .unwrap_or_else(|diagnostics| panic!("/4 with {roles:?} composes:\n{diagnostics}"))
            .to_canonical_json();
        assert_eq!(compiled, compiled_without, "{roles:?}");
    }
    let mut json: serde_json::Value =
        serde_json::from_str(&read_without.to_canonical_json()).expect("canonical JSON");
    json["roles"] = serde_json::Value::Null;
    let read =
        CompositionSpec::from_json(&json.to_string()).expect("/4 JSON with null roles reads");
    assert_eq!(read, read_without);
}

/// `ess-composition/1` does not admit `conformances`, even `[]`; an authored `null`
/// (`conformances:`, `conformances: ~`, JSON `null`) is a document carrying the key, and is
/// refused as `[]` is, rather than read as the absent key.
#[test]
fn an_ess_composition_1_document_carrying_a_null_conformances_key_is_refused() {
    let models = Models::new();
    let refused_as_v1 = |diagnostics: &CompositionDiagnostics, spelling: &str| {
        assert!(
            findings(diagnostics).iter().any(|(code, detail)| {
                code == "unsupported_format" && detail.contains("`conformances`")
            }),
            "/1 with {spelling} is refused as carrying `conformances`:\n{diagnostics}"
        );
    };
    for conformances in [
        "conformances:\n",
        "conformances: ~\n",
        "conformances: null\n",
    ] {
        let text = models.document("ess-composition/1", conformances);
        refused_as_v1(&models.refused(&text), conformances);
    }
    let without = CompositionSpec::from_yaml(&models.document("ess-composition/1", ""))
        .expect("the /1 document reads");
    let mut json: serde_json::Value =
        serde_json::from_str(&without.to_canonical_json()).expect("canonical JSON");
    json["conformances"] = serde_json::Value::Null;
    let read = CompositionSpec::from_json(&json.to_string())
        .expect("the DTO reads; the compiler checks the marker");
    match models.compose_spec(&read) {
        Ok(ir) => panic!(
            "/1 JSON with `conformances: null` is refused, not compiled:\n{}",
            ir.to_canonical_json()
        ),
        Err(diagnostics) => refused_as_v1(&diagnostics, "JSON null"),
    }
}

/// Under `/2` and `/3` an authored `null` `conformances` stays the absent key: it reads, writes no
/// `conformances`, and compiles to the bytes of the document without the key.
#[test]
fn ess_composition_2_and_3_documents_with_null_conformances_read_as_without_the_key() {
    let models = Models::new();
    for format in ["ess-composition/2", "ess-composition/3"] {
        let without = models.document(format, "");
        let read_without = CompositionSpec::from_yaml(&without).expect("the document reads");
        let compiled_without = models
            .compose(&without)
            .unwrap_or_else(|diagnostics| panic!("{format} composes:\n{diagnostics}"))
            .to_canonical_json();
        for conformances in [
            "conformances:\n",
            "conformances: ~\n",
            "conformances: null\n",
        ] {
            let text = models.document(format, conformances);
            let read = CompositionSpec::from_yaml(&text)
                .unwrap_or_else(|error| panic!("{format} with {conformances:?} reads: {error}"));
            assert!(read.conformances().is_empty(), "{format} {conformances:?}");
            assert_eq!(read, read_without, "{format} {conformances:?}");
            assert_eq!(
                read.to_canonical_json(),
                read_without.to_canonical_json(),
                "{format} {conformances:?}"
            );
            let compiled = models
                .compose(&text)
                .unwrap_or_else(|diagnostics| {
                    panic!("{format} with {conformances:?} composes:\n{diagnostics}")
                })
                .to_canonical_json();
            assert_eq!(compiled, compiled_without, "{format} {conformances:?}");
        }
        let mut json: serde_json::Value =
            serde_json::from_str(&read_without.to_canonical_json()).expect("canonical JSON");
        json["conformances"] = serde_json::Value::Null;
        let read = CompositionSpec::from_json(&json.to_string())
            .unwrap_or_else(|error| panic!("{format} JSON with null conformances reads: {error}"));
        assert_eq!(read, read_without, "{format}");
    }
}
