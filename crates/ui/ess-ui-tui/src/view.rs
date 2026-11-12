//! Drawing an [`App`] into a ratatui frame.
//!
//! The shell is a top bar (app, page, location and the live-channel status segment), the
//! navigation pane, the page outlet and two bottom lines (notifications or the prompt, and key
//! hints). An open overlay takes the whole frame above the bottom lines. Every composite and
//! primitive is drawn as lines of text; a section is a bordered block around its lines.

use std::borrow::Cow;
use std::fmt::Write as _;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use ess_yaml::Value;
use ratatui::backend::TestBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::{Frame, Terminal};

use ess_ui::{
    Body, ChartKind, Composite, Field, Form, HostSlot, MetricFormat, Node, NodePath, Primitive,
    TabFields, TextFormat, TextStyle, Tone,
};

use crate::app::{
    aggregate, bar_items, board_rows, client_sizes, columns_of, form_fields, graph_collection,
    group_names, paged, references_collection, App, BarItem, Ctx, Focus, Lifecycle, Mark, Prompt,
    ReadState, Region,
};
use crate::expr::{display, field_path, row_key, truthy};

const NAV_WIDTH: u16 = 26;
const SPARKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

fn reversed() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// How the terminal draws a tone (`no_color: emphasis`): `neutral` and `success` bold, `info`
/// italic, `warning` underlined, `danger` bold and underlined. Never reversed, which is the cursor.
fn emphasis(tone: &Tone) -> Style {
    let modifier = match tone {
        Tone::Neutral | Tone::Success => Modifier::BOLD,
        Tone::Info => Modifier::ITALIC,
        Tone::Warning => Modifier::UNDERLINED,
        Tone::Danger => Modifier::BOLD | Modifier::UNDERLINED,
        Tone::Unmapped(_) => Modifier::empty(),
    };
    Style::default().add_modifier(modifier)
}

/// The emphasis a field's value is drawn in: its tone's, or none when the field has no `tone`
/// and no `tone_by`.
fn cell_style(field: &Field, row: &Value) -> Style {
    if field.host.is_some() {
        return Style::default();
    }
    field
        .tone_of(&field_path(row, &field.field))
        .map_or_else(Style::default, |tone| emphasis(&tone))
}

/// `spans` cut to at most `width` terminal cells, ending in `…` when cut, as [`truncate`] cuts
/// their text.
fn truncate_spans(spans: Vec<Span<'static>>, width: usize) -> Vec<Span<'static>> {
    let whole: String = spans.iter().map(|span| span.content.as_ref()).collect();
    if whole.width() <= width {
        return spans;
    }
    let mut keep = truncate(&whole, width).chars().count();
    let ellipsis = width > 0;
    if ellipsis {
        keep -= 1;
    }
    let mut cut = Vec::new();
    for span in spans {
        if keep == 0 {
            break;
        }
        let letters = span.content.chars().count();
        if letters <= keep {
            keep -= letters;
            cut.push(span);
        } else {
            let text: String = span.content.chars().take(keep).collect();
            cut.push(Span::styled(text, span.style));
            keep = 0;
        }
    }
    if ellipsis {
        cut.push(Span::raw("…"));
    }
    cut
}

/// Where a body is drawn from: its node path, its interaction state and what it can see.
#[derive(Clone, Copy)]
struct Place<'a> {
    path: &'a NodePath,
    ui: &'a str,
    ctx: Ctx<'a>,
    focused: bool,
    width: usize,
    /// Whether a collection drawn right here records its rows and cells ([`crate::Region`]): only
    /// a section's or an overlay's own body, whose lines are placed as they are.
    record: bool,
}

/// A section's lines, ready to be placed.
struct SectionBlock {
    /// The sections drawn in the block: one, or those a shared frame holds.
    indices: Vec<usize>,
    path: String,
    title: String,
    lines: Vec<Line<'static>>,
    marks: Vec<Mark>,
}

/// What a refusal shown in a view was sent by.
#[derive(Clone, Copy)]
enum Refusing<'a> {
    Form,
    Confirm,
    Actions(&'a [ess_ui::Action]),
}

