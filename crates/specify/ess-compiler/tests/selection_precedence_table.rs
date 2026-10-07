//! The precedence plan of every command of every repository model, and every model's IR bytes
//! (`docs/design/selection-plan.md`, `story:selection-plan-design-and-type`).
//!
//! `tests/fixtures/selection-precedence-table.tsv` opens with a header stating how many models it
//! pins and how many command lines they hold, then holds one line per model — its
//! `EssIr::to_canonical_json` digest, or the stage it stopped at — followed by one line per command
//! of a model that compiles: its non-empty phases in order, each with its branches in order. The
//! digest lines were written at the story's base, before the plan existed, so a digest that moves
//! is the plan reaching the IR. A command whose plan changes fails here until the table is re-pinned
//! on purpose, and so does a model the tree holds that the table does not pin: the change that adds
//! one re-pins in its own commit.
//!
//! `SELECTION_PRECEDENCE_TABLE_WRITE=<path>` writes the table, header included, to `<path>` instead
//! of comparing it; after it, the test passes with nothing else edited.
//! The walker and the model list are those of
//! `crates/verify/ess-conformance/tests/external_beside_held_guard.rs`; this test only compiles, so
//! no model is exempt.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::ir::PrecedencePlan;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use sha2::{Digest, Sha256};

const TABLE: &str = include_str!("fixtures/selection-precedence-table.tsv");

/// The table's first line: how many models it pins and how many command lines they hold, written
/// with the table, so a table cut short or a line lost fails against its own count.
const HEADER: &str = "#\tmodels=";

/// How to re-pin: the write mode writes the header with the table, so nothing else is edited.
const REPIN: &str = "re-pin from the repository root with \
     SELECTION_PRECEDENCE_TABLE_WRITE=\"$PWD/crates/specify/ess-compiler/tests/fixtures/\
     selection-precedence-table.tsv\" cargo test -p ess-compiler --locked --test \
     selection_precedence_table, which writes the header counts with the table; nothing else needs \
     editing";

/// The counts the header line states.
fn header(table: &str) -> (usize, usize) {
    let first = table.lines().next().unwrap_or_default();
    let counts = first
        .strip_prefix(HEADER)
        .and_then(|rest| rest.split_once("\tcommands="))
        .and_then(|(models, commands)| Some((models.parse().ok()?, commands.parse().ok()?)));
    counts.unwrap_or_else(|| {
        panic!("the table opens with `#\\tmodels=<n>\\tcommands=<n>`, written with it: {first:?}")
    })
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn digest(bytes: &str) -> String {
    let hex = Sha256::digest(bytes.as_bytes())
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
/// and each `examples/<name>` directory as one system. `None` for one that does not parse.
type Model = Option<(Vec<(Source, RawSpecFile)>, SourceMap)>;

fn models(root: &Path) -> Vec<(String, Model)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut found = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        let label = path.strip_prefix(root).unwrap().display().to_string();
        let model = RawSpecFile::parse(&text).ok().map(|raw| {
            let mut sources = SourceMap::new();
            sources.insert(label.clone(), text.clone());
            (vec![(Source::new(label.clone()), raw)], sources)
        });
        found.push((label, model));
    }
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("examples"))
        .map(|entries| {
            entries
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    for dir in dirs {
        let mut members = Vec::new();
        walk(&dir, &mut members);
        members.sort();
        let mut sources = SourceMap::new();
        let mut documents = Vec::new();
        let mut unparsed = false;
        for path in members {
            let text = std::fs::read_to_string(&path).unwrap();
            if !text.lines().any(|line| line.starts_with("format: ess/")) {
                continue;
            }
            let label = path.strip_prefix(&dir).unwrap().display().to_string();
            match RawSpecFile::parse(&text) {
                Ok(raw) => {
                    sources.insert(label.clone(), text);
                    documents.push((Source::new(label), raw));
                }
                Err(_) => unparsed = true,
            }
        }
        if documents.is_empty() && !unparsed {
            continue;
        }
        let name = dir.strip_prefix(root).unwrap().display().to_string();
        found.push((
            format!("examples-dir:{name}"),
            (!unparsed).then_some((documents, sources)),
        ));
    }
    found
}

/// One model's lines: its digest line, then one line per command, by command name.
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
    let mut lines = vec![format!("{label}\tir={}", digest(&ir.to_canonical_json()))];
    for command in ir.commands().values() {
        let plan = PrecedencePlan::new(command, ir.format());
        let phases: Vec<String> = plan
            .phases()
            .iter()
            .filter(|planned| !planned.branches.is_empty())
            .map(|planned| {
                let branches: Vec<&str> = planned
                    .branches
                    .iter()
                    .map(|outcome| outcome.name.as_str())
                    .collect();
                format!("{}: {}", planned.phase, branches.join(", "))
            })
            .collect();
        lines.push(format!("{label}\t{}\t{}", command.name, phases.join(" | ")));
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
fn every_repository_command_keeps_its_plan_and_every_model_its_ir_bytes() {
    let here = table();
    let here_commands = here.iter().map(|(_, lines)| lines.len() - 1).sum::<usize>();
    if let Ok(path) = std::env::var("SELECTION_PRECEDENCE_TABLE_WRITE") {
        let mut text = format!("{HEADER}{}\tcommands={here_commands}\n", here.len());
        for line in here.iter().flat_map(|(_, lines)| lines) {
            writeln!(text, "{line}").unwrap();
        }
        std::fs::write(path, text).unwrap();
        return;
    }
    let (models, commands_stated) = header(TABLE);
    let mut pinned: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut order: Vec<&str> = Vec::new();
    for line in TABLE.lines().skip(1).filter(|line| !line.is_empty()) {
        let (label, _) = line.split_once('\t').expect("a label, then a tab");
        if !pinned.contains_key(label) {
            order.push(label);
        }
        pinned.entry(label).or_default().push(line);
    }
    let commands = pinned
        .values()
        .map(|lines| lines.len().saturating_sub(1))
        .sum::<usize>();
    assert_eq!(
        pinned.len(),
        models,
        "every pinned model is read: the table is cut short or edited by hand; {REPIN}"
    );
    assert_eq!(
        commands, commands_stated,
        "every pinned command is read: the table is cut short or edited by hand; {REPIN}"
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
        "a pinned plan or IR digest moved; where the move is intended, {REPIN}"
    );
    assert_eq!(
        (here.len(), here_commands),
        (models, commands_stated),
        "the tree holds as many models and commands as the table pins; {REPIN}"
    );
}
