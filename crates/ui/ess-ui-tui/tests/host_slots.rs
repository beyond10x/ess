//! A document with a host slot is refused by the terminal renderer, naming the slot, until the
//! terminal draws host slots (0.91.0). `ess generate ui --target tui` and `ess ui run --tui` both
//! hold the document to the terminal's profile first, so this refusal is theirs.

const DOCUMENT: &str = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: P
    sections:
      - {name: summary, reads: t.ById, fields: [name]}
      - {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart}
";

#[test]
fn the_terminal_refuses_a_host_slot_by_name() {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let refusal = ess_ui_tui::profile::check(&document, &ess_ui_tui::TUI)
        .expect_err("a host slot is not drawn yet");
    assert_eq!(refusal.path.to_string(), "pages/p/sections/map");
    assert!(
        refusal.message.contains("RouteMap")
            && refusal
                .message
                .contains("host slots are validated by `ess ui check` and not generated yet"),
        "{refusal}"
    );
}
