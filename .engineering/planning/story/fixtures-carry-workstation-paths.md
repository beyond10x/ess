---
format: aep.planning-md/1
id: story:fixtures-carry-workstation-paths
kind: story
status: draft
title: Coverage-producer fixtures carry workstation paths
summary: Two tracked coverage fixtures embed absolute workstation paths since 874962d; record repository-relative roots at the writer and refuse them in the gate.
tags:
- hygiene
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/fixtures/coverage-producers/input-catalog.json
- confidence: cited
  path: crates/edge/ess-cli/tests/fixtures/coverage-producers/semantic-plan.json
revision: 3
---
# Coverage-producer fixtures carry workstation paths

## Finding

`crates/edge/ess-cli/tests/fixtures/coverage-producers/input-catalog.json` (10 `original` values, lines 7-70) and `crates/edge/ess-cli/tests/fixtures/coverage-producers/semantic-plan.json` (`document_root` at line 5718, `root` at line 5812) contain absolute paths under the workstation's managed worktree root. They have been on `main` since 874962d (2026-09-06, "feat(conformance): admit declared coverage and original selection lineage"); found by the wave-21 coordinator's host-path check on 2026-09-09. The repository rule is that tracked files carry repository-relative paths and managed ids only (PR #16 removed the same class from the wave-20 records).

## Acceptance

No tracked file under `crates/`, `docs/`, `website/` or `models/` contains a path under a user's home directory, and a gate lane refuses one.

## Scope

- the two fixture files above — cited.
- the producer that wrote them (the coverage lineage recorder, `crates/edge/ess-cli/src/` or `crates/verify/ess-conformance/src/`) — inferred; the fix is to record a repository-relative `original`/`root`, or a managed id, at the writer.
- a Rust check in `crates/edge/ess-xtask` wired into `task check` — inferred (AGENTS.md: anything executable is Rust).
- Would collide with: any unit touching those fixtures or the coverage lineage writer — cited.
