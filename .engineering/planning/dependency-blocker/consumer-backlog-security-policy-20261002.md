---
format: aep.planning-md/3
id: dependency-blocker:consumer-backlog-security-policy-20261002
kind: dependency-blocker
status: cleared
title: Required security CI exceeds the historical scan limit
relations:
- blocks: task:consumer-backlog-20261002
- blocks: task:consumer-server-gate-corrections-20261002
- blocks: task:consumer-ui-consolidation-20261002
revision: 4
transitions:
- {from: "open", to: "cleared", at: "2026-10-02T09:49:31Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"verification":2}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Blocked boundary

Integration and release of the queued consumer-fix batches require the common Security and privacy check. Local implementation and verification may continue.

## Evidence

PR 381 at 28aeddddfc86c4a92c48aba98d5d70091ba4219d: Actions run 36961709429 failed with `candidate exceeds scan limit` while ESS Gate passed. PR 386 at f0b220099119090795c93cae07c14e6209918f67: https://github.com/beyond10x/ess/actions/runs/36987213274/job/110774865932 failed at 2026-10-02T09:03:49Z with `b10x-gates: candidate exceeds scan limit`.

The workspace operating instructions establish that the scan covers every commit from the policy adoption baseline, capped at 512 MiB, and that CI reads B10X_GATES_POLICY from the repository secret. Updating the separate gates-policy repository did not clear this exact failure on PRs 380 and 381. A local signed receipt is not replacement evidence for the required remote check.

## Clearing action and owner

The repository secret's owner refreshes the approved policy baseline in B10X_GATES_POLICY. Agents do not change this secret, bypass the hook, or repeatedly rerun unchanged candidates. Once the owner confirms the input has changed, rerun the failed common check on each exact candidate through the bot App, verify success alongside ESS Gate, and continue integration. This blocker is not a disposition of unrelated source defects and does not stop their local work.

## Authorized secret refresh 2026-10-02

The operator explicitly authorized a one-time personal gh CLI exception for the secret update. Trusted policy.json exactly matched committed remote main63186584eb48e99c8f1046dbf9f1ef32f638700a in gates-policy; its ESS adoption baseline is released0.51.0 commit0347ffa222939e3791e574d2dbe42d4b4b02d979. No policy rule, exception or baseline was edited.

The first secret update was rejected HTTP422 Value is too large. JSON whitespace compaction reduced60129bytes to47760bytes, and sorted structural comparison proved every value unchanged. Retrying the same authorized update succeeded (gh secret set B10X_GATES_POLICY exit0). Bot retrieval of secret metadata was refused403; no alternative metadata client was used.

The bot App then successfully requested only the failed jobs of security run36987213274 for PR386. ESS correctness Gate at exact candidate f0b220099 was already green and was not rerun. Keep this blocker open until the security rerun succeeds. Do not rerun old component candidates that are being consolidated into381; the final published UI candidate will receive its own checks.

## Resolution

Cleared after observing security run36987213274 attempt2 completed SUCCESS. The exact f0b220099 candidate has both common Security and privacy and ESS Gate completed successfully. No rule or exception was weakened; the previously approved ESS baseline was delivered to the repository secret using the operator's explicit one-time authorization. The unchanged correctness workflow was not rerun.

PR386 subsequently merged through the bot App as fa08de5bd816b3cc46694ec4d19d20d13e128764. GitHub reports merged_by b10x-bot[bot]; the branch-authority ruleset admits only that App. Git tree2105a56fd3b86a78d8db839ee04bc8826f663a38 exactly matches tested candidate f0b220099, and the merge first parent c211314af is already its ancestor. GitHub's merge committer is legitimate web-flow identity, as permitted by workspace operating instructions.
