//! Adversary pass 1 on the wave-2 `types` unit: a narrowing chain of prefixes lowered to Entity
//! Runtime (beyond10x/ess#146).

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

use entity_core::Registry;
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;
use serde_json::Value;

fn compile_changes(base: &Path, changes: &[(&str, &str, &str)]) -> EssIr {
    let mut pending = vec![base.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let (mut parsed, mut labels, mut sources) = (Vec::new(), Vec::new(), SourceMap::new());
    for path in paths {
        let label = path.strip_prefix(base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (target, before, after) in changes {
            if label == *target {
                assert!(text.contains(before), "{target}: {before}");
                text = text.replacen(before, after, 1);
            }
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed).unwrap_or_else(|e| panic!("{e}"));
    compile_locating(&specification, &sources, &labels).expect("compiles")
}

fn names(value: &Value, into: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(name)) = map.get("name") {
                if name.starts_with("nominal_") {
                    into.push(name.clone());
                }
            }
            map.values().for_each(|child| names(child, into));
        }
        Value::Array(items) => items.iter().for_each(|item| names(item, into)),
        _ => {}
    }
}

/// `Shared` narrows `Base`'s prefix `sh/` to `sh/x`. Each layer lowers its own `starts_with` rule
/// under the name `nominal_<path>_prefix`, so the two rules of one field share a name.
#[test]
fn adversary_146_a_narrowing_prefix_chain_lowers_to_distinctly_named_rules() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let ir = compile_changes(
        &fixture,
        &[
            ("system.yaml", "format: ess/4\n", "format: ess/15\n"),
            (
                "domains/local.yaml",
                "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n",
                "  - name: contract.local.Base\n    kind: newtype\n    of: String\n    prefix: \"sh/\"\n\n  - name: contract.local.Shared\n    kind: newtype\n    of: contract.local.Base\n    prefix: \"sh/x\"\n",
            ),
        ],
    );
    let plan = SynthesisPlan::of(&ir);
    let service = extract(&ir, &plan, &ComponentName::new("local-service").unwrap())
        .expect("service extracts");
    let options = LoweringOptions {
        definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
            .iter()
            .map(|entry| {
                (
                    QualifiedName::new(entry).unwrap(),
                    NonZeroU32::new(1).unwrap(),
                )
            })
            .collect(),
        scales: BTreeMap::new(),
    };
    let lowered = lower(&service, &options).unwrap_or_else(|d| panic!("lowers: {d:?}"));
    let mut registry = Registry::new();
    for (name, definition) in lowered.definitions() {
        registry
            .register(definition.as_definition().clone())
            .unwrap_or_else(|error| panic!("`{name}` registers: {error:?}"));
        let rendered = serde_json::to_value(definition.as_definition()).unwrap();
        let mut found = Vec::new();
        names(&rendered, &mut found);
        let mut unique = found.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(
            found.len(),
            unique.len(),
            "`{name}` carries a nominal rule name twice: {found:?}"
        );
    }
}
