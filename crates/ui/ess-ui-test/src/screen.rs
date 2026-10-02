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

/// Wide enough that rows are not cut, tall enough that every section is drawn.
const WIDTH: u16 = 160;
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
    /// Draws `app`.
    pub fn capture(app: &App) -> Self {
        let mut terminal = Terminal::new(TestBackend::new(WIDTH, HEIGHT)).expect("a test terminal");
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
