//! Two inline models and the documents that compare them.

use std::fmt::Write;

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_service, EssIr};
use ess_composition::{
    compile, CompiledService, CompositionCode, CompositionDiagnostics, CompositionSpec,
    EssCompositionIr, ServiceKey, SourceDigest,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

pub fn model(files: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw =
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label} parses: {error}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        parsed.push((Source::new(*label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("model validates:\n{errors}"));
    compile_service(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("model resolves:\n{diagnostics}"))
}

fn system(name: &str, component: &str, commands: &[&str], events: &[&str]) -> String {
    let mut text = format!(
        "format: ess/31\nsystem: {name}\nversion: v1\n\ndomains:\n  - {name}.session\n\n\
         components:\n  - component: {component}\n    owns:\n      domains:\n        - {name}.session\n"
    );
    if !commands.is_empty() {
        text.push_str("    accepts:\n      commands:\n");
        for command in commands {
            let _ = writeln!(text, "        - {command}");
        }
    }
    if !events.is_empty() {
        text.push_str("    publishes:\n      events:\n");
        for event in events {
            let _ = writeln!(text, "        - {event}");
        }
    }
    text.push_str(if commands.is_empty() {
        "    reached_by: in_process\n"
    } else {
        "    reached_by: network\n"
    });
    text
}

/// The commands a component accepts and the events it publishes.
pub type Surface<'a> = (&'a [&'a str], &'a [&'a str]);

pub struct Pair {
    pub relay: EssIr,
    pub wire: EssIr,
}

impl Pair {
    pub fn new(relay_domain: &str, wire_domain: &str) -> Self {
        Self::with_surface(relay_domain, (&[], &[]), wire_domain, (&[], &[]))
    }

    pub fn with_surface(
        relay_domain: &str,
        relay: Surface<'_>,
        wire_domain: &str,
        wire: Surface<'_>,
    ) -> Self {
        Self {
            relay: model(&[
                (
                    "system.yaml",
                    &system("relay", "relay-component", relay.0, relay.1),
                ),
                ("domains/session.yaml", relay_domain),
            ]),
            wire: model(&[
                (
                    "system.yaml",
                    &system("wire", "wire-server", wire.0, wire.1),
                ),
                ("domains/session.yaml", wire_domain),
            ]),
        }
    }

    pub fn document(&self, format: &str, entries: &str) -> String {
        format!(
            "format: {format}\n\
             composition: relay-wire\n\
             services:\n  \
               - {{key: relay, system: relay, version: v1, source_digest: {}, component: relay-component}}\n  \
               - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
             {entries}",
            SourceDigest::of(&self.relay),
            SourceDigest::of(&self.wire),
        )
    }

    /// One `conformances:` list item; `extra` is further lines at the entry's indent.
    pub fn entry(local: &str, imported: &str, extra: &str) -> String {
        format!(
            "conformances:\n  \
               - local: {{service: relay, {local}}}\n    \
                 conforms_to: {{service: wire, {imported}}}\n{extra}"
        )
    }

    pub fn compose(&self, text: &str) -> Result<EssCompositionIr, CompositionDiagnostics> {
        let specification = CompositionSpec::from_yaml(text)
            .unwrap_or_else(|error| panic!("the document reads: {error}\n{text}"));
        let (relay, wire) = (key("relay"), key("wire"));
        compile(
            &specification,
            [
                CompiledService::new(&relay, &self.relay),
                CompiledService::new(&wire, &self.wire),
            ],
        )
    }

    pub fn composes(&self, text: &str) -> EssCompositionIr {
        self.compose(text)
            .unwrap_or_else(|diagnostics| panic!("composes:\n{diagnostics}\n{text}"))
    }

    /// The detail of the one `type_conformance_drift` the document is refused with.
    pub fn drift(&self, text: &str) -> String {
        match self.compose(text) {
            Ok(ir) => panic!("refused, not compiled:\n{}\n{text}", ir.to_canonical_json()),
            Err(diagnostics) => {
                let all = diagnostics.as_slice();
                assert_eq!(all.len(), 1, "{diagnostics}");
                assert_eq!(all[0].code(), CompositionCode::TypeConformanceDrift);
                all[0].detail().to_owned()
            }
        }
    }
}

pub fn key(value: &str) -> ServiceKey {
    ServiceKey::new(value).expect("valid service key")
}
