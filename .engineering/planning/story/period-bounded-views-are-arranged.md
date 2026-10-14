---
format: aep.planning-md/3
id: story:period-bounded-views-are-arranged
kind: story
status: active
title: Synthesis arranges views bounded by period parameters, and an undecided view drops no other scenario
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#568
- provider: github
  reference: beyond10x/ess#569
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T14:33:20Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-14T14:33:20Z", actor: "human:timo", revision: 3}
---
## Outcome

Synthesis arranges views bounded by period parameters: a range filter (`started_at >= param.from`,
`started_at < param.to`) and an overlap filter (`started_at < param.to`, `ended_at > param.from`), on
list and aggregate views, binding the parameters around the values the scenario stored (rows before,
across, inside and after the window). A view whose filter is undecided never drops another
command's scenario.

Fits none of the four generic language features: it is synthesis coverage of the `from`/`to`
idiom ESS itself recommends (`docs/design/read-api-view-idioms.md:198-232`).

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/568 and
https://github.com/beyond10x/ess/issues/569 (a downstream incident-ledger specification, ess 0.59.0,
`ess/24`).

1. **Need.** (a) An aggregate view counting incidents that overlap a period is refused,
   `ESS-SYNTH-017 "… the parameter from is read other than by one top-level field == param.from
   conjunct …"` (`crates/verify/ess-conformance/src/synthesize/aggregate.rs:1639`). (b) A list view
   bounded by `from`/`to` makes synthesis drop three scenarios of **other** commands,
   `ESS-SYNTH-005 "… is undecided for an entity in Resolved; param.from is bound by nothing a scenario
   knows"` (`crates/verify/ess-conformance/src/synthesize.rs:1423-1435`).
2. **Class.** Defect: the decision on https://github.com/beyond10x/ess/issues/439 built a range
   selector for aggregate synthesis (`story:feature-request-439`, Decisions, "Built"); the overlap
   shape over two fields and the list-view case are not covered, and (b) costs scenarios unrelated
   to the view.
3. **Already expressible?** Yes in the language; not witnessed.
4. **Fit.** Extends the #439 range selector to (i) two fields bounded by two parameters, (ii)
   non-aggregate views, and (iii) "skip the view's observation, keep the command's scenario" when a
   parameter cannot be bound. Related: `story:feature-request-428` (parameter binding by name),
   `story:a-list-parameter-selects-rows-outside-a-group-key`.
5. **Second adopter.** Bookings overlapping a requested stay; the SLA report of
   https://github.com/beyond10x/ess/issues/439.
6. **Cost.** Synthesis only; no format change.
7. **Alternatives.** (a) Change nothing. (b) Authored scenarios: the reporter's figures stay
   hand-arranged. (c) Extend the range selector (chosen).

## Decisions

Accept as a synthesis defect.

## Acceptance

- `overlap_filter_on_an_aggregate_view_is_arranged`: rows before, across both bounds, inside and
  after; only the overlapping ones count.
- `range_bounded_list_view_is_arranged`: the list view gets scenarios with the bound either side of
  the stored instant.
- `an_undecided_view_keeps_other_commands_scenarios`: the three dropped scenarios return; the view's
  own observation is skipped with a note.
