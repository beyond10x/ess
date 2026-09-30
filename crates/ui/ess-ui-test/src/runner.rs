//! Running one test headless against the terminal renderer.
//!
//! Every step goes through what a reader of the terminal has: key presses, and the drawn screen.
//! A path is resolved against the document first ([`crate::target`]), so a path that names no node
//! fails the step naming the path. Reads and commands go through the document's fixture adapter,
//! whose command log `expect_command` reads; per-test fixtures are written next to the run and
//! replace the document's view files and channel scripts for that test only.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use ess_ui::{Action, Body, Composite, Document, FixtureIndex, Form, NodeRef, TabFields};
use ess_ui_tui::live::Script;
use ess_ui_tui::{App, DataAdapter, FixtureAdapter, Lifecycle, Options, ReadRequest, ReadResult};
use serde_yaml::{Mapping, Value};

use crate::screen::{Screen, SectionBox};
use crate::spec::{scalar, Check, Fixtures, Play, SectionState, Step, Test};
use crate::target::{self, Row, Target};

/// How many cycles of one looping fixture script a single `advance` (or `play`) may play. A
/// longer move is a failed step: every cycle's events are delivered one by one, so an unbounded
/// move over a looping script would run until memory is exhausted.
pub const MAX_CYCLES_PER_ADVANCE: u128 = 10_000;

/// The fixture adapter, shared so the runner can read its command log.
struct Shared(Rc<RefCell<FixtureAdapter>>);

impl DataAdapter for Shared {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        self.0.borrow().read(request)
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Result<String, String> {
        self.0.borrow_mut().run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.0.borrow().load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.0.borrow_mut().store_state(path, value);
    }
}

/// One test in progress.
pub(crate) struct Runner<'d> {
    doc: &'d Document,
    app: App,
    adapter: Rc<RefCell<FixtureAdapter>>,
    scripts: Vec<Script>,
    now: Duration,
}

impl<'d> Runner<'d> {
    /// Opens `doc` (read from `base`) with `test`'s fixtures, keeping files under `dir`.
    pub fn start(doc: &'d Document, base: &Path, test: &Test, dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        let fixtured = with_fixtures(doc, base, &test.fixtures, dir)?;
        let (adapter, mut scripts) =
            FixtureAdapter::load(&fixtured, base, None).map_err(|error| error.to_string())?;
        for (channel, value) in &test.fixtures.scripts {
            let script = Script::parse(channel, value)?;
            scripts.retain(|known| known.channel != script.channel);
            scripts.push(script);
        }
        let adapter = Rc::new(RefCell::new(adapter));
        let mut options = Options::new(dir.join("state"));
        options.read_latency = test.latency;
        let mut app = App::with_adapter(
            fixtured,
            Box::new(Shared(Rc::clone(&adapter))),
            scripts.clone(),
            options,
        )
        .map_err(|error| error.to_string())?;
        // What the terminal loop does before its first frame: play what is due at time zero.
        app.advance(Duration::ZERO);
        Ok(Self {
            doc,
            app,
            adapter,
            scripts,
            now: Duration::ZERO,
        })
    }

    /// Runs one step.
    pub fn step(&mut self, step: &Step) -> Result<(), String> {
        match step {
            Step::Open { page, params } => self.open(page, params),
            Step::Select { at } => {
                let target = self.resolve(at)?;
                self.select(&target)
            }
            Step::Type { at, text } => {
                let target = self.resolve(at)?;
                self.type_text(&target, text)
            }
            Step::Choose { at, option } => {
                let target = self.resolve(at)?;
                self.choose(&target, option)
            }
            Step::Act { at } => {
                let target = self.resolve(at)?;
                self.act(&target)
            }
            Step::Page { at, to } => {
                let target = self.resolve(at)?;
                self.page(&target, *to)
            }
            Step::Expect { at, checks } => {
                let target = self.resolve(at)?;
                checks
                    .iter()
                    .try_for_each(|check| self.expect(&target, check))
            }
            Step::Play(play) => self.play(play),
            Step::Advance(by) => self.advance(*by),
            Step::ExpectCommand { command, input } => self.expect_command(command, input),
        }
    }

