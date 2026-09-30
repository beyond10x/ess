//! Adversary pass 2: sign-out and the storage key correction 1 introduced.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui_tui::placement::{Placement, StateStore};
use ess_ui_tui::{App, FixtureAdapter, Options};
use serde_yaml::Value;

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adv2")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

fn open(dir: &Path) -> App {
    App::from_path(
        &example_dir().join("ui.yaml"),
        Options::new(dir.to_path_buf()),
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

/// Signs out through the account menu (palette) and signs in again on the sign-in page.
fn sign_out_and_in(app: &mut App) {
    app.keys(":sign out<enter>");
    assert_eq!(
        app.page(),
        "auth.sign_in",
        "sign out reached the sign-in page"
    );
    app.focus_section("form");
    app.keys("<enter>other@example.com<tab>pw<esc><c-s>");
    assert_ne!(app.page(), "auth.sign_in", "sign in left the sign-in page");
}

/// The schema's store matrix: `session_storage` has `survives_signout: false`, and a state's
/// `clear_on` defaults to `[signout, account_switch]`. The reply draft of tk-01 is placed in
/// `session_storage`; after sign-out and sign-in it is gone.
#[test]
fn adv2_sign_out_clears_the_session_storage_draft() {
    let dir = state_dir("signout-session");
    let mut app = open(&dir);
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    app.focus_section("reply");
    app.keys("<enter>previous-actor-draft<esc>");
    assert!(app.render_text(140, 60).contains("previous-actor-draft"));
    sign_out_and_in(&mut app);
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    let text = app.render_text(140, 60);
    assert!(
        !text.contains("previous-actor-draft"),
        "the reply draft typed before sign-out is shown after signing in again:\n{}",
        text.lines()
            .filter(|line| line.contains("previous-actor-draft"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// `memory` has `survives_signout: false`. The users edit drawer keeps its (sensitive) draft in
/// memory; a name typed there before sign-out is not shown after signing in again.
#[test]
fn adv2_sign_out_clears_memory_drafts() {
    let dir = state_dir("signout-memory");
    let mut app = open(&dir);
    app.open_page("users.list", &[]);
    app.focus_section("list");
    app.keys("e");
    app.keys("<enter>previous-actor-name<esc>");
    assert!(app.render_text(140, 60).contains("previous-actor-name"));
    app.keys("<esc>");
    sign_out_and_in(&mut app);
    app.open_page("users.list", &[]);
    app.focus_section("list");
    app.keys("e");
    let text = app.render_text(140, 60);
    assert!(
        !text.contains("previous-actor-name"),
        "the memory draft typed before sign-out is shown after signing in again:\n{text}"
    );
}

fn session_placement() -> BTreeMap<String, Placement> {
    BTreeMap::from([(
        "pages/p/draft".to_owned(),
        Placement {
            store: ess_ui::Store::SessionStorage,
            sensitive: false,
            default: None,
        },
    )])
}

/// `StateStore::new` documents "no two keys share a directory". An empty user id with account
/// `acct`, and user `acct` with an empty account id, are two different actors.
#[test]
fn adv2_empty_ids_do_not_collide_in_the_storage_key() {
    let dir = state_dir("empty-ids");
    let document = ess_ui::load_str(
        &std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads"),
    )
    .expect("the example loads");
    let (mut adapter, _) =
        FixtureAdapter::load(&document, &example_dir(), None).expect("the fixtures load");
    // Both stores exist before either writes: `new` clears its own session file.
    let mut first = StateStore::new(session_placement(), &dir, "portal", Some(""), Some("acct"));
    let second = StateStore::new(session_placement(), &dir, "portal", Some("acct"), Some(""));
    first.set(
        "pages/p/draft",
        Value::String("first-actor".into()),
        &mut adapter,
    );
    let seen = second.get("pages/p/draft", &adapter);
    assert_eq!(
        seen,
        Value::Null,
        "actor (user \"\", account acct) and actor (user acct, account \"\") share a storage \
         directory"
    );
}
