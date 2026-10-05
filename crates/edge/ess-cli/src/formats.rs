//! `ess specify formats`: the `format: ess/N` headers this build admits, and what each added
//! (beyond10x/ess#460).
//!
//! Everything printed is [`FORMAT_HISTORY`], the catalogue `ess-domain` holds beside
//! `SUPPORTED_FORMATS` and holds to it at compile time, so this command cannot list a format the
//! build refuses or leave out one it admits.

use std::fmt::Write as _;
use std::process::ExitCode;

use anyhow::Result;
use ess_domain::system::{FormatHistoryEntry, FormatVersion, FORMAT_HISTORY};

use crate::Format;

/// `ess specify formats`.
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// List only the formats after this one.
    #[arg(long, value_name = "ess/N", value_parser = parse_since)]
    since: Option<FormatVersion>,
    /// Output rendering.
    #[arg(long, value_enum, default_value_t = Format::Text)]
    format: Format,
}

/// One format as `--format json` and `--format yaml` print it.
#[derive(Debug, serde::Serialize)]
struct Row {
    format: String,
    release: Option<&'static str>,
    newest: bool,
    added: &'static [&'static str],
    stricter: &'static [&'static str],
}

fn parse_since(value: &str) -> Result<FormatVersion, String> {
    FormatVersion::parse(value).map_err(|error| error.to_string())
}

/// The rows to print: every format after `since`, oldest first.
fn rows(history: &'static [FormatHistoryEntry], since: Option<FormatVersion>) -> Vec<Row> {
    let newest = history.iter().map(|entry| entry.major).max();
    history
        .iter()
        .filter(|entry| since.is_none_or(|since| entry.major > since.major()))
        .map(|entry| Row {
            format: format!("{}{}", FormatVersion::PREFIX, entry.major),
            release: entry.release,
            newest: Some(entry.major) == newest,
            added: entry.added,
            stricter: entry.stricter,
        })
        .collect()
}

/// One unindented line per format, then what it added and what reads differently, indented.
fn text(rows: &[Row]) -> String {
    let width = rows.iter().map(|row| row.format.len()).max().unwrap_or(0);
    let mut out = String::new();
    for row in rows {
        let release = row.release.unwrap_or("unreleased");
        if row.newest {
            let _ = writeln!(out, "{:<width$}  {release:<10}  newest", row.format);
        } else {
            let _ = writeln!(out, "{:<width$}  {release}", row.format);
        }
        for item in row.added {
            let _ = writeln!(out, "  added: {item}");
        }
        for item in row.stricter {
            let _ = writeln!(out, "  stricter: {item}");
        }
    }
    out
}

/// Prints the formats this build implements.
pub(crate) fn run(args: &Args) -> Result<ExitCode> {
    let rows = rows(FORMAT_HISTORY, args.since);
    match args.format {
        Format::Text => print!("{}", text(&rows)),
        Format::Yaml | Format::Json => crate::render(&rows, args.format)?,
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn since_keeps_only_later_formats_and_newest_is_the_highest_overall() {
        let all = rows(FORMAT_HISTORY, None);
        let last = FORMAT_HISTORY.last().expect("a format").major;
        assert!(all
            .iter()
            .all(|row| row.newest == (row.format == format!("ess/{last}"))));
        let later = rows(
            FORMAT_HISTORY,
            Some(FormatVersion::new(last).expect("major")),
        );
        assert_eq!(later.len(), 0, "nothing is after the newest");
        let one = rows(
            FORMAT_HISTORY,
            Some(FormatVersion::new(last - 1).expect("major")),
        );
        assert_eq!(one.len(), 1);
        assert!(one[0].newest);
    }

    #[test]
    fn text_marks_the_newest_and_indents_what_each_added() {
        let printed = text(&rows(FORMAT_HISTORY, None));
        let headers: Vec<&str> = printed
            .lines()
            .filter(|line| !line.starts_with(' '))
            .collect();
        assert_eq!(headers.len(), FORMAT_HISTORY.len());
        assert!(headers[0].starts_with("ess/1 "), "{}", headers[0]);
        assert!(printed
            .lines()
            .skip(1)
            .take_while(|line| line.starts_with(' '))
            .all(|line| line.starts_with("  added: ") || line.starts_with("  stricter: ")));
    }
}
