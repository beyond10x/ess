//! Native HTTP codes come from explicit error naming, with legacy qualified-name fallback.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

const SOURCE: &str = r"
format: ess/4
system: desk
version: v1
domain: desk.api
errors:
  - name: desk.api.Invalid
    naming: {wire: bad_request}
  - name: desk.api.Unavailable
    naming: {wire: bad_request}
  - name: desk.api.Legacy
commands:
  - name: desk.api.First
    input: []
    outcomes: [{name: refused, error: desk.api.Invalid}]
  - name: desk.api.Second
    input: []
    outcomes: [{name: refused, error: desk.api.Unavailable}]
  - name: desk.api.Third
    input: []
    outcomes: [{name: refused, error: desk.api.Legacy}]
components:
  - component: api-service
    owns: {domains: [desk.api]}
    accepts: {commands: [desk.api.First, desk.api.Second, desk.api.Third]}
    reached_by: network
";

fn emitted(target: Target) -> String {
    emitted_source(target, SOURCE)
}

fn emitted_source(target: Target, source: &str) -> String {
    let spec =
        Specification::assemble([(Source::new("api.yaml"), RawSpecFile::parse(source).unwrap())])
            .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("api.yaml", source);
    let ir = compile(&spec, &sources).unwrap();
    assert_eq!(ir.errors().len(), 3);
    let generated = synthesize_for(&ir, target).unwrap();
    generated
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn rust_http_uses_aliases_without_changing_legacy_fallback() {
    let code = emitted(Target::Rust);
    assert_eq!(
        code.matches("json::push_text(&mut body, \"bad_request\");")
            .count(),
        2
    );
    assert_eq!(
        code.matches("json::push_text(&mut body, \"desk.api.Legacy\");")
            .count(),
        1
    );
    assert!(!code.contains("json::push_text(&mut body, \"desk.api.Invalid\");"));
    assert!(!code.contains("json::push_text(&mut body, \"desk.api.Unavailable\");"));
}

#[test]
fn go_http_uses_aliases_without_changing_legacy_fallback() {
    let code = emitted(Target::Go);
    assert_eq!(code.matches("body[\"error\"] = \"bad_request\"").count(), 2);
    assert_eq!(
        code.matches("body[\"error\"] = \"desk.api.Legacy\"")
            .count(),
        1
    );
    assert!(!code.contains("body[\"error\"] = \"desk.api.Invalid\""));
    assert!(!code.contains("body[\"error\"] = \"desk.api.Unavailable\""));
}

#[test]
fn go_http_escapes_arbitrary_declared_codes_as_valid_go_strings() {
    let source = SOURCE.replace("wire: bad_request", r#"wire: "\0\"\\""#);
    let code = emitted_source(Target::Go, &source);
    assert_eq!(code.matches(r#"body["error"] = "\u0000\"\\""#).count(), 2);
}
