//! Adversary case for `synthesis_suite_bytes_table.rs`: every `system-dir:` line it pins must be
//! the line of the directory as the `ess` CLI reads it.
//!
//! The CLI (`crates/edge/ess-cli/src/load.rs`, `parse_specification_inputs`) parses a directory's
//! files together with `RawSpecFile::parse_all`: a file that names no format reads the operand
//! grammar of the system header's format, and a newtype declared in one file may key a map in
//! another. The table's walker parses each member alone with `RawSpecFile::parse`, which reads a
//! format-less file with `ess/22`'s grammar. This case reads each pinned directory the CLI's way and
//! compares the line.
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use sha2::{Digest, Sha256};

const TABLE: &str = include_str!("fixtures/synthesis-suite-bytes-table.tsv");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn digest(bytes: &[u8]) -> String {
    let hex = Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    format!("{hex}:{}", bytes.len())
}

/// Every YAML file under `dir`, as `input_discovery::legacy_specification` collects them.
fn members(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            members(&path, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
}

fn listed<T: ToString>(items: &[T]) -> String {
    let printed: Vec<String> = items.iter().map(ToString::to_string).collect();
    format!(
        "{} {}",
        printed.len(),
        digest(printed.join("\n").as_bytes())
    )
}

/// The directory's line, read as `ess verify conform synthesize <dir>` reads it.
fn cli_line(label: &str, dir: &Path) -> String {
    let mut files = Vec::new();
    members(dir, &mut files);
    files.sort();
    let texts: Vec<(String, String)> = files
        .iter()
        .map(|path| {
            (
                path.strip_prefix(dir).unwrap().display().to_string(),
                std::fs::read_to_string(path).unwrap(),
            )
        })
        .collect();
    let every: Vec<&str> = texts.iter().map(|(_, text)| text.as_str()).collect();
    let mut sources = SourceMap::new();
    let mut documents = Vec::new();
    for ((name, text), parsed) in texts.iter().zip(RawSpecFile::parse_all(&every)) {
        sources.insert(name.clone(), text.clone());
        match parsed {
            Ok(raw) => documents.push((Source::new(name.clone()), raw)),
            Err(_) => return format!("{label}\tPARSE"),
        }
    }
    let Ok(spec) = Specification::assemble(documents) else {
        return format!("{label}\tASSEMBLE");
    };
    let Ok(ir) = compile(&spec, &sources) else {
        return format!("{label}\tCOMPILE");
    };
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let suite = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| format!("SUITE-ERROR {error:?}"));
    format!(
        "{label}\tsuite={} {}\trefusals={}\tnotes={}\toutside={}",
        synthesis.suite.scenarios.len(),
        digest(suite.as_bytes()),
        listed(&synthesis.refusals),
        listed(&synthesis.notes),
        listed(&synthesis.outside),
    )
}

#[test]
fn every_pinned_system_directory_is_read_as_the_cli_reads_it() {
    let root = root();
    let mut differ = Vec::new();
    let mut read = 0usize;
    for line in TABLE.lines().skip(1) {
        let Some((label, _)) = line.split_once('\t') else {
            continue;
        };
        let Some(dir) = label.strip_prefix("system-dir:") else {
            continue;
        };
        read += 1;
        let here = cli_line(label, &root.join(dir));
        if here != line {
            differ.push(format!("pinned {line}\n  cli    {here}"));
        }
    }
    assert!(read > 0, "the table pins system directories");
    assert_eq!(
        differ,
        Vec::<String>::new(),
        "{} of {read} pinned system directories are not what the CLI synthesizes",
        differ.len()
    );
}
