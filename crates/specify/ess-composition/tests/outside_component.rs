//! `reference_outside_component` names the domain and service a name is in, and the remedy for
//! its kind: which components export it, and how the selected component would.
//!
//! The fixture is two systems, `demo` (components `relay` and `auditor`, and a domain `demo.loose`
//! that no component owns) and `wire`. Every document selects `relay`.

use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{compile, CompiledService, CompositionSpec, ServiceKey, SourceDigest};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn compiled(system: &str) -> EssIr {
    let base: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/outside-component")
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

fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid fixture service key")
}

/// The one `reference_outside_component` detail a reference to `kind` `name` gets under `format`.
fn refusal(format: &str, kind: &str, name: &str) -> String {
    let demo = compiled("demo");
    let wire = compiled("wire");
    let text = format!(
        "format: {format}\n\
         composition: demo-wire\n\
         services:\n  \
           - {{key: demo, system: demo, version: v1, source_digest: {}, component: relay}}\n  \
           - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
         references:\n  \
           - service: demo\n    \
             semantic: {{kind: {kind}, name: {name}}}\n",
        SourceDigest::of(&demo),
        SourceDigest::of(&wire),
    );
    let specification = CompositionSpec::from_yaml(&text)
        .unwrap_or_else(|error| panic!("the document reads: {error}\n{text}"));
    let (demo_key, wire_key) = (key("demo"), key("wire"));
    let diagnostics = match compile(
        &specification,
        [
            CompiledService::new(&demo_key, &demo),
            CompiledService::new(&wire_key, &wire),
        ],
    ) {
        Ok(ir) => panic!("`{name}` is refused:\n{}", ir.to_canonical_json()),
        Err(diagnostics) => diagnostics,
    };
    let all = diagnostics.as_slice();
    assert_eq!(all.len(), 1, "{diagnostics}");
    assert_eq!(
        all[0].code().to_string(),
        "reference_outside_component",
        "{diagnostics}"
    );
    assert_eq!(all[0].service().map(ServiceKey::as_str), Some("demo"));
    all[0].detail().to_owned()
}

#[test]
fn the_refusal_names_domain_and_service() {
    let detail = refusal("ess-composition/4", "type", "demo.loose.Note");
    assert!(
        detail.starts_with(
            "service `demo` resolves `type demo.loose.Note`, but selected component `relay` \
             does not export it"
        ),
        "{detail}"
    );
    assert!(
        detail.contains("`demo.loose.Note` is in domain `demo.loose` of service `demo`"),
        "{detail}"
    );
    assert!(
        detail.contains("no component owns `demo.loose`: add it to `relay`'s `owns`"),
        "{detail}"
    );
}

#[test]
fn the_refusal_suggests_an_owner() {
    let detail = refusal("ess-composition/4", "type", "demo.audit.Entry");
    assert!(
        detail.contains("`demo.audit.Entry` is in domain `demo.audit` of service `demo`"),
        "{detail}"
    );
    assert!(
        detail.contains(
            "component `auditor` owns it: select it, or add `demo.audit` to `relay`'s `owns`"
        ),
        "{detail}"
    );
}

#[test]
fn a_command_outside_accepts_names_the_accepting_component() {
    let detail = refusal("ess-composition/4", "command", "demo.audit.Record");
    assert!(
        detail.contains("`demo.audit.Record` is in domain `demo.audit` of service `demo`"),
        "{detail}"
    );
    assert!(
        detail.contains(
            "component `auditor` accepts it: select it, or add it to `relay`'s `accepts`"
        ),
        "{detail}"
    );
    assert!(!detail.contains("`owns`"), "{detail}");
}

#[test]
fn an_event_outside_the_surface_names_its_publisher() {
    // Emitted by a command `auditor` accepts.
    let emitted = refusal("ess-composition/4", "event", "demo.audit.Recorded");
    assert!(
        emitted.contains("`demo.audit.Recorded` is in domain `demo.audit` of service `demo`"),
        "{emitted}"
    );
    assert!(
        emitted.contains(
            "component `auditor` publishes it or accepts a command that emits it: select it"
        ),
        "{emitted}"
    );
    assert!(!emitted.contains("`owns`"), "{emitted}");
    // Published by `auditor`, emitted by no command.
    let published = refusal("ess-composition/4", "event", "demo.audit.Flagged");
    assert!(
        published.contains(
            "component `auditor` publishes it or accepts a command that emits it: select it"
        ),
        "{published}"
    );
}

#[test]
fn an_error_outside_the_surface_names_the_component_raising_it() {
    let detail = refusal("ess-composition/4", "error", "demo.audit.Rejected");
    assert!(
        detail.contains("`demo.audit.Rejected` is in domain `demo.audit` of service `demo`"),
        "{detail}"
    );
    assert!(
        detail.contains("component `auditor` accepts a command that raises it: select it"),
        "{detail}"
    );
    assert!(!detail.contains("`owns`"), "{detail}");
}

#[test]
fn the_v1_hint_is_kept() {
    let detail = refusal("ess-composition/1", "type", "demo.gate.Unused");
    assert!(
        detail.starts_with(
            "service `demo` resolves `type demo.gate.Unused`, but selected component `relay` \
             does not export it; a type its owned domains declare is referenceable under \
             ess-composition/2"
        ),
        "{detail}"
    );
    assert!(
        detail.contains("`demo.gate.Unused` is in domain `demo.gate` of service `demo`"),
        "{detail}"
    );
    // `relay` owns the domain: no remedy tells it to add the domain it already owns.
    assert!(!detail.contains("to `relay`'s `owns`"), "{detail}");
}
