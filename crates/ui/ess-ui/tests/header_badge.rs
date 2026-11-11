//! A page header carries a badge: `badge: {text, tone}`, the tone from the closed tone set
//! (story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page).

use ess_ui::{load_str, Tone};

fn document(badge: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: files
  sections: [{{name: all, pages: [files]}}]
pages:
  files:
    kind: list_page
    title: Files
    header:
      title: Files
      badge: {badge}
    sections:
      - {{name: list, component: collection, reads: {{view: desk.Files}}, columns: [name]}}
"
    )
}

#[test]
fn a_header_badge_loads_its_text_and_tone() {
    let loaded = load_str(&document("{text: Beta, tone: info}"))
        .unwrap_or_else(|error| panic!("the document loads: {error}"));
    let badge = loaded.pages["files"]
        .header
        .as_ref()
        .and_then(|header| header.badge.as_ref())
        .expect("the header has a badge");
    assert_eq!(badge.text.0, "Beta");
    assert_eq!(badge.tone, Some(Tone::Info));
}

#[test]
fn an_unknown_badge_tone_is_refused_by_path() {
    let Err(error) = load_str(&document("{text: Beta, tone: loud}")) else {
        panic!("an unknown tone loads")
    };
    let at = error.path().to_string();
    assert!(
        at.starts_with("pages/files/header"),
        "the refusal is reported at `{at}`: {error}"
    );
    assert!(
        error.to_string().contains("Tone"),
        "the refusal does not name the tone: {error}"
    );
}
