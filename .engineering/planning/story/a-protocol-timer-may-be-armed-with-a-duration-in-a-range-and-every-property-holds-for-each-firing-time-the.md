---
format: aep.planning-md/3
id: story:a-protocol-timer-may-be-armed-with-a-duration-in-a-range-and-every-property-holds-for-each-firing-time-the
kind: story
status: draft
title: A protocol timer may be armed with a duration in a range, and every property holds for each firing time the range allows
summary: A timer duration is fixed or backoff only, so a jittered or randomised timeout needs one timer per value (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-protocol-timer-takes-its-duration-from-a-message-field
- informed_by: story:exploration-of-a-protocol-with-a-re-arming-timer-reaches-a-verdict-and-a-bounded-response-property-states-that
revision: 2
---
## Need

A timer whose duration is not one number but any value between two: a retry jitter (wait 800 to 1 200
ms), a randomised election timeout. The model must let the scheduler pick any value in the range, and
the properties must hold for every pick.

Today a duration is `Fixed { millis }` or `Backoff { timer, factor, ceiling_ms }`
(`crates/specify/ess-domain/src/protocol.rs:228`); `after: {kind: range, min: 800, max: 1200}` is
refused as an unknown variant. Several fixed timers do not state it: a participant with one transition
per spelling arms a different timer each, and a property over "every pick" then needs one run per
combination.

Reporter's proposal (theirs): `after: {kind: range, min, max}`.

## Outcome

`Duration` gains `Range { min_ms, max_ms }`. Arming a timer with it lets the scheduler fire it at any
deadline from `now + min` to `now + max`; exploration branches over the two ends and over every other
deadline another timer or message makes distinguishable, and a recorded run is accepted when the firing
time lies in the range.

## Acceptance

- `after: {kind: range, min: 800, max: 1200}` validates in `ess specify protocol validate`. Each of
  these is refused with a string code and its path: `min` of 0, `min` above `max`, and `max` above
  `max_time_ms`.
- `ess verify protocol explore` on a retry model with the range reaches the same verdict as the two
  fixed-duration models at 800 and at 1 200 ms for a property both satisfy, and fails a `bounded_response`
  of 1 000 ms with a counterexample trace whose firing time is the range's upper end (needs
  `story:exploration-of-a-protocol-with-a-re-arming-timer-reaches-a-verdict-and-a-bounded-response-property-states-that`; without it, a `never_state` property is used).
- A recorded run that fires the timer at 799 ms or at 1 201 ms fails the monitor; at 800 and 1 200 ms it
  passes.
- Both committed examples keep their digests and verdicts.
- Each test runs within 60 s and each test step within 120 s.

## Scope

- cited: `crates/specify/ess-domain/src/protocol.rs` (`Duration` 228), `crates/specify/ess-compiler/src/protocol.rs`,
  `crates/verify/ess-conformance/src/protocol/engine.rs` (arm, `AdvanceTo` 222-240), `monitor.rs`,
  `docs/design/protocol-specification.md`.
- Size: S, about 250 changed lines. Edits `Duration`, as `story:a-protocol-timer-takes-its-duration-from-a-message-field`
  does; the two go one after the other.

## Fit review

1. **Need.** A timer with a range of durations, checked for every value. Reporter's syntax is labelled
   theirs.
2. **Class: gap.**
3. **Already expressible?** Only as one timer per value, which checks the picks one run at a time and
   not as one model.
4. **Fit.** Same enum as the field-sourced duration of `story:a-protocol-timer-takes-its-duration-from-a-message-field`; the checker's time is already logical
   and strictly advancing, so a range is a branch of the existing `AdvanceTo`.
5. **Second adopter.** A client that backs off with jitter.
6. **Cost.** One variant, about three codes. Format per PR-D3 (additive in `/1`, default A).
7. **Alternatives.** Change nothing (one timer per value); only min and max (loses the in-between
   picks).

## Decisions

AC9-D4: the picks exploration tries. A: the two ends plus every deadline that another event makes distinguishable. B: every integer millisecond (exact, may exhaust `max_states`). C: only the ends. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as redesigned. Format follows PR-D3.
