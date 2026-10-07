//! The lowered definitions of every component of every repository model
//! (`story:entity-runtime-lowering-reads-selection-plan`).
//!
//! `tests/fixtures/lowered-definitions-table.tsv` opens with a header stating how many models it
//! pins and how many lines they hold, then holds one line per model — how many components it
//! declares, or the stage it stopped at — followed by each component's lines: the digest of every
//! Entity Runtime definition it lowers to, then the branches of each of its operations and of its
//! creation in their lowered order; or, for a component lowering refuses, the digest of its
//! refusals. The table was written at the story's base, before the lowering read the precedence
//! plan, so a digest that moves is the plan reaching the lowered bytes. The four lines of
//! `held-state-after-disjoint-accepting.yaml` were added after the change, with no earlier line
//! moved: that fixture is the epic's named exception, pinned in the plan's order.
//!
//! Every component is lowered twice: once with no definition versions, which refuses every entity
//! of its closure by name (`MissingDefinitionVersion`), then with each of those at version 1.
//!
//! `LOWERED_DEFINITIONS_TABLE_WRITE=<path>` writes the table, header included, to `<path>` instead
//! of comparing it; after it, the test passes with nothing else edited. The walker is that of
//! `crates/specify/ess-compiler/tests/selection_precedence_table.rs`; its directory models are
//! read differently ([`models`]).
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{
    lower_component, ComponentLoweringError, LoweredService, LoweringCode, LoweringOptions,
};
use sha2::{Digest, Sha256};

const TABLE: &str = include_str!("fixtures/lowered-definitions-table.tsv");

/// The table's first line: how many models it pins and how many lines follow it, written with the
/// table, so a table cut short or a line lost fails against its own count.
const HEADER: &str = "#\tmodels=";

/// How to re-pin: the write mode writes the header with the table, so nothing else is edited.
const REPIN: &str = "re-pin from the repository root with \
     LOWERED_DEFINITIONS_TABLE_WRITE=\"$PWD/crates/generate/ess-entity-runtime/tests/fixtures/\
     lowered-definitions-table.tsv\" cargo test -p ess-entity-runtime --locked --test \
     lowered_definitions_table, which writes the header counts with the table; nothing else needs \
     editing";

/// The counts the header line states.
fn header(table: &str) -> (usize, usize) {
    let first = table.lines().next().unwrap_or_default();
    let counts = first
        .strip_prefix(HEADER)
        .and_then(|rest| rest.split_once("\tlines="))
        .and_then(|(models, lines)| Some((models.parse().ok()?, lines.parse().ok()?)));
    counts.unwrap_or_else(|| {
        panic!("the table opens with `#\\tmodels=<n>\\tlines=<n>`, written with it: {first:?}")
    })
}

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

/// The documents and sources of every model this tree holds, by label: each ESS source file alone,
/// and each directory holding a `system.yaml` as one system of every YAML file under it, as the
/// `ess` CLI reads a specification directory. `None` for one that does not parse.
///
/// The precedent's walker reads an `examples/<name>` directory as its files carrying a `format:`
/// line, which a domain file does not, so every multi-file example stopped at assembly there; the
/// models that lower completely are those examples.
type Model = Option<(Vec<(Source, RawSpecFile)>, SourceMap)>;

fn models(root: &Path) -> Vec<(String, Model)> {
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
        let model = RawSpecFile::parse(&text).ok().map(|raw| {
            let mut sources = SourceMap::new();
            sources.insert(label.clone(), text.clone());
            (vec![(Source::new(label.clone()), raw)], sources)
        });
        found.push((label, model));
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
        let name = dir.strip_prefix(root).unwrap().display().to_string();
        found.push((
            format!("system-dir:{name}"),
            (!unparsed).then_some((documents, sources)),
        ));
    }
    found
}

/// The lines of a component lowering refused: the digest of its refusals, each as it prints,
/// sorted.
fn refused(prefix: &str, error: &ComponentLoweringError) -> Vec<String> {
    match error {
        ComponentLoweringError::Component(diagnostics) => vec![format!(
            "{prefix}\tCOMPONENT\t{}",
            digest(format!("{diagnostics:?}").as_bytes())
        )],
        ComponentLoweringError::Lowering(diagnostics) => {
            let mut printed: Vec<String> = diagnostics
                .as_slice()
                .iter()
                .map(ToString::to_string)
                .collect();
            printed.sort();
            vec![format!(
                "{prefix}\tREFUSED\t{}",
                digest(printed.join("\n").as_bytes())
            )]
        }
    }
}

