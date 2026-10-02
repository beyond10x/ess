//! The terminal application: navigation, pages, overlays, prompts, reads, live events and state.
//!
//! Nothing here draws; [`crate::view`] renders an [`App`] into a ratatui frame.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ess_ui::binding::{Answer, Binding};
use ess_ui::{
    Action, ActionConfirm, Body, Columns, Composite, Document, Effect, Field, FilterBar, Form,
    GuardThen, Live, NavPages, NodePath, Overlay, Page, PagedAway, Reads, Section, TabFields,
};
use serde_yaml::{Mapping, Value};

use crate::data::{DataAdapter, FixtureAdapter, ReadRequest, ReadResult};
use crate::expr::{self, display, truthy, Resolve, Resolved};
use crate::live::{derived_field, parse_duration, Beat, ChannelState, Played, Player, Script};
use crate::placement::StateStore;
use crate::profile::{self, Plan, TUI};
use crate::TuiError;

/// How a run is set up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Replaces the document's fixture directory.
    pub fixtures: Option<PathBuf>,
    /// Where file-placed state is kept, under a directory named after the app.
    pub state_dir: PathBuf,
    /// Rows per collection page.
    pub page_size: usize,
    /// How long a read takes on the virtual clock.
    pub read_latency: Duration,
}

impl Options {
    /// Defaults with state kept under `state_dir`.
    pub fn new(state_dir: PathBuf) -> Self {
        Self {
            fixtures: None,
            state_dir,
            page_size: 10,
            read_latency: Duration::ZERO,
        }
    }
}

/// Where a node was drawn on the last frame ([`App::regions`]).
///
/// Recorded are the page header and its actions, every drawn section box, the children of a
/// section, the open overlay, and for a collection that is a section's or an overlay's body: its
/// column headers (a table's only), each drawn row line, each cell of a row (not for cards, a
/// list or a tree with an `item`, whose cells the generated React project does not render) and
/// each row action a row offers. A node not listed was not drawn on its own cells (or not drawn
/// at all).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    /// The node's canonical path (`ess_ui::NodePath`); a row and the cells inside it are scoped as
    /// `<collection>/rows/<key>` and `<collection>/rows/<key>/columns/<column>`, the path the
    /// generated React project renders as `data-ui-path`.
    pub path: String,
    /// The row key (the section's `live.match` field, else `id`), for a row and its cells.
    pub row: Option<String>,
    /// The screen cells the node occupies.
    pub area: ratatui::layout::Rect,
    /// The node's whole text where the screen may cut it or does not show it: a row's cells, a
    /// cell's value, a column's label, an action's label. `None`: the text is what `area` shows.
    pub text: Option<String>,
}

/// A node's place inside a list of lines, before the list is placed on the screen.
#[derive(Debug, Clone)]
pub(crate) struct Mark {
    pub path: String,
    pub row: Option<String>,
    pub line: usize,
    pub height: usize,
    pub x: usize,
    /// `None`: to the right edge.
    pub width: Option<usize>,
    /// [`Region::text`].
    pub text: Option<String>,
}

/// The lifecycle state a section is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// An `on_demand` section not yet asked for.
    NotLoaded,
    /// Its read has not answered.
    Loading,
    /// Its read answered with rows (or it has no read).
    Ready,
    /// Its read answered with no rows.
    Empty,
    /// Its read was refused.
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReadState {
    Loading { since: Duration },
    Ready(ReadResult),
    Failed(String),
}

#[derive(Debug, Clone)]
struct CacheEntry {
    request: ReadRequest,
    state: ReadState,
    /// When the read last answered.
    answered: Duration,
}

/// A command that was not done, shown where the user acted.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Refused {
    /// What sent it: `form`, `confirm`, or the name of the action.
    pub by: String,
    /// What the answer means to the user ([`crate::http::shown`]).
    pub text: String,
    /// The declared error's fields, when it has any.
    pub payload: Option<Value>,
    /// The effect stands though the answer is a failure (`501` with `committed: true`): the
    /// command is done and must not be sent again.
    pub committed: bool,
}

impl Refused {
    /// The same refusal, sent by `by`.
    fn by(mut self, by: &str) -> Self {
        by.clone_into(&mut self.by);
        self
    }
}

/// A number field's value as a number when its text reads as one; anything else unchanged.
fn number(value: Value) -> Value {
    let Value::String(text) = &value else {
        return value;
    };
    let text = text.trim();
    if text.is_empty() {
        return Value::Null;
    }
    if let Ok(integer) = text.parse::<i64>() {
        return Value::Number(integer.into());
    }
    match text.parse::<f64>() {
        Ok(float) if float.is_finite() => Value::Number(float.into()),
        _ => value,
    }
}

/// How often each `live:` section of a bound document polls, by its node path: the served
/// surface streams no events, so the section takes its `no_live` fallback and polls at its read's
/// `refresh:` (5 s without one), unless it says `degrades: {no_live: refuse}`.
fn poll_intervals(document: &Document) -> Result<BTreeMap<String, Duration>, profile::Refusal> {
    let mut polls = BTreeMap::new();
    for (page_name, page) in &document.pages {
        for section in &page.sections {
            if section.live.is_none() {
                continue;
            }
            let at = NodePath::root()
                .child("pages")
                .child(page_name)
                .child("sections")
                .child(&section.name);
            if section.common.degrades.get("no_live").map(String::as_str) == Some("refuse") {
                return Err(profile::Refusal {
                    path: at.child("live"),
                    message: "the served surface streams no events, and this section's \
                              `degrades: {no_live: refuse}` refuses to poll instead"
                        .to_owned(),
                });
            }
            let every = match body_reads(&section.body).and_then(|reads| reads.refresh.as_ref()) {
                None => POLL,
                Some(refresh) => {
                    let at = at.child("reads").child("refresh");
                    let Some(every) = parse_duration(&refresh.0) else {
                        return Err(profile::Refusal {
                            path: at,
                            message: format!(
                                "a live section bound to the served surface polls at its \
                                 `refresh:`, and `{}` is not a whole number of ms, s, m or h \
                                 the terminal can poll at",
                                refresh.0
                            ),
                        });
                    };
                    if !(POLL_RANGE.0..=POLL_RANGE.1).contains(&every) {
                        return Err(profile::Refusal {
                            path: at,
                            message: format!(
                                "a live section bound to the served surface polls at its \
                                 `refresh:`, and {} ms is outside 1 s to 24 h",
                                every.as_millis()
                            ),
                        });
                    }
                    every
                }
            };
            polls.insert(at.to_string(), every);
        }
    }
    Ok(polls)
}

/// How often a bound `live:` section reads again when its read declares no `refresh:`.
const POLL: Duration = Duration::from_secs(5);

/// The shortest and the longest `refresh:` a bound `live:` section polls at.
const POLL_RANGE: (Duration, Duration) = (Duration::from_secs(1), Duration::from_secs(24 * 3600));

/// Where keyboard input goes when no overlay or prompt takes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Nav,
    Section(usize),
}

/// Per-view interaction state (component state, held in memory).
#[derive(Debug, Clone, Default)]
pub(crate) struct Ui {
    pub cursor: usize,
    pub page: usize,
    pub filter: String,
    pub sort: Option<(String, bool)>,
    pub expanded: bool,
    pub item: usize,
    pub option: usize,
    pub editing: bool,
    pub tab: usize,
    pub new_rows: usize,
    pub selected: BTreeSet<String>,
    pub typed: String,
    pub pending: Vec<(String, Value)>,
    /// The last command sent from here that was not done.
    pub refusal: Option<Refused>,
}

/// An open overlay.
#[derive(Debug, Clone)]
pub(crate) struct OpenOverlay {
    pub name: String,
    pub path: NodePath,
    pub overlay: Overlay,
    pub params: BTreeMap<String, Value>,
    pub then: Option<(Action, Option<Value>)>,
    /// The confirm's own command was accepted: confirming again retries only `then`.
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Prompt {
    Palette(String),
    Filter {
        ui: String,
        state: Option<String>,
        text: String,
    },
}

/// One entry of the navigation pane.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NavEntry {
    pub label: String,
    pub page: String,
    pub params: BTreeMap<String, Value>,
    pub synonyms: Vec<String>,
}

/// One heading of the navigation pane with its entries.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NavGroup {
    pub label: String,
    pub entries: Vec<NavEntry>,
}

/// What evaluation can see besides the page: a row, the section, the overlay and a draft.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Ctx<'a> {
    pub row: Option<&'a Value>,
    pub section: Option<&'a str>,
    pub overlay: bool,
    pub draft: Option<&'a str>,
}

/// Which view keys act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Section(usize),
    Overlay,
}

/// Letters no row action may take, because navigation uses them.
const RESERVED: &str = "jknpsSxqgRJK/: []";

/// A running `ess-ui/1` document.
pub struct App {
    pub(crate) doc: Document,
    pub(crate) plan: Plan,
    pub(crate) options: Options,
    adapter: Box<dyn DataAdapter>,
    player: Player,
    pub(crate) store: StateStore,
    pub(crate) now: Duration,
    pub(crate) page: String,
    pub(crate) params: BTreeMap<String, Value>,
    history: Vec<(String, BTreeMap<String, Value>)>,
    pub(crate) focus: Focus,
    pub(crate) nav_cursor: usize,
    pub(crate) uis: BTreeMap<String, Ui>,
    pub(crate) overlay: Option<OpenOverlay>,
    pub(crate) prompt: Option<Prompt>,
    pending_g: bool,
    pub(crate) notifications: Vec<String>,
    cache: BTreeMap<String, CacheEntry>,
    pub(crate) channels: BTreeMap<String, ChannelState>,
    demanded: BTreeSet<String>,
    batches: BTreeMap<String, (Duration, Vec<Value>)>,
    deferred: Vec<(String, Value)>,
    signed_out: bool,
    quit: bool,
    /// The served surface the run is bound to ([`App::bound`]); `None` for a fixture run.
    pub(crate) bound: Option<Binding>,
    /// How often each bound `live:` section polls, by its node path.
    polls: BTreeMap<String, Duration>,
    /// How many commands have been sent.
    pub(crate) commands_sent: u64,
    /// Marks of the lines being drawn, not yet placed on the screen.
    pub(crate) marks: std::cell::RefCell<Vec<Mark>>,
    /// What the last frame drew where.
    pub(crate) regions: std::cell::RefCell<Vec<Region>>,
}

