//! Adversary pass 2 on story:a-server-paged-collection-has-a-page-size-control: the fix round's
//! `pager_outside_collection` over every read a node holds, pagers inside widget uses, and
//! pagers whose two params cannot both be written.

use std::path::PathBuf;

use ess_ui_check::{check_source, Options, Report};

/// A page `p` with the given `state` entries and one section, and `widgets` at the root.
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
      at: {{type: integer, class: page_state, store: url, default: 1}}
      n: {{type: integer, class: page_state, store: url, default: 25}}
      q: {{type: {{record: {{page: integer, size: integer}}}}, class: page_state, store: url, default: {{page: 1, size: 25}}}}
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

const PAGER: &str = "params: {p: state.at}, paging: server, pager: {page: p}";

/// Every read a node can hold besides a collection's, each with a pager: the new check names
/// each one at its own `<read>/pager`.
#[test]
fn adversary_every_read_that_is_not_a_collections_reports_its_pager() {
    let cases = [
        (
            format!("{{name: s, component: record, reads: {{view: t.ById, {PAGER}}}, fields: [label]}}"),
            "pages/p/sections/s/reads/pager",
        ),
        (
            format!("{{name: s, component: metric, reads: {{view: t.Count, {PAGER}}}}}"),
            "pages/p/sections/s/reads/pager",
        ),
        (
            format!("{{name: s, component: chart, chart: bar, reads: {{view: t.Totals, {PAGER}}}}}"),
            "pages/p/sections/s/reads/pager",
        ),
        (
            format!("{{name: s, component: board, reads: {{view: t.Board, {PAGER}}}, widgets: {{rate: metric}}}}"),
            "pages/p/sections/s/reads/pager",
        ),
        (
            format!("{{name: s, component: graph_editor, reads: {{view: t.Graph, {PAGER}}}}}"),
            "pages/p/sections/s/reads/pager",
        ),
        (
            format!("{{name: s, component: graph_editor, reads: t.Graph, edges: {{reads: {{view: t.Edges, {PAGER}}}, from: from_id, to: to_id}}}}"),
            "pages/p/sections/s/edges/reads/pager",
        ),
        (
            format!("{{name: s, component: references, reads: {{view: t.UsedBy, {PAGER}}}}}"),
            "pages/p/sections/s/reads/pager",
        ),
        (
            format!("{{name: s, component: choice, reads: {{view: t.Items, {PAGER}}}, value: label}}"),
            "pages/p/sections/s/reads/pager",
        ),
        (
            format!("{{name: s, component: form, does: t.Save, loads: {{view: t.ById, {PAGER}}}}}"),
            "pages/p/sections/s/loads/pager",
        ),
        (
            format!("{{name: s, component: collection, reads: t.All, columns: [id], row_actions: [{{name: edit, does: t.Save, loads: {{view: t.ById, {PAGER}}}}}]}}"),
            "pages/p/sections/s/row_actions/edit/loads/pager",
        ),
    ];
    let mut missed = Vec::new();
    for (section, path) in &cases {
        let found = report(&document("", section));
        let paths = at(&found, "pager_outside_collection");
        if paths != [path.to_string()] {
            missed.push(format!(
                "{path}: pager_outside_collection at {paths:?}; findings {:#?}",
                found
                    .findings
                    .iter()
                    .map(|finding| format!(
                        "{} {} {}",
                        finding.check, finding.path, finding.message
                    ))
                    .collect::<Vec<_>>()
            ));
        }
    }
    assert!(missed.is_empty(), "{}", missed.join("\n"));
}

/// A widget declaration is checked at each use, on the expanded body: a record with a pager
/// inside a widget is reported at the use, and a collection's pager there is checked too.
#[test]
fn adversary_a_pager_inside_a_widget_use_is_checked_at_the_use() {
    let widgets = "widgets:
  w:
    summary: W
    params:
      page: {type: integer, required: true, note: n}
    body:
      - {name: one, component: record, reads: {view: t.ById, params: {p: args.page}, paging: server, pager: {page: p}}, fields: [label]}
      - {name: rows, component: collection, reads: {view: t.All, params: {p: args.page}, paging: client, pager: {page: p}}, columns: [id]}";
    let found = report(&document(
        widgets,
        "{name: summary, reads: t.ById, children: [{name: use, component: w, args: {page: state.at}}]}",
    ));
    assert_eq!(
        at(&found, "pager_outside_collection"),
        ["pages/p/sections/summary/children/use/body/one/reads/pager"],
        "{:#?}",
        found.findings
    );
    assert_eq!(
        at(&found, "pager_paging"),
        ["pages/p/sections/summary/children/use/body/rows/reads"],
        "{:#?}",
        found.findings
    );
}

/// Two distinct params bound to one state path: the select writes the size, then the page over
/// it, so the size chosen is never sent. The check accepts the pager.
#[test]
fn adversary_page_and_size_bound_to_one_state_path_is_refused() {
    let found = report(&document(
        "",
        "{name: s, component: collection, reads: {view: t.All, params: {page: state.at, size: state.at}, paging: server, pager: {page: page, size: size}}, columns: [id]}",
    ));
    assert!(
        !at(&found, "pager_params").is_empty(),
        "page and size both bound to state.at are accepted: {:#?}",
        found.findings
    );
}

/// The size bound inside the page's path (`page: state.q`, `size: state.q.size`): writing the
/// page replaces the record that holds the size.
#[test]
fn adversary_a_size_path_inside_the_page_path_is_refused() {
    let found = report(&document(
        "",
        "{name: s, component: collection, reads: {view: t.All, params: {page: state.q, size: state.q.size}, paging: server, pager: {page: page, size: size}}, columns: [id]}",
    ));
    assert!(
        !at(&found, "pager_params").is_empty(),
        "a size path under the page path is accepted: {:#?}",
        found.findings
    );
}
