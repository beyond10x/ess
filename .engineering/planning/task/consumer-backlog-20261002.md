---
format: aep.planning-md/3
id: task:consumer-backlog-20261002
kind: task
status: active
title: Process the full consumer-defect backlog in grouped deliveries
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T08:40:21Z", actor: "human:timo", revision: 2, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T08:40:21Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Every consumer defect or gap in the ESS GitHub issue backlog and the AEP store has an evidence-backed disposition, and every actionable defect is delivered in a coherent batch under the required gates.

## Acceptance

The reconciliation accounts for every open GitHub issue and every nonterminal consumer-facing story, links duplicates and existing implementations, and separates released fixes, merged fixes, queued candidates, declined requests with a demonstrated idiom, and concrete unresolved blockers. Actionable defects have regression evidence and a grouped pull request; queued or blocked work is never reported as shipped.

## Authority and execution

Operator direction on 2026-10-02: process the full consumer-defect backlog from AEP and GitHub today, ship fixes fast in groups, one managed worktree and one pull request per related batch. Prepare locally before expensive remote gates; keep one new batch in CI while the next is prepared. The request authorizes this continuing work, not fabricated approvals or bypassing required gates.

The initial source set is 82 open GitHub issues and 161 nonterminal stories in a 923-artifact AEP store. Recheck both sources before the completion audit. Counts are inventory, not completed work.

## Current evidence

- PR 381 at 28aeddddfc86c4a92c48aba98d5d70091ba4219d carries 21 fixes. Its ESS Gate passed; common security failed with `candidate exceeds scan limit` in Actions run 36961709429. The B10X_GATES_POLICY secret owner must refresh its approved baseline. No bypass or repeated unchanged rerun.
- Generated-server corrections are published in combined PR 386 at f0b220099119090795c93cae07c14e6209918f67. PRs 383 and 384 are closed as superseded after verifying both exact heads are ancestors of this candidate. Local affected packages: 453 passed, two existing ignored; exact task test-xtask: 362 passed, three existing ignored; concurrent served-view nextest: eight passed. Strict Clippy and formatting passed. Correctness CI run 36987215656 is pending; common security run 36987213274 failed with the same scan-limit refusal. No integration or release claimed.
- Story feature-request-309 has five measured failing regressions before the shared-input aggregate-key fix and five passing after it. All 300 package lanes completed successfully: 2145 passed, zero failed, 11 existing ignored. Independent review is in progress. The full-package baseline was interrupted for disk pressure and its total is unknown.
- The 0.51.0 GitHub Release is public with four archives and SHA256SUMS. No 0.52.0 tag exists at intake.

## Completion boundary

Implementation proceeds on unblocked items while external publication is blocked. A published branch is queued, not released. Design decisions and external dependencies require explicit linked blockers, with their exact question and next action. No item is deferred merely to make the inventory smaller.
