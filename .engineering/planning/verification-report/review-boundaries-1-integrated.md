---
format: aep.planning-md/1
id: verification-report:review-boundaries-1-integrated
kind: verification-report
status: draft
title: Boundary wave 1 integrated verification
relations:
- verifies: story:review-secret-sanitization
- verifies: story:review-report-reader-validation
revision: 1
---
## Subject and source

Integrated source commit `aefdc330089fd9a96baf39179857a84686efa944`, combining unit heads `7130468ce5baa8178407bc02df557f27a7cae7ee` and `6b887663736d088307b4f8957de8488740110e6a`. Only coordinator planning/evidence text changed while the integrated source gates ran; implementation and test bytes stayed at this source commit. Observed 2026-09-05; completion recorded at 10:26 UTC.

The implementation and immutable adversarial reports are linked from `docs/plan/2026-09-05-review-boundaries-1.md`. Original red adversarial patches remain in the engineering evidence. The coordinator routed the baseline-reproduced Secret stderr defect to `story:review-kubectl-diagnostic-sanitization` and characterized its current behavior without an ignore. The YAML case was corrected to distinguish opaque YAML string coercion from JSON null refusal. No production code changed in that routing.

## Package verification after routing

`cargo test --locked -p ess-kubernetes`: 8 executed, 8 passed, 0 failed, 0 ignored; library 2 and integration 6. Package formatter and strict Clippy exited 0.

`cargo test --locked -p ess-conformance`: 242 executed, 242 passed, 0 failed, 0 ignored; lanes 67, 47, 7, 12, 11, 7, 4, 14, 49, 24, and doc tests 0. Package formatter and strict Clippy exited 0 after adding numeric separators to an unchanged test timestamp.

## Integrated offline gate

Each underlying `task check` step was run separately with its own captured exit status. Exact status output:

```text
fmt-check 0
clippy 0
test 0
doc-check 0
example-check 0
projection-check 0
release-check 0
action-check 0
```

Runner summary totals: **1,438 passed, 0 failed, 0 ignored**, versus the 1,419-test unchanged-source baseline. All steps executed. The example-check infrastructure diagnostic fixture emitted its expected diagnostic findings and the task exited 0; this is not a failed gate step. The full logs were inspected for skipped steps, command-not-found failures and test failures; none established. Raw logs and status records remain under the coordinator's `target/review-boundaries-1/integration/`.

## Site build

`task site-build` exited 0 after the WebAssembly/browser-lab checks, pinned dependency installation and Docusaurus build:

```text
[SUCCESS] Generated static files in "build".
```

Npm also reported the same baseline 30 dependency audit findings (9 moderate, 21 high) and two blocked install scripts. Those advisories were not remediated by these two stories and this result is not a dependency security assessment.

## Limits and resources

The two stories validate existing boundaries without a version, field, canonical valid-byte or downstream AEP change. This record does not close the rest of F01–F17, the new kubectl-diagnostic story or the open format-rollout/recovery obligations. No tag, release, external publication or website delivery was performed.

Disk exhaustion interrupted adversarial verification before storage recovered; `resource-blocker:review-wave-storage-capacity` preserves the incident and clearance evidence. The final coordinator target measured 1,863,140 KiB; unit targets 170,956 and 313,324 KiB. Shared free capacity fell again after the completed gates, so the next wave must remeasure resources. No other session's processes or worktrees were modified.