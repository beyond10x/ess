//! Adversary, unit w091-u1-host, finding 2
//! (story:a-host-slot-reads-and-commands-only-what-it-declares, acceptance 1 and 2).
//!
//! The acceptance reads: "With `--model`, `ess ui check` checks a slot's `reads` and `commands`
//! against the specification **and its `ess-http/1` binding**", and "an operation the binding
//! does not serve is refused by the same diagnostics". `ess ui check` is `check_source` (through
//! `run_with_model`); `CheckArgs` takes no binding, and `check_source` never calls
//! `binding_over`. The binding refusals the unit's `binding.rs` test asserts arise only from
//! `binding`/`binding_over`, which `ess generate ui` calls. So a slot whose command no network
//! component serves passes `ess ui check --model` with exit 0, and is refused only at generation.

use std::path::{Path, PathBuf};

use ess_ui_check::{check_source, model_from_sources, Options};

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The fixture model, its one component no longer reached over a network: nothing is served.
fn unserved_shop() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model");
    let components = read(&root.join("components.yaml"));
    assert!(components.contains("    reached_by: network\n"));
    vec![
        ("system.yaml".to_owned(), read(&root.join("system.yaml"))),
        (
            "components.yaml".to_owned(),
            components.replace("    reached_by: network\n", ""),
        ),
        (
            "domains/stock.yaml".to_owned(),
            read(&root.join("domains/stock.yaml")),
        ),
        (
            "domains/audit.yaml".to_owned(),
            read(&root.join("domains/audit.yaml")),
        ),
    ]
}

#[test]
fn ess_ui_check_with_a_model_refuses_a_slot_command_nothing_serves() {
    let sources = unserved_shop();
    let model = model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|e| panic!("{e}"));
    let text = "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
         navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
         pages:\n  p: {kind: detail_page, title: P, sections: [\
         {name: map, host: StockMap, reason: a vendor map, stands_in_for: collection, \
          reads: stock.Items, commands: [stock.AddItem]}]}\n";
    // The binding refuses both (the unit's own `binding.rs` asserts that refusal).
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    assert!(
        ess_ui_check::binding(&document, &sources).is_err(),
        "the binding refuses the unserved slot"
    );
    // `ess ui check --model` does not.
    let report = check_source(
        text,
        "test.yaml",
        Path::new("."),
        Some(&model),
        &Options::default(),
    );
    assert!(
        report.has_errors()
            && report
                .findings
                .iter()
                .any(|finding| finding.path == "pages/p/sections/map/commands"),
        "`ess ui check --model` accepts a slot command no binding serves: {:#?}",
        report.findings
    );
}
