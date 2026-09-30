//! Adversary pass 1 on story:ui-spec-checks.
//!
//! Each case drives the checker from the schema (`schemas/ui/ess-ui.schema.yaml`) or the story's
//! acceptance statement, never from what the implementation happens to do.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ess_ui::NodeRef;
use ess_ui_check::{check_source, load_model, Finding, Model, Options, Report, Severity};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A complete document: `model`, then `widgets` before or after `pages` as asked.
fn document(model: &str, widgets: &str, pages: &str, widgets_first: bool) -> String {
    let mut text = format!(
        "format: ess-ui/1\napp: t\nmodel: {model}\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n"
    );
    if widgets_first {
        let _ = writeln!(text, "widgets: {widgets}");
        let _ = writeln!(text, "pages: {{p: {pages}}}");
    } else {
        let _ = writeln!(text, "pages: {{p: {pages}}}");
        let _ = writeln!(text, "widgets: {widgets}");
    }
    text
}

fn report(text: &str, model: Option<&Model>) -> Report {
    check_source(text, "adv.yaml", &crate_dir(), model, &Options::default())
}

fn of<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report.findings.iter().filter(|f| f.check == id).collect()
}

fn errors(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .collect()
}

fn model() -> Model {
    load_model(&crate_dir().join("tests/fixtures/model")).unwrap_or_else(|error| panic!("{error}"))
}

// ── widget_expands: args_match_param_types ──────────────────────────────────────────────────

/// `checks.list` `widget_expands.must` includes `args_match_param_types`. An enum param given a
/// value outside its enum is the plainest mismatch there is.
#[test]
fn widget_expands_fires_when_an_argument_does_not_match_its_param_type() {
    let widgets = "{tone: {summary: T, params: {level: {type: {enum: [low, high]}, required: true, note: n}}, \
                   body: [{name: t, primitive: text, text: args.level}]}}";
    let pages = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
                 children: [{name: use, component: tone, args: {level: sideways}}]}]}";
    let report = report(&document("t.system", widgets, pages, true), None);
    let found = of(&report, "widget_expands");
    assert!(
        found
            .iter()
            .any(|f| f.path == "pages/p/sections/summary/children/use"),
        "an enum param bound to `sideways` passed; findings: {:#?}",
        report.findings
    );
}

// ── widget declarations are checked at their use sites, not as written ──────────────────────

/// Widget doc: "Its `params` are typed; its `body` ... may read `args.<param>`. A widget is
/// expanded at its use site and then checked like a built-in." A `{ref: page}` param used as a
/// link target must not fail `page_refs` on the declaration, where `args.target` is unbound.
#[test]
fn a_page_param_used_as_a_link_target_is_not_a_page_refs_error_in_the_declaration() {
    let widgets =
        "{go: {summary: G, params: {target: {type: {ref: page}, required: true, note: n}}, \
                   body: [{name: link, primitive: link, text: Go, to: {to: args.target}}]}}";
    let pages = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
                 children: [{name: use, component: go, args: {target: p}}]}]}";
    let report = report(&document("t.system", widgets, pages, true), None);
    assert!(
        of(&report, "page_refs").is_empty(),
        "the use site binds `target` to page `p`; findings: {:#?}",
        report.findings
    );
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
}

/// The same with `--model`: a `{ref: view}` param read by a collection in the widget body.
#[test]
fn a_view_param_read_by_a_widget_body_is_not_a_view_in_model_error_in_the_declaration() {
    let widgets =
        "{lister: {summary: L, params: {source: {type: {ref: view}, required: true, note: n}}, \
                   body: [{name: rows, component: collection, reads: args.source}]}}";
    let pages = "{kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items, \
                 children: [{name: use, component: lister, args: {source: stock.Items}}]}]}";
    let model = model();
    let report = report(&document("shop", widgets, pages, true), Some(&model));
    assert!(
        of(&report, "view_in_model").is_empty(),
        "the use site binds `source` to `stock.Items`, a view of the model; findings: {:#?}",
        report.findings
    );
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
}

// ── section_readable ────────────────────────────────────────────────────────────────────────

/// Section `component` is `one_of [composite_kind, widget]`. A section rendered by a widget whose
/// body reads a view of a context no actor is granted anything in is shown to nobody, exactly as
/// the composite section in `tests/checks.rs` `section_readable` is.
#[test]
fn a_section_rendered_by_a_widget_reading_an_unreadable_view_is_reported() {
    let widgets =
        "{log: {summary: L, body: [{name: rows, component: collection, reads: audit.Entries}]}}";
    let pages = "{kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items}, \
                 {name: trail, component: log}]}";
    let model = model();
    let report = report(&document("shop", widgets, pages, true), Some(&model));
    assert!(
        of(&report, "section_readable")
            .iter()
            .any(|f| f.path == "pages/p/sections/trail"),
        "section `trail` reads only `shop.audit.Entries`; findings: {:#?}",
        report.findings
    );
}

// ── loader refusals filed under the check they break ────────────────────────────────────────

