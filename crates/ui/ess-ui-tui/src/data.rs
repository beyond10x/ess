//! Reads, commands and server-held state, behind one adapter trait, and the fixture adapter that
//! answers them from a document's fixtures.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui::Document;
use serde_yaml::{Mapping, Value};

use crate::expr::display;
use crate::live::Script;
use crate::TuiError;

/// One read: a view (or placeholder) and its evaluated params.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadRequest {
    /// The view, or the placeholder name of an unbound read.
    pub view: String,
    /// The fixture file a placeholder names, relative to the document.
    pub fixture: Option<String>,
    /// Params by name; absent params are not listed.
    pub params: BTreeMap<String, Value>,
}

impl ReadRequest {
    /// The cache key: the view and its params, in order.
    pub fn key(&self) -> String {
        let params: Vec<String> = self
            .params
            .iter()
            .map(|(name, value)| format!("{name}={}", display(value)))
            .collect();
        format!("{}?{}", self.view, params.join("&"))
    }
}

/// What a read returned.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReadResult {
    /// The rows, in order.
    pub rows: Vec<Value>,
    /// The total the view reports, when it pages.
    pub total: Option<u64>,
}

/// Where the terminal gets data from and sends commands to.
///
/// The TUI never reaches a backend itself: every read, every command and every piece of state a
/// document places on the server (`server`, `server_session`) goes through this trait.
pub trait DataAdapter {
    /// Answers one read.
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String>;
    /// Runs one command with its input; the text is shown as a notification.
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Result<String, String>;
    /// Loads a server-held state value by its node path.
    fn load_state(&self, path: &str) -> Option<Value>;
    /// Stores a server-held state value by its node path.
    fn store_state(&mut self, path: &str, value: Value);
}

#[derive(Debug, Clone, PartialEq, Default)]
struct ViewData {
    rows: Vec<Value>,
    total: Option<u64>,
    by_params: Option<Mapping>,
}

/// Answers every read from the fixture files a document names.
#[derive(Debug, Default)]
pub struct FixtureAdapter {
    base: PathBuf,
    fixture_dir: Option<(String, PathBuf)>,
    views: BTreeMap<String, ViewData>,
    derived: BTreeMap<String, Value>,
    server: BTreeMap<String, Value>,
    /// Every command run, in order, with its input.
    pub commands: Vec<(String, BTreeMap<String, Value>)>,
}

impl FixtureAdapter {
    /// Loads the fixture index of `document`, read relative to `base` (the document's directory).
    /// `fixtures` replaces the document's fixture directory.
    pub fn load(
        document: &Document,
        base: &Path,
        fixtures: Option<&Path>,
    ) -> Result<(Self, Vec<Script>), TuiError> {
        let mut adapter = Self {
            base: base.to_path_buf(),
            ..Self::default()
        };
        let Some(index) = &document.fixtures else {
            return Ok((adapter, Vec::new()));
        };
        let declared_dir = index.dir.clone().unwrap_or_default();
        let dir = match fixtures {
            Some(dir) => dir.to_path_buf(),
            None => base.join(&declared_dir),
        };
        adapter.fixture_dir = Some((declared_dir, dir.clone()));
        let mut views = index.views.clone();
        let mut derived = index.derived.clone();
        let mut scripts = index.scripts.clone();
        if let Some(file) = &index.index {
            let path = adapter.document_relative(file);
            let value = read_yaml(&path)?;
            let index: ess_ui::FixtureIndex = serde_yaml::from_value(value)
                .map_err(|error| TuiError::Fixture(format!("{}: {error}", path.display())))?;
            views.extend(index.views);
            derived.extend(index.derived);
            scripts.extend(index.scripts);
        }
        let mut files: BTreeMap<String, Value> = BTreeMap::new();
        for (view, file) in &views {
            if !files.contains_key(file) {
                files.insert(file.clone(), read_yaml(&dir.join(file))?);
            }
            let data = view_data(&files[file], view).ok_or_else(|| {
                TuiError::Fixture(format!("{file} holds no rows for view {view}"))
            })?;
            adapter.views.insert(view.clone(), data);
        }
        adapter.derived = derived;
        let mut loaded = Vec::new();
        for (channel, file) in &scripts {
            let path = dir.join(file);
            let script = Script::parse(channel, &read_yaml(&path)?)
                .map_err(|error| TuiError::Fixture(format!("{}: {error}", path.display())))?;
            loaded.push(script);
        }
        Ok((adapter, loaded))
    }

