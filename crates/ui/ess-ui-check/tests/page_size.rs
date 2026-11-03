//! A read's `pager` names params of that read, each bound to a `state.<path>`, on a server-paged
//! read, with positive page sizes (story:a-server-paged-collection-has-a-page-size-control).

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity, CHECKS};

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

fn report(text: &str) -> Report {
    check_source(
        text,
        "test.yaml",
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    )
}

fn tripped<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == id)
        .collect()
}

/// Asserts `reads` trips `id` at `path` as an error, and returns the finding.
fn trips(reads: &str, id: &str, path: &str) -> Finding {
    let report = report(&document(reads));
    let finding = tripped(&report, id)
        .into_iter()
        .find(|finding| finding.path == path)
        .unwrap_or_else(|| {
            panic!(
                "no `{id}` finding at `{path}` for {reads}; findings: {:#?}",
                report.findings
            )
        })
        .clone();
    assert!(
        CHECKS.iter().any(|check| check.id == id),
        "`{id}` is a named check"
    );
    assert_eq!(finding.severity, Severity::Error, "{finding:?}");
    finding
}

/// Asserts `reads` trips none of the pager checks and no error at all.
fn clean(reads: &str) {
    let report = report(&document(reads));
    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error || finding.check.starts_with("pager"))
        .collect();
    assert!(errors.is_empty(), "{reads}: {errors:#?}");
}

const GOOD: &str =
    "{view: t.All, params: {p: state.at, n: state.per}, paging: server, pager: {page: p, size: n, sizes: [10, 20]}}";

#[test]
fn a_pager_without_server_paging_is_an_error() {
    clean(GOOD);
    clean("{view: t.All, params: {p: state.at}, paging: server, pager: {page: p}}");
    for paging in [
        "",
        "paging: client,",
        "paging: cursor,",
        "paging: append,",
        "paging: none,",
    ] {
        let reads =
            format!("{{view: t.All, params: {{p: state.at}}, {paging} pager: {{page: p}}}}");
        let finding = trips(&reads, "pager_paging", READ);
        assert!(finding.message.contains("paging: server"), "{finding:?}");
    }
}

#[test]
fn a_pager_param_must_be_a_read_param_bound_to_state() {
    // `pager.page` or `pager.size` naming no param of the read.
    trips(
        "{view: t.All, params: {p: state.at}, paging: server, pager: {page: page}}",
        "pager_params",
        &format!("{READ}/pager/page"),
    );
    trips(
        "{view: t.All, params: {p: state.at}, paging: server, pager: {page: p, size: size}}",
        "pager_params",
        &format!("{READ}/pager/size"),
    );
    // A param bound to something other than `state.<path>`.
    for bound in ["params.id", "1", "'state.at == 1'", "row.page", "state"] {
        trips(
            &format!("{{view: t.All, params: {{p: {bound}, n: state.per}}, paging: server, pager: {{page: p, size: n}}}}"),
            "pager_params",
            &format!("{READ}/pager/page"),
        );
        trips(
            &format!("{{view: t.All, params: {{p: state.at, n: {bound}}}, paging: server, pager: {{page: p, size: n}}}}"),
            "pager_params",
            &format!("{READ}/pager/size"),
        );
    }
    // The page and the size are two params, so choosing a size cannot overwrite the page.
    trips(
        "{view: t.All, params: {p: state.at}, paging: server, pager: {page: p, size: p}}",
        "pager_params",
        &format!("{READ}/pager/size"),
    );
    // Nothing is recognised by its name: a read param named `page` is no pager param unless
    // the pager names it.
    clean("{view: t.All, params: {page: state.at, size: state.per}, paging: server, pager: {page: page, size: size}}");
}

#[test]
fn pager_sizes_are_positive() {
    trips(
        "{view: t.All, params: {p: state.at, n: state.per}, paging: server, pager: {page: p, size: n, sizes: []}}",
        "pager_sizes",
        &format!("{READ}/pager/sizes"),
    );
    for size in ["0", "-5"] {
        trips(
            &format!("{{view: t.All, params: {{p: state.at, n: state.per}}, paging: server, pager: {{page: p, size: n, sizes: [25, {size}]}}}}"),
            "pager_sizes",
            &format!("{READ}/pager/sizes"),
        );
    }
    clean("{view: t.All, params: {p: state.at, n: state.per}, paging: server, pager: {page: p, size: n, sizes: [1]}}");
}
