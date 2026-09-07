---
format: aep.planning-md/1
id: story:review-public-support-claims
kind: story
status: implemented
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
- confidence: cited
  path: crates/edge/ess-xtask/src/support.rs
- confidence: cited
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
revision: 23
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

Confirmed 2026-09-07 by the implementor at source `254db232b785ba3ed6166e9cd5f23e2ad7fe9679`,
with root readback in `target/review-boundaries-14/preparation/unit-handoff/verified.json` — cited.
The full report is SHA256 `20f61ec77772038624a76beb43ebaa8289d95a9584cabe5c0100c60df608ed00`.

- `website/docs/status/where-this-stands.md` — cited; complete maintained source matrix and separately dated release prose.
- `website/docs/concepts/ess.md` — cited; corrected site output and four synthesis targets; glossary prefix preserved.
- `website/docs/reference/cli.md` — cited; explicit projection choices, site output roots and current-source schema labels.
- `website/docs/guides/synthesize.md` — cited; four targets, Clap grammar/handler/dependency boundary and Binary64 refusal.
- `website/docs/guides/verify-conformance.md` — cited; built-in target inventory and current-source coverage versus release boundary.
- `crates/edge/ess-xtask/src/main.rs` — cited; Command/run dispatch and shared offline public CLI/projection-artifact route.
- `crates/edge/ess-xtask/src/support.rs` — cited; confirmed new private module and colocated tests. The proposed `support::check` spelling became `support::run(root, check)`; `compare` owns the complete-block refusal.
- `Taskfile.yml` — cited; offline support-check lane included in the existing check sequence.
- `docs/design/review-public-support-claims.md` — cited; accepted finite binding, root-owned and unchanged by the unit.
- The 46 evidence owners, public allowlist, manifests/lock and planning store outside these reservations were preserved — cited.
- Confidence: high for these exact write owners — cited; independent attack, full integration and publication remain separately required.
- Would collide with any edit to one of these files or a containing directory — inferred scheduling consequence.

## Preimplementation scope (retained history)

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

## Implementor handoff — wave 14

Source commit `254db232b785ba3ed6166e9cd5f23e2ad7fe9679` changed exactly the eight assigned
unit paths. The complete package executed 20 → 29 cases, with zero final failures/ignored cases;
four original behavioral assertions were observed red. Thirteen actual CLI documentation
mutations refused, and original source bytes were restored. Formatting, strict package Clippy,
projection drift, support CLI and support task returned zero. Full direct child output and
48 command receipts are under the unit's `target/review-boundaries-14/public-support/implementation`.
Root verified the complete report, eight source hashes, 46 source-owner hashes, command output
hashes and the native target/TMP census before committing with both bot identities. The checker
compares all 20 maintained rows plus their source-version sentence and complete markers/header;
ordinary release prose and other page corrections account for all 27 material requirements.

The 21 binding validation entries retain distinct package, semantic-owner, rendered-page,
release-readback and publication obligations. This handoff alone does not establish completion.


## Final source reviews — integration pending

Independent source attack 1 recorded one introduced README default-selection discrepancy;
correction `b5908e77a7fa8eafe5f8e387db431c967634cb67` resolved it. The second and final attack
recorded no findings and preserved all seven first-pass tests, adding an actual CLI boundary
test for adjacent README selection, explicit replacement and missing explicit front-page refusal.
Final unit commit is `9dbc7a9d10f89b1a77156a92168924e8410f5672`, with both bot identities.
All 32 unit and five layout cases passed, zero failed or ignored; formatting, strict package
Clippy, support and projection checks passed. The complete second report is immutable
`review-result:public-support-source-pass2`, SHA256
`df319005dddb7bc47c9e363d379a4b0c33192574d596778b401aee3ef4212425`.
Root read its entire 45,206 bytes and verified 39 controls, 88 material inputs, all eight source
hashes, every command output and 2,883 native entries (1,954,150,699 regular bytes).

The second attack's initial launch refused before tests because ancestor Cargo configuration
enabled sccache. Final launches explicitly set both wrapper variables empty; verbose evidence
records 67 direct pinned rustc launches. Earlier wrapper-free and complete outside-write claims
are not established by the original nonverbose logs. Initial launch and fixture/format correction
statuses remain retained. The report's closing no-Git statement means no Git mutations;
documented read-only Git commands did occur. Source findings compare as carried 0, new 0,
resolved 1. Full integration, rendered comparison and public delivery remain required before
implementation completion; package success alone does not satisfy them.


## Completed source and public delivery — wave 14

Source commit `57e242e8a0eaa721968c3970099b4bc561cb91aa` is published on remote main.
The bounded integration correction `2004bb1575d12656521216acce47d74acba49882` replaces
MDX-invalid HTML comment delimiters with passive Markdown reference definitions. Root verified
that all 20 rows, the complete test module and ordinary prose remained unchanged. All 37 package
cases, formatting, strict Clippy, render and support-check passed on that correction. The full
source gate passed all 11 lanes with 2,110 workspace cases, zero failed or ignored. Attempt 4
retains three passing same-source attempt-3 lanes and reruns the interrupted test plus every
previously unrun lane; the resource interruption is preserved, not counted as a successful run.

CI `34094711588`, documentation validation `34094711617` and source bundle `34094711604`
completed successfully for that exact source. Atlas publication `34094924671` succeeded and
produced artifact `10008305282`, ZIP SHA256
`40b9e58d5e361c6aec36bb6db5181cfd945fbd4c58c143c9157f3d0bb04c7ae6`.
Its source set pins ESS at that source, AEP at `658cf76e6371b1628f6de69548e724b52803f5c2`,
Website runtime `fc4571534765c098ed861bc326da4d3da0d1df63` and Atlas control
`d10b7484d64c28830774c9dae0ec531fcc47acb2`.

The independent artifact verifier and complete Website gate returned zero. Website ran 99 tests,
99 passed, zero failed, skipped or cancelled; it verified 357 routes and 1,332 production files.
Every immutable artifact entry remained unchanged. The rendered published status table matches
all 20 source rows and 63 header/data cells, source-version text and the separately dated release
qualifications/link. Its HTML SHA256 is
`aefd20148711d58aaa153fc2fc8d89af4a1b6f1de6fbfcda5d78a42ecdd7f3a1`.
All five changed public pages and both provenance routes returned HTTPS 200 with bytes exactly
equal to the admitted artifact. Final Atlas Pages verification returned zero: 37 repository
states, 26 Pages repositories and 52 routes. The broader Atlas fence still has its three recorded
pre-existing workspace failures; this completion makes no organization-wide green claim.

The separately read release remains public `0.20.0`, record `383642123`, published
2026-09-06T16:07:45Z; metadata SHA256
`2ce2479a525e9ffbffb5778333109e16995d1a323df7fabc89c7365435f449a7`.
Release metadata/asset names were observed; no release binary was executed. No release, tag,
version bump, install, default switch or live executor deployment was performed.

Full gate, render, publication and live receipts remain under coordinator
`target/review-boundaries-14`; Website gate completed at 2026-09-07T07:38:53Z.
The unit's complete target and both owned temporary roots were archived and independently
verified before removal. Website retention and exact managed cleanup are recorded in the wave
page as they finish. All 27 material requirements and 21 validation entries retain their
source, semantic-owner, rendered-page, release and publication evidence boundaries.
