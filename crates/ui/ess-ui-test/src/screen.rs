//! The terminal screen as the reader sees it: text, the cursor line, each section's box, and
//! where the terminal placed each node ([`ess_ui_tui::Region`]).
//!
//! The app is drawn into ratatui's test backend, the same way `ess-ui-tui`'s own headless tests
//! see it. Reversed cells mark the focused row; a section is the bordered block titled with its
//! name (`┌ <name> `).

use ratatui::backend::TestBackend;
use ratatui::style::Modifier;
use ratatui::Terminal;

use ess_ui_tui::{App, Region};

/// Wide enough that most rows are not cut, tall enough that every section is drawn; a screen
/// grows past `WIDTH`, up to `MAX_WIDTH`, until no row or column is cut ([`Screen::capture`]).
const WIDTH: u16 = 160;
const MAX_WIDTH: u16 = 2000;
const HEIGHT: u16 = 240;

#[derive(Debug, Clone)]
struct Cell {
    symbol: String,
    reversed: bool,
}

/// One drawn frame.
#[derive(Debug, Clone)]
pub(crate) struct Screen {
    lines: Vec<Vec<Cell>>,
    /// Where the frame drew each node the terminal places ([`App::regions`]).
    regions: Vec<Region>,
}

/// One line inside a box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoxLine {
    /// Its text, trailing blanks trimmed.
    pub text: String,
}

/// A section's box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SectionBox {
    /// The top border, with the title and any `[stale]` badge.
    pub title: String,
    /// The lines inside.
    pub lines: Vec<BoxLine>,
}

impl SectionBox {
    /// The title and every line, as text.
    pub fn text(&self) -> String {
        let mut text = self.title.clone();
        for line in &self.lines {
            text.push('\n');
            text.push_str(&line.text);
        }
        text
    }
}

impl Screen {
    /// Draws `app`, on a screen wide enough for every row and column the frame cuts at
    /// [`WIDTH`]: a table wider than the screen is drawn whole, as the browser draws it in a
    /// scrolling frame, so its last column has cells of its own (beyond10x/ess#722 item 19).
    pub fn capture(app: &App) -> Self {
        let mut width = WIDTH;
        let mut last = u16::MAX;
        loop {
            let screen = Self::capture_at(app, width);
            let wanted = screen.cut();
            // A text the frame shortens whatever its width (a label it draws shorter) stops it.
            if wanted == 0 || wanted >= last || width >= MAX_WIDTH {
                return screen;
            }
            last = wanted;
            width = width.saturating_add(wanted).min(MAX_WIDTH);
        }
    }

    /// The cells the widest cut row or column lacks: a row's whole text (after its two-cell
    /// selection mark) or a column's label or cell beyond the cells it was drawn on.
    fn cut(&self) -> u16 {
        self.regions
            .iter()
            .filter_map(|region| {
                let text = region.text.as_deref()?;
                let row = region
                    .row
                    .as_ref()
                    .is_some_and(|key| region.path.ends_with(&format!("/rows/{key}")));
                let column = region.path.contains("/columns/");
                if !row && !column {
                    return None;
                }
                let needed = ratatui::text::Line::from(text).width() + if row { 2 } else { 0 };
                let needed = u16::try_from(needed).unwrap_or(u16::MAX);
                needed
                    .checked_sub(region.area.width)
                    .filter(|lack| *lack > 0)
            })
            .max()
            .unwrap_or(0)
    }

