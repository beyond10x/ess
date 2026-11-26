//! A literal in an `expr` position renders as text (#353), written directly on a node and
//! passed through a widget argument; an expression passed the same way still evaluates.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

/// The partner-portal example with a `caption` widget and, in the overview's `kpis` record,
/// literal captions, a `status_badge` bound to `row.open_deals` and a node hidden by
/// `visible: "false"`.
fn document() -> String {
    let text = std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads");
    let widgets = "widgets:\n";
    let kpi = "          - {name: overdue, component: metric, \
               from: channel.metrics.overdue_invoices, label: Overdue invoices}\n";
    assert!(
        text.contains(widgets) && text.contains(kpi),
        "the example changed"
    );
    text.replacen(
        widgets,
        "widgets:\n  caption:\n    summary: A caption.\n    params:\n      \
         label: {type: string, required: true, note: the text}\n    \
         body:\n      - {name: t, primitive: text, text: args.label}\n",
        1,
    )
    .replacen(
        kpi,
        &format!(
            "{kpi}          - {{name: d1, primitive: text, text: Direct cost (cents)}}\n          \
             - {{name: w1, component: caption, args: {{label: Widget limit in cents}}}}\n          \
             - {{name: w2, component: caption, args: {{label: Widget cost (cents)}}}}\n          \
             - {{name: w3, component: caption, args: {{label: Terms and conditions}}}}\n          \
             - {{name: w4, component: caption, args: {{label: '\"Quoted (text)\"'}}}}\n          \
             - {{name: w5, component: caption, args: {{label: 4242}}}}\n          \
             - {{name: w6, component: status_badge, args: {{status: row.open_deals, tones: partner_tier}}}}\n          \
             - {{name: w7, primitive: text, text: Never shown, visible: \"false\"}}\n          \
             - {{name: v1, primitive: text, text: Shown by and, visible: \"channel.metrics.overdue_invoices == channel.metrics.overdue_invoices and 'yes' == 'yes'\"}}\n          \
             - {{name: v2, primitive: text, text: Shown by or, visible: \"'a' == 'b' or channel.metrics.overdue_invoices != 'x'\"}}\n          \
             - {{name: v3, primitive: text, text: Shown by in, visible: \"not ('a' in ['b', 'c']) and channel.metrics.overdue_invoices != 'x'\"}}\n          \
             - {{name: v4, primitive: text, text: Hidden by and, visible: \"channel.metrics.overdue_invoices != 'x' and 'a' == 'b'\"}}\n"
        ),
        1,
    )
}

#[test]
fn literal_text_renders_as_written_and_expressions_still_evaluate() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-tui/literal-text");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    let app = App::from_text(&document(), &example_dir(), Options::new(dir))
        .unwrap_or_else(|error| panic!("{error}"));
    let screen = app.render_text(160, 60);
    let missing: Vec<&str> = [
        "Direct cost (cents)",
        "Widget limit in cents",
        "Widget cost (cents)",
        "Terms and conditions",
        "Quoted (text)",
        "4242",
        "[37]",
    ]
    .into_iter()
    .filter(|wanted| !screen.contains(wanted))
    .collect();
    assert!(missing.is_empty(), "missing {missing:?}:\n{screen}");
    assert!(
        !screen.contains("\"Quoted"),
        "the quotes are not text:\n{screen}"
    );
    assert!(
        !screen.contains("Never shown"),
        "visible: \"false\" hides:\n{screen}"
    );
}

/// The terminal judges `and`, `or`, `in` and parentheses in a `visible` condition with the one
/// grammar React reads (`ess_ui::filter`), where it split on the first `==` it met
/// (story:both-renderers-read-one-binding-expression-grammar, beyond10x/ess#723).
#[test]
fn a_visible_condition_joins_with_and_or_in_as_react_reads_it() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-tui/visible-grammar");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    let app = App::from_text(&document(), &example_dir(), Options::new(dir))
        .unwrap_or_else(|error| panic!("{error}"));
    let screen = app.render_text(200, 120);
    let hidden: Vec<&str> = ["Shown by and", "Shown by or", "Shown by in"]
        .into_iter()
        .filter(|wanted| !screen.contains(wanted))
        .collect();
    assert!(hidden.is_empty(), "hidden {hidden:?}:\n{screen}");
    assert!(
        !screen.contains("Hidden by and"),
        "a false `and` hides:\n{screen}"
    );
}