    /// Moves the virtual clock by `by`. Fails, without moving it, when the clock would pass what
    /// a `Duration` holds or a looping script would play more than [`MAX_CYCLES_PER_ADVANCE`]
    /// cycles in this one move.
    fn advance(&mut self, by: Duration) -> Result<(), String> {
        let now = crate::clock::advance(&self.scripts, self.now, by)?;
        self.app.advance(by);
        self.now = now;
        Ok(())
    }

    /// Resolves `at`, refusing a path the generated React project renders no element at
    /// ([`crate::parity::unrendered`]).
    fn resolve(&self, at: &str) -> Result<Target, String> {
        let target = target::resolve(self.doc, at)?;
        if let Some(reason) = crate::parity::unrendered(self.doc, &target) {
            return Err(reason);
        }
        if let Some(page) = target.page() {
            if page != self.app.page() {
                return Err(format!(
                    "{at} is on page {page}; the page shown is {}",
                    self.app.page()
                ));
            }
        }
        Ok(target)
    }

    fn screen(&self) -> Screen {
        Screen::capture(&self.app)
    }

    // ── open, select, type, choose, act, page ───────────────────────────────────────────────

    fn open(&mut self, page: &str, params: &BTreeMap<String, String>) -> Result<(), String> {
        if !self.doc.pages.contains_key(page) {
            return Err(format!("no node at pages/{page}"));
        }
        let params: Vec<(&str, &str)> = params
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        self.app.open_page(page, &params);
        if self.app.page() != page {
            return Err(format!("opening {page} shows {} instead", self.app.page()));
        }
        Ok(())
    }

    fn select(&mut self, target: &Target) -> Result<(), String> {
        if target.is_page() {
            if self.screen().overlay_open() {
                self.app.keys("<esc>");
            }
            return Ok(());
        }
        if let Some((name, _)) = target.overlay() {
            return self.require_overlay(&name);
        }
        self.require_no_overlay(target)?;
        if let Some(row) = &target.row {
            return self.select_row(target, row);
        }
        let section = target.section().ok_or_else(|| {
            format!(
                "{}: the terminal selects sections, rows and overlays",
                target.written
            )
        })?;
        self.app.focus_section(&section);
        Ok(())
    }

    fn type_text(&mut self, target: &Target, text: &str) -> Result<(), String> {
        let typed = text.replace('<', "<lt>");
        if target.row.is_none() && target.is_section() {
            let section = target.section().unwrap_or_default();
            let body = &self.section_def(&section)?.body;
            match body {
                Body::Composite(Composite::FilterBar(bar)) if bar.search.is_none() => {
                    return Err(format!("{}: this filter bar has no search", target.written));
                }
                Body::Composite(Composite::FilterBar(_) | Composite::Collection(_)) => {}
                _ => {
                    return Err(format!(
                        "{}: type into a filter bar, a collection or a form field",
                        target.written
                    ))
                }
            }
            self.require_no_overlay(target)?;
            self.app.focus_section(&section);
            self.app.keys(&format!("/{typed}<enter>"));
            return Ok(());
        }
        let (form, in_overlay) = self.form_of(target)?;
        let name = target.node.rsplit('/').next().unwrap_or_default();
        let fields = form_fields(&form);
        let index = fields
            .iter()
            .position(|field| field.name == name)
            .ok_or_else(|| {
                format!(
                    "{}: type into a filter bar, a collection or a form field",
                    target.written
                )
            })?;
        if matches!(fields[index].field_as.as_deref(), Some("toggle" | "choice")) {
            return Err(format!(
                "{}: a {} field is chosen, not typed",
                target.written,
                fields[index].field_as.clone().unwrap_or_default()
            ));
        }
        if !in_overlay {
            self.require_no_overlay(target)?;
            self.app
                .focus_section(&target.section().unwrap_or_default());
        }
        self.app.keys(&"k".repeat(fields.len()));
        self.app.keys(&"j".repeat(index));
        self.app.keys(&format!("<enter>{typed}<enter>"));
        Ok(())
    }

