//! A calendar window over an instant, at UTC or a fixed UTC offset (beyond10x/ess#244 part b,
//! `docs/design/calendar-window-guards.md`).
//!
//! ```yaml
//! window: {at: now, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}
//! ```
//!
//! holds where the instant `at` names, moved by the offset, falls on a listed weekday at or after
//! `from` and before `to`. A window whose `from` is after its `to` crosses midnight and belongs to
//! the day it opens: `days: [fri], from: "22:00", to: "02:00"` holds Friday 22:00 to Saturday 02:00.
//!
//! # No zone database
//!
//! The offset is a fixed number of minutes, `Z` or `±HH:MM`. A named time zone — `Europe/Berlin`,
//! `UTC` — is refused by name: its offset moves with daylight saving, and every evaluator (Rust, Go,
//! TypeScript) would have to carry and agree on the same zone data to decide it. A window therefore
//! does not follow daylight saving, and says so where it is refused.

use std::fmt;

use crate::error::ParseError;
use crate::facts::{FactPath, FactSource};
use crate::node::Node;
use crate::predicate::Truth;
use crate::time::{CivilDate, Rfc3339Instant};

/// A day of the week, Monday first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weekday {
    /// `mon`.
    Monday,
    /// `tue`.
    Tuesday,
    /// `wed`.
    Wednesday,
    /// `thu`.
    Thursday,
    /// `fri`.
    Friday,
    /// `sat`.
    Saturday,
    /// `sun`.
    Sunday,
}

impl Weekday {
    /// Every day, Monday first.
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// The one spelling a window writes the day with.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::Monday => "mon",
            Self::Tuesday => "tue",
            Self::Wednesday => "wed",
            Self::Thursday => "thu",
            Self::Friday => "fri",
            Self::Saturday => "sat",
            Self::Sunday => "sun",
        }
    }

    /// The day spelled `keyword`, or `None`.
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|day| day.keyword() == keyword)
    }

    /// Monday is `0`, Sunday `6`.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The day `days` days after 1970-01-01, a Thursday.
    pub fn of_epoch_day(days: i64) -> Self {
        // `rem_euclid(7)` is in `0..7`, so the index is in range.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = (days + 3).rem_euclid(7) as usize;
        Self::ALL[index]
    }

    /// The day before this one.
    #[must_use]
    pub fn previous(self) -> Self {
        Self::ALL[(self.index() + 6) % 7]
    }
}

/// The days a window lists, as a set, iterated Monday first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Weekdays(u8);

impl Weekdays {
    /// No day.
    pub const NONE: Self = Self(0);

    /// Whether `day` is listed.
    pub fn contains(self, day: Weekday) -> bool {
        self.0 & (1 << day.index()) != 0
    }

    /// The set with `day` listed too.
    #[must_use]
    pub fn with(self, day: Weekday) -> Self {
        Self(self.0 | (1 << day.index()))
    }

    /// Whether no day is listed.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The listed days, Monday first.
    pub fn iter(self) -> impl Iterator<Item = Weekday> {
        Weekday::ALL
            .into_iter()
            .filter(move |day| self.contains(*day))
    }
}

impl FromIterator<Weekday> for Weekdays {
    fn from_iter<I: IntoIterator<Item = Weekday>>(days: I) -> Self {
        days.into_iter().fold(Self::NONE, Self::with)
    }
}

/// What a window tests: the decision's current time, or the instant a fact holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowInstant {
    /// `now`: the instant the evaluator is given ([`FactSource::now`]).
    Now,
    /// A `Timestamp` fact.
    Fact(FactPath),
}

impl WindowInstant {
    /// The word `at` reads as the current time.
    pub const NOW: &'static str = "now";

    /// The fact path, where this is one.
    pub fn fact_path(&self) -> Option<&FactPath> {
        match self {
            Self::Now => None,
            Self::Fact(path) => Some(path),
        }
    }
}

impl fmt::Display for WindowInstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Now => f.write_str(Self::NOW),
            Self::Fact(path) => write!(f, "{path}"),
        }
    }
}

/// The largest offset a window takes either way, in minutes: 14 hours.
pub const MAX_OFFSET_MINUTES: i32 = 14 * 60;

/// Minutes in one day; `24:00`, the end of the day a `to` may name.
pub const MINUTES_PER_DAY: u16 = 24 * 60;

/// Seconds in one day.
const SECONDS_PER_DAY: i64 = 86_400;

/// The Monday of the week synthesis tries a window's boundaries on: 2020-01-06, in the month every
/// `Timestamp` witness is built in.
const REFERENCE_MONDAY: (i32, u32, u32) = (2020, 1, 6);

