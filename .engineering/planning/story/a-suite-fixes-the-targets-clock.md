---
format: aep.planning-md/3
id: story:a-suite-fixes-the-targets-clock
kind: story
status: draft
title: A suite fixes the target's now for a step, and synthesis witnesses every now comparison on its boundary
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#586
relations:
- decomposes: epic:generic-expression-language
- serves: vision:O2
revision: 1
---
## Outcome

A suite can fix the instant a target reads as `now` for a step, through an optional target
capability beside the existing `Clock`. Where the target offers it, synthesis writes boundary
witnesses *on* every `now` comparison, in every position the expression language admits `now`:
command guards over inputs, `when_subject:` and `when_related:` predicates over stored `Timestamp`
fields, and view filters and fields (the request clock of
`docs/design/generic-language-features.md`, G1 synthesis cost, `now` in views). A target without the
capability keeps today's off-boundary witnesses, and the at-boundary scenarios are reported
unsupported with the capability named.

Findings covered: https://github.com/beyond10x/ess/issues/586, option 1 (a downstream audit on ess
0.60.0, `ess/25`). The same clock is the "suite-set read clock" that
`story:expression-arithmetic-and-time` names for `now` in a view filter
(https://github.com/beyond10x/ess/issues/567, closed).

## Fit review

1. **Need.** A held-row guard `expiry < now` and its `<=` mutant answer differently only when
   `expiry` equals the target's `now`, and no scenario can place a value there: synthesis decides
   witnesses against a fixed reference and keeps them a second from each boundary, never on it,
   because the target's clock moves (`crates/verify/ess-conformance/src/now_offset.rs:20-23`). The
   generated Go runner's `Clock` measures elapsed time and cannot set an instant
   (`crates/verify/ess-conformance/src/go/runtime.go:1387-1416`; Rust `runner.rs:114-127`).
   Minimal reproduction to write first: `demo.codes.Code {id, expiry: Timestamp}`, `Issue` setting
   `expiry` from input, `Redeem` refusing `when_subject: {predicate: expiry < now}`.
   Requester's proposal (theirs): an optional target capability to fix `now`, and boundary
   witnesses at `now`.
2. **Class.** Gap: the boundary is a domain fact ("redeemable at exactly its expiry") that no
   generated suite can check, while a hand-written frozen-clock test can.
3. **Already expressible?** Only by an authored scenario against a target-specific frozen clock,
   outside the suite.
4. **Fit.** One clock rule for every position that reads `now`, not one for stored timestamps: the
   design record already plans a suite-set request clock for views and states that synthesis decides
   `now` at the suite's reference instant (G1, synthesis cost table). This story is that clock, made a
   target capability so commands, stored-field guards and views share it. Steps carry the instant as
   they carry `now_offset` today; runners resolve it by asking the target to hold that instant.
5. **Second adopter.** A booking window that opens at exactly its start instant; a token valid up
   to and including its `not_after`; a view of "active" rows whose `ends_at >= now`.
6. **Cost.** A new optional target capability in the Rust, Go and TypeScript runners; a suite
   format pair for the fixed-instant step; mutation reports on clocked models change (the
   `guard-boundary` mutant becomes killable). Source format: none for commands; views follow the
   G1 format that admits `now` there.
7. **Alternatives.** (a) Change nothing and accept off-boundary witnesses. (b) Witnesses at the
   reference offset with zero latency tolerance: flaky by construction. (c) A settable clock as an
   optional capability, at-boundary witnesses only where it is offered (chosen). The report-side
   half of #586 (option 2) is `story:a-mutant-whose-suite-equals-the-baseline-is-unwitnessed`.

## Decisions

Accept, redesigned: the requester's capability, generalised from stored `Timestamp` guards to every
position that reads `now`, and shared with the view request clock the design record plans.

## Acceptance

- `fixed_clock_kills_the_boundary_mutant`: on the reproduction with a target offering the
  capability, the `expiry < now` to `expiry <= now` mutant is killed by a scenario that sets `now`
  to the stored `expiry`.
- `input_guard_boundary_on_now`: the same for `starts_at < now - 60s` on a command input.
- `target_without_the_capability`: the at-boundary scenarios are reported unsupported naming the
  capability; every other scenario keeps today's verdict and bytes.
- `runners_agree_on_the_fixed_instant`: the Rust, Go and TypeScript runners resolve the same step to
  the same instant and give the same verdicts.

## Scope (inferred)

`crates/verify/ess-conformance/src/now_offset.rs`, `runner.rs` (`Clock`), `go/runtime.go`, the
TypeScript runtime, synthesis of `now` witnesses, `docs/design/current-time-guards.md`.
