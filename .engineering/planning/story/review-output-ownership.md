---
format: aep.planning-md/1
id: story:review-output-ownership
kind: story
status: active
title: Make generated output replacement recoverable and ownership-aware
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-output-containment
scope:
- confidence: inferred
  path: .github/workflows/ci.yml
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/profiles.json
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/reviewed-candidates.json
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: docs/design/review-output-ownership.md
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: models/output-ownership
- confidence: inferred
  path: website/docs/guides/generate-artifacts.md
- confidence: inferred
  path: website/docs/reference/cli.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 9
---
## Finding and source

F10 (P1) from `docs/reviews/2026-09-05-architecture-review.md:365`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:2016`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A repeated generation updates only owned outputs as one recoverable operation while preserving authored files across stale-file retirement and injected failure.

## Implementation boundary

Design ownership and recovery against existing generated artifact paths before implementing staged writes. Define first-run adoption of legacy output, collision refusal, stale generated-file removal, interrupted staging/recovery and preservation of unowned authored additions. Add typed ownership data only after its design and compatibility policy are recorded.

## Validation

Inject mid-write/rename failures, rerun after interrupted staging, remove a formerly generated artifact and keep an unrelated authored file; prove either the previous complete output or recoverably staged new output, never an unexplained mixture.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Input discovery policy is separate; do not delete unknown files or infer ownership from an extension.

## Scope

Derived from the published-source scoper at `1e618d225f7f60f79c3ace7c0fa60f356cbc44d9`, then selected by the coordinator for wave19. Full report SHA256 `6a8a354085aa204cfc2f716c97cb6a8fd7cc0436811d5e870b23f3991b6c9974`; all28 input hashes independently verified. The report distinguishes fresh source reads from reused semantic analysis.

- `crates/edge/ess-cli` — cited; current edge/source boundary or selected typed design home.
- `crates/edge/ess-xtask/src/main.rs` — cited; current edge/source boundary or selected typed design home.
- `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json` — cited; current edge/source boundary or selected typed design home.
- `docs/design/review-output-ownership.md` — cited; current edge/source boundary or selected typed design home.
- `docs/design/source-pinned-data-normalization.md` — cited; current edge/source boundary or selected typed design home.
- `models/output-ownership` — cited; current edge/source boundary or selected typed design home.
- `website/docs/guides/generate-artifacts.md` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.
- `website/docs/reference/cli.md` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.
- `docs/design/review-format-catalog.md` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.
- `website/docs/reference/formats.md` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.
- `Cargo.lock` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.
- `.github/workflows/ci.yml` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.
- `crates/edge/ess-xtask/src/consumer_coverage/profiles.json` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.
- `crates/edge/ess-xtask/src/consumer_coverage/reviewed-candidates.json` — inferred; selected documentation, dependency, platform-validation or conditional consumer-evidence integration reservation.

The whole CLI package covers current main, coverage, model-types, normalization, schema and schema-bundle writers and their tests. New IO helpers remain edge-owned. Existing artifact bytes and actual route ownership stay explicit. The format catalogs and native macOS CI reservation were added by root after reading the persisted-format policy and current Linux/macOS release matrix. Profile/case files are conditional; their reservation grants no change to e005 eligibility.

Confidence: medium before implementation; exact current callers and interface consequences are established, while native adapters and failure handling require tests. This intersects CLI IO/tests, xtask synchronization/classification, the named documentation/model and selected dependency/evidence paths. N=1 prevents concurrent source claims; no disjointness is inferred from differing directory/file tokens.

The coordinator owns planning, wave records, accepted binding/model, CI and other shared integration files. The unit brief assigns source and test writes before dispatch. No implementor writes the planning store.

## Selected output-ownership binding and model — 2026-09-08

The coordinator selects `docs/design/review-output-ownership.md` under the existing standing implementation approval. Ordinary generation keeps its real signatures and fixed default roots. Legacy adoption is a separate operation comparing a settled, unmodified generated reference with exact target bytes; first enrollment for the selected owner or exact idempotence cannot discard another owner's or prior stale inventory. Compose uses one explicit enclosing root. The fixed G01–G14 owners, authored preservation, selected stale retirement, no-write checks and explicit recovery are retained.

The anchor owns at most one current transaction, via its UUID. `models/output-ownership/system.yaml` and `models/output-ownership/domains/ownership.yaml` give those nouns typed identities, the explicit relation and Prepare/Commit/Restore outcomes. The current retained ESS0.20.0 CLI (SHA25678976b7f5ac11dedd6da1649833415afc06d3f15c0de88d3dd1ddf26cd94f442) executed validation and compilation; both actual0. Validation output: `output v1 — 2 file(s), valid`. This is static model evidence, not executed filesystem recovery.

The selected protocol uses anchor-local, same-mount staging; top-down shared ancestor/exclusive anchor directory locks; Staging/Prepared/Committed/Restored checkpoints and settled Idle state; immutable preimages; explicit recovery; and retained decisions through cleanup. The later adjacent-stage/cross-filesystem alternative was considered and not selected. Linux and macOS support remain required; an empty hardware admission registry and ext4-only restriction were withdrawn. Process interruption and injected IO failures are tested; unconditional hardware power-loss survival and simultaneous multi-directory visibility are not promised.

All original W01–W17 obligations remain with the adoption/platform/cross-mount revisions recorded in the binding. Changing each of four bound flat consumer signatures would affect1806 cells (7224 combined); changing either nine-profile signature would affect16254. The chosen ordinary defaults avoid unnecessary declaration changes. Actual signature/profile changes still require new behavioral accounting; helper/body edits still need review and direct same-source execution. No dead wrapper, hidden context or baseline expansion is allowed.

Wave19 uses one implementation unit. Four original stories and the separate recovery implementation were outstanding after wave18; this selects only output ownership. The earlier consumer remediation is published at1e618d2 with CI34194913016 and documentation validation34194912907 successful; all three owned wave18 worktrees and their exact outputs were retired after complete evidence retention. No tag, version bump, release or downstream publication is selected.
