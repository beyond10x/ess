//! What the published documents claim about releases, checked against the source that ships them.
//!
//! The support matrix in `website/docs/status/where-this-stands.md` is generated and has been
//! current at every release; every sentence a person wrote beside it had drifted five releases by
//! 2026-09-20 — `ess/2` was still called unreleased two weeks after 0.20.0 shipped it, the install
//! walkthrough still downloaded 0.13.2, and `ess/5` was supported by the build while appearing in
//! no reference page. The difference between the two is that one had a checker. This is that
//! checker for the rest.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

/// The published document tree.
const DOCS: &str = "website/docs";

/// The pages that are allowed to be the only mention of a format version.
///
/// A supported version has to be written down somewhere a reader can find it. Either page counts:
/// the format table says what a version admits, the history page says which release introduced it,
/// and a version named by neither is one the build accepts and nobody documents.
const REFERENCE: &[&str] = &[
    "website/docs/reference/formats.md",
    "website/docs/reference/spec-versions.md",
];

/// Where the release notes live.
const BLOG: &str = "website/blog";

/// How many minor releases the newest release note may trail the newest release by.
///
/// Not one per release: a release whose only content is a fix has nothing to write about, and a
/// gate that fires on one becomes a gate people silence. Three is the point at which the notes
/// have stopped being a record of the project and started being a record of one month of it — the
/// state they were in on 2026-09-21, nineteen minors behind, which nothing noticed.
const BLOG_LAG: u64 = 3;

/// The pages whose version literals must name the newest release.
///
/// These are the pages a newcomer follows, so every version in them is an instruction to download
/// that version. A historical version number belongs on the history page, not here. The list is
/// the walkthrough `crates/edge/ess-cli/tests/tutorial_page.rs` runs, in the same order.
const INSTALL: &[&str] = &[
    "website/docs/start/install.md",
    "website/docs/start/first-specification.md",
    "website/docs/start/first-conformance-run.md",
    "website/docs/start/runners/typescript.md",
    "website/docs/start/runners/go.md",
    "website/docs/start/runners/rust.md",
    "website/docs/start/explore-the-example.md",
];

/// The repository front page, which a reader lands on before any site page.
///
/// Unlike [`INSTALL`], it may name a historical release in prose (`relations shipped in 0.5.0`),
/// so only its install instructions are held to the newest release: see [`readme_defects`].
const README: &str = "README.md";

/// The page that records which release introduced each format version.
const HISTORY: &str = "website/docs/reference/spec-versions.md";

/// Where each format family keeps the list of versions this build admits.
///
/// Read from source rather than repeated here, so growing a constant is enough to make
/// [`FORMAT_RELEASES`] incomplete and this lane refuse.
const SUPPORTED: &[(&str, &str, &str)] = &[
    (
        "ess",
        "crates/specify/ess-domain/src/system.rs",
        "SUPPORTED_FORMATS",
    ),
    (
        "ess-diff",
        "crates/verify/ess-diff/src/delta.rs",
        "SUPPORTED_DELTA_FORMATS",
    ),
    (
        "ess-conformance",
        "crates/verify/ess-conformance/src/scenario.rs",
        "SUPPORTED_SUITE_FORMATS",
    ),
    (
        "ess-composition",
        "crates/specify/ess-composition/src/lib.rs",
        "SUPPORTED_COMPOSITION_FORMATS",
    ),
];

