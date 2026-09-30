//! The documentation site's navigation, and the links between its pages.
//!
//! This is the check `story:docs-navigation-whole` is accepted against, as far as it can be decided
//! without a Docusaurus build. Five rules, each mechanical:
//!
//! - the sidebar has the adopter-facing categories in order, collapsed by default;
//! - every published page is in the sidebar, and every sidebar entry is a published page;
//! - a task page split out of a long guide is at most 400 lines;
//! - every relative link between pages lands on a page, and on a heading or anchor of it;
//! - every `#anchor` the two long guides published before they were split still lands on their
//!   old page, so a link from outside the repository keeps working.
//!
//! The link rule is the same one `onBrokenLinks: throw` enforces on a site build, extended to
//! anchors (which a build only warns about). It runs here because a site build needs the network.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// Every Markdown page under `dir`, recursively, sorted.
fn pages(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).expect("read a website directory") {
            let path = entry.expect("read a directory entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "md" || extension == "mdx")
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// The lines of a page that are prose: front matter and fenced code are dropped.
fn prose_lines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut all = text.lines().peekable();
    if all.peek() == Some(&"---") {
        all.next();
        for line in all.by_ref() {
            if line == "---" {
                break;
            }
        }
    }
    let mut fence: Option<&str> = None;
    for line in all {
        let trimmed = line.trim_start();
        match fence {
            Some(marker) => {
                if trimmed.starts_with(marker) {
                    fence = None;
                }
            }
            None if trimmed.starts_with("```") => fence = Some("```"),
            None if trimmed.starts_with("~~~") => fence = Some("~~~"),
            None => lines.push(line),
        }
    }
    lines
}

/// The slug Docusaurus gives a heading: the rendered text, lower-cased, with punctuation dropped and
/// each space turned into a hyphen.
fn slug(heading: &str) -> String {
    let mut text = String::new();
    let mut rest = heading;
    // A link in a heading renders as its text.
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find("](") else {
            break;
        };
        let Some(end) = rest[open + close..].find(')') else {
            break;
        };
        text.push_str(&rest[..open]);
        text.push_str(&rest[open + 1..open + close]);
        rest = &rest[open + close + end + 1..];
    }
    text.push_str(rest);
    text.to_lowercase()
        .chars()
        .filter_map(|character| match character {
            ' ' => Some('-'),
            '-' | '_' => Some(character),
            other if other.is_alphanumeric() => Some(other),
            _ => None,
        })
        .collect()
}

/// Every anchor a page offers: heading slugs (numbered when repeated, as Docusaurus does), explicit
/// `{#id}` heading ids, and `id="…"` attributes.
fn anchors(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for line in prose_lines(text) {
        let hashes = line
            .chars()
            .take_while(|&character| character == '#')
            .count();
        if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
            let heading = line[hashes..].trim();
            let id = match heading.rfind("{#") {
                Some(start) if heading.ends_with('}') => {
                    heading[start + 2..heading.len() - 1].to_owned()
                }
                _ => slug(heading),
            };
            let count = seen.entry(id.clone()).or_insert(0);
            found.insert(if *count == 0 {
                id.clone()
            } else {
                format!("{id}-{count}")
            });
            *count += 1;
        }
        let mut rest = line;
        while let Some(start) = rest.find(" id=\"") {
            let value = &rest[start + 5..];
            let end = value.find('"').expect("an id attribute closes");
            found.insert(value[..end].to_owned());
            rest = &value[end..];
        }
    }
    found
}

/// Every link target in a page's prose: inline `](target)` and reference definitions.
fn link_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in prose_lines(text) {
        // Inline code is not a link, and may hold `](`.
        let mut prose = String::new();
        for (index, part) in line.split('`').enumerate() {
            if index % 2 == 0 {
                prose.push_str(part);
            }
        }
        let mut rest = prose.as_str();
        while let Some(start) = rest.find("](") {
            let target = &rest[start + 2..];
            let end = target.find([')', ' ']).unwrap_or(target.len());
            targets.push(target[..end].to_owned());
            rest = &target[end..];
        }
        let trimmed = prose.trim_start();
        if let (true, Some(close)) = (trimmed.starts_with('['), trimmed.find("]: ")) {
            let target = trimmed[close + 3..].split_whitespace().next().unwrap_or("");
            targets.push(target.to_owned());
        }
    }
    targets
}

