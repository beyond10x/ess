---
format: aep.planning-md/3
id: runbook:release-bot-publication-20261003
kind: runbook
status: draft
title: ESS grouped source release and bot publication repair
relations:
- delivers: task:release-0-52-0-20261003
revision: 1
---
## Authority

The operator requested a new ESS release with all completed ESS work, assigned the frozen source-session handoff to this coordinator, explicitly approved implementing and independently reviewing Gates provenance support, and chose b10x-bot publishing. Existing PR398 is merged; no ESS PR was open on receipt. This branch becomes the single additional ESS release carrier. The separate Gates repository requires its own single dependency PR. No per-ticket ESS PRs.

## Units and owners

Coordinator owns managed tree ess-release-bot-20261003, branch release/0.52.0-bot-publication, and the source/plan/changelog integration. Child signal0e7fb770 is integrated unchanged as d2fdeb4ca after parent-byte comparison. Its prior independent review is retained, and final integrated verification remains due.

Publisher implementor owns managed tree ess-release-publisher-unit-20261003, branch unit/ess-bot-release-preparation, base d09cd00ba0ece15bf043798ac622d32c9f94eb3b. Build directory is its own target; scratch ess-release-dependencies-20261003/publisher-unit. Active story bot-release-publication scopes only workflow, xtask contract tests and repository instructions. It owns the one ESS compiler slot until its checks finish. Native Codex agent runs aep:implementor procedure from skill0.19.1 using gpt-5.6-sol. An independent adversary reviews the frozen result before integration.

The Gates repair has its separate governed story/runbook in Gates and a small independent compiler footprint. No shared target directories or App credentials in candidate-executing CI. Source, planning, review and release commits use the protected bot route. No publication of ESS ancestry until the reviewed delivered Gates verifier accepts it.

## Retention and completion

The full sixty-commit runtime bundle and dirty history/browser work remain frozen under the accepted handoff; they are not source of this release. The handoff manifest and live source checksum proofs are retained. All new worktrees keep explicit leases and evidence until publication or managed archival. Full local task check plus site-build including site-lab, exact remote checks, annotated tag, bot-authored GitHub Release and four archives with SHA256SUMS are required before reporting released. Documentation delivery remains asynchronous.