impl App {
    /// Loads a document file and opens its home page.
    pub fn from_path(path: &Path, options: Options) -> Result<Self, TuiError> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| TuiError::Io(format!("cannot read {}: {error}", path.display())))?;
        let base = path.parent().unwrap_or(Path::new("."));
        Self::from_text(&text, base, options)
    }

    /// Loads a document from text, resolving fixtures against `base`, and opens its home page.
    pub fn from_text(text: &str, base: &Path, options: Options) -> Result<Self, TuiError> {
        let document = ess_ui::load_str(text).map_err(TuiError::Load)?;
        profile::check(&document, &TUI).map_err(TuiError::Refused)?;
        let (adapter, scripts) =
            FixtureAdapter::load(&document, base, options.fixtures.as_deref())?;
        Self::with_adapter(document, Box::new(adapter), scripts, options)
    }

    /// Runs `document` against any data adapter, playing `scripts` on the virtual clock.
    pub fn with_adapter(
        document: Document,
        adapter: Box<dyn DataAdapter>,
        scripts: Vec<Script>,
        options: Options,
    ) -> Result<Self, TuiError> {
        Self::open(document, adapter, scripts, options, None)
    }

    /// Runs `document` against the served surface `binding` describes, through `adapter` (an
    /// [`crate::HttpAdapter`] over the same binding). No fixture channel plays: the served
    /// surface streams no events, so a `live:` section polls its read every `refresh:` (5 s
    /// without one), refused at `<section>/reads/refresh` when that is not a duration from 1 s to
    /// 24 h, and at `<section>/live` when the section says `degrades: {no_live: refuse}`. A
    /// command's answer is shown as the binding declares its errors; an accepted one, or a `409`
    /// refusal, reads every view again.
    pub fn bound(
        document: Document,
        adapter: Box<dyn DataAdapter>,
        binding: Binding,
        options: Options,
    ) -> Result<Self, TuiError> {
        Self::open(document, adapter, Vec::new(), options, Some(binding))
    }

    fn open(
        document: Document,
        adapter: Box<dyn DataAdapter>,
        scripts: Vec<Script>,
        options: Options,
        bound: Option<Binding>,
    ) -> Result<Self, TuiError> {
        let plan = profile::check(&document, &TUI).map_err(TuiError::Refused)?;
        let polls = if bound.is_some() {
            poll_intervals(&document).map_err(TuiError::Refused)?
        } else {
            BTreeMap::new()
        };
        let placements = crate::placement::resolve(&document).map_err(TuiError::Refused)?;
        let home = document.navigation.home.clone();
        let (user_id, account_id) = actor_ids(&document, adapter.as_ref());
        let store = StateStore::new(
            placements,
            &options.state_dir,
            &document.app,
            user_id.as_deref(),
            account_id.as_deref(),
        );
        let mut app = Self {
            doc: document,
            plan,
            options,
            adapter,
            player: Player::new(scripts),
            store,
            now: Duration::ZERO,
            page: String::new(),
            params: BTreeMap::new(),
            history: Vec::new(),
            focus: Focus::Nav,
            nav_cursor: 0,
            uis: BTreeMap::new(),
            overlay: None,
            prompt: None,
            pending_g: false,
            notifications: Vec::new(),
            cache: BTreeMap::new(),
            channels: BTreeMap::new(),
            demanded: BTreeSet::new(),
            batches: BTreeMap::new(),
            deferred: Vec::new(),
            signed_out: false,
            quit: false,
            bound,
            polls,
            commands_sent: 0,
            marks: std::cell::RefCell::default(),
            regions: std::cell::RefCell::default(),
        };
        app.go(&home, BTreeMap::new(), true);
        Ok(app)
    }

    // ── public surface ──────────────────────────────────────────────────────────────────────

    /// The page shown.
    pub fn page(&self) -> &str {
        &self.page
    }

    /// Every page of the document.
    pub fn page_names(&self) -> Vec<String> {
        self.doc.pages.keys().cloned().collect()
    }

    /// The overlays reachable from the page shown: its own and its shell's.
    pub fn overlay_names(&self) -> Vec<String> {
        let page = self.page_def();
        let mut names: Vec<String> = page.overlays.keys().cloned().collect();
        if let Some(shell) = self.doc.shells.get(&page.shell) {
            names.extend(shell.overlays.keys().cloned());
        }
        names
    }

    /// The TUI's location: the page, its params and its url-placed state.
    pub fn location(&self) -> String {
        let mut query: Vec<String> = self
            .params
            .iter()
            .map(|(name, value)| format!("{name}={}", location_value(value)))
            .collect();
        for (name, value) in self.store.query(&format!("pages/{}", self.page)) {
            query.push(format!("{name}={}", location_value(&value)));
        }
        if query.is_empty() {
            format!("/{}", self.page)
        } else {
            format!("/{}?{}", self.page, query.join("&"))
        }
    }

    /// The rows a section of the page shown holds now, live changes applied.
    pub fn rows(&self, section: &str) -> Vec<Value> {
        self.section_by_name(section)
            .and_then(|section| self.section_request(section))
            .and_then(|request| match self.read_state(&request) {
                Some(ReadState::Ready(result)) => Some(result.rows.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Where the last [`App::draw`] drew each node it records ([`Region`]), in drawing order.
    /// Empty before the first draw. Read-only: drawing again replaces it.
    pub fn regions(&self) -> Vec<Region> {
        self.regions.borrow().clone()
    }

    /// Every read the app holds whose answer was a failure, as `(view, error)` in request-key
    /// order.
    pub fn failed_reads(&self) -> Vec<(String, String)> {
        self.cache
            .values()
            .filter_map(|entry| match &entry.state {
                ReadState::Failed(error) => Some((entry.request.view.clone(), error.clone())),
                _ => None,
            })
            .collect()
    }

    /// The lifecycle state of a section of the page shown.
    pub fn section_state(&self, section: &str) -> Lifecycle {
        self.section_by_name(section)
            .map_or(Lifecycle::Failed, |section| self.lifecycle(section))
    }

    /// Opens a page; params the page does not declare are dropped.
    pub fn open_page(&mut self, page: &str, params: &[(&str, &str)]) {
        let declared = self
            .doc
            .pages
            .get(page)
            .map(|page| page.params.keys().cloned().collect::<BTreeSet<_>>())
            .unwrap_or_default();
        let params = params
            .iter()
            .filter(|(name, _)| declared.contains(*name))
            .map(|(name, value)| ((*name).to_owned(), Value::String((*value).to_owned())))
            .collect();
        self.go(page, params, true);
        self.pump();
    }

    /// Opens an overlay of the page shown (or of its shell), passing the page's params.
    pub fn open_overlay(&mut self, name: &str) {
        let params = self.params.clone();
        self.show_overlay(name, None, Some(params), None);
        self.pump();
    }

    /// Moves focus to a section of the page shown.
    pub fn focus_section(&mut self, name: &str) {
        if let Some(index) = self
            .page_def()
            .sections
            .iter()
            .position(|section| section.name == name)
        {
            self.focus = Focus::Section(index);
        }
    }

    /// Whether the user asked to quit.
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Types a key sequence (see [`crate::keys::parse`]).
    pub fn keys(&mut self, spec: &str) {
        for key in crate::keys::parse(spec) {
            self.key(key);
        }
    }

    /// Handles one key press.
    pub fn key(&mut self, key: KeyEvent) {
        self.handle(key);
        self.pump();
    }

    /// Moves the virtual clock, plays every script entry now due and answers due reads.
    pub fn advance(&mut self, by: Duration) {
        self.now += by;
        for played in self.player.due(self.now) {
            self.flush(played.at);
            self.deliver(played);
        }
        self.flush(self.now);
        self.pump();
    }

    // ── document lookups ────────────────────────────────────────────────────────────────────

    pub(crate) fn page_def(&self) -> &Page {
        &self.doc.pages[&self.page]
    }

    pub(crate) fn page_path(&self) -> NodePath {
        NodePath::root().child("pages").child(&self.page)
    }

    pub(crate) fn section_path(&self, name: &str) -> NodePath {
        self.page_path().child("sections").child(name)
    }

    fn section_by_name(&self, name: &str) -> Option<&Section> {
        self.page_def()
            .sections
            .iter()
            .find(|section| section.name == name)
    }

    pub(crate) fn visible_sections(&self) -> Vec<(usize, &Section)> {
        self.page_def()
            .sections
            .iter()
            .enumerate()
            .filter(|(_, section)| {
                let ctx = Ctx {
                    section: Some(&section.name),
                    ..Ctx::default()
                };
                self.visible(
                    section.common.visible.as_ref().map(|expr| expr.0.as_str()),
                    &ctx,
                )
            })
            .collect()
    }

    pub(crate) fn has_navigation(&self) -> bool {
        self.doc
            .shells
            .get(&self.page_def().shell)
            .is_some_and(|shell| {
                shell
                    .regions
                    .values()
                    .any(|region| region.kind == ess_ui::RegionKind::Navigation)
            })
    }

    pub(crate) fn nav(&self) -> Vec<NavGroup> {
        let mut groups = Vec::new();
        for section in &self.doc.navigation.sections {
            let mut entries = Vec::new();
            match &section.pages {
                NavPages::Fixed(pages) => {
                    for page in pages {
                        let Some(def) = self.doc.pages.get(page) else {
                            continue;
                        };
                        let nav = def.nav.as_ref();
                        entries.push(NavEntry {
                            label: nav
                                .and_then(|nav| nav.label.clone())
                                .or_else(|| def.title.clone())
                                .unwrap_or_else(|| page.clone()),
                            page: page.clone(),
                            params: BTreeMap::new(),
                            synonyms: nav.map(|nav| nav.synonyms.clone()).unwrap_or_default(),
                        });
                    }
                }
                NavPages::Dynamic(dynamic) => {
                    let request = ReadRequest {
                        view: dynamic.from_view.clone(),
                        fixture: None,
                        params: BTreeMap::new(),
                    };
                    if let Some(ReadState::Ready(result)) = self.read_state(&request) {
                        for row in &result.rows {
                            let ctx = Ctx {
                                row: Some(row),
                                ..Ctx::default()
                            };
                            let label = dynamic
                                .label
                                .as_ref()
                                .and_then(|label| self.eval(&label.0, &ctx))
                                .map_or_else(|| display(&row["id"]), |value| display(&value));
                            let mut params = BTreeMap::new();
                            params.insert(dynamic.param.clone(), row["id"].clone());
                            entries.push(NavEntry {
                                label,
                                page: dynamic.page.clone(),
                                params,
                                synonyms: dynamic.synonyms.clone(),
                            });
                        }
                    }
                }
            }
            groups.push(NavGroup {
                label: section
                    .label
                    .clone()
                    .unwrap_or_else(|| section.name.clone()),
                entries,
            });
        }
        groups
    }

    // ── evaluation ──────────────────────────────────────────────────────────────────────────

    pub(crate) fn eval(&self, text: &str, ctx: &Ctx<'_>) -> Option<Value> {
        expr::eval(text, &Scope { app: self, ctx })
    }

    /// A `visible` condition: absent or outside the fixture grammar means shown.
    pub(crate) fn visible(&self, condition: Option<&str>, ctx: &Ctx<'_>) -> bool {
        condition
            .and_then(|condition| self.eval(condition, ctx))
            .is_none_or(|value| truthy(&value))
    }

    fn state_path(&self, name: &str, ctx: &Ctx<'_>) -> String {
        let mut candidates = Vec::new();
        if ctx.overlay {
            if let Some(open) = &self.overlay {
                candidates.push(open.path.child("state").child(name));
            }
        }
        if let Some(section) = ctx.section {
            candidates.push(self.section_path(section).child("state").child(name));
        }
        let page = self.page_path().child("state").child(name);
        candidates
            .into_iter()
            .map(|path| path.to_string())
            .find(|path| self.store.placement(path).is_some())
            .unwrap_or_else(|| page.to_string())
    }

    fn shell_state_path(&self, name: &str) -> String {
        NodePath::root()
            .child("shells")
            .child(&self.page_def().shell)
            .child("state")
            .child(name)
            .to_string()
    }

    /// Writes the state an expression like `state.search` names.
    fn set_bound(&mut self, target: &str, value: Value, ctx: &Ctx<'_>) {
        let segments: Vec<&str> = target.split('.').collect();
        let path = match segments.as_slice() {
            ["state", name, ..] => self.state_path(name, ctx),
            ["shell", name, ..] => self.shell_state_path(name),
            ["draft", field] => {
                if let Some(draft) = ctx.draft {
                    let mut current = self.store.get(draft, self.adapter.as_ref());
                    if !current.is_mapping() {
                        current = Value::Mapping(Mapping::new());
                    }
                    current[*field] = value;
                    let draft = draft.to_owned();
                    self.store.set(&draft, current, self.adapter.as_mut());
                }
                return;
            }
            _ => return,
        };
        self.store.set(&path, value, self.adapter.as_mut());
    }

    pub(crate) fn channel_status(&self, channel: &str) -> String {
        let stale_after = self
            .doc
            .channels
            .get(channel)
            .and_then(|channel| channel.stale_after.as_deref())
            .and_then(parse_duration);
        // A session-scoped channel reports the session of the page shown; a page outside the
        // scripted session has no connection of its own playing.
        self.channels
            .get(channel)
            .filter(|state| self.shows_session(channel, state.session.as_ref()))
            .map_or("connecting", |state| state.effective(self.now, stale_after))
            .to_owned()
    }

    // ── reads ───────────────────────────────────────────────────────────────────────────────

    pub(crate) fn request(&self, reads: &Reads, ctx: &Ctx<'_>) -> ReadRequest {
        let mut params = BTreeMap::new();
        for (name, value) in &reads.params {
            if let Some(value) = self.eval(&value.0, ctx) {
                if !value.is_null() {
                    params.insert(name.clone(), value);
                }
            }
        }
        ReadRequest {
            view: reads
                .view
                .clone()
                .or_else(|| reads.placeholder.clone())
                .unwrap_or_default(),
            fixture: reads.fixture.clone(),
            params,
        }
    }

    pub(crate) fn read_state(&self, request: &ReadRequest) -> Option<&ReadState> {
        self.cache.get(&request.key()).map(|entry| &entry.state)
    }

    pub(crate) fn rows_of(&self, request: &ReadRequest) -> Option<&ReadResult> {
        match self.read_state(request) {
            Some(ReadState::Ready(result)) => Some(result),
            _ => None,
        }
    }

    pub(crate) fn section_request(&self, section: &Section) -> Option<ReadRequest> {
        let ctx = Ctx {
            section: Some(&section.name),
            ..Ctx::default()
        };
        body_reads(&section.body).map(|reads| self.request(reads, &ctx))
    }

    pub(crate) fn lifecycle(&self, section: &Section) -> Lifecycle {
        if section.load == Some(ess_ui::Load::OnDemand) && !self.demanded.contains(&section.name) {
            return Lifecycle::NotLoaded;
        }
        let Some(request) = self.section_request(section) else {
            return Lifecycle::Ready;
        };
        match self.read_state(&request) {
            None | Some(ReadState::Loading { .. }) => Lifecycle::Loading,
            Some(ReadState::Failed(_)) => Lifecycle::Failed,
            Some(ReadState::Ready(result)) => {
                let listed = matches!(
                    section.body,
                    Body::Composite(
                        Composite::Collection(_)
                            | Composite::References(_)
                            | Composite::Board(_)
                            | Composite::Chart(_)
                            | Composite::GraphEditor(_)
                    )
                );
                if listed && result.rows.is_empty() {
                    Lifecycle::Empty
                } else {
                    Lifecycle::Ready
                }
            }
        }
    }

    /// Enqueues every read the screen needs and answers those whose latency has passed.
    fn pump(&mut self) {
        self.poll();
        for _ in 0..8 {
            let mut changed = false;
            for request in self.needed_reads() {
                let key = request.key();
                if !self.cache.contains_key(&key) {
                    self.cache.insert(
                        key,
                        CacheEntry {
                            request,
                            state: ReadState::Loading { since: self.now },
                            answered: self.now,
                        },
                    );
                    changed = true;
                }
            }
            let due: Vec<String> = self
                .cache
                .iter()
                .filter(|(_, entry)| {
                    matches!(entry.state, ReadState::Loading { since }
                        if since + self.options.read_latency <= self.now)
                })
                .map(|(key, _)| key.clone())
                .collect();
            for key in due {
                let entry = self.cache.get_mut(&key).expect("a due read is cached");
                entry.state = match self.adapter.read(&entry.request) {
                    Ok(result) => ReadState::Ready(result),
                    Err(error) => ReadState::Failed(error),
                };
                entry.answered = self.now;
                changed = true;
            }
            if !changed {
                break;
            }
        }
        if !self.deferred.is_empty() {
            self.apply_deferred();
        }
    }

    /// Reads each shown bound `live:` section again once its interval has passed since its read
    /// last answered. A read still in flight is not started again; one that stalls is given up by
    /// the adapter ([`crate::http::TIMEOUT`]) and answers as failed, so the next tick reads.
    fn poll(&mut self) {
        if self.polls.is_empty() {
            return;
        }
        let due: Vec<String> = self
            .visible_sections()
            .into_iter()
            .filter_map(|(_, section)| {
                let every = self
                    .polls
                    .get(&self.section_path(&section.name).to_string())?;
                let key = self.section_request(section)?.key();
                let entry = self.cache.get(&key)?;
                let settled = !matches!(entry.state, ReadState::Loading { .. });
                (settled && self.now.saturating_sub(entry.answered) >= *every).then_some(key)
            })
            .collect();
        for key in due {
            if let Some(entry) = self.cache.get_mut(&key) {
                entry.state = ReadState::Loading { since: self.now };
            }
        }
    }

    /// Drops every cached read, so what the screen shows is read again.
    fn invalidate(&mut self) {
        self.cache.clear();
    }

    fn needed_reads(&self) -> Vec<ReadRequest> {
        let mut out = Vec::new();
        let page = self.page_def();
        if let Some(shell) = self.doc.shells.get(&page.shell) {
            if let Some(preload) = &shell.preload {
                if !preload.except_on.contains(&self.page) {
                    for view in &preload.views {
                        let mut params = BTreeMap::new();
                        for (name, value) in &view.params {
                            if let Some(value) = self.eval(&value.0, &Ctx::default()) {
                                params.insert(name.clone(), value);
                            }
                        }
                        out.push(ReadRequest {
                            view: view.view.clone(),
                            fixture: None,
                            params,
                        });
                    }
                }
            }
        }
        if self.has_navigation() {
            for section in &self.doc.navigation.sections {
                if let NavPages::Dynamic(dynamic) = &section.pages {
                    out.push(ReadRequest {
                        view: dynamic.from_view.clone(),
                        fixture: None,
                        params: BTreeMap::new(),
                    });
                }
            }
        }
        if let Some(header) = &page.header {
            for node in &header.metrics {
                self.node_reads(&node.body, "header", &Ctx::default(), &mut out);
            }
        }
        for (_, section) in self.visible_sections() {
            if self.lifecycle(section) == Lifecycle::NotLoaded {
                continue;
            }
            let ctx = Ctx {
                section: Some(&section.name),
                ..Ctx::default()
            };
            let ui = format!("s:{}", section.name);
            self.node_reads(&section.body, &ui, &ctx, &mut out);
            for child in &section.children {
                self.node_reads(&child.body, &ui, &ctx, &mut out);
            }
        }
        if let Some(open) = &self.overlay {
            let ctx = Ctx {
                overlay: true,
                ..Ctx::default()
            };
            self.node_reads(
                &open.overlay.body,
                &format!("o:{}", open.name),
                &ctx,
                &mut out,
            );
        }
        out
    }

    fn node_reads(&self, body: &Body, ui: &str, ctx: &Ctx<'_>, out: &mut Vec<ReadRequest>) {
        let each = |nodes: &[ess_ui::Node], ctx: &Ctx<'_>, out: &mut Vec<ReadRequest>| {
            for node in nodes {
                self.node_reads(&node.body, ui, ctx, out);
            }
        };
        match body {
            Body::Widget(widget) => each(&widget.body, ctx, out),
            Body::Primitive(_) => {}
            Body::Composite(composite) => {
                if let Some(reads) = body_reads(body) {
                    out.push(self.request(reads, ctx));
                }
                match composite {
                    Composite::Collection(collection) => {
                        if let Some(expand) = &collection.expand {
                            if self.ui(ui).expanded {
                                if let Some(row) = self.selected_row(ui, collection, ctx) {
                                    let inner = Ctx {
                                        row: Some(&row),
                                        ..*ctx
                                    };
                                    self.node_reads(&expand.body, ui, &inner, out);
                                }
                            }
                        }
                    }
                    Composite::Record(record) => {
                        let request = record.reads.as_ref().map(|reads| self.request(reads, ctx));
                        let row = request
                            .as_ref()
                            .and_then(|request| self.rows_of(request))
                            .and_then(|result| result.rows.first().cloned());
                        let inner = Ctx {
                            row: row.as_ref().or(ctx.row),
                            ..*ctx
                        };
                        each(&record.item, &inner, out);
                    }
                    Composite::Form(form) => {
                        for field in form_fields(form, 0) {
                            if let Some(choice) = &field.choice {
                                self.node_reads(&choice.body, ui, ctx, out);
                            }
                        }
                        for tab in &form.tabs {
                            if let Some(TabFields::Fields(fields)) = &tab.fields {
                                for field in fields {
                                    if let Some(choice) = &field.choice {
                                        self.node_reads(&choice.body, ui, ctx, out);
                                    }
                                }
                            }
                        }
                        each(&form.parts, ctx, out);
                        for node in [&form.record, &form.result].into_iter().flatten() {
                            self.node_reads(&node.body, ui, ctx, out);
                        }
                    }
                    Composite::FilterBar(bar) => each(&bar.choices, ctx, out),
                    Composite::Confirm(confirm) => {
                        if let Some(view) = &confirm.references {
                            out.push(ReadRequest {
                                view: view.clone(),
                                fixture: None,
                                params: self.overlay_params(),
                            });
                        }
                    }
                    Composite::Board(board) => {
                        for node in board.widgets.values() {
                            self.node_reads(&node.body, ui, ctx, out);
                        }
                    }
                    Composite::GraphEditor(editor) => each(&editor.toolbar, ctx, out),
                    Composite::RichText(text) => {
                        if let Some(view) = &text.completes {
                            out.push(ReadRequest {
                                view: view.clone(),
                                fixture: None,
                                params: BTreeMap::new(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn overlay_params(&self) -> BTreeMap<String, Value> {
        self.overlay
            .as_ref()
            .map(|open| open.params.clone())
            .unwrap_or_default()
    }

    // ── collections ─────────────────────────────────────────────────────────────────────────

    pub(crate) fn ui(&self, id: &str) -> Ui {
        self.uis.get(id).cloned().unwrap_or_default()
    }

    fn ui_mut(&mut self, id: &str) -> &mut Ui {
        self.uis.entry(id.to_owned()).or_default()
    }

    /// The rows of a collection after its local filter and sort, before paging.
    pub(crate) fn arranged_rows(
        &self,
        ui: &str,
        collection: &ess_ui::Collection,
        rows: &[Value],
    ) -> Vec<Value> {
        let state = self.ui(ui);
        let columns = columns_of(collection, rows);
        let needle = state.filter.to_lowercase();
        let mut rows: Vec<Value> = rows
            .iter()
            .filter(|row| {
                needle.is_empty()
                    || columns.iter().any(|field| {
                        display(&row[field.field.as_str()])
                            .to_lowercase()
                            .contains(&needle)
                    })
            })
            .cloned()
            .collect();
        let sort = state.sort.clone().or_else(|| {
            collection
                .sort
                .as_ref()
                .map(|sort| (sort.by.clone(), sort.dir == Some(ess_ui::SortDir::Desc)))
        });
        if let Some((by, descending)) = sort {
            rows.sort_by(|left, right| {
                let order = compare(&left[by.as_str()], &right[by.as_str()]);
                if descending {
                    order.reverse()
                } else {
                    order
                }
            });
        }
        if let Some(group) = &collection.group_by {
            rows.sort_by_key(|row| display(&row[group.as_str()]));
        }
        rows
    }

    /// The rows of the collection page shown.
    pub(crate) fn page_rows(
        &self,
        ui: &str,
        collection: &ess_ui::Collection,
        ctx: &Ctx<'_>,
    ) -> (Vec<Value>, usize) {
        let Some(reads) = &collection.reads else {
            return (Vec::new(), 0);
        };
        let request = self.request(reads, ctx);
        let Some(result) = self.rows_of(&request) else {
            return (Vec::new(), 0);
        };
        let rows = self.arranged_rows(ui, collection, &result.rows);
        let size = self.options.page_size.max(1);
        let pages = rows.len().div_ceil(size).max(1);
        let page = self.ui(ui).page.min(pages - 1);
        (
            rows.into_iter().skip(page * size).take(size).collect(),
            pages,
        )
    }

    pub(crate) fn selected_row(
        &self,
        ui: &str,
        collection: &ess_ui::Collection,
        ctx: &Ctx<'_>,
    ) -> Option<Value> {
        let (rows, _) = self.page_rows(ui, collection, ctx);
        let cursor = self.ui(ui).cursor.min(rows.len().saturating_sub(1));
        rows.get(cursor).cloned()
    }

    /// The row actions of a collection that apply to `row`, with their keys.
    pub(crate) fn action_keys(&self, actions: &[Action], ctx: &Ctx<'_>) -> Vec<(char, Action)> {
        let mut taken = String::new();
        let mut keyed = Vec::new();
        for action in actions {
            if !self.visible(action.visible.as_ref().map(|expr| expr.0.as_str()), ctx) {
                continue;
            }
            let key = action
                .name
                .chars()
                .filter(char::is_ascii_lowercase)
                .chain('1'..='9')
                .find(|key| !RESERVED.contains(*key) && !taken.contains(*key));
            if let Some(key) = key {
                taken.push(key);
                keyed.push((key, action.clone()));
            }
        }
        keyed
    }

    // ── live ────────────────────────────────────────────────────────────────────────────────

    fn deliver(&mut self, played: Played) {
        let resume = self
            .doc
            .channels
            .get(&played.channel)
            .map(|channel| channel.resume.clone());
        let state = self.channels.entry(played.channel.clone()).or_default();
        if played.session.is_some() {
            state.session.clone_from(&played.session);
        }
        let (name, payload) = match played.beat {
            Beat::Lifecycle(status) => {
                let reconnected =
                    status == "live" && matches!(state.status.as_str(), "reconnecting" | "stale");
                state.status = status;
                state.since = played.at;
                if reconnected && resume == Some(ess_ui::Resume::Refetch) {
                    self.refetch_fed(&played.channel);
                }
                return;
            }
            Beat::Event { name, payload } => (name, payload),
        };
        // An event is the channel live again; one that ends a reconnect resumes as a `live`
        // beat would.
        let reconnected = matches!(state.status.as_str(), "reconnecting" | "stale");
        if state.status != "live" {
            state.status = "live".into();
            state.since = played.at;
        }
        state.latest = Some(payload.clone());
        state.arrived.insert(name.clone(), played.at);
        if reconnected && resume == Some(ess_ui::Resume::Refetch) {
            self.refetch_fed(&played.channel);
        }
        if let Some(ess_ui::Carries::View(carried)) = self
            .doc
            .channels
            .get(&played.channel)
            .map(|channel| &channel.carries)
        {
            for entry in self.cache.values_mut() {
                if entry.request.view == carried.view {
                    entry.state = ReadState::Ready(ReadResult {
                        rows: vec![payload.clone()],
                        total: None,
                    });
                }
            }
        }
        let mut targets = Vec::new();
        for (_, section) in self.visible_sections() {
            let Some(live) = &section.live else { continue };
            if live.channel != played.channel || !(live.on.is_empty() || live.on.contains(&name)) {
                continue;
            }
            let Some(request) = self.section_request(section) else {
                continue;
            };
            if !self.in_session(&played.channel, played.session.as_ref(), &request) {
                continue;
            }
            targets.push((section.name.clone(), live.clone()));
        }
        for (section, live) in targets {
            // A reader paged away sees a count, not the rows, so there is no burst to batch.
            let paged_away = self.ui(&format!("s:{section}")).page > 0;
            match live.coalesce.as_deref().and_then(parse_duration) {
                Some(window) if window > Duration::ZERO && !paged_away => {
                    let batch = self
                        .batches
                        .entry(section)
                        .or_insert_with(|| (played.at + window, Vec::new()));
                    batch.1.push(payload.clone());
                }
                _ => self.apply_live(&section, &live, &payload),
            }
        }
    }

    /// Applies every coalesced batch whose window closed at or before `until`, as one change.
    fn flush(&mut self, until: Duration) {
        let due: Vec<String> = self
            .batches
            .iter()
            .filter(|(_, (deadline, _))| *deadline <= until)
            .map(|(section, _)| section.clone())
            .collect();
        for section in due {
            let Some((_, payloads)) = self.batches.remove(&section) else {
                continue;
            };
            let Some(live) = self
                .section_by_name(&section)
                .and_then(|section| section.live.clone())
            else {
                continue;
            };
            for payload in payloads {
                self.apply_live(&section, &live, &payload);
            }
        }
    }

    /// Whether an event of a session-scoped channel belongs to the section's session: the
    /// script's session value for the channel's `session.per` param must equal the value the
    /// section reads with under that param.
    fn in_session(&self, channel: &str, session: Option<&Value>, request: &ReadRequest) -> bool {
        let Some(per) = self
            .doc
            .channels
            .get(channel)
            .and_then(|channel| channel.session.as_ref())
            .map(|session| session.per.as_str())
        else {
            return true;
        };
        let Some(scripted) = session.and_then(|session| session.get(per)) else {
            // Fail closed: a session that does not name the `per` param belongs to no page.
            return false;
        };
        request
            .params
            .get(per)
            .is_some_and(|value| display(value) == display(scripted))
    }

    /// Whether the page shown belongs to a session-scoped channel's scripted session: one of its
    /// sections reads with the channel's `session.per` param at the scripted value.
    pub(crate) fn shows_session(&self, channel: &str, session: Option<&Value>) -> bool {
        let Some(per) = self
            .doc
            .channels
            .get(channel)
            .and_then(|channel| channel.session.as_ref())
            .map(|session| session.per.as_str())
        else {
            return true;
        };
        let Some(scripted) = session.and_then(|session| session.get(per)) else {
            // Fail closed, as `in_session` does.
            return false;
        };
        self.visible_sections()
            .into_iter()
            .filter_map(|(_, section)| self.section_request(section))
            .any(|request| {
                request
                    .params
                    .get(per)
                    .is_some_and(|value| display(value) == display(scripted))
            })
    }

    /// Re-reads every section of the page shown that the channel feeds, and every cached read
    /// of the view it carries.
    fn refetch_fed(&mut self, channel: &str) {
        let carried = self
            .doc
            .channels
            .get(channel)
            .and_then(|channel| match &channel.carries {
                ess_ui::Carries::View(view) => Some(view.view.clone()),
                ess_ui::Carries::Events(_) => None,
            });
        let mut keys: Vec<String> = self
            .visible_sections()
            .into_iter()
            .filter(|(_, section)| {
                section
                    .live
                    .as_ref()
                    .is_some_and(|live| live.channel == channel)
            })
            .filter_map(|(_, section)| self.section_request(section))
            .map(|request| request.key())
            .collect();
        keys.extend(
            self.cache
                .iter()
                .filter(|(_, entry)| Some(&entry.request.view) == carried.as_ref())
                .map(|(key, _)| key.clone()),
        );
        let now = self.now;
        for key in keys {
            if let Some(entry) = self.cache.get_mut(&key) {
                entry.state = ReadState::Loading { since: now };
            }
        }
    }

    /// Applies one live event to a section of the page shown.
    fn apply_live(&mut self, section: &str, live: &Live, payload: &Value) {
        let Some(request) = self
            .section_by_name(section)
            .and_then(|section| self.section_request(section))
        else {
            return;
        };
        let key = request.key();
        let match_field = live.match_field.as_deref().unwrap_or("id");
        let identity = payload.get(match_field).map(display);
        let ui = format!("s:{section}");
        let paged_away = self.ui(&ui).page > 0;
        // A row already held while paged away takes the event into the held row: one row, one
        // count, its latest state.
        if paged_away {
            if let Some(identity) = &identity {
                if let Some(held) = self
                    .ui_mut(&ui)
                    .pending
                    .iter_mut()
                    .find(|(held, _)| held == identity)
                {
                    merge(&mut held.1, payload);
                    return;
                }
            }
        }
        let result = match self.read_state(&request) {
            Some(ReadState::Ready(result)) => result,
            // A read outstanding (a resume refetch, a new filter): apply once it has answered.
            None | Some(ReadState::Loading { .. }) => {
                self.deferred.push((section.to_owned(), payload.clone()));
                return;
            }
            Some(ReadState::Failed(_)) => return,
        };
        let existing = identity.as_ref().and_then(|identity| {
            result
                .rows
                .iter()
                .find(|row| row.get(match_field).map(display).as_ref() == Some(identity))
        });
        // The event is judged as the row it would leave behind: the current row patched by it.
        let mut candidate = existing.cloned().unwrap_or(Value::Mapping(Mapping::new()));
        merge(&mut candidate, payload);
        if !self.passes(live, &request, &candidate) {
            return;
        }
        let inserts =
            existing.is_none() && matches!(live.effect, Effect::InsertOrPatch | Effect::InsertTop);
        if inserts && paged_away {
            match live.when_paged_away {
                Some(PagedAway::Insert) => {}
                Some(PagedAway::Ignore) => return,
                // `count_new` (and no declaration): count the row now, insert it when the
                // reader returns to the first page.
                _ => {
                    let state = self.ui_mut(&ui);
                    let identity = identity.unwrap_or_else(|| format!("#{}", state.pending.len()));
                    state.pending.push((identity, payload.clone()));
                    state.new_rows = state.pending.len();
                    return;
                }
            }
        }
        let now = self.now;
        let Some(entry) = self.cache.get_mut(&key) else {
            return;
        };
        let ReadState::Ready(result) = &mut entry.state else {
            return;
        };
        apply(live, &mut result.rows, payload);
        if live.effect == Effect::Refetch {
            entry.state = ReadState::Loading { since: now };
        }
    }

    /// Applies the live events that waited for a read, in arrival order; those whose read is
    /// still outstanding wait again.
    fn apply_deferred(&mut self) {
        for (section, payload) in std::mem::take(&mut self.deferred) {
            let Some(live) = self
                .section_by_name(&section)
                .and_then(|section| section.live.clone())
            else {
                continue;
            };
            self.apply_live(&section, &live, &payload);
        }
    }

    /// Puts a collection back on its first page, releasing the rows held while paged away.
    fn reset_page(&mut self, ui: &str) {
        let state = self.ui_mut(ui);
        state.page = 0;
        state.cursor = 0;
        self.insert_pending(ui);
    }

    /// Writes a collection's selected ids to the `selection`-class state of its section, else of
    /// its page, so bulk actions can bind it (`bind: {ids: state.selected}`).
    fn write_selection(&mut self, section: Option<&str>, ids: &BTreeSet<String>) {
        let page = self.page_def();
        let in_section = section
            .and_then(|name| {
                page.sections
                    .iter()
                    .find(|candidate| candidate.name == name)
            })
            .and_then(|section| {
                section
                    .common
                    .state
                    .iter()
                    .find(|(_, state)| state.class == ess_ui::StateClass::Selection)
                    .map(|(key, _)| self.section_path(&section.name).child("state").child(key))
            });
        let path = in_section.or_else(|| {
            page.state
                .iter()
                .find(|(_, state)| state.class == ess_ui::StateClass::Selection)
                .map(|(key, _)| self.page_path().child("state").child(key))
        });
        if let Some(path) = path {
            let value = Value::Sequence(ids.iter().cloned().map(Value::String).collect());
            self.store
                .set(&path.to_string(), value, self.adapter.as_mut());
        }
    }

    /// Inserts the rows counted while the reader was paged away.
    fn insert_pending(&mut self, ui: &str) {
        let pending = std::mem::take(&mut self.ui_mut(ui).pending);
        self.ui_mut(ui).new_rows = 0;
        let Some(section) = ui.strip_prefix("s:").map(str::to_owned) else {
            return;
        };
        let Some(live) = self
            .section_by_name(&section)
            .and_then(|section| section.live.clone())
        else {
            return;
        };
        // Oldest first: each held row already carries its latest state.
        for (_, payload) in &pending {
            self.apply_live(&section, &live, payload);
        }
    }

    /// `only_if`. `matches(params)` holds when the row satisfies every filter param the
    /// section reads with, judged as the fixture read judges them: `q` searches the row's text,
    /// a list narrows the field of the same name, a scalar must equal a field it names.
    fn passes(&self, live: &Live, request: &ReadRequest, row: &Value) -> bool {
        let Some(condition) = &live.only_if else {
            return true;
        };
        if condition.0.starts_with("matches(") {
            return request.params.iter().all(|(name, wanted)| {
                if name == "q" {
                    let needle = display(wanted).to_lowercase();
                    return needle.is_empty() || display(row).to_lowercase().contains(&needle);
                }
                let Some(have) = row.get(name.as_str()) else {
                    return true;
                };
                match wanted {
                    Value::Sequence(items) if items.is_empty() => true,
                    Value::Sequence(items) => {
                        items.iter().any(|item| display(item) == display(have))
                    }
                    _ => display(wanted).is_empty() || display(wanted) == display(have),
                }
            });
        }
        let ctx = Ctx {
            row: Some(row),
            ..Ctx::default()
        };
        self.eval(&condition.0, &ctx)
            .is_none_or(|value| truthy(&value))
    }

    // ── navigation ──────────────────────────────────────────────────────────────────────────

    fn go(&mut self, page: &str, params: BTreeMap<String, Value>, remember: bool) {
        if !self.doc.pages.contains_key(page) {
            self.notify(format!("no page {page}"));
            return;
        }
        let previous = (
            std::mem::take(&mut self.page),
            std::mem::take(&mut self.params),
        );
        page.clone_into(&mut self.page);
        self.params = params;
        if let Some(stop) = self.run_guards() {
            self.page = previous.0;
            self.params = previous.1;
            match stop {
                GuardStop::Refused(message) => self.notify(message),
                GuardStop::Redirect(target) if target != page => {
                    self.go(&target, BTreeMap::new(), remember);
                }
                GuardStop::Redirect(_) => {}
            }
            return;
        }
        if remember && !previous.0.is_empty() {
            self.history.push(previous);
        }
        self.overlay = None;
        self.uis
            .retain(|id, _| !id.starts_with("s:") && !id.starts_with("o:"));
        self.demanded.clear();
        self.batches.clear();
        self.deferred.clear();
        // A view cache in memory is lost on unmount: the next page reads again.
        if crate::placement::class_default(&self.doc, &ess_ui::StateClass::ViewCache)
            .is_none_or(|store| store == ess_ui::Store::Memory)
        {
            self.cache.clear();
        }
        self.focus = if self.page_def().sections.is_empty() {
            Focus::Nav
        } else {
            Focus::Section(0)
        };
        let page = self.page.clone();
        if let Some(index) = self
            .nav()
            .iter()
            .flat_map(|group| group.entries.iter())
            .position(|entry| entry.page == page)
        {
            self.nav_cursor = index;
        }
        self.pump();
    }

    /// Runs the shell's guards for the page just set. `Some` stops the navigation.
    fn run_guards(&mut self) -> Option<GuardStop> {
        let guards = self
            .doc
            .shells
            .get(&self.page_def().shell)
            .map(|shell| shell.guards.clone())
            .unwrap_or_default();
        for guard in guards {
            let holds = self
                .eval(&guard.when.0, &Ctx::default())
                .is_some_and(|value| truthy(&value));
            if !holds {
                continue;
            }
            match guard.then {
                GuardThen::Redirect(redirect) if redirect.redirect != self.page => {
                    return Some(GuardStop::Redirect(redirect.redirect));
                }
                GuardThen::Refuse(refuse) => return Some(GuardStop::Refused(refuse.refuse)),
                GuardThen::Set(set) => {
                    for (target, value) in set.set {
                        let value = self.eval(&value.0, &Ctx::default()).unwrap_or(Value::Null);
                        self.set_bound(&target, value, &Ctx::default());
                    }
                }
                GuardThen::Redirect(_) => {}
            }
        }
        None
    }

    fn back(&mut self) {
        if let Some((page, params)) = self.history.pop() {
            self.go(&page, params, false);
        }
    }

    fn goto_initial(&mut self, letter: char) {
        let groups = self.nav();
        let letter = letter.to_ascii_lowercase();
        let matching: Vec<usize> = groups
            .iter()
            .enumerate()
            .filter(|(_, group)| {
                group.label.to_lowercase().starts_with(letter) && !group.entries.is_empty()
            })
            .map(|(index, _)| index)
            .collect();
        let current = groups
            .iter()
            .position(|group| group.entries.iter().any(|entry| entry.page == self.page));
        let pick =
            match current.and_then(|current| matching.iter().position(|index| *index == current)) {
                Some(at) => matching[(at + 1) % matching.len()],
                None => match matching.first() {
                    Some(first) => *first,
                    None => return,
                },
            };
        let entry = groups[pick].entries[0].clone();
        self.go(&entry.page, entry.params, true);
    }

    fn cycle_focus(&mut self, forward: bool) {
        let mut order: Vec<Focus> = Vec::new();
        if self.has_navigation() {
            order.push(Focus::Nav);
        }
        order.extend(
            self.visible_sections()
                .into_iter()
                .map(|(index, _)| Focus::Section(index)),
        );
        if order.is_empty() {
            return;
        }
        let at = order
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        let len = order.len();
        let next = if forward {
            (at + 1) % len
        } else {
            (at + len - 1) % len
        };
        self.focus = order[next];
    }

    pub(crate) fn notify(&mut self, message: String) {
        self.notifications.push(message);
    }

    // ── key handling ────────────────────────────────────────────────────────────────────────

    fn handle(&mut self, key: KeyEvent) {
        if self.prompt.is_some() {
            self.prompt_key(key);
            return;
        }
        if self.pending_g {
            self.pending_g = false;
            if let KeyCode::Char(letter) = key.code {
                self.goto_initial(letter);
            }
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if self.overlay.is_some() {
            self.overlay_key(key);
            return;
        }
        if let Focus::Section(index) = self.focus {
            if self.ui(&self.target_ui(Target::Section(index))).editing {
                self.edit_key(Target::Section(index), key);
                return;
            }
        }
        match key.code {
            KeyCode::Char(':') => self.prompt = Some(Prompt::Palette(String::new())),
            KeyCode::Char('g') => self.pending_g = true,
            KeyCode::Tab => self.cycle_focus(true),
            KeyCode::BackTab => self.cycle_focus(false),
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Backspace => self.back(),
            KeyCode::Char('R') => self.reload_focused(),
            _ => match self.focus {
                Focus::Nav => self.nav_key(key),
                Focus::Section(index) => self.target_key(Target::Section(index), key),
            },
        }
    }

    fn reload_focused(&mut self) {
        let Focus::Section(index) = self.focus else {
            return;
        };
        let Some(section) = self.page_def().sections.get(index).cloned() else {
            return;
        };
        self.demanded.insert(section.name.clone());
        if let Some(request) = self.section_request(&section) {
            self.cache.remove(&request.key());
        }
    }

    fn nav_key(&mut self, key: KeyEvent) {
        let entries: Vec<NavEntry> = self
            .nav()
            .into_iter()
            .flat_map(|group| group.entries)
            .collect();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.nav_cursor = (self.nav_cursor + 1).min(entries.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => self.nav_cursor = self.nav_cursor.saturating_sub(1),
            KeyCode::Enter => {
                if let Some(entry) = entries.get(self.nav_cursor).cloned() {
                    self.go(&entry.page, entry.params, true);
                }
            }
            _ => {}
        }
    }

    fn target_ui(&self, target: Target) -> String {
        match target {
            Target::Section(index) => self
                .page_def()
                .sections
                .get(index)
                .map_or_else(String::new, |section| format!("s:{}", section.name)),
            Target::Overlay => self
                .overlay
                .as_ref()
                .map_or_else(String::new, |open| format!("o:{}", open.name)),
        }
    }

    fn target_body(&self, target: Target) -> Option<(Body, Option<String>)> {
        match target {
            Target::Section(index) => self
                .page_def()
                .sections
                .get(index)
                .map(|section| (section.body.clone(), Some(section.name.clone()))),
            Target::Overlay => self
                .overlay
                .as_ref()
                .map(|open| (open.overlay.body.clone(), None)),
        }
    }

    pub(crate) fn draft_path(&self, section: Option<&str>) -> String {
        match (section, &self.overlay) {
            (Some(section), _) => self.section_path(section).child("draft").to_string(),
            (None, Some(open)) => open.path.child("draft").to_string(),
            (None, None) => self.page_path().child("draft").to_string(),
        }
    }

    fn target_key(&mut self, target: Target, key: KeyEvent) {
        let Some((body, section)) = self.target_body(target) else {
            return;
        };
        let ui = self.target_ui(target);
        let draft = self.draft_path(section.as_deref());
        let Body::Composite(composite) = body else {
            return;
        };
        let overlay = target == Target::Overlay;
        match composite {
            Composite::Collection(collection) => {
                self.collection_key(&ui, &collection, section.as_deref(), overlay, key);
            }
            Composite::GraphEditor(editor) => {
                let collection = graph_collection(&editor);
                self.collection_key(&ui, &collection, section.as_deref(), overlay, key);
            }
            Composite::References(references) => {
                let collection = references_collection(&references);
                self.collection_key(&ui, &collection, section.as_deref(), overlay, key);
            }
            Composite::Form(form) => self.form_key(target, &ui, &form, &draft, key),
            Composite::FilterBar(bar) => self.bar_key(&ui, &bar, section.as_deref(), key),
            Composite::Record(record) => {
                if let KeyCode::Char(letter) = key.code {
                    let ctx = Ctx {
                        section: section.as_deref(),
                        overlay,
                        ..Ctx::default()
                    };
                    if let Some((_, action)) = self
                        .action_keys(&record.actions, &ctx)
                        .into_iter()
                        .find(|(key, _)| *key == letter)
                    {
                        self.run_action_at(&ui, &action, None);
                    }
                }
            }
            Composite::Confirm(confirm) => self.confirm_key(&ui, &confirm, key),
            Composite::RichText(text) => {
                if key.code == KeyCode::Enter && text.binds.is_some() {
                    self.ui_mut(&ui).editing = true;
                }
            }
            Composite::Board(board) => {
                let rows = board_rows(self, &board, section.as_deref());
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => {
                        let state = self.ui_mut(&ui);
                        state.cursor = (state.cursor + 1).min(rows.len().saturating_sub(1));
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        let state = self.ui_mut(&ui);
                        state.cursor = state.cursor.saturating_sub(1);
                    }
                    KeyCode::Char(letter) => {
                        let row = rows.get(self.ui(&ui).cursor).cloned();
                        let ctx = Ctx {
                            row: row.as_ref(),
                            ..Ctx::default()
                        };
                        if let Some((_, action)) = self
                            .action_keys(&board.item_actions, &ctx)
                            .into_iter()
                            .find(|(key, _)| *key == letter)
                        {
                            self.run_action_at(&ui, &action, row);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per collection key
    fn collection_key(
        &mut self,
        ui: &str,
        collection: &ess_ui::Collection,
        section: Option<&str>,
        overlay: bool,
        key: KeyEvent,
    ) {
        let ctx = Ctx {
            section,
            overlay,
            ..Ctx::default()
        };
        let (rows, pages) = self.page_rows(ui, collection, &ctx);
        let row = self.selected_row(ui, collection, &ctx);
        let row_ctx = Ctx {
            row: row.as_ref(),
            ..ctx
        };
        let actions = self.action_keys(&collection.row_actions, &row_ctx);
        let path = section.map(|section| self.section_path(section));
        let moves = path
            .as_ref()
            .and_then(|path| self.plan.fallback(path, "no_drag"))
            == Some("move_buttons");
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let state = self.ui_mut(ui);
                state.cursor = (state.cursor + 1).min(rows.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let state = self.ui_mut(ui);
                state.cursor = state.cursor.saturating_sub(1);
            }
            KeyCode::Char('n') => {
                let state = self.ui_mut(ui);
                if state.page + 1 < pages {
                    state.page += 1;
                    state.cursor = 0;
                }
            }
            KeyCode::Char('p') => {
                let state = self.ui_mut(ui);
                state.page = state.page.saturating_sub(1);
                state.cursor = 0;
                if state.page == 0 {
                    self.insert_pending(ui);
                }
            }
            KeyCode::Char('/') => {
                self.prompt = Some(Prompt::Filter {
                    ui: ui.to_owned(),
                    state: None,
                    text: self.ui(ui).filter,
                });
            }
            KeyCode::Char('s' | 'S') => {
                let mut fields: Vec<String> = collection
                    .sort
                    .as_ref()
                    .map(|sort| sort.allowed.clone())
                    .unwrap_or_default();
                if fields.is_empty() {
                    fields = columns_of(collection, &rows)
                        .into_iter()
                        .map(|field| field.field)
                        .collect();
                }
                let default = collection
                    .sort
                    .as_ref()
                    .map(|sort| (sort.by.clone(), sort.dir == Some(ess_ui::SortDir::Desc)));
                let state = self.ui_mut(ui);
                let current = state.sort.clone().or(default);
                state.sort = if key.code == KeyCode::Char('S') {
                    current.map(|(by, descending)| (by, !descending))
                } else {
                    let at = current
                        .as_ref()
                        .and_then(|(by, _)| fields.iter().position(|field| field == by));
                    let next = at.map_or(0, |at| (at + 1) % fields.len().max(1));
                    fields.get(next).map(|field| (field.clone(), false))
                };
                // A new order starts at the first page.
                self.reset_page(ui);
            }
            KeyCode::Char('x') => {
                let state = self.ui_mut(ui);
                state.expanded = !state.expanded;
            }
            KeyCode::Char(' ') => {
                if let Some(id) = row.as_ref().map(|row| display(&row["id"])) {
                    let state = self.ui_mut(ui);
                    if !state.selected.remove(&id) {
                        state.selected.insert(id);
                    }
                    let selected = state.selected.clone();
                    self.write_selection(section, &selected);
                }
            }
            KeyCode::Char(letter)
                if letter.is_ascii_uppercase()
                    && !matches!(letter, 'J' | 'K')
                    && !self.ui(ui).selected.is_empty() =>
            {
                if let Some((_, action)) = bulk_keys(&collection.bulk_actions)
                    .into_iter()
                    .find(|(key, _)| *key == letter)
                {
                    self.run_action_at(ui, &action, None);
                }
            }
            KeyCode::Char(direction @ ('J' | 'K')) if moves => {
                if let (Some(reorder), Some(row)) = (&collection.reorder, &row) {
                    let mut input = BTreeMap::new();
                    input.insert("id".to_owned(), row["id"].clone());
                    input.insert(
                        "direction".to_owned(),
                        Value::String(if direction == 'J' { "down" } else { "up" }.into()),
                    );
                    if let Some(refused) = self.run_command(&reorder.does, &input) {
                        self.notify(format!("{} refused: {}", reorder.does, refused.text));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some((_, action)) = actions.into_iter().next() {
                    self.run_action_at(ui, &action, row);
                }
            }
            KeyCode::Char(letter) => {
                if let Some((_, action)) = actions.into_iter().find(|(key, _)| *key == letter) {
                    self.run_action_at(ui, &action, row);
                }
            }
            _ => {}
        }
    }

    fn bar_key(&mut self, ui: &str, bar: &FilterBar, section: Option<&str>, key: KeyEvent) {
        let items = bar_items(bar);
        let ctx = Ctx {
            section,
            ..Ctx::default()
        };
        let state = self.ui(ui);
        let current = items
            .get(state.item.min(items.len().saturating_sub(1)))
            .copied();
        match key.code {
            KeyCode::Char('l') | KeyCode::Right => {
                let state = self.ui_mut(ui);
                state.item = (state.item + 1).min(items.len().saturating_sub(1));
                state.option = 0;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                let state = self.ui_mut(ui);
                state.item = state.item.saturating_sub(1);
                state.option = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => self.ui_mut(ui).option += 1,
            KeyCode::Char('k') | KeyCode::Up => {
                let state = self.ui_mut(ui);
                state.option = state.option.saturating_sub(1);
            }
            KeyCode::Char('/') => {
                if let Some(search) = &bar.search {
                    let text = self
                        .eval(&search.binds.0, &ctx)
                        .map(|value| display(&value))
                        .unwrap_or_default();
                    self.prompt = Some(Prompt::Filter {
                        ui: ui.to_owned(),
                        state: Some(search.binds.0.clone()),
                        text,
                    });
                }
            }
            KeyCode::Char(' ') => match current {
                Some(BarItem::Choice(index)) => {
                    let node = &bar.choices[index];
                    let Body::Composite(Composite::Choice(choice)) = &node.body else {
                        return;
                    };
                    let options = self.choice_options(choice, None, &ctx);
                    if options.is_empty() {
                        return;
                    }
                    let picked = options[state.option % options.len()].0.clone();
                    let binds = choice_binds(bar, node, choice);
                    let Some(binds) = binds else { return };
                    let mut value = self.eval(&binds, &ctx).unwrap_or(Value::Null);
                    if choice.multiple {
                        let mut items = value.as_sequence().cloned().unwrap_or_default();
                        if let Some(at) = items
                            .iter()
                            .position(|item| display(item) == display(&picked))
                        {
                            items.remove(at);
                        } else {
                            items.push(picked);
                        }
                        value = Value::Sequence(items);
                    } else {
                        value = picked;
                    }
                    self.set_bound(&binds, value, &ctx);
                }
                Some(BarItem::Window) => {
                    let Some(window) = &bar.window else { return };
                    let presets = self.window_presets(&window.binds.0, &ctx);
                    if presets.is_empty() {
                        return;
                    }
                    let mut value = self.eval(&window.binds.0, &ctx).unwrap_or(Value::Null);
                    if !value.is_mapping() {
                        value = Value::Mapping(Mapping::new());
                    }
                    let current = display(&value["preset"]);
                    let at = presets.iter().position(|preset| *preset == current);
                    let next = at.map_or(0, |at| (at + 1) % presets.len());
                    value["preset"] = Value::String(presets[next].clone());
                    let binds = window.binds.0.clone();
                    self.set_bound(&binds, value, &ctx);
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// The presets a time window's state type allows, read from the document's types.
    pub(crate) fn window_presets(&self, binds: &str, ctx: &Ctx<'_>) -> Vec<String> {
        let Some(name) = binds.strip_prefix("state.") else {
            return Vec::new();
        };
        let path = self.state_path(name, ctx);
        let ty = self
            .doc
            .nodes()
            .into_iter()
            .find_map(|located| match located.node {
                ess_ui::NodeRef::State(state) if located.path.to_string() == path => {
                    Some(state.ty.clone())
                }
                _ => None,
            });
        let mut ty = ty;
        for _ in 0..4 {
            match ty {
                Some(ess_ui::TypeExpr::Named(name)) => ty = self.doc.types.get(&name).cloned(),
                Some(ess_ui::TypeExpr::Record(record)) => {
                    if let Some(ess_ui::TypeExpr::Enum(values)) = record.record.get("preset") {
                        return values.values.clone();
                    }
                    return Vec::new();
                }
                _ => return Vec::new(),
            }
        }
        Vec::new()
    }

    /// The options of a choice: `(value, label)`. A row's value is its `field` key when the
    /// choice belongs to a form field and the row carries one, else its `id`, else the row.
    pub(crate) fn choice_options(
        &self,
        choice: &ess_ui::Choice,
        field: Option<&str>,
        ctx: &Ctx<'_>,
    ) -> Vec<(Value, String)> {
        if !choice.options.is_empty() {
            return choice
                .options
                .iter()
                .map(|option| (option.value.clone(), option.label.clone()))
                .collect();
        }
        let Some(reads) = &choice.reads else {
            return Vec::new();
        };
        let request = self.request(reads, ctx);
        self.rows_of(&request)
            .map(|result| {
                result
                    .rows
                    .iter()
                    .map(|row| {
                        let value = field
                            .and_then(|field| row.get(field))
                            .or_else(|| row.get("id"))
                            .cloned()
                            .unwrap_or_else(|| row.clone());
                        let label = row
                            .get("label")
                            .or_else(|| row.get("name"))
                            .map_or_else(|| display(&value), display);
                        (value, label)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    // ── forms ───────────────────────────────────────────────────────────────────────────────

    /// The value a form field shows: the draft, else the loaded row.
    pub(crate) fn field_value(
        &self,
        form: &Form,
        draft: &str,
        field: &str,
        ctx: &Ctx<'_>,
    ) -> Value {
        let current = self.store.get(draft, self.adapter.as_ref());
        if let Some(value) = current.get(field) {
            return value.clone();
        }
        form.loads
            .as_ref()
            .map(|reads| self.request(reads, ctx))
            .and_then(|request| self.rows_of(&request))
            .and_then(|result| result.rows.first())
            .map_or(Value::Null, |row| row[field].clone())
    }

    fn set_field(&mut self, draft: &str, field: &str, value: Value) {
        let mut current = self.store.get(draft, self.adapter.as_ref());
        if !current.is_mapping() {
            current = Value::Mapping(Mapping::new());
        }
        current[field] = value;
        self.store.set(draft, current, self.adapter.as_mut());
    }

    fn form_key(&mut self, target: Target, ui: &str, form: &Form, draft: &str, key: KeyEvent) {
        let state = self.ui(ui);
        let fields = form_fields(form, state.tab);
        let section = match target {
            Target::Section(index) => self
                .page_def()
                .sections
                .get(index)
                .map(|section| section.name.clone()),
            Target::Overlay => None,
        };
        let ctx = Ctx {
            section: section.as_deref(),
            overlay: target == Target::Overlay,
            ..Ctx::default()
        };
        let current = fields.get(state.item).map(|field| (*field).clone());
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            self.submit(target, form, draft);
            return;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let last = fields.len().saturating_sub(1);
                let state = self.ui_mut(ui);
                state.item = (state.item + 1).min(last);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let state = self.ui_mut(ui);
                state.item = state.item.saturating_sub(1);
            }
            KeyCode::Char(']') => {
                let tabs = form.tabs.len().max(1);
                let state = self.ui_mut(ui);
                state.tab = (state.tab + 1) % tabs;
                state.item = 0;
            }
            KeyCode::Char('[') => {
                let tabs = form.tabs.len().max(1);
                let state = self.ui_mut(ui);
                state.tab = (state.tab + tabs - 1) % tabs;
                state.item = 0;
            }
            KeyCode::Char(' ') => {
                if let Some(field) = current {
                    self.cycle_field(form, draft, &field, &ctx);
                }
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                if let Some(field) = current {
                    if matches!(field.field_as.as_deref(), Some("toggle" | "choice")) {
                        self.cycle_field(form, draft, &field, &ctx);
                    } else {
                        self.ui_mut(ui).editing = true;
                    }
                }
            }
            KeyCode::Char(letter) => {
                if let Some((_, action)) = self
                    .action_keys(&form.actions, &ctx)
                    .into_iter()
                    .find(|(key, _)| *key == letter)
                {
                    self.run_action_at(ui, &action, None);
                }
            }
            _ => {}
        }
    }

    fn cycle_field(&mut self, form: &Form, draft: &str, field: &Field, ctx: &Ctx<'_>) {
        let value = self.field_value(form, draft, &field.field, ctx);
        let next = match field.field_as.as_deref() {
            Some("toggle") => Value::Bool(!truthy(&value)),
            Some("choice") => {
                let Some(Body::Composite(Composite::Choice(choice))) =
                    field.choice.as_ref().map(|node| &node.body)
                else {
                    return;
                };
                let options = self.choice_options(choice, Some(&field.field), ctx);
                if options.is_empty() {
                    return;
                }
                let at = options
                    .iter()
                    .position(|(option, _)| display(option) == display(&value));
                options[at.map_or(0, |at| (at + 1) % options.len())]
                    .0
                    .clone()
            }
            _ => return,
        };
        self.set_field(draft, &field.field, next);
    }

    fn edit_key(&mut self, target: Target, key: KeyEvent) {
        let Some((body, section)) = self.target_body(target) else {
            return;
        };
        let ui = self.target_ui(target);
        let draft = self.draft_path(section.as_deref());
        let state = self.ui(&ui);
        if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
            self.ui_mut(&ui).editing = false;
            return;
        }
        let (field, form) = match &body {
            Body::Composite(Composite::Form(form)) => {
                let fields = form_fields(form, state.tab);
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                    self.submit(target, form, &draft);
                    return;
                }
                match key.code {
                    KeyCode::Tab => {
                        let len = fields.len().max(1);
                        self.ui_mut(&ui).item = (state.item + 1) % len;
                        return;
                    }
                    KeyCode::BackTab => {
                        let len = fields.len().max(1);
                        self.ui_mut(&ui).item = (state.item + len - 1) % len;
                        return;
                    }
                    _ => {}
                }
                let Some(field) = fields.get(state.item) else {
                    return;
                };
                if matches!(field.field_as.as_deref(), Some("toggle" | "choice")) {
                    return;
                }
                (field.field.clone(), Some(form.clone()))
            }
            Body::Composite(Composite::RichText(text)) => {
                let Some(field) = text
                    .binds
                    .as_ref()
                    .and_then(|binds| binds.0.strip_prefix("draft."))
                else {
                    return;
                };
                (field.to_owned(), None)
            }
            _ => return,
        };
        let ctx = Ctx {
            section: section.as_deref(),
            overlay: target == Target::Overlay,
            ..Ctx::default()
        };
        let mut text = match &form {
            Some(form) => display(&self.field_value(form, &draft, &field, &ctx)),
            None => display(&self.store.get(&draft, self.adapter.as_ref())[field.as_str()]),
        };
        match key.code {
            KeyCode::Backspace => {
                text.pop();
            }
            KeyCode::Char(letter) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                text.push(letter);
            }
            _ => return,
        }
        self.set_field(&draft, &field, Value::String(text));
    }

    fn submit(&mut self, target: Target, form: &Form, draft: &str) {
        let mut input = BTreeMap::new();
        let ctx = Ctx {
            overlay: target == Target::Overlay,
            ..Ctx::default()
        };
        for field in form_fields(form, self.ui(&self.target_ui(target)).tab) {
            let mut value = self.field_value(form, draft, &field.field, &ctx);
            // A served command reads a number field as a number, not as the text typed into it.
            if self.bound.is_some() && field.field_as.as_deref() == Some("number") {
                value = number(value);
            }
            if !value.is_null() {
                input.insert(field.field.clone(), value);
            }
        }
        if let Some(Value::String(id)) = self.overlay_params().get("id") {
            input
                .entry("id".to_owned())
                .or_insert_with(|| Value::String(id.clone()));
        }
        let ui = self.target_ui(target);
        self.ui_mut(&ui).editing = false;
        // A refused submit stays on the form that sent it, with its draft. One whose effect was
        // committed is submitted, as an accepted one is, so its draft is never sent again; its
        // failure is still shown, on the form when it stays open.
        let refused = self.run_command(&form.does, &input);
        if let Some(refused) = refused.as_ref().filter(|refused| !refused.committed) {
            self.ui_mut(&ui).refusal = Some(refused.clone().by("form"));
            return;
        }
        self.ui_mut(&ui).refusal = None;
        self.store.set(draft, Value::Null, self.adapter.as_mut());
        let closes = form
            .submit
            .as_ref()
            .and_then(|submit| submit.closes)
            .unwrap_or(true);
        let shut = target == Target::Overlay && closes;
        if shut {
            self.overlay = None;
        }
        if let Some(committed) = refused {
            if shut {
                self.notify(format!("{}: {}", form.does, committed.text));
            } else {
                self.ui_mut(&ui).refusal = Some(committed.by("form"));
            }
        }
    }

    // ── overlays, confirms and actions ──────────────────────────────────────────────────────

    fn overlay_key(&mut self, key: KeyEvent) {
        let ui = self.target_ui(Target::Overlay);
        if self.ui(&ui).editing {
            self.edit_key(Target::Overlay, key);
            return;
        }
        if key.code == KeyCode::Esc {
            self.overlay = None;
            return;
        }
        if key.code == KeyCode::Char(':') {
            self.prompt = Some(Prompt::Palette(String::new()));
            return;
        }
        self.target_key(Target::Overlay, key);
    }

    fn confirm_key(&mut self, ui: &str, confirm: &ess_ui::Confirm, key: KeyEvent) {
        let ctx = Ctx {
            overlay: true,
            ..Ctx::default()
        };
        if let Some(input) = &confirm.input {
            match key.code {
                KeyCode::Backspace => {
                    self.ui_mut(ui).typed.pop();
                }
                KeyCode::Enter => {
                    let wanted = input
                        .must_equal
                        .as_ref()
                        .and_then(|expr| self.eval(&expr.0, &ctx))
                        .map(|value| display(&value))
                        .unwrap_or_default();
                    if self.ui(ui).typed == wanted {
                        self.confirmed(confirm);
                    } else {
                        self.notify(format!("type {wanted} to confirm"));
                    }
                }
                KeyCode::Char(letter) => self.ui_mut(ui).typed.push(letter),
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => self.confirmed(confirm),
            KeyCode::Char('n') => self.overlay = None,
            KeyCode::Char(digit @ '1'..='9') => {
                let index = digit as usize - '1' as usize;
                if let Some(action) = confirm.alternatives.get(index).cloned() {
                    self.overlay = None;
                    self.run_action_notified(&action, None, true);
                }
            }
            _ => {}
        }
    }

    /// Runs a confirm's command, then the action it confirms. A refusal of either keeps the
    /// confirm open and shows on it; nothing after the refused command runs.
    fn confirmed(&mut self, confirm: &ess_ui::Confirm) {
        let Some(mut open) = self.overlay.take() else {
            return;
        };
        let ui = format!("o:{}", open.name);
        let mut refused = None;
        if let Some(does) = confirm.does.as_ref().filter(|_| !open.confirmed) {
            refused = self.run_command(does, &open.params);
            // Accepted: a retry after the action is refused sends only the action again.
            open.confirmed = refused.as_ref().is_none_or(|refused| refused.committed);
        }
        if refused.is_none() {
            if let Some((action, row)) = open.then.clone() {
                refused = self.run_action(&action, row, true);
            }
        }
        if let Some(refused) = refused {
            // Back on the confirm the user acted on, unless the command moved somewhere else.
            if self.overlay.is_none() {
                self.overlay = Some(open);
                self.ui_mut(&ui).refusal = Some(refused.by("confirm"));
            } else {
                self.notify(refused.text);
            }
        }
    }

    /// Records where a command an action sent was refused: beside the action, in the view `ui`.
    fn place_refusal(&mut self, ui: &str, action: &Action, refused: Option<Refused>) {
        if action.does.is_some() {
            self.ui_mut(ui).refusal = refused.map(|refused| refused.by(&action.name));
        }
    }

    /// Sends one command. An accepted one is notified (and, bound, reads every view again);
    /// one that was not done is returned for the caller to show where the user acted.
    fn run_command(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Option<Refused> {
        self.commands_sent += 1;
        let answer = self.adapter.run(command, input);
        if answer != Answer::Accepted {
            // What was read is out of date when the answer says the state moved (a declared
            // `409`) or that the effect was committed though not delivered: read it again.
            let committed =
                self.bound.is_some() && answer == Answer::Unfinished { committed: true };
            if committed || self.wrong_state(command, &answer) {
                self.invalidate();
            }
            let payload = match &answer {
                Answer::Refused {
                    payload: Some(payload),
                    ..
                } => Some(crate::http::to_yaml(payload)),
                _ => None,
            };
            return Some(Refused {
                by: String::new(),
                text: crate::http::shown(self.bound.as_ref(), command, &answer),
                payload,
                committed: answer == Answer::Unfinished { committed: true },
            });
        }
        let message = if self.bound.is_some() {
            self.invalidate();
            format!("{command} accepted")
        } else {
            let shown: Vec<String> = input
                .iter()
                .map(|(name, value)| format!("{name}={}", display(value)))
                .collect();
            if shown.is_empty() {
                format!("{command} accepted (fixture)")
            } else {
                format!("{command} accepted (fixture): {}", shown.join(", "))
            }
        };
        self.notify(message);
        // Fixture sessions: a `*.SignOut` command ends the session, so the shell's guards send
        // the reader to the sign-in page; a `*.SignIn` command starts it again at home.
        match command.rsplit('.').next() {
            Some("SignOut") => {
                self.signed_out = true;
                // Nothing of this session survives it: memory, session storage, sensitive and
                // server_session values, view caches, held and queued rows, open views.
                self.store.sign_out(self.adapter.as_mut());
                self.cache.clear();
                self.uis.clear();
                self.batches.clear();
                self.deferred.clear();
                self.overlay = None;
                self.prompt = None;
                self.history.clear();
                let (page, params) = (self.page.clone(), self.params.clone());
                self.go(&page, params, true);
            }
            Some("SignIn") if self.signed_out => {
                self.signed_out = false;
                let home = self.doc.navigation.home.clone();
                self.go(&home, BTreeMap::new(), true);
            }
            _ => {}
        }
        self.rekey();
        None
    }

    /// Whether a refused `answer` to `command` is its declared `409`: the state it acts on moved.
    fn wrong_state(&self, command: &str, answer: &Answer) -> bool {
        let Answer::Refused { error, .. } = answer else {
            return false;
        };
        self.bound
            .as_ref()
            .and_then(|binding| crate::http::error_status(binding, command, error))
            == Some(409)
    }

    /// Re-keys file storage when the actor (its user or account) has changed, for instance
    /// after a sign-in as someone else or an organization switch.
    fn rekey(&mut self) {
        let (user_id, account_id) = actor_ids(&self.doc, self.adapter.as_ref());
        if self.store.key() != (user_id.as_deref(), account_id.as_deref()) {
            self.store.rekey(user_id.as_deref(), account_id.as_deref());
        }
    }

    /// The actions of the shell's `account_menu` region (its `props.actions`).
    pub(crate) fn account_actions(&self) -> Vec<Action> {
        let Some(shell) = self.doc.shells.get(&self.page_def().shell) else {
            return Vec::new();
        };
        shell
            .regions
            .values()
            .filter(|region| region.kind == ess_ui::RegionKind::AccountMenu)
            .filter(|region| {
                self.visible(
                    region.visible.as_ref().map(|expr| expr.0.as_str()),
                    &Ctx::default(),
                )
            })
            .filter_map(|region| region.props.get("actions"))
            .filter_map(Value::as_sequence)
            .flatten()
            .filter_map(|action| serde_yaml::from_value::<Action>(action.clone()).ok())
            .collect()
    }

    /// Runs an action. Its command's refusal, if it sent one that was not done, is returned for
    /// the caller to show where the user acted.
    pub(crate) fn run_action(
        &mut self,
        action: &Action,
        row: Option<Value>,
        confirmed: bool,
    ) -> Option<Refused> {
        if !confirmed {
            match &action.confirm {
                Some(ActionConfirm::Opens(name)) => {
                    let name = name.clone();
                    self.show_overlay(
                        &name,
                        row.as_ref(),
                        None,
                        Some((action.clone(), row.clone())),
                    );
                    return None;
                }
                Some(ActionConfirm::Inline(inline)) => {
                    let path = self.page_path().child("confirm").child(&action.name);
                    let params = row
                        .as_ref()
                        .and_then(|row| row.get("id"))
                        .map(|id| BTreeMap::from([("id".to_owned(), id.clone())]))
                        .unwrap_or_default();
                    // A freshly opened confirm starts clean, as `show_overlay` does: no typed
                    // text and no refusal from an earlier attempt.
                    self.uis.retain(|id, _| !id.starts_with("o:"));
                    self.overlay = Some(OpenOverlay {
                        name: format!("confirm {}", action.name),
                        path,
                        overlay: (*inline.overlay).clone(),
                        params,
                        then: Some((action.clone(), row)),
                        confirmed: false,
                    });
                    return None;
                }
                None => {}
            }
        }
        let ctx = Ctx {
            row: row.as_ref(),
            ..Ctx::default()
        };
        if let Some(navigate) = &action.navigate {
            let params = navigate
                .params
                .iter()
                .filter_map(|(name, value)| {
                    self.eval(&value.0, &ctx).map(|value| (name.clone(), value))
                })
                .collect();
            self.go(&navigate.to, params, true);
            return None;
        }
        if let Some(overlay) = &action.opens {
            self.show_overlay(overlay, row.as_ref(), None, None);
            return None;
        }
        if let Some(does) = &action.does {
            let input = action
                .bind
                .iter()
                .filter_map(|(name, value)| {
                    self.eval(&value.0, &ctx).map(|value| (name.clone(), value))
                })
                .collect();
            // Nothing else the action does follows a command that was not done.
            if let Some(refused) = self.run_command(does, &input) {
                return Some(refused);
            }
        }
        if let Some(export) = &action.export {
            self.notify(format!(
                "exported {} as {} (fixture)",
                export.reads, export.export_as
            ));
        }
        if let Some(upload) = &action.upload {
            self.notify(format!(
                "upload for {}: pass a file path ({})",
                upload.does,
                upload.accept.join(", ")
            ));
        }
        if let Some(copy) = &action.copy {
            let value = self
                .eval(&copy.0, &ctx)
                .map(|value| display(&value))
                .unwrap_or_default();
            self.notify(format!("copied {value}"));
        }
        for (target, value) in &action.sets {
            let value = self.eval(&value.0, &ctx).unwrap_or(Value::Null);
            self.set_bound(target, value, &ctx);
        }
        None
    }

    /// Runs an action from somewhere that has no place of its own to show a refusal (the
    /// palette, a confirm's alternative): a refusal is notified.
    fn run_action_notified(&mut self, action: &Action, row: Option<Value>, confirmed: bool) {
        if let Some(refused) = self.run_action(action, row, confirmed) {
            let command = action.does.as_deref().unwrap_or(&action.name);
            self.notify(format!("{command} refused: {}", refused.text));
        }
    }

    /// Runs an action offered in the view `ui`, showing a refusal of its command beside it.
    fn run_action_at(&mut self, ui: &str, action: &Action, row: Option<Value>) {
        let refused = self.run_action(action, row, false);
        self.place_refusal(ui, action, refused);
    }

    fn show_overlay(
        &mut self,
        name: &str,
        row: Option<&Value>,
        params: Option<BTreeMap<String, Value>>,
        then: Option<(Action, Option<Value>)>,
    ) {
        let page = self.page_def();
        let (overlay, path) = if let Some(overlay) = page.overlays.get(name) {
            (
                overlay.clone(),
                self.page_path().child("overlays").child(name),
            )
        } else if let Some(overlay) = self
            .doc
            .shells
            .get(&page.shell)
            .and_then(|shell| shell.overlays.get(name))
        {
            (
                overlay.clone(),
                NodePath::root()
                    .child("shells")
                    .child(&page.shell)
                    .child("overlays")
                    .child(name),
            )
        } else {
            self.notify(format!("no overlay {name}"));
            return;
        };
        let ctx = Ctx {
            row,
            ..Ctx::default()
        };
        let params = params.unwrap_or_else(|| {
            if overlay.params.is_empty() {
                row.and_then(|row| row.get("id"))
                    .map(|id| BTreeMap::from([("id".to_owned(), id.clone())]))
                    .unwrap_or_default()
            } else {
                overlay
                    .params
                    .iter()
                    .filter_map(|(key, value)| {
                        self.eval(&value.0, &ctx).map(|value| (key.clone(), value))
                    })
                    .collect()
            }
        });
        self.uis.retain(|id, _| !id.starts_with("o:"));
        self.overlay = Some(OpenOverlay {
            name: name.to_owned(),
            path,
            overlay,
            params,
            then,
            confirmed: false,
        });
    }

    // ── prompts ─────────────────────────────────────────────────────────────────────────────

    fn prompt_key(&mut self, key: KeyEvent) {
        let Some(mut prompt) = self.prompt.take() else {
            return;
        };
        let text = match &mut prompt {
            Prompt::Palette(text) | Prompt::Filter { text, .. } => text,
        };
        match key.code {
            KeyCode::Esc => return,
            KeyCode::Backspace => {
                text.pop();
            }
            KeyCode::Char(letter) => text.push(letter),
            KeyCode::Enter => {
                match prompt {
                    Prompt::Palette(query) => {
                        if let Some(item) = self.palette(&query).into_iter().next() {
                            match item.target {
                                PaletteTarget::Page(page, params) => self.go(&page, params, true),
                                PaletteTarget::Action(action) => {
                                    self.run_action_notified(&action, None, false);
                                }
                            }
                        }
                    }
                    Prompt::Filter { ui, state, text } => {
                        match state {
                            None => {
                                self.ui_mut(&ui).filter = text;
                                self.reset_page(&ui);
                            }
                            Some(binds) => {
                                let section = ui.strip_prefix("s:").map(str::to_owned);
                                let ctx = Ctx {
                                    section: section.as_deref(),
                                    ..Ctx::default()
                                };
                                let value = if text.is_empty() {
                                    Value::Null
                                } else {
                                    Value::String(text)
                                };
                                self.set_bound(&binds, value, &ctx);
                                // A page-state filter moves every collection of the page back
                                // to its first page.
                                self.reset_page(&ui);
                                for id in self.uis.keys().cloned().collect::<Vec<_>>() {
                                    if id.starts_with("s:") {
                                        self.reset_page(&id);
                                    }
                                }
                            }
                        }
                    }
                }
                return;
            }
            _ => {}
        }
        self.prompt = Some(prompt);
    }

    /// Palette entries matching `query`, best first.
    pub(crate) fn palette(&self, query: &str) -> Vec<PaletteItem> {
        let mut items = Vec::new();
        for entry in self.nav().into_iter().flat_map(|group| group.entries) {
            let mut words = vec![entry.label.clone()];
            words.extend(entry.synonyms.clone());
            items.push((
                words,
                PaletteItem {
                    label: entry.label.clone(),
                    target: PaletteTarget::Page(entry.page, entry.params),
                },
            ));
        }
        if let Some(header) = &self.page_def().header {
            for action in &header.actions {
                let label = action.label.clone().unwrap_or_else(|| action.name.clone());
                items.push((
                    vec![label.clone()],
                    PaletteItem {
                        label: format!("do: {label}"),
                        target: PaletteTarget::Action(Box::new(action.clone())),
                    },
                ));
            }
        }
        for action in self.account_actions() {
            let label = action.label.clone().unwrap_or_else(|| action.name.clone());
            items.push((
                vec![label.clone()],
                PaletteItem {
                    label: format!("account: {label}"),
                    target: PaletteTarget::Action(Box::new(action)),
                },
            ));
        }
        let mut scored: Vec<(usize, usize, PaletteItem)> = items
            .into_iter()
            .enumerate()
            .filter_map(|(order, (words, item))| {
                words
                    .iter()
                    .filter_map(|word| fuzzy(query, word))
                    .min()
                    .map(|score| (score, order, item))
            })
            .collect();
        scored.sort_by_key(|(score, order, _)| (*score, *order));
        scored.into_iter().map(|(_, _, item)| item).collect()
    }
}

/// Why a guard stopped a navigation.
enum GuardStop {
    Redirect(String),
    Refused(String),
}

/// An entry of the palette.
#[derive(Debug, Clone)]
pub(crate) struct PaletteItem {
    pub label: String,
    pub target: PaletteTarget,
}

#[derive(Debug, Clone)]
pub(crate) enum PaletteTarget {
    Page(String, BTreeMap<String, Value>),
    Action(Box<Action>),
}

/// A subsequence match of `query` in `word`: lower is better, `None` is no match.
fn fuzzy(query: &str, word: &str) -> Option<usize> {
    let word: Vec<char> = word.to_lowercase().chars().collect();
    let mut at = 0;
    let mut first = None;
    let mut gaps = 0;
    for wanted in query.to_lowercase().chars() {
        let found = word[at..].iter().position(|have| *have == wanted)?;
        if first.is_none() {
            first = Some(at + found);
        } else {
            gaps += found;
        }
        at += found + 1;
    }
    Some(first.unwrap_or(0) * 4 + gaps * 2 + word.len().saturating_sub(at))
}

struct Scope<'a> {
    app: &'a App,
    ctx: &'a Ctx<'a>,
}

impl Resolve for Scope<'_> {
    #[allow(clippy::too_many_lines)] // one arm per expression root
    fn resolve(&self, root: &str, segments: &[&str]) -> Resolved {
        let app = self.app;
        let adapter = app.adapter.as_ref();
        match root {
            "row" => Resolved::Root(self.ctx.row.cloned().unwrap_or(Value::Null)),
            "params" => {
                let mut params = app.params.clone();
                if self.ctx.overlay {
                    params.extend(app.overlay_params());
                }
                Resolved::Root(to_mapping(params))
            }
            "state" => match segments.first() {
                Some(name) => {
                    let value = app.store.get(&app.state_path(name, self.ctx), adapter);
                    Resolved::Whole(expr::walk(&value, &segments[1..]))
                }
                None => Resolved::Whole(Value::Null),
            },
            "shell" => match segments.first() {
                Some(name) => {
                    let value = app.store.get(&app.shell_state_path(name), adapter);
                    Resolved::Whole(expr::walk(&value, &segments[1..]))
                }
                None => Resolved::Whole(Value::Null),
            },
            "draft" => Resolved::Root(
                self.ctx
                    .draft
                    .map_or(Value::Null, |draft| app.store.get(draft, adapter)),
            ),
            "channel" => {
                let (Some(channel), Some(field)) = (segments.first(), segments.get(1)) else {
                    return Resolved::Whole(Value::Null);
                };
                let declared = app
                    .doc
                    .channels
                    .get(*channel)
                    .and_then(|channel| channel.fields.get(*field));
                let Some(state) = app
                    .channels
                    .get(*channel)
                    .filter(|state| app.shows_session(channel, state.session.as_ref()))
                else {
                    return Resolved::Whole(Value::Null);
                };
                // A field declared as a derived signal is computed; otherwise it names a key of
                // the latest payload (itself, when not declared).
                let derived = declared.and_then(|text| derived_field(text, state, app.now));
                let value = derived.unwrap_or_else(|| {
                    let key = declared
                        .filter(|key| key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
                        .map_or(*field, String::as_str);
                    state
                        .latest
                        .as_ref()
                        .map_or(Value::Null, |latest| latest[key].clone())
                });
                Resolved::Whole(expr::walk(&value, &segments[2..]))
            }
            "actor" => {
                // In fixture mode a `from_session` document runs signed in, as the first row of
                // the first view its shell preloads.
                let mut actor = actor_row(&app.doc, &app.page_def().shell, adapter)
                    .filter(Value::is_mapping)
                    .unwrap_or(Value::Mapping(Mapping::new()));
                actor["signed_in"] = Value::Bool(
                    app.doc.actor == Some(ess_ui::ActorSource::FromSession) && !app.signed_out,
                );
                Resolved::Root(actor)
            }
            "section" => {
                let Some(name) = segments.first() else {
                    return Resolved::Whole(Value::Null);
                };
                let Some(section) = app
                    .page_def()
                    .sections
                    .iter()
                    .find(|section| section.name == *name)
                else {
                    return Resolved::Whole(Value::Null);
                };
                let ui = format!("s:{name}");
                let ctx = Ctx {
                    section: Some(name),
                    ..Ctx::default()
                };
                let selection = match &section.body {
                    Body::Composite(Composite::Collection(collection)) => {
                        // Evaluating the section's own params here would recurse into this
                        // lookup, so a dependent read sees the selection of the page shown.
                        let reads_self = collection.reads.as_ref().is_some_and(|reads| {
                            reads
                                .params
                                .values()
                                .any(|expr| expr.0.starts_with("section."))
                        });
                        if reads_self {
                            None
                        } else {
                            app.selected_row(&ui, collection, &ctx)
                        }
                    }
                    _ => None,
                };
                let mut value = Mapping::new();
                value.insert("selection".into(), selection.unwrap_or(Value::Null));
                Resolved::Whole(expr::walk(&Value::Mapping(value), &segments[1..]))
            }
            "url" => {
                let mut query = Mapping::new();
                for (name, value) in app.store.query(&format!("pages/{}", app.page)) {
                    query.insert(name.into(), value);
                }
                let mut url = Mapping::new();
                url.insert("query".into(), Value::Mapping(query));
                Resolved::Root(Value::Mapping(url))
            }
            _ => Resolved::Root(Value::Null),
        }
    }
}

fn to_mapping(values: BTreeMap<String, Value>) -> Value {
    Value::Mapping(
        values
            .into_iter()
            .map(|(key, value)| (Value::String(key), value))
            .collect(),
    )
}

fn location_value(value: &Value) -> String {
    match value {
        Value::Sequence(items) => items.iter().map(display).collect::<Vec<_>>().join(","),
        other => display(other),
    }
}

fn compare(left: &Value, right: &Value) -> std::cmp::Ordering {
    // A number, or an amount record (`{amount, currency}`), sorts by its number.
    let number = |value: &Value| {
        value
            .as_f64()
            .or_else(|| value.get("amount").and_then(Value::as_f64))
    };
    if let (Some(left), Some(right)) = (number(left), number(right)) {
        left.total_cmp(&right)
    } else {
        display(left).cmp(&display(right))
    }
}

/// The fixture actor: in fixture mode a `from_session` document runs as the first row of the
/// first view its shell preloads.
pub(crate) fn actor_row(
    document: &Document,
    shell: &str,
    adapter: &dyn DataAdapter,
) -> Option<Value> {
    let view = document
        .shells
        .get(shell)
        .and_then(|shell| shell.preload.as_ref())
        .and_then(|preload| preload.views.first())?;
    let request = ReadRequest {
        view: view.view.clone(),
        fixture: None,
        params: BTreeMap::new(),
    };
    adapter
        .read(&request)
        .ok()
        .and_then(|result| result.rows.first().cloned())
}

/// The fixture actor's user id (`user_id`, else `id`) and account id (`account_id`, else
/// `account.id`, else `organization.id`), read as the home page's shell sees the actor.
pub(crate) fn actor_ids(
    document: &Document,
    adapter: &dyn DataAdapter,
) -> (Option<String>, Option<String>) {
    let actor = document
        .pages
        .get(&document.navigation.home)
        .and_then(|page| actor_row(document, &page.shell, adapter))
        .unwrap_or(Value::Null);
    let id_of = |keys: &[&[&str]]| {
        keys.iter()
            .map(|path| expr::walk(&actor, path))
            .find(|value| !value.is_null())
            .map(|value| display(&value))
    };
    (
        id_of(&[&["user_id"], &["id"]]),
        id_of(&[&["account_id"], &["account", "id"], &["organization", "id"]]),
    )
}

/// Keys for a collection's bulk actions: an upper-case letter of the action's name, never one
/// the collection keys already use (`S` sort, `J`/`K` move, `R` reload).
pub(crate) fn bulk_keys(actions: &[Action]) -> Vec<(char, Action)> {
    let mut taken = String::new();
    let mut keyed = Vec::new();
    for action in actions {
        let key = action
            .name
            .chars()
            .filter(char::is_ascii_alphabetic)
            .map(|letter| letter.to_ascii_uppercase())
            .find(|key| !"SJKR".contains(*key) && !taken.contains(*key));
        if let Some(key) = key {
            taken.push(key);
            keyed.push((key, action.clone()));
        }
    }
    keyed
}

/// Writes every field of `payload` over `row`.
fn merge(row: &mut Value, payload: &Value) {
    if let (Value::Mapping(row), Value::Mapping(payload)) = (row, payload) {
        for (field, value) in payload {
            row.insert(field.clone(), value.clone());
        }
    }
}

/// Applies a live event's effect to rows.
fn apply(live: &Live, rows: &mut Vec<Value>, payload: &Value) {
    let key = live.match_field.as_deref().unwrap_or("id");
    let identity = payload.get(key).map(display);
    let found = identity.as_ref().and_then(|identity| {
        rows.iter()
            .position(|row| row.get(key).map(display).as_ref() == Some(identity))
    });
    match live.effect {
        Effect::PatchRow => {
            if let Some(at) = found {
                merge(&mut rows[at], payload);
            }
        }
        Effect::InsertOrPatch => match found {
            Some(at) => merge(&mut rows[at], payload),
            None => rows.insert(0, payload.clone()),
        },
        Effect::InsertTop => {
            rows.insert(0, payload.clone());
            if let Some(max) = live.max_rows {
                rows.truncate(max as usize);
            }
        }
        Effect::RemoveRow => {
            if let Some(at) = found {
                rows.remove(at);
            }
        }
        Effect::Replace => *rows = vec![payload.clone()],
        Effect::Refetch | Effect::Unmapped(_) => {}
    }
}

/// The read a composite's body is answered by.
pub(crate) fn body_reads(body: &Body) -> Option<&Reads> {
    let Body::Composite(composite) = body else {
        return None;
    };
    match composite {
        Composite::Collection(collection) => collection.reads.as_ref(),
        Composite::Record(record) => record.reads.as_ref(),
        Composite::Form(form) => form.loads.as_ref(),
        Composite::Choice(choice) => choice.reads.as_ref(),
        Composite::Metric(metric) => metric.reads.as_ref(),
        Composite::Chart(chart) => Some(&chart.reads),
        Composite::Board(board) => Some(&board.reads),
        Composite::GraphEditor(editor) => Some(&editor.reads),
        Composite::References(references) => Some(&references.reads),
        _ => None,
    }
}

/// The fields of a form on its current tab: plain fields, then group fields, then the tab's.
pub(crate) fn form_fields(form: &Form, tab: usize) -> Vec<&Field> {
    let mut fields: Vec<&Field> = form.fields.iter().collect();
    for group in &form.groups {
        fields.extend(group.fields.iter());
    }
    if let Some(TabFields::Fields(tab_fields)) =
        form.tabs.get(tab).and_then(|tab| tab.fields.as_ref())
    {
        fields.extend(tab_fields.iter());
    }
    fields
}

/// The columns of a collection, or one per key of its first row when it names none.
pub(crate) fn columns_of(collection: &ess_ui::Collection, rows: &[Value]) -> Vec<Field> {
    match &collection.columns {
        Some(Columns::Fixed(fields)) => fields.clone(),
        Some(Columns::Selectable(columns)) => columns.all.clone(),
        _ => rows
            .first()
            .and_then(Value::as_mapping)
            .map(|row| {
                row.keys()
                    .filter_map(Value::as_str)
                    .map(|key| Field {
                        field: key.to_owned(),
                        name: key.to_owned(),
                        label: None,
                        field_as: None,
                        choice: None,
                        sortable: false,
                        visible: None,
                        binds: None,
                        note: None,
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn plain_field(name: &str, field_as: Option<&str>) -> Field {
    Field {
        field: name.to_owned(),
        name: name.to_owned(),
        label: None,
        field_as: field_as.map(str::to_owned),
        choice: None,
        sortable: false,
        visible: None,
        binds: None,
        note: None,
    }
}

/// A graph editor degraded to a collection of its nodes: label, kind, and its actions.
pub(crate) fn graph_collection(editor: &ess_ui::GraphEditor) -> ess_ui::Collection {
    let mut columns = vec![plain_field("id", None), plain_field("label", None)];
    if let Some(nodes) = &editor.nodes {
        columns.push(plain_field(&nodes.kind_by, Some("badge")));
    }
    if let Some(edges) = &editor.edges {
        columns.push(plain_field(&edges.from, None));
        columns.push(plain_field(&edges.to, None));
        if let Some(kind) = &edges.kind_by {
            columns.push(plain_field(kind, Some("badge")));
        }
    }
    let mut row_actions = editor.node_actions.clone();
    row_actions.extend(editor.edge_actions.clone());
    ess_ui::Collection {
        reads: Some(editor.reads.clone()),
        columns: Some(Columns::Fixed(columns)),
        sort: None,
        style: None,
        selection: None,
        row_actions,
        bulk_actions: Vec::new(),
        actions: Vec::new(),
        expand: None,
        item: Vec::new(),
        reorder: None,
        group_by: None,
    }
}

/// A references composite as the collection it is drawn as.
pub(crate) fn references_collection(references: &ess_ui::References) -> ess_ui::Collection {
    ess_ui::Collection {
        reads: Some(references.reads.clone()),
        columns: (!references.columns.is_empty())
            .then(|| Columns::Fixed(references.columns.clone())),
        sort: None,
        style: None,
        selection: None,
        row_actions: Vec::new(),
        bulk_actions: Vec::new(),
        actions: Vec::new(),
        expand: None,
        item: Vec::new(),
        reorder: None,
        group_by: None,
    }
}

/// The rows a board places, in layout order (top to bottom, then left to right).
pub(crate) fn board_rows(app: &App, board: &ess_ui::Board, section: Option<&str>) -> Vec<Value> {
    let ctx = Ctx {
        section,
        ..Ctx::default()
    };
    let request = app.request(&board.reads, &ctx);
    let mut rows = app
        .rows_of(&request)
        .map(|result| result.rows.clone())
        .unwrap_or_default();
    rows.sort_by(|left, right| {
        compare(&left["y"], &right["y"]).then_with(|| compare(&left["x"], &right["x"]))
    });
    rows
}

/// One focusable item of a filter bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BarItem {
    Search,
    Window,
    Choice(usize),
    Input(usize),
}

pub(crate) fn bar_items(bar: &FilterBar) -> Vec<BarItem> {
    let mut items = Vec::new();
    if bar.search.is_some() {
        items.push(BarItem::Search);
    }
    if bar.window.is_some() {
        items.push(BarItem::Window);
    }
    items.extend((0..bar.choices.len()).map(BarItem::Choice));
    items.extend((0..bar.inputs.len()).map(BarItem::Input));
    items
}

/// The state a filter-bar choice writes: its own `binds`, else the bar's `state.<name>`.
pub(crate) fn choice_binds(
    bar: &FilterBar,
    node: &ess_ui::Node,
    choice: &ess_ui::Choice,
) -> Option<String> {
    if let Some(binds) = &choice.binds {
        return Some(binds.0.clone());
    }
    let name = node.common.name.as_deref()?;
    let wanted = format!("state.{name}");
    bar.binds
        .iter()
        .any(|binds| binds.0 == wanted)
        .then_some(wanted)
}
