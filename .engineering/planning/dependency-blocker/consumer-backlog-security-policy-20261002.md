---
format: aep.planning-md/3
id: dependency-blocker:consumer-backlog-security-policy-20261002
kind: dependency-blocker
status: open
title: Required security CI exceeds the historical scan limit
relations:
- blocks: task:consumer-backlog-20261002
- blocks: task:consumer-server-gate-corrections-20261002
- blocks: task:consumer-ui-consolidation-20261002
revision: 1
---
## Blocked boundary

Integration and release of the queued consumer-fix batches require the common Security and privacy check. Local implementation and verification may continue.

## Evidence

PR 381 at 28aeddddfc86c4a92c48aba98d5d70091ba4219d: Actions run 36961709429 failed with `candidate exceeds scan limit` while ESS Gate passed. PR 386 at f0b220099119090795c93cae07c14e6209918f67: https://github.com/beyond10x/ess/actions/runs/36987213274/job/110774865932 failed at 2026-10-02T09:03:49Z with `b10x-gates: candidate exceeds scan limit`.

The workspace operating instructions establish that the scan covers every commit from the policy adoption baseline, capped at 512 MiB, and that CI reads B10X_GATES_POLICY from the repository secret. Updating the separate gates-policy repository did not clear this exact failure on PRs 380 and 381. A local signed receipt is not replacement evidence for the required remote check.

## Clearing action and owner

The repository secret's owner refreshes the approved policy baseline in B10X_GATES_POLICY. Agents do not change this secret, bypass the hook, or repeatedly rerun unchanged candidates. Once the owner confirms the input has changed, rerun the failed common check on each exact candidate through the bot App, verify success alongside ESS Gate, and continue integration. This blocker is not a disposition of unrelated source defects and does not stop their local work.