/// The keys a window is written with, in canonical order.
const KEYS: [&str; 5] = ["at", "days", "from", "offset", "to"];

/// Keys an author reaching for a named zone writes, each refused naming the fixed offset.
const ZONE_KEYS: [&str; 4] = ["zone", "tz", "timezone", "time_zone"];

/// A calendar window over an instant (`docs/design/calendar-window-guards.md`).
///
/// Fields are public, as a quantifier's are, so a caller assembling or mutating a window can do so;
/// [`Self::parse_mapping`] is the reader that holds every rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarWindow {
    /// What instant is tested.
    pub at: WindowInstant,
    /// The listed days; never empty once read.
    pub days: Weekdays,
    /// The inclusive start, in minutes after midnight, `0..=1439`.
    pub from: u16,
    /// The exclusive end, in minutes after midnight, `1..=1440`; before `from` where the window
    /// crosses midnight.
    pub to: u16,
    /// Minutes east of UTC, `-840..=840`.
    pub offset_minutes: i32,
}

impl CalendarWindow {
    /// This window with the fact `at` reads moved to the path `map` answers; `now` stays `now`.
    #[must_use]
    pub fn map_path(&self, map: impl FnOnce(&FactPath) -> FactPath) -> Self {
        Self {
            at: match &self.at {
                WindowInstant::Now => WindowInstant::Now,
                WindowInstant::Fact(path) => WindowInstant::Fact(map(path)),
            },
            ..self.clone()
        }
    }

    /// Whether this window crosses midnight: `from` after `to`.
    pub fn crosses_midnight(&self) -> bool {
        self.from > self.to
    }

    /// Whether `instant` falls inside this window.
    ///
    /// The instant's whole UTC second, moved by the offset, is a local second: its civil day gives
    /// the weekday and its remainder the time of day. Dropping a fraction is exact, because every
    /// boundary is a whole minute.
    pub fn contains(&self, instant: Rfc3339Instant) -> bool {
        let local = instant.epoch_seconds() + i64::from(self.offset_minutes) * 60;
        let day = Weekday::of_epoch_day(local.div_euclid(SECONDS_PER_DAY));
        let second = local.rem_euclid(SECONDS_PER_DAY);
        let (from, to) = (i64::from(self.from) * 60, i64::from(self.to) * 60);
        if self.crosses_midnight() {
            (self.days.contains(day) && second >= from)
                || (self.days.contains(day.previous()) && second < to)
        } else {
            self.days.contains(day) && from <= second && second < to
        }
    }

    /// Evaluates this window against `facts`, with a note where it cannot be decided.
    ///
    /// `Unknown` where the fact is unobserved or absent, where what it holds is no RFC 3339
    /// instant, and where `at` is `now` and `facts` was given no clock.
    pub fn evaluate(&self, facts: &dyn FactSource) -> (Truth, Option<String>) {
        let instant = match &self.at {
            WindowInstant::Now => match facts.now() {
                Some(now) => now,
                None => return (Truth::Unknown, None),
            },
            WindowInstant::Fact(path) => {
                let Some(value) = facts.observe(path) else {
                    return (Truth::Unknown, None);
                };
                match value.as_text().and_then(Rfc3339Instant::parse_rfc3339) {
                    Some(instant) => instant,
                    None => {
                        return (
                            Truth::Unknown,
                            Some(format!(
                                "`{self}` cannot place {value} in the window: `{path}` must be an \
                                 RFC 3339 date-time"
                            )),
                        )
                    }
                }
            }
        };
        (Truth::from_bool(self.contains(instant)), None)
    }

    /// The instant at `minute` past local midnight on the civil `date` in this window's offset.
    fn local(&self, date: CivilDate, minute: u16) -> Option<Rfc3339Instant> {
        Rfc3339Instant::from_epoch_seconds(
            date.days_from_epoch() * SECONDS_PER_DAY + i64::from(minute) * 60
                - i64::from(self.offset_minutes) * 60,
        )
    }

