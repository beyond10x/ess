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

// ── stands_in_for (story:a-host-slot-names-the-generator-item-it-stands-in-for) ──────────────

/// One section slot standing in for `stands_in_for`, written as given.
fn standing_in_for(stands_in_for: &str) -> String {
    document(
        &format!(
            "{{name: summary, reads: t.ById}}, \
             {{name: map, host: RouteMap, reason: a map widget, {stands_in_for}}}"
        ),
        "",
        "{kind: detail_page, title: E}",
    )
}

#[test]
fn stands_in_for_accepts_every_composite_every_primitive_and_none() {
    let items = ess_ui::COMPOSITE_KINDS
        .iter()
        .chain(ess_ui::PRIMITIVE_KINDS)
        .chain(&["none"]);
    for item in items {
        let report = report(&standing_in_for(&format!("stands_in_for: {item}")));
        assert!(errors(&report).is_empty(), "{item}: {:#?}", report.findings);
    }
}

#[test]
fn the_schema_lists_the_stand_ins_the_loader_accepts() {
    let schema: ess_yaml::Value = ess_yaml::from_str(ess_ui::SCHEMA).expect("the schema");
    let names = |value: &ess_yaml::Value| -> Vec<String> {
        value
            .as_sequence()
            .expect("a list")
            .iter()
            .map(|name| name.as_str().expect("a name").to_owned())
            .collect()
    };
    let primitives =
        names(&schema["constructs"]["Primitive"]["fields"]["primitive"]["type"]["enum"]);
    assert_eq!(primitives, ess_ui::PRIMITIVE_KINDS);
    let one_of = &schema["constructs"]["HostSlot"]["fields"]["stands_in_for"]["type"]["one_of"];
    assert_eq!(one_of[0]["ref"].as_str(), Some("composite_kind"));
    assert_eq!(names(&one_of[1]["enum"]), ess_ui::PRIMITIVE_KINDS);
    assert_eq!(one_of[2]["const"].as_str(), Some("none"));
}

#[test]
fn any_other_stands_in_for_is_refused_by_name() {
    for item in ["map_view", "Chart", "widget", "nothing"] {
        refused(
            &report(&standing_in_for(&format!("stands_in_for: {item}"))),
            "host_slots",
            "pages/p/sections/map",
            &["RouteMap", &format!("`{item}`")],
        );
    }
}

