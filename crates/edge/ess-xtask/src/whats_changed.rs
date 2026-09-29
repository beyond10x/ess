//! `WHATS-CHANGED.md` and the site's What-changed page, rendered from the `changes/` fragments.
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
//! So this renders them into one file at the root and one page of the documentation site, which
//! also links each release's post from `website/blog/`, and checks them on the way past.

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Where the fragments live. Also packaged by `.github/workflows/b10x-docs-bundle.yml`.
const FRAGMENTS: &str = "changes";

/// The rendered file.
pub const RENDERED: &str = "WHATS-CHANGED.md";

/// The same record as a page of the documentation site.
pub const SITE_PAGE: &str = "website/docs/releases/what-changed.md";

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

/// Newest release first; inside one release, the order the fragments are named in.
fn newest_first(fragments: &[Fragment]) -> Vec<&Fragment> {
    let mut grouped: Vec<&Fragment> = fragments.iter().collect();
    grouped.sort_by(|left, right| {
        ordinal(&right.source.version)
            .cmp(&ordinal(&left.source.version))
            .then_with(|| left.id.cmp(&right.id))
    });
    grouped
}

fn render(fragments: &[Fragment]) -> String {
    let grouped = newest_first(fragments);

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

/// Where the release posts live, and where the site publishes them.
const BLOG: &str = "website/blog";
const POST_BASE: &str = "https://beyond10x.github.io/ess/releases";

/// The release post for each version, by the `release_tag` and `slug` its front matter declares.
///
/// Older posts carry a historical tag spelling (`0.3.0-ess-wave-1`) that names no fragment's
/// version, so only an exact version match links; a post without a `slug` is not linked, since
/// its address would have to be guessed.
fn release_posts(root: &Path) -> Result<BTreeMap<String, String>> {
    let directory = root.join(BLOG);
    let mut posts = BTreeMap::new();
    if !directory.is_dir() {
        return Ok(posts);
    }
    for entry in fs::read_dir(&directory).with_context(|| format!("read {BLOG}"))? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        let front: Vec<&str> = text
            .lines()
            .skip(1)
            .take_while(|line| line.trim_end() != "---")
            .collect();
        let field = |key: &str| {
            front
                .iter()
                .find_map(|line| line.trim().strip_prefix(key))
                .map(|value| value.trim().trim_matches('"').to_owned())
        };
        if let (Some(tag), Some(slug)) = (field("release_tag:"), field("slug:")) {
            posts.insert(tag, slug);
        }
    }
    Ok(posts)
}

/// Text made safe for a page the site parses as MDX, where `{` opens an expression and `<` a tag.
///
/// A backslash escape is plain Markdown, so the same text reads the same in a renderer that
/// is not MDX. Code spans are left alone: nothing inside one is parsed.
fn site_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_code = false;
    for character in text.chars() {
        if character == '`' {
            in_code = !in_code;
        } else if !in_code && matches!(character, '{' | '}' | '<') {
            out.push('\\');
        }
        out.push(character);
    }
    out
}

fn render_site(fragments: &[Fragment], posts: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    out.push_str(
        "---\ntitle: What changed\ndescription: What each ESS release changed for somebody using \
         it, newest release first.\n---\n\n",
    );
    out.push_str(
        "<!-- ess-what-changed-begin: generated from changes/*.yaml by `cargo xtask \
         whats-changed`; edit a change record, not this page -->\n\n",
    );
    out.push_str(
        "What each ESS release is worth to somebody using it: what became possible, how much it \
         matters, and where to read the rest, newest release first. The \
         [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) is the complete \
         record at the level each change was made; this page is the short one.\n\n",
    );
    out.push_str(
        "This page is generated from the change records kept in the repository. A release with no \
         entry here added nothing somebody using ESS would act on.\n",
    );
    let mut current = String::new();
    for fragment in newest_first(fragments) {
        let version = &fragment.source.version;
        if *version != current {
            current.clone_from(version);
            let _ = write!(
                out,
                "\n## {version} — {date}\n",
                date = &fragment.published_at[..10],
            );
        }
        let _ = write!(out, "\n### {}\n\n", site_text(&fragment.title));
        let _ = write!(
            out,
            "{kind} · {impact} impact · ",
            kind = fragment.kind,
            impact = fragment.impact,
        );
        if let Some(slug) = posts.get(version) {
            let _ = write!(out, "[release post]({POST_BASE}/{slug}) · ");
        }
        let _ = write!(out, "[release notes]({url})\n\n", url = fragment.source.url);
        out.push_str(&site_text(fragment.summary.trim()));
        out.push('\n');
    }
    out.push_str("\n<!-- ess-what-changed-end -->\n");
    out
}

