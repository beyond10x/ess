//! Adversary, pass 1, for unit E-U2: a byte-identity probe. For every YAML model in the repository
//! that compiles — as written, and relabelled `ess/22` — it writes one line of hashes (compiled IR,
//! synthesized suite and its refusals, mutants, generated Rust and Go artifacts) to the file named
//! by `ADV_E_U2_PROBE_OUT`. The same file, compiled in the unit's tree and in an export of its base,
//! yields two listings whose difference is every model whose bytes moved. Without the variable the
//! probe only counts the models it read.
#![allow(clippy::too_many_lines)]

use std::fmt::Write as _;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn hash(text: &str) -> String {
    let mut state = DefaultHasher::new();
    text.hash(&mut state);
    format!("{:016x}", state.finish())
}

fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if matches!(name, "node_modules" | "target" | ".git" | ".engineering") {
            continue;
        }
        if path.is_dir() {
            walk(&path, found);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "yaml")
        {
            found.push(path);
        }
    }
}

fn line(name: &str, text: &str) -> String {
    let raw = match RawSpecFile::parse(text) {
        Ok(raw) => raw,
        Err(error) => return format!("{name} PARSE {}", hash(&error.to_string())),
    };
    let documents = vec![(Source::new("model.yaml"), raw.clone())];
    let spec = match Specification::assemble([(Source::new("model.yaml"), raw)]) {
        Ok(spec) => spec,
        Err(errors) => return format!("{name} ASSEMBLE {}", hash(&errors.to_string())),
    };
    let ir = match compile(&spec, &SourceMap::new()) {
        Ok(ir) => ir,
        Err(error) => return format!("{name} COMPILE {}", hash(&format!("{error:?}"))),
    };
    let ir_text = serde_json::to_string(&ir).unwrap_or_else(|error| error.to_string());
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let suite = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| format!("ERR {error}"));
    let refusals = format!("{:?}", synthesis.refusals);
    let mutants: Vec<String> =
        ess_conformance::mutate::mutants(&documents, ess_conformance::mutate::MutantClass::ALL)
            .into_iter()
            .map(|mutant| format!("{} {}", mutant.id, mutant.change))
            .collect();
    let mut generated = Vec::new();
    for target in [ess_synth::Target::Rust, ess_synth::Target::Go] {
        generated.push(match ess_synth::synthesize_for(&ir, target) {
            Ok(synthesis) => {
                let mut all = String::new();
                for (path, artifact) in &synthesis.artifacts {
                    all.push_str(path);
                    all.push('\n');
                    all.push_str(&artifact.contents);
                }
                let _ = write!(all, "{:?}", synthesis.target);
                hash(&all)
            }
            Err(error) => format!("ERR{}", hash(&format!("{error:?}"))),
        });
    }
    format!(
        "{name} ir={} suite={} refusals={} mutants={} rust={} go={}",
        hash(&ir_text),
        hash(&suite),
        hash(&refusals),
        hash(&mutants.join("\n")),
        generated[0],
        generated[1]
    )
}

/// [`line`], or the panic it raised, hashed: a generator defect is one more byte to compare.
fn guarded(name: &str, text: &str) -> String {
    std::panic::catch_unwind(|| line(name, text)).unwrap_or_else(|panic| {
        let message = panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
            .unwrap_or_default();
        format!("{name} PANIC {}", hash(&message))
    })
}

#[test]
fn adv_e_u2_byte_identity_probe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let root = root.canonicalize().expect("the workspace root");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut lines = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.starts_with("format: ess/") {
            continue;
        }
        let name = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        lines.push(guarded(&name, &text));
        let header = text.lines().next().unwrap_or("");
        if header != "format: ess/22" {
            let relabelled = text.replacen(header, "format: ess/22", 1);
            lines.push(guarded(&format!("{name}@ess22"), &relabelled));
        }
    }
    if let Ok(out) = std::env::var("ADV_E_U2_PROBE_OUT") {
        std::fs::write(out, lines.join("\n") + "\n").expect("the probe writes");
    }
    assert!(lines.len() > 50, "{} models read", lines.len());
}
