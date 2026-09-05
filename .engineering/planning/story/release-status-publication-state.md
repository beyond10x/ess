---
format: aep.planning-md/1
id: story:release-status-publication-state
kind: story
status: draft
title: Release status distinguishes drafts from public releases
relations:
- informed_by: release-plan:consolidated-ess-019
revision: 1
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

- cited: crates/edge/ess-xtask/src/main.rs, release query, parser and status tests.

## Boundary

This is a follow-up tooling correction, not part of the already frozen 0.19.0
source. Its independent release verification uses the GitHub API and actual
downloaded packages. The historical 0.17.0 ancestry failure is a separate,
source-accounted reconciliation owned by release-plan:consolidated-ess-019.
