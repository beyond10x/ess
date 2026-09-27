//! Adversary pass 2 on the wave-2 `types` unit: Entity Runtime rule names after correction round 1.
//! (beyond10x/ess#146).

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::Path;

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

/// `unique_rule_name` suffixes a colliding name once, with the declaring layer. Two semantic paths
/// that sanitize to one text — the struct member `x.y` and the field `x_y` — each lower the same
/// two-layer prefix chain, so the second path's inner layer collides with the first path's
/// already-suffixed inner layer: `nominal_<..>_x_y_prefix_contract_local_Base` twice.
#[test]
fn adversary2_146_two_paths_that_sanitize_alike_keep_distinct_rule_names() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let ir = compile_changes(
        &fixture,
        &[
            ("system.yaml", "format: ess/4\n", "format: ess/15\n"),
            (
                "domains/local.yaml",
                "  - name: contract.local.Shared\n    kind: newtype\n    of: String\n",
                "  - name: contract.local.Base\n    kind: newtype\n    of: String\n    prefix: \"sh/\"\n\n  - name: contract.local.Shared\n    kind: newtype\n    of: contract.local.Base\n    prefix: \"sh/x\"\n\n  - name: contract.local.Wrap\n    kind: struct\n    fields:\n      - name: y\n        type: contract.local.Shared\n",
            ),
            (
                "domains/local.yaml",
                "      - name: memo\n        type: Optional<String>\n",
                "      - name: memo\n        type: Optional<String>\n      - name: x\n        type: Optional<contract.local.Wrap>\n      - name: x_y\n        type: Optional<contract.local.Shared>\n",
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
    for (name, definition) in lowered.definitions() {
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
