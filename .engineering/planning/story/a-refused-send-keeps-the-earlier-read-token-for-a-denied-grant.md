---
format: aep.planning-md/3
id: story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant
kind: story
status: implemented
title: A denied-grant snapshot reads its view at the token before the refused send
summary: A refused send drops the consistency token, so a read_your_writes denied-grant snapshot errors (adopter item 2, beyond10x/a2a); defect, 0.69.0.
tags:
- adopter-report
- defect
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/grant.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/grant_denied.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T04:08:30Z", actor: "human:timo", revision: 9, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T04:08:30Z", actor: "human:timo", revision: 10, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:24Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 2 of 9, 2026-10-22). The
denied-grant scenario of a view with `read_your_writes` consistency errors:

```text
error a2a.tasks.GetTask/grant/denied
    ESS-CF-SUITE: no consistent view query preceded the snapshot assertion
```

2 of the run's 14 errors are this one (GetTask and SubscribeToTask). The run: 120 scenarios,
106 passed, 14 error, exit 3.

Minimal reproduction (brand-free): a view `JobById` with `consistency: read_your_writes` and a
grant; synthesize the grant scenarios and run the `grant/denied` one against the interpreted target.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/grant.rs:589-606`: the denied scenario keeps the
  template's QueryView and its unchanged claim after the refused send.
- `crates/verify/ess-conformance/src/runner.rs:1036-1047`: the query is skipped when
  `last_command` carries no consistency token; message at `runner.rs:2833-2839`.
- `runner.rs:1422-1430` with `crates/verify/ess-conformance/src/target.rs:786-790`: the refused
  send is stored as an undeclared result with `consistency: None`.

## 0.68.0

No. Rerun on 0.67.0 and 0.68.0: the same 2 errors; none of these files changed in range.

## Cause

Verified by code read. The runner treats a refused send as a write without a consistency token,
skips the following `read_your_writes` query, and the snapshot assertion then finds no
`last_view`. Only commands with a `preserves:` claim carry a post-send unchanged assertion, which is
why only GetTask and SubscribeToTask hit it (inferred).

## Fit review

1. **Need.** A denied grant on a `read_your_writes` view must be checkable: the refusal wrote
   nothing, so the earlier read's token still bounds the snapshot. No syntax proposed.
2. **Class: defect.** A refused send is not a write (`target.rs:786-790` stores it as one without
   a token), and the suite cannot verify a scenario the specification declares.
3. **Already expressible?** Only by weakening consistency to eventual, which changes the domain.
4. **Fit.** No authored surface. The runner keeps the consistency token of the last successful
   read or write when a send is refused, so the following query runs. Same rule for every view
   consistency level; every target goes through the same runner path.
5. **Second adopter.** A document store whose `GetDocument` view is `read_your_writes` and whose
   `ShareDocument` is grant-guarded.
6. **Cost.** None in formats or diagnostics.
7. **Alternatives.** (a) Change nothing. (b) Drop the snapshot from denied scenarios: loses the
   "refusal changed nothing" evidence. (c) Chosen: a refused send keeps the earlier token.

## Decisions

**accept.** A denied-grant snapshot reads its view at the token held before the refused send.

2026-10-22: no format change. This story and
`story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued` are one unit (both
land on `runner.rs`). `story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain` depends
on this one and lands after it; its acceptance keeps this story's denied-grant case passing.

## Acceptance

Fixture: the minimal reproduction above (a `read_your_writes` view `JobById` with a grant),
written by the test itself.

- **The denied snapshot runs.** Transition: a grant-refused send follows a successful read.
  Before: `grant/denied` errors with `ESS-CF-SUITE: no consistent view query preceded the snapshot
  assertion`. After: the runner keeps the earlier token (`runner.rs:1036-1047`, `:1422-1430`), the
  query runs and the scenario passes. Test: `crates/verify/ess-conformance/tests/grant_denied.rs`,
  case `a_refused_send_keeps_the_earlier_read_token`. Failing before: the case errors on the
  current tree with the line above.
- **A refused send that writes is caught.** Transition: a mutation target changes the row on the
  refused send. After: the scenario fails at the unchanged assertion. Same file, case
  `a_refused_send_that_writes_fails_the_denied_snapshot`. Mutation check: a runner that skips the
  snapshot after a refused send (today's behaviour turned into a pass) makes this case green, so
  it is red against that mutant.
- **Other grant scenarios keep their verdicts.** Transition: none. After: every case in
  `tests/grant_denied.rs`, `tests/view_grant_denied.rs` and `tests/view_grant_security.rs` keeps
  its verdict, and synthesized suites are byte-identical.

## Schedule

Release 0.70.0 (defect, wave A unit A2 with its sibling on `runner.rs`). The store has no
target-release field; the release tag on the artifact marks it.

## a2a wave 1

Relayed by the orchestrator on 2026-10-22 from the adopter session specifying A2A in
beyond10x/a2a (ESS 0.67.0, a2a main 3a18050):

- **M8, second half** (a2a wave-1 gaps list, line 42): the 4 known conformance errors "no
  consistent view query preceded the snapshot assertion" on grant/denied. The same errors as this
  story's Need (it counted 2 on the earlier run). The other half of M8 (12 errors on agent moves)
  is `story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued`.

## Further report

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22), read from code on 0.67.0, reproduced on 0.68.0.

- Count: 4 scenarios error, not 2: `GetTask`, `GetTaskPushNotificationConfig`, `ListTaskPushNotificationConfigs` and `SubscribeToTask`, each `/grant/denied`. Each reads `ESS-CF-SUITE: no consistent view query preceded the snapshot assertion` (interpreted run of the adopter's suite on 0.68.0, 168 scenarios, 16 errors, exit 3).
- ESS file:line: `TargetError::NotGranted` carries no consistency token, `crates/verify/ess-conformance/src/target.rs:786-790`; the runner paths that drop it, `crates/verify/ess-conformance/src/runner.rs:1036-1047`, `:1422-1430`, `:2833-2839`; synthesis of the denied-grant snapshot, `crates/verify/ess-conformance/src/synthesize/grant.rs:589-606` (cited from code).
- Acceptance stays as written; the conformance scenario should cover a view read, a command read and a subscription after the refusal, since all three kinds error.