#[test]
fn a_none_slot_without_a_reason_is_refused() {
    let text = document(
        "{name: summary, reads: t.ById}, \
         {name: map, host: RouteMap, stands_in_for: none}",
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

/// The five-scope document with its overlay slot and a second field slot standing in for
/// nothing the generator has.
fn with_backlog() -> String {
    five_scopes()
        .replace(
            "captures a pen signature, \
                stands_in_for: form",
            "captures a pen signature, \
                stands_in_for: none",
        )
        .replace(
            "stands_in_for: rich_text}]",
            "stands_in_for: rich_text}, {field: colour, host: ColourWheel, \
             reason: no colour picker yet, stands_in_for: none}]",
        )
}

#[test]
fn the_reports_list_the_none_slots_as_the_generator_backlog() {
    let text = with_backlog();
    let report = report(&text);
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
    let backlog: Vec<(&str, &str, &str)> = report
        .backlog
        .iter()
        .map(|entry| {
            (
                entry.path.as_str(),
                entry.host.as_str(),
                entry.reason.as_str(),
            )
        })
        .collect();
    assert_eq!(
        backlog,
        [
            (
                "pages/p/overlays/sign",
                "SignaturePad",
                "captures a pen signature"
            ),
            (
                "pages/p/sections/summary/fields/colour",
                "ColourWheel",
                "no colour picker yet"
            ),
        ]
    );
    let text_report = report.render(OutputFormat::Text);
    for line in [
        "backlog pages/p/overlays/sign SignaturePad: captures a pen signature",
        "backlog pages/p/sections/summary/fields/colour ColourWheel: no colour picker yet",
    ] {
        assert!(
            text_report.lines().any(|written| written == line),
            "no line `{line}` in:\n{text_report}"
        );
    }
    let json: serde_json::Value =
        serde_json::from_str(&report.render(OutputFormat::Json)).expect("the report is JSON");
    assert_eq!(
        json["backlog"],
        serde_json::json!([
            {"path": "pages/p/overlays/sign", "host": "SignaturePad",
             "reason": "captures a pen signature"},
            {"path": "pages/p/sections/summary/fields/colour", "host": "ColourWheel",
             "reason": "no colour picker yet"},
        ])
    );
}

#[test]
fn a_document_without_none_slots_reports_as_before() {
    for text in [five_scopes(), standing_in_for("stands_in_for: chart")] {
        let report = report(&text);
        assert!(report.backlog.is_empty(), "{:#?}", report.backlog);
        let json: serde_json::Value =
            serde_json::from_str(&report.render(OutputFormat::Json)).expect("JSON");
        assert!(json.get("backlog").is_none(), "{json}");
        assert!(
            !report.render(OutputFormat::Text).contains("backlog"),
            "{}",
            report.render(OutputFormat::Text)
        );
    }
}

// ── access (story:a-host-slot-reads-and-commands-only-what-it-declares) ──────────────────────

/// The fixture model the model checks use: system `shop`, views `stock.Items` and
/// `audit.Entries`, commands `stock.AddItem` and `audit.RecordEntry`; only `stock` is granted to
/// an actor (`shop.stock.Clerk`, who may invoke `stock.AddItem`).
fn shop() -> ess_ui_check::Model {
    ess_ui_check::load_model(&crate_dir().join("tests/fixtures/model"))
        .unwrap_or_else(|error| panic!("{error}"))
}

/// A document of model `shop` whose page `p` is written as given, and whose page `editor` is
/// written as `editor`, or left out when it is empty.
fn shop_document(p: &str, editor: &str) -> String {
    let (pages, editor) = if editor.is_empty() {
        ("[p]", String::new())
    } else {
        ("[p, editor]", format!("\n  editor: {editor}"))
    };
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: {pages}}}]}}\n\
         pages:\n  p: {p}{editor}\n"
    )
}

fn checked(text: &str, model: Option<&ess_ui_check::Model>) -> Report {
    check_source(text, "test.yaml", &crate_dir(), model, &Options::default())
}

/// A slot at a section, an overlay, a field and a page, each naming a view or a command the model
/// does not have, beside a generated section reading a view it does not have either.
fn unknown_names() -> String {
    shop_document(
        "{kind: detail_page, title: P, sections: [\
           {name: map, host: StockMap, reason: a vendor map, stands_in_for: collection, \
            reads: stock.Missing, commands: [stock.Nope]}, \
           {name: list, component: collection, reads: stock.Missing}, \
           {name: summary, reads: stock.Items, fields: [{field: label, host: LabelEditor, \
            reason: needs spell checking, stands_in_for: rich_text, commands: [stock.Unknown]}]}], \
         overlays: {sign: {kind: dialog, host: SignaturePad, reason: a pen signature, \
                           stands_in_for: form, commands: [audit.Gone]}}}",
        "{kind: detail_page, title: E, host: FlowCanvas, reason: a vendor canvas, \
          stands_in_for: graph_editor, reads: audit.Absent}",
    )
}