/// The lines of a component that lowers: per definition, its digest, then each operation's and the
/// creation's branches in their lowered order.
fn lowered(prefix: &str, service: &LoweredService) -> Vec<String> {
    let mut lines = Vec::new();
    for (name, definition) in service.definitions() {
        let definition = definition.as_definition();
        let bytes = serde_json::to_vec(definition)
            .expect("a validated Entity Runtime definition serializes");
        lines.push(format!("{prefix}\t{name}\tdefinition={}", digest(&bytes)));
        let branches = |outcomes: &[entity_core::OutcomeDefinition]| {
            outcomes
                .iter()
                .map(|outcome| outcome.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        if !definition.create.outcomes.is_empty() {
            lines.push(format!(
                "{prefix}\t{name}\tcreate\t{}",
                branches(&definition.create.outcomes)
            ));
        }
        for (operation, declared) in &definition.operations {
            if !declared.outcomes.is_empty() {
                lines.push(format!(
                    "{prefix}\t{name}\t{operation}\t{}",
                    branches(&declared.outcomes)
                ));
            }
        }
    }
    lines
}

/// One component's lines: lowered with no definition versions, which names its closure by
/// refusing each entity of it, then with each of those at version 1.
fn component_lines(label: &str, ir: &EssIr, component: &ComponentName) -> Vec<String> {
    let prefix = format!("{label}\t{component}");
    let closure = match lower_component(ir, component, &LoweringOptions::default()) {
        Ok(service) => return lowered(&prefix, &service),
        Err(ComponentLoweringError::Lowering(diagnostics)) => diagnostics
            .as_slice()
            .iter()
            .filter(|diagnostic| diagnostic.code == LoweringCode::MissingDefinitionVersion)
            .map(|diagnostic| {
                QualifiedName::new(&diagnostic.path).expect("the refusal names an entity")
            })
            .collect::<Vec<_>>(),
        Err(error) => return refused(&prefix, &error),
    };
    let options = LoweringOptions {
        definition_versions: closure
            .into_iter()
            .map(|entity| (entity, NonZeroU32::MIN))
            .collect(),
        scales: BTreeMap::new(),
    };
    match lower_component(ir, component, &options) {
        Ok(service) => lowered(&prefix, &service),
        Err(error) => refused(&prefix, &error),
    }
}

/// One model's lines: its own line, then each component's, by component name.
fn model_lines(label: &str, model: Model) -> Vec<String> {
    let Some((documents, sources)) = model else {
        return vec![format!("{label}\tPARSE")];
    };
    let Ok(spec) = Specification::assemble(documents) else {
        return vec![format!("{label}\tASSEMBLE")];
    };
    let Ok(ir) = compile(&spec, &sources) else {
        return vec![format!("{label}\tCOMPILE")];
    };
    let mut lines = vec![format!("{label}\tcomponents={}", ir.components().len())];
    for component in ir.components().keys() {
        lines.extend(component_lines(label, &ir, component));
    }
    lines
}

/// Every model's lines, by model label, in walk order.
fn table() -> Vec<(String, Vec<String>)> {
    models(&root())
        .into_iter()
        .map(|(label, model)| {
            let lines = model_lines(&label, model);
            (label, lines)
        })
        .collect()
}

#[test]
fn every_repository_component_keeps_its_lowered_definitions() {
    let here = table();
    let here_lines = here.iter().map(|(_, lines)| lines.len()).sum::<usize>();
    if let Ok(path) = std::env::var("LOWERED_DEFINITIONS_TABLE_WRITE") {
        let mut text = format!("{HEADER}{}\tlines={here_lines}\n", here.len());
        for line in here.iter().flat_map(|(_, lines)| lines) {
            writeln!(text, "{line}").unwrap();
        }
        std::fs::write(path, text).unwrap();
        return;
    }
    let (models, lines_stated) = header(TABLE);
    let mut pinned: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut order: Vec<&str> = Vec::new();
    for line in TABLE.lines().skip(1).filter(|line| !line.is_empty()) {
        let (label, _) = line.split_once('\t').expect("a label, then a tab");
        if !pinned.contains_key(label) {
            order.push(label);
        }
        pinned.entry(label).or_default().push(line);
    }
    let lines = pinned.values().map(Vec::len).sum::<usize>();
    assert_eq!(
        pinned.len(),
        models,
        "every pinned model is read: the table is cut short or edited by hand; {REPIN}"
    );
    assert_eq!(
        lines, lines_stated,
        "every pinned line is read: the table is cut short or edited by hand; {REPIN}"
    );

    let unpinned: Vec<&str> = here
        .iter()
        .map(|(label, _)| label.as_str())
        .filter(|label| !pinned.contains_key(label))
        .collect();
    assert_eq!(
        unpinned,
        Vec::<&str>::new(),
        "the tree holds models the table does not pin; in the change that adds them, {REPIN}"
    );

    let mut moved = Vec::new();
    for label in order {
        let Some((_, lines)) = here.iter().find(|(name, _)| name == label) else {
            moved.push(format!("{label}: no longer found"));
            continue;
        };
        let base = &pinned[label];
        if lines.iter().map(String::as_str).collect::<Vec<_>>() != *base {
            moved.push(format!(
                "{label}\n  pinned {}\n  here   {}",
                base.join("\n         "),
                lines.join("\n         ")
            ));
        }
    }
    assert_eq!(
        moved,
        Vec::<String>::new(),
        "a lowered definition moved; where the move is intended, name it, then {REPIN}"
    );
    assert_eq!(
        (here.len(), here_lines),
        (models, lines_stated),
        "the tree holds as many models and lines as the table pins; {REPIN}"
    );
}