/// `primitive_props` (`props_of_kind_only`) is filed by re-reading the authored YAML at the
/// refusal's path. The loader names the expanded instance path when `pages` precedes `widgets`,
/// and that path has no authored `body`. The same document, keys reordered, must file the same.
#[test]
fn a_foreign_primitive_prop_in_a_widget_is_primitive_props_whatever_the_key_order() {
    let widgets =
        "{cap: {summary: C, body: [{name: caption, primitive: text, text: T, icon: star}]}}";
    let pages = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
                 children: [{name: use, component: cap}]}]}";
    for widgets_first in [true, false] {
        let report = report(&document("t.system", widgets, pages, widgets_first), None);
        let checks: Vec<(&str, &str)> = report
            .findings
            .iter()
            .map(|f| (f.check.as_str(), f.path.as_str()))
            .collect();
        assert!(
            checks.iter().any(|(check, _)| *check == "primitive_props"),
            "widgets_first = {widgets_first}: filed as {checks:?}"
        );
    }
}

/// `names_unique`: "every node, `unique_name_among_siblings`". Two pages under one key are two
/// siblings sharing a name; the YAML parser refuses them before any check runs.
#[test]
fn two_pages_with_one_name_are_names_unique() {
    let text = "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
                shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
                navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
                pages:\n  p: {kind: detail_page, title: P, sections: [{name: a, reads: t.ById}]}\n  \
                p: {kind: detail_page, title: Q, sections: [{name: b, reads: t.ById}]}\n";
    let report = report(text, None);
    let checks: Vec<(&str, &str)> = report
        .findings
        .iter()
        .map(|f| (f.check.as_str(), f.path.as_str()))
        .collect();
    assert!(
        checks.iter().any(|(check, _)| *check == "names_unique"),
        "filed as {checks:?}"
    );
}

// ── canonical paths from the expression walk ────────────────────────────────────────────────

/// `checks.report_by: NodePath`. An action written without `name` gets a derived one from the
/// loader; a `channel.<name>` finding inside it must name that node, and keep naming it when an
/// unnamed sibling action is inserted before it.
#[test]
fn an_expression_finding_in_an_unnamed_action_names_the_action_by_its_canonical_path() {
    let before = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
                  actions: [{does: t.Archive, visible: channel.ghost.up}]}]}";
    let after = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
                 actions: [{does: t.Restore}, {does: t.Archive, visible: channel.ghost.up}]}]}";
    for pages in [before, after] {
        let text = document("t.system", "{}", pages, true);
        let loaded = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
        let canonical: Vec<String> = loaded
            .nodes()
            .into_iter()
            .filter(|located| {
                matches!(located.node, NodeRef::Action(action)
                    if action.does.as_deref() == Some("t.Archive"))
            })
            .map(|located| located.path.to_string())
            .collect();
        assert_eq!(canonical.len(), 1, "{canonical:?}");
        let report = report(&text, None);
        let paths: Vec<&str> = of(&report, "channel_refs")
            .iter()
            .map(|f| f.path.as_str())
            .collect();
        assert_eq!(paths, [canonical[0].as_str()], "{:#?}", report.findings);
    }
}

// ── performance ─────────────────────────────────────────────────────────────────────────────

const BUDGET: Duration = Duration::from_secs(10);

/// 2,000 widgets, each using the next: no cycle, 2,000 declaration nodes.
#[test]
fn a_chain_of_2000_widgets_is_checked_within_the_budget() {
    let mut widgets = String::from("{");
    for index in 0..2000 {
        let body = if index == 1999 {
            "[{name: leaf, primitive: text, text: T}]".to_owned()
        } else {
            format!("[{{name: next, component: w{}}}]", index + 1)
        };
        let _ = write!(widgets, "w{index}: {{summary: W, body: {body}}}, ");
    }
    widgets.push('}');
    let pages = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}";
    let text = document("t.system", &widgets, pages, true);
    let loading = Instant::now();
    ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let loaded = loading.elapsed();
    let started = Instant::now();
    let report = report(&text, None);
    let took = started.elapsed();
    assert!(
        errors(&report).is_empty(),
        "{:#?}",
        &report.findings[..5.min(report.findings.len())]
    );
    assert!(
        took < BUDGET,
        "2,000 chained widgets took {took:?} to check, of which loading alone takes {loaded:?}"
    );
}

/// 2,000 sections on one page, each with state, a `depends_on` and an action.
#[test]
fn a_page_of_2000_sections_is_checked_within_the_budget() {
    let mut sections = String::from("[");
    for index in 0..2000 {
        let depends = if index == 0 {
            String::new()
        } else {
            format!(", depends_on: s{}", index - 1)
        };
        let _ = write!(
            sections,
            "{{name: s{index}, component: record, reads: t.ById{depends}, \
             state: {{picked: {{type: string, class: selection}}}}, \
             actions: [{{name: go, opens: edit}}]}}, "
        );
    }
    sections.push(']');
    let pages = format!(
        "{{kind: detail_page, title: P, \
         overlays: {{edit: {{kind: dialog, component: confirm, does: t.X}}}}, sections: {sections}}}"
    );
    let text = document("t.system", "{}", &pages, true);
    let started = Instant::now();
    let report = report(&text, None);
    let took = started.elapsed();
    assert!(
        errors(&report).is_empty(),
        "{:#?}",
        &report.findings[..5.min(report.findings.len())]
    );
    assert!(took < BUDGET, "2,000 sections took {took:?}");
}
