---
format: aep.planning-md/1
id: story:release-status-publication-state
kind: story
status: draft
title: Release status distinguishes drafts from public releases
relations:
- informed_by: release-plan:consolidated-ess-019
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
revision: 3
---
## Observed Gap

During release 0.19.0, `task release-status` printed `tagged and published`
while GitHub release 383390739 still had draft=true and published_at=null.
The actual API observation was made on 2026-09-05 after creating the draft
and before publishing its four native packages.

`crates/edge/ess-xtask/src/main.rs:344` requests only tagName from
`gh release list`; `release_tag_names` accepts every listed tag, and
`release_status` equates membership with publication. Draft visibility depends
on caller authentication, but a visible draft is not a public release.

## Acceptance

Release status must classify a visible draft as not published, using explicit
GitHub publication state. A published release retains its existing classification;
missing or malformed publication state is not silently admitted as published.
Pin these distinctions with parser/status tests and retain the existing offline
gate boundary. No release is edited, published, deleted or retagged by the check.

## Scope

Derived 2026-09-06 by `aep-drive:story-scoper` against production source at ESS `dcb84be861d2f906b3dd95254f03701cb264faa2`; coordinator planning edits were observed and left untouched — cited.

- **Primary surface/edit token:** `crates/edge/ess-xtask/src/main.rs` — cited; owns the release query, response parser, publication classification and inline regression tests.
- **Symbols:** `published_releases` requests only `tagName`; `release_tag_names` accepts every listed tag; `release_status` treats membership as publication. The status route connects these functions without another publication-state check — cited.
- **Regression surface:** extend the existing inline parser/status tests to distinguish visible drafts, explicit published state and missing/malformed publication state; retain untagged-version and off-main ancestry controls — cited.
- **Smallest implementation:** request explicit publication state and validate it before admitting a tag to the published set; existing `serde_json` support permits this without a new dependency — inferred.
- **Documents:** none required by the artifact’s acceptance — inferred.
- **Boundary:** preserve the separate networked status command and offline gate. No release workflow, tag, release asset, publication or historical ancestry mutation belongs to this correction — cited.
- **Ownership:** this imported draft belongs to the other session’s release follow-up and is excluded from the coordinator’s next implementation wave — cited.
- **Confidence:** high — the artifact’s defect sites and existing inline tests match current source — cited.
- **Would collide with:** any writer of the xtask main file, including other release-query/parser/status changes; planning and final change records remain coordinator-owned — inferred.

## Boundary

This is a follow-up tooling correction, not part of the already frozen 0.19.0
source. Its independent release verification uses the GitHub API and actual
downloaded packages. The historical 0.17.0 ancestry failure is a separate,
source-accounted reconciliation owned by release-plan:consolidated-ess-019.
