//! Adversary pass 3 on story:a-server-paged-collection-has-a-page-size-control: the fix round's
//! `overlaps` refusal under `pager_params`, on prefixes that are not path segments, on one root
//! at different depths, on padded texts, on a shared param, and through a widget use.

use std::path::PathBuf;

use ess_ui_check::{check_source, Options, Report};

/// A page `p` with states sharing text prefixes and nested records, `widgets` at the root and
/// one section.
fn document(widgets: &str, section: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
{widgets}
pages:
  p:
    kind: detail_page
    title: P
    state:
      page: {{type: integer, class: page_state, store: url, default: 1}}
      pages: {{type: integer, class: page_state, store: url, default: 25}}
      at: {{type: integer, class: page_state, store: url, default: 1}}
      at_: {{type: integer, class: page_state, store: url, default: 25}}
      q: {{type: {{record: {{page: integer, pages: integer, size: integer}}}}, class: page_state, store: url, default: {{page: 1, pages: 25, size: 25}}}}
      qs: {{type: integer, class: page_state, store: url, default: 25}}
      r: {{type: {{record: {{a: {{record: {{b: integer}}}}, size: integer}}}}, class: page_state, store: url, default: {{a: {{b: 1}}, size: 25}}}}
    sections:
      - {section}
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

fn at(report: &Report, check: &str) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == check)
        .map(|finding| finding.path.clone())
        .collect()
}

fn collection(page: &str, size: &str) -> String {
    format!(
        "{{name: s, component: collection, reads: {{view: t.All, params: {{a: {page}, b: {size}}}, paging: server, pager: {{page: a, size: b}}}}, columns: [id]}}"
    )
}

/// Paths that share a text prefix but not a segment are apart; sibling members of one record,
/// at one depth or two, are apart too.
#[test]
fn adversary_text_prefixes_and_siblings_are_not_overlaps() {
    for (page, size) in [
        ("state.page", "state.pages"),
        ("state.pages", "state.page"),
        ("state.at", "state.at_"),
        ("state.q", "state.qs"),
        ("state.q.page", "state.q.pages"),
        ("state.q.page", "state.q.size"),
        ("state.r.a.b", "state.r.size"),
    ] {
        let found = report(&document("", &collection(page, size)));
        assert_eq!(
            at(&found, "pager_params"),
            Vec::<String>::new(),
            "page {page} and size {size} are apart: {:#?}",
            found.findings
        );
    }
}

/// One root at different depths, either way round, refused once at `.../reads/pager`.
#[test]
fn adversary_one_root_at_different_depths_is_refused_once() {
    for (page, size) in [
        ("state.q.page", "state.q"),
        ("state.q", "state.q.page"),
        ("state.r.a.b", "state.r"),
        ("state.r", "state.r.a.b"),
        ("state.r.a", "state.r.a.b"),
        ("state.at", "state.at"),
    ] {
        let found = report(&document("", &collection(page, size)));
        assert_eq!(
            at(&found, "pager_params"),
            ["pages/p/sections/s/reads/pager"],
            "page {page} and size {size} overlap: {:#?}",
            found.findings
        );
    }
}

/// Padding around a bound path is trimmed before the paths are compared.
#[test]
fn adversary_padded_texts_of_one_path_are_refused() {
    let found = report(&document("", &collection("' state.at'", "'state.at '")));
    assert_eq!(
        at(&found, "pager_params"),
        ["pages/p/sections/s/reads/pager"],
        "{:#?}",
        found.findings
    );
}

/// The size naming the page param is reported once, at `pager/size`, not again as an overlap.
#[test]
fn adversary_one_param_for_both_is_reported_once() {
    let found = report(&document(
        "",
        "{name: s, component: collection, reads: {view: t.All, params: {a: state.at}, paging: server, pager: {page: a, size: a}}, columns: [id]}",
    ));
    assert_eq!(
        at(&found, "pager_params"),
        ["pages/p/sections/s/reads/pager/size"],
        "{:#?}",
        found.findings
    );
}

/// A bracketed or numeric segment is no state path: reported at the param, no overlap row.
#[test]
fn adversary_bracketed_and_numeric_segments_are_no_overlap() {
    for size in ["'state.q[0]'", "state.q.0", "'state.q[\"page\"]'"] {
        let found = report(&document("", &collection("state.q", size)));
        assert_eq!(
            at(&found, "pager_params"),
            ["pages/p/sections/s/reads/pager/size"],
            "size {size}: {:#?}",
            found.findings
        );
    }
}

/// Through a widget use: the body names `args.page` and `args.size`, the use binds them to one
/// record and a member of it, and the expanded body is refused at the use.
#[test]
fn adversary_a_widget_use_binding_overlapping_paths_is_refused() {
    let widgets = "widgets:
  w:
    summary: W
    params:
      page: {type: integer, required: true, note: n}
      size: {type: integer, required: true, note: n}
    body:
      - {name: rows, component: collection, reads: {view: t.All, params: {a: args.page, b: args.size}, paging: server, pager: {page: a, size: b}}, columns: [id]}";
    let apart = report(&document(
        widgets,
        "{name: summary, reads: t.ById, children: [{name: use, component: w, args: {page: state.page, size: state.pages}}]}",
    ));
    assert_eq!(
        at(&apart, "pager_params"),
        Vec::<String>::new(),
        "{:#?}",
        apart.findings
    );
    let found = report(&document(
        widgets,
        "{name: summary, reads: t.ById, children: [{name: use, component: w, args: {page: state.q.page, size: state.q}}]}",
    ));
    assert_eq!(
        at(&found, "pager_params"),
        ["pages/p/sections/summary/children/use/body/rows/reads/pager"],
        "{:#?}",
        found.findings
    );
}
