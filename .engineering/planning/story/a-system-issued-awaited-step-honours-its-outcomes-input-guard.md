---
format: aep.planning-md/3
id: story:a-system-issued-awaited-step-honours-its-outcomes-input-guard
kind: story
status: active
title: An awaited system-issued step honours its outcome input guard
summary: The rewrite to awaited steps drops the outcome input guard, so a correct target fails; reporter repro system-guard, 2 of 7 fail.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/system_issued_synthesis.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:56Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:56Z", actor: "human:timo", revision: 7}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which quarantines 9 scenarios for gaps 1 and 2).

When synthesis rewrites a step the system issues into an awaited step (`crates/verify/ess-conformance/src/synthesize.rs` about :2887), the awaited input gets no constraint from the outcome's input guard: an outcome guarded `when: verdict == Refused` is awaited with an input the guard does not admit, so a correct target fails. Reporter repro `system-guard` (7 scenarios, 2 fail): the run report shows the awaited `RecordCheck` input checked against the wrong branch.

Reproduced from the reporter's synthesized suite and run report; not re-run against a target here.

## Acceptance

- `an_awaited_system_step_carries_its_outcomes_input_guard`: the neutral repro, checked in as a fixture, synthesizes awaited steps whose expected input satisfies the outcome guard, and the reference target passes all 7 scenarios.
- A guard that cannot be expressed as an awaited-input constraint is a typed refusal naming the scenario, not a scenario that fails a correct target.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/system_issued.rs` — cited. The story's "synthesize.rs about :2887" is stale: the rewrite moved into this submodule, called from `synthesize.rs` near :3421.
- **Files:** same file `:263-264`: `claim_of` returns `Claim::Reached` for any branch with no error before it reads the branch's condition, so an accepting branch's input guard is never examined; `:192-199`: the `AwaitOutcome` is written with `input: BTreeMap::new()` — cited.
- **Also likely:** `crates/verify/ess-conformance/tests/system_issued_synthesis.rs` for the acceptance test and a new neutral fixture — inferred; `src/scenario.rs` (the `AwaitOutcome.input` doc says "authored only", comment only) — cited.
- **Documents:** `docs/design/system-issued-commands.md:167` describes the accepting row without a guard; a line may follow — inferred.
- **Confidence:** medium — the code path is cited; the fix shape for non-equality guards is inferred.
- **Would collide with:** any unit editing `claim_of` in `synthesize/system_issued.rs` (notably the draft `story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes`, its input-guard arm at `:277-283`) or `tests/system_issued_synthesis.rs` — cited.
- **Safety fact:** the runners already compare every stated await input (`runner/await_outcome.rs:304-318`, `src/go/await.go:172`) — cited.
- **Not established:** the repro `system-guard` and the test name are not in the tree (new work); accepting `Otherwise` and `ExternalWhen` arms do not reduce to an equality input, so a typed refusal there would withdraw scenarios that pass today — the implementor decides — inferred.
- **Size:** medium, about 90-150 lines; no format version (`ORDINARY = 52`, `src/system_issued.rs:119`, whose `step_keys` already admit an optional `input` on `await_outcome`, `:230-233`) — cited.
- **ess-conformance targets:** `system_issued_synthesis` — inferred.

## Schedule (D401, 2026-11-02)

Left wave W1 (orchestrator D401). It waits on `decision-blocker:system-issued-await-carries-compared-values`
(Decider: operator) and is not waved until that is answered. The SYN1 patch from W1 is parked in the
coordinator scratch, outside the repository.