/// Every supported format version and the release that first shipped it.
///
/// Each row was read from the commit that added the version to its `SUPPORTED_*` constant, and
/// then from the earliest version tag containing that commit:
///
/// | commit | constants it moved | earliest tag |
/// |---|---|---|
/// | `73c55bfd` | `ess/1` | 0.1.0 |
/// | `86e071ae` | `ess-diff/1` | 0.1.0 |
/// | `695090ee` | `ess-conformance/1` | 0.1.0 |
/// | `6dfbd733` | `ess-conformance/2` | 0.7.0 |
/// | `11fc6694` | `ess-conformance/3` | 0.16.0 |
/// | `a85adfe3` | `ess-conformance/4` | 0.18.0 |
/// | `25580795` | `ess-diff/2` | 0.19.0 |
/// | `bf16e504` | `ess/2` | 0.20.0 |
/// | `874962d3` | `ess-conformance/5` | 0.21.0 |
/// | `9572af9b` | `ess/3`, `ess/4`, `ess-diff/3`, `ess-diff/4`, `ess-conformance/6`–`/9` | 0.23.0 |
/// | `9746c09f` | `ess/5`, `ess-diff/5` | 0.27.0 |
///
/// `ess-composition` gained `SUPPORTED_COMPOSITION_FORMATS` with `/2`; its `/1` row is read the way
/// the rows below are, `/2` shipped in 0.38.0 and `/3` is unreleased.
///
/// The families after `ess-composition` have no `SUPPORTED_*` constant to read. Their rows are the
/// ones the version history gives a release (`HISTORY`), each read as the earliest version tag
/// whose non-test Rust source carries the quoted `"family/N"` literal
/// (`git grep -F '"family/N"' <tag> -- '*.rs' ':!*/tests/*'`, tags in version order). A version
/// no tag's source names — `ess-impact/1`, `ess-conformance-run/1` — has no row.
///
/// The families after `ess-mutation-manifest` are every other family a published page names, plus
/// the `ess-execution-*` files `ess deployment reconcile --authority` reads and writes
/// (`crates/edge/ess-cli/src/recovery/model.rs`). Each row is the commit that first added the
/// quoted literal to non-test Rust source (`git log --reverse -S'"family/N"' -- '*.rs'`) and the
/// earliest version tag containing it, cross-checked against the earliest tag whose source
/// carries it; the two agree for every row. Three first appear under the `v0.3.0` tag —
/// `ess-browser-catalog/1`, `ess-client-plan/1` and `ess-composition/1` — and are recorded as
/// 0.4.0, the first published release carrying them, as the version history's own table of
/// unpublished versions says. `ess-types-report` starts at `/3`: no tag's source names `/1` or `/2`.
///
/// Not tracked, on purpose: the `ess-consumer-*` records, `ess-feature-preservation/1` and
/// `ess-infra-acceptance-receipt/1`. `cargo xtask` writes them for this repository's own
/// consumer-coverage and infrastructure-acceptance lanes; the `ess` binary neither reads nor
/// writes them, no adopter holds one, and no published page names one. A page that starts naming
/// one makes it public, and [`untracked_named`] then refuses until it has a row here.
///
/// A version that is supported in this checkout and not yet in any release carries `None`, and is
/// the one case a document may still call unreleased.
const FORMAT_RELEASES: &[(&str, u32, Option<&str>)] = &[
    ("ess", 1, Some("0.1.0")),
    ("ess", 2, Some("0.20.0")),
    ("ess", 3, Some("0.23.0")),
    ("ess", 4, Some("0.23.0")),
    ("ess", 5, Some("0.27.0")),
    ("ess", 6, Some("0.28.0")),
    ("ess", 7, Some("0.29.0")),
    ("ess", 8, Some("0.34.0")),
    ("ess", 9, Some("0.34.0")),
    ("ess", 10, Some("0.34.0")),
    ("ess", 11, Some("0.34.0")),
    ("ess", 12, Some("0.34.0")),
    ("ess", 13, Some("0.35.0")),
    ("ess", 14, Some("0.36.0")),
    ("ess", 15, Some("0.37.0")),
    ("ess", 16, Some("0.38.0")),
    ("ess", 17, Some("0.39.0")),
    ("ess", 18, Some("0.41.0")),
    ("ess", 19, Some("0.46.0")),
    ("ess-diff", 1, Some("0.1.0")),
    ("ess-diff", 2, Some("0.19.0")),
    ("ess-diff", 3, Some("0.23.0")),
    ("ess-diff", 4, Some("0.23.0")),
    ("ess-diff", 5, Some("0.27.0")),
    ("ess-diff", 6, Some("0.29.0")),
    ("ess-diff", 7, Some("0.34.0")),
    ("ess-diff", 8, Some("0.34.0")),
    ("ess-diff", 9, Some("0.38.0")),
    ("ess-diff", 10, Some("0.41.0")),
    ("ess-diff", 11, Some("0.42.0")),
    ("ess-diff", 12, Some("0.46.1")),
    ("ess-conformance", 1, Some("0.1.0")),
    ("ess-conformance", 2, Some("0.7.0")),
    ("ess-conformance", 3, Some("0.16.0")),
    ("ess-conformance", 4, Some("0.18.0")),
    ("ess-conformance", 5, Some("0.21.0")),
    ("ess-conformance", 6, Some("0.23.0")),
    ("ess-conformance", 7, Some("0.23.0")),
    ("ess-conformance", 8, Some("0.23.0")),
    ("ess-conformance", 9, Some("0.23.0")),
    ("ess-conformance", 10, Some("0.28.0")),
    ("ess-conformance", 11, Some("0.28.0")),
    ("ess-conformance", 12, Some("0.29.0")),
    ("ess-conformance", 13, Some("0.29.0")),
    ("ess-conformance", 14, Some("0.34.0")),
    ("ess-conformance", 15, Some("0.34.0")),
    ("ess-conformance", 16, Some("0.34.0")),
    ("ess-conformance", 17, Some("0.34.0")),
    ("ess-conformance", 18, Some("0.35.0")),
    ("ess-conformance", 19, Some("0.35.0")),
    ("ess-conformance", 20, Some("0.37.0")),
    ("ess-conformance", 21, Some("0.37.0")),
    ("ess-conformance", 22, Some("0.37.0")),
    ("ess-conformance", 23, Some("0.37.0")),
    ("ess-conformance", 24, Some("0.37.0")),
    ("ess-conformance", 25, Some("0.37.0")),
    ("ess-conformance", 26, Some("0.38.0")),
    ("ess-conformance", 27, Some("0.38.0")),
    ("ess-conformance", 28, Some("0.39.0")),
    ("ess-conformance", 29, Some("0.39.0")),
    ("ess-conformance", 30, Some("0.41.0")),
    ("ess-conformance", 31, Some("0.41.0")),
    ("ess-conformance", 32, Some("0.43.0")),
    ("ess-conformance", 33, Some("0.43.0")),
    ("ess-composition", 1, Some("0.4.0")),
    ("ess-composition", 2, Some("0.38.0")),
    ("ess-composition", 3, Some("0.40.0")),
    ("ess-scenario", 1, Some("0.16.0")),
    ("ess-scenario", 2, Some("0.23.0")),
    ("ess-scenario", 3, Some("0.35.0")),
    ("ess-scenario", 4, Some("0.39.0")),
    ("ess-normalization", 1, Some("0.19.0")),
    ("ess-normalization", 2, Some("0.20.0")),
    ("ess-normalization", 3, Some("0.20.0")),
    ("ess-normalization", 4, Some("0.20.0")),
    ("ess-normalization", 5, Some("0.20.0")),
    ("ess-normalization", 6, Some("0.20.0")),
    ("ess-conformance-report", 1, Some("0.1.0")),
    ("ess-conformance-report", 2, Some("0.19.0")),
    ("ess-schema-bundle", 1, Some("0.19.0")),
    ("ess-schema-bundle", 2, Some("0.19.0")),
    ("ess-impact", 2, Some("0.1.0")),
    ("ess-impact", 3, Some("0.19.0")),
    ("ess-conformance-run", 2, Some("0.20.0")),
    ("ess-conformance-results", 1, None),
    ("ess-target-failure", 1, Some("0.19.0")),
    ("ess-target-failure", 2, Some("0.20.0")),
    ("ess-target-failure", 3, Some("0.23.0")),
    ("infra-observation", 1, Some("0.1.0")),
    ("infra-observation", 2, Some("0.1.0")),
    ("infra-observation", 3, Some("0.33.0")),
    ("infra-ir", 1, Some("0.1.0")),
    ("infra-ir", 2, Some("0.21.0")),
    ("infra-ir", 3, Some("0.33.0")),
    ("infra-drift", 1, Some("0.1.0")),
    ("infra-drift", 2, Some("0.21.0")),
    ("infra-drift", 3, Some("0.33.0")),
    ("ess-observed-bindings-report", 1, Some("0.21.0")),
    ("ess-observed-bindings-report", 2, Some("0.32.0")),
    ("ess-observed-bindings-report", 3, Some("0.33.0")),
    ("ess-observed-bindings", 1, Some("0.21.0")),
    ("ess-observed-bindings", 2, Some("0.33.0")),
    ("ess-history", 1, Some("0.39.0")),
    ("ess-history-adapter", 1, Some("0.39.0")),
    ("ess-mutation-report", 1, Some("0.34.0")),
    ("ess-mutation-report", 2, Some("0.41.0")),
    ("ess-mutation-report", 3, Some("0.42.0")),
    ("ess-mutation-manifest", 1, Some("0.37.0")),
    ("ess-mutation-manifest", 2, Some("0.41.0")),
    ("ess-mutation-manifest", 3, Some("0.42.0")),
    ("ess-service-interface", 1, Some("0.1.0")),
    ("infra-spec", 1, Some("0.1.0")),
    ("infra-graph", 1, Some("0.1.0")),
    ("infra-graph", 2, Some("0.21.0")),
    ("infra-simulation", 1, Some("0.1.0")),
    ("infra-simulation", 2, Some("0.21.0")),
    ("infra-projection", 1, Some("0.1.0")),
    ("ess-browser-catalog", 1, Some("0.4.0")),
    ("ess-client-plan", 1, Some("0.4.0")),
    ("ess-docs", 1, Some("0.4.0")),
    ("ess-realization", 1, Some("0.8.0")),
    ("ess-realization", 2, Some("0.21.0")),
    ("ess-realization-ir", 1, Some("0.8.0")),
    ("ess-realization-ir", 2, Some("0.21.0")),
    ("ess-build", 1, Some("0.9.0")),
    ("ess-build-ir", 1, Some("0.9.0")),
    ("ess-runtime", 1, Some("0.9.0")),
    ("ess-runtime-ir", 1, Some("0.9.0")),
    ("ess-release", 1, Some("0.9.0")),
    ("ess-release-catalog", 1, Some("0.9.0")),
    ("ess-stack", 1, Some("0.9.0")),
    ("ess-stack-lock", 1, Some("0.9.0")),
    ("ess-environment", 1, Some("0.9.0")),
    ("ess-deployment", 1, Some("0.9.0")),
    ("ess-deployment-diff", 1, Some("0.9.0")),
    ("ess-component", 1, Some("0.13.0")),
    ("ess-component-ir", 1, Some("0.13.0")),
    ("ess-release-bundle", 1, Some("0.13.0")),
    ("ess-types-report", 3, Some("0.19.0")),
    ("ess-normalization-target", 1, Some("0.19.0")),
    ("ess-normalization-target", 2, Some("0.20.0")),
    ("ess-normalization-target", 3, Some("0.20.0")),
    ("ess-openapi-import", 1, Some("0.20.0")),
    ("ess-openapi-service-subset", 1, Some("0.20.0")),
    ("ess-conformance-input", 1, Some("0.21.0")),
    ("ess-conformance-replay", 1, Some("0.21.0")),
    ("ess-inputs", 1, Some("0.21.0")),
    ("ess-inputs", 2, Some("0.34.0")),
    ("ess-output-state", 1, Some("0.21.0")),
    ("ess-output-state", 2, Some("0.34.0")),
    ("ess-cli", 1, Some("0.21.0")),
    ("ess-cli-plan", 1, Some("0.21.0")),
    ("ess-cli-artifacts", 1, Some("0.21.0")),
    ("ess-cli-generation", 1, Some("0.21.0")),
    ("ess-execution-authority", 1, Some("0.21.0")),
    ("ess-execution-registry", 1, Some("0.21.0")),
    ("ess-execution-lock", 1, Some("0.21.0")),
    ("ess-execution-store", 1, Some("0.21.0")),
    ("ess-execution-evidence", 1, Some("0.21.0")),
];