pub fn run(root: &Path, check: bool) -> Result<String> {
    let fragments = read_all(root)?;
    let posts = release_posts(root)?;
    let outputs = [
        (RENDERED, render(&fragments)),
        (SITE_PAGE, render_site(&fragments, &posts)),
    ];
    if !check {
        for (relative, rendered) in &outputs {
            let path = root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, rendered).with_context(|| format!("write {relative}"))?;
        }
        return Ok(format!(
            "{RENDERED} and {SITE_PAGE}: {} entries from {FRAGMENTS}/\n",
            fragments.len()
        ));
    }
    let stale: Vec<&str> = outputs
        .iter()
        .filter(|(relative, rendered)| {
            fs::read_to_string(root.join(relative)).unwrap_or_default() != *rendered
        })
        .map(|(relative, _)| *relative)
        .collect();
    if !stale.is_empty() {
        bail!(
            "{} not what {FRAGMENTS}/ renders to; run `cargo xtask whats-changed`. A fragment or a \
             release post was added or edited and the rendered output was left behind.",
            match stale.as_slice() {
                [one] => format!("{one} is"),
                many => format!("{} are", many.join(" and ")),
            }
        );
    }
    Ok(format!(
        "{RENDERED} and {SITE_PAGE}: {} entries, byte-identical to {FRAGMENTS}/\n",
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

    /// A throwaway repository root holding two fragments and a release post for one of them.
    fn fixture(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "ess-xtask-whats-changed-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(FRAGMENTS)).expect("fragments directory");
        fs::create_dir_all(root.join("website/blog")).expect("blog directory");
        for (stem, version, title, summary) in [
            (
                "older-0.1.0",
                "0.1.0",
                "The older change",
                "Nothing links this one to a post.",
            ),
            (
                "newer-0.2.0",
                "0.2.0",
                "The newer change",
                "A value {like this} or a <tag> is text, and `{$instance}` stays code.",
            ),
        ] {
            let text = format!(
                "schema: {SCHEMA}\nid: ess/{stem}\nrepository: ess\n\
                 publishedAt: 2026-09-29T20:00:00Z\ntitle: {title}\nsummary: >-\n  {summary}\n\
                 kind: capability\nimpact: significant\nsource:\n  \
                 url: https://github.com/beyond10x/ess/releases/tag/{version}\n  \
                 version: {version}\njourneys: [specify]\naffectedSurfaces: [ess/docs]\n"
            );
            fs::write(root.join(FRAGMENTS).join(format!("{stem}.yaml")), text).expect("fragment");
        }
        fs::write(
            root.join("website/blog/2026-09-29-2000-newer.md"),
            "---\ntitle: \"0.2 — newer\"\nslug: the-newer-post\ntags: [release, ess]\n\
             release_tag: \"0.2.0\"\n---\n\nBody.\n",
        )
        .expect("post");
        root
    }

    #[test]
    fn the_site_page_is_written_with_links_to_the_release_posts() {
        let root = fixture("write");
        run(&root, false).expect("renders");
        let page = fs::read_to_string(root.join(SITE_PAGE)).expect("the site page is written");
        assert!(page.starts_with("---\ntitle: What changed\n"), "{page}");
        assert!(page.contains("generated"), "the page says it is generated");
        let newer = page.find("### The newer change").expect("newer section");
        let older = page.find("### The older change").expect("older section");
        assert!(newer < older, "newest first");
        assert!(
            page.contains(
                "[release post](https://beyond10x.github.io/ess/releases/the-newer-post)"
            ),
            "{page}"
        );
        assert_eq!(
            page.matches("[release post]").count(),
            1,
            "0.1.0 has no post"
        );
        assert!(
            page.contains(
                r"A value \{like this\} or a \<tag> is text, and `{$instance}` stays code."
            ),
            "{page}"
        );
        run(&root, true).expect("fresh outputs pass the check");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_check_fails_when_the_site_page_is_stale() {
        let root = fixture("stale");
        run(&root, false).expect("renders");
        let page = root.join(SITE_PAGE);
        fs::create_dir_all(page.parent().expect("parent")).expect("page directory");
        fs::write(&page, "---\ntitle: What changed\n---\n").expect("stale page");
        let error = run(&root, true).expect_err("a stale site page fails the check");
        assert!(format!("{error:#}").contains(SITE_PAGE), "{error:#}");
        fs::remove_file(&page).expect("remove page");
        run(&root, true).expect_err("a missing site page fails the check");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_summary_over_the_public_limit_is_refused() {
        let long = "x".repeat(SUMMARY_LIMIT + 1);
        assert!(long.chars().count() > SUMMARY_LIMIT);
    }
}
