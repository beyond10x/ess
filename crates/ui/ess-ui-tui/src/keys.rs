//! Key sequences written as text, for tests and scripted runs.
//!
//! Every character is one key press; a name in angle brackets is a special key: `<enter>`,
//! `<esc>`, `<tab>`, `<backtab>`, `<bs>`, `<space>`, `<up>`, `<down>`, `<left>`, `<right>`,
//! `<lt>` (a literal `<`), and `<c-x>` for control plus a letter.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Parses a key sequence. An unknown `<name>` is typed as its characters.
pub fn parse(spec: &str) -> Vec<KeyEvent> {
    let mut keys = Vec::new();
    let mut rest = spec;
    while let Some(first) = rest.chars().next() {
        if first == '<' {
            if let Some(end) = rest.find('>') {
                let name = &rest[1..end];
                if let Some(key) = named(name) {
                    keys.push(key);
                    rest = &rest[end + 1..];
                    continue;
                }
            }
        }
        keys.push(KeyEvent::new(KeyCode::Char(first), KeyModifiers::NONE));
        rest = &rest[first.len_utf8()..];
    }
    keys
}

fn named(name: &str) -> Option<KeyEvent> {
    let plain = |code| Some(KeyEvent::new(code, KeyModifiers::NONE));
    match name {
        "enter" => plain(KeyCode::Enter),
        "esc" => plain(KeyCode::Esc),
        "tab" => plain(KeyCode::Tab),
        "backtab" => Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
        "bs" => plain(KeyCode::Backspace),
        "space" => plain(KeyCode::Char(' ')),
        "up" => plain(KeyCode::Up),
        "down" => plain(KeyCode::Down),
        "left" => plain(KeyCode::Left),
        "right" => plain(KeyCode::Right),
        "lt" => plain(KeyCode::Char('<')),
        _ => {
            let letter = name.strip_prefix("c-")?;
            let mut chars = letter.chars();
            let (Some(letter), None) = (chars.next(), chars.next()) else {
                return None;
            };
            Some(KeyEvent::new(KeyCode::Char(letter), KeyModifiers::CONTROL))
        }
    }
}
