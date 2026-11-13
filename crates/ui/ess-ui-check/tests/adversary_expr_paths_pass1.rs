//! Adversarial pass 1 on `ess ui check` resolving the root and the section of every binding path:
//! expression sites the walk misses, a path reported under two rules, and a widget argument read
//! as an expression where it is substituted as text.

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

const SHELL: &str = "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
shells: {app: {regions: {main: {kind: page_outlet}}, \
state: {theme: {type: string, class: preference, store: local_storage}}}}\n";

const Q: &str =
    "  q:\n    kind: detail_page\n    title: Q\n    params: {id: string}\n    sections:\n      \
- {name: only, component: record, reads: {view: t.ById, params: {id: params.id}}, fields: [id]}\n";

/// Page `l` of kind `list_page`, whose `list` section is written as `section`.
fn list_page(section: &str) -> String {
    format!(
        "{SHELL}navigation: {{home: l, sections: [{{name: all, pages: [l, q]}}]}}\n\
         pages:\n  l:\n    kind: list_page\n    title: L\n    sections:\n      - {section}\n{Q}"
    )
}

/// Acceptance 5: a navigate parameter is a site wherever the action stands. A page of a built-in
/// kind that overrides the kind's `list` section without restating `component` (as
/// `examples/partner-portal/ui.yaml` does on `users.list`) still renders a collection whose row
/// actions navigate; the walk reads the document as written and finds no composite there.
#[test]
fn a_navigate_in_a_section_that_inherits_its_component_is_a_site() {
    let action = "row_actions: [{name: open, label: Open, \
                  navigate: {to: q, params: {id: section.nosuch.selection.id}}}]";
    let restated = run(&list_page(&format!(
        "{{name: list, component: collection, reads: t.Deals, columns: [id], {action}}}"
    )));
    assert_eq!(
        of(&restated, "section_refs").len(),
        1,
        "control, `component` restated: {:#?}",
        restated.findings
    );
    let inherited = run(&list_page(&format!(
        "{{name: list, reads: t.Deals, columns: [id], {action}}}"
    )));
    assert!(
        !inherited
            .findings
            .iter()
            .any(|finding| finding.check == "document_loads"),
        "{:#?}",
        inherited.findings
    );
    let found = of(&inherited, "section_refs");
    assert_eq!(
        found.len(),
        1,
        "`section.nosuch` in the navigate of a row action of an inherited collection: {:#?}",
        inherited.findings
    );
    assert_eq!(found[0].path, "pages/l/sections/list/row_actions/open");
}

/// The same at the two other places a composite is written without `component`: a `list_page`'s
/// `filters` bar whose `binds` (typed `{list: expr}`) the page restates, as every list page of
/// `examples/partner-portal/ui.yaml` does, and an overlay that copies another with `same_as`.
#[test]
fn a_filter_bar_and_a_same_as_overlay_that_inherit_their_component_are_sites() {
    let text = list_page(
        "{name: filters, binds: [state.nosuch_bar]}\n      \
         - {name: list, reads: t.Deals, columns: [id], row_actions: [{name: edit, opens: edit}, \
         {name: view, opens: view}]}\n    \
         overlays:\n      \
         edit: {kind: drawer, component: form, does: t.Edit, params: {key: row.id}, fields: [name]}\n      \
         view: {same_as: l.edit, title: View, fields: [{field: z, visible: params.nosuch_view}]}",
    );
    let report = run(&text);
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.check == "document_loads"),
        "{:#?}",
        report.findings
    );
    let found: Vec<&str> = of(&report, "expr_paths")
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        found,
        ["pages/l/overlays/view/fields/z", "pages/l/sections/filters"],
        "{:#?}",
        report.findings
    );
}