/// Checks the published documents against the source and the changelog.
///
/// Every defect class is collected before anything is reported, so one run names everything that
/// has to be repaired rather than the first thing it met.
pub fn run(root: &Path) -> Result<String, String> {
    let supported = supported_versions(root)?;
    let released: BTreeMap<(&str, u32), &str> = FORMAT_RELEASES
        .iter()
        .filter_map(|&(family, version, release)| release.map(|value| ((family, version), value)))
        .collect();
    let reference = read_reference(root)?;
    let (undeclared, undocumented) = recorded(&supported, &reference);

    let untracked = untracked_in(root)?;

    let documents = read_documents(root)?;
    let tracked: BTreeSet<&str> = FORMAT_RELEASES
        .iter()
        .map(|&(family, _, _)| family)
        .collect();
    let unnamed = untracked_named(&documents, &tracked);

    let mut stale = Vec::new();
    for (path, text) in &documents {
        stale.extend(stale_claims(path, text, &released));
    }

    let changelog = fs::read_to_string(root.join("CHANGELOG.md"))
        .map_err(|error| format!("read CHANGELOG.md: {error}"))?;
    let newest = newest_release(&changelog)?;
    let mut pinned = Vec::new();
    for page in INSTALL {
        let install =
            fs::read_to_string(root.join(page)).map_err(|error| format!("read {page}: {error}"))?;
        pinned.extend(install_defects(page, &install, &newest));
    }

    let notes = newest_note(root)?;
    let trailing = match (
        minor(&newest),
        notes.as_ref().and_then(|(_, tag)| minor(tag)),
    ) {
        (Some(release), Some(note)) => release.saturating_sub(note),
        _ => 0,
    };

    let mut refusals = Vec::new();
    match &notes {
        None => refusals.push(format!(
            "no file in {BLOG} declares a `release_tag`, so how far the release notes trail cannot \
             be read"
        )),
        Some((path, tag)) if trailing > BLOG_LAG => refusals.push(format!(
            "the newest release note is {path}, for {tag}; the newest release is {newest}, \
             {trailing} minors later, and {BLOG_LAG} is the most this lane admits"
        )),
        Some(_) => {}
    }

    if !undeclared.is_empty() {
        refusals.push(format!(
            "supported format versions with no release recorded in FORMAT_RELEASES: {}",
            undeclared.join(", ")
        ));
    }
    if !untracked.is_empty() {
        refusals.push(format!(
            "format families {HISTORY} gives a release and FORMAT_RELEASES does not track: {}",
            untracked.join(", ")
        ));
    }
    if !unnamed.is_empty() {
        refusals.push(format!(
            "format families named under {DOCS} that FORMAT_RELEASES does not track:\n{}",
            unnamed.join("\n")
        ));
    }
    if !undocumented.is_empty() {
        refusals.push(format!(
            "tracked or supported format versions named in no reference page: {}",
            undocumented.join(", ")
        ));
    }
    if !stale.is_empty() {
        stale.sort();
        stale.dedup();
        refusals.push(format!(
            "released formats still called unreleased:\n{}",
            stale.join("\n")
        ));
    }
    if !pinned.is_empty() {
        refusals.push(format!(
            "the install walkthrough names a version that is not the newest release:\n{}",
            pinned.join("\n")
        ));
    }
    refusals.extend(readme_refusal(root, &newest)?);
    if !refusals.is_empty() {
        return Err(refusals.join("\n\n"));
    }

    let count: usize = supported.values().map(Vec::len).sum();
    let rows = FORMAT_RELEASES.len();
    let families = tracked.len();
    Ok(format!(
        "{count} supported format versions, each with a recorded release; {rows} tracked versions \
         of {families} families, each named in a reference page, every family a page names among \
         them, none called unreleased; the install walkthrough names {newest} and {README} installs no other; the newest release \
         note trails it by {trailing}\n"
    ))
}