    /// The instants either side of each boundary of this window on the reference week (Monday
    /// 2020-01-06 to Sunday 2020-01-12 in its offset), in time order, without repeats: for every
    /// listed day and every day after a listed one, a second before `from`, `from`, a second before
    /// `to` and `to` — `to` on the next day where the window crosses midnight.
    ///
    /// What synthesis witnesses a window over a fact at; [`Self::contains`] says which side each is.
    pub fn boundaries(&self) -> Vec<Rfc3339Instant> {
        let (year, month, day) = REFERENCE_MONDAY;
        let Ok(monday) = CivilDate::new(year, month, day) else {
            return Vec::new();
        };
        let monday = monday.days_from_epoch();
        let mut found = Vec::new();
        for (index, weekday) in Weekday::ALL.into_iter().enumerate() {
            if !self.days.contains(weekday) && !self.days.contains(weekday.previous()) {
                continue;
            }
            let offset = i64::try_from(index).unwrap_or(0);
            let today = CivilDate::from_epoch_day(monday + offset);
            let tomorrow = CivilDate::from_epoch_day(monday + offset + 1);
            let end = if self.crosses_midnight() {
                (tomorrow, self.to)
            } else {
                (today, self.to)
            };
            for (date, minute) in [(today, self.from), end] {
                if let Some(instant) = self.local(date, minute) {
                    found.extend(instant.plus_seconds(-1));
                    found.push(instant);
                }
            }
        }
        found.sort();
        found.dedup();
        found
    }

    /// A few instants of [`Self::boundaries`] that decide this window, most telling first: `from`
    /// on the first listed day (inside), `to` after it (outside, which an inclusive `to` takes in),
    /// `from` on each unlisted day (outside, which an extra listed day takes in), a second before
    /// the first `from` (outside), a second before the last `to` (inside), and `from` on every other
    /// listed day (inside, which a dropped day leaves out).
    ///
    /// What synthesis arranges a window over a stored instant at, where each further row is a row
    /// of its own and the rows are bounded.
    pub fn deciding_instants(&self) -> Vec<Rfc3339Instant> {
        let (year, month, day) = REFERENCE_MONDAY;
        let Ok(monday) = CivilDate::new(year, month, day) else {
            return Vec::new();
        };
        let monday = monday.days_from_epoch();
        let date = |weekday: Weekday, later: i64| {
            CivilDate::from_epoch_day(monday + i64::try_from(weekday.index()).unwrap_or(0) + later)
        };
        let end = |weekday: Weekday| {
            let later = i64::from(self.crosses_midnight());
            self.local(date(weekday, later), self.to)
        };
        let listed: Vec<Weekday> = self.days.iter().collect();
        let (Some(first), Some(last)) = (listed.first().copied(), listed.last().copied()) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        found.extend(self.local(date(first, 0), self.from));
        found.extend(end(first));
        for weekday in Weekday::ALL {
            if !self.days.contains(weekday) {
                found.extend(self.local(date(weekday, 0), self.from));
            }
        }
        found.extend(
            self.local(date(first, 0), self.from)
                .and_then(|from| from.plus_seconds(-1)),
        );
        found.extend(end(last).and_then(|to| to.plus_seconds(-1)));
        for weekday in listed.iter().skip(1) {
            found.extend(self.local(date(*weekday, 0), self.from));
        }
        let mut kept = Vec::new();
        for instant in found {
            if !kept.contains(&instant) {
                kept.push(instant);
            }
        }
        kept
    }

    /// `instant` spelled at a fixed offset under which its written clock, read as UTC, falls on the
    /// other side of this window — so a reader that drops an instant's own offset, or compares the
    /// written clock, decides it otherwise — or in UTC where no whole-hour offset does.
    ///
    /// What synthesis sends a window's further witnesses as: the instant is the same, and only a
    /// reader of its spelling is told apart.
    pub fn spelled_against(&self, instant: Rfc3339Instant) -> String {
        let inside = self.contains(instant);
        for hours in [
            -5, 5, -1, 1, -2, 2, -3, 3, -4, 4, -6, 6, -8, 8, -10, 10, -12, 12, -14, 14,
        ] {
            let minutes: i32 = hours * 60;
            let read_as_utc = instant.plus_seconds(i64::from(minutes) * 60);
            if read_as_utc.is_some_and(|read| self.contains(read) != inside) {
                if let Some(text) = instant.to_rfc3339_at(minutes) {
                    return text;
                }
            }
        }
        instant.to_rfc3339()
    }

