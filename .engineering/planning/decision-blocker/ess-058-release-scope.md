---
format: aep.planning-md/3
id: decision-blocker:ess-058-release-scope
kind: decision-blocker
status: open
title: 'Release 0.58.0 from main now, or after the #521 and caller-attributes wave'
relations:
- blocks: release-plan:ess-058
revision: 1
---
## Question

0.58.0 is planned to carry https://github.com/beyond10x/ess/issues/521 (`story:related-guard-on-a-captured-struct-input-member`) and `story:authored-act-states-caller-attributes`. Neither is implemented on `main` at 7ac8f4ef93; what is on `main` since 0.57.0 is https://github.com/beyond10x/ess/issues/500 (https://github.com/beyond10x/ess/pull/523) and waves 3-5 of `epic:one-selection-plan` (https://github.com/beyond10x/ess/pull/522). Release now, or deliver the two stories first?

## Options

- **A.** Release 0.58.0 from `main` now, after the full pre-release suite; the two stories form the next wave and ship in 0.59.0. Cost: the caller-attributes adopter waits one more suite run.
- **B.** Deliver the two stories as one wave, then the suite and 0.58.0. Cost: #500 and the selection plan stay unreleased until that wave lands (two serial builds, then the suite).

## Recommendation

A: the fixes on `main` ship on the daily cadence, and the wave's builds cannot overlap the suite anyway.
