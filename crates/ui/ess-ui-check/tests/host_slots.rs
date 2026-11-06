//! Host slots in `ess-ui/1` (design `ui-host-slots`): a declared place where the host application
//! supplies its own component, which ESS does not draw.
//!
//! A slot stands where a node may stand: a page, a section, an overlay, a tab (as the tab's
//! `form` node) or a field. It carries `host`, `reason` and `stands_in_for`, and optionally
//! `reads` and `commands`. `ess ui check` refuses a slot without a reason, two slots of one host
//! name, a slot beside `component` or `primitive`, and with `--hosts` a registered name no slot
//! declares.

use std::path::PathBuf;

use ess_ui_check::{check_source, exit_code, CheckArgs, Finding, Options, OutputFormat, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// A document of two pages: `p`, whose sections, overlays and pages are written as given, and
/// `editor`, written as `editor`.
fn document(sections: &str, overlays: &str, editor: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p, editor]}}]}}\n\
         pages:\n  \
           p: {{kind: detail_page, title: P, sections: [{sections}], overlays: {{{overlays}}}}}\n  \
           editor: {editor}\n"
    )
}

/// A slot at each of the five scopes.
fn five_scopes() -> String {
    document(
        "{name: summary, reads: t.ById, \
          fields: [name, {field: notes, host: NotesEditor, reason: needs spell checking, \
                          stands_in_for: rich_text}], \
          tabs: [{name: history, form: {host: AuditTrail, reason: draws a timeline, \
                                        stands_in_for: collection, reads: t.History}}]}, \
         {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart, reads: t.Routes}",
        "sign: {kind: dialog, host: SignaturePad, reason: captures a pen signature, \
                stands_in_for: form, commands: [t.Sign]}",
        "{kind: detail_page, title: Editor, host: FlowCanvas, reason: a vendor canvas, \
          stands_in_for: graph_editor, reads: t.Flow, commands: [t.SaveFlow]}",
    )
}

fn report_with(text: &str, options: &Options) -> Report {
    check_source(text, "test.yaml", &crate_dir(), None, options)
}

fn report(text: &str) -> Report {
    report_with(text, &Options::default())
}

fn errors(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == ess_ui_check::Severity::Error)
        .collect()
}

fn tripped<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == id)
        .collect()
}

/// Asserts the report has a `check` error at `path` whose message holds every one of `words`.
fn refused(report: &Report, check: &str, path: &str, words: &[&str]) {
    let found = tripped(report, check);
    assert!(
        found.iter().any(|finding| finding.path == path
            && words.iter().all(|word| finding.message.contains(word))),
        "no `{check}` finding at `{path}` naming {words:?}; findings: {:#?}",
        report.findings
    );
    assert!(report.has_errors());
}

#[test]
fn a_host_slot_stands_at_a_page_a_section_an_overlay_a_tab_and_a_field() {
    let text = five_scopes();
    let report = report(&text);
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let slots: Vec<(String, &str, &str, &str)> = document
        .host_slots()
        .into_iter()
        .map(|(path, slot)| {
            (
                path.to_string(),
                slot.host.as_str(),
                slot.reason.as_str(),
                slot.stands_in_for.as_str(),
            )
        })
        .collect();
    assert_eq!(
        slots,
        vec![
            (
                "pages/editor".to_owned(),
                "FlowCanvas",
                "a vendor canvas",
                "graph_editor"
            ),
            (
                "pages/p/sections/summary/fields/notes".to_owned(),
                "NotesEditor",
                "needs spell checking",
                "rich_text"
            ),
            (
                "pages/p/sections/summary/tabs/history/form".to_owned(),
                "AuditTrail",
                "draws a timeline",
                "collection"
            ),
            (
                "pages/p/sections/map".to_owned(),
                "RouteMap",
                "a map widget",
                "chart"
            ),
            (
                "pages/p/overlays/sign".to_owned(),
                "SignaturePad",
                "captures a pen signature",
                "form"
            ),
        ]
    );
    let editor = &document.pages["editor"];
    assert!(editor.sections.is_empty(), "{editor:#?}");
    let editor_slot = editor.host.as_ref().expect("the editor page is a slot");
    assert_eq!(
        editor_slot
            .reads
            .as_ref()
            .and_then(|reads| reads.view.as_deref()),
        Some("t.Flow")
    );
    assert_eq!(editor_slot.commands, vec!["t.SaveFlow".to_owned()]);
}

#[test]
fn a_host_slot_without_a_reason_or_with_an_empty_one_is_refused() {
    for reason in ["", "reason: \"\", ", "reason: \"  \", "] {
        let text = document(
            &format!("{{name: map, host: RouteMap, {reason}stands_in_for: chart}}"),
            "",
            "{kind: detail_page, title: E}",
        );
        refused(
            &report(&text),
            "host_slots",
            "pages/p/sections/map",
            &["RouteMap", "reason"],
        );
    }
    let field = document(
        "{name: summary, reads: t.ById, fields: [{field: notes, host: NotesEditor, \
          stands_in_for: rich_text}]}",
        "",
        "{kind: detail_page, title: E}",
    );
    refused(
        &report(&field),
        "host_slots",
        "pages/p/sections/summary/fields/notes",
        &["NotesEditor", "reason"],
    );
    let page = document(
        "{name: summary, reads: t.ById}",
        "",
        "{kind: detail_page, title: E, host: FlowCanvas, stands_in_for: graph_editor}",
    );
    refused(
        &report(&page),
        "host_slots",
        "pages/editor",
        &["FlowCanvas", "reason"],
    );
}

