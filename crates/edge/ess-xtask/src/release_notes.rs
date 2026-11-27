//! The GitHub release body of each release, written by hand beside its changelog section and held
//! to hard limits here.
//!
//! A one-sentence summary of a 2,000-character changelog entry cannot be derived by a program, so
//! the body is written once per release as `release-notes/<version>.md`, in the
//! release-preparation commit, and `cargo xtask release notes <version>` prints that file once it
//! validates. The changelog section stays the full record; the body links to it.
//!
//! [`validate`] refuses, each refusal at its line with the limit and the measured value:
//!
//! - a shape other than: a lead, a blank line, `**Breaking**`, a blank line, a bullet list, a
//!   blank line, `**Changed**`, a blank line, a bullet list, a blank line, the link line and one
//!   final newline; any other heading, bold line, table or code fence;
//! - a body over [`BODY_CHARACTERS`] characters before its link line, or over
//!   [`BODY_CHARACTERS_WIDE`] when `**Breaking**` holds more than [`WIDE_AFTER`] bullets
//!   (Unicode scalar values, newlines included, as `head -n -1 | wc -m` counts);
//! - a lead of more than one line, more than one sentence (a `.`, `?` or `!` followed by a space
//!   and an upper-case letter or a backtick starts another), more than [`LEAD_WORDS`]
//!   whitespace-separated words, or a count of entries (`2 added`, `1 fixed`);
//! - a list of no bullet or more than [`LIST_BULLETS`]; a bullet continued on a second line, over
//!   [`BULLET_CHARACTERS`] characters after `- `, or ending with a period;
//! - `- None.` anywhere but as the single bullet of `**Breaking**`, and as that bullet when the
//!   section [`marks_breaking`] (the reverse is allowed: a section may state a breaking change in
//!   words without marking it);
//! - in a bullet, outside code spans: "so that", a parenthesis, a code point (`U+` and a hex
//!   digit), and a semicolon, except one in a `**Breaking**` bullet; in the lead and the bullets,
//!   outside code spans: the whole words "now" and "again" and the phrase "its own", any case;
//! - a word of a code span that does not occur in the version's section (see [`normalise`]);
//! - a last line other than the link to the section on the tag, at [`anchor`] of its heading.
//!
//! What a program cannot decide stays the author's duty: a bullet is one clause, with no second
//! clause after "and" and no list of three or more examples; it names no internals, rationale,
//! exit statuses or numeric limits.

use crate::whats_changed::ordinal;
use std::fmt;
use std::fs;
use std::path::Path;

/// Where the notes files live, one `<version>.md` per release, at the repository root.
///
/// Not under `changes/`: that directory is the organization's change feed, which the docs bundle
/// publishes (`changes/**/*.yaml`) and Atlas reads, so a body there would sit inside another
/// contract.
pub const DIRECTORY: &str = "release-notes";

/// The first release whose notes file `release verify` requires.
const REQUIRED_FROM: (u64, u64, u64) = (0, 79, 0);

/// The most characters of a body before its link line.
const BODY_CHARACTERS: usize = 900;

/// The most characters of a body before its link line when `**Breaking**` holds more than
/// [`WIDE_AFTER`] bullets.
const BODY_CHARACTERS_WIDE: usize = 1200;

/// The number of `**Breaking**` bullets past which [`BODY_CHARACTERS_WIDE`] applies.
const WIDE_AFTER: usize = 3;

/// The most whitespace-separated words of the lead, code spans' words included.
const LEAD_WORDS: usize = 25;

/// The most characters a bullet holds after its `- `.
const BULLET_CHARACTERS: usize = 110;

/// The most bullets a list holds.
const LIST_BULLETS: usize = 5;

const BREAKING: &str = "**Breaking**";
const CHANGED: &str = "**Changed**";

/// The single `**Breaking**` bullet of a release without a breaking change.
const NONE: &str = "None.";

/// The headings of a changelog section, which a lead that counts entries pairs with a number.
const COUNTED: &[&str] = &["added", "changed", "fixed", "removed"];

/// One thing wrong with a notes file, at a 1-based line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "line {}: {}", self.line, self.message)
    }
}

fn refusal(line: usize, message: impl Into<String>) -> Refusal {
    Refusal {
        line,
        message: message.into(),
    }
}

/// A release's dated `CHANGELOG.md` section: its heading line and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub heading: String,
    pub body: String,
}

/// The dated section of `version`, from its `## [<version>] — <date>` heading to the next release
/// heading. `release verify` and `release status` refuse a version without one.
pub fn section(changelog: &str, version: &str) -> Result<Section, String> {
    let prefix = format!("## [{version}] ");
    let mut lines = changelog.lines();
    let found = lines
        .by_ref()
        .find(|line| line.starts_with(&prefix))
        .ok_or_else(|| format!("CHANGELOG.md has no release heading for {version}"))?;
    if !found.contains('—') {
        return Err(format!("release heading for {version} has no date"));
    }
    let body = lines
        .take_while(|line| !line.starts_with("## ["))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned();
    if body.is_empty() {
        return Err(format!("release heading for {version} has no notes"));
    }
    Ok(Section {
        heading: found.to_owned(),
        body: format!("{body}\n"),
    })
}

/// The notes file of `version`, relative to the repository root.
pub fn path(version: &str) -> String {
    format!("{DIRECTORY}/{version}.md")
}

/// Whether `release verify` requires the notes file of `version`.
pub fn required(version: &str) -> bool {
    ordinal(version).is_some_and(|number| number >= REQUIRED_FROM)
}

/// The refusal for a release without its notes file: the path and the rule.
pub fn missing(version: &str) -> String {
    let path = path(version);
    format!(
        "{path} does not exist. The GitHub release body of {version} is that file, written by \
         hand in the release-preparation commit beside the CHANGELOG.md section of {version} \
         (AGENTS.md, \"The GitHub release body is `release-notes/<version>.md`\"); \
         `cargo xtask release notes {version}` validates and prints it"
    )
}

/// GitHub's anchor of a changelog heading line, as the heading spells it.
///
/// The `#`s and the space after them are not part of the title. Of the title: lowercase; every
/// character that is not a letter, a digit, a space, a hyphen or an underscore removed; each space
/// a hyphen. `## [0.97.0] — 2026-11-21` is `0970--2026-11-21`: the em dash goes and its two spaces
/// stay as two hyphens.
pub fn anchor(heading: &str) -> String {
    let mut out = String::new();
    for character in heading.trim_start_matches('#').trim().chars() {
        if character.is_alphanumeric() || character == '-' || character == '_' {
            out.extend(character.to_lowercase());
        } else if character == ' ' {
            out.push('-');
        }
    }
    out
}

/// Whether a changelog section marks a breaking change.
///
/// The predicate: a top-level entry, a line starting `- ` at column 0, whose text starts with
/// `**Breaking`. ESS spells it `**Breaking:**` and, from 0.79.0 to 0.97.0,
/// `**Breaking for <who>**:`; both start that way. A breaking change stated only in words is not
/// marked.
pub fn marks_breaking(section: &str) -> bool {
    section.lines().any(|line| {
        line.strip_prefix("- ")
            .is_some_and(|text| text.starts_with("**Breaking"))
    })
}