/// Reads each `SUPPORTED_*` constant out of the source that defines it.
fn supported_versions(root: &Path) -> Result<BTreeMap<String, Vec<u32>>, String> {
    let mut versions = BTreeMap::new();
    for &(family, path, constant) in SUPPORTED {
        let text =
            fs::read_to_string(root.join(path)).map_err(|error| format!("read {path}: {error}"))?;
        versions.insert(family.to_owned(), constant_list(&text, path, constant)?);
    }
    Ok(versions)
}

/// The numbers one `pub const NAME: &[u32] = &[…];` declares.
///
/// Read up to the `=` and then past whatever whitespace `rustfmt` puts before the list once it no
/// longer fits on the declaration's line — which a family's list does as it grows, as
/// `SUPPORTED_SUITE_FORMATS` did at `ess-conformance/17`.
fn constant_list(text: &str, path: &str, constant: &str) -> Result<Vec<u32>, String> {
    let needle = format!("pub const {constant}: &[u32] =");
    let declared = text
        .find(&needle)
        .ok_or_else(|| format!("{path} has no {constant}"))?
        + needle.len();
    let list = text[declared..]
        .trim_start()
        .strip_prefix("&[")
        .ok_or_else(|| format!("{constant} in {path} is not one bracketed list"))?;
    let end = list
        .find(']')
        .ok_or_else(|| format!("{constant} in {path} is not one bracketed list"))?;
    list[..end]
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| format!("{constant} in {path} holds `{value}`"))
        })
        .collect()
}

/// The supported versions with no release recorded, and every version no reference page names.
fn recorded(
    supported: &BTreeMap<String, Vec<u32>>,
    reference: &[String],
) -> (Vec<String>, Vec<String>) {
    let declared: BTreeSet<(&str, u32)> = FORMAT_RELEASES
        .iter()
        .map(|&(family, version, _)| (family, version))
        .collect();
    let mut undeclared = Vec::new();
    let mut undocumented = unreferenced(reference);
    for (family, versions) in supported {
        for &version in versions {
            if !declared.contains(&(family.as_str(), version)) {
                undeclared.push(format!("{family}/{version}"));
            } else if !reference.iter().any(|page| names(page, family, version)) {
                undocumented.push(format!("{family}/{version}"));
            }
        }
    }
    undocumented.sort();
    undocumented.dedup();
    (undeclared, undocumented)
}

/// Every [`FORMAT_RELEASES`] version that no reference page names.
///
/// A tracked family is one a reader may meet, so each version needs a line saying what it is,
/// whether or not a `SUPPORTED_*` constant lists it.
fn unreferenced(reference: &[String]) -> Vec<String> {
    FORMAT_RELEASES
        .iter()
        .filter(|&&(family, version, _)| !reference.iter().any(|page| names(page, family, version)))
        .map(|&(family, version, _)| format!("{family}/{version}"))
        .collect()
}

/// Reads the reference pages a format version may be documented in.
fn read_reference(root: &Path) -> Result<Vec<String>, String> {
    REFERENCE
        .iter()
        .map(|path| {
            fs::read_to_string(root.join(path)).map_err(|error| format!("read {path}: {error}"))
        })
        .collect()
}

/// Reads every published document, deepest path last, with its repository-relative path.
fn read_documents(root: &Path) -> Result<Vec<(String, String)>, String> {
    let mut documents = Vec::new();
    let mut directories = vec![root.join(DOCS)];
    while let Some(directory) = directories.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("read {}: {error}", directory.display()))?;
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "md") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|_| format!("{} is outside the workspace", path.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            let text =
                fs::read_to_string(&path).map_err(|error| format!("read {relative}: {error}"))?;
            documents.push((relative, text));
        }
    }
    documents.sort();
    Ok(documents)
}

/// The newest dated release the changelog records.
fn newest_release(changelog: &str) -> Result<String, String> {
    changelog
        .lines()
        .filter_map(|line| line.strip_prefix("## ["))
        .filter(|line| line.contains('—'))
        .filter_map(|line| line.split(']').next())
        .find(|version| !versions_in(version).is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "CHANGELOG.md has no dated release heading".to_owned())
}

/// Every phrasing `website/docs` has used to call a format unreleased.
///
/// Read from the history of the tree (`git log -p -- website/docs`), not guessed: `ess/9`–`/13`
/// were "not yet released", the revised-envelope table wrote "next release" in its release
/// column, and a traversal step was "not yet shipped". Matched case-insensitively.
const UNRELEASED: &[&str] = &[
    "unreleased",
    "not yet released",
    "not released",
    "next release",
    "upcoming",
    "not yet shipped",
];