#[test]
fn with_a_model_a_slots_reads_and_commands_resolve_as_a_generated_nodes_do() {
    let text = unknown_names();
    // Without a model nothing is resolved, so the slots stand.
    let unmodelled = checked(&text, None);
    assert!(errors(&unmodelled).is_empty(), "{:#?}", unmodelled.findings);
    let model = shop();
    let report = checked(&text, Some(&model));
    // The generated section's unknown read and the slot's are the same diagnostic.
    refused(
        &report,
        "view_in_model",
        "pages/p/sections/list/reads",
        &["`stock.Missing`", "`shop`"],
    );
    refused(
        &report,
        "view_in_model",
        "pages/p/sections/map/reads",
        &["`stock.Missing`", "`shop`"],
    );
    refused(
        &report,
        "view_in_model",
        "pages/editor/reads",
        &["`audit.Absent`"],
    );
    for (path, name) in [
        ("pages/p/sections/map/commands", "`stock.Nope`"),
        (
            "pages/p/sections/summary/fields/label/commands",
            "`stock.Unknown`",
        ),
        ("pages/p/overlays/sign/commands", "`audit.Gone`"),
    ] {
        refused(&report, "command_in_model", path, &[name, "`shop`"]);
    }
    assert_eq!(tripped(&report, "view_in_model").len(), 3, "{report:#?}");
    assert_eq!(tripped(&report, "command_in_model").len(), 3, "{report:#?}");
}

#[test]
fn a_slot_reading_a_view_no_actor_can_read_is_reported_as_a_section_would_be() {
    let model = shop();
    let text = shop_document(
        "{kind: detail_page, title: P, sections: [\
           {name: map, host: EntryMap, reason: a vendor map, stands_in_for: collection, \
            reads: audit.Entries}, \
           {name: list, component: collection, reads: audit.Entries}]}",
        "",
    );
    let report = checked(&text, Some(&model));
    for section in ["pages/p/sections/map", "pages/p/sections/list"] {
        refused(
            &report,
            "section_readable",
            section,
            &["`shop.audit.Entries`"],
        );
    }
}

#[test]
fn a_slot_sending_a_command_its_pages_actor_is_not_granted_is_reported() {
    let model = shop();
    let text = shop_document(
        "{kind: detail_page, title: P, actor: shop.stock.Clerk, sections: [\
           {name: summary, reads: stock.Items}, \
           {name: note, host: NotePad, reason: a vendor editor, stands_in_for: form, \
            commands: [stock.AddItem, audit.RecordEntry]}]}",
        "",
    );
    let report = checked(&text, Some(&model));
    refused(
        &report,
        "page_actor_grants",
        "pages/p/sections/note/commands",
        &["`shop.audit.RecordEntry`", "`shop.stock.Clerk`"],
    );
    assert_eq!(
        tripped(&report, "page_actor_grants").len(),
        1,
        "{report:#?}"
    );
}

#[test]
fn a_slot_declaring_only_what_the_model_serves_passes() {
    let model = shop();
    let text = shop_document(
        "{kind: detail_page, title: P, sections: [\
           {name: map, host: StockMap, reason: a vendor map, stands_in_for: collection, \
            reads: stock.Items, commands: [stock.AddItem]}]}",
        "",
    );
    let report = checked(&text, Some(&model));
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
}

#[test]
fn a_slot_with_no_reads_and_no_commands_is_accepted_and_reaches_no_row() {
    let model = shop();
    let text = shop_document(
        "{kind: detail_page, title: P, sections: [\
           {name: map, host: Decoration, reason: a vendor animation, stands_in_for: none}]}",
        "",
    );
    let report = checked(&text, Some(&model));
    assert!(errors(&report).is_empty(), "{:#?}", report.findings);
    let coverage = report
        .coverage
        .expect("a check with a model counts coverage");
    assert_eq!(
        (coverage.covered, coverage.host_covered, coverage.uncovered),
        (0, 0, 4)
    );
}

// ── coverage (story:host-covered-rows-are-their-own-count) ───────────────────────────────────

