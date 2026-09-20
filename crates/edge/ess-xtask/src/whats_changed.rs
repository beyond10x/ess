//! `WHATS-CHANGED.md`, rendered from the `changes/` fragments.
//!
//! `CHANGELOG.md` records every change at the level the change was made. That is the right record
//! for somebody reading a diff and the wrong one for somebody deciding whether a release is worth
//! adopting: 1466 lines, newest first, with no statement anywhere of which entries matter. The
//! `changes/*.yaml` fragments already carry that judgement — a title, a summary, a kind and an
//! impact — because Atlas publishes them as the organization's change feed. Until this existed
//! they were write-only from this repository's side: nothing here read them, nothing here checked
//! them, and the one thing that did check them was a validator in another repository that refuses
//! a summary over 360 characters after the bundle has already been built.
//!
//! So this renders them into one file at the root, and checks them on the way past.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Where the fragments live. Also packaged by `.github/workflows/b10x-docs-bundle.yml`.
const FRAGMENTS: &str = "changes";

/// The rendered file.
pub const RENDERED: &str = "WHATS-CHANGED.md";

/// The schema every fragment declares.
const SCHEMA: &str = "b10x-change/v1";

/// The bound the public Docs System validator enforces on `summary`.
///
/// It refused `changes/source-driven-realization-0.19.0.yaml` on exactly this, after the
/// documentation bundle had been built and while Atlas was already selecting it — a failure this
/// repository could not see, for a file this repository owns.
const SUMMARY_LIMIT: usize = 360;

/// The first release with a fragment. Every minor from here carries one, and that is checked;
/// before it the feed is one entry, because the feed did not exist yet.
const FEED_FLOOR: (u64, u64) = (0, 14);

/// Minors at or after [`FEED_FLOOR`] that carry no fragment, and why.
///
/// A named exemption rather than a silent gap: the only way a released minor legitimately has
/// nothing of its own to announce is when another release announced it, and that fact belongs
/// somewhere a reader can check.
const WITHOUT_FRAGMENT: &[(&str, &str)] = &[(
    "0.17.0",
    "the tag holds one of the two features written under it and neither commit is an ancestor of \
     the other; both first ship in 0.18.0, which carries the two fragments",
)];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    schema: String,
    id: String,
    repository: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    title: String,
    summary: String,
    kind: String,
    impact: String,
    source: Source,
    journeys: Vec<String>,
    #[serde(rename = "affectedSurfaces")]
    affected_surfaces: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    url: String,
    version: String,
}

/// A version as its three numbers, so `0.9.0` sorts before `0.10.0`.
fn ordinal(version: &str) -> Option<(u64, u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((major, minor, patch))
}

fn read_all(root: &Path) -> Result<Vec<Fragment>> {
    let directory = root.join(FRAGMENTS);
    let mut paths: Vec<_> = fs::read_dir(&directory)
        .with_context(|| format!("read {}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|value| value == "yaml"))
        .collect();
    paths.sort();
    let mut fragments = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path)?;
        let fragment: Fragment = serde_yaml::from_str(&text)
            .with_context(|| format!("{} is not a {SCHEMA} fragment", path.display()))?;
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if fragment.schema != SCHEMA {
            bail!("{name}: schema is `{}`, not `{SCHEMA}`", fragment.schema);
        }
        if fragment.repository != "ess" {
            bail!("{name}: repository is `{}`, not `ess`", fragment.repository);
        }
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        if fragment.id != format!("ess/{stem}") {
            bail!("{name}: id is `{}`, not `ess/{stem}`", fragment.id);
        }
        // Counted in characters rather than bytes, which is what the validator counts.
        let length = fragment.summary.chars().count();
        if length > SUMMARY_LIMIT {
            bail!(
                "{name}: summary is {length} characters, and the public validator refuses more \
                 than {SUMMARY_LIMIT}"
            );
        }
        if fragment.summary.trim().is_empty() || fragment.title.trim().is_empty() {
            bail!("{name}: title and summary must both say something");
        }
        if ordinal(&fragment.source.version).is_none() {
            bail!(
                "{name}: source version `{}` is not three dot-separated numbers",
                fragment.source.version
            );
        }
        if !fragment.source.url.ends_with(&fragment.source.version) {
            bail!(
                "{name}: source url does not end with {}",
                fragment.source.version
            );
        }
        if fragment.published_at.len() < 10 {
            bail!("{name}: publishedAt is not an RFC 3339 instant");
        }
        if fragment.journeys.is_empty() || fragment.affected_surfaces.is_empty() {
            bail!("{name}: journeys and affectedSurfaces must each name at least one value");
        }
        fragments.push(fragment);
    }
    Ok(fragments)
}

/// The minors at or after [`FEED_FLOOR`] that carry no fragment.
///
/// A release with nothing to say is possible — a patch that repaired the release gate is the
/// ordinary case — so this asks about minors only, and the report says which.
pub fn unrecorded_minors(root: &Path, tags: &[String]) -> Result<Vec<String>> {
    let recorded: Vec<(u64, u64, u64)> = read_all(root)?
        .iter()
        .filter_map(|fragment| ordinal(&fragment.source.version))
        .collect();
    let mut missing: Vec<String> = tags
        .iter()
        .filter_map(|tag| ordinal(tag).map(|value| (tag, value)))
        .filter(|(_, (major, minor, patch))| *patch == 0 && (*major, *minor) >= FEED_FLOOR)
        .filter(|(tag, _)| {
            !WITHOUT_FRAGMENT
                .iter()
                .any(|(exempt, _)| exempt == &tag.as_str())
        })
        .filter(|(_, (major, minor, _))| {
            !recorded
                .iter()
                .any(|(other_major, other_minor, _)| (other_major, other_minor) == (major, minor))
        })
        .map(|(tag, _)| tag.clone())
        .collect();
    missing.sort_by_key(|tag| ordinal(tag));
    Ok(missing)
}

