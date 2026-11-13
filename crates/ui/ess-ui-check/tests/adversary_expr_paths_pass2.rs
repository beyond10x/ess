//! Adversarial pass 2 on `ess ui check` resolving the root and the section of every binding path:
//! a nested node whose `primitive` comes from a page kind, and a widget argument substituted into
//! an `expr` position.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run(text: &str) -> Report {
    check_source(text, "test.yaml", &crate_dir(), None, &Options::default())
}

fn of<'a>(report: &'a Report, check: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == check)
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

const HEAD: &str = "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
navigation: {home: k1, sections: [{name: all, pages: [k1]}]}\n";

/// Page `k1` of kind `boxed`, whose `summary` section has a text primitive `t` and a metric `c`;
/// the page restates the section's children as `children`.
fn boxed(children: &str) -> String {
    format!(
        "{HEAD}page_kinds:\n  boxed:\n    extends: detail_page\n    purpose: x\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], \
         children: [{{name: t, primitive: text, text: x}}, \
         {{name: c, component: metric, reads: t.Count, label: L}}]}}\n\
         pages:\n  k1:\n    kind: boxed\n    title: K\n    \
         state: {{mine: {{type: string, class: page_state}}}}\n    sections:\n      \
         - {{name: summary, children: [{children}]}}\n"
    )
}

/// The fix takes the kind of a node the document writes without `component` from the loaded
/// node, for composites and widgets only. Named lists merge at every depth, so a page restates a
/// kind's nested primitive by name without `primitive`; its `text` and `visible` are then read by
/// no table. The nested composite beside it is the control.
#[test]
fn a_nested_primitive_that_inherits_its_kind_is_a_site() {
    let report = run(&boxed(
        "{name: t, text: state.nosuch_prim}, {name: c, visible: state.nosuch_comp}",
    ));
    loads(&report);
    let found: Vec<&str> = of(&report, "expr_paths")
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        found,
        [
            "pages/k1/sections/summary/children/c",
            "pages/k1/sections/summary/children/t"
        ],
        "{:#?}",
        report.findings
    );
}

/// Acceptance 5 lists `args` as a site of points 1 to 4. An argument substituted into an `expr`
/// position (a text primitive's `text`) is an expression there, and a misspelt root in it
/// (`stat.mine` for `state.mine`) is the warning point 4 asks for. The fix silences every
/// argument's dotted word.
#[test]
fn a_misspelt_root_in_an_argument_bound_into_an_expression_warns() {
    let text = format!(
        "{HEAD}widgets:\n  card:\n    summary: C\n    \
         params: {{who: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: args.who}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    \
         state: {{mine: {{type: string, class: page_state}}}}\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], \
         children: [{{name: w, component: card, args: {{who: stat.mine}}}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_roots");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/k1/sections/summary/children/w");
}
