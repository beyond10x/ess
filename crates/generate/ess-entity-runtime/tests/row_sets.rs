//! A branch reading the rows a selector selects — `when_related: {entity, where, exists | count |
//! forall}` — or a value read from the one row it selects — `{related: {entity, where, field}}`
//! (ess/22, beyond10x/ess#228, #299, `docs/design/filtered-related-reads.md`) — has no Entity
//! Runtime definition: entity-core decides from a command's arguments and the one row its request
//! names, and has neither a query over other rows nor the authority to read them atomically in one
//! decision. Lowering refuses the command by name — `RowSetUnsupported` — rather than dropping the
//! guard or the read.
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

/// The focused contract fixture at ess/22, with each `(file, before, after)` edit applied once.
fn contract(changes: &[(&str, &str, &str)]) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contract");
    let mut changes = changes.to_vec();
    changes.push(("system.yaml", "format: ess/4", "format: ess/22"));
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

/// `Run` refused while another `Child` of the owner carries the note.
const ROW_SET: (&str, &str, &str) = (
    "domains/local.yaml",
    "      - name: rejected\n",
    "      - name: taken\n        when_related:\n          entity: contract.local.Child\n          where: {all: [owner_id == input.owner_id, note == input.note]}\n          exists: true\n        error: contract.local.Rejected\n        payload:\n          contract.local.Rejected: {detail: input.note}\n      - name: rejected\n",
);

/// `Run` stores the memo of the one `Child` of the owner it selects.
const READ: (&str, &str, &str) = (
    "domains/local.yaml",
    "          note: input.note\n        summary: The child is stored",
    "          note: input.note\n          memo:\n            related:\n              entity: contract.local.Child\n              where: owner_id == input.owner_id\n              field: memo\n        summary: The child is stored",
);

fn named(diagnostics: &[LoweringDiagnostic]) -> Vec<&LoweringDiagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == LoweringCode::RowSetUnsupported)
        .collect()
}

#[test]
fn a_row_set_guard_and_a_filtered_read_are_refused_by_name() {
    for (change, branch) in [(ROW_SET, "taken"), (READ, "completed")] {
        let diagnostics = refused(&contract(&[change]));
        let found = named(&diagnostics);
        assert_eq!(found.len(), 1, "{branch}: {diagnostics:#?}");
        assert_eq!(
            found[0].path,
            format!("contract.local.Run.{branch}.when_related"),
            "{diagnostics:#?}"
        );
        assert!(
            found[0].message.contains("selector"),
            "{}",
            found[0].message
        );
        // Not the identity-addressed refusal: the construct is named as what it is.
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != LoweringCode::RelatedGuardUnsupported),
            "{diagnostics:#?}"
        );
    }
}

#[test]
fn the_same_command_without_a_row_set_is_not_refused_for_one() {
    let diagnostics = refused(&contract(&[]));
    assert_eq!(named(&diagnostics).len(), 0, "{diagnostics:#?}");
}