    fn document_relative(&self, file: &str) -> PathBuf {
        if let Some((declared, dir)) = &self.fixture_dir {
            if let Some(rest) = file
                .strip_prefix(declared.as_str())
                .and_then(|rest| rest.strip_prefix('/'))
            {
                if !declared.is_empty() {
                    return dir.join(rest);
                }
            }
        }
        self.base.join(file)
    }

    fn view(
        &self,
        view: &str,
        params: &BTreeMap<String, Value>,
        depth: u8,
    ) -> Result<ViewData, String> {
        if let Some(data) = self.views.get(view) {
            return Ok(data.clone());
        }
        let missing = || format!("no fixture answers {view}");
        let rule = self.derived.get(view).ok_or_else(missing)?;
        if depth > 8 {
            return Err(format!("derived fixture {view} does not resolve"));
        }
        if let Some(source) = rule.get("same_as").and_then(Value::as_str) {
            return self.view(source, params, depth + 1);
        }
        if let Some(source) = rule.get("by_id_from").and_then(Value::as_str) {
            let key = rule.get("key").and_then(Value::as_str).unwrap_or("id");
            let wanted = params
                .get(key)
                .or_else(|| params.get("id"))
                .map(display)
                .unwrap_or_default();
            let mut data = self.view(source, params, depth + 1)?;
            data.rows
                .retain(|row| row.get(key).map(display).as_deref() == Some(wanted.as_str()));
            data.total = None;
            data.by_params = None;
            return Ok(data);
        }
        Err(missing())
    }
}

impl DataAdapter for FixtureAdapter {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let data = match &request.fixture {
            Some(file) => {
                let path = self.document_relative(file);
                let value = read_yaml(&path).map_err(|error| error.to_string())?;
                view_data(&value, &request.view)
                    .ok_or_else(|| format!("{file} holds no rows for {}", request.view))?
            }
            None => self.view(&request.view, &request.params, 0)?,
        };
        let mut rows = data.rows;
        let mut filtered = false;
        if let Some(by_params) = &data.by_params {
            for (key, answered) in by_params {
                let key = display(key);
                if let Some(asked) = request.params.get(&key) {
                    if !asked.is_null() && display(asked) != display(answered) {
                        rows.clear();
                        filtered = true;
                    }
                }
            }
        }
        for (name, value) in &request.params {
            if name == "q" {
                let needle = display(value).to_lowercase();
                if !needle.is_empty() {
                    rows.retain(|row| display(row).to_lowercase().contains(&needle));
                    filtered = true;
                }
                continue;
            }
            // A list-valued param narrows the field of the same name (a multi-choice filter).
            let Value::Sequence(wanted) = value else {
                continue;
            };
            if wanted.is_empty() {
                continue;
            }
            let wanted: Vec<String> = wanted.iter().map(display).collect();
            rows.retain(|row| match row.get(name.as_str()) {
                Some(Value::Sequence(have)) => {
                    have.iter().any(|item| wanted.contains(&display(item)))
                }
                Some(have) => wanted.contains(&display(have)),
                None => true,
            });
            filtered = true;
        }
        let total = if filtered {
            Some(rows.len() as u64)
        } else {
            data.total
        };
        Ok(ReadResult { rows, total })
    }

    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Result<String, String> {
        self.commands.push((command.to_owned(), input.clone()));
        let input: Vec<String> = input
            .iter()
            .map(|(name, value)| format!("{name}={}", display(value)))
            .collect();
        if input.is_empty() {
            Ok(format!("{command} accepted (fixture)"))
        } else {
            Ok(format!(
                "{command} accepted (fixture): {}",
                input.join(", ")
            ))
        }
    }

    fn load_state(&self, path: &str) -> Option<Value> {
        self.server.get(path).cloned()
    }

    fn store_state(&mut self, path: &str, value: Value) {
        self.server.insert(path.to_owned(), value);
    }
}

fn read_yaml(path: &Path) -> Result<Value, TuiError> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| TuiError::Fixture(format!("cannot read {}: {error}", path.display())))?;
    serde_yaml::from_str(&text)
        .map_err(|error| TuiError::Fixture(format!("{}: {error}", path.display())))
}

fn view_data(file: &Value, view: &str) -> Option<ViewData> {
    let entry = if file.get("rows").is_some() {
        file
    } else {
        file.get("views")?.get(view)?
    };
    Some(ViewData {
        rows: entry.get("rows")?.as_sequence()?.clone(),
        total: entry.get("total").and_then(Value::as_u64),
        by_params: entry.get("by_params").and_then(Value::as_mapping).cloned(),
    })
}