/// Every line of one document that calls a released format unreleased.
fn stale_claims(path: &str, text: &str, released: &BTreeMap<(&str, u32), &str>) -> Vec<String> {
    let families: BTreeSet<&str> = released.keys().map(|&(family, _)| family).collect();
    let mut claims = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        if !UNRELEASED.iter().any(|phrase| lower.contains(phrase)) {
            continue;
        }
        for (family, version) in formats_on_line(line, &families) {
            if let Some(release) = released.get(&(family, version)) {
                claims.push(format!(
                    "{path}:{}: {family}/{version} shipped in {release}",
                    number + 1
                ));
            }
        }
    }
    claims
}

/// Every `family/version` one line names, including a bare `/N` after a family.
///
/// The revised-envelope table names its family once, as `infra-ir/`, and its versions bare, as
/// `/3`; prose writes `ess-conformance/18` and `/19`. A bare version belongs to the family
/// most recently named before it on the line, and is only read where no token precedes the
/// slash, so `tag/0.35.0` and `docs/2` are not versions.
fn formats_on_line<'a>(line: &str, families: &BTreeSet<&'a str>) -> BTreeSet<(&'a str, u32)> {
    let bytes = line.as_bytes();
    let mut found = BTreeSet::new();
    let mut current: Option<&str> = None;
    for start in 0..bytes.len() {
        if !line.is_char_boundary(start) || (start > 0 && is_token(bytes[start - 1])) {
            continue;
        }
        let rest = &line[start..];
        let (family, digits) = if let Some(after) = rest.strip_prefix('/') {
            (current, after)
        } else if let Some(&family) = families
            .iter()
            .filter(|family| rest.starts_with(&format!("{family}/")))
            .max_by_key(|family| family.len())
        {
            current = Some(family);
            (current, &rest[family.len() + 1..])
        } else {
            continue;
        };
        let end = digits
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(digits.len());
        let mut after = digits[end..].bytes();
        let closes = match after.next() {
            None => true,
            Some(b'.') => !after.next().is_some_and(|byte| byte.is_ascii_digit()),
            Some(byte) => !is_token(byte),
        };
        if let (Some(family), Ok(version), true) = (family, digits[..end].parse::<u32>(), closes) {
            found.insert((family, version));
        }
    }
    found
}

/// The families the committed version history gives a release and [`FORMAT_RELEASES`] does not track.
fn untracked_in(root: &Path) -> Result<Vec<String>, String> {
    let history = fs::read_to_string(root.join(HISTORY))
        .map_err(|error| format!("read {HISTORY}: {error}"))?;
    let tracked: BTreeSet<&str> = FORMAT_RELEASES
        .iter()
        .map(|&(family, _, _)| family)
        .collect();
    Ok(untracked_families(&history, &tracked))
}

/// Every format family the version history gives a release that [`FORMAT_RELEASES`] does not track.
///
/// A family is taken from the first backticked `family/…` token of every line that carries a
/// bracketed release, which is how both the version tables and the prose paragraphs write one.
/// A family this lane does not track can be called unreleased for ever, as `ess-scenario` was.
fn untracked_families(page: &str, tracked: &BTreeSet<&str>) -> Vec<String> {
    let mut untracked = BTreeSet::new();
    for line in page.lines() {
        if !versions_in(line)
            .iter()
            .any(|version| line.contains(&format!("[{version}]")))
        {
            continue;
        }
        let family = line.split('`').skip(1).step_by(2).find_map(|span| {
            let (family, version) = span.split_once('/')?;
            let named = !family.is_empty()
                && family
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                && version.bytes().all(|byte| byte.is_ascii_digit());
            named.then_some(family)
        });
        if let Some(family) = family.filter(|family| !tracked.contains(family)) {
            untracked.insert(family.to_owned());
        }
    }
    untracked.into_iter().collect()
}

/// Families a published page may name without [`FORMAT_RELEASES`] tracking them, each with why.
///
/// A family on this list is one no build reads or writes, so there is no release to record.
const UNVERSIONED: &[(&str, &str)] = &[(
    "ess-ir",
    "the compiled IR carries no format header; the roadmap names `ess-ir/2` only as a version \
     it will not take without a persisted compatibility reason",
)];

/// Whether a name is one of the format families ESS mints.
///
/// Every document format the `ess` binary reads or writes is named `ess`, `ess-…` or `infra-…`.
/// Digest profiles (`sha256-json-bytes/1`, `slice-sha256/2`) and prose shorthand (`report/2`,
/// `suite/4`) are not families and are not read.
fn is_family(name: &str) -> bool {
    name == "ess" || name.starts_with("ess-") || name.starts_with("infra-")
}

/// Every format family a published page names that [`FORMAT_RELEASES`] does not track.
///
/// A family is any `family/N` token on a page — backticked or not — whose name [`is_family`]
/// admits. Each untracked family is reported once, at the first page and line naming it, so the
/// refusal says where to look. A family this lane does not track can be called unreleased for ever,
/// as `ess-scenario` was; [`untracked_families`] covers only the history page, and this every page.
fn untracked_named(documents: &[(String, String)], tracked: &BTreeSet<&str>) -> Vec<String> {
    let mut first: BTreeMap<String, String> = BTreeMap::new();
    for (path, text) in documents {
        for (number, line) in text.lines().enumerate() {
            for family in families_on_line(line) {
                if tracked.contains(family)
                    || UNVERSIONED.iter().any(|&(excused, _)| excused == family)
                {
                    continue;
                }
                first
                    .entry(family.to_owned())
                    .or_insert_with(|| format!("{path}:{}", number + 1));
            }
        }
    }
    first
        .into_iter()
        .map(|(family, at)| format!("{family} ({at})"))
        .collect()
}

