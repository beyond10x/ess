//! Adversary pass 2 on `ess generate ui --target tui` (story:ui-tui-app-generator), after the
//! correction of pass 1: the headless run of the built crate against a surface that refuses or is
//! down, the generated code under `cargo clippy -- -D warnings`, and the `--out` guard against a
//! path it does not check.

use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("a member of this workspace lies under its root")
        .to_path_buf()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The unit's desk over `examples/gatepass`, with `app:` replaced by `app` (a YAML scalar).
fn desk(app: &str) -> String {
    format!(
        "format: ess-ui/1\napp: {app}\nmodel: gatepass\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: desk, sections: [{{name: all, pages: [desk]}}]}}\n\
         pages: {{desk: {{kind: detail_page, title: Desk, sections: [{{name: expected, \
         component: collection, reads: visit.ExpectedVisits, columns: [visitor, building]}}]}}}}\n"
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-tui-app-pass2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale scratch directory is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

/// Generates the terminal app for `document` (written to `path`) into `out`.
fn generate_from(path: &Path, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["generate", "ui", "--target", "tui", "--path"])
        .arg(path)
        .args(["--model", "examples/gatepass", "--out"])
        .arg(out)
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs")
}

fn generate(dir: &Path, document: &str, out: &Path) -> Output {
    let path = dir.join("ui.yaml");
    std::fs::write(&path, document).expect("writable");
    generate_from(&path, out)
}

fn cargo() -> Command {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO_INCREMENTAL", "0");
    command
}

/// Runs `subcommand` of Cargo over the generated crate in `out`, offline: `ess-ui-tui` patched to
/// this workspace and the workspace lock extended with the generated package, as
/// `generate_ui_tui.rs` builds it.
fn cargo_offline(out: &Path, subcommand: &[&str]) -> Output {
    let manifest = std::fs::read_to_string(out.join("Cargo.toml")).expect("a manifest");
    let name = manifest
        .lines()
        .find_map(|line| line.strip_prefix("name = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("the manifest names its package")
        .to_owned();
    let mut lock =
        std::fs::read_to_string(workspace_root().join("Cargo.lock")).expect("the lock reads");
    let entry = format!(
        "\n[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\n\
         dependencies = [\n \"clap\",\n \"ess-ui-tui\",\n]\n"
    );
    lock.push_str(&entry);
    std::fs::write(out.join("Cargo.lock"), lock).expect("the lock is written");
    let patch = format!(
        "patch.\"https://github.com/beyond10x/ess\".ess-ui-tui.path = \"{}\"",
        workspace_root().join("crates/ui/ess-ui-tui").display()
    );
    cargo()
        .arg(subcommand[0])
        .args(["--offline", "--quiet", "--config", &patch])
        .arg("--manifest-path")
        .arg(out.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(Path::new(env!("CARGO_TARGET_TMPDIR")).join("generate-ui-tui-target"))
        .args(&subcommand[1..])
        .output()
        .expect("cargo runs")
}

/// The desk named `adv2desk`, generated and built once; returns the built binary.
fn built_desk() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        let dir = scratch("built");
        let out = dir.join("app");
        let generated = generate(&dir, &desk("adv2desk"), &out);
        assert_eq!(
            generated.status.code(),
            Some(0),
            "{}",
            text(&generated.stderr)
        );
        let built = cargo_offline(&out, &["build"]);
        assert!(built.status.success(), "{}", text(&built.stderr));
        Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("generate-ui-tui-target/debug")
            .join(format!("adv2desk{}", std::env::consts::EXE_SUFFIX))
    })
}

/// Runs the built desk headless against `base`.
fn screen_once(base: &str, state: &Path) -> Output {
    Command::new(built_desk())
        .args(["--base-url", base, "--screen-once", "100x12"])
        .env_remove("ESS_UI_AUTHORIZATION")
        .env("XDG_STATE_HOME", state)
        .stdin(Stdio::null())
        .output()
        .expect("the generated binary runs")
}

/// A surface on `127.0.0.1` that answers every request `403 Forbidden`.
fn forbidding_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port is free");
    let port = listener.local_addr().expect("bound").port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                match stream.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(read) => request.extend_from_slice(&buffer[..read]),
                }
            }
            let body = r#"{"error":"forbidden"}"#;
            let _ = write!(
                stream,
                "HTTP/1.1 403 Forbidden\r\ncontent-type: application/json\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    format!("http://127.0.0.1:{port}")
}

