//! Refuses an admonition opened with a bare title, `:::note Some title`.
//!
//! Docusaurus 3 takes an admonition title only in brackets, `:::note[Some title]`. A space-separated
//! title is not an error to it: the line reaches the page as literal text and the box never opens.
//! The docs-system build guard fails a site build on that, and its source check lints
//! `^:::[a-z]+ +\S`; this is the same rule, offline, so `task check` finds it before the
//! documentation lanes do. Fenced code is skipped, since a page may quote the wrong form.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// The published page trees, relative to the repository root.
const ROOTS: &[&str] = &["website/docs", "website/blog"];

/// Checks every Markdown page under [`ROOTS`].
pub(super) fn run(root: &Path) -> Result<String> {
    let mut pages = Vec::new();
    for tree in ROOTS {
        collect(&root.join(tree), &mut pages)?;
    }
    pages.sort();
    let mut found = Vec::new();
    for page in &pages {
        let text =
            fs::read_to_string(page).with_context(|| format!("reading {}", page.display()))?;
        for line in raw_titles(&text) {
            let shown = page.strip_prefix(root).unwrap_or(page).display();
            found.push(format!("{shown}:{line}"));
        }
    }
    if !found.is_empty() {
        bail!(
            "admonition titles must be bracketed (`:::note[Title]`), not space-separated: {}",
            found.join(", ")
        );
    }
    Ok(format!(
        "{} page(s): no admonition with a space-separated title\n",
        pages.len()
    ))
}

fn collect(directory: &Path, pages: &mut Vec<PathBuf>) -> Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in
        fs::read_dir(directory).with_context(|| format!("reading {}", directory.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, pages)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "md" || extension == "mdx")
        {
            pages.push(path);
        }
    }
    Ok(())
}

/// One-based numbers of the lines outside fenced code that match `^:::[a-z]+ +\S`.
fn raw_titles(text: &str) -> Vec<usize> {
    let mut fence: Option<&str> = None;
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        match fence {
            Some(marker) => {
                if trimmed.starts_with(marker) {
                    fence = None;
                }
                continue;
            }
            None if trimmed.starts_with("```") => {
                fence = Some("```");
                continue;
            }
            None if trimmed.starts_with("~~~") => {
                fence = Some("~~~");
                continue;
            }
            None => {}
        }
        if is_raw_title(line) {
            found.push(index + 1);
        }
    }
    found
}

fn is_raw_title(line: &str) -> bool {
    let Some(rest) = line.strip_prefix(":::") else {
        return false;
    };
    let kind = rest.bytes().take_while(u8::is_ascii_lowercase).count();
    if kind == 0 {
        return false;
    }
    let after = &rest[kind..];
    let spaces = after.bytes().take_while(|byte| *byte == b' ').count();
    spaces > 0
        && after[spaces..]
            .chars()
            .next()
            .is_some_and(|c| !c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_space_separated_title_is_found_and_the_bracketed_form_is_not() {
        assert!(is_raw_title(":::note Some title"));
        assert!(is_raw_title(":::tip  x"));
        assert!(!is_raw_title(":::note[Some title]"));
        assert!(!is_raw_title(":::note"));
        assert!(!is_raw_title(":::note   "));
        assert!(!is_raw_title(":::"));
        assert!(!is_raw_title(" :::note Title"));
        assert!(!is_raw_title(":::Note Title"));
    }

    #[test]
    fn fenced_code_may_quote_the_wrong_form() {
        let text = "# Page\n\n```md\n:::note Quoted\n```\n\n:::warning Real\nbody\n:::\n";
        assert_eq!(raw_titles(text), [7]);
    }
}
