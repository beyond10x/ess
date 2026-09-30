//! When a `play` step's beat next plays on the fixture scripts' virtual clock.
//!
//! Both renderers play every channel's fixture script on one clock from the moment the
//! application starts; `play` runs that clock to the next moment the named beat plays, so the
//! same test means the same thing in the terminal and in the browser.

use std::time::Duration;

use ess_ui_tui::live::{Beat, Script};

use crate::spec::{scalar, Play, PlayBeat};

/// Where the clock is after moving it from `now` by `by`, or why the move is refused: the clock
/// would pass what a `Duration` holds, or a looping script would play more than
/// [`crate::MAX_CYCLES_PER_ADVANCE`] cycles in this one move. The terminal fails the step with
/// this reason and the Playwright spec marks the test `test.fixme` with it.
pub(crate) fn advance(scripts: &[Script], now: Duration, by: Duration) -> Result<Duration, String> {
    let moved = now.checked_add(by).ok_or_else(|| {
        format!(
            "the clock is at {}ms; advancing {}ms passes the end of time",
            now.as_millis(),
            by.as_millis()
        )
    })?;
    for script in scripts.iter().filter(|script| script.looping) {
        let length = script
            .entries
            .last()
            .map_or(Duration::ZERO, |entry| entry.at);
        if length.is_zero() {
            continue;
        }
        let cycles = by.as_nanos() / length.as_nanos();
        if cycles > crate::MAX_CYCLES_PER_ADVANCE {
            return Err(format!(
                "advancing {}ms plays the looping {} script {cycles} times; one advance plays at \
                 most {} cycles of a script",
                by.as_millis(),
                script.channel,
                crate::MAX_CYCLES_PER_ADVANCE
            ));
        }
    }
    Ok(moved)
}

/// The first time after `now` at which `play`'s beat plays, when any script plays it.
pub(crate) fn next(scripts: &[Script], play: &Play, now: Duration) -> Option<Duration> {
    let mut next: Option<Duration> = None;
    for script in scripts {
        if play
            .channel
            .as_ref()
            .is_some_and(|channel| *channel != script.channel)
        {
            continue;
        }
        // A looping script restarts when its last entry has played (`ess_ui_tui::live::Player`).
        let length = script
            .entries
            .last()
            .map_or(Duration::ZERO, |entry| entry.at);
        for entry in &script.entries {
            if !matches(play, &entry.beat) {
                continue;
            }
            let at = if entry.at > now {
                Some(entry.at)
            } else if script.looping && length > Duration::ZERO {
                let behind = now.saturating_sub(entry.at).as_nanos() / length.as_nanos() + 1;
                u32::try_from(behind)
                    .ok()
                    .and_then(|cycles| length.checked_mul(cycles))
                    .and_then(|start| start.checked_add(entry.at))
            } else {
                None
            };
            if let Some(at) = at {
                next = Some(next.map_or(at, |next| next.min(at)));
            }
        }
    }
    next
}

fn matches(play: &Play, beat: &Beat) -> bool {
    match (&play.beat, beat) {
        (PlayBeat::Event(wanted), Beat::Event { name, payload }) => {
            wanted == name
                && play.with.iter().all(|(field, value)| {
                    payload.get(field.as_str()).map(scalar) == Some(scalar(value))
                })
        }
        (PlayBeat::Lifecycle(wanted), Beat::Lifecycle(state)) => {
            play.with.is_empty() && wanted == state
        }
        _ => false,
    }
}
