//! `ess ui check` resolves the root and the section of every binding path, without `--model`:
//! `section.<name>` and its member (`section_refs`), `state.<name>`, `params.<name>` and
//! `shell.<name>` (`expr_paths`), and a dotted word whose first segment is no root, which stays
//! literal text and draws a warning (`expr_roots`). The sites are the document's expression
//! fields, so each case stands in a read's `params` and in a navigate's `params`, and one case
//! stands at each further kind of site.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A document whose page `p` reads `read` in the params of a read and sends `nav` in the params of
/// a navigate; page `q` takes the parameter `id`.
fn document(read: &str, nav: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}, \
         state: {{theme: {{type: string, class: preference, store: local_storage}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p, q]}}]}}\n\
         pages:\n  p:\n    kind: detail_page\n    title: P\n    \
         state: {{search: {{type: string, class: page_state}}}}\n    sections:\n      \
         - {{name: deals, component: collection, reads: t.Deals, columns: [id], \
         row_actions: [{{name: open, label: Open, navigate: {{to: q, params: {{id: {nav}}}}}}}]}}\n      \
         - {{name: detail, component: record, reads: {{view: t.Detail, params: {{deal_id: {read}}}}}, \
         fields: [id]}}\n  \
         q:\n    kind: detail_page\n    title: Q\n    params: {{id: string}}\n    sections:\n      \
         - {{name: only, component: record, reads: {{view: t.ById, params: {{id: params.id}}}}, \
         fields: [id]}}\n"
    )
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

const CHECKS_OF_THIS_STORY: [&str; 3] = ["section_refs", "expr_paths", "expr_roots"];

fn story_findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| CHECKS_OF_THIS_STORY.contains(&finding.check.as_str()))
        .collect()
}

/// The two sites of a path: the read's params, then the navigate's.
fn at_both_sites(path: &str) -> [(String, &'static str, &'static str); 2] {
    let fine = "section.deals.selection.id";
    [
        (
            document(path, fine),
            "pages/p/sections/detail",
            "reads/params/deal_id",
        ),
        (
            document(fine, path),
            "pages/p/sections/deals/row_actions/open",
            "navigate/params/id",
        ),
    ]
}

/// One finding of `check` at each site, with `severity`, naming every fragment of `names`.
fn found_at_both_sites(path: &str, check: &str, severity: Severity, names: &[&str]) {
    for (text, node, field) in at_both_sites(path) {
        let report = run(&text);
        let found = of(&report, check);
        assert_eq!(found.len(), 1, "{path} at {node}: {:#?}", report.findings);
        assert_eq!(found[0].path, node, "{path}: {found:#?}");
        assert_eq!(found[0].severity, severity, "{path}: {found:#?}");
        assert!(
            found[0].message.contains(field),
            "{path}: names its field `{field}`: {found:#?}"
        );
        for name in names {
            assert!(
                found[0].message.contains(name),
                "{path}: names {name}: {found:#?}"
            );
        }
        assert_eq!(
            story_findings(&report).len(),
            1,
            "{path}: one finding of this story: {:#?}",
            report.findings
        );
    }
}

