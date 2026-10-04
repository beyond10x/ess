//! Adversary pass 1: state placement, inspected on disk after a session.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adv1")
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

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(walk(&path));
        } else {
            files.push(path);
        }
    }
    files
}

fn holding(dir: &Path, needle: &str) -> Vec<PathBuf> {
    walk(dir)
        .into_iter()
        .filter(|file| {
            std::fs::read_to_string(file)
                .unwrap_or_default()
                .contains(needle)
        })
        .collect()
}

/// Every sensitive draft the example has, typed through the keyboard; no file under the state
/// dir may hold any of them afterwards.
#[test]
fn adv1_no_sensitive_value_reaches_a_state_file() {
    let dir = state_dir("sensitive");
    let mut app = open(&dir);
    // A non-sensitive session_storage draft, so the dir is known to be written at all.
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    app.focus_section("reply");
    app.keys("<enter>visible-draft<esc>");
    assert_eq!(holding(&dir, "visible-draft").len(), 1);

    // users.list edit drawer: password `as: secret`, draft sensitive in memory.
    app.open_page("users.list", &[]);
    app.focus_section("list");
    app.keys("e");
    app.keys("jj<enter>pw-in-drawer<esc>");
    // settings.organization: api key `as: secret` in a sensitive draft.
    app.open_page("settings.organization", &[]);
    app.focus_section("settings");
    app.keys("jjjjjj<enter>api-key-typed<esc>");
    // auth.sign_in: password.
    app.open_page("auth.sign_in", &[]);
    app.focus_section("form");
    app.keys("<enter>me@example.com<tab>hunter2-typed<esc>");
    drop(app);
    for secret in [
        "pw-in-drawer",
        "api-key-typed",
        "hunter2-typed",
        "me@example.com",
    ] {
        assert!(
            holding(&dir, secret).is_empty(),
            "{secret} was written to {:?}",
            holding(&dir, secret)
        );
    }
}

/// `session_storage` "survives reload; lost when the tab closes": a new run starts without the
/// last run's session drafts, and its form is empty.
#[test]
fn adv1_session_storage_is_empty_at_the_start_of_a_run() {
    let dir = state_dir("session-start");
    let mut app = open(&dir);
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    app.focus_section("reply");
    app.keys("<enter>left-over<esc>");
    assert_eq!(holding(&dir, "left-over").len(), 1);
    drop(app);
    let mut app = open(&dir);
    assert_eq!(holding(&dir, "left-over").len(), 0);
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    assert!(!app.render_text(120, 60).contains("left-over"));
}

/// The schema keys `session_storage` and `local_storage` by `[origin, actor.user_id,
/// actor.account_id]`. The fixture actor is us-01 in org-01 (session.Me); a file holding its
/// draft must be keyed by it, so another actor on this machine does not read it.
#[test]
fn adv1_storage_files_are_namespaced_by_actor() {
    let dir = state_dir("actor-key");
    let mut app = open(&dir);
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    app.focus_section("reply");
    app.keys("<enter>actor-draft<esc>");
    let files = holding(&dir, "actor-draft");
    assert_eq!(files.len(), 1, "{files:?}");
    let path = files[0].strip_prefix(&dir).unwrap().display().to_string();
    assert!(
        path.contains("us-01") && path.contains("org-01"),
        "the draft file `{path}` is keyed by app only, not by actor.user_id and actor.account_id"
    );
}
