//! The reference page documents the language: one example per step, every example a valid test
//! file, and design notes comparing node paths with selector-based browser tests.

use std::path::{Path, PathBuf};

fn page() -> String {
    let file: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/reference/ess-ui-test.md");
    std::fs::read_to_string(&file).unwrap_or_else(|error| panic!("{}: {error}", file.display()))
}

/// Every fenced `yaml` block of the page.
fn yaml_blocks(page: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in page.lines() {
        match (&mut current, line.trim_start()) {
            (None, "```yaml") => current = Some(String::new()),
            (Some(block), "```") => {
                blocks.push(std::mem::take(block));
                current = None;
            }
            (Some(block), _) => {
                block.push_str(line);
                block.push('\n');
            }
            (None, _) => {}
        }
    }
    blocks
}

#[test]
fn every_test_file_on_the_page_parses() {
    let page = page();
    let files: Vec<String> = yaml_blocks(&page)
        .into_iter()
        .filter(|block| block.starts_with("format: ess-ui-test/1"))
        .collect();
    assert!(!files.is_empty(), "the page shows a whole test file");
    for block in files {
        ess_ui_test::parse_str(&block, Path::new("ess-ui-test.md"))
            .unwrap_or_else(|error| panic!("{error}\n{block}"));
    }
}

#[test]
fn every_step_has_an_example_that_parses() {
    let page = page();
    let steps: Vec<String> = yaml_blocks(&page)
        .into_iter()
        .filter(|block| block.trim_start().starts_with("- "))
        .collect();
    for keyword in ess_ui_test::STEPS {
        let example = steps
            .iter()
            .find(|block| {
                block
                    .trim_start()
                    .strip_prefix("- ")
                    .is_some_and(|rest| rest.starts_with(&format!("{keyword}:")))
            })
            .unwrap_or_else(|| panic!("no example for `{keyword}`"));
        ess_ui_test::parse_steps(example)
            .unwrap_or_else(|error| panic!("{keyword}: {error}\n{example}"));
    }
}

#[test]
fn the_design_notes_compare_node_paths_with_selector_based_tests() {
    let page = page();
    for wanted in [
        "## Design notes",
        "data attribute",
        "CSS class",
        "visible text",
        "data-ui-path",
        "ess-ui-test-report/1",
    ] {
        assert!(page.contains(wanted), "{wanted}");
    }
}
