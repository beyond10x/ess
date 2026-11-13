//! Adversarial pass 3 on `ess ui check` resolving the root and the section of every binding path:
//! a widget argument that lands inside the arguments of a call, a widget argument that names state
//! the widget's own body declares, and a page kind that restates the nested nodes of the kind it
//! extends.

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

/// A widget that shows its body only to an actor who may send the command it is given:
/// `visible: actor.may(args.cmd)`. The use names the command, `cmd: orders.Approve`, which the
/// expansion substitutes into the call's argument, where a dotted word is a command name and no
/// stray (the subject of `expr_roots`: "outside the arguments of a call"; React passes a call's
/// unrooted argument as its text, `expr.ts.tmpl` `run`, case `call`). The same call written
/// directly in the page (`d`) draws nothing; through the widget it draws `expr_roots`, whose
/// message calls the command name literal text.
#[test]
fn a_command_handed_to_actor_may_through_a_widget_is_no_stray() {
    let text = format!(
        "{HEAD}widgets:\n  gated:\n    summary: G\n    \
         params: {{cmd: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, text: Hi, visible: actor.may(args.cmd)}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], children: [\
         {{name: w, component: gated, args: {{cmd: orders.Approve}}}}, \
         {{name: d, primitive: text, text: Hi, visible: actor.may(orders.Approve)}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    assert!(
        of(&report, "expr_roots").is_empty(),
        "{:#?}",
        report.findings
    );
}

/// A widget whose body node declares its own state (`note`) and shows the text it is handed
/// (`text: args.shown`). The use hands it `state.note`; the expansion writes
/// `text: state.note` into the node that declares `note`, so the path resolves there: React draws
/// a node that holds state inside a scope layer of that state (`emit.rs` `node`, `scope_layer`),
/// and the terminal keeps every state name. The check judges the argument against the scope of
/// the use, where `note` is not declared, and reports `expr_paths` as an error.
#[test]
fn an_argument_naming_state_the_widgets_body_declares_resolves() {
    let text = format!(
        "{HEAD}widgets:\n  disc:\n    summary: D\n    \
         params: {{shown: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: t, primitive: text, \
         state: {{note: {{type: string, class: component_state, default: hello}}}}, \
         text: args.shown}}]\n\
         pages:\n  k1:\n    kind: detail_page\n    title: K\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], \
         children: [{{name: w, component: disc, args: {{shown: state.note}}}}]}}\n"
    );
    let report = run(&text);
    loads(&report);
    assert!(
        of(&report, "expr_paths").is_empty(),
        "{:#?}",
        report.findings
    );
}

/// A page kind that extends another restates the nested nodes of the kind it extends by name,
/// without `component` or `primitive`, as a page does. The fix takes such a node's kind from the
/// loaded nodes, but the loaded document lists a page kind as one node (`page_kinds/<name>`), not
/// its sections and their children, so the restated text `t` and metric `c` are walked with no
/// table and their strays are never read. The page `k1` inherits both without restating them, so
/// nothing reports them anywhere.
#[test]
fn a_kind_that_restates_its_parent_kinds_nested_nodes_is_walked() {
    let text = format!(
        "{HEAD}page_kinds:\n  inner:\n    extends: detail_page\n    purpose: x\n    sections:\n      \
         - {{name: summary, component: record, reads: t.ById, fields: [id], children: [\
         {{name: t, primitive: text, text: x}}, \
         {{name: c, component: metric, reads: t.Count, label: L}}]}}\n  \
         outer:\n    extends: inner\n    purpose: y\n    sections:\n      \
         - {{name: summary, children: [{{name: t, text: bogus.prim}}, \
         {{name: c, visible: bogus.comp}}]}}\n\
         pages:\n  k1:\n    kind: outer\n    title: K\n    sections:\n      - {{name: summary}}\n"
    );
    let report = run(&text);
    loads(&report);
    let found = of(&report, "expr_roots");
    assert_eq!(found.len(), 2, "{:#?}", report.findings);
    for word in ["bogus.prim", "bogus.comp"] {
        assert!(
            found
                .iter()
                .any(|finding| finding.path.starts_with("page_kinds/outer")
                    && finding.message.contains(&format!("`{word}`"))),
            "{word}: {:#?}",
            report.findings
        );
    }
}