    /// Reads the mapping written under `window:`.
    ///
    /// # Errors
    ///
    /// A [`ParseError::Predicate`] naming what is wrong: a key other than the five, a missing one,
    /// a named time zone, an offset that is not `Z` or `±HH:MM` within 14 hours, a time that is not
    /// quoted `HH:MM`, `from` equal to `to`, `to: "00:00"` (written `24:00`), no day, a day twice or
    /// a day not spelled `mon` to `sun`, and an `at` that is neither `now` nor a fact path.
    pub fn parse_mapping(value: &Node) -> Result<Self, ParseError> {
        let written = || format!("window: {}", shallow(value));
        let refuse = |reason: String| ParseError::predicate(&written(), reason);
        let Node::Map(fields) = value else {
            return Err(refuse(
                "a window is a mapping of `at`, `days`, `from`, `to` and `offset`".to_owned(),
            ));
        };
        for key in fields.keys() {
            if ZONE_KEYS.contains(&key.as_str()) {
                return Err(refuse(format!(
                    "`{key}` names a time zone, and a calendar window takes a fixed offset: write \
                     `offset:` as `Z` or `±HH:MM`, such as `+01:00`. {NO_ZONE_DATA}"
                )));
            }
            if !KEYS.contains(&key.as_str()) {
                return Err(refuse(format!(
                    "a window takes `at`, `days`, `from`, `to` and `offset`, and nothing else; \
                     `{key}` is none of them"
                )));
            }
        }
        let field = |key: &str| {
            fields
                .get(key)
                .ok_or_else(|| refuse(format!("a window takes `{key}`, and this one has none")))
        };
        let at = match field("at")? {
            Node::Text(text) if text == WindowInstant::NOW => WindowInstant::Now,
            Node::Text(text) => WindowInstant::Fact(FactPath::new(text).map_err(|error| {
                refuse(format!(
                    "`at` is `now` or the fact path of a Timestamp: {error}"
                ))
            })?),
            other => {
                return Err(refuse(format!(
                    "`at` is `now` or the fact path of a Timestamp, not {}",
                    other.type_name()
                )))
            }
        };
        let days = days(field("days")?).map_err(refuse)?;
        let from = time_of_day("from", field("from")?).map_err(refuse)?;
        let to = time_of_day("to", field("to")?).map_err(refuse)?;
        if from == MINUTES_PER_DAY {
            return Err(refuse(
                "`from` is at most 23:59: a window starts within the day it opens on".to_owned(),
            ));
        }
        if to == 0 {
            return Err(refuse(
                "`to` 00:00 is the end of the day, which a window writes `24:00`".to_owned(),
            ));
        }
        if from == to {
            return Err(refuse(format!(
                "`from` and `to` are both {}: a window is either empty or the whole day, and the \
                 whole day is `from: \"00:00\", to: \"24:00\"`",
                clock(from)
            )));
        }
        let offset_minutes = offset(field("offset")?).map_err(refuse)?;
        Ok(Self {
            at,
            days,
            from,
            to,
            offset_minutes,
        })
    }

    /// The canonical mapping: every key, days Monday first, times `HH:MM`, the offset `Z` or
    /// `±HH:MM`.
    pub fn to_node(&self) -> Node {
        Node::Map(
            [
                ("at".to_owned(), Node::Text(self.at.to_string())),
                (
                    "days".to_owned(),
                    Node::Seq(
                        self.days
                            .iter()
                            .map(|day| Node::Text(day.keyword().to_owned()))
                            .collect(),
                    ),
                ),
                ("from".to_owned(), Node::Text(clock(self.from))),
                ("to".to_owned(), Node::Text(clock(self.to))),
                ("offset".to_owned(), Node::Text(self.offset_text())),
            ]
            .into(),
        )
    }

    /// The offset as written back: `Z`, or `±HH:MM`.
    pub fn offset_text(&self) -> String {
        if self.offset_minutes == 0 {
            return "Z".to_owned();
        }
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let magnitude = self.offset_minutes.unsigned_abs();
        format!("{sign}{:02}:{:02}", magnitude / 60, magnitude % 60)
    }
}

impl fmt::Display for CalendarWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let days: Vec<&str> = self.days.iter().map(Weekday::keyword).collect();
        write!(
            f,
            "window(at {}, {}, {}-{}, {})",
            self.at,
            days.join(" "),
            clock(self.from),
            clock(self.to),
            self.offset_text()
        )
    }
}

/// Why a zone is refused, appended to every refusal of one.
const NO_ZONE_DATA: &str = "A window is evaluated in UTC or a fixed offset because a named zone's \
                            offset moves with daylight saving and would need zone data every \
                            evaluator must agree on; a fixed offset does not follow daylight \
                            saving.";

