---
format: aep.planning-md/3
id: story:a-terminal-metric-shows-a-failed-read
kind: story
status: draft
title: A terminal metric shows a failed or loading read
summary: The TUI metric draws a null value for a failed read while React draws the status (turn 1 metric adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-metric-adv-1
revision: 1
---
## Need

Turn 1 of the zero-open-issues plan made the generated React metric draw an alert naming the
status of a failed read and a busy placeholder before the first answer
(story:metric-error-state-on-failed-read). Its adversary pass (review-result:zi-w1-metric-adv-1)
found that the terminal renderer has no such states: `crates/ui/ess-ui-tui/src/view.rs:2040-2044`
draws the null value when `reader_result` is None, so a metric over a view that answers 404, 500
or 501 shows nothing distinguishable from an empty answer, and the two renderers disagree.

## Acceptance

- A terminal metric whose read failed draws the read's status (or `Error` when none is known)
  in the failed style, and a metric still waiting for its first answer draws a loading mark.
- An answered, empty view still draws `0`.
- A test drives the terminal metric through a failed and a pending read.

## Scope

- `crates/ui/ess-ui-tui/src/view.rs` (cited by the review)
