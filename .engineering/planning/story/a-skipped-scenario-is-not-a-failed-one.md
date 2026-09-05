---
format: aep.planning-md/1
id: story:a-skipped-scenario-is-not-a-failed-one
kind: story
status: draft
title: A conformance report counts skipped scenarios as failed
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-conformance-format-design
- depends_on: story:review-report-reader-validation
scope:
- confidence: inferred
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: docs/design
- confidence: inferred
  path: website/docs
- confidence: inferred
  path: website/docs/guides/verify-conformance.md
revision: 8
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

## Historical requested checks

- `scenarios_failed` counts scenarios whose status is `failed`.
- The report carries `scenarios_skipped` beside it, so the split is readable without parsing
  `failed_scenarios`.
- `status` is unchanged: `passed`, `failed`, `inconclusive` already distinguish the three cases and
  that part was right.
- A test that a run with a skip and no failure reports `scenarios_failed: 0`.
- The Rust runner is checked for the same defect rather than assumed clean.

## Scope

Derived 2026-09-05 by `aep-drive:story-scoper` from the story and current tree — cited.

- **Primary surface:** `crates/verify/ess-conformance` — cited; the standalone report type, Rust producer and emitted Go producer own the count semantics being migrated.
- **Rust boundary:** `src/evidence.rs:11`, `src/evidence.rs:19`, `src/evidence.rs:59` within the primary surface — cited; `STANDALONE_REPORT_FORMAT`, `StandaloneConformanceReport` and `ConformanceReport::standalone` currently publish v1, with `scenarios_failed` counting every non-pass.
- **Go boundary:** `src/go/runtime.go:581`, `src/go/runtime.go:606`, `src/go/runtime.go:627` within the primary surface — cited; `report` and `writeReport` publish skips in both `ScenariosFailed` and `FailedScenarios`.
- **Status semantics:** `src/report.rs:46`, `src/report.rs:556`, `src/evidence.rs:82`, `src/go/runtime.go:619` within the primary surface — cited; Rust distinguishes failed, error and unsupported, while Go distinguishes failed and skipped; this migration must preserve their existing aggregate verdicts.
- **Reader and compatibility tests:** package-local tests beside `src/evidence.rs:180` — inferred; introduce version-specific accounting and old/new reader fixtures while preserving the meaning and canonical bytes of valid v1 reports.
- **Additional crate:** `crates/edge/ess-cli` — inferred; update the v1-specific report-output help at `src/main.rs:473` and extend emitted-Go report integration coverage in `tests/go_conformance.rs:109`, including a skip-only report.
- **Binding design:** `docs/design` — inferred; the versioned count contract and compatibility behavior need a binding design under the location required by `AGENTS.md:107`; the exact document remains to be selected.
- **Public documentation:** `website/docs/guides/verify-conformance.md` — inferred; its line 33 currently describes report output as v1 and must reflect the selected migration behavior.
- **Version consequence:** a separately versioned report contract is required — cited; the acceptance changes count meaning and adds a persisted field, which falls under `AGENTS.md:47` and the ESS specification skill's format-change rule.
- **Downstream boundary:** AEP adaptation is separate coordinated work — cited; the story names its conformance principle, and the inspected AEP adapter contains an independent strict v1 wire transcription.
- **Confidence:** high for the primary crate, medium for the complete migration footprint — inferred; producer sites are established, while the design document, compatibility policy and downstream delivery order remain unresolved.
- **Would collide with:** units recording `crates/verify/ess-conformance`, `crates/edge/ess-cli`, `docs/design` or `website/docs/guides/verify-conformance.md` — inferred; crate tokens include their package-local tests and fixtures.

## Acceptance

A skip-only execution emits a versioned report with zero actual failures and explicit non-pass category accounting while preserving its inconclusive execution verdict.

## Remediation ownership and compatibility

Owns the F03 count split, not exact-suite coverage binding. The conformance-format design must settle Rust error/unsupported versus Go skipped mapping, list semantics and old/current reader behavior before this implementation. Valid v1 bytes and meanings stay frozen; a new count meaning is not retrofitted to v1. Downstream AEP migration requires separate governed work and an Atlas ADR before enabling a new default writer. The narrow v1 reader story lands first. For computed collisions, the guide's parent `website/docs` token is also recorded, inferred, matching the public-doc stories' chosen granularity.