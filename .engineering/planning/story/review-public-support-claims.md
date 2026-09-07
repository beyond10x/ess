---
format: aep.planning-md/1
id: story:review-public-support-claims
kind: story
status: active
title: Keep public support claims aligned with shipped evidence
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/support.rs
- confidence: inferred
  path: docs/design/review-public-support-claims.md
- confidence: cited
  path: website/docs/concepts/ess.md
- confidence: cited
  path: website/docs/guides/synthesize.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 19
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

Derived 2026-09-07 by `aep-drive:story-scoper` 0.8.0 against ESS ecb7efc22ad9b19b85ef4debd8143491d6a66ef3 — cited.

- Primary surface: public capability documentation and the existing Rust repository maintenance command — cited.
- File: `website/docs/status/where-this-stands.md` — cited; current-release wording, projection output description and maintained source/support matrix.
- File: `website/docs/concepts/ess.md` — cited; obsolete site-output row and prose, plus the incomplete synthesis-target enumeration.
- File: `website/docs/reference/cli.md` — cited; obsolete site-output command row, omitted docs-ir spelling and source/release labels on schema-command rows.
- File: `website/docs/guides/synthesize.md` — cited; three-emitter table omits the existing Clap target and needs its distinct obligations/dependency boundary.
- File: `website/docs/guides/verify-conformance.md` — cited; distinguish current-source suite/5 instructions from the separately observed 0.20.0 release.
- File: `crates/edge/ess-xtask/src/main.rs` — cited; existing Command/run dispatch, workspace_version, projections and generated-artifact inventory provide the maintenance owner.
- File: `crates/edge/ess-xtask/src/support.rs` — inferred; proposed private Rust support-table renderer/checker and its unit tests.
- File: `Taskfile.yml` — cited; existing offline check and separate site-build/release-status composition.
- File: `docs/design/review-public-support-claims.md` — inferred; proposed binding for the finite maintained claims, their evidence boundaries and drift-check behavior.
- Symbols: `Command`, `run`, `workspace_version`, `projections`, `Generated`, `Projection`, and `ess_gen::generators` — cited.
- Proposed symbol: `support::check` — inferred; no command or implementation currently exists.
- Confidence: high — cited; the story names the public discrepancy, and the actual CLI dispatch, emitters, release receipt and maintenance path resolve its owners.
- Would collide with: edits to any exact file reserved above, especially the shared concept page, CLI reference, xtask dispatch and Taskfile — inferred.
- Read-only evidence owners outside these reservations require no implementation edits for this candidate — inferred.

## Earlier scope (retained history)

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `website/docs` — cited; owning implementation or documented surface.
- `crates/edge/ess-xtask` — inferred; planned edit surface, verify before dispatch.
- `Taskfile.yml` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

## Source-and-release evidence matrix

Required rows are site output kind (HTML, source dispatch), the current release claim (exact remotely observed release record), workspace version (Cargo metadata, separately labeled), and every changed support-table entry (the owned support test or explicit refusal). Compare the rendered status text with each cited value; any mismatch fails the acceptance. Offline drift checks use maintained source records, while release publication is verified separately.

## Accepted implementation boundary

The complete finite matrix and evidence boundaries in `docs/design/review-public-support-claims.md`
are binding for this story. Root accepted candidate v2 after an independent no-findings document
pass and exact input readback; no tests or release binaries were run by that pass. The full
acceptance, changed-row owners, rendered status comparison, separate release verification and
publication sequence above remain mandatory. The five public pages and three maintenance files
are the implementation unit; the binding, shared journal, wave page and changelog are root-owned.
No generic support registry, persisted metadata format, new domain or dependency is introduced.
