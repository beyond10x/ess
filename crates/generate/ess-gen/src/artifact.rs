//! What a generator produces, and the one trait every generator implements.

use std::collections::BTreeMap;

use ess_compiler::EssIr;

use crate::provenance::{ModelSlice, Provenance, ProvenanceMint};

/// One generated file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Artifact {
    /// Where it goes, relative to the output root. Always `/`-separated.
    pub path: String,
    /// Its contents.
    pub contents: String,
    /// The model slice it derives from.
    ///
    /// [`ModelSlice::WholeModel`] unless the generator narrowed it, and that default is the
    /// polarity of the whole wave: an artifact whose slice nobody thought about is owed
    /// regeneration whenever anything moves, never quietly current.
    pub slice: ModelSlice,
}

impl Artifact {
    /// Checks this artifact's destination using [`validate_path`].
    ///
    /// Constructors and fields remain available for in-memory producers. A consumer of an
    /// untrusted artifact must validate the complete destination set with [`validate_paths`]
    /// before collecting it into a map or writing any file.
    pub fn validate(&self) -> Result<(), String> {
        validate_path(&self.path)
    }

    /// Builds one that derives from the whole model.
    pub fn new(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            contents: contents.into(),
            slice: ModelSlice::WholeModel,
        }
    }

    /// Builds one that derives from a narrower slice.
    ///
    /// The slice comes attached to the provenance that was stamped into `contents` —
    /// [`ProvenanceMint::of_seeds`] hands both out as one value — so the recorded slice and the
    /// stamped digest cannot be paired up wrong by a generator, and [`run`] still checks.
    pub fn sliced(path: impl Into<String>, contents: impl Into<String>, slice: ModelSlice) -> Self {
        Self {
            path: path.into(),
            contents: contents.into(),
            slice,
        }
    }
}

/// Checks a canonical, portable path relative to an output root.
///
/// Components contain ASCII letters, digits, `.`, `_` and `-`, with no empty, `.` or `..`
/// components, trailing dots, or Windows device names (including names with extensions).
/// `/` is the only separator. Absolute paths, drive/UNC prefixes, alternate data streams,
/// backslashes, control characters and non-ASCII aliases are therefore refused rather than
/// normalized. These are destination admission rules; they do not rewrite artifact bytes.
///
/// This is lexical validation only. Filesystem writers must additionally inspect existing roots,
/// ancestors and destinations for aliases, symlinks and hardlinks before writing the entire set.
pub fn validate_path(path: &str) -> Result<(), String> {
    for component in path.split('/') {
        let stem = component.split('.').next().unwrap_or_default();
        let upper = stem.to_ascii_uppercase();
        let device = matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ["COM", "LPT"].iter().any(|prefix| {
                upper.strip_prefix(prefix).is_some_and(|suffix| {
                    suffix.len() == 1 && matches!(suffix.as_bytes()[0], b'1'..=b'9')
                })
            });
        if component.is_empty()
            || matches!(component, "." | "..")
            || component.ends_with('.')
            || device
            || !component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(format!("invalid portable output path `{path}`"));
        }
    }
    Ok(())
}

/// Checks every destination before any collection can discard a duplicate.
///
/// Comparison folds ASCII case, including directory names: `A/one` and `a/two` are refused as
/// incompatible spellings of one directory. Files cannot also be parents of other files. Valid
/// paths retain their original spelling and order; no filesystem is accessed by this check.
pub fn validate_paths<'a>(paths: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
    let mut entries: BTreeMap<String, (&str, bool)> = BTreeMap::new();
    for path in paths {
        validate_path(path)?;
        for (end, _) in path
            .match_indices('/')
            .chain(std::iter::once((path.len(), "")))
        {
            let prefix = &path[..end];
            let file = end == path.len();
            let folded = prefix.to_ascii_lowercase();
            if let Some((previous, previous_file)) = entries.get(&folded) {
                if *previous != prefix || file || *previous_file {
                    return Err(format!("colliding output paths `{previous}` and `{path}`"));
                }
            } else {
                entries.insert(folded, (prefix, file));
            }
        }
    }
    Ok(())
}

/// A projection of the model.
///
/// One trait rather than one crate per projection (review F9): what differs between `OpenAPI` and
/// Markdown is the body, not how it is invoked, and eleven crates cost review attention every time
/// someone opens the tree.
pub trait Generator {
    /// What this projection is called on the command line — `docs`, `openapi`.
    fn name(&self) -> &'static str;

    /// One line for `--help` and for the generated index.
    fn describes(&self) -> &'static str;

    /// The subdirectory its artifacts go in, relative to the output root.
    fn directory(&self) -> &'static str;

