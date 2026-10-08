---
format: aep.planning-md/3
id: decision-blocker:boolean-binding-and-store-only-outcome-format
kind: decision-blocker
status: cleared
title: Which source format carries Boolean binding conditions and store-only outcomes?
relations:
- blocks: story:binding-conditions-compare-boolean-event-fields
- blocks: story:an-outcome-that-only-stores-is-observed-through-a-view
- serves: vision:O2
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-07T23:38:48Z", actor: "human:timo", revision: 3}
---
## Question

Gaps 1 and 2 of https://github.com/beyond10x/ess/issues/492 (a Boolean event field in a binding
condition or selection; an outcome that only stores and is observed through a view) each need a new
source format: the fit reviews on `story:binding-conditions-compare-boolean-event-fields` and
`story:an-outcome-that-only-stores-is-observed-through-a-view` show that no `ess/23` spelling keeps
the binding's claim, that a `creates:` takes its identity from an emitted event today, and that
existing `defined()` reads over a Boolean keep their canonical bytes only if the new IR leaf is minted
from a new format. `release-plan:ess-24-one-language` already holds an unscheduled `ess/24`.

| option | what it does | cost |
|---|---|---|
| (a) carry both in `ess/24` | both stories move to `release-plan:ess-24-one-language`; 0.57.0 ships gaps 3 and 4 only; the issue is answered with the plan and closed | no format bump in 0.57.0; the two gaps wait for `ess/24` |
| (b) start `ess/24` in 0.57.0 with these two | 0.57.0 bumps the source format for two constructs | a format bump that the larger `ess/24` scope then extends or bumps again |
| (c) `ess/23` additive | accept the new keys in `ess/23` | breaks this repository's format rule for a change of meaning |

## Decision

Option (a), 2026-10-07: both stories are carried by `release-plan:ess-24-one-language` with the
rest of `ess/24`. ESS 0.57.0 ships gap 3 and answers gap 4 with the idiom; the issue is answered
with this plan and closed.
