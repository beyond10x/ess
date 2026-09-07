---
format: aep.planning-md/1
id: story:review-authored-discovery
kind: story
status: implemented
title: Define predictable discovery for co-located ESS documents
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:scenarios-directory-compiles-nothing
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: docs/design/models/authored-discovery/domains/discovery.yaml
- confidence: cited
  path: docs/design/models/authored-discovery/system.yaml
- confidence: cited
  path: docs/design/review-authored-discovery.md
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 14
---
## Finding and source

F10 (P1) from `docs/reviews/2026-09-05-architecture-review.md:365`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/load.rs:25`, `crates/edge/ess-cli/src/main.rs:2458`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A documented mixed source/scenario/generated-output layout resolves the same intended authored inputs deterministically without ingesting generated YAML as source.

## Implementation boundary

Specify an explicit input manifest or uniform typed discovery contract before implementation; preserve supported existing layouts or supply actionable migration refusals. Resolve recursion, exclusions, duplicate documents and unknown kinds consistently. Retain the existing zero-authored explicit-path refusal when no scenario document is selected.

## Validation

Fixture the old layouts and a mixed tree with nested authored scenarios and generated YAML; compare resolved input identities over filesystem ordering changes and assert no silent empty success.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

The existing scenarios-directory-compiles-nothing story owns the immediate zero-result refusal and remains a prerequisite.

## Scope

Confirmed from the implementor's complete report, section 1, at base
`b20f0da03986c00bdc79ab1397b4ad44a064f7ed`, with root's independent fourteen-path
source readback in `target/review-boundaries-17/preparation/implementation-source-readback.json`.
The nine original reservations and their cited/inferred provenance remain visible below.

- **Primary acquisition:** `crates/edge/ess-cli` — originally cited; confirmed. The new `src/input_discovery.rs` owns manifest and legacy acquisition. `load.rs`, `coverage.rs` and `main.rs` adapt acquired sources to the existing semantic readers. CLI help, the A1–A24 fixture helper and observed-bindings, release/runtime and normalization caller tests live in this package.
- **Binding:** `docs/design/review-authored-discovery.md` — originally inferred; confirmed. The accepted contract and complete matrix were committed in the opening. Their bytes remained unchanged during implementation.
- **Public model layout:** `website/docs/guides/write-a-specification.md` — originally cited; confirmed. The guide documents the mixed layout, exact manifest selection and retained legacy layouts.
- **Public authored discovery:** `website/docs/guides/verify-conformance.md` — originally cited; confirmed. The guide retains omitted-scenario and direct-file behavior, shallow legacy selection and suite/5 source evidence.
- **CLI reference:** `website/docs/reference/cli.md` — originally inferred; confirmed. The existing reference documents directory configuration and active roles without adding a flag.
- **Public format reference:** `website/docs/reference/formats.md` — originally inferred; confirmed. The reference documents the closed `ess-inputs/1` acquisition format, compatibility refusal and reader obligations. It does not add a manifest digest to existing persisted envelopes.
- **Engineering format catalog:** `docs/design/review-format-catalog.md` — originally inferred; confirmed. One acquisition-format row was added; existing catalog entries remain.
- **Model header:** `docs/design/models/authored-discovery/system.yaml` — originally inferred; confirmed. The 74-byte named declaration was committed in the opening and remained unchanged during implementation.
- **Model values:** `docs/design/models/authored-discovery/domains/discovery.yaml` — originally inferred; confirmed. The 538-byte manifest value declaration remained unchanged. The A24 fixture checks the actual generated projection and reader agreement and separately demonstrates an expression gap; no independent entity, identity or ownership semantics were introduced.
- **Library and dependency boundary:** originally inferred; confirmed by the complete source delta. No library source, Cargo manifest, lockfile or Taskfile changed in this unit. Existing domain/compiler and conformance types retain semantic authority.
- **Prerequisite:** the implemented `scenarios-directory-compiles-nothing` refusal remains covered by the unchanged legacy tests.
- **Confidence:** high for these actual reservations. The implementor reported no incorrect inferred owner. Root independently verified the complete changed-path set and exact source pins; review and integration outcomes are recorded separately.
- **Would collide with:** another writer in the CLI package, either named public guide, either reference page, the engineering catalog or the exact binding/model paths.

The former proposed scope is retained in the opening commit `b20f0da03986c00bdc79ab1397b4ad44a064f7ed`; this confirmation preserves its nine surfaces and marks which original entries were inferred. It does not widen the unit's write boundary.

## Accepted implementation decision

On 2026-09-07 the coordinator selects the optional immediate-directory ess-inputs.yaml contract
in docs/design/review-authored-discovery.md under the recorded standing implementation approval.
The complete A1–A24 matrix remains required. This updates the former four-reservation Scope to
nine, including the public CLI/formats reference, the engineering format catalog and two named
model files; the former source citations and inference history remain in Git. The current
read-only scoper refresh at 603dd90 confirms the same package-local acquisition owners and adds
observed-bindings and delivery qualification to A19/A21. No new library or dependency owner is
selected. These decisions are coordinator choices informed by actual source, not runtime results.

The retained candidate was already reviewed by review-result:authored-discovery-binding-pass1
with no findings. The unchanged named model now lives at docs/design/models/authored-discovery/
(system.yaml and domains/discovery.yaml). Current-source ESS 0.20.0 validation and compilation
each exited zero; validation printed `discovery v1 — 2 file(s), valid`. Compilation produced
2,192 bytes. The retained validator SHA256 is 94983b7227d8b5c3c69cdb448cf4dfe7a4d8c65b086ae7ce54d1db75c607ebf8.
These checks establish named declaration validity only. Reader behavior, path/filesystem policy,
whole configuration admission and the complete executed acceptance matrix remain owed.

The new wave is docs/plan/2026-09-07-review-boundaries-17.md. Root verified the scoper's 10 current
source pins against exact 603dd90 Git blobs in target/review-boundaries-17/preparation/scoper-root-readback.json.
The previously implemented scenarios-directory-compiles-nothing prerequisite stays intact.
The operator explicitly stopped Atlas follow-up during this continuation; source implementation,
its required source/site checks and incremental publication remain authorized. No release tag,
version bump or live ESS deployment is authorized.

## Implementation verification

The implementor's report is retained at
`target/review-boundaries-17/authored-discovery/report.md` in the managed unit,
SHA256 `60dc069169f24be2009cdf0ecfdafcd7726a5044a01d7b89df7dd2000770fa76`.
The actual package checks progressed from 315 to 340 passing cases across the same
41 native summaries; all 25 added discovery cases executed, with zero failed or ignored.
Strict Clippy and the formatter each returned zero on the same final source manifests.
Root verified all fourteen source pins, 111 seal-file pins and the complete retained native
inventory. The complete A1–A24 mapping includes the current acquisition callers.

The baseline CLI executable was overwritten before an independent native copy was made.
Its exact-hash search found no match; source snapshots, output and process records remain.
The separate frozen old-reader executable is an actual compatibility control, not a replacement
for those missing baseline bytes. All 42 final executables were independently copied and verified.

Independent source review completed with no findings and is recorded in full as
`review-result:authored-discovery-source-pass1`, followed by the recorded no-op outcome.
The reviewer added three cases, then executed all 343 package cases across 42 native summaries,
with zero failures or ignored cases. Strict Clippy and formatting returned zero on identical source.
The report is SHA256 `911515b2b3b5d28d09c9f21fa5cffceea238edc523ad1220e8028fe758bf5ed9`.
Root verified its full seal, all 43 retained executables and the complete native inventory.
One interrupted earlier package attempt remains recorded as incomplete; it contributes no verdict.

## Integrated completion evidence

Unit `6701d88ce66bd63a1ae6307450d0c0589ea8b925` was merged at
`f82fbc67a4c1cfaef920ac94ca1449bf8d947575`. On that exact clean commit all eleven
repository/site gate lanes returned zero. The workspace test lane executed 2,233 passing cases
across 198 native summaries, zero failed or ignored. Site-build exercised the actual WASM/browser
lab and generated the static site. The full gate finished at 2026-09-07T20:15:17.585456Z;
root observed process session 99580 exit zero and independently verified all log hashes, direct
exits, 1,191 unchanged tracked files and retained actual Rust/Go producer executables.

The native gate records remain at
`target/review-boundaries-17/gate-f82fbc67a4c1-attempt1` in the coordinator.
Completion SHA256: `99fcf39445faf2fe8a0c4723b7fc76804dffa2694a4a5cb7e0faa75f0e5db3e4`.
Results SHA256: `915337861a2d16d7a2baaf87fa645d9a1ddd1cd1e31b96cc93b9e2efbd85045d`.
Independent readback: `target/review-boundaries-17/preparation/integration-gate-root-readback.json`,
SHA256 `f4de5a168e2da23b33cbf21e3d89adf7134a72dc72ca4ceda41a508df33d2622`.

These are source implementation and validation results. Publication and managed unit retirement
are recorded in the wave page after their actual results. No versioned release is claimed.
