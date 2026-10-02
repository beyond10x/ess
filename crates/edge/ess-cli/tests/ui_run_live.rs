//! `ess ui run --tui --model <spec> --base-url <url>`: the terminal bound to the HTTP surface a
//! specification serves (beyond10x/ess#311, S3). Every case here is refused before the terminal
//! is touched, so it runs without one.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The workspace root, found by walking up rather than by counting `..`.
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

/// One run of the built binary, from the workspace root so example paths resolve, with no
/// credential in its environment.
fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .current_dir(workspace_root())
        .env_remove("ESS_UI_AUTHORIZATION")
        .output()
        .expect("the ess binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("output is UTF-8")
}

fn utf8(path: &Path) -> &str {
    path.to_str().expect("the scratch path is UTF-8")
}

/// A desk over `examples/gatepass`, whose one served component is `pass-service`.
const DESK: &str = "format: ess-ui/1\napp: desk\nmodel: gatepass\nplacement_profile: fat\n\
    shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
    navigation: {home: desk, sections: [{name: all, pages: [desk]}]}\n\
    pages: {desk: {kind: detail_page, title: Desk, sections: [{name: expected, \
    component: collection, reads: visit.ExpectedVisits, \
    row_actions: [{name: depart, does: visit.SignOutVisitor, bind: {visit_id: row.visit_id}}]}]}}\n";

#[test]
fn base_url_without_model_is_refused() {
    let output = ess(&[
        "ui",
        "run",
        "--tui",
        "--path",
        "examples/partner-portal/ui.yaml",
        "--base-url",
        "http://127.0.0.1:9",
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", text(&output.stderr));
    assert!(
        text(&output.stderr).contains("--model"),
        "{}",
        text(&output.stderr)
    );
}

#[test]
fn an_unnamed_base_url_with_two_served_components_is_refused() {
    // The shop fixture model of ess-ui-check, its two domains served by two components.
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let fixture = workspace_root().join("crates/ui/ess-ui-check/tests/fixtures/model");
    let model = scratch.path().join("model");
    std::fs::create_dir_all(model.join("domains")).expect("writable");
    for file in ["system.yaml", "domains/stock.yaml", "domains/audit.yaml"] {
        std::fs::copy(fixture.join(file), model.join(file)).expect("the fixture copies");
    }
    std::fs::write(
        model.join("components.yaml"),
        "components:\n  - component: stock-service\n    summary: Holds the stock.\n    \
         owns:\n      domains: [shop.stock]\n    accepts:\n      commands: [shop.stock.AddItem]\n    \
         publishes:\n      events: [shop.stock.ItemAdded]\n    reached_by: network\n  \
         - component: audit-service\n    summary: Holds the audit record.\n    \
         owns:\n      domains: [shop.audit]\n    accepts:\n      commands: [shop.audit.RecordEntry]\n    \
         publishes:\n      events: [shop.audit.EntryRecorded]\n    reached_by: network\n",
    )
    .expect("writable");
    let document = scratch.path().join("shop.yaml");
    std::fs::write(
        &document,
        "format: ess-ui/1\napp: shop\nmodel: shop\nplacement_profile: fat\n\
         shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
         navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
         pages: {p: {kind: detail_page, title: P, sections: [\
         {name: items, component: collection, reads: stock.Items}, \
         {name: entries, component: collection, reads: audit.Entries}]}}\n",
    )
    .expect("writable");

    let output = ess(&[
        "ui",
        "run",
        "--tui",
        "--path",
        utf8(&document),
        "--model",
        utf8(&model),
        "--base-url",
        "http://127.0.0.1:9",
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    for named in ["audit-service", "stock-service", "<component>=<url>"] {
        assert!(stderr.contains(named), "`{named}` is not named: {stderr}");
    }

    // Naming only one leaves the other without a base URL: refused, naming it.
    let one = ess(&[
        "ui",
        "run",
        "--tui",
        "--path",
        utf8(&document),
        "--model",
        utf8(&model),
        "--base-url",
        "stock-service=http://127.0.0.1:9",
    ]);
    let stderr = text(&one.stderr);
    assert_eq!(one.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("audit-service"), "{stderr}");
}

#[test]
fn https_is_refused_by_name() {
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let document = scratch.path().join("desk.yaml");
    std::fs::write(&document, DESK).expect("writable");
    for base in ["https://127.0.0.1:9", "pass-service=https://127.0.0.1:9"] {
        let output = ess(&[
            "ui",
            "run",
            "--tui",
            "--path",
            utf8(&document),
            "--model",
            "examples/gatepass",
            "--base-url",
            base,
        ]);
        let stderr = text(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{base}: {stderr}");
        assert!(stderr.contains("https://"), "{base}: {stderr}");
    }
}
