//! Adversary case (beyond10x/ess#285, pass 1): every single-file ESS model in the repository and
//! every `examples/<name>` model keeps its canonical IR, its synthesized suite and its synthesis
//! refusals byte for byte against the integration head 637c8be3a (the base of #285 was 26ad39057; #273 later changed event expectations, so suite digests were re-pinned at 637c8be3a and checked identical with and without #285).
//!
//! `fixtures/adversary-285-base-digests.tsv` holds, per model, the SHA-256 and length of
//! `EssIr::to_canonical_json()`, of `ConformanceSuite::to_canonical_json()` and of the refusals
//! joined by newlines, read from a build of an export of 26ad39057 with this file's walk.
//!
//! `arrangement-input-refusal.yaml`'s suite digest is re-pinned for beyond10x/ess#455 (32afe8ef4):
//! its three overlapping input refusals now give the earlier one a witness outside the later
//! guards and send each overlap separately, requiring the first declared. Its IR and refusal
//! digests are unchanged, and the old suite digest is what the released 0.53.0 synthesizes.
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use sha2::{Digest, Sha256};

const BASE: &str = include_str!("fixtures/adversary-285-base-digests.tsv");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn digest(bytes: &str) -> String {
    let hex = Sha256::digest(bytes.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    format!("{hex}:{}", bytes.len())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "yaml") {
            out.push(path);
        }
    }
}

fn line(parsed: Vec<(Source, RawSpecFile)>, sources: &SourceMap) -> Result<String, String> {
    let spec = Specification::assemble(parsed).map_err(|errors| errors.to_string())?;
    let ir = compile(&spec, sources).map_err(|error| format!("{error:?}"))?;
    let canonical = ir.to_canonical_json();
    let suite = ess_conformance::synthesize::synthesize(&ir);
    let refusals: Vec<String> = suite.refusals.iter().map(ToString::to_string).collect();
    let suite = suite
        .suite
        .to_canonical_json()
        .map_err(|error| format!("{error:?}"))?;
    Ok(format!(
        "{}\t{}\t{}",
        digest(&canonical),
        digest(&suite),
        digest(&refusals.join("\n"))
    ))
}

/// The digests of the model `label` names at this tree.
fn digests(root: &Path, label: &str) -> Result<String, String> {
    if let Some(example) = label.strip_prefix("examples-dir:") {
        let dir = root.join("examples").join(example);
        let mut files = Vec::new();
        walk(&dir, &mut files);
        files.sort();
        let mut sources = SourceMap::new();
        let mut parsed = Vec::new();
        for path in files {
            let label = path.strip_prefix(&dir).unwrap().display().to_string();
            let text = std::fs::read_to_string(&path).unwrap();
            let raw = RawSpecFile::parse(&text).map_err(|error| error.to_string())?;
            sources.insert(label.clone(), text);
            parsed.push((Source::new(label), raw));
        }
        return line(parsed, &sources);
    }
    let text = std::fs::read_to_string(root.join(label)).map_err(|error| error.to_string())?;
    let raw = RawSpecFile::parse(&text).map_err(|error| error.to_string())?;
    let mut sources = SourceMap::new();
    sources.insert(label.to_owned(), text);
    line(vec![(Source::new(label.to_owned()), raw)], &sources)
}

#[test]
fn adversary_285_every_committed_model_keeps_its_ir_suite_and_refusal_bytes() {
    let root = root();
    let mut moved = Vec::new();
    let mut compared = 0;
    for pinned in BASE.lines().filter(|line| !line.is_empty()) {
        let (label, base) = pinned.split_once('\t').expect("label, then digests");
        compared += 1;
        match digests(&root, label) {
            Ok(here) if here == base => {}
            Ok(here) => moved.push(format!("{label}\n  base {base}\n  here {here}")),
            Err(error) => moved.push(format!("{label}: no longer compiles: {error}")),
        }
    }
    assert_eq!(compared, 88, "every pinned model is compared");
    assert_eq!(moved, Vec::<String>::new(), "bytes moved against the pinned digests (re-pinned at integration head 637c8be3a, after #273 changed event expectations)");
}
