---
format: aep.planning-md/2
id: review-result:concurrent-history-acceptance-round-1
kind: review-result
status: active
title: 'Acceptance critic, round 1: concurrent history conformance'
relations:
- reviews: story:session-and-eventual-view-checks
- reviews: story:concurrent-history-format
- reviews: story:concurrent-history-lanes
- reviews: story:concurrent-explorer-runner
- reviews: story:recorded-history-validation
- reviews: story:linearizability-checker-over-the-interpreter
- reviews: story:declared-fault-injection
revision: 1
---
approve/needs-revision verdict follows. Findings are scoped to acceptance-checkability only, per the plan-critic-acceptance role; coupling, parent-coverage and shared-surface concerns are explicitly out of my lane and not represented below.

needs-revision

story:concurrent-history-format — the acceptance bundles two independent malformed-history cases ("no return instant" and "return precedes its invoke") into one statement, so one refusal rule could land while the other does not and the bullet still reads pass/fail as a single unit — .engineering/planning/story/concurrent-history-format.md:29-30
story:concurrent-explorer-runner — the acceptance bundles the search capability ("200 seeds find a violation") and the shrink capability ("holds at most 2 clients and 4 operations") into one statement, so a working search paired with a weak or absent shrinker cannot be told apart from both working — .engineering/planning/story/concurrent-explorer-runner.md:31-32
story:declared-fault-injection — the acceptance bundles two independently-landable new faulty.rs rows, `DoubleApplyOnRedelivery` and `RetryCreatesSecondEntity`, into one statement, so one row shipping without the other reads the same as both shipping — .engineering/planning/story/declared-fault-injection.md:31-32

What I read: all 7 artifacts in full via `aep plan artifact show <id>` (story:concurrent-history-format, story:linearizability-checker-over-the-interpreter, story:concurrent-explorer-runner, story:session-and-eventual-view-checks, story:declared-fault-injection, story:concurrent-history-lanes, story:recorded-history-validation), plus `aep plan artifact kinds` and `aep plan artifact lifecycle story`. Cross-checked symbols named in acceptance text against the tree: `faulty.rs`, `Billing::DEFAULT_LAG` in `reference.rs`, `web.rs`, the `ess-gen` crate, the CLI's `VerifyCommand` enum, and `models/concurrent-history/` all exist as named.

What I could not establish: the "committed corpus of histories" for the Porcupine differential test (story:linearizability-checker-over-the-interpreter) does not exist in the tree yet, but that is expected new-work scope rather than an acceptance defect, so I did not flag it. For the four stories not listed above (story:linearizability-checker-over-the-interpreter, story:session-and-eventual-view-checks, story:concurrent-history-lanes, story:recorded-history-validation), the recurring positive/negative-control pairs in their acceptance bullets ("Violation on the faulty target, and Linearizable on the correct one"; "passes, and a variant that never converges fails"; two sentences on the same recorded-history adapter) are, in my judgment, a single mechanism tested both ways rather than two independently-landable pieces — I did not flag these, and note the distinction so a second reader can override it.

```findings
- file: .engineering/planning/story/concurrent-history-format.md
  line: 29
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance bundles two independent malformed-history cases ("no return instant" and "return precedes its invoke") into one refusal statement, so one validation rule could land while the other does not
- file: .engineering/planning/story/concurrent-explorer-runner.md
  line: 31
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance bundles the search capability ("200 seeds find a violation") and the shrink capability ("holds at most 2 clients and 4 operations") into one statement, so a working search with a weak shrinker cannot be told apart from both working
- file: .engineering/planning/story/declared-fault-injection.md
  line: 31
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance bundles two independently-landable new faulty.rs rows, DoubleApplyOnRedelivery and RetryCreatesSecondEntity, into one statement, so one row shipping without the other reads the same as both shipping
```
