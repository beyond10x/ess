//! A branch guarded by a row of another entity (`when_related:`, ess/18, beyond10x/ess#211) has no
//! Entity Runtime definition: an entity-core operation reads its arguments and the one row its
//! request names, so lowering refuses the command by name — `RelatedGuardUnsupported` — rather than
//! dropping the guard and lowering a creation that would take every request.

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
use ess_entity_runtime::{lower, LoweringCode, LoweringDiagnostic, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

/// The focused contract fixture at ess/18, with each `(file, before, after)` edit applied once.
fn contract(changes: &[(&str, &str, &str)]) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let mut changes = changes.to_vec();
    changes.push(("system.yaml", "format: ess/4", "format: ess/18"));
    let mut paths = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
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
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (target, before, after) in &changes {
            if label == *target {
                assert!(text.contains(before), "{target} holds {before}");
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
    let specification = Specification::assemble(parsed).unwrap_or_else(|errors| panic!("{errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn refused(ir: &EssIr) -> Vec<LoweringDiagnostic> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("local-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: ["contract.foreign.Owner", "contract.local.Child"]
            .into_iter()
            .map(|name| {
                (
                    QualifiedName::new(name).unwrap(),
                    NonZeroU32::new(1).unwrap(),
                )
            })
            .collect(),
        scales: BTreeMap::new(),
    };
    match lower(&service, &options) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics.into_vec(),
    }
}

/// `Run` refused when no `Owner` carries the identity its input names.
const NO_OWNER: (&str, &str, &str) = (
    "domains/local.yaml",
    "      - name: rejected\n",
    "      - name: no-owner\n        when_related: {via: input.owner_id, exists: false}\n        error: contract.local.Rejected\n      - name: rejected\n",
);

#[test]
fn issue_211_a_command_guarded_by_a_related_row_is_refused_by_name() {
    let diagnostics = refused(&contract(&[NO_OWNER]));
    let related: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LoweringCode::RelatedGuardUnsupported)
        .collect();
    assert_eq!(related.len(), 1, "{diagnostics:#?}");
    assert_eq!(
        related[0].path, "contract.local.Run.no-owner.when_related",
        "{diagnostics:#?}"
    );
    assert!(
        related[0].message.contains("another entity"),
        "{}",
        related[0].message
    );
}

#[test]
fn issue_211_the_same_command_without_the_guard_is_not_refused_for_it() {
    let diagnostics = refused(&contract(&[]));
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != LoweringCode::RelatedGuardUnsupported),
        "{diagnostics:#?}"
    );
}
