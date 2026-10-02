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
             - {{name: w6, component: status_badge, args: {{status: row.open_deals, tones: {{}}}}}}\n          \
             - {{name: w7, primitive: text, text: Never shown, visible: \"false\"}}\n"
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