/// `--screen-once` is the generated app's only headless mode, so it is what a script or a smoke
/// check runs. When the home page's one read is refused (`403` on every read) or the surface is
/// down, it still prints a frame and exits 0: a check of exit status cannot tell a desk that read
/// its rows from one that read nothing.
#[test]
fn screen_once_exits_nonzero_when_the_home_read_fails() {
    let state = scratch("state");
    let forbidding = forbidding_server();
    let down = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a port is free");
        let port = listener.local_addr().expect("bound").port();
        drop(listener);
        format!("http://127.0.0.1:{port}")
    };
    let mut exited_zero = Vec::new();
    for (surface, base) in [("refuses every read", forbidding), ("is down", down)] {
        let output = screen_once(&base, &state);
        if output.status.code() == Some(0) {
            exited_zero.push(format!(
                "the surface {surface}, and --screen-once exited 0 printing:\n{}{}",
                text(&output.stdout),
                text(&output.stderr)
            ));
        }
    }
    assert!(exited_zero.is_empty(), "{}", exited_zero.join("\n"));
}

/// `app` is copied into the `//!` and `///` comments of `src/main.rs` inside a code span; a
/// backtick in it closes the span, and the rest is Markdown. `[^1]` there makes the generated crate
/// fail `cargo clippy -- -D warnings` (`clippy::doc_suspicious_footnotes`).
#[test]
fn the_generated_crate_is_clippy_clean_for_any_app() {
    let dir = scratch("clippy-footnote");
    let out = dir.join("app");
    let generated = generate(&dir, &desk("\"desk` [^1] `two\""), &out);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}",
        text(&generated.stderr)
    );
    let linted = cargo_offline(&out, &["clippy", "--", "-D", "warnings"]);
    assert!(
        linted.status.success(),
        "the generated crate fails clippy -D warnings: {}",
        text(&linted.stderr)
    );
}

/// The guard checks each of the four files, not the directories they go in. With `src` a plain
/// file under `--out`, every check passes, `Cargo.toml` is written, then `src/` cannot be created:
/// the run is refused and a lone generated `Cargo.toml` is left behind.
#[test]
fn a_refused_generation_writes_nothing_when_src_is_a_file() {
    let dir = scratch("src-is-a-file");
    let out = dir.join("app");
    std::fs::create_dir_all(&out).expect("writable");
    std::fs::write(out.join("src"), "notes\n").expect("writable");
    let generated = generate(&dir, &desk("desk"), &out);
    assert_ne!(generated.status.code(), Some(0), "src is a file");
    assert!(
        !out.join("Cargo.toml").exists(),
        "refused ({}), yet Cargo.toml was written",
        text(&generated.stderr).trim()
    );
}

/// The guard reads and writes `--out/<file>` through a symbolic link. A `Cargo.toml` that is a link
/// to another generated crate's manifest passes the check (its first line is the mark), and that
/// other crate's manifest is rewritten with this crate's name.
#[cfg(unix)]
#[test]
fn a_symlinked_manifest_is_not_written_through() {
    let dir = scratch("symlink");
    let other = dir.join("other");
    let generated = generate(&dir, &desk("other"), &other);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}",
        text(&generated.stderr)
    );
    let before = std::fs::read_to_string(other.join("Cargo.toml")).expect("readable");
    let out = dir.join("app");
    std::fs::create_dir_all(&out).expect("writable");
    std::os::unix::fs::symlink(other.join("Cargo.toml"), out.join("Cargo.toml"))
        .expect("a link is made");
    let _ = generate(&dir, &desk("desk"), &out);
    let after = std::fs::read_to_string(other.join("Cargo.toml")).expect("readable");
    assert_eq!(
        after, before,
        "a crate outside --out had its manifest rewritten"
    );
}

/// Generating again from the crate's own `src/ui.yaml` (the document it holds) adds a second mark
/// line each time: the generated document is not a fixed point of the generator.
#[test]
fn generating_from_the_generated_document_is_a_fixed_point() {
    let dir = scratch("fixed-point");
    let out = dir.join("app");
    let generated = generate(&dir, &desk("desk"), &out);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}",
        text(&generated.stderr)
    );
    let held = out.join("src/ui.yaml");
    let first = std::fs::read_to_string(&held).expect("readable");
    let copy = dir.join("held.yaml");
    std::fs::copy(&held, &copy).expect("copied");
    let again = generate_from(&copy, &out);
    assert_eq!(again.status.code(), Some(0), "{}", text(&again.stderr));
    let second = std::fs::read_to_string(&held).expect("readable");
    assert_eq!(
        second.lines().take(3).collect::<Vec<_>>(),
        first.lines().take(3).collect::<Vec<_>>(),
        "src/ui.yaml grows a mark line per regeneration"
    );
}
