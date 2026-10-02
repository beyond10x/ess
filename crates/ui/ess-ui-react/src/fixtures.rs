//! The document's fixtures, read from disk and written as `src/fixtures.ts`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui::{Body, Composite, Document, NodeRef, Reads};
use serde_yaml::{Mapping, Value};

use crate::ts::{self, Writer};
use crate::GenerateError;

#[derive(Default)]
struct Fixtures {
    views: BTreeMap<String, Value>,
    derived: BTreeMap<String, Value>,
    scripts: BTreeMap<String, Value>,
}

fn read_yaml(path: &Path) -> Result<Value, GenerateError> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        GenerateError::new(format!("cannot read fixture {}: {error}", path.display()))
    })?;
    serde_yaml::from_str(&text).map_err(|error| {
        GenerateError::new(format!("fixture {} is not YAML: {error}", path.display()))
    })
}

fn get<'v>(value: &'v Value, key: &str) -> Option<&'v Value> {
    value.as_mapping().and_then(|mapping| mapping.get(key))
}

/// A view file answers one view (`view:` + `rows:`) or several (`views:`).
fn views_of(file: &Value) -> Vec<(String, Value)> {
    if let Some(Value::Mapping(views)) = get(file, "views") {
        return views
            .iter()
            .map(|(name, body)| (ts::key_text(name), body.clone()))
            .collect();
    }
    if let Some(Value::String(view)) = get(file, "view") {
        let mut body = file.as_mapping().cloned().unwrap_or_default();
        body.remove("view");
        return vec![(view.clone(), Value::Mapping(body))];
    }
    Vec::new()
}

/// Durations `500ms`, `4s`, `2m`, `1h` in milliseconds; `None` for anything else, and for a
/// duration too large for a `u64` of milliseconds.
pub(crate) fn millis(text: &str) -> Option<u64> {
    let (digits, factor) = if let Some(digits) = text.strip_suffix("ms") {
        (digits, 1)
    } else if let Some(digits) = text.strip_suffix('s') {
        (digits, 1000)
    } else if let Some(digits) = text.strip_suffix('m') {
        (digits, 60_000)
    } else {
        (text.strip_suffix('h')?, 3_600_000)
    };
    digits
        .trim()
        .parse::<u64>()
        .ok()
        .and_then(|value| value.checked_mul(factor))
}

/// A script with every `at` in milliseconds.
fn script(file: &Value) -> Value {
    let mut out = Mapping::new();
    out.insert(
        "loop".into(),
        Value::Bool(get(file, "loop").and_then(Value::as_bool).unwrap_or(false)),
    );
    if let Some(session) = get(file, "session") {
        out.insert("session".into(), session.clone());
    }
    let events: Vec<Value> = get(file, "events")
        .and_then(Value::as_sequence)
        .map(|events| {
            events
                .iter()
                .map(|entry| {
                    let mut mapping = entry.as_mapping().cloned().unwrap_or_default();
                    let at = mapping
                        .get("at")
                        .and_then(|at| match at {
                            Value::String(text) => millis(text),
                            Value::Number(number) => number.as_u64(),
                            _ => None,
                        })
                        .unwrap_or(0);
                    mapping.insert("at".into(), Value::Number(at.into()));
                    Value::Mapping(mapping)
                })
                .collect()
        })
        .unwrap_or_default();
    out.insert("events".into(), Value::Sequence(events));
    Value::Mapping(out)
}