    fn capture_at(app: &App, width: u16) -> Self {
        let mut terminal = Terminal::new(TestBackend::new(width, HEIGHT)).expect("a test terminal");
        terminal
            .draw(|frame| app.draw(frame))
            .expect("a test terminal draws");
        let buffer = terminal.backend().buffer();
        let lines = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| {
                        let cell = &buffer[(x, y)];
                        Cell {
                            symbol: cell.symbol().to_owned(),
                            reversed: cell.modifier.contains(Modifier::REVERSED),
                        }
                    })
                    .collect()
            })
            .collect();
        Self {
            lines,
            regions: app.regions(),
        }
    }

    fn line_text(cells: &[Cell]) -> String {
        cells
            .iter()
            .map(|cell| cell.symbol.as_str())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }

    /// The whole screen as text.
    pub fn text(&self) -> String {
        let mut lines: Vec<String> = self
            .lines
            .iter()
            .map(|line| Self::line_text(line))
            .collect();
        while lines.last().is_some_and(String::is_empty) {
            lines.pop();
        }
        lines.join("\n")
    }

    /// Where `path` was drawn, when the terminal placed it on cells of its own.
    pub fn region(&self, path: &str) -> Option<&Region> {
        self.regions.iter().find(|region| region.path == path)
    }

    /// The key the terminal offers the action drawn at `path` under ([`Region::key`]), when it
    /// drew one there.
    pub fn key_at(&self, path: &str) -> Option<char> {
        self.regions
            .iter()
            .filter(|region| region.path == path)
            .find_map(|region| region.key)
    }

    /// Whether any node was placed at a path `wanted` accepts.
    pub fn region_where(&self, wanted: impl Fn(&str) -> bool) -> bool {
        self.regions.iter().any(|region| wanted(&region.path))
    }

    /// The keys of the rows drawn for `collection`, top to bottom, and whether each is drawn
    /// reversed (the cursor).
    pub fn rows(&self, collection: &str) -> Vec<(String, bool)> {
        let prefix = format!("{collection}/rows/");
        let mut rows: Vec<&Region> = self
            .regions
            .iter()
            .filter(|region| {
                region
                    .row
                    .as_ref()
                    .is_some_and(|key| region.path == format!("{prefix}{key}"))
            })
            .collect();
        rows.sort_by_key(|region| (region.area.y, region.area.x));
        rows.into_iter()
            .map(|region| {
                (
                    region.row.clone().unwrap_or_default(),
                    self.cells(region.area).any(|cell| cell.reversed),
                )
            })
            .collect()
    }

    /// Whether the node at `path` is drawn reversed (the tab or row shown as current), when it
    /// was drawn at all.
    pub fn reversed(&self, path: &str) -> Option<bool> {
        let region = self.region(path)?;
        Some(self.cells(region.area).any(|cell| cell.reversed))
    }

    fn cells(&self, area: ratatui::layout::Rect) -> impl Iterator<Item = &Cell> {
        let (x, width) = (usize::from(area.x), usize::from(area.width));
        self.lines
            .iter()
            .skip(usize::from(area.y))
            .take(usize::from(area.height))
            .flat_map(move |line| line.iter().skip(x).take(width))
            .filter(|cell| !cell.symbol.trim().is_empty())
    }

    /// The text drawn in `region`, one line per screen line, trailing blanks trimmed.
    pub fn text_in(&self, region: &Region) -> String {
        let (x, width) = (usize::from(region.area.x), usize::from(region.area.width));
        self.lines
            .iter()
            .skip(usize::from(region.area.y))
            .take(usize::from(region.area.height))
            .map(|line| {
                let end = (x + width).min(line.len());
                Self::line_text(&line[x.min(end)..end])
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The text of every line that holds a reversed cell: where the cursor is.
    pub fn cursor_lines(&self) -> Vec<String> {
        self.lines
            .iter()
            .filter(|line| line.iter().any(|cell| cell.reversed))
            .map(|line| Self::line_text(line))
            .collect()
    }

    /// The key hints on the last line.
    fn hints(&self) -> String {
        self.lines
            .last()
            .map(|line| Self::line_text(line))
            .unwrap_or_default()
    }

    /// Whether an overlay takes the screen.
    pub fn overlay_open(&self) -> bool {
        self.hints().contains("esc close")
    }

    /// The box of section `name`, when it is drawn.
    pub fn section(&self, name: &str) -> Option<SectionBox> {
        let marker: Vec<String> = format!("┌ {name} ").chars().map(String::from).collect();
        for (top, line) in self.lines.iter().enumerate() {
            let Some(left) = (0..line.len()).find(|&x| {
                marker.iter().enumerate().all(|(offset, symbol)| {
                    line.get(x + offset)
                        .is_some_and(|cell| cell.symbol == *symbol)
                })
            }) else {
                continue;
            };
            let right = (left + 1..line.len())
                .find(|&x| line[x].symbol == "┐")
                .unwrap_or(line.len());
            let bottom = (top + 1..self.lines.len())
                .find(|&y| {
                    self.lines[y]
                        .get(left)
                        .is_some_and(|cell| cell.symbol == "└")
                })
                .unwrap_or(self.lines.len());
            let title = Self::line_text(&line[left..=right.min(line.len() - 1)]);
            let lines = self.lines[top + 1..bottom]
                .iter()
                .map(|row| {
                    let inside = &row[(left + 1).min(row.len())..right.min(row.len())];
                    BoxLine {
                        text: Self::line_text(inside),
                    }
                })
                .collect();
            return Some(SectionBox { title, lines });
        }
        None
    }
}
