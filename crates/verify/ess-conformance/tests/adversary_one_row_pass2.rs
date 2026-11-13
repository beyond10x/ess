//! Pass 2 against `story:the-one-row-check-reads-the-instance-its-scenario-bound`.
//!
//! `one_row` decides a filtered view on the second run's row only where that run names the row
//! the scenario's last call addresses (`same_row`); otherwise only views without a filter carry
//! the snapshot, and with none left the scenario is withdrawn.
//!
//! * `create-or-update-stored-fields.yaml` with its one identity view filtered on `call_ref == ""`:
//!   0.94.0 files `reopened` with a one-row snapshot of `seat` read from that view, and the
//!   interpreter passes it. The scenario's last `SignIn` addresses `seat-3`, not the second run's
//!   `seat`, so `same_row` is false, the only view is filtered, and the whole scenario is
//!   withdrawn: a correct check, and the branch's scenario with it, lost.
//! * Over every model the repository holds: an updating branch beside an `unknown_instance:`
//!   creation that is withdrawn for the one-row check, or keeps its scenario without a snapshot from
//!   a filtered read-your-writes identity view, is listed.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::{EssIr, ResolvedCondition, ResolvedEffect, ResolvedInstance};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::view::Consistency;

const MODEL: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/create-or-update-stored-fields.yaml"
);

const REOPENED: &str = "demo.desk.SignIn/outcome/reopened";

/// The fixture with its one view of the seats limited to those no call holds.
fn free_seats_only() -> String {
    let out = MODEL
        .replace(
            "  - name: demo.desk.Seats\n",
            "  - name: demo.desk.FreeSeats\n",
        )
        .replace(
            "    consistency: read_your_writes\n",
            "    consistency: read_your_writes\n    filter: call_ref == \"\"\n",
        );
    assert!(
        out.contains("filter: call_ref == \"\"") && out.contains("demo.desk.FreeSeats"),
        "the fixture's view is rewritten"
    );
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let results: BTreeMap<String, (Status, Vec<String>)> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnostics = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            (result.scenario.to_string(), (result.status, diagnostics))
        })
        .collect();
    results
        .into_iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, diagnostics))| format!("{id}: {status:?} {diagnostics:?}"))
        .collect()
}

fn listing(synthesis: &Synthesis, id: &str) -> String {
    let steps = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || "(not in the suite)".to_owned(),
            |(_, scenario)| {
                scenario
                    .steps
                    .iter()
                    .map(|step| {
                        serde_json::to_value(step)
                            .expect("a step serializes")
                            .to_string()
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            },
        );
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{:?}", refusal.cause))
        .collect();
    format!("{steps}\nrefusals: {refused:?}")
}

#[test]
fn adv2_one_row_filtered_only_view_keeps_the_reopened_check() {
    let model = ir(&free_seats_only());
    let result = synthesize(&model);
    let detail = listing(&result, REOPENED);
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == REOPENED),
        "0.94.0 files `reopened` with a one-row snapshot from FreeSeats and the interpreter passes \
         it; here:\n{detail}"
    );
    let failed: Vec<String> = not_passed(model, &result.suite)
        .into_iter()
        .filter(|line| line.starts_with(&format!("{REOPENED}:")))
        .collect();
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the scenario of `reopened`:\n{detail}"
    );
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

const UNAFFORDABLE: &str =
    "crates/generate/ess-synth/tests/fixtures/conditional-measures-generated.yaml";

fn compiled(documents: Vec<(Source, RawSpecFile)>, sources: &SourceMap) -> Option<EssIr> {
    let spec = Specification::assemble(documents).ok()?;
    compile(&spec, sources).ok()
}

