//! Error codes are explicit transport spelling, independent of semantic identity.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const SOURCE: &str = "format: ess/4\nsystem: desk\nversion: v1\ndomains: [desk.api]\ndomain: desk.api\nerrors:\n  - name: desk.api.Invalid\n    naming: {wire: bad_request}\n  - name: desk.api.Unavailable\n    naming: {wire: bad_request}\n";

#[test]
fn aliased_wire_codes_survive_source_and_compilation() {
    let raw = RawSpecFile::parse(SOURCE).expect("explicit error naming parses");
    let spec = Specification::assemble([(Source::new("errors.yaml"), raw)]).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("errors.yaml", SOURCE);
    let ir = compile(&spec, &sources).unwrap();
    assert_eq!(ir.errors().len(), 2);
    for error in ir.errors().values() {
        let json = serde_json::to_value(error).unwrap();
        assert_eq!(json["naming"]["wire"], "bad_request");
    }
    assert_ne!(ir.errors().keys().next(), ir.errors().keys().nth(1));
}

#[test]
fn absent_and_empty_naming_preserve_legacy_error_bytes() {
    use ess_domain::command::{ErrorSpec, RawErrorSpec};
    let old = r#"{"name":"desk.api.Invalid","summary":"Invalid"}"#;
    for text in [old.to_owned(), old.replace('}', ",\"naming\":{}}")] {
        let raw: RawErrorSpec = serde_json::from_str(&text).unwrap();
        let admitted = ErrorSpec::try_from(raw).unwrap();
        assert_eq!(serde_json::to_string(&admitted).unwrap(), old);
    }
}

#[test]
fn old_source_versions_refuse_explicit_error_naming() {
    for version in 1..=3 {
        let raw = RawSpecFile::parse(&SOURCE.replace("ess/4", &format!("ess/{version}"))).unwrap();
        let errors = Specification::assemble([(Source::new("errors.yaml"), raw)]).unwrap_err();
        assert!(errors
            .to_string()
            .contains("declared error naming requires specification format ess/4"));
    }
}

#[test]
fn compiled_legacy_error_has_exact_previous_shape() {
    let source = SOURCE
        .replace("    naming: {wire: bad_request}\n", "")
        .replace("ess/4", "ess/1");
    let spec = Specification::assemble([(
        Source::new("errors.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("errors.yaml", source);
    let ir = compile(&spec, &sources).unwrap();
    assert_eq!(
        serde_json::to_string(ir.errors().values().next().unwrap()).unwrap(),
        r#"{"name":"desk.api.Invalid","domain":"desk.api","fields":[]}"#
    );
}