/// Text with every run of whitespace, line breaks and indentation included, as one space, trimmed.
///
/// The section is read this way, so a name the changelog wraps across two lines reads as it
/// renders. A code span's words are then looked up in it one by one: a span may join words the
/// section spells apart (`ess verify conform run --target process -- <program>`), and invents a
/// name only when one of its words is nowhere in the section.
fn normalise(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The code spans of `text` and the text outside them, each span replaced by `` `x` ``.
///
/// A span opens with a run of backticks and closes at the next run of the same length, as in
/// Markdown. `Err` when a run closes nothing.
fn code_spans(text: &str) -> Result<(Vec<String>, String), ()> {
    let characters: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    let mut prose = String::new();
    let mut at = 0;
    while at < characters.len() {
        if characters[at] != '`' {
            prose.push(characters[at]);
            at += 1;
            continue;
        }
        let open = at;
        while at < characters.len() && characters[at] == '`' {
            at += 1;
        }
        let width = at - open;
        let mut scan = at;
        let mut close = None;
        while scan < characters.len() {
            if characters[scan] == '`' {
                let start = scan;
                while scan < characters.len() && characters[scan] == '`' {
                    scan += 1;
                }
                if scan - start == width {
                    close = Some(start);
                    break;
                }
            } else {
                scan += 1;
            }
        }
        let close = close.ok_or(())?;
        spans.push(characters[at..close].iter().collect());
        prose.push_str("`x`");
        at = close + width;
    }
    Ok((spans, prose))
}

/// The sentences of the lead's prose: one, and one more at each `.`, `?` or `!` followed by a
/// space and an upper-case letter or a backtick.
fn sentences(prose: &str) -> usize {
    let characters: Vec<char> = prose.chars().collect();
    1 + characters
        .windows(3)
        .filter(|window| {
            matches!(window[0], '.' | '?' | '!')
                && window[1] == ' '
                && (window[2].is_uppercase() || window[2] == '`')
        })
        .count()
}

/// The words of prose: its runs of letters and digits, each with its byte range.
fn words(prose: &str) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut start = None;
    for (index, character) in prose.char_indices() {
        match (character.is_alphanumeric(), start) {
            (true, None) => start = Some(index),
            (false, Some(open)) => {
                found.push((open, index));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(open) = start {
        found.push((open, prose.len()));
    }
    found
}

/// The first `<n> added`, `<n> changed`, `<n> fixed` or `<n> removed` of prose, any case.
fn entry_count(prose: &str) -> Option<String> {
    words(prose).windows(2).find_map(|pair| {
        let (number, word) = (&prose[pair[0].0..pair[0].1], &prose[pair[1].0..pair[1].1]);
        let counted = number.chars().all(|character| character.is_ascii_digit())
            && COUNTED.contains(&word.to_lowercase().as_str());
        counted.then(|| format!("{number} {word}"))
    })
}

/// The words and two-word phrases of prose the rule refuses, as the prose spells them: `now`,
/// `again` and `its own` always, `so that` when `phrases` is set.
fn refused_words(prose: &str, phrases: bool) -> Vec<String> {
    let found = words(prose);
    let mut refused = Vec::new();
    for (index, &(start, end)) in found.iter().enumerate() {
        let word = prose[start..end].to_lowercase();
        if word == "now" || word == "again" {
            refused.push(prose[start..end].to_owned());
            continue;
        }
        let Some(&(next_start, next_end)) = found.get(index + 1) else {
            continue;
        };
        let gap = &prose[end..next_start];
        if gap.is_empty() || !gap.chars().all(char::is_whitespace) {
            continue;
        }
        let next = prose[next_start..next_end].to_lowercase();
        if (word == "its" && next == "own") || (phrases && word == "so" && next == "that") {
            refused.push(prose[start..next_end].to_owned());
        }
    }
    refused
}

/// Each code point of prose: `U+` and its run of hex digits.
fn code_points(prose: &str) -> Vec<String> {
    prose
        .match_indices("U+")
        .filter_map(|(index, _)| {
            let digits: String = prose[index + 2..]
                .chars()
                .take_while(char::is_ascii_hexdigit)
                .collect();
            (!digits.is_empty()).then(|| format!("U+{digits}"))
        })
        .collect()
}

/// The last line of the notes of `version`, whose section has `heading`.
fn link(version: &str, heading: &str) -> String {
    format!(
        "Full entry: [CHANGELOG.md, {version}](https://github.com/beyond10x/ess/blob/{version}/CHANGELOG.md#{})",
        anchor(heading)
    )
}

/// The lines of a construct the body does not hold, each refused by name.
fn forbidden(lines: &[&str]) -> Vec<Refusal> {
    let mut refusals = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let number = index + 1;
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            refusals.push(refusal(
                number,
                format!(
                    "`{trimmed}` is a heading; the body has no heading but the lines \
                     `{BREAKING}` and `{CHANGED}`"
                ),
            ));
        } else if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            refusals.push(refusal(number, "a code fence; the body holds none"));
        } else if trimmed.starts_with('|') {
            refusals.push(refusal(number, "a table row; the body holds no table"));
        } else if trimmed.len() > 4
            && trimmed.starts_with("**")
            && trimmed.ends_with("**")
            && trimmed != BREAKING
            && trimmed != CHANGED
        {
            refusals.push(refusal(
                number,
                format!(
                    "`{trimmed}` is a bold line; the only bold lines are `{BREAKING}` and \
                     `{CHANGED}`"
                ),
            ));
        }
    }
    refusals
}

/// A bullet list of the body: its first line's number and each bullet's number and text.
struct List<'a> {
    first: usize,
    bullets: Vec<(usize, &'a str)>,
}

/// Reads the shape of the body, refusing the first line out of place.
struct Shape<'a> {
    lines: &'a [&'a str],
    at: usize,
}

impl<'a> Shape<'a> {
    fn number(&self) -> usize {
        (self.at + 1).min(self.lines.len()).max(1)
    }

    fn found(&self) -> String {
        self.lines.get(self.at).map_or_else(
            || "the end of the file".to_owned(),
            |line| format!("`{line}`"),
        )
    }

    fn blank(&mut self, after: &str) -> Result<(), Refusal> {
        match self.lines.get(self.at) {
            Some(line) if line.trim().is_empty() => {
                self.at += 1;
                Ok(())
            }
            _ => Err(refusal(
                self.number(),
                format!(
                    "expected a blank line after {after}, found {}",
                    self.found()
                ),
            )),
        }
    }

    fn exact(&mut self, expected: &str) -> Result<(), Refusal> {
        if self.lines.get(self.at) == Some(&expected) {
            self.at += 1;
            Ok(())
        } else {
            Err(refusal(
                self.number(),
                format!("expected the line `{expected}`, found {}", self.found()),
            ))
        }
    }

