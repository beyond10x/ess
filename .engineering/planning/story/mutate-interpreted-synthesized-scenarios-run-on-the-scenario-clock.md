---
format: aep.planning-md/3
id: story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock
kind: story
status: active
title: mutate --target interpreted runs synthesized scenarios on the scenario clock run uses
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/target.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:27Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:27Z", actor: "human:timo", revision: 9}
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

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (mutation audit, `src/mutate.rs`) — cited
- **Files:** `crates/verify/ess-conformance/src/mutate.rs:4655-4660` (`run_checked`: `Runner::for_suite(&suite)` and `new_target()`, no command clock; reached from `baseline_with` at :4722 and `measure` at :4793) — cited
- **Symbols:** `now_offset::scenario_clock_runner` (the authored half already uses it, `mutate.rs:4091-4095`), `Runner::command_clock` (`runner.rs:360`), `Interpreted::with_command_clock` (`interpret.rs:205`) — cited
- **Also likely:** `crates/verify/ess-conformance/src/target.rs` and `src/interpret.rs`, if the clock reaches the synthesized target as a `ConformanceTarget` capability beside `hold_now` (`target.rs:418`) — inferred
- **Also likely:** `crates/edge/ess-cli/src/main.rs:4496-4505`, only if the clock is passed through the caller's closure instead — inferred
- **Documents:** `CHANGELOG.md:871-873` records the gap as shipped behaviour; `docs/design/mutation-audit-and-model-runner.md:82-86` describes only the authored half — cited
- **Confidence:** high for the site; medium for the fix shape (capability vs closure)
- **Would collide with:** any unit touching the audit's run path in `mutate.rs` (`run_checked`, `baseline_with`, `measure`, `AuthoredSuite`), the public `mutate::audit_with` / `audit_with_authored` signatures, and the `ess verify conform mutate` dispatch (`main.rs:4439`, closure at :4496-4505) — inferred
- **Safety fact:** an unclocked `Interpreted` reads no clock (`interpret.rs:257`), so on a model with no `now` guard the switch changes only the clock the runner resolves `now_offset` values against — inferred
- **Not established:** no `mutate --target interpreted` run on a `now`-guarded model was made; whether a verdict change on clocked models needs a new `ess-mutation-report` version.
- **Size:** medium, about 60-100 lines plus one test file; no format version (`ess-mutation-report/7`, `mutate.rs:92`; `ess-mutation-manifest/3`, `mutate.rs:4982`) — inferred
- **Targeted run:** ess-conformance `--test mutation_authored`, `mutation_audit`, `interpreted_step_instant`, `mutation_stored_guards` — inferred
