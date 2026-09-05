//! Authored site inputs retain source identity without implicitly publishing sibling files.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ess-authored-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("model")).unwrap();
        fs::write(
            root.join("model/system.yaml"),
            "format: ess/1\nsystem: publication\nversion: v1\ndomains: []\n",
        )
        .unwrap();
        fs::write(root.join("README.md"), "# Read first\n\n[Guide](docs/guide.md?mode=read#details) [Index](docs/) [Data](data%20set.json) [Web](https://example.org/a.md?q=x#y)\n").unwrap();
        fs::write(root.join("docs/guide.md"), "# Guide\n\n## Details\n\n[Home](../README.md) [Local](#details)\n\n```mermaid\nflowchart LR\nA --> B\n```\n").unwrap();
        fs::write(
            root.join("docs/README.md"),
            "# Chapters\n\n[Guide](guide.md)\n",
        )
        .unwrap();
        fs::write(root.join("data set.json"), "{\"value\": 1}\n").unwrap();
        Self(root)
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command.current_dir(&self.0).args([
            "generate",
            "--kind",
            "site",
            "--path",
            "model",
            "--front-page",
            "README.md",
            "--include",
            "chapters/deep/guide=docs/guide.md",
            "--include",
            "chapters/index=docs/README.md",
            "--asset",
            "downloads/data.json=data set.json",
            "--strict-links",
            "--out",
            "out",
        ]);
        command
    }

    fn success(&self) {
        let output = self.command().output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn refused(&self, output: &Output, expected: &str) {
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(expected), "{error}");
        assert!(
            !self.0.join("out").exists(),
            "refusal must precede output creation"
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads() {
    let fixture = Fixture::new();
    fixture.success();
    let index = fs::read_to_string(fixture.0.join("out/index.html")).unwrap();
    assert!(index.contains("href=\"chapters/deep/guide.html?mode=read#details\""));
    assert!(index.contains("href=\"chapters/index.html\""));
    assert!(index.contains("href=\"downloads/data.json\""));
    assert!(index.contains("href=\"https://example.org/a.md?q=x#y\""));
    let guide = fs::read_to_string(fixture.0.join("out/chapters/deep/guide.html")).unwrap();
    assert!(guide.contains("href=\"../../index.html\""));
    assert!(guide.contains("href=\"#details\""));
    assert!(guide.contains("<pre class=\"mermaid\">flowchart LR\nA --&gt; B</pre>"));
    assert!(guide.contains("../../assets/mermaid.min.js"));
    assert_eq!(
        fs::read(fixture.0.join("out/downloads/data.json")).unwrap(),
        fs::read(fixture.0.join("data set.json")).unwrap()
    );
    fixture.success();
    assert_eq!(
        index,
        fs::read_to_string(fixture.0.join("out/index.html")).unwrap()
    );
    assert_eq!(
        guide,
        fs::read_to_string(fixture.0.join("out/chapters/deep/guide.html")).unwrap()
    );
}

#[test]
fn undeclared_existing_and_escaping_targets_are_refused_with_source_lines() {
    for url in [
        "secret.md",
        "../../../private.md",
        "/private.md",
        "%2e%2e/private.md",
    ] {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("secret.md"), "not declared").unwrap();
        fs::write(
            fixture.0.join("README.md"),
            format!("# Page\n\n[Missing]({url})\n"),
        )
        .unwrap();
        let output = fixture.command().output().unwrap();
        fixture.refused(&output, "README.md:3: link");
    }
}

#[test]
fn strict_links_check_local_and_cross_page_fragments() {
    for url in ["#missing", "docs/guide.md#missing"] {
        let fixture = Fixture::new();
        fs::write(
            fixture.0.join("README.md"),
            format!("# Page\n\n[Missing]({url})\n"),
        )
        .unwrap();
        let output = fixture.command().output().unwrap();
        fixture.refused(&output, "fragment `missing` is absent");
    }
}

#[test]
fn assets_cannot_collide_with_generated_output_or_escape_it() {
    for name in [
        "index.html",
        "assets/style.css",
        "../outside.json",
        "chapters/deep/guide.html",
    ] {
        let fixture = Fixture::new();
        let output = fixture
            .command()
            .args(["--asset", &format!("{name}=data set.json")])
            .output()
            .unwrap();
        fixture.refused(&output, "");
    }
}

#[test]
fn asset_symlink_output_is_refused_before_any_page_changes() {
    #[cfg(unix)]
    {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.0.join("out/downloads")).unwrap();
        fs::write(fixture.0.join("out/index.html"), "sentinel").unwrap();
        std::os::unix::fs::symlink(
            fixture.0.join("data set.json"),
            fixture.0.join("out/downloads/data.json"),
        )
        .unwrap();
        let output = fixture.command().output().unwrap();
        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(fixture.0.join("out/index.html")).unwrap(),
            "sentinel"
        );
    }
}

#[test]
fn publication_without_strict_mode_preserves_unpublished_links() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("model/README.md"),
        "[Not published](missing.md)\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(&fixture.0)
        .args([
            "generate", "--kind", "site", "--path", "model", "--out", "out",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fs::read_to_string(fixture.0.join("out/index.html"))
        .unwrap()
        .contains("href=\"missing.md\""));
    assert!(!fixture.0.join("out/missing.md").exists());
}

#[test]
fn an_explicit_missing_front_page_is_an_error() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.0.join("README.md")).unwrap();
    let output = fixture.command().output().unwrap();
    fixture.refused(&output, "reading selected UTF-8 source README.md");
}

#[test]
fn binary_downloads_are_not_silently_decoded() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("data set.json"), [0xff, 0x00]).unwrap();
    let output = fixture.command().output().unwrap();
    fixture.refused(&output, "reading selected UTF-8 source");
}

#[test]
fn a_page_identity_can_itself_end_in_html() {
    let fixture = Fixture::new();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(&fixture.0)
        .args([
            "generate",
            "--kind",
            "site",
            "--path",
            "model",
            "--include",
            "guide.html=docs/guide.md",
            "--out",
            "out",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(fixture.0.join("out/guide.html.html").is_file());
    assert!(!fixture.0.join("out/guide.html").exists());
}
