---
format: aep.planning-md/3
id: story:feature-request-244
kind: story
status: implemented
title: Guards over elapsed time since a stored instant and over calendar windows
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#244
relations:
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T15:27:21Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-04T15:27:21Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-06T09:45:03Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Guards over elapsed time since a stored instant and over calendar windows (beyond10x/ess#244).

## Status

Feature request, triaged 2026-09-29 as outside the defect batches. Not scheduled; acceptance is written when it is.

## Reconciliation

Backlog reconciliation (coordinator, 2026-10-01).

- Absorbs #233 item 5 (`now >= expires_at` over a stored Timestamp).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-244 (elapsed time since a stored instant; calendar windows)

Tested with `ess 0.44.0`; rules re-read in the 0.48.0 read tree (`gaps-270`, `e3bc9a2ff`).

1. **Need.**
   - (a) A command is refused until a fixed time has passed since an instant stored on the row (promotion only 24h after the previous one).
   - (b) A command is admitted only inside a weekly window in a named zone (Mon–Thu, 08:00–16:00).
   - Repro `repro-244/a-elapsed-subject.yaml`: `when_subject: {predicate: "expires_at > now - 24h"}` -> `ESS-COMMAND-002` "admitted only in a command outcome's `when:`" (0.44.0). (b) has no construct at all.
   - *Requester's syntax:* `now - promoted_at >= 24h` with a Duration, and a window predicate (weekdays, time of day, zone).
2. **Class.**
   - (a) gap: deliberately left out (`docs/design/current-time-guards.md:150-151`).
   - (b) gap: `timestamp-clock-provenance.md:20` infers "no ... timezone database ... or calendar arithmetic", and `current-time-guards.md:149` excludes days and calendars.
3. **Already expressible?** No.
   - `now` only on the right of an ordering over a `Timestamp` input in `when:` (`predicates.md:557-590`).
   - `Duration` has no ordering (`website/docs/guides/specify/fields-and-invariants.md:110`).
   - Copying the stored instant into the input makes the caller the authority (`cross-record-and-stored-field-guards.md:30`).
4. **Fit.**
   - (a) As asked (`now - field`, a Duration on the left) it adds left-side arithmetic and Duration ordering, both new. Rewritten as `promoted_at <= now - 24h` it is exactly family F A3 (`now` in `when_subject`/`when_related`) with today's offset grammar (`current-time-guards.md:27-36`). No new spelling, and it is identical to #233 item 5.
   - Witnessing (a) needs the stored instant arranged from a creator's input as a `now_offset` (`current-time-guards.md:86-92`).
   - A `promoted_at` stamped by the implementation cannot be put 24h in the past without a target clock seam. That case is refused by name, like a structure-held instant today (`current-time-guards.md:108-113`).
   - (b) needs zone rules (IANA data), and the Rust, Go and TypeScript lanes would have to agree on its version. A window is also not an instant comparison, so the boundary witnesses (a second each side) would need DST-aware instants.
5. **Second adopter.**
   - (a) A cool-down: a password reset only 15m after the last request; a refund only within 30 days of capture (needs days; whole hours cover it as `720h`).
   - (b) Change freeze windows, market trading hours.
6. **Cost.**
   - (a): the ess/20 bundle gate; no suite format, because a `now_offset` already exists in `ess-conformance/26`/`27` (`current-time-guards.md:134`). Entity Runtime keeps `CurrentTimeUnsupported`.
   - (b): a new predicate kind, a zone-data dependency in three evaluator lanes, a suite major and an Entity Runtime refusal.
7. **Considered.**
   - (a): change nothing (fields "documented as unchecked", per the issue); the requester's Duration arithmetic; family F A2+A3. A2+A3 is chosen: it adds no keyword.
   - (b): change nothing; a UTC-only window (no zone data, but not the stated rule); a zoned window with pinned zone data. Nobody has decided whether ESS takes a timezone dependency, so (b) waits on that decision.

## Decisions

- **Split (proposed).**
  - (a), elapsed time since a stored instant: **accept, redesigned.** It is family F A3 with A2's offset, written `promoted_at <= now - 24h` rather than the requester's `now - promoted_at >= 24h`. That adds no Duration ordering. An implementation-stamped instant is refused by name in synthesis. This is a **duplicate of #233 item 5** and is merged into that story.
  - (b), calendar windows: **defer.** Raise a `decision-blocker`: "Does ESS take a timezone-data dependency, pinned identically across the Rust, Go and TypeScript evaluators, and what do days and DST mean in a guard?" It blocks this story. `timestamp-clock-provenance.md:20` currently rules zone data out.
  - Not already fixed (tree `e3bc9a2ff`). Overlaps #171 (closed; built `now` in `when:` only).

- Coordinator (2026-10-01): adopted as family F below. Format: ess/21, because ess/20 ships alone in 0.49.0 and family F lands as one bump. Family F (one design across #225, #228, #233, #237, #244): A1 a bare right-hand root names a field, input or binder; A2 one `± constant` offset (Integer or Timestamp); A3 `now` in `when_subject`/`when_related`; A4 `input.<dotted path>` in values; B `when_related: {entity, where, exists | count | forall}` over row sets; C `distinct` over lists and `.utf8_bytes`. Half (b), calendar windows, is deferred: decision-blocker:calendar-window-time-zone.
