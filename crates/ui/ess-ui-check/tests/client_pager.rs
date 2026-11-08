//! A client-paged collection's `pager` offers sizes and names no params; a server-paged one still
//! names its page (story:a-client-paged-collection-offers-sizes-and-a-range).

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

const READ: &str = "pages/p/sections/list/reads";

/// A page whose `list` section reads `t.All` as written in `reads`.
fn document(reads: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: P
    state:
      at: {{type: integer, class: page_state, store: url, default: 1}}
      per: {{type: integer, class: page_state, store: url, default: 25}}
    sections:
      - {{name: list, component: collection, reads: {reads}, columns: [id]}}
"
    )
}

fn report(reads: &str) -> Report {
    check_source(
        &document(reads),
        "test.yaml",
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    )
}

/// The pager findings and every error.
fn errors(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error || finding.check.starts_with("pager"))
        .collect()
}

fn pager_paging(reads: &str) -> Vec<Finding> {
    report(reads)
        .findings
        .into_iter()
        .filter(|finding| finding.check == "pager_paging")
        .collect()
}

#[test]
fn a_client_pager_with_sizes_alone_is_clean() {
    for reads in [
        "{view: t.All, paging: client, pager: {sizes: [10, 25]}}",
        "{view: t.All, paging: client, pager: {}}",
        "{view: t.All, params: {q: state.at}, paging: client, pager: {sizes: [5]}}",
    ] {
        let report = report(reads);
        assert!(
            errors(&report).is_empty(),
            "{reads}: {:#?}",
            errors(&report)
        );
    }
}

#[test]
fn a_client_pager_refuses_page_and_size_by_name() {
    for (key, reads) in [
        (
            "page",
            "{view: t.All, params: {p: state.at}, paging: client, pager: {page: p, sizes: [10]}}",
        ),
        (
            "size",
            "{view: t.All, params: {n: state.per}, paging: client, pager: {size: n, sizes: [10]}}",
        ),
    ] {
        let found = pager_paging(reads);
        assert_eq!(found.len(), 1, "{reads}: {found:#?}");
        assert_eq!(found[0].path, READ, "{found:#?}");
        assert_eq!(found[0].severity, Severity::Error);
        assert!(
            found[0].message.contains(&format!("`{key}`"))
                && found[0].message.contains("paging: client"),
            "the refusal names `{key}`: {:?}",
            found[0].message
        );
    }
    let both = pager_paging(
        "{view: t.All, params: {p: state.at, n: state.per}, paging: client, pager: {page: p, size: n}}",
    );
    assert_eq!(both.len(), 2, "each key is refused by name: {both:#?}");
}

#[test]
fn a_server_pager_still_requires_its_page() {
    let found = pager_paging(
        "{view: t.All, params: {n: state.per}, paging: server, pager: {size: n, sizes: [10]}}",
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].path, READ);
    assert!(
        found[0].message.contains("`page`") && found[0].message.contains("paging: server"),
        "{:?}",
        found[0].message
    );
    let report = report(
        "{view: t.All, params: {p: state.at, n: state.per}, paging: server, pager: {page: p, size: n, sizes: [10, 20]}}",
    );
    assert!(errors(&report).is_empty(), "{:#?}", errors(&report));
}

#[test]
fn a_client_pager_sizes_are_still_checked() {
    let report = report("{view: t.All, paging: client, pager: {sizes: [0]}}");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.check == "pager_sizes"
                && finding.path == format!("{READ}/pager/sizes")),
        "{:#?}",
        report.findings
    );
}

#[test]
fn a_pager_on_another_paging_still_needs_server_or_client() {
    for paging in ["", "paging: cursor,", "paging: append,", "paging: none,"] {
        let reads = format!("{{view: t.All, {paging} pager: {{sizes: [10]}}}}");
        let found = pager_paging(&reads);
        assert_eq!(found.len(), 1, "{reads}: {found:#?}");
        assert!(
            found[0].message.contains("paging: server")
                && found[0].message.contains("paging: client"),
            "{:?}",
            found[0].message
        );
    }
}