fn normalize(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normal.pop();
            }
            Component::CurDir => {}
            other => normal.push(other),
        }
    }
    normal
}

/// A page's Docusaurus id: its path below `website/docs`, without the extension.
fn doc_id(docs: &Path, page: &Path) -> String {
    page.strip_prefix(docs)
        .expect("a page under website/docs")
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/")
}

/// The single-quoted literals of `sidebars.ts` that name a page: array elements and `id:` values.
fn sidebar_ids(sidebar: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = sidebar;
    let mut before = String::new();
    while let Some(start) = rest.find('\'') {
        before.push_str(&rest[..start]);
        let value = &rest[start + 1..];
        let end = value.find('\'').expect("a literal closes");
        let context = before.trim_end();
        if context.ends_with('[') || context.ends_with(',') || context.ends_with("id:") {
            ids.push(value[..end].to_owned());
        }
        before.clear();
        rest = &value[end + 1..];
    }
    ids
}

/// The `label:` values of `sidebars.ts`, in order.
fn sidebar_labels(sidebar: &str) -> Vec<String> {
    sidebar
        .split("label: '")
        .skip(1)
        .map(|part| part[..part.find('\'').expect("a label closes")].to_owned())
        .collect()
}

#[test]
fn the_sidebar_has_the_adopter_categories_in_order_collapsed() {
    let sidebar = read(&root().join("website/sidebars.ts"));
    assert_eq!(
        sidebar_labels(&sidebar),
        [
            "Start here",
            "Runners",
            "Concepts",
            "Guides",
            "Write a specification",
            "Verify conformance",
            "Reference",
            "Examples",
            "Releases",
            "Release posts",
            "Status",
        ],
        "website/sidebars.ts categories"
    );
    assert!(
        !sidebar.contains("collapsed: false"),
        "every sidebar category starts collapsed"
    );

    let reference = &sidebar[sidebar
        .find("label: 'Reference'")
        .expect("a Reference category")..];
    let reference = &reference[..reference
        .find("label: 'Examples'")
        .expect("Examples follows")];
    assert_eq!(
        sidebar_ids(reference),
        [
            "reference/cli",
            "reference/diagnostics",
            "reference/formats",
            "reference/ess-ui",
            "reference/spec-versions",
            "reference/predicates",
            "reference/glossary",
        ],
        "the Reference category"
    );
}

#[test]
fn every_page_is_in_the_sidebar_and_every_entry_is_a_page() {
    let root = root();
    let docs = root.join("website/docs");
    let sidebar = read(&root.join("website/sidebars.ts"));
    let listed: BTreeSet<String> = sidebar_ids(&sidebar).into_iter().collect();
    let published: BTreeSet<String> = pages(&docs)
        .iter()
        .map(|page| doc_id(&docs, page))
        .collect();

    let unlisted: Vec<_> = published.difference(&listed).collect();
    assert!(
        unlisted.is_empty(),
        "pages no sidebar entry reaches: {unlisted:?}"
    );
    let missing: Vec<_> = listed.difference(&published).collect();
    assert!(
        missing.is_empty(),
        "sidebar entries with no page: {missing:?}"
    );
}

#[test]
fn a_split_guide_page_is_at_most_400_lines() {
    let docs = root().join("website/docs");
    let mut split = Vec::new();
    for directory in ["guides/specify", "guides/verify"] {
        let pages = if docs.join(directory).is_dir() {
            pages(&docs.join(directory))
        } else {
            Vec::new()
        };
        assert!(
            pages.len() >= 5,
            "{directory} holds the split task pages, found {}",
            pages.len()
        );
        split.extend(pages);
    }
    split.push(docs.join("guides/write-a-specification.md"));
    split.push(docs.join("guides/verify-conformance.md"));
    let long: Vec<_> = split
        .iter()
        .map(|page| (doc_id(&docs, page), read(page).lines().count()))
        .filter(|&(_, lines)| lines > 400)
        .collect();
    assert!(long.is_empty(), "guide pages over 400 lines: {long:?}");
}

