---
format: aep.planning-md/3
id: story:a-duration-renders-as-hours-minutes-seconds
kind: story
status: active
title: A duration renders as hours, minutes and seconds
summary: 'as: duration takes unit ms or s (default ms) and renders h:mm:ss with unpadded hours (G-1 answered; adopter report)'
tags:
- ess-w1-0870
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T10:58:23Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-04T10:58:24Z", actor: "human:timo", revision: 3}
---
## Outcome

A field declared `as: duration` renders as `h:mm:ss`, from milliseconds or seconds as the field says.

## Context

Today the React runtime prints a duration's raw number with an `s` suffix. Adopter reports adopter report ask
for a readable form, and reconcile question G-1 is answered (see approval-record:ess-0870-wave, "Answer to G-1:
duration form").

## Acceptance

Named tests; each runs and passes, and each fails before the change.

- `as: duration` accepts an optional `unit: ms | s`, default `ms`, in the ess-ui/1 model and schema; any other
  unit is refused by the model with its path.
- The runtime (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`) renders `h:mm:ss` with the hours
  unpadded:
  - 73000 with the default unit renders `0:01:13`;
  - 73 with `unit: s` renders `0:01:13`;
  - 26 hours renders `26:00:00`, never a day count.
- The terminal renderer shows the same form for the same field.

## Out of scope

- `as: date` locale and format options (G-2).
- Metric `format: duration`.

## Schedule

Ride-along of the 0.87.0 wave, after U3 merges (it shares fields.tsx.tmpl and the field model). The CHANGELOG cites
adopter report and says G-1 is answered.
