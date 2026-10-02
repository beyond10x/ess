//! Adversary pass 1 on `ess generate ui --target tui` (story:ui-tui-app-generator): documents
//! whose `app` the generator turns into a crate that Cargo or rustc refuses, and an `--out` that
//! already holds another crate.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

fn utf8(path: &Path) -> &str {
    path.to_str().expect("the scratch path is UTF-8")
}

/// The unit's own desk over `examples/gatepass`, with `app:` replaced by `app`.
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
        .join("adversary-tui-app-pass1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale scratch directory is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

/// Generates the terminal app for `document` into `out`.
fn generate(dir: &Path, document: &str, out: &Path) -> Output {
    let path = dir.join("ui.yaml");
    std::fs::write(&path, document).expect("writable");
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["generate", "ui", "--target", "tui", "--path"])
        .arg(&path)
        .args(["--model", "examples/gatepass", "--out"])
        .arg(out)
        .current_dir(workspace_root())
        .output()
        .expect("the ess binary runs")
}

fn cargo() -> Command {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .env_remove("CARGO_TARGET_DIR")
        .env("CARGO_INCREMENTAL", "0");
    command
}

/// The package name the generated manifest declares.
fn package_name(out: &Path) -> String {
    let manifest = std::fs::read_to_string(out.join("Cargo.toml")).expect("a manifest");
    manifest
        .lines()
        .find_map(|line| line.strip_prefix("name = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("the manifest names its package")
        .to_owned()
}

/// Builds the generated crate offline, the way `generate_ui_tui.rs` does: `ess-ui-tui` patched to
/// this workspace and the workspace lock extended with the generated package.
fn build_offline(out: &Path) -> Output {
    let name = package_name(out);
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
        .args(["build", "--offline", "--quiet", "--config", &patch])
        .arg("--manifest-path")
        .arg(out.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(Path::new(env!("CARGO_TARGET_TMPDIR")).join("generate-ui-tui-target"))
        .output()
        .expect("cargo runs")
}

/// `app: build` (and `deps`, `examples`, `incremental`, or any casing of them) is a name the
/// document loader accepts; the generator copies it into `[[bin]] name`, which Cargo refuses to
/// parse: "the binary target name `build` is forbidden, it conflicts with cargo's build directory
/// names".
#[test]
fn an_app_named_build_generates_a_manifest_cargo_parses() {
    for app in ["build", "Deps", "examples", "incremental"] {
        let dir = scratch(&format!("reserved-{app}"));
        let out = dir.join("app");
        let generated = generate(&dir, &desk(app), &out);
        assert_eq!(
            generated.status.code(),
            Some(0),
            "{}",
            text(&generated.stderr)
        );
        let parsed = cargo()
            .args([
                "metadata",
                "--offline",
                "--no-deps",
                "--format-version",
                "1",
            ])
            .arg("--manifest-path")
            .arg(out.join("Cargo.toml"))
            .output()
            .expect("cargo runs");
        assert!(
            parsed.status.success(),
            "app `{app}`: cargo cannot read the generated manifest: {}",
            text(&parsed.stderr)
        );
    }
}

/// `app:` is interpolated unescaped into `//!` and `///` comments of `src/main.rs`. The loader
/// accepts any string there, so a quoted line break ends the comment and the rest of the name is
/// compiled as code.
#[test]
fn an_app_name_with_a_line_break_generates_a_crate_that_builds() {
    let dir = scratch("line-break");
    let out = dir.join("app");
    let generated = generate(&dir, &desk("\"front\\ndesk\""), &out);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}",
        text(&generated.stderr)
    );
    let built = build_offline(&out);
    assert!(
        built.status.success(),
        "the generated crate does not build: {}",
        text(&built.stderr)
    );
}

/// `--out` naming a directory that already holds a crate: the generator writes `Cargo.toml` and
/// `src/main.rs` over that crate's own, with no refusal, so `--out .` in a Rust project replaces
/// its manifest and its entry point.
#[test]
fn generating_into_an_existing_crate_does_not_replace_its_manifest() {
    let dir = scratch("existing-crate");
    let out = dir.join("mine");
    std::fs::create_dir_all(out.join("src")).expect("writable");
    let manifest = "[package]\nname = \"mine\"\nversion = \"1.2.3\"\nedition = \"2021\"\n";
    let main = "fn main() { println!(\"mine\"); }\n";
    std::fs::write(out.join("Cargo.toml"), manifest).expect("writable");
    std::fs::write(out.join("src/main.rs"), main).expect("writable");
    let generated = generate(&dir, &desk("desk"), &out);
    let after_manifest = std::fs::read_to_string(out.join("Cargo.toml")).expect("readable");
    let after_main = std::fs::read_to_string(out.join("src/main.rs")).expect("readable");
    assert!(
        after_manifest == manifest && after_main == main,
        "exit {:?}: {}\nthe existing crate's Cargo.toml became:\n{after_manifest}",
        generated.status.code(),
        utf8(&out)
    );
}