    fn list(&mut self, heading: &str) -> Result<List<'a>, Refusal> {
        let first = self.at + 1;
        let mut bullets = Vec::new();
        while let Some(line) = self.lines.get(self.at) {
            if line.trim().is_empty() {
                break;
            }
            let Some(text) = line.strip_prefix("- ") else {
                let message = if bullets.is_empty() {
                    format!(
                        "the `{heading}` list has no bullet: expected a line starting `- `, \
                         found {}",
                        self.found()
                    )
                } else if *line == BREAKING || *line == CHANGED {
                    format!("a blank line must separate the `{heading}` list from `{line}`")
                } else {
                    format!(
                        "a bullet is one line starting `- `; {} continues the bullet above or is \
                         no bullet",
                        self.found()
                    )
                };
                return Err(refusal(self.number(), message));
            };
            bullets.push((self.at + 1, text));
            self.at += 1;
        }
        if bullets.is_empty() {
            return Err(refusal(
                self.number(),
                format!(
                    "the `{heading}` list has no bullet: expected a line starting `- `, found {}",
                    self.found()
                ),
            ));
        }
        Ok(List { first, bullets })
    }
}

/// The parts of a body whose shape is right.
struct Parts<'a> {
    lead: Vec<&'a str>,
    breaking: List<'a>,
    changed: List<'a>,
    /// The refusal of a last line that is not the link, when it is not.
    link: Option<Refusal>,
}

