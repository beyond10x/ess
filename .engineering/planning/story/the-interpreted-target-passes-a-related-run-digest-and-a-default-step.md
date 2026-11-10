---
format: aep.planning-md/3
id: story:the-interpreted-target-passes-a-related-run-digest-and-a-default-step
kind: story
status: active
title: The interpreted target passes a whole-model related-run digest and default step
summary: 15 interpreted scenarios fail on DigestMismatch and an await deadline in a whole model (workbench report items 37, 38)
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: .engineering/repro/workbench-2026-11-09/examples/37/spec/system.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/related_guard_accepting_arrangement.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/system_issued_unclaimed_send.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:40Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:40Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

On the reporter's whole model (not minimised, not in the repository; the reporter holds it),
synthesis on 0.92.0 gives 879 scenarios and the interpreted target passes 861, fails 15 and leaves
3 unsupported. Two failure shapes, items 37 and 38 of the 2026-11-09 workbench report:

- `MarkConforming/outcome/conforming` is answered with the model's own `DigestMismatch` refusal and
  the view keeps showing the subject `Validated` (item 37). The report's diagnosis: the scenario's
  setup does not record the related run whose digest the guard compares, or records it with another
  digest. Inferred, not checked: the accepting branch sits behind a `when_related:` refusal over the
  related run's stored digest, and synthesis arranges a related row the guard does not accept, so
  the interpreted target answers the refusal. `DigestMismatch` is a name in the reporter's model;
  ESS has no such error. A probe comparing the related row's field with the subject's own field
  (`predicate: digest != subject.digest`) is refused on 0.90.0 as `unobservable_fact` (a
  `when_related:` predicate reads the related row's stored fields only), so the reporter's guard
  compares against an input or a literal.
- `TakeDefault/outcome/no-default` ends `nothing reached by the runner's deadline, after 50
  await(s)` (item 38), the await-outcome timeout of
  `crates/verify/ess-conformance/src/runner/await_outcome.rs:141`, recorded when every await
  answers `NotIssued`.

Item 38 reproduces on the committed example 28
(`.engineering/repro/workbench-2026-11-09/examples/28/spec`, whose `TakeDefault` is issued by the
system actor `gap.demo.Clock`). With ess 0.90.0, `ess verify conform run --path spec --target
interpreted --report-format 2` fails `gap.demo.TakeDefault/outcome/no-default` with
`ESS-CF-AWAIT`, "nothing reached by the runner's deadline, after 50 await(s)". The synthesized
scenario (`TestStrategy::ObserveSubjectFact`) begins with `await_outcome` of `TakeDefault` on the
literal `id`, before `CreateItem` has created any row, and ends correctly with `expect_not_issued`
on the created `item`. The cause is the system-issued rewrite
(`crates/verify/ess-conformance/src/synthesize/system_issued.rs`, around line 161,
`outcome.map_or(Claim::Reached, claim_of)`): a send followed by no `expect_outcome` of the command
before the next send claims no branch, and is rewritten as an accepting `await_outcome`. The
interpreter answers that await `NotIssued` as its documented rule says (`interpret/await_outcome.rs`
module doc: a combination selecting a refusal is not reached; no row carries `id`).

Also seen on example 28, and not this story: `gap.demo.TakeDefault/outcome/defaulted` and
`gap.demo.Item/transition/finish/by/gap.demo.TakeDefault/defaulted` fail the same way, because no
command writes `dflt`, so `no-default` (`not defined(dflt)`) claims every row and `defaulted` is
unreachable.

## Decisions

- Item 38 is a synthesis defect: the interpreted target answers the model (nothing is issued for
  an identity no record carries), and the scenario awaits a step the model never takes.
- A send of a system-issued command that claims no branch is dropped by the rewrite, and the
  checks after it stay: there is no branch for an await to witness, and an `expect_not_issued`
  without `await_within` would hold each scenario for the runner's whole deadline.
- Item 37 is treated as a synthesis defect in the related-row arrangement, as the report diagnoses:
  the interpreted target answering the model's own refusal for the row synthesis arranged is the
  model's answer, and no evidence points at the interpreter.

## Acceptance

- `cargo test -p ess-conformance --locked --test system_issued_unclaimed_send` passes, asserting
  that the suite synthesized from example 28's model has, in
  `gap.demo.TakeDefault/outcome/no-default`, no `await_outcome` step and one `expect_not_issued`
  of `gap.demo.TakeDefault` on the instance `item`.
- The same test runs that suite on the interpreted target and
  `gap.demo.TakeDefault/outcome/no-default` passes.
- `.engineering/repro/workbench-2026-11-09/examples/37/spec` holds a minimised model on which
  `ess verify conform run --target interpreted --path spec --report-format 2`, built from the
  release before the fix, fails the scenario of the accepting branch behind the `when_related:`
  refusal with the model's own refusal as the answer.
- `cargo test -p ess-conformance --locked --test related_guard_accepting_arrangement` passes,
  asserting that the suite synthesized from that minimised model arranges a related row the
  `when_related:` refusal does not select, and that the accepting scenario passes on the
  interpreted target.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/system_issued.rs (`claimed`, the rewrite
  around line 161)
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs (item 37, related-row
  arrangement)
- inferred: crates/verify/ess-conformance/tests/system_issued_unclaimed_send.rs (new)
- inferred: crates/verify/ess-conformance/tests/related_guard_accepting_arrangement.rs (new)
- inferred: .engineering/repro/workbench-2026-11-09/examples/37/spec/system.yaml (new)

confidence: medium

Shared surfaces moved: re-pins the suite-bytes rows of models whose suites send a system-issued
command that claims no branch.

## Fit review

Item 38: need and class are answered above (a synthesis defect, reproduced on example 28). Item
37: pending minimisation. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, items 37 and 38.
