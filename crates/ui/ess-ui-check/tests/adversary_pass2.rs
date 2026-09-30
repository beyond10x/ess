//! Adversary pass 2 on story:ui-spec-checks.
//!
//! Each case attacks a fix or a decision of correction 1 and is driven from the schema
//! (`schemas/ui/ess-ui.schema.yaml`) or the story's acceptance, never from what the implementation
//! happens to do.

use std::fmt::Write as _;
use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A complete document under `placement_profile`, with `widgets` and page `p`.
fn document(profile: &str, widgets: &str, page: &str) -> String {
    let mut text = format!(
        "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: {profile}\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n"
    );
    let _ = writeln!(text, "widgets: {widgets}");
    let _ = writeln!(text, "pages: {{p: {page}}}");
    text
}

/// Page `p` with one section whose single child is `child`.
fn page_with_child(child: &str) -> String {
    format!(
        "{{kind: detail_page, title: P, sections: [{{name: summary, reads: t.ById, \
         children: [{child}]}}]}}"
    )
}

fn report(text: &str) -> Report {
    check_source(text, "adv2.yaml", &crate_dir(), None, &Options::default())
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

fn paths<'a>(findings: &[&'a Finding]) -> Vec<&'a str> {
    findings.iter().map(|f| f.path.as_str()).collect()
}

// ── widget_expands: no_recursion ────────────────────────────────────────────────────────────

/// `cycles` doc: "Every declared widget that lies on a cycle of uses". `c` uses `b`, `b` uses `a`,
/// `a` uses `c`: `c` contains itself (c → b → a → c). A white/grey/black search from `a` closes
/// `a → b → a` first, blackens `b`, and then reaches `b` from `c` as black, so no cycle through
/// `c` is ever closed.
#[test]
fn every_widget_on_a_cycle_is_reported_even_when_the_cycle_closes_through_a_finished_widget() {
    let widgets =
        "{a: {summary: A, body: [{name: to_b, component: b}, {name: to_c, component: c}]}, \
                   b: {summary: B, body: [{name: back, component: a}]}, \
                   c: {summary: C, body: [{name: via, component: b}]}}";
    let page = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}";
    let report = report(&document("fat", widgets, page));
    let found = of(&report, "widget_expands");
    let at = paths(&found);
    for widget in ["widgets/a", "widgets/b", "widgets/c"] {
        assert!(
            at.contains(&widget),
            "`{widget}` lies on a cycle and is not reported; widget_expands at {at:?}"
        );
    }
}

/// Schema rule moved from the schema unit: "a self-containing widget before use". An action's
/// inline confirm overlay (`confirm: {overlay: …}`) is rendered by a component like any node; the
/// loader expands a widget there, so a widget whose confirm dialog is itself contains itself.
#[test]
fn a_widget_containing_itself_through_an_inline_confirm_overlay_is_reported_before_use() {
    let widgets = "{a: {summary: A, body: [{name: go, primitive: button, label: Go, \
                   action: {name: go, does: t.X, confirm: {overlay: {kind: dialog, component: a}}}}]}}";
    let page = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}";
    let report = report(&document("fat", widgets, page));
    assert!(
        paths(&of(&report, "widget_expands")).contains(&"widgets/a"),
        "widget `a` renders its own confirm dialog; findings: {:#?}",
        report.findings
    );
}

// ── widget_expands: args_match_param_types ──────────────────────────────────────────────────

/// `WidgetParam.default` is the value the body is expanded with when a use omits the argument,
/// so it is an argument of every such use. An enum param defaulting outside its enum expands to a
/// value the type forbids.
#[test]
fn a_param_default_outside_its_type_is_widget_expands() {
    let widgets =
        "{tone: {summary: T, params: {level: {type: {enum: [low, high]}, default: sideways, \
                   note: n}}, body: [{name: t, primitive: text, text: args.level}]}}";
    let page = page_with_child("{name: use, component: tone}");
    let report = report(&document("fat", widgets, &page));
    assert!(
        !of(&report, "widget_expands").is_empty(),
        "the default `sideways` is no value of {{enum: [low, high]}}; findings: {:#?}",
        report.findings
    );
}

/// `unmapped_marker.accepted_by.types` lists `{enum: …}`: a retrofit may write the marker where
/// an enum value is expected, and the validator reports it as a warning, not an error.
#[test]
fn an_unmapped_marker_bound_to_an_enum_param_is_a_warning_not_an_error() {
    let widgets =
        "{tone: {summary: T, params: {level: {type: {enum: [low, high]}, required: true, \
                   note: n}}, body: [{name: t, primitive: text, text: args.level}]}}";
    let page = page_with_child(
        "{name: use, component: tone, args: {level: 'UNMAPPED: the old screen picked it per tenant'}}",
    );
    let report = report(&document("fat", widgets, &page));
    assert!(
        of(&report, "widget_expands").is_empty(),
        "the marker is accepted by an enum type; findings: {:#?}",
        report.findings
    );
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
}

/// `expressions.forms` names roots (`state.`, `row.`, …) and operators over them; a sentence that
/// happens to contain the word `and` is a literal, and a literal outside the enum is a mismatch.
#[test]
fn a_prose_literal_containing_and_is_not_taken_for_an_expression() {
    let widgets = "{doc_link: {summary: D, params: {which: {type: {enum: [terms, privacy]}, \
                   required: true, note: n}}, body: [{name: t, primitive: text, text: args.which}]}}";
    let page =
        page_with_child("{name: use, component: doc_link, args: {which: Terms and conditions}}");
    let report = report(&document("fat", widgets, &page));
    assert!(
        paths(&of(&report, "widget_expands")).contains(&"pages/p/sections/summary/children/use"),
        "`Terms and conditions` is no value of {{enum: [terms, privacy]}}; findings: {:#?}",
        report.findings
    );
}

