---
format: aep.planning-md/3
id: runbook:ui-live-apps-serial-integration-20261003
kind: runbook
status: draft
title: Five serial ui-live-apps waves, one integration branch and one PR
relations:
- informed_by: epic:ui-live-apps
revision: 1
---
## Authority and delivery

Operator approved the five proposed waves serially on 2026-10-03, requiring all of them on one integration branch and one PR. This supersedes per-wave publication cadence: no unit PRs and no intermediate merge to main. Integration branch: batch/ui-live-apps-complete-20261003, based on origin/main 1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036. No open PR matches this batch; the open #396/#397 type-generation PRs are unrelated. Create one bot-authenticated PR when the batch is reviewable, and update that same PR thereafter.

Sequence: served-store-and-entry (#318); feature-request-282 (guard precedence); related-via-optional-input (#304); related-via-stored-reference (#304); related-guard-behaviour (#319). Each unit gets its own managed checkout, serial implementation and independent adversary review. Integrate verified commits into the branch above. Source compile/test, scope and format requirements remain binding. Record local completion separately from final CI evidence; do not claim implemented on a skipped or absent acceptance lane. The common ess/21 bundle constraint in .engineering/waves/downstream-gaps.md remains unresolved until its actual scope and current ownership are reconciled.

## Recovered server candidates

Previous session candidate and full recovery record remain on unit/served-store-and-entry-public at c12cc427a74b3da022708f9a2ca92496e50e047d, including runbook:ui-live-apps-wave-7-resume, raw-report normalization provenance and the first adversary correction. It is retained as unpublished recovery evidence, not overwritten or silently promoted.

The newer accepted story on current main contains additional structural-identity, component-reachability and fallible-context requirements. Committed candidate e9355b003a8c0153d927fbe89c8597cebc667787 from batch/consumer-served-entry-20261002 was applied without committing onto this integration branch as a starting point. Its original checkout remains untouched. Bot author and committer were verified. Its tests/results are not assumed green here; reconcile its implementation with the earlier candidate's useful regression coverage, generated fixtures, docs, dependencies and all current acceptance before freezing review.

## Custody and resources

Integration checkout: ess-w7-server-public; owner session codex-resume-459ab620-public. Historical raw evidence and logs: `$HOME/.cache/uilab-todo/w7s-codex`. New unit checkouts and their targets will be recorded before dispatch. No shared compile target between trees. Rust is mandatory for committed executable repository code; generated Go output remains an explicit accepted target. Keep scratch outside repositories and outside /tmp; 10 GiB free space is the build floor. Preserve original review evidence privately; public reports normalize workstation prefixes with an explicit note and original digest.

## Cleanup and release audit in progress

Current integration checkout was clean before branch creation. Predecessor ids ess-w4-ui-tui-app, ess-w5-go-behaviour and ess-w6-view-params have no live leases or tracked/untracked changes, but retain ignored target output; the third also retains examples/billing-web/Cargo.lock. They require reviewed recovery proof and evidence preservation before managed cleanup. Original ess-w7-store-entry was already privately archived and removed through managed GC. Do not infer the workspace is globally clean: other sessions own active work and open PRs.

Read-only GitHub release listing shows 0.51.0 published at 2026-10-02T03:20:46Z. Exact tag/check/artifact verification is being performed independently; later merged UI work is not assumed released. No new release is cut as part of this audit.
