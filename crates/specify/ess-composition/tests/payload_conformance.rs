//! `ess-composition/5` conformance ends that name an event, an error or a command's input or
//! response: a payload is compared as an anonymous struct, against another payload or a struct
//! type (`docs/design/composition-type-conformance.md`).
//!
//! The fixture is two systems, `demo` (component `relay`) and `wire` (component `wire-server`,
//! and `wire-admin`, whose surface no document selects).

use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionDiagnostics, CompositionSpec, EssCompositionIr,
    ServiceKey, SourceDigest,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn fixture(set: &str, system: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(set)
        .join(system)
}

fn compiled(set: &str, system: &str) -> EssIr {
    let base = fixture(set, system);
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
    fn of(set: &str) -> Self {
        Self {
            demo: compiled(set, "demo"),
            wire: compiled(set, "wire"),
        }
    }

    fn new() -> Self {
        Self::of("payload-conformance")
    }

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

    /// A `/5` document whose one conformance is `local` against `imported`, flow-mapped ends.
    fn conformance(&self, local: &str, imported: &str, extra: &str) -> String {
        self.document(
            "ess-composition/5",
            &format!(
                "conformances:\n  \
                   - local: {{service: demo, {local}}}\n    \
                     conforms_to: {{service: wire, {imported}}}\n{extra}"
            ),
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

    fn composes(&self, text: &str) -> EssCompositionIr {
        self.compose(text)
            .unwrap_or_else(|diagnostics| panic!("composes:\n{diagnostics}\n{text}"))
    }

    /// The one refusal, as `(code, detail)`.
    fn refused(&self, text: &str) -> (String, String) {
        match self.compose(text) {
            Ok(ir) => panic!("refused, not compiled:\n{}", ir.to_canonical_json()),
            Err(diagnostics) => {
                let all = diagnostics.as_slice();
                assert_eq!(all.len(), 1, "{diagnostics}");
                (all[0].code().to_string(), all[0].detail().to_owned())
            }
        }
    }
}

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid fixture service key")
}

fn ir_json(ir: &EssCompositionIr) -> serde_json::Value {
    serde_json::from_str(&ir.to_canonical_json()).expect("composition IR is JSON")
}

#[test]
fn an_event_conforms_to_an_owner_event() {
    let models = Models::new();
    let ir = models.composes(&models.conformance(
        "event: demo.gate.Admitted",
        "event: wire.server.Subscribed",
        "",
    ));
    let json = ir_json(&ir);
    assert_eq!(json["format"], "ess-composition/5");
    assert_eq!(
        json["conformances"],
        serde_json::json!([{
            "local": {"service": "demo", "event": "demo.gate.Admitted"},
            "conforms_to": {"service": "wire", "event": "wire.server.Subscribed"},
        }]),
        "{json:#}"
    );
    // A conformance end is never added to the client surface.
    let without = models.composes(&models.document("ess-composition/5", ""));
    assert_eq!(
        ir.client_plan().to_canonical_json(),
        without.client_plan().to_canonical_json()
    );
}

#[test]
fn a_command_input_conforms() {
    let models = Models::new();
    let ir = models.composes(&models.conformance(
        "command: demo.gate.Admit, part: input",
        "command: wire.server.Subscribe, part: input",
        "",
    ));
    assert_eq!(
        ir_json(&ir)["conformances"][0]["conforms_to"],
        serde_json::json!({"service": "wire", "command": "wire.server.Subscribe", "part": "input"})
    );
}

#[test]
fn a_command_response_conforms() {
    let models = Models::new();
    models.composes(&models.conformance(
        "command: demo.gate.Ask, part: response",
        "command: wire.server.Ask, part: response",
        "",
    ));
    // A response against another command's input is compared too, and here agrees.
    models.composes(&models.conformance(
        "command: demo.gate.Ask, part: response",
        "command: wire.server.Subscribe, part: input",
        "",
    ));
}

#[test]
fn an_error_payload_conforms() {
    let models = Models::new();
    let ir = models.composes(&models.conformance(
        "error: demo.gate.Refused",
        "error: wire.server.Refused",
        "",
    ));
    assert_eq!(
        ir_json(&ir)["conformances"][0]["local"],
        serde_json::json!({"service": "demo", "error": "demo.gate.Refused"})
    );
}

#[test]
fn a_struct_type_conforms_to_an_event_payload() {
    let models = Models::new();
    models.composes(&models.conformance(
        "type: demo.gate.Subscription",
        "event: wire.server.Subscribed",
        "",
    ));
    // A newtype meets no payload.
    let (code, detail) = models.refused(&models.conformance(
        "type: demo.gate.Name",
        "event: wire.server.Subscribed",
        "",
    ));
    assert_eq!(code, "type_conformance_drift", "{detail}");
    assert!(
        detail.contains(
            "`demo.gate.Name` is a newtype where `wire.server.Subscribed` is an event payload"
        ),
        "{detail}"
    );
}

#[test]
fn a_drifting_event_field_is_reported() {
    let models = Models::new();
    let (code, detail) = models.refused(&models.conformance(
        "event: demo.gate.Counted",
        "event: wire.server.Subscribed",
        "",
    ));
    assert_eq!(code, "type_conformance_drift", "{detail}");
    assert!(
        detail.starts_with(
            "`demo` event `demo.gate.Counted` does not conform to `wire` event \
             `wire.server.Subscribed`"
        ),
        "{detail}"
    );
    assert!(detail.contains("field `channel`"), "{detail}");
    assert!(detail.contains("Integer"), "{detail}");
}

#[test]
fn a_reader_event_end_omits_a_field() {
    let models = Models::new();
    models.composes(&models.conformance(
        "event: demo.gate.Listed",
        "event: wire.server.Listed",
        "    reader: true\n",
    ));
    let (code, detail) = models.refused(&models.conformance(
        "event: demo.gate.Listed",
        "event: wire.server.Listed",
        "",
    ));
    assert_eq!(code, "type_conformance_drift", "{detail}");
    assert!(detail.contains("field `count`"), "{detail}");
}

#[test]
fn a_command_end_requires_its_part() {
    let models = Models::new();
    let text = models.conformance(
        "command: demo.gate.Admit",
        "command: wire.server.Subscribe, part: input",
        "",
    );
    let error = CompositionSpec::from_yaml(&text)
        .expect_err("a command end without `part` does not read")
        .to_string();
    assert!(error.contains("`part`"), "{error}");
    assert!(
        error.contains("`input`") && error.contains("`response`"),
        "{error}"
    );

    for (end, says) in [
        ("event: demo.gate.Admitted, part: input", "`part`"),
        (
            "event: demo.gate.Admitted, type: demo.gate.Channel",
            "exactly one of",
        ),
        ("part: input", "exactly one of"),
    ] {
        let text = models.conformance(end, "event: wire.server.Subscribed", "");
        let error = CompositionSpec::from_yaml(&text)
            .expect_err("a malformed end does not read")
            .to_string();
        assert!(error.contains(says), "{end}: {error}");
    }

    let text = models.conformance(
        "command: demo.gate.Admit, part: output",
        "command: wire.server.Subscribe, part: input",
        "",
    );
    let error = CompositionSpec::from_yaml(&text)
        .expect_err("a part outside the set does not read")
        .to_string();
    assert!(error.contains("unknown variant"), "{error}");
}

#[test]
fn a_response_end_on_a_command_without_response_is_refused() {
    let models = Models::new();
    let (code, detail) = models.refused(&models.conformance(
        "command: demo.gate.Ask, part: response",
        "command: wire.server.Subscribe, part: response",
        "",
    ));
    assert_eq!(code, "unresolved_semantic_reference", "{detail}");
    assert!(detail.contains("wire.server.Subscribe"), "{detail}");
    assert!(detail.contains("no `response`"), "{detail}");
}

#[test]
fn an_event_end_outside_the_surface_is_refused() {
    let models = Models::new();
    let (code, detail) = models.refused(&models.conformance(
        "event: demo.gate.Admitted",
        "event: wire.admin.Drained",
        "",
    ));
    assert_eq!(code, "reference_outside_component", "{detail}");
    assert!(detail.contains("wire.admin.Drained"), "{detail}");
    assert!(detail.contains("`wire-admin`"), "{detail}");
}

#[test]
fn an_older_composition_format_refuses_the_kind() {
    let models = Models::new();
    for format in [
        "ess-composition/2",
        "ess-composition/3",
        "ess-composition/4",
    ] {
        for (end, field) in [
            ("event: demo.gate.Admitted", "event"),
            ("error: demo.gate.Refused", "error"),
            ("command: demo.gate.Admit, part: input", "command"),
            ("type: demo.gate.Subscription, part: input", "part"),
        ] {
            let text = models.document(
                format,
                &format!(
                    "conformances:\n  \
                       - local: {{service: demo, {end}}}\n    \
                         conforms_to: {{service: wire, type: wire.server.Channel}}\n"
                ),
            );
            let error = CompositionSpec::from_yaml(&text)
                .expect_err("a payload end is unknown before /5")
                .to_string();
            assert!(
                error.contains(&format!(
                    "unknown field `{field}`, expected `service` or `type`"
                )),
                "{format} {end}: {error}"
            );
            assert!(
                error.contains("ess-composition/5"),
                "{format} {end}: {error}"
            );
        }
    }
}

/// The `/3` document of `tests/roles.rs`, a reference and two type conformances.
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
fn v4_documents_keep_their_bytes_and_results() {
    let models = Models::of("contract-role");
    let expected =
        std::fs::read_to_string(fixture("contract-role", "expected/composition-3.ir.json"))
            .expect("the /3 IR fixture is readable")
            .replace("<demo>", SourceDigest::of(&models.demo).as_str())
            .replace("<wire>", SourceDigest::of(&models.wire).as_str());
    for format in ["ess-composition/4", "ess-composition/5"] {
        let ir = models.composes(&models.document(format, COMPOSITION_3_REST));
        assert_eq!(
            ir.to_canonical_json(),
            expected.replace("ess-composition/3", format),
            "{format}"
        );
    }
    // `/5` keeps `roles` and `reader`, and a type end reads and writes as `{service, type}`.
    let roles = "roles:\n  \
                   - service: demo\n    \
                     plays: wire\n    \
                     commands:\n      \
                       - {local: demo.gate.Admit, stands: implements, contract: wire.server.Subscribe}\n";
    let four = models.composes(&models.document("ess-composition/4", roles));
    let five = models.composes(&models.document("ess-composition/5", roles));
    assert_eq!(
        five.to_canonical_json(),
        four.to_canonical_json()
            .replace("ess-composition/4", "ess-composition/5")
    );
    let drift = "roles:\n  \
                   - service: demo\n    \
                     plays: wire\n    \
                     commands:\n      \
                       - {local: demo.gate.Bare, stands: implements, contract: wire.server.Subscribe}\n";
    for format in ["ess-composition/4", "ess-composition/5"] {
        let diagnostics = models
            .compose(&models.document(format, drift))
            .expect_err("the drifting role is refused");
        assert!(
            diagnostics
                .as_slice()
                .iter()
                .all(|found| found.code().to_string() == "role_input_drift"),
            "{format}: {diagnostics}"
        );
    }
}