// ── declaration findings: false errors where every use is valid ─────────────────────────────

/// Widget doc: "A widget is expanded at its use site and then checked like a built-in." An
/// optional param without a default is absent at a use that omits it, so a text taking either a
/// literal or a row field has exactly one of them at every use. The declaration, where both are
/// still `args.…`, is not a use.
#[test]
fn a_text_taking_either_param_is_not_primitive_props_in_its_declaration() {
    let widgets = "{cell: {summary: C, params: {label: {type: string, note: n}, \
                   column: {type: name, note: n}}, \
                   body: [{name: t, primitive: text, text: args.label, field: args.column}]}}";
    let page = page_with_child("{name: use, component: cell, args: {label: Hello}}");
    let report = report(&document("fat", widgets, &page));
    assert!(
        of(&report, "primitive_props").is_empty(),
        "the only use binds `label` alone; findings: {:#?}",
        report.findings
    );
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
}

/// `PlacementProfile.resolution.order` resolves a state through its section's and page's
/// profile. A widget declaration has neither, so in a hybrid document its state cannot resolve
/// there — but it is only ever rendered at a use, here inside a `profile: fat` page, where
/// `selection` resolves to `memory`.
#[test]
fn widget_state_in_a_hybrid_document_resolves_at_its_use_not_in_its_declaration() {
    let widgets =
        "{picker: {summary: P, body: [{name: list, component: collection, reads: t.ById, \
                   state: {picked: {type: string, class: selection}}}]}}";
    let page = "{kind: detail_page, title: P, profile: fat, sections: [{name: summary, \
                reads: t.ById, children: [{name: use, component: picker}]}]}";
    let report = report(&document("hybrid", widgets, page));
    assert!(
        of(&report, "state_resolves").is_empty(),
        "the only use is in a fat page; findings: {:#?}",
        report.findings
    );
}

// ── primitive_props classified from serde's expected fields ─────────────────────────────────

/// `props_of_kind_only` on every one of the nine kinds, not only `text`.
#[test]
fn a_foreign_prop_on_every_primitive_kind_is_primitive_props() {
    let kinds = [
        "primitive: text, zzz: 1, text: T",
        "primitive: badge, zzz: 1, text: T",
        "primitive: icon, zzz: 1, icon: star, label: L",
        "primitive: button, zzz: 1, label: L, action: {name: a, does: t.X}",
        "primitive: link, zzz: 1, text: T, href: row.url",
        "primitive: input, zzz: 1, binds: state.q",
        "primitive: toggle, zzz: 1, label: L, binds: state.q",
        "primitive: image, zzz: 1, src: row.logo, alt: A",
        "primitive: divider, zzz: 1",
    ];
    let mut misfiled = Vec::new();
    for kind in kinds {
        let page = page_with_child(&format!("{{name: leaf, {kind}}}"));
        let report = report(&document("fat", "{}", &page));
        let checks: Vec<(&str, &str)> = report
            .findings
            .iter()
            .map(|f| (f.check.as_str(), f.path.as_str()))
            .collect();
        if !checks.iter().any(|(check, _)| *check == "primitive_props") {
            misfiled.push(format!("{kind}: {checks:?}"));
        }
    }
    assert!(misfiled.is_empty(), "{misfiled:#?}");
}

// ── duplicate YAML keys ─────────────────────────────────────────────────────────────────────

const HEAD: &str = "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
                    shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
                    navigation: {home: p, sections: [{name: all, pages: [p]}]}\n";

/// `names_unique`: "every node, `unique_name_among_siblings`". A property written twice is not
/// two nodes sharing a name: `title` names no node. The refusal is the loader's, and no check of
/// the list names it, which is what `document_loads` is for.
#[test]
fn a_property_written_twice_is_not_names_unique() {
    let text = format!(
        "{HEAD}pages:\n  p:\n    kind: detail_page\n    title: P\n    title: Q\n    \
         sections: [{{name: summary, reads: t.ById}}]\n"
    );
    let report = report(&text);
    let checks: Vec<(&str, &str, &str)> = report
        .findings
        .iter()
        .map(|f| (f.check.as_str(), f.path.as_str(), f.message.as_str()))
        .collect();
    assert!(
        checks
            .iter()
            .any(|(check, _, _)| *check == "document_loads")
            && !checks.iter().any(|(check, _, _)| *check == "names_unique"),
        "filed as {checks:#?}"
    );
}

/// `checks.report_by: NodePath`. A key written twice inside an unnamed action is a refusal of
/// that action, and the action's path is the one the loader derives (`archive`), as correction 1
/// made the expression walk give it.
#[test]
fn a_duplicate_key_in_an_unnamed_action_names_the_action_by_its_canonical_path() {
    let text = format!(
        "{HEAD}pages:\n  p:\n    kind: detail_page\n    title: P\n    sections:\n      \
         - name: summary\n        reads: t.ById\n        actions:\n          \
         - does: t.Restore\n          - does: t.Archive\n            visible: row.open\n            \
         visible: row.closed\n"
    );
    let report = report(&text);
    let at: Vec<(&str, &str)> = report
        .findings
        .iter()
        .map(|f| (f.check.as_str(), f.path.as_str()))
        .collect();
    assert!(
        at.iter()
            .any(|(_, path)| *path == "pages/p/sections/summary/actions/archive"),
        "reported at {at:?}"
    );
}
