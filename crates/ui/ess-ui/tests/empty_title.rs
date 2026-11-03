//! An empty state may carry a `title` beside its `message`
//! (story:a-refused-node-is-reported-at-its-own-path, beyond10x/ess#723).

use ess_ui::load_str;

const DOCUMENT: &str = "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: files
  sections: [{name: all, pages: [files]}]
pages:
  files:
    kind: list_page
    title: Files
    sections:
      - name: list
        component: collection
        reads: {view: desk.Files}
        columns: [name]
        states: {empty: {title: No files, message: Upload a file to see it here.}}
";

#[test]
fn an_empty_state_takes_a_title() {
    let document = load_str(DOCUMENT).unwrap_or_else(|error| panic!("the document loads: {error}"));
    let empty = document.pages["files"]
        .sections
        .iter()
        .find(|section| section.name == "list")
        .expect("the page has the section `list`")
        .states
        .as_ref()
        .and_then(|states| states.empty.as_ref())
        .expect("the section has an empty state");
    assert_eq!(empty.title.as_deref(), Some("No files"));
    assert_eq!(empty.message, "Upload a file to see it here.");
}
