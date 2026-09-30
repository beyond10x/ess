//! The `ess-ui-test/1` file: a document, and tests of steps, each step one YAML map entry.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_yaml::{Mapping, Value};

use crate::{TestError, FORMAT};

/// A test file.
#[derive(Debug, Clone, PartialEq)]
pub struct TestFile {
    /// Where it was read from; `document` is relative to its directory.
    pub source: PathBuf,
    /// The document it tests, as written.
    pub document: String,
    /// Its tests, in order.
    pub tests: Vec<Test>,
}

/// One test.
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    /// Its name.
    pub name: String,
    /// Fixture data replacing the document's for this test.
    pub fixtures: Fixtures,
    /// How long every read takes on the virtual clock.
    pub latency: Duration,
    /// Its steps, in order.
    pub steps: Vec<Step>,
}

/// Per-test fixtures: view data by view, event scripts by channel.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Fixtures {
    /// View name to `{rows, total?, by_params?}`, as a fixture file holds it.
    pub views: BTreeMap<String, Value>,
    /// Channel name to `{events, loop?, session?}`, as a script file holds it.
    pub scripts: BTreeMap<String, Value>,
}

impl Fixtures {
    /// Whether the test replaces nothing.
    pub fn is_empty(&self) -> bool {
        self.views.is_empty() && self.scripts.is_empty()
    }
}

/// One step.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// `open: <page>` or `open: {page, params}`.
    Open {
        /// The page.
        page: String,
        /// Its params.
        params: BTreeMap<String, String>,
    },
    /// `select: <path>`: focus a section, move to a row.
    Select {
        /// The node.
        at: String,
    },
    /// `type: {at, text}`: keystrokes into a search, filter or form field.
    Type {
        /// The node.
        at: String,
        /// What is typed.
        text: String,
    },
    /// `choose: {at, option}`: pick an option of a choice.
    Choose {
        /// The choice.
        at: String,
        /// The option's value or label.
        option: String,
    },
    /// `act: <path>`: run an action, or an open overlay's primary action.
    Act {
        /// The action or overlay.
        at: String,
    },
    /// `page: {at, to}`: move a collection to a page, from 1.
    Page {
        /// The collection.
        at: String,
        /// The page.
        to: usize,
    },
    /// `expect: {at, text?, not_text?, rows?, state?}`.
    Expect {
        /// The node.
        at: String,
        /// What must hold.
        checks: Vec<Check>,
    },
    /// `play: <event>` or `play: {event | lifecycle, channel?, with?}`.
    Play(Play),
    /// `advance: <duration>`.
    Advance(Duration),
    /// `expect_command: <command>` or `expect_command: {command, input?}`.
    ExpectCommand {
        /// The command.
        command: String,
        /// Input fields the command must have been sent with.
        input: BTreeMap<String, Value>,
    },
}

impl Step {
    /// The step's keyword.
    pub fn keyword(&self) -> &'static str {
        match self {
            Self::Open { .. } => "open",
            Self::Select { .. } => "select",
            Self::Type { .. } => "type",
            Self::Choose { .. } => "choose",
            Self::Act { .. } => "act",
            Self::Page { .. } => "page",
            Self::Expect { .. } => "expect",
            Self::Play(_) => "play",
            Self::Advance(_) => "advance",
            Self::ExpectCommand { .. } => "expect_command",
        }
    }
}

impl fmt::Display for Step {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open { page, .. } => write!(formatter, "open {page}"),
            Self::Select { at } => write!(formatter, "select {at}"),
            Self::Type { at, text } => write!(formatter, "type {text:?} at {at}"),
            Self::Choose { at, option } => write!(formatter, "choose {option:?} at {at}"),
            Self::Act { at } => write!(formatter, "act {at}"),
            Self::Page { at, to } => write!(formatter, "page {to} of {at}"),
            Self::Expect { at, .. } => write!(formatter, "expect at {at}"),
            Self::Play(play) => write!(formatter, "play {play}"),
            Self::Advance(by) => write!(formatter, "advance {}ms", by.as_millis()),
            Self::ExpectCommand { command, .. } => write!(formatter, "expect_command {command}"),
        }
    }
}

/// One expectation of an `expect` step.
#[derive(Debug, Clone, PartialEq)]
pub enum Check {
    /// The node shows this text.
    Text(String),
    /// The node does not show this text.
    NotText(String),
    /// The collection shows this many rows.
    RowCount(usize),
    /// The collection shows the rows with these keys, in this order.
    RowKeys(Vec<String>),
    /// The section is in this state.
    State(SectionState),
}

