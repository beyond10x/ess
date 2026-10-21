---
format: aep.planning-md/3
id: story:release-record-cache-save-clause
kind: story
status: draft
title: Drop the release-record cache clause that can never be true
revision: 2
---
# Story: release-record-cache-save-clause

## Outcome

The release-record workflow's cache `save-if` says only what can happen, so a reader is not told
that pull requests save the cache.

## Context

Review 2026-10-21, finding 8. `.github/workflows/release-record.yml` runs on `release`,
`workflow_run`, `schedule` and `workflow_dispatch`, never on `pull_request`, so the
`github.event_name == 'pull_request'` clause of the `Swatinem/rust-cache` `save-if` can never be
true.

## Acceptance

- The clause is removed; the cache still saves on the scheduled run on `main`.

## Out of Scope

The cache key.

## Ambiguities

None.

## Open Questions

None.
