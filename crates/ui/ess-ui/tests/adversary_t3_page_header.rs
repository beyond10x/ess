//! Adversary, turn 3 (story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page):
//! the unit's commit says "A refused header or badge is reported at its own path", and its case
//! only asserts the path starts with `pages/files/header`. Without the `HeaderBadge` arm of
//! `locate.rs` a refused badge surfaces at the header and that case stays green; these pin each
//! refusal at its own node.

use ess_ui::load_str;

fn document(header: &str) -> String {
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
    header: {header}
    sections:
      - {{name: list, component: collection, reads: {{view: desk.Files}}, columns: [name]}}
"
    )
}

fn refused_at(header: &str) -> String {
    let Err(error) = load_str(&document(header)) else {
        panic!("`{header}` loads")
    };
    error.path().to_string()
}

#[test]
fn a_refused_badge_is_reported_at_the_badge_and_a_refused_header_at_the_header() {
    assert_eq!(
        refused_at("{title: Files, badge: {text: Beta, tone: loud}}"),
        "pages/files/header/badge"
    );
    assert_eq!(
        refused_at("{title: Files, badge: {text: Beta, colour: red}}"),
        "pages/files/header/badge"
    );
    assert_eq!(
        refused_at("{title: Files, badge: {tone: info}}"),
        "pages/files/header/badge"
    );
    assert_eq!(
        refused_at("{title: Files, titel: Files}"),
        "pages/files/header"
    );
}