fn shape<'a>(lines: &'a [&'a str], expected_link: &str) -> Result<Parts<'a>, Refusal> {
    let lead_end = lines
        .iter()
        .position(|line| line.trim().is_empty())
        .unwrap_or(lines.len());
    let lead = lines[..lead_end].to_vec();
    let missing_lead = refusal(
        1,
        format!(
            "the lead is missing: the body opens with one sentence, then a blank line and \
             `{BREAKING}`"
        ),
    );
    match lead.first() {
        None => return Err(missing_lead),
        Some(first) if *first == BREAKING || *first == CHANGED || first.starts_with("- ") => {
            return Err(missing_lead)
        }
        Some(_) => {}
    }
    for (index, line) in lead.iter().enumerate().skip(1) {
        if *line == BREAKING || *line == CHANGED {
            return Err(refusal(
                index + 1,
                format!("a blank line must separate the lead from `{line}`"),
            ));
        }
        if line.starts_with("- ") {
            return Err(refusal(index + 1, "the lead is a sentence, not a list"));
        }
    }
    let mut cursor = Shape {
        lines,
        at: lead_end,
    };
    cursor.blank("the lead")?;
    cursor.exact(BREAKING)?;
    cursor.blank(&format!("`{BREAKING}`"))?;
    let breaking = cursor.list(BREAKING)?;
    cursor.blank(&format!("the `{BREAKING}` list"))?;
    cursor.exact(CHANGED)?;
    cursor.blank(&format!("`{CHANGED}`"))?;
    let changed = cursor.list(CHANGED)?;
    cursor.blank(&format!("the `{CHANGED}` list"))?;
    let link = match lines.get(cursor.at) {
        Some(line) if *line == expected_link => None,
        // A wrong link leaves every other part in place, so the rest is still checked.
        Some(line) => Some(refusal(
            cursor.at + 1,
            format!("the last line must be `{expected_link}`, found `{line}`"),
        )),
        None => {
            return Err(refusal(
                cursor.number(),
                format!("the body ends before its last line `{expected_link}`"),
            ))
        }
    };
    if cursor.at + 1 < lines.len() {
        return Err(refusal(
            cursor.at + 2,
            "nothing follows the last line, the link to the changelog section",
        ));
    }
    Ok(Parts {
        lead,
        breaking,
        changed,
        link,
    })
}

/// The refusals of one line of prose that holds code spans: an unclosed span, and each word of a
/// span the section does not spell. `Some` carries the prose outside the spans.
fn spans(
    line: usize,
    text: &str,
    names: &str,
    version: &str,
    refusals: &mut Vec<Refusal>,
) -> Option<String> {
    let Ok((found, prose)) = code_spans(text) else {
        refusals.push(refusal(
            line,
            "an unclosed code span: a run of backticks opens one that no run of the same length \
             closes",
        ));
        return None;
    };
    for span in found {
        for word in span.split_whitespace() {
            if !names.contains(word) {
                refusals.push(refusal(
                    line,
                    format!(
                        "`{word}` in a code span does not occur in the CHANGELOG.md section of \
                         {version}; every word of a backticked name is spelled as the section \
                         spells it"
                    ),
                ));
            }
        }
    }
    Some(prose)
}

fn words_refusal(line: usize, word: &str) -> Refusal {
    refusal(
        line,
        format!("`{word}`: the body never says \"now\", \"again\" or \"its own\""),
    )
}

/// The refusals of one bullet, in the list under `heading`.
fn bullet(
    heading: &str,
    line: usize,
    text: &str,
    sole_none: bool,
    names: &str,
    version: &str,
    refusals: &mut Vec<Refusal>,
) {
    let none_here = heading == BREAKING && sole_none;
    if text == NONE && !none_here {
        refusals.push(refusal(
            line,
            format!("`- {NONE}` is the single bullet of the `{BREAKING}` list and nothing else"),
        ));
    }
    let length = text.chars().count();
    if text.trim().is_empty() {
        refusals.push(refusal(line, "an empty bullet"));
    } else if length > BULLET_CHARACTERS {
        refusals.push(refusal(
            line,
            format!("a bullet holds {length} characters after `- `; at most {BULLET_CHARACTERS}"),
        ));
    }
    if text.trim_end().ends_with('.') && !none_here {
        refusals.push(refusal(
            line,
            format!("a bullet ends with a period; only `- {NONE}` does"),
        ));
    }
    let Some(prose) = spans(line, text, names, version, refusals) else {
        return;
    };
    for word in refused_words(&prose, true) {
        if word.to_lowercase().starts_with("so") {
            refusals.push(refusal(
                line,
                format!("`{word}` in a bullet: a bullet is one clause, without \"so that\""),
            ));
        } else {
            refusals.push(words_refusal(line, &word));
        }
    }
    for parenthesis in ['(', ')'] {
        if prose.contains(parenthesis) {
            refusals.push(refusal(
                line,
                format!("`{parenthesis}` in a bullet: a bullet holds no parentheses"),
            ));
        }
    }
    for point in code_points(&prose) {
        refusals.push(refusal(
            line,
            format!("`{point}` in a bullet: a bullet names no character code point"),
        ));
    }
    let semicolons = prose.matches(';').count();
    let allowed = usize::from(heading == BREAKING);
    if semicolons > allowed {
        let noun = if semicolons == 1 {
            "semicolon"
        } else {
            "semicolons"
        };
        refusals.push(refusal(
            line,
            format!(
                "a `{heading}` bullet holds {semicolons} {noun}; at most {allowed}, one in a \
                 `{BREAKING}` bullet before its action"
            ),
        ));
    }
}

/// The refusal of a body over its length before the link line, the last of `lines`: newlines
/// included, Unicode scalar values counted.
fn body_length(lines: &[&str], breaking_bullets: usize, refusals: &mut Vec<Refusal>) {
    let body: usize = lines[..lines.len() - 1]
        .iter()
        .map(|line| line.chars().count() + 1)
        .sum();
    let limit = if breaking_bullets > WIDE_AFTER {
        BODY_CHARACTERS_WIDE
    } else {
        BODY_CHARACTERS
    };
    if body > limit {
        refusals.push(refusal(
            1,
            format!(
                "the body holds {body} characters before its last line; at most {limit} with \
                 {breaking_bullets} `{BREAKING}` bullets ({BODY_CHARACTERS_WIDE} only with more \
                 than {WIDE_AFTER})"
            ),
        ));
    }
}

/// The refusals of the lead, its lines as the body spells them.
fn lead(lines: &[&str], names: &str, version: &str, refusals: &mut Vec<Refusal>) {
    if lines.len() > 1 {
        refusals.push(refusal(
            2,
            format!("the lead runs over {} lines; at most 1", lines.len()),
        ));
    }
    let lead = lines.join(" ");
    let lead_words = lead.split_whitespace().count();
    if lead_words > LEAD_WORDS {
        refusals.push(refusal(
            1,
            format!("the lead holds {lead_words} words; at most {LEAD_WORDS}"),
        ));
    }
    let Some(prose) = spans(1, &lead, names, version, refusals) else {
        return;
    };
    let count = sentences(&prose);
    if count > 1 {
        refusals.push(refusal(
            1,
            format!("the lead holds {count} sentences; exactly 1"),
        ));
    }
    if let Some(counted) = entry_count(&prose) {
        refusals.push(refusal(
            1,
            format!(
                "the lead counts entries (`{counted}`); it says what a user can do, not how many \
                 entries changed"
            ),
        ));
    }
    for word in refused_words(&prose, false) {
        refusals.push(words_refusal(1, &word));
    }
}

/// Validates the notes of `version` against its section in `changelog`; every refusal, by line.
pub fn validate(changelog: &str, version: &str, notes: &str) -> Result<(), Vec<Refusal>> {
    let section = section(changelog, version).map_err(|message| vec![refusal(1, message)])?;
    let (lines, final_newline): (Vec<&str>, bool) = match notes.strip_suffix('\n') {
        Some(body) => (body.split('\n').collect(), true),
        None => (notes.split('\n').collect(), false),
    };
    let mut refusals = forbidden(&lines);
    if !refusals.is_empty() {
        return Err(refusals);
    }
    if !final_newline {
        refusals.push(refusal(
            lines.len(),
            "the file does not end with a newline after its last line",
        ));
    }
    let parts = match shape(&lines, &link(version, &section.heading)) {
        Ok(parts) => parts,
        Err(found) => {
            refusals.push(found);
            refusals.sort_by_key(|found| found.line);
            return Err(refusals);
        }
    };
    refusals.extend(parts.link.clone());
    let names = normalise(&section.body);
    body_length(&lines, parts.breaking.bullets.len(), &mut refusals);
    lead(&parts.lead, &names, version, &mut refusals);
    let sole_none = matches!(parts.breaking.bullets.as_slice(), [(_, text)] if *text == NONE);
    if sole_none && marks_breaking(&section.body) {
        refusals.push(refusal(
            parts.breaking.first,
            format!(
                "the CHANGELOG.md section of {version} marks a breaking change (an entry starting \
                 `**Breaking`), so the `{BREAKING}` list names it instead of `- {NONE}`"
            ),
        ));
    }
    for (heading, list) in [(BREAKING, &parts.breaking), (CHANGED, &parts.changed)] {
        if list.bullets.len() > LIST_BULLETS {
            refusals.push(refusal(
                list.first,
                format!(
                    "the `{heading}` list holds {} bullets; at most {LIST_BULLETS}",
                    list.bullets.len()
                ),
            ));
        }
        for &(line, text) in &list.bullets {
            bullet(
                heading,
                line,
                text,
                sole_none,
                &names,
                version,
                &mut refusals,
            );
        }
    }
    if refusals.is_empty() {
        return Ok(());
    }
    refusals.sort_by_key(|found| found.line);
    Err(refusals)
}

/// Each refusal of the notes of `version` as `release-notes/<version>.md:<line>: <message>`.
pub fn refusal_report(version: &str, refusals: &[Refusal]) -> String {
    let path = path(version);
    refusals
        .iter()
        .map(|found| format!("{path}:{}: {}", found.line, found.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `cargo xtask release notes <version>`: the notes file of `version`, once it validates.
pub fn print(root: &Path, changelog: &str, version: &str) -> Result<String, String> {
    let notes = match fs::read_to_string(root.join(path(version))) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Err(missing(version)),
        Err(error) => return Err(format!("read {}: {error}", path(version))),
    };
    validate(changelog, version, &notes).map_err(|refusals| refusal_report(version, &refusals))?;
    Ok(notes)
}

/// `cargo xtask release notes --check`: every notes file present, each against its own section.
///
/// An absent or empty directory passes. Anything in it but a `<version>.md` file is refused, so a
/// draft or a misnamed body cannot sit there unchecked.
pub fn check(root: &Path, changelog: &str) -> Result<String, String> {
    let directory = root.join(DIRECTORY);
    if !directory.exists() {
        return Ok(format!(
            "release notes: no notes files under {DIRECTORY}/\n"
        ));
    }
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(&directory)
        .and_then(Iterator::collect)
        .map_err(|error| format!("read {DIRECTORY}/: {error}"))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    let mut failures = Vec::new();
    let mut valid = 0;
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let version = name
            .strip_suffix(".md")
            .filter(|stem| ordinal(stem).is_some())
            .filter(|_| entry.path().is_file());
        let Some(version) = version else {
            failures.push(format!(
                "{DIRECTORY}/{name}: {DIRECTORY}/ holds only `<version>.md` release bodies"
            ));
            continue;
        };
        let notes = fs::read_to_string(entry.path())
            .map_err(|error| format!("read {DIRECTORY}/{name}: {error}"))?;
        match validate(changelog, version, &notes) {
            Ok(()) => valid += 1,
            Err(refusals) => failures.push(refusal_report(version, &refusals)),
        }
    }
    if !failures.is_empty() {
        return Err(failures.join("\n"));
    }
    Ok(match valid {
        0 => format!("release notes: no notes files under {DIRECTORY}/\n"),
        count => format!("release notes: {count} notes files valid under {DIRECTORY}/\n"),
    })
}

/// The entries `changelog` lists under `## [Unreleased]`, each with its 1-based line: every line
/// starting `- ` or `* ` from that heading to the next `## ` heading.
pub fn unreleased_entries(changelog: &str) -> Vec<(usize, &str)> {
    let mut inside = false;
    let mut entries = Vec::new();
    for (index, line) in changelog.lines().enumerate() {
        if line.starts_with("## ") {
            inside = line.trim_end() == "## [Unreleased]";
            continue;
        }
        if inside && (line.starts_with("- ") || line.starts_with("* ")) {
            entries.push((index + 1, line));
        }
    }
    entries
}

/// `cargo xtask release unreleased`: refuses a changelog that lists an entry under
/// `[Unreleased]`.
///
/// A tag's tree is all released, so what it ships stands in its dated section. `release.yml`
/// runs this on a tag push and no pull-request gate does, because `main` lists unreleased entries
/// between releases. 0.99.8 merged `main` and shipped 25 entries its tree still listed there.
pub fn unreleased(changelog: &str) -> Result<String, String> {
    let entries = unreleased_entries(changelog);
    if entries.is_empty() {
        return Ok("release: CHANGELOG.md lists no entry under [Unreleased]\n".to_owned());
    }
    let count = match entries.len() {
        1 => "1 entry".to_owned(),
        count => format!("{count} entries"),
    };
    let lines: Vec<String> = entries
        .iter()
        .map(|(line, text)| {
            let shown: String = text.chars().take(100).collect();
            format!("CHANGELOG.md:{line}: {shown}")
        })
        .collect();
    Err(format!(
        "CHANGELOG.md lists {count} under [Unreleased] on a tagged tree; move each into the dated \
         section of the release that ships it:\n{}",
        lines.join("\n")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scratch::Scratch;
    use std::fs;

    /// A changelog in the shape ESS writes: headed sections, a wrapped breaking entry whose name
    /// wraps inside its code span, and the language count line.
    const CHANGELOG: &str = "# Changelog\n\n## [Unreleased]\n\n\
## [0.98.0] — 2026-11-22\n\n### Added\n\n\
- **`ess verify conform serve` holds a running target to its suite.** It reads `--suite <path>`\n  \
and drives the target process named by `--target`, so a program in any language is checked.\n\n\
### Changed\n\n\
- **Breaking for a runner that reads `suite/3`**: the suite format is `suite/4`; regenerate with `ess\n  \
generate suite`.\n\n\
### Fixed\n\n\
- The text report escapes control and bidirectional characters (#801).\n\n\
Language: 0 general, 1 one-to-one, 0 part (general share 0%; cumulative 17%).\n\n\
## [0.97.0] — 2026-11-21\n\n### Fixed\n\n\
- `ess specify validate` names the file of a refused node.\n";

    /// A body in the operator's measure's shape, against the 0.98.0 section above.
    const MODEL: &str = "Drive a target process in any language through its suite with `ess verify conform serve`.\n\
\n\
**Breaking**\n\
\n\
- A runner that reads `suite/3` must read `suite/4`; regenerate with `ess generate suite`\n\
\n\
**Changed**\n\
\n\
- `ess verify conform serve` reads `--suite <path>` and drives the process `--target` names\n\
- The text report escapes control and bidirectional characters\n\
\n\
Full entry: [CHANGELOG.md, 0.98.0](https://github.com/beyond10x/ess/blob/0.98.0/CHANGELOG.md#0980--2026-11-22)\n";

    /// A patch release with no breaking change, against the 0.97.0 section.
    const PATCH: &str = "`ess specify validate` names the file of a refused node.\n\
\n\
**Breaking**\n\
\n\
- None.\n\
\n\
**Changed**\n\
\n\
- `ess specify validate` names the file of a refused node\n\
\n\
Full entry: [CHANGELOG.md, 0.97.0](https://github.com/beyond10x/ess/blob/0.97.0/CHANGELOG.md#0970--2026-11-21)\n";

    /// The 0.97.0 body as the operator accepts it (brief v2, "The measure").
    const MEASURE: &str = "Test a program written in any language against its ESS specification.\n\
\n\
**Breaking**\n\
\n\
- None.\n\
\n\
**Changed**\n\
\n\
- `ess verify conform run --target process -- <program>` runs a suite against any program\n\
- `ess verify conform serve` is the reference peer for writing an adapter\n\
- The text report escapes control characters a target reports\n\
\n\
Full entry: [CHANGELOG.md, 0.97.0](https://github.com/beyond10x/ess/blob/0.97.0/CHANGELOG.md#0970--2026-11-21)\n";

    /// The second bullet of the model's `**Changed**` list, line 10.
    const LINE_10: &str = "- The text report escapes control and bidirectional characters\n";

    fn refusals(version: &str, notes: &str) -> Vec<Refusal> {
        validate(CHANGELOG, version, notes)
            .err()
            .unwrap_or_default()
    }

    /// The notes are refused with a refusal at `line` whose message contains every fragment.
    #[track_caller]
    fn assert_refused(version: &str, notes: &str, line: usize, fragments: &[&str]) {
        let found = refusals(version, notes);
        assert!(
            found.iter().any(|refusal| refusal.line == line
                && fragments
                    .iter()
                    .all(|fragment| refusal.message.contains(fragment))),
            "expected a refusal at line {line} containing {fragments:?}, got {found:#?}"
        );
    }

    #[track_caller]
    fn assert_valid(version: &str, notes: &str) {
        assert_eq!(validate(CHANGELOG, version, notes), Ok(()), "{notes}");
    }

    /// A bullet line of exactly `length` characters after `- `, with no word the rules refuse.
    fn bullet(n: usize, length: usize) -> String {
        let mut text = format!("The text report escapes control characters, case {n} ");
        while text.chars().count() < length {
            text.push('x');
        }
        text.truncate(length);
        format!("- {text}\n")
    }

    fn bullets(count: usize, length: usize) -> String {
        (1..=count).map(|n| bullet(n, length)).collect()
    }

    /// The characters of a body before its last line, the link, as `head -n -1 | wc -m` counts.
    fn counted(notes: &str) -> usize {
        let body = notes.strip_suffix('\n').unwrap_or(notes);
        body.rsplit_once('\n')
            .map_or(0, |(head, _)| head.chars().count() + 1)
    }

    #[test]
    fn the_model_bodies_are_accepted() {
        assert_valid("0.98.0", MODEL);
        assert_valid("0.97.0", PATCH);
    }

    #[test]
    fn the_operator_s_measure_validates_against_the_committed_0_97_0_section() {
        let changelog = include_str!("../../../../CHANGELOG.md");
        assert_eq!(validate(changelog, "0.97.0", MEASURE), Ok(()));
        assert_eq!(counted(MEASURE), 334);
    }

    #[test]
    fn a_body_over_nine_hundred_characters_is_refused() {
        let notes = MODEL
            .replacen(
                "- A runner that reads `suite/3` must read `suite/4`; regenerate with `ess generate suite`\n",
                &bullets(3, 110),
                1,
            )
            .replacen(
                "- `ess verify conform serve` reads `--suite <path>` and drives the process `--target` names\n",
                &bullets(4, 110),
                1,
            )
            .replacen(LINE_10, &bullet(9, 110), 1);
        let length = counted(&notes);
        assert!(length > 900 && length <= 1200, "{length}");
        assert_refused(
            "0.98.0",
            &notes,
            1,
            &[&format!("{length} characters"), "at most 900"],
        );
    }

    #[test]
    fn more_than_three_breaking_bullets_allow_twelve_hundred_characters_and_no_more() {
        let four = MODEL
            .replacen(
                "- A runner that reads `suite/3` must read `suite/4`; regenerate with `ess generate suite`\n",
                &bullets(4, 110),
                1,
            )
            .replacen(
                "- `ess verify conform serve` reads `--suite <path>` and drives the process `--target` names\n",
                &bullets(4, 110),
                1,
            )
            .replacen(LINE_10, &bullet(9, 110), 1);
        let length = counted(&four);
        assert!(length > 900 && length <= 1200, "{length}");
        assert_valid("0.98.0", &four);

        let five = four.replacen(
            &bullet(4, 110),
            &format!("{}{}", bullet(4, 110), bullet(5, 110)),
            1,
        );
        let lead = "Drive a target process in any language through its suite with `ess verify conform serve`.";
        let longer = five.replacen(
            lead,
            "Drive a target process written in any language through its suite and its report with `ess verify conform serve` and `--suite <path>`.",
            1,
        );
        let length = counted(&longer);
        assert!(length > 1200, "{length}");
        assert_refused(
            "0.98.0",
            &longer,
            1,
            &[&format!("{length} characters"), "at most 1200"],
        );
    }

    #[test]
    fn the_body_length_counts_characters_not_bytes() {
        // Each `é` is two bytes and one character: five bullets of them keep a body under 900
        // characters whose bytes before the link line are over 900.
        let accented = format!("- The report {}\n", "é".repeat(97));
        let notes = MODEL
            .replacen(
                "- `ess verify conform serve` reads `--suite <path>` and drives the process `--target` names\n",
                &accented.repeat(4),
                1,
            )
            .replacen(LINE_10, &accented, 1);
        let bytes = notes
            .rsplit_once("\nFull entry")
            .map_or(0, |(head, _)| head.len() + 1);
        assert!(counted(&notes) <= 900, "{}", counted(&notes));
        assert!(bytes > 900, "{bytes}");
        assert_valid("0.98.0", &notes);
    }

    #[test]
    fn a_missing_lead_is_refused() {
        let notes = format!("\n{MODEL}");
        assert_refused("0.98.0", &notes, 1, &["lead"]);
        let notes = MODEL.split_once("\n\n").map(|(_, rest)| rest).unwrap();
        assert_refused("0.98.0", notes, 1, &["lead"]);
    }

    #[test]
    fn a_lead_of_a_second_sentence_is_refused() {
        for second in [
            "suite. With",
            "suite? With",
            "suite! With",
            "suite. `ess generate suite` with",
        ] {
            let notes = MODEL.replacen("suite with", second, 1);
            assert_refused("0.98.0", &notes, 1, &["2 sentences", "exactly 1"]);
        }
        // A version number, or a lower-case word after a period, starts no sentence.
        for one in ["suite in 0.98.0 with", "suite, e.g. with"] {
            assert_valid("0.98.0", &MODEL.replacen("suite with", one, 1));
        }
    }

    #[test]
    fn a_lead_over_twenty_five_words_is_refused() {
        let lead = "Drive a target process in any language through its suite with `ess verify conform serve`.";
        // 15 whitespace-separated words, 4 of them inside the code span: 10 more make 25, 11 more
        // make 26.
        let words = |count: usize| {
            format!(
                "Drive a target process in any language through its suite {}with `ess verify conform serve`.",
                "and ".repeat(count)
            )
        };
        assert_valid("0.98.0", &MODEL.replacen(lead, &words(10), 1));
        assert_refused(
            "0.98.0",
            &MODEL.replacen(lead, &words(11), 1),
            1,
            &["26 words", "at most 25"],
        );
    }

    #[test]
    fn a_lead_over_two_lines_is_refused() {
        let notes = MODEL.replacen("suite with", "suite\nwith", 1);
        assert_refused("0.98.0", &notes, 2, &["2 lines", "at most 1"]);
    }

    #[test]
    fn a_lead_that_counts_entries_is_refused() {
        for count in ["2 added", "1 changed", "3 fixed", "1 removed", "2 Added"] {
            let notes = MODEL.replacen("Drive", &format!("With {count}, drive"), 1);
            assert_refused("0.98.0", &notes, 1, &["counts entries", count]);
        }
    }

    #[test]
    fn a_second_lead_paragraph_is_refused() {
        let notes = MODEL.replacen("serve`.\n", "serve`.\n\nA second paragraph.\n", 1);
        assert_refused("0.98.0", &notes, 3, &["`**Breaking**`"]);
    }

    #[test]
    fn a_missing_breaking_heading_is_refused() {
        let notes = MODEL.replacen("**Breaking**\n\n", "", 1);
        assert_refused("0.98.0", &notes, 3, &["`**Breaking**`"]);
    }

    #[test]
    fn a_missing_changed_heading_is_refused() {
        let notes = MODEL.replacen("**Changed**\n\n", "", 1);
        assert_refused("0.98.0", &notes, 7, &["`**Changed**`"]);
    }

    #[test]
    fn another_heading_is_refused() {
        for heading in ["# Notes", "## Notes", "### Fixed"] {
            let notes = MODEL.replacen("**Changed**", heading, 1);
            assert_refused("0.98.0", &notes, 7, &["heading"]);
        }
    }

    #[test]
    fn another_bold_line_is_refused() {
        let notes = MODEL.replacen("**Changed**", "**Fixed**", 1);
        assert_refused("0.98.0", &notes, 7, &["`**Fixed**`"]);
    }

    #[test]
    fn a_table_is_refused() {
        let notes = MODEL.replacen(LINE_10, "| a | b |\n", 1);
        assert_refused("0.98.0", &notes, 10, &["table"]);
    }

    #[test]
    fn a_code_fence_is_refused() {
        let notes = MODEL.replacen(LINE_10, "```\n", 1);
        assert_refused("0.98.0", &notes, 10, &["code fence"]);
    }

    #[test]
    fn a_bullet_over_one_hundred_and_ten_characters_is_refused() {
        let notes = MODEL.replacen(LINE_10, &bullet(1, 111), 1);
        assert_refused("0.98.0", &notes, 10, &["111 characters", "at most 110"]);
        assert_valid("0.98.0", &MODEL.replacen(LINE_10, &bullet(1, 110), 1));
    }

    #[test]
    fn a_bullet_continued_on_a_second_line_is_refused() {
        let notes = MODEL.replacen(
            LINE_10,
            "- The text report escapes control\n  and bidirectional characters\n",
            1,
        );
        assert_refused("0.98.0", &notes, 11, &["one line"]);
    }

    #[test]
    fn an_empty_list_is_refused() {
        let notes = MODEL.replacen(
            &format!(
                "- `ess verify conform serve` reads `--suite <path>` and drives the process `--target` names\n{LINE_10}\n"
            ),
            "",
            1,
        );
        assert_refused("0.98.0", &notes, 9, &["no bullet"]);
    }

    #[test]
    fn a_list_of_more_than_five_bullets_is_refused() {
        let changed = MODEL.replacen(LINE_10, &bullets(5, 40), 1);
        assert_refused(
            "0.98.0",
            &changed,
            9,
            &["`**Changed**`", "6 bullets", "at most 5"],
        );
        assert_valid("0.98.0", &MODEL.replacen(LINE_10, &bullets(4, 40), 1));
        let breaking = MODEL.replacen(
            "- A runner that reads `suite/3` must read `suite/4`; regenerate with `ess generate suite`\n",
            &bullets(6, 40),
            1,
        );
        assert_refused(
            "0.98.0",
            &breaking,
            5,
            &["`**Breaking**`", "6 bullets", "at most 5"],
        );
    }

    #[test]
    fn none_is_only_the_single_breaking_bullet() {
        let beside = PATCH.replacen(
            "- None.\n",
            "- None.\n- `ess specify validate` names the file\n",
            1,
        );
        assert_refused("0.97.0", &beside, 5, &["`- None.`"]);
        let changed = PATCH.replacen(
            "- `ess specify validate` names the file of a refused node\n",
            "- None.\n",
            1,
        );
        assert_refused("0.97.0", &changed, 9, &["`- None.`"]);
        // The first brief's spelling is no longer the no-breaking form: it is a bullet that ends
        // with a period.
        let old = PATCH.replacen("- None.\n", "- No breaking changes.\n", 1);
        assert_refused("0.97.0", &old, 5, &["period"]);
    }

    #[test]
    fn a_breaking_change_the_section_marks_cannot_be_dropped() {
        let notes = MODEL.replacen(
            "- A runner that reads `suite/3` must read `suite/4`; regenerate with `ess generate suite`\n",
            "- None.\n",
            1,
        );
        assert_refused(
            "0.98.0",
            &notes,
            5,
            &["marks a breaking change", "`**Breaking`"],
        );
    }

    #[test]
    fn breaking_bullets_the_section_does_not_mark_are_allowed() {
        let notes = PATCH.replacen(
            "- None.\n",
            "- A caller parsing the text of `ess specify validate` reads the file it names\n",
            1,
        );
        assert_valid("0.97.0", &notes);
    }

    #[test]
    fn a_bullet_refuses_so_that_parentheses_and_code_points() {
        for (text, fragment) in [
            (
                "The text report escapes characters so that a terminal is safe",
                "`so that`",
            ),
            (
                "The text report escapes characters So That a terminal is safe",
                "`So That`",
            ),
            ("The text report escapes control characters (C1)", "`(`"),
            ("The text report escapes control characters C1)", "`)`"),
            ("The text report escapes U+2028", "`U+2028`"),
        ] {
            let notes = MODEL.replacen(LINE_10, &format!("- {text}\n"), 1);
            assert_refused("0.98.0", &notes, 10, &[fragment]);
        }
        // `U+` before no hex digit, and `so` without `that`, are words like any other.
        assert_valid(
            "0.98.0",
            &MODEL.replacen(LINE_10, "- The text report escapes U+ and so on\n", 1),
        );
    }

    #[test]
    fn a_semicolon_is_refused_except_one_in_a_breaking_bullet() {
        let changed = MODEL.replacen(LINE_10, "- The text report escapes control; and more\n", 1);
        assert_refused("0.98.0", &changed, 10, &["1 semicolon", "at most 0"]);
        let two = MODEL.replacen("`suite/4`; regenerate", "`suite/4`; then; regenerate", 1);
        assert_refused("0.98.0", &two, 5, &["2 semicolons", "at most 1"]);
        assert_valid("0.98.0", MODEL);
    }

    #[test]
    fn now_again_and_its_own_are_refused_as_whole_words_outside_code() {
        for (text, fragment) in [
            ("The text report now escapes control characters", "`now`"),
            ("Now the text report escapes control characters", "`Now`"),
            (
                "The text report escapes control characters again",
                "`again`",
            ),
            ("The text report escapes its own characters", "`its own`"),
            ("The text report escapes Its  Own characters", "`Its  Own`"),
        ] {
            let notes = MODEL.replacen(LINE_10, &format!("- {text}\n"), 1);
            assert_refused("0.98.0", &notes, 10, &[fragment]);
        }
        let lead = MODEL.replacen("Drive a", "Now drive a", 1);
        assert_refused("0.98.0", &lead, 1, &["`Now`"]);
        assert_valid(
            "0.98.0",
            &MODEL.replacen(
                LINE_10,
                "- The known text report escapes against snow and its ownership\n",
                1,
            ),
        );
    }

    #[test]
    fn a_bullet_ending_with_a_period_is_refused() {
        let notes = MODEL.replacen(
            LINE_10,
            "- The text report escapes control and bidirectional characters.\n",
            1,
        );
        assert_refused("0.98.0", &notes, 10, &["period"]);
    }

    #[test]
    fn every_token_of_a_code_span_occurs_in_the_section() {
        let notes = MODEL.replacen("`--target` names", "`--target-process` names", 1);
        assert_refused("0.98.0", &notes, 9, &["`--target-process`", "0.98.0"]);
        let lead = MODEL.replacen("its suite", "its `suite/9`", 1);
        assert_refused("0.98.0", &lead, 1, &["`suite/9`"]);
        // A name from another release's section is not this release's name.
        let other = MODEL.replacen("`--target` names", "`ess specify validate` names", 1);
        assert_refused("0.98.0", &other, 9, &["`specify`"]);
        assert_refused("0.98.0", &other, 9, &["`validate`"]);
        // A span whose tokens all occur is accepted though the section never spells it whole.
        let joined = MODEL.replacen(
            "- `ess verify conform serve` reads `--suite <path>` and drives the process `--target` names\n",
            "- `ess verify conform serve --target <path>` drives a process\n",
            1,
        );
        assert_valid("0.98.0", &joined);
        let double = MODEL.replacen("`ess generate suite`", "``ess generate suite``", 1);
        assert_valid("0.98.0", &double);
    }

    #[test]
    fn a_name_wrapped_across_changelog_lines_is_read_joined() {
        // The section wraps `ess generate suite` after `ess`; the model names it.
        assert!(MODEL.contains("`ess generate suite`"));
        assert_valid("0.98.0", MODEL);
    }

    #[test]
    fn an_unclosed_code_span_is_refused() {
        let notes = MODEL.replacen("`--target` names", "`--target names", 1);
        assert_refused("0.98.0", &notes, 9, &["unclosed"]);
    }

    #[test]
    fn a_wrong_link_line_is_refused() {
        let notes = MODEL.replacen("#0980--2026-11-22", "#0980-2026-11-22", 1);
        assert_refused(
            "0.98.0",
            &notes,
            12,
            &["Full entry: [CHANGELOG.md, 0.98.0](https://github.com/beyond10x/ess/blob/0.98.0/CHANGELOG.md#0980--2026-11-22)"],
        );
        let missing = MODEL.replacen("\nFull entry", "\nEvery change", 1);
        assert_refused("0.98.0", &missing, 12, &["Full entry"]);
    }

    #[test]
    fn anything_after_the_link_line_is_refused() {
        let notes = format!("{MODEL}\nMore.\n");
        assert_refused("0.98.0", &notes, 13, &["last line"]);
        let trailing = format!("{MODEL}\n");
        assert_refused("0.98.0", &trailing, 13, &["last line"]);
    }

    #[test]
    fn a_body_without_its_final_newline_is_refused() {
        let notes = MODEL.trim_end_matches('\n');
        assert_refused("0.98.0", notes, 12, &["newline"]);
    }

    #[test]
    fn notes_for_a_version_without_a_section_are_refused() {
        assert_refused(
            "0.99.0",
            MODEL,
            1,
            &["CHANGELOG.md has no release heading for 0.99.0"],
        );
    }

    #[test]
    fn every_refusal_is_reported_not_only_the_first() {
        let notes = MODEL
            .replacen("`--target` names", "`--target-process` names", 1)
            .replacen(LINE_10, &bullet(1, 111), 1);
        let found = refusals("0.98.0", &notes);
        assert_eq!(
            found.iter().map(|refusal| refusal.line).collect::<Vec<_>>(),
            [9, 10],
            "{found:#?}"
        );
    }

    #[test]
    fn the_anchor_is_github_s_anchor_of_the_heading_as_spelled() {
        assert_eq!(anchor("## [0.97.0] — 2026-11-21"), "0970--2026-11-21");
        assert_eq!(anchor("## [0.80.0] - 2026-11-01"), "0800---2026-11-01");
        assert_eq!(anchor("## [0.80.0] 2026-11-01"), "0800-2026-11-01");
        assert_eq!(anchor("## [1.0.0_rc] — Ünïcode"), "100_rc--ünïcode");
    }

    #[test]
    fn every_heading_from_0_79_0_on_has_the_anchor_of_its_spelling() {
        // Every release heading from 0.79.0 to 0.97.0 is spelled `## [X.Y.Z] — YYYY-MM-DD`.
        let changelog = include_str!("../../../../CHANGELOG.md");
        let headings: Vec<&str> = changelog
            .lines()
            .filter(|line| line.starts_with("## [0."))
            .filter(|line| {
                let version = &line[4..line.find(']').unwrap_or(4)];
                ordinal(version).is_some_and(|number| ((0, 79, 0)..=(0, 97, 0)).contains(&number))
            })
            .collect();
        assert_eq!(headings.len(), 21, "{headings:#?}");
        for heading in headings {
            let rest = &heading[heading.find("] — ").expect("an em dash heading") + "] — ".len()..];
            let version: String = heading[4..heading.find(']').unwrap()]
                .chars()
                .filter(char::is_ascii_digit)
                .collect();
            assert_eq!(anchor(heading), format!("{version}--{rest}"), "{heading}");
        }
    }

    #[test]
    fn a_section_marks_breaking_by_an_entry_that_starts_with_bold_breaking() {
        assert!(marks_breaking("- **Breaking:** suites are `s/2`.\n"));
        assert!(marks_breaking(
            "### Changed\n\n- **Breaking for a client that sends a list**: refused.\n"
        ));
        assert!(!marks_breaking(
            "- A refusal. Breaking for nobody.\n- **Fixed** a thing.\n"
        ));
        assert!(!marks_breaking(
            "Breaking changes are named in words here.\n"
        ));
        assert!(!marks_breaking("  - **Breaking:** a nested list item.\n"));
    }

    #[test]
    fn a_section_stops_before_the_next_release() {
        let changelog = "# Changelog\n\n## [0.1.0] — 2026-09-01\n\n- First.\n\n## [0.0.1] — 2026-08-01\n\n- Old.\n";
        assert_eq!(
            section(changelog, "0.1.0"),
            Ok(Section {
                heading: "## [0.1.0] — 2026-09-01".to_owned(),
                body: "- First.\n".to_owned()
            })
        );
    }

    #[test]
    fn an_undated_release_is_refused_by_name() {
        assert_eq!(
            section("## [0.1.0] 2026-09-01\n\n- First.\n", "0.1.0"),
            Err("release heading for 0.1.0 has no date".to_owned())
        );
    }

    #[test]
    fn an_empty_release_is_refused_by_name() {
        assert_eq!(
            section("## [0.1.0] — 2026-09-01\n\n", "0.1.0"),
            Err("release heading for 0.1.0 has no notes".to_owned())
        );
    }

    #[test]
    fn notes_are_required_from_0_79_0() {
        assert!(!required("0.53.0"));
        assert!(!required("0.78.9"));
        assert!(required("0.79.0"));
        assert!(required("0.98.0"));
        assert!(required("1.0.0"));
    }

    fn root(name: &str) -> Scratch {
        let scratch = Scratch::new(&format!("ess-xtask-release-notes-{name}"));
        fs::write(scratch.path().join("CHANGELOG.md"), CHANGELOG).expect("changelog");
        scratch
    }

    #[test]
    fn printing_a_missing_file_names_the_path_and_the_rule() {
        let scratch = root("missing");
        let refusal = print(scratch.path(), CHANGELOG, "0.98.0").expect_err("no file");
        assert!(refusal.contains("release-notes/0.98.0.md"), "{refusal}");
        assert!(refusal.contains("release body"), "{refusal}");
        assert_eq!(refusal, missing("0.98.0"));
    }

    #[test]
    fn printing_a_valid_file_prints_it_byte_for_byte() {
        let scratch = root("print");
        fs::create_dir_all(scratch.path().join(DIRECTORY)).expect("directory");
        fs::write(scratch.path().join(path("0.98.0")), MODEL).expect("notes");
        assert_eq!(
            print(scratch.path(), CHANGELOG, "0.98.0"),
            Ok(MODEL.to_owned())
        );
    }

    #[test]
    fn printing_an_invalid_file_names_the_file_and_each_line() {
        let scratch = root("invalid");
        fs::create_dir_all(scratch.path().join(DIRECTORY)).expect("directory");
        let notes = MODEL.replacen("`--target` names", "`--target-process` names", 1);
        fs::write(scratch.path().join(path("0.98.0")), notes).expect("notes");
        let refusal = print(scratch.path(), CHANGELOG, "0.98.0").expect_err("invalid");
        assert!(refusal.contains("release-notes/0.98.0.md:9: "), "{refusal}");
    }

    #[test]
    fn checking_an_absent_or_empty_directory_passes() {
        let scratch = root("empty");
        assert!(check(scratch.path(), CHANGELOG).is_ok());
        fs::create_dir_all(scratch.path().join(DIRECTORY)).expect("directory");
        let report = check(scratch.path(), CHANGELOG).expect("empty directory");
        assert!(report.contains("no notes files"), "{report}");
    }

    #[test]
    fn checking_validates_every_file_against_its_own_section_and_names_each_failure() {
        let scratch = root("all");
        let directory = scratch.path().join(DIRECTORY);
        fs::create_dir_all(&directory).expect("directory");
        fs::write(directory.join("0.98.0.md"), MODEL).expect("notes");
        fs::write(directory.join("0.97.0.md"), PATCH).expect("notes");
        let report = check(scratch.path(), CHANGELOG).expect("both valid");
        assert!(report.contains("2 notes files"), "{report}");

        // 0.97.0's body is checked against 0.97.0's section, so 0.98.0's names are invented there.
        fs::write(
            directory.join("0.97.0.md"),
            MODEL.replace("0.98.0", "0.97.0"),
        )
        .expect("notes");
        fs::write(directory.join("README.md"), "Notes.\n").expect("stray");
        fs::create_dir_all(directory.join("drafts")).expect("stray directory");
        let refusal = check(scratch.path(), CHANGELOG).expect_err("failures");
        assert!(refusal.contains("release-notes/0.97.0.md:1: "), "{refusal}");
        assert!(refusal.contains("release-notes/README.md"), "{refusal}");
        assert!(refusal.contains("release-notes/drafts"), "{refusal}");
        assert!(!refusal.contains("release-notes/0.98.0.md"), "{refusal}");
    }

    #[test]
    fn a_tagged_tree_with_an_unreleased_entry_is_refused_at_its_line() {
        let changelog = CHANGELOG.replace(
            "## [Unreleased]\n\n",
            "## [Unreleased]\n\n### Added\n\n- A chart takes `sort: {by, dir}`.\n\n",
        );
        let refusal = unreleased(&changelog).expect_err("an unreleased entry");
        assert!(refusal.contains("1 entry under [Unreleased]"), "{refusal}");
        assert!(
            refusal.contains("CHANGELOG.md:7: - A chart takes `sort: {by, dir}`."),
            "{refusal}"
        );
    }

    #[test]
    fn a_tagged_tree_whose_entries_all_stand_in_dated_sections_passes() {
        assert!(unreleased_entries(CHANGELOG).is_empty());
        let report = unreleased(CHANGELOG).expect("every entry is dated");
        assert!(report.contains("no entry under [Unreleased]"), "{report}");
    }
}
