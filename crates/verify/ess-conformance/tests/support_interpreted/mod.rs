//! Compile the actual examples, preserving their original source labels and bytes.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::path::{Path, PathBuf};
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}
pub fn model(example: &str) -> EssIr {
    model_with(example, str::to_owned)
}
pub fn model_with(example: &str, rewrite: impl Fn(&str) -> String) -> EssIr {
    let base = root().join("examples").join(example);
    let mut pending = vec![base.clone()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "yaml") {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in files {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let text = rewrite(&std::fs::read_to_string(path).unwrap());
        let raw = RawSpecFile::parse(&text).unwrap();
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}
pub fn suite(example: &str) -> ess_conformance::AdmittedSuite {
    ess_conformance::AdmittedSuite::from_json(
        &std::fs::read_to_string(
            root()
                .join("suites/generated")
                .join(example)
                .join("suite.json"),
        )
        .unwrap(),
    )
    .unwrap()
}