/// The cells inside a bordered block.
fn inner(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

impl App {
    fn mark(&self, mark: Mark) {
        self.marks.borrow_mut().push(mark);
    }

    fn take_marks(&self) -> Vec<Mark> {
        std::mem::take(&mut *self.marks.borrow_mut())
    }

    fn region(&self, path: String, row: Option<String>, area: Rect, text: Option<String>) {
        self.regions.borrow_mut().push(Region {
            path,
            row,
            area,
            text,
        });
    }

    /// Places `marks`, made against lines drawn from the top left of `area`, on the screen;
    /// what falls outside `area` was not drawn and is dropped.
    fn place_marks(&self, marks: Vec<Mark>, area: Rect) {
        for mark in marks {
            let (Ok(line), Ok(x)) = (u16::try_from(mark.line), u16::try_from(mark.x)) else {
                continue;
            };
            if line >= area.height || x >= area.width {
                continue;
            }
            let height = u16::try_from(mark.height)
                .unwrap_or(u16::MAX)
                .min(area.height - line);
            let width = mark
                .width
                .map_or(u16::MAX, |width| u16::try_from(width).unwrap_or(u16::MAX))
                .min(area.width - x);
            if height == 0 || width == 0 {
                continue;
            }
            self.region(
                mark.path,
                mark.row,
                Rect {
                    x: area.x + x,
                    y: area.y + line,
                    width,
                    height,
                },
                mark.text,
            );
        }
    }

    /// Renders the screen into a `width` × `height` test backend and returns its text, one line
    /// per row with trailing blanks trimmed.
    pub fn render_text(&self, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("a test terminal");
        terminal
            .draw(|frame| self.draw(frame))
            .expect("a test terminal draws");
        let buffer = terminal.backend().buffer();
        let mut text = String::new();
        for y in 0..buffer.area.height {
            let mut line = String::new();
            for x in 0..buffer.area.width {
                line.push_str(buffer[(x, y)].symbol());
            }
            text.push_str(line.trim_end());
            text.push('\n');
        }
        text
    }

    /// Draws the whole screen.
    pub fn draw(&self, frame: &mut Frame<'_>) {
        self.regions.borrow_mut().clear();
        self.marks.borrow_mut().clear();
        let [top, body, status, hints] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());
        frame.render_widget(Paragraph::new(self.top_bar(top.width as usize)), top);
        if self.overlay.is_some() {
            let pane = Rect {
                y: top.y,
                height: top.height + body.height,
                ..body
            };
            self.draw_overlay(frame, pane);
        } else if self.has_navigation() {
            let [nav, outlet] =
                Layout::horizontal([Constraint::Length(NAV_WIDTH), Constraint::Min(10)])
                    .areas(body);
            self.draw_nav(frame, nav);
            self.draw_outlet(frame, outlet);
        } else {
            self.draw_outlet(frame, body);
        }
        frame.render_widget(Paragraph::new(self.status_line()), status);
        frame.render_widget(Paragraph::new(Line::styled(self.hints(), dim())), hints);
        if let Some(Prompt::Palette(query)) = &self.prompt {
            self.draw_palette(frame, body, query);
        }
    }

    fn top_bar(&self, width: usize) -> Line<'static> {
        let page = self.page_def();
        let title = self
            .doc
            .title
            .clone()
            .unwrap_or_else(|| self.doc.app.clone());
        let left = format!(
            " {title} │ {} │ {}",
            page.title.clone().unwrap_or_else(|| self.page.clone()),
            self.location()
        );
        // A bound run holds no channel open, so its top bar shows no live state.
        let live: Vec<String> = page
            .header
            .as_ref()
            .filter(|_| self.bound.is_none())
            .map(|header| header.live.clone())
            .unwrap_or_default()
            .iter()
            .map(|channel| {
                let status = self.channel_status(channel);
                let mark = if status == "live" { '●' } else { '○' };
                format!("{mark} {channel} {status}")
            })
            .collect();
        let right = live.join("  ");
        let pad = width.saturating_sub(left.width() + right.width() + 1);
        Line::from(vec![
            Span::styled(left, bold()),
            Span::raw(" ".repeat(pad)),
            Span::raw(right),
        ])
    }

    fn status_line(&self) -> Line<'static> {
        match &self.prompt {
            Some(Prompt::Filter { text, .. }) => Line::from(format!("/{text}▏")),
            Some(Prompt::Palette(query)) => Line::from(format!(":{query}▏")),
            Some(Prompt::Upload { does, text }) => Line::from(format!("file for {does}: {text}▏")),
            None => Line::from(
                self.notifications
                    .last()
                    .map(|message| format!(" {message}"))
                    .unwrap_or_default(),
            ),
        }
    }

    fn hints(&self) -> String {
        if self.overlay.is_some() {
            return " esc close · tab field · ctrl-s submit · y confirm".into();
        }
        match self.focus {
            Focus::Nav => " j/k move · enter open · tab focus · : palette · g+letter section · q quit".into(),
            Focus::Section(_) => {
                " tab focus · : palette · g+letter section · / filter · n/p page · s sort · enter act · q quit"
                    .into()
            }
        }
    }

    fn draw_nav(&self, frame: &mut Frame<'_>, area: Rect) {
        let mut lines = Vec::new();
        let mut index = 0;
        for group in self.nav() {
            lines.push(Line::styled(group.label.clone(), bold()));
            for entry in group.entries {
                let current = entry.page == self.page
                    && entry
                        .params
                        .iter()
                        .all(|(key, value)| self.params.get(key) == Some(value));
                let marker = if current { "▸ " } else { "  " };
                let style = if self.focus == Focus::Nav && index == self.nav_cursor {
                    reversed()
                } else {
                    Style::default()
                };
                lines.push(Line::styled(format!("{marker}{}", entry.label), style));
                index += 1;
            }
        }
        let account = self.account_actions();
        if !account.is_empty() {
            lines.push(Line::styled("Account (: to run)", bold()));
            for action in account {
                let label = action.label.unwrap_or(action.name);
                lines.push(Line::styled(format!("  {label}"), dim()));
            }
        }
        let block = Block::bordered().title(" Navigation ");
        let block = if self.focus == Focus::Nav {
            block.border_style(bold())
        } else {
            block
        };
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    #[allow(clippy::too_many_lines)] // the header, the stack, the shared frames and the marks of one outlet
    fn draw_outlet(&self, frame: &mut Frame<'_>, area: Rect) {
        self.marks.borrow_mut().clear();
        let header = self.header_lines(area.width as usize);
        let header_height = u16::try_from(header.len())
            .unwrap_or(u16::MAX)
            .min(area.height);
        let header_area = Rect {
            height: header_height,
            ..area
        };
        if self.page_def().header.is_some() {
            self.region(
                format!("{}/header", self.page_path()),
                None,
                header_area,
                None,
            );
        }
        self.place_marks(self.take_marks(), header_area);
        frame.render_widget(Paragraph::new(header), header_area);
        let mut rest = Rect {
            y: area.y + header_height,
            height: area.height - header_height,
            ..area
        };
        // A page that is a host slot draws no sections: its label, bordered, below the header.
        if let Some(slot) = &self.page_def().host {
            let drawn = Rect {
                height: rest.height.min(3),
                ..rest
            };
            self.region(self.page_path().to_string(), None, drawn, None);
            frame.render_widget(
                Paragraph::new(host_label(slot)).block(Block::bordered()),
                drawn,
            );
            return;
        }
        let width = rest.width.saturating_sub(2) as usize;
        let sections = self.visible_sections();
        let frames = self.shared_frames();
        let mut blocks: Vec<SectionBlock> = Vec::new();
        let mut drawn_paths: Vec<String> = Vec::new();
        for (index, section) in &sections {
            let frame = frames
                .iter()
                .find(|frame| frame.sections.contains(&section.name));
            if let Some(frame) = frame {
                // A shared frame is one block, at the place of the first of its sections.
                let path = frame.path.to_string();
                if !drawn_paths.contains(&path) {
                    let members: Vec<(usize, &ess_ui::Section)> = sections
                        .iter()
                        .filter(|(_, member)| frame.sections.contains(&member.name))
                        .map(|(index, member)| (*index, *member))
                        .collect();
                    blocks.push(self.frame_block(frame, &members, width));
                    drawn_paths.push(path);
                }
                continue;
            }
            self.marks.borrow_mut().clear();
            let (title, lines) = self.section_lines(section, *index, width);
            blocks.push(SectionBlock {
                indices: vec![*index],
                path: self.section_path(&section.name).to_string(),
                title,
                lines,
                marks: self.take_marks(),
            });
        }
        let heights: Vec<u16> = blocks
            .iter()
            .map(|block| u16::try_from(block.lines.len() + 2).unwrap_or(u16::MAX))
            .collect();
        let focused = blocks
            .iter()
            .position(|block| {
                block
                    .indices
                    .iter()
                    .any(|index| Focus::Section(*index) == self.focus)
            })
            .unwrap_or(0);
        let mut start = 0;
        while start < focused && heights[start..=focused].iter().sum::<u16>() > rest.height {
            start += 1;
        }
        for (section, height) in blocks.into_iter().zip(heights).skip(start) {
            if rest.height < 3 {
                break;
            }
            let height = height.min(rest.height);
            let block = Block::bordered().title(section.title);
            let block = if section
                .indices
                .iter()
                .any(|index| Focus::Section(*index) == self.focus)
            {
                block.border_style(bold())
            } else {
                block
            };
            let drawn = Rect { height, ..rest };
            self.region(section.path, None, drawn, None);
            self.place_marks(section.marks, inner(drawn));
            frame.render_widget(Paragraph::new(section.lines).block(block), drawn);
            rest.y += height;
            rest.height -= height;
        }
    }

    #[allow(clippy::too_many_lines)] // title, badge, total, live, switch, metrics, actions, help
    fn header_lines(&self, width: usize) -> Vec<Line<'static>> {
        let page = self.page_def();
        let Some(header) = &page.header else {
            return vec![Line::styled(
                page.title.clone().unwrap_or_else(|| self.page.clone()),
                bold(),
            )];
        };
        // `title_from`: the field of the record the named section holds, live changes applied;
        // the literal title until the record holds it (beyond10x/ess#354).
        let from_record = header.title_from.as_ref().and_then(|from| {
            let row = self.rows(&from.section).into_iter().next()?;
            let value = field_path(&row, &from.field);
            (!value.is_null()).then(|| display(&value))
        });
        let mut title = vec![Span::styled(
            from_record
                .or_else(|| header.title.clone())
                .or_else(|| page.title.clone())
                .unwrap_or_else(|| self.page.clone()),
            bold(),
        )];
        if let Some(badge) = &header.badge {
            let text = self
                .eval(&badge.text.0, &Ctx::default())
                .map_or_else(|| badge.text.0.clone(), |value| display(&value));
            let tone = badge.tone.clone().unwrap_or(Tone::Neutral);
            title.push(Span::raw(" "));
            title.push(Span::styled(format!("[{text}]"), emphasis(&tone)));
        }
        if let Some(total) = &header.total {
            if let Some(count) = self.total(total) {
                title.push(Span::raw(format!(" ({count})")));
            }
        }
        // A bound run holds no channel open, so its header shows no live state.
        if self.bound.is_none() {
            for channel in &header.live {
                let status = self.channel_status(channel);
                title.push(Span::styled(format!("  ● {status}"), dim()));
            }
        }
        let mut switch: Vec<String> = header
            .switch
            .iter()
            .chain(page.switch_to.iter())
            .cloned()
            .collect();
        if !switch.is_empty() && !switch.contains(&self.page) {
            switch.insert(0, self.page.clone());
        }
        if !switch.is_empty() {
            let entries: Vec<String> = switch
                .iter()
                .map(|name| {
                    if *name == self.page {
                        format!("▸ {name}")
                    } else {
                        name.clone()
                    }
                })
                .collect();
            title.push(Span::styled(format!("  ⇄ {}", entries.join(", ")), dim()));
        }
        let mut lines = vec![Line::from(title)];
        if !header.metrics.is_empty() {
            let ctx = Ctx::default();
            let path = self.page_path().child("header");
            let place = Place {
                path: &path,
                ui: "header",
                ctx,
                focused: false,
                width,
                record: false,
            };
            let spans: Vec<Span<'static>> = header
                .metrics
                .iter()
                .flat_map(|node| {
                    let mut spans = flatten(self.node_lines(node, &place));
                    spans.push(Span::raw("   "));
                    spans
                })
                .collect();
            lines.push(Line::from(spans));
        }
        if !header.actions.is_empty() {
            let labels: Vec<String> = header
                .actions
                .iter()
                .map(|action| {
                    format!(
                        "[{}]",
                        action.label.clone().unwrap_or_else(|| action.name.clone())
                    )
                })
                .collect();
            let mut x = 0;
            for (action, label) in header.actions.iter().zip(&labels) {
                self.mark(Mark {
                    path: format!("{}/header/actions/{}", self.page_path(), action.name),
                    row: None,
                    line: lines.len(),
                    height: 1,
                    x,
                    width: Some(label.width()),
                    text: Some(browser_label(action)),
                });
                x += label.width() + 1;
            }
            lines.push(Line::styled(
                format!("{} (: to run)", labels.join(" ")),
                dim(),
            ));
        }
        if let Some(help) = &header.help {
            lines.push(self.header_help_line(help, lines.len()));
        }
        lines
    }

    /// A page header's help as one dim line at `line` of the header, recorded as a node of its
    /// own (`<header>/help`), as the generated React project draws it.
    fn header_help_line(&self, help: &ess_ui::Help, line: usize) -> Line<'static> {
        let text = [help.text.clone(), help.link.clone()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" — ");
        self.mark(Mark {
            path: format!("{}/header/help", self.page_path()),
            row: None,
            line,
            height: 1,
            x: 0,
            width: None,
            text: Some(text.clone()),
        });
        Line::styled(format!("? {text}"), dim())
    }

    fn total(&self, section: &str) -> Option<u64> {
        let section = self
            .page_def()
            .sections
            .iter()
            .find(|candidate| candidate.name == section)?;
        let ctx = Ctx {
            section: Some(&section.name),
            ..Ctx::default()
        };
        let result = self.read_rows(crate::app::body_reads(&section.body)?, &ctx)?;
        Some(result.total.unwrap_or(result.rows.len() as u64))
    }

    fn feeding_channels(&self, section: &ess_ui::Section) -> Vec<String> {
        let mut channels = Vec::new();
        if let Some(live) = &section.live {
            channels.push(live.channel.clone());
        }
        let text = format!("{:?}", section.body);
        for channel in self.doc.channels.keys() {
            if text.contains(&format!("channel.{channel}.")) && !channels.contains(channel) {
                channels.push(channel.clone());
            }
        }
        channels
    }

    /// The block of a shared frame: its heading in the border and, while it is open, each of its
    /// sections under its own heading line, in order. A closed frame is its heading alone.
    fn frame_block(
        &self,
        frame: &crate::app::SharedFrame,
        members: &[(usize, &ess_ui::Section)],
        width: usize,
    ) -> SectionBlock {
        let open = self.is_open(&frame.path, frame.collapsed);
        let marker = fold_marker(frame.collapsed, open);
        let mut block = SectionBlock {
            indices: members.iter().map(|(index, _)| *index).collect(),
            path: frame.path.to_string(),
            title: frame
                .title
                .as_ref()
                .map_or_else(String::new, |title| format!(" {marker}{title} ")),
            lines: Vec::new(),
            marks: Vec::new(),
        };
        if !open {
            return block;
        }
        for (index, section) in members {
            self.marks.borrow_mut().clear();
            let (heading, lines) = self.section_lines(section, *index, width);
            let marks = self.take_marks();
            let start = block.lines.len();
            block
                .lines
                .push(Line::styled(heading.trim().to_owned(), bold()));
            block.marks.push(Mark {
                path: self.section_path(&section.name).to_string(),
                row: None,
                line: start,
                height: lines.len() + 1,
                x: 0,
                width: None,
                text: None,
            });
            for mut mark in marks {
                mark.line += start + 1;
                block.marks.push(mark);
            }
            block.lines.extend(lines);
        }
        block
    }

    #[allow(clippy::too_many_lines)] // one arm per lifecycle state
    fn section_lines(
        &self,
        section: &ess_ui::Section,
        index: usize,
        width: usize,
    ) -> (String, Vec<Line<'static>>) {
        let path = self.section_path(&section.name);
        let ui = format!("s:{}", section.name);
        let lifecycle = self.lifecycle(section);
        let stale = self
            .feeding_channels(section)
            .iter()
            .any(|channel| self.channel_status(channel) == "stale");
        // The name stays first in the border, where a reader of the screen finds the section; the
        // heading follows it (#281).
        let mut title = match &section.title {
            Some(heading) => format!(" {} · {heading} ", section.name),
            None => format!(" {} ", section.name),
        };
        if stale {
            let mark = section
                .states
                .as_ref()
                .and_then(|states| states.stale.as_ref())
                .map_or("badge", |stale| match stale.mark {
                    ess_ui::StaleMark::Dim => "dim",
                    _ => "badge",
                });
            if mark == "badge" {
                title.push_str("[stale] ");
            }
        }
        // A section that may start closed is its heading line while closed.
        if section.collapsed == Some(true) {
            let open = self.is_open(&path, true);
            title = format!(" {}{}", fold_marker(true, open), title.trim_start());
            if !open {
                return (title, Vec::new());
            }
        }
        let states = section.states.as_ref();
        let lines = match lifecycle {
            Lifecycle::NotLoaded => vec![Line::styled("not loaded · R loads it", dim())],
            Lifecycle::Loading => match states.and_then(|states| states.loading.as_ref()) {
                Some(ess_ui::LoadingStyle::Skeleton) => vec![
                    Line::styled("loading…", dim()),
                    Line::styled("░░░░░░░░░░░░░░░░░░░░░░░░", dim()),
                    Line::styled("░░░░░░░░░░░░░░░░", dim()),
                ],
                _ => vec![Line::styled("loading…", dim())],
            },
            Lifecycle::Failed => {
                let error = self
                    .section_request(section)
                    .and_then(|request| match self.read_state(&request) {
                        Some(ReadState::Failed(error)) => Some(error.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                let failed = states.and_then(|states| states.failed.as_ref());
                let mut lines = vec![Line::from(format!(
                    "failed: {}",
                    failed
                        .and_then(|failed| failed.message.clone())
                        .unwrap_or_else(|| error.clone())
                ))];
                if failed.and_then(|failed| failed.message.as_ref()).is_some() {
                    lines.push(Line::styled(error, dim()));
                }
                if failed.is_none_or(|failed| failed.retry) {
                    lines.push(Line::styled("R retries", dim()));
                }
                lines
            }
            Lifecycle::Empty => {
                let empty = states.and_then(|states| states.empty.as_ref());
                // `title: ""` is no title, as React draws no heading for it.
                let mut lines: Vec<Line<'static>> = empty
                    .and_then(|empty| empty.title.clone())
                    .filter(|title| !title.is_empty())
                    .map(|title| Line::styled(title, bold()))
                    .into_iter()
                    .collect();
                lines.push(Line::from(empty.map_or_else(
                    || "No rows".to_owned(),
                    |empty| empty.message.clone(),
                )));
                if let Some(action) = empty.and_then(|empty| empty.action.as_ref()) {
                    lines.push(Line::styled(
                        format!(
                            "[{}]",
                            action.label.clone().unwrap_or_else(|| action.name.clone())
                        ),
                        dim(),
                    ));
                }
                lines
            }
            Lifecycle::Ready => {
                let place = Place {
                    path: &path,
                    ui: &ui,
                    ctx: Ctx {
                        section: Some(&section.name),
                        ..Ctx::default()
                    },
                    focused: Focus::Section(index) == self.focus,
                    width,
                    record: true,
                };
                let mut lines = self.body_lines(&section.body, &place);
                for child in &section.children {
                    let drawn = self.node_lines(child, &place);
                    if let Some(name) = &child.common.name {
                        self.mark(Mark {
                            path: format!("{path}/children/{name}"),
                            row: None,
                            line: lines.len(),
                            height: drawn.len(),
                            x: 0,
                            width: None,
                            text: None,
                        });
                    }
                    lines.extend(drawn);
                }
                lines
            }
        };
        let lines = if stale
            && states
                .and_then(|states| states.stale.as_ref())
                .is_some_and(|stale| {
                    matches!(stale.mark, ess_ui::StaleMark::Dim | ess_ui::StaleMark::Both)
                }) {
            lines
                .into_iter()
                .map(|line| line.patch_style(dim()))
                .collect()
        } else {
            lines
        };
        (title, lines)
    }

    fn draw_overlay(&self, frame: &mut Frame<'_>, area: Rect) {
        let Some(open) = &self.overlay else { return };
        let overlay = &open.overlay;
        let kind = match overlay.kind {
            ess_ui::OverlayKind::Drawer => "drawer",
            ess_ui::OverlayKind::Dialog => "dialog",
            ess_ui::OverlayKind::Fullscreen => "fullscreen",
            ess_ui::OverlayKind::Popover => "popover",
            ess_ui::OverlayKind::Unmapped(_) => "overlay",
        };
        let title = overlay.title.clone().unwrap_or_else(|| open.name.clone());
        let ui = format!("o:{}", open.name);
        let place = Place {
            path: &open.path,
            ui: &ui,
            ctx: Ctx {
                overlay: true,
                ..Ctx::default()
            },
            focused: true,
            width: area.width.saturating_sub(2) as usize,
            record: true,
        };
        let mut lines = vec![Line::styled(title.clone(), bold()), Line::raw("")];
        self.marks.borrow_mut().clear();
        lines.extend(self.body_lines(&overlay.body, &place));
        let mut marks = self.take_marks();
        for mark in &mut marks {
            mark.line += 2;
        }
        self.region(
            open.drawn_at
                .clone()
                .unwrap_or_else(|| open.path.to_string()),
            None,
            area,
            None,
        );
        self.place_marks(marks, inner(area));
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::bordered()
                    .title(format!(" {title} · {kind} "))
                    .title_bottom(" esc closes "),
            ),
            area,
        );
    }

    fn draw_palette(&self, frame: &mut Frame<'_>, area: Rect, query: &str) {
        let matches = self.palette(query);
        let lines: Vec<Line<'static>> = std::iter::once(Line::from(format!(":{query}▏")))
            .chain(matches.iter().take(8).enumerate().map(|(index, item)| {
                let style = if index == 0 {
                    reversed()
                } else {
                    Style::default()
                };
                Line::styled(item.label.clone(), style)
            }))
            .collect();
        let height = u16::try_from(lines.len() + 2)
            .unwrap_or(u16::MAX)
            .min(area.height);
        let width = area.width.min(48);
        let popup = Rect {
            x: area.x + area.width.saturating_sub(width) / 2,
            y: area.y + 1,
            width,
            height,
        }
        .intersection(area)
        .intersection(frame.area());
        // Too small for a bordered list: the prompt line still shows what is typed.
        if popup.width < 3 || popup.height < 3 {
            return;
        }
        frame.render_widget(Clear, popup);
        frame.render_widget(
            Paragraph::new(lines).block(Block::bordered().title(" go to ")),
            popup,
        );
    }

    // ── bodies ──────────────────────────────────────────────────────────────────────────────

    fn node_lines(&self, node: &Node, place: &Place<'_>) -> Vec<Line<'static>> {
        if !self.visible(
            node.common.visible.as_ref().map(|expr| expr.0.as_str()),
            &place.ctx,
        ) {
            return Vec::new();
        }
        let path = match &node.common.name {
            Some(name) => place.path.child(name),
            None => place.path.clone(),
        };
        let inner = Place {
            path: &path,
            ctx: Ctx {
                owner: Some(&path),
                ..place.ctx
            },
            record: false,
            ..*place
        };
        let mut lines = self.body_lines(&node.body, &inner);
        // After the body, so the marks it recorded keep their lines.
        lines.extend(self.live_stale(node));
        lines
    }

    /// Draws through `built`, a collection made from a graph editor or references list whose own
    /// `Reads` is `origin`, so a nested live node's own rows are the rows drawn
    /// (beyond10x/ess#354).
    fn aliased<R>(
        &self,
        built: &ess_ui::Collection,
        origin: &ess_ui::Reads,
        draw: impl FnOnce() -> R,
    ) -> R {
        let Some(reads) = built.reads.as_ref() else {
            return draw();
        };
        let pair = (
            std::ptr::from_ref(reads) as usize,
            std::ptr::from_ref(origin) as usize,
        );
        let previous = self.alias.replace(Some(pair));
        let drawn = draw();
        self.alias.replace(previous);
        drawn
    }

    /// `[stale]` after a nested live node whose channel is stale: a stale value is marked stale,
    /// never shown as current (beyond10x/ess#354).
    fn live_stale(&self, node: &Node) -> Option<Line<'static>> {
        let live = node.live.as_ref()?;
        (self.channel_status(&live.channel) == "stale").then(|| Line::styled("[stale]", reversed()))
    }

    fn body_lines(&self, body: &Body, place: &Place<'_>) -> Vec<Line<'static>> {
        match body {
            Body::Primitive(primitive) => {
                vec![Line::from(self.primitive_spans(primitive, &place.ctx))]
            }
            Body::Widget(widget) => {
                let arrange = self
                    .doc
                    .widgets
                    .get(&widget.component)
                    .and_then(|widget| widget.arrange.clone());
                let lines: Vec<Vec<Line<'static>>> = widget
                    .body
                    .iter()
                    .map(|node| self.node_lines(node, place))
                    .collect();
                if arrange == Some(ess_ui::Arrange::Column) {
                    lines.into_iter().flatten().collect()
                } else {
                    vec![Line::from(
                        lines
                            .into_iter()
                            .flat_map(|lines| {
                                let mut spans = flatten(lines);
                                spans.push(Span::raw(" "));
                                spans
                            })
                            .collect::<Vec<_>>(),
                    )]
                }
            }
            Body::Composite(composite) => self.composite_lines(composite, place),
            Body::Host(slot) => host_box(slot),
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per composite kind
    fn composite_lines(&self, composite: &Composite, place: &Place<'_>) -> Vec<Line<'static>> {
        match composite {
            Composite::Collection(collection) => {
                self.collection_lines(collection, place, Some("row_actions"))
            }
            Composite::Record(record) => {
                let row = self.record_row(record, &place.ctx).unwrap_or(Value::Null);
                let mut lines = Vec::new();
                let shown = self.labelled(&record.fields, &row);
                for field in &record.fields {
                    self.mark_field_slot(field, place, &place.path.child("fields"), lines.len());
                    lines.push(labelled(field, &shown));
                }
                // Groups of fields under their headings; a collapsed one is its heading line.
                for group in &record.groups {
                    let at = place.path.child("groups").child(&group.name);
                    let collapsed = group.collapsed == Some(true);
                    let open = self.is_open(&at, collapsed);
                    let heading = group.label.clone().unwrap_or_else(|| group.name.clone());
                    lines.push(Line::styled(
                        format!("{}{heading}", fold_marker(collapsed, open)),
                        bold(),
                    ));
                    if open {
                        let shown = self.labelled(&group.fields, &row);
                        for field in &group.fields {
                            self.mark_field_slot(field, place, &at.child("fields"), lines.len());
                            lines.push(labelled(field, &shown));
                        }
                    }
                }
                let tab = self
                    .ui(place.ui)
                    .tab
                    .min(record.tabs.len().saturating_sub(1));
                let inner = Place {
                    ctx: Ctx {
                        row: Some(&row),
                        ..place.ctx
                    },
                    record: false,
                    ..*place
                };
                if !record.tabs.is_empty() {
                    self.mark_tabs(&record.tabs, place, lines.len());
                    lines.push(tab_line(
                        record
                            .tabs
                            .iter()
                            .map(|tab| tab.label.clone().unwrap_or_else(|| tab.name.clone())),
                        tab,
                    ));
                    let current = record.tabs.get(tab);
                    if let Some(TabFields::Fields(fields)) =
                        current.and_then(|tab| tab.fields.as_ref())
                    {
                        let shown = self.labelled(fields, &row);
                        let at = current.map_or_else(
                            || place.path.child("fields"),
                            |tab| place.path.child("tabs").child(&tab.name).child("fields"),
                        );
                        for field in fields {
                            self.mark_field_slot(field, place, &at, lines.len());
                            lines.push(labelled(field, &shown));
                        }
                    }
                    // A tab's nested node draws below its fields, as the generated app shows it.
                    if let (Some(current), Some(ess_ui::TabForm::Node(node))) =
                        (current, current.and_then(|tab| tab.form.as_ref()))
                    {
                        let path = place.path.child("tabs").child(&current.name).child("form");
                        let nested = Place {
                            path: &path,
                            ..inner
                        };
                        let drawn = self.body_lines(&node.body, &nested);
                        // A section's or an overlay's own record records a tab's slot at its
                        // path, so a test reads its label there.
                        if place.record && node.body.host().is_some() {
                            self.mark(Mark {
                                path: path.to_string(),
                                row: None,
                                line: lines.len(),
                                height: drawn.len(),
                                x: 0,
                                width: None,
                                text: None,
                            });
                        }
                        lines.extend(drawn);
                        lines.extend(self.live_stale(node));
                    }
                }
                for node in &record.item {
                    lines.extend(self.node_lines(node, &inner));
                }
                let actions = record_actions(record, tab);
                lines.extend(self.action_hint(&actions, &inner.ctx, ""));
                lines.extend(self.refusal_lines(place.ui, Refusing::Actions(&actions)));
                lines
            }
            Composite::Form(form) => self.form_lines(form, place),
            Composite::Choice(choice) => {
                let value = choice
                    .binds
                    .as_ref()
                    .and_then(|binds| self.eval(&binds.0, &place.ctx))
                    .unwrap_or(Value::Null);
                // A choice that is a section of its own shows its option cursor while focused.
                let cursor = place
                    .focused
                    .then(|| self.ui(place.ui).option)
                    .filter(|_| choice.binds.is_some());
                vec![Line::from(self.choice_spans(choice, &value, place, cursor))]
            }
            Composite::FilterBar(bar) => self.bar_lines(bar, place),
            Composite::Confirm(confirm) => self.confirm_lines(confirm, place),
            Composite::Metric(metric) => vec![Line::from(self.metric_spans(metric, place))],
            Composite::Chart(chart) => self.chart_lines(chart, place),
            Composite::Board(board) => {
                let rows = board_rows(self, board, &place.ctx);
                let by = board.widget_by.as_deref().unwrap_or("type");
                let cursor = self.ui(place.ui).cursor;
                let mut lines = Vec::new();
                for (index, row) in rows.iter().enumerate() {
                    let kind = display(&row[by]);
                    let style = if place.focused && index == cursor {
                        reversed()
                    } else {
                        bold()
                    };
                    lines.push(Line::styled(
                        format!("── {kind} ({}) ──", display(&row["widget"])),
                        style,
                    ));
                    if let Some(node) = board.widgets.get(&kind) {
                        let path = place.path.child("widgets").child(&kind);
                        let inner = Place {
                            path: &path,
                            ui: "board-widget",
                            focused: false,
                            record: false,
                            ..*place
                        };
                        lines.extend(self.body_lines(&node.body, &inner));
                    }
                }
                let row = rows.get(cursor);
                let ctx = Ctx { row, ..place.ctx };
                lines.extend(self.action_hint(&board.item_actions, &ctx, ""));
                lines.extend(self.refusal_lines(place.ui, Refusing::Actions(&board.item_actions)));
                lines
            }
            Composite::GraphEditor(editor) => {
                let collection = graph_collection(editor);
                let mut lines: Vec<Line<'static>> = editor
                    .toolbar
                    .iter()
                    .map(|node| Line::from(flatten(self.node_lines(node, place))))
                    .collect();
                let before = self.marks.borrow().len();
                lines.extend(self.aliased(&collection, &editor.reads, || {
                    self.collection_lines(&collection, place, None)
                }));
                for mark in self.marks.borrow_mut().iter_mut().skip(before) {
                    mark.line += editor.toolbar.len();
                }
                lines.extend(self.aliased(&collection, &editor.reads, || {
                    self.graph_edge_lines(editor, &collection, place)
                }));
                lines
            }
            Composite::RichText(text) => {
                let value = text
                    .binds
                    .as_ref()
                    .and_then(|binds| self.eval(&binds.0, &place.ctx))
                    .map(|value| display(&value))
                    .unwrap_or_default();
                let syntax = text.syntax.as_ref().map_or("plain", |syntax| match syntax {
                    ess_ui::RichTextSyntax::Expression => "expression",
                    ess_ui::RichTextSyntax::Ssml => "ssml",
                    ess_ui::RichTextSyntax::Json => "json",
                    ess_ui::RichTextSyntax::Curl => "curl",
                    _ => "plain",
                });
                let mut lines = vec![Line::from(format!("✎ [{value:<30}] ({syntax})"))];
                if let Some(view) = &text.completes {
                    let request = crate::data::ReadRequest {
                        view: view.clone(),
                        fixture: None,
                        params: std::collections::BTreeMap::new(),
                    };
                    if let Some(result) = self.rows_of(&request) {
                        let labels: Vec<String> = result
                            .rows
                            .iter()
                            .map(|row| display(&row["label"]))
                            .collect();
                        lines.push(Line::styled(
                            format!("completes: {}", labels.join(" ")),
                            dim(),
                        ));
                    }
                }
                lines
            }
            Composite::References(references) => {
                let collection = references_collection(references);
                self.aliased(&collection, &references.reads, || {
                    self.collection_lines(&collection, place, None)
                })
            }
        }
    }

    /// Records each row action a recorded row offers at `<rows>/<key>/<segment>/<name>`, carrying
    /// the label its control shows in the generated React project, which renders the actions on
    /// every row. The cursor row's actions are placed on their entry in the hint line drawn at
    /// `hint_line` ([`App::action_hint`]); another row's, which the terminal offers once its
    /// row has the cursor, on that row's line.
    #[allow(clippy::too_many_arguments)]
    fn mark_row_actions(
        &self,
        actions: &[ess_ui::Action],
        rows_path: &str,
        segment: &str,
        recorded: &[(String, usize, &Value)],
        cursor_key: Option<&str>,
        hint_line: usize,
        place: &Place<'_>,
    ) {
        for (key, line, row) in recorded {
            let ctx = Ctx {
                row: Some(row),
                ..place.ctx
            };
            // Where each keyed action's label starts in the hint: `<key> <switch><label>`, joined
            // by ` · `.
            let mut hinted = std::collections::BTreeMap::new();
            if cursor_key == Some(key.as_str()) {
                let mut x = 0;
                for (letter, action) in self.action_keys(actions, &ctx) {
                    let shown = action.label.clone().unwrap_or_else(|| action.name.clone());
                    let start = x + letter.len_utf8() + 1 + self.switch_mark(&action, &ctx).width();
                    hinted.insert(action.name.clone(), (start, shown.width()));
                    x = start + shown.width() + " · ".width();
                }
            }
            for action in actions {
                if !self.visible(action.visible.as_ref().map(|expr| expr.0.as_str()), &ctx) {
                    continue;
                }
                let (line, x, width) = match hinted.get(&action.name) {
                    Some((x, width)) => (hint_line, *x, Some(*width)),
                    None => (*line, 0, None),
                };
                self.mark(Mark {
                    path: format!("{rows_path}/{key}/{segment}/{}", action.name),
                    row: Some(key.clone()),
                    line,
                    height: 1,
                    x,
                    width,
                    text: Some(browser_label(action)),
                });
            }
        }
    }

    /// The refusal of the last command sent from the view `ui`, when `by` sent it: `form`,
    /// `confirm`, or one of `actions`.
    fn refusal_lines(&self, ui: &str, by: Refusing<'_>) -> Vec<Line<'static>> {
        let Some(refused) = self.ui(ui).refusal else {
            return Vec::new();
        };
        let sent = match by {
            Refusing::Form => refused.by == "form",
            Refusing::Confirm => refused.by == "confirm",
            Refusing::Actions(actions) => actions.iter().any(|action| action.name == refused.by),
        };
        if !sent {
            return Vec::new();
        }
        let mut lines = vec![Line::styled(format!("✗ {}", refused.text), bold())];
        if let Some(Value::Mapping(fields)) = &refused.payload {
            let fields: Vec<String> = fields
                .iter()
                .map(|(name, value)| format!("{}: {}", display(name), display(value)))
                .collect();
            if !fields.is_empty() {
                lines.push(Line::from(format!("  {}", fields.join(" · "))));
            }
        }
        lines
    }

    /// The keyed actions of a view, as its key handler reaches them: none takes one of the letters
    /// of `view_keys`, which the view takes itself.
    fn action_hint(
        &self,
        actions: &[ess_ui::Action],
        ctx: &Ctx<'_>,
        view_keys: &str,
    ) -> Vec<Line<'static>> {
        let keyed = self.action_keys_besides(actions, ctx, view_keys);
        if keyed.is_empty() {
            return Vec::new();
        }
        let hint: Vec<String> = keyed
            .iter()
            .map(|(key, action)| {
                format!(
                    "{key} {}{}",
                    self.switch_mark(action, ctx),
                    action.label.clone().unwrap_or_else(|| action.name.clone())
                )
            })
            .collect();
        vec![Line::styled(hint.join(" · "), dim())]
    }

    /// A toggle's on/off mark before its label in the hint (`[x] ` or `[ ] `, as a toggle column
    /// draws it); nothing for another action.
    fn switch_mark(&self, action: &ess_ui::Action, ctx: &Ctx<'_>) -> &'static str {
        match self.toggle_on(action, ctx) {
            Some(true) => "[x] ",
            Some(false) => "[ ] ",
            None => "",
        }
    }

    /// A collection's lines. `row_actions_at` is the path segment the document lists the row
    /// actions under (`row_actions` for a collection); `None` records no row action.
    #[allow(clippy::too_many_lines)] // header, rows, detail strip, expansion and footer
    fn collection_lines(
        &self,
        collection: &ess_ui::Collection,
        place: &Place<'_>,
        row_actions_at: Option<&str>,
    ) -> Vec<Line<'static>> {
        let Some(reads) = &collection.reads else {
            return vec![Line::styled("(no reads)", dim())];
        };
        let request = self.request(reads, &place.ctx);
        let read = match self.read_state(&request) {
            Some(ReadState::Ready(_)) => self.read_rows(reads, &place.ctx),
            Some(ReadState::Failed(error)) => return vec![Line::from(format!("failed: {error}"))],
            _ => return vec![Line::styled("loading…", dim())],
        };
        let all: &[Value] = read.as_ref().map_or(&[], |result| &result.rows);
        let columns: Vec<Field> = columns_of(collection, all)
            .into_iter()
            .filter(|field| {
                self.visible(
                    field.visible.as_ref().map(|expr| expr.0.as_str()),
                    &place.ctx,
                )
            })
            .collect();
        // Filtered and sorted once: the page shown and the footer's row count both read it.
        let arranged = self.arranged_rows(place.ui, collection, all);
        let (rows, pages) = self.page_of(place.ui, collection, &arranged);
        let state = self.ui(place.ui);
        let cursor = state.cursor.min(rows.len().saturating_sub(1));
        let cards = matches!(
            collection.style,
            Some(ess_ui::CollectionStyle::Cards | ess_ui::CollectionStyle::List)
        );
        // The generated React project renders column headers for a table only, and a row's cells
        // for cards, a list or a tree only when the collection has no `item`: a node it renders
        // no element for is not recorded here either.
        let table = matches!(
            collection.style,
            None | Some(ess_ui::CollectionStyle::Table)
        );
        let record_cells = table || collection.item.is_empty();
        let mut lines = Vec::new();
        // Each row as shown: a `label_from` column holds its label (beyond10x/ess#364).
        let labels = self.label_index(&columns);
        let shown_rows: Vec<Cow<'_, Value>> = rows
            .iter()
            .map(|row| App::labelled_by(&labels, &columns, row))
            .collect();
        let natural: Vec<usize> = columns
            .iter()
            .map(|field| {
                shown_rows
                    .iter()
                    .map(|row| cell(field, row).width())
                    .chain(std::iter::once(label_of(field).width()))
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        // A line is the two-cell selection mark, then the columns.
        let widths = column_widths(&natural, place.width.saturating_sub(2));
        // Recorded paths: the collection's, and its columns' where the document declares them.
        let container = place.path.to_string();
        let column_path = |field: &Field| match &collection.columns {
            Some(ess_ui::Columns::Fixed(_)) => Some(format!("columns/{}", field.name)),
            Some(ess_ui::Columns::Selectable(_)) => Some(format!("columns/all/{}", field.name)),
            _ => None,
        };
        // The collection's own read names its key first (an overlay or a widget holds one), then
        // the section it is placed in (#320).
        let key_field = collection
            .reads
            .as_ref()
            .and_then(|reads| reads.key.clone())
            .or_else(|| {
                place
                    .ctx
                    .section
                    .and_then(|name| {
                        self.page_def()
                            .sections
                            .iter()
                            .find(|section| section.name == name)
                    })
                    .and_then(|section| section.row_key().map(str::to_owned))
            })
            .unwrap_or_else(|| "id".to_owned());
        // Where each column starts in a table line: after the two-cell selection mark, columns
        // are padded to their width and separated by two cells.
        let starts: Vec<usize> = widths
            .iter()
            .scan(2, |x, width| {
                let start = *x;
                *x += width + 2;
                Some(start)
            })
            .collect();
        if !cards {
            let header: Vec<String> = columns
                .iter()
                .zip(&widths)
                .map(|(field, width)| pad(&label_of(field), *width))
                .collect();
            if place.record && table {
                for ((field, width), x) in columns.iter().zip(&widths).zip(&starts) {
                    if let Some(column) = column_path(field) {
                        self.mark(Mark {
                            path: format!("{container}/{column}"),
                            row: None,
                            line: lines.len(),
                            height: 1,
                            x: *x,
                            width: Some(*width),
                            text: Some(browser_field_label(field)),
                        });
                    }
                }
            }
            lines.push(Line::styled(format!("  {}", header.join("  ")), bold()));
            // A column's help, one line each, under the heading.
            for field in &columns {
                let lead = format!("  {}: ", label_of(field));
                lines.extend(field.help.as_ref().and_then(|help| help_line(&lead, help)));
            }
        }
        let mut group = None;
        // The groups in the order they are shown, and how many of them are already drawn: a
        // declared group no row falls under is drawn, empty, where it stands in that order.
        let groups = group_names(collection, all);
        let mut drawn = 0;
        let empty_heading = |name: &str, lines: &mut Vec<Line<'static>>| {
            if let Some(by) = &collection.group_by {
                if collection.show_empty_groups
                    && !all.iter().any(|row| display(&row[by.as_str()]) == name)
                {
                    lines.push(Line::styled(format!("── {by}: {name} ──"), bold()));
                    lines.push(Line::styled("  (none)", dim()));
                }
            }
        };
        // Each recorded row: its key, its line and its data.
        let mut recorded: Vec<(String, usize, &Value)> = Vec::new();
        for (index, &row) in rows.iter().enumerate() {
            if let Some(by) = &collection.group_by {
                let current = display(&row[by.as_str()]);
                if group.as_ref() != Some(&current) {
                    if let Some(at) = groups.iter().position(|name| *name == current) {
                        for name in groups.get(drawn..at).unwrap_or_default() {
                            empty_heading(name, &mut lines);
                        }
                        drawn = drawn.max(at + 1);
                    }
                    lines.push(Line::styled(format!("── {by}: {current} ──"), bold()));
                    group = Some(current);
                }
            }
            let selected =
                state
                    .selected
                    .contains(&self.selection_key(row, Some(reads), place.ctx.section));
            let mark = if selected { "✓ " } else { "  " };
            let text = if cards {
                let mut parts: Vec<String> = columns
                    .iter()
                    .map(|field| cell(field, &shown_rows[index]))
                    .collect();
                parts.retain(|part| !part.is_empty());
                parts.join(" · ")
            } else {
                columns
                    .iter()
                    .zip(&widths)
                    .map(|(field, width)| pad(&cell(field, &shown_rows[index]), *width))
                    .collect::<Vec<_>>()
                    .join("  ")
            };
            let style = if place.focused && index == cursor {
                reversed()
            } else {
                Style::default()
            };
            if place.record {
                let key = row_key(row, &key_field);
                // The row's and each cell's whole text: the screen cuts a cell at its column's
                // width and a line at the box's, the generated React project does not.
                let whole = if cards {
                    text.clone()
                } else {
                    columns
                        .iter()
                        .map(|field| cell(field, &shown_rows[index]))
                        .collect::<Vec<_>>()
                        .join("  ")
                };
                self.mark(Mark {
                    path: format!("{container}/rows/{key}"),
                    row: Some(key.clone()),
                    line: lines.len(),
                    height: 1,
                    x: 0,
                    width: None,
                    text: Some(whole),
                });
                recorded.push((key.clone(), lines.len(), row));
                let mut x = 2;
                for (field, (width, start)) in columns.iter().zip(widths.iter().zip(&starts)) {
                    let shown = cell(field, &shown_rows[index]);
                    let (at, cells) = if cards {
                        if shown.is_empty() {
                            continue;
                        }
                        let at = x;
                        x += shown.width() + " · ".width();
                        (at, shown.width())
                    } else {
                        (*start, *width)
                    };
                    if !record_cells {
                        continue;
                    }
                    if let Some(column) = column_path(field) {
                        self.mark(Mark {
                            path: format!("{container}/rows/{key}/{column}"),
                            row: Some(key.clone()),
                            line: lines.len(),
                            height: 1,
                            x: at,
                            width: Some(cells),
                            text: Some(shown),
                        });
                    }
                }
            }
            // The row's text, each cell in the emphasis of its field's tone.
            let mut spans = vec![Span::raw(mark)];
            if cards {
                let shown = columns
                    .iter()
                    .map(|field| (cell(field, &shown_rows[index]), field))
                    .filter(|(text, _)| !text.is_empty());
                for (at, (text, field)) in shown.enumerate() {
                    if at > 0 {
                        spans.push(Span::raw(" · "));
                    }
                    spans.push(Span::styled(text, cell_style(field, &shown_rows[index])));
                }
            } else {
                for (at, (field, width)) in columns.iter().zip(&widths).enumerate() {
                    if at > 0 {
                        spans.push(Span::raw("  "));
                    }
                    spans.push(Span::styled(
                        pad(&cell(field, &shown_rows[index]), *width),
                        cell_style(field, &shown_rows[index]),
                    ));
                }
            }
            lines.push(Line::from(truncate_spans(spans, place.width)).style(style));
        }
        if collection.group_by.is_some() && state.page.min(pages - 1) + 1 == pages {
            for name in groups.get(drawn..).unwrap_or_default() {
                empty_heading(name, &mut lines);
            }
        }
        let row = rows.get(cursor).copied();
        let row_ctx = Ctx { row, ..place.ctx };
        if row.is_some() {
            let inner = Place {
                ctx: row_ctx,
                record: false,
                ..*place
            };
            if !collection.item.is_empty() {
                let spans: Vec<Span<'static>> = collection
                    .item
                    .iter()
                    .flat_map(|node| {
                        let mut spans = flatten(self.node_lines(node, &inner));
                        spans.push(Span::raw("  "));
                        spans
                    })
                    .collect();
                let mut line = vec![Span::styled("▸ ", dim())];
                line.extend(spans);
                lines.push(Line::from(line));
            }
            if state.expanded {
                if let Some(expand) = &collection.expand {
                    let start = lines.len();
                    for line in self.node_lines(expand, &inner) {
                        let mut spans = vec![Span::raw("    ")];
                        spans.extend(line.spans);
                        lines.push(Line::from(spans));
                    }
                    // The expanded detail is the cursor row's, drawn below the rows: its own
                    // region, at the path React scopes it to.
                    if place.record && lines.len() > start {
                        if let Some(row) = row {
                            let key = row_key(row, &key_field);
                            self.mark(Mark {
                                path: format!("{container}/rows/{key}/expand"),
                                row: Some(key),
                                line: start,
                                height: lines.len() - start,
                                x: 0,
                                width: None,
                                text: None,
                            });
                        }
                    }
                }
            }
        }
        let page = state.page.min(pages - 1);
        let mut footer = vec![format!("page {}/{pages}", page + 1)];
        footer.push(format!("{} rows", arranged.len()));
        // A paged read's footer shows the rows drawn as a range of all of them, as React's
        // pager does; it follows `page N/M · N rows`, so a reader of those reads them as before.
        // A server pager's range is the server's page of the server's total, from the page and
        // size params the read sent, and none without a size or a total, as in React.
        let size = self.page_size(place.ui, collection);
        let range = match (&reads.paging, &reads.pager) {
            (Some(ess_ui::Paging::Server), Some(server_pager)) => server_range(
                server_pager,
                &request.params,
                read.as_ref().and_then(|result| result.total),
                arranged.len(),
            ),
            _ if paged(collection) => page_range(
                page as u64 + 1,
                size as u64,
                rows.len(),
                arranged.len() as u64,
            ),
            _ => None,
        };
        footer.extend(range);
        if client_sizes(collection).is_some_and(|sizes| sizes.len() > 1) {
            footer.push(format!("+ {size} per page"));
        }
        if let Some((by, descending)) = state.sort.clone().or_else(|| {
            collection
                .sort
                .as_ref()
                .map(|sort| (sort.by.clone(), sort.dir == Some(ess_ui::SortDir::Desc)))
        }) {
            footer.push(format!("sort {by} {}", if descending { '↓' } else { '↑' }));
        }
        if !state.filter.is_empty() {
            footer.push(format!("filter \"{}\"", state.filter));
        }
        if state.new_rows > 0 {
            footer.push(format!("+{} new", state.new_rows));
        }
        if collection.expand.is_some() {
            footer.push("x expand".into());
        }
        if self.plan.fallback(place.path, "no_drag") == Some("move_buttons") {
            footer.push("J/K move".into());
        }
        lines.push(Line::styled(footer.join(" · "), dim()));
        if let Some(segment) = row_actions_at {
            let cursor_key = rows.get(cursor).map(|row| row_key(row, &key_field));
            self.mark_row_actions(
                &collection.row_actions,
                &format!("{container}/rows"),
                segment,
                &recorded,
                cursor_key.as_deref(),
                lines.len(),
                place,
            );
        }
        lines.extend(self.action_hint(&collection.row_actions, &row_ctx, ""));
        lines.extend(self.refusal_lines(place.ui, Refusing::Actions(&collection.row_actions)));
        let bulk = crate::app::bulk_keys(&collection.bulk_actions);
        if !bulk.is_empty() {
            let offered: Vec<String> = bulk
                .iter()
                .map(|(key, action)| {
                    format!(
                        "{key} {}",
                        action.label.clone().unwrap_or_else(|| action.name.clone())
                    )
                })
                .collect();
            let line = if state.selected.is_empty() {
                format!("space selects rows for: {}", offered.join(" · "))
            } else {
                format!("{} selected: {}", state.selected.len(), offered.join(" · "))
            };
            lines.push(Line::styled(line, dim()));
            lines.extend(self.refusal_lines(place.ui, Refusing::Actions(&collection.bulk_actions)));
        }
        lines
    }
}

/// The label an action's control shows in the generated React project: its `label`, else its name
/// with `_` read as a space.
/// What leads the heading of a part that may start closed: `▸` while closed, `▾` while open, and
/// nothing on one that cannot fold.
fn fold_marker(collapsible: bool, open: bool) -> &'static str {
    match (collapsible, open) {
        (false, _) => "",
        (true, true) => "▾ ",
        (true, false) => "▸ ",
    }
}

fn browser_label(action: &ess_ui::Action) -> String {
    action
        .label
        .clone()
        .unwrap_or_else(|| action.name.replace('_', " "))
}

/// The label of a host slot: `<host>: <reason>`. The terminal cannot draw the host's component,
/// so it marks the slot's place with this.
fn host_label(slot: &HostSlot) -> String {
    format!("{}: {}", slot.host, slot.reason)
}

/// A host slot drawn as a bordered box around its label.
fn host_box(slot: &HostSlot) -> Vec<Line<'static>> {
    let label = host_label(slot);
    let rule = "─".repeat(label.width());
    vec![
        Line::from(format!("┌{rule}┐")),
        Line::from(format!("│{label}│")),
        Line::from(format!("└{rule}┘")),
    ]
}

/// One cell of a row, drawn by the field's `as`; a field slot's label, bordered, in place of
/// its value.
fn cell(field: &Field, row: &Value) -> String {
    if let Some(slot) = &field.host {
        return format!("│{}│", host_label(slot));
    }
    let value = field_path(row, &field.field);
    labelled_cell(field, &value).unwrap_or_else(|| cell_value(field, &value))
}

/// A displayed value the field labels (`labels`, or a choice's fixed option), drawn by its `as`;
/// `None` when it has no label or its `as` does not show a value as text.
fn labelled_cell(field: &Field, value: &Value) -> Option<String> {
    let label = field.value_label(value)?;
    match field.field_as.as_deref() {
        Some("badge") => Some(format!("[{label}]")),
        Some("choice") => Some(format!("‹{label}›")),
        Some("tags" | "secret" | "toggle" | "json") => None,
        _ => Some(label.to_owned()),
    }
}

/// A field's value, drawn by the field's `as`.
fn cell_value(field: &Field, value: &Value) -> String {
    match field.field_as.as_deref() {
        _ if value.is_null() => String::new(),
        Some("badge") => format!("[{}]", display(value)),
        Some("tags") => value.as_sequence().map_or_else(
            || display(value),
            |items| {
                // A null entry carries no tag, so it draws no chip.
                items
                    .iter()
                    .filter(|item| !item.is_null())
                    .map(|item| format!("#{}", display(item)))
                    .collect::<Vec<_>>()
                    .join(" ")
            },
        ),
        Some("secret") => "•".repeat(display(value).chars().count()),
        Some("choice") => format!("‹{}›", display(value)),
        Some("toggle") => if truthy(value) { "[x]" } else { "[ ]" }.into(),
        Some("time") => {
            if let Some(at) = instant(value) {
                format!("{:02}:{:02}", at.hour, at.minute)
            } else {
                let text = display(value);
                text.split_once('T').map_or(text.clone(), |(_, time)| {
                    time.trim_end_matches('Z').chars().take(5).collect()
                })
            }
        }
        Some("timestamp") => match instant(value) {
            Some(at) => format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                at.year, at.month, at.day, at.hour, at.minute
            ),
            None => display(value),
        },
        Some("date") => match instant(value) {
            Some(at) => format!("{:04}-{:02}-{:02}", at.year, at.month, at.day),
            None => display(value),
        },
        Some("percent") => value.as_f64().map_or_else(
            || display(value),
            |number| percent(number, &field.percent_scale()),
        ),
        Some("json") => ess_yaml::to_string(value)
            .map(|text| text.replace('\n', " ").trim().to_owned())
            .unwrap_or_default(),
        Some("audio" | "iframe") => format!("↗ {}", display(value)),
        Some("bytes") => value.as_f64().map_or_else(|| display(value), bytes),
        Some("duration") => value.as_f64().map_or_else(
            || display(value),
            |count| duration(count, &field.duration_unit()),
        ),
        _ => display(value),
    }
}

/// A duration counted in `unit` as `h:mm:ss`, in whole seconds, the hours unpadded and never
/// folded into days, as the React target shows it.
fn duration(count: f64, unit: &ess_ui::DurationUnit) -> String {
    if !count.is_finite() {
        return count.to_string();
    }
    let millis = match unit {
        ess_ui::DurationUnit::S => count * 1000.0,
        ess_ui::DurationUnit::M => count * 60_000.0,
        ess_ui::DurationUnit::H => count * 3_600_000.0,
        _ => count,
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // finite, floored, non-negative
    let seconds = (millis.abs() / 1000.0).floor() as u64;
    let sign = if millis <= -1000.0 { "-" } else { "" };
    format!(
        "{sign}{}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// A share on its `scale`, as a whole percent.
fn percent(number: f64, scale: &ess_ui::PercentScale) -> String {
    let shift = match scale {
        ess_ui::PercentScale::Hundredths => 0,
        _ => 2,
    };
    // At most one fraction digit, rounded half away from zero on the value's shortest decimal
    // form, as the React target's `Intl` shows it.
    let Some(tenths) = tenths_of(number, shift) else {
        return format!("{:.1}%", number * 100.0);
    };
    let sign = if number < 0.0 && tenths > 0 { "-" } else { "" };
    if tenths % 10 == 0 {
        format!("{sign}{}%", tenths / 10)
    } else {
        format!("{sign}{}.{}%", tenths / 10, tenths % 10)
    }
}

/// `number` times ten to the `shift`, in tenths of the magnitude, rounded half up on the shortest
/// decimal form of `number`. `None` when that overflows.
fn tenths_of(number: f64, shift: usize) -> Option<u128> {
    if !number.is_finite() {
        return None;
    }
    let text = format!("{}", number.abs());
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    // digits of the shifted value, with at least two places after the point
    let mut digits = format!("{whole}{fraction}");
    let point = whole.len() + shift;
    while digits.len() < point + 2 {
        digits.push('0');
    }
    let tenths: u128 = digits[..=point].parse().ok()?;
    let next = digits.as_bytes()[point + 1] - b'0';
    Some(tenths + u128::from(next >= 5))
}

/// An instant in UTC, to the minute.
struct Instant {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
}

/// The instant `value` holds: epoch milliseconds, or an ISO text (`2026-01-02T03:04:05+02:00`;
/// no offset reads as UTC), in UTC as the React target shows it. `None` for anything else.
fn instant(value: &Value) -> Option<Instant> {
    let minutes = if let Some(millis) = value.as_f64() {
        if !millis.is_finite() {
            return None;
        }
        #[allow(clippy::cast_possible_truncation)]
        // finite epoch milliseconds, floored to the minute
        let minutes = (millis / 60_000.0).floor() as i64;
        minutes
    } else {
        iso_minutes(&display(value))?
    };
    let days = minutes.div_euclid(1440);
    let of_day = minutes.rem_euclid(1440);
    // days to a civil date: the proleptic Gregorian calendar, March-based years.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * day_of_year + 2) / 153;
    let month = if shifted < 10 {
        shifted + 3
    } else {
        shifted - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    Some(Instant {
        year,
        month,
        day: day_of_year - (153 * shifted + 2) / 5 + 1,
        hour: of_day / 60,
        minute: of_day % 60,
    })
}

/// The epoch minutes of an ISO date and time text, offset applied.
fn iso_minutes(text: &str) -> Option<i64> {
    let (date, rest) = text.split_once(['T', ' ']).unwrap_or((text, "00:00"));
    let mut parts = date.splitn(3, '-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let (clock, offset) = match rest.find(['Z', 'z', '+', '-']) {
        Some(at) => (&rest[..at], &rest[at..]),
        None => (rest, ""),
    };
    let mut parts = clock.splitn(3, ':');
    let hour: i64 = parts.next()?.parse().ok()?;
    let minute: i64 = parts.next()?.parse().ok()?;
    if hour > 23 || minute > 59 {
        return None;
    }
    let offset_minutes = match offset.chars().next() {
        Some('+' | '-') => {
            let sign = if offset.starts_with('-') { -1 } else { 1 };
            let digits: String = offset[1..].chars().filter(char::is_ascii_digit).collect();
            if digits.len() != 4 {
                return None;
            }
            sign * (digits[..2].parse::<i64>().ok()? * 60 + digits[2..].parse::<i64>().ok()?)
        }
        _ => 0,
    };
    // a civil date to days: the inverse of the conversion in `instant`.
    let shifted_year = year - i64::from(month <= 2);
    let era = shifted_year.div_euclid(400);
    let year_of_era = shifted_year.rem_euclid(400);
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some(days * 1440 + hour * 60 + minute - offset_minutes)
}

/// A byte count scaled to B, KB, MB or GB in 1024 steps, with at most one decimal, as the React
/// target shows it.
fn bytes(count: f64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let tenths = |scaled: f64| (scaled * 10.0).round() / 10.0;
    let mut unit = 0;
    let mut scaled = count;
    while unit < UNITS.len() - 1 && scaled.abs() >= 1024.0 {
        scaled /= 1024.0;
        unit += 1;
    }
    // 1048575 B is 1023.999 KB, which rounds to the next unit.
    if unit < UNITS.len() - 1 && tenths(scaled).abs() >= 1024.0 {
        scaled /= 1024.0;
        unit += 1;
    }
    let text = format!("{:.1}", tenths(scaled));
    let text = text.strip_suffix(".0").unwrap_or(&text);
    format!("{text} {}", UNITS[unit])
}

/// The spark for a level between 0 and 7.
fn spark(level: f64) -> char {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0..=7
    let index = level.round().clamp(0.0, 7.0) as usize;
    SPARKS[index]
}

impl App {
    #[allow(clippy::too_many_lines)] // record, tabs, variants, fields by group, parts and submit
    fn form_lines(&self, form: &Form, place: &Place<'_>) -> Vec<Line<'static>> {
        let state = self.ui(place.ui);
        let draft = self.draft_path(place.ctx.section);
        let mut lines = Vec::new();
        if let Some(record) = &form.record {
            lines.extend(self.node_lines(record, place));
        }
        if !form.tabs.is_empty() {
            lines.push(tab_line(
                form.tabs
                    .iter()
                    .map(|tab| tab.label.clone().unwrap_or_else(|| tab.name.clone())),
                state.tab,
            ));
        }
        if let Some(variant) = &form.variant_by {
            lines.push(Line::styled(
                format!(
                    "variant by {}: {}",
                    variant.field,
                    variant.forms.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
                dim(),
            ));
        }
        let fields = form_fields(form, state.tab);
        let group_of = |field: &Field| {
            form.groups.iter().find(|group| {
                group
                    .fields
                    .iter()
                    .any(|candidate| candidate.name == field.name)
            })
        };
        let mut heading = None;
        // The cursor walks the fields drawn: those of a closed group are not among them.
        let mut drawn = 0;
        for field in &fields {
            if let Some(group) = group_of(field) {
                let collapsed = group.collapsed == Some(true);
                let open = self.is_open(&place.path.child("groups").child(&group.name), collapsed);
                if heading != Some(&group.name) {
                    let save = group.save.clone().unwrap_or_else(|| "with_form".into());
                    lines.push(Line::styled(
                        format!(
                            "{}{} (saves {save})",
                            fold_marker(collapsed, open),
                            group.label.clone().unwrap_or_else(|| group.name.clone())
                        ),
                        bold(),
                    ));
                    if open {
                        lines.extend(group.help.as_ref().and_then(|help| help_line("", help)));
                    }
                    heading = Some(&group.name);
                }
                if !open {
                    continue;
                }
            }
            let index = drawn;
            drawn += 1;
            let value = self.field_value(form, &draft, &field.field, &place.ctx);
            let shown = match field.field_as.as_deref() {
                Some("choice") => {
                    let choice = match field.choice.as_ref().map(|node| &node.body) {
                        Some(Body::Composite(Composite::Choice(choice))) => Some(choice),
                        _ => None,
                    };
                    let mut options = choice.map_or_else(Vec::new, |choice| {
                        self.choice_options(choice, Some(&field.field), &place.ctx)
                            .into_iter()
                            .map(|(_, label)| label)
                            .collect::<Vec<_>>()
                    });
                    // A multiple choice field being edited marks the option `space` adds or
                    // takes out (moved by `l` and `h`).
                    let multiple = crate::app::is_multiple_choice(field);
                    if multiple && place.focused && index == state.item && !options.is_empty() {
                        let at = state.option % options.len();
                        options[at] = format!("›{}‹", options[at]);
                    }
                    let shown = match &value {
                        Value::Sequence(items) if multiple => {
                            items.iter().map(display).collect::<Vec<_>>().join(", ")
                        }
                        other => display(other),
                    };
                    // Nothing chosen: the choice's placeholder.
                    let chosen = Some(shown)
                        .filter(|text| !text.is_empty())
                        .or_else(|| choice.and_then(|choice| choice.placeholder.clone()))
                        .unwrap_or_default();
                    format!("‹{chosen}› of {}", options.join("/"))
                }
                Some("file") => format!("path: {}", display(&value)),
                // A toggle nobody touched is unchecked, as the form sends it.
                Some("toggle") => if truthy(&value) { "[x]" } else { "[ ]" }.into(),
                // An input edits the number the field holds, as the React input does.
                Some("duration") => display(&value),
                _ => cell_value(field, &value),
            };
            let current = place.focused && index == state.item;
            let editing = current && state.editing;
            let cursor = if editing { "▏" } else { "" };
            // A field slot's input is the host's: its label, bordered, stands in its place.
            let (input, read) = match &field.host {
                Some(slot) => {
                    let label = format!("│{}│", host_label(slot));
                    (label.clone(), label)
                }
                None => (format!("[{shown}{cursor}]"), format!("[{shown}]")),
            };
            let text = format!("{} {input}", pad(&label_of(field), 16));
            // A section's or an overlay's own form records each field's line at the field's path,
            // with its whole label, so a test reads a control's state there.
            if place.record {
                self.mark(Mark {
                    path: form_field_path(place.path, form, field, state.tab),
                    row: None,
                    line: lines.len(),
                    height: 1,
                    x: 0,
                    width: None,
                    text: Some(format!("{} {read}", label_of(field))),
                });
            }
            let style = if current {
                reversed()
            } else {
                Style::default()
            };
            lines.push(Line::styled(text, style));
            lines.extend(field.help.as_ref().and_then(|help| help_line("  ", help)));
        }
        for part in &form.parts {
            let draft_ctx = Place {
                ctx: Ctx {
                    draft: Some(&draft),
                    ..place.ctx
                },
                record: false,
                ..*place
            };
            lines.extend(self.node_lines(part, &draft_ctx));
        }
        let label = form
            .submit
            .as_ref()
            .and_then(|submit| submit.label.clone())
            .unwrap_or_else(|| "Submit".into());
        lines.push(Line::styled(
            format!("[ {label} ] ctrl-s · enter edits · tab next field"),
            dim(),
        ));
        lines.extend(self.refusal_lines(place.ui, Refusing::Form));
        lines.extend(self.action_hint(&form.actions, &place.ctx, crate::app::FORM_KEYS));
        lines.extend(self.refusal_lines(place.ui, Refusing::Actions(&form.actions)));
        for group in &form.groups {
            lines.extend(self.action_hint(&group.actions, &place.ctx, crate::app::FORM_KEYS));
            lines.extend(self.refusal_lines(place.ui, Refusing::Actions(&group.actions)));
        }
        if let Some(result) = &form.result {
            lines.extend(self.node_lines(result, place));
        }
        lines
    }

    fn choice_spans(
        &self,
        choice: &ess_ui::Choice,
        value: &Value,
        place: &Place<'_>,
        cursor: Option<usize>,
    ) -> Vec<Span<'static>> {
        let chosen: Vec<String> = match value {
            Value::Sequence(items) => items.iter().map(display).collect(),
            Value::Null => Vec::new(),
            other => vec![display(other)],
        };
        let options = self.choice_options(choice, None, &place.ctx);
        let count = options.len().max(1);
        // Nothing chosen: the placeholder, before the options.
        let placeholder = choice
            .placeholder
            .as_ref()
            .filter(|_| chosen.iter().all(String::is_empty))
            .map(|placeholder| Span::styled(format!("‹{placeholder}› "), dim()));
        let options = options
            .into_iter()
            .enumerate()
            .map(|(index, (value, label))| {
                let on = chosen.contains(&display(&value));
                let mark = match (choice.multiple, on) {
                    (true, true) => "[x]",
                    (true, false) => "[ ]",
                    (false, true) => "(•)",
                    (false, false) => "( )",
                };
                let style = if cursor.is_some_and(|cursor| cursor % count == index) {
                    reversed()
                } else {
                    Style::default()
                };
                Span::styled(format!("{mark} {label} "), style)
            });
        placeholder.into_iter().chain(options).collect()
    }

    fn bar_lines(&self, bar: &ess_ui::FilterBar, place: &Place<'_>) -> Vec<Line<'static>> {
        let items = bar_items(bar);
        let state = self.ui(place.ui);
        let current = place
            .focused
            .then(|| {
                items
                    .get(state.item.min(items.len().saturating_sub(1)))
                    .copied()
            })
            .flatten();
        let style = |item: BarItem| {
            if current == Some(item) {
                reversed()
            } else {
                Style::default()
            }
        };
        let mut lines = Vec::new();
        if let Some(search) = &bar.search {
            let value = self
                .eval(&search.binds.0, &place.ctx)
                .map(|value| display(&value))
                .unwrap_or_default();
            let hint = search
                .placeholder
                .clone()
                .unwrap_or_else(|| "/ search".into());
            let shown = if value.is_empty() { hint } else { value };
            lines.push(Line::styled(
                format!("search [{shown}]"),
                style(BarItem::Search),
            ));
        }
        if let Some(window) = &bar.window {
            let value = self
                .eval(&window.binds.0, &place.ctx)
                .unwrap_or(Value::Null);
            let preset = display(&value["preset"]);
            let presets = self.window_presets(&window.binds.0, &place.ctx);
            lines.push(Line::styled(
                format!("window ‹{preset}› of {}", presets.join("/")),
                style(BarItem::Window),
            ));
        }
        for (index, node) in bar.choices.iter().enumerate() {
            let Body::Composite(Composite::Choice(choice)) = &node.body else {
                continue;
            };
            let binds = crate::app::choice_binds(bar, node, choice);
            let value = binds
                .as_ref()
                .and_then(|binds| self.eval(binds, &place.ctx))
                .unwrap_or(Value::Null);
            let focused = current == Some(BarItem::Choice(index));
            let mut spans = vec![Span::styled(
                format!("{}: ", node.common.name.clone().unwrap_or_default()),
                if focused { bold() } else { Style::default() },
            )];
            spans.extend(self.choice_spans(choice, &value, place, focused.then_some(state.option)));
            lines.push(Line::from(spans));
        }
        for (index, input) in bar.inputs.iter().enumerate() {
            let value = input
                .binds
                .as_ref()
                .and_then(|binds| self.eval(&binds.0, &place.ctx))
                .map(|value| display(&value))
                .unwrap_or_default();
            lines.push(Line::styled(
                format!("{} [{value}]", label_of(input)),
                style(BarItem::Input(index)),
            ));
        }
        let mut extra = self.action_hint(&bar.actions, &place.ctx, crate::app::BAR_KEYS);
        if bar.offers_reset() {
            let label = bar.reset_label().unwrap_or("reset all").to_owned();
            extra.push(Line::styled(label, dim()));
        }
        lines.extend(extra);
        lines
    }

    fn confirm_lines(&self, confirm: &ess_ui::Confirm, place: &Place<'_>) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if let Some(body) = &confirm.body {
            lines.push(Line::from(body.clone()));
        }
        for consequence in &confirm.consequences {
            lines.push(Line::from(format!("• {consequence}")));
        }
        if let Some(view) = &confirm.references {
            let request = crate::data::ReadRequest {
                view: view.clone(),
                fixture: None,
                params: self.overlay_params(),
            };
            lines.push(Line::styled(format!("Used by ({view}):"), bold()));
            match self.read_state(&request) {
                Some(ReadState::Ready(result)) if result.rows.is_empty() => {
                    lines.push(Line::styled("  nothing", dim()));
                }
                Some(ReadState::Ready(result)) => {
                    for row in &result.rows {
                        let label = row.get("label").map_or_else(|| display(row), display);
                        lines.push(Line::from(format!(
                            "  {label}  [{}]",
                            display(&row["type"])
                        )));
                    }
                }
                Some(ReadState::Failed(error)) => {
                    lines.push(Line::from(format!("  failed: {error}")));
                }
                _ => lines.push(Line::styled("  loading…", dim())),
            }
        }
        if let Some(input) = &confirm.input {
            lines.push(Line::from(format!(
                "{} [{}▏]",
                input.label,
                self.ui(place.ui).typed
            )));
        }
        let label = confirm
            .confirm_label
            .clone()
            .unwrap_or_else(|| "Confirm".into());
        let danger = if confirm.danger == Some(true) {
            " (destructive)"
        } else {
            ""
        };
        let yes = if confirm.input.is_some() {
            "enter"
        } else {
            "y"
        };
        let mut buttons = format!("[{yes}] {label}{danger}   [esc] Cancel");
        for (index, alternative) in confirm.alternatives.iter().enumerate() {
            let label = alternative.label.as_ref().unwrap_or(&alternative.name);
            let _ = write!(buttons, "   [{}] {label}", index + 1);
        }
        lines.extend(self.refusal_lines(place.ui, Refusing::Confirm));
        lines.push(Line::from(buttons));
        lines
    }

    fn metric_spans(&self, metric: &ess_ui::Metric, place: &Place<'_>) -> Vec<Span<'static>> {
        let mut stale = false;
        let mut value = Value::Null;
        if let Some(from) = &metric.from {
            value = self.eval(&from.0, &place.ctx).unwrap_or(Value::Null);
            let mut parts = from.0.split('.');
            if let (Some("channel"), Some(channel), Some(field)) =
                (parts.next(), parts.next(), parts.next())
            {
                stale = self.channel_status(channel) == "stale";
                if value.is_null() {
                    let key = self
                        .doc
                        .channels
                        .get(channel)
                        .and_then(|channel| channel.fields.get(field))
                        .map_or(field, String::as_str);
                    if let Some(row) = place.ctx.row {
                        value = row[key].clone();
                    }
                }
            }
        } else if let (Some(reads), Some(kind)) = (&metric.reads, metric.aggregate) {
            let request = self.request(reads, &place.ctx);
            if let Some(result) = self.reader_result(reads, &request) {
                value = aggregate(kind, metric.field.as_deref(), &result.rows);
            }
        } else if let Some(reads) = &metric.reads {
            let request = self.request(reads, &place.ctx);
            if let Some(row) = self
                .reader_result(reads, &request)
                .and_then(|result| result.rows.first())
            {
                value = metric
                    .match_field
                    .as_ref()
                    .map(|field| row[field.as_str()].clone())
                    .or_else(|| row.get("value").cloned())
                    .unwrap_or_else(|| {
                        row.as_mapping()
                            .and_then(|row| row.values().find(|value| value.is_number()).cloned())
                            .unwrap_or(Value::Null)
                    });
            }
        }
        let shown = match (&metric.format, value.as_f64()) {
            (_, None) if value.is_null() => "–".to_owned(),
            (Some(MetricFormat::Percent), Some(number)) => percent(
                number,
                &metric
                    .scale
                    .clone()
                    .unwrap_or(ess_ui::PercentScale::Fraction),
            ),
            (Some(MetricFormat::Duration), Some(number)) => duration(
                number,
                metric.unit.as_ref().unwrap_or(&ess_ui::DurationUnit::S),
            ),
            (Some(MetricFormat::Bytes), Some(number)) => bytes(number),
            _ => display(&value),
        };
        let label = metric.label.clone().unwrap_or_else(|| "metric".into());
        let mut spans = vec![Span::raw(format!("{label}: ")), Span::styled(shown, bold())];
        if let Some(window) = &metric.window {
            spans.push(Span::styled(format!(" ({window})"), dim()));
        }
        if stale {
            spans.push(Span::styled(" [stale]", reversed()));
        }
        spans
    }

    /// The edges of a graph whose edges have a read of their own, one line each, as
    /// `<from node label> → <to node label>`, after its node collection.
    fn graph_edge_lines(
        &self,
        editor: &ess_ui::GraphEditor,
        nodes: &ess_ui::Collection,
        place: &Place<'_>,
    ) -> Vec<Line<'static>> {
        let Some(edges) = editor.edges.as_ref() else {
            return Vec::new();
        };
        let Some(reads) = &edges.reads else {
            return Vec::new();
        };
        let Some(result) = self.read_rows(reads, &place.ctx) else {
            return vec![Line::styled("edges loading…", dim())];
        };
        let spec = editor.nodes.as_ref();
        let key = spec.and_then(|n| n.key.as_deref()).unwrap_or("id");
        let label = spec.and_then(|n| n.label.as_deref()).unwrap_or("label");
        let node_rows = nodes
            .reads
            .as_ref()
            .and_then(|reads| self.read_rows(reads, &place.ctx))
            .map(|result| result.rows.clone())
            .unwrap_or_default();
        let end = |row: &Value, field: &str| {
            let wanted = display(&row[field]);
            node_rows
                .iter()
                .find(|node| display(&node[key]) == wanted)
                .map(|node| display(&node[label]))
                .filter(|text| !text.is_empty())
                .unwrap_or(wanted)
        };
        let mut lines = vec![Line::styled("edges", dim())];
        for row in &result.rows {
            let mut text = format!("  {} → {}", end(row, &edges.from), end(row, &edges.to));
            if let Some(kind) = &edges.kind_by {
                let _ = write!(text, " [{}]", display(&row[kind.as_str()]));
            }
            lines.push(Line::from(text));
        }
        lines
    }

    #[allow(clippy::too_many_lines)] // sparkline, table and metric forms
    fn chart_lines(&self, chart: &ess_ui::Chart, place: &Place<'_>) -> Vec<Line<'static>> {
        let Some(result) = self.read_rows(&chart.reads, &place.ctx) else {
            return vec![Line::styled("loading…", dim())];
        };
        let kind = match &chart.chart {
            ChartKind::Fixed(kind) => kind.clone(),
            ChartKind::Chosen(chosen) => self
                .eval(&chosen.binds.0, &place.ctx)
                .map(|value| display(&value))
                .filter(|kind| !kind.is_empty())
                .or_else(|| chosen.options.first().cloned())
                .unwrap_or_else(|| "line".into()),
        };
        let fallback = self.plan.fallback(place.path, "no_charts");
        let series: Vec<String> = if chart.series.is_empty() {
            vec!["value".into()]
        } else {
            chart.series.clone()
        };
        let x = chart.x.clone().unwrap_or_else(|| "x".into());
        let as_table = matches!(kind.as_str(), "table" | "list") || fallback == Some("table");
        if fallback == Some("metric") || kind == "single_number" {
            let last = result
                .rows
                .last()
                .map_or(Value::Null, |row| row[series[0].as_str()].clone());
            return vec![Line::from(vec![
                Span::raw(format!("{}: ", series[0])),
                Span::styled(display(&last), bold()),
            ])];
        }
        if as_table {
            let mut fields = vec![x.clone()];
            fields.extend(series);
            let widths: Vec<usize> = fields
                .iter()
                .map(|field| {
                    result
                        .rows
                        .iter()
                        .map(|row| display(&row[field.as_str()]).width())
                        .chain(std::iter::once(field.width()))
                        .max()
                        .unwrap_or(0)
                })
                .collect();
            let mut lines = vec![Line::styled(
                fields
                    .iter()
                    .zip(&widths)
                    .map(|(field, width)| pad(field, *width))
                    .collect::<Vec<_>>()
                    .join("  "),
                bold(),
            )];
            for row in &result.rows {
                lines.push(Line::from(
                    fields
                        .iter()
                        .zip(&widths)
                        .map(|(field, width)| pad(&display(&row[field.as_str()]), *width))
                        .collect::<Vec<_>>()
                        .join("  "),
                ));
            }
            return lines;
        }
        let width = series.iter().map(|name| name.width()).max().unwrap_or(0);
        let mut lines = Vec::new();
        for name in &series {
            let values: Vec<f64> = result
                .rows
                .iter()
                .map(|row| row[name.as_str()].as_f64().unwrap_or(0.0))
                .collect();
            let max = values.iter().copied().fold(f64::MIN, f64::max);
            let min = values.iter().copied().fold(f64::MAX, f64::min).min(0.0);
            let spark: String = values
                .iter()
                .map(|value| {
                    let span = (max - min).max(f64::EPSILON);
                    spark((value - min) / span * 7.0)
                })
                .collect();
            lines.push(Line::from(vec![
                Span::raw(format!("{} ", pad(name, width))),
                Span::styled(spark, bold()),
                Span::styled(format!("  max {}", trim_float(max)), dim()),
            ]));
        }
        let first = result
            .rows
            .first()
            .map(|row| display(&row[x.as_str()]))
            .unwrap_or_default();
        let last = result
            .rows
            .last()
            .map(|row| display(&row[x.as_str()]))
            .unwrap_or_default();
        lines.push(Line::styled(
            format!("{kind} · {x}: {first} … {last}"),
            dim(),
        ));
        lines
    }

    fn primitive_spans(&self, primitive: &Primitive, ctx: &Ctx<'_>) -> Vec<Span<'static>> {
        let text_of = |text: Option<&ess_ui::Expr>, field: Option<&String>| {
            let value = match (text, field, ctx.row) {
                (Some(text), _, _) => self.eval(&text.0, ctx).unwrap_or(Value::Null),
                (None, Some(field), Some(row)) => row[field.as_str()].clone(),
                _ => Value::Null,
            };
            value
        };
        match primitive {
            Primitive::Text(text) => {
                let value = text_of(text.text.as_ref(), text.field.as_ref());
                let mut shown = display(&value);
                if text.format == Some(TextFormat::Currency) {
                    if let Some(currency) = text
                        .currency
                        .as_ref()
                        .and_then(|currency| self.eval(&currency.0, ctx))
                    {
                        shown = format!("{shown} {}", display(&currency));
                    }
                }
                let style = match text.style {
                    Some(TextStyle::Heading) => bold(),
                    Some(TextStyle::Caption) => dim(),
                    _ => Style::default(),
                };
                vec![Span::styled(shown, style)]
            }
            Primitive::Badge(badge) => {
                let value = text_of(badge.text.as_ref(), badge.field.as_ref());
                let shown = badge
                    .labels
                    .as_ref()
                    .and_then(|labels| labels.label(&value))
                    .map_or_else(|| display(&value), str::to_owned);
                vec![Span::styled(format!("[{shown}]"), bold())]
            }
            Primitive::Icon(icon) => {
                let glyph = match icon.icon.as_str() {
                    "home" => "⌂",
                    "typing" => "…",
                    "settings" => "⚙",
                    _ => "•",
                };
                vec![Span::raw(format!("{glyph} {}", icon.label))]
            }
            Primitive::Button(button) => {
                vec![Span::styled(format!("[ {} ]", button.label), bold())]
            }
            Primitive::Link(link) => {
                let text = self
                    .eval(&link.text.0, ctx)
                    .map(|value| display(&value))
                    .unwrap_or_default();
                vec![Span::raw(format!("{text} ↗"))]
            }
            Primitive::Input(input) => {
                let value = self
                    .eval(&input.binds.0, ctx)
                    .map(|value| display(&value))
                    .unwrap_or_default();
                let shown = if value.is_empty() {
                    input.placeholder.clone().unwrap_or_default()
                } else {
                    value
                };
                vec![Span::raw(format!("[{shown:<16}]"))]
            }
            Primitive::Toggle(toggle) => {
                let on = toggle
                    .binds
                    .as_ref()
                    .and_then(|binds| self.eval(&binds.0, ctx))
                    .is_some_and(|value| truthy(&value));
                vec![Span::raw(format!(
                    "{} {}",
                    if on { "[x]" } else { "[ ]" },
                    toggle.label
                ))]
            }
            Primitive::Image(image) => vec![Span::styled(format!("[image: {}]", image.alt), dim())],
            Primitive::Divider(divider) => {
                let vertical = divider.orientation.as_deref() == Some("vertical");
                vec![Span::styled(
                    if vertical { "│" } else { "────" }.to_owned(),
                    dim(),
                )]
            }
        }
    }
}

fn flatten(lines: Vec<Line<'static>>) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (index, line) in lines.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        spans.extend(line.spans);
    }
    spans
}

fn label_of(field: &Field) -> String {
    field.label.clone().unwrap_or_else(|| field.field.clone())
}

/// The canonical path of a field of `form` drawn at `at`: `fields/<f>`, `groups/<g>/fields/<f>`
/// or, for a field of the current tab, `tabs/<t>/fields/<f>`.
fn form_field_path(at: &NodePath, form: &Form, field: &Field, tab: usize) -> String {
    let is = |fields: &[Field]| {
        fields
            .iter()
            .any(|candidate| std::ptr::eq(candidate, field))
    };
    let owner = if is(&form.fields) {
        at.clone()
    } else if let Some(group) = form.groups.iter().find(|group| is(&group.fields)) {
        at.child("groups").child(&group.name)
    } else if let Some(current) = form.tabs.get(tab) {
        at.child("tabs").child(&current.name)
    } else {
        at.clone()
    };
    owner.child("fields").child(&field.name).to_string()
}

/// A `help: {text, link}` of a form group, field or column as one dim line after `lead`: its
/// text, then its link; `None` when it holds neither.
fn help_line(lead: &str, help: &ess_ui::Help) -> Option<Line<'static>> {
    let text = [help.text.clone(), help.link.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" — ");
    (!text.is_empty()).then(|| Line::styled(format!("{lead}? {text}"), dim()))
}

/// The heading a column shows in the generated React project: its `label`, else its name with
/// `_` read as a space.
fn browser_field_label(field: &Field) -> String {
    field
        .label
        .clone()
        .unwrap_or_else(|| field.name.replace('_', " "))
}

/// A record's line for `field`: its label, then its value in the emphasis of its tone.
fn labelled(field: &Field, row: &Value) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{} ", pad(&label_of(field), 16)), dim()),
        Span::styled(cell(field, row), cell_style(field, row)),
    ])
}

/// The actions a record offers on the tab shown: its own, then the tab's `form` when that is an
/// action. Drawing and key handling read the same list, so a hint's key runs its action.
pub(crate) fn record_actions(record: &ess_ui::Record, tab: usize) -> Vec<ess_ui::Action> {
    let mut actions = record.actions.clone();
    if let Some(ess_ui::TabForm::Action(action)) =
        record.tabs.get(tab).and_then(|tab| tab.form.as_ref())
    {
        actions.push((**action).clone());
    }
    actions
}

impl App {
    /// Records each tab of a section's or an overlay's own record at `<path>/tabs/<name>`, on
    /// the tab line drawn at `line` ([`tab_line`]).
    /// Records a record's field slot on its line at `fields`/`<name>`, with its label and the
    /// slot's label, when the record is a section's or an overlay's own body.
    fn mark_field_slot(&self, field: &Field, place: &Place<'_>, fields: &NodePath, line: usize) {
        let Some(slot) = &field.host else { return };
        if !place.record {
            return;
        }
        self.mark(Mark {
            path: fields.child(&field.name).to_string(),
            row: None,
            line,
            height: 1,
            x: 0,
            width: None,
            text: Some(format!("{} │{}│", label_of(field), host_label(slot))),
        });
    }

    fn mark_tabs(&self, tabs: &[ess_ui::Tab], place: &Place<'_>, line: usize) {
        if !place.record {
            return;
        }
        let mut x = 0;
        for tab in tabs {
            let label = tab.label.clone().unwrap_or_else(|| tab.name.clone());
            let width = label.width() + 2;
            self.mark(Mark {
                path: format!("{}/tabs/{}", place.path, tab.name),
                row: None,
                line,
                height: 1,
                x,
                width: Some(width),
                text: Some(label),
            });
            x += width + 1;
        }
    }
}

fn tab_line(labels: impl Iterator<Item = String>, current: usize) -> Line<'static> {
    let mut spans = Vec::new();
    for (index, label) in labels.enumerate() {
        let style = if index == current {
            reversed()
        } else {
            Style::default()
        };
        spans.push(Span::styled(format!(" {label} "), style));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled("[ ] switch tabs", dim()));
    Line::from(spans)
}

/// The rows a page draws as a range of all of them: `11-20 of 34`, `0 of 0` with none, as
/// React's `rangeLabel`. `page` counts from 1. A page that would reach past the total (a server
/// answering more rows than its total says) has no range.
fn page_range(page: u64, size: u64, rows: usize, total: u64) -> Option<String> {
    if rows == 0 || total == 0 {
        return Some(format!("0 of {total}"));
    }
    let from = page
        .saturating_sub(1)
        .saturating_mul(size)
        .saturating_add(1);
    let to = from.saturating_add(rows as u64 - 1);
    (to <= total).then(|| format!("{from}-{to} of {total}"))
}

/// A server pager's range: the page and size the read sent in its `page` and `size` params, of
/// the total the server answered. None without a `size` param holding a number of at least 1 or
/// without a total, as React draws none; a page param that is not a number of at least 1 is
/// page 1.
fn server_range(
    pager: &ess_ui::Pager,
    params: &std::collections::BTreeMap<String, Value>,
    total: Option<u64>,
    rows: usize,
) -> Option<String> {
    let number = |name: &Option<String>| {
        let number = match params.get(name.as_ref()?)? {
            Value::Number(number) => number.as_u64(),
            Value::String(text) => text.trim().parse::<u64>().ok(),
            _ => None,
        }?;
        (number >= 1).then_some(number)
    };
    let size = number(&pager.size)?;
    let page = number(&pager.page).unwrap_or(1);
    page_range(page, size, rows, total?)
}

/// The fewest cells a table column is cut to while its table's line does not fit the box.
const COLUMN_CAP: usize = 28;

/// Each table column's width in a line of `room` cells, columns two cells apart, from each
/// column's widest cell (`natural`): every column whole while they all fit. Otherwise the widest
/// columns give up cells, the narrower ones keeping theirs, but none is cut below
/// [`COLUMN_CAP`]; a line that still does not fit is cut at the box.
fn column_widths(natural: &[usize], room: usize) -> Vec<usize> {
    let room = room.saturating_sub(2 * natural.len().saturating_sub(1));
    let total = |cap: usize| natural.iter().map(|width| (*width).min(cap)).sum::<usize>();
    if total(usize::MAX) <= room {
        return natural.to_vec();
    }
    // The widest cap the line fits: every column at most that wide.
    let mut cap = COLUMN_CAP;
    if total(COLUMN_CAP) <= room {
        let mut over = natural.iter().copied().max().unwrap_or(0);
        while cap < over {
            let mid = cap + (over - cap).div_ceil(2);
            if total(mid) <= room {
                cap = mid;
            } else {
                over = mid - 1;
            }
        }
    }
    let mut widths: Vec<usize> = natural.iter().map(|width| (*width).min(cap)).collect();
    // The cells left over go one each to the columns still cut, left to right.
    let mut spare = room.saturating_sub(widths.iter().sum());
    for (width, whole) in widths.iter_mut().zip(natural) {
        if spare == 0 {
            break;
        }
        if *width < *whole {
            *width += 1;
            spare -= 1;
        }
    }
    widths
}

/// `text` cut to `width` terminal cells and padded to exactly that many.
fn pad(text: &str, width: usize) -> String {
    let text = truncate(text, width);
    let cells = text.width();
    format!("{text}{}", " ".repeat(width.saturating_sub(cells)))
}

/// `text` cut to at most `width` terminal cells, ending in `…` when cut.
fn truncate(text: &str, width: usize) -> String {
    if text.width() <= width {
        return text.to_owned();
    }
    let mut cut = String::new();
    let mut cells = 0;
    for letter in text.chars() {
        let next = letter.width().unwrap_or(0);
        if cells + next + 1 > width {
            break;
        }
        cut.push(letter);
        cells += next;
    }
    if width > 0 {
        cut.push('…');
    }
    cut
}

fn trim_float(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}
