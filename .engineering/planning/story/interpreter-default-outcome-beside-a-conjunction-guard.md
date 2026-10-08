---
format: aep.planning-md/3
id: story:interpreter-default-outcome-beside-a-conjunction-guard
kind: story
status: active
title: The interpreted target passes the default outcome when a sibling guard is a conjunction
refs:
- provider: github
  reference: beyond10x/ess#471
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T11:19:51Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-07T11:19:52Z", actor: "human:timo", revision: 3}
---
## Outcome

The built-in `interpreted` target passes the default outcome's scenario when a sibling outcome is
guarded by a two-condition `all:` (or `any:`), and a scenario it cannot run reports `unsupported`
with a diagnostic instead of a bare `error` (https://github.com/beyond10x/ess/issues/471). Reported
on 0.53.0; reproduce on the newest release first.

## Acceptance

- The issue's `shop.orders` reproducer passes 2 of 2 against `interpreted`, for `all:`, `any:` and
  `tier: [Gold, Platinum]`; `ess verify conform mutate --target interpreted` runs on it.
- An `error` outcome in the report carries a diagnostic naming the cause.
