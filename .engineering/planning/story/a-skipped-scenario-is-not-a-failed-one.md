---
format: aep.planning-md/1
id: story:a-skipped-scenario-is-not-a-failed-one
kind: story
status: draft
title: A conformance report counts skipped scenarios as failed
revision: 1
---
## What is wrong

`ess-conformance-report/1` carries `scenarios_failed`, and the Go runner counts **skipped**
scenarios in it.

`crates/ess-conformance/src/go/runtime.go:465-476`:

```go
failed := make([]string, 0)
for _, result := range results {
    switch result.status {
    case statusPassed:
        continue
    case statusFailed:
        anyFailed = true
    case statusSkipped:
        anySkipped = true
    }
    failed = append(failed, result.status+" "+result.id)
}
```

Both non-passing statuses land in `failed`, and `ScenariosFailed: len(failed)` publishes the count
under a name that says one of them.

Measured on `sbf/acd`, 2026-09-03, ess 0.11.0:

```json
{"status": "inconclusive", "scenarios_total": 24, "scenarios_failed": 9}
```

Nothing failed. Nine scenarios were skipped, all of them `acd.routing.DispatchCall`, which that
implementation exposes no surface for.

## Why it is worth fixing rather than documenting

`failed_scenarios` — the **list** — is honest: every entry is prefixed with its own status, so a
reader who opens it sees `skipped …`. The count is what consumers read, and a consumer cannot
recover the split from it.

AEP's `ess-conformance` principle
(`principles/verification/ess-conformance.yaml:28`) asserts:

```yaml
predicates:
  - ess_conformance.passed
  - ess_conformance.scenarios.failed == 0
```

The second reads this number. As it stands, a suite with any skip can never satisfy it — which may
even be the wanted rule, but it is being enforced by the wrong fact. `ess_conformance.passed` is
already false for an `inconclusive` run and already carries that claim; `scenarios.failed` is
supposed to carry a different one and does not.

Recorded downstream in `sbf/acd` at
`.engineering/planning/executable-system-specification/acd-v3.md`, where the consequence is that
the artifact cannot honestly be moved to `conforming` and the reason is two-thirds wording.

## Acceptance

- `scenarios_failed` counts scenarios whose status is `failed`.
- The report carries `scenarios_skipped` beside it, so the split is readable without parsing
  `failed_scenarios`.
- `status` is unchanged: `passed`, `failed`, `inconclusive` already distinguish the three cases and
  that part was right.
- A test that a run with a skip and no failure reports `scenarios_failed: 0`.
- The Rust runner is checked for the same defect rather than assumed clean.
