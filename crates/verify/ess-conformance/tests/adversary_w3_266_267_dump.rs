//! Adversary pass 1 against beyond10x/ess#266 and #267: suite byte drift.
//!
//! Synthesizes every single-file fixture with bindings or without, the committed examples and the
//! toolchain model, twice each, and requires one document per model. With `ADV_DUMP_DIR` set it
//! also writes each canonical suite and its refusals there, so the same file run at the unit's base
//! commit gives a byte-for-byte comparison. Uses only API the base already has, and reads every
//! model from `ADV_MODEL_ROOT` (default: this checkout) so both runs read the same inputs.

use std::path::{Path, PathBuf};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::synthesize::synthesize;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn root() -> PathBuf {
    std::env::var_os("ADV_MODEL_ROOT").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."),
        PathBuf::from,
    )
}

fn yaml_under(dir: &Path, recurse: bool) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(path) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&path) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if recurse {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|ext| ext == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// One model: its canonical suite and refusals, or why it could not be synthesized.
fn synthesized(files: &[PathBuf], base: &Path) -> String {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in files {
        let label = path
            .strip_prefix(base)
            .unwrap_or(path)
            .display()
            .to_string();
        let text = std::fs::read_to_string(path).unwrap();
        let raw = match RawSpecFile::parse(&text) {
            Ok(raw) => raw,
            Err(error) => return format!("PARSE {error}"),
        };
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let spec = match Specification::assemble(parsed) {
        Ok(spec) => spec,
        Err(errors) => return format!("ASSEMBLE {errors}"),
    };
    let ir = match compile(&spec, &sources) {
        Ok(ir) => ir,
        Err(diagnostics) => return format!("COMPILE {diagnostics}"),
    };
    let Ok(synthesis) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| synthesize(&ir)))
    else {
        return "PANIC".to_owned();
    };
    let mut out = synthesis.suite.to_canonical_json().unwrap();
    out.push_str("\n---- refusals\n");
    for refusal in &synthesis.refusals {
        use std::fmt::Write as _;
        writeln!(out, "{refusal}").unwrap();
    }
    out
}

#[ignore = "slow probe: `task test-slow-probes`"]
#[test]
fn adv_every_model_synthesizes_one_document_and_dumps_for_base_comparison() {
    let root = root();
    let mut models: Vec<(String, Vec<PathBuf>, PathBuf)> = Vec::new();
    for dir in [
        "crates/verify/ess-conformance/tests/fixtures",
        "crates/generate/ess-synth/tests/fixtures",
        "crates/edge/ess-cli/tests/fixtures",
        "crates/specify/ess-compiler/tests/fixtures",
        "crates/specify/ess-domain/tests/fixtures",
        "crates/specify/ess-service-contract/tests/fixtures",
        "crates/generate/ess-entity-runtime/tests/fixtures/contract",
    ] {
        for file in yaml_under(&root.join(dir), false) {
            let name = format!(
                "{}__{}",
                dir.replace('/', "_"),
                file.file_name().unwrap().to_string_lossy()
            );
            models.push((
                name,
                vec![file.clone()],
                file.parent().unwrap().to_path_buf(),
            ));
        }
    }
    for dir in [
        "examples/billing",
        "examples/gatepass",
        "examples/oracle-fixture",
        "models/toolchain",
    ] {
        let base = root.join(dir);
        models.push((dir.replace('/', "_"), yaml_under(&base, true), base));
    }
    let out = std::env::var_os("ADV_DUMP_DIR").map(PathBuf::from);
    let mut synthesized_count = 0;
    for (name, files, base) in &models {
        let first = synthesized(files, base);
        let second = synthesized(files, base);
        assert_eq!(first, second, "{name}: two syntheses, one document");
        if !first.starts_with("PARSE")
            && !first.starts_with("ASSEMBLE")
            && !first.starts_with("COMPILE")
        {
            synthesized_count += 1;
        }
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join(format!("{name}.out")), &first).unwrap();
        }
    }
    eprintln!("models: {}, synthesized: {synthesized_count}", models.len());
    assert!(synthesized_count >= 70, "{synthesized_count}");
}
