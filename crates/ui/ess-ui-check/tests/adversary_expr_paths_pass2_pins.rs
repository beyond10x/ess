//! Adversarial pass 2: what the fix keeps. A widget argument under a real root is still resolved,
//! a shell's overlay that copies a page's with `same_as` is walked with the copied kind, and an
//! expression that uses `contains` is reported by no `filter_*` rule beside `expr_paths`.

use std::path::PathBuf;

use ess_ui_check::{check_source, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run(text: &str) -> Report {
    check_source(text, "test.yaml", &crate_dir(), None, &Options::default())
}

fn paths<'a>(report: &'a Report, check: &str) -> Vec<&'a str> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == check)
        .map(|finding| finding.path.as_str())
        .collect()
}

fn loads(report: &Report) {
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.check == "document_loads"),
        "{:#?}",
        report.findings
    );
}

const WIDGET: &str = "widgets:\n  card:\n    summary: C\n    \
params: {who: {type: string, required: true, note: n}}\n    \
body: [{name: t, primitive: text, text: args.who}]\n";

fn page(children: &str, shell_overlays: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}, overlays: {{{shell_overlays}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n{WIDGET}\
         pages:\n  p:\n    kind: detail_page\n    title: P\n    \
         state: {{mine: {{type: string, class: page_state}}}}\n    \
         overlays: {{edit: {{kind: drawer, component: form, does: t.Edit, fields: [name]}}}}\n    \
         sections:\n      \
         - {{name: summary, component: collection, reads: t.Rows, columns: [id], \
         row_actions: [{{name: s, label: S, as: toggle, sets: {{state.mine: toggle}}, \
         checked: \"row.tags contains state.nosuch_checked\"}}], \
         item: [{children}]}}\n"
    )
}

#[test]
fn an_argument_under_a_real_root_is_still_resolved() {
    let report = run(&page(
        "{name: w1, component: card, args: {who: state.mine}}, \
         {name: w2, component: card, args: {who: state.nosuch_arg}}, \
         {name: w3, component: card, args: {who: section.nosuch_arg.row}}",
        "",
    ));
    loads(&report);
    assert_eq!(
        paths(&report, "expr_paths"),
        [
            "pages/p/sections/summary/item/w2",
            "pages/p/sections/summary/row_actions/s"
        ],
        "{:#?}",
        report.findings
    );
    assert_eq!(
        paths(&report, "section_refs"),
        ["pages/p/sections/summary/item/w3"],
        "{:#?}",
        report.findings
    );
    for rule in ["filter_expr", "filter_roots", "filter_paths"] {
        assert!(
            paths(&report, rule).is_empty(),
            "{rule}: {:#?}",
            report.findings
        );
    }
}

#[test]
fn a_shell_overlay_that_copies_a_page_overlay_is_walked() {
    let report = run(&page(
        "{name: w1, component: card, args: {who: state.mine}}",
        "copy: {same_as: p.edit, title: C, fields: [{field: z, visible: state.nosuch_copy}]}",
    ));
    loads(&report);
    assert!(
        paths(&report, "expr_paths").contains(&"shells/app/overlays/copy/fields/z"),
        "{:#?}",
        report.findings
    );
}
