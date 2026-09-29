//! The example page, "A specification and its contracts", shows real files: its own words are
//! that everything on it is copied out of the repository. These cases hold it to that.
//!
//! Every fenced block on the page names the file it was copied from in its info string —
//! `file=<repository path>`, with `lines=<first>-<last>` when it is an excerpt — and each block is
//! compared with that file, byte for byte, so a regenerated artifact the page was not updated for
//! fails here rather than on a reader. A block with no `file=` fails too: the page makes the claim
//! for everything below its introduction, so a new block cannot opt out by saying nothing.
//!
//! The page also renders on two sites — this repository's own and the unified documentation site —
//! and the second compiles plain Markdown only, so the page may not import a component, and the
//! documentation manifest may not exclude it.

use std::path::{Path, PathBuf};

const PAGE: &str = "website/docs/examples/specification-to-contracts.md";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| {
            std::fs::read_to_string(directory.join("Cargo.toml"))
                .is_ok_and(|manifest| manifest.starts_with("[workspace]"))
        })
        .expect("the workspace root is an ancestor of the crate")
        .to_path_buf()
}

fn page() -> String {
    std::fs::read_to_string(repository_root().join(PAGE)).expect("the example page exists")
}

/// One fenced block: the line its opening fence is on, its info string, and its body with every
/// line newline-terminated — the form a copied run of lines has in the file it came from.
struct Block {
    line: usize,
    info: String,
    body: String,
}

fn fenced_blocks(markdown: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut open: Option<(usize, String, String, String)> = None;
    for (index, line) in markdown.lines().enumerate() {
        match &mut open {
            None => {
                let fence: String = line.chars().take_while(|c| *c == '`').collect();
                if fence.len() >= 3 {
                    let info = line[fence.len()..].trim().to_owned();
                    open = Some((index + 1, fence, info, String::new()));
                }
            }
            Some((start, fence, info, body)) => {
                if line.trim_end() == fence.as_str() {
                    blocks.push(Block {
                        line: *start,
                        info: std::mem::take(info),
                        body: std::mem::take(body),
                    });
                    open = None;
                } else {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    assert!(open.is_none(), "{PAGE}: a fenced block is never closed");
    blocks
}

fn attribute<'a>(info: &'a str, key: &str) -> Option<&'a str> {
    info.split_whitespace()
        .find_map(|word| word.strip_prefix(key)?.strip_prefix('='))
}

/// The bytes a block claims to hold: the whole file, or the named run of its lines (1-based,
/// inclusive), each with its newline.
fn source_text(block: &Block, file: &str) -> Result<String, String> {
    let path = repository_root().join(file);
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let Some(range) = attribute(&block.info, "lines") else {
        return Ok(contents);
    };
    let (first, last) = range
        .split_once('-')
        .and_then(|(a, b)| Some((a.parse::<usize>().ok()?, b.parse::<usize>().ok()?)))
        .filter(|(first, last)| *first >= 1 && first <= last)
        .ok_or_else(|| format!("`lines={range}` is not <first>-<last>"))?;
    let lines: Vec<&str> = contents.lines().collect();
    if last > lines.len() {
        return Err(format!(
            "`lines={range}` runs past the end of {file}, which has {} lines",
            lines.len()
        ));
    }
    let mut excerpt = String::new();
    for line in &lines[first - 1..last] {
        excerpt.push_str(line);
        excerpt.push('\n');
    }
    Ok(excerpt)
}

/// The first line on which two texts differ, for a failure a reader can act on.
fn first_difference(page: &str, source: &str) -> String {
    let (mut page_lines, mut source_lines) = (page.lines(), source.lines());
    let mut number = 1;
    loop {
        match (page_lines.next(), source_lines.next()) {
            (Some(a), Some(b)) if a == b => number += 1,
            (None, None) => return "the texts differ only in their final newline".to_owned(),
            (a, b) => {
                return format!(
                    "line {number} of the block\n    page:   {}\n    source: {}",
                    a.unwrap_or("<the block ends here>"),
                    b.unwrap_or("<the source ends here>")
                );
            }
        }
    }
}

#[test]
fn every_block_on_the_example_page_is_the_file_it_names() {
    let blocks = fenced_blocks(&page());
    assert!(
        blocks.len() >= 6,
        "{PAGE}: expected the page's copied blocks, found {} fenced blocks",
        blocks.len()
    );
    let mut failures = Vec::new();
    for block in &blocks {
        let Some(file) = attribute(&block.info, "file") else {
            failures.push(format!(
                "{PAGE}:{}: fenced block `{}` names no `file=`; the page says everything on it is \
                 copied out of the repository, so every block names what it copies",
                block.line, block.info
            ));
            continue;
        };
        match source_text(block, file) {
            Err(error) => failures.push(format!("{PAGE}:{}: {error}", block.line)),
            Ok(source) if source != block.body => failures.push(format!(
                "{PAGE}:{}: the block no longer matches {file}{}; first difference at {}",
                block.line,
                attribute(&block.info, "lines")
                    .map(|range| format!(" lines {range}"))
                    .unwrap_or_default(),
                first_difference(&block.body, &source)
            )),
            Ok(_) => {}
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_example_page_is_plain_markdown() {
    let text = page();
    let mut in_fence = false;
    let mut failures = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let trimmed = line.trim_start();
        let component = trimmed
            .strip_prefix('<')
            .and_then(|rest| rest.chars().next())
            .is_some_and(|c| c.is_ascii_uppercase());
        if trimmed.starts_with("import ") || trimmed.starts_with("export ") || component {
            failures.push(format!("{PAGE}:{}: `{line}`", index + 1));
        }
    }
    assert!(
        failures.is_empty(),
        "the unified documentation site compiles plain Markdown, so the page may not import or \
         render a component; the lab has its own page on this site:\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_documentation_manifest_publishes_the_example_page() {
    let manifest = std::fs::read_to_string(repository_root().join("b10x.docs.yaml"))
        .expect("the documentation manifest exists");
    let relative = PAGE
        .strip_prefix("website/")
        .expect("the page is under the site root");
    assert!(
        !manifest.contains(relative),
        "b10x.docs.yaml still names {relative}, which keeps the example page off the unified site"
    );
}
