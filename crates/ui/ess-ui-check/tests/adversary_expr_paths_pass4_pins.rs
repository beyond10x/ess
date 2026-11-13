//! Adversarial pass 4 on `ess ui check` resolving the root and the section of every binding path:
//! what the third fix holds on models other than the third pass's cases.

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
            .any(|finding| finding.check == "document_loads" || finding.check == "widget_expands"),
        "{:#?}",
        report.findings
    );
}

const HEAD: &str = "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
navigation: {home: k1, sections: [{name: all, pages: [k1]}]}\n";

fn paths<'a>(findings: &[&'a Finding]) -> Vec<&'a str> {
    let mut paths: Vec<&str> = findings
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    paths.sort_unstable();
    paths
}

const NOTE: &str = "{note: {type: string, class: component_state, default: hi}}";

/// State on a sibling of the reading node is still an error (`u2`); state declared only by the
/// inner of two widgets around its read is allowed (`u5`), and so is state the use's own node
/// declares (`u6`).
#[test]
fn body_state_is_allowed_around_the_read_only() {
    let text = format!(
        "{HEAD}widgets:\n  sibling:\n    summary: S\n    \
         params: {{shown: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: a, primitive: text, text: Hi, state: {NOTE}}}, \
         {{name: b, primitive: text, text: args.shown}}]\n  \
         inner:\n    summary: I\n    \
         params: {{shown: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, state: {NOTE}, text: args.shown}}]\n  \
         outer_only:\n    summary: O\n    \
         params: {{p: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: m, component: inner, args: {{shown: args.p}}}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], children: [\
         {{name: u2, component: sibling, args: {{shown: state.note}}}}, \
         {{name: u5, component: outer_only, args: {{p: state.note}}}}, \
         {{name: u6, component: outer_only, state: {NOTE}, args: {{p: state.note}}}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    assert_eq!(
        paths(&of(&report, "expr_paths")),
        ["pages/k1/sections/summary/children/u2"],
        "{:#?}",
        report.findings
    );
}

/// A parameter read inside a call and outside one (`u1`) or in the filter grammar (`u3`) is an
/// `expr` position, as the same expression written directly (`d1`, `d2`) is; one read only inside
/// a negated call (`u2`) or handed on only inside a call (`u4`) is not.
#[test]
fn a_call_makes_a_parameter_no_expression_position_only_where_it_alone_reads_it() {
    let text = format!(
        "{HEAD}widgets:\n  both:\n    summary: B\n    \
         params: {{c: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: Hi, \
         visible: \"actor.may(args.c) and args.c == x\"}}]\n  \
         incall:\n    summary: I\n    \
         params: {{c: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: Hi, visible: \"not actor.may(args.c)\"}}]\n  \
         filt:\n    summary: F\n    \
         params: {{c: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: Hi, visible: row.tags contains args.c}}]\n  \
         shown:\n    summary: S\n    \
         params: {{v: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: args.v}}]\n  \
         handcall:\n    summary: H\n    \
         params: {{c: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: m, component: shown, args: {{v: \"actor.may(args.c)\"}}}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], children: [\
         {{name: u1, component: both, args: {{c: orders.Approve}}}}, \
         {{name: u2, component: incall, args: {{c: orders.Approve}}}}, \
         {{name: u3, component: filt, args: {{c: foo.bar}}}}, \
         {{name: u4, component: handcall, args: {{c: orders.Approve}}}}, \
         {{name: d1, primitive: text, text: Hi, \
         visible: \"actor.may(orders.Approve) and orders.Approve == x\"}}, \
         {{name: d2, primitive: text, text: Hi, visible: row.tags contains foo.bar}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    let at = "pages/k1/sections/summary/children/";
    assert_eq!(
        paths(&of(&report, "expr_roots")),
        [
            format!("{at}d1"),
            format!("{at}d2"),
            format!("{at}u1"),
            format!("{at}u3")
        ],
        "{:#?}",
        report.findings
    );
}

/// `kc` extends `kb` extends `ka` extends a built-in kind; `kc` restates a text that `kb` also
/// restated and a metric that only `ka` writes, and the page restates the text again. Each stray
/// is reported once, where it is written.
#[test]
fn an_extends_chain_of_three_gives_each_restated_node_its_kind() {
    let text = format!(
        "{HEAD}page_kinds:\n  ka:\n    extends: detail_page\n    purpose: a\n    sections:\n      \
         - {{name: summary, reads: t.ById, fields: [id], children: [\
         {{name: t, primitive: text, text: x}}, \
         {{name: m, component: metric, reads: t.Count, label: L}}]}}\n  \
         kb:\n    extends: ka\n    purpose: b\n    sections:\n      \
         - {{name: summary, children: [{{name: t, text: y}}]}}\n  \
         kc:\n    extends: kb\n    purpose: c\n    sections:\n      \
         - {{name: summary, children: [{{name: t, text: bogus.c}}, \
         {{name: m, visible: bogus.m}}]}}\n\
         pages:\n  k1:\n    kind: kc\n    title: K\n    sections:\n      \
         - {{name: summary, children: [{{name: t, text: bogus.p}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_roots");
    for (word, at) in [
        ("bogus.c", "page_kinds/kc"),
        ("bogus.m", "page_kinds/kc"),
        ("bogus.p", "pages/k1/sections/summary/children/t"),
    ] {
        let hits = found
            .iter()
            .filter(|finding| finding.path == at && finding.message.contains(&format!("`{word}`")))
            .count();
        assert_eq!(hits, 1, "{word}: {:#?}", report.findings);
    }
    assert_eq!(found.len(), 3, "{:#?}", report.findings);
}
