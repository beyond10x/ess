//! The concepts page that names the ways to adopt ESS, and the two Start pages that link it.
use std::{fs, path::PathBuf};

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("website/docs").is_dir())
        .expect("the workspace holds website/docs")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    fs::read_to_string(workspace().join(relative))
        .unwrap_or_else(|_| panic!("missing page {relative}"))
}

const PAGE: &str = "website/docs/concepts/adoption-modes.md";

#[test]
fn adoption_modes_page_states_the_modes() {
    let page = read(PAGE);
    assert!(
        page.contains("title: Adopt ESS at the depth you need"),
        "{PAGE}: missing title `Adopt ESS at the depth you need`"
    );

    let columns = [
        "mode",
        "commands",
        "what you commit",
        "what it checks",
        "what it does not",
        "next step",
    ];
    let header = format!("| {} |", columns.join(" | "));
    let tables = page.lines().filter(|line| line.starts_with("| ")).count();
    let Some(start) = page.lines().position(|line| line == header) else {
        let found = page
            .lines()
            .find(|line| line.starts_with("| "))
            .unwrap_or("no table");
        let missing: Vec<_> = columns
            .iter()
            .filter(|column| !found.contains(&format!("| {column} |")))
            .collect();
        panic!("{PAGE}: table header is not `{header}`; missing column(s) {missing:?}");
    };
    let rows: Vec<&str> = page
        .lines()
        .skip(start + 2)
        .take_while(|line| line.starts_with('|'))
        .collect();
    assert_eq!(tables, rows.len() + 1, "{PAGE}: exactly one table");
    for row in &rows {
        assert_eq!(
            row.matches(" | ").count(),
            columns.len() - 1,
            "{PAGE}: a row has one cell per column: {row}"
        );
    }
    for mode in [
        "specification only",
        "conformance against a hand-written implementation",
        "generated contracts beside hand-written code",
        "generated behaviour",
        "retrofitting",
    ] {
        assert!(
            rows.iter().any(|row| row
                .trim_start_matches('|')
                .split(" | ")
                .next()
                .is_some_and(|cell| cell.to_lowercase().contains(mode))),
            "{PAGE}: no row's mode is {mode}"
        );
    }
    for phrase in ["the modes are independent", "stop at any of them"] {
        assert!(
            page.to_lowercase().contains(phrase),
            "{PAGE}: missing phrase `{phrase}`"
        );
    }
    assert!(
        page.contains("../guides/commit-generated-files.md"),
        "{PAGE}: missing link to guides/commit-generated-files.md"
    );

    let start = read("website/docs/getting-started.md");
    assert!(
        start.contains("./concepts/adoption-modes.md"),
        "website/docs/getting-started.md: missing link to concepts/adoption-modes.md"
    );
    let index = read("website/docs/index.md");
    let next = index
        .split("## Where to go next")
        .nth(1)
        .expect("website/docs/index.md: missing `## Where to go next`");
    let next = next.split("\n## ").next().unwrap_or(next);
    assert!(
        next.lines()
            .any(|line| line.starts_with('|') && line.contains("./concepts/adoption-modes.md")),
        "website/docs/index.md: the `Where to go next` table has no link to concepts/adoption-modes.md"
    );
}
