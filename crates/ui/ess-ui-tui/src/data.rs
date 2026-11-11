//! Reads, commands and server-held state, behind one adapter trait, and the fixture adapter that
//! answers them from a document's fixtures.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui::binding::{
    Answer, CommandEffect, FixtureEffects, RowChange, RowFilter, SetValue, ViewShape,
};
use ess_ui::Document;
use ess_yaml::{Mapping, Value};

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
    /// Runs one command with its input and answers what it meant to the user who sent it, as
    /// [`ess_ui::binding::classify`] reads a served command's answer.
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Answer;
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

/// How a view is answered: by one answer (the map form of a fixture file), by the first of a list
/// whose `by_params` all equal the params asked, or by failing.
#[derive(Debug, Clone, PartialEq)]
enum Answers {
    One(ViewData),
    List(Vec<ViewData>),
    Fails(String),
}

/// Answers every read from the fixture files a document names.
#[derive(Debug, Default)]
pub struct FixtureAdapter {
    base: PathBuf,
    fixture_dir: Option<(String, PathBuf)>,
    views: BTreeMap<String, Answers>,
    /// Views a test replaced: a placeholder read of one is answered from it, not from its file.
    replaced: std::collections::BTreeSet<String>,
    /// Command to the refusal that answers it: the error's wire code and its payload.
    refusals: BTreeMap<String, (String, Option<serde_json::Value>)>,
    derived: BTreeMap<String, Value>,
    server: BTreeMap<String, Value>,
    /// What a command does to the rows the views read, from the model; none without one.
    effects: Option<FixtureEffects>,
    /// Every command run, in order, with its input.
    pub commands: Vec<(String, BTreeMap<String, Value>)>,
}

thread_local! {
    static RUN_EFFECTS: std::cell::RefCell<Option<FixtureEffects>> =
        const { std::cell::RefCell::new(None) };
}

/// Restores the effects a run replaced, also when the run panics.
struct Restore(Option<FixtureEffects>);

