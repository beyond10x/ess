---
format: aep.planning-md/3
id: story:release-record-cache-save-clause
kind: story
status: archived
title: Drop the release-record cache clause that can never be true
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-10-21T14:14:23Z", actor: "human:timo", revision: 4}
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

## Resolution

Not done. `crates/edge/ess-xtask/tests/ci_lanes.rs` holds one `CACHE_SAVE_IF` string for every
`Swatinem/rust-cache` step in every workflow
(`compile_caches_are_saved_only_where_a_later_run_restores_them`), so a reader checks one rule
instead of one per workflow. Dropping the clause here failed that test; the clause stays, and the
story is archived.
