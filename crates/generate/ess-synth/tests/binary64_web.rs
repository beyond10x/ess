//! The web target carries a finite `Binary64` through its bridge and gives it a number control on
//! the page, and still refuses an uncarried position (an identity) with the detail the Rust target
//! gives. No test compiles generated code.

#[path = "binary64_served/support.rs"]
mod support;

use std::process::Command;

use ess_synth::{synthesize_for, Synthesis, Target, TargetFailureCode};

fn artifact<'a>(synthesis: &'a Synthesis, path: &str) -> &'a str {
    &synthesis
        .artifacts
        .values()
        .find(|artifact| artifact.path == path)
        .unwrap_or_else(|| panic!("no artifact {path}"))
        .contents
}

/// The page's module script, exactly as `index.html` carries it.
fn page_script(page: &str) -> &str {
    let open = "<script type=\"module\">\n";
    let start = page.find(open).expect("the page has a module script") + open.len();
    let end = page[start..]
        .find("\n</script>")
        .expect("the script closes")
        + start;
    &page[start..end]
}

#[test]
fn the_web_target_carries_a_finite_binary64() {
    let synthesis =
        synthesize_for(&support::model(), Target::Web).expect("the web target carries it");
    let page = artifact(&synthesis, "index.html");
    assert!(page.contains("if (name === \"binary64\") {"));
    assert!(page.contains("step: \"any\""));
    assert!(artifact(&synthesis, "TARGET.md")
        .contains("a `Binary64` value crosses with its sign of zero"));
    let wire = artifact(&synthesis, "crates/demo-web/src/wire.rs");
    assert!(wire.contains("fn binary64_at("));
    assert!(wire.contains(".token());"));
}

#[test]
fn a_web_model_without_binary64_keeps_its_bytes() {
    let source = support::MODEL.replace("Binary64", "Decimal");
    let synthesis = synthesize_for(&support::compile(&source), Target::Web).expect("synthesizes");
    assert!(!artifact(&synthesis, "index.html").contains("\"binary64\""));
    assert!(!artifact(&synthesis, "TARGET.md").contains("Binary64"));
}

#[test]
fn a_binary64_identity_is_still_refused_by_the_web_target() {
    let source = support::MODEL.replace(
        "{name: demo.search.DocId, kind: newtype, of: Uuid}",
        "{name: demo.search.DocId, kind: newtype, of: Binary64}",
    );
    assert_ne!(source, support::MODEL);
    let Err(failure) = synthesize_for(&support::compile(&source), Target::Web) else {
        panic!("a Binary64 identity is refused");
    };
    let causes = failure.causes();
    assert_eq!(causes.len(), 1, "{causes:?}");
    assert_eq!(causes[0].code(), TargetFailureCode::MissingRepresentation);
    assert_eq!(causes[0].sources(), ["types.demo.search.DocId.of"]);
    assert!(
        causes[0]
            .detail()
            .starts_with("a Binary64 here is a key, a lookup or a delivery route"),
        "{}",
        causes[0].detail()
    );
}

#[test]
fn the_page_with_a_binary64_control_type_checks_with_tsc() {
    let synthesis =
        synthesize_for(&support::model(), Target::Web).expect("the web target carries it");
    let directory = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("binary64-web-tsc");
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("page.js"),
        format!(
            "// @ts-check\n{}\n",
            page_script(artifact(&synthesis, "index.html"))
        ),
    )
    .unwrap();
    std::fs::write(
        directory.join("bridge.js"),
        artifact(&synthesis, "bridge.js"),
    )
    .unwrap();
    let output = Command::new("tsc")
        .current_dir(&directory)
        .args([
            "--noEmit",
            "--allowJs",
            "--strict",
            "false",
            "--target",
            "es2022",
            "--module",
            "es2022",
            "--moduleResolution",
            "bundler",
            "--lib",
            "es2022,dom",
            "page.js",
        ])
        .output()
        .expect("tsc runs");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
