//! Adversarial pass 4 on `ess ui check` resolving the root and the section of every binding path:
//! the state a widget argument may name because the widget's body declares it.

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

/// A document whose page `k1` uses each widget of `widgets` once, as `children`.
fn page(widgets: &str, children: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: k1, sections: [{{name: all, pages: [k1]}}]}}\n\
         widgets:\n{widgets}\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], \
         children: [{children}]}}\n"
    )
}

const NOTE: &str = "{note: {type: string, class: component_state, default: hi}}";

/// The body reads `args.shown` twice: in `a`, which declares `note`, and in its sibling `b`,
/// which does not. The expansion writes `text: state.note` into both; React resolves it in `a`
/// (inside `a`'s state layer) and not in `b`, which draws an undeclared state, the case
/// `expr_paths` reports when it is written directly. The fix allows an argument every state
/// declared around any one read of the parameter, so the use passes.
#[test]
fn a_state_declared_around_one_read_of_two_is_still_an_error() {
    let text = page(
        &format!(
            "  twice:\n    summary: T\n    \
             params: {{shown: {{type: string, required: true, note: n}}}}\n    \
             body: [{{name: a, primitive: text, state: {NOTE}, text: args.shown}}, \
             {{name: b, primitive: text, text: args.shown}}]\n"
        ),
        "{name: u, component: twice, args: {shown: state.note}}",
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert!(found[0].message.contains("`state.note`"), "{found:#?}");
}

/// The same through a hand-on: `outer` hands `shown` to `inner`, whose body declares `note`
/// around its read, and also shows `args.shown` itself in `own`, where nothing declares `note`.
/// `settle()` merges the inner widget's states into the outer parameter's, so the use passes
/// though `own` draws an undeclared state.
#[test]
fn a_state_declared_only_by_the_inner_widget_does_not_cover_the_outer_read() {
    let text = page(
        &format!(
            "  inner:\n    summary: I\n    \
             params: {{shown: {{type: string, required: true, note: n}}}}\n    \
             body: [{{name: t, primitive: text, state: {NOTE}, text: args.shown}}]\n  \
             outer:\n    summary: O\n    \
             params: {{shown: {{type: string, required: true, note: n}}}}\n    \
             body: [{{name: m, component: inner, args: {{shown: args.shown}}}}, \
             {{name: own, primitive: text, text: args.shown}}]\n"
        ),
        "{name: u, component: outer, args: {shown: state.note}}",
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert!(found[0].message.contains("`state.note`"), "{found:#?}");
}

/// The body reads the parameter in the `visible` of the node that declares `open`. React draws
/// that node's condition outside the node's own state layer (`emit.rs` `node`: the state layer is
/// inside the hoisted component, `visible` wraps it; `core.tsx.tmpl` `Visible` reads the parent
/// scope), so `state.open` is not resolved there. The fix counts the node's own state as around
/// its own `visible` and accepts the use.
#[test]
#[ignore = "story:a-nodes-own-visible-is-judged-in-the-scope-the-renderers-draw-it-in (backlog)"]
fn a_state_the_reading_node_declares_does_not_cover_its_own_visible() {
    let text = page(
        "  ownvis:\n    summary: V\n    \
         params: {when: {type: string, required: true, note: n}}\n    \
         body: [{name: t, primitive: text, text: Hi, \
         state: {open: {type: boolean, class: component_state, default: true}}, \
         visible: args.when}]\n",
        "{name: u, component: ownvis, args: {when: state.open}}",
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert!(found[0].message.contains("`state.open`"), "{found:#?}");
}