/// The state of a section, as a reader sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionState {
    /// An `on_demand` section not yet asked for.
    NotLoaded,
    /// Its read has not answered.
    Loading,
    /// Its read answered with no rows.
    Empty,
    /// Its read was refused.
    Failed,
    /// A channel feeding it is stale.
    Stale,
    /// It shows its data, fresh.
    Ready,
}

impl SectionState {
    const ALL: [(&'static str, Self); 6] = [
        ("not_loaded", Self::NotLoaded),
        ("loading", Self::Loading),
        ("empty", Self::Empty),
        ("failed", Self::Failed),
        ("stale", Self::Stale),
        ("ready", Self::Ready),
    ];

    /// The state's name in a test file.
    pub fn name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(_, state)| *state == self)
            .map_or("ready", |(name, _)| name)
    }
}

/// What a `play` step plays.
#[derive(Debug, Clone, PartialEq)]
pub struct Play {
    /// The channel; any channel whose script has the event when absent.
    pub channel: Option<String>,
    /// An event or a lifecycle beat.
    pub beat: PlayBeat,
    /// Payload fields the event must carry.
    pub with: BTreeMap<String, Value>,
}

/// An event name or a lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayBeat {
    /// `event: <name>`.
    Event(String),
    /// `lifecycle: <state>`.
    Lifecycle(String),
}

impl fmt::Display for Play {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.beat {
            PlayBeat::Event(name) => write!(formatter, "{name}")?,
            PlayBeat::Lifecycle(state) => write!(formatter, "lifecycle {state}")?,
        }
        if let Some(channel) = &self.channel {
            write!(formatter, " on {channel}")?;
        }
        for (key, value) in &self.with {
            write!(formatter, " {key}={}", scalar(value))?;
        }
        Ok(())
    }
}

/// A scalar as text; other values as compact YAML.
pub(crate) fn scalar(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => String::new(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim_end()
            .to_owned(),
    }
}

/// Parses a test file's text; `source` names it in errors.
pub fn parse_str(text: &str, source: &Path) -> Result<TestFile, TestError> {
    let at = |message: String| TestError(format!("{}: {message}", source.display()));
    let value: Value = serde_yaml::from_str(text).map_err(|error| at(error.to_string()))?;
    let root = value
        .as_mapping()
        .ok_or_else(|| at("a test file is a map".into()))?;
    known(root, &["format", "document", "tests"]).map_err(at)?;
    let format = root.get("format").and_then(Value::as_str);
    if format != Some(FORMAT) {
        return Err(at(format!(
            "`format` is `{FORMAT}`, not `{}`",
            format.unwrap_or("")
        )));
    }
    let document = string(root, "document").map_err(at)?;
    let tests = root
        .get("tests")
        .and_then(Value::as_sequence)
        .ok_or_else(|| at("`tests` is a list".into()))?;
    let mut parsed = Vec::new();
    for (index, test) in tests.iter().enumerate() {
        parsed.push(parse_test(test).map_err(|message| {
            let name = test
                .get("name")
                .and_then(Value::as_str)
                .map_or_else(String::new, |name| format!(" ({name})"));
            at(format!("tests[{}]{name}: {message}", index + 1))
        })?);
    }
    Ok(TestFile {
        source: source.to_path_buf(),
        document,
        tests: parsed,
    })
}

/// Parses a YAML list of steps.
pub fn parse_steps(text: &str) -> Result<Vec<Step>, TestError> {
    let value: Value = serde_yaml::from_str(text).map_err(|error| TestError(error.to_string()))?;
    let steps = value
        .as_sequence()
        .ok_or_else(|| TestError("steps are a list".into()))?;
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            parse_step(step).map_err(|message| TestError(format!("step {}: {message}", index + 1)))
        })
        .collect()
}

