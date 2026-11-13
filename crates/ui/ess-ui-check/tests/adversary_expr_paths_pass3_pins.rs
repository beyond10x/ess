//! Adversarial pass 3 on `ess ui check` resolving the root and the section of every binding path:
//! what the second fix holds, on models other than the second pass's cases.

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

/// `outer` hands `p` to `mid` inside a record's `item`, `mid` hands it to `card`, and `card`
/// substitutes it into a text's `text` (an `expr`). `q` takes the same two hops but lands in an
/// image's `alt` alone. `card` also shows `who` in an `alt`, which does not take it out of an
/// `expr` position. Only `p`'s misspelt root warns.
#[test]
fn a_parameter_reaches_an_expression_through_two_hops() {
    let text = format!(
        "{HEAD}widgets:\n  card:\n    summary: C\n    \
         params: {{who: {{type: string, required: true, note: n}}, \
         cap: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: args.who}}, \
         {{name: i, primitive: image, src: logo, alt: args.cap}}, \
         {{name: both, primitive: image, src: logo, alt: args.who}}]\n  \
         mid:\n    summary: M\n    \
         params: {{x: {{type: string, required: true, note: n}}, \
         y: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: c, component: card, args: {{who: args.x, cap: args.y}}}}]\n  \
         outer:\n    summary: O\n    \
         params: {{p: {{type: string, required: true, note: n}}, \
         q: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: l, component: record, reads: t.ById, fields: [id], \
         item: [{{name: m, component: mid, args: {{x: args.p, y: args.q}}}}]}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    \
         state: {{mine: {{type: string, class: page_state}}}}\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], \
         children: [{{name: w, component: outer, args: {{p: stat.mine, q: diagram.png}}}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_roots");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/k1/sections/summary/children/w");
    assert!(
        found[0].message.contains("`stat.mine`") && found[0].message.contains("args/p"),
        "{found:#?}"
    );
}

/// A parameter read only in a node's `visible`, and one read only in a link's `href`, are in an
/// `expr` position; one read only in an image's `alt` is not.
#[test]
fn a_parameter_read_in_visible_or_href_alone_is_an_expression() {
    let text = format!(
        "{HEAD}widgets:\n  vis:\n    summary: V\n    \
         params: {{v: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: Hi, visible: args.v}}]\n  \
         web:\n    summary: W\n    \
         params: {{h: {{type: string, required: true, note: n}}, \
         label: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: l, primitive: link, text: Open, href: args.h}}, \
         {{name: i, primitive: image, src: logo, alt: args.label}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    \
         state: {{mine: {{type: string, class: page_state}}}}\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], children: [\
         {{name: a, component: vis, args: {{v: stat.mine}}}}, \
         {{name: b, component: web, args: {{h: example.com, label: diagram.png}}}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    let mut found: Vec<(&str, bool, bool)> = of(&report, "expr_roots")
        .iter()
        .map(|finding| {
            (
                finding.path.as_str(),
                finding.message.contains("args/v"),
                finding.message.contains("args/h"),
            )
        })
        .collect();
    found.sort_unstable();
    assert_eq!(
        found,
        [
            ("pages/k1/sections/summary/children/a", true, false),
            ("pages/k1/sections/summary/children/b", false, true)
        ],
        "{:#?}",
        report.findings
    );
}

/// Every primitive kind with an `expr` field, two lists deep in a page kind and restated by the
/// page without `primitive`, plus a text in an overlay of the kind restated the same way: each
/// undeclared state is reported once.
#[test]
fn every_inherited_primitive_is_walked_at_depth() {
    let text = format!(
        "{HEAD}page_kinds:\n  deep:\n    extends: detail_page\n    purpose: x\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], item: [\
         {{name: g, component: record, reads: t.ById, fields: [id], item: [\
         {{name: bd, primitive: badge, text: x, tone_by: {{value: row.id, map: {{a: info}}}}}}, \
         {{name: ln, primitive: link, text: Open, href: x}}, \
         {{name: im, primitive: image, src: logo, alt: L}}, \
         {{name: tg, primitive: toggle, label: T, binds: state.mine}}, \
         {{name: bt, primitive: button, label: B, action: {{name: go, sets: {{state.mine: x}}}}}}, \
         {{name: ip, primitive: input, binds: state.mine}}]}}]}}\n    \
         overlays:\n      ov: {{kind: dialog, component: record, reads: t.ById, fields: [id], \
         item: [{{name: ot, primitive: text, text: x}}]}}\n\
         pages:\n  k1:\n    kind: deep\n    title: K\n    \
         state: {{mine: {{type: string, class: page_state}}}}\n    sections:\n      \
         - {{name: summary, item: [{{name: g, item: [\
         {{name: bd, tone_by: {{value: state.nos_bd}}}}, \
         {{name: ln, href: state.nos_ln}}, \
         {{name: im, src: state.nos_im}}, \
         {{name: tg, binds: state.nos_tg}}, \
         {{name: bt, action: {{name: go, sets: {{state.nos_bt: x}}}}}}, \
         {{name: ip, binds: state.nos_ip}}]}}]}}\n    \
         overlays:\n      ov: {{item: [{{name: ot, text: state.nos_ot}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_paths");
    for name in [
        "nos_bd", "nos_ln", "nos_im", "nos_tg", "nos_bt", "nos_ip", "nos_ot",
    ] {
        let hits = found
            .iter()
            .filter(|finding| finding.message.contains(&format!("`state.{name}`")))
            .count();
        assert_eq!(hits, 1, "{name}: {:#?}", report.findings);
    }
    assert_eq!(found.len(), 7, "{:#?}", report.findings);
}

/// Two widgets that use each other: the loader refuses the cycle, and the check returns.
#[test]
fn a_widget_cycle_is_refused_and_the_check_returns() {
    let text = format!(
        "{HEAD}widgets:\n  loop_a:\n    summary: A\n    \
         params: {{v: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: b, component: loop_b, args: {{v: args.v}}}}]\n  \
         loop_b:\n    summary: B\n    \
         params: {{v: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: a, component: loop_a, args: {{v: args.v}}}}, \
         {{name: t, primitive: text, text: args.v}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id]}}\n"
    );
    let report = run(&text);
    assert!(
        of(&report, "widget_expands")
            .iter()
            .any(|finding| finding.message.contains("contains itself")),
        "{:#?}",
        report.findings
    );
}