/// A generated section reading `stock.Items`, and — when `slot` holds — a page slot reading
/// `audit.Entries` and sending `audit.RecordEntry`, plus a section slot reading `stock.Items`,
/// which a generated node already reaches.
fn covered_document(slot: bool) -> String {
    let p = if slot {
        "{kind: detail_page, title: P, sections: [{name: list, component: collection, \
          reads: stock.Items}, {name: map, host: StockMap, reason: a vendor map, \
          stands_in_for: chart, reads: stock.Items}]}"
    } else {
        "{kind: detail_page, title: P, sections: [{name: list, component: collection, \
          reads: stock.Items}]}"
    };
    let editor = if slot {
        "{kind: detail_page, title: E, host: AuditCanvas, reason: a vendor canvas, \
          stands_in_for: graph_editor, reads: audit.Entries, commands: [audit.RecordEntry]}"
    } else {
        ""
    };
    shop_document(p, editor)
}

#[test]
fn the_rows_a_slot_reaches_are_host_covered_and_never_covered() {
    let model = shop();
    let with = checked(&covered_document(true), Some(&model));
    let without = checked(&covered_document(false), Some(&model));
    assert!(errors(&with).is_empty(), "{:#?}", with.findings);
    assert!(errors(&without).is_empty(), "{:#?}", without.findings);
    let with = with.coverage.expect("counted with a model");
    let without = without.coverage.expect("counted with a model");
    // The slot's view and command are host-covered; `stock.Items`, which a generated node reads
    // too, stays covered; `stock.AddItem` nobody reaches.
    assert_eq!((with.covered, with.host_covered, with.uncovered), (1, 2, 1));
    // The covered count is the one the document has with its slots removed.
    assert_eq!(with.covered, without.covered);
    assert_eq!(
        (without.covered, without.host_covered, without.uncovered),
        (1, 0, 3)
    );
}

#[test]
fn the_text_and_json_reports_carry_the_third_count_and_list_each_slot() {
    let model = shop();
    let report = checked(&covered_document(true), Some(&model));
    let text = report.render(OutputFormat::Text);
    for line in [
        "host pages/editor AuditCanvas stands in for graph_editor: a vendor canvas",
        "host pages/p/sections/map StockMap stands in for chart: a vendor map",
        "coverage: 1 covered, 2 host-covered, 1 uncovered of 4 views and commands",
    ] {
        assert!(
            text.lines().any(|written| written == line),
            "no line `{line}` in:\n{text}"
        );
    }
    let json: serde_json::Value =
        serde_json::from_str(&report.render(OutputFormat::Json)).expect("the report is JSON");
    assert_eq!(
        json["coverage"],
        serde_json::json!({"covered": 1, "host_covered": 2, "uncovered": 1})
    );
    assert_eq!(
        json["hosts"],
        serde_json::json!([
            {"path": "pages/editor", "host": "AuditCanvas", "reason": "a vendor canvas",
             "stands_in_for": "graph_editor"},
            {"path": "pages/p/sections/map", "host": "StockMap", "reason": "a vendor map",
             "stands_in_for": "chart"},
        ])
    );
    // Without a model the slots are listed and no row is counted: the rows are the model's.
    let unmodelled = checked(&covered_document(true), None);
    assert!(unmodelled.coverage.is_none());
    let json: serde_json::Value =
        serde_json::from_str(&unmodelled.render(OutputFormat::Json)).expect("JSON");
    assert!(json.get("coverage").is_none(), "{json}");
    assert_eq!(json["hosts"].as_array().map(Vec::len), Some(2), "{json}");
}

#[test]
fn a_document_without_slots_checked_without_a_model_renders_as_before() {
    let text = shop_document(
        "{kind: detail_page, title: P, sections: [{name: list, component: collection, \
          reads: stock.Items}]}",
        "",
    );
    let report = checked(&text, None);
    assert_eq!(report.hosts, Vec::new());
    let json: serde_json::Value =
        serde_json::from_str(&report.render(OutputFormat::Json)).expect("JSON");
    let mut keys: Vec<&str> = json
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["document", "findings", "format"], "{json}");
    let rendered = report.render(OutputFormat::Text);
    assert!(
        rendered
            .lines()
            .all(|line| !line.starts_with("host ") && !line.starts_with("coverage")),
        "{rendered}"
    );
}
