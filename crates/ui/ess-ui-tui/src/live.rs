//! Channel event scripts played against a virtual clock.
//!
//! A fixture script lists entries at offsets from the start of the run. The player starts at
//! zero and moves only when [`crate::App::advance`] moves the clock, so a test decides exactly
//! which entries have played. A looping script restarts when its last entry has played: its
//! cycle is the offset of that last entry.

use std::collections::BTreeMap;
use std::time::Duration;

use serde_yaml::Value;

/// One channel's event script.
#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    /// The channel it plays.
    pub channel: String,
    /// Whether it restarts after its last entry.
    pub looping: bool,
    /// The session the script plays, for a channel with `session: {per}` (`{ticket_id: tk-01}`).
    pub session: Option<Value>,
    /// Entries in time order.
    pub entries: Vec<Entry>,
}

/// One scripted entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Offset from the start of a cycle.
    pub at: Duration,
    /// What happens.
    pub beat: Beat,
}

/// An event delivered, or a connection lifecycle change.
#[derive(Debug, Clone, PartialEq)]
pub enum Beat {
    /// An event with its payload.
    Event {
        /// The ESS event (or the view, for a channel carrying one).
        name: String,
        /// Its payload.
        payload: Value,
    },
    /// A lifecycle state: connecting, loading, live, reconnecting, stale, closed.
    Lifecycle(String),
}

impl Script {
    /// Reads a script file's content.
    pub fn parse(channel: &str, value: &Value) -> Result<Self, String> {
        let entries = value
            .get("events")
            .and_then(Value::as_sequence)
            .ok_or("a script lists `events`")?;
        let mut parsed = Vec::new();
        for entry in entries {
            let at = entry
                .get("at")
                .and_then(Value::as_str)
                .ok_or("every script entry has `at`")?;
            let at = parse_duration(at).ok_or_else(|| format!("`{at}` is not a duration"))?;
            let beat = if let Some(name) = entry.get("event").and_then(Value::as_str) {
                Beat::Event {
                    name: name.to_owned(),
                    payload: entry.get("payload").cloned().unwrap_or(Value::Null),
                }
            } else if let Some(state) = entry.get("lifecycle").and_then(Value::as_str) {
                Beat::Lifecycle(state.to_owned())
            } else {
                return Err("a script entry has `event` or `lifecycle`".into());
            };
            parsed.push(Entry { at, beat });
        }
        parsed.sort_by_key(|entry| entry.at);
        Ok(Self {
            channel: value
                .get("channel")
                .and_then(Value::as_str)
                .unwrap_or(channel)
                .to_owned(),
            looping: value.get("loop").and_then(Value::as_bool).unwrap_or(false),
            session: value.get("session").cloned(),
            entries: parsed,
        })
    }

    fn cycle(&self) -> Duration {
        self.entries.last().map_or(Duration::ZERO, |entry| entry.at)
    }
}

/// Plays scripts up to the clock.
#[derive(Debug, Clone, Default)]
pub struct Player {
    scripts: Vec<(Script, u32, usize)>,
}

/// One entry played, with the virtual time it played at.
#[derive(Debug, Clone, PartialEq)]
pub struct Played {
    /// The channel.
    pub channel: String,
    /// When, on the virtual clock.
    pub at: Duration,
    /// What.
    pub beat: Beat,
    /// The script's session.
    pub session: Option<Value>,
}

impl Player {
    /// A player for these scripts, at time zero with nothing played.
    pub fn new(scripts: Vec<Script>) -> Self {
        Self {
            scripts: scripts.into_iter().map(|script| (script, 0, 0)).collect(),
        }
    }

    /// Every entry due at or before `now` that has not played, in time order.
    pub fn due(&mut self, now: Duration) -> Vec<Played> {
        let mut played = Vec::new();
        for (script, cycle, index) in &mut self.scripts {
            let length = script.cycle();
            while let Some(entry) = script.entries.get(*index) {
                let at = length * *cycle + entry.at;
                if at > now {
                    break;
                }
                played.push(Played {
                    channel: script.channel.clone(),
                    at,
                    beat: entry.beat.clone(),
                    session: script.session.clone(),
                });
                *index += 1;
                if *index == script.entries.len() && script.looping && length > Duration::ZERO {
                    *index = 0;
                    *cycle += 1;
                }
            }
        }
        played.sort_by_key(|played| played.at);
        played
    }
}

/// `500ms`, `12s`, `5m`, `1h`. `None` when the text is not of that form or its value does not fit
/// a `Duration` in seconds.
pub fn parse_duration(text: &str) -> Option<Duration> {
    let text = text.trim();
    let split = text.find(|c: char| !c.is_ascii_digit())?;
    let (number, unit) = text.split_at(split);
    let number: u64 = number.parse().ok()?;
    match unit {
        "ms" => Some(Duration::from_millis(number)),
        "s" => Some(Duration::from_secs(number)),
        "m" => number.checked_mul(60).map(Duration::from_secs),
        "h" => number.checked_mul(3600).map(Duration::from_secs),
        _ => None,
    }
}

/// A channel's connection as the user sees it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChannelState {
    /// The last lifecycle state the script set (`connecting` before anything played).
    pub status: String,
    /// When that state began.
    pub since: Duration,
    /// The last payload delivered.
    pub latest: Option<Value>,
    /// The session the last script entry belonged to.
    pub session: Option<Value>,
    /// When each event (by its full name) last arrived.
    pub arrived: BTreeMap<String, Duration>,
}

/// A channel field the document declares as `true while a <Event> event is younger than
/// <duration>` (the form `Channel.fields` uses for derived signals such as `typing`). `<Event>`
/// matches an event's full name or its last segment. `None` when the declaration is not of
/// that form; the caller then reads the field as a payload key.
pub fn derived_field(declaration: &str, state: &ChannelState, now: Duration) -> Option<Value> {
    let rest = declaration
        .strip_prefix("true while a ")
        .or_else(|| declaration.strip_prefix("true while an "))?;
    let (event, age) = rest.split_once(" event is younger than ")?;
    let age = parse_duration(age.trim())?;
    let event = event.trim();
    let last = state
        .arrived
        .iter()
        .filter(|(name, _)| *name == event || name.rsplit('.').next() == Some(event))
        .map(|(_, at)| *at)
        .max();
    Some(Value::Bool(
        last.is_some_and(|at| now.saturating_sub(at) < age),
    ))
}

impl ChannelState {
    /// The state shown at `now`: `reconnecting` longer than `stale_after` reads as `stale`.
    pub fn effective(&self, now: Duration, stale_after: Option<Duration>) -> &str {
        match stale_after {
            Some(limit)
                if self.status == "reconnecting" && now.saturating_sub(self.since) >= limit =>
            {
                "stale"
            }
            _ if self.status.is_empty() => "connecting",
            _ => &self.status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_looping_script_plays_up_to_the_clock_and_restarts() {
        let value: Value = serde_yaml::from_str(
            "{channel: c, loop: true, events: [{at: 0s, event: e, payload: 1}, {at: 4s, lifecycle: stale}]}",
        )
        .unwrap();
        let mut player = Player::new(vec![Script::parse("c", &value).unwrap()]);
        assert_eq!(player.due(Duration::ZERO).len(), 1);
        assert!(player.due(Duration::from_secs(3)).is_empty());
        let beats = player.due(Duration::from_secs(8));
        let times: Vec<u64> = beats.iter().map(|beat| beat.at.as_secs()).collect();
        assert_eq!(times, [4, 4, 8, 8]);
    }
}