#[test]
fn every_relative_link_lands_on_a_page_and_an_anchor() {
    let root = root();
    let mut broken = Vec::new();
    let mut checked = 0;
    for page in pages(&root.join("website/docs"))
        .into_iter()
        .chain(pages(&root.join("website/blog")))
    {
        let text = read(&page);
        for target in link_targets(&text) {
            // `[marker]: #` is the comment form generated sections are bracketed with.
            if target == "#"
                || target.contains("://")
                || target.starts_with('/')
                || target.starts_with("mailto:")
            {
                continue;
            }
            let (path, anchor) = match target.split_once('#') {
                Some((path, anchor)) => (path, Some(anchor)),
                None => (target.as_str(), None),
            };
            let destination = if path.is_empty() {
                page.clone()
            } else if Path::new(path)
                .extension()
                .is_some_and(|extension| extension == "md" || extension == "mdx")
            {
                normalize(&page.parent().expect("a page has a directory").join(path))
            } else {
                continue;
            };
            checked += 1;
            let shown = page
                .strip_prefix(&root)
                .unwrap_or(&page)
                .display()
                .to_string();
            if !destination.is_file() {
                broken.push(format!("{shown}: {target} (no such page)"));
                continue;
            }
            if let Some(anchor) = anchor {
                if !anchors(&read(&destination)).contains(anchor) {
                    broken.push(format!("{shown}: {target} (no such anchor)"));
                }
            }
        }
    }
    assert!(
        checked >= 50,
        "the scan read {checked} links; it is not reading the pages"
    );
    assert!(
        broken.is_empty(),
        "{} broken links:\n{}",
        broken.len(),
        broken.join("\n")
    );
}

/// The anchors `guides/write-a-specification` published before it was split into task pages.
const SPECIFICATION_ANCHORS: &[&str] = &[
    "layout",
    "keep-sources-and-generated-output-together",
    "name-the-ess-release-the-specification-is-maintained-with",
    "validate-early-read-the-refusals",
    "what-the-model-insists-on",
    "a-command-that-can-be-refused-says-so",
    "an-invariant-reads-only-what-every-creation-sets",
    "cover-every-declared-enum-value",
    "order-two-instants",
    "say-which-characters-a-text-may-hold-and-how-long-it-may-be",
    "say-what-a-text-starts-with",
    "carry-any-json-value",
    "carry-finite-binary-floating-point-values",
    "select-an-outcome-from-the-held-subject-state",
    "guard-an-outcome-by-the-subjects-stored-fields",
    "an-outcome-the-input-cannot-decide-says-that-too",
    "one-outcome-for-many-commands",
    "illegal-lifecycle-moves-are-illegal-by-absence",
    "a-command-says-what-it-answers-when-invoked-in-the-wrong-state",
    "an-unknown-instance-can-have-its-own-outcome",
    "a-request-with-no-input-can-have-its-own-outcome",
    "an-outcome-can-be-selected-by-whether-the-record-exists",
    "an-outcome-can-change-every-record-a-filter-selects",
    "an-outcome-can-delete-its-subject",
    "a-creation-can-land-in-a-declared-state",
    "an-accepted-request-can-change-nothing",
    "a-system-can-run-inside-ambient-preconditions",
    "an-events-values-need-a-declared-source",
    "value-expressions",
    "read-the-callers-credential",
    "an-input-refused-when-absent-is-present-afterwards",
    "a-view-declares-its-consistency",
    "a-view-can-be-paged",
    "aggregate-views",
    "a-binding-says-what-happens-when-it-fails",
    "bound-a-retry",
    "read-a-field-inside-an-event-envelope",
    "select-ordered-records-in-a-binding",
    "declare-a-periodic-host-cause",
    "read-the-channel-an-event-arrived-on",
    "preserve-clock-reading-provenance",
    "crossing-contexts-takes-a-declared-conversion",
    "an-enum-variant-can-carry-its-own-wire-spelling",
    "a-field-can-carry-its-own-wire-name",
    "say-whether-an-absent-optional-is-sent-as-null",
    "three-layers-above-the-domains",
    "check-what-you-just-wrote-resolved",
    "names",
    "next",
];