/// `HH:MM` for a number of minutes after midnight, `24:00` included.
fn clock(minutes: u16) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// A time of day written `"HH:MM"`, `00:00` to `24:00`, as minutes after midnight.
fn time_of_day(key: &str, node: &Node) -> Result<u16, String> {
    let Node::Text(text) = node else {
        return Err(format!(
            "`{key}` is a time written as quoted text, \"HH:MM\"; this one is {} (an unquoted \
             time may be read as a number before ESS sees it)",
            node.type_name()
        ));
    };
    let bytes = text.as_bytes();
    let digit = |index: usize| {
        bytes
            .get(index)
            .filter(|byte| byte.is_ascii_digit())
            .map(|byte| u16::from(byte - b'0'))
    };
    let parsed = match (bytes.len(), bytes.get(2)) {
        (5, Some(b':')) => match (digit(0), digit(1), digit(3), digit(4)) {
            (Some(h1), Some(h2), Some(m1), Some(m2)) => Some((h1 * 10 + h2, m1 * 10 + m2)),
            _ => None,
        },
        _ => None,
    };
    match parsed {
        Some((hours, minutes)) if minutes < 60 && (hours < 24 || (hours == 24 && minutes == 0)) => {
            Ok(hours * 60 + minutes)
        }
        _ => Err(format!(
            "`{key}: {text}` is no time of day: write two-digit hours and minutes, \"HH:MM\", \
             from 00:00 to 24:00"
        )),
    }
}

/// The listed days, refusing an empty list, a repeat and a day not spelled `mon` to `sun`.
fn days(node: &Node) -> Result<Weekdays, String> {
    let Node::Seq(items) = node else {
        return Err(format!(
            "`days` is a list of `mon`, `tue`, `wed`, `thu`, `fri`, `sat` and `sun`, not {}",
            node.type_name()
        ));
    };
    if items.is_empty() {
        return Err("`days` lists no day, so the window holds at no instant".to_owned());
    }
    let mut days = Weekdays::NONE;
    for item in items {
        let day = item
            .as_text()
            .and_then(Weekday::from_keyword)
            .ok_or_else(|| {
                format!(
                    "`{}` is no day: write `mon`, `tue`, `wed`, `thu`, `fri`, `sat` or `sun`",
                    shallow(item)
                )
            })?;
        if days.contains(day) {
            return Err(format!("`days` lists `{}` twice", day.keyword()));
        }
        days = days.with(day);
    }
    Ok(days)
}

/// A fixed offset `Z` or `±HH:MM`, at most 14 hours either way, as minutes east of UTC.
fn offset(node: &Node) -> Result<i32, String> {
    let Node::Text(text) = node else {
        return Err(format!(
            "`offset` is `Z` or `±HH:MM`, written as text, not {}",
            node.type_name()
        ));
    };
    if text == "Z" {
        return Ok(0);
    }
    let bytes = text.as_bytes();
    let digit = |index: usize| {
        bytes
            .get(index)
            .filter(|byte| byte.is_ascii_digit())
            .map(|byte| i32::from(byte - b'0'))
    };
    let signed = match (bytes.len(), bytes.first(), bytes.get(3)) {
        (6, Some(sign @ (b'+' | b'-')), Some(b':')) => {
            match (digit(1), digit(2), digit(4), digit(5)) {
                (Some(h1), Some(h2), Some(m1), Some(m2)) if m1 * 10 + m2 < 60 => {
                    let minutes = (h1 * 10 + h2) * 60 + m1 * 10 + m2;
                    Some(if *sign == b'-' { -minutes } else { minutes })
                }
                _ => None,
            }
        }
        _ => None,
    };
    match signed {
        Some(0) if text.starts_with('-') => {
            Err("`offset: -00:00` is RFC 3339's unknown local offset; write `Z` for UTC".to_owned())
        }
        Some(minutes) if minutes.abs() <= MAX_OFFSET_MINUTES => Ok(minutes),
        Some(_) => Err(format!(
            "`offset: {text}` is past 14:00 either way, which no civil offset is"
        )),
        None if text
            .chars()
            .any(|character| character.is_ascii_alphabetic() || character == '/') =>
        {
            let hint = if ["UTC", "GMT", "UT", "Zulu", "utc", "z"].contains(&text.as_str()) {
                "write `Z`".to_owned()
            } else {
                "write the fixed offset, such as `+01:00`".to_owned()
            };
            Err(format!(
                "`{text}` names a time zone, and a calendar window takes a fixed offset: {hint}. \
                 {NO_ZONE_DATA}"
            ))
        }
        None => Err(format!(
            "`offset: {text}` is no fixed offset: write `Z` or `±HH:MM`, such as `+01:00`"
        )),
    }
}

/// A node as it reads in a refusal: one level only, never walking what it refuses, and cut short.
fn shallow(node: &Node) -> String {
    let text = match node {
        Node::Text(text) => text.clone(),
        Node::Seq(items) => format!("a list of {}", items.len()),
        Node::Map(entries) => format!(
            "{{{}}}",
            entries.keys().cloned().collect::<Vec<_>>().join(", ")
        ),
        scalar => scalar.type_name().to_owned(),
    };
    if text.chars().count() > 60 {
        format!("{}…", text.chars().take(60).collect::<String>())
    } else {
        text
    }
}
