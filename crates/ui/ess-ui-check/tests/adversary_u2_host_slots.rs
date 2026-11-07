//! Adversary cases for host slots (story:a-ui-document-declares-a-host-slot,
//! story:a-host-slot-names-the-generator-item-it-stands-in-for), driven from the unit's own
//! schema entry and reference: `host` is typed `name`, `reason` is prose, a page slot "draws no
//! sections" and its header is still the document's, and `--hosts` reads one name per line.

use std::path::PathBuf;

use ess_ui_check::{check_source, exit_code, CheckArgs, Options, OutputFormat, Report, Severity};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn report(text: &str) -> Report {
    check_source(text, "test.yaml", &crate_dir(), None, &Options::default())
}

fn error_lines(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| format!("{} {}: {}", finding.check, finding.path, finding.message))
        .collect()
}

/// A page `p` with a summary section and `slot` as a second section, written as given.
fn with_section_slot(slot: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages:\n  \
           p: {{kind: detail_page, title: P, sections: [{{name: summary, reads: t.ById}}, {slot}]}}\n"
    )
}

/// `HostSlot.host` is `{type: name}` in the unit's schema, and `name` is "a node name; matches
/// `NodePath.syntax.segment_pattern`". A host name that is empty, holds a space or holds `#` (which
/// `--hosts` reads as a comment, so the name can never be registered) is not a name.
#[test]
fn adversary_u2_a_host_name_that_is_not_a_name_is_refused() {
    for host in ["\"\"", "\"Route Map\"", "\"Route#Map\""] {
        let text = with_section_slot(&format!(
            "{{name: map, host: {host}, reason: a map widget, stands_in_for: chart}}"
        ));
        let report = report(&text);
        assert!(
            report.has_errors(),
            "`host: {host}` is not a name (schema: HostSlot.host type name), and the check \
             accepted it; findings: {:#?}",
            report.findings
        );
    }
}

/// `reason` is prose ("why the generator cannot draw it"). A reason that mentions a channel the
/// host component listens to is not a binding expression of the document.
#[test]
fn adversary_u2_a_reason_is_prose_not_an_expression() {
    let text = with_section_slot(
        "{name: map, host: RouteMap, reason: follows the vendor's channel.trucks feed, \
          stands_in_for: none}",
    );
    let report = report(&text);
    assert_eq!(error_lines(&report), Vec::<String>::new());
}

/// A page slot "draws no sections" and inherits neither its kind's sections nor their layout,
/// while "its header and overlays are still the document's". The built-in `list_page` kind's
/// header names its `list` and `filters` sections, which the slot page never has: the author
/// wrote no reference to either.
#[test]
fn adversary_u2_a_page_slot_over_a_list_page_is_not_refused_for_its_kinds_header() {
    let text = "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
                shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
                navigation: {home: p, sections: [{name: all, pages: [p, grid]}]}\n\
                pages:\n  \
                  p: {kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}\n  \
                  grid: {kind: list_page, title: Grid, host: VendorGrid, \
                         reason: a vendor data grid, stands_in_for: collection}\n";
    let report = report(text);
    assert_eq!(error_lines(&report), Vec::<String>::new());
}

/// The backlog is one text line per `none` slot. A long reason is written the way the schema
/// writes its own prose, as a folded block, which YAML ends with a newline.
#[test]
fn adversary_u2_a_folded_reason_keeps_the_text_report_one_line_per_entry() {
    let text = "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
                shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
                navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
                pages:\n  \
                  p:\n    \
                    kind: detail_page\n    \
                    title: P\n    \
                    sections:\n      \
                      - {name: summary, reads: t.ById}\n      \
                      - name: map\n        \
                        host: RouteMap\n        \
                        stands_in_for: none\n        \
                        reason: >\n          \
                          the generator has no map composite, and the vendor map\n          \
                          draws its own tiles\n";
    let report = report(text);
    assert_eq!(report.backlog.len(), 1, "{:#?}", report.findings);
    let rendered = report.render(OutputFormat::Text);
    assert!(
        rendered.lines().all(|line| !line.is_empty()),
        "the text report holds an empty line:\n{rendered:?}"
    );
}

/// `--hosts` reads one name per line. A file saved with a byte-order mark (as some editors on
/// Windows save text) registers `RouteMap`, not `\u{feff}RouteMap`.
#[test]
fn adversary_u2_a_hosts_file_with_a_byte_order_mark_registers_its_first_name() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("adversary_u2_hosts_bom");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the test directory");
    let ui = root.join("ui.yaml");
    std::fs::write(
        &ui,
        with_section_slot(
            "{name: map, host: RouteMap, reason: a map widget, stands_in_for: chart}",
        ),
    )
    .expect("the document");
    let registered = root.join("hosts.txt");
    std::fs::write(&registered, "\u{feff}RouteMap\r\n").expect("the hosts file");
    let args = CheckArgs {
        path: ui,
        model: None,
        format: OutputFormat::Text,
        lacks: Vec::new(),
        hosts: Some(registered),
    };
    let code = exit_code(&args).expect("the check runs");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        code, 0,
        "a BOM-prefixed hosts file registers a name no slot declares"
    );
}

/// Probes that should hold: a slot at a shell overlay, a collection column and a header metric
/// is a slot (the generators refuse by `host_slots`), and `stands_in_for: None` or `null` is
/// refused.
#[test]
fn adversary_u2_slots_at_every_node_position_are_found() {
    let text = "format: ess-ui/1\napp: t\nmodel: t.system\nplacement_profile: fat\n\
                shells: {app: {regions: {main: {kind: page_outlet}}, \
                  overlays: {help: {kind: drawer, host: HelpPanel, reason: vendor help, \
                                    stands_in_for: none}}}}\n\
                navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
                pages:\n  \
                  p: {kind: detail_page, title: P, \
                      header: {title: P, metrics: [{name: load, host: Gauge, reason: a dial, \
                                                    stands_in_for: metric}]}, \
                      sections: [{name: summary, component: collection, reads: t.All, \
                                  columns: [name, {field: avatar, host: Avatar, reason: crops, \
                                                   stands_in_for: image}]}]}\n";
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let mut found: Vec<String> = document
        .host_slots()
        .into_iter()
        .map(|(path, slot)| format!("{path} {}", slot.host))
        .collect();
    found.sort();
    assert_eq!(
        found,
        [
            "pages/p/header/metrics/load Gauge",
            "pages/p/sections/summary/columns/avatar Avatar",
            "shells/app/overlays/help HelpPanel",
        ]
    );
    for item in ["None", "null", "\"\""] {
        let report = report(&with_section_slot(&format!(
            "{{name: map, host: RouteMap, reason: a map widget, stands_in_for: {item}}}"
        )));
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.check == "host_slots"
                    && finding.path == "pages/p/sections/map"),
            "{item}: {:#?}",
            report.findings
        );
    }
}