impl Drop for Restore {
    fn drop(&mut self) {
        RUN_EFFECTS.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

/// Runs `run`, during which every [`FixtureAdapter`] loaded on this thread applies `effects`, the
/// table `ess_ui::binding::ModelEnums::fixture_effects` computes, to its rows when a command runs.
/// It is how a headless test run, which loads its adapter itself, carries the model's effects.
pub fn with_fixture_effects<T>(effects: Option<FixtureEffects>, run: impl FnOnce() -> T) -> T {
    let _restore = Restore(RUN_EFFECTS.with(|slot| slot.replace(effects)));
    run()
}

/// The `moves:` and `sets:` of a command's success on the rows of one view: the rows after the
/// change, or `None` when no row of it is the one the command names.
fn place(
    rows: &[Value],
    shape: &ViewShape,
    change: &RowChange,
    input: &BTreeMap<String, Value>,
    wanted: &str,
) -> Result<Option<Vec<Value>>, String> {
    let mut placed = Vec::with_capacity(rows.len());
    let mut changed = false;
    for row in rows {
        let named = row
            .get(shape.identity.as_str())
            .is_some_and(|identity| display(identity) == wanted);
        if !named {
            placed.push(row.clone());
            continue;
        }
        changed = true;
        if change.delete {
            continue;
        }
        let mut patched = row.clone();
        let Some(entries) = patched.as_mapping_mut() else {
            placed.push(patched);
            continue;
        };
        if let Some(to) = &change.to {
            entries.insert(
                Value::String(shape.state.clone()),
                Value::String(to.clone()),
            );
        }
        for (field, source) in &change.sets {
            let value = match source {
                SetValue::Input(name) => input
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("its input carries no `{name}`"))?,
                SetValue::Value(value) => ess_yaml::to_value(value).map_err(|e| e.to_string())?,
            };
            let key = shape.wires.get(field).unwrap_or(field);
            entries.insert(Value::String(key.clone()), value);
        }
        if shape
            .filter
            .as_ref()
            .map_or(Ok(true), |filter| listed(filter, &patched))?
        {
            placed.push(patched);
        }
    }
    Ok(changed.then_some(placed))
}

/// Whether `row` is listed by a view's filter; an error when the row lacks a field it reads.
fn listed(filter: &RowFilter, row: &Value) -> Result<bool, String> {
    let have = |field: &str| {
        row.get(field)
            .map(display)
            .ok_or_else(|| format!("its view filters on `{field}`, which the row does not carry"))
    };
    let text = |value: &serde_json::Value| {
        ess_yaml::to_value(value)
            .map(|value| display(&value))
            .unwrap_or_default()
    };
    Ok(match filter {
        RowFilter::All { of } => {
            let mut every = true;
            for part in of {
                every &= listed(part, row)?;
            }
            every
        }
        RowFilter::Any { of } => {
            let mut any = false;
            for part in of {
                any |= listed(part, row)?;
            }
            any
        }
        RowFilter::Not { of } => !listed(of, row)?,
        RowFilter::Eq { field, value } => have(field)? == text(value),
        RowFilter::Ne { field, value } => have(field)? != text(value),
        RowFilter::In { field, values } => {
            let have = have(field)?;
            values.iter().any(|value| text(value) == have)
        }
        RowFilter::NotIn { field, values } => {
            let have = have(field)?;
            !values.iter().any(|value| text(value) == have)
        }
    })
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
            effects: RUN_EFFECTS.with(|slot| slot.borrow().clone()),
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
            let index: ess_ui::FixtureIndex = ess_yaml::from_value(value)
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
            adapter.views.insert(view.clone(), Answers::One(data));
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

    /// Replaces the answers of `view` (or of a placeholder read of that name) with what a test
    /// declares: `{rows, total?, by_params?}`, a list of those, or `{fails: <message>}`.
    pub fn replace_view(&mut self, view: &str, declared: &Value) -> Result<(), String> {
        let answers = match declared {
            Value::Sequence(items) => Answers::List(
                items
                    .iter()
                    .map(|item| {
                        view_data(item, view).ok_or_else(|| "an answer lists `rows`".to_owned())
                    })
                    .collect::<Result<_, _>>()?,
            ),
            Value::Mapping(map) if map.get("fails").is_some() => Answers::Fails(
                map.get("fails")
                    .and_then(Value::as_str)
                    .ok_or("`fails` is a message")?
                    .to_owned(),
            ),
            other => Answers::One(view_data(other, view).ok_or("a view lists `rows`")?),
        };
        self.views.insert(view.to_owned(), answers);
        self.replaced.insert(view.to_owned());
        Ok(())
    }

    /// Applies what the model says `command` does to the rows the views read: `Ok(true)` when a
    /// row changed, `Ok(false)` when the command changes none, and the reason when its effect
    /// cannot be applied, leaving every row as it was.
    fn apply_effect(
        &mut self,
        command: &str,
        input: &BTreeMap<String, Value>,
    ) -> Result<bool, String> {
        let effects = self
            .effects
            .as_ref()
            .ok_or_else(|| "no model was given (run `ess ui test --model`)".to_owned())?;
        let change = match effects.commands.get(command) {
            None => return Ok(false),
            Some(CommandEffect::NotApplied { unapplied }) => return Err(unapplied.clone()),
            Some(CommandEffect::Applies(change)) => change,
        };
        let wanted = input
            .get(&change.instance)
            .map(display)
            .ok_or_else(|| format!("its input carries no `{}`", change.instance))?;
        let mut placed: Vec<(&String, Answers)> = Vec::new();
        for (view, shape) in &effects.views {
            if shape.entity != change.entity {
                continue;
            }
            let Some(answers) = self.views.get(view) else {
                continue;
            };
            if let Some(reason) = &shape.unapplied {
                return Err(reason.clone());
            }
            let rewrite = |data: &ViewData| -> Result<Option<ViewData>, String> {
                Ok(
                    place(&data.rows, shape, change, input, &wanted)?.map(|rows| ViewData {
                        rows,
                        total: None,
                        by_params: data.by_params.clone(),
                    }),
                )
            };
            let next = match answers {
                Answers::One(data) => rewrite(data)?.map(Answers::One),
                Answers::List(list) => {
                    let mut changed = false;
                    let mut next = Vec::with_capacity(list.len());
                    for data in list {
                        match rewrite(data)? {
                            Some(data) => {
                                changed = true;
                                next.push(data);
                            }
                            None => next.push(data.clone()),
                        }
                    }
                    changed.then_some(Answers::List(next))
                }
                Answers::Fails(_) => None,
            };
            placed.extend(next.map(|next| (view, next)));
        }
        let changed = !placed.is_empty();
        let placed: Vec<(String, Answers)> = placed
            .into_iter()
            .map(|(view, answers)| (view.clone(), answers))
            .collect();
        self.views.extend(placed);
        Ok(changed)
    }

    /// Answers `command` with the refusal `error` (its wire code) and its `payload`.
    pub fn refuse(&mut self, command: &str, error: &str, payload: Option<&Value>) {
        let payload = payload.and_then(|payload| serde_json::to_value(payload).ok());
        self.refusals
            .insert(command.to_owned(), (error.to_owned(), payload));
    }

    /// The file a placeholder's `fixture:` names: in the fixture directory when the document
    /// declares `fixtures.dir`, as a view or script file is (a path that starts with the declared
    /// directory names the same file, a leading `./` on either changing nothing); else relative
    /// to the document.
    fn placeholder_file(&self, file: &str) -> PathBuf {
        match &self.fixture_dir {
            Some((declared, dir)) if !declared.is_empty() => {
                let file = without_dot(Path::new(file));
                dir.join(
                    file.strip_prefix(without_dot(Path::new(declared)))
                        .unwrap_or(&file),
                )
            }
            _ => self.base.join(file),
        }
    }

    /// A file named relative to the document: in the fixture directory a run gives when it lies
    /// under the declared one (a leading `./` on either changing nothing), else beside the
    /// document.
    fn document_relative(&self, file: &str) -> PathBuf {
        if let Some((declared, dir)) = &self.fixture_dir {
            let declared = without_dot(Path::new(declared));
            if !declared.as_os_str().is_empty() {
                if let Ok(rest) = without_dot(Path::new(file)).strip_prefix(&declared) {
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
        match self.views.get(view) {
            Some(Answers::One(data)) => return Ok(data.clone()),
            Some(Answers::Fails(message)) => return Err(message.clone()),
            Some(Answers::List(answers)) => {
                let asked = |answer: &&ViewData| {
                    answer.by_params.as_ref().is_some_and(|wanted| {
                        wanted.iter().all(|(key, value)| {
                            params
                                .get(&display(key))
                                .is_some_and(|asked| display(asked) == display(value))
                        })
                    })
                };
                let chosen = answers
                    .iter()
                    .find(asked)
                    .or_else(|| answers.iter().find(|answer| answer.by_params.is_none()));
                return Ok(chosen.cloned().map_or_else(ViewData::default, |mut data| {
                    data.by_params = None;
                    data
                }));
            }
            None => {}
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

/// `path` without its `.` components: `Path::strip_prefix` compares component-wise and keeps a
/// leading `./`, so `./data` is no prefix of `data/quarters.yaml` until it is dropped.
fn without_dot(path: &Path) -> PathBuf {
    path.components()
        .filter(|component| !matches!(component, std::path::Component::CurDir))
        .collect()
}

impl DataAdapter for FixtureAdapter {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let from_file = request
            .fixture
            .as_ref()
            .filter(|_| !self.replaced.contains(&request.view));
        let data = match from_file {
            Some(file) => {
                let path = self.placeholder_file(file);
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
            // Any other param narrows the field of the same name, as the React output's
            // `matchesParams` does: compared as text, a list param (a multi-choice filter)
            // matching any of its items, a row without the field kept.
            let wanted: Vec<String> = match value {
                Value::Sequence(items) => items.iter().map(display).collect(),
                Value::Null | Value::Mapping(_) => continue,
                Value::String(text) if text.is_empty() => continue,
                scalar => {
                    if !rows.iter().any(|row| row.get(name.as_str()).is_some()) {
                        continue;
                    }
                    vec![display(scalar)]
                }
            };
            if wanted.is_empty() {
                continue;
            }
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

    /// Every command is logged in [`FixtureAdapter::commands`]; it is accepted unless a test
    /// declared a refusal for it ([`FixtureAdapter::refuse`]).
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Answer {
        self.commands.push((command.to_owned(), input.clone()));
        if let Some((error, payload)) = self.refusals.get(command) {
            return Answer::Refused {
                error: error.clone(),
                payload: payload.clone(),
            };
        }
        match self.apply_effect(command, input) {
            Ok(true) => Answer::FixtureApplied,
            Ok(false) => Answer::Accepted,
            Err(reason) => Answer::FixtureNotApplied { reason },
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
    ess_yaml::from_str(&text)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn adapter(rows: &str) -> FixtureAdapter {
        let mut adapter = FixtureAdapter::default();
        adapter.views.insert(
            "things.Page".to_owned(),
            Answers::One(ViewData {
                rows: ess_yaml::from_str(rows).expect("the rows parse"),
                total: Some(40),
                by_params: None,
            }),
        );
        adapter
    }

    fn ids(adapter: &FixtureAdapter, params: &str) -> (Vec<String>, Option<u64>) {
        let request = ReadRequest {
            view: "things.Page".to_owned(),
            fixture: None,
            params: ess_yaml::from_str(params).expect("the params parse"),
        };
        let result = adapter.read(&request).expect("the read is answered");
        let ids = result.rows.iter().map(|row| display(&row["id"])).collect();
        (ids, result.total)
    }

    /// A single-value param narrows the field of the same name the way the React output's
    /// `matchesParams` does: compared as text, a row without the field kept, an empty value
    /// ignored.
    #[test]
    fn a_single_value_param_narrows_the_field_of_its_name() {
        let adapter = adapter(
            "[{id: a, rank: 1, tags: [x, y]}, {id: b, rank: \"2\", tags: [y]}, {id: c}, plain]",
        );
        assert_eq!(
            ids(&adapter, "{id: b}"),
            (vec!["b".to_owned(), String::new()], Some(2)),
            "the rows with another id go; the non-record stays"
        );
        assert_eq!(
            ids(&adapter, "{rank: 2}").0,
            ["b", "c", ""],
            "a number param matches a string field by its text; the row without the field stays"
        );
        assert_eq!(
            ids(&adapter, "{tags: x}").0,
            ["a", "c", ""],
            "a scalar param matches a list field holding it"
        );
        assert_eq!(
            ids(&adapter, "{id: \"\"}"),
            (
                vec![
                    "a".to_owned(),
                    "b".to_owned(),
                    "c".to_owned(),
                    String::new()
                ],
                Some(40)
            ),
            "an empty value narrows nothing and leaves the view's total"
        );
        assert_eq!(
            ids(&adapter, "{page: 2}"),
            (
                vec![
                    "a".to_owned(),
                    "b".to_owned(),
                    "c".to_owned(),
                    String::new()
                ],
                Some(40)
            ),
            "a param no row carries narrows nothing and leaves the view's total"
        );
    }
}