fn repository_models(root: &Path) -> Vec<(String, EssIr)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut found = Vec::new();
    let mut systems = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        if path.file_name().is_some_and(|name| name == "system.yaml") {
            systems.push(path.parent().unwrap().to_path_buf());
        }
        let label = path.strip_prefix(root).unwrap().display().to_string();
        if label == UNAFFORDABLE {
            continue;
        }
        let Ok(raw) = RawSpecFile::parse(&text) else {
            continue;
        };
        let mut sources = SourceMap::new();
        sources.insert(label.clone(), text);
        if let Some(ir) = compiled(vec![(Source::new(label.clone()), raw)], &sources) {
            found.push((label, ir));
        }
    }
    for dir in systems {
        let mut members = Vec::new();
        walk(&dir, &mut members);
        members.sort();
        let mut sources = SourceMap::new();
        let mut documents = Vec::new();
        let mut unparsed = false;
        for path in members {
            let text = std::fs::read_to_string(&path).unwrap();
            let label = path.strip_prefix(&dir).unwrap().display().to_string();
            match RawSpecFile::parse(&text) {
                Ok(raw) => {
                    sources.insert(label.clone(), text);
                    documents.push((Source::new(label), raw));
                }
                Err(_) => unparsed = true,
            }
        }
        if unparsed {
            continue;
        }
        let name = dir.strip_prefix(root).unwrap().display().to_string();
        if let Some(ir) = compiled(documents, &sources) {
            found.push((format!("system-dir:{name}"), ir));
        }
    }
    found
}

/// For each updating branch beside an `unknown_instance:` creation of one entity: a line where its
/// scenario is withdrawn for the one-row check, and one per filtered read-your-writes identity view
/// of that entity its scenario takes no snapshot from, with the instance its snapshots name.
fn one_row_losses(ir: &EssIr, synthesis: &Synthesis) -> Vec<String> {
    let mut out = Vec::new();
    for command in ir.commands().values() {
        for creating in command.outcomes.iter().filter(|outcome| {
            outcome.condition == ResolvedCondition::UnknownInstance
                && outcome
                    .subject
                    .as_ref()
                    .is_some_and(|subject| subject.effect == ResolvedEffect::Creates)
        }) {
            let entity = &creating.subject.as_ref().expect("filtered").entity;
            let identity = &ir.entity(entity).identity.name;
            for updating in command.outcomes.iter().filter(|outcome| {
                outcome.subject.as_ref().is_some_and(|subject| {
                    &subject.entity == entity
                        && matches!(
                            subject.effect,
                            ResolvedEffect::Moves { .. } | ResolvedEffect::Updates
                        )
                        && matches!(subject.instance, ResolvedInstance::Supplied { .. })
                })
            }) {
                let id = format!("{}/outcome/{}", command.name, updating.name);
                let withdrawn = synthesis.refusals.iter().any(|refusal| {
                    refusal
                        .scenario
                        .as_ref()
                        .is_some_and(|scenario| scenario.to_string() == id)
                        && format!("{:?}", refusal.cause).contains("one-row check")
                });
                if withdrawn {
                    out.push(format!("{id}: withdrawn for the one-row check"));
                    continue;
                }
                let Some((_, scenario)) = synthesis
                    .suite
                    .scenarios
                    .iter()
                    .find(|(key, _)| key.to_string() == id)
                else {
                    continue;
                };
                let steps: Vec<serde_json::Value> = scenario
                    .steps
                    .iter()
                    .map(|step| serde_json::to_value(step).expect("a step serializes"))
                    .collect();
                let snapped: Vec<(String, String)> = steps
                    .iter()
                    .filter(|step| step["step"] == "snapshot_subject")
                    .map(|step| {
                        (
                            step["view"].as_str().unwrap_or("?").to_owned(),
                            step["subject"][identity.as_str()]["instance"]
                                .as_str()
                                .unwrap_or("?")
                                .to_owned(),
                        )
                    })
                    .collect();
                for view in ir.views().values().filter(|view| {
                    &view.source == entity
                        && !view.is_aggregate()
                        && view.filter.is_some()
                        && view.params.is_empty()
                        && view.consistency == Consistency::ReadYourWrites
                        && view.field(identity).is_some()
                }) {
                    let name = view.name.to_string();
                    if !snapped.iter().any(|(snap, _)| *snap == name) {
                        out.push(format!(
                            "{id}: no snapshot from filtered {name}; snapshots {snapped:?}"
                        ));
                    }
                }
            }
        }
    }
    out
}

#[test]
fn adv2_one_row_no_repository_model_loses_a_one_row_snapshot() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let models = repository_models(&root);
    assert!(models.len() > 100, "the walk reads the repository's models");
    let lost: Vec<String> = models
        .iter()
        .flat_map(|(label, ir)| {
            one_row_losses(ir, &synthesize(ir))
                .into_iter()
                .map(move |line| format!("{label}: {line}"))
        })
        .collect();
    assert_eq!(lost, Vec::<String>::new());
}
