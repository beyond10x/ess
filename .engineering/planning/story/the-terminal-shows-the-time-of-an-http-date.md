---
format: aep.planning-md/3
id: story:the-terminal-shows-the-time-of-an-http-date
kind: story
status: draft
title: The terminal shows the time of day of an HTTP-style date
summary: 'The terminal as: time fallback splits at the first T, so an HTTP date shows empty where React shows 03:04 (turn-2 adversary, pre-existing, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-format-adv-2
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

The terminal renderer shows `as: time` by reading the value as an instant, and when that fails it
falls back to splitting the text at its first `T` (`crates/ui/ess-ui-tui/src/view.rs`, the time
fallback near `fn cell_value`). An HTTP-style date such as `Tue, 03 Nov 2026 03:04:05 GMT` holds a
`T` only in `GMT`, so the terminal shows an empty cell, while React parses it and shows `03:04`.
Found by the turn-2 adversary (`review-result:zi-t2-format-adv-2`, `view.rs:1686`); the fallback is
at the turn's base `7623ee5ec1`, so it is pre-existing.

## Acceptance

- A field `as: time` holding `Tue, 03 Nov 2026 03:04:05 GMT` shows `03:04` in the terminal, as it
  does in React.
- A text that is no instant at all is shown as it is, not cut at a `T`.

## Scope

- `crates/ui/ess-ui-tui/src/view.rs` (the `as: time` fallback)
- `crates/ui/ess-ui-tui/tests/value_formats.rs`