/// The anchors `guides/verify-conformance` published before it was split into task pages.
const CONFORMANCE_ANCHORS: &[&str] = &[
    "generate-the-suite",
    "select-authored-scenarios-explicitly",
    "expect-an-external-branch-in-an-authored-scenario",
    "establish-backend-state-in-an-authored-scenario",
    "name-several-instances-in-one-input",
    "observe-outcomes-selected-by-held-state",
    "observe-retries-of-the-original-result",
    "observe-selection-periodic-activity-and-clock-evidence",
    "deliver-an-event-with-its-context",
    "run-a-supported-target",
    "hold-your-own-implementation-to-the-suite",
    "audit-the-suite-with-specification-mutants",
    "explore-random-command-sequences",
    "check-a-concurrent-history",
    "draw-a-history-as-client-lanes",
    "import-a-recorded-log",
    "opt-into-explicit-outcome-counts",
    "where-passed-failed-and-skipped-live",
    "opt-into-declared-coverage",
    "observe-bounded-binding-accessors",
    "what-the-report-proves",
    "a-target-in-rust",
];

#[test]
fn every_anchor_the_long_guides_published_still_lands_on_their_old_page() {
    let docs = root().join("website/docs");
    for (page, published) in [
        ("guides/write-a-specification.md", SPECIFICATION_ANCHORS),
        ("guides/verify-conformance.md", CONFORMANCE_ANCHORS),
    ] {
        let offered = anchors(&read(&docs.join(page)));
        let lost: Vec<_> = published
            .iter()
            .filter(|anchor| !offered.contains(**anchor))
            .collect();
        assert!(lost.is_empty(), "{page} no longer offers {lost:?}");
    }
}

#[test]
fn the_glossary_defines_30_to_60_terms_each_linking_its_page() {
    let glossary = root().join("website/docs/reference/glossary.md");
    assert!(glossary.is_file(), "reference/glossary.md exists");
    let text = read(&glossary);
    let rows: Vec<&str> = prose_lines(&text)
        .into_iter()
        .filter(|line| line.starts_with('|'))
        .skip(2)
        .collect();
    assert!(
        (30..=60).contains(&rows.len()),
        "the glossary defines {} terms",
        rows.len()
    );
    let unlinked: Vec<_> = rows.iter().filter(|row| !row.contains("](")).collect();
    assert!(
        unlinked.is_empty(),
        "terms with no defining page: {unlinked:?}"
    );
}

/// A site build stops on front matter that is not YAML — an unquoted `: ` inside a description is
/// the usual cause — so every page's front matter must parse as a mapping with a title.
#[test]
fn every_page_has_front_matter_that_parses() {
    let root = root();
    let mut refused = Vec::new();
    for page in pages(&root.join("website/docs")) {
        let text = read(&page);
        let shown = page
            .strip_prefix(&root)
            .unwrap_or(&page)
            .display()
            .to_string();
        let Some(body) = text.strip_prefix("---\n") else {
            refused.push(format!("{shown}: no front matter"));
            continue;
        };
        let Some(end) = body.find("\n---\n") else {
            refused.push(format!("{shown}: front matter does not close"));
            continue;
        };
        match serde_yaml::from_str::<serde_yaml::Mapping>(&body[..end]) {
            Ok(mapping) if mapping.contains_key("title") => {}
            Ok(_) => refused.push(format!("{shown}: front matter has no title")),
            Err(error) => refused.push(format!("{shown}: {error}")),
        }
    }
    assert!(refused.is_empty(), "{}", refused.join("\n"));
}

#[test]
fn the_slug_matches_what_docusaurus_renders() {
    assert_eq!(
        slug("Validate early, read the refusals"),
        "validate-early-read-the-refusals"
    );
    assert_eq!(
        slug("Name the `ess` release the specification is maintained with"),
        "name-the-ess-release-the-specification-is-maintained-with"
    );
    assert_eq!(
        slug("Read the caller's credential"),
        "read-the-callers-credential"
    );
    assert_eq!(slug("A [linked](x.md) heading"), "a-linked-heading");
    let page =
        "# Title\n\n## Next\n\n## Next\n\n```yaml\n## not a heading\n```\n<a id=\"old\"></a>\n";
    assert_eq!(
        anchors(page),
        ["next", "next-1", "old", "title"]
            .map(str::to_owned)
            .into_iter()
            .collect()
    );
}
