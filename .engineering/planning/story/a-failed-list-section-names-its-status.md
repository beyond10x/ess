---
format: aep.planning-md/3
id: story:a-failed-list-section-names-its-status
kind: story
status: draft
title: A failed list section names its status
summary: A list or table section whose read failed says 'Could not load.' without the HTTP status (turn 1 metric adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-metric-adv-1
revision: 1
---
## Need

The generated React runtime now carries a failed read's HTTP status (`ReadState.errorStatus`,
turn 1, story:metric-error-state-on-failed-read) and the metric draws it. A list or table section
over the same failed read still says "Could not load." with no status
(`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:448`), found by
review-result:zi-w1-metric-adv-1.

## Acceptance

- A list or table section whose read failed names the status the read carries (403, 404, 500,
  501), falling back to the present text when none is known.
- A test pins the failed section for a view without a fixture (404).

## Scope

- `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (cited by the review)