    /// Generates every artifact, in a stable order.
    ///
    /// Infallible on purpose. A generator reaching a construct it cannot project is a gap in this
    /// crate, not a fault in the specification — and the specification has already been refused if it
    /// was wrong, because this takes an [`EssIr`] and there is no way to hold one that did not
    /// resolve. So there is nothing left for a `Result` to report.
    ///
    /// The mint, not a `Provenance`: since wave 7 each artifact carries the digest of the model
    /// slice it derives from, and a single pre-computed provenance could only say "the whole
    /// model" about every file. A generator that has nothing narrower to say still stamps
    /// [`ProvenanceMint::whole`].
    fn generate(&self, ir: &EssIr, mint: &ProvenanceMint) -> Vec<Artifact>;
}

/// Runs one generator and returns its artifacts keyed by path.
///
/// Keyed, and therefore deduplicated: two artifacts claiming one path means the second silently
/// overwrites the first, and the output tree looks complete while missing a file.
/// # Panics
///
/// When an artifact's stamped contract digest disagrees with the digest its recorded slice
/// computes, or when an artifact carries no readable provenance at all. Both are defects in a
/// generator, not in any specification — the same class as two artifacts claiming one path, but
/// worse, because a wrong stamp *ships*: a committed artifact claiming derivation from a slice it
/// was not stamped for is exactly the false claim wave 7's drift check exists to refuse, and this
/// is the one place every artifact of every generator passes through before it can be written.
pub fn run(
    generator: &dyn Generator,
    ir: &EssIr,
) -> Result<BTreeMap<String, Artifact>, DuplicatePath> {
    let mint = ProvenanceMint::new(ir);
    let mut out: BTreeMap<String, Artifact> = BTreeMap::new();
    for artifact in generator.generate(ir, &mint) {
        let stamped = Provenance::read_digests(&artifact.contents).unwrap_or_else(|| {
            panic!(
                "the `{}` generator wrote `{}` without readable provenance; an artifact that \
                 cannot say what it derives from is an artifact nobody can audit",
                generator.name(),
                artifact.path
            )
        });
        let computed = mint.digest_of(&artifact.slice);
        assert_eq!(
            stamped.contract_digest,
            computed,
            "the `{}` generator stamped `{}` with a contract digest its recorded slice does not \
             compute; the stamp and the slice must come from one `ProvenanceMint` call",
            generator.name(),
            artifact.path
        );
        let path = format!("{}/{}", generator.directory(), artifact.path);
        if out.contains_key(&path) {
            return Err(DuplicatePath {
                generator: generator.name(),
                path,
            });
        }
        out.insert(
            path.clone(),
            Artifact::sliced(path, artifact.contents, artifact.slice),
        );
    }
    Ok(out)
}

/// Two artifacts claimed one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicatePath {
    /// Which generator.
    pub generator: &'static str,
    /// The path claimed twice.
    pub path: String,
}

impl std::fmt::Display for DuplicatePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the `{}` generator produced two artifacts at `{}`; the second would overwrite the \
             first and the output would look complete",
            self.generator, self.path
        )
    }
}

impl std::error::Error for DuplicatePath {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_artifacts_refuse_escape_and_platform_aliases() {
        for path in [
            "",
            ".",
            "..",
            "../file",
            "/file",
            "a//b",
            "a/./b",
            "a/../b",
            "a/",
            "C:/file",
            "C:file",
            "\\\\server\\share",
            "a\\b",
            "a:b",
            "a\0b",
            "a\nb",
            "NUL.txt",
            "com1.txt",
            "LPT9/a",
            "a.",
            "a ",
            "caf\u{e9}",
            "a%2fb",
        ] {
            assert!(
                Artifact::new(path, "contents").validate().is_err(),
                "{path:?}"
            );
        }
        for path in [
            "index.html",
            ".well-known/schema.json",
            "plan/Board_2.md",
            "assets/mermaid.LICENSE",
            "CONTRACT",
            "COM10.txt",
        ] {
            assert!(Artifact::new(path, "contents").validate().is_ok(), "{path}");
        }
    }

    #[test]
    fn a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order() {
        for pair in [
            ["a", "a"],
            ["a", "A"],
            ["a", "a/b"],
            ["dir/a", "DIR/b"],
            ["a.html", "a.html/child.html"],
        ] {
            assert!(validate_paths(pair).is_err(), "{pair:?}");
            assert!(validate_paths(pair.into_iter().rev()).is_err(), "{pair:?}");
        }
        assert!(validate_paths(["plan.html", "plan/board.html", "plan/another.html"]).is_ok());
    }
}