/// Every format family one line names as `family/N`.
///
/// A name starts where no token character precedes it, so `crates/generate/ess-deployment/src` and
/// `tag/0.35.0` are not read, and the version must end there, so `ess/1.2` is not one either.
fn families_on_line(line: &str) -> BTreeSet<&str> {
    let bytes = line.as_bytes();
    let mut found = BTreeSet::new();
    let mut start = 0;
    while start < bytes.len() {
        let is_name = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-';
        if !bytes[start].is_ascii_lowercase() || (start > 0 && is_token(bytes[start - 1])) {
            start += 1;
            continue;
        }
        let mut end = start;
        while end < bytes.len() && is_name(bytes[end]) {
            end += 1;
        }
        let name = &line[start..end];
        let digits = bytes[end..].strip_prefix(b"/").map_or(0, |rest| {
            rest.iter().take_while(|byte| byte.is_ascii_digit()).count()
        });
        if digits > 0 && is_family(name) {
            let after = end + 1 + digits;
            let closes = match bytes.get(after) {
                None => true,
                Some(b'.') => !bytes.get(after + 1).is_some_and(u8::is_ascii_digit),
                Some(&byte) => !is_token(byte),
            };
            if closes {
                found.insert(name);
            }
        }
        start = end.max(start + 1);
    }
    found
}

/// Every line of the install walkthrough that names a version other than the newest release.
fn install_defects(path: &str, text: &str, newest: &str) -> Vec<String> {
    let mut defects = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for version in versions_in(line) {
            if version != newest {
                defects.push(format!(
                    "{path}:{}: names {version}, newest release is {newest}",
                    number + 1
                ));
            }
        }
    }
    defects
}

/// The refusal for a README that installs anything but the newest release, if it does.
fn readme_refusal(root: &Path, newest: &str) -> Result<Option<String>, String> {
    let text =
        fs::read_to_string(root.join(README)).map_err(|error| format!("read {README}: {error}"))?;
    let defects = readme_defects(README, &text, newest);
    Ok((!defects.is_empty()).then(|| {
        format!(
            "{README} installs a version that is not the newest release:\n{}",
            defects.join("\n")
        )
    }))
}

/// Every README install instruction that names a version other than the newest release.
///
/// An install instruction is a line that says `install`, assigns `version=`, or downloads from
/// `releases/download/`. Path segments are read one at a time, so the version inside a download
/// URL counts, while a historical release in prose and a chart's `--version 1.0.0` do not.
fn readme_defects(path: &str, text: &str, newest: &str) -> Vec<String> {
    let mut defects = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        let instructs = lower.contains("install")
            || lower.contains("version=")
            || lower.contains("releases/download/");
        if !instructs {
            continue;
        }
        for version in line.split('/').flat_map(versions_in) {
            if version != newest {
                defects.push(format!(
                    "{path}:{}: names {version} as the release to install, newest release is \
                     {newest}",
                    number + 1
                ));
            }
        }
    }
    defects
}

/// The newest release note, by the release its front matter declares.
///
/// `release_tag` carries a plain version on a current note and a historical wave spelling on an
/// older one (`0.3.0-ess-wave-1`), so the leading three numbers are read and the rest ignored.
fn newest_note(root: &Path) -> Result<Option<(String, String)>, String> {
    let directory = root.join(BLOG);
    let entries = fs::read_dir(&directory).map_err(|error| format!("read {BLOG}: {error}"))?;
    let mut newest: Option<(u64, String, String)> = None;
    for entry in entries {
        let entry = entry.map_err(|error| format!("read {BLOG}: {error}"))?;
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "md") {
            continue;
        }
        let name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text =
            fs::read_to_string(&path).map_err(|error| format!("read {BLOG}/{name}: {error}"))?;
        let Some(tag) = text
            .lines()
            .skip(1)
            .take_while(|line| line.trim_end() != "---")
            .find_map(|line| line.trim().strip_prefix("release_tag:"))
            .map(|value| value.trim().trim_matches('"').to_owned())
        else {
            continue;
        };
        let Some(ordinal) = minor(&tag) else { continue };
        if newest.as_ref().is_none_or(|(held, _, _)| ordinal > *held) {
            newest = Some((ordinal, format!("{BLOG}/{name}"), tag));
        }
    }
    Ok(newest.map(|(_, path, tag)| (path, tag)))
}

/// A version's major and minor as one increasing number, ignoring any suffix.
fn minor(version: &str) -> Option<u64> {
    let mut parts = version.split('.');
    let major: u64 = parts.next()?.parse().ok()?;
    let rest = parts.next()?;
    let digits = rest
        .find(|character: char| !character.is_ascii_digit())
        .map_or(rest, |end| &rest[..end]);
    Some(major * 1000 + digits.parse::<u64>().ok()?)
}

/// Every `1.2.3` in one line.
///
/// A run of digits and dots is taken whole, so `0.13.2.1` and `v0.3.0-rc1` are not mistaken for a
/// three-number version that happens to start them.
fn versions_in(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
            index += 1;
        }
        let opens = start == 0 || !is_token(bytes[start - 1]);
        let closes = index == bytes.len() || !is_token(bytes[index]);
        let run = &line[start..index];
        let parts: Vec<&str> = run.split('.').collect();
        if opens
            && closes
            && parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        {
            found.push(run.to_owned());
        }
    }
    found
}

/// Whether a byte continues a version-like token.
fn is_token(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' || byte == b'/'
}

