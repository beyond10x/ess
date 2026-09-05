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
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: website/docs
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 11
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

Derived 2026-09-05 by independent story-scoper from revision 8 at coordinator f353eaffcddcb923d047666cad5893f38c90a4d7, final binding design (75 rows and P1–P10), implemented dependency stories and actual source. No builds or compatibility execution ran. Coordinator condensed the returned scope below; the binding design remains authority.

- **Primary surface — cited:** crates/verify/ess-conformance. src/evidence.rs:12,19,44,139 owns the v1 discriminator, strict reader and Rust producer. Add separate report/2 admission/production, bound producer profiles, five outcome partitions, checked u64 counts, SuiteReference, execution/conformance status and unknown legacy coverage. Keep v1 meanings and bytes frozen.
- **Rust aggregation — cited:** src/report.rs:46,62,535,556,583 owns scenario precedence, detailed ConformanceReport, whole-run verdict and counts. Preserve failed > error > unsupported > passed inside a scenario; across scenarios failed or unsupported dominates error. Separate unsupported/error scenarios can produce failed=0 with a failed execution verdict. Rust skipped stays zero. Detailed run/2 is a separate envelope; legacy serialization remains.
- **Smallest stage — inferred:** opt-in report/2 and run/2 against admitted legacy suites 1–4, coverage exactly unknown. An execution pass is still inconclusive conformance. Preserve suite/4, report/1 and diagnostic defaults. Suite/5 builders, source/refusal inventory, filters and qualification remain review-conformance-coverage, which depends on this story.
- **Exact legacy-suite admission — cited:** src/scenario.rs:190,204,308,374,423 and src/runner.rs:286 currently parse/execute typed suites without supported-membership enforcement at execution. Retain original bytes for sha256-json-bytes/1 and exact selected outcome-ID membership. In-memory input serializes once, admits/hashes those bytes and executes the admitted value. Version/pairing admission precedes target identity and callbacks. Report/1's allowlist stays explicitly 1–4 regardless of future general support helpers.
- **Go generator — cited:** src/go/mod.rs:49 embeds runtime.go and suite.json; this belongs to ess-conformance, not ess-gen. runtime.go:499,541,581,600 owns execution, terminal collection and report writing. Add ESS_REPORT_FORMAT dispatch and bound v2 output, retaining the v1 writer. Ordinary Go errors remain failed, unsupported observations skip, and EndScenario errors override skips at :686–704.
- **Missing Go execution — cited:** runtime.go:541–557 initializes passed outside t.Run and appends a result even when a host filter omits the subtest callback. V2 must distinguish invoked terminal scenarios from unexecuted ones and refuse complete publication when selected outcomes are missing, including when no report destination was requested. Do not turn count correction into a false pass.
- **Exact timestamps — cited:** ess-primitives/src/time.rs:25–58 is Timestamp(u64), report.rs:541 uses it; legacy Go runtime.go:591,637 uses int64/UnixMilli. New report/run tokens are exact unsigned decimal u64. Preserve 9007199254740993, i64::MAX+1 and u64::MAX; refuse negative, overflow, fraction, exponent, signed and quoted tokens. New Go fields are uint64 with checked nonnegative clock conversion. Keep legacy producer domains frozen.
- **Arithmetic boundary — inferred:** put new wire admission/conversion inside ess-conformance, with no shared Timestamp change. The scoper identified saturating runner arithmetic at src/runner.rs:153,353–367 for inspection. Coordinator clarification: design M75 governs new reader/adaptation arithmetic; do not change legacy virtual-clock execution merely because its timestamps can reach a new report. Every arithmetic operation introduced by v2 must use checked failure rather than saturation, wrapping or narrowing.
- **CLI — cited:** crates/edge/ess-cli/src/main.rs:460–480,2444–2498 owns acquisition, target invocation, detailed JSON/YAML, --report-out and exits. Add --report-format 1|2, strict/allow-incomplete controls and stage-appropriate pairing before execution. Report/2 detailed output is run/2; --report-out stays standalone JSON. No automatic promotion, strictness downgrade or v5-to-v1 conversion.
- **Tests — cited:** ess-cli/tests/go_conformance.rs:57,109–146 actually invokes generated Go and reads reports through Rust. Cover skip-only, ordinary error, teardown override, omitted subtests, invalid configuration and negative clock, plus exact integer/partition/paired-ID and detailed-versus-standalone admission and legacy bytes in package tests. Test applicable legacy P1–P4 and configuration refusals; suite/5 remains unsupported until coverage admission exists, preserving P5's explicit report/1 prohibition. Do not claim all 75 rows or P1–P10 executed by this stage.
- **Public guide — cited:** website/docs/guides/verify-conformance.md:33 currently describes report/1. Explain explicit opt-in, separate run/2, legacy unknown coverage and unchanged defaults without announcing suite/5 or completed downstream rollout.
- **Parent collision token — inferred:** retain website/docs to match existing collision granularity.
- **Root lock — inferred:** Cargo.lock likely changes because ess-conformance needs the existing locked SHA-256 dependency directly for original suite bytes; ess-gen's model-slice hashing is a different profile. Package manifest falls under the crate scope. Coordinator owns any actual root lock update.
- **Design input — cited:** docs/design/review-conformance-coverage.md already binds shapes, semantics, timestamps, hashing, strictness and rollout. Remove obsolete broad docs/design edit scope; no design revision is scheduled here.
- **Coverage overlap — cited:** review-conformance-coverage shares evidence/report/scenario/runner/Go/CLI sources and must follow this stage. New shared report types, exact legacy bytes, ID membership and configuration checks must land coherently. Suite/5 inventory/refusal multiplicity/filtering/qualification, browser and impact remain its work. These are not parallel disjoint units.
- **Actual AEP boundary — cited:** local published object 00c742e4179593738a2e8aa69e2ecc07d3c89402 has independent readers in crates/observe/aep-ess-evidence/src/lib.rs:15–32,138–167 and crates/edge/aep-cli/src/planning.rs:5907–5985; crates/govern/aep-domain/src/evidence.rs:1006–1051,1862–1889 owns closed results/facts. Adapter Timestamp is u64 but planning recorded_from_report uses Value::as_i64. Separate governed domain/reader/policy migration must preserve exact wire values and explicitly refuse narrower adaptation, never call a valid upper-u64 wire timestamp missing or wrap it.
- **SDK distinction — cited:** object 48833c6d14ec37cb3b614fca05cf7dd78f63b743 pins six ESS source APIs at d1a6677, but has no discovered ess-conformance dependency or report/run parser. service-builder/src/ess.rs:102 compiles source; service-conformance/src/lib.rs:25,84 owns the separate service-conformance-report/1. Its type name alone does not establish an ESS report migration.
- **Rollout — cited:** the two dependency stories are implemented; Atlas inventory/ADR, readers before opt-in writers, separately governed AEP domain/readers/policy, regenerated adopter runtimes and defaults last remain binding. This scoping establishes no deployed versions or compatibility. Source objects were inspected without refreshing remote claims in this pass.
- **Excluded — cited:** no ess-gen, ess-primitives, ess-realization, ess-diff, browser/player or SDK edit is established for the count stage. Source/whole-contract digest contracts stay frozen.
- **Confidence — cited:** high for actual producer/reader/generator/CLI/guide/downstream boundaries; internal admitted-byte API and root lock remain inferred.
- **Collisions — cited/inferred separated:** cited packages ess-conformance, ess-cli and exact guide; inferred root Cargo.lock and website/docs parent token. Planning, Atlas work, publication and lifecycle stay coordinator-owned.

## Acceptance

A skip-only execution emits a versioned report with zero actual failures and explicit non-pass category accounting while preserving its inconclusive execution verdict.

## Remediation ownership and compatibility

Owns the F03 count split, not exact-suite coverage binding. The conformance-format design must settle Rust error/unsupported versus Go skipped mapping, list semantics and old/current reader behavior before this implementation. Valid v1 bytes and meanings stay frozen; a new count meaning is not retrofitted to v1. Downstream AEP migration requires separate governed work and an Atlas ADR before enabling a new default writer. The narrow v1 reader story lands first. For computed collisions, the guide's parent `website/docs` token is also recorded, inferred, matching the public-doc stories' chosen granularity.
