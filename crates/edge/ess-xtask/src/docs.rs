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

/// The page whose version literals must name the newest release.
///
/// This is the page a newcomer follows, so every version in it is an instruction to download that
/// version. A historical version number belongs on the history page, not here.
const INSTALL: &str = "website/docs/getting-started.md";

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
    ("ess-diff", 1, Some("0.1.0")),
    ("ess-diff", 2, Some("0.19.0")),
    ("ess-diff", 3, Some("0.23.0")),
    ("ess-diff", 4, Some("0.23.0")),
    ("ess-diff", 5, Some("0.27.0")),
    ("ess-diff", 6, Some("0.29.0")),
    ("ess-diff", 7, Some("0.34.0")),
    ("ess-diff", 8, Some("0.34.0")),
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
    let declared: BTreeSet<(&str, u32)> = FORMAT_RELEASES
        .iter()
        .map(|&(family, version, _)| (family, version))
        .collect();

    let mut undeclared = Vec::new();
    let mut undocumented = Vec::new();
    let reference = read_reference(root)?;
    for (family, versions) in &supported {
        for &version in versions {
            let key = (family.as_str(), version);
            if !declared.contains(&key) {
                undeclared.push(format!("{family}/{version}"));
                continue;
            }
            if !reference.iter().any(|page| names(page, family, version)) {
                undocumented.push(format!("{family}/{version}"));
            }
        }
    }

    let mut stale = Vec::new();
    for (path, text) in read_documents(root)? {
        stale.extend(stale_claims(&path, &text, &released));
    }

    let changelog = fs::read_to_string(root.join("CHANGELOG.md"))
        .map_err(|error| format!("read CHANGELOG.md: {error}"))?;
    let newest = newest_release(&changelog)?;
    let install = fs::read_to_string(root.join(INSTALL))
        .map_err(|error| format!("read {INSTALL}: {error}"))?;
    let pinned = install_defects(INSTALL, &install, &newest);

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
    if !undocumented.is_empty() {
        refusals.push(format!(
            "supported format versions named in no reference page: {}",
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
    if !refusals.is_empty() {
        return Err(refusals.join("\n\n"));
    }

    let count: usize = supported.values().map(Vec::len).sum();
    Ok(format!(
        "{count} supported format versions, each with a recorded release, each named in a \
         reference page, none called unreleased; the install walkthrough names {newest}; the \
         newest release note trails it by {trailing}\n"
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

/// Every line of one document that calls a released format unreleased.
fn stale_claims(path: &str, text: &str, released: &BTreeMap<(&str, u32), &str>) -> Vec<String> {
    let mut claims = Vec::new();
    for (number, line) in text.lines().enumerate() {
        if !line.to_lowercase().contains("unreleased") {
            continue;
        }
        for (&(family, version), release) in released {
            if names(line, family, version) {
                claims.push(format!(
                    "{path}:{}: {family}/{version} shipped in {release}",
                    number + 1
                ));
            }
        }
    }
    claims
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
            INSTALL,
            "downloads the latest published release, `0.13.2`,\n$ version=0.27.0\n",
            "0.27.0",
        );
        assert_eq!(
            defects,
            vec![format!(
                "{INSTALL}:1: names 0.13.2, newest release is 0.27.0"
            )]
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
