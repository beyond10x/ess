---
format: aep.planning-md/3
id: review-result:concurrent-history-acceptance-round-3
kind: review-result
status: active
title: Acceptance critic, round 3 (wave-1 revisions)
relations:
- reviews: epic:concurrent-history-conformance
- reviews: story:concurrent-explorer-runner
- reviews: story:concurrent-history-format
- reviews: story:session-and-eventual-view-checks
- reviews: story:declared-fault-injection
revision: 1
---
needs-revision

story:session-and-eventual-view-checks — the second acceptance bullet bundles two independently-checkable claims (that the new `StaleReadUnderReadYourWrites` row is caught only by the history check, and that the pre-existing `StaleReadYourWrites` row stays a separate row still caught by the single-client suite), so a change that lands one while breaking the other reads the same as passing both — .engineering/planning/story/session-and-eventual-view-checks.md:42-45

What I read: all five revised artifacts in full via `aep plan artifact show <id>` (epic:concurrent-history-conformance, story:declared-fault-injection, story:concurrent-history-format, story:concurrent-explorer-runner, story:session-and-eventual-view-checks), plus `aep plan artifact show review-result:concurrent-history-acceptance-round-1` and `round-2` for prior findings, `aep plan artifact kinds`, `aep plan artifact lifecycle epic`/`story`, and `aep plan artifact show story:external-mutation-explorer-and-toolchain` to confirm archival status. Cross-checked the on-disk store files at `.engineering/planning/story/{concurrent-history-format,concurrent-explorer-runner,session-and-eventual-view-checks,declared-fault-injection}.md` for exact line numbers.

What I could not establish: nothing outside the finding above — the model-drift bullet added to story:concurrent-history-format ("missing from the type, or the type carries one the model does not declare") reads as one bijection test rather than two independent capabilities, consistent with round 2's treatment of similar single-mechanism-tested-both-ways bullets, so I did not flag it. Separately, out of my lane: the epic's "Depends on existing work" prose still names the archived `story:external-mutation-explorer-and-toolchain` (#156) even though its `relations` were repointed to the three successor stories (`.engineering/planning/epic/concurrent-history-conformance.md`, relations vs. body text) — that's a coupling/consistency question for `plan-critic-design` or `plan-critic-scope`, not acceptance, and it does not set my verdict.

```findings
- file: .engineering/planning/story/session-and-eventual-view-checks.md
  line: 42
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance bundles two independently-checkable claims (that the new StaleReadUnderReadYourWrites row is caught only by the history check, and that the pre-existing StaleReadYourWrites row stays a separate row still caught by the single-client suite) into one bullet, so one could pass while the other fails and the bullet would still read as a single unit
```
