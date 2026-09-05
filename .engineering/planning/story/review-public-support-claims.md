---
format: aep.planning-md/1
id: story:review-public-support-claims
kind: story
status: draft
title: Keep public support claims aligned with shipped evidence
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-xtask
- confidence: cited
  path: website/docs
revision: 6
---
## Finding and source

F16 (P1) from `docs/reviews/2026-09-05-architecture-review.md:522`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `website/docs/status/where-this-stands.md:8`, `crates/edge/ess-cli/src/main.rs:2088`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The public status page agrees with the source-and-release evidence matrix defined in this story.

## Implementation boundary

Correct source discrepancies and connect maintained support/format tables to owned metadata where practical. Distinguish workspace version from a remotely published release; only call a release current after reading its release record. Add a narrow deterministic drift check for maintained source claims without making the offline gate depend on live network.

## Validation

Run the relevant source consistency check and site-build; verify cited release records separately. Publishing later follows repository-commit publication, Website source-lock refresh, Atlas snapshot and delivery gates.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Do not copy internal review/planning records into the public allowlist or claim the currently live website was audited by the source review.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `website/docs` — cited; owning implementation or documented surface.
- `crates/edge/ess-xtask` — inferred; planned edit surface, verify before dispatch.
- `Taskfile.yml` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.


## Source-and-release evidence matrix

Required rows are site output kind (HTML, source dispatch), the current release claim (exact remotely observed release record), workspace version (Cargo metadata, separately labeled), and every changed support-table entry (the owned support test or explicit refusal). Compare the rendered status text with each cited value; any mismatch fails the acceptance. Offline drift checks use maintained source records, while release publication is verified separately.