fn parse_test(value: &Value) -> Result<Test, String> {
    let test = value.as_mapping().ok_or("a test is a map")?;
    known(test, &["name", "fixtures", "latency", "steps"])?;
    let name = string(test, "name")?;
    let fixtures = match test.get("fixtures") {
        None => Fixtures::default(),
        Some(value) => parse_fixtures(value).map_err(|message| format!("fixtures: {message}"))?,
    };
    let latency = match test.get("latency") {
        None => Duration::ZERO,
        Some(value) => duration(value)?,
    };
    let steps = test
        .get("steps")
        .and_then(Value::as_sequence)
        .ok_or("`steps` is a list")?;
    let steps = steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            parse_step(step).map_err(|message| format!("step {}: {message}", index + 1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Test {
        name,
        fixtures,
        latency,
        steps,
    })
}

fn parse_fixtures(value: &Value) -> Result<Fixtures, String> {
    let map = value.as_mapping().ok_or("a map of `views` and `scripts`")?;
    known(map, &["views", "scripts"])?;
    let mut fixtures = Fixtures::default();
    if let Some(views) = map.get("views") {
        for (view, data) in views.as_mapping().ok_or("`views` is a map")? {
            let view = view.as_str().ok_or("a view name is a string")?;
            if data.get("rows").and_then(Value::as_sequence).is_none() {
                return Err(format!("views.{view} lists `rows`"));
            }
            fixtures.views.insert(view.to_owned(), data.clone());
        }
    }
    if let Some(scripts) = map.get("scripts") {
        for (channel, script) in scripts.as_mapping().ok_or("`scripts` is a map")? {
            let channel = channel.as_str().ok_or("a channel name is a string")?;
            ess_ui_tui::live::Script::parse(channel, script)
                .map_err(|message| format!("scripts.{channel}: {message}"))?;
            fixtures.scripts.insert(channel.to_owned(), script.clone());
        }
    }
    Ok(fixtures)
}

fn parse_step(value: &Value) -> Result<Step, String> {
    let map = value.as_mapping().ok_or("a step is a map with one key")?;
    let mut entries = map.iter();
    let (Some((keyword, body)), None) = (entries.next(), entries.next()) else {
        return Err("a step is a map with one key".into());
    };
    let keyword = keyword.as_str().ok_or("a step's key is its keyword")?;
    let at = |fields: &Mapping| string(fields, "at");
    match keyword {
        "open" => parse_open(body),
        "select" | "act" => {
            let at = match body {
                Value::String(path) => path.clone(),
                Value::Mapping(fields) => {
                    known(fields, &["at"])?;
                    at(fields)?
                }
                _ => return Err(format!("`{keyword}` names a node path")),
            };
            Ok(if keyword == "act" {
                Step::Act { at }
            } else {
                Step::Select { at }
            })
        }
        "type" => {
            let fields = body.as_mapping().ok_or("`type` is {at, text}")?;
            known(fields, &["at", "text"])?;
            Ok(Step::Type {
                at: at(fields)?,
                text: scalar_field(fields, "text")?,
            })
        }
        "choose" => {
            let fields = body.as_mapping().ok_or("`choose` is {at, option}")?;
            known(fields, &["at", "option"])?;
            Ok(Step::Choose {
                at: at(fields)?,
                option: scalar_field(fields, "option")?,
            })
        }
        "page" => {
            let fields = body.as_mapping().ok_or("`page` is {at, to}")?;
            known(fields, &["at", "to"])?;
            let to = fields
                .get("to")
                .and_then(Value::as_u64)
                .filter(|to| *to >= 1)
                .ok_or("`to` is a page number from 1")?;
            Ok(Step::Page {
                at: at(fields)?,
                to: usize::try_from(to).map_err(|error| error.to_string())?,
            })
        }
        "expect" => parse_expect(body),
        "play" => parse_play(body).map(Step::Play),
        "advance" => duration(body).map(Step::Advance),
        "expect_command" => match body {
            Value::String(command) => Ok(Step::ExpectCommand {
                command: command.clone(),
                input: BTreeMap::new(),
            }),
            Value::Mapping(fields) => {
                known(fields, &["command", "input"])?;
                let mut input = BTreeMap::new();
                if let Some(given) = fields.get("input") {
                    for (name, value) in given.as_mapping().ok_or("`input` is a map")? {
                        let name = name.as_str().ok_or("an input name is a string")?;
                        input.insert(name.to_owned(), value.clone());
                    }
                }
                Ok(Step::ExpectCommand {
                    command: string(fields, "command")?,
                    input,
                })
            }
            _ => Err("`expect_command` names a command".into()),
        },
        other => Err(format!(
            "`{other}` is not a step; a step is one of {}",
            crate::STEPS.join(", ")
        )),
    }
}

fn parse_open(body: &Value) -> Result<Step, String> {
    match body {
        Value::String(page) => Ok(Step::Open {
            page: page.clone(),
            params: BTreeMap::new(),
        }),
        Value::Mapping(fields) => {
            known(fields, &["page", "params"])?;
            let mut params = BTreeMap::new();
            if let Some(given) = fields.get("params") {
                for (name, value) in given.as_mapping().ok_or("`params` is a map")? {
                    let name = name.as_str().ok_or("a param name is a string")?;
                    params.insert(name.to_owned(), scalar(value));
                }
            }
            Ok(Step::Open {
                page: string(fields, "page")?,
                params,
            })
        }
        _ => Err("`open` names a page".into()),
    }
}

fn parse_expect(body: &Value) -> Result<Step, String> {
    let fields = body
        .as_mapping()
        .ok_or("`expect` is {at, text | not_text | rows | state}")?;
    known(fields, &["at", "text", "not_text", "rows", "state"])?;
    let mut checks = Vec::new();
    if fields.contains_key("text") {
        checks.push(Check::Text(scalar_field(fields, "text")?));
    }
    if fields.contains_key("not_text") {
        checks.push(Check::NotText(scalar_field(fields, "not_text")?));
    }
    match fields.get("rows") {
        None => {}
        Some(Value::Number(count)) => checks.push(Check::RowCount(
            count
                .as_u64()
                .and_then(|count| usize::try_from(count).ok())
                .ok_or("`rows` is a count or a list of row keys")?,
        )),
        Some(Value::Sequence(keys)) => {
            checks.push(Check::RowKeys(keys.iter().map(scalar).collect()));
        }
        Some(_) => return Err("`rows` is a count or a list of row keys".into()),
    }
    if let Some(state) = fields.get("state") {
        let name = state.as_str().unwrap_or_default();
        let state = SectionState::ALL
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, state)| *state)
            .ok_or_else(|| {
                format!(
                    "`{}` is not a section state; one of {}",
                    scalar(state),
                    SectionState::ALL
                        .iter()
                        .map(|(name, _)| *name)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        checks.push(Check::State(state));
    }
    if checks.is_empty() {
        return Err("`expect` checks one of text, not_text, rows, state".into());
    }
    Ok(Step::Expect {
        at: string(fields, "at")?,
        checks,
    })
}

fn parse_play(body: &Value) -> Result<Play, String> {
    match body {
        Value::String(event) => Ok(Play {
            channel: None,
            beat: PlayBeat::Event(event.clone()),
            with: BTreeMap::new(),
        }),
        Value::Mapping(fields) => {
            known(fields, &["event", "lifecycle", "channel", "with"])?;
            let channel = fields
                .get("channel")
                .map(|channel| {
                    channel
                        .as_str()
                        .map(str::to_owned)
                        .ok_or("`channel` is a name")
                })
                .transpose()?;
            let beat = match (fields.get("event"), fields.get("lifecycle")) {
                (Some(event), None) => PlayBeat::Event(scalar(event)),
                (None, Some(state)) if channel.is_some() => PlayBeat::Lifecycle(scalar(state)),
                (None, Some(_)) => return Err("a lifecycle beat names its `channel`".into()),
                _ => return Err("`play` has exactly one of `event` and `lifecycle`".into()),
            };
            let mut with = BTreeMap::new();
            if let Some(given) = fields.get("with") {
                for (name, value) in given.as_mapping().ok_or("`with` is a map")? {
                    let name = name.as_str().ok_or("a payload field is a string")?;
                    with.insert(name.to_owned(), value.clone());
                }
            }
            Ok(Play {
                channel,
                beat,
                with,
            })
        }
        _ => Err("`play` names an event".into()),
    }
}

fn duration(value: &Value) -> Result<Duration, String> {
    let text = scalar(value);
    ess_ui_tui::live::parse_duration(&text)
        .ok_or_else(|| format!("`{text}` is not a duration (500ms, 12s, 5m, 1h)"))
}

fn known(map: &Mapping, keys: &[&str]) -> Result<(), String> {
    for key in map.keys() {
        let key = key.as_str().unwrap_or_default();
        if !keys.contains(&key) {
            return Err(format!(
                "unknown key `{key}`; expected one of {}",
                keys.join(", ")
            ));
        }
    }
    Ok(())
}

fn string(map: &Mapping, key: &str) -> Result<String, String> {
    map.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("`{key}` is a string"))
}

fn scalar_field(map: &Mapping, key: &str) -> Result<String, String> {
    match map.get(key) {
        Some(value @ (Value::String(_) | Value::Number(_) | Value::Bool(_))) => Ok(scalar(value)),
        _ => Err(format!("`{key}` is text")),
    }
}