#[test]
fn the_document_the_cases_vary_is_clean() {
    let report = run(&document(
        "section.deals.selection.id",
        "section.deals.row.id",
    ));
    assert!(!report.has_errors(), "{:#?}", report.findings);
    assert!(story_findings(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_section_the_page_does_not_have_is_an_error() {
    found_at_both_sites(
        "section.nosuch.selection.id",
        "section_refs",
        Severity::Error,
        &["`section.nosuch`", "`deals`", "`detail`"],
    );
}

#[test]
fn a_member_the_reference_does_not_list_is_an_error() {
    found_at_both_sites(
        "section.deals.junk.id",
        "section_refs",
        Severity::Error,
        &["`section.deals.junk`", "`selection`", "`row`"],
    );
}

#[test]
fn the_listed_members_of_a_section_resolve() {
    for path in ["section.deals.selection.id", "section.deals.row.id"] {
        for (text, _, _) in at_both_sites(path) {
            let report = run(&text);
            assert!(
                story_findings(&report).is_empty(),
                "{path}: {:#?}",
                report.findings
            );
            assert!(!report.has_errors(), "{path}: {:#?}", report.findings);
        }
    }
}

#[test]
fn an_unknown_root_is_a_warning_naming_the_roots_and_the_quoted_form() {
    found_at_both_sites(
        "bogusroot.x",
        "expr_roots",
        Severity::Warning,
        &[
            "`bogusroot.x`",
            "`section`",
            "`state`",
            "`actor`",
            "literal",
            "\"bogusroot.x\"",
        ],
    );
    for (text, _, _) in at_both_sites("bogusroot.x") {
        assert!(!run(&text).has_errors());
    }
}

#[test]
fn a_domain_name_is_a_warning_and_stays_text() {
    found_at_both_sites(
        "example.com",
        "expr_roots",
        Severity::Warning,
        &["`example.com`"],
    );
}

#[test]
fn quoted_text_a_number_and_an_undotted_word_draw_nothing() {
    for value in [
        "'\"example.com\"'",
        "\"'example.com'\"",
        "'3'",
        "'1.5'",
        "'-2.5'",
        "'10.0.0.1'",
        "plain",
        "'true'",
    ] {
        for (text, _, _) in at_both_sites(value) {
            let report = run(&text);
            assert!(
                story_findings(&report).is_empty(),
                "{value}: {:#?}",
                report.findings
            );
        }
    }
}

#[test]
fn a_state_a_parameter_and_a_shell_state_out_of_scope_are_errors() {
    found_at_both_sites(
        "state.nosuch",
        "expr_paths",
        Severity::Error,
        &["`state.nosuch`", "`search`"],
    );
    found_at_both_sites(
        "params.nosuch",
        "expr_paths",
        Severity::Error,
        &["`params.nosuch`", "no parameter"],
    );
    found_at_both_sites(
        "shell.nosuch",
        "expr_paths",
        Severity::Error,
        &["`shell.nosuch`", "`theme`"],
    );
}

#[test]
fn a_state_a_parameter_and_a_shell_state_in_scope_resolve() {
    for path in ["state.search", "state.search.from", "shell.theme"] {
        for (text, _, _) in at_both_sites(path) {
            let report = run(&text);
            assert!(
                story_findings(&report).is_empty(),
                "{path}: {:#?}",
                report.findings
            );
        }
    }
    // `params.id` is the parameter of page `q`, read in `q` by the clean document.
    let report = run(&document(
        "section.deals.selection.id",
        "section.deals.selection.id",
    ));
    assert!(story_findings(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_bind_a_visible_and_a_field_value_are_sites() {
    let text = document("section.deals.selection.id", "section.deals.selection.id")
        .replace(
            "navigate: {to: q, params: {id: section.deals.selection.id}}}]",
            "navigate: {to: q, params: {id: section.deals.selection.id}}}, \
             {name: drop, label: Drop, does: t.Drop, bind: {id: state.nosuch}}]",
        )
        .replace(
            "fields: [id]}\n  q:",
            "fields: [id], visible: section.nosuch.row, \
             children: [{name: tone, primitive: badge, field: id, \
             tone_by: {value: shell.nosuch, map: {a: info}}}]}\n  q:",
        );
    let report = run(&text);
    let by_check = |check| -> Vec<(&str, &str)> {
        of(&report, check)
            .into_iter()
            .map(|finding| (finding.path.as_str(), finding.message.as_str()))
            .collect()
    };
    let paths: Vec<&str> = of(&report, "expr_paths")
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert_eq!(
        paths,
        [
            "pages/p/sections/deals/row_actions/drop",
            "pages/p/sections/detail/children/tone"
        ],
        "{:#?}",
        report.findings
    );
    let sections = by_check("section_refs");
    assert_eq!(sections.len(), 1, "{:#?}", report.findings);
    assert_eq!(sections[0].0, "pages/p/sections/detail");
    assert!(sections[0].1.contains("(in `visible`)"), "{sections:?}");
}

#[test]
fn a_guards_when_and_a_widgets_args_are_sites() {
    let guarded = document("section.deals.selection.id", "section.deals.selection.id").replace(
        "state: {theme: {type: string, class: preference, store: local_storage}}}}",
        "state: {theme: {type: string, class: preference, store: local_storage}}, \
         guards: [{name: stale, when: shell.nosuch, then: {refuse: nope}}, \
         {name: odd, when: bogusroot.x, then: {refuse: nope}}]}}",
    );
    let report = run(&guarded);
    let paths: Vec<(&str, &str)> = report
        .findings
        .iter()
        .filter(|finding| ["expr_paths", "expr_roots"].contains(&finding.check.as_str()))
        .map(|finding| (finding.check.as_str(), finding.path.as_str()))
        .collect();
    assert_eq!(
        paths,
        [
            ("expr_roots", "shells/app/guards/odd"),
            ("expr_paths", "shells/app/guards/stale")
        ],
        "{:#?}",
        report.findings
    );

    let widget = document("section.deals.selection.id", "section.deals.selection.id")
        .replace(
            "pages:\n",
            "widgets:\n  w:\n    summary: W\n    params: {x: {type: string, required: true, note: n}}\n    \
             body: [{name: t, primitive: text, text: args.x}]\npages:\n",
        )
        .replace(
            "fields: [id]}\n  q:",
            "fields: [id], children: [{name: use, component: w, args: {x: state.nosuch}}]}\n  q:",
        );
    let report = run(&widget);
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/sections/detail/children/use");
    assert!(found[0].message.contains("(in `args/x`)"), "{found:?}");
}

/// The clean document with the overlays `overlays` (flow-mapping entries) on page `p`.
fn with_overlays(overlays: &str) -> String {
    document("section.deals.selection.id", "section.deals.selection.id").replace(
        "class: page_state}}\n    sections:\n",
        &format!("class: page_state}}}}\n    overlays: {{{overlays}}}\n    sections:\n"),
    )
}

#[test]
fn params_in_an_overlay_that_declares_none_are_not_judged() {
    let text = with_overlays(
        "o: {kind: drawer, component: record, title: T, \
         reads: {view: t.ById, params: {id: params.id, other: params.anything}}, fields: [id]}",
    );
    let report = run(&text);
    assert!(story_findings(&report).is_empty(), "{:#?}", report.findings);
    assert!(!report.has_errors(), "{:#?}", report.findings);
}

#[test]
fn an_overlay_that_declares_params_is_held_to_them() {
    let text = with_overlays(
        "o: {kind: drawer, component: record, title: T, params: {key: row.id}, \
         reads: {view: t.ById, params: {id: params.key, other: params.nosuch}}, fields: [id]}",
    );
    let report = run(&text);
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/overlays/o");
    assert!(
        found[0].message.contains("`params.nosuch`") && found[0].message.contains("`key`"),
        "{found:#?}"
    );
}

#[test]
fn a_bare_section_is_resolved_by_its_name_only() {
    for (text, _, _) in at_both_sites("section.deals") {
        let report = run(&text);
        assert!(story_findings(&report).is_empty(), "{:#?}", report.findings);
    }
    found_at_both_sites(
        "section.nosuch",
        "section_refs",
        Severity::Error,
        &["`section.nosuch`"],
    );
}

/// An action's `sets` is a map keyed by `expr`: the key is the state written, and a state out of
/// scope there is the error.
#[test]
fn a_key_typed_as_an_expression_is_a_site() {
    let with_sets = |key: &str| {
        document("section.deals.selection.id", "section.deals.selection.id").replace(
            "navigate: {to: q, params: {id: section.deals.selection.id}}}]",
            &format!(
                "navigate: {{to: q, params: {{id: section.deals.selection.id}}}}}}, \
                 {{name: star, label: Star, sets: {{{key}: toggle}}}}]"
            ),
        )
    };
    let report = run(&with_sets("state.search"));
    assert!(story_findings(&report).is_empty(), "{:#?}", report.findings);
    let report = run(&with_sets("state.nosuch"));
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/sections/deals/row_actions/star");
    assert!(found[0].message.contains("`state.nosuch`"), "{found:#?}");
}

/// A guard's `then` is a `one_of` of a refusal, a redirect and a `set` map: the `set` is reached
/// through it, with its key and its value read as expressions.
#[test]
fn a_type_that_is_one_of_several_is_followed() {
    let guarded = |then: &str| {
        document("section.deals.selection.id", "section.deals.selection.id").replace(
            "state: {theme: {type: string, class: preference, store: local_storage}}}}",
            &format!(
                "state: {{theme: {{type: string, class: preference, store: local_storage}}}}, \
                 guards: [{{name: g, when: url.query.x, then: {then}}}]}}}}"
            ),
        )
    };
    let report = run(&guarded("{set: {shell.theme: url.query.x}}"));
    assert!(story_findings(&report).is_empty(), "{:#?}", report.findings);
    let report = run(&guarded("{set: {shell.theme: bogusroot.x}}"));
    assert_eq!(of(&report, "expr_roots").len(), 1, "{:#?}", report.findings);
    let report = run(&guarded("{set: {shell.nosuch: url.query.x}}"));
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "shells/app/guards/g");
}

/// A widget argument is an expression where the widget's declaration substitutes it into an
/// `expr` position, also when the declaration only hands it on as the argument of another widget;
/// where it lands in text alone (an image's `alt`), a dotted word is what is shown.
#[test]
fn a_widget_argument_is_an_expression_where_the_declaration_substitutes_it_into_one() {
    let widgets = "widgets:\n  card:\n    summary: C\n    \
         params: {who: {type: string, required: true, note: n}, \
         cap: {type: string, required: true, note: n}}\n    \
         body: [{name: t, primitive: text, text: args.who}, \
         {name: i, primitive: image, src: row.logo, alt: args.cap}]\n  \
         wrap:\n    summary: W\n    \
         params: {who: {type: string, required: true, note: n}, \
         cap: {type: string, required: true, note: n}}\n    \
         body: [{name: c, component: card, args: {who: args.who, cap: args.cap}}]\n";
    let use_of = |widget: &str, args: &str| {
        document("section.deals.selection.id", "section.deals.selection.id")
            .replace("pages:\n", &format!("{widgets}pages:\n"))
            .replace(
                "fields: [id]}\n  q:",
                &format!(
                    "fields: [id], children: [{{name: use, component: {widget}, args: {args}}}]}}\n  q:"
                ),
            )
    };
    for widget in ["card", "wrap"] {
        let report = run(&use_of(widget, "{who: stat.mine, cap: diagram.png}"));
        assert!(
            !report.findings.iter().any(|f| f.check == "document_loads"),
            "{widget}: {:#?}",
            report.findings
        );
        let found = of(&report, "expr_roots");
        assert_eq!(found.len(), 1, "{widget}: {:#?}", report.findings);
        assert!(found[0].message.contains("`stat.mine`"), "{found:#?}");
        assert!(found[0].message.contains("(in `args/who`)"), "{found:#?}");
    }
}

/// A parameter read inside the arguments of a call alone is no `expr` position (a dotted word is
/// a command name there); one also read outside a call is.
#[test]
fn a_parameter_read_inside_and_outside_a_call_is_an_expression() {
    let widgets = |body: &str| {
        document("section.deals.selection.id", "section.deals.selection.id")
            .replace(
                "pages:\n",
                &format!(
                    "widgets:\n  gated:\n    summary: G\n    \
                     params: {{cmd: {{type: string, required: true, note: n}}}}\n    \
                     body: [{body}]\npages:\n"
                ),
            )
            .replace(
                "fields: [id]}\n  q:",
                "fields: [id], children: [{name: use, component: gated, \
                 args: {cmd: orders.Approve}}]}\n  q:",
            )
    };
    let report = run(&widgets(
        "{name: t, primitive: text, text: Hi, visible: actor.may(args.cmd)}",
    ));
    assert!(
        of(&report, "expr_roots").is_empty(),
        "{:#?}",
        report.findings
    );
    let report = run(&widgets(
        "{name: t, primitive: text, text: args.cmd, visible: actor.may(args.cmd)}",
    ));
    assert_eq!(of(&report, "expr_roots").len(), 1, "{:#?}", report.findings);
}

/// A page kind that extends a built-in kind and restates its section without `component` is
/// walked with the composite the built-in kind gives that section.
#[test]
fn a_kind_restating_a_built_in_kinds_section_is_walked_as_its_composite() {
    let text = "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
         navigation: {home: k1, sections: [{name: all, pages: [k1]}]}\n\
         page_kinds:\n  mine:\n    extends: detail_page\n    purpose: x\n    sections:\n      \
         - {name: summary, reads: t.ById, fields: [id], \
         item: [{name: t, primitive: text, text: bogus.word}]}\n\
         pages:\n  k1:\n    kind: mine\n    title: K\n    sections:\n      - {name: summary}\n";
    let report = run(text);
    let found = of(&report, "expr_roots");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert!(found[0].path.starts_with("page_kinds/mine"), "{found:#?}");
}

/// An argument may name state the widget's body declares on the node that reads it or on a node
/// around it; state that neither the use nor the body declares is still an error.
#[test]
fn an_argument_may_name_state_the_body_declares_and_no_other() {
    let widgets = |shown: &str| {
        document("section.deals.selection.id", "section.deals.selection.id")
            .replace(
                "pages:\n",
                "widgets:\n  disc:\n    summary: D\n    \
                 params: {shown: {type: string, required: true, note: n}}\n    \
                 body: [{name: outer, component: record, reads: t.ById, fields: [id], \
                 state: {note: {type: string, class: component_state}}, \
                 item: [{name: t, primitive: text, text: args.shown}]}]\npages:\n",
            )
            .replace(
                "fields: [id]}\n  q:",
                &format!(
                    "fields: [id], children: [{{name: use, component: disc, \
                     args: {{shown: {shown}}}}}]}}\n  q:"
                ),
            )
    };
    let report = run(&widgets("state.note"));
    assert!(
        of(&report, "expr_paths").is_empty(),
        "{:#?}",
        report.findings
    );
    let report = run(&widgets("state.other"));
    assert_eq!(of(&report, "expr_paths").len(), 1, "{:#?}", report.findings);
}
