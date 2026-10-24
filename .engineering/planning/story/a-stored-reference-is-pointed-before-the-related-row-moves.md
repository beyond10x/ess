---
format: aep.planning-md/3
id: story:a-stored-reference-is-pointed-before-the-related-row-moves
kind: story
status: active
title: A stored reference is pointed at the related row before that row moves into the state the branch reads
summary: Setup arranges the related row in its target state before the subject's creation that names it; where the creator refuses that state, setup fails; 1 of 18 downstream scenarios fails on 0.68.0 and the 0.69.0 build.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T23:40:01Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-22T23:40:01Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Need

A downstream one-domain specification declares `StartMachine/frozen` as `when_related: {via: site_id, predicate: state == Frozen}`, read through the machine's stored `site_id`. `AddMachine`, which creates the machine and sets `site_id` from its input, refuses a frozen site (`frozen`: `when_related: {via: input.site_id, predicate: state == Frozen}`). The state is reachable: add the machine, then freeze the site. The synthesized scenario arranges the site in `Frozen` first and creates the machine in it afterwards, so `AddMachine` answers `frozen` and setup fails.

## Reproduction

Synthesize, then run on the interpreted target with `--report-format 2`: 18 scenarios, 15 passed, 3 failed on 0.67.0 (downstream report), 0.68.0 and the 0.69.0 integration build (commit 52bd110dd). This story is one of the three:

```
StartMachine/outcome/frozen: outcome repro.fleet.AddMachine/added obs=["outcome = frozen"]
```

Steps of the written scenario: `FreezeSite(site-138601)`, then `AddMachine(site_id = site-138601)`.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs:343` (`arranged_at`): in the present-row case `with_row` (`:415`) arranges the related row in the state the branch needs, `relink` (`:214`, called at `:437-443`) points the subject's creation at it, and `:446-448` puts all of the related row's steps ahead of the subject's own arrangement.
- Precedent in the same function: for a missing row, a removal is held back in `after` (`:401`) and runs once the subject exists (beyond10x/ess#509). The present-row case has no such split.

Not the cause of `story:a-row-arranged-beside-is-created-once-under-its-name` or `story:an-existence-outcome-passes-a-related-list-membership-guard`: this is the stored-reference path (ess/22), not an input-addressed related row.

## Acceptance (provisional)

- For the specification above, `StartMachine/outcome/frozen` is synthesized with `AddMachine` before `FreezeSite` and passes on the interpreted target.
- A conformance scenario in this repository holds a minimal specification of this shape (a stored-reference state guard whose state the subject's creator refuses) to that result.
- Where no order reaches the state, the branch is refused by name at synthesis time instead of being written as a failing scenario.