/// Acceptance 3 and 5: `checked` is typed `expr`, and the schema says it may use `contains`. A
/// state out of scope in it is the same error as in any other expression.
#[test]
fn a_state_out_of_scope_in_a_checked_that_uses_contains_is_an_error() {
    let page = |checked: &str| {
        format!(
            "{SHELL}navigation: {{home: p, sections: [{{name: all, pages: [p, q]}}]}}\n\
             pages:\n  p:\n    kind: detail_page\n    title: P\n    \
             state: {{search: {{type: string, class: page_state}}}}\n    sections:\n      \
             - {{name: deals, component: collection, reads: t.Deals, columns: [id], \
             row_actions: [{{name: star, label: Star, as: toggle, sets: {{state.search: toggle}}, \
             checked: \"{checked}\"}}]}}\n{Q}"
        )
    };
    let plain = run(&page("state.nosuch"));
    assert_eq!(
        of(&plain, "expr_paths").len(),
        1,
        "control, no `contains`: {:#?}",
        plain.findings
    );
    let contains = run(&page("row.tags contains state.nosuch"));
    let found = of(&contains, "expr_paths");
    assert_eq!(
        found.len(),
        1,
        "`state.nosuch` on the right of `contains`: {:#?}",
        contains.findings
    );
    assert!(found[0].message.contains("`state.nosuch`"), "{found:#?}");
}

/// One path, one finding: the walk leaves a read's row filter to the `filter_*` rules, which also
/// hold a dynamic menu entry's `filter`. There an unknown root is refused by `filter_roots`
/// (error) and also called literal text by `expr_roots` (warning).
#[test]
fn an_unknown_root_in_a_dynamic_menu_filter_is_reported_once() {
    let text = format!(
        "{SHELL}navigation: {{home: p, sections: [{{name: all, pages: [p]}}, \
         {{name: saved, label: Saved, pages: {{from_view: t.Saved, page: q, param: id, \
         filter: \"bogus.x == 'a'\"}}}}]}}\n\
         pages:\n  p:\n    kind: detail_page\n    title: P\n    sections:\n      \
         - {{name: deals, component: record, reads: t.Deals, fields: [id]}}\n{Q}"
    );
    let report = run(&text);
    let on_menu: Vec<&Finding> = report
        .findings
        .iter()
        .filter(|finding| finding.path.starts_with("navigation"))
        .filter(|finding| ["filter_roots", "expr_roots"].contains(&finding.check.as_str()))
        .collect();
    assert!(
        on_menu
            .iter()
            .any(|finding| finding.check == "filter_roots"),
        "precondition, the filter rules hold the menu filter: {:#?}",
        report.findings
    );
    assert_eq!(
        on_menu.len(),
        1,
        "`bogus.x` in one menu filter: {on_menu:#?}"
    );
}

/// The reference: a widget argument is an expression where it is substituted into an `expr`
/// position. Substituted only into `alt` (typed `string`), `diagram.png` is the text shown, and
/// the warning's remedy (write `"diagram.png"`) would show the quotes.
#[test]
fn a_widget_argument_substituted_only_as_text_is_no_expression() {
    let text = format!(
        "{SHELL}navigation: {{home: p, sections: [{{name: all, pages: [p, q]}}]}}\n\
         widgets:\n  pic:\n    summary: P\n    \
         params: {{src: {{type: string, required: true, note: n}}, \
         caption: {{type: string, required: true, note: n}}}}\n    \
         body: [{{name: img, primitive: image, src: args.src, alt: args.caption}}]\n\
         pages:\n  p:\n    kind: detail_page\n    title: P\n    sections:\n      \
         - {{name: deals, component: record, reads: t.Deals, fields: [id], \
         children: [{{name: logo, component: pic, args: {{src: row.logo, caption: diagram.png}}}}]}}\n{Q}"
    );
    let report = run(&text);
    assert!(
        !report
            .findings
            .iter()
            .any(|finding| finding.check == "document_loads"),
        "{:#?}",
        report.findings
    );
    assert!(
        of(&report, "expr_roots").is_empty(),
        "`caption` is only ever text: {:#?}",
        report.findings
    );
}