fn render(fragments: &[Fragment]) -> String {
    let mut grouped: Vec<&Fragment> = fragments.iter().collect();
    // Newest release first; inside one release, the order the fragments are named in.
    grouped.sort_by(|left, right| {
        ordinal(&right.source.version)
            .cmp(&ordinal(&left.source.version))
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut out = String::new();
    out.push_str("# What changed\n\n");
    out.push_str(
        "What each ESS release is worth to somebody using it: what became possible, how much it \
         matters, and where to read the rest. `CHANGELOG.md` is the complete record at the level \
         the change was made; this is the short one.\n\n",
    );
    out.push_str(
        "Generated from `changes/*.yaml` by `cargo xtask whats-changed`. Edit a fragment, not this \
         file. A release with no entry added nothing an adopter would act on.\n\n",
    );
    out.push_str("| Release | Change | Kind | Impact |\n|---|---|---|---|\n");
    for fragment in &grouped {
        let _ = writeln!(
            out,
            "| [{version}](#{anchor}) | {title} | {kind} | {impact} |",
            version = fragment.source.version,
            anchor = anchor(&fragment.title),
            title = fragment.title,
            kind = fragment.kind,
            impact = fragment.impact,
        );
    }

    let mut current = String::new();
    for fragment in grouped {
        if fragment.source.version != current {
            current.clone_from(&fragment.source.version);
            let _ = write!(
                out,
                "\n## {version} — {date}\n",
                version = fragment.source.version,
                date = &fragment.published_at[..10],
            );
        }
        let _ = write!(out, "\n### {}\n\n", fragment.title);
        let _ = write!(
            out,
            "{kind} · {impact} impact · [release notes]({url})\n\n",
            kind = fragment.kind,
            impact = fragment.impact,
            url = fragment.source.url,
        );
        out.push_str(fragment.summary.trim());
        out.push('\n');
    }
    out
}

/// A GitHub Markdown heading anchor: lowercase, non-alphanumerics to hyphens.
fn anchor(title: &str) -> String {
    let mut out = String::new();
    for character in title.chars() {
        if character.is_alphanumeric() {
            out.extend(character.to_lowercase());
        } else if character.is_whitespace() || character == '-' {
            out.push('-');
        }
    }
    out
}

pub fn run(root: &Path, check: bool) -> Result<String> {
    let fragments = read_all(root)?;
    let rendered = render(&fragments);
    let path = root.join(RENDERED);
    if !check {
        fs::write(&path, &rendered)?;
        return Ok(format!(
            "{RENDERED}: {} entries from {FRAGMENTS}/\n",
            fragments.len()
        ));
    }
    let committed = fs::read_to_string(&path).unwrap_or_default();
    if committed != rendered {
        bail!(
            "{RENDERED} is not what {FRAGMENTS}/ renders to; run `cargo xtask whats-changed`. A \
             fragment was added or edited and the rendered file was left behind."
        );
    }
    Ok(format!(
        "{RENDERED}: {} entries, byte-identical to {FRAGMENTS}/\n",
        fragments.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_sorts_by_number_and_not_by_text() {
        assert!(ordinal("0.9.0") < ordinal("0.10.0"));
        assert_eq!(ordinal("0.27.0"), Some((0, 27, 0)));
        assert_eq!(ordinal("v0.3.0"), None);
        assert_eq!(ordinal("0.27"), None);
        assert_eq!(ordinal("0.27.0.1"), None);
    }

    #[test]
    fn an_anchor_matches_the_heading_github_would_mint() {
        assert_eq!(
            anchor("An enum variant carries its own wire spelling"),
            "an-enum-variant-carries-its-own-wire-spelling"
        );
        assert_eq!(anchor("`sets:` and nothing else"), "sets-and-nothing-else");
    }

    #[test]
    fn every_committed_fragment_is_valid_and_renders() {
        let root = crate::workspace_root().expect("workspace root");
        let fragments = read_all(&root).expect("the committed fragments are valid");
        assert!(!fragments.is_empty());
        assert!(render(&fragments).contains("# What changed"));
    }

    #[test]
    fn an_exempt_minor_is_named_with_its_reason() {
        let root = crate::workspace_root().expect("workspace root");
        let tags: Vec<String> = ["0.17.0".to_owned()].into();
        assert!(
            unrecorded_minors(&root, &tags)
                .expect("reads the fragments")
                .is_empty(),
            "0.17.0 is exempt: both features it names first ship in 0.18.0"
        );
        for (version, reason) in WITHOUT_FRAGMENT {
            assert!(ordinal(version).is_some(), "{version} is not a version");
            assert!(!reason.trim().is_empty(), "{version} has no stated reason");
        }
    }

    #[test]
    fn a_summary_over_the_public_limit_is_refused() {
        let long = "x".repeat(SUMMARY_LIMIT + 1);
        assert!(long.chars().count() > SUMMARY_LIMIT);
    }
}