fn placeholder_reads(document: &Document) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut push = |reads: &Reads| {
        if let (Some(view), Some(file)) = (&reads.placeholder, &reads.fixture) {
            found.push((view.clone(), file.clone()));
        }
    };
    for located in document.nodes() {
        let body = match located.node {
            NodeRef::Section(section) => &section.body,
            NodeRef::Overlay(overlay) => &overlay.body,
            NodeRef::Node(node) => &node.body,
            NodeRef::Action(action) => {
                if let Some(reads) = &action.loads {
                    push(reads);
                }
                continue;
            }
            _ => continue,
        };
        if let Body::Composite(composite) = body {
            let reads = match composite {
                Composite::Collection(c) => c.reads.as_ref(),
                Composite::Record(c) => c.reads.as_ref(),
                Composite::Form(c) => c.loads.as_ref(),
                Composite::Choice(c) => c.reads.as_ref(),
                Composite::Metric(c) => c.reads.as_ref(),
                Composite::Chart(c) => Some(&c.reads),
                Composite::Board(c) => Some(&c.reads),
                Composite::GraphEditor(c) => Some(&c.reads),
                Composite::References(c) => Some(&c.reads),
                Composite::FilterBar(_) | Composite::Confirm(_) | Composite::RichText(_) => None,
            };
            if let Some(reads) = reads {
                push(reads);
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

fn load(document: &Document, source_dir: &Path) -> Result<Fixtures, GenerateError> {
    let mut fixtures = Fixtures::default();
    let Some(index) = &document.fixtures else {
        for (view, file) in placeholder_reads(document) {
            load_view_file(&mut fixtures, &source_dir.join(file), Some(&view))?;
        }
        return Ok(fixtures);
    };
    let dir: PathBuf = index
        .dir
        .as_ref()
        .map_or_else(|| source_dir.to_path_buf(), |dir| source_dir.join(dir));
    let mut views = index.views.clone();
    let mut derived: BTreeMap<String, Value> = index.derived.clone();
    let mut scripts = index.scripts.clone();
    if let Some(file) = &index.index {
        let value = read_yaml(&source_dir.join(file))?;
        if let Some(Value::Mapping(map)) = get(&value, "views") {
            for (view, file) in map {
                if let Value::String(file) = file {
                    views
                        .entry(ts::key_text(view))
                        .or_insert_with(|| file.clone());
                }
            }
        }
        if let Some(Value::Mapping(map)) = get(&value, "derived") {
            for (view, rule) in map {
                derived
                    .entry(ts::key_text(view))
                    .or_insert_with(|| rule.clone());
            }
        }
        if let Some(Value::Mapping(map)) = get(&value, "scripts") {
            for (channel, file) in map {
                if let Value::String(file) = file {
                    scripts
                        .entry(ts::key_text(channel))
                        .or_insert_with(|| file.clone());
                }
            }
        }
    }
    let mut files: BTreeMap<String, Value> = BTreeMap::new();
    for (view, file) in &views {
        if !files.contains_key(file) {
            files.insert(file.clone(), read_yaml(&dir.join(file))?);
        }
        let answered = views_of(&files[file]);
        if let Some((_, body)) = answered.into_iter().find(|(name, _)| name == view) {
            fixtures.views.insert(view.clone(), body);
        }
    }
    fixtures.derived = derived;
    for (channel, file) in &scripts {
        fixtures
            .scripts
            .insert(channel.clone(), script(&read_yaml(&dir.join(file))?));
    }
    for (view, file) in placeholder_reads(document) {
        load_view_file(&mut fixtures, &source_dir.join(file), Some(&view))?;
    }
    Ok(fixtures)
}

fn load_view_file(
    fixtures: &mut Fixtures,
    path: &Path,
    only: Option<&str>,
) -> Result<(), GenerateError> {
    let file = read_yaml(path)?;
    for (view, body) in views_of(&file) {
        if only.is_none_or(|only| only == view) {
            fixtures.views.insert(view, body);
        }
    }
    Ok(())
}

fn view_literal(body: &Value) -> String {
    let rows = get(body, "rows")
        .and_then(Value::as_sequence)
        .cloned()
        .unwrap_or_default();
    let mut parts = Vec::new();
    if let Some(total) = get(body, "total") {
        parts.push(format!("total: {}", ts::literal(total)));
    }
    if let Some(by_params) = get(body, "by_params") {
        parts.push(format!("by_params: {}", ts::literal(by_params)));
    }
    let rows: Vec<String> = rows.iter().map(ts::literal).collect();
    parts.push(format!("rows: [\n      {},\n    ]", rows.join(",\n      ")));
    format!("{{ {} }}", parts.join(", "))
}

/// `src/fixtures.ts`.
pub(crate) fn emit(document: &Document, source_dir: &Path) -> Result<String, GenerateError> {
    let fixtures = load(document, source_dir)?;
    let mut w = Writer::default();
    w.line("// Generated by ess-ui-react from the document's fixtures. Do not edit.");
    w.line("import type { Json } from \"./runtime/json\";");
    w.line("");
    w.line("/** Rows answering one view. */");
    w.line("export interface FixtureView {");
    w.line("  rows: Json[];");
    w.line("  total?: number;");
    w.line("  by_params?: Json;");
    w.line("}");
    w.line("");
    w.line("/** One entry of a channel script, `at` in milliseconds from the start. */");
    w.line("export interface FixtureScriptEntry {");
    w.line("  at: number;");
    w.line("  event?: string;");
    w.line("  lifecycle?: string;");
    w.line("  payload?: Json;");
    w.line("}");
    w.line("");
    w.line("/** A channel's event script. */");
    w.line("export interface FixtureScript {");
    w.line("  loop: boolean;");
    w.line("  session?: Json;");
    w.line("  events: FixtureScriptEntry[];");
    w.line("}");
    w.line("");
    w.line("/** A view answered from another view's fixture. */");
    w.line("export type DerivedView = { same_as: string } | { by_id_from: string; key: string };");
    w.line("");
    w.open("export const fixtures: {");
    w.line("views: Record<string, FixtureView>;");
    w.line("derived: Record<string, DerivedView>;");
    w.line("scripts: Record<string, FixtureScript>;");
    w.close("} = {");
    w.indent();
    w.open("views: {");
    for (view, body) in &fixtures.views {
        w.line(format!("{}: {},", ts::string(view), view_literal(body)));
    }
    w.close("},");
    w.open("derived: {");
    for (view, rule) in &fixtures.derived {
        w.line(format!("{}: {},", ts::string(view), ts::literal(rule)));
    }
    w.close("},");
    w.open("scripts: {");
    for (channel, script) in &fixtures.scripts {
        w.line(format!("{}: {},", ts::string(channel), ts::literal(script)));
    }
    w.close("},");
    w.dedent();
    w.line("};");
    Ok(w.finish())
}
