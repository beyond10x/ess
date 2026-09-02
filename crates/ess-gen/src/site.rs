//! The documentation, ready to publish.
//!
//! `docs` writes markdown a person can read in a repository. A static site generator wants two more
//! things from the same bytes — a title it can put in a sidebar and an order to put pages in — and
//! every adopter who published these pages wrote that layer themselves: a script that renamed
//! files, prepended frontmatter and rewrote the links it had just broken. That script is the thing
//! this projection replaces, and it belongs here because both halves of it are readings of the
//! model.
//!
//! # It wraps `docs` rather than repeating it
//!
//! One projection produces the prose and this one presents it. A second copy of the domain walk
//! would be a second place for a section to be forgotten, and the pages would drift apart exactly
//! where a reader compares them.
//!
//! # What it does not carry
//!
//! A palette, a font, a component vocabulary. Presentation belongs to whatever builds the site —
//! `docs-system` for a beyond10x repository, a plain Docusaurus theme for anybody else — and a
//! generator that emitted CSS would be a second design system nobody asked for, in a tool whose
//! subject is specifications.

use std::fmt::Write as _;

use ess_compiler::EssIr;

use crate::artifact::{Artifact, Generator};
use crate::docs::Docs;
use crate::provenance::{ProvenanceMint, SlicedProvenance};

/// The documentation with the frontmatter and the sidebar a static site needs.
pub struct Site;

impl Generator for Site {
    fn name(&self) -> &'static str {
        "site"
    }

    fn describes(&self) -> &'static str {
        "the documentation with frontmatter and a sidebar, ready for a static site"
    }

    fn directory(&self) -> &'static str {
        "site"
    }

    fn generate(&self, ir: &EssIr, mint: &ProvenanceMint) -> Vec<Artifact> {
        let pages = Docs.generate(ir, mint);

        // The index first, then every other page in path order. Alphabetical is not a judgement
        // about importance — it is the one order two runs agree on, and a projection that ranked
        // pages by significance would be inventing a claim the model does not make.
        let mut ordered: Vec<&Artifact> = pages.iter().collect();
        ordered.sort_by_key(|artifact| (artifact.path != "index.md", artifact.path.clone()));
        let position: std::collections::BTreeMap<&str, usize> = ordered
            .iter()
            .enumerate()
            .map(|(index, artifact)| (artifact.path.as_str(), index))
            .collect();

        let mut out: Vec<Artifact> = pages
            .iter()
            .map(|artifact| Artifact {
                path: artifact.path.clone(),
                contents: with_frontmatter(
                    &artifact.contents,
                    position.get(artifact.path.as_str()).copied().unwrap_or(0),
                ),
                slice: artifact.slice.clone(),
            })
            .collect();

        out.push(Artifact::new(
            "sidebar.json",
            sidebar(&mint.whole(), &ordered),
        ));
        out
    }
}

/// The page with a frontmatter block in front of it.
///
/// The title is the page's own `# ` heading rather than a name re-derived here: the heading is what
/// a reader sees at the top of the page, and a sidebar entry that says something else is a sidebar
/// entry pointing at what looks like a different page.
fn with_frontmatter(contents: &str, position: usize) -> String {
    let title = heading(contents).unwrap_or("Documentation");
    let mut out = String::from("---\n");
    // Quoted, because a title may hold a colon — `acd v3: the boundary` — and an unquoted YAML
    // scalar with one in it is a mapping.
    let _ = writeln!(out, "title: {}", quoted(title));
    let _ = writeln!(out, "sidebar_position: {position}");
    out.push_str("---\n\n");
    out.push_str(contents);
    out
}

/// The first `# ` heading, which every generated page has.
fn heading(contents: &str) -> Option<&str> {
    contents
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(str::trim)
}

/// A YAML double-quoted scalar.
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The sidebar, as the `sidebar.json` a site's own configuration imports.
///
/// A flat list of ids under `items`, and not a tree. Nesting would be this projection deciding that
/// `domains/` is a group, which is a fact about a file layout rather than about the model — and a
/// site that wants groups has its own configuration to say so in.
///
/// An object rather than a bare array, because JSON has no comments and every artifact has to say
/// what it derives from. The two digests are the same stamp the markdown pages carry in an HTML
/// comment, spelled the way a JSON reader can hold it.
fn sidebar(sliced: &SlicedProvenance, ordered: &[&Artifact]) -> String {
    let entries: Vec<String> = ordered
        .iter()
        .map(|artifact| {
            let id = artifact
                .path
                .strip_suffix(".md")
                .unwrap_or(&artifact.path)
                .to_owned();
            format!("    \"{id}\"")
        })
        .collect();
    let provenance = &sliced.provenance;
    format!(
        "{{\n  \"source_digest\": \"{}\",\n  \"contract_digest\": \"{}\",\n  \"items\": [\n{}\n  ]\n}}\n",
        provenance.source_digest,
        provenance.contract_digest,
        entries.join(",\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_carries_its_own_heading_as_its_title() {
        let page = with_frontmatter("<!--\ngenerated\n-->\n\n# Call routing\n\nSome prose.\n", 3);
        assert!(page.starts_with("---\ntitle: \"Call routing\"\nsidebar_position: 3\n---\n\n"));
        assert!(
            page.contains("# Call routing"),
            "the heading stays: frontmatter is added, not substituted"
        );
    }

    #[test]
    fn a_title_holding_a_colon_is_quoted_rather_than_read_as_a_mapping() {
        let page = with_frontmatter("# acd v3: the boundary\n", 0);
        assert!(page.contains("title: \"acd v3: the boundary\""), "{page}");
    }

    #[test]
    fn the_index_is_first_and_the_rest_are_in_one_order_twice() {
        let pages: Vec<Artifact> = ["topology.md", "index.md", "domains/acd-routing.md"]
            .into_iter()
            .map(|path| Artifact::new(path, "# A page\n"))
            .collect();
        let mut ordered: Vec<&Artifact> = pages.iter().collect();
        ordered.sort_by_key(|artifact| (artifact.path != "index.md", artifact.path.clone()));

        assert_eq!(
            ordered
                .iter()
                .map(|it| it.path.as_str())
                .collect::<Vec<_>>(),
            vec!["index.md", "domains/acd-routing.md", "topology.md"]
        );
    }
}
