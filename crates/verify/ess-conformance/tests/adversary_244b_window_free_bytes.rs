//! Adversary case, beyond10x/ess#244 part b, pass 1: every ESS model in the repository that carries
//! no calendar window keeps, byte for byte against the unit's base `3b1684d1a`, its canonical IR,
//! its synthesized suite, its synthesis refusals, its mutant list (ids and changes) and, where it
//! assembles at all, the same refusal text — as written, and relabelled `format: ess/22`.
//!
//! `fixtures/adversary-244b-base-digests.tsv` was written by this same file run with
//! `ADV244B_WRITE=<path>` in an export of `3b1684d1a` (`git archive`), built in its own target
//! directory.

// Style lints only, so the probe stays byte for byte what wrote the base digests.
#![allow(clippy::needless_pass_by_value, clippy::format_collect)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::mutate::{self, MutantClass};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn digest(bytes: &str) -> String {
    let hex =
        Sha256::digest(bytes.as_bytes())
            .iter()
            .take(10)
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
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if !["target", "node_modules", ".git", ".engineering"].contains(&name.as_str()) {
                walk(&path, out);
            }
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
}

fn line(documents: Vec<(Source, RawSpecFile)>, sources: &SourceMap) -> String {
    let spec = match Specification::assemble(documents.clone()) {
        Ok(spec) => spec,
        Err(errors) => return format!("ASSEMBLE {}", digest(&errors.to_string())),
    };
    let ir = match compile(&spec, sources) {
        Ok(ir) => ir,
        Err(error) => return format!("COMPILE {}", digest(&format!("{error:?}"))),
    };
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    let suite = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| format!("SUITE-ERROR {error:?}"));
    let mutants: Vec<String> = mutate::mutants(&documents, MutantClass::ALL)
        .iter()
        .map(|mutant| format!("{}\t{}", mutant.id, mutant.change))
        .collect();
    format!(
        "ir={} suite={} refusals={} mutants={}",
        digest(&ir.to_canonical_json()),
        digest(&suite),
        digest(&refusals.join("\n")),
        digest(&mutants.join("\n"))
    )
}

fn single(label: &str, text: &str) -> String {
    match RawSpecFile::parse(text) {
        Ok(raw) => {
            let mut sources = SourceMap::new();
            sources.insert(label.to_owned(), text.to_owned());
            line(vec![(Source::new(label.to_owned()), raw)], &sources)
        }
        Err(error) => format!("PARSE {}", digest(&error.to_string())),
    }
}

/// Every model this tree holds: each ESS source file alone, as written and relabelled `ess/22`,
/// and each `examples/<name>` directory as one system.
fn digests(root: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut found = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        let label = path.strip_prefix(root).unwrap().display().to_string();
        found.push((label.clone(), single(&label, &text)));
        let relabelled: String = text
            .lines()
            .map(|line| {
                if line.starts_with("format: ess/") {
                    "format: ess/22".to_owned()
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        if relabelled.trim_end() != text.trim_end() {
            found.push((format!("{label}@ess22"), single(&label, &relabelled)));
        }
    }
    let examples = root.join("examples");
    if let Ok(entries) = std::fs::read_dir(&examples) {
        let mut dirs: Vec<PathBuf> = entries
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
        dirs.sort();
        for dir in dirs {
            let mut members = Vec::new();
            walk(&dir, &mut members);
            members.sort();
            let mut sources = SourceMap::new();
            let mut documents = Vec::new();
            let mut parse_error = None;
            for path in members {
                let text = std::fs::read_to_string(&path).unwrap();
                if !text.lines().any(|line| line.starts_with("format: ess/")) {
                    continue;
                }
                let label = path.strip_prefix(&dir).unwrap().display().to_string();
                match RawSpecFile::parse(&text) {
                    Ok(raw) => {
                        sources.insert(label.clone(), text);
                        documents.push((Source::new(label), raw));
                    }
                    Err(error) => parse_error = Some(error.to_string()),
                }
            }
            if documents.is_empty() {
                continue;
            }
            let name = dir.strip_prefix(root).unwrap().display().to_string();
            let value = match parse_error {
                Some(error) => format!("PARSE {}", digest(&error)),
                None => line(documents, &sources),
            };
            found.push((format!("examples-dir:{name}"), value));
        }
    }
    found
}

const BASE: &str = include_str!("fixtures/adversary-244b-base-digests.tsv");

/// Moves a later release made on purpose, each admitted only at the exact digests it produced, so
/// any further change to the same model fails the probe again.
///
/// 0.54.0 (beyond10x/ess#454, beyond10x/ess#455): an input refusal's scenario also arranges an
/// unknown identity beside stored-row branches, and the overlap of two input refusals; the
/// `id-required` and `secret-too-short` scenarios of `arrangement-input-refusal` and the
/// `blank-note` scenario of `now-stored-rows` gain steps. IR, refusals and mutants are unchanged.
const INTENDED: &[(&str, &str)] = &[
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-input-refusal.yaml",
        "ir=8f73acbe936097824f8e:9715 suite=d10530fe00b5eb7bfc37:32088 refusals=e3b0c44298fc1c149afb:0 mutants=9b0ca26989288b2d82cb:1856",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-input-refusal.yaml@ess22",
        "ir=8f73acbe936097824f8e:9715 suite=d10530fe00b5eb7bfc37:32088 refusals=e3b0c44298fc1c149afb:0 mutants=9b0ca26989288b2d82cb:1856",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/now-stored-rows.yaml",
        "ir=fad93168fa11c42db418:30720 suite=37ae4b7217b154be6972:109803 refusals=b0abbc30c3882c8c5037:423 mutants=cba7fdbc721a7942835e:2631",
    ),
];

#[ignore = "slow probe: `task test-slow-probes`"]
#[test]
fn adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a() {
    let here = digests(&root());
    if let Ok(path) = std::env::var("ADV244B_WRITE") {
        let text: String = here
            .iter()
            .map(|(label, value)| format!("{label}\t{value}\n"))
            .collect();
        std::fs::write(path, text).unwrap();
        return;
    }
    for (label, _) in INTENDED {
        assert!(
            BASE.lines()
                .any(|line| line.split('\t').next() == Some(*label)),
            "intended move {label} names no pinned model"
        );
    }
    let mut moved = Vec::new();
    let mut compared = 0usize;
    let mut compiled = 0usize;
    for pinned in BASE.lines().filter(|line| !line.is_empty()) {
        let (label, base) = pinned.split_once('\t').expect("label, then digests");
        compared += 1;
        if base.starts_with("ir=") {
            compiled += 1;
        }
        match here.iter().find(|(name, _)| name == label) {
            Some((_, value)) if value == base => {}
            Some((_, value)) if INTENDED.contains(&(label, value.as_str())) => {}
            Some((_, value)) => moved.push(format!("{label}\n  base {base}\n  here {value}")),
            None => moved.push(format!("{label}: no longer found")),
        }
    }
    let new: Vec<&String> = here
        .iter()
        .map(|(label, _)| label)
        .filter(|label| {
            !BASE
                .lines()
                .any(|line| line.split('\t').next() == Some(label.as_str()))
        })
        .collect();
    eprintln!("compared {compared} (compiled {compiled}); only here: {new:?}");
    assert!(compared > 200, "{compared}");
    assert_eq!(moved, Vec::<String>::new(), "bytes moved against 3b1684d1a");
}
