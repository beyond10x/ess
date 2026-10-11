---
format: aep.planning-md/3
id: story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock
kind: story
status: draft
title: mutate --target interpreted runs synthesized scenarios on the scenario clock run uses
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess verify conform mutate --target interpreted` runs its synthesized scenarios with the same
scenario clock `ess verify conform run --target interpreted` uses, so a scenario that `run` passes
on a `now` guard executes in the audit, and a guard mutant on that guard is scored by it.

## Evidence

Found while correcting `mutate --scenarios` (wave 2026-10-11j). On a model whose command guards on
`exp < now - 10m`, `run --target interpreted` passes all three synthesized scenarios, while
`mutate --target interpreted` reports two of them unsupported ("not scored"), and the
guard-boundary mutant on that guard comes out `survived` although nothing that ran tested the
guard. Authored scenarios in `mutate` already run on `now_offset::scenario_clock_runner`; the
synthesized half does not, because the library runs a caller-supplied target and cannot hand it the
runner's clock. Present since the interpreted target joined the audit
(https://github.com/beyond10x/ess/issues/510).

## Acceptance

- On a `now`-guarded model, every synthesized scenario `run --target interpreted` passes executes
  in `mutate --target interpreted`, for the baseline and every mutant.
- A guard mutant on a `now` guard is killed by the scenario that pins it, not reported `survived`.
- Models with no `now` guard keep their audit report and manifest bytes; any byte change on a
  clocked model is listed in the CHANGELOG.