    fn choose(&mut self, target: &Target, option: &str) -> Result<(), String> {
        let unsupported = || {
            format!(
                "{}: the terminal chooses in a filter bar's choices",
                target.written
            )
        };
        let segments: Vec<&str> = target.node.split('/').collect();
        let ["pages", _, "sections", section, "choices", choice] = segments.as_slice() else {
            return Err(unsupported());
        };
        let Body::Composite(Composite::FilterBar(bar)) = &self.section_def(section)?.body else {
            return Err(unsupported());
        };
        let bar = bar.clone();
        let position = bar
            .choices
            .iter()
            .position(|node| node.common.name.as_deref() == Some(*choice))
            .ok_or_else(unsupported)?;
        let Body::Composite(Composite::Choice(definition)) = &bar.choices[position].body else {
            return Err(unsupported());
        };
        let options = self.choice_options(definition);
        let index = options
            .iter()
            .position(|(value, label)| scalar(value) == option || label == option)
            .ok_or_else(|| {
                format!(
                    "{} has no option {option:?}; it offers [{}]",
                    target.written,
                    options
                        .iter()
                        .map(|(_, label)| label.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        let item = usize::from(bar.search.is_some()) + usize::from(bar.window.is_some()) + position;
        let items = item + bar.choices.len() + bar.inputs.len();
        self.require_no_overlay(target)?;
        self.app.focus_section(section);
        self.app.keys(&"h".repeat(items));
        self.app.keys(&"l".repeat(item));
        self.app.keys(&"j".repeat(index));
        self.app.keys("<space>");
        Ok(())
    }

    fn act(&mut self, target: &Target) -> Result<(), String> {
        if let Some(row) = &target.row {
            self.select_row(target, row)?;
            if target.node == row.container {
                self.app.keys("<enter>");
                return Ok(());
            }
            let action = self.action(target)?;
            let lines = self.section_box(&row_section(target, row)?)?.lines;
            let key = hint_key(lines.iter().map(|line| line.text.as_str()), &action).ok_or_else(
                || {
                    format!(
                        "{}: the row does not offer {}",
                        target.written,
                        label(&action)
                    )
                },
            )?;
            self.app.keys(&key_spec(key));
            return Ok(());
        }
        if let Some((name, path)) = target.overlay() {
            self.require_overlay(&name)?;
            let overlay_body = self.overlay_body(&path)?;
            if target.node == path {
                return match overlay_body {
                    Body::Composite(Composite::Confirm(_)) => {
                        self.app.keys("y");
                        Ok(())
                    }
                    Body::Composite(Composite::Form(_)) => {
                        self.app.keys("<c-s>");
                        Ok(())
                    }
                    _ => Err(format!(
                        "{}: this overlay has no primary action",
                        target.written
                    )),
                };
            }
            let action = self.action(target)?;
            if let Body::Composite(Composite::Confirm(confirm)) = &overlay_body {
                if let Some(index) = confirm
                    .alternatives
                    .iter()
                    .position(|alternative| alternative.name == action.name)
                {
                    self.app.keys(&(index + 1).to_string());
                    return Ok(());
                }
            }
            let text = self.screen().text();
            let key = hint_key(text.lines(), &action).ok_or_else(|| {
                format!(
                    "{}: the overlay does not offer {}",
                    target.written,
                    label(&action)
                )
            })?;
            self.app.keys(&key_spec(key));
            return Ok(());
        }
        let action = self.action(target)?;
        self.require_no_overlay(target)?;
        let segments: Vec<&str> = target.node.split('/').collect();
        if let ["pages", _, "header", "actions", _] = segments.as_slice() {
            self.app
                .keys(&format!(":{}<enter>", label(&action).replace('<', "<lt>")));
            return Ok(());
        }
        let section = target.section().ok_or_else(|| {
            format!(
                "{}: the terminal acts on actions of sections, rows, overlays and the header",
                target.written
            )
        })?;
        self.app.focus_section(&section);
        let lines = self.section_box(&section)?.lines;
        let key =
            hint_key(lines.iter().map(|line| line.text.as_str()), &action).ok_or_else(|| {
                format!(
                    "{}: the section does not offer {}",
                    target.written,
                    label(&action)
                )
            })?;
        self.app.keys(&key_spec(key));
        Ok(())
    }

    fn page(&mut self, target: &Target, to: usize) -> Result<(), String> {
        let section = self.collection_section(target)?;
        self.require_no_overlay(target)?;
        self.app.focus_section(&section);
        let (_, pages) = self.pager(&section)?;
        if to > pages {
            return Err(format!("{} has {pages} pages, not {to}", target.written));
        }
        self.app.keys(&"p".repeat(pages));
        self.app.keys(&"n".repeat(to - 1));
        let (shown, _) = self.pager(&section)?;
        if shown != to {
            return Err(format!("{} shows page {shown}, not {to}", target.written));
        }
        Ok(())
    }

    // ── rows ────────────────────────────────────────────────────────────────────────────────

    fn select_row(&mut self, target: &Target, row: &Row) -> Result<(), String> {
        let section = row_section(target, row)?;
        self.app.focus_section(&section);
        let rows = self.app.rows(&section);
        let field = self.key_field(&section);
        rows.iter()
            .position(|data| key_of(data, &field) == row.key)
            .ok_or_else(|| {
                format!(
                    "no row {} in {}; it holds [{}]",
                    row.key,
                    row.container,
                    rows.iter()
                        .map(|data| key_of(data, &field))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        let (_, pages) = self.pager(&section)?;
        self.app.keys(&"p".repeat(pages));
        for page in 0..pages {
            self.app.keys(&"k".repeat(rows.len()));
            // The cursor moves one row per `j`; the rows it passes are counted by their keys, so
            // rows that show the same values are still told apart.
            for _ in 0..=rows.len() {
                let shown = self.screen().rows(&row.container);
                match shown.iter().find(|(_, cursor)| *cursor) {
                    Some((key, _)) if *key == row.key => return Ok(()),
                    Some(_) => {}
                    None => break,
                }
                let last = shown.last().is_some_and(|(_, cursor)| *cursor);
                if last {
                    break;
                }
                self.app.keys("j");
            }
            if page + 1 < pages {
                self.app.keys("n");
            }
        }
        Err(format!(
            "row {} of {} is not shown (filtered out?)",
            row.key, row.container
        ))
    }

    /// The keys of the rows the section's collection draws, in screen order.
    fn shown_rows(&self, section: &str) -> Result<Vec<String>, String> {
        self.section_box(section)?;
        let collection = format!("pages/{}/sections/{section}", self.app.page());
        Ok(self
            .screen()
            .rows(&collection)
            .into_iter()
            .map(|(key, _)| key)
            .collect())
    }

    /// The page shown and the page count, from the collection's footer.
    fn pager(&self, section: &str) -> Result<(usize, usize), String> {
        let drawn = self.section_box(section)?;
        drawn
            .lines
            .iter()
            .find_map(|line| {
                let rest = line.text.trim_start().strip_prefix("page ")?;
                let (shown, rest) = rest.split_once('/')?;
                let pages: String = rest.chars().take_while(char::is_ascii_digit).collect();
                Some((shown.parse().ok()?, pages.parse().ok()?))
            })
            .ok_or_else(|| format!("section {section} shows no page footer: {}", drawn.text()))
    }

    fn key_field(&self, section: &str) -> String {
        self.section_def(section)
            .ok()
            .and_then(|section| section.live.as_ref())
            .and_then(|live| live.match_field.clone())
            .unwrap_or_else(|| "id".to_owned())
    }

    // ── expect ──────────────────────────────────────────────────────────────────────────────

    fn expect(&self, target: &Target, check: &Check) -> Result<(), String> {
        match check {
            Check::Text(text) | Check::NotText(text) => {
                if let Some(reason) = crate::parity::text(self.doc, target) {
                    return Err(reason);
                }
                let region = self.region(target)?;
                let wanted = matches!(check, Check::Text(_));
                if region.contains(text.as_str()) == wanted {
                    return Ok(());
                }
                Err(format!(
                    "expected {}{text:?} at {}; it shows:\n{region}",
                    if wanted { "" } else { "no " },
                    target.written
                ))
            }
            Check::RowCount(count) => {
                let section = self.collection_section(target)?;
                let keys = self.shown_rows(&section)?;
                if keys.len() == *count {
                    return Ok(());
                }
                Err(format!(
                    "expected {count} rows at {}; it shows {} [{}]",
                    target.written,
                    keys.len(),
                    keys.join(", ")
                ))
            }
            Check::RowKeys(wanted) => {
                let section = self.collection_section(target)?;
                let keys = self.shown_rows(&section)?;
                if keys == *wanted {
                    return Ok(());
                }
                Err(format!(
                    "expected rows [{}] at {}; it shows [{}]",
                    wanted.join(", "),
                    target.written,
                    keys.join(", ")
                ))
            }
            Check::State(wanted) => {
                let section = match (target.is_section(), target.section()) {
                    (true, Some(section)) if target.row.is_none() => section,
                    _ => return Err(format!("{}: a state is a section's", target.written)),
                };
                let drawn = self.section_box(&section)?;
                let stale = drawn.text().contains("[stale]");
                let actual = match self.app.section_state(&section) {
                    Lifecycle::NotLoaded => SectionState::NotLoaded,
                    Lifecycle::Loading => SectionState::Loading,
                    Lifecycle::Failed => SectionState::Failed,
                    _ if stale => SectionState::Stale,
                    Lifecycle::Empty => SectionState::Empty,
                    Lifecycle::Ready => SectionState::Ready,
                };
                if actual == *wanted {
                    return Ok(());
                }
                Err(format!(
                    "expected {} to be {}; it is {}:\n{}",
                    target.written,
                    wanted.name(),
                    actual.name(),
                    drawn.text()
                ))
            }
        }
    }

    /// The text a node shows on its own cells: the screen for a page, a section's box, the open
    /// overlay's pane, and otherwise the cells the terminal drew the node on
    /// ([`ess_ui_tui::Region`]): a row's line, a cell, a column header, a section's child, a
    /// header action. A node the terminal does not draw on cells of its own is refused naming its
    /// path — never widened to the row, the box or the screen around it.
    fn region(&self, target: &Target) -> Result<String, String> {
        if target.is_page() {
            return Ok(self.screen().text());
        }
        if let Some((name, path)) = target.overlay() {
            self.require_overlay(&name)?;
            if target.node == path && target.row.is_none() {
                let screen = self.screen();
                return Ok(screen
                    .region(&path)
                    .map_or_else(|| screen.text(), |region| screen.text_in(region)));
            }
        } else if target.is_section() && target.row.is_none() {
            let section = target.section().unwrap_or_default();
            return Ok(self.section_box(&section)?.text());
        }
        let screen = self.screen();
        if let Some(region) = screen.region(&target.written) {
            // A cut cell, a column's label and an action's label read whole, as in the browser.
            return Ok(region
                .text
                .clone()
                .unwrap_or_else(|| screen.text_in(region)));
        }
        if let Some(row) = &target.row {
            if !screen
                .rows(&row.container)
                .iter()
                .any(|(key, _)| *key == row.key)
            {
                return Err(format!("row {} of {} is not shown", row.key, row.container));
            }
        }
        Err(format!(
            "{}: the terminal does not draw this node on cells of its own, so its text cannot be \
             told from its neighbours'; expect text at the row, section, overlay or page instead",
            target.written
        ))
    }

    // ── live ────────────────────────────────────────────────────────────────────────────────

    fn play(&mut self, play: &Play) -> Result<(), String> {
        let at = crate::clock::next(&self.scripts, play, self.now).ok_or_else(|| {
            format!(
                "the fixture scripts play no {play} after {}ms",
                self.now.as_millis()
            )
        })?;
        self.advance(at.saturating_sub(self.now))
    }

    fn expect_command(&self, command: &str, input: &BTreeMap<String, Value>) -> Result<(), String> {
        let sent = self.adapter.borrow().commands.clone();
        let named: Vec<&BTreeMap<String, Value>> = sent
            .iter()
            .filter(|(name, _)| name == command)
            .map(|(_, input)| input)
            .collect();
        if named.is_empty() {
            let others: Vec<&str> = sent.iter().map(|(name, _)| name.as_str()).collect();
            return Err(format!(
                "command {command} was never sent; sent: [{}]",
                others.join(", ")
            ));
        }
        let holds = |given: &BTreeMap<String, Value>| {
            input
                .iter()
                .all(|(field, value)| given.get(field).map(scalar) == Some(scalar(value)))
        };
        if named.iter().any(|given| holds(given)) {
            return Ok(());
        }
        Err(format!(
            "command {command} was sent with {}, not with {}",
            named
                .iter()
                .map(|given| show_input(given))
                .collect::<Vec<_>>()
                .join(" and "),
            show_input(input)
        ))
    }

    // ── document lookups ────────────────────────────────────────────────────────────────────

    fn section_def(&self, name: &str) -> Result<&'d ess_ui::Section, String> {
        let page = self
            .doc
            .pages
            .get(self.app.page())
            .ok_or("no page is shown")?;
        page.sections
            .iter()
            .find(|section| section.name == name)
            .ok_or_else(|| format!("no node at pages/{}/sections/{name}", self.app.page()))
    }

    fn section_box(&self, name: &str) -> Result<SectionBox, String> {
        self.screen()
            .section(name)
            .ok_or_else(|| format!("section {name} is not drawn"))
    }

    fn collection_section(&self, target: &Target) -> Result<String, String> {
        let section = target
            .section()
            .filter(|_| target.is_section() && target.row.is_none());
        let section = section
            .ok_or_else(|| format!("{}: rows are a section's collection's", target.written))?;
        match &self.section_def(&section)?.body {
            Body::Composite(
                Composite::Collection(_) | Composite::References(_) | Composite::GraphEditor(_),
            ) => Ok(section),
            _ => Err(format!("{} is not a collection", target.written)),
        }
    }

    fn action(&self, target: &Target) -> Result<Action, String> {
        self.doc
            .nodes()
            .into_iter()
            .find_map(|located| match located.node {
                NodeRef::Action(action) if located.path.to_string() == target.node => {
                    Some(action.clone())
                }
                _ => None,
            })
            .ok_or_else(|| format!("{} is not an action", target.written))
    }

    fn overlay_body(&self, path: &str) -> Result<Body, String> {
        self.doc
            .nodes()
            .into_iter()
            .find_map(|located| match located.node {
                NodeRef::Overlay(overlay) if located.path.to_string() == path => {
                    Some(overlay.body.clone())
                }
                _ => None,
            })
            .ok_or_else(|| format!("no node at {path}"))
    }

    fn form_of(&self, target: &Target) -> Result<(Form, bool), String> {
        let unsupported = || {
            format!(
                "{}: type into a filter bar, a collection or a form field",
                target.written
            )
        };
        if let Some((name, path)) = target.overlay() {
            self.require_overlay(&name)?;
            return match self.overlay_body(&path)? {
                Body::Composite(Composite::Form(form)) => Ok((form, true)),
                _ => Err(unsupported()),
            };
        }
        let section = target.section().ok_or_else(unsupported)?;
        match &self.section_def(&section)?.body {
            Body::Composite(Composite::Form(form)) => Ok((form.clone(), false)),
            _ => Err(unsupported()),
        }
    }

    fn choice_options(&self, choice: &ess_ui::Choice) -> Vec<(Value, String)> {
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
        let request = ReadRequest {
            view: reads
                .view
                .clone()
                .or_else(|| reads.placeholder.clone())
                .unwrap_or_default(),
            fixture: reads.fixture.clone(),
            params: BTreeMap::new(),
        };
        self.adapter
            .borrow()
            .read(&request)
            .map(|result| {
                result
                    .rows
                    .iter()
                    .map(|row| {
                        let value = row.get("id").cloned().unwrap_or_else(|| row.clone());
                        let label = row
                            .get("label")
                            .or_else(|| row.get("name"))
                            .map_or_else(|| scalar(&value), scalar);
                        (value, label)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn require_overlay(&self, name: &str) -> Result<(), String> {
        let screen = self.screen();
        let title = self.overlay_title(name).unwrap_or_else(|| name.to_owned());
        // The terminal records the open overlay by its path: a page's or shell's
        // `overlays/<name>`, or an action's inline `<name>/confirm/overlay`.
        let open = screen.region_where(|path| {
            path.ends_with(&format!("/overlays/{name}"))
                || path.ends_with(&format!("/{name}/confirm/overlay"))
        });
        if screen.overlay_open() && open && screen.text().contains(&title) {
            return Ok(());
        }
        Err(format!("overlay {name} is not open"))
    }

    fn require_no_overlay(&self, target: &Target) -> Result<(), String> {
        if self.screen().overlay_open() {
            return Err(format!(
                "{}: an overlay is open; act on it, or select the page to close it",
                target.written
            ));
        }
        Ok(())
    }

    fn overlay_title(&self, name: &str) -> Option<String> {
        let page = self.doc.pages.get(self.app.page())?;
        let overlay = page.overlays.get(name).or_else(|| {
            self.doc
                .shells
                .get(&page.shell)
                .and_then(|shell| shell.overlays.get(name))
        })?;
        overlay.title.clone()
    }
}

/// The section of a row's collection: the collection is the section's body.
fn row_section(target: &Target, row: &Row) -> Result<String, String> {
    let segments: Vec<&str> = row.container.split('/').collect();
    match segments.as_slice() {
        ["pages", _, "sections", section] => Ok((*section).to_owned()),
        _ => Err(format!(
            "{}: the terminal addresses rows of a section's collection",
            target.written
        )),
    }
}

fn key_of(row: &Value, field: &str) -> String {
    row.get(field).map(scalar).unwrap_or_default()
}

fn label(action: &Action) -> String {
    action.label.clone().unwrap_or_else(|| action.name.clone())
}

/// The key an action hint offers `action` under: hints read `k label · k label`.
fn hint_key<'a>(lines: impl Iterator<Item = &'a str>, action: &Action) -> Option<char> {
    let wanted = label(action);
    for line in lines {
        for hint in line.split(" · ") {
            let hint = hint.trim();
            let mut chars = hint.chars();
            if let (Some(key), Some(' ')) = (chars.next(), chars.next()) {
                if chars.as_str() == wanted {
                    return Some(key);
                }
            }
        }
    }
    None
}

fn key_spec(key: char) -> String {
    match key {
        '<' => "<lt>".to_owned(),
        ' ' => "<space>".to_owned(),
        other => other.to_string(),
    }
}

fn show_input(input: &BTreeMap<String, Value>) -> String {
    let fields: Vec<String> = input
        .iter()
        .map(|(name, value)| format!("{name}={}", scalar(value)))
        .collect();
    format!("{{{}}}", fields.join(", "))
}

/// The fields a form edits, in the order the terminal moves through them (first tab).
fn form_fields(form: &Form) -> Vec<ess_ui::Field> {
    let mut fields = form.fields.clone();
    for group in &form.groups {
        fields.extend(group.fields.iter().cloned());
    }
    if let Some(TabFields::Fields(tab)) = form.tabs.first().and_then(|tab| tab.fields.as_ref()) {
        fields.extend(tab.iter().cloned());
    }
    fields
}

/// `doc` with `fixtures` replacing its view data: every fixture file is named by absolute path,
/// and each replaced view points at a file written under `dir`.
fn with_fixtures(
    doc: &Document,
    base: &Path,
    fixtures: &Fixtures,
    dir: &Path,
) -> Result<Document, String> {
    let mut doc = doc.clone();
    if fixtures.views.is_empty() {
        return Ok(doc);
    }
    let mut index = doc.fixtures.clone().unwrap_or(FixtureIndex {
        dir: None,
        index: None,
        views: BTreeMap::new(),
        derived: BTreeMap::new(),
        scripts: BTreeMap::new(),
    });
    let fixture_dir = base.join(index.dir.clone().unwrap_or_default());
    if let Some(file) = index.index.take() {
        let path = base.join(&file);
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let listed: FixtureIndex =
            serde_yaml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        index.views.extend(listed.views);
        index.derived.extend(listed.derived);
        index.scripts.extend(listed.scripts);
    }
    for file in index.views.values_mut().chain(index.scripts.values_mut()) {
        *file = fixture_dir.join(&*file).display().to_string();
    }
    let file = dir.join("views.yaml");
    let mut views = Mapping::new();
    for (view, data) in &fixtures.views {
        views.insert(Value::String(view.clone()), data.clone());
        index.views.insert(view.clone(), file.display().to_string());
    }
    let mut root = Mapping::new();
    root.insert(Value::String("views".into()), Value::Mapping(views));
    let text = serde_yaml::to_string(&Value::Mapping(root)).map_err(|error| error.to_string())?;
    std::fs::write(&file, text).map_err(|error| format!("{}: {error}", file.display()))?;
    doc.fixtures = Some(index);
    Ok(doc)
}