#[test]
fn a_host_slot_without_stands_in_for_is_refused() {
    let text = document(
        "{name: map, host: RouteMap, reason: a map widget}",
        "",
        "{kind: detail_page, title: E}",
    );
    refused(
        &report(&text),
        "host_slots",
        "pages/p/sections/map",
        &["RouteMap", "stands_in_for"],
    );
}

#[test]
fn two_slots_with_one_host_name_are_refused() {
    let text = document(
        "{name: summary, reads: t.ById}, \
         {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart}",
        "routes: {kind: drawer, host: RouteMap, reason: again, stands_in_for: chart}",
        "{kind: detail_page, title: E}",
    );
    let report = report(&text);
    refused(
        &report,
        "host_slots",
        "pages/p/overlays/routes",
        &["RouteMap", "pages/p/sections/map"],
    );
}

#[test]
fn a_host_beside_a_component_or_a_primitive_key_is_refused() {
    let component = document(
        "{name: map, component: chart, reads: t.Routes, host: RouteMap, reason: a map widget, \
          stands_in_for: chart}",
        "",
        "{kind: detail_page, title: E}",
    );
    refused(
        &report(&component),
        "host_slots",
        "pages/p/sections/map",
        &["RouteMap", "`component`"],
    );
    let primitive = document(
        "{name: summary, reads: t.ById, children: [{name: hint, primitive: text, text: T, \
          host: Hint, reason: a tooltip, stands_in_for: text}]}",
        "",
        "{kind: detail_page, title: E}",
    );
    refused(
        &report(&primitive),
        "host_slots",
        "pages/p/sections/summary/children/hint",
        &["Hint", "`primitive`"],
    );
}

#[test]
fn a_page_slot_draws_no_sections_of_its_own() {
    let text = document(
        "{name: summary, reads: t.ById}",
        "",
        "{kind: detail_page, title: E, host: FlowCanvas, reason: a vendor canvas, \
          stands_in_for: graph_editor, sections: [{name: summary, reads: t.ById}]}",
    );
    refused(
        &report(&text),
        "host_slots",
        "pages/editor",
        &["FlowCanvas", "`sections`"],
    );
}

#[test]
fn a_page_without_sections_or_host_is_still_refused() {
    let text = "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
                shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
                navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
                page_kinds: {bare: {purpose: nothing}}\n\
                pages: {p: {kind: bare, title: P}}\n";
    let report = report(text);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.path == "pages/p" && finding.message.contains("sections")),
        "{:#?}",
        report.findings
    );
    assert!(report.has_errors());
}

#[test]
fn a_registered_name_no_slot_declares_is_refused() {
    let text = five_scopes();
    let declared = Options {
        hosts: Some(vec!["RouteMap".to_owned(), "FlowCanvas".to_owned()]),
        ..Options::default()
    };
    let report_declared = report_with(&text, &declared);
    assert!(
        errors(&report_declared).is_empty(),
        "{:#?}",
        report_declared.findings
    );
    let ghost = Options {
        hosts: Some(vec!["RouteMap".to_owned(), "Ghost".to_owned()]),
        ..Options::default()
    };
    let report_ghost = report_with(&text, &ghost);
    refused(&report_ghost, "hosts_registered", "/", &["`Ghost`"]);
    assert_eq!(tripped(&report_ghost, "hosts_registered").len(), 1);
}

#[test]
fn ess_ui_check_hosts_reads_one_name_per_line() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("host_slots_hosts_file");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test directory");
    let ui = root.join("ui.yaml");
    std::fs::write(&ui, five_scopes()).expect("the document");
    let registered = root.join("hosts.txt");
    std::fs::write(
        &registered,
        "# names the host registers\nRouteMap\n\n  FlowCanvas  \n",
    )
    .expect("the hosts file");
    let mut args = CheckArgs {
        path: ui.clone(),
        model: None,
        format: OutputFormat::Text,
        lacks: Vec::new(),
        hosts: Some(registered.clone()),
    };
    assert_eq!(exit_code(&args).expect("the check runs"), 0);
    std::fs::write(&registered, "RouteMap\nGhost\n").expect("the hosts file");
    assert_eq!(exit_code(&args).expect("the check runs"), 1);
    args.hosts = Some(root.join("missing.txt"));
    let error = exit_code(&args).expect_err("an unreadable hosts file is no report");
    assert!(error.to_string().contains("missing.txt"), "{error}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_schema_and_the_reference_describe_the_host_slot() {
    let schema: ess_yaml::Value = ess_yaml::from_str(ess_ui::SCHEMA).expect("the schema");
    let fields = schema["constructs"]["HostSlot"]["fields"]
        .as_mapping()
        .expect("the schema declares HostSlot with fields");
    let keys: Vec<&str> = fields.keys().filter_map(|key| key.as_str()).collect();
    assert_eq!(
        keys,
        ["host", "reason", "stands_in_for", "reads", "commands"],
        "{fields:?}"
    );
    for required in ["host", "reason", "stands_in_for"] {
        assert_eq!(
            fields[required]["required"].as_bool(),
            Some(true),
            "`{required}` is required"
        );
    }
    let page =
        std::fs::read_to_string(crate_dir().join("../../../website/docs/reference/ess-ui.md"))
            .expect("the reference page");
    assert!(
        page.contains("### HostSlot"),
        "the reference has no HostSlot section"
    );
    for scope in ["page", "section", "overlay", "tab", "field"] {
        assert!(
            schema["constructs"]["HostSlot"]["doc"]
                .as_str()
                .is_some_and(|doc| doc.contains(scope)),
            "the HostSlot doc names no `{scope}` scope"
        );
    }
}
