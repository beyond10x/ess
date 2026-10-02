//! The header's `switch_to` links carry the current page's params, so a sibling view of the same
//! record opens on that record (beyond10x/ess#355). `hrefFor` fills only the params the target's
//! route declares, by name.

use std::path::{Path, PathBuf};

const DOC: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: things.detail
  sections: [{name: all, pages: [things.detail]}]
pages:
  things.detail:
    kind: detail_page
    title: Thing
    params: {id: string}
    switch_to: [things.history]
    header: {title: Thing}
    sections:
      - name: summary
        reads: {view: things.ById, params: {id: params.id}}
        fields: [title]
  things.history:
    kind: detail_page
    title: History
    params: {id: string}
    switch_to: [things.detail]
    header: {title: History}
    sections:
      - name: summary
        reads: {view: things.ById, params: {id: params.id}}
        fields: [title]
";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-header-switch")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

#[test]
fn a_switch_link_carries_the_current_page_params() {
    let out = scratch("carries");
    let doc = ess_ui::load_str(DOC).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&doc, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    let page = std::fs::read_to_string(out.join("src/pages/ThingsDetail.tsx"))
        .expect("the page is generated");
    assert!(
        page.contains(r#"hrefFor("things.history", __params)"#),
        "the switch link is built from the page params:\n{page}"
    );
    assert!(
        !page.contains(r#"hrefFor("things.history", {})"#),
        "the switch link drops no params:\n{page}"
    );
}