/// Whether one line names exactly `family/version`.
///
/// `ess/1` must not match inside `ess-diff/1`, and `ess-conformance/1` must not match inside a
/// hypothetical `ess-conformance/10`, so both edges are checked rather than the substring alone.
fn names(text: &str, family: &str, version: u32) -> bool {
    let token = format!("{family}/{version}");
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(offset) = text[from..].find(&token) {
        let start = from + offset;
        let end = start + token.len();
        let before = start == 0 || !is_token(bytes[start - 1]);
        let after = end == bytes.len() || !bytes[end].is_ascii_digit();
        if before && after {
            return true;
        }
        from = start + 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn released() -> BTreeMap<(&'static str, u32), &'static str> {
        FORMAT_RELEASES
            .iter()
            .filter_map(|&(family, version, release)| {
                release.map(|value| ((family, version), value))
            })
            .collect()
    }

    #[test]
    fn a_shipped_format_still_called_unreleased_is_refused() {
        let claims = stale_claims(
            "website/docs/guides/write-a-specification.md",
            "prose\nThe unreleased `ess/3` format adds bounded binding accessors.\n",
            &released(),
        );
        assert_eq!(
            claims,
            vec![
                "website/docs/guides/write-a-specification.md:2: ess/3 shipped in 0.23.0"
                    .to_owned()
            ]
        );
    }

    #[test]
    fn every_phrasing_the_docs_have_used_for_an_unreleased_format_is_read() {
        // Each line is a phrasing that has stood in website/docs beside a format that had
        // already shipped; the history of the tree names them, not a guess about English.
        for line in [
            "`ess/13`, not yet released, admits `fixture_inputs:` on a command",
            "`ess/13` (not yet released) declares that",
            "`ess/13` (unreleased) adds command",
            "| `ess/13` | next release | Fixture inputs. |",
            "`ess/13` is designed and **not yet shipped**.",
            "`ess/13` returns it in the upcoming pre-1.0 minor release.",
        ] {
            let claims = stale_claims("page.md", line, &released());
            assert_eq!(
                claims,
                vec!["page.md:1: ess/13 shipped in 0.35.0".to_owned()],
                "{line}"
            );
        }
    }

    #[test]
    fn a_bare_version_belongs_to_the_family_named_before_it_on_the_line() {
        let row = "| `ess-scenario/` | [0.23.0][r23], next release | `/2` authored setup. `/3` adds typed `fixtures:`. |";
        assert_eq!(
            stale_claims("page.md", row, &released()),
            vec![
                "page.md:1: ess-scenario/2 shipped in 0.23.0".to_owned(),
                "page.md:1: ess-scenario/3 shipped in 0.35.0".to_owned(),
            ]
        );
        let prose = "`ess-conformance/18` and `/19`, not yet released, carry fixture values.";
        assert_eq!(
            stale_claims("page.md", prose, &released()),
            vec![
                "page.md:1: ess-conformance/18 shipped in 0.35.0".to_owned(),
                "page.md:1: ess-conformance/19 shipped in 0.35.0".to_owned(),
            ]
        );
        // Published prose is not ASCII; a curly quote beside a format is still read.
        assert_eq!(
            stale_claims(
                "page.md",
                "the checkout’s `ess/13` is unreleased",
                &released()
            ),
            vec!["page.md:1: ess/13 shipped in 0.35.0".to_owned()]
        );
        // A path or a link is not a bare version.
        assert!(
            stale_claims("page.md", "unreleased tag/0.35.0 and docs/2", &released()).is_empty()
        );
    }

    #[test]
    fn a_family_the_version_history_gives_a_release_and_this_lane_does_not_track_is_refused() {
        let tracked = BTreeSet::from(["ess"]);
        let page = "| `ess/1` | [0.1.0][r1] | first |
| `ess-widget/` | [0.2.0][r2] | `/2` more |
`ess-gadget/3`, introduced in [0.3.0][r3], admits.
`ess-draft/1`, next release.
";
        assert_eq!(
            untracked_families(page, &tracked),
            vec!["ess-gadget".to_owned(), "ess-widget".to_owned()]
        );
    }

    #[test]
    fn every_family_the_committed_version_history_gives_a_release_is_tracked() {
        let root = crate::workspace_root().expect("workspace root");
        let page = fs::read_to_string(root.join(HISTORY)).expect("version history");
        let tracked: BTreeSet<&str> = FORMAT_RELEASES
            .iter()
            .map(|&(family, _, _)| family)
            .collect();
        assert_eq!(untracked_families(&page, &tracked), Vec::<String>::new());
    }

    #[test]
    fn a_family_a_page_names_and_this_lane_does_not_track_is_refused() {
        let tracked = BTreeSet::from(["ess", "ess-diff", "infra-ir"]);
        let documents = vec![
            (
                "website/docs/a.md".to_owned(),
                "Write `ess/18`; the delta is `ess-diff/11`.\n\
                 The planted `ess-widget/2` and `infra-gadget/1`, and `format: ess-widget/3`.\n"
                    .to_owned(),
            ),
            (
                "website/docs/b.md".to_owned(),
                "A path `crates/generate/ess-deployment/src` and `ess-inputs.yaml` name no version.\n\
                 A digest profile `sha256-json-bytes/1`, a shorthand report/2 and `tag/0.35.0`.\n\
                 `infra-ir/3` is tracked; `ess-ir/2` is excused.\n"
                    .to_owned(),
            ),
        ];
        assert_eq!(
            untracked_named(&documents, &tracked),
            vec![
                "ess-widget (website/docs/a.md:2)".to_owned(),
                "infra-gadget (website/docs/a.md:2)".to_owned(),
            ]
        );
    }

    #[test]
    fn every_family_a_published_page_names_is_tracked() {
        let root = crate::workspace_root().expect("workspace root");
        let documents = read_documents(&root).expect("published documents");
        let tracked: BTreeSet<&str> = FORMAT_RELEASES
            .iter()
            .map(|&(family, _, _)| family)
            .collect();
        assert_eq!(untracked_named(&documents, &tracked), Vec::<String>::new());
    }

    #[test]
    fn every_tracked_format_version_is_named_in_a_reference_page() {
        let root = crate::workspace_root().expect("workspace root");
        let reference = read_reference(&root).expect("reference pages");
        assert_eq!(unreferenced(&reference), Vec::<String>::new());
    }

    #[test]
    fn a_tracked_version_no_reference_page_names_is_reported() {
        let pages = vec!["`ess/1` and `ess-diff/10`".to_owned()];
        let missing = unreferenced(&pages);
        assert!(missing.contains(&"ess-diff/1".to_owned()), "{missing:?}");
        assert!(missing.contains(&"ess-execution-lock/1".to_owned()));
        assert!(!missing.contains(&"ess/1".to_owned()));
        assert!(!missing.contains(&"ess-diff/10".to_owned()));
    }

    #[test]
    fn a_format_version_no_release_ships_may_still_be_called_unreleased() {
        // A fixed release inventory keeps this pre-release example valid when the real
        // format registry gains another published version.
        let released = BTreeMap::from([(("ess", 1), "0.1.0")]);
        assert!(stale_claims("page.md", "The unreleased `ess/2` format.\n", &released).is_empty());
    }

    #[test]
    fn one_family_does_not_match_another_whose_name_ends_in_it() {
        assert!(names("admits `ess/1` documents", "ess", 1));
        assert!(!names("admits `ess-diff/1` documents", "ess", 1));
        assert!(!names("suite `ess-conformance/10`", "ess-conformance", 1));
        assert!(names("suite `ess-conformance/1`.", "ess-conformance", 1));
    }

    #[test]
    fn an_install_walkthrough_pinned_to_an_older_release_is_refused() {
        let defects = install_defects(
            INSTALL[0],
            "downloads the latest published release, `0.13.2`,\n$ version=0.27.0\n",
            "0.27.0",
        );
        assert_eq!(
            defects,
            vec![format!(
                "{}:1: names 0.13.2, newest release is 0.27.0",
                INSTALL[0]
            )]
        );
    }

    #[test]
    fn a_readme_that_installs_an_older_release_is_refused() {
        let text = "## Install\n\nInstall the current release, 0.42.0:\n\n```console\n\
                    version=0.41.0\n\
                    curl -LO https://github.com/beyond10x/ess/releases/download/0.40.0/SHA256SUMS\n\
                    ```\n";
        assert_eq!(
            readme_defects(README, text, "0.43.0"),
            vec![
                format!(
                    "{README}:3: names 0.42.0 as the release to install, newest release is 0.43.0"
                ),
                format!(
                    "{README}:6: names 0.41.0 as the release to install, newest release is 0.43.0"
                ),
                format!(
                    "{README}:7: names 0.40.0 as the release to install, newest release is 0.43.0"
                ),
            ]
        );
    }

    #[test]
    fn a_readme_may_name_a_historical_release_outside_its_install_instructions() {
        let text = "Entity relations shipped in `0.5.0`.\n\
                    ```console\nversion=0.43.0\n\
                    ess generate project helm --chart example --version 1.0.0\n```\n";
        assert!(readme_defects(README, text, "0.43.0").is_empty());
    }

    #[test]
    fn the_committed_readme_installs_the_newest_release() {
        let root = crate::workspace_root().expect("workspace root");
        let changelog = fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");
        let newest = newest_release(&changelog).expect("newest release");
        let readme = fs::read_to_string(root.join(README)).expect("readme");
        assert_eq!(
            readme_defects(README, &readme, &newest),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_version_run_is_read_whole() {
        assert_eq!(versions_in("ess 0.27.0"), vec!["0.27.0".to_owned()]);
        assert!(versions_in("ess-0.27.0-aarch64").is_empty());
        assert!(versions_in("0.13.2.1").is_empty());
        assert!(versions_in("draft 2020-12").is_empty());
    }

    #[test]
    fn the_newest_dated_heading_is_the_newest_release() {
        let changelog = "# Changelog\n\n## [Unreleased]\n\n## [0.27.0] — 2026-09-20\n\n## [0.26.1] — 2026-09-17\n";
        assert_eq!(newest_release(changelog).as_deref(), Ok("0.27.0"));
        assert!(newest_release("## [0.27.0]\n").is_err());
    }

    #[test]
    fn a_wave_spelling_and_a_plain_version_both_order_by_minor() {
        assert_eq!(minor("0.27.0"), Some(27));
        assert_eq!(minor("0.3.0-ess-wave-1"), Some(3));
        assert_eq!(minor("0.7.1-infra-waves-1-4"), Some(7));
        assert!(minor("0.27.0").unwrap() > minor("0.9.2").unwrap());
        assert!(minor("1.0.0").unwrap() > minor("0.99.0").unwrap());
        assert_eq!(minor("v0.3.0"), None);
        assert_eq!(minor("0"), None);
    }

    #[test]
    fn the_committed_release_notes_do_not_trail_the_newest_release() {
        let root = crate::workspace_root().expect("workspace root");
        let changelog = fs::read_to_string(root.join("CHANGELOG.md")).expect("changelog");
        let newest = newest_release(&changelog).expect("newest release");
        let (path, tag) = newest_note(&root)
            .expect("reads the release notes")
            .expect("a release note declares a release_tag");
        let trailing = minor(&newest).expect("release minor") - minor(&tag).expect("note minor");
        assert!(
            trailing <= BLOG_LAG,
            "{path} is for {tag}, {trailing} minors behind {newest}"
        );
    }

    #[test]
    fn a_supported_list_is_read_whether_or_not_rustfmt_wrapped_it() {
        let one_line = "pub const X: &[u32] = &[1, 2, 3];\n";
        let wrapped = "pub const X: &[u32] =\n    &[1, 2, 3, 16, 17];\n";
        let exploded = "pub const X: &[u32] = &[\n    1, 2,\n    3,\n];\n";
        assert_eq!(constant_list(one_line, "a.rs", "X"), Ok(vec![1, 2, 3]));
        assert_eq!(
            constant_list(wrapped, "a.rs", "X"),
            Ok(vec![1, 2, 3, 16, 17])
        );
        assert_eq!(constant_list(exploded, "a.rs", "X"), Ok(vec![1, 2, 3]));
        assert!(constant_list("pub const X: &[u32] = Y;\n", "a.rs", "X").is_err());
        assert!(constant_list("pub const Z: &[u32] = &[1];\n", "a.rs", "X").is_err());
    }

    #[test]
    fn every_recorded_release_is_a_version_and_every_supported_version_is_recorded() {
        let root = crate::workspace_root().expect("workspace root");
        let supported = supported_versions(&root).expect("reads the SUPPORTED_ constants");
        for (family, versions) in &supported {
            for version in versions {
                assert!(
                    FORMAT_RELEASES
                        .iter()
                        .any(|&(name, number, _)| name == family && number == *version),
                    "{family}/{version} is supported and has no recorded release"
                );
            }
        }
        for &(family, version, release) in FORMAT_RELEASES {
            if let Some(release) = release {
                assert_eq!(
                    versions_in(release).len(),
                    1,
                    "{family}/{version} records `{release}`"
                );
            }
        }
    }
}
