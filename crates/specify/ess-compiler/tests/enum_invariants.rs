//! Owner-level enum admission and source attribution, before any runtime lowering.

use ess_compiler::resolve::{codes, diagnose_locating};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const HEADER: &str = "format: ess/1\nsystem: sample\nversion: v1\ndomains: [sample.work]\n";
const DOMAIN: &str = "\
domain: sample.work
types:
  - name: sample.work.Status
    kind: enum
    variants: [Ready, Stopped]
entities:
  - name: sample.work.Work
    identity: {name: id, type: Uuid}
    fields:
      - {name: status, type: sample.work.Status}
      - {name: optional_status, type: 'Optional<sample.work.Status>'}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
    invariants:
      - INVARIANT
views:
  - name: sample.work.WorkView
    source: sample.work.Work
    fields:
      - {name: status, type: sample.work.Status}
    filter: FILTER
";

fn assemble(
    invariant: &str,
    filter: &str,
) -> (
    Result<Specification, ess_primitives::error::ValidationErrors>,
    SourceMap,
) {
    let text = DOMAIN
        .replace("INVARIANT", invariant)
        .replace("FILTER", filter);
    let mut sources = SourceMap::new();
    sources.insert("system.yaml", HEADER);
    sources.insert("domains/work.yaml", &text);
    let result = Specification::assemble([
        (
            Source::new("system.yaml"),
            RawSpecFile::parse(HEADER).expect("header"),
        ),
        (
            Source::new("domains/work.yaml"),
            RawSpecFile::parse(&text).expect("domain"),
        ),
    ]);
    (result, sources)
}

#[test]
fn entity_and_view_enum_refusals_name_the_owner_source_and_vocabulary() {
    for (predicate, enumeration, values) in [
        (
            "status == Missing",
            "sample.work.Status",
            &["Ready", "Stopped"][..],
        ),
        (
            "optional_status != Missing",
            "sample.work.Status",
            &["Ready", "Stopped"][..],
        ),
        (
            "{status: {any_of: [Ready, Missing]}}",
            "sample.work.Status",
            &["Ready", "Stopped"][..],
        ),
        (
            "{status: {none_of: [Missing]}}",
            "sample.work.Status",
            &["Ready", "Stopped"][..],
        ),
        (
            "state == Missing",
            "sample.work.Work.State",
            &["Active"][..],
        ),
    ] {
        for entity_owner in [true, false] {
            let (invariant, filter, code) = if entity_owner {
                (
                    predicate,
                    "status == Ready",
                    codes::ENTITY_UNDECLARED_REFERENCE,
                )
            } else {
                (
                    "status == Ready",
                    predicate,
                    codes::VIEW_UNDECLARED_REFERENCE,
                )
            };
            let (result, sources) = assemble(invariant, filter);
            let errors = result.expect_err("a nonexistent variant must be refused");
            let diagnostics =
                diagnose_locating(&errors, &sources, &["system.yaml", "domains/work.yaml"]);
            assert_eq!(diagnostics.len(), 1, "{predicate}: {diagnostics}");
            let diagnostic = &diagnostics.as_slice()[0];
            assert_eq!(diagnostic.code, code, "{predicate}: {diagnostics}");
            let span = diagnostic.span.as_ref().expect("the owner has a source");
            assert_eq!(span.source, "domains/work.yaml");
            assert!(
                span.located.is_some(),
                "the declaration must be located: {span}"
            );
            let rendered = diagnostic.to_string();
            for expected in std::iter::once(enumeration)
                .chain(std::iter::once("Missing"))
                .chain(values.iter().copied())
            {
                assert!(
                    rendered.contains(expected),
                    "missing {expected}: {rendered}"
                );
            }
        }
    }
}

#[test]
fn declared_enum_values_still_compile_for_both_owners() {
    for predicate in [
        "status == Ready",
        "optional_status != Stopped",
        "{status: {any_of: [Ready, Stopped]}}",
        "{status: {none_of: [Stopped]}}",
        "state == Active",
    ] {
        let (result, sources) = assemble(predicate, predicate);
        let specification = result.unwrap_or_else(|errors| panic!("{predicate}: {errors}"));
        ess_compiler::compile(&specification, &sources)
            .unwrap_or_else(|errors| panic!("{predicate}: {errors}"));
    }
}

#[test]
fn declaration_prefixes_do_not_hide_or_impersonate_an_exact_declaration() {
    use ess_compiler::resolve::Locator;

    for key in ["name", "id", "component"] {
        let longer = format!(
            "{key}: sample.WorkView\n{key}: sample.Work.State\n{key}: sample.Work-extra\n{key}: sample.Work_extra\n"
        );
        let needle = format!("{key}: sample.Work");
        let mut sources = SourceMap::new();
        sources.insert("longer.yaml", longer);
        let labels = ["longer.yaml", "exact.yaml", "duplicate.yaml"];
        let span = Locator::new(&sources, &labels).span("owner", std::slice::from_ref(&needle));
        assert_eq!(span.source, Source::DOCUMENT, "a prefix is not the owner");
        assert!(span.located.is_none());

        sources.insert("exact.yaml", format!("{needle} # the actual declaration\n"));
        let span = Locator::new(&sources, &labels).span("owner", std::slice::from_ref(&needle));
        assert_eq!(span.source, "exact.yaml");
        assert_eq!(span.located.expect("exact declaration").line, 1);

        sources.insert("duplicate.yaml", format!("{needle}\n"));
        let span = Locator::new(&sources, &labels).span("owner", &[needle]);
        assert_eq!(span.source, Source::DOCUMENT, "duplicates remain ambiguous");
        assert!(span.located.is_none());
    }
}
