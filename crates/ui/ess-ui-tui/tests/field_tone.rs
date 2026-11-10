//! A field takes a tone in the terminal (story:a-badge-field-takes-a-tone): a column and a record
//! field are drawn in the emphasis of the tone of their value (`neutral` and `success` bold,
//! `info` italic, `warning` underlined, `danger` bold and underlined), a band map tones a number,
//! and a field with no `tone` and no `tone_by` is drawn as it was.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Modifier;
use ratatui::Terminal;

const DOC: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {views: {t.Runs: data.yaml, t.ById: data.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: runs
  sections: [{name: all, pages: [runs, run]}]
tone_maps:
  run_state: {failed: danger, done: success}
  load: {bands: [{from: 0, tone: success}, {from: 50, tone: warning}, {from: 80, tone: danger}]}
pages:
  runs:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: {view: t.Runs}
        columns:
          - {field: title}
          - {field: state, as: badge, tone: info, tone_by: run_state}
          - {field: load, tone_by: load}
  run:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        component: record
        reads: {view: t.ById}
        fields:
          - {field: state, as: badge, tone_by: run_state}
          - {field: title}
";

const DATA: &str = r"
views:
  t.Runs:
    rows:
      - {id: r-1, title: Alpha, state: failed, load: 90}
      - {id: r-2, title: Bravo, state: queued, load: 60}
      - {id: r-3, title: Delta, state: done, load: 10}
  t.ById:
    rows:
      - {id: r-1, title: Alpha, state: failed, load: 90}
";

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-field-tone")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("data.yaml"), DATA).expect("the fixture is written");
    dir
}

fn drawn(test: &str, page: &str) -> Buffer {
    let base = dir(test);
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page(page, &[]);
    let mut terminal = Terminal::new(TestBackend::new(140, 40)).expect("a terminal");
    terminal.draw(|frame| app.draw(frame)).expect("a frame");
    terminal.backend().buffer().clone()
}

/// The modifier of the first cell of `text` on the screen.
fn modifier(buffer: &Buffer, text: &str) -> Modifier {
    let area = buffer.area;
    for y in 0..area.height {
        let line: Vec<&str> = (0..area.width).map(|x| buffer[(x, y)].symbol()).collect();
        let joined = line.concat();
        if let Some(byte) = joined.find(text) {
            let column = joined[..byte].chars().count();
            return buffer[(u16::try_from(column).expect("a column"), y)].modifier;
        }
    }
    panic!("`{text}` is not on the screen");
}

#[test]
fn a_column_is_drawn_in_the_emphasis_of_its_tone() {
    let buffer = drawn("column", "runs");
    let bold_underlined = Modifier::BOLD | Modifier::UNDERLINED;
    assert!(
        modifier(&buffer, "[failed]").contains(bold_underlined),
        "danger is bold and underlined"
    );
    assert!(
        modifier(&buffer, "[queued]").contains(Modifier::ITALIC),
        "a value the map does not name takes the field's tone, info: italic"
    );
    assert!(
        modifier(&buffer, "[done]").contains(Modifier::BOLD),
        "success is bold"
    );
}

#[test]
fn a_band_map_tones_a_number() {
    let buffer = drawn("bands", "runs");
    assert!(
        modifier(&buffer, "90").contains(Modifier::UNDERLINED),
        "danger"
    );
    assert!(
        modifier(&buffer, "60").contains(Modifier::UNDERLINED),
        "warning"
    );
    assert!(
        !modifier(&buffer, "60").contains(Modifier::BOLD),
        "warning is underlined, not bold"
    );
    assert!(modifier(&buffer, "10").contains(Modifier::BOLD), "success");
}

#[test]
fn a_field_without_a_tone_is_drawn_as_it_was() {
    let buffer = drawn("plain", "runs");
    for title in ["Alpha", "Bravo", "Delta"] {
        let drawn = modifier(&buffer, title);
        assert!(
            !drawn.intersects(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED),
            "{title} carries no emphasis"
        );
    }
}

#[test]
fn a_record_field_is_drawn_in_the_emphasis_of_its_tone() {
    let buffer = drawn("record", "run");
    assert!(
        modifier(&buffer, "[failed]").contains(Modifier::BOLD | Modifier::UNDERLINED),
        "danger"
    );
}
