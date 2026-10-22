---
format: aep.planning-md/3
id: story:a-guard-matches-an-input-against-a-stored-wildcard-pattern
kind: story
status: rejected
title: A guard matches an input against a stored wildcard pattern, or the external idiom is recorded
summary: No predicate matches an input against a stored pattern field; string operators take a literal, param or input operand only, and patterns held as data are declined by design (adopter item 7, 2026-10-22); same on 0.67.0.
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T08:59:12Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "rejected", at: "2026-10-22T08:59:12Z", actor: "human:timo", revision: 5}
---
## Need

Reported by an adopter on 2026-10-22 (item 7 of the second batch, relayed by the orchestrator; recorded on 0.56.0, re-probed
here). A subscription stores a channel pattern (`/foo/*` one segment, `/foo/**` any depth); a publish reaches the subscriptions
whose stored pattern matches the published channel. No predicate can match an input against a stored pattern.

Needed: a match predicate whose pattern operand is a stored field, not a literal, or a recorded decision that this is the
system's evaluator (an `external:` outcome) with the idiom named.

## Reproduction

Brand-free fixture, `format: ess/29`, domain `demo.server`: entity `Subscription` with a stored `pattern: String`, `Subscribe`
creating it, and `Publish(subscription_id, channel)` with a `delivered` branch guarded by `when_subject: {predicate: GUARD}`.
`ess specify validate --path spec.yaml`, 0.68.0 and 0.67.0 identical:

| GUARD | result |
|---|---|
| `{pattern: {starts_with: {input: channel}}}` | valid: the stored field may start with the input, the reverse of the need |
| `{channel: {starts_with: pattern}}` | `[type_mismatch] ... reads pattern as the text literal "pattern", not the field pattern: a string operator compares with a literal only` (plus `unobservable_fact` for `channel`) |
| `{input.channel: {starts_with: {stored: pattern}}}` | `[unparsable_predicate] ... a comparison operand must be a scalar, or for a string operator {param: <name>} naming one view parameter or {input: <name>} naming one command input` (ESS-SPEC-012) |
| `{pattern: {matches: {input: channel}}}` | `[unparsable_predicate] ... unknown operator "matches"; expected one of eq, ne, lt, lte, gt, gte, any_of, none_of, exists, truthy, starts_with, ends_with, contains, equals_ignore_case, in_ignore_case, trimmed` |

Even with a stored-field operand, `starts_with` does not express `*` (exactly one more segment).

## Where it lives

- `crates/specify/ess-primitives/src/predicate.rs:386` (`TextOperand`): a literal, or a fact in `TextNamespace` (:345), which is
  `Param` or `Input` only; no stored-field namespace and no pattern operator.
- The regular-expression work does not cover it: `docs/design/string-predicate-operators.md`, "Out of scope", declines regular
  expressions (#449), and `docs/design/stored-rules-boundary.md`, "Patterns held as data", says a pattern stored as data is the
  system's evaluator and its verdict an `external:` outcome. So the item is not closed by a pointer: the fit review decides
  between a segment-wildcard predicate over a stored operand and recording the `external:` idiom for this case.

## Regression status

Not a 0.68.0 regression: identical refusals on 0.67.0; no commit in `0.67.0..52bd110dd` touches `predicate.rs`. Gap. Does not
block 0.68.0.

## Decision (2026-10-22)

Declined as by design. A channel pattern a subscription stores and the system evaluates against a published channel is a
pattern held as data: the evaluator is the system's, and its verdict is an `external:` outcome on the branch that delivers.
`docs/design/stored-rules-boundary.md`, "Patterns held as data", states this case and says ESS has no pattern predicate in a
guard (beyond10x/ess#449, closed, which follows #95 and #103). A segment-wildcard predicate over a stored operand would need the
same portable dialect, per-runner translation and matching/near-miss synthesis that #449 declined.

The idiom for the adopter: model the delivery decision as an `external:` outcome of `Publish` (the system says which stored
subscriptions matched), and keep specification-checked text shapes to `alphabet:`, `prefix:`, `.count`, `starts_with`,
`ends_with`, `contains` and `any_of`. The request reopens only on the condition the design page names: a guard, not stored data,
needing a position-dependent character class the idiom cannot state.

Re-checked on 0.68.0 (integration build at 52bd110dd) and 0.67.0 on 2026-10-22: the refusals in Reproduction are unchanged.
