//! The terminal draws a shared frame as one bordered block titled by its `title`, a record's groups
//! under their labels, and a collapsed frame, section or group as its heading line, opened by `z`
//! (story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed, beyond10x/ess#723
//! G24 and G25).

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const HEAD: &str = "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
fixtures: {views: {shop.Customers: data.yaml, shop.Orders: data.yaml, shop.History: data.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: customer
  sections: [{name: all, pages: [customer]}]
pages:
  customer:
    kind: detail_page
    title: Customer
";

const SECTIONS: &str = "    sections:
      - name: profile
        component: record
        reads: {view: shop.Customers}
        fields: [name]
        groups:
          - {name: contact, label: Contact, fields: [email]}
          - {name: audit, label: Audit, collapsed: true, fields: [created_by]}
      - {name: orders, title: Open orders, component: collection, reads: {view: shop.Orders}, columns: [total]}
      - {name: history, title: History, collapsed: true, component: collection, reads: {view: shop.History}, columns: [at]}
";

const DATA: &str = "views:
  shop.Customers:
    rows: [{name: Ada, email: ada@example.test, created_by: robot}]
  shop.Orders:
    rows: [{total: 1250}]
  shop.History:
    rows: [{at: yesterday}]
";

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-shared-frame")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(dir.join("data.yaml"), DATA).expect("the fixture is written");
    dir
}

fn app(test: &str, layout: &str) -> App {
    let dir = scratch(test);
    let source = format!("{HEAD}    layout: {layout}\n{SECTIONS}");
    App::from_text(&source, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"))
}

fn line_of(screen: &str, text: &str) -> usize {
    screen
        .lines()
        .position(|line| line.contains(text))
        .unwrap_or_else(|| panic!("`{text}` is not shown:\n{screen}"))
}

const COLUMN: &str = "{columns: [{name: main, frame: shared, title: Customer, sections: [profile, orders]}, {name: side, sections: [history]}]}";

#[test]
fn a_shared_frame_is_one_bordered_block_titled_by_its_title() {
    let screen = app("shared", COLUMN).render_text(100, 40);
    let titled: Vec<&str> = screen
        .lines()
        .filter(|line| line.contains("┌ Customer"))
        .collect();
    assert_eq!(titled.len(), 1, "one block is titled Customer:\n{screen}");
    for own in ["┌ profile", "┌ orders"] {
        assert!(
            !screen.contains(own),
            "a framed section draws no border of its own (`{own}`):\n{screen}"
        );
    }
    // The sections' lines sit under their own titles, inside the one block, in order.
    let top = line_of(&screen, "┌ Customer");
    let profile = line_of(&screen, "profile");
    let orders = line_of(&screen, "Open orders");
    let total = line_of(&screen, "1250");
    let history = line_of(&screen, "┌ ▸ history · History");
    assert!(
        top < profile && profile < orders && orders < total,
        "{screen}"
    );
    let bottom = screen
        .lines()
        .enumerate()
        .skip(top + 1)
        .find(|(_, line)| line.contains('└'))
        .map(|(at, _)| at)
        .expect("the block closes");
    assert!(total < bottom, "the orders are inside the frame:\n{screen}");
    // The unshared column's section keeps its own block, after the frame.
    assert!(bottom < history, "{screen}");
}

#[test]
fn a_shared_area_is_one_bordered_block_at_its_first_sections_place() {
    let screen = app(
        "area",
        "{areas: {grid: [[main, side]], place: {main: {sections: [profile, orders], frame: shared, title: Customer}, side: [history]}}}",
    )
    .render_text(100, 40);
    assert_eq!(
        screen
            .lines()
            .filter(|line| line.contains("┌ Customer"))
            .count(),
        1,
        "{screen}"
    );
    assert!(
        line_of(&screen, "┌ Customer") < line_of(&screen, "┌ ▸ history"),
        "{screen}"
    );
}

#[test]
fn a_collapsed_frame_is_its_heading_line_and_z_opens_it() {
    let mut app = app(
        "collapsed-frame",
        "{columns: [{name: main, frame: shared, title: Customer, collapsed: true, sections: [profile, orders]}, {name: side, sections: [history]}]}",
    );
    app.focus_section("profile");
    let closed = app.render_text(100, 40);
    let heading = closed
        .lines()
        .find(|line| line.contains('┌') && line.contains("Customer"))
        .unwrap_or_else(|| panic!("the frame draws its heading:\n{closed}"));
    assert!(
        heading.contains('▸'),
        "a closed heading is marked ▸: {heading}"
    );
    for hidden in ["Open orders", "1250", "ada@example.test"] {
        assert!(
            !closed.contains(hidden),
            "`{hidden}` is drawn in a closed frame:\n{closed}"
        );
    }
    app.keys("z");
    let open = app.render_text(100, 40);
    assert!(
        open.lines()
            .any(|line| line.contains("Customer") && line.contains('▾')),
        "an open heading is marked ▾:\n{open}"
    );
    for shown in ["Open orders", "1250", "ada@example.test"] {
        assert!(
            open.contains(shown),
            "`{shown}` is not drawn once opened:\n{open}"
        );
    }
    app.keys("z");
    let again = app.render_text(100, 40);
    assert!(!again.contains("1250"), "z closes what it opened:\n{again}");
}

#[test]
fn a_record_draws_its_groups_under_their_labels_in_order() {
    let mut app = app("record", "stack");
    app.focus_section("profile");
    let screen = app.render_text(100, 40);
    let contact = line_of(&screen, "Contact");
    let email = line_of(&screen, "ada@example.test");
    let audit = line_of(&screen, "Audit");
    assert!(
        contact < email && email < audit,
        "the groups keep their order:\n{screen}"
    );
    // The collapsed group is its heading line.
    assert!(screen.lines().nth(audit).unwrap().contains('▸'), "{screen}");
    assert!(
        !screen.contains("robot"),
        "a collapsed group draws no fields:\n{screen}"
    );
    app.keys("z");
    let open = app.render_text(100, 40);
    assert!(open.contains("robot"), "z opens the group:\n{open}");
    assert!(
        line_of(&open, "Audit") < line_of(&open, "robot"),
        "the field is under its group's label:\n{open}"
    );
}

/// A record with a collapsed group and an action whose first free letter is `z`, and an overlay
/// form with a collapsed group.
const ADVERSARY_FOLDS: &str = "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
fixtures: {views: {shop.Customers: data.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}, overlay: {kind: overlay_outlet}}}
navigation:
  home: customer
  sections: [{name: all, pages: [customer]}]
pages:
  customer:
    kind: detail_page
    title: Customer
    sections:
      - name: profile
        component: record
        reads: {view: shop.Customers}
        fields: [name]
        groups:
          - {name: audit, label: Audit, collapsed: true, fields: [created_by]}
        actions:
          - {name: zap, label: Zap, opens: note}
    overlays:
      note:
        kind: dialog
        title: Note
        component: form
        does: shop.Note
        fields: [subject]
        groups:
          - {name: advanced, label: Advanced, collapsed: true, fields: [internal_ref]}
";

fn adversary_app(test: &str) -> App {
    let dir = scratch(test);
    App::from_text(ADVERSARY_FOLDS, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn adversary_the_key_an_action_hint_shows_runs_that_action_beside_a_collapsed_group() {
    let mut app = adversary_app("adversary-zap");
    app.focus_section("profile");
    let screen = app.render_text(100, 40);
    // The hint reads `<key> Zap`: press the key it shows.
    let hint = screen
        .lines()
        .find(|line| line.contains(" Zap"))
        .unwrap_or_else(|| panic!("the record hints its action:\n{screen}"));
    let at = hint.find(" Zap").expect("found above");
    let key = hint[..at].chars().last().expect("a key leads the label");
    app.keys(&key.to_string());
    let after = app.render_text(100, 40);
    assert!(
        after.contains("subject"),
        "the hint says `{key}` runs Zap, which opens the Note overlay; it did not:\n{after}"
    );
}

#[test]
fn adversary_a_collapsed_group_of_an_overlay_form_opens_on_z() {
    let mut app = adversary_app("adversary-overlay");
    app.open_overlay("note");
    let closed = app.render_text(100, 40);
    assert!(
        closed
            .lines()
            .any(|line| line.contains("Advanced") && line.contains('▸')),
        "the closed group draws its heading:\n{closed}"
    );
    assert!(!closed.contains("internal_ref"), "{closed}");
    app.keys("z");
    let open = app.render_text(100, 40);
    assert!(
        open.contains("internal_ref"),
        "z opens a collapsed form group in an overlay as it does on a page:\n{open}"
    );
}

#[test]
fn a_collapsed_section_is_its_heading_line_and_z_opens_it() {
    let mut app = app("section", "stack");
    app.focus_section("history");
    let closed = app.render_text(100, 40);
    let heading = closed
        .lines()
        .find(|line| line.contains("History"))
        .unwrap_or_else(|| panic!("the section draws its heading:\n{closed}"));
    assert!(heading.contains('▸'), "{heading}");
    assert!(!closed.contains("yesterday"), "{closed}");
    app.keys("z");
    let open = app.render_text(100, 40);
    assert!(open.contains("yesterday"), "z opens the section:\n{open}");
    assert!(
        open.lines()
            .any(|line| line.contains("History") && line.contains('▾')),
        "{open}"
    );
}
